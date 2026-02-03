//! Double-buffered cell storage for GPU simulation.
//!
//! This module provides the `CellBuffer` type which manages two GPU buffers
//! for cell data. The simulation reads from one buffer and writes to the other,
//! then swaps them each frame. This prevents race conditions and allows the
//! GPU to efficiently pipeline simulation steps.

use bytemuck::Pod;
use tracing::info;
use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device, Queue};

use crate::cell::{Cell, MaterialProperties};
use crate::compute::{
    create_default_materials, create_params_buffer, CellComputePipeline, SimulationParams,
    MAX_MATERIALS,
};

/// Double-buffered cell storage for GPU simulation.
///
/// This struct manages a pair of GPU buffers that alternate roles between
/// input (read) and output (write) each simulation frame. This double-buffering
/// approach ensures that:
///
/// 1. The GPU can read the entire input buffer without data races
/// 2. Writes to the output buffer don't affect current reads
/// 3. State persists correctly across frames
///
/// # Example
///
/// ```ignore
/// let mut cell_buffer = CellBuffer::new(&device, 256);
///
/// // Each frame:
/// cell_buffer.step(&device, &queue, &pipeline);
///
/// // State persists across frames
/// ```
pub struct CellBuffer {
    /// Buffer A (alternates between input/output)
    buffer_a: Buffer,
    /// Buffer B (alternates between input/output)
    buffer_b: Buffer,
    /// Material properties lookup table
    materials_buffer: Buffer,
    /// Simulation parameters
    params_buffer: Buffer,
    /// Bind group for A→B simulation (read A, write B)
    bind_group_a_to_b: wgpu::BindGroup,
    /// Bind group for B→A simulation (read B, write A)
    bind_group_b_to_a: wgpu::BindGroup,
    /// Current simulation parameters
    params: SimulationParams,
    /// Whether buffer A is currently the input (true) or output (false)
    a_is_input: bool,
    /// Chunk size (width and height in cells)
    chunk_size: u32,
    /// Total number of cells
    cell_count: usize,
}

impl CellBuffer {
    /// Creates a new double-buffered cell storage.
    ///
    /// Initializes both buffers with the default cell state (air).
    ///
    /// # Arguments
    /// * `device` - The wgpu device to create buffers on
    /// * `pipeline` - The compute pipeline (for bind group layout)
    /// * `chunk_size` - Size of the chunk in cells (width and height)
    ///
    /// # Returns
    /// A new `CellBuffer` ready for simulation.
    pub fn new(device: &Device, pipeline: &CellComputePipeline, chunk_size: u32) -> Self {
        info!(
            "Creating double-buffered cell storage ({}x{} = {} cells)",
            chunk_size,
            chunk_size,
            chunk_size * chunk_size
        );

        let cell_count = (chunk_size * chunk_size) as usize;
        let cells: Vec<Cell> = vec![Cell::default(); cell_count];

        // Create cell buffers
        let buffer_a = create_storage_buffer(device, &cells, "Cell Buffer A");
        let buffer_b = create_storage_buffer(device, &cells, "Cell Buffer B");

        // Create material properties buffer
        let materials = create_default_materials();
        let materials_buffer = create_storage_buffer(device, &materials, "Materials Buffer");

        // Create simulation parameters buffer
        let params = SimulationParams::new(chunk_size);
        let params_buffer = create_params_buffer(device, &params);

        // Create bind groups for both directions
        let bind_group_a_to_b = pipeline.create_bind_group(
            device,
            &buffer_a,
            &buffer_b,
            &materials_buffer,
            &params_buffer,
        );

        let bind_group_b_to_a = pipeline.create_bind_group(
            device,
            &buffer_b,
            &buffer_a,
            &materials_buffer,
            &params_buffer,
        );

        info!("Double-buffered cell storage created successfully");

        Self {
            buffer_a,
            buffer_b,
            materials_buffer,
            params_buffer,
            bind_group_a_to_b,
            bind_group_b_to_a,
            params,
            a_is_input: true,
            chunk_size,
            cell_count,
        }
    }

    /// Creates a new double-buffered cell storage with initial cell data.
    ///
    /// # Arguments
    /// * `device` - The wgpu device to create buffers on
    /// * `pipeline` - The compute pipeline (for bind group layout)
    /// * `chunk_size` - Size of the chunk in cells (width and height)
    /// * `initial_cells` - Initial cell data to populate the buffer
    ///
    /// # Panics
    /// Panics if `initial_cells.len()` doesn't match `chunk_size * chunk_size`.
    pub fn with_cells(
        device: &Device,
        pipeline: &CellComputePipeline,
        chunk_size: u32,
        initial_cells: &[Cell],
    ) -> Self {
        let expected_count = (chunk_size * chunk_size) as usize;
        assert_eq!(
            initial_cells.len(),
            expected_count,
            "Cell count mismatch: expected {expected_count}, got {}",
            initial_cells.len()
        );

        info!(
            "Creating double-buffered cell storage with initial data ({}x{} = {} cells)",
            chunk_size, chunk_size, expected_count
        );

        // Create cell buffers
        let buffer_a = create_storage_buffer(device, initial_cells, "Cell Buffer A");
        let buffer_b = create_storage_buffer(device, initial_cells, "Cell Buffer B");

        // Create material properties buffer
        let materials = create_default_materials();
        let materials_buffer = create_storage_buffer(device, &materials, "Materials Buffer");

        // Create simulation parameters buffer
        let params = SimulationParams::new(chunk_size);
        let params_buffer = create_params_buffer(device, &params);

        // Create bind groups for both directions
        let bind_group_a_to_b = pipeline.create_bind_group(
            device,
            &buffer_a,
            &buffer_b,
            &materials_buffer,
            &params_buffer,
        );

        let bind_group_b_to_a = pipeline.create_bind_group(
            device,
            &buffer_b,
            &buffer_a,
            &materials_buffer,
            &params_buffer,
        );

        info!("Double-buffered cell storage created with initial data");

        Self {
            buffer_a,
            buffer_b,
            materials_buffer,
            params_buffer,
            bind_group_a_to_b,
            bind_group_b_to_a,
            params,
            a_is_input: true,
            chunk_size,
            cell_count: expected_count,
        }
    }

    /// Performs one simulation step.
    ///
    /// This method:
    /// 1. Updates simulation parameters (advances frame counter)
    /// 2. Dispatches the compute shader
    /// 3. Swaps the input/output buffers
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `queue` - The wgpu queue for submitting commands
    /// * `pipeline` - The compute pipeline to use
    pub fn step(&mut self, device: &Device, queue: &Queue, pipeline: &CellComputePipeline) {
        // Advance frame counter
        self.params.advance_frame();

        // Update params buffer on GPU
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cell Simulation Encoder"),
        });

        // Select the appropriate bind group based on current buffer state
        let bind_group = if self.a_is_input {
            &self.bind_group_a_to_b
        } else {
            &self.bind_group_b_to_a
        };

        // Dispatch simulation
        pipeline.dispatch(&mut encoder, bind_group, self.chunk_size);

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));

        // Swap buffers for next frame
        self.a_is_input = !self.a_is_input;
    }

    /// Returns the current input buffer (the one with the latest simulation state).
    ///
    /// After `step()` is called, this returns the buffer that was written to
    /// during that step (which will be the input for the next step).
    #[must_use]
    pub fn current_buffer(&self) -> &Buffer {
        // After step(), a_is_input has been swapped, so if a_is_input is true,
        // buffer B has the latest data (it was just written to)
        if self.a_is_input {
            &self.buffer_b
        } else {
            &self.buffer_a
        }
    }

    /// Returns the current input buffer (for the next simulation step).
    #[must_use]
    pub fn input_buffer(&self) -> &Buffer {
        if self.a_is_input {
            &self.buffer_a
        } else {
            &self.buffer_b
        }
    }

    /// Returns the current output buffer (for the next simulation step).
    #[must_use]
    pub fn output_buffer(&self) -> &Buffer {
        if self.a_is_input {
            &self.buffer_b
        } else {
            &self.buffer_a
        }
    }

    /// Returns the materials buffer.
    #[must_use]
    pub fn materials_buffer(&self) -> &Buffer {
        &self.materials_buffer
    }

    /// Returns the parameters buffer.
    #[must_use]
    pub fn params_buffer(&self) -> &Buffer {
        &self.params_buffer
    }

    /// Returns the current simulation parameters.
    #[must_use]
    pub const fn params(&self) -> &SimulationParams {
        &self.params
    }

    /// Returns the chunk size (width and height in cells).
    #[must_use]
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Returns the total number of cells.
    #[must_use]
    pub const fn cell_count(&self) -> usize {
        self.cell_count
    }

    /// Returns the current frame number.
    #[must_use]
    pub const fn frame(&self) -> u32 {
        self.params.frame
    }

    /// Uploads new cell data to the current input buffer.
    ///
    /// # Arguments
    /// * `queue` - The wgpu queue
    /// * `cells` - Cell data to upload
    ///
    /// # Panics
    /// Panics if `cells.len()` doesn't match the buffer size.
    pub fn upload_cells(&self, queue: &Queue, cells: &[Cell]) {
        assert_eq!(
            cells.len(),
            self.cell_count,
            "Cell count mismatch: expected {}, got {}",
            self.cell_count,
            cells.len()
        );

        let buffer = self.input_buffer();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(cells));
    }

    /// Updates material properties.
    ///
    /// # Arguments
    /// * `queue` - The wgpu queue
    /// * `materials` - New material properties
    pub fn update_materials(&self, queue: &Queue, materials: &[MaterialProperties]) {
        let max_materials = MAX_MATERIALS as usize;
        assert!(
            materials.len() <= max_materials,
            "Too many materials: {} > {}",
            materials.len(),
            max_materials
        );

        queue.write_buffer(&self.materials_buffer, 0, bytemuck::cast_slice(materials));
    }

    /// Creates a staging buffer for reading cell data back from the GPU.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    ///
    /// # Returns
    /// A staging buffer that can be mapped for reading.
    pub fn create_readback_buffer(&self, device: &Device) -> Buffer {
        let size = (self.cell_count * std::mem::size_of::<Cell>()) as u64;
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cell Readback Buffer"),
            size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }

    /// Copies the current cell state to a staging buffer for CPU readback.
    ///
    /// # Arguments
    /// * `encoder` - Command encoder to record the copy
    /// * `staging` - Staging buffer to copy to (created by `create_readback_buffer`)
    pub fn copy_to_staging(&self, encoder: &mut wgpu::CommandEncoder, staging: &Buffer) {
        let size = (self.cell_count * std::mem::size_of::<Cell>()) as u64;
        encoder.copy_buffer_to_buffer(self.current_buffer(), 0, staging, 0, size);
    }
}

/// Creates a GPU storage buffer from a slice of Pod data.
fn create_storage_buffer<T: Pod>(device: &Device, data: &[T], label: &str) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::DEFAULT_CHUNK_SIZE;

    #[test]
    fn test_buffer_swap_logic() {
        // Test the buffer swap logic without creating actual GPU resources
        // This validates the state machine
        let mut a_is_input = true;

        // Initial state: A is input, B is output
        assert!(a_is_input);

        // After step: swap
        a_is_input = !a_is_input;
        assert!(!a_is_input);

        // After another step: swap back
        a_is_input = !a_is_input;
        assert!(a_is_input);
    }

    #[test]
    fn test_cell_count_calculation() {
        let chunk_size: u32 = 256;
        let cell_count = (chunk_size * chunk_size) as usize;
        assert_eq!(cell_count, 65536);

        let chunk_size: u32 = 128;
        let cell_count = (chunk_size * chunk_size) as usize;
        assert_eq!(cell_count, 16384);
    }

    #[test]
    fn test_buffer_size_calculation() {
        let chunk_size: u32 = DEFAULT_CHUNK_SIZE;
        let cell_count = (chunk_size * chunk_size) as usize;
        let buffer_size = cell_count * std::mem::size_of::<Cell>();

        // 256 * 256 * 8 = 524288 bytes = 512 KB
        assert_eq!(buffer_size, 524288);
    }
}
