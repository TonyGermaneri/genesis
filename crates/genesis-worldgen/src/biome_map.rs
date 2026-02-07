//! Biome-to-texture mapping system.
//!
//! Maps cubiomes BiomeIDs to texture file paths or fallback solid colors.
//! Configuration is serialized to/from a TOML file for easy editing.

use cubiomes_sys::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How a biome is visually represented.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeVisual {
    /// Solid color fill (R, G, B).
    Color([u8; 3]),
    /// Path to a texture file (relative to game assets dir).
    Texture(String),
}

/// Entry for a single biome in the texture map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeEntry {
    /// Display name of the biome.
    pub name: String,
    /// Biome ID from cubiomes.
    pub id: i32,
    /// Visual representation.
    pub visual: BiomeVisual,
}

/// Maps biome IDs to their visual representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeTextureMap {
    /// Map of biome ID → entry.
    pub entries: HashMap<i32, BiomeEntry>,
}

impl Default for BiomeTextureMap {
    fn default() -> Self {
        Self::from_cubiomes_defaults()
    }
}

impl BiomeTextureMap {
    /// Create a new empty biome texture map.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Build a map using cubiomes default biome colors.
    /// Each biome gets its cubiomes-default color as a solid fill.
    pub fn from_cubiomes_defaults() -> Self {
        let colors = default_biome_colors();
        let mut entries = HashMap::new();

        // Register all known overworld biomes with their default colors
        let biomes = [
            (BIOME_OCEAN, "ocean"),
            (BIOME_PLAINS, "plains"),
            (BIOME_DESERT, "desert"),
            (BIOME_MOUNTAINS, "mountains"),
            (BIOME_FOREST, "forest"),
            (BIOME_TAIGA, "taiga"),
            (BIOME_SWAMP, "swamp"),
            (BIOME_RIVER, "river"),
            (BIOME_FROZEN_OCEAN, "frozen_ocean"),
            (BIOME_FROZEN_RIVER, "frozen_river"),
            (BIOME_SNOWY_TUNDRA, "snowy_tundra"),
            (BIOME_SNOWY_MOUNTAINS, "snowy_mountains"),
            (BIOME_MUSHROOM_FIELDS, "mushroom_fields"),
            (BIOME_MUSHROOM_FIELD_SHORE, "mushroom_field_shore"),
            (BIOME_BEACH, "beach"),
            (BIOME_DESERT_HILLS, "desert_hills"),
            (BIOME_WOODED_HILLS, "wooded_hills"),
            (BIOME_TAIGA_HILLS, "taiga_hills"),
            (BIOME_MOUNTAIN_EDGE, "mountain_edge"),
            (BIOME_JUNGLE, "jungle"),
            (BIOME_JUNGLE_HILLS, "jungle_hills"),
            (BIOME_JUNGLE_EDGE, "jungle_edge"),
            (BIOME_DEEP_OCEAN, "deep_ocean"),
            (BIOME_STONE_SHORE, "stone_shore"),
            (BIOME_SNOWY_BEACH, "snowy_beach"),
            (BIOME_BIRCH_FOREST, "birch_forest"),
            (BIOME_BIRCH_FOREST_HILLS, "birch_forest_hills"),
            (BIOME_DARK_FOREST, "dark_forest"),
            (BIOME_SNOWY_TAIGA, "snowy_taiga"),
            (BIOME_SNOWY_TAIGA_HILLS, "snowy_taiga_hills"),
            (BIOME_GIANT_TREE_TAIGA, "giant_tree_taiga"),
            (BIOME_GIANT_TREE_TAIGA_HILLS, "giant_tree_taiga_hills"),
            (BIOME_WOODED_MOUNTAINS, "wooded_mountains"),
            (BIOME_SAVANNA, "savanna"),
            (BIOME_SAVANNA_PLATEAU, "savanna_plateau"),
            (BIOME_BADLANDS, "badlands"),
            (BIOME_WOODED_BADLANDS_PLATEAU, "wooded_badlands_plateau"),
            (BIOME_BADLANDS_PLATEAU, "badlands_plateau"),
            (BIOME_WARM_OCEAN, "warm_ocean"),
            (BIOME_LUKEWARM_OCEAN, "lukewarm_ocean"),
            (BIOME_COLD_OCEAN, "cold_ocean"),
            (BIOME_DEEP_WARM_OCEAN, "deep_warm_ocean"),
            (BIOME_DEEP_LUKEWARM_OCEAN, "deep_lukewarm_ocean"),
            (BIOME_DEEP_COLD_OCEAN, "deep_cold_ocean"),
            (BIOME_DEEP_FROZEN_OCEAN, "deep_frozen_ocean"),
            (BIOME_BAMBOO_JUNGLE, "bamboo_jungle"),
            (BIOME_BAMBOO_JUNGLE_HILLS, "bamboo_jungle_hills"),
            (BIOME_SOUL_SAND_VALLEY, "soul_sand_valley"),
            (BIOME_CRIMSON_FOREST, "crimson_forest"),
            (BIOME_WARPED_FOREST, "warped_forest"),
            (BIOME_BASALT_DELTAS, "basalt_deltas"),
            (BIOME_DRIPSTONE_CAVES, "dripstone_caves"),
            (BIOME_LUSH_CAVES, "lush_caves"),
            (BIOME_MEADOW, "meadow"),
            (BIOME_GROVE, "grove"),
            (BIOME_SNOWY_SLOPES, "snowy_slopes"),
            (BIOME_JAGGED_PEAKS, "jagged_peaks"),
            (BIOME_FROZEN_PEAKS, "frozen_peaks"),
            (BIOME_STONY_PEAKS, "stony_peaks"),
            (BIOME_DEEP_DARK, "deep_dark"),
            (BIOME_MANGROVE_SWAMP, "mangrove_swamp"),
            (BIOME_CHERRY_GROVE, "cherry_grove"),
            (BIOME_PALE_GARDEN, "pale_garden"),
            // Mutated variants
            (BIOME_SUNFLOWER_PLAINS, "sunflower_plains"),
            (BIOME_DESERT_LAKES, "desert_lakes"),
            (BIOME_GRAVELLY_MOUNTAINS, "gravelly_mountains"),
            (BIOME_FLOWER_FOREST, "flower_forest"),
            (BIOME_TAIGA_MOUNTAINS, "taiga_mountains"),
            (BIOME_SWAMP_HILLS, "swamp_hills"),
            (BIOME_ICE_SPIKES, "ice_spikes"),
            (BIOME_MODIFIED_JUNGLE, "modified_jungle"),
            (BIOME_MODIFIED_JUNGLE_EDGE, "modified_jungle_edge"),
            (BIOME_TALL_BIRCH_FOREST, "tall_birch_forest"),
            (BIOME_TALL_BIRCH_HILLS, "tall_birch_hills"),
            (BIOME_DARK_FOREST_HILLS, "dark_forest_hills"),
            (BIOME_SNOWY_TAIGA_MOUNTAINS, "snowy_taiga_mountains"),
            (BIOME_GIANT_SPRUCE_TAIGA, "giant_spruce_taiga"),
            (BIOME_GIANT_SPRUCE_TAIGA_HILLS, "giant_spruce_taiga_hills"),
            (BIOME_MODIFIED_GRAVELLY_MOUNTAINS, "modified_gravelly_mountains"),
            (BIOME_SHATTERED_SAVANNA, "shattered_savanna"),
            (BIOME_SHATTERED_SAVANNA_PLATEAU, "shattered_savanna_plateau"),
            (BIOME_ERODED_BADLANDS, "eroded_badlands"),
            (BIOME_MODIFIED_WOODED_BADLANDS_PLATEAU, "modified_wooded_badlands_plateau"),
            (BIOME_MODIFIED_BADLANDS_PLATEAU, "modified_badlands_plateau"),
        ];

        for (id, name) in biomes {
            let color = if (id as usize) < 256 {
                colors[id as usize]
            } else {
                [128, 128, 128]
            };
            entries.insert(
                id,
                BiomeEntry {
                    name: name.to_string(),
                    id,
                    visual: BiomeVisual::Color(color),
                },
            );
        }

        Self { entries }
    }

    /// Get the visual for a biome ID. Falls back to gray if not found.
    pub fn get_visual(&self, id: i32) -> &BiomeVisual {
        static FALLBACK: BiomeVisual = BiomeVisual::Color([128, 128, 128]);
        self.entries
            .get(&id)
            .map(|e| &e.visual)
            .unwrap_or(&FALLBACK)
    }

    /// Get the color for a biome ID (resolves texture entries to their base color).
    pub fn get_color(&self, id: i32) -> [u8; 3] {
        match self.get_visual(id) {
            BiomeVisual::Color(c) => *c,
            BiomeVisual::Texture(_) => {
                // If it's a texture, fall back to cubiomes default color
                let colors = default_biome_colors();
                if (id as usize) < 256 {
                    colors[id as usize]
                } else {
                    [128, 128, 128]
                }
            }
        }
    }

    /// Set a biome's visual to a solid color.
    pub fn set_color(&mut self, id: i32, color: [u8; 3]) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.visual = BiomeVisual::Color(color);
        }
    }

    /// Set a biome's visual to a texture path.
    pub fn set_texture(&mut self, id: i32, path: String) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.visual = BiomeVisual::Texture(path);
        }
    }

    /// Get biome name by ID.
    pub fn biome_name(&self, id: i32) -> &str {
        self.entries
            .get(&id)
            .map(|e| e.name.as_str())
            .unwrap_or("unknown")
    }

    /// Get all entries sorted by biome ID.
    pub fn sorted_entries(&self) -> Vec<&BiomeEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by_key(|e| e.id);
        entries
    }

    /// Convert biome chunk to RGBA pixel data using the texture map.
    /// Each biome cell becomes `tile_size × tile_size` pixels.
    pub fn chunk_to_rgba(&self, chunk: &super::generator::BiomeChunk, tile_size: u32) -> Vec<u8> {
        let img_w = chunk.width as u32 * tile_size;
        let img_h = chunk.height as u32 * tile_size;
        let mut rgba = vec![0u8; (img_w * img_h * 4) as usize];

        for bz in 0..chunk.height {
            for bx in 0..chunk.width {
                let biome_id = chunk.get(bx, bz);
                let color = self.get_color(biome_id);

                // Fill tile_size × tile_size pixels
                for ty in 0..tile_size {
                    for tx in 0..tile_size {
                        let px = bx as u32 * tile_size + tx;
                        let py = bz as u32 * tile_size + ty;
                        let idx = ((py * img_w + px) * 4) as usize;
                        rgba[idx] = color[0];
                        rgba[idx + 1] = color[1];
                        rgba[idx + 2] = color[2];
                        rgba[idx + 3] = 255;
                    }
                }
            }
        }

        rgba
    }
}
