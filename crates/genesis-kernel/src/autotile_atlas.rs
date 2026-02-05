//! Autotile-based terrain atlas system.
//!
//! This module provides terrain rendering using pre-organized autotile sheets
//! where each terrain type has 47/48 pre-computed edge/corner combinations.
//!
//! ## Autotile Layout (GameMaker/Godot 47-tile blob format)
//!
//! The autotile sheet is 576×4992 pixels:
//! - 12 tiles wide × 4 tiles tall per terrain = 48 tiles/terrain
//! - 26 terrain types stacked vertically
//! - Each tile is 48×48 pixels
//!
//! ## 8-bit Bitmask Format
//!
//! Each tile's appearance is determined by an 8-bit neighbor bitmask:
//! ```text
//! NW(1)  N(2)  NE(4)
//!  W(8)   *   E(16)
//! SW(32) S(64) SE(128)
//! ```
//!
//! Corner bits are only counted if both adjacent cardinal neighbors exist.
//! This reduces 256 combinations to 47 unique tiles.
//!
//! ## Tile Index Calculation
//!
//! For a given terrain type and neighbor configuration:
//! - `terrain_row = terrain_type` (row 0-25)
//! - `tile_index = BITMASK_TO_TILE[effective_mask]` (0-46)
//! - `atlas_x = (tile_index % 12) * 48`
//! - `atlas_y = (terrain_row * 4 + tile_index / 12) * 48`

use std::path::Path;

use bytemuck::{Pod, Zeroable};
use tracing::info;
use wgpu::{Device, Queue};

use crate::terrain_assets::PixelRGBA;

/// Tile size in pixels
pub const TILE_SIZE: u32 = 48;

/// Tiles per row in autotile strip
pub const TILES_PER_STRIP_ROW: u32 = 12;

/// Rows per terrain strip
pub const ROWS_PER_TERRAIN: u32 = 4;

/// Tiles per terrain type
pub const TILES_PER_TERRAIN: u32 = TILES_PER_STRIP_ROW * ROWS_PER_TERRAIN; // 48

/// Total terrain types in the autotile sheet
pub const TERRAIN_COUNT: u32 = 26;

/// Autotile sheet dimensions
pub const AUTOTILE_WIDTH: u32 = TILES_PER_STRIP_ROW * TILE_SIZE; // 576
pub const AUTOTILE_HEIGHT: u32 = TERRAIN_COUNT * ROWS_PER_TERRAIN * TILE_SIZE; // 4992

/// Terrain type identifiers (based on visual inspection of autotile order)
/// These map to rows 0-25 in the autotile sheet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AutotileTerrainType {
    /// Row 0: Light grass
    GrassLight = 0,
    /// Row 1: Medium grass
    GrassMedium = 1,
    /// Row 2: Dark grass
    GrassDark = 2,
    /// Row 3: Very dark grass / forest floor
    GrassForest = 3,
    /// Row 4: Grass with water edge (transition)
    GrassWater1 = 4,
    /// Row 5: Grass with water edge variant 2
    GrassWater2 = 5,
    /// Row 6: Grass with water edge variant 3
    GrassWater3 = 6,
    /// Row 7: Grass with water edge variant 4
    GrassWater4 = 7,
    /// Row 8: Fenced grass
    GrassFenced = 8,
    /// Row 9: Deep water
    DeepWater = 9,
    /// Row 10: Fence type 1
    Fence1 = 10,
    /// Row 11: Fence type 2
    Fence2 = 11,
    /// Row 12: Fence type 3
    Fence3 = 12,
    /// Row 13: Mound type 1
    Mound1 = 13,
    /// Row 14: Mound type 2
    Mound2 = 14,
    /// Row 15: Wall type 1
    Wall1 = 15,
    /// Row 16: Wall type 2
    Wall2 = 16,
    /// Row 17: Wall type 3
    Wall3 = 17,
    /// Row 18: Dirt/path
    Dirt = 18,
    /// Row 19: Reserved (grass props)
    GrassProps = 19,
    /// Row 20: Reserved (fence props)
    FenceProps = 20,
    /// Row 21: Reserved (wall props)
    WallProps = 21,
    /// Row 22: Reserved (water props)
    WaterProps = 22,
    /// Row 23: Reserved (dirt props)
    DirtProps = 23,
    /// Row 24: Other 1
    Other1 = 24,
    /// Row 25: Other 2
    Other2 = 25,
}

impl AutotileTerrainType {
    /// Get terrain type from biome ID
    #[must_use]
    pub fn from_biome(biome_id: u8) -> Self {
        match biome_id {
            0 => Self::GrassMedium,    // Forest - medium grass
            1 => Self::Dirt,           // Desert - dirt (no sand autotile)
            2 => Self::GrassDark,      // Cave - dark (use dark grass as base)
            3 => Self::DeepWater,      // Ocean - deep water
            4 => Self::GrassLight,     // Plains - light grass
            5 => Self::Mound1,         // Mountain - mound/rocky
            6 => Self::GrassForest,    // Swamp - forest floor
            7 => Self::GrassWater1,    // River - grass/water transition
            8 => Self::GrassFenced,    // Farm - fenced grass
            _ => Self::GrassMedium,    // Default
        }
    }

    /// Get row index in the autotile sheet
    #[must_use]
    pub const fn row_index(self) -> u32 {
        self as u32
    }
}

/// Neighbor mask for autotile selection
/// Bits represent adjacent cells of the same terrain type
///
/// Standard 8-bit blob autotile format:
/// ```text
/// NW(1)  N(2)  NE(4)
///  W(8)   *   E(16)
/// SW(32) S(64) SE(128)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AutotileNeighborMask(pub u8);

impl AutotileNeighborMask {
    // Standard 8-bit blob format (NW=1, N=2, NE=4, W=8, E=16, SW=32, S=64, SE=128)
    pub const NW: u8 = 1;
    pub const NORTH: u8 = 2;
    pub const NE: u8 = 4;
    pub const WEST: u8 = 8;
    pub const EAST: u8 = 16;
    pub const SW: u8 = 32;
    pub const SOUTH: u8 = 64;
    pub const SE: u8 = 128;

    /// All neighbors present (center fill)
    pub const ALL: Self = Self(255);
    /// No neighbors (isolated tile)
    pub const NONE: Self = Self(0);

    /// Create mask from neighbor flags
    /// Corner bits are only set if BOTH adjacent cardinal neighbors exist
    #[must_use]
    pub const fn new(n: bool, e: bool, s: bool, w: bool, ne: bool, se: bool, sw: bool, nw: bool) -> Self {
        let mut mask = 0u8;

        // Cardinals are always included
        if n { mask |= Self::NORTH; }
        if e { mask |= Self::EAST; }
        if s { mask |= Self::SOUTH; }
        if w { mask |= Self::WEST; }

        // Corners only count if BOTH adjacent cardinals are present
        if nw && n && w { mask |= Self::NW; }
        if ne && n && e { mask |= Self::NE; }
        if sw && s && w { mask |= Self::SW; }
        if se && s && e { mask |= Self::SE; }

        Self(mask)
    }

    /// Convert 8-bit mask to tile index (0-46) in the autotile strip
    /// Uses the standard 47-tile blob lookup table
    #[must_use]
    pub fn to_tile_index(self) -> u32 {
        BITMASK_TO_TILE[self.0 as usize] as u32
    }
}

/// Lookup table: 8-bit neighbor bitmask → tile index (0-46)
///
/// The 47-tile blob format reduces 256 possible combinations to 47 unique tiles
/// by ignoring corners when adjacent cardinals are missing.
///
/// This table maps each of the 256 possible bitmask values to the correct tile index.
/// Many bitmasks map to the same tile (because corner bits are ignored when
/// their adjacent cardinals are not set).
pub const BITMASK_TO_TILE: [u8; 256] = generate_bitmask_lut();

/// Generate the 256-entry lookup table at compile time
const fn generate_bitmask_lut() -> [u8; 256] {
    // The standard 47-tile blob autotile has tiles arranged to match specific
    // neighbor configurations. We build the LUT by computing the "effective"
    // bitmask (with corner-removal) and mapping to sequential tile indices.

    // First, the 47 unique effective masks in order:
    const UNIQUE_MASKS: [u8; 47] = [
        0,   // 0: Isolated (no neighbors)
        2,   // 1: N only
        8,   // 2: W only
        10,  // 3: N+W
        11,  // 4: N+W+NW
        16,  // 5: E only
        18,  // 6: N+E
        22,  // 7: N+E+NE
        24,  // 8: W+E
        26,  // 9: N+W+E
        27,  // 10: N+W+E+NW
        30,  // 11: N+E+W+NE
        31,  // 12: N+E+W+NW+NE
        64,  // 13: S only
        66,  // 14: N+S
        72,  // 15: W+S
        74,  // 16: N+W+S
        75,  // 17: N+W+S+NW
        80,  // 18: E+S
        82,  // 19: N+E+S
        86,  // 20: N+E+S+NE
        88,  // 21: W+E+S
        90,  // 22: N+W+E+S
        91,  // 23: N+W+E+S+NW
        94,  // 24: N+W+E+S+NE
        95,  // 25: N+W+E+S+NW+NE
        104, // 26: W+S+SW
        106, // 27: N+W+S+SW
        107, // 28: N+W+S+NW+SW
        120, // 29: W+E+S+SW
        122, // 30: N+W+E+S+SW
        123, // 31: N+W+E+S+NW+SW
        126, // 32: N+W+E+S+NE+SW
        127, // 33: N+W+E+S+NW+NE+SW
        208, // 34: E+S+SE
        210, // 35: N+E+S+SE
        214, // 36: N+E+S+NE+SE
        216, // 37: W+E+S+SE
        218, // 38: N+W+E+S+SE
        219, // 39: N+W+E+S+NW+SE
        222, // 40: N+W+E+S+NE+SE
        223, // 41: N+W+E+S+NW+NE+SE
        248, // 42: W+E+S+SW+SE
        250, // 43: N+W+E+S+SW+SE
        251, // 44: N+W+E+S+NW+SW+SE
        254, // 45: N+W+E+S+NE+SW+SE
        255, // 46: All neighbors (center fill)
    ];

    let mut lut = [0u8; 256];
    let mut i = 0u16;

    while i < 256 {
        let mask = i as u8;

        // Compute effective mask by removing corners without adjacent cardinals
        let n = (mask & 2) != 0;
        let e = (mask & 16) != 0;
        let s = (mask & 64) != 0;
        let w = (mask & 8) != 0;

        let mut effective = mask & (2 | 8 | 16 | 64); // Start with cardinals only

        // Add corners only if both adjacent cardinals are present
        if n && w && (mask & 1) != 0 { effective |= 1; }   // NW
        if n && e && (mask & 4) != 0 { effective |= 4; }   // NE
        if s && w && (mask & 32) != 0 { effective |= 32; } // SW
        if s && e && (mask & 128) != 0 { effective |= 128; } // SE

        // Find this effective mask in UNIQUE_MASKS
        let mut tile_idx = 0u8;
        let mut j = 0;
        while j < 47 {
            if UNIQUE_MASKS[j] == effective {
                tile_idx = j as u8;
                break;
            }
            j += 1;
        }

        lut[i as usize] = tile_idx;
        i += 1;
    }

    lut
}

/// GPU-compatible autotile atlas parameters
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct AutotileAtlasParams {
    /// Tile size in pixels (48)
    pub tile_size: u32,
    /// Tiles per row in strip (12)
    pub tiles_per_strip_row: u32,
    /// Rows per terrain (4)
    pub rows_per_terrain: u32,
    /// Total terrain types (26)
    pub terrain_count: u32,
    /// Atlas width in pixels (576)
    pub atlas_width: u32,
    /// Atlas height in pixels (4992)
    pub atlas_height: u32,
    /// Padding for alignment
    pub _padding: [u32; 2],
}

impl Default for AutotileAtlasParams {
    fn default() -> Self {
        Self {
            tile_size: TILE_SIZE,
            tiles_per_strip_row: TILES_PER_STRIP_ROW,
            rows_per_terrain: ROWS_PER_TERRAIN,
            terrain_count: TERRAIN_COUNT,
            atlas_width: AUTOTILE_WIDTH,
            atlas_height: AUTOTILE_HEIGHT,
            _padding: [0; 2],
        }
    }
}

/// Autotile texture atlas
pub struct AutotileAtlas {
    /// GPU texture
    texture: Option<wgpu::Texture>,
    /// Texture view for shader binding
    texture_view: Option<wgpu::TextureView>,
    /// Sampler
    sampler: Option<wgpu::Sampler>,
    /// CPU-side pixel data
    atlas_data: Vec<PixelRGBA>,
    /// Atlas parameters
    params: AutotileAtlasParams,
    /// Whether atlas is loaded
    loaded: bool,
}

impl AutotileAtlas {
    /// Create a new empty atlas
    #[must_use]
    pub fn new() -> Self {
        Self {
            texture: None,
            texture_view: None,
            sampler: None,
            atlas_data: Vec::new(),
            params: AutotileAtlasParams::default(),
            loaded: false,
        }
    }

    /// Load autotile atlas from a PNG file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();

        info!("Loading autotile atlas from: {}", path.display());

        // Read and decode image using the image crate
        let file_data = std::fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let img = image::load_from_memory(&file_data)
            .map_err(|e| format!("Failed to decode image: {}", e))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        if width != AUTOTILE_WIDTH || height != AUTOTILE_HEIGHT {
            return Err(format!(
                "Invalid autotile dimensions: {}x{} (expected {}x{})",
                width, height, AUTOTILE_WIDTH, AUTOTILE_HEIGHT
            ));
        }

        // Convert to PixelRGBA
        let pixel_count = (width * height) as usize;
        self.atlas_data = Vec::with_capacity(pixel_count);

        for pixel in rgba.pixels() {
            self.atlas_data.push(PixelRGBA::new(
                pixel[0],
                pixel[1],
                pixel[2],
                pixel[3],
            ));
        }

        self.params.atlas_width = width;
        self.params.atlas_height = height;
        self.loaded = true;

        info!(
            "Loaded autotile atlas: {}x{} ({} terrain types, {} tiles each)",
            width, height, TERRAIN_COUNT, TILES_PER_TERRAIN
        );

        Ok(())
    }

    /// Upload atlas to GPU
    pub fn upload_to_gpu(&mut self, device: &Device, queue: &Queue) -> Result<(), String> {
        if !self.loaded || self.atlas_data.is_empty() {
            return Err("No atlas data to upload".to_string());
        }

        // Create texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("autotile_atlas"),
            size: wgpu::Extent3d {
                width: self.params.atlas_width,
                height: self.params.atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Convert pixels to bytes
        let bytes: Vec<u8> = self
            .atlas_data
            .iter()
            .flat_map(|p| [p.r, p.g, p.b, p.a])
            .collect();

        // Upload to GPU
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.params.atlas_width * 4),
                rows_per_image: Some(self.params.atlas_height),
            },
            wgpu::Extent3d {
                width: self.params.atlas_width,
                height: self.params.atlas_height,
                depth_or_array_layers: 1,
            },
        );

        // Create view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler (nearest neighbor for pixel art)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("autotile_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.texture = Some(texture);
        self.texture_view = Some(view);
        self.sampler = Some(sampler);

        info!("Uploaded autotile atlas to GPU: {}x{}",
            self.params.atlas_width, self.params.atlas_height);

        Ok(())
    }

    /// Get texture view for shader binding
    pub fn texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }

    /// Get sampler for shader binding
    pub fn sampler(&self) -> Option<&wgpu::Sampler> {
        self.sampler.as_ref()
    }

    /// Get atlas parameters for uniform buffer
    pub fn params(&self) -> AutotileAtlasParams {
        self.params
    }

    /// Check if atlas is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get pixel at atlas coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<PixelRGBA> {
        if x >= self.params.atlas_width || y >= self.params.atlas_height {
            return None;
        }
        let idx = (y * self.params.atlas_width + x) as usize;
        self.atlas_data.get(idx).copied()
    }

    /// Sample a specific tile at local coordinates
    pub fn sample_tile(
        &self,
        terrain_type: AutotileTerrainType,
        tile_index: u32,
        local_x: u32,
        local_y: u32,
    ) -> Option<PixelRGBA> {
        if tile_index >= TILES_PER_TERRAIN || local_x >= TILE_SIZE || local_y >= TILE_SIZE {
            return None;
        }

        let terrain_row = terrain_type.row_index();
        let strip_col = tile_index % TILES_PER_STRIP_ROW;
        let strip_row = tile_index / TILES_PER_STRIP_ROW;

        let atlas_x = strip_col * TILE_SIZE + local_x;
        let atlas_y = (terrain_row * ROWS_PER_TERRAIN + strip_row) * TILE_SIZE + local_y;

        self.get_pixel(atlas_x, atlas_y)
    }
}

impl Default for AutotileAtlas {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to compute neighbor mask from surrounding biomes
pub fn compute_neighbor_mask(
    center_biome: u8,
    n: u8, e: u8, s: u8, w: u8,
    ne: u8, se: u8, sw: u8, nw: u8,
) -> AutotileNeighborMask {
    AutotileNeighborMask::new(
        n == center_biome,
        e == center_biome,
        s == center_biome,
        w == center_biome,
        ne == center_biome,
        se == center_biome,
        sw == center_biome,
        nw == center_biome,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_type_from_biome() {
        assert_eq!(AutotileTerrainType::from_biome(0), AutotileTerrainType::GrassMedium);
        assert_eq!(AutotileTerrainType::from_biome(3), AutotileTerrainType::DeepWater);
        assert_eq!(AutotileTerrainType::from_biome(5), AutotileTerrainType::Mound1);
    }

    #[test]
    fn test_neighbor_mask_all() {
        let mask = AutotileNeighborMask::ALL;
        let tile_idx = mask.to_tile_index();
        // All neighbors = tile 46 (last tile, center fill)
        assert_eq!(tile_idx, 46);
    }

    #[test]
    fn test_neighbor_mask_none() {
        let mask = AutotileNeighborMask::NONE;
        let tile_idx = mask.to_tile_index();
        // No neighbors = tile 0 (isolated)
        assert_eq!(tile_idx, 0);
    }

    #[test]
    fn test_bitmask_lut_unique_masks() {
        // Test that the unique mask values map to sequential tiles
        assert_eq!(BITMASK_TO_TILE[0], 0);    // Isolated
        assert_eq!(BITMASK_TO_TILE[2], 1);    // N only
        assert_eq!(BITMASK_TO_TILE[8], 2);    // W only
        assert_eq!(BITMASK_TO_TILE[64], 13);  // S only
        assert_eq!(BITMASK_TO_TILE[16], 5);   // E only
        assert_eq!(BITMASK_TO_TILE[255], 46); // All neighbors
    }

    #[test]
    fn test_corner_removal() {
        // NW corner bit set but N is missing → NW should be ignored
        // mask = NW(1) + W(8) = 9
        // effective = W only = 8 → tile 2
        assert_eq!(BITMASK_TO_TILE[9], 2);

        // NW corner bit set with N and W present → NW should be included
        // mask = NW(1) + N(2) + W(8) = 11 → tile 4
        assert_eq!(BITMASK_TO_TILE[11], 4);
    }

    #[test]
    fn test_atlas_params() {
        let params = AutotileAtlasParams::default();
        assert_eq!(params.tile_size, 48);
        assert_eq!(params.tiles_per_strip_row, 12);
        assert_eq!(params.terrain_count, 26);
        assert_eq!(params.atlas_width, 576);
        assert_eq!(params.atlas_height, 4992);
    }
}
