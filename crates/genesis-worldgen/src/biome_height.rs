//! Biome-derived heightmap for top-down terrain shading.
//!
//! Maps each biome ID to a normalized elevation value (0.0–1.0)
//! based on Minecraft's approximate surface height for that biome type.
//! This is used for:
//!
//! - **Elevation shading**: higher terrain is brighter, lower is darker
//! - **Shadow casting**: taller neighbours cast shadows based on sun angle

use cubiomes_sys::*;

/// Returns a normalized height (0.0 = deep ocean floor, 1.0 = mountain peak)
/// for the given biome ID, approximating Minecraft surface elevation.
///
/// Heights are grouped into tiers:
/// - 0.0–0.15  Deep oceans
/// - 0.15–0.25 Shallow oceans
/// - 0.25–0.35 Rivers, beaches, shores
/// - 0.35–0.50 Swamps, mushroom fields, low plains
/// - 0.45–0.55 Plains, forests, deserts (sea-level land)
/// - 0.55–0.65 Hills, plateaus
/// - 0.65–0.80 Mountains, taller hills
/// - 0.80–1.00 Extreme peaks
pub fn biome_height(id: i32) -> f32 {
    match id {
        // Deep oceans — lowest
        BIOME_DEEP_OCEAN => 0.05,
        47 /*DEEP_WARM_OCEAN*/ => 0.05,
        BIOME_DEEP_LUKEWARM_OCEAN => 0.05,
        BIOME_DEEP_COLD_OCEAN => 0.06,
        BIOME_DEEP_FROZEN_OCEAN => 0.06,
        BIOME_DEEP_DARK => 0.02, // underground cavern

        // Shallow oceans
        BIOME_OCEAN => 0.18,
        BIOME_WARM_OCEAN => 0.20,
        BIOME_LUKEWARM_OCEAN => 0.19,
        BIOME_COLD_OCEAN => 0.17,
        BIOME_FROZEN_OCEAN => 0.16,

        // Rivers, beaches, shores
        BIOME_RIVER => 0.28,
        BIOME_FROZEN_RIVER => 0.27,
        BIOME_BEACH => 0.32,
        BIOME_SNOWY_BEACH => 0.31,
        BIOME_STONE_SHORE => 0.35,
        BIOME_MUSHROOM_FIELD_SHORE => 0.33,

        // Low-lying land
        BIOME_SWAMP => 0.38,
        134 /*SWAMP_HILLS*/ => 0.42,
        BIOME_MANGROVE_SWAMP => 0.37,
        BIOME_MUSHROOM_FIELDS => 0.40,

        // Sea-level land (plains, forests, deserts)
        BIOME_PLAINS => 0.48,
        BIOME_SUNFLOWER_PLAINS => 0.48,
        BIOME_FOREST => 0.50,
        BIOME_FLOWER_FOREST => 0.50,
        BIOME_BIRCH_FOREST => 0.50,
        BIOME_DARK_FOREST => 0.52,
        BIOME_PALE_GARDEN => 0.50,
        BIOME_TAIGA => 0.50,
        BIOME_SNOWY_TAIGA => 0.50,
        BIOME_SNOWY_TUNDRA => 0.47,
        BIOME_DESERT => 0.48,
        BIOME_JUNGLE => 0.50,
        BIOME_JUNGLE_EDGE => 0.48,
        BIOME_BAMBOO_JUNGLE => 0.50,
        BIOME_SAVANNA => 0.48,
        BIOME_CHERRY_GROVE => 0.52,
        BIOME_MEADOW => 0.55,
        BIOME_DRIPSTONE_CAVES => 0.30,
        BIOME_LUSH_CAVES => 0.28,
        BIOME_ICE_SPIKES => 0.52,

        // Hills
        BIOME_DESERT_HILLS => 0.58,
        BIOME_WOODED_HILLS => 0.58,
        BIOME_TAIGA_HILLS => 0.57,
        BIOME_JUNGLE_HILLS => 0.58,
        BIOME_BIRCH_FOREST_HILLS => 0.57,
        BIOME_SNOWY_TAIGA_HILLS => 0.57,
        BIOME_GIANT_TREE_TAIGA => 0.53,
        BIOME_GIANT_TREE_TAIGA_HILLS => 0.58,
        BIOME_TALL_BIRCH_FOREST => 0.52,
        BIOME_TALL_BIRCH_HILLS => 0.58,
        BIOME_DARK_FOREST_HILLS => 0.58,
        BIOME_GIANT_SPRUCE_TAIGA => 0.53,
        BIOME_GIANT_SPRUCE_TAIGA_HILLS => 0.58,
        BIOME_BAMBOO_JUNGLE_HILLS => 0.58,
        BIOME_MOUNTAIN_EDGE => 0.60,

        // Plateaus and elevated terrain
        BIOME_SAVANNA_PLATEAU => 0.62,
        BIOME_SHATTERED_SAVANNA => 0.65,
        BIOME_SHATTERED_SAVANNA_PLATEAU => 0.68,
        BIOME_BADLANDS => 0.60,
        BIOME_BADLANDS_PLATEAU => 0.65,
        BIOME_WOODED_BADLANDS_PLATEAU => 0.65,
        BIOME_ERODED_BADLANDS => 0.62,
        BIOME_MODIFIED_BADLANDS_PLATEAU => 0.67,
        BIOME_MODIFIED_WOODED_BADLANDS_PLATEAU => 0.67,
        BIOME_GROVE => 0.65,
        BIOME_SNOWY_SLOPES => 0.70,

        // Mountains
        BIOME_MOUNTAINS => 0.72,
        BIOME_WOODED_MOUNTAINS => 0.73,
        BIOME_GRAVELLY_MOUNTAINS => 0.72,
        BIOME_MODIFIED_GRAVELLY_MOUNTAINS => 0.74,
        BIOME_SNOWY_MOUNTAINS => 0.72,
        BIOME_TAIGA_MOUNTAINS => 0.68,
        BIOME_SNOWY_TAIGA_MOUNTAINS => 0.70,

        // Extreme peaks — tallest
        BIOME_STONY_PEAKS => 0.85,
        BIOME_JAGGED_PEAKS => 0.92,
        BIOME_FROZEN_PEAKS => 0.90,

        // Nether / End — treat as mid-height
        BIOME_NETHER_WASTES | BIOME_SOUL_SAND_VALLEY |
        BIOME_CRIMSON_FOREST | BIOME_WARPED_FOREST |
        BIOME_BASALT_DELTAS => 0.45,
        BIOME_THE_END | BIOME_SMALL_END_ISLANDS |
        BIOME_END_MIDLANDS | BIOME_END_HIGHLANDS |
        BIOME_END_BARRENS => 0.50,

        // Unknown — default to sea-level land
        _ => 0.48,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_height_range() {
        // Spot-check key biomes
        assert!(biome_height(BIOME_DEEP_OCEAN) < 0.1);
        assert!(biome_height(BIOME_OCEAN) < 0.25);
        assert!((biome_height(BIOME_PLAINS) - 0.48).abs() < 0.01);
        assert!(biome_height(BIOME_MOUNTAINS) > 0.70);
        assert!(biome_height(BIOME_JAGGED_PEAKS) > 0.90);
    }

    #[test]
    fn test_all_heights_in_bounds() {
        for id in -1..256 {
            let h = biome_height(id);
            assert!(h >= 0.0 && h <= 1.0, "biome {} height {} out of bounds", id, h);
        }
    }
}
