//! Terrain tile rendering system for biome-based world generation.
//!
//! Renders a grid of colored tiles based on biome data from cubiomes.
//! Each tile represents one biome cell, rendered as a solid-color quad
//! with heightmap-based shading and time-of-day shadow casting.

use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use tracing::info;
use wgpu::util::DeviceExt;

const TERRAIN_SHADER: &str = "
struct CameraUniform {
    camera_pos: vec2<f32>,
    screen_size: vec2<f32>,
    zoom: f32,
    time_of_day: f32,
    sun_intensity: f32,
    _pad0: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) tile_pos: vec2<f32>,
    @location(2) tile_size: vec2<f32>,
    @location(3) tile_color: vec4<f32>,
    @location(4) height: f32,
    @location(5) height_deltas: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) height: f32,
    @location(2) height_deltas: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = instance.tile_pos + vertex.position * instance.tile_size;
    let relative = world_pos - camera.camera_pos;
    let screen_pos = relative * camera.zoom / (camera.screen_size * 0.5);
    out.clip_position = vec4<f32>(screen_pos.x, -screen_pos.y, 0.0, 1.0);
    out.color = instance.tile_color;
    out.height = instance.height;
    out.height_deltas = instance.height_deltas;
    return out;
}

// Cubiomes-viewer style height shading.
// Uses a fixed NW->SE directional light based on height gradients,
// plus contour lines at 16-block height intervals.

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = in.color.rgb;

    // Height deltas: north, east, south, west (relative to center)
    let d_north = in.height_deltas.x;
    let d_east  = in.height_deltas.y;
    let d_south = in.height_deltas.z;
    let d_west  = in.height_deltas.w;

    // cubiomes-viewer shading: d0 = NW sum (north + west), d1 = SE sum (east + south)
    // light = 1.0 + (d1 - d0) * mul, where mul = 0.25 for scale=1
    let d0 = d_north + d_west;
    let d1 = d_east + d_south;
    var light = 1.0 + (d1 - d0) * 0.25;

    // Clamp light between 0.5 and 1.5 (matching cubiomes-viewer lmin/lmax)
    light = clamp(light, 0.5, 1.5);

    // Contour lines at 16-block height intervals (cubiomes-viewer style)
    // If any neighbor crosses a 16-block boundary relative to center, darken
    let center_h = in.height;
    let h_north = center_h + d_north;
    let h_east  = center_h + d_east;
    let h_south = center_h + d_south;
    let h_west  = center_h + d_west;
    let min_neighbor = min(min(h_north, h_east), min(h_south, h_west));
    let center_contour = floor(center_h / 16.0);
    let neighbor_contour = floor(min_neighbor / 16.0);
    if (center_contour != neighbor_contour) {
        light = light * 0.5;
    }

    let final_color = clamp(base * light, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(final_color, in.color.a);
}
";

/// Instance data for a single terrain tile (56 bytes = 14 x f32).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TerrainTileInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub height: f32,
    pub height_deltas: [f32; 4],
    pub _pad: [f32; 1],
}

/// Camera uniform for terrain rendering (32 bytes = 8 x f32).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct TerrainCameraUniform {
    camera_pos: [f32; 2],
    screen_size: [f32; 2],
    zoom: f32,
    time_of_day: f32,
    sun_intensity: f32,
    _pad0: f32,
}

#[allow(dead_code)]
struct CachedChunk {
    chunk_x: i32,
    chunk_y: i32,
    instances: Vec<TerrainTileInstance>,
}

pub struct TerrainRenderConfig {
    pub tile_size: f32,
    pub biome_scale: i32,
    pub render_radius: i32,
}

impl Default for TerrainRenderConfig {
    fn default() -> Self {
        Self {
            tile_size: 16.0,
            biome_scale: 4,
            render_radius: 8,
        }
    }
}

pub struct TerrainTileRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    max_instances: usize,
    instance_count: u32,
    cached_chunks: HashMap<(i32, i32), CachedChunk>,
    config: TerrainRenderConfig,
    enabled: bool,
}

impl TerrainTileRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let config = TerrainRenderConfig::default();
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain Tile Shader"),
            source: wgpu::ShaderSource::Wgsl(TERRAIN_SHADER.into()),
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Terrain Camera Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        let instance_stride = std::mem::size_of::<TerrainTileInstance>() as u64;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: instance_stride,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 3,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 32,
                                shader_location: 4,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 36,
                                shader_location: 5,
                            },
                        ],
                    },
                ],
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
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let vertices: &[[f32; 2]] = &[
            [-0.5, -0.5], [ 0.5, -0.5], [ 0.5,  0.5], [-0.5,  0.5],
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let max_instances = 65536;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Instance Buffer"),
            size: (max_instances * std::mem::size_of::<TerrainTileInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Terrain Camera Buffer"),
            size: std::mem::size_of::<TerrainCameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Terrain Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        info!(
            "Terrain tile renderer initialized (max {} instances, {} bytes/instance)",
            max_instances,
            std::mem::size_of::<TerrainTileInstance>()
        );
        Self {
            pipeline, vertex_buffer, index_buffer, instance_buffer,
            camera_buffer, camera_bind_group, max_instances,
            instance_count: 0, cached_chunks: HashMap::new(),
            config, enabled: false,
        }
    }

    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }
    pub fn is_enabled(&self) -> bool { self.enabled }
    pub fn set_config(&mut self, config: TerrainRenderConfig) { self.config = config; }
    pub fn config(&self) -> &TerrainRenderConfig { &self.config }
    pub fn clear_cache(&mut self) { self.cached_chunks.clear(); }

    pub fn cache_chunk(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        biomes: &[i32],
        heights: &[f32],
        width: i32,
        height: i32,
        colors: &dyn Fn(i32) -> [u8; 3],
    ) {
        let tile_size = self.config.tile_size;
        let w = width as usize;
        let h = height as usize;
        let mut instances = Vec::with_capacity(w * h);
        for by in 0..h {
            for bx in 0..w {
                let idx = by * w + bx;
                let biome_id = biomes[idx];
                let color = colors(biome_id);
                let self_h = heights[idx];
                let h_north = if by > 0     { heights[(by - 1) * w + bx] } else { self_h };
                let h_south = if by + 1 < h { heights[(by + 1) * w + bx] } else { self_h };
                let h_east  = if bx + 1 < w { heights[by * w + bx + 1]  } else { self_h };
                let h_west  = if bx > 0     { heights[by * w + bx - 1]  } else { self_h };
                let world_x = (chunk_x * width  + bx as i32) as f32 * tile_size;
                let world_y = (chunk_y * height + by as i32) as f32 * tile_size;
                instances.push(TerrainTileInstance {
                    position: [world_x, world_y],
                    size: [tile_size, tile_size],
                    color: [
                        color[0] as f32 / 255.0,
                        color[1] as f32 / 255.0,
                        color[2] as f32 / 255.0,
                        1.0,
                    ],
                    height: self_h,
                    height_deltas: [
                        h_north - self_h,
                        h_east  - self_h,
                        h_south - self_h,
                        h_west  - self_h,
                    ],
                    _pad: [0.0],
                });
            }
        }
        self.cached_chunks.insert(
            (chunk_x, chunk_y),
            CachedChunk { chunk_x, chunk_y, instances },
        );
    }

    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        camera_pos: (f32, f32),
        screen_size: (u32, u32),
        zoom: f32,
        time_of_day: f32,
        sun_intensity: f32,
    ) {
        let uniform = TerrainCameraUniform {
            camera_pos: [camera_pos.0, camera_pos.1],
            screen_size: [screen_size.0 as f32, screen_size.1 as f32],
            zoom,
            time_of_day,
            sun_intensity,
            _pad0: 0.0,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn update_visible_tiles(
        &mut self,
        queue: &wgpu::Queue,
        camera_pos: (f32, f32),
        zoom: f32,
        screen_size: (u32, u32),
    ) {
        if !self.enabled {
            self.instance_count = 0;
            return;
        }
        let tile_size = self.config.tile_size;
        let chunk_size = 16;
        let half_w = screen_size.0 as f32 / (2.0 * zoom);
        let half_h = screen_size.1 as f32 / (2.0 * zoom);
        let chunk_span = chunk_size as f32 * tile_size;
        let chunk_left   = ((camera_pos.0 - half_w) / chunk_span).floor() as i32 - 1;
        let chunk_right  = ((camera_pos.0 + half_w) / chunk_span).ceil()  as i32 + 1;
        let chunk_top    = ((camera_pos.1 - half_h) / chunk_span).floor() as i32 - 1;
        let chunk_bottom = ((camera_pos.1 + half_h) / chunk_span).ceil()  as i32 + 1;
        let mut all_instances: Vec<TerrainTileInstance> = Vec::new();
        for cy in chunk_top..=chunk_bottom {
            for cx in chunk_left..=chunk_right {
                if let Some(chunk) = self.cached_chunks.get(&(cx, cy)) {
                    all_instances.extend_from_slice(&chunk.instances);
                }
            }
        }
        let count = all_instances.len().min(self.max_instances);
        self.instance_count = count as u32;
        if count > 0 {
            queue.write_buffer(
                &self.instance_buffer, 0,
                bytemuck::cast_slice(&all_instances[..count]),
            );
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.enabled || self.instance_count == 0 { return; }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..self.instance_count);
    }

    pub fn cached_chunk_count(&self) -> usize { self.cached_chunks.len() }
    pub fn instance_count(&self) -> u32 { self.instance_count }
    pub fn cached_chunk_coords(&self) -> Vec<(i32, i32)> {
        self.cached_chunks.keys().copied().collect()
    }
    pub fn is_chunk_cached(&self, chunk_x: i32, chunk_y: i32) -> bool {
        self.cached_chunks.contains_key(&(chunk_x, chunk_y))
    }
}
