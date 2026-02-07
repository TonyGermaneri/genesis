//! Raw FFI bindings to the cubiomes C library.
//!
//! This crate compiles the cubiomes C source and provides unsafe FFI functions
//! for Minecraft biome generation. Use `genesis-worldgen` for a safe wrapper.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int, c_uint};

// ============================================================================
// Opaque Generator type
// ============================================================================

/// Opaque Generator struct. Must be heap-allocated via `cubiomes_generator_new`.
/// Size: 27592 bytes, align: 8 on 64-bit macOS.
#[repr(C)]
pub struct Generator {
    _opaque: [u8; 27592],
}

/// Opaque SurfaceNoise struct. Must be heap-allocated via `cubiomes_surface_noise_new`.
/// Contains Perlin octave noise data used by `mapApproxHeight`.
#[repr(C)]
pub struct SurfaceNoise {
    _opaque: [u8; 0], // Actual size determined at runtime; always heap-allocated via C helper
}

/// Range struct for specifying biome generation areas.
/// This struct is simple enough to replicate directly.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Range {
    /// Horizontal scale factor: 1, 4, 16, 64, or 256
    pub scale: c_int,
    /// Horizontal position X (north-west corner)
    pub x: c_int,
    /// Horizontal position Z (north-west corner)
    pub z: c_int,
    /// Horizontal size X (width)
    pub sx: c_int,
    /// Horizontal size Z (height)
    pub sz: c_int,
    /// Vertical position Y
    pub y: c_int,
    /// Vertical size
    pub sy: c_int,
}

/// Pos struct returned by various finder functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Pos {
    pub x: c_int,
    pub z: c_int,
}

// ============================================================================
// MCVersion enum
// ============================================================================

pub const MC_UNDEF: c_int = 0;
pub const MC_B1_7: c_int = 1;
pub const MC_B1_8: c_int = 2;
pub const MC_1_0: c_int = 3;
pub const MC_1_1: c_int = 4;
pub const MC_1_2: c_int = 5;
pub const MC_1_3: c_int = 6;
pub const MC_1_4: c_int = 7;
pub const MC_1_5: c_int = 8;
pub const MC_1_6: c_int = 9;
pub const MC_1_7: c_int = 10;
pub const MC_1_8: c_int = 11;
pub const MC_1_9: c_int = 12;
pub const MC_1_10: c_int = 13;
pub const MC_1_11: c_int = 14;
pub const MC_1_12: c_int = 15;
pub const MC_1_13: c_int = 16;
pub const MC_1_14: c_int = 17;
pub const MC_1_15: c_int = 18;
pub const MC_1_16_1: c_int = 19;
pub const MC_1_16: c_int = 20;
pub const MC_1_17: c_int = 21;
pub const MC_1_18: c_int = 22;
pub const MC_1_19_2: c_int = 23;
pub const MC_1_19: c_int = 24;
pub const MC_1_20: c_int = 25;
pub const MC_1_21_1: c_int = 26;
pub const MC_1_21_3: c_int = 27;
pub const MC_1_21_WD: c_int = 28;
pub const MC_1_21: c_int = MC_1_21_WD;
pub const MC_NEWEST: c_int = MC_1_21;

// ============================================================================
// Dimension enum
// ============================================================================

pub const DIM_NETHER: c_int = -1;
pub const DIM_OVERWORLD: c_int = 0;
pub const DIM_END: c_int = 1;

// ============================================================================
// Generator flags
// ============================================================================

pub const LARGE_BIOMES: c_uint = 0x1;
pub const NO_BETA_OCEAN: c_uint = 0x2;
pub const FORCE_OCEAN_VARIANTS: c_uint = 0x4;

// ============================================================================
// BiomeID constants (Overworld biomes we care about for top-down rendering)
// ============================================================================

pub const BIOME_NONE: c_int = -1;
pub const BIOME_OCEAN: c_int = 0;
pub const BIOME_PLAINS: c_int = 1;
pub const BIOME_DESERT: c_int = 2;
pub const BIOME_MOUNTAINS: c_int = 3;
pub const BIOME_FOREST: c_int = 4;
pub const BIOME_TAIGA: c_int = 5;
pub const BIOME_SWAMP: c_int = 6;
pub const BIOME_RIVER: c_int = 7;
pub const BIOME_NETHER_WASTES: c_int = 8;
pub const BIOME_THE_END: c_int = 9;
pub const BIOME_FROZEN_OCEAN: c_int = 10;
pub const BIOME_FROZEN_RIVER: c_int = 11;
pub const BIOME_SNOWY_TUNDRA: c_int = 12;
pub const BIOME_SNOWY_MOUNTAINS: c_int = 13;
pub const BIOME_MUSHROOM_FIELDS: c_int = 14;
pub const BIOME_MUSHROOM_FIELD_SHORE: c_int = 15;
pub const BIOME_BEACH: c_int = 16;
pub const BIOME_DESERT_HILLS: c_int = 17;
pub const BIOME_WOODED_HILLS: c_int = 18;
pub const BIOME_TAIGA_HILLS: c_int = 19;
pub const BIOME_MOUNTAIN_EDGE: c_int = 20;
pub const BIOME_JUNGLE: c_int = 21;
pub const BIOME_JUNGLE_HILLS: c_int = 22;
pub const BIOME_JUNGLE_EDGE: c_int = 23;
pub const BIOME_DEEP_OCEAN: c_int = 24;
pub const BIOME_STONE_SHORE: c_int = 25;
pub const BIOME_SNOWY_BEACH: c_int = 26;
pub const BIOME_BIRCH_FOREST: c_int = 27;
pub const BIOME_BIRCH_FOREST_HILLS: c_int = 28;
pub const BIOME_DARK_FOREST: c_int = 29;
pub const BIOME_SNOWY_TAIGA: c_int = 30;
pub const BIOME_SNOWY_TAIGA_HILLS: c_int = 31;
pub const BIOME_GIANT_TREE_TAIGA: c_int = 32;
pub const BIOME_GIANT_TREE_TAIGA_HILLS: c_int = 33;
pub const BIOME_WOODED_MOUNTAINS: c_int = 34;
pub const BIOME_SAVANNA: c_int = 35;
pub const BIOME_SAVANNA_PLATEAU: c_int = 36;
pub const BIOME_BADLANDS: c_int = 37;
pub const BIOME_WOODED_BADLANDS_PLATEAU: c_int = 38;
pub const BIOME_BADLANDS_PLATEAU: c_int = 39;
pub const BIOME_SMALL_END_ISLANDS: c_int = 40;
pub const BIOME_END_MIDLANDS: c_int = 41;
pub const BIOME_END_HIGHLANDS: c_int = 42;
pub const BIOME_END_BARRENS: c_int = 43;
pub const BIOME_WARM_OCEAN: c_int = 44;
pub const BIOME_LUKEWARM_OCEAN: c_int = 45;
pub const BIOME_COLD_OCEAN: c_int = 46;
pub const BIOME_DEEP_WARM_OCEAN: c_int = 47;
pub const BIOME_DEEP_LUKEWARM_OCEAN: c_int = 48;
pub const BIOME_DEEP_COLD_OCEAN: c_int = 49;
pub const BIOME_DEEP_FROZEN_OCEAN: c_int = 50;

// 1.14+
pub const BIOME_BAMBOO_JUNGLE: c_int = 168;
pub const BIOME_BAMBOO_JUNGLE_HILLS: c_int = 169;

// 1.16+
pub const BIOME_SOUL_SAND_VALLEY: c_int = 170;
pub const BIOME_CRIMSON_FOREST: c_int = 171;
pub const BIOME_WARPED_FOREST: c_int = 172;
pub const BIOME_BASALT_DELTAS: c_int = 173;

// 1.17+
pub const BIOME_DRIPSTONE_CAVES: c_int = 174;
pub const BIOME_LUSH_CAVES: c_int = 175;

// 1.18+
pub const BIOME_MEADOW: c_int = 177;
pub const BIOME_GROVE: c_int = 178;
pub const BIOME_SNOWY_SLOPES: c_int = 179;
pub const BIOME_JAGGED_PEAKS: c_int = 180;
pub const BIOME_FROZEN_PEAKS: c_int = 181;
pub const BIOME_STONY_PEAKS: c_int = 182;

// 1.19+
pub const BIOME_DEEP_DARK: c_int = 183;
pub const BIOME_MANGROVE_SWAMP: c_int = 184;

// 1.20+
pub const BIOME_CHERRY_GROVE: c_int = 185;

// 1.21+
pub const BIOME_PALE_GARDEN: c_int = 186;

// Mutated biome variants (id + 128)
pub const BIOME_SUNFLOWER_PLAINS: c_int = 129;
pub const BIOME_DESERT_LAKES: c_int = 130;
pub const BIOME_GRAVELLY_MOUNTAINS: c_int = 131;
pub const BIOME_FLOWER_FOREST: c_int = 132;
pub const BIOME_TAIGA_MOUNTAINS: c_int = 133;
pub const BIOME_SWAMP_HILLS: c_int = 134;
pub const BIOME_ICE_SPIKES: c_int = 140;
pub const BIOME_MODIFIED_JUNGLE: c_int = 149;
pub const BIOME_MODIFIED_JUNGLE_EDGE: c_int = 151;
pub const BIOME_TALL_BIRCH_FOREST: c_int = 155;
pub const BIOME_TALL_BIRCH_HILLS: c_int = 156;
pub const BIOME_DARK_FOREST_HILLS: c_int = 157;
pub const BIOME_SNOWY_TAIGA_MOUNTAINS: c_int = 158;
pub const BIOME_GIANT_SPRUCE_TAIGA: c_int = 160;
pub const BIOME_GIANT_SPRUCE_TAIGA_HILLS: c_int = 161;
pub const BIOME_MODIFIED_GRAVELLY_MOUNTAINS: c_int = 162;
pub const BIOME_SHATTERED_SAVANNA: c_int = 163;
pub const BIOME_SHATTERED_SAVANNA_PLATEAU: c_int = 164;
pub const BIOME_ERODED_BADLANDS: c_int = 165;
pub const BIOME_MODIFIED_WOODED_BADLANDS_PLATEAU: c_int = 166;
pub const BIOME_MODIFIED_BADLANDS_PLATEAU: c_int = 167;

// ============================================================================
// Structure types (for finder functions)
// ============================================================================

pub const STRUCT_DESERT_PYRAMID: c_int = 1;
pub const STRUCT_JUNGLE_TEMPLE: c_int = 2;
pub const STRUCT_SWAMP_HUT: c_int = 3;
pub const STRUCT_IGLOO: c_int = 4;
pub const STRUCT_VILLAGE: c_int = 5;
pub const STRUCT_OCEAN_RUIN: c_int = 6;
pub const STRUCT_SHIPWRECK: c_int = 7;
pub const STRUCT_MONUMENT: c_int = 8;
pub const STRUCT_MANSION: c_int = 9;
pub const STRUCT_OUTPOST: c_int = 10;
pub const STRUCT_RUINED_PORTAL: c_int = 11;
pub const STRUCT_RUINED_PORTAL_N: c_int = 12;
pub const STRUCT_ANCIENT_CITY: c_int = 13;
pub const STRUCT_TREASURE: c_int = 14;
pub const STRUCT_MINESHAFT: c_int = 15;
pub const STRUCT_FORTRESS: c_int = 16;
pub const STRUCT_BASTION: c_int = 17;
pub const STRUCT_END_CITY: c_int = 18;
pub const STRUCT_END_GATEWAY: c_int = 19;
pub const STRUCT_TRAIL_RUINS: c_int = 20;
pub const STRUCT_TRIAL_CHAMBERS: c_int = 21;

// ============================================================================
// FFI function declarations
// ============================================================================

extern "C" {
    // --- Generator lifecycle (via helper) ---

    /// Allocate a new Generator on the heap.
    pub fn cubiomes_generator_new() -> *mut Generator;

    /// Free a heap-allocated Generator.
    pub fn cubiomes_generator_free(g: *mut Generator);

    /// Setup generator, apply seed in one call.
    pub fn cubiomes_generator_init(
        g: *mut Generator,
        mc: c_int,
        flags: c_uint,
        dim: c_int,
        seed: u64,
    );

    /// Get sizeof(Generator).
    pub fn cubiomes_generator_size() -> usize;

    /// Get alignof(Generator).
    pub fn cubiomes_generator_align() -> usize;

    /// Get mc version from generator.
    pub fn cubiomes_generator_get_mc(g: *const Generator) -> c_int;

    /// Get seed from generator.
    pub fn cubiomes_generator_get_seed(g: *const Generator) -> u64;

    /// Get dimension from generator.
    pub fn cubiomes_generator_get_dim(g: *const Generator) -> c_int;

    // --- Core cubiomes API ---

    /// Setup a biome generator for a given MC version.
    pub fn setupGenerator(g: *mut Generator, mc: c_int, flags: c_uint);

    /// Initialize the generator for a given dimension and seed.
    pub fn applySeed(g: *mut Generator, dim: c_int, seed: u64);

    /// Get minimum cache size for generating biomes in a range.
    pub fn getMinCacheSize(g: *const Generator, scale: c_int, sx: c_int, sy: c_int, sz: c_int) -> usize;

    /// Allocate a biome ID cache for the given range.
    pub fn allocCache(g: *const Generator, r: Range) -> *mut c_int;

    /// Generate biomes for a range. Returns 0 on success.
    pub fn genBiomes(g: *const Generator, cache: *mut c_int, r: Range) -> c_int;

    /// Get biome at a specific scaled position.
    pub fn getBiomeAt(g: *const Generator, scale: c_int, x: c_int, y: c_int, z: c_int) -> c_int;

    // --- Utility ---

    /// Initialize default biome colors array.
    pub fn initBiomeColors(biome_colors: *mut [u8; 3]);

    /// Convert biome ID to string name.
    pub fn biome2str(mc: c_int, id: c_int) -> *const c_char;

    /// Convert MC version to string.
    pub fn mc2str(mc: c_int) -> *const c_char;

    /// Convert string to MC version.
    pub fn str2mc(s: *const c_char) -> c_int;

    /// Render biomes to an RGB image buffer.
    pub fn biomesToImage(
        rgb: *mut u8,
        biome_colors: *const [u8; 3],
        biome_ids: *const c_int,
        sx: c_int,
        sz: c_int,
        pix_scale: c_int,
        ty: c_int,
    );

    // --- Structure finding ---

    /// Get structure position for a given region.
    pub fn getStructurePos(
        struct_type: c_int,
        mc: c_int,
        seed: u64,
        reg_x: c_int,
        reg_z: c_int,
        pos: *mut Pos,
    ) -> c_int;

    /// Check if a structure position has viable biomes.
    pub fn isViableStructurePos(
        struct_type: c_int,
        g: *mut Generator,
        x: c_int,
        z: c_int,
        flags: c_uint,
    ) -> c_int;

    // --- Biome classification helpers ---

    /// Check if a biome is oceanic.
    pub fn isOceanic(id: c_int) -> c_int;

    /// Check if a biome is snowy.
    pub fn isSnowy(id: c_int) -> c_int;

    /// Check if biome is valid for given MC version in the overworld.
    pub fn isOverworld(mc: c_int, id: c_int) -> c_int;

    // --- SurfaceNoise lifecycle (via helper) ---

    /// Get sizeof(SurfaceNoise).
    pub fn cubiomes_surface_noise_size() -> usize;

    /// Get alignof(SurfaceNoise).
    pub fn cubiomes_surface_noise_align() -> usize;

    /// Allocate a new SurfaceNoise on the heap.
    pub fn cubiomes_surface_noise_new() -> *mut SurfaceNoise;

    /// Free a heap-allocated SurfaceNoise.
    pub fn cubiomes_surface_noise_free(sn: *mut SurfaceNoise);

    /// Initialize a SurfaceNoise for a given dimension and seed.
    pub fn cubiomes_surface_noise_init(sn: *mut SurfaceNoise, dim: c_int, seed: u64);

    /// Map an approximation of the Overworld surface height at 1:4 scale.
    /// Writes `w * h` floats into `y`. If `ids` is non-null, fills with biome IDs.
    /// Returns 0 on success.
    pub fn cubiomes_map_approx_height(
        y: *mut f32,
        ids: *mut c_int,
        g: *const Generator,
        sn: *const SurfaceNoise,
        x: c_int,
        z: c_int,
        w: c_int,
        h: c_int,
    ) -> c_int;
}

// ============================================================================
// Safe helpers
// ============================================================================

/// Get the name of a biome as a Rust string.
pub fn biome_name(mc: c_int, id: c_int) -> String {
    unsafe {
        let ptr = biome2str(mc, id);
        if ptr.is_null() {
            return format!("unknown_{}", id);
        }
        std::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned()
    }
}

/// Get MC version name as a Rust string.
pub fn mc_version_name(mc: c_int) -> String {
    unsafe {
        let ptr = mc2str(mc);
        if ptr.is_null() {
            return format!("unknown_mc_{}", mc);
        }
        std::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned()
    }
}

/// Get default biome colors (256 entries, RGB).
pub fn default_biome_colors() -> [[u8; 3]; 256] {
    let mut colors = [[0u8; 3]; 256];
    unsafe {
        initBiomeColors(colors.as_mut_ptr());
    }
    colors
}

/// List of all MC versions with their names and enum values.
pub fn all_mc_versions() -> Vec<(c_int, String)> {
    let versions = [
        MC_B1_7, MC_B1_8, MC_1_0, MC_1_1, MC_1_2, MC_1_3, MC_1_4, MC_1_5,
        MC_1_6, MC_1_7, MC_1_8, MC_1_9, MC_1_10, MC_1_11, MC_1_12, MC_1_13,
        MC_1_14, MC_1_15, MC_1_16_1, MC_1_16, MC_1_17, MC_1_18, MC_1_19_2,
        MC_1_19, MC_1_20, MC_1_21_1, MC_1_21_3, MC_1_21_WD,
    ];
    versions
        .iter()
        .map(|&v| (v, mc_version_name(v)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    #[test]
    fn test_generator_lifecycle() {
        unsafe {
            let g = cubiomes_generator_new();
            assert!(!g.is_null());

            cubiomes_generator_init(g, MC_1_18, 0, DIM_OVERWORLD, 12345);

            assert_eq!(cubiomes_generator_get_mc(g), MC_1_18);
            assert_eq!(cubiomes_generator_get_seed(g), 12345);
            assert_eq!(cubiomes_generator_get_dim(g), DIM_OVERWORLD);

            // Generate a single biome
            let biome = getBiomeAt(g, 4, 0, 63, 0);
            assert_ne!(biome, BIOME_NONE);

            cubiomes_generator_free(g);
        }
    }

    #[test]
    fn test_biome_generation_range() {
        unsafe {
            let g = cubiomes_generator_new();
            cubiomes_generator_init(g, MC_1_18, 0, DIM_OVERWORLD, 42);

            let r = Range {
                scale: 4,
                x: 0,
                z: 0,
                sx: 16,
                sz: 16,
                y: 15,
                sy: 1,
            };

            let cache = allocCache(g, r);
            assert!(!cache.is_null());

            let err = genBiomes(g, cache, r);
            assert_eq!(err, 0);

            // Check that we got valid biome IDs
            for i in 0..(16 * 16) {
                let biome = *cache.add(i);
                assert!(biome >= 0 && biome < 256, "Invalid biome: {}", biome);
            }

            libc_free(cache as *mut c_void);
            cubiomes_generator_free(g);
        }
    }

    #[test]
    fn test_biome_colors() {
        let colors = default_biome_colors();
        // Plains should be green-ish
        let plains = colors[BIOME_PLAINS as usize];
        assert!(plains[1] > plains[0], "Plains should be greenish");
    }

    #[test]
    fn test_biome_names() {
        let name = biome_name(MC_1_18, BIOME_PLAINS);
        assert_eq!(name, "plains");

        let name = biome_name(MC_1_18, BIOME_DESERT);
        assert_eq!(name, "desert");
    }

    extern "C" {
        fn free(ptr: *mut c_void);
    }
    unsafe fn libc_free(ptr: *mut c_void) {
        free(ptr);
    }
}
