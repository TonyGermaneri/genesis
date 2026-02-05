//! GPU Terrain Texture Atlas for pixel-perfect terrain rendering.
//!
//! This module provides:
//! - GPU texture atlas creation from loaded terrain tiles
//! - Shader bindings for sampling terrain textures
//! - Pixel-level simulation effects (burn, wet, damage)
//! - Autotile selection based on neighbor patterns

use std::collections::HashMap;
use std::path::Path;

use bytemuck::{Pod, Zeroable};
use tracing::{debug, info, warn};
use wgpu::{Device, Queue};

use crate::terrain_assets::{
    BiomeTerrainMapping, PixelRGBA, TerrainAssetManifest, TerrainCategory,
    TilePosition, TILE_SIZE,
};
use crate::texture_loader::{load_manifest_textures, TextureLoaderConfig};

/// Maximum tiles in the atlas (32x32 = 1024 tiles)
pub const MAX_ATLAS_TILES: u32 = 1024;

/// Atlas dimensions (32 tiles * 48 pixels = 1536 pixels)
pub const ATLAS_TILES_PER_ROW: u32 = 32;
pub const ATLAS_SIZE: u32 = ATLAS_TILES_PER_ROW * TILE_SIZE;

/// GPU-compatible tile metadata
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TileMetadata {
    /// Atlas X offset in pixels
    pub atlas_x: u32,
    /// Atlas Y offset in pixels
    pub atlas_y: u32,
    /// Terrain category (see TerrainCategory enum)
    pub category: u32,
    /// Tile position (see TilePosition enum)
    pub position: u32,
}

/// GPU-compatible terrain atlas parameters
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainAtlasParams {
    /// Tile size in pixels (48)
    pub tile_size: u32,
    /// Total number of tiles in atlas
    pub tile_count: u32,
    /// Atlas width in pixels
    pub atlas_width: u32,
    /// Atlas height in pixels
    pub atlas_height: u32,
}

/// Autotile neighbor mask bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeighborMask(pub u8);

impl NeighborMask {
    /// No neighbors of same type
    pub const NONE: Self = Self(0);
    /// North neighbor
    pub const NORTH: u8 = 0b0000_0001;
    /// South neighbor
    pub const SOUTH: u8 = 0b0000_0010;
    /// East neighbor
    pub const EAST: u8 = 0b0000_0100;
    /// West neighbor
    pub const WEST: u8 = 0b0000_1000;
    /// North-East corner
    pub const NE: u8 = 0b0001_0000;
    /// North-West corner
    pub const NW: u8 = 0b0010_0000;
    /// South-East corner
    pub const SE: u8 = 0b0100_0000;
    /// South-West corner
    pub const SW: u8 = 0b1000_0000;

    /// Create from individual flags
    #[must_use]
    pub const fn new(n: bool, s: bool, e: bool, w: bool, ne: bool, nw: bool, se: bool, sw: bool) -> Self {
        let mut mask = 0u8;
        if n { mask |= Self::NORTH; }
        if s { mask |= Self::SOUTH; }
        if e { mask |= Self::EAST; }
        if w { mask |= Self::WEST; }
        if ne { mask |= Self::NE; }
        if nw { mask |= Self::NW; }
        if se { mask |= Self::SE; }
        if sw { mask |= Self::SW; }
        Self(mask)
    }

    /// Get appropriate tile position for this neighbor configuration
    #[must_use]
    pub fn to_tile_position(&self) -> TilePosition {
        let has_n = self.0 & Self::NORTH != 0;
        let has_s = self.0 & Self::SOUTH != 0;
        let has_e = self.0 & Self::EAST != 0;
        let has_w = self.0 & Self::WEST != 0;

        // Check for edge tiles
        if !has_n && has_s && has_e && has_w {
            return TilePosition::Top;
        }
        if has_n && !has_s && has_e && has_w {
            return TilePosition::Bottom;
        }
        if has_n && has_s && !has_e && has_w {
            return TilePosition::Right;
        }
        if has_n && has_s && has_e && !has_w {
            return TilePosition::Left;
        }

        // Check for corner tiles
        if !has_n && !has_w && has_s && has_e {
            return TilePosition::TopLeft;
        }
        if !has_n && !has_e && has_s && has_w {
            return TilePosition::TopRight;
        }
        if !has_s && !has_w && has_n && has_e {
            return TilePosition::BottomLeft;
        }
        if !has_s && !has_e && has_n && has_w {
            return TilePosition::BottomRight;
        }

        // Check for single/isolated
        if !has_n && !has_s && !has_e && !has_w {
            return TilePosition::Single;
        }

        // Default to center for all-surrounded
        TilePosition::Center
    }
}

/// Terrain Texture Atlas - manages GPU textures and tile lookups
pub struct TerrainTextureAtlas {
    /// The GPU texture
    texture: Option<wgpu::Texture>,
    /// Texture view for shader binding
    texture_view: Option<wgpu::TextureView>,
    /// Sampler for texture lookups
    sampler: Option<wgpu::Sampler>,
    /// CPU-side atlas data (for fallback/debugging)
    atlas_data: Vec<PixelRGBA>,
    /// Atlas dimensions
    atlas_width: u32,
    atlas_height: u32,
    /// Tile metadata buffer
    tile_metadata: Vec<TileMetadata>,
    /// Lookup: (category, position) -> list of tile indices
    tile_lookup: HashMap<(TerrainCategory, TilePosition), Vec<u32>>,
    /// Biome to terrain mapping
    biome_mapping: BiomeTerrainMapping,
    /// Number of loaded tiles
    tile_count: u32,
}

impl TerrainTextureAtlas {
    /// Create a new empty atlas
    #[must_use]
    pub fn new() -> Self {
        Self {
            texture: None,
            texture_view: None,
            sampler: None,
            atlas_data: Vec::new(),
            atlas_width: 0,
            atlas_height: 0,
            tile_metadata: Vec::new(),
            tile_lookup: HashMap::new(),
            biome_mapping: BiomeTerrainMapping::new(),
            tile_count: 0,
        }
    }

    /// Load terrain assets from a directory
    pub fn load_from_directory<P: AsRef<Path>>(&mut self, assets_dir: P) -> Result<usize, String> {
        let assets_dir = assets_dir.as_ref();

        // Create manifest and scan directory
        let mut manifest = TerrainAssetManifest::new();
        manifest.set_base_path(assets_dir);

        let scan_count = manifest
            .scan_directory(assets_dir)
            .map_err(|e| format!("Failed to scan directory: {}", e))?;

        info!("Scanned {} terrain tile files", scan_count);

        if scan_count == 0 {
            return Ok(0);
        }

        // Load texture data
        let config = TextureLoaderConfig::default();
        let (success, failed) = load_manifest_textures(&mut manifest, &config);

        info!("Loaded {} textures ({} failed)", success, failed);

        // Build atlas from loaded tiles
        self.build_atlas(&manifest);

        Ok(success)
    }

    /// Build the atlas from a manifest
    fn build_atlas(&mut self, manifest: &TerrainAssetManifest) {
        let tile_count = manifest.tile_count();
        if tile_count == 0 {
            return;
        }

        // Calculate atlas dimensions
        let tiles_needed = tile_count as u32;
        let rows = (tiles_needed + ATLAS_TILES_PER_ROW - 1) / ATLAS_TILES_PER_ROW;
        self.atlas_width = ATLAS_TILES_PER_ROW * TILE_SIZE;
        self.atlas_height = rows * TILE_SIZE;
        self.tile_count = tiles_needed;

        // Allocate atlas
        self.atlas_data = vec![PixelRGBA::transparent(); (self.atlas_width * self.atlas_height) as usize];
        self.tile_metadata.clear();
        self.tile_lookup.clear();

        // Copy tiles to atlas
        for id in 0..tile_count {
            let tile = match manifest.get_tile(id as u32) {
                Some(t) => t,
                None => continue,
            };

            let tile_x = (id as u32 % ATLAS_TILES_PER_ROW) * TILE_SIZE;
            let tile_y = (id as u32 / ATLAS_TILES_PER_ROW) * TILE_SIZE;

            // Record metadata
            self.tile_metadata.push(TileMetadata {
                atlas_x: tile_x,
                atlas_y: tile_y,
                category: tile.category as u32,
                position: tile.position as u32,
            });

            // Update lookup
            let key = (tile.category, tile.position);
            self.tile_lookup.entry(key).or_default().push(id as u32);

            // Copy pixel data if loaded
            if tile.is_loaded() {
                for y in 0..TILE_SIZE {
                    for x in 0..TILE_SIZE {
                        if let Some(pixel) = tile.get_pixel(x, y) {
                            let atlas_idx = ((tile_y + y) * self.atlas_width + (tile_x + x)) as usize;
                            if atlas_idx < self.atlas_data.len() {
                                self.atlas_data[atlas_idx] = pixel;
                            }
                        }
                    }
                }
            }
        }

        debug!(
            "Built atlas: {}x{} pixels, {} tiles",
            self.atlas_width, self.atlas_height, self.tile_count
        );
    }

    /// Upload atlas to GPU
    pub fn upload_to_gpu(&mut self, device: &Device, queue: &Queue) {
        if self.atlas_data.is_empty() {
            warn!("No atlas data to upload");
            return;
        }

        // Create texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain_atlas"),
            size: wgpu::Extent3d {
                width: self.atlas_width,
                height: self.atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Convert pixel data to raw bytes
        let bytes: Vec<u8> = self.atlas_data.iter()
            .flat_map(|p| [p.r, p.g, p.b, p.a])
            .collect();

        // Upload texture data
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
                bytes_per_row: Some(self.atlas_width * 4),
                rows_per_image: Some(self.atlas_height),
            },
            wgpu::Extent3d {
                width: self.atlas_width,
                height: self.atlas_height,
                depth_or_array_layers: 1,
            },
        );

        // Create view and sampler
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // Pixel-perfect
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.texture = Some(texture);
        self.texture_view = Some(view);
        self.sampler = Some(sampler);

        info!("Uploaded terrain atlas to GPU: {}x{}", self.atlas_width, self.atlas_height);
    }

    /// Get texture view for shader binding
    #[must_use]
    pub fn texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }

    /// Get sampler for shader binding
    #[must_use]
    pub fn sampler(&self) -> Option<&wgpu::Sampler> {
        self.sampler.as_ref()
    }

    /// Get atlas parameters for shader uniform
    #[must_use]
    pub fn params(&self) -> TerrainAtlasParams {
        TerrainAtlasParams {
            tile_size: TILE_SIZE,
            tile_count: self.tile_count,
            atlas_width: self.atlas_width,
            atlas_height: self.atlas_height,
        }
    }

    /// Get tile index for a biome and neighbor configuration
    #[must_use]
    pub fn get_tile_for_biome(
        &self,
        biome_id: u8,
        neighbors: NeighborMask,
        variation_seed: u64,
    ) -> Option<u32> {
        // Get terrain category for biome
        let category = self.biome_mapping.get_primary_terrain(biome_id);
        let position = neighbors.to_tile_position();

        // Look up available tiles
        let key = (category, position);
        if let Some(tiles) = self.tile_lookup.get(&key) {
            if !tiles.is_empty() {
                let idx = (variation_seed % tiles.len() as u64) as usize;
                return Some(tiles[idx]);
            }
        }

        // Fallback to center tiles
        let fallback_key = (category, TilePosition::Center);
        if let Some(tiles) = self.tile_lookup.get(&fallback_key) {
            if !tiles.is_empty() {
                let idx = (variation_seed % tiles.len() as u64) as usize;
                return Some(tiles[idx]);
            }
        }

        // No tile found
        None
    }

    /// Get tile metadata
    #[must_use]
    pub fn get_tile_metadata(&self, tile_id: u32) -> Option<&TileMetadata> {
        self.tile_metadata.get(tile_id as usize)
    }

    /// Sample a pixel from the atlas (CPU-side, for debugging/fallback)
    #[must_use]
    pub fn sample_pixel(&self, tile_id: u32, local_x: u32, local_y: u32) -> Option<PixelRGBA> {
        let meta = self.get_tile_metadata(tile_id)?;

        if local_x >= TILE_SIZE || local_y >= TILE_SIZE {
            return None;
        }

        let atlas_x = meta.atlas_x + local_x;
        let atlas_y = meta.atlas_y + local_y;
        let idx = (atlas_y * self.atlas_width + atlas_x) as usize;

        self.atlas_data.get(idx).copied()
    }

    /// Apply burn effect to a region (CPU-side, for demonstration)
    pub fn apply_burn_effect(&mut self, tile_id: u32, intensity: f32) {
        if let Some(meta) = self.tile_metadata.get(tile_id as usize) {
            let start_x = meta.atlas_x;
            let start_y = meta.atlas_y;

            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    let idx = ((start_y + y) * self.atlas_width + (start_x + x)) as usize;
                    if let Some(pixel) = self.atlas_data.get_mut(idx) {
                        *pixel = pixel.burned(intensity);
                    }
                }
            }
        }
    }

    /// Number of tiles in atlas
    #[must_use]
    pub fn tile_count(&self) -> u32 {
        self.tile_count
    }

    /// Check if atlas has been uploaded to GPU
    #[must_use]
    pub fn is_gpu_ready(&self) -> bool {
        self.texture.is_some()
    }

    /// Get atlas dimensions
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        (self.atlas_width, self.atlas_height)
    }

    /// Get center tiles for a terrain category
    #[must_use]
    pub fn get_center_tiles(&self, category: TerrainCategory) -> Vec<u32> {
        // Try Center first
        if let Some(tiles) = self.tile_lookup.get(&(category, TilePosition::Center)) {
            if !tiles.is_empty() {
                return tiles.clone();
            }
        }
        // Fall back to Modular tiles
        if let Some(tiles) = self.tile_lookup.get(&(category, TilePosition::Modular)) {
            if !tiles.is_empty() {
                return tiles.clone();
            }
        }
        // Fall back to Single tiles
        if let Some(tiles) = self.tile_lookup.get(&(category, TilePosition::Single)) {
            if !tiles.is_empty() {
                return tiles.clone();
            }
        }
        Vec::new()
    }

    /// Get all available terrain categories with their center tile counts
    #[must_use]
    pub fn get_category_tile_counts(&self) -> Vec<(TerrainCategory, usize)> {
        TerrainCategory::all()
            .iter()
            .map(|cat| (*cat, self.get_center_tiles(*cat).len()))
            .collect()
    }

    /// Build a flat array of center tile indices for each category
    /// Returns (tile_indices, category_offsets) where category_offsets[category_id] = (start, count)
    #[must_use]
    pub fn build_category_tile_buffer(&self) -> (Vec<u32>, Vec<(u32, u32)>) {
        let mut tile_indices = Vec::new();
        let mut category_offsets = Vec::new();

        for category in TerrainCategory::all() {
            let start = tile_indices.len() as u32;
            let tiles = self.get_center_tiles(*category);
            let count = tiles.len() as u32;
            tile_indices.extend(tiles);
            category_offsets.push((start, count));
        }

        (tile_indices, category_offsets)
    }

    /// Get tile lookup for debugging
    #[must_use]
    pub fn tile_lookup_summary(&self) -> String {
        let mut summary = String::new();
        for category in TerrainCategory::all() {
            let center_count = self.tile_lookup.get(&(*category, TilePosition::Center)).map_or(0, |v| v.len());
            let modular_count = self.tile_lookup.get(&(*category, TilePosition::Modular)).map_or(0, |v| v.len());
            let top_count = self.tile_lookup.get(&(*category, TilePosition::Top)).map_or(0, |v| v.len());
            let left_count = self.tile_lookup.get(&(*category, TilePosition::Left)).map_or(0, |v| v.len());
            summary.push_str(&format!(
                "{:?}: center={}, modular={}, top={}, left={}\n",
                category, center_count, modular_count, top_count, left_count
            ));
        }
        summary
    }
}

impl Default for TerrainTextureAtlas {
    fn default() -> Self {
        Self::new()
    }
}

/// Pixel effect state for simulation
#[derive(Debug, Clone, Copy, Default)]
pub struct PixelEffectState {
    /// Burn intensity (0.0 = none, 1.0 = fully burned/black)
    pub burn: f32,
    /// Wet intensity (0.0 = dry, 1.0 = soaked)
    pub wet: f32,
    /// Damage level (0.0 = pristine, 1.0 = destroyed)
    pub damage: f32,
    /// Is currently on fire
    pub on_fire: bool,
    /// Smoke emission rate
    pub smoke: f32,
}

impl PixelEffectState {
    /// Apply this effect to a pixel color
    #[must_use]
    pub fn apply_to(&self, pixel: PixelRGBA) -> PixelRGBA {
        let mut result = pixel;

        // Apply burn effect
        if self.burn > 0.0 {
            result = result.burned(self.burn);
        }

        // Apply wet effect
        if self.wet > 0.0 {
            result = result.wet(self.wet);
        }

        // Fire effect: orange/red tint
        if self.on_fire {
            let fire_r = (result.r as f32 * 0.5 + 200.0 * 0.5).min(255.0) as u8;
            let fire_g = (result.g as f32 * 0.5 + 100.0 * 0.5).min(255.0) as u8;
            let fire_b = (result.b as f32 * 0.3).min(255.0) as u8;
            result = PixelRGBA::new(fire_r, fire_g, fire_b, result.a);
        }

        result
    }

    /// Update simulation state
    pub fn simulate(&mut self, dt: f32) {
        // Fire spreads burn
        if self.on_fire {
            self.burn = (self.burn + dt * 0.5).min(1.0);
            self.wet = (self.wet - dt * 2.0).max(0.0);
            self.smoke = 1.0;

            // Fire dies when fully burned
            if self.burn >= 1.0 {
                self.on_fire = false;
            }
        } else {
            self.smoke = (self.smoke - dt * 0.5).max(0.0);
        }

        // Wet things dry slowly
        if self.wet > 0.0 && !self.on_fire {
            self.wet = (self.wet - dt * 0.1).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbor_mask_to_position() {
        // All surrounded -> center
        let all = NeighborMask::new(true, true, true, true, true, true, true, true);
        assert_eq!(all.to_tile_position(), TilePosition::Center);

        // No north -> top edge
        let no_north = NeighborMask::new(false, true, true, true, false, false, true, true);
        assert_eq!(no_north.to_tile_position(), TilePosition::Top);

        // Isolated -> single
        let none = NeighborMask::NONE;
        assert_eq!(none.to_tile_position(), TilePosition::Single);
    }

    #[test]
    fn test_pixel_effect_burn() {
        let pixel = PixelRGBA::rgb(100, 200, 100);
        let effect = PixelEffectState {
            burn: 0.5,
            ..Default::default()
        };

        let result = effect.apply_to(pixel);
        assert!(result.r < pixel.r);
        assert!(result.g < pixel.g);
    }

    #[test]
    fn test_atlas_creation() {
        let atlas = TerrainTextureAtlas::new();
        assert_eq!(atlas.tile_count(), 0);
        assert!(!atlas.is_gpu_ready());
    }
}
