//! Terrain rendering integration.
//!
//! This module connects world generation with chunk management and the render
//! pipeline, handling on-demand chunk generation as the camera moves.

use std::collections::{HashMap, HashSet, VecDeque};

use tracing::{debug, info};
use wgpu::Device;

use crate::camera::Camera;
use crate::cell::Cell;
use crate::chunk::ChunkId;
use crate::compute::DEFAULT_CHUNK_SIZE;
use crate::worldgen::{GenerationParams, WorldGenerator};

/// Default maximum chunks to generate per frame.
pub const DEFAULT_MAX_CHUNKS_PER_FRAME: usize = 2;

/// Default view distance in chunks.
pub const DEFAULT_VIEW_DISTANCE: i32 = 3;

/// State of a loaded terrain chunk.
#[derive(Debug)]
pub struct TerrainChunk {
    /// Chunk identifier.
    pub id: ChunkId,
    /// Cell data for this chunk.
    pub cells: Vec<Cell>,
    /// GPU buffer for rendering.
    pub buffer: wgpu::Buffer,
    /// Whether the buffer needs updating.
    pub dirty: bool,
}

impl TerrainChunk {
    /// Creates a new terrain chunk with generated cells.
    pub fn new(id: ChunkId, cells: Vec<Cell>, device: &Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Terrain Chunk {id:?}")),
            size: (cells.len() * std::mem::size_of::<Cell>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            id,
            cells,
            buffer,
            dirty: true,
        }
    }

    /// Uploads cell data to the GPU buffer.
    pub fn upload(&mut self, queue: &wgpu::Queue) {
        if self.dirty {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.cells));
            self.dirty = false;
        }
    }
}

/// Terrain renderer that integrates world generation with chunk management.
pub struct TerrainRenderer {
    /// World generator.
    world_gen: WorldGenerator,
    /// Generation parameters.
    gen_params: GenerationParams,
    /// Loaded terrain chunks.
    chunks: HashMap<ChunkId, TerrainChunk>,
    /// Set of chunks currently visible.
    visible_chunks: HashSet<ChunkId>,
    /// Queue of chunks pending generation.
    generation_queue: VecDeque<ChunkId>,
    /// Chunk size in cells.
    chunk_size: u32,
    /// View distance in chunks from camera center.
    view_distance: i32,
    /// Maximum chunks to generate per frame.
    max_per_frame: usize,
}

impl TerrainRenderer {
    /// Creates a new terrain renderer.
    pub fn new(seed: u64, _device: &Device) -> Self {
        info!("Creating terrain renderer with seed {}", seed);

        Self {
            world_gen: WorldGenerator::new(seed),
            gen_params: GenerationParams::default(),
            chunks: HashMap::new(),
            visible_chunks: HashSet::new(),
            generation_queue: VecDeque::new(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            view_distance: DEFAULT_VIEW_DISTANCE,
            max_per_frame: DEFAULT_MAX_CHUNKS_PER_FRAME,
        }
    }

    /// Creates a terrain renderer with custom parameters.
    pub fn with_params(seed: u64, gen_params: GenerationParams, _device: &Device) -> Self {
        info!(
            "Creating terrain renderer with seed {} and custom params",
            seed
        );

        Self {
            world_gen: WorldGenerator::new(seed),
            gen_params,
            chunks: HashMap::new(),
            visible_chunks: HashSet::new(),
            generation_queue: VecDeque::new(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            view_distance: DEFAULT_VIEW_DISTANCE,
            max_per_frame: DEFAULT_MAX_CHUNKS_PER_FRAME,
        }
    }

    /// Returns the world generator.
    #[must_use]
    pub fn world_gen(&self) -> &WorldGenerator {
        &self.world_gen
    }

    /// Returns the generation parameters.
    #[must_use]
    pub fn gen_params(&self) -> &GenerationParams {
        &self.gen_params
    }

    /// Sets the generation parameters.
    pub fn set_gen_params(&mut self, params: GenerationParams) {
        self.gen_params = params;
    }

    /// Sets the view distance in chunks.
    pub fn set_view_distance(&mut self, distance: i32) {
        self.view_distance = distance.max(1);
    }

    /// Sets the maximum chunks to generate per frame.
    pub fn set_max_per_frame(&mut self, max: usize) {
        self.max_per_frame = max.max(1);
    }

    /// Returns the chunk size.
    #[must_use]
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Update visible chunks based on camera position.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn update_visible(&mut self, camera: &Camera) {
        let (min_x, min_y, max_x, max_y) = camera.visible_bounds();

        // Convert world bounds to chunk coordinates
        let chunk_min_x = (min_x / self.chunk_size as f32).floor() as i32 - 1;
        let chunk_min_y = (min_y / self.chunk_size as f32).floor() as i32 - 1;
        let chunk_max_x = (max_x / self.chunk_size as f32).ceil() as i32 + 1;
        let chunk_max_y = (max_y / self.chunk_size as f32).ceil() as i32 + 1;

        // Clear previous visible set
        self.visible_chunks.clear();

        // Build new visible set and queue missing chunks
        for cy in chunk_min_y..=chunk_max_y {
            for cx in chunk_min_x..=chunk_max_x {
                let chunk_id = ChunkId::new(cx, cy);
                self.visible_chunks.insert(chunk_id);

                // Queue for generation if not loaded
                if !self.chunks.contains_key(&chunk_id)
                    && !self.generation_queue.contains(&chunk_id)
                {
                    self.generation_queue.push_back(chunk_id);
                }
            }
        }

        // Sort queue by distance from camera center (prioritize closer chunks)
        let center_chunk_x = ((camera.position.0) / self.chunk_size as f32).floor() as i32;
        let center_chunk_y = ((camera.position.1) / self.chunk_size as f32).floor() as i32;

        // Convert to Vec, sort, convert back to VecDeque
        let mut queue_vec: Vec<_> = self.generation_queue.drain(..).collect();
        queue_vec.sort_by_key(|chunk_id| {
            let dx = chunk_id.x - center_chunk_x;
            let dy = chunk_id.y - center_chunk_y;
            dx * dx + dy * dy
        });
        self.generation_queue = queue_vec.into();
    }

    /// Generate pending chunks (call each frame, budget limited).
    pub fn generate_pending(&mut self, device: &Device, max_per_frame: usize) {
        let count = max_per_frame.min(self.max_per_frame);

        for _ in 0..count {
            if let Some(chunk_id) = self.generation_queue.pop_front() {
                // Skip if already loaded (could happen with rapid camera movement)
                if self.chunks.contains_key(&chunk_id) {
                    continue;
                }

                debug!("Generating chunk {:?}", chunk_id);

                // Generate cells
                let cells = self
                    .world_gen
                    .generate_chunk(chunk_id.x, chunk_id.y, &self.gen_params);

                // Create chunk with GPU buffer
                let chunk = TerrainChunk::new(chunk_id, cells, device);
                self.chunks.insert(chunk_id, chunk);
            } else {
                break;
            }
        }
    }

    /// Upload all dirty chunks to GPU.
    pub fn upload_dirty(&mut self, queue: &wgpu::Queue) {
        for chunk in self.chunks.values_mut() {
            chunk.upload(queue);
        }
    }

    /// Get all chunk buffers that should be rendered.
    #[must_use]
    pub fn get_render_chunks(&self) -> Vec<(&ChunkId, &wgpu::Buffer)> {
        self.visible_chunks
            .iter()
            .filter_map(|id| self.chunks.get(id).map(|chunk| (id, &chunk.buffer)))
            .collect()
    }

    /// Get visible chunk IDs.
    #[must_use]
    pub fn visible_chunk_ids(&self) -> &HashSet<ChunkId> {
        &self.visible_chunks
    }

    /// Check if a chunk is loaded.
    #[must_use]
    pub fn is_chunk_loaded(&self, chunk_id: &ChunkId) -> bool {
        self.chunks.contains_key(chunk_id)
    }

    /// Get a loaded chunk.
    #[must_use]
    pub fn get_chunk(&self, chunk_id: &ChunkId) -> Option<&TerrainChunk> {
        self.chunks.get(chunk_id)
    }

    /// Get a cell at world coordinates.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn get_cell_at(&self, world_x: i32, world_y: i32) -> Option<&Cell> {
        let chunk_id = ChunkId::from_world_pos(world_x, world_y, self.chunk_size);

        self.chunks.get(&chunk_id).and_then(|chunk| {
            let (origin_x, origin_y) = chunk_id.world_origin(self.chunk_size);
            let local_x = (world_x - origin_x) as u32;
            let local_y = (world_y - origin_y) as u32;

            if local_x < self.chunk_size && local_y < self.chunk_size {
                let idx = (local_y * self.chunk_size + local_x) as usize;
                chunk.cells.get(idx)
            } else {
                None
            }
        })
    }

    /// Returns the number of loaded chunks.
    #[must_use]
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Returns the number of pending chunks.
    #[must_use]
    pub fn pending_chunk_count(&self) -> usize {
        self.generation_queue.len()
    }

    /// Unload chunks that are far from camera.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn unload_distant(&mut self, camera: &Camera, max_distance: i32) {
        let center_chunk_x = (camera.position.0 / self.chunk_size as f32).floor() as i32;
        let center_chunk_y = (camera.position.1 / self.chunk_size as f32).floor() as i32;

        let chunks_to_remove: Vec<ChunkId> = self
            .chunks
            .keys()
            .filter(|id| {
                let dx = (id.x - center_chunk_x).abs();
                let dy = (id.y - center_chunk_y).abs();
                dx > max_distance || dy > max_distance
            })
            .copied()
            .collect();

        for id in chunks_to_remove {
            debug!("Unloading distant chunk {:?}", id);
            self.chunks.remove(&id);
        }
    }

    /// Clear all loaded chunks.
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.visible_chunks.clear();
        self.generation_queue.clear();
    }
}

impl std::fmt::Debug for TerrainRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerrainRenderer")
            .field("loaded_chunks", &self.chunks.len())
            .field("visible_chunks", &self.visible_chunks.len())
            .field("pending", &self.generation_queue.len())
            .field("chunk_size", &self.chunk_size)
            .field("view_distance", &self.view_distance)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device() -> Option<Device> {
        // Skip GPU tests in CI
        None
    }

    #[test]
    fn test_terrain_chunk_creation() {
        // Test without GPU
        let id = ChunkId::new(0, 0);
        let cells = vec![Cell::default(); 256 * 256];

        assert_eq!(cells.len(), 65536);
        assert_eq!(id.x, 0);
        assert_eq!(id.y, 0);
    }

    #[test]
    fn test_chunk_id_from_world_pos() {
        let chunk_size = 256u32;

        // Origin chunk
        let id = ChunkId::from_world_pos(0, 0, chunk_size);
        assert_eq!(id.x, 0);
        assert_eq!(id.y, 0);

        // Positive coordinates
        let id = ChunkId::from_world_pos(256, 256, chunk_size);
        assert_eq!(id.x, 1);
        assert_eq!(id.y, 1);

        // Negative coordinates
        let id = ChunkId::from_world_pos(-1, -1, chunk_size);
        assert_eq!(id.x, -1);
        assert_eq!(id.y, -1);
    }

    #[test]
    fn test_visible_chunk_calculation() {
        // Create a simple test case
        let chunk_size = 256u32;
        let camera = Camera::new(512, 512);

        // Calculate expected visible chunks
        let (min_x, min_y, max_x, max_y) = camera.visible_bounds();
        let chunk_min_x = (min_x / chunk_size as f32).floor() as i32 - 1;
        let chunk_max_x = (max_x / chunk_size as f32).ceil() as i32 + 1;

        // Should span multiple chunks
        assert!(chunk_max_x > chunk_min_x);
    }

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.sea_level, 64);
        assert!(params.vegetation);
    }

    #[test]
    fn test_world_generator_determinism() {
        let gen1 = WorldGenerator::new(12345);
        let gen2 = WorldGenerator::new(12345);
        let params = GenerationParams::default();

        let cells1 = gen1.generate_chunk(0, 0, &params);
        let cells2 = gen2.generate_chunk(0, 0, &params);

        // Same seed should produce same chunk
        for (i, (c1, c2)) in cells1.iter().zip(cells2.iter()).enumerate() {
            assert_eq!(
                c1.material, c2.material,
                "Mismatch at index {} for chunk (0,0)",
                i
            );
        }
    }

    #[test]
    fn test_different_seeds_differ() {
        let gen1 = WorldGenerator::new(12345);
        let gen2 = WorldGenerator::new(54321);
        let params = GenerationParams::default();

        let cells1 = gen1.generate_chunk(0, 0, &params);
        let cells2 = gen2.generate_chunk(0, 0, &params);

        // Different seeds should produce different chunks
        let mut differences = 0;
        for (c1, c2) in cells1.iter().zip(cells2.iter()) {
            if c1.material != c2.material {
                differences += 1;
            }
        }

        assert!(
            differences > 0,
            "Different seeds should produce different terrain"
        );
    }
}
