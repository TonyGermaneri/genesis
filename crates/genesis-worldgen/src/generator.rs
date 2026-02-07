//! Safe world generator wrapper around cubiomes.
//!
//! Generates 2D biome maps for top-down rendering by sampling the
//! Minecraft XZ horizontal plane at a fixed Y (height) level.
//!
//! ## Coordinate mapping (Minecraft 3D → Game 2D)
//!
//! | Minecraft axis | Direction    | Game axis  | Direction           |
//! |----------------|--------------|------------|---------------------|
//! | X              | East/West    | X          | Screen horizontal   |
//! | Z              | North/South  | Y          | Screen vertical     |
//! | Y              | Up/Down      | —          | Fixed `y_level`     |
//!
//! The cubiomes `Range` struct uses `{x, z, sx, sz}` for the horizontal
//! plane and `{y, sy}` for vertical sampling. Our `generate_chunk(cx, cy)`
//! maps game chunk-Y to cubiomes Z internally.

use cubiomes_sys::*;
use std::collections::HashMap;
use tracing::info;

/// Configuration for world generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldGenConfig {
    /// Minecraft version to use for biome generation.
    pub mc_version: i32,
    /// World seed.
    pub seed: u64,
    /// Generator flags (e.g. LARGE_BIOMES).
    pub flags: u32,
    /// Scale for biome generation (1, 4, 16, 64, 256).
    /// Lower = more detail, higher = faster.
    pub scale: i32,
    /// Y level for biome sampling.
    /// At scale=4: y=16 → block y=64 (sea level in MC 1.18+).
    /// This samples the surface biome layer for our top-down view.
    pub y_level: i32,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            mc_version: MC_1_18,
            seed: 0,
            flags: 0,
            scale: 1,
            y_level: 64, // block y=64 → sea level (surface biomes)
        }
    }
}

/// Generated biome data for a rectangular region.
///
/// Biomes are indexed as `[row * width + col]` where:
/// - `row` = cubiomes Z offset (maps to game Y / screen vertical)
/// - `col` = cubiomes X offset (maps to game X / screen horizontal)
#[derive(Debug, Clone)]
pub struct BiomeChunk {
    /// Biome IDs, indexed as `[z_offset * width + x_offset]`.
    pub biomes: Vec<i32>,
    /// Width of the chunk in biome cells.
    pub width: i32,
    /// Height of the chunk in biome cells.
    pub height: i32,
    /// World X origin (in biome coordinates at the configured scale).
    pub origin_x: i32,
    /// World Z origin (in biome coordinates at the configured scale).
    pub origin_z: i32,
    /// Scale used for generation.
    pub scale: i32,
}

impl BiomeChunk {
    /// Get biome at local position (x, z) within this chunk.
    pub fn get(&self, x: i32, z: i32) -> i32 {
        if x < 0 || x >= self.width || z < 0 || z >= self.height {
            return BIOME_NONE;
        }
        self.biomes[(z * self.width + x) as usize]
    }
}

/// Safe wrapper around the cubiomes Generator.
pub struct WorldGenerator {
    /// Pointer to the C Generator struct.
    gen: *mut Generator,
    /// Pointer to the C SurfaceNoise struct (for height sampling).
    sn: *mut SurfaceNoise,
    /// Current configuration.
    config: WorldGenConfig,
}

// SAFETY: The Generator is used single-threaded and the pointer is stable.
unsafe impl Send for WorldGenerator {}

impl WorldGenerator {
    /// Create a new world generator with the given configuration.
    pub fn new(config: WorldGenConfig) -> Self {
        unsafe {
            let gen = cubiomes_generator_new();
            assert!(!gen.is_null(), "Failed to allocate cubiomes Generator");
            cubiomes_generator_init(
                gen,
                config.mc_version,
                config.flags,
                DIM_OVERWORLD,
                config.seed,
            );

            let sn = cubiomes_surface_noise_new();
            assert!(!sn.is_null(), "Failed to allocate cubiomes SurfaceNoise");
            cubiomes_surface_noise_init(sn, DIM_OVERWORLD, config.seed);

            info!(
                "WorldGenerator initialized: mc={}, seed={}, scale={}, flags={}",
                mc_version_name(config.mc_version),
                config.seed,
                config.scale,
                config.flags
            );
            Self { gen, sn, config }
        }
    }

    /// Update the generator with new configuration.
    pub fn reconfigure(&mut self, config: WorldGenConfig) {
        unsafe {
            cubiomes_generator_init(
                self.gen,
                config.mc_version,
                config.flags,
                DIM_OVERWORLD,
                config.seed,
            );
            cubiomes_surface_noise_init(self.sn, DIM_OVERWORLD, config.seed);
        }
        self.config = config;
        info!(
            "WorldGenerator reconfigured: mc={}, seed={}, scale={}",
            mc_version_name(self.config.mc_version),
            self.config.seed,
            self.config.scale,
        );
    }

    /// Get the current configuration.
    pub fn config(&self) -> &WorldGenConfig {
        &self.config
    }

    /// Generate biomes for a rectangular region.
    ///
    /// `x` and `z` are in world biome coordinates (at the configured scale).
    /// `width` and `height` are the size of the region.
    pub fn generate_region(&self, x: i32, z: i32, width: i32, height: i32) -> BiomeChunk {
        let r = Range {
            scale: self.config.scale,
            x,
            z,
            sx: width,
            sz: height,
            y: self.config.y_level,
            sy: 1,
        };

        unsafe {
            let cache = allocCache(self.gen, r);
            assert!(!cache.is_null(), "Failed to allocate biome cache");

            let err = genBiomes(self.gen, cache, r);
            assert_eq!(err, 0, "genBiomes failed with error {}", err);

            let count = (width * height) as usize;
            let biomes = std::slice::from_raw_parts(cache, count).to_vec();

            // Free the C-allocated cache
            libc_free(cache as *mut std::ffi::c_void);

            BiomeChunk {
                biomes,
                width,
                height,
                origin_x: x,
                origin_z: z,
                scale: self.config.scale,
            }
        }
    }

    /// Generate biomes for a chunk at the given game chunk coordinates.
    ///
    /// Each chunk is 16×16 biome cells at the configured scale.
    /// `chunk_x` maps to cubiomes X (east/west).
    /// `chunk_y` maps to cubiomes Z (north/south) for top-down view.
    pub fn generate_chunk(&self, chunk_x: i32, chunk_y: i32) -> BiomeChunk {
        let chunk_size = 16;
        self.generate_region(
            chunk_x * chunk_size,
            chunk_y * chunk_size,
            chunk_size,
            chunk_size,
        )
    }

    /// Get a single biome at world coordinates (block scale).
    pub fn get_biome_at(&self, x: i32, z: i32) -> i32 {
        unsafe { getBiomeAt(self.gen, self.config.scale, x, self.config.y_level, z) }
    }

    /// Generate approximate surface heights for a rectangular region at 1:4 scale.
    ///
    /// `x` and `z` are in 1:4 coordinates (same as biome coords at scale=4).
    /// Returns `width * height` floats in block-level Y values (e.g., 60-120 for
    /// typical overworld terrain). Sea level is ~63.
    pub fn generate_heights(&self, x: i32, z: i32, width: i32, height: i32) -> Vec<f32> {
        let count = (width * height) as usize;
        let mut heights = vec![0.0f32; count];
        unsafe {
            let ret = cubiomes_map_approx_height(
                heights.as_mut_ptr(),
                std::ptr::null_mut(),
                self.gen,
                self.sn,
                x,
                z,
                width,
                height,
            );
            if ret != 0 {
                tracing::warn!("mapApproxHeight returned error {}, using zeros", ret);
            }
        }
        heights
    }

    /// Generate heights for a chunk at game chunk coordinates.
    /// Each chunk is 16×16 cells at 1:4 scale.
    pub fn generate_chunk_heights(&self, chunk_x: i32, chunk_y: i32) -> Vec<f32> {
        let chunk_size = 16;
        self.generate_heights(
            chunk_x * chunk_size,
            chunk_y * chunk_size,
            chunk_size,
            chunk_size,
        )
    }

    /// Generate surface heights at true block-level (1:1) resolution for MC 1.18+.
    ///
    /// `bx` and `bz` are in block coordinates.
    /// Returns `width * height` floats with the same height scale as `generate_heights`.
    pub fn generate_block_heights(&self, bx: i32, bz: i32, width: i32, height: i32) -> Vec<f32> {
        let count = (width * height) as usize;
        let mut heights = vec![0.0f32; count];
        unsafe {
            let ret = cubiomes_map_block_height(
                heights.as_mut_ptr(),
                self.gen,
                bx,
                bz,
                width,
                height,
            );
            if ret != 0 {
                tracing::warn!("cubiomes_map_block_height returned error {}, falling back to 1:4", ret);
                // Fallback to 1:4 scale for unsupported MC versions
                return self.generate_heights(bx / 4, bz / 4, width, height);
            }
        }
        heights
    }

    /// Generate block-level heights for a chunk at game chunk coordinates.
    /// Each chunk is 16×16 blocks at 1:1 scale.
    pub fn generate_chunk_block_heights(&self, chunk_x: i32, chunk_y: i32) -> Vec<f32> {
        let chunk_size = 16;
        self.generate_block_heights(
            chunk_x * chunk_size,
            chunk_y * chunk_size,
            chunk_size,
            chunk_size,
        )
    }

    /// Get the biome name for a given ID.
    pub fn biome_name(&self, id: i32) -> String {
        biome_name(self.config.mc_version, id)
    }

    /// Get all overworld biome IDs valid for the current MC version.
    pub fn valid_biomes(&self) -> Vec<(i32, String)> {
        let mut result = Vec::new();
        for id in 0..256i32 {
            unsafe {
                if isOverworld(self.config.mc_version, id) != 0 {
                    result.push((id, biome_name(self.config.mc_version, id)));
                }
            }
        }
        result
    }

    /// Generate a large overview map and return biome IDs + RGB colors.
    /// Useful for minimap rendering.
    pub fn generate_overview(
        &self,
        center_x: i32,
        center_z: i32,
        radius: i32,
    ) -> (BiomeChunk, Vec<[u8; 3]>) {
        let size = radius * 2;
        let chunk = self.generate_region(center_x - radius, center_z - radius, size, size);

        let colors = default_biome_colors();
        let rgb: Vec<[u8; 3]> = chunk
            .biomes
            .iter()
            .map(|&id| {
                if id >= 0 && id < 256 {
                    colors[id as usize]
                } else {
                    [128, 128, 128] // gray for unknown
                }
            })
            .collect();

        (chunk, rgb)
    }
}

impl Drop for WorldGenerator {
    fn drop(&mut self) {
        unsafe {
            cubiomes_generator_free(self.gen);
            cubiomes_surface_noise_free(self.sn);
        }
    }
}

extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

unsafe fn libc_free(ptr: *mut std::ffi::c_void) {
    unsafe { free(ptr) }
}

/// Unique set of biomes found in a generated chunk.
pub fn unique_biomes(chunk: &BiomeChunk) -> HashMap<i32, usize> {
    let mut counts = HashMap::new();
    for &id in &chunk.biomes {
        *counts.entry(id).or_insert(0) += 1;
    }
    counts
}
