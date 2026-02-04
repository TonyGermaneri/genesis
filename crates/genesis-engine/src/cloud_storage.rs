//! Cloud storage preparation and abstraction.
//!
//! This module provides:
//! - StorageBackend trait for storage abstraction
//! - LocalStorage implementation
//! - Sync status tracking
//! - Conflict resolution hooks

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur in cloud storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// File not found.
    #[error("File not found: {0}")]
    NotFound(String),

    /// Sync conflict.
    #[error("Sync conflict: local and remote versions differ")]
    Conflict,

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Quota exceeded.
    #[error("Storage quota exceeded")]
    QuotaExceeded,

    /// Invalid data.
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Operation cancelled.
    #[error("Operation cancelled")]
    Cancelled,

    /// Backend not available.
    #[error("Storage backend not available: {0}")]
    Unavailable(String),
}

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

/// Sync status for a save file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// File is synced with remote.
    Synced,
    /// Local changes pending upload.
    PendingUpload,
    /// Remote changes pending download.
    PendingDownload,
    /// Sync conflict detected.
    Conflict,
    /// Sync in progress.
    Syncing,
    /// Local only (not synced).
    LocalOnly,
    /// Unknown/error state.
    Unknown,
}

impl SyncStatus {
    /// Returns display name for the status.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Synced => "Synced",
            Self::PendingUpload => "Uploading...",
            Self::PendingDownload => "Downloading...",
            Self::Conflict => "Conflict",
            Self::Syncing => "Syncing...",
            Self::LocalOnly => "Local Only",
            Self::Unknown => "Unknown",
        }
    }

    /// Returns whether the file is synced.
    #[must_use]
    pub fn is_synced(self) -> bool {
        matches!(self, Self::Synced)
    }

    /// Returns whether there's a conflict.
    #[must_use]
    pub fn has_conflict(self) -> bool {
        matches!(self, Self::Conflict)
    }
}

/// Metadata for a stored file.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File name/key.
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// Last modified timestamp.
    pub modified: u64,
    /// Content hash/etag.
    pub hash: Option<String>,
    /// Sync status.
    pub sync_status: SyncStatus,
}

impl FileMetadata {
    /// Creates new file metadata.
    #[must_use]
    pub fn new(name: impl Into<String>, size: u64, modified: u64) -> Self {
        Self {
            name: name.into(),
            size,
            modified,
            hash: None,
            sync_status: SyncStatus::Unknown,
        }
    }
}

/// Resolution for sync conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Keep the local version.
    KeepLocal,
    /// Keep the remote version.
    KeepRemote,
    /// Keep both (rename one).
    KeepBoth,
    /// Merge if possible.
    Merge,
    /// Ask user to decide.
    AskUser,
}

/// Conflict information.
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    /// File name.
    pub file_name: String,
    /// Local file metadata.
    pub local: FileMetadata,
    /// Remote file metadata.
    pub remote: FileMetadata,
    /// Suggested resolution.
    pub suggested_resolution: ConflictResolution,
}

/// Callback for conflict resolution.
pub type ConflictResolver = Box<dyn Fn(&ConflictInfo) -> ConflictResolution + Send + Sync>;

/// Trait for storage backends.
pub trait StorageBackend: Send + Sync {
    /// Backend name.
    fn name(&self) -> &str;

    /// Checks if the backend is available/connected.
    fn is_available(&self) -> bool;

    /// Reads a file from storage.
    fn read(&self, key: &str) -> StorageResult<Vec<u8>>;

    /// Writes a file to storage.
    fn write(&self, key: &str, data: &[u8]) -> StorageResult<()>;

    /// Deletes a file from storage.
    fn delete(&self, key: &str) -> StorageResult<()>;

    /// Lists all files in storage.
    fn list(&self) -> StorageResult<Vec<FileMetadata>>;

    /// Gets metadata for a file.
    fn metadata(&self, key: &str) -> StorageResult<FileMetadata>;

    /// Checks if a file exists.
    fn exists(&self, key: &str) -> StorageResult<bool>;

    /// Gets storage quota info (used, total).
    fn quota(&self) -> StorageResult<(u64, u64)>;
}

/// Local filesystem storage implementation.
pub struct LocalStorage {
    /// Base directory for storage.
    base_path: PathBuf,
    /// File metadata cache.
    #[allow(dead_code)]
    metadata_cache: HashMap<String, FileMetadata>,
}

impl LocalStorage {
    /// Creates a new local storage backend.
    #[must_use]
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Returns the base path.
    #[must_use]
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Gets the full path for a key.
    fn key_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    /// Ensures the base directory exists.
    fn ensure_dir(&self) -> StorageResult<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)?;
        }
        Ok(())
    }

    /// Calculates simple hash of data.
    #[allow(dead_code)]
    fn calculate_hash(data: &[u8]) -> String {
        // Simple hash for demo - in production use proper hashing
        let mut sum: u64 = 0;
        for (i, &byte) in data.iter().enumerate() {
            sum = sum.wrapping_add((byte as u64).wrapping_mul((i + 1) as u64));
        }
        format!("{:016x}", sum)
    }
}

impl StorageBackend for LocalStorage {
    fn name(&self) -> &str {
        "Local Storage"
    }

    fn is_available(&self) -> bool {
        self.ensure_dir().is_ok()
    }

    fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let path = self.key_path(key);
        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        debug!("Read {} bytes from {}", data.len(), key);
        Ok(data)
    }

    fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.ensure_dir()?;

        let path = self.key_path(key);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(data)?;
        writer.flush()?;

        debug!("Wrote {} bytes to {}", data.len(), key);
        Ok(())
    }

    fn delete(&self, key: &str) -> StorageResult<()> {
        let path = self.key_path(key);
        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        fs::remove_file(&path)?;
        debug!("Deleted {}", key);
        Ok(())
    }

    fn list(&self) -> StorageResult<Vec<FileMetadata>> {
        self.ensure_dir()?;

        let mut files = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy().to_string();
                    let metadata = entry.metadata()?;
                    let modified = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    files.push(FileMetadata {
                        name,
                        size: metadata.len(),
                        modified,
                        hash: None,
                        sync_status: SyncStatus::LocalOnly,
                    });
                }
            }
        }

        Ok(files)
    }

    fn metadata(&self, key: &str) -> StorageResult<FileMetadata> {
        let path = self.key_path(key);
        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        let metadata = fs::metadata(&path)?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(FileMetadata {
            name: key.to_string(),
            size: metadata.len(),
            modified,
            hash: None,
            sync_status: SyncStatus::LocalOnly,
        })
    }

    fn exists(&self, key: &str) -> StorageResult<bool> {
        Ok(self.key_path(key).exists())
    }

    fn quota(&self) -> StorageResult<(u64, u64)> {
        // Local storage doesn't have a fixed quota
        // Return used space and a large "total" value
        let used: u64 = self
            .list()?
            .iter()
            .map(|f| f.size)
            .sum();

        Ok((used, u64::MAX))
    }
}

/// Configuration for cloud sync.
#[derive(Debug, Clone)]
pub struct CloudSyncConfig {
    /// Whether cloud sync is enabled.
    pub enabled: bool,
    /// Auto-sync interval in seconds.
    pub sync_interval: f64,
    /// Sync on save.
    pub sync_on_save: bool,
    /// Sync on load.
    pub sync_on_load: bool,
    /// Default conflict resolution.
    pub default_conflict_resolution: ConflictResolution,
    /// Upload bandwidth limit (bytes/sec, 0 = unlimited).
    pub upload_limit: u64,
    /// Download bandwidth limit (bytes/sec, 0 = unlimited).
    pub download_limit: u64,
}

impl Default for CloudSyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sync_interval: 300.0,
            sync_on_save: true,
            sync_on_load: true,
            default_conflict_resolution: ConflictResolution::AskUser,
            upload_limit: 0,
            download_limit: 0,
        }
    }
}

/// Sync operation result.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Files uploaded.
    pub uploaded: Vec<String>,
    /// Files downloaded.
    pub downloaded: Vec<String>,
    /// Files with conflicts.
    pub conflicts: Vec<String>,
    /// Errors encountered.
    pub errors: Vec<(String, String)>,
    /// Total bytes transferred.
    pub bytes_transferred: u64,
    /// Duration of sync.
    pub duration: Duration,
}

impl SyncResult {
    /// Creates an empty sync result.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            uploaded: Vec::new(),
            downloaded: Vec::new(),
            conflicts: Vec::new(),
            errors: Vec::new(),
            bytes_transferred: 0,
            duration: Duration::ZERO,
        }
    }

    /// Checks if sync was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors.is_empty() && self.conflicts.is_empty()
    }

    /// Checks if there were any transfers.
    #[must_use]
    pub fn had_transfers(&self) -> bool {
        !self.uploaded.is_empty() || !self.downloaded.is_empty()
    }
}

/// Manager for cloud storage sync.
pub struct CloudStorageManager {
    /// Local storage backend.
    local: LocalStorage,
    /// Remote storage backend (optional).
    remote: Option<Box<dyn StorageBackend>>,
    /// Sync configuration.
    config: CloudSyncConfig,
    /// Conflict resolver callback.
    conflict_resolver: Option<ConflictResolver>,
    /// Time since last sync.
    time_since_sync: f64,
    /// Current sync status.
    sync_status: SyncStatus,
    /// Pending conflicts.
    pending_conflicts: Vec<ConflictInfo>,
}

impl CloudStorageManager {
    /// Creates a new cloud storage manager.
    #[must_use]
    pub fn new(local_path: impl AsRef<Path>) -> Self {
        Self {
            local: LocalStorage::new(local_path),
            remote: None,
            config: CloudSyncConfig::default(),
            conflict_resolver: None,
            time_since_sync: 0.0,
            sync_status: SyncStatus::LocalOnly,
            pending_conflicts: Vec::new(),
        }
    }

    /// Returns the local storage backend.
    #[must_use]
    pub fn local(&self) -> &LocalStorage {
        &self.local
    }

    /// Returns the sync configuration.
    #[must_use]
    pub fn config(&self) -> &CloudSyncConfig {
        &self.config
    }

    /// Returns the current sync status.
    #[must_use]
    pub fn sync_status(&self) -> SyncStatus {
        self.sync_status
    }

    /// Returns pending conflicts.
    #[must_use]
    pub fn pending_conflicts(&self) -> &[ConflictInfo] {
        &self.pending_conflicts
    }

    /// Checks if cloud sync is available.
    #[must_use]
    pub fn is_cloud_available(&self) -> bool {
        self.remote.as_ref().is_some_and(|r| r.is_available())
    }

    /// Updates the sync configuration.
    pub fn update_config(&mut self, config: CloudSyncConfig) {
        self.config = config;
        info!("Cloud sync config updated, enabled: {}", self.config.enabled);
    }

    /// Sets the remote storage backend.
    pub fn set_remote(&mut self, backend: Box<dyn StorageBackend>) {
        info!("Remote storage backend set: {}", backend.name());
        self.remote = Some(backend);
    }

    /// Sets the conflict resolver callback.
    pub fn set_conflict_resolver(&mut self, resolver: ConflictResolver) {
        self.conflict_resolver = Some(resolver);
    }

    /// Updates the cloud storage manager (call each frame).
    pub fn update(&mut self, delta_time: f64) {
        if !self.config.enabled {
            return;
        }

        self.time_since_sync += delta_time;
    }

    /// Checks if auto-sync should trigger.
    #[must_use]
    pub fn should_sync(&self) -> bool {
        self.config.enabled
            && self.time_since_sync >= self.config.sync_interval
            && self.is_cloud_available()
    }

    /// Reads a file, preferring local if available.
    pub fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        // Try local first
        match self.local.read(key) {
            Ok(data) => Ok(data),
            Err(StorageError::NotFound(_)) => {
                // Try remote if available
                if let Some(remote) = &self.remote {
                    if remote.is_available() {
                        return remote.read(key);
                    }
                }
                Err(StorageError::NotFound(key.to_string()))
            }
            Err(e) => Err(e),
        }
    }

    /// Writes a file to local storage.
    pub fn write(&mut self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.local.write(key, data)?;
        self.sync_status = SyncStatus::PendingUpload;
        Ok(())
    }

    /// Deletes a file from local storage.
    pub fn delete(&mut self, key: &str) -> StorageResult<()> {
        self.local.delete(key)?;
        self.sync_status = SyncStatus::PendingUpload;
        Ok(())
    }

    /// Lists all files.
    pub fn list(&self) -> StorageResult<Vec<FileMetadata>> {
        self.local.list()
    }

    /// Performs a full sync with remote storage.
    pub fn sync(&mut self) -> StorageResult<SyncResult> {
        let start = std::time::Instant::now();

        if !self.config.enabled {
            return Ok(SyncResult::empty());
        }

        let remote = match &self.remote {
            Some(r) if r.is_available() => r,
            _ => {
                warn!("Remote storage not available for sync");
                return Ok(SyncResult::empty());
            }
        };

        self.sync_status = SyncStatus::Syncing;

        let mut result = SyncResult::empty();

        // Get file lists
        let local_files = self.local.list()?;
        let remote_files = remote.list()?;

        let local_map: HashMap<_, _> = local_files
            .into_iter()
            .map(|f| (f.name.clone(), f))
            .collect();
        let remote_map: HashMap<_, _> = remote_files
            .into_iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        // Check for files to upload (local but not remote, or local newer)
        for (name, local_meta) in &local_map {
            match remote_map.get(name) {
                None => {
                    // Upload new file
                    if let Ok(data) = self.local.read(name) {
                        if remote.write(name, &data).is_ok() {
                            result.uploaded.push(name.clone());
                            result.bytes_transferred += data.len() as u64;
                        }
                    }
                }
                Some(remote_meta) => {
                    if local_meta.modified > remote_meta.modified {
                        // Local is newer - potential conflict
                        let conflict = ConflictInfo {
                            file_name: name.clone(),
                            local: local_meta.clone(),
                            remote: remote_meta.clone(),
                            suggested_resolution: self.config.default_conflict_resolution,
                        };

                        let resolution = self
                            .conflict_resolver
                            .as_ref()
                            .map(|r| r(&conflict))
                            .unwrap_or(self.config.default_conflict_resolution);

                        match resolution {
                            ConflictResolution::KeepLocal => {
                                if let Ok(data) = self.local.read(name) {
                                    if remote.write(name, &data).is_ok() {
                                        result.uploaded.push(name.clone());
                                        result.bytes_transferred += data.len() as u64;
                                    }
                                }
                            }
                            ConflictResolution::KeepRemote => {
                                if let Ok(data) = remote.read(name) {
                                    if self.local.write(name, &data).is_ok() {
                                        result.downloaded.push(name.clone());
                                        result.bytes_transferred += data.len() as u64;
                                    }
                                }
                            }
                            _ => {
                                self.pending_conflicts.push(conflict);
                                result.conflicts.push(name.clone());
                            }
                        }
                    }
                }
            }
        }

        // Check for files to download (remote but not local)
        for (name, _remote_meta) in &remote_map {
            if !local_map.contains_key(name) {
                if let Ok(data) = remote.read(name) {
                    if self.local.write(name, &data).is_ok() {
                        result.downloaded.push(name.clone());
                        result.bytes_transferred += data.len() as u64;
                    }
                }
            }
        }

        result.duration = start.elapsed();
        self.time_since_sync = 0.0;

        if result.is_success() {
            self.sync_status = SyncStatus::Synced;
            info!(
                "Sync complete: {} uploaded, {} downloaded",
                result.uploaded.len(),
                result.downloaded.len()
            );
        } else {
            self.sync_status = SyncStatus::Conflict;
            warn!(
                "Sync completed with issues: {} conflicts, {} errors",
                result.conflicts.len(),
                result.errors.len()
            );
        }

        Ok(result)
    }

    /// Resolves a pending conflict.
    pub fn resolve_conflict(
        &mut self,
        file_name: &str,
        resolution: ConflictResolution,
    ) -> StorageResult<()> {
        let conflict = self
            .pending_conflicts
            .iter()
            .find(|c| c.file_name == file_name)
            .cloned();

        let Some(conflict) = conflict else {
            return Err(StorageError::NotFound(file_name.to_string()));
        };

        let remote = self
            .remote
            .as_ref()
            .ok_or_else(|| StorageError::Unavailable("No remote backend".to_string()))?;

        match resolution {
            ConflictResolution::KeepLocal => {
                let data = self.local.read(&conflict.file_name)?;
                remote.write(&conflict.file_name, &data)?;
            }
            ConflictResolution::KeepRemote => {
                let data = remote.read(&conflict.file_name)?;
                self.local.write(&conflict.file_name, &data)?;
            }
            ConflictResolution::KeepBoth => {
                // Rename local and download remote
                let data = self.local.read(&conflict.file_name)?;
                let backup_name = format!("{}_local_backup", conflict.file_name);
                self.local.write(&backup_name, &data)?;

                let remote_data = remote.read(&conflict.file_name)?;
                self.local.write(&conflict.file_name, &remote_data)?;
            }
            _ => {
                // Can't auto-resolve
                return Ok(());
            }
        }

        // Remove from pending
        self.pending_conflicts
            .retain(|c| c.file_name != file_name);

        if self.pending_conflicts.is_empty() {
            self.sync_status = SyncStatus::Synced;
        }

        Ok(())
    }

    /// Clears all pending conflicts.
    pub fn clear_conflicts(&mut self) {
        self.pending_conflicts.clear();
    }

    /// Resets the sync timer.
    pub fn reset_sync_timer(&mut self) {
        self.time_since_sync = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_storage_dir() -> PathBuf {
        env::temp_dir().join("genesis_test_cloud")
    }

    fn cleanup_test_dir(path: &Path) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }

    #[test]
    fn test_sync_status() {
        assert!(SyncStatus::Synced.is_synced());
        assert!(!SyncStatus::PendingUpload.is_synced());
        assert!(SyncStatus::Conflict.has_conflict());
        assert!(!SyncStatus::Synced.has_conflict());
    }

    #[test]
    fn test_sync_status_display() {
        assert_eq!(SyncStatus::Synced.display_name(), "Synced");
        assert_eq!(SyncStatus::Conflict.display_name(), "Conflict");
    }

    #[test]
    fn test_file_metadata() {
        let meta = FileMetadata::new("test.sav", 1024, 12345);
        assert_eq!(meta.name, "test.sav");
        assert_eq!(meta.size, 1024);
        assert_eq!(meta.modified, 12345);
    }

    #[test]
    fn test_local_storage_write_read() {
        let dir = test_storage_dir().join("test_write_read");
        cleanup_test_dir(&dir);

        let storage = LocalStorage::new(&dir);

        let data = b"Hello, world!";
        storage.write("test.txt", data).expect("Write failed");

        let read_data = storage.read("test.txt").expect("Read failed");
        assert_eq!(read_data, data);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_local_storage_delete() {
        let dir = test_storage_dir().join("test_delete");
        cleanup_test_dir(&dir);

        let storage = LocalStorage::new(&dir);

        storage.write("to_delete.txt", b"data").expect("Write failed");
        assert!(storage.exists("to_delete.txt").unwrap());

        storage.delete("to_delete.txt").expect("Delete failed");
        assert!(!storage.exists("to_delete.txt").unwrap());

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_local_storage_list() {
        let dir = test_storage_dir().join("test_list");
        cleanup_test_dir(&dir);

        let storage = LocalStorage::new(&dir);

        storage.write("file1.txt", b"data1").expect("Write failed");
        storage.write("file2.txt", b"data2").expect("Write failed");

        let files = storage.list().expect("List failed");
        assert_eq!(files.len(), 2);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_local_storage_metadata() {
        let dir = test_storage_dir().join("test_metadata");
        cleanup_test_dir(&dir);

        let storage = LocalStorage::new(&dir);

        let data = b"Hello!";
        storage.write("test.txt", data).expect("Write failed");

        let meta = storage.metadata("test.txt").expect("Metadata failed");
        assert_eq!(meta.name, "test.txt");
        assert_eq!(meta.size, data.len() as u64);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_local_storage_not_found() {
        let dir = test_storage_dir().join("test_not_found");
        cleanup_test_dir(&dir);

        let storage = LocalStorage::new(&dir);
        let result = storage.read("nonexistent.txt");
        assert!(matches!(result, Err(StorageError::NotFound(_))));

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_cloud_sync_config_default() {
        let config = CloudSyncConfig::default();
        assert!(!config.enabled);
        assert!(config.sync_on_save);
        assert_eq!(config.default_conflict_resolution, ConflictResolution::AskUser);
    }

    #[test]
    fn test_cloud_storage_manager_new() {
        let dir = test_storage_dir().join("test_manager");
        cleanup_test_dir(&dir);

        let manager = CloudStorageManager::new(&dir);
        assert!(!manager.is_cloud_available());
        assert_eq!(manager.sync_status(), SyncStatus::LocalOnly);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_cloud_storage_manager_write_read() {
        let dir = test_storage_dir().join("test_manager_rw");
        cleanup_test_dir(&dir);

        let mut manager = CloudStorageManager::new(&dir);

        let data = b"Test data";
        manager.write("test.sav", data).expect("Write failed");

        let read_data = manager.read("test.sav").expect("Read failed");
        assert_eq!(read_data, data);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_sync_result_empty() {
        let result = SyncResult::empty();
        assert!(result.is_success());
        assert!(!result.had_transfers());
    }

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::NotFound("test.txt".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("not found"));

        let err = StorageError::Conflict;
        let msg = format!("{err}");
        assert!(msg.contains("conflict"));
    }

    #[test]
    fn test_conflict_resolution() {
        assert_eq!(
            ConflictResolution::KeepLocal,
            ConflictResolution::KeepLocal
        );
        assert_ne!(
            ConflictResolution::KeepLocal,
            ConflictResolution::KeepRemote
        );
    }

    #[test]
    fn test_cloud_storage_manager_update() {
        let dir = test_storage_dir().join("test_update");
        cleanup_test_dir(&dir);

        let mut manager = CloudStorageManager::new(&dir);
        manager.update_config(CloudSyncConfig {
            enabled: true,
            sync_interval: 10.0,
            ..Default::default()
        });

        // Not enough time
        manager.update(5.0);
        assert!(!manager.should_sync());

        // Still not enough (and no remote)
        manager.update(6.0);
        assert!(!manager.should_sync()); // No remote available

        cleanup_test_dir(&dir);
    }
}
