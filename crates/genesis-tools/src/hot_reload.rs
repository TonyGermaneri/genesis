//! Hot reload support for assets.
//!
//! This module provides:
//! - File watching for asset changes
//! - Safe resource swapping
//! - Reload notifications
//! - Support for materials and shaders

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};

/// Result type for hot reload operations.
pub type HotReloadResult<T> = Result<T, HotReloadError>;

/// Errors that can occur during hot reload.
#[derive(Debug, Clone)]
pub enum HotReloadError {
    /// File not found
    FileNotFound(String),
    /// Failed to read file
    ReadError(String),
    /// Failed to parse asset
    ParseError(String),
    /// Watcher error
    WatcherError(String),
    /// Resource is locked
    ResourceLocked(String),
}

impl std::fmt::Display for HotReloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => write!(f, "File not found: {path}"),
            Self::ReadError(msg) => write!(f, "Read error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::WatcherError(msg) => write!(f, "Watcher error: {msg}"),
            Self::ResourceLocked(name) => write!(f, "Resource locked: {name}"),
        }
    }
}

impl std::error::Error for HotReloadError {}

/// Type of asset being watched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    /// Material definition (RON, JSON, etc.)
    Material,
    /// Shader source (GLSL, WGSL, etc.)
    Shader,
    /// Texture image
    Texture,
    /// Configuration file
    Config,
    /// Generic asset
    Other,
}

impl AssetType {
    /// Returns typical file extensions for this asset type.
    #[must_use]
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Material => &["ron", "json", "toml", "yaml"],
            Self::Shader => &["glsl", "wgsl", "vert", "frag", "comp", "hlsl"],
            Self::Texture => &["png", "jpg", "jpeg", "bmp", "tga"],
            Self::Config => &["ron", "json", "toml", "yaml", "cfg"],
            Self::Other => &[],
        }
    }

    /// Guesses the asset type from a file extension.
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        let ext = ext.to_lowercase();
        if Self::Material.extensions().contains(&ext.as_str()) {
            Self::Material
        } else if Self::Shader.extensions().contains(&ext.as_str()) {
            Self::Shader
        } else if Self::Texture.extensions().contains(&ext.as_str()) {
            Self::Texture
        } else {
            Self::Other
        }
    }
}

/// A file change event.
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    /// Path to the changed file
    pub path: PathBuf,
    /// Type of asset
    pub asset_type: AssetType,
    /// Type of change
    pub change_type: FileChangeType,
    /// When the change was detected
    pub timestamp: Instant,
}

/// Type of file change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
}

/// Configuration for the hot reload system.
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Debounce duration (minimum time between reloads)
    pub debounce: Duration,
    /// Whether to auto-reload on change
    pub auto_reload: bool,
    /// Directories to watch
    pub watch_dirs: Vec<PathBuf>,
    /// File extensions to watch (empty = all)
    pub extensions: Vec<String>,
    /// Whether enabled
    pub enabled: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(100),
            auto_reload: true,
            watch_dirs: vec![],
            extensions: vec![],
            enabled: true,
        }
    }
}

/// A watched resource handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle(u64);

impl ResourceHandle {
    /// Creates a new handle.
    fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID.
    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.0
    }
}

/// Metadata about a watched resource.
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// Handle for this resource
    pub handle: ResourceHandle,
    /// Path to the source file
    pub path: PathBuf,
    /// Asset type
    pub asset_type: AssetType,
    /// Last modification time
    pub last_modified: Option<SystemTime>,
    /// Last reload time
    pub last_reload: Option<Instant>,
    /// Number of times reloaded
    pub reload_count: u32,
    /// Whether currently loading
    pub is_loading: bool,
}

/// Callback for when a resource is reloaded.
pub type ReloadCallback = Box<dyn Fn(&ResourceInfo) + Send + Sync>;

/// Hot reload manager.
#[derive(Debug)]
pub struct HotReloader {
    /// Configuration
    config: HotReloadConfig,
    /// Registered resources
    resources: RwLock<HashMap<ResourceHandle, ResourceInfo>>,
    /// Path to handle mapping
    path_to_handle: RwLock<HashMap<PathBuf, ResourceHandle>>,
    /// Next handle ID
    next_id: AtomicU64,
    /// Pending changes
    pending_changes: RwLock<Vec<FileChangeEvent>>,
    /// Last check time
    last_check: RwLock<Instant>,
}

impl Default for HotReloader {
    fn default() -> Self {
        Self::new(HotReloadConfig::default())
    }
}

impl HotReloader {
    /// Creates a new hot reloader.
    #[must_use]
    pub fn new(config: HotReloadConfig) -> Self {
        Self {
            config,
            resources: RwLock::new(HashMap::new()),
            path_to_handle: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            pending_changes: RwLock::new(Vec::new()),
            last_check: RwLock::new(Instant::now()),
        }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &HotReloadConfig {
        &self.config
    }

    /// Enables or disables hot reload.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Registers a resource to be watched.
    pub fn register<P: AsRef<Path>>(
        &self,
        path: P,
        asset_type: AssetType,
    ) -> HotReloadResult<ResourceHandle> {
        let path = path.as_ref().to_path_buf();

        // Check if already registered
        if let Ok(path_map) = self.path_to_handle.read() {
            if let Some(&handle) = path_map.get(&path) {
                return Ok(handle);
            }
        }

        let handle = ResourceHandle::new(self.next_id.fetch_add(1, Ordering::Relaxed));

        let last_modified = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());

        let info = ResourceInfo {
            handle,
            path: path.clone(),
            asset_type,
            last_modified,
            last_reload: None,
            reload_count: 0,
            is_loading: false,
        };

        if let Ok(mut resources) = self.resources.write() {
            resources.insert(handle, info);
        }

        if let Ok(mut path_map) = self.path_to_handle.write() {
            path_map.insert(path, handle);
        }

        Ok(handle)
    }

    /// Unregisters a resource.
    pub fn unregister(&self, handle: ResourceHandle) {
        if let Ok(mut resources) = self.resources.write() {
            if let Some(info) = resources.remove(&handle) {
                if let Ok(mut path_map) = self.path_to_handle.write() {
                    path_map.remove(&info.path);
                }
            }
        }
    }

    /// Gets information about a resource.
    #[must_use]
    pub fn get_info(&self, handle: ResourceHandle) -> Option<ResourceInfo> {
        self.resources
            .read()
            .ok()
            .and_then(|r| r.get(&handle).cloned())
    }

    /// Gets a handle by path.
    #[must_use]
    pub fn get_handle<P: AsRef<Path>>(&self, path: P) -> Option<ResourceHandle> {
        self.path_to_handle
            .read()
            .ok()
            .and_then(|m| m.get(path.as_ref()).copied())
    }

    /// Checks for file changes (manual polling).
    pub fn check_for_changes(&self) -> Vec<FileChangeEvent> {
        if !self.config.enabled {
            return Vec::new();
        }

        let now = Instant::now();

        // Check debounce
        if let Ok(last) = self.last_check.read() {
            if now.duration_since(*last) < self.config.debounce {
                return Vec::new();
            }
        }

        if let Ok(mut last) = self.last_check.write() {
            *last = now;
        }

        let mut changes = Vec::new();

        if let Ok(mut resources) = self.resources.write() {
            for info in resources.values_mut() {
                if let Ok(metadata) = std::fs::metadata(&info.path) {
                    if let Ok(modified) = metadata.modified() {
                        let changed = info.last_modified.map_or(true, |last| modified > last);

                        if changed {
                            info.last_modified = Some(modified);
                            changes.push(FileChangeEvent {
                                path: info.path.clone(),
                                asset_type: info.asset_type,
                                change_type: FileChangeType::Modified,
                                timestamp: now,
                            });
                        }
                    }
                }
            }
        }

        changes
    }

    /// Records a file change event.
    pub fn record_change(&self, event: FileChangeEvent) {
        if let Ok(mut pending) = self.pending_changes.write() {
            pending.push(event);
        }
    }

    /// Takes all pending changes.
    pub fn take_pending_changes(&self) -> Vec<FileChangeEvent> {
        if let Ok(mut pending) = self.pending_changes.write() {
            std::mem::take(&mut *pending)
        } else {
            Vec::new()
        }
    }

    /// Marks a resource as reloaded.
    pub fn mark_reloaded(&self, handle: ResourceHandle) {
        if let Ok(mut resources) = self.resources.write() {
            if let Some(info) = resources.get_mut(&handle) {
                info.last_reload = Some(Instant::now());
                info.reload_count += 1;
                info.is_loading = false;
            }
        }
    }

    /// Marks a resource as loading.
    pub fn mark_loading(&self, handle: ResourceHandle) {
        if let Ok(mut resources) = self.resources.write() {
            if let Some(info) = resources.get_mut(&handle) {
                info.is_loading = true;
            }
        }
    }

    /// Returns all registered handles.
    #[must_use]
    pub fn all_handles(&self) -> Vec<ResourceHandle> {
        self.resources
            .read()
            .ok()
            .map(|r| r.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Returns the number of registered resources.
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resources.read().ok().map_or(0, |r| r.len())
    }
}

/// A versioned resource that can be safely swapped.
#[derive(Debug)]
pub struct VersionedResource<T> {
    /// Current version
    current: RwLock<Arc<T>>,
    /// Version number
    version: AtomicU64,
    /// Associated handle
    handle: Option<ResourceHandle>,
}

impl<T> VersionedResource<T> {
    /// Creates a new versioned resource.
    pub fn new(value: T) -> Self {
        Self {
            current: RwLock::new(Arc::new(value)),
            version: AtomicU64::new(1),
            handle: None,
        }
    }

    /// Creates with an associated handle.
    pub fn with_handle(value: T, handle: ResourceHandle) -> Self {
        Self {
            current: RwLock::new(Arc::new(value)),
            version: AtomicU64::new(1),
            handle: Some(handle),
        }
    }

    /// Gets a read reference to the current value.
    pub fn get(&self) -> Option<Arc<T>> {
        self.current.read().ok().map(|r| Arc::clone(&r))
    }

    /// Swaps the current value with a new one.
    pub fn swap(&self, new_value: T) -> u64 {
        let version = self.version.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut current) = self.current.write() {
            *current = Arc::new(new_value);
        }
        version
    }

    /// Returns the current version.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    /// Returns the associated handle.
    #[must_use]
    pub const fn handle(&self) -> Option<ResourceHandle> {
        self.handle
    }
}

/// A simple material definition for hot reload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDef {
    /// Material name
    pub name: String,
    /// Base color (RGBA)
    pub color: [f32; 4],
    /// Roughness (0.0 - 1.0)
    pub roughness: f32,
    /// Metallic (0.0 - 1.0)
    pub metallic: f32,
    /// Optional texture path
    pub texture: Option<String>,
    /// Optional normal map path
    pub normal_map: Option<String>,
}

impl Default for MaterialDef {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            texture: None,
            normal_map: None,
        }
    }
}

impl MaterialDef {
    /// Loads from a JSON file.
    pub fn load_json<P: AsRef<Path>>(path: P) -> HotReloadResult<Self> {
        let path = path.as_ref();
        let contents =
            std::fs::read_to_string(path).map_err(|e| HotReloadError::ReadError(e.to_string()))?;
        serde_json::from_str(&contents).map_err(|e| HotReloadError::ParseError(e.to_string()))
    }

    /// Saves to a JSON file.
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> HotReloadResult<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| HotReloadError::ParseError(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| HotReloadError::ReadError(e.to_string()))
    }
}

/// A simple shader definition for hot reload.
#[derive(Debug, Clone)]
pub struct ShaderDef {
    /// Shader name
    pub name: String,
    /// Shader source code
    pub source: String,
    /// Shader stage (vertex, fragment, compute)
    pub stage: ShaderStage,
}

/// Shader stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderStage {
    /// Vertex shader
    Vertex,
    /// Fragment shader
    Fragment,
    /// Compute shader
    Compute,
}

impl ShaderDef {
    /// Loads from a file.
    pub fn load<P: AsRef<Path>>(path: P, stage: ShaderStage) -> HotReloadResult<Self> {
        let path = path.as_ref();
        let source =
            std::fs::read_to_string(path).map_err(|e| HotReloadError::ReadError(e.to_string()))?;

        let name = path.file_stem().map_or_else(
            || "unnamed".to_string(),
            |s| s.to_string_lossy().to_string(),
        );

        Ok(Self {
            name,
            source,
            stage,
        })
    }
}

/// Notification channel for reload events.
pub struct ReloadNotifier {
    /// Sender for notifications
    sender: mpsc::Sender<ReloadNotification>,
    /// Receiver for notifications
    receiver: mpsc::Receiver<ReloadNotification>,
}

impl Default for ReloadNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ReloadNotifier {
    /// Creates a new notifier.
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }

    /// Sends a notification.
    pub fn notify(&self, notification: ReloadNotification) {
        let _ = self.sender.send(notification);
    }

    /// Tries to receive a notification (non-blocking).
    pub fn try_recv(&self) -> Option<ReloadNotification> {
        self.receiver.try_recv().ok()
    }

    /// Receives all pending notifications.
    pub fn drain(&self) -> Vec<ReloadNotification> {
        let mut notifications = Vec::new();
        while let Ok(n) = self.receiver.try_recv() {
            notifications.push(n);
        }
        notifications
    }
}

/// A reload notification.
#[derive(Debug, Clone)]
pub struct ReloadNotification {
    /// Handle of the reloaded resource
    pub handle: ResourceHandle,
    /// Path to the resource
    pub path: PathBuf,
    /// Asset type
    pub asset_type: AssetType,
    /// Whether reload was successful
    pub success: bool,
    /// Optional error message
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_asset_type_extensions() {
        assert!(AssetType::Material.extensions().contains(&"ron"));
        assert!(AssetType::Shader.extensions().contains(&"glsl"));
        assert!(AssetType::Texture.extensions().contains(&"png"));
    }

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("glsl"), AssetType::Shader);
        assert_eq!(AssetType::from_extension("png"), AssetType::Texture);
        assert_eq!(AssetType::from_extension("xyz"), AssetType::Other);
    }

    #[test]
    fn test_hot_reload_config_defaults() {
        let config = HotReloadConfig::default();
        assert!(config.auto_reload);
        assert!(config.enabled);
        assert!(config.debounce > Duration::ZERO);
    }

    #[test]
    fn test_resource_handle() {
        let handle = ResourceHandle::new(42);
        assert_eq!(handle.raw(), 42);
    }

    #[test]
    fn test_hot_reloader_register() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.ron");
        std::fs::write(&file_path, "test").expect("write");

        let reloader = HotReloader::default();
        let handle = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");

        assert!(reloader.get_info(handle).is_some());
        assert_eq!(reloader.resource_count(), 1);
    }

    #[test]
    fn test_hot_reloader_unregister() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.ron");
        std::fs::write(&file_path, "test").expect("write");

        let reloader = HotReloader::default();
        let handle = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");

        reloader.unregister(handle);
        assert!(reloader.get_info(handle).is_none());
        assert_eq!(reloader.resource_count(), 0);
    }

    #[test]
    fn test_hot_reloader_duplicate_register() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.ron");
        std::fs::write(&file_path, "test").expect("write");

        let reloader = HotReloader::default();
        let handle1 = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");
        let handle2 = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");

        assert_eq!(handle1, handle2);
        assert_eq!(reloader.resource_count(), 1);
    }

    #[test]
    fn test_hot_reloader_check_changes() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.ron");
        std::fs::write(&file_path, "initial").expect("write");

        let mut config = HotReloadConfig::default();
        config.debounce = Duration::from_millis(1);
        let reloader = HotReloader::new(config);

        let _handle = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");

        // Initial check should return no changes
        let changes = reloader.check_for_changes();
        assert!(changes.is_empty());

        // Wait and modify
        std::thread::sleep(Duration::from_millis(10));
        std::fs::write(&file_path, "modified").expect("write");

        std::thread::sleep(Duration::from_millis(10));
        let changes = reloader.check_for_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, FileChangeType::Modified);
    }

    #[test]
    fn test_hot_reloader_disabled() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.ron");
        std::fs::write(&file_path, "test").expect("write");

        let mut config = HotReloadConfig::default();
        config.enabled = false;
        let reloader = HotReloader::new(config);

        let _handle = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");

        let changes = reloader.check_for_changes();
        assert!(changes.is_empty());
    }

    #[test]
    fn test_versioned_resource() {
        let resource = VersionedResource::new("v1".to_string());

        assert_eq!(resource.version(), 1);
        assert_eq!(*resource.get().expect("get"), "v1");

        let new_version = resource.swap("v2".to_string());
        assert_eq!(new_version, 2);
        assert_eq!(*resource.get().expect("get"), "v2");
    }

    #[test]
    fn test_versioned_resource_with_handle() {
        let handle = ResourceHandle::new(42);
        let resource = VersionedResource::with_handle(123, handle);

        assert_eq!(resource.handle(), Some(handle));
        assert_eq!(*resource.get().expect("get"), 123);
    }

    #[test]
    fn test_material_def_default() {
        let mat = MaterialDef::default();
        assert_eq!(mat.name, "default");
        assert_eq!(mat.roughness, 0.5);
    }

    #[test]
    fn test_material_def_json_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("material.json");

        let mat = MaterialDef {
            name: "test".to_string(),
            color: [1.0, 0.0, 0.0, 1.0],
            roughness: 0.7,
            metallic: 0.3,
            texture: Some("texture.png".to_string()),
            normal_map: None,
        };

        mat.save_json(&file_path).expect("save");
        let loaded = MaterialDef::load_json(&file_path).expect("load");

        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(loaded.roughness, 0.7);
    }

    #[test]
    fn test_shader_def_load() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.glsl");
        std::fs::write(&file_path, "void main() {}").expect("write");

        let shader = ShaderDef::load(&file_path, ShaderStage::Vertex).expect("load");
        assert_eq!(shader.name, "test");
        assert_eq!(shader.source, "void main() {}");
        assert_eq!(shader.stage, ShaderStage::Vertex);
    }

    #[test]
    fn test_reload_notifier() {
        let notifier = ReloadNotifier::new();

        assert!(notifier.try_recv().is_none());

        let notification = ReloadNotification {
            handle: ResourceHandle::new(1),
            path: PathBuf::from("test.ron"),
            asset_type: AssetType::Material,
            success: true,
            error: None,
        };

        notifier.notify(notification.clone());

        let received = notifier.try_recv();
        assert!(received.is_some());
        assert!(received.expect("receive").success);
    }

    #[test]
    fn test_reload_notifier_drain() {
        let notifier = ReloadNotifier::new();

        for i in 0..3 {
            notifier.notify(ReloadNotification {
                handle: ResourceHandle::new(i),
                path: PathBuf::from(format!("test{i}.ron")),
                asset_type: AssetType::Material,
                success: true,
                error: None,
            });
        }

        let all = notifier.drain();
        assert_eq!(all.len(), 3);
        assert!(notifier.try_recv().is_none());
    }

    #[test]
    fn test_mark_loading_and_reloaded() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.ron");
        std::fs::write(&file_path, "test").expect("write");

        let reloader = HotReloader::default();
        let handle = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");

        reloader.mark_loading(handle);
        let info = reloader.get_info(handle).expect("info");
        assert!(info.is_loading);
        assert_eq!(info.reload_count, 0);

        reloader.mark_reloaded(handle);
        let info = reloader.get_info(handle).expect("info");
        assert!(!info.is_loading);
        assert_eq!(info.reload_count, 1);
    }

    #[test]
    fn test_get_handle_by_path() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("test.ron");
        std::fs::write(&file_path, "test").expect("write");

        let reloader = HotReloader::default();
        let handle = reloader
            .register(&file_path, AssetType::Material)
            .expect("register");

        assert_eq!(reloader.get_handle(&file_path), Some(handle));
        assert_eq!(reloader.get_handle("nonexistent.ron"), None);
    }

    #[test]
    fn test_all_handles() {
        let dir = tempdir().expect("tempdir");

        let reloader = HotReloader::default();

        let path1 = dir.path().join("test1.ron");
        let path2 = dir.path().join("test2.glsl");
        std::fs::write(&path1, "test").expect("write");
        std::fs::write(&path2, "test").expect("write");

        let h1 = reloader
            .register(&path1, AssetType::Material)
            .expect("register");
        let h2 = reloader
            .register(&path2, AssetType::Shader)
            .expect("register");

        let handles = reloader.all_handles();
        assert_eq!(handles.len(), 2);
        assert!(handles.contains(&h1));
        assert!(handles.contains(&h2));
    }

    #[test]
    fn test_file_change_event() {
        let event = FileChangeEvent {
            path: PathBuf::from("test.ron"),
            asset_type: AssetType::Material,
            change_type: FileChangeType::Modified,
            timestamp: Instant::now(),
        };

        assert_eq!(event.asset_type, AssetType::Material);
        assert_eq!(event.change_type, FileChangeType::Modified);
    }

    #[test]
    fn test_hot_reload_error_display() {
        let err = HotReloadError::FileNotFound("test.ron".to_string());
        assert!(format!("{err}").contains("test.ron"));

        let err = HotReloadError::ParseError("invalid json".to_string());
        assert!(format!("{err}").contains("Parse"));
    }

    #[test]
    fn test_pending_changes() {
        let reloader = HotReloader::default();

        reloader.record_change(FileChangeEvent {
            path: PathBuf::from("test.ron"),
            asset_type: AssetType::Material,
            change_type: FileChangeType::Modified,
            timestamp: Instant::now(),
        });

        let pending = reloader.take_pending_changes();
        assert_eq!(pending.len(), 1);

        // Should be empty after taking
        let pending = reloader.take_pending_changes();
        assert!(pending.is_empty());
    }
}
