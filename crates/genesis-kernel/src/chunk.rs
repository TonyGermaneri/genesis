//! Chunk management for multi-chunk simulation.
//!
//! This module provides infrastructure for simulating multiple chunks:
//! - `ChunkId`: Unique identifier for chunks based on grid coordinates
//! - `ChunkManager`: Manages active chunks around the camera
//! - `ChunkState`: Tracks simulation state of each chunk

use std::collections::{HashMap, HashSet};

use tracing::info;
use wgpu::Device;

use crate::{CellBuffer, CellComputePipeline};

/// Maximum number of active chunks to simulate at once.
pub const MAX_ACTIVE_CHUNKS: usize = 9; // 3x3 grid around camera

/// Unique identifier for a chunk based on its grid coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId {
    /// X coordinate in chunk grid space
    pub x: i32,
    /// Y coordinate in chunk grid space
    pub y: i32,
}

impl ChunkId {
    /// Creates a new chunk ID.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the chunk ID containing the given world position.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn from_world_pos(world_x: i32, world_y: i32, chunk_size: u32) -> Self {
        let chunk_size_i = chunk_size as i32;
        Self {
            x: world_x.div_euclid(chunk_size_i),
            y: world_y.div_euclid(chunk_size_i),
        }
    }

    /// Returns the world position of this chunk's origin (bottom-left corner).
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn world_origin(&self, chunk_size: u32) -> (i32, i32) {
        let chunk_size_i = chunk_size as i32;
        (self.x * chunk_size_i, self.y * chunk_size_i)
    }

    /// Returns neighboring chunk IDs (including diagonals).
    #[must_use]
    pub fn neighbors(&self) -> [ChunkId; 8] {
        [
            ChunkId::new(self.x - 1, self.y - 1), // bottom-left
            ChunkId::new(self.x, self.y - 1),     // bottom
            ChunkId::new(self.x + 1, self.y - 1), // bottom-right
            ChunkId::new(self.x - 1, self.y),     // left
            ChunkId::new(self.x + 1, self.y),     // right
            ChunkId::new(self.x - 1, self.y + 1), // top-left
            ChunkId::new(self.x, self.y + 1),     // top
            ChunkId::new(self.x + 1, self.y + 1), // top-right
        ]
    }

    /// Returns cardinal neighbor chunk IDs (no diagonals).
    #[must_use]
    pub fn cardinal_neighbors(&self) -> [ChunkId; 4] {
        [
            ChunkId::new(self.x, self.y - 1), // bottom
            ChunkId::new(self.x - 1, self.y), // left
            ChunkId::new(self.x + 1, self.y), // right
            ChunkId::new(self.x, self.y + 1), // top
        ]
    }
}

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk({}, {})", self.x, self.y)
    }
}

/// State of a loaded chunk.
pub struct ChunkState {
    /// Unique identifier
    pub id: ChunkId,
    /// Cell buffer for this chunk (double-buffered)
    pub buffer: CellBuffer,
    /// Whether this chunk needs simulation this frame
    pub needs_update: bool,
    /// Frame count since last activity (for unloading decisions)
    pub idle_frames: u32,
}

impl ChunkState {
    /// Creates a new chunk state.
    pub fn new(
        id: ChunkId,
        device: &Device,
        pipeline: &CellComputePipeline,
        chunk_size: u32,
    ) -> Self {
        info!("Loading chunk {}", id);
        Self {
            id,
            buffer: CellBuffer::new(device, pipeline, chunk_size),
            needs_update: true,
            idle_frames: 0,
        }
    }

    /// Marks this chunk as needing simulation.
    pub fn mark_active(&mut self) {
        self.needs_update = true;
        self.idle_frames = 0;
    }

    /// Marks this chunk as idle for this frame.
    pub fn mark_idle(&mut self) {
        self.needs_update = false;
        self.idle_frames = self.idle_frames.saturating_add(1);
    }
}

/// Direction for neighbor relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Negative Y
    Bottom,
    /// Positive Y
    Top,
    /// Negative X
    Left,
    /// Positive X
    Right,
}

impl Direction {
    /// Returns the opposite direction.
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Direction::Bottom => Direction::Top,
            Direction::Top => Direction::Bottom,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// Returns all cardinal directions.
    #[must_use]
    pub const fn all() -> [Direction; 4] {
        [
            Direction::Bottom,
            Direction::Top,
            Direction::Left,
            Direction::Right,
        ]
    }
}

/// Manages multiple chunks for simulation.
pub struct ChunkManager {
    /// Loaded chunks by ID
    chunks: HashMap<ChunkId, ChunkState>,
    /// Set of chunk IDs currently active for simulation
    active_chunks: HashSet<ChunkId>,
    /// Chunk size in cells
    chunk_size: u32,
    /// Maximum chunks to keep loaded
    max_loaded_chunks: usize,
    /// Camera position in world coordinates
    camera_x: i32,
    camera_y: i32,
}

impl ChunkManager {
    /// Creates a new chunk manager.
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            active_chunks: HashSet::new(),
            chunk_size,
            max_loaded_chunks: MAX_ACTIVE_CHUNKS * 2, // Buffer for unloading
            camera_x: 0,
            camera_y: 0,
        }
    }

    /// Updates the camera position and determines active chunks.
    pub fn update_camera(&mut self, camera_x: i32, camera_y: i32) {
        self.camera_x = camera_x;
        self.camera_y = camera_y;
    }

    /// Computes which chunks should be active based on camera position.
    ///
    /// Returns a 3x3 grid of chunk IDs centered on the camera chunk.
    #[must_use]
    pub fn compute_active_chunks(&self) -> Vec<ChunkId> {
        let center = ChunkId::from_world_pos(self.camera_x, self.camera_y, self.chunk_size);

        let mut active = Vec::with_capacity(9);
        for dy in -1..=1 {
            for dx in -1..=1 {
                active.push(ChunkId::new(center.x + dx, center.y + dy));
            }
        }
        active
    }

    /// Ensures chunks are loaded and marks them for simulation.
    pub fn prepare_simulation(&mut self, device: &Device, pipeline: &CellComputePipeline) {
        use std::collections::hash_map::Entry;

        let active_ids = self.compute_active_chunks();

        // Mark current active set
        self.active_chunks.clear();
        for id in &active_ids {
            self.active_chunks.insert(*id);
        }

        // Load any missing chunks, mark existing as active
        for id in active_ids {
            match self.chunks.entry(id) {
                Entry::Vacant(entry) => {
                    let state = ChunkState::new(id, device, pipeline, self.chunk_size);
                    entry.insert(state);
                },
                Entry::Occupied(mut entry) => {
                    entry.get_mut().mark_active();
                },
            }
        }

        // Mark inactive chunks as idle
        for (id, chunk) in &mut self.chunks {
            if !self.active_chunks.contains(id) {
                chunk.mark_idle();
            }
        }

        // Unload old chunks if over limit
        self.unload_old_chunks();
    }

    /// Unloads chunks that have been idle for too long.
    fn unload_old_chunks(&mut self) {
        if self.chunks.len() <= self.max_loaded_chunks {
            return;
        }

        // Find chunks to unload (most idle, not currently active)
        let mut candidates: Vec<_> = self
            .chunks
            .iter()
            .filter(|(id, _)| !self.active_chunks.contains(id))
            .map(|(id, state)| (*id, state.idle_frames))
            .collect();

        // Sort by idle frames (most idle first)
        candidates.sort_by_key(|(_, frames)| std::cmp::Reverse(*frames));

        // Unload excess chunks
        let to_unload = self.chunks.len() - self.max_loaded_chunks;
        for (id, _) in candidates.into_iter().take(to_unload) {
            info!("Unloading chunk {} (idle too long)", id);
            self.chunks.remove(&id);
        }
    }

    /// Runs simulation for all active chunks.
    pub fn step_simulation(
        &mut self,
        device: &Device,
        queue: &wgpu::Queue,
        pipeline: &CellComputePipeline,
    ) {
        for id in &self.active_chunks.clone() {
            if let Some(chunk) = self.chunks.get_mut(id) {
                if chunk.needs_update {
                    chunk.buffer.step(device, queue, pipeline);
                }
            }
        }
    }

    /// Returns an iterator over active chunks.
    pub fn active_chunks(&self) -> impl Iterator<Item = &ChunkState> {
        self.active_chunks
            .iter()
            .filter_map(|id| self.chunks.get(id))
    }

    /// Returns a mutable iterator over active chunks.
    pub fn active_chunks_mut(&mut self) -> impl Iterator<Item = &mut ChunkState> {
        let active = self.active_chunks.clone();
        self.chunks
            .iter_mut()
            .filter(move |(id, _)| active.contains(id))
            .map(|(_, state)| state)
    }

    /// Returns the chunk containing the given world position, if loaded.
    #[must_use]
    pub fn chunk_at_world_pos(&self, world_x: i32, world_y: i32) -> Option<&ChunkState> {
        let id = ChunkId::from_world_pos(world_x, world_y, self.chunk_size);
        self.chunks.get(&id)
    }

    /// Returns the chunk with the given ID, if loaded.
    #[must_use]
    pub fn chunk(&self, id: &ChunkId) -> Option<&ChunkState> {
        self.chunks.get(id)
    }

    /// Returns the chunk with the given ID, if loaded.
    pub fn chunk_mut(&mut self, id: &ChunkId) -> Option<&mut ChunkState> {
        self.chunks.get_mut(id)
    }

    /// Returns the number of loaded chunks.
    #[must_use]
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Returns the number of active chunks.
    #[must_use]
    pub fn active_chunk_count(&self) -> usize {
        self.active_chunks.len()
    }

    /// Returns the chunk size.
    #[must_use]
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Returns the camera position.
    #[must_use]
    pub const fn camera_position(&self) -> (i32, i32) {
        (self.camera_x, self.camera_y)
    }

    /// Loads a specific chunk.
    pub fn load_chunk(
        &mut self,
        device: &wgpu::Device,
        pipeline: &CellComputePipeline,
        chunk_id: ChunkId,
    ) {
        use std::collections::hash_map::Entry;

        if let Entry::Vacant(entry) = self.chunks.entry(chunk_id) {
            let state = ChunkState::new(chunk_id, device, pipeline, self.chunk_size);
            entry.insert(state);
        }
    }

    /// Unloads a specific chunk.
    pub fn unload_chunk(&mut self, chunk_id: &ChunkId) {
        self.chunks.remove(chunk_id);
        self.active_chunks.remove(chunk_id);
    }

    /// Returns whether a chunk is loaded.
    #[must_use]
    pub fn is_chunk_loaded(&self, chunk_id: &ChunkId) -> bool {
        self.chunks.contains_key(chunk_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_id_from_world_pos() {
        // Test positive coordinates
        let id = ChunkId::from_world_pos(100, 200, 256);
        assert_eq!(id.x, 0);
        assert_eq!(id.y, 0);

        // Test coordinates at chunk boundary
        let id = ChunkId::from_world_pos(256, 512, 256);
        assert_eq!(id.x, 1);
        assert_eq!(id.y, 2);

        // Test negative coordinates (should use floor division)
        let id = ChunkId::from_world_pos(-1, -1, 256);
        assert_eq!(id.x, -1);
        assert_eq!(id.y, -1);

        let id = ChunkId::from_world_pos(-257, -1, 256);
        assert_eq!(id.x, -2);
        assert_eq!(id.y, -1);
    }

    #[test]
    fn test_chunk_id_world_origin() {
        let id = ChunkId::new(0, 0);
        assert_eq!(id.world_origin(256), (0, 0));

        let id = ChunkId::new(1, 2);
        assert_eq!(id.world_origin(256), (256, 512));

        let id = ChunkId::new(-1, -1);
        assert_eq!(id.world_origin(256), (-256, -256));
    }

    #[test]
    fn test_chunk_id_neighbors() {
        let id = ChunkId::new(0, 0);
        let neighbors = id.neighbors();

        assert_eq!(neighbors.len(), 8);
        assert!(neighbors.contains(&ChunkId::new(-1, -1)));
        assert!(neighbors.contains(&ChunkId::new(0, -1)));
        assert!(neighbors.contains(&ChunkId::new(1, -1)));
        assert!(neighbors.contains(&ChunkId::new(-1, 0)));
        assert!(neighbors.contains(&ChunkId::new(1, 0)));
        assert!(neighbors.contains(&ChunkId::new(-1, 1)));
        assert!(neighbors.contains(&ChunkId::new(0, 1)));
        assert!(neighbors.contains(&ChunkId::new(1, 1)));
    }

    #[test]
    fn test_chunk_manager_compute_active() {
        let manager = ChunkManager::new(256);
        let active = manager.compute_active_chunks();

        assert_eq!(active.len(), 9);
        assert!(active.contains(&ChunkId::new(0, 0)));
        assert!(active.contains(&ChunkId::new(-1, -1)));
        assert!(active.contains(&ChunkId::new(1, 1)));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Bottom.opposite(), Direction::Top);
        assert_eq!(Direction::Top.opposite(), Direction::Bottom);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }
}
