//! Texture-based terrain rendering integration.
//!
//! This module provides the bridge between the terrain texture atlas
//! and the cell-based rendering system. It enables:
//! - Sampling pixel colors from 48x48 texture tiles
//! - Mapping biome/material to specific texture tiles
//! - Sub-tile pixel positioning for exploded textures
//! - Simulation effects applied to texture pixels

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use parking_lot::RwLock;
use wgpu::{Device, Queue};

use crate::terrain_atlas::TerrainTextureAtlas;
use crate::terrain_assets::{PixelRGBA, TILE_SIZE};
use crate::biome::BiomeId;

/// Tile reference stored per-cell for texture lookup
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct CellTextureRef {
    /// Tile ID in the atlas (0 = no texture, use procedural)
    pub tile_id: u16,
    /// Local X within tile (0-47)
    pub local_x: u8,
    /// Local Y within tile (0-47)
    pub local_y: u8,
}

impl CellTextureRef {
    /// Create a new texture reference
    #[must_use]
    pub const fn new(tile_id: u16, local_x: u8, local_y: u8) -> Self {
        Self { tile_id, local_x, local_y }
    }

    /// No texture - use procedural coloring
    #[must_use]
    pub const fn none() -> Self {
        Self { tile_id: 0, local_x: 0, local_y: 0 }
    }

    /// Check if this reference has a texture
    #[must_use]
    pub const fn has_texture(&self) -> bool {
        self.tile_id > 0
    }
}

/// Terrain texture integration for a chunk
pub struct ChunkTextureLayer {
    /// Texture references for each cell in the chunk
    texture_refs: Vec<CellTextureRef>,
    /// Chunk size (width and height)
    chunk_size: u32,
    /// Whether this layer has been populated
    populated: bool,
}

impl ChunkTextureLayer {
    /// Create a new empty texture layer for a chunk
    #[must_use]
    pub fn new(chunk_size: u32) -> Self {
        let cell_count = (chunk_size * chunk_size) as usize;
        Self {
            texture_refs: vec![CellTextureRef::none(); cell_count],
            chunk_size,
            populated: false,
        }
    }

    /// Get texture reference for a cell
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> CellTextureRef {
        if x >= self.chunk_size || y >= self.chunk_size {
            return CellTextureRef::none();
        }
        let idx = (y * self.chunk_size + x) as usize;
        self.texture_refs.get(idx).copied().unwrap_or(CellTextureRef::none())
    }

    /// Set texture reference for a cell
    pub fn set(&mut self, x: u32, y: u32, tex_ref: CellTextureRef) {
        if x >= self.chunk_size || y >= self.chunk_size {
            return;
        }
        let idx = (y * self.chunk_size + x) as usize;
        if idx < self.texture_refs.len() {
            self.texture_refs[idx] = tex_ref;
        }
    }

    /// Populate the layer using texture atlas for a biome
    pub fn populate_from_atlas(
        &mut self,
        atlas: &TerrainTextureAtlas,
        biome_id: BiomeId,
        chunk_x: i32,
        chunk_y: i32,
    ) {
        // Calculate base seed for this chunk for variation
        let chunk_seed = ((chunk_x as i64).wrapping_mul(73856093)
            ^ (chunk_y as i64).wrapping_mul(19349663)) as u64;

        // We'll tile 48x48 textures across the 256x256 chunk
        // 256 / 48 = 5.33, so we need 6 tiles per row/column
        let tiles_per_axis = (self.chunk_size + TILE_SIZE - 1) / TILE_SIZE;

        for tile_y in 0..tiles_per_axis {
            for tile_x in 0..tiles_per_axis {
                // Get a tile for this position
                let tile_seed = chunk_seed
                    .wrapping_add(tile_x as u64 * 12345)
                    .wrapping_add(tile_y as u64 * 67890);

                // Determine neighbors for autotiling (simplified - all same biome)
                let neighbors = crate::terrain_atlas::NeighborMask::new(
                    true, true, true, true, true, true, true, true
                );

                if let Some(tile_id) = atlas.get_tile_for_biome(biome_id, neighbors, tile_seed) {
                    // Map this tile's pixels to cells
                    let base_cell_x = tile_x * TILE_SIZE;
                    let base_cell_y = tile_y * TILE_SIZE;

                    for local_y in 0..TILE_SIZE {
                        for local_x in 0..TILE_SIZE {
                            let cell_x = base_cell_x + local_x;
                            let cell_y = base_cell_y + local_y;

                            if cell_x < self.chunk_size && cell_y < self.chunk_size {
                                self.set(cell_x, cell_y, CellTextureRef::new(
                                    (tile_id + 1) as u16, // +1 because 0 = none
                                    local_x as u8,
                                    local_y as u8,
                                ));
                            }
                        }
                    }
                }
            }
        }

        self.populated = true;
    }

    /// Check if layer has been populated
    #[must_use]
    pub fn is_populated(&self) -> bool {
        self.populated
    }

    /// Get raw texture refs slice
    #[must_use]
    pub fn as_slice(&self) -> &[CellTextureRef] {
        &self.texture_refs
    }
}

/// CPU-side texture color lookup for cells
pub struct TextureColorLookup {
    /// Reference to the terrain atlas
    atlas: Arc<RwLock<TerrainTextureAtlas>>,
}

impl TextureColorLookup {
    /// Create a new color lookup with the given atlas
    #[must_use]
    pub fn new(atlas: Arc<RwLock<TerrainTextureAtlas>>) -> Self {
        Self { atlas }
    }

    /// Get the color for a cell's texture reference
    #[must_use]
    pub fn get_color(&self, tex_ref: CellTextureRef) -> Option<PixelRGBA> {
        if !tex_ref.has_texture() {
            return None;
        }

        let atlas = self.atlas.read();
        let tile_id = (tex_ref.tile_id - 1) as u32; // Convert back from 1-indexed
        atlas.sample_pixel(tile_id, tex_ref.local_x as u32, tex_ref.local_y as u32)
    }

    /// Get color with simulation effects applied
    #[must_use]
    pub fn get_color_with_effects(
        &self,
        tex_ref: CellTextureRef,
        burn_intensity: f32,
        wet_intensity: f32,
    ) -> Option<PixelRGBA> {
        let base_color = self.get_color(tex_ref)?;

        let mut result = base_color;
        if burn_intensity > 0.0 {
            result = result.burned(burn_intensity);
        }
        if wet_intensity > 0.0 {
            result = result.wet(wet_intensity);
        }

        Some(result)
    }
}

/// GPU buffer for texture references per chunk
pub struct ChunkTextureBuffer {
    /// GPU buffer containing CellTextureRef data
    buffer: Option<wgpu::Buffer>,
    /// Number of cells
    cell_count: u32,
}

impl ChunkTextureBuffer {
    /// Create a new texture buffer for a chunk
    #[must_use]
    pub fn new(chunk_size: u32) -> Self {
        Self {
            buffer: None,
            cell_count: chunk_size * chunk_size,
        }
    }

    /// Upload texture layer to GPU
    pub fn upload(&mut self, device: &Device, queue: &Queue, layer: &ChunkTextureLayer) {
        let data = bytemuck::cast_slice(layer.as_slice());

        if self.buffer.is_none() {
            // Create buffer
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("chunk_texture_refs"),
                size: data.len() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer = Some(buffer);
        }

        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, data);
        }
    }

    /// Get buffer reference for binding
    #[must_use]
    pub fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_ref()
    }
}

/// Terrain texture renderer configuration
#[derive(Debug, Clone)]
pub struct TextureRenderConfig {
    /// Enable texture-based rendering (vs procedural)
    pub enabled: bool,
    /// Apply simulation effects to textures
    pub apply_effects: bool,
    /// Blend between texture and procedural colors
    pub blend_factor: f32,
}

impl Default for TextureRenderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            apply_effects: true,
            blend_factor: 1.0, // 100% texture
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_texture_ref_none() {
        let tex_ref = CellTextureRef::none();
        assert!(!tex_ref.has_texture());
        assert_eq!(tex_ref.tile_id, 0);
    }

    #[test]
    fn test_cell_texture_ref_valid() {
        let tex_ref = CellTextureRef::new(5, 10, 20);
        assert!(tex_ref.has_texture());
        assert_eq!(tex_ref.tile_id, 5);
        assert_eq!(tex_ref.local_x, 10);
        assert_eq!(tex_ref.local_y, 20);
    }

    #[test]
    fn test_chunk_texture_layer_creation() {
        let layer = ChunkTextureLayer::new(256);
        assert!(!layer.is_populated());
        assert!(!layer.get(0, 0).has_texture());
    }

    #[test]
    fn test_chunk_texture_layer_set_get() {
        let mut layer = ChunkTextureLayer::new(256);
        layer.set(10, 20, CellTextureRef::new(42, 5, 7));

        let tex_ref = layer.get(10, 20);
        assert!(tex_ref.has_texture());
        assert_eq!(tex_ref.tile_id, 42);
        assert_eq!(tex_ref.local_x, 5);
        assert_eq!(tex_ref.local_y, 7);
    }
}
