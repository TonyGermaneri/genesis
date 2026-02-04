//! UI module for game interface components.
//!
//! This module provides specialized UI widgets and panels:
//! - Inventory panel (6x9 grid)
//! - Player stats HUD (health, hunger, stamina)
//! - Environment HUD (time, weather)
//! - Chunk-based minimap
//! - Biome visualization and debug tools

pub mod biome;
pub mod environment;
pub mod inventory;
pub mod minimap;
pub mod stats;

pub use biome::*;
pub use environment::*;
pub use inventory::*;
pub use minimap::*;
pub use stats::*;
