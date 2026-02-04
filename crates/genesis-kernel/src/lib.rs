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

pub mod audio;
pub mod audio_backend;
pub mod audio_legacy;
pub mod audio_resource;
pub mod audio_spatial;
pub mod benchmark;
pub mod biome;
pub mod buffer;
pub mod camera;
pub mod cell;
pub mod chunk;
pub mod chunk_manager;
pub mod collision;
pub mod compute;
pub mod edge;
pub mod event;
pub mod intent;
pub mod lighting;
pub mod npc_collision;
pub mod npc_render;
pub mod particles;
pub mod quadtree;
pub mod readback;
pub mod render;
pub mod streaming;
pub mod terrain_render;
pub mod topdown_physics;
pub mod validation;
pub mod world_init;
pub mod worldgen;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::audio::*;
    pub use crate::benchmark::*;
    pub use crate::biome::*;
    pub use crate::buffer::*;
    pub use crate::camera::*;
    pub use crate::cell::*;
    pub use crate::chunk::*;
    pub use crate::chunk_manager::*;
    pub use crate::collision::*;
    pub use crate::compute::*;
    pub use crate::edge::*;
    pub use crate::event::*;
    pub use crate::intent::*;
    pub use crate::lighting::*;
    pub use crate::npc_collision::*;
    pub use crate::npc_render::*;
    pub use crate::particles::*;
    pub use crate::quadtree::*;
    pub use crate::readback::*;
    pub use crate::render::*;
    pub use crate::streaming::*;
    pub use crate::terrain_render::*;
    pub use crate::topdown_physics::*;
    pub use crate::validation::*;
    pub use crate::world_init::*;
    pub use crate::worldgen::*;
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
