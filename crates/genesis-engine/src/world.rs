//! World management and biome generation integration.
//!
//! This module provides centralized world seed management and wires
//! terrain generation into chunk creation.

#![allow(dead_code)]

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use genesis_common::WorldCoord;
use genesis_kernel::{BiomeId, WorldGenerator};
use tracing::{debug, info, warn};

/// Configuration for world generation.
#[derive(Debug, Clone)]
pub struct WorldConfig {
    /// World seed for deterministic generation
    pub seed: u64,
    /// Chunk size in cells
    pub chunk_size: u32,
    /// Render distance in chunks
    pub render_distance: u32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: Self::random_seed(),
            chunk_size: 256,
            render_distance: 3,
        }
    }
}

impl WorldConfig {
    /// Creates a new world config with the given seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Generates a random seed using system time.
    #[must_use]
    pub fn random_seed() -> u64 {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42)
    }
}

/// Centralized world seed manager.
///
/// Handles seed storage and propagation to all world generation systems.
#[derive(Debug)]
pub struct WorldSeedManager {
    /// Current world seed
    seed: u64,
    /// World generator instance
    world_gen: WorldGenerator,
    /// World configuration
    config: WorldConfig,
    /// Whether the world needs regeneration
    needs_regeneration: bool,
}

impl WorldSeedManager {
    /// Creates a new seed manager with the given config.
    #[must_use]
    pub fn new(config: WorldConfig) -> Self {
        info!("Initializing world seed manager with seed: {}", config.seed);
        let world_gen = WorldGenerator::new(config.seed);

        Self {
            seed: config.seed,
            world_gen,
            config,
            needs_regeneration: false,
        }
    }

    /// Creates a seed manager with a random seed.
    #[must_use]
    pub fn with_random_seed() -> Self {
        Self::new(WorldConfig::default())
    }

    /// Creates a seed manager from engine config.
    #[must_use]
    pub fn from_engine_config(engine_config: &crate::config::EngineConfig) -> Self {
        let seed = engine_config
            .world_seed
            .unwrap_or_else(WorldConfig::random_seed);
        let config = WorldConfig {
            seed,
            chunk_size: engine_config.chunk_size,
            render_distance: engine_config.render_distance,
        };
        Self::new(config)
    }

    /// Returns the current world seed.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns a reference to the world generator.
    #[must_use]
    pub fn world_gen(&self) -> &WorldGenerator {
        &self.world_gen
    }

    /// Returns a mutable reference to the world generator.
    pub fn world_gen_mut(&mut self) -> &mut WorldGenerator {
        &mut self.world_gen
    }

    /// Returns the world configuration.
    #[must_use]
    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    /// Changes the world seed (triggers regeneration).
    pub fn set_seed(&mut self, new_seed: u64) {
        if new_seed != self.seed {
            info!("World seed changed: {} -> {}", self.seed, new_seed);
            self.seed = new_seed;
            self.config.seed = new_seed;
            self.world_gen = WorldGenerator::new(new_seed);
            self.needs_regeneration = true;
        }
    }

    /// Checks and clears the regeneration flag.
    pub fn check_regeneration(&mut self) -> bool {
        let needs = self.needs_regeneration;
        self.needs_regeneration = false;
        needs
    }

    /// Generates biome data for a chunk.
    ///
    /// Returns the biome ID at the center of the chunk.
    #[must_use]
    pub fn get_chunk_biome(&self, chunk_x: i32, chunk_y: i32) -> BiomeId {
        let world_x = i64::from(chunk_x) * i64::from(self.config.chunk_size)
            + i64::from(self.config.chunk_size / 2);
        let world_y = i64::from(chunk_y) * i64::from(self.config.chunk_size)
            + i64::from(self.config.chunk_size / 2);
        let coord = WorldCoord::new(world_x, world_y);
        self.world_gen.biome_manager().get_biome_at(coord)
    }
}

/// Biome data for a chunk.
#[derive(Debug, Clone)]
pub struct ChunkBiomeData {
    /// Chunk coordinates
    pub chunk_x: i32,
    pub chunk_y: i32,
    /// Primary biome ID for this chunk
    pub biome_id: BiomeId,
    /// Per-cell biome IDs (optional, for biome blending)
    pub cell_biomes: Option<Vec<BiomeId>>,
    /// Generation timestamp
    pub generated_at: Instant,
}

impl ChunkBiomeData {
    /// Creates new biome data for a chunk.
    #[must_use]
    pub fn new(chunk_x: i32, chunk_y: i32, biome_id: BiomeId) -> Self {
        Self {
            chunk_x,
            chunk_y,
            biome_id,
            cell_biomes: None,
            generated_at: Instant::now(),
        }
    }

    /// Creates biome data with per-cell biomes.
    #[must_use]
    pub fn with_cell_biomes(
        chunk_x: i32,
        chunk_y: i32,
        biome_id: BiomeId,
        cell_biomes: Vec<BiomeId>,
    ) -> Self {
        Self {
            chunk_x,
            chunk_y,
            biome_id,
            cell_biomes: Some(cell_biomes),
            generated_at: Instant::now(),
        }
    }
}

/// Metrics for biome/terrain generation performance.
#[derive(Debug, Clone)]
pub struct BiomeGenerationMetrics {
    /// Recent generation times
    generation_times: VecDeque<Duration>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Total chunks generated this session
    total_chunks_generated: u64,
    /// Peak generation time
    peak_generation_time: Duration,
    /// Warning threshold (default 16ms)
    warning_threshold: Duration,
}

impl Default for BiomeGenerationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeGenerationMetrics {
    /// Creates a new metrics tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            generation_times: VecDeque::with_capacity(60),
            max_samples: 60,
            total_chunks_generated: 0,
            peak_generation_time: Duration::ZERO,
            warning_threshold: Duration::from_millis(16),
        }
    }

    /// Records a chunk generation time.
    pub fn record_generation(&mut self, duration: Duration) {
        self.generation_times.push_back(duration);
        if self.generation_times.len() > self.max_samples {
            self.generation_times.pop_front();
        }

        self.total_chunks_generated += 1;

        if duration > self.peak_generation_time {
            self.peak_generation_time = duration;
        }

        if duration > self.warning_threshold {
            warn!(
                "Chunk generation exceeded budget: {:.2}ms (limit: {:.2}ms)",
                duration.as_secs_f64() * 1000.0,
                self.warning_threshold.as_secs_f64() * 1000.0
            );
        }
    }

    /// Returns the average generation time in milliseconds.
    #[must_use]
    pub fn avg_generation_time_ms(&self) -> f64 {
        if self.generation_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.generation_times.iter().sum();
        total.as_secs_f64() * 1000.0 / self.generation_times.len() as f64
    }

    /// Returns the peak generation time in milliseconds.
    #[must_use]
    pub fn peak_generation_time_ms(&self) -> f64 {
        self.peak_generation_time.as_secs_f64() * 1000.0
    }

    /// Returns total chunks generated this session.
    #[must_use]
    pub fn total_chunks_generated(&self) -> u64 {
        self.total_chunks_generated
    }

    /// Returns whether the last generation exceeded the warning threshold.
    #[must_use]
    pub fn exceeds_budget(&self) -> bool {
        self.generation_times
            .back()
            .is_some_and(|d| *d > self.warning_threshold)
    }

    /// Sets the warning threshold.
    pub fn set_warning_threshold(&mut self, threshold_ms: f64) {
        self.warning_threshold = Duration::from_secs_f64(threshold_ms / 1000.0);
    }

    /// Clears all recorded metrics.
    pub fn clear(&mut self) {
        self.generation_times.clear();
        self.total_chunks_generated = 0;
        self.peak_generation_time = Duration::ZERO;
    }
}

/// Terrain generation wrapper that tracks metrics.
pub struct TerrainGenerationService {
    /// World seed manager
    seed_manager: WorldSeedManager,
    /// Generation metrics
    metrics: BiomeGenerationMetrics,
}

impl TerrainGenerationService {
    /// Creates a new terrain generation service.
    #[must_use]
    pub fn new(seed_manager: WorldSeedManager) -> Self {
        Self {
            seed_manager,
            metrics: BiomeGenerationMetrics::new(),
        }
    }

    /// Creates from engine config.
    #[must_use]
    pub fn from_engine_config(config: &crate::config::EngineConfig) -> Self {
        let seed_manager = WorldSeedManager::from_engine_config(config);
        Self::new(seed_manager)
    }

    /// Returns a reference to the seed manager.
    #[must_use]
    pub fn seed_manager(&self) -> &WorldSeedManager {
        &self.seed_manager
    }

    /// Returns a mutable reference to the seed manager.
    pub fn seed_manager_mut(&mut self) -> &mut WorldSeedManager {
        &mut self.seed_manager
    }

    /// Returns the generation metrics.
    #[must_use]
    pub fn metrics(&self) -> &BiomeGenerationMetrics {
        &self.metrics
    }

    /// Returns the current world seed.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed_manager.seed()
    }

    /// Changes the world seed.
    pub fn set_seed(&mut self, seed: u64) {
        self.seed_manager.set_seed(seed);
    }

    /// Generates terrain cells for a chunk with timing.
    #[must_use]
    pub fn generate_chunk(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
    ) -> (Vec<genesis_kernel::Cell>, ChunkBiomeData) {
        let start = Instant::now();

        // Get biome for chunk
        let biome_id = self.seed_manager.get_chunk_biome(chunk_x, chunk_y);

        // Generate terrain cells
        let params = genesis_kernel::GenerationParams::default();
        let cells = self
            .seed_manager
            .world_gen()
            .generate_chunk(chunk_x, chunk_y, &params);

        // Record metrics
        let elapsed = start.elapsed();
        self.metrics.record_generation(elapsed);

        debug!(
            "Generated chunk ({}, {}) biome={} in {:.2}ms",
            chunk_x,
            chunk_y,
            biome_id,
            elapsed.as_secs_f64() * 1000.0
        );

        let biome_data = ChunkBiomeData::new(chunk_x, chunk_y, biome_id);
        (cells, biome_data)
    }

    /// Generates only biome data for a chunk (faster, no cell generation).
    #[must_use]
    pub fn generate_biome_data(&self, chunk_x: i32, chunk_y: i32) -> ChunkBiomeData {
        let biome_id = self.seed_manager.get_chunk_biome(chunk_x, chunk_y);
        ChunkBiomeData::new(chunk_x, chunk_y, biome_id)
    }

    /// Generates detailed per-cell biome data.
    #[must_use]
    pub fn generate_detailed_biome_data(&mut self, chunk_x: i32, chunk_y: i32) -> ChunkBiomeData {
        let start = Instant::now();
        let chunk_size = self.seed_manager.config().chunk_size;
        let biome_manager = self.seed_manager.world_gen().biome_manager();

        let mut cell_biomes = Vec::with_capacity((chunk_size * chunk_size) as usize);
        let base_x = i64::from(chunk_x) * i64::from(chunk_size);
        let base_y = i64::from(chunk_y) * i64::from(chunk_size);

        for y in 0..chunk_size {
            for x in 0..chunk_size {
                let world_x = base_x + i64::from(x);
                let world_y = base_y + i64::from(y);
                let coord = WorldCoord::new(world_x, world_y);
                cell_biomes.push(biome_manager.get_biome_at(coord));
            }
        }

        let elapsed = start.elapsed();
        self.metrics.record_generation(elapsed);

        // Primary biome is the center cell's biome
        let center_idx = (chunk_size / 2 * chunk_size + chunk_size / 2) as usize;
        let primary_biome = cell_biomes.get(center_idx).copied().unwrap_or(0);

        ChunkBiomeData::with_cell_biomes(chunk_x, chunk_y, primary_biome, cell_biomes)
    }

    /// Checks if world regeneration is needed (seed changed).
    pub fn check_regeneration(&mut self) -> bool {
        self.seed_manager.check_regeneration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_config_default() {
        let config = WorldConfig::default();
        assert_eq!(config.chunk_size, 256);
        assert_eq!(config.render_distance, 3);
        // Seed should be non-zero (random)
        assert_ne!(config.seed, 0);
    }

    #[test]
    fn test_world_config_with_seed() {
        let config = WorldConfig::with_seed(12345);
        assert_eq!(config.seed, 12345);
    }

    #[test]
    fn test_seed_manager_creation() {
        let config = WorldConfig::with_seed(42);
        let manager = WorldSeedManager::new(config);
        assert_eq!(manager.seed(), 42);
    }

    #[test]
    fn test_seed_manager_change_seed() {
        let config = WorldConfig::with_seed(42);
        let mut manager = WorldSeedManager::new(config);

        manager.set_seed(999);
        assert_eq!(manager.seed(), 999);
        assert!(manager.check_regeneration());
        // Second check should be false
        assert!(!manager.check_regeneration());
    }

    #[test]
    fn test_biome_metrics_recording() {
        let mut metrics = BiomeGenerationMetrics::new();

        metrics.record_generation(Duration::from_millis(5));
        metrics.record_generation(Duration::from_millis(10));
        metrics.record_generation(Duration::from_millis(15));

        assert_eq!(metrics.total_chunks_generated(), 3);
        let avg = metrics.avg_generation_time_ms();
        assert!((avg - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_biome_metrics_peak() {
        let mut metrics = BiomeGenerationMetrics::new();

        metrics.record_generation(Duration::from_millis(5));
        metrics.record_generation(Duration::from_millis(20));
        metrics.record_generation(Duration::from_millis(10));

        assert!((metrics.peak_generation_time_ms() - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_chunk_biome_data() {
        let data = ChunkBiomeData::new(1, 2, 3);
        assert_eq!(data.chunk_x, 1);
        assert_eq!(data.chunk_y, 2);
        assert_eq!(data.biome_id, 3);
        assert!(data.cell_biomes.is_none());
    }

    #[test]
    fn test_terrain_service_creation() {
        let config = WorldConfig::with_seed(12345);
        let seed_manager = WorldSeedManager::new(config);
        let service = TerrainGenerationService::new(seed_manager);

        assert_eq!(service.seed(), 12345);
    }

    #[test]
    fn test_deterministic_generation() {
        let config1 = WorldConfig::with_seed(42);
        let config2 = WorldConfig::with_seed(42);

        let manager1 = WorldSeedManager::new(config1);
        let manager2 = WorldSeedManager::new(config2);

        let biome1 = manager1.get_chunk_biome(0, 0);
        let biome2 = manager2.get_chunk_biome(0, 0);

        assert_eq!(biome1, biome2);
    }
}
