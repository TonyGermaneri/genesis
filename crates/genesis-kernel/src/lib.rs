//! # Genesis Kernel
//!
//! GPU compute kernel for pixel-cell simulation.
//!
//! This crate provides the core GPU-accelerated simulation:
//! - Pixel-cell simulation (each pixel is a simulated cell)
//! - Buffer layouts for GPU compute
//! - Double-buffered cell storage for efficient GPU simulation
//! - Intent buffer for CPU → GPU communication
//! - Event buffer for GPU → CPU communication
//! - GPU validation harness
//! - Performance benchmarking utilities
//!
//! ## Architecture
//!
//! The kernel runs on the GPU and is the physical authority for the world.
//! The CPU entity layer (gameplay crate) submits intents which the kernel
//! executes as part of the simulation step.
//!
//! ## Double Buffering
//!
//! The simulation uses double-buffering to ensure correct GPU execution:
//! - Two cell buffers alternate between input (read) and output (write)
//! - Each simulation step reads from one buffer and writes to the other
//! - Buffers are swapped after each step
//!
//! ## Intent/Event System
//!
//! Communication between CPU and GPU uses bounded queues:
//! - **Intents** (CPU → GPU): Actions to apply to cells (place material, ignite, etc.)
//! - **Events** (GPU → CPU): Notifications about cell state changes
//!
//! ## Validation
//!
//! In debug builds, wgpu validation is enabled to catch GPU errors early.
//! Use `create_validated_instance()` to create a wgpu instance with appropriate settings.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod benchmark;
pub mod buffer;
pub mod cell;
pub mod chunk;
pub mod compute;
pub mod event;
pub mod intent;
pub mod render;
pub mod validation;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::benchmark::*;
    pub use crate::buffer::*;
    pub use crate::cell::*;
    pub use crate::chunk::*;
    pub use crate::compute::*;
    pub use crate::event::*;
    pub use crate::intent::*;
    pub use crate::render::*;
    pub use crate::validation::*;
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_size() {
        // Ensure cell fits in expected GPU buffer alignment
        assert_eq!(std::mem::size_of::<Cell>(), 8);
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.material, 0);
        assert_eq!(cell.flags, 0);
    }
}
