//! Asset management for the Genesis engine.
//!
//! Stub module - terrain asset loading has been removed.
//! This provides placeholder types for API compatibility.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use wgpu::{Device, Queue};
use tracing::info;

/// Default path for terrain assets (unused - terrain removed)
pub const DEFAULT_TERRAIN_ASSETS_PATH: &str = "";

/// Default path for autotile atlas (unused - terrain removed)
pub const DEFAULT_AUTOTILE_PATH: &str = "";

/// Path for debug autotile atlas (unused - terrain removed)
pub const DEBUG_AUTOTILE_PATH: &str = "";

/// Asset loading configuration (stub)
#[derive(Debug, Clone)]
pub struct AssetConfig {
    /// Base path for terrain assets
    pub terrain_path: PathBuf,
    /// Path for autotile atlas
    pub autotile_path: PathBuf,
    /// Whether to load assets on startup
    pub load_on_startup: bool,
    /// Maximum textures to load
    pub max_terrain_tiles: usize,
    /// Use autotile atlas
    pub use_autotiles: bool,
    /// Use debug atlas
    pub use_debug_atlas: bool,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            terrain_path: PathBuf::new(),
            autotile_path: PathBuf::new(),
            load_on_startup: false,
            max_terrain_tiles: 0,
            use_autotiles: false,
            use_debug_atlas: false,
        }
    }
}

impl AssetConfig {
    /// Create config for debug atlas (stub)
    #[must_use]
    pub fn with_debug_atlas() -> Self {
        Self::default()
    }
}

/// Asset loading status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetLoadStatus {
    /// Not loaded
    NotLoaded,
    /// Currently loading
    Loading,
    /// Successfully loaded
    Loaded,
    /// Failed to load
    Failed,
}

/// Asset loading statistics (stub)
#[derive(Debug, Clone, Default)]
pub struct AssetStats {
    /// Number of terrain tiles loaded
    pub terrain_tiles_loaded: usize,
    /// Number of terrain tiles failed
    pub terrain_tiles_failed: usize,
    /// Time taken to load terrain
    pub terrain_load_time: f64,
    /// Atlas width
    pub atlas_width: u32,
    /// Atlas height
    pub atlas_height: u32,
    /// GPU upload status
    pub gpu_uploaded: bool,
    /// Using autotiles
    pub using_autotiles: bool,
    /// Number of terrain types in autotile atlas
    pub autotile_terrain_count: u32,
}

/// Asset manager - stub (terrain removed)
pub struct AssetManager {
    /// Configuration
    config: AssetConfig,
    /// Loading status
    status: AssetLoadStatus,
    /// Statistics
    stats: AssetStats,
}

impl AssetManager {
    /// Create a new asset manager
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(AssetConfig::default())
    }

    /// Create an asset manager with custom config
    #[must_use]
    pub fn with_config(config: AssetConfig) -> Self {
        Self {
            config,
            status: AssetLoadStatus::NotLoaded,
            stats: AssetStats::default(),
        }
    }

    /// Get current loading status
    #[must_use]
    pub fn status(&self) -> AssetLoadStatus {
        self.status
    }

    /// Get loading statistics
    #[must_use]
    pub fn stats(&self) -> &AssetStats {
        &self.stats
    }

    /// Load autotile atlas (stub - always fails)
    pub fn load_autotile_atlas(&mut self) -> Result<(), String> {
        info!("Autotile atlas loading skipped (terrain system removed)");
        Err("Terrain system removed".to_string())
    }

    /// Load terrain assets (stub - always fails)
    pub fn load_terrain_assets(&mut self) -> Result<usize, String> {
        info!("Terrain asset loading skipped (terrain system removed)");
        Err("Terrain system removed".to_string())
    }

    /// Upload terrain atlas to GPU (stub - no-op)
    pub fn upload_terrain_to_gpu(&mut self, _device: &Device, _queue: &Queue) {
        info!("Terrain GPU upload skipped (terrain system removed)");
    }

    /// Get terrain atlas (stub - always None)
    #[must_use]
    pub fn terrain_atlas<T>(&self) -> Option<T> {
        None
    }

    /// Get autotile atlas (stub - always None)
    #[must_use]
    pub fn autotile_atlas<T>(&self) -> Option<T> {
        None
    }

    /// Check if using autotiles
    #[must_use]
    pub fn is_using_autotiles(&self) -> bool {
        false
    }

    /// Set terrain asset path
    pub fn set_terrain_path<P: AsRef<Path>>(&mut self, path: P) {
        self.config.terrain_path = path.as_ref().to_path_buf();
    }

    /// Set autotile atlas path
    pub fn set_autotile_path<P: AsRef<Path>>(&mut self, path: P) {
        self.config.autotile_path = path.as_ref().to_path_buf();
    }

    /// Get terrain tile count (always 0)
    #[must_use]
    pub fn terrain_tile_count(&self) -> u32 {
        0
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_config_default() {
        let config = AssetConfig::default();
        assert!(!config.load_on_startup);
        assert_eq!(config.max_terrain_tiles, 0);
    }

    #[test]
    fn test_asset_manager_creation() {
        let manager = AssetManager::new();
        assert_eq!(manager.status(), AssetLoadStatus::NotLoaded);
        assert_eq!(manager.terrain_tile_count(), 0);
    }

    #[test]
    fn test_asset_stats_default() {
        let stats = AssetStats::default();
        assert_eq!(stats.terrain_tiles_loaded, 0);
        assert!(!stats.gpu_uploaded);
    }
}
