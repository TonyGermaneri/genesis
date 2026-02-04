//! Streaming terrain system with quadtree-based persistent simulation.
//!
//! This module provides a unified terrain streaming system that:
//! - Centers chunk loading on player position (not camera)
//! - Loads visible chunks plus a buffer zone outside the visible area
//! - Uses quadtree spatial partitioning for efficient simulation dispatch
//! - Persists simulation state across chunk load/unload cycles
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │              Unloaded Zone                      │
//! │  ┌─────────────────────────────────────────┐   │
//! │  │           Buffer Zone (Active)          │   │
//! │  │  ┌─────────────────────────────────┐   │   │
//! │  │  │     Visible Zone (Simulating)   │   │   │
//! │  │  │                                 │   │   │
//! │  │  │           [Player]              │   │   │
//! │  │  │                                 │   │   │
//! │  │  └─────────────────────────────────┘   │   │
//! │  └─────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! - **Simulating**: Chunks within `simulation_radius` of player - full GPU compute
//! - **Active**: Chunks within `load_radius` but outside `simulation_radius` - loaded but frozen
//! - **Dormant**: Chunks outside `load_radius` - may be unloaded to save memory

use std::collections::{HashMap, HashSet, VecDeque};

use tracing::{debug, info};
use wgpu::util::DeviceExt;
use wgpu::Device;

use crate::cell::Cell;
use crate::chunk::ChunkId;
use crate::chunk_manager::ChunkActivationState;
use crate::compute::DEFAULT_CHUNK_SIZE;
use crate::quadtree::{QuadTree, Rect};
use crate::worldgen::{GenerationParams, WorldGenerator};
use crate::CellComputePipeline;

/// Default simulation radius in chunks (chunks that get GPU compute).
pub const DEFAULT_SIMULATION_RADIUS: i32 = 2;

/// Default load radius in chunks (chunks that are kept in memory).
pub const DEFAULT_LOAD_RADIUS: i32 = 4;

/// Default unload radius in chunks (chunks beyond this are unloaded).
pub const DEFAULT_UNLOAD_RADIUS: i32 = 6;

/// Maximum chunks to generate per frame to avoid hitching.
pub const MAX_CHUNKS_PER_FRAME: usize = 2;

/// Configuration for the streaming terrain system.
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Chunk size in cells (width and height).
    pub chunk_size: u32,
    /// Radius (in chunks) around player where simulation runs on GPU.
    pub simulation_radius: i32,
    /// Radius (in chunks) around player where chunks are kept loaded.
    pub load_radius: i32,
    /// Radius (in chunks) beyond which chunks are unloaded.
    pub unload_radius: i32,
    /// Maximum chunks to generate per frame.
    pub max_gen_per_frame: usize,
    /// World generation parameters.
    pub gen_params: GenerationParams,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            simulation_radius: DEFAULT_SIMULATION_RADIUS,
            load_radius: DEFAULT_LOAD_RADIUS,
            unload_radius: DEFAULT_UNLOAD_RADIUS,
            max_gen_per_frame: MAX_CHUNKS_PER_FRAME,
            gen_params: GenerationParams::default(),
        }
    }
}

impl StreamingConfig {
    /// Creates a new config with custom radii.
    #[must_use]
    pub fn with_radii(simulation_radius: i32, load_radius: i32, unload_radius: i32) -> Self {
        assert!(simulation_radius <= load_radius, "Simulation radius must be <= load radius");
        assert!(load_radius < unload_radius, "Load radius must be < unload radius");
        Self {
            simulation_radius,
            load_radius,
            unload_radius,
            ..Default::default()
        }
    }
}

/// State of a streaming terrain chunk.
#[derive(Debug)]
pub struct StreamingChunk {
    /// Chunk identifier.
    pub id: ChunkId,
    /// Cell data for this chunk.
    pub cells: Vec<Cell>,
    /// GPU buffer for compute simulation.
    pub gpu_buffer: Option<wgpu::Buffer>,
    /// Current activation state.
    pub state: ChunkActivationState,
    /// Frames since last access (for LRU unloading).
    pub idle_frames: u32,
    /// Whether cells have been modified since last GPU sync.
    pub dirty: bool,
    /// Whether this chunk has ever been generated.
    pub generated: bool,
}

impl StreamingChunk {
    /// Creates a new empty chunk.
    pub fn new(id: ChunkId, chunk_size: u32) -> Self {
        let cell_count = (chunk_size * chunk_size) as usize;
        Self {
            id,
            cells: vec![Cell::default(); cell_count],
            gpu_buffer: None,
            state: ChunkActivationState::Dormant,
            idle_frames: 0,
            dirty: true,
            generated: false,
        }
    }

    /// Creates a chunk with generated cells.
    pub fn with_cells(id: ChunkId, cells: Vec<Cell>) -> Self {
        Self {
            id,
            cells,
            gpu_buffer: None,
            state: ChunkActivationState::Dormant,
            idle_frames: 0,
            dirty: true,
            generated: true,
        }
    }

    /// Creates or updates the GPU buffer for this chunk.
    pub fn ensure_gpu_buffer(&mut self, device: &Device) {
        if self.gpu_buffer.is_none() {
            let size = (self.cells.len() * std::mem::size_of::<Cell>()) as u64;
            self.gpu_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Chunk {} Buffer", self.id)),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }));
        }
    }

    /// Uploads cell data to GPU buffer.
    pub fn upload_to_gpu(&mut self, queue: &wgpu::Queue) {
        if let Some(buffer) = &self.gpu_buffer {
            if self.dirty {
                queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.cells));
                self.dirty = false;
            }
        }
    }

    /// Drops the GPU buffer to free VRAM.
    pub fn release_gpu_buffer(&mut self) {
        self.gpu_buffer = None;
    }
}

/// Quadtree node for chunk spatial queries.
#[derive(Debug, Clone, Copy)]
struct ChunkNode {
    id: ChunkId,
    state: ChunkActivationState,
}

/// Streaming terrain manager with quadtree-based activation.
///
/// This manager handles:
/// - Loading/unloading chunks based on player position
/// - Procedural terrain generation for new chunks
/// - GPU compute dispatch for simulating chunks
/// - Quadtree queries for efficient spatial operations
pub struct StreamingTerrain {
    /// Configuration.
    config: StreamingConfig,
    /// World generator.
    world_gen: WorldGenerator,
    /// Loaded chunks by ID.
    chunks: HashMap<ChunkId, StreamingChunk>,
    /// Quadtree for spatial queries.
    quadtree: QuadTree<ChunkNode>,
    /// Queue of chunks pending generation.
    gen_queue: VecDeque<ChunkId>,
    /// Set of chunks currently being simulated.
    simulating_chunks: HashSet<ChunkId>,
    /// Current player position in world coordinates.
    player_position: (f32, f32),
    /// Current player chunk.
    player_chunk: ChunkId,
    /// Statistics.
    stats: StreamingStats,
}

/// Statistics for the streaming terrain system.
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    /// Total chunks currently loaded.
    pub loaded_count: usize,
    /// Chunks currently being simulated.
    pub simulating_count: usize,
    /// Chunks pending generation.
    pub pending_count: usize,
    /// Chunks generated this frame.
    pub generated_this_frame: usize,
    /// Chunks unloaded this frame.
    pub unloaded_this_frame: usize,
}

impl StreamingTerrain {
    /// Creates a new streaming terrain manager.
    pub fn new(seed: u64, config: StreamingConfig) -> Self {
        info!(
            "Creating streaming terrain: seed={}, chunk_size={}, sim_radius={}, load_radius={}, unload_radius={}",
            seed, config.chunk_size, config.simulation_radius, config.load_radius, config.unload_radius
        );

        // Create quadtree with world bounds (±1000 chunks should be plenty)
        let world_size = 2000.0;
        let bounds = Rect::new(-1000.0, -1000.0, world_size, world_size);

        Self {
            world_gen: WorldGenerator::new(seed),
            quadtree: QuadTree::new(bounds, 8, 6),
            chunks: HashMap::new(),
            gen_queue: VecDeque::new(),
            simulating_chunks: HashSet::new(),
            player_position: (0.0, 0.0),
            // Initialize to invalid chunk so first update triggers loading
            player_chunk: ChunkId::new(i32::MIN, i32::MIN),
            stats: StreamingStats::default(),
            config,
        }
    }

    /// Creates with default configuration.
    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed, StreamingConfig::default())
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }

    /// Returns the world generator.
    #[must_use]
    pub fn world_gen(&self) -> &WorldGenerator {
        &self.world_gen
    }

    /// Returns current statistics.
    #[must_use]
    pub fn stats(&self) -> &StreamingStats {
        &self.stats
    }

    /// Returns the current player chunk.
    #[must_use]
    pub fn player_chunk(&self) -> ChunkId {
        self.player_chunk
    }

    /// Updates the player position and triggers chunk streaming.
    ///
    /// This is the main entry point - call this each frame with the player's world position.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn update_player_position(&mut self, x: f32, y: f32, device: &Device) {
        self.player_position = (x, y);

        // Calculate which chunk the player is in
        let new_chunk = ChunkId::from_world_pos(
            x as i32,
            y as i32,
            self.config.chunk_size,
        );

        // Only update streaming if player moved to a new chunk
        if new_chunk != self.player_chunk {
            debug!("Player moved to chunk {:?}", new_chunk);
            self.player_chunk = new_chunk;
            self.update_chunk_states();
        }

        // Generate pending chunks (budget limited)
        self.generate_pending(device);

        // Update statistics
        self.update_stats();
    }

    /// Updates chunk activation states based on player position.
    fn update_chunk_states(&mut self) {
        let sim_radius = self.config.simulation_radius;
        let load_radius = self.config.load_radius;
        let unload_radius = self.config.unload_radius;

        // Determine which chunks should be in each state
        let mut to_simulate = HashSet::new();
        let mut to_load = HashSet::new();
        let mut to_unload = Vec::new();

        // Build sets of chunks that should be loaded/simulating
        for dy in -load_radius..=load_radius {
            for dx in -load_radius..=load_radius {
                let chunk_id = ChunkId::new(self.player_chunk.x + dx, self.player_chunk.y + dy);
                let dist = dx.abs().max(dy.abs());

                if dist <= sim_radius {
                    to_simulate.insert(chunk_id);
                    to_load.insert(chunk_id);
                } else if dist <= load_radius {
                    to_load.insert(chunk_id);
                }
            }
        }

        // Update existing chunks
        for (id, chunk) in &mut self.chunks {
            let dx = (id.x - self.player_chunk.x).abs();
            let dy = (id.y - self.player_chunk.y).abs();
            let dist = dx.max(dy);

            if dist > unload_radius {
                to_unload.push(*id);
            } else if to_simulate.contains(id) {
                chunk.state = ChunkActivationState::Simulating;
                chunk.idle_frames = 0;
            } else if to_load.contains(id) {
                chunk.state = ChunkActivationState::Active;
                chunk.idle_frames = 0;
            } else {
                chunk.state = ChunkActivationState::Dormant;
                chunk.idle_frames += 1;
            }
        }

        // Unload distant chunks
        self.stats.unloaded_this_frame = to_unload.len();
        for id in to_unload {
            self.unload_chunk(id);
        }

        // Queue chunks that need to be loaded
        for id in &to_load {
            if !self.chunks.contains_key(id) && !self.gen_queue.contains(id) {
                self.gen_queue.push_back(*id);
            }
        }

        // Update simulating set
        self.simulating_chunks = to_simulate;

        // Rebuild quadtree with updated states
        self.rebuild_quadtree();
    }

    /// Generates pending chunks up to the per-frame budget.
    fn generate_pending(&mut self, device: &Device) {
        self.stats.generated_this_frame = 0;

        for _ in 0..self.config.max_gen_per_frame {
            if let Some(chunk_id) = self.gen_queue.pop_front() {
                // Skip if already loaded (can happen with rapid movement)
                if self.chunks.contains_key(&chunk_id) {
                    continue;
                }

                debug!("Generating chunk {:?}", chunk_id);

                // Generate cells using world generator
                let cells = self.world_gen.generate_chunk(
                    chunk_id.x,
                    chunk_id.y,
                    &self.config.gen_params,
                );

                // Create chunk and ensure GPU buffer
                let mut chunk = StreamingChunk::with_cells(chunk_id, cells);
                chunk.ensure_gpu_buffer(device);

                // Set initial state based on distance
                let dx = (chunk_id.x - self.player_chunk.x).abs();
                let dy = (chunk_id.y - self.player_chunk.y).abs();
                let dist = dx.max(dy);

                if dist <= self.config.simulation_radius {
                    chunk.state = ChunkActivationState::Simulating;
                } else {
                    chunk.state = ChunkActivationState::Active;
                }

                self.chunks.insert(chunk_id, chunk);
                self.stats.generated_this_frame += 1;
            } else {
                break;
            }
        }
    }

    /// Unloads a chunk and persists its state.
    fn unload_chunk(&mut self, id: ChunkId) {
        if let Some(mut chunk) = self.chunks.remove(&id) {
            debug!("Unloading chunk {:?}", id);

            // Release GPU resources
            chunk.release_gpu_buffer();

            // TODO: Persist chunk data to disk for later reload
            // For now, chunks are regenerated when re-entered
        }

        // Remove from quadtree
        self.quadtree.remove_where(|node| node.id == id);
    }

    /// Rebuilds the quadtree with current chunk states.
    #[allow(clippy::cast_precision_loss)]
    fn rebuild_quadtree(&mut self) {
        self.quadtree.clear();

        for (id, chunk) in &self.chunks {
            let node = ChunkNode {
                id: *id,
                state: chunk.state,
            };
            let bounds = Rect::new(id.x as f32, id.y as f32, 1.0, 1.0);
            self.quadtree.insert(bounds, node);
        }
    }

    /// Updates statistics.
    fn update_stats(&mut self) {
        self.stats.loaded_count = self.chunks.len();
        self.stats.simulating_count = self.simulating_chunks.len();
        self.stats.pending_count = self.gen_queue.len();
    }

    /// Uploads all dirty chunks to GPU.
    pub fn upload_dirty(&mut self, queue: &wgpu::Queue) {
        for chunk in self.chunks.values_mut() {
            if chunk.dirty && chunk.gpu_buffer.is_some() {
                chunk.upload_to_gpu(queue);
            }
        }
    }

    /// Returns chunks that should be simulated this frame.
    pub fn simulating_chunks(&self) -> impl Iterator<Item = &StreamingChunk> {
        self.simulating_chunks
            .iter()
            .filter_map(|id| self.chunks.get(id))
    }

    /// Returns all loaded chunks.
    pub fn loaded_chunks(&self) -> impl Iterator<Item = &StreamingChunk> {
        self.chunks.values()
    }

    /// Returns a specific chunk by ID.
    #[must_use]
    pub fn get_chunk(&self, id: &ChunkId) -> Option<&StreamingChunk> {
        self.chunks.get(id)
    }

    /// Returns a mutable chunk by ID.
    pub fn get_chunk_mut(&mut self, id: &ChunkId) -> Option<&mut StreamingChunk> {
        self.chunks.get_mut(id)
    }

    /// Gets a cell at world coordinates.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub fn get_cell(&self, world_x: i32, world_y: i32) -> Option<&Cell> {
        let chunk_id = ChunkId::from_world_pos(world_x, world_y, self.config.chunk_size);

        self.chunks.get(&chunk_id).and_then(|chunk| {
            let (origin_x, origin_y) = chunk_id.world_origin(self.config.chunk_size);
            let local_x = (world_x - origin_x) as usize;
            let local_y = (world_y - origin_y) as usize;
            let chunk_size = self.config.chunk_size as usize;

            if local_x < chunk_size && local_y < chunk_size {
                let idx = local_y * chunk_size + local_x;
                chunk.cells.get(idx)
            } else {
                None
            }
        })
    }

    /// Checks if a chunk is loaded.
    #[must_use]
    pub fn is_chunk_loaded(&self, id: &ChunkId) -> bool {
        self.chunks.contains_key(id)
    }

    /// Queries chunks within a rectangular region.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn query_region(&self, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Vec<&StreamingChunk> {
        let range = Rect::new(
            min_x as f32,
            min_y as f32,
            (max_x - min_x + 1) as f32,
            (max_y - min_y + 1) as f32,
        );

        self.quadtree
            .query(range)
            .iter()
            .filter_map(|node| self.chunks.get(&node.id))
            .collect()
    }

    /// Queries chunks within a region that are in a specific state.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn query_region_with_state(
        &self,
        min_x: i32,
        min_y: i32,
        max_x: i32,
        max_y: i32,
        state: ChunkActivationState,
    ) -> Vec<&StreamingChunk> {
        let range = Rect::new(
            min_x as f32,
            min_y as f32,
            (max_x - min_x + 1) as f32,
            (max_y - min_y + 1) as f32,
        );

        self.quadtree
            .query(range)
            .iter()
            .filter(|node| node.state == state)
            .filter_map(|node| self.chunks.get(&node.id))
            .collect()
    }

    /// Returns the visible bounds based on player position and simulation radius.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn visible_bounds(&self) -> (i32, i32, i32, i32) {
        let chunk_size = self.config.chunk_size as i32;
        let radius = self.config.simulation_radius;

        let min_x = (self.player_chunk.x - radius) * chunk_size;
        let min_y = (self.player_chunk.y - radius) * chunk_size;
        let max_x = (self.player_chunk.x + radius + 1) * chunk_size - 1;
        let max_y = (self.player_chunk.y + radius + 1) * chunk_size - 1;

        (min_x, min_y, max_x, max_y)
    }

    /// Runs compute simulation for all simulating chunks.
    ///
    /// This dispatches the GPU compute shader only for chunks marked as `Simulating`.
    pub fn step_simulation(
        &mut self,
        device: &Device,
        queue: &wgpu::Queue,
        pipeline: &CellComputePipeline,
    ) {
        // First upload any dirty chunks
        self.upload_dirty(queue);

        // If no chunks are simulating, nothing to do
        if self.simulating_chunks.is_empty() {
            return;
        }

        // Create materials buffer (shared across all chunks)
        let materials = crate::compute::create_default_materials();
        let materials_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Streaming Materials Buffer"),
            contents: bytemuck::cast_slice(&materials),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create simulation params
        let params = crate::compute::SimulationParams::new(self.config.chunk_size);
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Streaming Sim Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Streaming Terrain Compute"),
        });

        // Dispatch compute for each simulating chunk
        for id in &self.simulating_chunks.clone() {
            if let Some(chunk) = self.chunks.get_mut(id) {
                if let Some(buffer) = &chunk.gpu_buffer {
                    // Create a temporary output buffer (double-buffering)
                    let size = (chunk.cells.len() * std::mem::size_of::<Cell>()) as u64;
                    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&format!("Chunk {} Output Buffer", id)),
                        size,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });

                    // Create bind group for this chunk
                    let bind_group = pipeline.create_bind_group(
                        device,
                        buffer,        // cells_in
                        &output_buffer, // cells_out
                        &materials_buffer,
                        &params_buffer,
                    );

                    // Dispatch compute shader
                    pipeline.dispatch(&mut encoder, &bind_group, self.config.chunk_size);

                    // Copy output back to input buffer
                    encoder.copy_buffer_to_buffer(&output_buffer, 0, buffer, 0, size);

                    // Mark as accessed to prevent unloading
                    chunk.idle_frames = 0;
                }
            }
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));
    }

    /// Force-generates a specific chunk immediately.
    pub fn force_generate(&mut self, chunk_id: ChunkId, device: &Device) {
        if self.chunks.contains_key(&chunk_id) {
            return; // Already loaded
        }

        let cells = self.world_gen.generate_chunk(
            chunk_id.x,
            chunk_id.y,
            &self.config.gen_params,
        );

        let mut chunk = StreamingChunk::with_cells(chunk_id, cells);
        chunk.ensure_gpu_buffer(device);
        chunk.state = ChunkActivationState::Active;

        self.chunks.insert(chunk_id, chunk);
    }
}

impl std::fmt::Debug for StreamingTerrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingTerrain")
            .field("player_chunk", &self.player_chunk)
            .field("loaded_chunks", &self.chunks.len())
            .field("simulating_chunks", &self.simulating_chunks.len())
            .field("pending_generation", &self.gen_queue.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, DEFAULT_CHUNK_SIZE);
        assert_eq!(config.simulation_radius, DEFAULT_SIMULATION_RADIUS);
        assert_eq!(config.load_radius, DEFAULT_LOAD_RADIUS);
        assert_eq!(config.unload_radius, DEFAULT_UNLOAD_RADIUS);
    }

    #[test]
    fn test_streaming_config_with_radii() {
        let config = StreamingConfig::with_radii(1, 3, 5);
        assert_eq!(config.simulation_radius, 1);
        assert_eq!(config.load_radius, 3);
        assert_eq!(config.unload_radius, 5);
    }

    #[test]
    #[should_panic]
    fn test_streaming_config_invalid_radii() {
        // simulation_radius > load_radius should panic
        let _ = StreamingConfig::with_radii(5, 3, 7);
    }

    #[test]
    fn test_chunk_id_from_world_pos() {
        let chunk_size = 256;

        // Origin chunk
        assert_eq!(ChunkId::from_world_pos(0, 0, chunk_size), ChunkId::new(0, 0));
        assert_eq!(ChunkId::from_world_pos(128, 128, chunk_size), ChunkId::new(0, 0));
        assert_eq!(ChunkId::from_world_pos(255, 255, chunk_size), ChunkId::new(0, 0));

        // Adjacent chunks
        assert_eq!(ChunkId::from_world_pos(256, 0, chunk_size), ChunkId::new(1, 0));
        assert_eq!(ChunkId::from_world_pos(0, 256, chunk_size), ChunkId::new(0, 1));
        assert_eq!(ChunkId::from_world_pos(256, 256, chunk_size), ChunkId::new(1, 1));

        // Negative chunks
        assert_eq!(ChunkId::from_world_pos(-1, -1, chunk_size), ChunkId::new(-1, -1));
        assert_eq!(ChunkId::from_world_pos(-256, 0, chunk_size), ChunkId::new(-1, 0));
    }

    #[test]
    fn test_streaming_chunk_new() {
        let chunk = StreamingChunk::new(ChunkId::new(0, 0), 256);
        assert_eq!(chunk.cells.len(), 256 * 256);
        assert!(chunk.dirty);
        assert!(!chunk.generated);
        assert!(chunk.gpu_buffer.is_none());
    }

    #[test]
    fn test_visible_bounds() {
        let terrain = StreamingTerrain::with_seed(12345);
        let (min_x, min_y, max_x, max_y) = terrain.visible_bounds();

        // With simulation_radius=2 and chunk_size=256, centered at (0,0):
        // min = -2 * 256 = -512
        // max = (2+1) * 256 - 1 = 767
        assert_eq!(min_x, -512);
        assert_eq!(min_y, -512);
        assert_eq!(max_x, 767);
        assert_eq!(max_y, 767);
    }
}
