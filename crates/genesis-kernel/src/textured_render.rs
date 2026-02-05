//! Textured terrain render pipeline.
//!
//! This module extends the cell rendering to support texture atlas sampling
//! for pixel-perfect terrain textures from 48x48 tile images.

use std::collections::HashMap;
use std::sync::Arc;

use bytemuck::Pod;
use parking_lot::RwLock;
use tracing::{debug, info};
use wgpu::{util::DeviceExt, Device, Queue};

use crate::camera::Camera;
use crate::chunk_manager::{VisibleChunk, VisibleChunkManager};
use crate::render::{ChunkRenderParams, RenderParams, create_default_colors};
use crate::terrain_atlas::{TerrainAtlasParams, TerrainTextureAtlas};

/// Textured multi-chunk render shader in WGSL with terrain atlas support.
pub const TEXTURED_RENDER_SHADER: &str = r#"
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

// Atlas parameters
struct AtlasParams {
    tile_size: u32,
    tile_count: u32,
    atlas_width: u32,
    atlas_height: u32,
}

// Cell flag bits
const FLAG_BURNING: u32 = 4u;

// Material IDs
const MAT_AIR: u32 = 0u;
const MAT_WATER: u32 = 1u;
const MAT_SAND: u32 = 2u;
const MAT_GRASS: u32 = 3u;
const MAT_DIRT: u32 = 4u;
const MAT_STONE: u32 = 5u;
const MAT_SNOW: u32 = 6u;
const MAT_SANDSTONE: u32 = 7u;

// Biome IDs
const BIOME_FOREST: u32 = 0u;
const BIOME_DESERT: u32 = 1u;
const BIOME_CAVE: u32 = 2u;
const BIOME_OCEAN: u32 = 3u;
const BIOME_PLAINS: u32 = 4u;
const BIOME_MOUNTAIN: u32 = 5u;

// Biome to atlas row mapping (48x48 tiles)
fn get_biome_atlas_row(biome_id: u32) -> u32 {
    switch biome_id {
        case BIOME_FOREST: { return 0u; }   // Grass tiles
        case BIOME_DESERT: { return 1u; }   // Sand tiles
        case BIOME_OCEAN: { return 2u; }    // Water tiles
        case BIOME_PLAINS: { return 3u; }   // Plains grass
        case BIOME_MOUNTAIN: { return 4u; } // Stone tiles
        case BIOME_CAVE: { return 5u; }     // Dark stone
        default: { return 0u; }
    }
}

// Fallback biome colors (K-32)
const FOREST_GRASS: vec3<f32> = vec3<f32>(0.290, 0.486, 0.137);
const DESERT_SAND: vec3<f32> = vec3<f32>(0.761, 0.651, 0.333);
const OCEAN_WATER: vec3<f32> = vec3<f32>(0.227, 0.486, 0.647);
const PLAINS_GRASS: vec3<f32> = vec3<f32>(0.486, 0.702, 0.259);
const MOUNTAIN_STONE: vec3<f32> = vec3<f32>(0.478, 0.478, 0.478);
const CAVE_STONE: vec3<f32> = vec3<f32>(0.350, 0.340, 0.350);

// Bindings
@group(0) @binding(0) var<storage, read> cells: array<Cell>;
@group(0) @binding(1) var<storage, read> colors: array<MaterialColor>;
@group(0) @binding(2) var<uniform> params: RenderParams;
@group(0) @binding(3) var<uniform> chunk_params: ChunkParams;
@group(0) @binding(4) var<uniform> atlas_params: AtlasParams;
@group(0) @binding(5) var terrain_atlas: texture_2d<f32>;
@group(0) @binding(6) var terrain_sampler: sampler;

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

// Get biome fallback color
fn get_biome_fallback_color(biome_id: u32) -> vec3<f32> {
    switch biome_id {
        case BIOME_FOREST: { return FOREST_GRASS; }
        case BIOME_DESERT: { return DESERT_SAND; }
        case BIOME_OCEAN: { return OCEAN_WATER; }
        case BIOME_PLAINS: { return PLAINS_GRASS; }
        case BIOME_MOUNTAIN: { return MOUNTAIN_STONE; }
        case BIOME_CAVE: { return CAVE_STONE; }
        default: { return FOREST_GRASS; }
    }
}

// Sample terrain atlas at world position
fn sample_terrain(world_x: i32, world_y: i32, biome_id: u32) -> vec4<f32> {
    let tile_size = atlas_params.tile_size;
    if tile_size == 0u || atlas_params.tile_count == 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0); // No atlas loaded
    }

    // Calculate local pixel position within a tile (0-47)
    let tile_x = u32(abs(world_x) % 48);
    let tile_y = u32(abs(world_y) % 48);

    // Calculate which tile we're in (world coords / 48)
    let world_tile_x = abs(world_x) / 48;
    let world_tile_y = abs(world_y) / 48;

    // Use biome_id and world position to select a tile from the atlas
    // Hash the position to get variation
    let hash_input = u32(world_tile_x + world_tile_y * 1000 + i32(biome_id) * 10000);
    let hash = (hash_input * 2654435761u) % atlas_params.tile_count;
    let tile_index = hash;

    // Calculate atlas position (tiles are arranged in rows of tiles_per_row)
    let tiles_per_row = atlas_params.atlas_width / tile_size;
    let atlas_tile_x = tile_index % tiles_per_row;
    let atlas_tile_y = tile_index / tiles_per_row;
    let atlas_px_x = atlas_tile_x * tile_size + tile_x;
    let atlas_px_y = atlas_tile_y * tile_size + tile_y;

    // Sample from atlas (normalize to 0-1 UV)
    let uv = vec2<f32>(
        f32(atlas_px_x) / f32(atlas_params.atlas_width),
        f32(atlas_px_y) / f32(atlas_params.atlas_height)
    );

    return textureSample(terrain_atlas, terrain_sampler, uv);
}

// Apply day/night lighting
fn apply_lighting(color: vec3<f32>, time_of_day: f32) -> vec3<f32> {
    // Calculate ambient light based on time of day
    // 0.0 = midnight (darkest), 0.5 = noon (brightest)
    let noon_distance = abs(time_of_day - 0.5);
    let ambient = 0.3 + 0.7 * (1.0 - noon_distance * 2.0);

    // Nighttime tint (blueish)
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
    // Darken and add orange/red tint
    let burn_color = vec3<f32>(0.8, 0.3, 0.1);
    return mix(color * 0.3, burn_color, 0.5);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate world position from screen position
    let screen_x = i32(input.position.x);
    let screen_y = i32(input.position.y);

    // Apply camera offset and zoom
    let world_x = params.camera_x + i32(f32(screen_x) / params.zoom);
    let world_y = params.camera_y + i32(f32(screen_y) / params.zoom);

    // Apply chunk offset
    let local_x = world_x - chunk_params.world_offset_x;
    let local_y = world_y - chunk_params.world_offset_y;

    // Bounds check
    if local_x < 0 || local_y < 0 || local_x >= i32(params.chunk_size) || local_y >= i32(params.chunk_size) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Get cell
    let idx = u32(local_y) * params.chunk_size + u32(local_x);
    let cell = cells[idx];

    // Extract cell data
    let material_id = cell.material & 0xFFFFu;
    let flags = (cell.material >> 16u) & 0xFFu;
    let growth = (cell.material >> 24u) & 0xFFu;
    let biome_id = (cell.velocity_data >> 16u) & 0xFFu;
    let elevation = (cell.velocity_data >> 24u) & 0xFFu;

    // Skip air
    if material_id == MAT_AIR {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Try to sample from terrain atlas
    var color: vec3<f32>;
    let tex_sample = sample_terrain(world_x, world_y, biome_id);

    if tex_sample.a > 0.1 {
        // Use texture color
        color = tex_sample.rgb;
    } else {
        // Fallback to procedural biome color
        color = get_biome_fallback_color(biome_id);

        // Apply material variation
        let mat_color = colors[material_id % 16u];
        color = mix(color, vec3<f32>(mat_color.r, mat_color.g, mat_color.b), 0.3);
    }

    // Apply elevation shading for mountains
    if biome_id == BIOME_MOUNTAIN && elevation > 128u {
        let snow_factor = f32(elevation - 128u) / 127.0;
        color = mix(color, vec3<f32>(0.95, 0.95, 0.95), snow_factor);
    }

    // Apply burning effect
    if (flags & FLAG_BURNING) != 0u {
        color = apply_burn_effect(color);
    }

    // Apply day/night lighting
    color = apply_lighting(color, params.time_of_day);

    return vec4<f32>(color, 1.0);
}
"#;

/// GPU buffer for a single chunk in textured rendering
struct TexturedChunkGpuBuffer {
    /// Cell data buffer
    cells_buffer: wgpu::Buffer,
    /// Chunk world offset params buffer
    chunk_params_buffer: wgpu::Buffer,
    /// Bind group for this chunk
    bind_group: wgpu::BindGroup,
}

/// Textured multi-chunk renderer with terrain atlas support
pub struct TexturedChunkRenderer {
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,
    /// Material color buffer
    color_buffer: wgpu::Buffer,
    /// Render params buffer
    params_buffer: wgpu::Buffer,
    /// Current render params
    params: RenderParams,
    /// Atlas params buffer
    atlas_params_buffer: wgpu::Buffer,
    /// Current atlas params
    atlas_params: TerrainAtlasParams,
    /// Per-chunk GPU buffers
    chunk_buffers: HashMap<(i32, i32), TexturedChunkGpuBuffer>,
    /// Chunk size
    chunk_size: u32,
    /// Reference to terrain atlas
    terrain_atlas: Option<Arc<RwLock<TerrainTextureAtlas>>>,
    /// Fallback texture view for when atlas is not loaded
    fallback_texture: wgpu::Texture,
    fallback_view: wgpu::TextureView,
    fallback_sampler: wgpu::Sampler,
}

impl TexturedChunkRenderer {
    /// Create a new textured chunk renderer
    pub fn new(
        device: &Device,
        surface_format: wgpu::TextureFormat,
        chunk_size: u32,
    ) -> Self {
        info!("Creating textured chunk renderer with chunk_size={}", chunk_size);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Textured Chunk Render Shader"),
            source: wgpu::ShaderSource::Wgsl(TEXTURED_RENDER_SHADER.into()),
        });

        // Create bind group layout with texture support
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Textured Chunk Render Bind Group Layout"),
            entries: &[
                // cells - storage buffer
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
                // colors - storage buffer
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
                // params - uniform buffer
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
                // chunk_params - uniform buffer
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
                // atlas_params - uniform buffer
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
                // terrain_atlas - texture
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
                // terrain_sampler - sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Textured Chunk Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Textured Chunk Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create color buffer
        let colors = create_default_colors();
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Colors Buffer"),
            contents: bytemuck::cast_slice(&colors),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create params buffer
        let mut params = RenderParams::default();
        params.chunk_size = chunk_size;
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Render Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create atlas params buffer (empty initially)
        let atlas_params = TerrainAtlasParams {
            tile_size: 0,
            tile_count: 0,
            atlas_width: 0,
            atlas_height: 0,
        };
        let atlas_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Params Buffer"),
            contents: bytemuck::bytes_of(&atlas_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create fallback 1x1 transparent texture
        let fallback_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fallback_texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let fallback_view = fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let fallback_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fallback_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        info!("Textured chunk renderer created successfully");

        Self {
            pipeline,
            bind_group_layout,
            color_buffer,
            params_buffer,
            params,
            atlas_params_buffer,
            atlas_params,
            chunk_buffers: HashMap::new(),
            chunk_size,
            terrain_atlas: None,
            fallback_texture,
            fallback_view,
            fallback_sampler,
        }
    }

    /// Set the terrain atlas
    pub fn set_terrain_atlas(
        &mut self,
        queue: &Queue,
        atlas: Arc<RwLock<TerrainTextureAtlas>>,
    ) {
        // Update atlas params
        let atlas_guard = atlas.read();
        self.atlas_params = atlas_guard.params();
        queue.write_buffer(&self.atlas_params_buffer, 0, bytemuck::bytes_of(&self.atlas_params));
        drop(atlas_guard);

        self.terrain_atlas = Some(atlas);

        // Clear chunk buffers to force rebind with new atlas
        self.chunk_buffers.clear();

        info!(
            "Textured renderer bound to terrain atlas: {} tiles, {}x{}",
            self.atlas_params.tile_count,
            self.atlas_params.atlas_width,
            self.atlas_params.atlas_height
        );
    }

    /// Update render parameters
    pub fn update_params(&mut self, queue: &Queue, params: RenderParams) {
        self.params = params;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
    }

    /// Updates render params from a Camera struct.
    #[allow(clippy::cast_possible_truncation)]
    pub fn update_camera(&mut self, queue: &Queue, camera: &Camera) {
        self.params.camera_x = camera.position.0 as i32;
        self.params.camera_y = camera.position.1 as i32;
        self.params.zoom = camera.zoom;
        self.params.screen_width = camera.viewport_size.0;
        self.params.screen_height = camera.viewport_size.1;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
    }

    /// Sets the time of day for day/night cycle.
    pub fn set_time_of_day(&mut self, queue: &Queue, time: f32) {
        self.params.time_of_day = time.clamp(0.0, 1.0);
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
    }

    /// Uploads a chunk to the GPU or updates it if already uploaded.
    #[allow(clippy::cast_possible_wrap)]
    pub fn upload_chunk(&mut self, device: &Device, queue: &Queue, chunk: &VisibleChunk) {
        let pos = chunk.position;
        let world_offset_x = pos.0 * self.chunk_size as i32;
        let world_offset_y = pos.1 * self.chunk_size as i32;

        if let Some(gpu_buffer) = self.chunk_buffers.get(&pos) {
            // Update existing buffer
            queue.write_buffer(&gpu_buffer.cells_buffer, 0, bytemuck::cast_slice(&chunk.cells));
            debug!("Updated textured chunk {:?} on GPU", pos);
        } else {
            // Create new cell buffer
            let cells_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Textured Chunk Cells Buffer {:?}", pos)),
                contents: bytemuck::cast_slice(&chunk.cells),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            // Create chunk params buffer
            let chunk_params = ChunkRenderParams::new(world_offset_x, world_offset_y);
            let chunk_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Textured Chunk Params Buffer {:?}", pos)),
                contents: bytemuck::bytes_of(&chunk_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            // Create bind group with atlas or fallback
            let bind_group = self.create_chunk_bind_group(
                device,
                &cells_buffer,
                &chunk_params_buffer,
                pos.0,
                pos.1
            );

            self.chunk_buffers.insert(pos, TexturedChunkGpuBuffer {
                cells_buffer,
                chunk_params_buffer,
                bind_group,
            });

            debug!("Uploaded new textured chunk {:?} to GPU", pos);
        }
    }

    /// Create bind group for a chunk (with atlas or fallback texture)
    fn create_chunk_bind_group(
        &self,
        device: &Device,
        cells_buffer: &wgpu::Buffer,
        chunk_params_buffer: &wgpu::Buffer,
        chunk_x: i32,
        chunk_y: i32,
    ) -> wgpu::BindGroup {
        // Try to get atlas texture view and sampler
        if let Some(atlas) = &self.terrain_atlas {
            let atlas_guard = atlas.read();
            if let (Some(view), Some(samp)) = (atlas_guard.texture_view(), atlas_guard.sampler()) {
                return device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Textured Chunk Bind Group ({}, {})", chunk_x, chunk_y)),
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
                            resource: self.atlas_params_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Sampler(samp),
                        },
                    ],
                });
            }
        }

        // Fallback to placeholder texture
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Textured Chunk Bind Group (fallback) ({}, {})", chunk_x, chunk_y)),
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
                    resource: self.atlas_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&self.fallback_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&self.fallback_sampler),
                },
            ],
        })
    }

    /// Uploads all dirty chunks from a chunk manager.
    pub fn upload_dirty_chunks(
        &mut self,
        device: &Device,
        queue: &Queue,
        chunk_manager: &mut VisibleChunkManager,
    ) {
        for chunk in chunk_manager.visible_chunks_mut() {
            if chunk.dirty {
                self.upload_chunk(device, queue, chunk);
                chunk.clear_dirty();
            }
        }
    }

    /// Removes chunks that are no longer visible.
    pub fn remove_unloaded_chunks(&mut self, chunk_manager: &VisibleChunkManager) {
        let visible: std::collections::HashSet<_> =
            chunk_manager.visible_chunks().map(|c| c.position).collect();

        self.chunk_buffers.retain(|pos, _| visible.contains(pos));
    }

    /// Synchronizes GPU buffers with the chunk manager.
    pub fn sync_with_chunk_manager(
        &mut self,
        device: &Device,
        queue: &Queue,
        chunk_manager: &mut VisibleChunkManager,
    ) {
        self.upload_dirty_chunks(device, queue, chunk_manager);
        self.remove_unloaded_chunks(chunk_manager);
    }

    /// Renders all visible chunks.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);

        for gpu_buffer in self.chunk_buffers.values() {
            render_pass.set_bind_group(0, &gpu_buffer.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }

    /// Returns the number of chunks currently on the GPU.
    #[must_use]
    pub fn gpu_chunk_count(&self) -> usize {
        self.chunk_buffers.len()
    }

    /// Returns current render params.
    #[must_use]
    pub const fn params(&self) -> &RenderParams {
        &self.params
    }

    /// Returns the chunk size.
    #[must_use]
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Check if terrain atlas is bound
    #[must_use]
    pub fn has_terrain_atlas(&self) -> bool {
        self.terrain_atlas.is_some() && self.atlas_params.tile_count > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_params_layout() {
        // Ensure struct is properly laid out for GPU
        assert_eq!(std::mem::size_of::<TerrainAtlasParams>(), 16);
    }
}
