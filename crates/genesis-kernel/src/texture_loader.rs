//! Texture loading utilities for terrain assets.
//!
//! Loads 48x48 PNG textures and extracts RGBA pixel data.

use std::path::Path;
use tracing::{debug, warn};

use crate::terrain_assets::{PixelRGBA, TerrainAssetManifest, TerrainTile, TILE_SIZE};

/// Error type for texture loading
#[derive(Debug)]
pub enum TextureLoadError {
    /// File not found
    NotFound(String),
    /// Invalid image format
    InvalidFormat(String),
    /// Image decode error
    DecodeError(String),
    /// Wrong dimensions
    WrongDimensions { expected: u32, actual_width: u32, actual_height: u32 },
}

impl std::fmt::Display for TextureLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "File not found: {}", path),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Self::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            Self::WrongDimensions { expected, actual_width, actual_height } => {
                write!(f, "Wrong dimensions: expected {}x{}, got {}x{}",
                    expected, expected, actual_width, actual_height)
            }
        }
    }
}

impl std::error::Error for TextureLoadError {}

/// Texture loader configuration
#[derive(Debug, Clone)]
pub struct TextureLoaderConfig {
    /// Expected tile size (48 for Modern Exteriors 48x48)
    pub tile_size: u32,
    /// Allow non-square textures (will crop to tile_size)
    pub allow_crop: bool,
    /// Allow smaller textures (will pad with transparent)
    pub allow_padding: bool,
}

impl Default for TextureLoaderConfig {
    fn default() -> Self {
        Self {
            tile_size: TILE_SIZE,
            allow_crop: false,
            allow_padding: true,
        }
    }
}

/// Load a PNG texture and extract RGBA pixels
pub fn load_texture_rgba<P: AsRef<Path>>(
    path: P,
    config: &TextureLoaderConfig,
) -> Result<Vec<PixelRGBA>, TextureLoadError> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(TextureLoadError::NotFound(path.display().to_string()));
    }

    // Read file
    let file_data = std::fs::read(path)
        .map_err(|e| TextureLoadError::DecodeError(e.to_string()))?;

    // Decode PNG
    let img = image::load_from_memory(&file_data)
        .map_err(|e| TextureLoadError::DecodeError(e.to_string()))?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Check dimensions
    if width != config.tile_size || height != config.tile_size {
        if !config.allow_crop && !config.allow_padding {
            return Err(TextureLoadError::WrongDimensions {
                expected: config.tile_size,
                actual_width: width,
                actual_height: height,
            });
        }
    }

    // Extract pixels
    let mut pixels = Vec::with_capacity((config.tile_size * config.tile_size) as usize);

    for y in 0..config.tile_size {
        for x in 0..config.tile_size {
            if x < width && y < height {
                let pixel = rgba.get_pixel(x, y);
                pixels.push(PixelRGBA::new(pixel[0], pixel[1], pixel[2], pixel[3]));
            } else if config.allow_padding {
                pixels.push(PixelRGBA::transparent());
            }
        }
    }

    Ok(pixels)
}

/// Load pixel data into a terrain tile
pub fn load_tile_pixels(
    tile: &mut TerrainTile,
    base_path: &Path,
    config: &TextureLoaderConfig,
) -> Result<(), TextureLoadError> {
    let path = base_path.join(&tile.filename);
    let pixels = load_texture_rgba(&path, config)?;
    tile.pixels = pixels;
    debug!("Loaded {} pixels for tile: {}", tile.pixels.len(), tile.filename);
    Ok(())
}

/// Batch load all tiles in a manifest
pub fn load_manifest_textures(
    manifest: &mut TerrainAssetManifest,
    config: &TextureLoaderConfig,
) -> (usize, usize) {
    let base_path = manifest.base_path().to_path_buf();
    let mut success_count = 0;
    let mut fail_count = 0;

    for id in 0..manifest.tile_count() {
        if let Some(tile) = manifest.get_tile_mut(id as u32) {
            match load_tile_pixels(tile, &base_path, config) {
                Ok(()) => success_count += 1,
                Err(e) => {
                    warn!("Failed to load tile {}: {}", tile.filename, e);
                    fail_count += 1;
                }
            }
        }
    }

    (success_count, fail_count)
}

/// Create a texture atlas from loaded tiles
///
/// Returns (atlas_data, atlas_width, atlas_height, tile_positions)
/// where tile_positions maps tile_id -> (x, y) in atlas
pub fn create_texture_atlas(
    manifest: &TerrainAssetManifest,
    tiles_per_row: u32,
) -> (Vec<PixelRGBA>, u32, u32, std::collections::HashMap<u32, (u32, u32)>) {
    use std::collections::HashMap;

    let tile_count = manifest.tile_count() as u32;
    if tile_count == 0 {
        return (Vec::new(), 0, 0, HashMap::new());
    }

    let rows = (tile_count + tiles_per_row - 1) / tiles_per_row;
    let atlas_width = tiles_per_row * TILE_SIZE;
    let atlas_height = rows * TILE_SIZE;

    let mut atlas = vec![PixelRGBA::transparent(); (atlas_width * atlas_height) as usize];
    let mut positions = HashMap::new();

    for id in 0..tile_count {
        if let Some(tile) = manifest.get_tile(id) {
            if !tile.is_loaded() {
                continue;
            }

            let tile_x = (id % tiles_per_row) * TILE_SIZE;
            let tile_y = (id / tiles_per_row) * TILE_SIZE;
            positions.insert(id, (tile_x, tile_y));

            // Copy tile pixels to atlas
            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    if let Some(pixel) = tile.get_pixel(x, y) {
                        let atlas_idx = ((tile_y + y) * atlas_width + (tile_x + x)) as usize;
                        if atlas_idx < atlas.len() {
                            atlas[atlas_idx] = pixel;
                        }
                    }
                }
            }
        }
    }

    (atlas, atlas_width, atlas_height, positions)
}

/// Extract the dominant color from a tile (for fallback/preview)
pub fn get_tile_dominant_color(tile: &TerrainTile) -> PixelRGBA {
    if tile.pixels.is_empty() {
        return PixelRGBA::rgb(128, 128, 128); // Gray default
    }

    let mut r_sum: u64 = 0;
    let mut g_sum: u64 = 0;
    let mut b_sum: u64 = 0;
    let mut count: u64 = 0;

    for pixel in &tile.pixels {
        if pixel.a > 128 {
            // Only count mostly-opaque pixels
            r_sum += pixel.r as u64;
            g_sum += pixel.g as u64;
            b_sum += pixel.b as u64;
            count += 1;
        }
    }

    if count == 0 {
        return PixelRGBA::rgb(128, 128, 128);
    }

    PixelRGBA::rgb(
        (r_sum / count) as u8,
        (g_sum / count) as u8,
        (b_sum / count) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_loader_config_default() {
        let config = TextureLoaderConfig::default();
        assert_eq!(config.tile_size, 48);
        assert!(!config.allow_crop);
        assert!(config.allow_padding);
    }

    #[test]
    fn test_dominant_color_empty() {
        let tile = TerrainTile::new(
            0,
            "test.png".to_string(),
            crate::terrain_assets::TerrainCategory::Grass,
            crate::terrain_assets::TilePosition::Center,
            1,
        );
        let color = get_tile_dominant_color(&tile);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 128);
    }
}
