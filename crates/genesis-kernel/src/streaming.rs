//! Chunk streaming system for dynamic loading/unloading.
//!
//! This module provides infrastructure for streaming chunks based on
//! camera/player position, enabling infinite worlds with bounded memory.
//!
//! ## Overview
//!
//! The streaming system:
//! - Loads chunks in a spiral pattern from the center
//! - Prioritizes chunks in the view frustum
//! - Budgets chunk loads per frame to prevent hitching
//! - Unloads distant chunks to free memory

use std::collections::{HashSet, VecDeque};

use genesis_common::WorldCoord;

use crate::chunk::{ChunkId, ChunkManager};
use crate::CellComputePipeline;

/// Default load radius in chunks.
pub const DEFAULT_LOAD_RADIUS: u32 = 3;

/// Default unload radius in chunks (should be > load_radius).
pub const DEFAULT_UNLOAD_RADIUS: u32 = 5;

/// Maximum chunk loads per frame to prevent hitching.
pub const MAX_LOADS_PER_FRAME: usize = 2;

/// Streaming state for a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingState {
    /// Chunk is not loaded
    Unloaded,
    /// Chunk is queued for loading
    PendingLoad,
    /// Chunk is fully loaded
    Loaded,
    /// Chunk is queued for unloading
    PendingUnload,
}

/// Manages chunk streaming based on position.
pub struct ChunkStreamer {
    /// Radius of chunks to keep loaded (in chunk units)
    load_radius: u32,
    /// Radius beyond which chunks are unloaded
    unload_radius: u32,
    /// Queue of chunks pending load (spiral order)
    pending_loads: VecDeque<ChunkId>,
    /// Queue of chunks pending unload
    pending_unloads: VecDeque<ChunkId>,
    /// Currently loaded chunk IDs
    loaded_chunks: HashSet<ChunkId>,
    /// Last center position (for detecting movement)
    last_center: Option<ChunkId>,
    /// Chunk size for coordinate conversion
    chunk_size: u32,
}

impl ChunkStreamer {
    /// Creates a new chunk streamer.
    #[must_use]
    pub fn new(load_radius: u32, unload_radius: u32, chunk_size: u32) -> Self {
        assert!(
            unload_radius > load_radius,
            "Unload radius must be greater than load radius"
        );
        Self {
            load_radius,
            unload_radius,
            pending_loads: VecDeque::new(),
            pending_unloads: VecDeque::new(),
            loaded_chunks: HashSet::new(),
            last_center: None,
            chunk_size,
        }
    }

    /// Creates a chunk streamer with default radii.
    #[must_use]
    pub fn with_defaults(chunk_size: u32) -> Self {
        Self::new(DEFAULT_LOAD_RADIUS, DEFAULT_UNLOAD_RADIUS, chunk_size)
    }

    /// Updates streaming based on a world position.
    ///
    /// This method:
    /// 1. Converts world position to chunk coordinates
    /// 2. Determines which chunks should be loaded
    /// 3. Queues loads/unloads as needed
    pub fn update(&mut self, center: WorldCoord, manager: &mut ChunkManager) {
        let center_chunk = self.world_to_chunk(center);

        // Check if we've moved to a new chunk
        let center_changed = self.last_center != Some(center_chunk);
        if center_changed {
            self.last_center = Some(center_chunk);
            self.recalculate_streaming(center_chunk);
        }

        // Update manager's camera position
        #[allow(clippy::cast_possible_truncation)]
        manager.update_camera(center.x as i32, center.y as i32);
    }

    /// Recalculates which chunks to load/unload based on new center.
    fn recalculate_streaming(&mut self, center: ChunkId) {
        // Clear pending queues (we'll rebuild them)
        self.pending_loads.clear();
        self.pending_unloads.clear();

        // Generate spiral of chunks to load
        let chunks_to_load = Self::spiral_chunks(center, self.load_radius);

        // Queue loads for chunks not already loaded
        for chunk_id in chunks_to_load {
            if !self.loaded_chunks.contains(&chunk_id) {
                self.pending_loads.push_back(chunk_id);
            }
        }

        // Find chunks to unload (outside unload radius)
        #[allow(clippy::cast_possible_wrap)]
        let unload_threshold = self.unload_radius as i32;
        let to_unload: Vec<ChunkId> = self
            .loaded_chunks
            .iter()
            .filter(|id| {
                let dx = (id.x - center.x).abs();
                let dy = (id.y - center.y).abs();
                dx > unload_threshold || dy > unload_threshold
            })
            .copied()
            .collect();

        for chunk_id in to_unload {
            self.pending_unloads.push_back(chunk_id);
        }
    }

    /// Generates chunks in spiral order from center outward.
    fn spiral_chunks(center: ChunkId, radius: u32) -> Vec<ChunkId> {
        let mut result = Vec::new();

        // Start with center
        result.push(center);

        // Spiral outward ring by ring
        #[allow(clippy::cast_possible_wrap)]
        for ring in 1..=radius as i32 {
            // Top edge (left to right, excluding right corner)
            for x in -ring..ring {
                result.push(ChunkId::new(center.x + x, center.y + ring));
            }
            // Right edge (top to bottom, excluding bottom corner)
            for y in (-ring..ring).rev() {
                result.push(ChunkId::new(center.x + ring, center.y + y));
            }
            // Bottom edge (right to left, excluding left corner)
            for x in (-ring..ring).rev() {
                result.push(ChunkId::new(center.x + x, center.y - ring));
            }
            // Left edge (bottom to top, excluding top corner)
            for y in -ring..ring {
                result.push(ChunkId::new(center.x - ring, center.y + y));
            }
        }

        result
    }

    /// Processes pending loads/unloads for this frame.
    ///
    /// Returns the number of chunks loaded this frame.
    pub fn process_frame(
        &mut self,
        device: &wgpu::Device,
        pipeline: &CellComputePipeline,
        manager: &mut ChunkManager,
    ) -> usize {
        let mut loads_this_frame = 0;

        // Process unloads first (frees memory)
        while let Some(chunk_id) = self.pending_unloads.pop_front() {
            if self.loaded_chunks.remove(&chunk_id) {
                manager.unload_chunk(&chunk_id);
            }
        }

        // Process loads (budgeted)
        while loads_this_frame < MAX_LOADS_PER_FRAME {
            if let Some(chunk_id) = self.pending_loads.pop_front() {
                if !self.loaded_chunks.contains(&chunk_id) {
                    manager.load_chunk(device, pipeline, chunk_id);
                    self.loaded_chunks.insert(chunk_id);
                    loads_this_frame += 1;
                }
            } else {
                break;
            }
        }

        loads_this_frame
    }

    /// Returns chunks pending load.
    #[must_use]
    pub fn get_pending_loads(&self) -> &VecDeque<ChunkId> {
        &self.pending_loads
    }

    /// Returns chunks pending unload.
    #[must_use]
    pub fn get_pending_unloads(&self) -> &VecDeque<ChunkId> {
        &self.pending_unloads
    }

    /// Returns the number of loaded chunks.
    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// Returns the streaming state of a chunk.
    #[must_use]
    pub fn chunk_state(&self, chunk_id: &ChunkId) -> StreamingState {
        if self.pending_loads.contains(chunk_id) {
            StreamingState::PendingLoad
        } else if self.pending_unloads.contains(chunk_id) {
            StreamingState::PendingUnload
        } else if self.loaded_chunks.contains(chunk_id) {
            StreamingState::Loaded
        } else {
            StreamingState::Unloaded
        }
    }

    /// Returns the load radius.
    #[must_use]
    pub const fn load_radius(&self) -> u32 {
        self.load_radius
    }

    /// Returns the unload radius.
    #[must_use]
    pub const fn unload_radius(&self) -> u32 {
        self.unload_radius
    }

    /// Converts world coordinate to chunk ID.
    fn world_to_chunk(&self, coord: WorldCoord) -> ChunkId {
        #[allow(clippy::cast_possible_truncation)]
        let chunk_size_i = self.chunk_size as i64;
        ChunkId::new(
            coord.x.div_euclid(chunk_size_i) as i32,
            coord.y.div_euclid(chunk_size_i) as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streamer_creation() {
        let streamer = ChunkStreamer::new(3, 5, 256);
        assert_eq!(streamer.load_radius(), 3);
        assert_eq!(streamer.unload_radius(), 5);
        assert_eq!(streamer.loaded_count(), 0);
    }

    #[test]
    fn test_spiral_generation() {
        let center = ChunkId::new(0, 0);
        let spiral = ChunkStreamer::spiral_chunks(center, 1);

        // Radius 1 spiral should have center + 8 surrounding = 9 chunks
        assert_eq!(spiral.len(), 9);
        assert_eq!(spiral[0], center);
    }

    #[test]
    fn test_spiral_radius_2() {
        let center = ChunkId::new(0, 0);
        let spiral = ChunkStreamer::spiral_chunks(center, 2);

        // Radius 2: center (1) + ring 1 (8) + ring 2 (16) = 25 chunks
        assert_eq!(spiral.len(), 25);
        assert_eq!(spiral[0], center);
    }

    #[test]
    fn test_world_to_chunk() {
        let streamer = ChunkStreamer::new(2, 4, 256);

        let coord = WorldCoord::new(100, 200);
        let chunk = streamer.world_to_chunk(coord);
        assert_eq!(chunk.x, 0);
        assert_eq!(chunk.y, 0);

        let coord = WorldCoord::new(256, 512);
        let chunk = streamer.world_to_chunk(coord);
        assert_eq!(chunk.x, 1);
        assert_eq!(chunk.y, 2);

        // Negative coordinates
        let coord = WorldCoord::new(-1, -1);
        let chunk = streamer.world_to_chunk(coord);
        assert_eq!(chunk.x, -1);
        assert_eq!(chunk.y, -1);
    }

    #[test]
    fn test_chunk_state() {
        let mut streamer = ChunkStreamer::new(2, 4, 256);
        let chunk_id = ChunkId::new(5, 5);

        assert_eq!(streamer.chunk_state(&chunk_id), StreamingState::Unloaded);

        streamer.pending_loads.push_back(chunk_id);
        assert_eq!(streamer.chunk_state(&chunk_id), StreamingState::PendingLoad);

        streamer.pending_loads.clear();
        streamer.loaded_chunks.insert(chunk_id);
        assert_eq!(streamer.chunk_state(&chunk_id), StreamingState::Loaded);

        streamer.pending_unloads.push_back(chunk_id);
        assert_eq!(
            streamer.chunk_state(&chunk_id),
            StreamingState::PendingUnload
        );
    }

    #[test]
    #[should_panic(expected = "Unload radius must be greater than load radius")]
    fn test_invalid_radii() {
        let _ = ChunkStreamer::new(5, 3, 256);
    }
}
