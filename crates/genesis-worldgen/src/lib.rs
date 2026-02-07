//! World generation module using cubiomes.
//!
//! Provides safe Rust wrappers around the cubiomes C library for
//! Minecraft-style biome generation, biome-to-texture mapping,
//! and chunk-based terrain generation for 2D top-down rendering.

pub mod biome_height;
pub mod biome_map;
pub mod generator;

pub use biome_height::biome_height;
pub use biome_map::{BiomeEntry, BiomeTextureMap, BiomeVisual};
pub use generator::{BiomeChunk, WorldGenConfig, WorldGenerator};

// Re-export key cubiomes constants for convenience
pub use cubiomes_sys::{
    // MC versions
    MC_1_18, MC_1_19, MC_1_20, MC_1_21, MC_NEWEST,
    // Flags
    LARGE_BIOMES, FORCE_OCEAN_VARIANTS,
    // Biome IDs
    BIOME_OCEAN, BIOME_PLAINS, BIOME_DESERT, BIOME_FOREST, BIOME_TAIGA,
    BIOME_SWAMP, BIOME_RIVER, BIOME_MOUNTAINS, BIOME_BEACH,
    BIOME_SNOWY_TUNDRA, BIOME_JUNGLE, BIOME_BIRCH_FOREST, BIOME_DARK_FOREST,
    BIOME_SAVANNA, BIOME_BADLANDS, BIOME_MUSHROOM_FIELDS, BIOME_MEADOW,
    BIOME_CHERRY_GROVE, BIOME_DEEP_OCEAN, BIOME_WARM_OCEAN,
    // Utility
    all_mc_versions, mc_version_name, biome_name,
};
