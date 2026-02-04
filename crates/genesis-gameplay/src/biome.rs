//! Biome system for terrain generation and resource distribution.
//!
//! This module provides:
//! - Biome type definitions with environmental properties
//! - Procedural terrain generation using noise functions
//! - Resource distribution rules per biome
//! - Biome-specific cell material variants

use genesis_common::WorldCoord;
use serde::{Deserialize, Serialize};

// ============================================================================
// G-33: Biome Type Definitions
// ============================================================================

/// Types of biomes in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BiomeType {
    /// Dense tree coverage, moderate humidity.
    #[default]
    Forest,
    /// Hot and dry, sand and cacti.
    Desert,
    /// Open water body.
    Lake,
    /// Open grassland with sparse trees.
    Plains,
    /// High elevation, rocky terrain.
    Mountain,
    /// Wet, low-lying area with murky water.
    Swamp,
}

impl BiomeType {
    /// Get the display name for this biome.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Forest => "Forest",
            Self::Desert => "Desert",
            Self::Lake => "Lake",
            Self::Plains => "Plains",
            Self::Mountain => "Mountain",
            Self::Swamp => "Swamp",
        }
    }

    /// Get default properties for this biome.
    #[must_use]
    pub fn default_properties(self) -> BiomeProperties {
        match self {
            Self::Forest => BiomeProperties {
                temperature: 0.3,
                humidity: 0.7,
                vegetation_density: 0.8,
                elevation_min: 0.1,
                elevation_max: 0.5,
            },
            Self::Desert => BiomeProperties {
                temperature: 0.9,
                humidity: 0.1,
                vegetation_density: 0.1,
                elevation_min: 0.1,
                elevation_max: 0.4,
            },
            Self::Lake => BiomeProperties {
                temperature: 0.4,
                humidity: 1.0,
                vegetation_density: 0.2,
                elevation_min: -0.5,
                elevation_max: 0.0,
            },
            Self::Plains => BiomeProperties {
                temperature: 0.5,
                humidity: 0.4,
                vegetation_density: 0.4,
                elevation_min: 0.0,
                elevation_max: 0.3,
            },
            Self::Mountain => BiomeProperties {
                temperature: -0.3,
                humidity: 0.3,
                vegetation_density: 0.2,
                elevation_min: 0.6,
                elevation_max: 1.0,
            },
            Self::Swamp => BiomeProperties {
                temperature: 0.4,
                humidity: 0.9,
                vegetation_density: 0.6,
                elevation_min: -0.1,
                elevation_max: 0.1,
            },
        }
    }

    /// Get all biome types.
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Forest,
            Self::Desert,
            Self::Lake,
            Self::Plains,
            Self::Mountain,
            Self::Swamp,
        ]
    }

    /// Check if this biome has water.
    #[must_use]
    pub fn has_water(self) -> bool {
        matches!(self, Self::Lake | Self::Swamp)
    }

    /// Check if this biome is hospitable for farming.
    #[must_use]
    pub fn is_farmable(self) -> bool {
        matches!(self, Self::Forest | Self::Plains | Self::Swamp)
    }

    /// Get the base movement speed modifier for this biome.
    #[must_use]
    pub fn movement_modifier(self) -> f32 {
        match self {
            Self::Forest => 0.9,
            Self::Desert => 0.85,
            Self::Lake => 0.5, // Swimming
            Self::Plains => 1.0,
            Self::Mountain => 0.7,
            Self::Swamp => 0.6,
        }
    }
}

/// Environmental properties of a biome.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BiomeProperties {
    /// Temperature from -1.0 (cold) to 1.0 (hot).
    pub temperature: f32,
    /// Humidity from 0.0 (dry) to 1.0 (wet).
    pub humidity: f32,
    /// Vegetation density from 0.0 to 1.0.
    pub vegetation_density: f32,
    /// Minimum elevation for this biome.
    pub elevation_min: f32,
    /// Maximum elevation for this biome.
    pub elevation_max: f32,
}

impl BiomeProperties {
    /// Create new biome properties.
    #[must_use]
    pub fn new(
        temperature: f32,
        humidity: f32,
        vegetation_density: f32,
        elevation_min: f32,
        elevation_max: f32,
    ) -> Self {
        Self {
            temperature: temperature.clamp(-1.0, 1.0),
            humidity: humidity.clamp(0.0, 1.0),
            vegetation_density: vegetation_density.clamp(0.0, 1.0),
            elevation_min,
            elevation_max,
        }
    }

    /// Check if elevation is within this biome's range.
    #[must_use]
    pub fn elevation_in_range(&self, elevation: f32) -> bool {
        elevation >= self.elevation_min && elevation <= self.elevation_max
    }

    /// Get comfort level for entities (0.0 = hostile, 1.0 = comfortable).
    #[must_use]
    pub fn comfort_level(&self) -> f32 {
        // Moderate temperature and humidity are most comfortable
        let temp_comfort = 1.0 - self.temperature.abs();
        let humid_comfort = 1.0 - (self.humidity - 0.5).abs() * 2.0;
        (temp_comfort + humid_comfort) / 2.0
    }
}

impl Default for BiomeProperties {
    fn default() -> Self {
        BiomeType::Plains.default_properties()
    }
}

// ============================================================================
// G-34: Terrain Generation Logic
// ============================================================================

/// Simple LCG-based random number generator for deterministic noise.
#[derive(Debug, Clone)]
struct NoiseRng {
    state: u64,
}

impl NoiseRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }
}

/// Simple 2D noise implementation for terrain generation.
#[derive(Debug, Clone)]
pub struct SimplexNoise {
    permutation: [u8; 512],
    scale: f32,
}

impl SimplexNoise {
    /// Create a new simplex noise generator with the given seed.
    #[must_use]
    pub fn new(seed: u64, scale: f32) -> Self {
        let mut perm = [0u8; 256];
        for (i, p) in perm.iter_mut().enumerate() {
            *p = i as u8;
        }

        // Fisher-Yates shuffle
        let mut rng = NoiseRng::new(seed);
        for i in (1..256).rev() {
            let j = (rng.next() % (i as u64 + 1)) as usize;
            perm.swap(i, j);
        }

        let mut permutation = [0u8; 512];
        for i in 0..512 {
            permutation[i] = perm[i % 256];
        }

        Self { permutation, scale }
    }

    /// Get noise value at a position (returns -1.0 to 1.0).
    #[must_use]
    pub fn get(&self, x: f32, y: f32) -> f32 {
        let x = x * self.scale;
        let y = y * self.scale;

        // Simplified 2D noise using gradient interpolation
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        // Smoothstep interpolation
        let sx = fx * fx * (3.0 - 2.0 * fx);
        let sy = fy * fy * (3.0 - 2.0 * fy);

        // Get gradient values at corners
        let n00 = self.gradient(x0, y0, fx, fy);
        let n10 = self.gradient(x1, y0, fx - 1.0, fy);
        let n01 = self.gradient(x0, y1, fx, fy - 1.0);
        let n11 = self.gradient(x1, y1, fx - 1.0, fy - 1.0);

        // Bilinear interpolation
        let nx0 = n00 + sx * (n10 - n00);
        let nx1 = n01 + sx * (n11 - n01);
        nx0 + sy * (nx1 - nx0)
    }

    /// Get fractal/octave noise (more natural-looking).
    #[must_use]
    pub fn get_fractal(&self, x: f32, y: f32, octaves: u32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            value += self.get(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        value / max_value
    }

    fn gradient(&self, ix: i32, iy: i32, dx: f32, dy: f32) -> f32 {
        let h = self.hash(ix, iy) & 7;
        let (gx, gy) = match h {
            0 => (1.0, 0.0),
            1 => (-1.0, 0.0),
            2 => (0.0, 1.0),
            3 => (0.0, -1.0),
            4 => (0.707, 0.707),
            5 => (-0.707, 0.707),
            6 => (0.707, -0.707),
            _ => (-0.707, -0.707),
        };
        gx * dx + gy * dy
    }

    fn hash(&self, x: i32, y: i32) -> u8 {
        let x = (x & 255) as usize;
        let y = (y & 255) as usize;
        self.permutation[self.permutation[x] as usize ^ y]
    }
}

/// Terrain generator using noise-based biome assignment.
#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    seed: u64,
    temperature_noise: SimplexNoise,
    humidity_noise: SimplexNoise,
    elevation_noise: SimplexNoise,
    detail_noise: SimplexNoise,
}

impl TerrainGenerator {
    /// Create a new terrain generator with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            temperature_noise: SimplexNoise::new(seed, 0.005),
            humidity_noise: SimplexNoise::new(seed.wrapping_add(1000), 0.007),
            elevation_noise: SimplexNoise::new(seed.wrapping_add(2000), 0.003),
            detail_noise: SimplexNoise::new(seed.wrapping_add(3000), 0.02),
        }
    }

    /// Get the seed used for generation.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Get the biome at a world position.
    #[must_use]
    pub fn get_biome_at(&self, world_x: i32, world_y: i32) -> BiomeType {
        let temperature = self.get_temperature_at(world_x, world_y);
        let humidity = self.get_humidity_at(world_x, world_y);
        let elevation = self.get_elevation_at(world_x, world_y);

        Self::classify_biome(temperature, humidity, elevation)
    }

    /// Get the biome at a world coordinate.
    #[must_use]
    pub fn get_biome_at_coord(&self, coord: WorldCoord) -> BiomeType {
        self.get_biome_at(coord.x as i32, coord.y as i32)
    }

    /// Get temperature at a position (-1.0 to 1.0).
    #[must_use]
    pub fn get_temperature_at(&self, world_x: i32, world_y: i32) -> f32 {
        let x = world_x as f32;
        let y = world_y as f32;
        self.temperature_noise.get_fractal(x, y, 3)
    }

    /// Get humidity at a position (0.0 to 1.0).
    #[must_use]
    pub fn get_humidity_at(&self, world_x: i32, world_y: i32) -> f32 {
        let x = world_x as f32;
        let y = world_y as f32;
        (self.humidity_noise.get_fractal(x, y, 3) + 1.0) / 2.0
    }

    /// Get elevation at a position (-1.0 to 1.0).
    #[must_use]
    pub fn get_elevation_at(&self, world_x: i32, world_y: i32) -> f32 {
        let x = world_x as f32;
        let y = world_y as f32;
        self.elevation_noise.get_fractal(x, y, 4)
    }

    /// Get detail noise for resource placement (0.0 to 1.0).
    #[must_use]
    pub fn get_detail_at(&self, world_x: i32, world_y: i32) -> f32 {
        let x = world_x as f32;
        let y = world_y as f32;
        (self.detail_noise.get(x, y) + 1.0) / 2.0
    }

    /// Classify a biome based on temperature, humidity, and elevation.
    fn classify_biome(temperature: f32, humidity: f32, elevation: f32) -> BiomeType {
        // Very low elevation = Lake
        if elevation < -0.3 {
            return BiomeType::Lake;
        }

        // High elevation = Mountain
        if elevation > 0.5 {
            return BiomeType::Mountain;
        }

        // Hot and dry = Desert
        if temperature > 0.4 && humidity < 0.3 {
            return BiomeType::Desert;
        }

        // Cool/moderate and very wet = Swamp (low elevation)
        if humidity > 0.7 && elevation < 0.1 && temperature < 0.5 {
            return BiomeType::Swamp;
        }

        // Warm and humid = Forest
        if temperature > 0.0 && humidity > 0.5 {
            return BiomeType::Forest;
        }

        // Default = Plains
        BiomeType::Plains
    }

    /// Get biome properties at a position (interpolated).
    #[must_use]
    pub fn get_properties_at(&self, world_x: i32, world_y: i32) -> BiomeProperties {
        let temperature = self.get_temperature_at(world_x, world_y);
        let humidity = self.get_humidity_at(world_x, world_y);
        let elevation = self.get_elevation_at(world_x, world_y);

        // Get base properties from biome
        let biome = Self::classify_biome(temperature, humidity, elevation);
        let base = biome.default_properties();

        // Return actual measured values for more variation
        BiomeProperties {
            temperature,
            humidity,
            vegetation_density: base.vegetation_density * humidity,
            elevation_min: elevation - 0.1,
            elevation_max: elevation + 0.1,
        }
    }

    /// Get blended biome influence (for smooth transitions).
    /// Returns weights for each biome type at this position.
    #[must_use]
    pub fn get_biome_blend(&self, world_x: i32, world_y: i32) -> BiomeBlend {
        let primary = self.get_biome_at(world_x, world_y);

        // Sample neighbors for blending
        let neighbors = [
            self.get_biome_at(world_x - 8, world_y),
            self.get_biome_at(world_x + 8, world_y),
            self.get_biome_at(world_x, world_y - 8),
            self.get_biome_at(world_x, world_y + 8),
        ];

        // Count biome occurrences
        let mut weights = [0.0f32; 6];
        weights[primary as usize] = 2.0; // Primary has more weight

        for n in neighbors {
            weights[n as usize] += 1.0;
        }

        // Normalize
        let total: f32 = weights.iter().sum();
        for w in &mut weights {
            *w /= total;
        }

        BiomeBlend {
            primary,
            weights,
            is_border: neighbors.iter().any(|&n| n != primary),
        }
    }
}

/// Blended biome information for smooth transitions.
#[derive(Debug, Clone, Copy)]
pub struct BiomeBlend {
    /// The primary biome at this position.
    pub primary: BiomeType,
    /// Weight of each biome type (indexed by BiomeType as usize).
    pub weights: [f32; 6],
    /// Whether this position is at a biome border.
    pub is_border: bool,
}

impl BiomeBlend {
    /// Get the weight for a specific biome.
    #[must_use]
    pub fn weight(&self, biome: BiomeType) -> f32 {
        self.weights[biome as usize]
    }

    /// Get the secondary biome (highest weight after primary).
    #[must_use]
    pub fn secondary(&self) -> Option<BiomeType> {
        if !self.is_border {
            return None;
        }

        let mut max_weight = 0.0;
        let mut secondary = None;

        for (i, &w) in self.weights.iter().enumerate() {
            if i != self.primary as usize && w > max_weight {
                max_weight = w;
                secondary = Some(match i {
                    0 => BiomeType::Forest,
                    1 => BiomeType::Desert,
                    2 => BiomeType::Lake,
                    3 => BiomeType::Plains,
                    4 => BiomeType::Mountain,
                    _ => BiomeType::Swamp,
                });
            }
        }

        secondary
    }
}

// ============================================================================
// G-35: Biome Resource Distribution
// ============================================================================

/// Types of resources that can spawn in biomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Oak, pine, etc.
    Tree,
    /// Desert cactus.
    Cactus,
    /// Stone, ore.
    Rock,
    /// Berry bush, shrub.
    Bush,
    /// Water plant.
    Reed,
    /// Water creature.
    Fish,
    /// Grass patch (decorative).
    Grass,
    /// Dead/withered tree.
    DeadTree,
    /// Snow patch.
    Snow,
    /// Flower.
    Flower,
}

impl ResourceType {
    /// Get display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Tree => "Tree",
            Self::Cactus => "Cactus",
            Self::Rock => "Rock",
            Self::Bush => "Bush",
            Self::Reed => "Reed",
            Self::Fish => "Fish",
            Self::Grass => "Grass",
            Self::DeadTree => "Dead Tree",
            Self::Snow => "Snow",
            Self::Flower => "Flower",
        }
    }

    /// Check if this resource blocks movement.
    #[must_use]
    pub fn blocks_movement(self) -> bool {
        matches!(
            self,
            Self::Tree | Self::Cactus | Self::Rock | Self::DeadTree
        )
    }

    /// Check if this resource can be harvested.
    #[must_use]
    pub fn is_harvestable(self) -> bool {
        matches!(
            self,
            Self::Tree | Self::Cactus | Self::Rock | Self::Bush | Self::Reed
        )
    }
}

/// Spawn probability for a resource in a biome.
#[derive(Debug, Clone, Copy)]
pub struct SpawnRule {
    /// Base probability (0.0 to 1.0).
    pub probability: f32,
    /// Minimum cluster size.
    pub min_cluster: u32,
    /// Maximum cluster size.
    pub max_cluster: u32,
    /// Spacing between clusters.
    pub cluster_spacing: u32,
}

impl SpawnRule {
    /// Create a new spawn rule.
    #[must_use]
    pub const fn new(probability: f32, min_cluster: u32, max_cluster: u32, spacing: u32) -> Self {
        Self {
            probability,
            min_cluster,
            max_cluster,
            cluster_spacing: spacing,
        }
    }

    /// No spawning.
    pub const NONE: Self = Self::new(0.0, 0, 0, 0);
    /// Very rare spawning.
    pub const RARE: Self = Self::new(0.02, 1, 2, 32);
    /// Low density.
    pub const LOW: Self = Self::new(0.05, 1, 3, 16);
    /// Medium density.
    pub const MEDIUM: Self = Self::new(0.15, 2, 5, 12);
    /// High density.
    pub const HIGH: Self = Self::new(0.3, 3, 8, 8);
    /// Very high density.
    pub const DENSE: Self = Self::new(0.5, 4, 12, 6);
}

/// Resource distribution rules for a biome.
#[derive(Debug, Clone)]
pub struct BiomeResources {
    /// Biome this applies to.
    pub biome: BiomeType,
    /// Spawn rules for each resource type.
    rules: [(ResourceType, SpawnRule); 10],
}

impl BiomeResources {
    /// Get default resources for a biome.
    #[must_use]
    pub fn for_biome(biome: BiomeType) -> Self {
        let rules = match biome {
            BiomeType::Forest => [
                (ResourceType::Tree, SpawnRule::HIGH),
                (ResourceType::Bush, SpawnRule::MEDIUM),
                (ResourceType::Rock, SpawnRule::LOW),
                (ResourceType::Grass, SpawnRule::DENSE),
                (ResourceType::Flower, SpawnRule::LOW),
                (ResourceType::Cactus, SpawnRule::NONE),
                (ResourceType::Reed, SpawnRule::NONE),
                (ResourceType::Fish, SpawnRule::NONE),
                (ResourceType::DeadTree, SpawnRule::RARE),
                (ResourceType::Snow, SpawnRule::NONE),
            ],
            BiomeType::Desert => [
                (ResourceType::Cactus, SpawnRule::MEDIUM),
                (ResourceType::Rock, SpawnRule::HIGH),
                (ResourceType::Tree, SpawnRule::NONE),
                (ResourceType::Bush, SpawnRule::RARE),
                (ResourceType::Grass, SpawnRule::RARE),
                (ResourceType::Flower, SpawnRule::NONE),
                (ResourceType::Reed, SpawnRule::NONE),
                (ResourceType::Fish, SpawnRule::NONE),
                (ResourceType::DeadTree, SpawnRule::LOW),
                (ResourceType::Snow, SpawnRule::NONE),
            ],
            BiomeType::Lake => [
                (ResourceType::Fish, SpawnRule::MEDIUM),
                (ResourceType::Reed, SpawnRule::MEDIUM),
                (ResourceType::Tree, SpawnRule::NONE),
                (ResourceType::Bush, SpawnRule::NONE),
                (ResourceType::Rock, SpawnRule::LOW),
                (ResourceType::Grass, SpawnRule::NONE),
                (ResourceType::Flower, SpawnRule::NONE),
                (ResourceType::Cactus, SpawnRule::NONE),
                (ResourceType::DeadTree, SpawnRule::NONE),
                (ResourceType::Snow, SpawnRule::NONE),
            ],
            BiomeType::Plains => [
                (ResourceType::Grass, SpawnRule::DENSE),
                (ResourceType::Flower, SpawnRule::MEDIUM),
                (ResourceType::Tree, SpawnRule::RARE),
                (ResourceType::Bush, SpawnRule::LOW),
                (ResourceType::Rock, SpawnRule::RARE),
                (ResourceType::Cactus, SpawnRule::NONE),
                (ResourceType::Reed, SpawnRule::NONE),
                (ResourceType::Fish, SpawnRule::NONE),
                (ResourceType::DeadTree, SpawnRule::NONE),
                (ResourceType::Snow, SpawnRule::NONE),
            ],
            BiomeType::Mountain => [
                (ResourceType::Rock, SpawnRule::DENSE),
                (ResourceType::Snow, SpawnRule::HIGH),
                (ResourceType::Tree, SpawnRule::LOW),
                (ResourceType::Bush, SpawnRule::RARE),
                (ResourceType::Grass, SpawnRule::LOW),
                (ResourceType::Flower, SpawnRule::RARE),
                (ResourceType::Cactus, SpawnRule::NONE),
                (ResourceType::Reed, SpawnRule::NONE),
                (ResourceType::Fish, SpawnRule::NONE),
                (ResourceType::DeadTree, SpawnRule::RARE),
            ],
            BiomeType::Swamp => [
                (ResourceType::Reed, SpawnRule::HIGH),
                (ResourceType::DeadTree, SpawnRule::MEDIUM),
                (ResourceType::Grass, SpawnRule::MEDIUM),
                (ResourceType::Bush, SpawnRule::LOW),
                (ResourceType::Fish, SpawnRule::LOW),
                (ResourceType::Tree, SpawnRule::LOW),
                (ResourceType::Rock, SpawnRule::RARE),
                (ResourceType::Flower, SpawnRule::RARE),
                (ResourceType::Cactus, SpawnRule::NONE),
                (ResourceType::Snow, SpawnRule::NONE),
            ],
        };

        Self { biome, rules }
    }

    /// Get the spawn rule for a resource type.
    #[must_use]
    pub fn get_rule(&self, resource: ResourceType) -> SpawnRule {
        self.rules
            .iter()
            .find(|(r, _)| *r == resource)
            .map_or(SpawnRule::NONE, |(_, rule)| *rule)
    }

    /// Iterate over all resources with non-zero spawn rates.
    pub fn iter_spawnable(&self) -> impl Iterator<Item = (ResourceType, SpawnRule)> + '_ {
        self.rules
            .iter()
            .filter(|(_, rule)| rule.probability > 0.0)
            .copied()
    }
}

/// Resource spawner that uses terrain generator for placement.
#[derive(Debug, Clone)]
pub struct ResourceSpawner {
    generator: TerrainGenerator,
}

impl ResourceSpawner {
    /// Create a new resource spawner.
    #[must_use]
    pub fn new(generator: TerrainGenerator) -> Self {
        Self { generator }
    }

    /// Create with a seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self::new(TerrainGenerator::new(seed))
    }

    /// Check if a resource should spawn at this position.
    #[must_use]
    pub fn should_spawn(&self, world_x: i32, world_y: i32, resource: ResourceType) -> bool {
        let biome = self.generator.get_biome_at(world_x, world_y);
        let resources = BiomeResources::for_biome(biome);
        let rule = resources.get_rule(resource);

        if rule.probability <= 0.0 {
            return false;
        }

        // Use detail noise for natural clustering
        let detail = self.generator.get_detail_at(world_x, world_y);

        // Check spacing (simple grid-based clustering)
        #[allow(clippy::cast_possible_wrap)]
        let spacing = rule.cluster_spacing as i32;
        if spacing > 0 {
            let grid_x = world_x / spacing;
            let grid_y = world_y / spacing;
            let hash = self.position_hash(grid_x, grid_y, resource as u64);

            // Only spawn near cluster centers
            #[allow(clippy::cast_possible_wrap)]
            let cluster_x = (grid_x * spacing) + (hash % spacing as u64) as i32;
            #[allow(clippy::cast_possible_wrap)]
            let cluster_y = (grid_y * spacing) + ((hash / spacing as u64) % spacing as u64) as i32;
            let dist_sq = (world_x - cluster_x).pow(2) + (world_y - cluster_y).pow(2);
            #[allow(clippy::cast_possible_wrap)]
            let max_dist = (rule.max_cluster as i32).pow(2);

            if dist_sq > max_dist {
                return false;
            }
        }

        detail < rule.probability
    }

    /// Get all resources that should spawn at this position.
    #[must_use]
    pub fn get_resources_at(&self, world_x: i32, world_y: i32) -> Vec<ResourceType> {
        let biome = self.generator.get_biome_at(world_x, world_y);
        let resources = BiomeResources::for_biome(biome);

        resources
            .iter_spawnable()
            .filter(|(resource, _)| self.should_spawn(world_x, world_y, *resource))
            .map(|(resource, _)| resource)
            .collect()
    }

    /// Get the primary resource at this position (if any).
    #[must_use]
    pub fn get_primary_resource(&self, world_x: i32, world_y: i32) -> Option<ResourceType> {
        // Return the first blocking resource, or the first decorative one
        let resources = self.get_resources_at(world_x, world_y);

        resources
            .iter()
            .find(|r| r.blocks_movement())
            .or_else(|| resources.first())
            .copied()
    }

    fn position_hash(&self, x: i32, y: i32, extra: u64) -> u64 {
        let mut state = self.generator.seed();
        state = state.wrapping_mul(31).wrapping_add(x as u64);
        state = state.wrapping_mul(31).wrapping_add(y as u64);
        state = state.wrapping_mul(31).wrapping_add(extra);
        state
    }
}

// ============================================================================
// G-36: Biome-Specific Cell Types
// ============================================================================

/// Material variant for biome-specific appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeMaterial {
    // Grass variants
    /// Lush forest grass.
    ForestGrass,
    /// Yellow plains grass.
    PlainsGrass,
    /// Swamp grass (brownish).
    SwampGrass,
    /// Mountain grass (sparse).
    MountainGrass,

    // Sand variants
    /// Desert sand (yellow).
    DesertSand,
    /// Beach sand (lighter).
    BeachSand,

    // Stone variants
    /// Mountain stone (grey).
    MountainStone,
    /// Cave stone (dark).
    CaveStone,
    /// River stone (smooth).
    RiverStone,

    // Dirt variants
    /// Forest dirt (dark).
    ForestDirt,
    /// Plains dirt (brown).
    PlainsDirt,
    /// Swamp mud.
    SwampMud,

    // Water variants
    /// Lake water (blue).
    LakeWater,
    /// Swamp water (murky).
    SwampWater,
    /// River water (clear).
    RiverWater,

    // Snow
    /// Mountain snow.
    Snow,
    /// Ice.
    Ice,
}

impl BiomeMaterial {
    /// Get the base material category.
    #[must_use]
    pub fn category(self) -> MaterialCategory {
        match self {
            Self::ForestGrass | Self::PlainsGrass | Self::SwampGrass | Self::MountainGrass => {
                MaterialCategory::Grass
            },
            Self::DesertSand | Self::BeachSand => MaterialCategory::Sand,
            Self::MountainStone | Self::CaveStone | Self::RiverStone => MaterialCategory::Stone,
            Self::ForestDirt | Self::PlainsDirt | Self::SwampMud => MaterialCategory::Dirt,
            Self::LakeWater | Self::SwampWater | Self::RiverWater => MaterialCategory::Water,
            Self::Snow | Self::Ice => MaterialCategory::Snow,
        }
    }

    /// Get the material ID for rendering.
    #[must_use]
    pub fn material_id(self) -> u8 {
        self as u8
    }

    /// Get appropriate grass material for a biome.
    #[must_use]
    pub fn grass_for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Forest => Self::ForestGrass,
            BiomeType::Swamp => Self::SwampGrass,
            BiomeType::Mountain => Self::MountainGrass,
            // Plains, Desert, and Lake use plains grass as fallback
            BiomeType::Plains | BiomeType::Desert | BiomeType::Lake => Self::PlainsGrass,
        }
    }

    /// Get appropriate ground material for a biome.
    #[must_use]
    pub fn ground_for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Forest => Self::ForestDirt,
            BiomeType::Desert => Self::DesertSand,
            BiomeType::Lake => Self::LakeWater,
            BiomeType::Plains => Self::PlainsDirt,
            BiomeType::Mountain => Self::MountainStone,
            BiomeType::Swamp => Self::SwampMud,
        }
    }

    /// Get appropriate water material for a biome.
    #[must_use]
    pub fn water_for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Swamp => Self::SwampWater,
            BiomeType::Lake => Self::LakeWater,
            _ => Self::RiverWater,
        }
    }
}

/// Material category for grouping similar materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialCategory {
    /// Grass types.
    Grass,
    /// Sand types.
    Sand,
    /// Stone types.
    Stone,
    /// Dirt types.
    Dirt,
    /// Water types.
    Water,
    /// Snow/ice types.
    Snow,
}

impl MaterialCategory {
    /// Check if this material is solid (walkable).
    #[must_use]
    pub fn is_solid(self) -> bool {
        !matches!(self, Self::Water)
    }

    /// Check if this material is diggable.
    #[must_use]
    pub fn is_diggable(self) -> bool {
        matches!(self, Self::Grass | Self::Sand | Self::Dirt | Self::Snow)
    }

    /// Get movement speed modifier.
    #[must_use]
    pub fn movement_modifier(self) -> f32 {
        match self {
            Self::Grass | Self::Stone => 1.0,
            Self::Sand => 0.85,
            Self::Dirt => 0.95,
            Self::Water => 0.5,
            Self::Snow => 0.75,
        }
    }
}

/// World generator that combines terrain and resources.
#[derive(Debug, Clone)]
pub struct WorldGenerator {
    terrain: TerrainGenerator,
    resources: ResourceSpawner,
}

impl WorldGenerator {
    /// Create a new world generator with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let terrain = TerrainGenerator::new(seed);
        let resources = ResourceSpawner::new(terrain.clone());
        Self { terrain, resources }
    }

    /// Get the seed.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.terrain.seed()
    }

    /// Get the terrain generator.
    #[must_use]
    pub fn terrain(&self) -> &TerrainGenerator {
        &self.terrain
    }

    /// Get the resource spawner.
    #[must_use]
    pub fn resources(&self) -> &ResourceSpawner {
        &self.resources
    }

    /// Generate cell data for a position.
    #[must_use]
    pub fn generate_cell(&self, world_x: i32, world_y: i32) -> GeneratedCell {
        let biome = self.terrain.get_biome_at(world_x, world_y);
        let elevation = self.terrain.get_elevation_at(world_x, world_y);
        let resource = self.resources.get_primary_resource(world_x, world_y);

        // Determine ground material
        let ground_material = if elevation > 0.7 {
            BiomeMaterial::Snow
        } else if biome == BiomeType::Lake {
            BiomeMaterial::LakeWater
        } else {
            BiomeMaterial::ground_for_biome(biome)
        };

        // Add grass layer if appropriate
        let surface_material = if ground_material.category().is_solid()
            && biome.default_properties().vegetation_density > 0.3
            && resource.is_none()
        {
            let detail = self.terrain.get_detail_at(world_x, world_y);
            if detail < biome.default_properties().vegetation_density {
                Some(BiomeMaterial::grass_for_biome(biome))
            } else {
                None
            }
        } else {
            None
        };

        GeneratedCell {
            biome,
            elevation,
            ground_material,
            surface_material,
            resource,
        }
    }

    /// Generate cells for a chunk.
    #[allow(clippy::cast_possible_wrap)]
    pub fn generate_chunk(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_size: u32,
    ) -> Vec<GeneratedCell> {
        let base_x = chunk_x * chunk_size as i32;
        let base_y = chunk_y * chunk_size as i32;

        let mut cells = Vec::with_capacity((chunk_size * chunk_size) as usize);
        for y in 0..chunk_size as i32 {
            for x in 0..chunk_size as i32 {
                cells.push(self.generate_cell(base_x + x, base_y + y));
            }
        }
        cells
    }
}

/// Generated cell data from world generator.
#[derive(Debug, Clone, Copy)]
pub struct GeneratedCell {
    /// The biome at this position.
    pub biome: BiomeType,
    /// Elevation (-1.0 to 1.0).
    pub elevation: f32,
    /// Ground material.
    pub ground_material: BiomeMaterial,
    /// Optional surface material (e.g., grass on dirt).
    pub surface_material: Option<BiomeMaterial>,
    /// Optional resource at this position.
    pub resource: Option<ResourceType>,
}

impl GeneratedCell {
    /// Check if this cell is walkable.
    #[must_use]
    pub fn is_walkable(&self) -> bool {
        self.ground_material.category().is_solid()
            && self.resource.map_or(true, |r| !r.blocks_movement())
    }

    /// Get the movement modifier for this cell.
    #[must_use]
    pub fn movement_modifier(&self) -> f32 {
        let base = self.ground_material.category().movement_modifier();
        let biome = self.biome.movement_modifier();
        base * biome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // G-33 Tests: Biome Type Definitions

    #[test]
    fn test_biome_type_display_names() {
        assert_eq!(BiomeType::Forest.display_name(), "Forest");
        assert_eq!(BiomeType::Desert.display_name(), "Desert");
        assert_eq!(BiomeType::Lake.display_name(), "Lake");
        assert_eq!(BiomeType::Plains.display_name(), "Plains");
        assert_eq!(BiomeType::Mountain.display_name(), "Mountain");
        assert_eq!(BiomeType::Swamp.display_name(), "Swamp");
    }

    #[test]
    fn test_biome_type_all() {
        let all = BiomeType::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&BiomeType::Forest));
        assert!(all.contains(&BiomeType::Swamp));
    }

    #[test]
    fn test_biome_type_properties() {
        assert!(BiomeType::Lake.has_water());
        assert!(BiomeType::Swamp.has_water());
        assert!(!BiomeType::Desert.has_water());

        assert!(BiomeType::Plains.is_farmable());
        assert!(!BiomeType::Mountain.is_farmable());
    }

    #[test]
    fn test_biome_movement_modifiers() {
        assert_eq!(BiomeType::Plains.movement_modifier(), 1.0);
        assert!(BiomeType::Swamp.movement_modifier() < 1.0);
        assert!(BiomeType::Lake.movement_modifier() < BiomeType::Swamp.movement_modifier());
    }

    #[test]
    fn test_biome_properties_creation() {
        let props = BiomeProperties::new(0.5, 0.7, 0.8, 0.0, 0.5);
        assert_eq!(props.temperature, 0.5);
        assert_eq!(props.humidity, 0.7);
        assert_eq!(props.vegetation_density, 0.8);
    }

    #[test]
    fn test_biome_properties_clamping() {
        let props = BiomeProperties::new(2.0, -0.5, 1.5, 0.0, 1.0);
        assert_eq!(props.temperature, 1.0);
        assert_eq!(props.humidity, 0.0);
        assert_eq!(props.vegetation_density, 1.0);
    }

    #[test]
    fn test_biome_properties_elevation_range() {
        let props = BiomeProperties::new(0.5, 0.5, 0.5, 0.2, 0.8);
        assert!(props.elevation_in_range(0.5));
        assert!(!props.elevation_in_range(0.1));
        assert!(!props.elevation_in_range(0.9));
    }

    #[test]
    fn test_biome_default_properties() {
        let forest = BiomeType::Forest.default_properties();
        assert!(forest.humidity > 0.5);
        assert!(forest.vegetation_density > 0.5);

        let desert = BiomeType::Desert.default_properties();
        assert!(desert.humidity < 0.3);
        assert!(desert.vegetation_density < 0.3);
    }

    // G-34 Tests: Terrain Generation

    #[test]
    fn test_simplex_noise_creation() {
        let noise = SimplexNoise::new(12345, 0.01);
        let val = noise.get(100.0, 100.0);
        assert!((-1.0..=1.0).contains(&val));
    }

    #[test]
    fn test_simplex_noise_deterministic() {
        let noise1 = SimplexNoise::new(12345, 0.01);
        let noise2 = SimplexNoise::new(12345, 0.01);

        assert_eq!(noise1.get(50.0, 50.0), noise2.get(50.0, 50.0));
    }

    #[test]
    fn test_simplex_noise_different_seeds() {
        let noise1 = SimplexNoise::new(12345, 0.01);
        let noise2 = SimplexNoise::new(54321, 0.01);

        // Different seeds should produce different values at most positions
        // Check multiple positions to ensure they differ somewhere
        let mut differs = false;
        for x in 0..10 {
            for y in 0..10 {
                let fx = x as f32 * 17.3;
                let fy = y as f32 * 23.7;
                if noise1.get(fx, fy) != noise2.get(fx, fy) {
                    differs = true;
                    break;
                }
            }
            if differs {
                break;
            }
        }
        assert!(
            differs,
            "Noise with different seeds should differ somewhere"
        );
    }

    #[test]
    fn test_simplex_noise_fractal() {
        let noise = SimplexNoise::new(12345, 0.01);
        let val = noise.get_fractal(100.0, 100.0, 4);
        assert!((-1.0..=1.0).contains(&val));
    }

    #[test]
    fn test_terrain_generator_creation() {
        let gen = TerrainGenerator::new(42);
        assert_eq!(gen.seed(), 42);
    }

    #[test]
    fn test_terrain_generator_deterministic() {
        let gen1 = TerrainGenerator::new(42);
        let gen2 = TerrainGenerator::new(42);

        assert_eq!(gen1.get_biome_at(100, 100), gen2.get_biome_at(100, 100));
        assert_eq!(
            gen1.get_elevation_at(100, 100),
            gen2.get_elevation_at(100, 100)
        );
    }

    #[test]
    fn test_terrain_generator_biome_variety() {
        let gen = TerrainGenerator::new(12345);

        // Sample many positions to find different biomes
        let mut found_biomes = std::collections::HashSet::new();
        for x in (-1000..1000).step_by(50) {
            for y in (-1000..1000).step_by(50) {
                found_biomes.insert(gen.get_biome_at(x, y));
            }
        }

        // Should find at least 3 different biomes
        assert!(found_biomes.len() >= 3);
    }

    #[test]
    fn test_terrain_generator_elevation_range() {
        let gen = TerrainGenerator::new(42);
        for x in 0..100 {
            for y in 0..100 {
                let elev = gen.get_elevation_at(x, y);
                assert!((-1.0..=1.0).contains(&elev));
            }
        }
    }

    #[test]
    fn test_terrain_generator_humidity_range() {
        let gen = TerrainGenerator::new(42);
        for x in 0..100 {
            for y in 0..100 {
                let humid = gen.get_humidity_at(x, y);
                assert!((0.0..=1.0).contains(&humid));
            }
        }
    }

    #[test]
    fn test_terrain_generator_coord_variant() {
        let gen = TerrainGenerator::new(42);
        let coord = WorldCoord::new(50, 75);

        assert_eq!(gen.get_biome_at_coord(coord), gen.get_biome_at(50, 75));
    }

    #[test]
    fn test_biome_blend() {
        let gen = TerrainGenerator::new(42);
        let blend = gen.get_biome_blend(100, 100);

        // Weights should sum to 1.0
        let total: f32 = blend.weights.iter().sum();
        assert!((total - 1.0).abs() < 0.001);

        // Primary biome should have positive weight
        assert!(blend.weight(blend.primary) > 0.0);
    }

    #[test]
    fn test_biome_blend_border_detection() {
        let gen = TerrainGenerator::new(12345);

        // Find a position near a biome border
        let mut found_border = false;
        for x in 0..500 {
            for y in 0..500 {
                let blend = gen.get_biome_blend(x, y);
                if blend.is_border {
                    found_border = true;
                    assert!(blend.secondary().is_some());
                    break;
                }
            }
            if found_border {
                break;
            }
        }
    }

    // G-35 Tests: Resource Distribution

    #[test]
    fn test_resource_type_properties() {
        assert!(ResourceType::Tree.blocks_movement());
        assert!(!ResourceType::Grass.blocks_movement());

        assert!(ResourceType::Rock.is_harvestable());
        assert!(!ResourceType::Fish.is_harvestable());
    }

    #[test]
    fn test_spawn_rule_constants() {
        assert_eq!(SpawnRule::NONE.probability, 0.0);
        assert!(SpawnRule::HIGH.probability > SpawnRule::LOW.probability);
        assert!(SpawnRule::DENSE.probability > SpawnRule::HIGH.probability);
    }

    #[test]
    fn test_biome_resources_forest() {
        let resources = BiomeResources::for_biome(BiomeType::Forest);
        assert_eq!(resources.biome, BiomeType::Forest);

        let tree_rule = resources.get_rule(ResourceType::Tree);
        assert!(tree_rule.probability > 0.2);

        let cactus_rule = resources.get_rule(ResourceType::Cactus);
        assert_eq!(cactus_rule.probability, 0.0);
    }

    #[test]
    fn test_biome_resources_desert() {
        let resources = BiomeResources::for_biome(BiomeType::Desert);

        let cactus_rule = resources.get_rule(ResourceType::Cactus);
        assert!(cactus_rule.probability > 0.0);

        let tree_rule = resources.get_rule(ResourceType::Tree);
        assert_eq!(tree_rule.probability, 0.0);
    }

    #[test]
    fn test_biome_resources_iteration() {
        let resources = BiomeResources::for_biome(BiomeType::Forest);
        let spawnable: Vec<_> = resources.iter_spawnable().collect();

        assert!(!spawnable.is_empty());
        assert!(spawnable.iter().all(|(_, rule)| rule.probability > 0.0));
    }

    #[test]
    fn test_resource_spawner_creation() {
        let spawner = ResourceSpawner::with_seed(42);
        // Should not panic
        let _ = spawner.should_spawn(0, 0, ResourceType::Tree);
    }

    #[test]
    fn test_resource_spawner_deterministic() {
        let spawner1 = ResourceSpawner::with_seed(42);
        let spawner2 = ResourceSpawner::with_seed(42);

        for x in 0..50 {
            for y in 0..50 {
                assert_eq!(
                    spawner1.should_spawn(x, y, ResourceType::Tree),
                    spawner2.should_spawn(x, y, ResourceType::Tree)
                );
            }
        }
    }

    #[test]
    fn test_resource_spawner_biome_appropriate() {
        let spawner = ResourceSpawner::with_seed(42);

        // Find a desert position
        let gen = TerrainGenerator::new(42);
        let mut desert_pos = None;
        for x in -500..500 {
            for y in -500..500 {
                if gen.get_biome_at(x, y) == BiomeType::Desert {
                    desert_pos = Some((x, y));
                    break;
                }
            }
            if desert_pos.is_some() {
                break;
            }
        }

        if let Some((x, y)) = desert_pos {
            // Trees shouldn't spawn in desert
            assert!(!spawner.should_spawn(x, y, ResourceType::Tree));
        }
    }

    // G-36 Tests: Biome-Specific Cell Types

    #[test]
    fn test_biome_material_category() {
        assert_eq!(
            BiomeMaterial::ForestGrass.category(),
            MaterialCategory::Grass
        );
        assert_eq!(BiomeMaterial::DesertSand.category(), MaterialCategory::Sand);
        assert_eq!(
            BiomeMaterial::MountainStone.category(),
            MaterialCategory::Stone
        );
        assert_eq!(BiomeMaterial::LakeWater.category(), MaterialCategory::Water);
    }

    #[test]
    fn test_biome_material_id() {
        // Each material should have a unique ID
        let materials = [
            BiomeMaterial::ForestGrass,
            BiomeMaterial::PlainsGrass,
            BiomeMaterial::DesertSand,
            BiomeMaterial::MountainStone,
        ];

        let ids: Vec<_> = materials.iter().map(|m| m.material_id()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn test_biome_material_for_biome() {
        assert_eq!(
            BiomeMaterial::grass_for_biome(BiomeType::Forest),
            BiomeMaterial::ForestGrass
        );
        assert_eq!(
            BiomeMaterial::grass_for_biome(BiomeType::Plains),
            BiomeMaterial::PlainsGrass
        );
        assert_eq!(
            BiomeMaterial::ground_for_biome(BiomeType::Desert),
            BiomeMaterial::DesertSand
        );
        assert_eq!(
            BiomeMaterial::water_for_biome(BiomeType::Swamp),
            BiomeMaterial::SwampWater
        );
    }

    #[test]
    fn test_material_category_properties() {
        assert!(MaterialCategory::Grass.is_solid());
        assert!(!MaterialCategory::Water.is_solid());

        assert!(MaterialCategory::Sand.is_diggable());
        assert!(!MaterialCategory::Stone.is_diggable());
    }

    #[test]
    fn test_material_category_movement() {
        assert_eq!(MaterialCategory::Grass.movement_modifier(), 1.0);
        assert!(MaterialCategory::Sand.movement_modifier() < 1.0);
        assert!(
            MaterialCategory::Water.movement_modifier()
                < MaterialCategory::Sand.movement_modifier()
        );
    }

    // World Generator Tests

    #[test]
    fn test_world_generator_creation() {
        let gen = WorldGenerator::new(42);
        assert_eq!(gen.seed(), 42);
    }

    #[test]
    fn test_world_generator_cell_generation() {
        let gen = WorldGenerator::new(42);
        let cell = gen.generate_cell(100, 100);

        // Cell should have valid data
        assert!((-1.0..=1.0).contains(&cell.elevation));
    }

    #[test]
    fn test_world_generator_deterministic() {
        let gen1 = WorldGenerator::new(42);
        let gen2 = WorldGenerator::new(42);

        let cell1 = gen1.generate_cell(100, 100);
        let cell2 = gen2.generate_cell(100, 100);

        assert_eq!(cell1.biome, cell2.biome);
        assert_eq!(cell1.elevation, cell2.elevation);
        assert_eq!(cell1.ground_material, cell2.ground_material);
    }

    #[test]
    fn test_world_generator_chunk() {
        let gen = WorldGenerator::new(42);
        let cells = gen.generate_chunk(0, 0, 16);

        assert_eq!(cells.len(), 256); // 16x16
    }

    #[test]
    fn test_generated_cell_walkability() {
        let cell = GeneratedCell {
            biome: BiomeType::Plains,
            elevation: 0.2,
            ground_material: BiomeMaterial::PlainsDirt,
            surface_material: Some(BiomeMaterial::PlainsGrass),
            resource: None,
        };
        assert!(cell.is_walkable());

        let water_cell = GeneratedCell {
            biome: BiomeType::Lake,
            elevation: -0.4,
            ground_material: BiomeMaterial::LakeWater,
            surface_material: None,
            resource: None,
        };
        assert!(!water_cell.is_walkable());
    }

    #[test]
    fn test_generated_cell_with_resource() {
        let cell_with_tree = GeneratedCell {
            biome: BiomeType::Forest,
            elevation: 0.2,
            ground_material: BiomeMaterial::ForestDirt,
            surface_material: Some(BiomeMaterial::ForestGrass),
            resource: Some(ResourceType::Tree),
        };
        assert!(!cell_with_tree.is_walkable()); // Tree blocks movement

        let cell_with_grass = GeneratedCell {
            biome: BiomeType::Plains,
            elevation: 0.1,
            ground_material: BiomeMaterial::PlainsDirt,
            surface_material: Some(BiomeMaterial::PlainsGrass),
            resource: Some(ResourceType::Grass),
        };
        assert!(cell_with_grass.is_walkable()); // Grass doesn't block
    }

    #[test]
    fn test_generated_cell_movement_modifier() {
        let plains = GeneratedCell {
            biome: BiomeType::Plains,
            elevation: 0.2,
            ground_material: BiomeMaterial::PlainsDirt,
            surface_material: None,
            resource: None,
        };

        let swamp = GeneratedCell {
            biome: BiomeType::Swamp,
            elevation: 0.0,
            ground_material: BiomeMaterial::SwampMud,
            surface_material: None,
            resource: None,
        };

        assert!(plains.movement_modifier() > swamp.movement_modifier());
    }
}
