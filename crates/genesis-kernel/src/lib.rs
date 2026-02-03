//! # Genesis Kernel
//!
//! GPU compute kernel for pixel-cell simulation.
//!
//! This crate provides the core GPU-accelerated simulation:
//! - Pixel-cell simulation (each pixel is a simulated cell)
//! - Buffer layouts for GPU compute
//! - GPU validation harness
//!
//! ## Architecture
//!
//! The kernel runs on the GPU and is the physical authority for the world.
//! The CPU entity layer (gameplay crate) submits intents which the kernel
//! executes as part of the simulation step.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod cell;
pub mod compute;
pub mod validation;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::cell::*;
    pub use crate::compute::*;
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
