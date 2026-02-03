//! Intent buffer system for CPU → GPU communication.
//!
//! This module provides the `Intent` type and `IntentBuffer` for uploading
//! player and gameplay intents to the GPU for processing during simulation.
//!
//! Intents represent actions that the CPU wants the GPU to perform on specific
//! cells, such as placing materials, applying forces, or igniting cells.

use bytemuck::{Pod, Zeroable};
use tracing::{info, warn};
use wgpu::{Buffer, BufferUsages, Device, Queue};

/// Maximum number of intents that can be queued per frame.
pub const MAX_INTENTS: usize = 1024;

/// Intent action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IntentAction {
    /// Set the material of a cell (payload: material_id u16, flags u8)
    SetMaterial = 0,
    /// Apply force to a cell (payload: force_x i8, force_y i8)
    ApplyForce = 1,
    /// Ignite a cell (start burning)
    Ignite = 2,
    /// Extinguish a cell (stop burning)
    Extinguish = 3,
    /// Set temperature (payload: temperature u8)
    SetTemperature = 4,
    /// Destroy/clear a cell (set to air)
    Destroy = 5,
    /// Electrify a cell
    Electrify = 6,
}

impl From<u8> for IntentAction {
    fn from(value: u8) -> Self {
        match value {
            1 => IntentAction::ApplyForce,
            2 => IntentAction::Ignite,
            3 => IntentAction::Extinguish,
            4 => IntentAction::SetTemperature,
            5 => IntentAction::Destroy,
            6 => IntentAction::Electrify,
            _ => IntentAction::SetMaterial, // 0 and fallback
        }
    }
}

/// A single intent to be processed by the GPU.
///
/// Intents represent CPU-side requests to modify cell state during simulation.
/// The GPU will process these intents and apply them to the appropriate cells.
///
/// # Layout (16 bytes, GPU-aligned)
/// ```text
/// ┌────────────┬────────────┬────────────┬─────────────────────────┐
/// │ x (u32)    │ y (u32)    │ action(u8) │ payload (7 bytes)       │
/// └────────────┴────────────┴────────────┴─────────────────────────┘
/// ```
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct Intent {
    /// X coordinate of target cell
    pub x: u32,
    /// Y coordinate of target cell
    pub y: u32,
    /// Action type (see IntentAction)
    pub action: u8,
    /// Action-specific payload data
    pub payload: [u8; 7],
}

impl Intent {
    /// Creates a new intent at the given position with the specified action.
    #[must_use]
    pub const fn new(x: u32, y: u32, action: IntentAction) -> Self {
        Self {
            x,
            y,
            action: action as u8,
            payload: [0; 7],
        }
    }

    /// Creates an intent to set a cell's material.
    #[must_use]
    pub fn set_material(x: u32, y: u32, material_id: u16, flags: u8) -> Self {
        let mut intent = Self::new(x, y, IntentAction::SetMaterial);
        let material_bytes = material_id.to_le_bytes();
        intent.payload[0] = material_bytes[0];
        intent.payload[1] = material_bytes[1];
        intent.payload[2] = flags;
        intent
    }

    /// Creates an intent to apply force to a cell.
    #[must_use]
    pub fn apply_force(x: u32, y: u32, force_x: i8, force_y: i8) -> Self {
        let mut intent = Self::new(x, y, IntentAction::ApplyForce);
        intent.payload[0] = force_x as u8;
        intent.payload[1] = force_y as u8;
        intent
    }

    /// Creates an intent to ignite a cell.
    #[must_use]
    pub const fn ignite(x: u32, y: u32) -> Self {
        Self::new(x, y, IntentAction::Ignite)
    }

    /// Creates an intent to extinguish a cell.
    #[must_use]
    pub const fn extinguish(x: u32, y: u32) -> Self {
        Self::new(x, y, IntentAction::Extinguish)
    }

    /// Creates an intent to set a cell's temperature.
    #[must_use]
    pub fn set_temperature(x: u32, y: u32, temperature: u8) -> Self {
        let mut intent = Self::new(x, y, IntentAction::SetTemperature);
        intent.payload[0] = temperature;
        intent
    }

    /// Creates an intent to destroy (clear) a cell.
    #[must_use]
    pub const fn destroy(x: u32, y: u32) -> Self {
        Self::new(x, y, IntentAction::Destroy)
    }

    /// Creates an intent to electrify a cell.
    #[must_use]
    pub const fn electrify(x: u32, y: u32) -> Self {
        Self::new(x, y, IntentAction::Electrify)
    }

    /// Returns the action type.
    #[must_use]
    pub fn action_type(&self) -> IntentAction {
        IntentAction::from(self.action)
    }

    /// Extracts material_id from SetMaterial payload.
    #[must_use]
    pub fn material_id(&self) -> u16 {
        u16::from_le_bytes([self.payload[0], self.payload[1]])
    }

    /// Extracts flags from SetMaterial payload.
    #[must_use]
    pub fn material_flags(&self) -> u8 {
        self.payload[2]
    }
}

/// Intent buffer header for GPU communication.
///
/// This header is stored at the start of the GPU buffer and tells the
/// shader how many intents to process.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct IntentBufferHeader {
    /// Number of intents in the buffer
    pub count: u32,
    /// Maximum capacity (for validation)
    pub capacity: u32,
    /// Reserved for future use
    _reserved: [u32; 2],
}

impl Default for IntentBufferHeader {
    fn default() -> Self {
        Self {
            count: 0,
            capacity: MAX_INTENTS as u32,
            _reserved: [0; 2],
        }
    }
}

/// CPU-side intent queue that uploads to GPU.
///
/// The `IntentBuffer` collects intents from gameplay systems and uploads
/// them to a GPU storage buffer before each simulation step. After the
/// dispatch completes, the buffer is cleared for the next frame.
///
/// # Example
///
/// ```ignore
/// let mut intent_buffer = IntentBuffer::new(&device);
///
/// // Queue intents during gameplay
/// intent_buffer.push(Intent::set_material(10, 20, 5, 0)); // Place stone
/// intent_buffer.push(Intent::ignite(15, 25));            // Start fire
///
/// // Before simulation step
/// intent_buffer.upload(&queue);
///
/// // After simulation step
/// intent_buffer.clear();
/// ```
pub struct IntentBuffer {
    /// GPU storage buffer for intents
    buffer: Buffer,
    /// CPU-side intent queue
    queue: Vec<Intent>,
    /// Header for GPU communication
    header: IntentBufferHeader,
}

impl IntentBuffer {
    /// Creates a new intent buffer.
    ///
    /// # Arguments
    /// * `device` - The wgpu device to create the buffer on
    pub fn new(device: &Device) -> Self {
        info!("Creating intent buffer (capacity: {})", MAX_INTENTS);

        // Calculate buffer size: header + intents
        let header_size = std::mem::size_of::<IntentBufferHeader>();
        let intents_size = MAX_INTENTS * std::mem::size_of::<Intent>();
        let total_size = header_size + intents_size;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Intent Buffer"),
            size: total_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            queue: Vec::with_capacity(MAX_INTENTS),
            header: IntentBufferHeader::default(),
        }
    }

    /// Pushes an intent to the queue.
    ///
    /// Returns `true` if the intent was added, `false` if the queue is full.
    pub fn push(&mut self, intent: Intent) -> bool {
        if self.queue.len() >= MAX_INTENTS {
            warn!(
                "Intent buffer full ({} intents), dropping intent at ({}, {})",
                MAX_INTENTS, intent.x, intent.y
            );
            return false;
        }
        self.queue.push(intent);
        true
    }

    /// Pushes multiple intents to the queue.
    ///
    /// Returns the number of intents successfully added.
    pub fn push_many(&mut self, intents: &[Intent]) -> usize {
        let available = MAX_INTENTS.saturating_sub(self.queue.len());
        let to_add = intents.len().min(available);

        if to_add < intents.len() {
            warn!(
                "Intent buffer overflow: {} intents dropped",
                intents.len() - to_add
            );
        }

        self.queue.extend_from_slice(&intents[..to_add]);
        to_add
    }

    /// Uploads queued intents to the GPU buffer.
    ///
    /// This should be called before dispatching the compute shader.
    pub fn upload(&mut self, queue: &Queue) {
        // Update header with current count
        self.header.count = self.queue.len() as u32;

        // Upload header
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.header));

        // Upload intents (if any)
        if !self.queue.is_empty() {
            let header_size = std::mem::size_of::<IntentBufferHeader>() as u64;
            queue.write_buffer(&self.buffer, header_size, bytemuck::cast_slice(&self.queue));
        }
    }

    /// Clears the intent queue.
    ///
    /// This should be called after the compute dispatch completes.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.header.count = 0;
    }

    /// Returns the number of queued intents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns whether the intent queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns whether the intent queue is full.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.queue.len() >= MAX_INTENTS
    }

    /// Returns the GPU buffer for binding.
    #[must_use]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Returns the current intents in the queue (for debugging).
    #[must_use]
    pub fn intents(&self) -> &[Intent] {
        &self.queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_size() {
        // Ensure intent is properly aligned for GPU
        assert_eq!(std::mem::size_of::<Intent>(), 16);
    }

    #[test]
    fn test_intent_header_size() {
        // Ensure header is properly aligned
        assert_eq!(std::mem::size_of::<IntentBufferHeader>(), 16);
    }

    #[test]
    fn test_intent_set_material() {
        let intent = Intent::set_material(100, 200, 5, 0b0000_0001);
        assert_eq!(intent.x, 100);
        assert_eq!(intent.y, 200);
        assert_eq!(intent.action_type(), IntentAction::SetMaterial);
        assert_eq!(intent.material_id(), 5);
        assert_eq!(intent.material_flags(), 0b0000_0001);
    }

    #[test]
    fn test_intent_apply_force() {
        let intent = Intent::apply_force(50, 75, -10, 20);
        assert_eq!(intent.x, 50);
        assert_eq!(intent.y, 75);
        assert_eq!(intent.action_type(), IntentAction::ApplyForce);
        #[allow(clippy::cast_possible_wrap)]
        {
            assert_eq!(intent.payload[0] as i8, -10);
            assert_eq!(intent.payload[1] as i8, 20);
        }
    }

    #[test]
    fn test_intent_ignite() {
        let intent = Intent::ignite(30, 40);
        assert_eq!(intent.action_type(), IntentAction::Ignite);
    }

    #[test]
    fn test_intent_action_from_u8() {
        assert_eq!(IntentAction::from(0), IntentAction::SetMaterial);
        assert_eq!(IntentAction::from(1), IntentAction::ApplyForce);
        assert_eq!(IntentAction::from(2), IntentAction::Ignite);
        assert_eq!(IntentAction::from(255), IntentAction::SetMaterial); // Fallback
    }
}
