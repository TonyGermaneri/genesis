//! # Genesis Engine
//!
//! Main entry point for Project Genesis - an action RPG with GPU-compute
//! pixel-cell simulation.
//!
//! This crate ties together all subsystems:
//! - Kernel: GPU compute pipeline for pixel-cell simulation
//! - World: Chunk streaming and persistence
//! - Gameplay: Entities, inventory, crafting, economy

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

mod app;
mod config;
mod environment;
mod input;
mod perf;
mod renderer;
mod timing;
mod world;

/// Analytics module for opt-in gameplay telemetry
pub mod analytics;
/// Audio asset loading and caching
pub mod audio_assets;
/// Audio system integration
pub mod audio_integration;
/// Audio state management
pub mod audio_state;
/// Combat event integration
pub mod combat_events;
/// Combat profiling and metrics
pub mod combat_profile;
/// Combat persistence
pub mod combat_save;
/// Crafting event integration
pub mod crafting_events;
/// Crafting profiling and metrics
pub mod crafting_profile;
/// Crafting persistence
pub mod crafting_save;
/// Crash reporting and error capture
pub mod crash_report;
/// Recipe asset loading
pub mod recipe_loader;
/// Weapon data loading
pub mod weapon_loader;

// === Save System ===
/// Auto-save system
pub mod autosave;
/// Cloud storage abstraction
pub mod cloud_storage;
/// Save file manager
pub mod save_manager;
/// Save file versioning
pub mod save_version;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Main entry point.
fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("genesis=info".parse()?))
        .init();

    info!("Project Genesis starting...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Run the application
    app::run()?;

    info!("Project Genesis shutdown complete");
    Ok(())
}
