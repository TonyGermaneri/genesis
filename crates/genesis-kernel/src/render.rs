//! Cell rendering pipeline for visualizing simulation state.
//!
//! This module provides the render pipeline for displaying cell buffers
//! as colored pixels on screen. It reads from the compute output buffer
//! and maps material IDs to colors.

use bytemuck::{Pod, Zeroable};
use tracing::info;
use wgpu::{util::DeviceExt, Device};

/// Number of builtin material colors
pub const NUM_MATERIAL_COLORS: usize = 16;

/// Material color mapping for rendering.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct MaterialColor {
    /// Red component (0.0-1.0 range)
    pub r: f32,
    /// Green component (0.0-1.0 range)
    pub g: f32,
    /// Blue component (0.0-1.0 range)
    pub b: f32,
    /// Alpha component (0.0-1.0 range)
    pub a: f32,
}

impl MaterialColor {
    /// Creates a new material color.
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from RGB bytes (0-255).
    #[must_use]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }
}

/// Render parameters for the cell render shader.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RenderParams {
    /// Chunk size in cells
    pub chunk_size: u32,
    /// Screen width in pixels
    pub screen_width: u32,
    /// Screen height in pixels
    pub screen_height: u32,
    /// Camera X offset (in cells)
    pub camera_x: i32,
    /// Camera Y offset (in cells)
    pub camera_y: i32,
    /// Zoom level (pixels per cell)
    pub zoom: f32,
    /// Padding for alignment
    _padding: [u32; 2],
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            chunk_size: 256,
            screen_width: 1280,
            screen_height: 720,
            camera_x: 0,
            camera_y: 0,
            zoom: 2.0,
            _padding: [0; 2],
        }
    }
}

impl RenderParams {
    /// Creates new render parameters.
    #[must_use]
    pub const fn new(chunk_size: u32, screen_width: u32, screen_height: u32) -> Self {
        Self {
            chunk_size,
            screen_width,
            screen_height,
            camera_x: 0,
            camera_y: 0,
            zoom: 2.0,
            _padding: [0; 2],
        }
    }

    /// Sets the camera position.
    pub fn set_camera(&mut self, x: i32, y: i32) {
        self.camera_x = x;
        self.camera_y = y;
    }

    /// Sets the zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.1);
    }
}

/// Cell render shader in WGSL.
///
/// This shader reads from the cell buffer and outputs colored pixels.
pub const CELL_RENDER_SHADER: &str = r"
// Cell structure (must match compute shader)
struct Cell {
    material: u32,      // u16 material + u8 flags + u8 temperature
    velocity_data: u32, // i8 vel_x + i8 vel_y + u16 data
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
    _padding: vec2<u32>,
}

// Cell flag bits
const FLAG_BURNING: u32 = 4u;

// Bindings
@group(0) @binding(0) var<storage, read> cells: array<Cell>;
@group(0) @binding(1) var<storage, read> colors: array<MaterialColor>;
@group(0) @binding(2) var<uniform> params: RenderParams;

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Fullscreen triangle vertices
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate fullscreen triangle
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);
    
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    
    return out;
}

// Helper: get material ID from cell
fn get_material(cell: Cell) -> u32 {
    return cell.material & 0xFFFFu;
}

// Helper: get flags from cell
fn get_flags(cell: Cell) -> u32 {
    return (cell.material >> 16u) & 0xFFu;
}

// Helper: get temperature from cell
fn get_temperature(cell: Cell) -> u32 {
    return (cell.material >> 24u) & 0xFFu;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to pixel coordinates
    let pixel_x = i32(in.uv.x * f32(params.screen_width));
    let pixel_y = i32(in.uv.y * f32(params.screen_height));
    
    // Convert pixel to cell coordinates (accounting for camera and zoom)
    let cell_x = i32(f32(pixel_x) / params.zoom) + params.camera_x;
    let cell_y = i32(f32(pixel_y) / params.zoom) + params.camera_y;
    
    // Bounds check
    let size = i32(params.chunk_size);
    if cell_x < 0 || cell_y < 0 || cell_x >= size || cell_y >= size {
        // Out of bounds - dark background
        return vec4<f32>(0.02, 0.02, 0.05, 1.0);
    }
    
    // Get cell at coordinates
    let idx = u32(cell_y) * params.chunk_size + u32(cell_x);
    let cell = cells[idx];
    
    // Get base color from material
    let material_id = get_material(cell);
    let num_colors = arrayLength(&colors);
    let color_idx = min(material_id, num_colors - 1u);
    var color = colors[color_idx];
    
    // Modulate color based on cell state
    let flags = get_flags(cell);
    let temp = get_temperature(cell);
    
    // Add burning effect (orange glow)
    if (flags & FLAG_BURNING) != 0u {
        color.r = min(color.r + 0.5, 1.0);
        color.g = min(color.g + 0.2, 1.0);
    }
    
    // Temperature visualization (subtle)
    let temp_factor = f32(temp) / 255.0;
    if temp_factor > 0.5 {
        // Hot - shift towards red
        color.r = min(color.r + (temp_factor - 0.5) * 0.3, 1.0);
    }
    
    return vec4<f32>(color.r, color.g, color.b, color.a);
}
";

/// Creates default material colors for visualization.
#[must_use]
pub fn create_default_colors() -> Vec<MaterialColor> {
    vec![
        MaterialColor::from_rgb(20, 20, 30),    // 0: Air (dark)
        MaterialColor::from_rgb(64, 164, 223),  // 1: Water (blue)
        MaterialColor::from_rgb(194, 178, 128), // 2: Sand (tan)
        MaterialColor::from_rgb(86, 125, 70),   // 3: Grass (green)
        MaterialColor::from_rgb(139, 90, 43),   // 4: Dirt (brown)
        MaterialColor::from_rgb(128, 128, 128), // 5: Stone (gray)
        MaterialColor::from_rgb(240, 250, 255), // 6: Snow (white)
        MaterialColor::from_rgb(192, 192, 192), // 7: Metal (silver)
        MaterialColor::from_rgb(139, 90, 43),   // 8: Wood (brown)
        MaterialColor::from_rgb(200, 220, 255), // 9: Glass (light blue)
        MaterialColor::from_rgb(160, 160, 160), // 10: Concrete (gray)
        MaterialColor::from_rgb(255, 100, 50),  // 11: Lava (orange)
        MaterialColor::from_rgb(150, 75, 0),    // 12: Oil (dark brown)
        MaterialColor::from_rgb(200, 200, 50),  // 13: Acid (yellow-green)
        MaterialColor::from_rgb(100, 50, 150),  // 14: Plasma (purple)
        MaterialColor::from_rgb(255, 255, 255), // 15: Light (white)
    ]
}

/// Cell render pipeline for visualizing simulation state.
pub struct CellRenderPipeline {
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,
    /// Color buffer
    color_buffer: wgpu::Buffer,
    /// Params buffer
    params_buffer: wgpu::Buffer,
    /// Current render params
    params: RenderParams,
}

impl CellRenderPipeline {
    /// Creates a new cell render pipeline.
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        info!("Creating cell render pipeline...");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cell Render Shader"),
            source: wgpu::ShaderSource::Wgsl(CELL_RENDER_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cell Render Bind Group Layout"),
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
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cell Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cell Render Pipeline"),
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
                    blend: Some(wgpu::BlendState::REPLACE),
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
        let params = RenderParams::default();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Render Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        info!("Cell render pipeline created successfully");

        Self {
            pipeline,
            bind_group_layout,
            color_buffer,
            params_buffer,
            params,
        }
    }

    /// Updates render parameters.
    pub fn update_params(&mut self, queue: &wgpu::Queue, params: RenderParams) {
        self.params = params;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
    }

    /// Sets the screen size.
    pub fn set_screen_size(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.params.screen_width = width;
        self.params.screen_height = height;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
    }

    /// Sets the camera position.
    pub fn set_camera(&mut self, queue: &wgpu::Queue, x: i32, y: i32) {
        self.params.camera_x = x;
        self.params.camera_y = y;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
    }

    /// Creates a bind group for rendering.
    #[must_use]
    pub fn create_bind_group(
        &self,
        device: &Device,
        cell_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cell Render Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cell_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.color_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Renders cells to the given render pass.
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        bind_group: &'a wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1); // Fullscreen triangle
    }

    /// Returns current render params.
    #[must_use]
    pub const fn params(&self) -> &RenderParams {
        &self.params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_color_from_rgb() {
        let color = MaterialColor::from_rgb(255, 128, 0);
        assert!((color.r - 1.0).abs() < f32::EPSILON);
        assert!((color.g - 0.501_960_8).abs() < 0.001);
        assert!((color.b - 0.0).abs() < f32::EPSILON);
        assert!((color.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_render_params_default() {
        let params = RenderParams::default();
        assert_eq!(params.chunk_size, 256);
        assert_eq!(params.camera_x, 0);
        assert_eq!(params.camera_y, 0);
    }

    #[test]
    fn test_render_params_size() {
        // Ensure params are properly aligned for GPU uniform buffers
        assert_eq!(std::mem::size_of::<RenderParams>(), 32);
    }

    #[test]
    fn test_default_colors() {
        let colors = create_default_colors();
        assert_eq!(colors.len(), 16);

        // Air should be dark
        assert!(colors[0].r < 0.2);
        assert!(colors[0].g < 0.2);

        // Water should be blue
        assert!(colors[1].b > colors[1].r);
    }
}
