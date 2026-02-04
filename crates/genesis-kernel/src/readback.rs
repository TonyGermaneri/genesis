//! GPU readback optimization for async GPU→CPU data transfer.
//!
//! This module provides efficient mechanisms for reading data back from the GPU
//! to the CPU. It uses double-buffered staging buffers to enable async transfers
//! without blocking the main simulation loop.

use std::collections::VecDeque;

use tracing::{debug, warn};
use wgpu::{Buffer, BufferUsages, Device, MapMode};

use crate::cell::Cell;
use crate::chunk::ChunkId;
use crate::compute::DEFAULT_CHUNK_SIZE;

/// Maximum number of frames a readback can be pending before timeout.
pub const READBACK_TIMEOUT_FRAMES: u64 = 10;

/// Maximum pending readbacks in the queue.
pub const MAX_PENDING_READBACKS: usize = 16;

/// Status of a pending readback operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadbackStatus {
    /// The readback has been requested but not yet submitted.
    Pending,
    /// The readback has been submitted to the GPU.
    Submitted,
    /// The readback is ready to be mapped and read.
    Ready,
    /// The readback timed out.
    TimedOut,
    /// The readback completed successfully.
    Complete,
}

/// Information about a pending readback operation.
#[derive(Debug, Clone)]
pub struct PendingRead {
    /// The chunk being read back.
    pub chunk_id: ChunkId,
    /// Frame number when the readback was submitted.
    pub frame_submitted: u64,
    /// Current status.
    pub status: ReadbackStatus,
    /// Index into the staging buffer ring.
    staging_index: usize,
}

impl PendingRead {
    /// Creates a new pending read.
    fn new(chunk_id: ChunkId, frame: u64, staging_index: usize) -> Self {
        Self {
            chunk_id,
            frame_submitted: frame,
            status: ReadbackStatus::Pending,
            staging_index,
        }
    }

    /// Checks if this read has timed out given the current frame.
    #[must_use]
    pub fn is_timed_out(&self, current_frame: u64) -> bool {
        current_frame.saturating_sub(self.frame_submitted) > READBACK_TIMEOUT_FRAMES
    }
}

/// A staging buffer for GPU→CPU transfers.
#[derive(Debug)]
struct StagingBuffer {
    /// The wgpu buffer.
    buffer: Buffer,
    /// Whether this buffer is currently in use.
    in_use: bool,
    /// The chunk size this buffer can hold.
    chunk_size: u32,
}

impl StagingBuffer {
    /// Creates a new staging buffer.
    fn new(device: &Device, chunk_size: u32, index: usize) -> Self {
        let cell_count = (chunk_size * chunk_size) as usize;
        let size = cell_count * std::mem::size_of::<Cell>();

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Staging Buffer {index}")),
            size: size as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            in_use: false,
            chunk_size,
        }
    }

    /// Returns the expected byte size of this buffer.
    fn byte_size(&self) -> u64 {
        let cell_count = (self.chunk_size * self.chunk_size) as u64;
        cell_count * std::mem::size_of::<Cell>() as u64
    }
}

/// Manager for GPU readback operations.
///
/// Provides async GPU→CPU data transfer with double-buffered staging.
/// Tracks pending readbacks and handles timeout/completion.
///
/// # Example
///
/// ```ignore
/// let mut readback = ReadbackManager::new(&device, 256);
/// readback.request_readback(ChunkId::new(0, 0));
///
/// // Later, poll for completions
/// let completed = readback.poll_readbacks(&device, &queue, current_frame);
/// for (chunk_id, cells) in completed {
///     process_cells(chunk_id, cells);
/// }
/// ```
#[derive(Debug)]
pub struct ReadbackManager {
    /// Double-buffered staging buffers.
    staging_buffers: Vec<StagingBuffer>,
    /// Pending readback operations.
    pending_reads: VecDeque<PendingRead>,
    /// Current staging buffer index (for round-robin).
    current_staging: usize,
    /// Size of chunks in cells.
    #[allow(dead_code)]
    chunk_size: u32,
}

impl ReadbackManager {
    /// Creates a new readback manager with the specified chunk size.
    ///
    /// Creates two staging buffers for double-buffering.
    pub fn new(device: &Device, chunk_size: u32) -> Self {
        let staging_buffers = vec![
            StagingBuffer::new(device, chunk_size, 0),
            StagingBuffer::new(device, chunk_size, 1),
        ];

        Self {
            staging_buffers,
            pending_reads: VecDeque::new(),
            current_staging: 0,
            chunk_size,
        }
    }

    /// Creates a readback manager with default chunk size.
    pub fn with_default_size(device: &Device) -> Self {
        Self::new(device, DEFAULT_CHUNK_SIZE)
    }

    /// Requests a readback for the specified chunk.
    ///
    /// Returns `true` if the request was queued, `false` if the queue is full.
    #[must_use]
    pub fn request_readback(&mut self, chunk_id: ChunkId, current_frame: u64) -> bool {
        // Check if we're at capacity
        if self.pending_reads.len() >= MAX_PENDING_READBACKS {
            warn!("Readback queue full, dropping request for {chunk_id:?}");
            return false;
        }

        // Check if this chunk already has a pending readback
        if self.is_pending(chunk_id) {
            debug!("Chunk {chunk_id:?} already has pending readback");
            return false;
        }

        // Find an available staging buffer
        let staging_index = self.find_available_staging();
        if staging_index.is_none() {
            debug!("No staging buffers available");
            return false;
        }

        let staging_index = staging_index.expect("just checked");
        self.staging_buffers[staging_index].in_use = true;

        let pending = PendingRead::new(chunk_id, current_frame, staging_index);
        self.pending_reads.push_back(pending);

        debug!("Queued readback for {chunk_id:?} in staging {staging_index}");
        true
    }

    /// Finds an available staging buffer index.
    fn find_available_staging(&self) -> Option<usize> {
        // Try round-robin starting from current
        for i in 0..self.staging_buffers.len() {
            let idx = (self.current_staging + i) % self.staging_buffers.len();
            if !self.staging_buffers[idx].in_use {
                return Some(idx);
            }
        }
        None
    }

    /// Checks if a chunk has a pending readback.
    #[must_use]
    pub fn is_pending(&self, chunk_id: ChunkId) -> bool {
        self.pending_reads.iter().any(|r| r.chunk_id == chunk_id)
    }

    /// Returns the number of pending readbacks.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending_reads.len()
    }

    /// Gets information about pending reads.
    #[must_use]
    pub fn pending_reads(&self) -> &VecDeque<PendingRead> {
        &self.pending_reads
    }

    /// Gets the staging buffer for a pending read to submit the copy command.
    ///
    /// This should be called to get the destination buffer for a GPU copy.
    #[must_use]
    pub fn get_staging_buffer(&self, chunk_id: ChunkId) -> Option<&Buffer> {
        self.pending_reads
            .iter()
            .find(|r| r.chunk_id == chunk_id)
            .map(|r| &self.staging_buffers[r.staging_index].buffer)
    }

    /// Marks a readback as submitted (copy command encoded).
    pub fn mark_submitted(&mut self, chunk_id: ChunkId) {
        if let Some(pending) = self
            .pending_reads
            .iter_mut()
            .find(|r| r.chunk_id == chunk_id)
        {
            pending.status = ReadbackStatus::Submitted;
            debug!("Marked {chunk_id:?} as submitted");
        }
    }

    /// Submits the GPU copy command for a pending readback.
    ///
    /// # Arguments
    /// * `chunk_id` - The chunk to read back
    /// * `source_buffer` - The GPU buffer containing the cell data
    /// * `encoder` - Command encoder to record the copy
    ///
    /// Returns `true` if the copy was submitted.
    pub fn submit_copy(
        &mut self,
        chunk_id: ChunkId,
        source_buffer: &Buffer,
        encoder: &mut wgpu::CommandEncoder,
    ) -> bool {
        if let Some(pending) = self
            .pending_reads
            .iter_mut()
            .find(|r| r.chunk_id == chunk_id)
        {
            let staging = &self.staging_buffers[pending.staging_index];
            let size = staging.byte_size();

            encoder.copy_buffer_to_buffer(source_buffer, 0, &staging.buffer, 0, size);

            pending.status = ReadbackStatus::Submitted;
            debug!("Submitted copy for {chunk_id:?}");
            true
        } else {
            false
        }
    }

    /// Polls for completed readbacks.
    ///
    /// This checks all submitted readbacks and attempts to map them.
    /// Returns completed readbacks with their cell data.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `current_frame` - Current frame number for timeout detection
    #[must_use]
    pub fn poll_readbacks(
        &mut self,
        device: &Device,
        current_frame: u64,
    ) -> Vec<(ChunkId, Vec<Cell>)> {
        let mut completed = Vec::new();
        let mut to_remove = Vec::new();

        for (index, pending) in self.pending_reads.iter_mut().enumerate() {
            // Check for timeout
            if pending.is_timed_out(current_frame) {
                warn!("Readback for {:?} timed out", pending.chunk_id);
                pending.status = ReadbackStatus::TimedOut;
                to_remove.push(index);
                continue;
            }

            // Only process submitted readbacks
            if pending.status != ReadbackStatus::Submitted {
                continue;
            }

            // Try to map the buffer
            let staging = &self.staging_buffers[pending.staging_index];
            let slice = staging.buffer.slice(..);

            // Start async mapping
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(MapMode::Read, move |result| {
                let _ = tx.send(result);
            });

            // Poll the device to progress the mapping
            device.poll(wgpu::Maintain::Wait);

            // Check if mapping completed
            if let Ok(Ok(())) = rx.try_recv() {
                let data = slice.get_mapped_range();
                let cells: Vec<Cell> = bytemuck::cast_slice(&data).to_vec();
                drop(data);

                staging.buffer.unmap();
                pending.status = ReadbackStatus::Complete;

                debug!("Readback complete for {:?}", pending.chunk_id);
                completed.push((pending.chunk_id, cells));
                to_remove.push(index);
            }
        }

        // Remove completed/timed out entries (in reverse order to preserve indices)
        for index in to_remove.into_iter().rev() {
            if let Some(removed) = self.pending_reads.remove(index) {
                self.staging_buffers[removed.staging_index].in_use = false;
                // Advance round-robin
                self.current_staging = (removed.staging_index + 1) % self.staging_buffers.len();
            }
        }

        completed
    }

    /// Cancels a pending readback.
    pub fn cancel(&mut self, chunk_id: ChunkId) -> bool {
        if let Some(pos) = self
            .pending_reads
            .iter()
            .position(|r| r.chunk_id == chunk_id)
        {
            let removed = self.pending_reads.remove(pos);
            if let Some(removed) = removed {
                self.staging_buffers[removed.staging_index].in_use = false;
                debug!("Cancelled readback for {chunk_id:?}");
                return true;
            }
        }
        false
    }

    /// Clears all pending readbacks.
    pub fn clear(&mut self) {
        for staging in &mut self.staging_buffers {
            staging.in_use = false;
        }
        self.pending_reads.clear();
        debug!("Cleared all pending readbacks");
    }

    /// Returns statistics about the readback manager.
    #[must_use]
    pub fn stats(&self) -> ReadbackStats {
        let submitted = self
            .pending_reads
            .iter()
            .filter(|r| r.status == ReadbackStatus::Submitted)
            .count();
        let pending = self
            .pending_reads
            .iter()
            .filter(|r| r.status == ReadbackStatus::Pending)
            .count();
        let buffers_in_use = self.staging_buffers.iter().filter(|b| b.in_use).count();

        ReadbackStats {
            pending_count: self.pending_reads.len(),
            submitted_count: submitted,
            waiting_count: pending,
            buffers_in_use,
            buffer_count: self.staging_buffers.len(),
        }
    }
}

/// Statistics about readback operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadbackStats {
    /// Total pending readbacks.
    pub pending_count: usize,
    /// Readbacks that have been submitted to GPU.
    pub submitted_count: usize,
    /// Readbacks waiting to be submitted.
    pub waiting_count: usize,
    /// Staging buffers currently in use.
    pub buffers_in_use: usize,
    /// Total staging buffer count.
    pub buffer_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::create_validated_instance;

    fn create_test_device() -> Option<Device> {
        // Try to create a wgpu device for testing
        let instance = create_validated_instance();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: true,
        }))?;

        let (device, _queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .ok()?;

        Some(device)
    }

    #[test]
    fn test_pending_read_timeout() {
        let pending = PendingRead::new(ChunkId::new(0, 0), 100, 0);

        assert!(!pending.is_timed_out(100));
        assert!(!pending.is_timed_out(105));
        assert!(!pending.is_timed_out(110));
        assert!(pending.is_timed_out(111));
        assert!(pending.is_timed_out(200));
    }

    #[test]
    fn test_readback_status() {
        assert_eq!(ReadbackStatus::Pending, ReadbackStatus::Pending);
        assert_ne!(ReadbackStatus::Pending, ReadbackStatus::Submitted);
    }

    #[test]
    fn test_readback_stats_default() {
        let stats = ReadbackStats::default();
        assert_eq!(stats.pending_count, 0);
        assert_eq!(stats.submitted_count, 0);
        assert_eq!(stats.buffers_in_use, 0);
    }

    #[test]
    fn test_readback_manager_creation() {
        if let Some(device) = create_test_device() {
            let manager = ReadbackManager::new(&device, 64);
            assert_eq!(manager.staging_buffers.len(), 2);
            assert_eq!(manager.pending_count(), 0);
        }
    }

    #[test]
    fn test_request_readback() {
        if let Some(device) = create_test_device() {
            let mut manager = ReadbackManager::new(&device, 64);

            // First request should succeed
            let chunk = ChunkId::new(1, 2);
            assert!(manager.request_readback(chunk, 0));
            assert_eq!(manager.pending_count(), 1);
            assert!(manager.is_pending(chunk));

            // Duplicate request should fail
            assert!(!manager.request_readback(chunk, 0));
            assert_eq!(manager.pending_count(), 1);
        }
    }

    #[test]
    fn test_request_multiple_readbacks() {
        if let Some(device) = create_test_device() {
            let mut manager = ReadbackManager::new(&device, 64);

            // Can request up to buffer count
            let chunk1 = ChunkId::new(0, 0);
            let chunk2 = ChunkId::new(1, 0);

            assert!(manager.request_readback(chunk1, 0));
            assert!(manager.request_readback(chunk2, 0));
            assert_eq!(manager.pending_count(), 2);

            // Third should fail (only 2 staging buffers)
            let chunk3 = ChunkId::new(2, 0);
            assert!(!manager.request_readback(chunk3, 0));
        }
    }

    #[test]
    fn test_cancel_readback() {
        if let Some(device) = create_test_device() {
            let mut manager = ReadbackManager::new(&device, 64);

            let chunk = ChunkId::new(0, 0);
            assert!(manager.request_readback(chunk, 0));
            assert!(manager.is_pending(chunk));

            assert!(manager.cancel(chunk));
            assert!(!manager.is_pending(chunk));
            assert_eq!(manager.pending_count(), 0);

            // Cancel non-existent should return false
            assert!(!manager.cancel(chunk));
        }
    }

    #[test]
    fn test_clear_readbacks() {
        if let Some(device) = create_test_device() {
            let mut manager = ReadbackManager::new(&device, 64);

            assert!(manager.request_readback(ChunkId::new(0, 0), 0));
            assert!(manager.request_readback(ChunkId::new(1, 0), 0));
            assert_eq!(manager.pending_count(), 2);

            manager.clear();
            assert_eq!(manager.pending_count(), 0);

            // Stats should reflect cleared state
            let stats = manager.stats();
            assert_eq!(stats.buffers_in_use, 0);
        }
    }

    #[test]
    fn test_stats() {
        if let Some(device) = create_test_device() {
            let mut manager = ReadbackManager::new(&device, 64);

            let stats = manager.stats();
            assert_eq!(stats.buffer_count, 2);
            assert_eq!(stats.pending_count, 0);
            assert_eq!(stats.buffers_in_use, 0);

            assert!(manager.request_readback(ChunkId::new(0, 0), 0));
            let stats = manager.stats();
            assert_eq!(stats.pending_count, 1);
            assert_eq!(stats.buffers_in_use, 1);
            assert_eq!(stats.waiting_count, 1);
        }
    }

    #[test]
    fn test_mark_submitted() {
        if let Some(device) = create_test_device() {
            let mut manager = ReadbackManager::new(&device, 64);

            let chunk = ChunkId::new(0, 0);
            assert!(manager.request_readback(chunk, 0));

            let stats = manager.stats();
            assert_eq!(stats.submitted_count, 0);
            assert_eq!(stats.waiting_count, 1);

            manager.mark_submitted(chunk);

            let stats = manager.stats();
            assert_eq!(stats.submitted_count, 1);
            assert_eq!(stats.waiting_count, 0);
        }
    }

    #[test]
    fn test_get_staging_buffer() {
        if let Some(device) = create_test_device() {
            let mut manager = ReadbackManager::new(&device, 64);

            let chunk = ChunkId::new(0, 0);
            assert!(manager.get_staging_buffer(chunk).is_none());

            assert!(manager.request_readback(chunk, 0));
            assert!(manager.get_staging_buffer(chunk).is_some());
        }
    }
}
