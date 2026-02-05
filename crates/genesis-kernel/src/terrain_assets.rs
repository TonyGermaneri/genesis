//! Terrain asset system for loading and managing 48x48 pixel textures.
//!
//! This module provides:
//! - Terrain texture loading from external asset packs
//! - Pixel-level data extraction for simulation
//! - Tile position mapping for autotiling (Left, Right, Top, Bottom, Corner, etc.)
//! - Integration with the biome system
//!
//! ## Concept: Exploded Textures
//!
//! Each 48x48 texture can be "exploded" into 2304 individual pixels.
//! Each pixel maintains its original RGB value but can be modified by simulation:
//! - Burning pixels darken toward black
//! - Fire emits flames and smoke particles
//! - Wet pixels darken slightly
//! - Damaged pixels show wear

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Tile size in pixels (48x48 Modern Exteriors assets)
pub const TILE_SIZE: u32 = 48;

/// Total pixels per tile
pub const PIXELS_PER_TILE: u32 = TILE_SIZE * TILE_SIZE;

/// RGBA pixel data
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct PixelRGBA {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
}

impl PixelRGBA {
    /// Create a new pixel
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create an opaque pixel
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a transparent pixel
    #[must_use]
    pub const fn transparent() -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }

    /// Check if pixel is transparent
    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        self.a == 0
    }

    /// Apply burn effect (darken toward black)
    #[must_use]
    pub fn burned(&self, intensity: f32) -> Self {
        let factor = 1.0 - intensity.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * factor) as u8,
            g: (self.g as f32 * factor) as u8,
            b: (self.b as f32 * factor) as u8,
            a: self.a,
        }
    }

    /// Apply wet effect (darken slightly, shift toward blue)
    #[must_use]
    pub fn wet(&self, intensity: f32) -> Self {
        let factor = 1.0 - (intensity * 0.3).clamp(0.0, 0.3);
        Self {
            r: (self.r as f32 * factor) as u8,
            g: (self.g as f32 * factor) as u8,
            b: ((self.b as f32 * factor) + (intensity * 10.0)).min(255.0) as u8,
            a: self.a,
        }
    }
}

/// Position of a tile in an autotile arrangement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilePosition {
    /// Center/fill tile (can be used anywhere)
    Center,
    /// Top edge
    Top,
    /// Bottom edge
    Bottom,
    /// Left edge
    Left,
    /// Right edge
    Right,
    /// Top-left corner (outer)
    TopLeft,
    /// Top-right corner (outer)
    TopRight,
    /// Bottom-left corner (outer)
    BottomLeft,
    /// Bottom-right corner (outer)
    BottomRight,
    /// Top-left inner corner
    TopLeftInner,
    /// Top-right inner corner
    TopRightInner,
    /// Bottom-left inner corner
    BottomLeftInner,
    /// Bottom-right inner corner
    BottomRightInner,
    /// Modular/repeating variant
    Modular,
    /// Diagonal transition
    Diagonal,
    /// Single/isolated tile
    Single,
}

impl TilePosition {
    /// Parse tile position from filename keywords
    #[must_use]
    pub fn from_filename(name: &str) -> Self {
        let name_lower = name.to_lowercase();

        // Check for inner corners first (more specific)
        if name_lower.contains("inner") {
            if name_lower.contains("top") && name_lower.contains("left") {
                return Self::TopLeftInner;
            }
            if name_lower.contains("top") && name_lower.contains("right") {
                return Self::TopRightInner;
            }
            if name_lower.contains("bottom") && name_lower.contains("left") {
                return Self::BottomLeftInner;
            }
            if name_lower.contains("bottom") && name_lower.contains("right") {
                return Self::BottomRightInner;
            }
        }

        // Check for outer corners
        if (name_lower.contains("top") || name_lower.contains("up"))
            && (name_lower.contains("left"))
        {
            return Self::TopLeft;
        }
        if (name_lower.contains("top") || name_lower.contains("up"))
            && (name_lower.contains("right"))
        {
            return Self::TopRight;
        }
        if (name_lower.contains("bottom") || name_lower.contains("down"))
            && (name_lower.contains("left"))
        {
            return Self::BottomLeft;
        }
        if (name_lower.contains("bottom") || name_lower.contains("down"))
            && (name_lower.contains("right"))
        {
            return Self::BottomRight;
        }

        // Check for edges
        if name_lower.contains("left_side") || name_lower.contains("_left.") {
            return Self::Left;
        }
        if name_lower.contains("right_side") || name_lower.contains("_right.") {
            return Self::Right;
        }
        if name_lower.contains("_top") || name_lower.contains("top_") {
            return Self::Top;
        }
        if name_lower.contains("_bottom") || name_lower.contains("bottom_") {
            return Self::Bottom;
        }

        // Check for modular/diagonal
        if name_lower.contains("modular") || name_lower.contains("middle") {
            return Self::Modular;
        }
        if name_lower.contains("diagonal") {
            return Self::Diagonal;
        }

        // Check for single
        if name_lower.contains("single") {
            return Self::Single;
        }

        // Default to center
        Self::Center
    }
}

/// Terrain type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainCategory {
    /// Grass terrain (multiple variants)
    Grass,
    /// Water (deep and shallow)
    Water,
    /// Sand/beach terrain
    Sand,
    /// Dirt/soil terrain
    Dirt,
    /// Stone/rock terrain
    Stone,
    /// Asphalt/road terrain
    Asphalt,
    /// Sidewalk/paved terrain
    Sidewalk,
    /// Mound/hill terrain
    Mound,
    /// Snow terrain
    Snow,
    /// Swamp terrain
    Swamp,
}

impl TerrainCategory {
    /// Parse terrain category from filename
    #[must_use]
    pub fn from_filename(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase();

        if name_lower.contains("grass") {
            return Some(Self::Grass);
        }
        if name_lower.contains("water") || name_lower.contains("deep_water") {
            return Some(Self::Water);
        }
        if name_lower.contains("sand") || name_lower.contains("beach") {
            return Some(Self::Sand);
        }
        if name_lower.contains("dirt") {
            return Some(Self::Dirt);
        }
        if name_lower.contains("stone") || name_lower.contains("rock") {
            return Some(Self::Stone);
        }
        if name_lower.contains("asphalt") {
            return Some(Self::Asphalt);
        }
        if name_lower.contains("sidewalk") {
            return Some(Self::Sidewalk);
        }
        if name_lower.contains("mound") {
            return Some(Self::Mound);
        }
        if name_lower.contains("snow") {
            return Some(Self::Snow);
        }
        if name_lower.contains("swamp") {
            return Some(Self::Swamp);
        }

        None
    }

    /// Get all terrain categories
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Grass,
            Self::Water,
            Self::Sand,
            Self::Dirt,
            Self::Stone,
            Self::Asphalt,
            Self::Sidewalk,
            Self::Mound,
            Self::Snow,
            Self::Swamp,
        ]
    }
}

/// A single terrain tile with its pixel data
#[derive(Debug, Clone)]
pub struct TerrainTile {
    /// Unique identifier
    pub id: u32,
    /// Source filename
    pub filename: String,
    /// Terrain category
    pub category: TerrainCategory,
    /// Position in autotile arrangement
    pub position: TilePosition,
    /// Variant number (for randomization)
    pub variant: u32,
    /// Raw pixel data (48x48 = 2304 pixels)
    pub pixels: Vec<PixelRGBA>,
}

impl TerrainTile {
    /// Create a new terrain tile (placeholder with no pixel data)
    #[must_use]
    pub fn new(
        id: u32,
        filename: String,
        category: TerrainCategory,
        position: TilePosition,
        variant: u32,
    ) -> Self {
        Self {
            id,
            filename,
            category,
            position,
            variant,
            pixels: Vec::new(),
        }
    }

    /// Get pixel at local coordinates (0-47)
    #[must_use]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<PixelRGBA> {
        if x >= TILE_SIZE || y >= TILE_SIZE {
            return None;
        }
        let idx = (y * TILE_SIZE + x) as usize;
        self.pixels.get(idx).copied()
    }

    /// Check if tile has pixel data loaded
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        self.pixels.len() == PIXELS_PER_TILE as usize
    }
}

/// Terrain asset manifest - catalog of all available terrain textures
#[derive(Debug, Default)]
pub struct TerrainAssetManifest {
    /// All registered tiles
    tiles: Vec<TerrainTile>,
    /// Lookup by category and position
    category_index: HashMap<(TerrainCategory, TilePosition), Vec<usize>>,
    /// Base path for assets
    base_path: PathBuf,
}

impl TerrainAssetManifest {
    /// Create a new empty manifest
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base path for assets
    pub fn set_base_path<P: AsRef<Path>>(&mut self, path: P) {
        self.base_path = path.as_ref().to_path_buf();
    }

    /// Get the base path
    #[must_use]
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Register a tile from a filename
    pub fn register_tile(&mut self, filename: &str) -> Option<u32> {
        // Parse category
        let category = TerrainCategory::from_filename(filename)?;

        // Parse position
        let position = TilePosition::from_filename(filename);

        // Parse variant number (look for _N. pattern at end)
        let variant = Self::parse_variant(filename);

        // Create tile
        let id = self.tiles.len() as u32;
        let tile = TerrainTile::new(id, filename.to_string(), category, position, variant);
        self.tiles.push(tile);

        // Update index
        let key = (category, position);
        self.category_index.entry(key).or_default().push(id as usize);

        Some(id)
    }

    /// Parse variant number from filename (e.g., "_1.png" -> 1)
    fn parse_variant(filename: &str) -> u32 {
        // Look for pattern like _N.png or _N_Variation_M.png
        let name = filename.trim_end_matches(".png");
        if let Some(last_part) = name.rsplit('_').next() {
            if let Ok(n) = last_part.parse::<u32>() {
                return n;
            }
        }
        0
    }

    /// Get all tiles for a category
    #[must_use]
    pub fn get_tiles_by_category(&self, category: TerrainCategory) -> Vec<&TerrainTile> {
        self.tiles
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Get tiles for a category and position
    #[must_use]
    pub fn get_tiles(&self, category: TerrainCategory, position: TilePosition) -> Vec<&TerrainTile> {
        let key = (category, position);
        self.category_index
            .get(&key)
            .map(|indices| indices.iter().filter_map(|&i| self.tiles.get(i)).collect())
            .unwrap_or_default()
    }

    /// Get a random tile for a category and position
    #[must_use]
    pub fn get_random_tile(
        &self,
        category: TerrainCategory,
        position: TilePosition,
        seed: u64,
    ) -> Option<&TerrainTile> {
        let tiles = self.get_tiles(category, position);
        if tiles.is_empty() {
            // Fallback to center/modular tiles
            let fallback_tiles = self.get_tiles(category, TilePosition::Center);
            if fallback_tiles.is_empty() {
                return None;
            }
            let idx = (seed % fallback_tiles.len() as u64) as usize;
            return Some(fallback_tiles[idx]);
        }
        let idx = (seed % tiles.len() as u64) as usize;
        Some(tiles[idx])
    }

    /// Get tile by ID
    #[must_use]
    pub fn get_tile(&self, id: u32) -> Option<&TerrainTile> {
        self.tiles.get(id as usize)
    }

    /// Get mutable tile by ID
    pub fn get_tile_mut(&mut self, id: u32) -> Option<&mut TerrainTile> {
        self.tiles.get_mut(id as usize)
    }

    /// Total number of registered tiles
    #[must_use]
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Scan a directory and register all terrain tiles
    pub fn scan_directory<P: AsRef<Path>>(&mut self, dir: P) -> std::io::Result<usize> {
        let dir_path = dir.as_ref();
        let mut count = 0;

        if !dir_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Directory not found: {:?}", dir_path),
            ));
        }

        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("png") {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if self.register_tile(filename).is_some() {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }
}

/// Mapping from biome IDs to terrain categories
#[derive(Debug, Clone)]
pub struct BiomeTerrainMapping {
    mappings: HashMap<u8, Vec<(TerrainCategory, f32)>>,
}

impl Default for BiomeTerrainMapping {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeTerrainMapping {
    /// Create default biome-terrain mappings
    #[must_use]
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Plains (biome 0) - primarily grass
        mappings.insert(0, vec![(TerrainCategory::Grass, 1.0)]);

        // Forest (biome 1) - grass with dirt patches
        mappings.insert(1, vec![
            (TerrainCategory::Grass, 0.7),
            (TerrainCategory::Dirt, 0.3),
        ]);

        // Desert (biome 2) - sand
        mappings.insert(2, vec![(TerrainCategory::Sand, 1.0)]);

        // Snow (biome 3) - snow terrain
        mappings.insert(3, vec![(TerrainCategory::Snow, 1.0)]);

        // Swamp (biome 4) - mix of grass and water
        mappings.insert(4, vec![
            (TerrainCategory::Swamp, 0.5),
            (TerrainCategory::Water, 0.3),
            (TerrainCategory::Grass, 0.2),
        ]);

        // Mountain (biome 5) - stone
        mappings.insert(5, vec![
            (TerrainCategory::Stone, 0.7),
            (TerrainCategory::Dirt, 0.3),
        ]);

        // Cave (biome 6) - stone
        mappings.insert(6, vec![(TerrainCategory::Stone, 1.0)]);

        // Ocean (biome 7) - deep water
        mappings.insert(7, vec![(TerrainCategory::Water, 1.0)]);

        // Beach (biome 8) - sand
        mappings.insert(8, vec![
            (TerrainCategory::Sand, 0.8),
            (TerrainCategory::Grass, 0.2),
        ]);

        Self { mappings }
    }

    /// Get terrain category for a biome with weighted random selection
    #[must_use]
    pub fn get_terrain(&self, biome_id: u8, rand_value: f32) -> TerrainCategory {
        if let Some(options) = self.mappings.get(&biome_id) {
            let mut cumulative = 0.0;
            for (category, weight) in options {
                cumulative += weight;
                if rand_value < cumulative {
                    return *category;
                }
            }
            // Return last option if we somehow didn't match
            if let Some((category, _)) = options.last() {
                return *category;
            }
        }
        // Default to grass
        TerrainCategory::Grass
    }

    /// Get primary terrain for a biome
    #[must_use]
    pub fn get_primary_terrain(&self, biome_id: u8) -> TerrainCategory {
        self.mappings
            .get(&biome_id)
            .and_then(|v| v.first())
            .map(|(cat, _)| *cat)
            .unwrap_or(TerrainCategory::Grass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_position_parsing() {
        assert_eq!(
            TilePosition::from_filename("21_Beach_48x48_Sand_Mountain_Big_Left_Side_Modular.png"),
            TilePosition::Left
        );
        assert_eq!(
            TilePosition::from_filename("ME_Singles_Terrains_and_Fences_48x48_Grass_1_1.png"),
            TilePosition::Center
        );
        assert_eq!(
            TilePosition::from_filename("Some_Tile_Top_Left.png"),
            TilePosition::TopLeft
        );
        assert_eq!(
            TilePosition::from_filename("Tile_Bottom_Right_Corner.png"),
            TilePosition::BottomRight
        );
    }

    #[test]
    fn test_terrain_category_parsing() {
        assert_eq!(
            TerrainCategory::from_filename("ME_Singles_Terrains_and_Fences_48x48_Grass_1_1.png"),
            Some(TerrainCategory::Grass)
        );
        assert_eq!(
            TerrainCategory::from_filename("ME_Singles_Terrains_and_Fences_48x48_Deep_Water_1_1.png"),
            Some(TerrainCategory::Water)
        );
        assert_eq!(
            TerrainCategory::from_filename("21_Beach_48x48_Sand_Castle.png"),
            Some(TerrainCategory::Sand)
        );
    }

    #[test]
    fn test_pixel_effects() {
        let pixel = PixelRGBA::rgb(100, 200, 100);

        let burned = pixel.burned(0.5);
        assert_eq!(burned.r, 50);
        assert_eq!(burned.g, 100);
        assert_eq!(burned.b, 50);

        let wet = pixel.wet(1.0);
        assert!(wet.r < pixel.r);
        assert!(wet.b >= pixel.b); // Blue increases slightly
    }

    #[test]
    fn test_biome_terrain_mapping() {
        let mapping = BiomeTerrainMapping::new();

        // Plains should be grass
        assert_eq!(mapping.get_primary_terrain(0), TerrainCategory::Grass);

        // Desert should be sand
        assert_eq!(mapping.get_primary_terrain(2), TerrainCategory::Sand);

        // Ocean should be water
        assert_eq!(mapping.get_primary_terrain(7), TerrainCategory::Water);
    }

    #[test]
    fn test_manifest_registration() {
        let mut manifest = TerrainAssetManifest::new();

        let id = manifest.register_tile("ME_Singles_Terrains_and_Fences_48x48_Grass_1_5.png");
        assert!(id.is_some());

        let tile = manifest.get_tile(id.unwrap()).unwrap();
        assert_eq!(tile.category, TerrainCategory::Grass);
        assert_eq!(tile.variant, 5);
    }
}
