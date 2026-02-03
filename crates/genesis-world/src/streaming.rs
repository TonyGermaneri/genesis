//! World streaming and chunk management.

use dashmap::DashMap;
use genesis_common::ChunkCoord;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

use crate::chunk::{Chunk, ChunkResult};
use crate::generation::WorldGenerator;

/// Chunk manager configuration.
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// World save directory
    pub save_dir: PathBuf,
    /// Maximum loaded chunks
    pub max_loaded_chunks: usize,
    /// Chunk size
    pub chunk_size: u32,
    /// Auto-save interval (in ticks)
    pub auto_save_interval: u32,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            save_dir: PathBuf::from("saves/world"),
            max_loaded_chunks: 64,
            chunk_size: 256,
            auto_save_interval: 6000, // ~100 seconds at 60fps
        }
    }
}

/// Manages chunk loading, unloading, and persistence.
pub struct ChunkManager {
    /// Configuration
    config: StreamingConfig,
    /// Loaded chunks
    chunks: DashMap<ChunkCoord, Arc<RwLock<Chunk>>>,
    /// World generator
    generator: WorldGenerator,
    /// Ticks since last auto-save
    ticks_since_save: u32,
}

impl ChunkManager {
    /// Creates a new chunk manager.
    #[must_use]
    pub fn new(config: StreamingConfig, generator: WorldGenerator) -> Self {
        Self {
            config,
            chunks: DashMap::new(),
            generator,
            ticks_since_save: 0,
        }
    }

    /// Gets or loads a chunk at the given coordinate.
    pub fn get_chunk(&self, coord: ChunkCoord) -> Arc<RwLock<Chunk>> {
        // Check if already loaded
        if let Some(chunk) = self.chunks.get(&coord) {
            return Arc::clone(chunk.value());
        }

        // Try to load from disk
        let chunk = match self.load_from_disk(coord) {
            Ok(c) => c,
            Err(_) => {
                // Generate new chunk
                self.generator.generate_chunk(coord)
            },
        };

        let chunk_arc = Arc::new(RwLock::new(chunk));
        self.chunks.insert(coord, Arc::clone(&chunk_arc));

        // Check if we need to unload chunks
        self.maybe_unload_chunks();

        chunk_arc
    }

    /// Checks if a chunk is loaded.
    #[must_use]
    pub fn is_loaded(&self, coord: ChunkCoord) -> bool {
        self.chunks.contains_key(&coord)
    }

    /// Returns the number of loaded chunks.
    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    /// Saves all dirty chunks to disk.
    pub fn save_all(&self) -> ChunkResult<usize> {
        let mut saved = 0;
        for entry in &self.chunks {
            let chunk = entry.value().read();
            if chunk.is_dirty() {
                if let Err(e) = self.save_to_disk(&chunk) {
                    warn!("Failed to save chunk {:?}: {e}", chunk.coord());
                } else {
                    saved += 1;
                }
            }
        }
        info!("Saved {saved} chunks");
        Ok(saved)
    }

    /// Updates the manager (handles auto-save).
    pub fn tick(&mut self) {
        self.ticks_since_save += 1;
        if self.ticks_since_save >= self.config.auto_save_interval {
            self.ticks_since_save = 0;
            let _ = self.save_all();
        }
    }

    /// Loads a chunk from disk.
    fn load_from_disk(&self, coord: ChunkCoord) -> ChunkResult<Chunk> {
        let path = self.chunk_path(coord);
        let bytes = std::fs::read(&path).map_err(|e| {
            crate::chunk::ChunkError::DeserializationFailed(format!("read failed: {e}"))
        })?;
        Chunk::deserialize(&bytes)
    }

    /// Saves a chunk to disk.
    fn save_to_disk(&self, chunk: &Chunk) -> ChunkResult<()> {
        let path = self.chunk_path(chunk.coord());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::chunk::ChunkError::SerializationFailed(format!("mkdir failed: {e}"))
            })?;
        }
        let bytes = chunk.serialize()?;
        std::fs::write(&path, bytes).map_err(|e| {
            crate::chunk::ChunkError::SerializationFailed(format!("write failed: {e}"))
        })?;
        Ok(())
    }

    /// Returns the file path for a chunk.
    fn chunk_path(&self, coord: ChunkCoord) -> PathBuf {
        self.config
            .save_dir
            .join(format!("chunk_{}_{}.gnch", coord.x, coord.y))
    }

    /// Unloads chunks if over the limit.
    fn maybe_unload_chunks(&self) {
        if self.chunks.len() <= self.config.max_loaded_chunks {
            return;
        }

        // Simple LRU: remove oldest chunks
        // In a real implementation, we'd track access times
        let to_remove = self.chunks.len() - self.config.max_loaded_chunks;
        let coords: Vec<_> = self
            .chunks
            .iter()
            .take(to_remove)
            .map(|e| *e.key())
            .collect();

        for coord in coords {
            if let Some((_, chunk)) = self.chunks.remove(&coord) {
                let c = chunk.read();
                if c.is_dirty() {
                    let _ = self.save_to_disk(&c);
                }
            }
        }
    }
}
