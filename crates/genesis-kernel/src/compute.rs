//! GPU compute pipeline for cell simulation.
//!
//! This module provides the core GPU compute infrastructure for pixel-cell simulation.
//! It includes the WGSL compute shader and Rust bindings for cell buffers, materials,
//! and simulation parameters.

use bytemuck::{Pod, Zeroable};
use tracing::info;
use wgpu::{util::DeviceExt, Device};

use crate::cell::{Cell, CellFlags, MaterialProperties};

/// Default chunk size in cells (256x256 = 65536 cells per chunk)
pub const DEFAULT_CHUNK_SIZE: u32 = 256;

/// Workgroup size for compute shader (16x16 = 256 threads per workgroup)
pub const WORKGROUP_SIZE: u32 = 16;

/// Maximum number of materials supported
pub const MAX_MATERIALS: u32 = 1024;

/// Cell simulation compute shader in WGSL.
///
/// This shader implements pixel-cell physics simulation:
/// - Gravity and falling for non-solid materials
/// - Liquid flow simulation
/// - Temperature propagation
/// - Collision detection
pub const CELL_SIMULATION_SHADER: &str = r"
// Cell structure (8 bytes, matches Rust Cell struct)
struct Cell {
    material: u32,      // u16 material + u8 flags + u8 temperature
    velocity_data: u32, // i8 vel_x + i8 vel_y + u16 data
}

// Material properties (8 bytes, matches Rust MaterialProperties)
struct MaterialProps {
    density_friction: u32,    // u16 density + u8 friction + u8 flammability
    conductivity_flags: u32,  // u8 conductivity + u8 hardness + u8 flags + u8 reserved
}

// Simulation parameters passed as uniforms
struct SimParams {
    chunk_size: u32,
    frame: u32,
    gravity: f32,
    _padding: u32,
}

// Cell flag bits (must match Rust CellFlags)
const FLAG_SOLID: u32 = 1u;
const FLAG_LIQUID: u32 = 2u;
const FLAG_BURNING: u32 = 4u;
const FLAG_ELECTRIC: u32 = 8u;
const FLAG_UPDATED: u32 = 16u;

// Bindings
@group(0) @binding(0) var<storage, read> cells_in: array<Cell>;
@group(0) @binding(1) var<storage, read_write> cells_out: array<Cell>;
@group(0) @binding(2) var<storage, read> materials: array<MaterialProps>;
@group(0) @binding(3) var<uniform> params: SimParams;

// Helper: extract material ID from packed cell
fn get_material(cell: Cell) -> u32 {
    return cell.material & 0xFFFFu;
}

// Helper: extract flags from packed cell
fn get_flags(cell: Cell) -> u32 {
    return (cell.material >> 16u) & 0xFFu;
}

// Helper: extract temperature from packed cell
fn get_temperature(cell: Cell) -> u32 {
    return (cell.material >> 24u) & 0xFFu;
}

// Helper: extract velocity X from packed cell
fn get_velocity_x(cell: Cell) -> i32 {
    return i32(cell.velocity_data & 0xFFu) - 128;
}

// Helper: extract velocity Y from packed cell
fn get_velocity_y(cell: Cell) -> i32 {
    return i32((cell.velocity_data >> 8u) & 0xFFu) - 128;
}

// Helper: extract data from packed cell
fn get_data(cell: Cell) -> u32 {
    return (cell.velocity_data >> 16u) & 0xFFFFu;
}

// Helper: pack cell components back into Cell struct
fn pack_cell(material: u32, flags: u32, temp: u32, vel_x: i32, vel_y: i32, data: u32) -> Cell {
    var cell: Cell;
    cell.material = material | (flags << 16u) | (temp << 24u);
    let vx = u32(vel_x + 128) & 0xFFu;
    let vy = u32(vel_y + 128) & 0xFFu;
    cell.velocity_data = vx | (vy << 8u) | (data << 16u);
    return cell;
}

// Helper: get density from material properties
fn get_density(mat_idx: u32) -> u32 {
    if mat_idx >= arrayLength(&materials) {
        return 0u;
    }
    return materials[mat_idx].density_friction & 0xFFFFu;
}

// Helper: check if material is solid
fn is_solid(flags: u32) -> bool {
    return (flags & FLAG_SOLID) != 0u;
}

// Helper: check if material is liquid
fn is_liquid(flags: u32) -> bool {
    return (flags & FLAG_LIQUID) != 0u;
}

// Helper: convert 2D coordinates to linear index
fn coord_to_idx(x: u32, y: u32, size: u32) -> u32 {
    return y * size + x;
}

// Helper: check if coordinates are in bounds
fn in_bounds(x: i32, y: i32, size: u32) -> bool {
    return x >= 0 && y >= 0 && u32(x) < size && u32(y) < size;
}

// Get cell at position, returns air cell if out of bounds
fn get_cell(x: i32, y: i32, size: u32) -> Cell {
    if !in_bounds(x, y, size) {
        // Return air cell
        return pack_cell(0u, 0u, 20u, 0, 0, 0u);
    }
    return cells_in[coord_to_idx(u32(x), u32(y), size)];
}

// Main compute shader entry point
@compute @workgroup_size(16, 16)
fn simulate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    let size = params.chunk_size;

    // Bounds check
    if x >= size || y >= size {
        return;
    }

    let idx = coord_to_idx(x, y, size);
    let cell = cells_in[idx];

    let material = get_material(cell);
    let flags = get_flags(cell);
    let temp = get_temperature(cell);
    var vel_x = get_velocity_x(cell);
    var vel_y = get_velocity_y(cell);
    let data = get_data(cell);

    // Air cells don't simulate
    if material == 0u {
        cells_out[idx] = cell;
        return;
    }

    let density = get_density(material);
    let is_solid_cell = is_solid(flags);
    let is_liquid_cell = is_liquid(flags);

    // Solid cells don't move (but can be modified by intents)
    if is_solid_cell {
        cells_out[idx] = cell;
        return;
    }

    // Gravity simulation for non-solid cells
    let below = get_cell(i32(x), i32(y) + 1, size);
    let below_material = get_material(below);
    let below_flags = get_flags(below);
    let below_density = get_density(below_material);

    // Check if we can fall down
    if below_material == 0u || (!is_solid(below_flags) && below_density < density) {
        // Swap with cell below - output this cell's position as the cell below
        // The cell below us will write itself here
        // For simplicity, we mark for swap by clearing this cell
        cells_out[idx] = pack_cell(0u, 0u, 20u, 0, 0, 0u);

        // Apply gravity to velocity
        vel_y = min(vel_y + 1, 127);

        // Write ourselves to position below (if in bounds)
        if in_bounds(i32(x), i32(y) + 1, size) {
            let below_idx = coord_to_idx(x, y + 1u, size);
            cells_out[below_idx] = pack_cell(material, flags | FLAG_UPDATED, temp, vel_x, vel_y, data);
        }
        return;
    }

    // Liquid spreading simulation
    if is_liquid_cell {
        // Try to spread horizontally with some randomness based on frame and position
        let rand_seed = params.frame + x * 31u + y * 17u;
        let try_left_first = (rand_seed % 2u) == 0u;

        let left = get_cell(i32(x) - 1, i32(y), size);
        let right = get_cell(i32(x) + 1, i32(y), size);
        let left_material = get_material(left);
        let right_material = get_material(right);

        var moved = false;

        if try_left_first {
            if left_material == 0u && in_bounds(i32(x) - 1, i32(y), size) {
                cells_out[idx] = pack_cell(0u, 0u, 20u, 0, 0, 0u);
                let left_idx = coord_to_idx(x - 1u, y, size);
                cells_out[left_idx] = pack_cell(material, flags | FLAG_UPDATED, temp, -1, vel_y, data);
                moved = true;
            } else if right_material == 0u && in_bounds(i32(x) + 1, i32(y), size) {
                cells_out[idx] = pack_cell(0u, 0u, 20u, 0, 0, 0u);
                let right_idx = coord_to_idx(x + 1u, y, size);
                cells_out[right_idx] = pack_cell(material, flags | FLAG_UPDATED, temp, 1, vel_y, data);
                moved = true;
            }
        } else {
            if right_material == 0u && in_bounds(i32(x) + 1, i32(y), size) {
                cells_out[idx] = pack_cell(0u, 0u, 20u, 0, 0, 0u);
                let right_idx = coord_to_idx(x + 1u, y, size);
                cells_out[right_idx] = pack_cell(material, flags | FLAG_UPDATED, temp, 1, vel_y, data);
                moved = true;
            } else if left_material == 0u && in_bounds(i32(x) - 1, i32(y), size) {
                cells_out[idx] = pack_cell(0u, 0u, 20u, 0, 0, 0u);
                let left_idx = coord_to_idx(x - 1u, y, size);
                cells_out[left_idx] = pack_cell(material, flags | FLAG_UPDATED, temp, -1, vel_y, data);
                moved = true;
            }
        }

        if moved {
            return;
        }
    }

    // Cell didn't move, copy to output
    cells_out[idx] = pack_cell(material, flags & ~FLAG_UPDATED, temp, vel_x, vel_y, data);
}
";

/// Simulation parameters passed to the compute shader as uniforms.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SimulationParams {
    /// Chunk size in cells (width and height)
    pub chunk_size: u32,
    /// Current frame number (for deterministic randomness)
    pub frame: u32,
    /// Gravity strength (typically 1.0)
    pub gravity: f32,
    /// Padding for alignment
    _padding: u32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            frame: 0,
            gravity: 1.0,
            _padding: 0,
        }
    }
}

impl SimulationParams {
    /// Creates new simulation parameters with the given chunk size.
    #[must_use]
    pub const fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            frame: 0,
            gravity: 1.0,
            _padding: 0,
        }
    }

    /// Advances the frame counter.
    pub fn advance_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }
}

/// GPU compute pipeline for pixel-cell simulation.
///
/// This pipeline manages the WGSL compute shader and provides methods
/// to dispatch simulation steps on cell buffers.
pub struct CellComputePipeline {
    /// Compute pipeline
    pipeline: wgpu::ComputePipeline,
    /// Bind group layout for cell buffers
    bind_group_layout: wgpu::BindGroupLayout,
    /// Whether the pipeline is ready
    ready: bool,
}

impl CellComputePipeline {
    /// Creates a new compute pipeline.
    ///
    /// # Arguments
    /// * `device` - The wgpu device to create resources on
    ///
    /// # Returns
    /// A new `CellComputePipeline` ready for dispatching simulation.
    pub fn new(device: &Device) -> Self {
        info!("Creating cell compute pipeline...");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cell Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(CELL_SIMULATION_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cell Bind Group Layout"),
            entries: &[
                // cells_in - read-only input buffer
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
                // cells_out - read-write output buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // materials - read-only material properties LUT
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
                // params - simulation parameters uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
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
            label: Some("Cell Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cell Simulation Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("simulate"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        info!("Cell compute pipeline created successfully");

        Self {
            pipeline,
            bind_group_layout,
            ready: true,
        }
    }

    /// Checks if the pipeline is ready for use.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.ready
    }

    /// Returns the bind group layout.
    #[must_use]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Returns a reference to the compute pipeline.
    #[must_use]
    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    /// Creates a bind group for cell simulation.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `cells_in` - Input cell buffer (read-only)
    /// * `cells_out` - Output cell buffer (read-write)
    /// * `materials` - Material properties buffer
    /// * `params` - Simulation parameters uniform buffer
    #[must_use]
    pub fn create_bind_group(
        &self,
        device: &Device,
        cells_in: &wgpu::Buffer,
        cells_out: &wgpu::Buffer,
        materials: &wgpu::Buffer,
        params: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cell Simulation Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cells_in.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cells_out.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: materials.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params.as_entire_binding(),
                },
            ],
        })
    }

    /// Dispatches the compute shader for a single simulation step.
    ///
    /// # Arguments
    /// * `encoder` - Command encoder to record the dispatch
    /// * `bind_group` - Bind group with cell buffers
    /// * `chunk_size` - Size of the chunk in cells (width and height)
    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_group: &wgpu::BindGroup,
        chunk_size: u32,
    ) {
        let workgroups = chunk_size.div_ceil(WORKGROUP_SIZE);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Cell Simulation Pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 1);
    }
}

/// Creates default material properties for builtin materials.
///
/// This matches the materials defined in `spec/schemas/cell_format.ron`.
#[must_use]
pub fn create_default_materials() -> Vec<MaterialProperties> {
    let mut materials = vec![MaterialProperties::default(); MAX_MATERIALS as usize];

    // Air (id: 0) - empty, no properties
    materials[0] = MaterialProperties {
        density: 0,
        friction: 0,
        flammability: 0,
        conductivity: 0,
        hardness: 0,
        flags: 0,
        reserved: 0,
    };

    // Water (id: 1) - liquid, flows
    materials[1] = MaterialProperties {
        density: 1000,
        friction: 10,
        flammability: 0,
        conductivity: 50,
        hardness: 0,
        flags: CellFlags::LIQUID,
        reserved: 0,
    };

    // Sand (id: 2) - powder, falls
    materials[2] = MaterialProperties {
        density: 1600,
        friction: 80,
        flammability: 0,
        conductivity: 10,
        hardness: 30,
        flags: 0,
        reserved: 0,
    };

    // Grass (id: 3) - solid
    materials[3] = MaterialProperties {
        density: 1200,
        friction: 90,
        flammability: 60,
        conductivity: 5,
        hardness: 20,
        flags: CellFlags::SOLID,
        reserved: 0,
    };

    // Dirt (id: 4) - solid
    materials[4] = MaterialProperties {
        density: 1500,
        friction: 85,
        flammability: 0,
        conductivity: 8,
        hardness: 25,
        flags: CellFlags::SOLID,
        reserved: 0,
    };

    // Stone (id: 5) - solid
    materials[5] = MaterialProperties {
        density: 2600,
        friction: 70,
        flammability: 0,
        conductivity: 30,
        hardness: 90,
        flags: CellFlags::SOLID,
        reserved: 0,
    };

    // Snow (id: 6) - powder
    materials[6] = MaterialProperties {
        density: 300,
        friction: 20,
        flammability: 0,
        conductivity: 5,
        hardness: 5,
        flags: 0,
        reserved: 0,
    };

    // Metal (id: 7) - solid, conductive
    materials[7] = MaterialProperties {
        density: 7800,
        friction: 60,
        flammability: 0,
        conductivity: 255,
        hardness: 100,
        flags: CellFlags::SOLID,
        reserved: 0,
    };

    // Wood (id: 8) - solid, flammable
    materials[8] = MaterialProperties {
        density: 600,
        friction: 75,
        flammability: 80,
        conductivity: 15,
        hardness: 40,
        flags: CellFlags::SOLID,
        reserved: 0,
    };

    // Glass (id: 9) - solid, fragile
    materials[9] = MaterialProperties {
        density: 2500,
        friction: 50,
        flammability: 0,
        conductivity: 20,
        hardness: 60,
        flags: CellFlags::SOLID,
        reserved: 0,
    };

    // Concrete (id: 10) - solid
    materials[10] = MaterialProperties {
        density: 2400,
        friction: 80,
        flammability: 0,
        conductivity: 25,
        hardness: 85,
        flags: CellFlags::SOLID,
        reserved: 0,
    };

    materials
}

/// Creates a GPU buffer with the given cells.
///
/// # Arguments
/// * `device` - The wgpu device
/// * `cells` - Cell data to upload
/// * `label` - Buffer label for debugging
///
/// # Returns
/// A GPU buffer containing the cell data.
pub fn create_cell_buffer(device: &Device, cells: &[Cell], label: &str) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(cells),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    })
}

/// Creates a GPU buffer with the given material properties.
///
/// # Arguments
/// * `device` - The wgpu device
/// * `materials` - Material properties to upload
///
/// # Returns
/// A GPU buffer containing the material properties.
pub fn create_material_buffer(device: &Device, materials: &[MaterialProperties]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Properties Buffer"),
        contents: bytemuck::cast_slice(materials),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

/// Creates a GPU uniform buffer with simulation parameters.
///
/// # Arguments
/// * `device` - The wgpu device
/// * `params` - Simulation parameters
///
/// # Returns
/// A GPU uniform buffer containing the parameters.
pub fn create_params_buffer(device: &Device, params: &SimulationParams) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Simulation Parameters Buffer"),
        contents: bytemuck::cast_slice(&[*params]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_params_default() {
        let params = SimulationParams::default();
        assert_eq!(params.chunk_size, DEFAULT_CHUNK_SIZE);
        assert_eq!(params.frame, 0);
        assert!((params.gravity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_simulation_params_advance() {
        let mut params = SimulationParams::default();
        params.advance_frame();
        assert_eq!(params.frame, 1);
        params.advance_frame();
        assert_eq!(params.frame, 2);
    }

    #[test]
    fn test_default_materials() {
        let materials = create_default_materials();
        assert_eq!(materials.len(), MAX_MATERIALS as usize);

        // Air has zero density
        assert_eq!(materials[0].density, 0);

        // Water is liquid
        assert_eq!(materials[1].flags & CellFlags::LIQUID, CellFlags::LIQUID);

        // Stone is solid
        assert_eq!(materials[5].flags & CellFlags::SOLID, CellFlags::SOLID);
    }

    #[test]
    fn test_simulation_params_size() {
        // Ensure params are properly aligned for GPU uniform buffers
        assert_eq!(std::mem::size_of::<SimulationParams>(), 16);
    }
}
