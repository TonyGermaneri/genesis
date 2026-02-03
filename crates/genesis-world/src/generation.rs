//! Procedural world generation.

use genesis_common::ChunkCoord;
use genesis_kernel::Cell;
use noise::{NoiseFn, Perlin};

use crate::chunk::Chunk;

/// World generator configuration.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// World seed
    pub seed: u32,
    /// Chunk size in pixels
    pub chunk_size: u32,
    /// Terrain scale (larger = smoother)
    pub terrain_scale: f64,
    /// Height scale
    pub height_scale: f64,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            chunk_size: 256,
            terrain_scale: 100.0,
            height_scale: 50.0,
        }
    }
}

/// Procedural world generator.
pub struct WorldGenerator {
    /// Configuration
    config: GeneratorConfig,
    /// Terrain noise
    terrain_noise: Perlin,
    /// Detail noise
    detail_noise: Perlin,
}

impl WorldGenerator {
    /// Creates a new generator with the given config.
    #[must_use]
    pub fn new(config: GeneratorConfig) -> Self {
        let terrain_noise = Perlin::new(config.seed);
        let detail_noise = Perlin::new(config.seed.wrapping_add(1));

        Self {
            config,
            terrain_noise,
            detail_noise,
        }
    }

    /// Creates a generator with default config.
    #[must_use]
    pub fn with_seed(seed: u32) -> Self {
        Self::new(GeneratorConfig {
            seed,
            ..Default::default()
        })
    }

    /// Generates a chunk at the given coordinate.
    #[must_use]
    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord, self.config.chunk_size);
        let world_x = coord.x as f64 * self.config.chunk_size as f64;
        let world_y = coord.y as f64 * self.config.chunk_size as f64;

        let cells = chunk.cells_mut();

        for y in 0..self.config.chunk_size {
            for x in 0..self.config.chunk_size {
                let wx = (world_x + x as f64) / self.config.terrain_scale;
                let wy = (world_y + y as f64) / self.config.terrain_scale;

                // Sample terrain height
                let height = self.terrain_noise.get([wx, wy]);
                let detail = self.detail_noise.get([wx * 4.0, wy * 4.0]) * 0.1;
                let combined = (height + detail + 1.0) / 2.0; // Normalize to 0-1

                // Determine material based on height
                let material = Self::height_to_material(combined);

                let index = (y * self.config.chunk_size + x) as usize;
                cells[index] = Cell::new(material);
            }
        }

        chunk.mark_clean(); // Generated chunks start clean
        chunk
    }

    /// Converts a height value (0-1) to a material ID.
    #[must_use]
    fn height_to_material(height: f64) -> u16 {
        // Material IDs:
        // 0 = air/void
        // 1 = water
        // 2 = sand
        // 3 = grass
        // 4 = dirt
        // 5 = stone
        // 6 = snow
        match height {
            h if h < 0.3 => 1,  // water
            h if h < 0.35 => 2, // sand (beach)
            h if h < 0.6 => 3,  // grass
            h if h < 0.7 => 4,  // dirt
            h if h < 0.85 => 5, // stone
            _ => 6,             // snow (peaks)
        }
    }

    /// Returns the generator configuration.
    #[must_use]
    pub const fn config(&self) -> &GeneratorConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_deterministic() {
        let gen1 = WorldGenerator::with_seed(42);
        let gen2 = WorldGenerator::with_seed(42);

        let chunk1 = gen1.generate_chunk(ChunkCoord::new(0, 0));
        let chunk2 = gen2.generate_chunk(ChunkCoord::new(0, 0));

        assert_eq!(chunk1.cells(), chunk2.cells());
    }

    #[test]
    fn test_different_seeds_different_terrain() {
        let gen1 = WorldGenerator::with_seed(42);
        let gen2 = WorldGenerator::with_seed(999);

        let chunk1 = gen1.generate_chunk(ChunkCoord::new(0, 0));
        let chunk2 = gen2.generate_chunk(ChunkCoord::new(0, 0));

        assert_ne!(chunk1.cells(), chunk2.cells());
    }
}
