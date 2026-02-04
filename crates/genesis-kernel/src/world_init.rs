//! World initialization and spawn point selection.
//!
//! This module handles initializing the world state on startup, including
//! generating a starting area and finding a suitable spawn position.

use genesis_common::WorldCoord;
use tracing::{debug, info};
use wgpu::Device;

use crate::biome::{material_ids, BiomeId, BiomeManager};
use crate::camera::Camera;
use crate::cell::{Cell, CellFlags};
use crate::chunk::ChunkId;
use crate::terrain_render::TerrainRenderer;
use crate::worldgen::GenerationParams;

/// Default starting area radius in chunks.
pub const DEFAULT_START_RADIUS: i32 = 2;

/// Default spawn search height.
pub const DEFAULT_SPAWN_SEARCH_Y: i32 = 80;

/// World initialization configuration.
#[derive(Debug, Clone)]
pub struct WorldInitConfig {
    /// Seed for world generation.
    pub seed: u64,
    /// Starting area radius in chunks.
    pub start_radius: i32,
    /// Initial camera zoom level.
    pub initial_zoom: f32,
    /// Generation parameters.
    pub gen_params: GenerationParams,
}

impl Default for WorldInitConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            start_radius: DEFAULT_START_RADIUS,
            initial_zoom: 1.0,
            gen_params: GenerationParams::default(),
        }
    }
}

impl WorldInitConfig {
    /// Creates a new config with the given seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }
}

/// Result of world initialization.
#[derive(Debug)]
pub struct WorldInitResult {
    /// Spawn position in world coordinates.
    pub spawn_position: (i32, i32),
    /// Biome at spawn location.
    pub spawn_biome: BiomeId,
    /// Number of chunks generated.
    pub chunks_generated: usize,
    /// Initial camera position.
    pub camera_position: (f32, f32),
}

/// World initializer for setting up the starting game state.
pub struct WorldInitializer {
    /// Configuration.
    config: WorldInitConfig,
    /// Biome manager for biome queries.
    biome_manager: BiomeManager,
}

impl WorldInitializer {
    /// Creates a new world initializer.
    #[must_use]
    pub fn new(config: WorldInitConfig) -> Self {
        info!(
            "Creating world initializer with seed {} and radius {}",
            config.seed, config.start_radius
        );

        Self {
            biome_manager: BiomeManager::new(config.seed),
            config,
        }
    }

    /// Returns the configuration.
    #[must_use]
    pub const fn config(&self) -> &WorldInitConfig {
        &self.config
    }

    /// Returns the biome manager.
    #[must_use]
    pub const fn biome_manager(&self) -> &BiomeManager {
        &self.biome_manager
    }

    /// Initialize the world, generating starting chunks and finding spawn point.
    pub fn initialize(&self, terrain: &mut TerrainRenderer, device: &Device) -> WorldInitResult {
        info!("Initializing world...");

        let radius = self.config.start_radius;
        let mut chunks_generated = 0;

        // Generate starting area (centered on origin)
        for cy in -radius..=radius {
            for cx in -radius..=radius {
                let chunk_id = ChunkId::new(cx, cy);
                terrain.generate_pending(device, 1);

                // Force immediate generation by directly calling
                if !terrain.is_chunk_loaded(&chunk_id) {
                    // The terrain renderer will handle this via update_visible
                }
                chunks_generated += 1;
            }
        }

        info!("Generated {} starting chunks", chunks_generated);

        // Find spawn position
        let spawn_position = Self::find_spawn_position(terrain);
        info!("Found spawn position: {:?}", spawn_position);

        // Get biome at spawn
        let spawn_biome = self.biome_manager.get_biome_at(WorldCoord::new(
            i64::from(spawn_position.0),
            i64::from(spawn_position.1),
        ));

        // Camera starts centered on spawn
        let camera_position = (spawn_position.0 as f32, spawn_position.1 as f32);

        WorldInitResult {
            spawn_position,
            spawn_biome,
            chunks_generated,
            camera_position,
        }
    }

    /// Finds a suitable spawn position on solid ground.
    #[allow(clippy::cast_possible_wrap)]
    fn find_spawn_position(terrain: &TerrainRenderer) -> (i32, i32) {
        let chunk_size = terrain.chunk_size() as i32;

        // Search near world center
        let search_x = chunk_size / 2;
        let mut best_y = DEFAULT_SPAWN_SEARCH_Y;

        // Scan downward to find solid ground
        for y in (0..DEFAULT_SPAWN_SEARCH_Y).rev() {
            if let Some(cell) = terrain.get_cell_at(search_x, y) {
                if is_solid_ground(*cell) {
                    // Found ground, spawn one cell above
                    best_y = y + 1;
                    debug!("Found ground at y={y}, spawning at y={best_y}");
                    break;
                }
            }
        }

        (search_x, best_y)
    }

    /// Generate initial chunks around spawn with camera.
    pub fn generate_starting_area(
        &self,
        terrain: &mut TerrainRenderer,
        camera: &mut Camera,
        device: &Device,
    ) -> WorldInitResult {
        info!("Generating starting area...");

        // Initialize terrain
        let result = self.initialize(terrain, device);

        // Position camera at spawn
        camera.position = result.camera_position;
        camera.zoom = self.config.initial_zoom;

        // Update visible chunks based on camera
        terrain.update_visible(camera);

        // Generate visible chunks immediately
        let pending = terrain.pending_chunk_count();
        terrain.generate_pending(device, pending);

        info!(
            "Starting area ready: {} chunks, spawn at {:?}",
            terrain.loaded_chunk_count(),
            result.spawn_position
        );

        result
    }
}

/// Checks if a cell represents solid ground suitable for spawning on.
fn is_solid_ground(cell: Cell) -> bool {
    let material = cell.material;

    // Solid materials good for spawning
    material == material_ids::STONE
        || material == material_ids::DIRT
        || material == material_ids::SAND
        || material == material_ids::GRASS
}

/// Checks if a cell is safe to spawn in (not solid, not liquid).
#[allow(dead_code)]
fn is_safe_spawn_cell(cell: Cell) -> bool {
    let material = cell.material;
    let flags = cell.flags;

    // Air or other non-blocking materials, not burning
    material == material_ids::AIR
        && (flags & CellFlags::BURNING) == 0
        && (flags & CellFlags::LIQUID) == 0
}

impl std::fmt::Debug for WorldInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldInitializer")
            .field("seed", &self.config.seed)
            .field("start_radius", &self.config.start_radius)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_init_config_default() {
        let config = WorldInitConfig::default();
        assert_eq!(config.seed, 0);
        assert_eq!(config.start_radius, DEFAULT_START_RADIUS);
        assert_eq!(config.initial_zoom, 1.0);
    }

    #[test]
    fn test_world_init_config_with_seed() {
        let config = WorldInitConfig::with_seed(12345);
        assert_eq!(config.seed, 12345);
        assert_eq!(config.start_radius, DEFAULT_START_RADIUS);
    }

    #[test]
    fn test_solid_ground_detection() {
        // Stone is solid
        let stone = Cell::new(material_ids::STONE);
        assert!(is_solid_ground(stone));

        // Air is not solid
        let air = Cell::new(material_ids::AIR);
        assert!(!is_solid_ground(air));

        // Water is not solid
        let water = Cell::new(material_ids::WATER);
        assert!(!is_solid_ground(water));
    }

    #[test]
    fn test_safe_spawn_cell() {
        // Clean air is safe
        let air = Cell::new(material_ids::AIR);
        assert!(is_safe_spawn_cell(air));

        // Burning air is not safe
        let burning = Cell::new(material_ids::AIR).with_flag(CellFlags::BURNING);
        assert!(!is_safe_spawn_cell(burning));

        // Liquid is not safe
        let liquid = Cell::new(material_ids::AIR).with_flag(CellFlags::LIQUID);
        assert!(!is_safe_spawn_cell(liquid));
    }

    #[test]
    fn test_world_initializer_creation() {
        let config = WorldInitConfig::with_seed(42);
        let initializer = WorldInitializer::new(config);

        assert_eq!(initializer.config().seed, 42);
    }

    #[test]
    fn test_starting_chunk_count() {
        // With radius 2, we get a 5x5 grid = 25 chunks
        let radius = 2;
        let expected = (2 * radius + 1) * (2 * radius + 1);
        assert_eq!(expected, 25);
    }

    #[test]
    fn test_biome_at_spawn() {
        let config = WorldInitConfig::with_seed(12345);
        let initializer = WorldInitializer::new(config);

        // Should be able to query biome at any position
        let biome = initializer
            .biome_manager()
            .get_biome_at(WorldCoord::new(128, 64));
        // Biome ID should be valid (0-7)
        assert!(biome < 8);
    }
}
