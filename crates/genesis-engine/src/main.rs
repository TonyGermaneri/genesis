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
mod input;
mod perf;
mod renderer;
mod timing;

/// Analytics module for opt-in gameplay telemetry
pub mod analytics;
/// Crash reporting and error capture
pub mod crash_report;

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
