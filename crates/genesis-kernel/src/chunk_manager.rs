//! Multi-chunk management for visible area rendering.
//!
//! This module provides chunk loading/unloading based on camera position,
//! enabling seamless rendering of large worlds.

use std::collections::HashMap;

use tracing::{debug, info};

use crate::camera::Camera;
use crate::cell::Cell;
use crate::worldgen::WorldGenerator;

/// Default chunk size for chunk manager in cells.
pub const CHUNK_MANAGER_DEFAULT_SIZE: u32 = 256;

/// Default render distance in chunks.
pub const CHUNK_MANAGER_RENDER_DISTANCE: u32 = 3;

/// A single chunk of the world.
#[derive(Debug)]
pub struct VisibleChunk {
    /// Chunk coordinates (not world coordinates).
    pub position: (i32, i32),
    /// Cell data for this chunk.
    pub cells: Vec<Cell>,
    /// Whether the chunk needs to be uploaded to GPU.
    pub dirty: bool,
}

impl VisibleChunk {
    /// Creates a new empty chunk.
    #[must_use]
    pub fn new(position: (i32, i32), chunk_size: u32) -> Self {
        let cell_count = (chunk_size * chunk_size) as usize;
        Self {
            position,
            cells: vec![Cell::default(); cell_count],
            dirty: true,
        }
    }

    /// Creates a chunk with the given cells.
    #[must_use]
    pub fn with_cells(position: (i32, i32), cells: Vec<Cell>) -> Self {
        Self {
            position,
            cells,
            dirty: true,
        }
    }

    /// Gets a cell at local coordinates.
    #[must_use]
    pub fn get_cell(&self, local_x: u32, local_y: u32, chunk_size: u32) -> Option<&Cell> {
        if local_x >= chunk_size || local_y >= chunk_size {
            return None;
        }
        let idx = (local_y * chunk_size + local_x) as usize;
        self.cells.get(idx)
    }

    /// Gets a mutable cell at local coordinates.
    pub fn get_cell_mut(
        &mut self,
        local_x: u32,
        local_y: u32,
        chunk_size: u32,
    ) -> Option<&mut Cell> {
        if local_x >= chunk_size || local_y >= chunk_size {
            return None;
        }
        let idx = (local_y * chunk_size + local_x) as usize;
        self.cells.get_mut(idx)
    }

    /// Sets a cell at local coordinates.
    pub fn set_cell(&mut self, local_x: u32, local_y: u32, chunk_size: u32, cell: Cell) {
        if local_x >= chunk_size || local_y >= chunk_size {
            return;
        }
        let idx = (local_y * chunk_size + local_x) as usize;
        if let Some(c) = self.cells.get_mut(idx) {
            *c = cell;
            self.dirty = true;
        }
    }

    /// Marks the chunk as needing GPU upload.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Returns the world origin of this chunk.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn world_origin(&self, chunk_size: u32) -> (i32, i32) {
        (
            self.position.0 * chunk_size as i32,
            self.position.1 * chunk_size as i32,
        )
    }
}

/// Manages multiple chunks for visible area rendering.
pub struct VisibleChunkManager {
    /// Loaded chunks, keyed by chunk coordinates.
    chunks: HashMap<(i32, i32), VisibleChunk>,
    /// Size of each chunk in cells.
    chunk_size: u32,
    /// How many chunks to keep loaded around camera.
    render_distance: u32,
    /// Current center chunk (camera is here).
    center_chunk: (i32, i32),
}

impl VisibleChunkManager {
    /// Creates a new chunk manager.
    #[must_use]
    pub fn new(chunk_size: u32, render_distance: u32) -> Self {
        info!(
            "Creating chunk manager with chunk_size={}, render_distance={}",
            chunk_size, render_distance
        );

        Self {
            chunks: HashMap::new(),
            chunk_size,
            render_distance,
            center_chunk: (0, 0),
        }
    }

    /// Returns the chunk size.
    #[must_use]
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Returns the render distance.
    #[must_use]
    pub const fn render_distance(&self) -> u32 {
        self.render_distance
    }

    /// Sets the render distance.
    pub fn set_render_distance(&mut self, distance: u32) {
        self.render_distance = distance;
    }

    /// Returns the current center chunk.
    #[must_use]
    pub const fn center_chunk(&self) -> (i32, i32) {
        self.center_chunk
    }

    /// Converts world coordinates to chunk coordinates.
    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn world_to_chunk(x: i32, y: i32, chunk_size: u32) -> (i32, i32) {
        let size = chunk_size as i32;
        (
            if x >= 0 { x / size } else { (x + 1) / size - 1 },
            if y >= 0 { y / size } else { (y + 1) / size - 1 },
        )
    }

    /// Converts world coordinates to local chunk coordinates.
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    pub fn world_to_local(x: i32, y: i32, chunk_size: u32) -> (u32, u32) {
        let size = chunk_size as i32;
        (x.rem_euclid(size) as u32, y.rem_euclid(size) as u32)
    }

    /// Updates which chunks are loaded based on camera position.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn update_visible(&mut self, camera: &Camera, world_gen: &WorldGenerator) {
        // Determine center chunk from camera
        let camera_x = camera.position.0 as i32;
        let camera_y = camera.position.1 as i32;
        let new_center = Self::world_to_chunk(camera_x, camera_y, self.chunk_size);

        if new_center != self.center_chunk {
            debug!(
                "Camera moved to new center chunk: {:?} -> {:?}",
                self.center_chunk, new_center
            );
            self.center_chunk = new_center;
        }

        let distance = self.render_distance as i32;

        // Determine required chunks
        let mut required = Vec::new();
        for dy in -distance..=distance {
            for dx in -distance..=distance {
                required.push((self.center_chunk.0 + dx, self.center_chunk.1 + dy));
            }
        }

        // Unload chunks that are too far
        let to_unload: Vec<_> = self
            .chunks
            .keys()
            .filter(|pos| {
                let dx = (pos.0 - self.center_chunk.0).abs();
                let dy = (pos.1 - self.center_chunk.1).abs();
                dx > distance || dy > distance
            })
            .copied()
            .collect();

        for pos in to_unload {
            debug!("Unloading chunk {:?}", pos);
            self.chunks.remove(&pos);
        }

        // Load missing chunks
        for pos in required {
            if let std::collections::hash_map::Entry::Vacant(entry) = self.chunks.entry(pos) {
                debug!("Loading chunk {:?}", pos);
                let cells = world_gen.generate_chunk(
                    pos.0,
                    pos.1,
                    &crate::worldgen::GenerationParams::default(),
                );
                let chunk = VisibleChunk::with_cells(pos, cells);
                entry.insert(chunk);
            }
        }
    }

    /// Gets a cell at world coordinates.
    #[must_use]
    pub fn get_cell(&self, world_x: i32, world_y: i32) -> Option<&Cell> {
        let chunk_pos = Self::world_to_chunk(world_x, world_y, self.chunk_size);
        let (local_x, local_y) = Self::world_to_local(world_x, world_y, self.chunk_size);

        self.chunks
            .get(&chunk_pos)
            .and_then(|chunk| chunk.get_cell(local_x, local_y, self.chunk_size))
    }

    /// Gets a mutable cell at world coordinates.
    pub fn get_cell_mut(&mut self, world_x: i32, world_y: i32) -> Option<&mut Cell> {
        let chunk_pos = Self::world_to_chunk(world_x, world_y, self.chunk_size);
        let (local_x, local_y) = Self::world_to_local(world_x, world_y, self.chunk_size);

        self.chunks
            .get_mut(&chunk_pos)
            .and_then(|chunk| chunk.get_cell_mut(local_x, local_y, self.chunk_size))
    }

    /// Sets a cell at world coordinates.
    pub fn set_cell(&mut self, world_x: i32, world_y: i32, cell: Cell) {
        let chunk_pos = Self::world_to_chunk(world_x, world_y, self.chunk_size);
        let (local_x, local_y) = Self::world_to_local(world_x, world_y, self.chunk_size);

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_cell(local_x, local_y, self.chunk_size, cell);
        }
    }

    /// Gets all visible chunks for rendering.
    pub fn visible_chunks(&self) -> impl Iterator<Item = &VisibleChunk> {
        self.chunks.values()
    }

    /// Gets all visible chunks mutably.
    pub fn visible_chunks_mut(&mut self) -> impl Iterator<Item = &mut VisibleChunk> {
        self.chunks.values_mut()
    }

    /// Gets a specific chunk by chunk coordinates.
    #[must_use]
    pub fn get_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<&VisibleChunk> {
        self.chunks.get(&(chunk_x, chunk_y))
    }

    /// Gets a mutable chunk by chunk coordinates.
    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_y: i32) -> Option<&mut VisibleChunk> {
        self.chunks.get_mut(&(chunk_x, chunk_y))
    }

    /// Returns the number of loaded chunks.
    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    /// Returns the number of dirty chunks needing upload.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.chunks.values().filter(|c| c.dirty).count()
    }

    /// Marks all chunks as needing upload.
    pub fn mark_all_dirty(&mut self) {
        for chunk in self.chunks.values_mut() {
            chunk.dirty = true;
        }
    }

    /// Clears all chunks.
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Returns iterator over chunk positions that should be loaded.
    #[allow(clippy::cast_possible_wrap)]
    pub fn required_chunk_positions(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let distance = self.render_distance as i32;
        let center = self.center_chunk;

        (-distance..=distance).flat_map(move |dy| {
            (-distance..=distance).map(move |dx| (center.0 + dx, center.1 + dy))
        })
    }

    /// Checks if a chunk at the given position is loaded.
    #[must_use]
    pub fn is_chunk_loaded(&self, chunk_x: i32, chunk_y: i32) -> bool {
        self.chunks.contains_key(&(chunk_x, chunk_y))
    }
}

impl std::fmt::Debug for VisibleChunkManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkManager")
            .field("chunk_size", &self.chunk_size)
            .field("render_distance", &self.render_distance)
            .field("center_chunk", &self.center_chunk)
            .field("loaded_chunks", &self.chunks.len())
            .field("dirty_chunks", &self.dirty_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_chunk() {
        let chunk_size = 256u32;

        // Positive coordinates
        assert_eq!(
            VisibleChunkManager::world_to_chunk(0, 0, chunk_size),
            (0, 0)
        );
        assert_eq!(
            VisibleChunkManager::world_to_chunk(255, 255, chunk_size),
            (0, 0)
        );
        assert_eq!(
            VisibleChunkManager::world_to_chunk(256, 256, chunk_size),
            (1, 1)
        );
        assert_eq!(
            VisibleChunkManager::world_to_chunk(512, 512, chunk_size),
            (2, 2)
        );

        // Negative coordinates
        assert_eq!(
            VisibleChunkManager::world_to_chunk(-1, -1, chunk_size),
            (-1, -1)
        );
        assert_eq!(
            VisibleChunkManager::world_to_chunk(-256, -256, chunk_size),
            (-1, -1)
        );
        assert_eq!(
            VisibleChunkManager::world_to_chunk(-257, -257, chunk_size),
            (-2, -2)
        );
    }

    #[test]
    fn test_world_to_local() {
        let chunk_size = 256u32;

        assert_eq!(
            VisibleChunkManager::world_to_local(0, 0, chunk_size),
            (0, 0)
        );
        assert_eq!(
            VisibleChunkManager::world_to_local(100, 100, chunk_size),
            (100, 100)
        );
        assert_eq!(
            VisibleChunkManager::world_to_local(256, 256, chunk_size),
            (0, 0)
        );
        assert_eq!(
            VisibleChunkManager::world_to_local(300, 300, chunk_size),
            (44, 44)
        );

        // Negative coordinates
        assert_eq!(
            VisibleChunkManager::world_to_local(-1, -1, chunk_size),
            (255, 255)
        );
        assert_eq!(
            VisibleChunkManager::world_to_local(-256, -256, chunk_size),
            (0, 0)
        );
    }

    #[test]
    fn test_chunk_manager_creation() {
        let manager = VisibleChunkManager::new(256, 3);
        assert_eq!(manager.chunk_size(), 256);
        assert_eq!(manager.render_distance(), 3);
        assert_eq!(manager.center_chunk(), (0, 0));
        assert_eq!(manager.loaded_count(), 0);
    }

    #[test]
    fn test_chunk_creation() {
        let chunk = VisibleChunk::new((0, 0), 256);
        assert_eq!(chunk.position, (0, 0));
        assert_eq!(chunk.cells.len(), 256 * 256);
        assert!(chunk.dirty);
    }

    #[test]
    fn test_chunk_world_origin() {
        let chunk = VisibleChunk::new((1, 2), 256);
        assert_eq!(chunk.world_origin(256), (256, 512));

        let neg_chunk = VisibleChunk::new((-1, -1), 256);
        assert_eq!(neg_chunk.world_origin(256), (-256, -256));
    }

    #[test]
    fn test_chunk_get_set_cell() {
        let mut chunk = VisibleChunk::new((0, 0), 256);

        // Default cell
        let cell = chunk.get_cell(0, 0, 256);
        assert!(cell.is_some());
        assert_eq!(cell.unwrap().material, 0);

        // Set cell
        let new_cell = Cell::new(5);
        chunk.set_cell(10, 10, 256, new_cell);
        assert!(chunk.dirty);

        let cell = chunk.get_cell(10, 10, 256);
        assert_eq!(cell.unwrap().material, 5);

        // Out of bounds
        assert!(chunk.get_cell(256, 256, 256).is_none());
    }

    #[test]
    fn test_required_chunk_positions() {
        let manager = VisibleChunkManager::new(256, 1);
        let positions: Vec<_> = manager.required_chunk_positions().collect();

        // 3x3 grid around center
        assert_eq!(positions.len(), 9);
        assert!(positions.contains(&(-1, -1)));
        assert!(positions.contains(&(0, 0)));
        assert!(positions.contains(&(1, 1)));
    }

    #[test]
    fn test_chunk_manager_cell_access() {
        let mut manager = VisibleChunkManager::new(256, 1);

        // Manually insert a chunk
        let mut chunk = VisibleChunk::new((0, 0), 256);
        let test_cell = Cell::new(42);
        chunk.set_cell(50, 50, 256, test_cell);
        manager.chunks.insert((0, 0), chunk);

        // Read it back
        let cell = manager.get_cell(50, 50);
        assert!(cell.is_some());
        assert_eq!(cell.unwrap().material, 42);

        // Modify it
        manager.set_cell(50, 50, Cell::new(99));
        let cell = manager.get_cell(50, 50);
        assert_eq!(cell.unwrap().material, 99);

        // Access non-loaded chunk returns None
        assert!(manager.get_cell(1000, 1000).is_none());
    }

    #[test]
    fn test_visible_chunks_iterator() {
        let mut manager = VisibleChunkManager::new(256, 1);
        manager
            .chunks
            .insert((0, 0), VisibleChunk::new((0, 0), 256));
        manager
            .chunks
            .insert((1, 0), VisibleChunk::new((1, 0), 256));
        manager
            .chunks
            .insert((0, 1), VisibleChunk::new((0, 1), 256));

        let visible: Vec<_> = manager.visible_chunks().collect();
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn test_dirty_count() {
        let mut manager = VisibleChunkManager::new(256, 1);
        manager
            .chunks
            .insert((0, 0), VisibleChunk::new((0, 0), 256));
        manager
            .chunks
            .insert((1, 0), VisibleChunk::new((1, 0), 256));

        assert_eq!(manager.dirty_count(), 2);

        // Clear one
        if let Some(chunk) = manager.get_chunk_mut(0, 0) {
            chunk.clear_dirty();
        }
        assert_eq!(manager.dirty_count(), 1);

        // Mark all dirty
        manager.mark_all_dirty();
        assert_eq!(manager.dirty_count(), 2);
    }
}
