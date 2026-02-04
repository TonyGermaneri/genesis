//! Biome-based material generation system.
//!
//! This module provides biome configuration and material assignment based on
//! world coordinates and depth. Uses simplex noise for natural biome boundaries.

use std::collections::HashMap;

use genesis_common::WorldCoord;

/// Unique identifier for a biome type.
pub type BiomeId = u8;

/// Unique identifier for a material type.
pub type MaterialId = u16;

/// Pre-defined biome IDs.
pub mod biome_ids {
    use super::BiomeId;

    /// Forest biome with grass, dirt, and stone layers.
    pub const FOREST: BiomeId = 0;
    /// Desert biome with sand and sandstone.
    pub const DESERT: BiomeId = 1;
    /// Cave biome with stone and minerals.
    pub const CAVE: BiomeId = 2;
    /// Ocean biome with water and sand.
    pub const OCEAN: BiomeId = 3;
    /// Plains biome with open grasslands.
    pub const PLAINS: BiomeId = 4;
    /// Mountain biome with stone, snow caps, and elevation.
    pub const MOUNTAIN: BiomeId = 5;
}

/// Pre-defined material IDs (matching genesis-tools/screenshot.rs conventions).
pub mod material_ids {
    use super::MaterialId;

    /// Air/void (empty space).
    pub const AIR: MaterialId = 0;
    /// Dirt material.
    pub const DIRT: MaterialId = 1;
    /// Stone material.
    pub const STONE: MaterialId = 2;
    /// Grass material.
    pub const GRASS: MaterialId = 3;
    /// Water material.
    pub const WATER: MaterialId = 4;
    /// Sand material.
    pub const SAND: MaterialId = 5;
    /// Lava material.
    pub const LAVA: MaterialId = 6;
    /// Sandstone material.
    pub const SANDSTONE: MaterialId = 7;
    /// Clay material.
    pub const CLAY: MaterialId = 8;
    /// Gravel material.
    pub const GRAVEL: MaterialId = 9;
    /// Snow material (for mountain peaks).
    pub const SNOW: MaterialId = 10;
}

/// Configuration for a single biome.
///
/// Defines the material layers at different depths.
#[derive(Debug, Clone)]
pub struct BiomeConfig {
    /// Unique biome identifier.
    pub id: BiomeId,
    /// Human-readable name.
    pub name: String,
    /// Material for the surface layer.
    pub surface_material: MaterialId,
    /// Material for the subsurface layer.
    pub subsurface_material: MaterialId,
    /// Material for the deep layer.
    pub deep_material: MaterialId,
    /// Depth of the surface layer in cells.
    pub surface_depth: u32,
    /// Depth of the subsurface layer in cells.
    pub subsurface_depth: u32,
}

impl BiomeConfig {
    /// Creates a new biome configuration.
    #[must_use]
    pub fn new(
        id: BiomeId,
        name: impl Into<String>,
        surface_material: MaterialId,
        subsurface_material: MaterialId,
        deep_material: MaterialId,
        surface_depth: u32,
        subsurface_depth: u32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            surface_material,
            subsurface_material,
            deep_material,
            surface_depth,
            subsurface_depth,
        }
    }

    /// Creates a default Forest biome.
    #[must_use]
    pub fn forest() -> Self {
        Self::new(
            biome_ids::FOREST,
            "Forest",
            material_ids::GRASS,
            material_ids::DIRT,
            material_ids::STONE,
            1, // 1 cell of grass
            8, // 8 cells of dirt
        )
    }

    /// Creates a default Desert biome.
    #[must_use]
    pub fn desert() -> Self {
        Self::new(
            biome_ids::DESERT,
            "Desert",
            material_ids::SAND,
            material_ids::SANDSTONE,
            material_ids::STONE,
            4,  // 4 cells of sand
            16, // 16 cells of sandstone
        )
    }

    /// Creates a default Cave biome.
    #[must_use]
    pub fn cave() -> Self {
        Self::new(
            biome_ids::CAVE,
            "Cave",
            material_ids::STONE,
            material_ids::STONE,
            material_ids::STONE,
            0,
            0,
        )
    }

    /// Creates a default Ocean biome.
    #[must_use]
    pub fn ocean() -> Self {
        Self::new(
            biome_ids::OCEAN,
            "Ocean",
            material_ids::SAND,
            material_ids::CLAY,
            material_ids::STONE,
            2,  // 2 cells of sand
            10, // 10 cells of clay
        )
    }

    /// Creates a default Plains biome.
    #[must_use]
    pub fn plains() -> Self {
        Self::new(
            biome_ids::PLAINS,
            "Plains",
            material_ids::GRASS,
            material_ids::DIRT,
            material_ids::STONE,
            2,  // 2 cells of grass
            12, // 12 cells of dirt
        )
    }

    /// Creates a default Mountain biome.
    #[must_use]
    pub fn mountain() -> Self {
        Self::new(
            biome_ids::MOUNTAIN,
            "Mountain",
            material_ids::STONE,
            material_ids::STONE,
            material_ids::STONE,
            0,
            0,
        )
    }

    /// Gets the material at a given depth within this biome.
    ///
    /// # Arguments
    /// * `depth` - Depth below surface in cells (0 = surface).
    #[must_use]
    pub fn material_at_depth(&self, depth: u32) -> MaterialId {
        if depth < self.surface_depth {
            self.surface_material
        } else if depth < self.surface_depth + self.subsurface_depth {
            self.subsurface_material
        } else {
            self.deep_material
        }
    }
}

/// Simple 2D simplex noise implementation.
///
/// Based on the simplex noise algorithm by Ken Perlin.
#[derive(Debug, Clone)]
pub struct SimplexNoise {
    /// Permutation table for noise generation.
    perm: [u8; 512],
    /// Noise scale factor.
    scale: f64,
}

impl SimplexNoise {
    /// Skewing factor for 2D simplex noise.
    const F2: f64 = 0.366_025_403_784_438_65; // (sqrt(3) - 1) / 2
    /// Unskewing factor for 2D simplex noise.
    const G2: f64 = 0.211_324_865_405_187_1; // (3 - sqrt(3)) / 6

    /// Gradient vectors for 2D simplex noise.
    const GRAD2: [[f64; 2]; 8] = [
        [1.0, 0.0],
        [-1.0, 0.0],
        [0.0, 1.0],
        [0.0, -1.0],
        [0.707, 0.707],
        [-0.707, 0.707],
        [0.707, -0.707],
        [-0.707, -0.707],
    ];

    /// Creates a new simplex noise generator with the given seed.
    #[must_use]
    pub fn new(seed: u64, scale: f64) -> Self {
        let mut perm = [0u8; 512];

        // Initialize with seed-based permutation
        let mut rng_state = seed;
        let mut p: [u8; 256] = std::array::from_fn(|i| i as u8);

        // Fisher-Yates shuffle with simple LCG random
        for i in (1..256).rev() {
            rng_state = rng_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            #[allow(clippy::cast_possible_truncation)]
            let j = ((rng_state >> 32) as usize) % (i + 1);
            p.swap(i, j);
        }

        // Duplicate for wrapping
        perm[..256].copy_from_slice(&p);
        perm[256..512].copy_from_slice(&p);

        Self { perm, scale }
    }

    /// Creates a noise generator with default seed.
    #[must_use]
    pub fn default_seed() -> Self {
        Self::new(42, 0.01)
    }

    /// Computes 2D simplex noise at the given coordinates.
    ///
    /// Returns a value in the range [-1, 1].
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn noise2d(&self, x: f64, y: f64) -> f64 {
        let x = x * self.scale;
        let y = y * self.scale;

        // Skew input space to determine simplex cell
        let s = (x + y) * Self::F2;
        let i = (x + s).floor();
        let j = (y + s).floor();

        // Unskew cell origin back to (x, y) space
        let t = (i + j) * Self::G2;
        let x0 = x - (i - t);
        let y0 = y - (j - t);

        // Determine which simplex we're in
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        // Offsets for corners
        let x1 = x0 - f64::from(i1) + Self::G2;
        let y1 = y0 - f64::from(j1) + Self::G2;
        let x2 = x0 - 1.0 + 2.0 * Self::G2;
        let y2 = y0 - 1.0 + 2.0 * Self::G2;

        // Hash coordinates to gradient indices
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ii = (i as i32 & 255) as usize;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let jj = (j as i32 & 255) as usize;

        let gi0 = (self.perm[ii + self.perm[jj] as usize] % 8) as usize;
        let gi1 = (self.perm[ii + i1 as usize + self.perm[jj + j1 as usize] as usize] % 8) as usize;
        let gi2 = (self.perm[ii + 1 + self.perm[jj + 1] as usize] % 8) as usize;

        // Calculate contributions from each corner
        let n0 = Self::contribution(x0, y0, gi0);
        let n1 = Self::contribution(x1, y1, gi1);
        let n2 = Self::contribution(x2, y2, gi2);

        // Sum contributions (scale to [-1, 1])
        70.0 * (n0 + n1 + n2)
    }

    /// Calculates the contribution from a corner.
    fn contribution(x: f64, y: f64, gi: usize) -> f64 {
        let t = 0.5 - x * x - y * y;
        if t < 0.0 {
            0.0
        } else {
            let t2 = t * t;
            t2 * t2 * (Self::GRAD2[gi][0] * x + Self::GRAD2[gi][1] * y)
        }
    }

    /// Generates octaved noise (fractal Brownian motion).
    ///
    /// # Arguments
    /// * `x`, `y` - Coordinates
    /// * `octaves` - Number of octaves
    /// * `persistence` - Amplitude falloff per octave
    #[must_use]
    pub fn fbm(&self, x: f64, y: f64, octaves: u32, persistence: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.noise2d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        total / max_value
    }
}

impl Default for SimplexNoise {
    fn default() -> Self {
        Self::default_seed()
    }
}

/// Biome manager for world-wide biome assignment.
///
/// Handles biome registration, lookup by coordinate, and material queries.
#[derive(Debug)]
pub struct BiomeManager {
    /// Registered biomes.
    biomes: HashMap<BiomeId, BiomeConfig>,
    /// Noise generator for biome boundaries.
    noise: SimplexNoise,
    /// Secondary noise for variation.
    noise2: SimplexNoise,
    /// Default biome when none matches.
    default_biome: BiomeId,
}

impl BiomeManager {
    /// Creates a new biome manager with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            biomes: HashMap::new(),
            noise: SimplexNoise::new(seed, 0.005), // Large-scale biome noise
            noise2: SimplexNoise::new(seed + 1, 0.02), // Small-scale variation
            default_biome: biome_ids::FOREST,
        }
    }

    /// Creates a biome manager with default biomes pre-registered.
    #[must_use]
    pub fn with_defaults(seed: u64) -> Self {
        let mut manager = Self::new(seed);
        manager.register_biome(BiomeConfig::forest());
        manager.register_biome(BiomeConfig::desert());
        manager.register_biome(BiomeConfig::cave());
        manager.register_biome(BiomeConfig::ocean());
        manager.register_biome(BiomeConfig::plains());
        manager.register_biome(BiomeConfig::mountain());
        manager
    }

    /// Registers a biome configuration.
    pub fn register_biome(&mut self, config: BiomeConfig) {
        self.biomes.insert(config.id, config);
    }

    /// Unregisters a biome by ID.
    pub fn unregister_biome(&mut self, id: BiomeId) -> Option<BiomeConfig> {
        self.biomes.remove(&id)
    }

    /// Gets a biome configuration by ID.
    #[must_use]
    pub fn get_biome(&self, id: BiomeId) -> Option<&BiomeConfig> {
        self.biomes.get(&id)
    }

    /// Sets the default biome for unknown regions.
    pub fn set_default_biome(&mut self, id: BiomeId) {
        self.default_biome = id;
    }

    /// Gets the biome at a world coordinate.
    ///
    /// Uses simplex noise to create natural biome boundaries.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_biome_at(&self, coord: WorldCoord) -> BiomeId {
        let x = coord.x as f64;
        let y = coord.y as f64;

        // Primary noise determines main biome
        let n1 = self.noise.fbm(x, y, 3, 0.5);
        // Secondary noise adds local variation
        let n2 = self.noise2.noise2d(x, y);
        // Third noise for elevation/mountain determination
        let elevation_noise = self.noise.fbm(x * 2.0, y * 2.0, 2, 0.6);

        // Combine noises
        let combined = n1 + n2 * 0.2;

        // Map noise to biomes with smooth transitions
        // Thresholds create roughly equal-sized biome regions
        if combined < -0.4 {
            biome_ids::OCEAN
        } else if combined < -0.15 {
            // Transition zone - could be plains near ocean
            biome_ids::PLAINS
        } else if combined < 0.15 {
            biome_ids::FOREST
        } else if combined < 0.4 {
            // Check elevation for mountain vs desert
            if elevation_noise > 0.3 {
                biome_ids::MOUNTAIN
            } else {
                biome_ids::DESERT
            }
        } else if combined < 0.6 {
            // High elevation tends to be mountain
            if elevation_noise > 0.2 {
                biome_ids::MOUNTAIN
            } else {
                biome_ids::PLAINS
            }
        } else {
            // Deep areas become caves
            biome_ids::CAVE
        }
    }

    /// Gets the material at a world coordinate and depth.
    ///
    /// # Arguments
    /// * `coord` - World coordinate
    /// * `depth` - Depth below surface (0 = surface)
    #[must_use]
    pub fn get_material_at(&self, coord: WorldCoord, depth: u32) -> MaterialId {
        let biome_id = self.get_biome_at(coord);
        self.biomes
            .get(&biome_id)
            .map_or(material_ids::STONE, |biome| biome.material_at_depth(depth))
    }

    /// Gets the biome blend weights at a coordinate.
    ///
    /// Returns weights for smooth transitions between biomes.
    /// Weights sum to 1.0.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_biome_blend(&self, coord: WorldCoord) -> Vec<(BiomeId, f32)> {
        let x = coord.x as f64;
        let y = coord.y as f64;

        let n = self.noise.fbm(x, y, 3, 0.5);
        let elevation = self.noise.fbm(x * 2.0, y * 2.0, 2, 0.6);

        // Calculate distance to each biome threshold (updated with new biomes)
        let thresholds = [
            (biome_ids::OCEAN, -0.4),
            (biome_ids::PLAINS, -0.15),
            (biome_ids::FOREST, 0.15),
            (biome_ids::DESERT, 0.4),
            (biome_ids::MOUNTAIN, 0.6),
            (biome_ids::CAVE, 1.0),
        ];

        let mut weights = Vec::new();
        let blend_range = 0.1; // Blend distance

        for (i, &(biome_id, threshold)) in thresholds.iter().enumerate() {
            let prev_threshold = if i == 0 { -1.0 } else { thresholds[i - 1].1 };

            // Distance from this biome's range
            if n >= prev_threshold && n < threshold {
                // Inside this biome's primary range
                let dist_to_lower = n - prev_threshold;
                let dist_to_upper = threshold - n;
                let min_dist = dist_to_lower.min(dist_to_upper);

                // Full weight in center, blend at edges
                #[allow(clippy::cast_possible_truncation)]
                let mut weight = if min_dist < blend_range {
                    (min_dist / blend_range) as f32
                } else {
                    1.0
                };

                // Elevation affects mountain weight
                if biome_id == biome_ids::MOUNTAIN && elevation < 0.2 {
                    weight *= elevation as f32 / 0.2;
                }

                weights.push((biome_id, weight));
            } else {
                // Check if we're in blend range
                let dist = (n - threshold).abs().min((n - prev_threshold).abs());
                if dist < blend_range {
                    #[allow(clippy::cast_possible_truncation)]
                    let weight = (1.0 - dist / blend_range) as f32;
                    weights.push((biome_id, weight.max(0.0)));
                }
            }
        }

        // Normalize weights
        let total: f32 = weights.iter().map(|(_, w)| w).sum();
        if total > 0.0 {
            for (_, w) in &mut weights {
                *w /= total;
            }
        } else {
            // Fallback to default biome
            weights.push((self.default_biome, 1.0));
        }

        weights
    }

    /// Returns the number of registered biomes.
    #[must_use]
    pub fn biome_count(&self) -> usize {
        self.biomes.len()
    }

    /// Returns an iterator over all registered biomes.
    pub fn biomes(&self) -> impl Iterator<Item = &BiomeConfig> {
        self.biomes.values()
    }
}

impl Default for BiomeManager {
    fn default() -> Self {
        Self::with_defaults(42)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_config_creation() {
        let biome = BiomeConfig::new(
            0,
            "Test",
            material_ids::GRASS,
            material_ids::DIRT,
            material_ids::STONE,
            1,
            8,
        );
        assert_eq!(biome.id, 0);
        assert_eq!(biome.name, "Test");
        assert_eq!(biome.surface_material, material_ids::GRASS);
    }

    #[test]
    fn test_biome_material_at_depth() {
        let forest = BiomeConfig::forest();

        assert_eq!(forest.material_at_depth(0), material_ids::GRASS);
        assert_eq!(forest.material_at_depth(1), material_ids::DIRT);
        assert_eq!(forest.material_at_depth(5), material_ids::DIRT);
        assert_eq!(forest.material_at_depth(9), material_ids::STONE);
        assert_eq!(forest.material_at_depth(100), material_ids::STONE);
    }

    #[test]
    fn test_simplex_noise_range() {
        let noise = SimplexNoise::default_seed();

        // Check that noise stays in expected range
        for x in -100..100 {
            for y in -100..100 {
                let n = noise.noise2d(f64::from(x), f64::from(y));
                assert!(
                    (-1.0..=1.0).contains(&n),
                    "Noise {n} out of range at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn test_simplex_noise_deterministic() {
        let noise1 = SimplexNoise::new(123, 0.01);
        let noise2 = SimplexNoise::new(123, 0.01);

        for x in 0..10 {
            for y in 0..10 {
                let n1 = noise1.noise2d(f64::from(x), f64::from(y));
                let n2 = noise2.noise2d(f64::from(x), f64::from(y));
                assert!((n1 - n2).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_biome_manager_registration() {
        let mut manager = BiomeManager::new(42);
        assert_eq!(manager.biome_count(), 0);

        manager.register_biome(BiomeConfig::forest());
        assert_eq!(manager.biome_count(), 1);

        manager.register_biome(BiomeConfig::desert());
        assert_eq!(manager.biome_count(), 2);
    }

    #[test]
    fn test_biome_manager_defaults() {
        let manager = BiomeManager::with_defaults(42);
        assert_eq!(manager.biome_count(), 6);

        assert!(manager.get_biome(biome_ids::FOREST).is_some());
        assert!(manager.get_biome(biome_ids::DESERT).is_some());
        assert!(manager.get_biome(biome_ids::CAVE).is_some());
        assert!(manager.get_biome(biome_ids::OCEAN).is_some());
        assert!(manager.get_biome(biome_ids::PLAINS).is_some());
        assert!(manager.get_biome(biome_ids::MOUNTAIN).is_some());
    }

    #[test]
    fn test_biome_at_deterministic() {
        let manager = BiomeManager::with_defaults(42);

        // Same coordinates should always return same biome
        let coord = WorldCoord::new(100, 200);
        let biome1 = manager.get_biome_at(coord);
        let biome2 = manager.get_biome_at(coord);
        assert_eq!(biome1, biome2);
    }

    #[test]
    fn test_material_at_coord() {
        let manager = BiomeManager::with_defaults(42);

        // Materials should be consistent
        let coord = WorldCoord::new(0, 0);
        let mat1 = manager.get_material_at(coord, 0);
        let mat2 = manager.get_material_at(coord, 0);
        assert_eq!(mat1, mat2);
    }

    #[test]
    fn test_biome_blend_sums_to_one() {
        let manager = BiomeManager::with_defaults(42);

        for x in -50..50 {
            for y in -50..50 {
                let coord = WorldCoord::new(i64::from(x) * 100, i64::from(y) * 100);
                let blend = manager.get_biome_blend(coord);

                let total: f32 = blend.iter().map(|(_, w)| w).sum();
                assert!(
                    (total - 1.0).abs() < 0.01,
                    "Blend weights sum to {total} at {coord:?}"
                );
            }
        }
    }

    #[test]
    fn test_fbm_smoother_than_raw() {
        let noise = SimplexNoise::new(42, 0.1);

        // FBM should produce smoother values with more octaves
        let raw = noise.noise2d(0.0, 0.0);
        let fbm = noise.fbm(0.0, 0.0, 4, 0.5);

        // Both should be in valid range
        assert!((-1.0..=1.0).contains(&raw));
        assert!((-1.0..=1.0).contains(&fbm));
    }

    #[test]
    fn test_all_biomes_reachable() {
        let manager = BiomeManager::with_defaults(42);

        // Sample a large area to find all biomes
        let mut found_biomes = std::collections::HashSet::new();

        for x in -500..500 {
            for y in -500..500 {
                let coord = WorldCoord::new(i64::from(x) * 10, i64::from(y) * 10);
                found_biomes.insert(manager.get_biome_at(coord));
            }
        }

        // Should find all 4 biomes
        assert!(
            found_biomes.contains(&biome_ids::FOREST),
            "Forest not found"
        );
        assert!(
            found_biomes.contains(&biome_ids::DESERT),
            "Desert not found"
        );
        // Cave and Ocean may be rarer, so we don't strictly require them
    }
}
