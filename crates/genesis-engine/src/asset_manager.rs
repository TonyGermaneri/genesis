//! Asset management for the Genesis engine.
//!
//! This module handles loading and managing game assets:
//! - Terrain textures (48x48 pixel tiles)
//! - Autotile atlases for GPU rendering
//! - Asset hot-reloading (future)

use std::path::{Path, PathBuf};
use std::sync::Arc;

use genesis_kernel::{TerrainTextureAtlas, TerrainAssetManifest, TerrainCategory};
use genesis_kernel::autotile_atlas::AutotileAtlas;
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};
use wgpu::{Device, Queue};

/// Default path for Modern Exteriors 48x48 assets (singles)
pub const DEFAULT_TERRAIN_ASSETS_PATH: &str =
    "/Users/tonygermaneri/gh/game_assets/modernexteriors-win/Modern_Exteriors_48x48/Modern_Exteriors_Complete_Singles_48x48";

/// Default path for Modern Exteriors 48x48 autotiles
pub const DEFAULT_AUTOTILE_PATH: &str =
    "/Users/tonygermaneri/gh/game_assets/modernexteriors-win/Modern_Exteriors_48x48/Autotiles_48x48/Godot_Autotiles_48x48.png";

/// Asset loading configuration
#[derive(Debug, Clone)]
pub struct AssetConfig {
    /// Base path for terrain assets (singles)
    pub terrain_path: PathBuf,
    /// Path for autotile atlas
    pub autotile_path: PathBuf,
    /// Whether to load assets on startup
    pub load_on_startup: bool,
    /// Maximum textures to load (for memory limits)
    pub max_terrain_tiles: usize,
    /// Use autotile atlas instead of singles
    pub use_autotiles: bool,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            terrain_path: PathBuf::from(DEFAULT_TERRAIN_ASSETS_PATH),
            autotile_path: PathBuf::from(DEFAULT_AUTOTILE_PATH),
            load_on_startup: true,
            max_terrain_tiles: 1024,
            use_autotiles: true, // Prefer autotiles by default
        }
    }
}

/// Asset loading status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetLoadStatus {
    /// Not yet started
    NotLoaded,
    /// Currently loading
    Loading,
    /// Successfully loaded
    Loaded,
    /// Failed to load
    Failed,
}

/// Asset loading statistics
#[derive(Debug, Clone, Default)]
pub struct AssetStats {
    /// Number of terrain tiles loaded
    pub terrain_tiles_loaded: usize,
    /// Number of terrain tiles failed
    pub terrain_tiles_failed: usize,
    /// Time taken to load terrain (seconds)
    pub terrain_load_time: f64,
    /// Atlas dimensions
    pub atlas_width: u32,
    pub atlas_height: u32,
    /// GPU upload status
    pub gpu_uploaded: bool,
    /// Using autotiles
    pub using_autotiles: bool,
    /// Number of terrain types in autotile atlas
    pub autotile_terrain_count: u32,
}

/// Asset manager - central hub for all game assets
pub struct AssetManager {
    /// Configuration
    config: AssetConfig,
    /// Terrain texture atlas (singles)
    terrain_atlas: Arc<RwLock<TerrainTextureAtlas>>,
    /// Autotile atlas
    autotile_atlas: Arc<RwLock<AutotileAtlas>>,
    /// Loading status
    status: AssetLoadStatus,
    /// Statistics
    stats: AssetStats,
}

impl AssetManager {
    /// Create a new asset manager with default config
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(AssetConfig::default())
    }

    /// Create an asset manager with custom config
    #[must_use]
    pub fn with_config(config: AssetConfig) -> Self {
        Self {
            config,
            terrain_atlas: Arc::new(RwLock::new(TerrainTextureAtlas::new())),
            autotile_atlas: Arc::new(RwLock::new(AutotileAtlas::new())),
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

    /// Load autotile atlas from PNG file
    pub fn load_autotile_atlas(&mut self) -> Result<(), String> {
        info!("Loading autotile atlas from: {:?}", self.config.autotile_path);
        self.status = AssetLoadStatus::Loading;

        let start = std::time::Instant::now();

        // Check if path exists
        if !self.config.autotile_path.exists() {
            let msg = format!("Autotile atlas path does not exist: {:?}", self.config.autotile_path);
            error!("{}", msg);
            self.status = AssetLoadStatus::Failed;
            return Err(msg);
        }

        // Load autotile atlas
        let mut atlas = self.autotile_atlas.write();
        match atlas.load_from_file(&self.config.autotile_path) {
            Ok(()) => {
                let params = atlas.params();
                self.stats.terrain_tiles_loaded = (params.terrain_count * 48) as usize; // 48 tiles per terrain
                self.stats.terrain_load_time = start.elapsed().as_secs_f64();
                self.stats.atlas_width = params.atlas_width;
                self.stats.atlas_height = params.atlas_height;
                self.stats.using_autotiles = true;
                self.stats.autotile_terrain_count = params.terrain_count;
                self.status = AssetLoadStatus::Loaded;

                info!(
                    "Loaded autotile atlas in {:.2}s ({} terrain types, {}x{})",
                    self.stats.terrain_load_time, params.terrain_count,
                    params.atlas_width, params.atlas_height
                );

                Ok(())
            }
            Err(e) => {
                error!("Failed to load autotile atlas: {}", e);
                self.status = AssetLoadStatus::Failed;
                Err(e)
            }
        }
    }

    /// Load all terrain assets (singles - legacy method)
    pub fn load_terrain_assets(&mut self) -> Result<usize, String> {
        info!("Loading terrain assets from: {:?}", self.config.terrain_path);
        self.status = AssetLoadStatus::Loading;

        let start = std::time::Instant::now();

        // Check if path exists
        if !self.config.terrain_path.exists() {
            let msg = format!("Terrain asset path does not exist: {:?}", self.config.terrain_path);
            error!("{}", msg);
            self.status = AssetLoadStatus::Failed;
            return Err(msg);
        }

        // Load into atlas
        let mut atlas = self.terrain_atlas.write();
        match atlas.load_from_directory(&self.config.terrain_path) {
            Ok(count) => {
                self.stats.terrain_tiles_loaded = count;
                self.stats.terrain_load_time = start.elapsed().as_secs_f64();
                let (w, h) = atlas.dimensions();
                self.stats.atlas_width = w;
                self.stats.atlas_height = h;
                self.status = AssetLoadStatus::Loaded;

                info!(
                    "Loaded {} terrain tiles in {:.2}s (atlas: {}x{})",
                    count, self.stats.terrain_load_time, w, h
                );

                Ok(count)
            }
            Err(e) => {
                error!("Failed to load terrain assets: {}", e);
                self.status = AssetLoadStatus::Failed;
                Err(e)
            }
        }
    }

    /// Upload terrain atlas to GPU
    pub fn upload_terrain_to_gpu(&mut self, device: &Device, queue: &Queue) {
        if self.status != AssetLoadStatus::Loaded {
            warn!("Cannot upload terrain to GPU - not loaded yet");
            return;
        }

        if self.stats.using_autotiles {
            // Upload autotile atlas
            let mut atlas = self.autotile_atlas.write();
            match atlas.upload_to_gpu(device, queue) {
                Ok(()) => {
                    self.stats.gpu_uploaded = true;
                    info!("Autotile atlas uploaded to GPU");
                }
                Err(e) => {
                    error!("Failed to upload autotile atlas to GPU: {}", e);
                    self.stats.gpu_uploaded = false;
                }
            }
        } else {
            // Upload singles atlas (legacy)
            let mut atlas = self.terrain_atlas.write();
            atlas.upload_to_gpu(device, queue);
            self.stats.gpu_uploaded = atlas.is_gpu_ready();

            if self.stats.gpu_uploaded {
                info!("Terrain atlas uploaded to GPU");
            } else {
                error!("Failed to upload terrain atlas to GPU");
            }
        }
    }

    /// Get shared reference to terrain atlas (singles, if loaded)
    #[must_use]
    pub fn terrain_atlas(&self) -> Option<Arc<RwLock<TerrainTextureAtlas>>> {
        let atlas = self.terrain_atlas.read();
        if atlas.tile_count() > 0 {
            drop(atlas);
            Some(Arc::clone(&self.terrain_atlas))
        } else {
            None
        }
    }

    /// Get shared reference to autotile atlas (if loaded)
    #[must_use]
    pub fn autotile_atlas(&self) -> Option<Arc<RwLock<AutotileAtlas>>> {
        let atlas = self.autotile_atlas.read();
        if atlas.is_loaded() {
            drop(atlas);
            Some(Arc::clone(&self.autotile_atlas))
        } else {
            None
        }
    }

    /// Check if using autotiles
    #[must_use]
    pub fn is_using_autotiles(&self) -> bool {
        self.stats.using_autotiles
    }

    /// Set custom terrain asset path
    pub fn set_terrain_path<P: AsRef<Path>>(&mut self, path: P) {
        self.config.terrain_path = path.as_ref().to_path_buf();
        self.status = AssetLoadStatus::NotLoaded;
    }

    /// Set custom autotile atlas path
    pub fn set_autotile_path<P: AsRef<Path>>(&mut self, path: P) {
        self.config.autotile_path = path.as_ref().to_path_buf();
        self.status = AssetLoadStatus::NotLoaded;
    }

    /// Check if a specific terrain category has any tiles loaded
    #[must_use]
    pub fn has_terrain_category(&self, category: TerrainCategory) -> bool {
        if self.stats.using_autotiles {
            // Autotiles have all terrain types
            true
        } else {
            let atlas = self.terrain_atlas.read();
            // Check if we can get any tile for this category
            atlas.get_tile_for_biome(
                match category {
                    TerrainCategory::Grass => 0,
                    TerrainCategory::Water => 7,
                    TerrainCategory::Sand => 2,
                    TerrainCategory::Dirt => 1,
                    TerrainCategory::Stone => 5,
                    TerrainCategory::Asphalt => 0,
                    TerrainCategory::Sidewalk => 0,
                    TerrainCategory::Mound => 5,
                    TerrainCategory::Snow => 3,
                    TerrainCategory::Swamp => 4,
                },
                genesis_kernel::NeighborMask::NONE,
                0,
            ).is_some()
        }
    }

    /// Get terrain tile count
    #[must_use]
    pub fn terrain_tile_count(&self) -> u32 {
        if self.stats.using_autotiles {
            self.stats.autotile_terrain_count * 48 // 48 tiles per terrain type
        } else {
            self.terrain_atlas.read().tile_count()
        }
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global asset manager instance (for easy access from render pipeline)
static ASSET_MANAGER: parking_lot::RwLock<Option<AssetManager>> = parking_lot::RwLock::new(None);

/// Initialize global asset manager
pub fn init_global_asset_manager(config: AssetConfig) {
    let mut manager = ASSET_MANAGER.write();
    *manager = Some(AssetManager::with_config(config));
}

/// Get reference to global asset manager
pub fn global_asset_manager() -> Option<parking_lot::RwLockReadGuard<'static, Option<AssetManager>>> {
    let guard = ASSET_MANAGER.read();
    if guard.is_some() {
        Some(guard)
    } else {
        None
    }
}

/// Get mutable reference to global asset manager
pub fn global_asset_manager_mut() -> Option<parking_lot::RwLockWriteGuard<'static, Option<AssetManager>>> {
    let guard = ASSET_MANAGER.write();
    if guard.is_some() {
        Some(guard)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_config_default() {
        let config = AssetConfig::default();
        assert!(config.load_on_startup);
        assert_eq!(config.max_terrain_tiles, 1024);
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
