//! Dynamic lighting system with GPU-accelerated light propagation.
//!
//! This module provides a compute-based lighting system that propagates light
//! through the world, blocked by solid cells. Supports point lights, directional
//! lights, and ambient lighting for day/night cycles.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use tracing::debug;
use wgpu::{util::DeviceExt, Device, Queue};

/// Maximum number of dynamic lights supported.
pub const MAX_LIGHTS: usize = 256;

/// Default ambient light level (0.0-1.0).
pub const DEFAULT_AMBIENT: f32 = 0.1;

/// Unique identifier for a light source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightId(u32);

impl LightId {
    /// Creates a new light ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Type of light source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum LightType {
    /// Point light that radiates in all directions.
    #[default]
    Point = 0,
    /// Directional light (sun/moon).
    Directional = 1,
    /// Ambient light affecting all cells equally.
    Ambient = 2,
}

impl LightType {
    /// Converts from raw u32 value.
    #[must_use]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Directional,
            2 => Self::Ambient,
            _ => Self::Point,
        }
    }
}

/// A dynamic light source.
#[derive(Debug, Clone, Copy)]
pub struct Light {
    /// Position in world coordinates.
    pub position: (f32, f32),
    /// Light color (RGB, 0.0-1.0).
    pub color: [f32; 3],
    /// Light intensity multiplier.
    pub intensity: f32,
    /// Light radius (for point lights).
    pub radius: f32,
    /// Type of light.
    pub light_type: LightType,
    /// Direction (for directional lights, normalized).
    pub direction: (f32, f32),
    /// Whether the light is enabled.
    pub enabled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            radius: 100.0,
            light_type: LightType::Point,
            direction: (0.0, -1.0),
            enabled: true,
        }
    }
}

impl Light {
    /// Creates a new point light.
    #[must_use]
    pub fn point(position: (f32, f32), color: [f32; 3], intensity: f32, radius: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            radius,
            light_type: LightType::Point,
            ..Default::default()
        }
    }

    /// Creates a new directional light (like sun/moon).
    #[must_use]
    pub fn directional(direction: (f32, f32), color: [f32; 3], intensity: f32) -> Self {
        // Normalize direction
        let len = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
        let dir = if len > 0.0 {
            (direction.0 / len, direction.1 / len)
        } else {
            (0.0, -1.0)
        };

        Self {
            position: (0.0, 0.0),
            color,
            intensity,
            radius: f32::MAX,
            light_type: LightType::Directional,
            direction: dir,
            ..Default::default()
        }
    }

    /// Creates an ambient light.
    #[must_use]
    pub fn ambient(color: [f32; 3], intensity: f32) -> Self {
        Self {
            position: (0.0, 0.0),
            color,
            intensity,
            radius: f32::MAX,
            light_type: LightType::Ambient,
            ..Default::default()
        }
    }

    /// Sets the light position.
    #[must_use]
    pub const fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Enables or disables the light.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// GPU-compatible light data structure.
/// Layout: 48 bytes total, aligned for GPU buffers.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GpuLight {
    /// Position XY (8 bytes).
    pub position: [f32; 2],
    /// Direction XY for directional lights (8 bytes).
    pub direction: [f32; 2],
    /// Color RGB + intensity packed as vec4 (16 bytes).
    pub color_intensity: [f32; 4],
    /// Radius (4 bytes).
    pub radius: f32,
    /// Light type: 0=point, 1=directional, 2=ambient (4 bytes).
    pub light_type: u32,
    /// Enabled flag: 1=enabled, 0=disabled (4 bytes).
    pub enabled: u32,
    /// Padding for 16-byte alignment (4 bytes).
    padding: u32,
}

impl From<&Light> for GpuLight {
    fn from(light: &Light) -> Self {
        Self {
            position: [light.position.0, light.position.1],
            direction: [light.direction.0, light.direction.1],
            color_intensity: [light.color[0], light.color[1], light.color[2], light.intensity],
            radius: light.radius,
            light_type: light.light_type as u32,
            enabled: u32::from(light.enabled),
            padding: 0,
        }
    }
}

/// GPU-compatible lighting parameters.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LightingParams {
    /// Viewport width in pixels.
    pub width: u32,
    /// Viewport height in pixels.
    pub height: u32,
    /// Number of active lights.
    pub light_count: u32,
    /// Ambient light level.
    pub ambient: f32,
    /// Time of day (0.0-1.0, 0=midnight, 0.5=noon).
    pub time_of_day: f32,
    /// Padding for alignment.
    padding: [u32; 3],
}

impl Default for LightingParams {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            light_count: 0,
            ambient: DEFAULT_AMBIENT,
            time_of_day: 0.5,
            padding: [0, 0, 0],
        }
    }
}

/// Lighting compute shader in WGSL.
pub const LIGHTING_SHADER: &str = r"
// Light structure (48 bytes, matches GpuLight)
struct Light {
    position: vec2<f32>,       // 8 bytes
    direction: vec2<f32>,      // 8 bytes
    color_intensity: vec4<f32>, // 16 bytes (rgb + intensity)
    radius: f32,               // 4 bytes
    light_type: u32,           // 4 bytes
    enabled: u32,              // 4 bytes
    _padding: u32,             // 4 bytes
}

// Lighting parameters
struct LightingParams {
    width: u32,
    height: u32,
    light_count: u32,
    ambient: f32,
    time_of_day: f32,
    _padding: vec3<u32>,
}

// Cell structure (simplified, we only need to check solidity)
struct Cell {
    material: u32,
    velocity_data: u32,
}

// Flag constants
const FLAG_SOLID: u32 = 1u;

@group(0) @binding(0) var<storage, read> lights: array<Light>;
@group(0) @binding(1) var<uniform> params: LightingParams;
@group(0) @binding(2) var<storage, read> cells: array<Cell>;
@group(0) @binding(3) var light_map: texture_storage_2d<rgba8unorm, write>;

// Check if a cell is solid (blocks light)
fn is_solid(cell: Cell) -> bool {
    let flags = (cell.material >> 16u) & 0xFFu;
    return (flags & FLAG_SOLID) != 0u;
}

// Get cell at position
fn get_cell(x: u32, y: u32) -> Cell {
    let idx = y * params.width + x;
    return cells[idx];
}

// Calculate light contribution from a point light
fn calculate_point_light(pixel_pos: vec2<f32>, light: Light) -> vec3<f32> {
    let diff = pixel_pos - light.position;
    let dist = length(diff);
    
    if dist > light.radius {
        return vec3<f32>(0.0);
    }
    
    // Smooth falloff
    let attenuation = 1.0 - smoothstep(0.0, light.radius, dist);
    let color = light.color_intensity.xyz;
    let intensity = light.color_intensity.w;
    return color * intensity * attenuation;
}

// Calculate light contribution from directional light
fn calculate_directional_light(pixel_pos: vec2<f32>, light: Light) -> vec3<f32> {
    // Directional light affects all cells equally (sun)
    let color = light.color_intensity.xyz;
    let intensity = light.color_intensity.w;
    return color * intensity;
}

// Main compute shader
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    
    if x >= params.width || y >= params.height {
        return;
    }
    
    let pixel_pos = vec2<f32>(f32(x), f32(y));
    
    // Start with ambient light
    var total_light = vec3<f32>(params.ambient);
    
    // Add day/night cycle ambient
    let day_factor = sin(params.time_of_day * 3.14159);
    total_light += vec3<f32>(0.3, 0.3, 0.4) * max(0.0, day_factor);
    
    // Check if this cell is underground (solid above)
    let is_underground = y > 0u && is_solid(get_cell(x, y - 1u));
    
    // Accumulate light from all sources
    for (var i = 0u; i < params.light_count; i++) {
        let light = lights[i];
        
        if light.enabled == 0u {
            continue;
        }
        
        var contribution = vec3<f32>(0.0);
        
        switch light.light_type {
            case 0u: { // Point light
                contribution = calculate_point_light(pixel_pos, light);
            }
            case 1u: { // Directional light (only affects surface)
                if !is_underground {
                    contribution = calculate_directional_light(pixel_pos, light);
                }
            }
            case 2u: { // Ambient
                let color = light.color_intensity.xyz;
                let intensity = light.color_intensity.w;
                contribution = color * intensity;
            }
            default: {}
        }
        
        total_light += contribution;
    }
    
    // Clamp and store
    total_light = clamp(total_light, vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(light_map, vec2<i32>(i32(x), i32(y)), vec4<f32>(total_light, 1.0));
}
";

/// GPU-accelerated lighting system.
///
/// Computes light propagation using a compute shader and stores results
/// in a light map texture that can be used during rendering.
pub struct LightingSystem {
    /// Light data buffer.
    light_buffer: wgpu::Buffer,
    /// Lighting parameters buffer.
    params_buffer: wgpu::Buffer,
    /// Light map texture.
    light_map: wgpu::Texture,
    /// Light map view.
    light_map_view: wgpu::TextureView,
    /// Compute pipeline.
    compute_pipeline: wgpu::ComputePipeline,
    /// Bind group layout.
    bind_group_layout: wgpu::BindGroupLayout,
    /// Active lights.
    lights: HashMap<LightId, Light>,
    /// Next light ID.
    next_light_id: u32,
    /// Current lighting parameters.
    params: LightingParams,
    /// Whether lights have changed and need upload.
    dirty: bool,
}

impl LightingSystem {
    /// Creates a new lighting system.
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        // Create light buffer
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: (MAX_LIGHTS * std::mem::size_of::<GpuLight>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create params buffer
        let params = LightingParams {
            width,
            height,
            ..Default::default()
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lighting Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create light map texture
        let light_map = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Light Map"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let light_map_view = light_map.create_view(&wgpu::TextureViewDescriptor::default());

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lighting Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        // Create compute pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting Shader"),
            source: wgpu::ShaderSource::Wgsl(LIGHTING_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Lighting Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Lighting Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        debug!("Created lighting system {}x{}", width, height);

        Self {
            light_buffer,
            params_buffer,
            light_map,
            light_map_view,
            compute_pipeline,
            bind_group_layout,
            lights: HashMap::new(),
            next_light_id: 0,
            params,
            dirty: true,
        }
    }

    /// Adds a light to the system.
    ///
    /// Returns a unique ID for the light, or `None` if at capacity.
    pub fn add_light(&mut self, light: Light) -> Option<LightId> {
        if self.lights.len() >= MAX_LIGHTS {
            return None;
        }

        let id = LightId::new(self.next_light_id);
        self.next_light_id += 1;
        self.lights.insert(id, light);
        self.dirty = true;

        debug!("Added light {:?}", id);
        Some(id)
    }

    /// Removes a light from the system.
    pub fn remove_light(&mut self, id: LightId) -> Option<Light> {
        let removed = self.lights.remove(&id);
        if removed.is_some() {
            self.dirty = true;
            debug!("Removed light {:?}", id);
        }
        removed
    }

    /// Updates an existing light.
    pub fn update_light(&mut self, id: LightId, light: Light) -> bool {
        use std::collections::hash_map::Entry;
        if let Entry::Occupied(mut entry) = self.lights.entry(id) {
            entry.insert(light);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Gets a light by ID.
    #[must_use]
    pub fn get_light(&self, id: LightId) -> Option<&Light> {
        self.lights.get(&id)
    }

    /// Returns the number of active lights.
    #[must_use]
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// Sets the ambient light level.
    pub fn set_ambient(&mut self, ambient: f32) {
        self.params.ambient = ambient.clamp(0.0, 1.0);
        self.dirty = true;
    }

    /// Gets the ambient light level.
    #[must_use]
    pub fn ambient(&self) -> f32 {
        self.params.ambient
    }

    /// Sets the time of day (0.0-1.0, 0=midnight, 0.5=noon).
    pub fn set_time_of_day(&mut self, time: f32) {
        self.params.time_of_day = time.rem_euclid(1.0);
        self.dirty = true;
    }

    /// Gets the time of day.
    #[must_use]
    pub fn time_of_day(&self) -> f32 {
        self.params.time_of_day
    }

    /// Uploads light data to the GPU.
    pub fn upload(&mut self, queue: &Queue) {
        if !self.dirty {
            return;
        }

        // Prepare light data
        let mut gpu_lights = vec![GpuLight::zeroed(); MAX_LIGHTS];
        for (i, light) in self.lights.values().enumerate() {
            if i >= MAX_LIGHTS {
                break;
            }
            gpu_lights[i] = GpuLight::from(light);
        }

        // Upload lights
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&gpu_lights));

        // Update params
        self.params.light_count = self.lights.len() as u32;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));

        self.dirty = false;
    }

    /// Computes lighting into the light map.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `encoder` - Command encoder
    /// * `cell_buffer` - Buffer containing cell data
    pub fn compute_lighting(
        &self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        cell_buffer: &wgpu::Buffer,
    ) {
        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lighting Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cell_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.light_map_view),
                },
            ],
        });

        // Dispatch compute
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Lighting Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        // Workgroup size is 16x16
        let workgroups_x = self.params.width.div_ceil(16);
        let workgroups_y = self.params.height.div_ceil(16);
        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }

    /// Gets the light map texture.
    #[must_use]
    pub fn get_light_map(&self) -> &wgpu::Texture {
        &self.light_map
    }

    /// Gets the light map texture view.
    #[must_use]
    pub fn get_light_map_view(&self) -> &wgpu::TextureView {
        &self.light_map_view
    }

    /// Gets the bind group layout for rendering.
    #[must_use]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Clears all lights.
    pub fn clear(&mut self) {
        self.lights.clear();
        self.dirty = true;
    }
}

impl std::fmt::Debug for LightingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LightingSystem")
            .field("light_count", &self.lights.len())
            .field("params", &self.params)
            .field("dirty", &self.dirty)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_id() {
        let id = LightId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_light_type_conversion() {
        assert_eq!(LightType::from_u32(0), LightType::Point);
        assert_eq!(LightType::from_u32(1), LightType::Directional);
        assert_eq!(LightType::from_u32(2), LightType::Ambient);
        assert_eq!(LightType::from_u32(99), LightType::Point); // fallback
    }

    #[test]
    fn test_light_creation() {
        let point = Light::point((100.0, 200.0), [1.0, 0.5, 0.0], 2.0, 50.0);
        assert!((point.position.0 - 100.0).abs() < f32::EPSILON);
        assert!((point.position.1 - 200.0).abs() < f32::EPSILON);
        assert_eq!(point.light_type, LightType::Point);
        assert!((point.radius - 50.0).abs() < f32::EPSILON);

        let dir = Light::directional((1.0, -1.0), [1.0, 1.0, 0.9], 1.0);
        assert_eq!(dir.light_type, LightType::Directional);

        let ambient = Light::ambient([0.1, 0.1, 0.15], 1.0);
        assert_eq!(ambient.light_type, LightType::Ambient);
    }

    #[test]
    fn test_light_builder() {
        let light = Light::default()
            .with_position(50.0, 75.0)
            .with_enabled(false);

        assert!((light.position.0 - 50.0).abs() < f32::EPSILON);
        assert!((light.position.1 - 75.0).abs() < f32::EPSILON);
        assert!(!light.enabled);
    }

    #[test]
    fn test_gpu_light_from_light() {
        let light = Light::point((10.0, 20.0), [1.0, 0.0, 0.0], 1.5, 100.0);
        let gpu: GpuLight = (&light).into();

        assert!((gpu.position[0] - 10.0).abs() < f32::EPSILON);
        assert!((gpu.position[1] - 20.0).abs() < f32::EPSILON);
        // color_intensity packs color RGB and intensity
        assert!((gpu.color_intensity[0] - 1.0).abs() < f32::EPSILON);
        assert!((gpu.color_intensity[1] - 0.0).abs() < f32::EPSILON);
        assert!((gpu.color_intensity[2] - 0.0).abs() < f32::EPSILON);
        assert!((gpu.color_intensity[3] - 1.5).abs() < f32::EPSILON);
        assert!((gpu.radius - 100.0).abs() < f32::EPSILON);
        assert_eq!(gpu.light_type, 0);
        assert_eq!(gpu.enabled, 1);
    }

    #[test]
    fn test_lighting_params_default() {
        let params = LightingParams::default();
        assert!((params.ambient - DEFAULT_AMBIENT).abs() < f32::EPSILON);
        assert!((params.time_of_day - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gpu_light_size() {
        // Ensure proper alignment for GPU
        assert_eq!(std::mem::size_of::<GpuLight>(), 48);
    }

    #[test]
    fn test_lighting_params_size() {
        // Check struct size for GPU compatibility
        assert_eq!(std::mem::size_of::<LightingParams>(), 32);
    }

    #[test]
    fn test_directional_light_normalization() {
        let light = Light::directional((3.0, 4.0), [1.0, 1.0, 1.0], 1.0);
        let len = (light.direction.0.powi(2) + light.direction.1.powi(2)).sqrt();
        assert!((len - 1.0).abs() < 0.001);
    }
}
