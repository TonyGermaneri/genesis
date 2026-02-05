//! Autotile-based terrain render pipeline.
//!
//! This module provides GPU rendering using the autotile atlas system
//! where each terrain type has 48 pre-computed edge/corner tile variants.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::info;
use wgpu::{util::DeviceExt, Device, Queue};

use crate::autotile_atlas::{AutotileAtlas, AutotileAtlasParams};
use crate::camera::Camera;
use crate::render::{ChunkRenderParams, RenderParams, create_default_colors};

/// Autotile render shader in WGSL.
///
/// This shader samples from the autotile atlas using:
/// - terrain_type (row 0-25) based on biome_id
/// - tile_index (0-47) for variation within terrain type
///
/// For simplicity, we use center-fill tiles (indices 24-35) and pick
/// based on world position hash for natural variation.
pub const AUTOTILE_RENDER_SHADER: &str = r#"
// Cell structure (must match compute shader)
struct Cell {
    material: u32,      // u16 material + u8 flags + u8 temperature/growth
    velocity_data: u32, // i8 vel_x + i8 vel_y + u8 biome_id + u8 elevation
}

// Material color (RGBA float)
struct MaterialColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

// Render parameters
struct RenderParams {
    chunk_size: u32,
    screen_width: u32,
    screen_height: u32,
    camera_x: i32,
    camera_y: i32,
    zoom: f32,
    time_of_day: f32,
    _padding: u32,
}

// Chunk offset parameters
struct ChunkParams {
    world_offset_x: i32,
    world_offset_y: i32,
    _padding: vec2<u32>,
}

// Autotile atlas parameters
struct AutotileParams {
    tile_size: u32,          // 48
    tiles_per_strip_row: u32, // 12
    rows_per_terrain: u32,    // 4
    terrain_count: u32,       // 26
    atlas_width: u32,         // 576
    atlas_height: u32,        // 4992
    _padding: vec2<u32>,
}

// Cell flag bits
const FLAG_BURNING: u32 = 4u;

// Material IDs
const MAT_AIR: u32 = 0u;

// Biome to terrain type mapping
// Maps biome_id (0-8) to autotile terrain row (0-25)
fn get_terrain_type(biome_id: u32) -> u32 {
    switch biome_id {
        case 0u: { return 1u; }  // Forest -> GrassMedium (row 1)
        case 1u: { return 18u; } // Desert -> Dirt (row 18, no sand autotile)
        case 2u: { return 2u; }  // Cave -> GrassDark (row 2)
        case 3u: { return 9u; }  // Ocean -> DeepWater (row 9)
        case 4u: { return 0u; }  // Plains -> GrassLight (row 0)
        case 5u: { return 13u; } // Mountain -> Mound1 (row 13)
        case 6u: { return 3u; }  // Swamp -> GrassForest (row 3)
        case 7u: { return 4u; }  // River -> GrassWater1 (row 4)
        case 8u: { return 8u; }  // Farm -> GrassFenced (row 8)
        default: { return 1u; } // Default to medium grass
    }
}

// Fallback colors for when atlas isn't loaded
const FOREST_GRASS: vec3<f32> = vec3<f32>(0.290, 0.486, 0.137);
const DESERT_SAND: vec3<f32> = vec3<f32>(0.761, 0.651, 0.333);
const OCEAN_WATER: vec3<f32> = vec3<f32>(0.227, 0.486, 0.647);
const PLAINS_GRASS: vec3<f32> = vec3<f32>(0.486, 0.702, 0.259);
const MOUNTAIN_STONE: vec3<f32> = vec3<f32>(0.478, 0.478, 0.478);
const CAVE_STONE: vec3<f32> = vec3<f32>(0.350, 0.340, 0.350);
const SWAMP_MUD: vec3<f32> = vec3<f32>(0.369, 0.420, 0.286);
const RIVER_WATER: vec3<f32> = vec3<f32>(0.310, 0.545, 0.702);
const FARM_SOIL: vec3<f32> = vec3<f32>(0.545, 0.408, 0.286);

// Simple hash function for procedural noise
fn hash(p: vec2<i32>) -> f32 {
    let n = u32(p.x * 374761393 + p.y * 668265263);
    let m = (n ^ (n >> 13u)) * 1274126177u;
    return f32(m & 0xFFFFu) / 65535.0;
}

// Smooth interpolation (smoothstep)
fn smoothstep(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

// Value noise with bilinear interpolation for smooth transitions
fn value_noise(x: f32, y: f32) -> f32 {
    // Integer coordinates of the cell
    let ix = i32(floor(x));
    let iy = i32(floor(y));
    
    // Fractional part within the cell
    let fx = x - floor(x);
    let fy = y - floor(y);
    
    // Hash values at 4 corners
    let v00 = hash(vec2<i32>(ix, iy));
    let v10 = hash(vec2<i32>(ix + 1, iy));
    let v01 = hash(vec2<i32>(ix, iy + 1));
    let v11 = hash(vec2<i32>(ix + 1, iy + 1));
    
    // Smooth interpolation factors
    let sx = smoothstep(fx);
    let sy = smoothstep(fy);
    
    // Bilinear interpolation
    let top = mix(v00, v10, sx);
    let bottom = mix(v01, v11, sx);
    return mix(top, bottom, sy);
}

// Multi-octave smooth noise for terrain variation
fn terrain_noise(world_x: i32, world_y: i32) -> f32 {
    let x = f32(world_x);
    let y = f32(world_y);
    
    var value = 0.0;
    
    // 4 octaves of noise for natural-looking variation at multiple scales
    // Higher base frequency (0.15) for more visible texture
    value += value_noise(x * 0.15, y * 0.15) * 0.4;   // Large-scale variation
    value += value_noise(x * 0.3, y * 0.3) * 0.25;    // Medium variation
    value += value_noise(x * 0.6, y * 0.6) * 0.2;     // Fine detail
    value += value_noise(x * 1.2, y * 1.2) * 0.15;    // Very fine grain
    
    return value;
}

// Get variation-adjusted color for a biome at world position
fn get_biome_color_varied(biome_id: u32, world_x: i32, world_y: i32) -> vec3<f32> {
    let base = get_biome_fallback_color(biome_id);
    
    // Generate noise for this position
    let noise = terrain_noise(world_x, world_y);
    
    // Different variation strategies per biome
    switch biome_id {
        case 0u: { // Forest - green variation
            let shade = 0.85 + noise * 0.3;
            return base * shade;
        }
        case 1u: { // Desert - sand grain variation  
            let shade = 0.9 + noise * 0.2;
            return base * shade;
        }
        case 3u: { // Ocean - wave shimmer
            let wave = sin(f32(world_x + world_y) * 0.1) * 0.05;
            let shade = 0.95 + noise * 0.1 + wave;
            return base * shade;
        }
        case 4u: { // Plains - grass texture
            let shade = 0.88 + noise * 0.24;
            return base * shade;
        }
        case 5u: { // Mountain - rocky variation
            let shade = 0.8 + noise * 0.4;
            return base * shade;
        }
        default: {
            let shade = 0.9 + noise * 0.2;
            return base * shade;
        }
    }
}

fn get_biome_fallback_color(biome_id: u32) -> vec3<f32> {
    switch biome_id {
        case 0u: { return FOREST_GRASS; }
        case 1u: { return DESERT_SAND; }
        case 2u: { return CAVE_STONE; }
        case 3u: { return OCEAN_WATER; }
        case 4u: { return PLAINS_GRASS; }
        case 5u: { return MOUNTAIN_STONE; }
        case 6u: { return SWAMP_MUD; }
        case 7u: { return RIVER_WATER; }
        case 8u: { return FARM_SOIL; }
        default: { return FOREST_GRASS; }
    }
}

// Bindings
@group(0) @binding(0) var<storage, read> cells: array<Cell>;
@group(0) @binding(1) var<storage, read> colors: array<MaterialColor>;
@group(0) @binding(2) var<uniform> params: RenderParams;
@group(0) @binding(3) var<uniform> chunk_params: ChunkParams;
@group(0) @binding(4) var<uniform> autotile_params: AutotileParams;
@group(0) @binding(5) var autotile_atlas: texture_2d<f32>;
@group(0) @binding(6) var autotile_sampler: sampler;

// 47-tile blob autotile lookup table
// Maps 8-bit neighbor bitmask to tile index (0-46)
// Bitmask format: NW(1) N(2) NE(4) W(8) E(16) SW(32) S(64) SE(128)
// Corner bits are only relevant when both adjacent cardinals are present
const BITMASK_TO_TILE: array<u32, 256> = array<u32, 256>(
    // 0-15
    0u, 0u, 1u, 1u, 0u, 0u, 1u, 1u, 2u, 2u, 3u, 4u, 2u, 2u, 3u, 4u,
    // 16-31
    5u, 5u, 6u, 6u, 5u, 5u, 7u, 7u, 8u, 8u, 9u, 10u, 8u, 8u, 11u, 12u,
    // 32-47
    0u, 0u, 1u, 1u, 0u, 0u, 1u, 1u, 2u, 2u, 3u, 4u, 2u, 2u, 3u, 4u,
    // 48-63
    5u, 5u, 6u, 6u, 5u, 5u, 7u, 7u, 8u, 8u, 9u, 10u, 8u, 8u, 11u, 12u,
    // 64-79
    13u, 13u, 14u, 14u, 13u, 13u, 14u, 14u, 15u, 15u, 16u, 17u, 15u, 15u, 16u, 17u,
    // 80-95
    18u, 18u, 19u, 19u, 18u, 18u, 20u, 20u, 21u, 21u, 22u, 23u, 21u, 21u, 24u, 25u,
    // 96-111
    13u, 13u, 14u, 14u, 13u, 13u, 14u, 14u, 26u, 26u, 27u, 28u, 26u, 26u, 27u, 28u,
    // 112-127
    18u, 18u, 19u, 19u, 18u, 18u, 20u, 20u, 29u, 29u, 30u, 31u, 29u, 29u, 32u, 33u,
    // 128-143
    0u, 0u, 1u, 1u, 0u, 0u, 1u, 1u, 2u, 2u, 3u, 4u, 2u, 2u, 3u, 4u,
    // 144-159
    5u, 5u, 6u, 6u, 5u, 5u, 7u, 7u, 8u, 8u, 9u, 10u, 8u, 8u, 11u, 12u,
    // 160-175
    0u, 0u, 1u, 1u, 0u, 0u, 1u, 1u, 2u, 2u, 3u, 4u, 2u, 2u, 3u, 4u,
    // 176-191
    5u, 5u, 6u, 6u, 5u, 5u, 7u, 7u, 8u, 8u, 9u, 10u, 8u, 8u, 11u, 12u,
    // 192-207
    13u, 13u, 14u, 14u, 13u, 13u, 14u, 14u, 15u, 15u, 16u, 17u, 15u, 15u, 16u, 17u,
    // 208-223
    34u, 34u, 35u, 35u, 34u, 34u, 36u, 36u, 37u, 37u, 38u, 39u, 37u, 37u, 40u, 41u,
    // 224-239
    13u, 13u, 14u, 14u, 13u, 13u, 14u, 14u, 26u, 26u, 27u, 28u, 26u, 26u, 27u, 28u,
    // 240-255
    34u, 34u, 35u, 35u, 34u, 34u, 36u, 36u, 42u, 42u, 43u, 44u, 42u, 42u, 45u, 46u
);

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Fullscreen triangle vertices
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    // Generate fullscreen triangle (covers screen with 3 vertices)
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    output.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    output.uv = vec2<f32>(x, y);

    return output;
}

// Sample terrain from autotile atlas with edge detection
// Computes neighbor bitmask by checking surrounding cells for same biome
fn sample_autotile(world_x: i32, world_y: i32, biome_id: u32, local_x: i32, local_y: i32) -> vec4<f32> {
    let tile_size = autotile_params.tile_size;
    if tile_size == 0u || autotile_params.terrain_count == 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0); // No atlas loaded
    }

    // Get terrain type from biome
    let terrain_type = get_terrain_type(biome_id);

    // Compute neighbor bitmask by checking 8 surrounding cells
    // Bitmask format: NW(1) N(2) NE(4) W(8) E(16) SW(32) S(64) SE(128)
    var mask: u32 = 0u;
    let chunk_size = i32(params.chunk_size);

    // Check each neighbor and set bit if same biome
    let nw_biome = get_neighbor_biome(local_x - 1, local_y - 1, chunk_size);
    let n_biome = get_neighbor_biome(local_x, local_y - 1, chunk_size);
    let ne_biome = get_neighbor_biome(local_x + 1, local_y - 1, chunk_size);
    let w_biome = get_neighbor_biome(local_x - 1, local_y, chunk_size);
    let e_biome = get_neighbor_biome(local_x + 1, local_y, chunk_size);
    let sw_biome = get_neighbor_biome(local_x - 1, local_y + 1, chunk_size);
    let s_biome = get_neighbor_biome(local_x, local_y + 1, chunk_size);
    let se_biome = get_neighbor_biome(local_x + 1, local_y + 1, chunk_size);

    // Set cardinal direction bits
    let n_match = (n_biome == biome_id);
    let e_match = (e_biome == biome_id);
    let s_match = (s_biome == biome_id);
    let w_match = (w_biome == biome_id);

    if n_match { mask |= 2u; }
    if e_match { mask |= 16u; }
    if s_match { mask |= 64u; }
    if w_match { mask |= 8u; }

    // Set corner bits only if BOTH adjacent cardinals match
    if nw_biome == biome_id && n_match && w_match { mask |= 1u; }
    if ne_biome == biome_id && n_match && e_match { mask |= 4u; }
    if sw_biome == biome_id && s_match && w_match { mask |= 32u; }
    if se_biome == biome_id && s_match && e_match { mask |= 128u; }

    // Look up tile index from bitmask
    let tile_index = BITMASK_TO_TILE[mask];

    // Calculate position within tile (0-47 for each axis)
    // We tile the 48x48 texture across the world
    let local_px_x = u32(abs(world_x)) % tile_size;
    let local_px_y = u32(abs(world_y)) % tile_size;

    // Calculate tile position in the strip (12 cols × 4 rows = 48 tiles)
    let strip_col = tile_index % autotile_params.tiles_per_strip_row; // 0-11
    let strip_row = tile_index / autotile_params.tiles_per_strip_row; // 0-3

    // Calculate atlas pixel coordinates
    // Each terrain type occupies 4 rows (192 pixels) in the atlas
    let terrain_base_y = terrain_type * autotile_params.rows_per_terrain * tile_size;
    let tile_offset_y = strip_row * tile_size;

    let atlas_px_x = strip_col * tile_size + local_px_x;
    let atlas_px_y = terrain_base_y + tile_offset_y + local_px_y;

    // Sample from atlas (normalize to 0-1 UV)
    let uv = vec2<f32>(
        f32(atlas_px_x) / f32(autotile_params.atlas_width),
        f32(atlas_px_y) / f32(autotile_params.atlas_height)
    );

    return textureSample(autotile_atlas, autotile_sampler, uv);
}

// Get biome of a neighboring cell (returns 255 if out of bounds)
fn get_neighbor_biome(local_x: i32, local_y: i32, chunk_size: i32) -> u32 {
    // If out of bounds, treat as same biome to avoid edges at chunk boundaries
    if local_x < 0 || local_y < 0 || local_x >= chunk_size || local_y >= chunk_size {
        return 255u; // Will never match, creating edge at boundary
    }

    let idx = u32(local_y) * u32(chunk_size) + u32(local_x);
    let cell = cells[idx];
    return (cell.velocity_data >> 16u) & 0xFFu;
}

// Apply day/night lighting
fn apply_lighting(color: vec3<f32>, time_of_day: f32) -> vec3<f32> {
    let noon_distance = abs(time_of_day - 0.5);
    let ambient = 0.3 + 0.7 * (1.0 - noon_distance * 2.0);

    var tint = vec3<f32>(1.0, 1.0, 1.0);
    if time_of_day < 0.25 || time_of_day > 0.75 {
        let night_factor = select(
            (0.25 - time_of_day) / 0.25,
            (time_of_day - 0.75) / 0.25,
            time_of_day > 0.5
        );
        tint = mix(vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(0.6, 0.7, 1.0), night_factor);
    }

    return color * ambient * tint;
}

// Apply burn effect
fn apply_burn_effect(color: vec3<f32>) -> vec3<f32> {
    let burn_color = vec3<f32>(0.8, 0.3, 0.1);
    return mix(color * 0.3, burn_color, 0.5);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate world position from screen position
    // Note: camera position is CENTER of view, so we need to offset by half screen size
    let screen_x = i32(input.position.x);
    let screen_y = i32(input.position.y);
    let half_width = f32(params.screen_width) / 2.0;
    let half_height = f32(params.screen_height) / 2.0;

    // Apply camera offset (centered) and zoom
    let world_x = params.camera_x + i32((f32(screen_x) - half_width) / params.zoom);
    let world_y = params.camera_y + i32((f32(screen_y) - half_height) / params.zoom);

    // Apply chunk offset
    let local_x = world_x - chunk_params.world_offset_x;
    let local_y = world_y - chunk_params.world_offset_y;

    // Bounds check - return transparent for out-of-bounds
    if local_x < 0 || local_y < 0 || local_x >= i32(params.chunk_size) || local_y >= i32(params.chunk_size) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Get cell at this position
    let idx = u32(local_y) * params.chunk_size + u32(local_x);
    let cell = cells[idx];

    // Extract material ID (lower 16 bits of material field)
    let material_id = cell.material & 0xFFFFu;

    // Extract biome ID from velocity_data (byte 2 of the upper 16 bits)
    let biome_id = (cell.velocity_data >> 16u) & 0xFFu;

    // In top-down mode, there's no "air" - every cell is ground terrain.
    // If material is 0 (uninitialized/default), use biome fallback color.
    // This handles edge cases where cells haven't been properly generated.

    // DEBUG MODE: Skip atlas entirely, use pure biome colors + noise
    // Set to true to isolate whether grid patterns come from noise or atlas sampling
    let DEBUG_PURE_NOISE = false;
    
    if DEBUG_PURE_NOISE {
        // Pure noise-based coloring - no atlas, no texture sampling
        let varied = get_biome_color_varied(biome_id, world_x, world_y);
        let lit = apply_lighting(varied, params.time_of_day);
        return vec4<f32>(lit.r, lit.g, lit.b, 1.0);
    }

    // Sample texture from autotile atlas
    let tex_color = sample_autotile(world_x, world_y, biome_id, local_x, local_y);
    
    // If texture sampling failed (atlas not loaded or material 0), use varied fallback color
    if tex_color.a < 0.01 || material_id == 0u {
        let varied = get_biome_color_varied(biome_id, world_x, world_y);
        return vec4<f32>(varied.r, varied.g, varied.b, 1.0);
    }

    // Add variation to sampled texture as well
    let noise_factor = terrain_noise(world_x, world_y);
    let shade = 0.9 + noise_factor * 0.2;
    let varied_tex = tex_color.rgb * shade;

    // Apply day/night lighting
    let lit_color = apply_lighting(varied_tex, params.time_of_day);

    return vec4<f32>(lit_color.r, lit_color.g, lit_color.b, tex_color.a);
}
"#;

/// GPU buffer for a single chunk in autotile rendering
struct AutotileChunkGpuBuffer {
    /// Cell data buffer
    cells_buffer: wgpu::Buffer,
    /// Chunk world offset params buffer
    chunk_params_buffer: wgpu::Buffer,
    /// Bind group for this chunk
    bind_group: wgpu::BindGroup,
}

/// Autotile-based chunk renderer
pub struct AutotileChunkRenderer {
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,
    /// Material color buffer
    color_buffer: wgpu::Buffer,
    /// Render params buffer
    params_buffer: wgpu::Buffer,
    /// Autotile params buffer
    autotile_params_buffer: wgpu::Buffer,
    /// Per-chunk GPU buffers
    chunk_buffers: HashMap<(i32, i32), AutotileChunkGpuBuffer>,
    /// Autotile atlas reference
    atlas: Option<Arc<RwLock<AutotileAtlas>>>,
    /// Atlas bind group (texture + sampler)
    atlas_bind_group_entries: Option<(wgpu::TextureView, wgpu::Sampler)>,
    /// Chunk size
    chunk_size: u32,
    /// Whether atlas is bound
    atlas_bound: bool,
}

impl AutotileChunkRenderer {
    /// Create a new autotile chunk renderer
    pub fn new(device: &Device, chunk_size: u32) -> Self {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("autotile_render_shader"),
            source: wgpu::ShaderSource::Wgsl(AUTOTILE_RENDER_SHADER.into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("autotile_bind_group_layout"),
            entries: &[
                // 0: Cell data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 1: Material colors
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 2: Render params
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 3: Chunk params
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 4: Autotile params
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 5: Autotile texture
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 6: Autotile sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("autotile_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("autotile_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create material color buffer
        let colors = create_default_colors();
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("autotile_color_buffer"),
            contents: bytemuck::cast_slice(&colors),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create render params buffer
        let params = RenderParams::default();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("autotile_params_buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create autotile params buffer
        let autotile_params = AutotileAtlasParams::default();
        let autotile_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("autotile_params_uniform"),
            contents: bytemuck::bytes_of(&autotile_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            bind_group_layout,
            color_buffer,
            params_buffer,
            autotile_params_buffer,
            chunk_buffers: HashMap::new(),
            atlas: None,
            atlas_bind_group_entries: None,
            chunk_size,
            atlas_bound: false,
        }
    }

    /// Bind the autotile atlas
    pub fn bind_atlas(&mut self, atlas: Arc<RwLock<AutotileAtlas>>, queue: &Queue) {
        let atlas_guard = atlas.read();

        if let (Some(view), Some(sampler)) = (atlas_guard.texture_view(), atlas_guard.sampler()) {
            // Update params buffer
            let params = atlas_guard.params();
            queue.write_buffer(&self.autotile_params_buffer, 0, bytemuck::bytes_of(&params));

            // Store references (we'll use them when creating bind groups)
            self.atlas = Some(atlas.clone());
            self.atlas_bound = true;

            info!(
                "Autotile renderer bound to atlas: {} terrain types, {}x{}",
                params.terrain_count, params.atlas_width, params.atlas_height
            );
        }
    }

    /// Check if atlas is bound
    pub fn is_atlas_bound(&self) -> bool {
        self.atlas_bound
    }

    /// Get the number of chunks in buffer
    pub fn chunk_buffer_count(&self) -> usize {
        self.chunk_buffers.len()
    }

    /// Update render parameters
    pub fn update_params(&self, queue: &Queue, camera: &Camera, time_of_day: f32) {
        let mut params = RenderParams::new(
            self.chunk_size,
            camera.viewport_size.0,
            camera.viewport_size.1,
        );
        params.set_camera(camera.position.0 as i32, camera.position.1 as i32);
        params.set_zoom(camera.zoom);
        params.set_time_of_day(time_of_day);
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    /// Update or create chunk buffer
    pub fn update_chunk(
        &mut self,
        device: &Device,
        queue: &Queue,
        chunk_x: i32,
        chunk_y: i32,
        cell_data: &[u8],
    ) {
        let key = (chunk_x, chunk_y);
        let world_offset_x = chunk_x * self.chunk_size as i32;
        let world_offset_y = chunk_y * self.chunk_size as i32;

        if let Some(buf) = self.chunk_buffers.get(&key) {
            // Update existing buffer
            queue.write_buffer(&buf.cells_buffer, 0, cell_data);
            let chunk_params = ChunkRenderParams::new(world_offset_x, world_offset_y);
            queue.write_buffer(&buf.chunk_params_buffer, 0, bytemuck::bytes_of(&chunk_params));
        } else {
            // Create new buffer
            let cells_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("autotile_chunk_{}_{}", chunk_x, chunk_y)),
                contents: cell_data,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let chunk_params = ChunkRenderParams::new(world_offset_x, world_offset_y);
            let chunk_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("autotile_chunk_params_{}_{}", chunk_x, chunk_y)),
                contents: bytemuck::bytes_of(&chunk_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            // Create bind group
            let bind_group = self.create_chunk_bind_group(
                device,
                &cells_buffer,
                &chunk_params_buffer,
            );

            self.chunk_buffers.insert(
                key,
                AutotileChunkGpuBuffer {
                    cells_buffer,
                    chunk_params_buffer,
                    bind_group,
                },
            );
        }
    }

    /// Create bind group for a chunk
    fn create_chunk_bind_group(
        &self,
        device: &Device,
        cells_buffer: &wgpu::Buffer,
        chunk_params_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        // Get atlas texture view and sampler
        let atlas_guard = self.atlas.as_ref().map(|a| a.read());
        let (texture_view, sampler) = if let Some(ref guard) = atlas_guard {
            if let (Some(view), Some(samp)) = (guard.texture_view(), guard.sampler()) {
                (view, samp)
            } else {
                panic!("Atlas not uploaded to GPU");
            }
        } else {
            panic!("No atlas bound");
        };

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("autotile_chunk_bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cells_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.color_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: chunk_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.autotile_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    /// Render all visible chunks
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        visible_chunks: &[(i32, i32)],
    ) {
        if !self.atlas_bound {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);

        for (chunk_x, chunk_y) in visible_chunks {
            if let Some(buf) = self.chunk_buffers.get(&(*chunk_x, *chunk_y)) {
                render_pass.set_bind_group(0, &buf.bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }
    }

    /// Clear all chunk buffers
    pub fn clear(&mut self) {
        self.chunk_buffers.clear();
    }

    /// Remove chunks outside visible range
    pub fn prune_chunks(&mut self, visible: &[(i32, i32)]) {
        let visible_set: std::collections::HashSet<_> = visible.iter().copied().collect();
        self.chunk_buffers.retain(|k, _| visible_set.contains(k));
    }
}
