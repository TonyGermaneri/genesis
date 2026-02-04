//! Asset Management System
//!
//! Provides manifest-based asset loading with compression, caching,
//! and async support.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

/// Unique identifier for an asset
pub type AssetId = String;

/// Errors that can occur during asset operations
#[derive(Debug, Error)]
pub enum AssetError {
    /// Asset not found in manifest
    #[error("Asset not found: {0}")]
    NotFound(String),

    /// I/O error during asset loading
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error decompressing asset data
    #[error("Decompression error")]
    DecompressionError,

    /// Asset manifest is corrupt or invalid
    #[error("Manifest corrupt or invalid: {0}")]
    ManifestCorrupt(String),

    /// Asset type mismatch
    #[error("Asset type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch {
        /// Expected asset type
        expected: AssetType,
        /// Actual asset type
        actual: AssetType,
    },

    /// Memory budget exceeded
    #[error("Memory budget exceeded: {current} / {budget} bytes")]
    MemoryBudgetExceeded {
        /// Current memory usage
        current: usize,
        /// Memory budget
        budget: usize,
    },
}

/// Types of assets supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    /// Image/texture asset
    Texture,
    /// Sound effect
    Sound,
    /// Background music
    Music,
    /// Font file
    Font,
    /// Shader source
    Shader,
    /// Generic data file
    Data,
    /// Localization strings
    Localization,
}

/// Entry in the asset manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    /// Relative path to the asset file
    pub path: String,
    /// Type of asset
    pub asset_type: AssetType,
    /// SHA-256 hash of the asset
    pub hash: String,
    /// Size in bytes (uncompressed)
    pub size: u64,
    /// Whether the asset is compressed
    pub compressed: bool,
    /// Optional asset group for preloading
    #[serde(default)]
    pub group: Option<String>,
}

/// Asset manifest containing all asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    /// Manifest format version
    pub version: u32,
    /// Map of asset ID to entry
    pub assets: HashMap<String, AssetEntry>,
}

impl Default for AssetManifest {
    fn default() -> Self {
        Self {
            version: 1,
            assets: HashMap::new(),
        }
    }
}

/// A cached asset with metadata
#[derive(Debug)]
pub struct CachedAsset {
    /// Raw asset data
    pub data: Vec<u8>,
    /// Type of the asset
    pub asset_type: AssetType,
    /// When the asset was loaded
    pub loaded_at: Instant,
    /// Size in bytes
    pub size: usize,
}

/// Handle for async asset loading
#[derive(Clone)]
pub struct AssetHandle {
    id: AssetId,
    state: Arc<RwLock<AssetLoadState>>,
}

/// State of an async asset load
#[derive(Debug, Clone)]
pub enum AssetLoadState {
    /// Asset is being loaded
    Loading,
    /// Asset loaded successfully
    Loaded,
    /// Asset load failed
    Failed(String),
}

impl AssetHandle {
    /// Check if the asset is loaded
    pub fn is_loaded(&self) -> bool {
        matches!(*self.state.read(), AssetLoadState::Loaded)
    }

    /// Check if the asset load failed
    pub fn is_failed(&self) -> bool {
        matches!(*self.state.read(), AssetLoadState::Failed(_))
    }

    /// Get the asset ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Asset manager for loading and caching game assets
pub struct AssetManager {
    base_path: PathBuf,
    cache: HashMap<AssetId, CachedAsset>,
    manifest: AssetManifest,
    memory_budget: usize,
    current_memory: usize,
    #[cfg(debug_assertions)]
    hot_reload_enabled: bool,
}

impl AssetManager {
    /// Default memory budget: 512 MB
    pub const DEFAULT_MEMORY_BUDGET: usize = 512 * 1024 * 1024;

    /// Create a new asset manager
    ///
    /// # Arguments
    /// * `base_path` - Base directory for assets
    ///
    /// # Returns
    /// Asset manager or error if manifest cannot be loaded
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self, AssetError> {
        let base_path = base_path.as_ref().to_path_buf();
        let manifest_path = base_path.join("manifest.json");

        let manifest = if manifest_path.exists() {
            let data = std::fs::read_to_string(&manifest_path)?;
            serde_json::from_str(&data).map_err(|e| AssetError::ManifestCorrupt(e.to_string()))?
        } else {
            AssetManifest::default()
        };

        Ok(Self {
            base_path,
            cache: HashMap::new(),
            manifest,
            memory_budget: Self::DEFAULT_MEMORY_BUDGET,
            current_memory: 0,
            #[cfg(debug_assertions)]
            hot_reload_enabled: false,
        })
    }

    /// Create an asset manager with a custom memory budget
    pub fn with_memory_budget(
        base_path: impl AsRef<Path>,
        memory_budget: usize,
    ) -> Result<Self, AssetError> {
        let mut manager = Self::new(base_path)?;
        manager.memory_budget = memory_budget;
        Ok(manager)
    }

    /// Load an asset by ID
    ///
    /// # Arguments
    /// * `id` - Asset identifier
    ///
    /// # Returns
    /// Reference to cached asset or error
    pub fn load(&mut self, id: &str) -> Result<&CachedAsset, AssetError> {
        // Return cached if available
        if self.cache.contains_key(id) {
            return Ok(self.cache.get(id).expect("just checked"));
        }

        // Get manifest entry
        let entry = self
            .manifest
            .assets
            .get(id)
            .ok_or_else(|| AssetError::NotFound(id.to_string()))?
            .clone();

        // Check memory budget
        let required_size = entry.size as usize;
        if self.current_memory + required_size > self.memory_budget {
            // Try to evict old assets
            self.evict_lru(required_size)?;
        }

        // Load the asset
        let asset_path = self.base_path.join(&entry.path);
        let data = std::fs::read(&asset_path)?;

        // Decompress if needed
        let data = if entry.compressed {
            self.decompress(&data)?
        } else {
            data
        };

        let size = data.len();
        let cached = CachedAsset {
            data,
            asset_type: entry.asset_type,
            loaded_at: Instant::now(),
            size,
        };

        self.current_memory += size;
        self.cache.insert(id.to_string(), cached);

        Ok(self.cache.get(id).expect("just inserted"))
    }

    /// Start an async asset load
    ///
    /// # Arguments
    /// * `id` - Asset identifier
    ///
    /// # Returns
    /// Handle to track load progress
    pub fn load_async(&mut self, id: &str) -> AssetHandle {
        let state = Arc::new(RwLock::new(AssetLoadState::Loading));
        let handle = AssetHandle {
            id: id.to_string(),
            state: state.clone(),
        };

        // For now, do a blocking load and update state
        // In a real implementation, this would spawn a task
        match self.load(id) {
            Ok(_) => *state.write() = AssetLoadState::Loaded,
            Err(e) => *state.write() = AssetLoadState::Failed(e.to_string()),
        }

        handle
    }

    /// Unload an asset from cache
    pub fn unload(&mut self, id: &str) {
        if let Some(asset) = self.cache.remove(id) {
            self.current_memory = self.current_memory.saturating_sub(asset.size);
        }
    }

    /// Preload all assets in a group
    pub fn preload_group(&mut self, group: &str) {
        let ids: Vec<String> = self
            .manifest
            .assets
            .iter()
            .filter(|(_, entry)| entry.group.as_deref() == Some(group))
            .map(|(id, _)| id.clone())
            .collect();

        for id in ids {
            let _ = self.load(&id);
        }
    }

    /// Get current memory usage in bytes
    pub fn get_memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Get memory budget in bytes
    pub fn get_memory_budget(&self) -> usize {
        self.memory_budget
    }

    /// Clear all cached assets
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.current_memory = 0;
    }

    /// Check if an asset is cached
    pub fn is_cached(&self, id: &str) -> bool {
        self.cache.contains_key(id)
    }

    /// Get the number of cached assets
    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }

    /// Enable hot reload in debug mode
    #[cfg(debug_assertions)]
    pub fn enable_hot_reload(&mut self, enabled: bool) {
        self.hot_reload_enabled = enabled;
    }

    /// Check for file changes and reload modified assets
    #[cfg(debug_assertions)]
    pub fn check_hot_reload(&mut self) -> Vec<String> {
        if !self.hot_reload_enabled {
            return Vec::new();
        }

        // In a real implementation, this would watch for file changes
        // and reload modified assets
        Vec::new()
    }

    /// Decompress LZ4 compressed data
    #[allow(clippy::unnecessary_wraps)] // Will return Err when real compression is added
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, AssetError> {
        // Simple decompression stub - in production would use lz4
        // For now, assume data is not actually compressed
        let _ = self; // Silence unused_self - will use self for config in real impl
        Ok(data.to_vec())
    }

    /// Evict least recently used assets to free memory
    fn evict_lru(&mut self, required: usize) -> Result<(), AssetError> {
        let mut candidates: Vec<_> = self
            .cache
            .iter()
            .map(|(id, asset)| (id.clone(), asset.loaded_at, asset.size))
            .collect();

        // Sort by load time (oldest first)
        candidates.sort_by_key(|(_, time, _)| *time);

        let mut freed = 0;
        for (id, _, size) in candidates {
            if freed >= required {
                break;
            }
            self.unload(&id);
            freed += size;
        }

        if self.current_memory + required > self.memory_budget {
            return Err(AssetError::MemoryBudgetExceeded {
                current: self.current_memory,
                budget: self.memory_budget,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (AssetManager, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");

        // Create a minimal manifest
        let manifest = AssetManifest {
            version: 1,
            assets: HashMap::from([(
                "test/asset".to_string(),
                AssetEntry {
                    path: "test.txt".to_string(),
                    asset_type: AssetType::Data,
                    hash: "abc123".to_string(),
                    size: 11,
                    compressed: false,
                    group: Some("test".to_string()),
                },
            )]),
        };

        std::fs::write(
            dir.path().join("manifest.json"),
            serde_json::to_string(&manifest).expect("serialize"),
        )
        .expect("write manifest");

        std::fs::write(dir.path().join("test.txt"), "hello world").expect("write test file");

        let manager = AssetManager::new(dir.path()).expect("create manager");
        (manager, dir)
    }

    #[test]
    fn test_asset_manager_creation() {
        let (manager, _dir) = create_test_manager();
        assert_eq!(manager.get_memory_usage(), 0);
        assert_eq!(manager.cached_count(), 0);
    }

    #[test]
    fn test_asset_load() {
        let (mut manager, _dir) = create_test_manager();

        let asset = manager.load("test/asset").expect("load asset");
        assert_eq!(asset.data, b"hello world");
        assert_eq!(asset.asset_type, AssetType::Data);
        assert!(manager.is_cached("test/asset"));
    }

    #[test]
    fn test_asset_not_found() {
        let (mut manager, _dir) = create_test_manager();

        let result = manager.load("nonexistent");
        assert!(matches!(result, Err(AssetError::NotFound(_))));
    }

    #[test]
    fn test_clear_cache() {
        let (mut manager, _dir) = create_test_manager();

        let _ = manager.load("test/asset");
        assert!(manager.get_memory_usage() > 0);

        manager.clear_cache();
        assert_eq!(manager.get_memory_usage(), 0);
        assert_eq!(manager.cached_count(), 0);
    }
}
