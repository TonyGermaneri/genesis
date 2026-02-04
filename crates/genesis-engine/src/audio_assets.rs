//! Audio asset loading and management.
//!
//! This module provides:
//! - Loading MP3/WAV files from assets/sounds/
//! - Caching strategy (SFX cached, music streamed)
//! - Asset validation on load
//! - Fallback/placeholder for missing files
//! - Hot-reload support for development

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use rodio::{Decoder, Source};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Default asset base path relative to executable.
pub const DEFAULT_ASSET_PATH: &str = "assets/sounds";

/// Errors that can occur during audio asset operations.
#[derive(Debug, Error)]
pub enum AudioAssetError {
    /// File not found.
    #[error("Audio file not found: {0}")]
    NotFound(PathBuf),

    /// Failed to read file.
    #[error("Failed to read audio file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to decode audio.
    #[error("Failed to decode audio: {0}")]
    DecodeError(String),

    /// Asset is a stub placeholder.
    #[error("Asset is a stub placeholder: {0}")]
    StubFile(PathBuf),

    /// Invalid audio format.
    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),
}

/// Result type for audio asset operations.
pub type AudioAssetResult<T> = Result<T, AudioAssetError>;

/// Metadata for a loaded audio asset.
#[derive(Debug, Clone)]
pub struct AudioAssetInfo {
    /// Path to the asset file.
    pub path: PathBuf,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Duration in seconds (if known).
    pub duration_secs: Option<f32>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: u16,
    /// Whether this is a stub/placeholder.
    pub is_stub: bool,
    /// Last modification time.
    pub modified: Option<SystemTime>,
}

/// A cached audio asset with its raw data.
#[derive(Debug, Clone)]
pub struct CachedAudio {
    /// Raw audio data (WAV/MP3 bytes).
    pub data: Arc<Vec<u8>>,
    /// Asset metadata.
    pub info: AudioAssetInfo,
}

impl CachedAudio {
    /// Creates a decoder for this cached audio.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio data cannot be decoded.
    pub fn decoder(&self) -> AudioAssetResult<Decoder<Cursor<Vec<u8>>>> {
        let cursor = Cursor::new((*self.data).clone());
        Decoder::new(cursor).map_err(|e| AudioAssetError::DecodeError(e.to_string()))
    }

    /// Returns true if this is a placeholder/stub asset.
    #[must_use]
    pub fn is_placeholder(&self) -> bool {
        self.info.is_stub
    }
}

/// Asset category for organization and caching policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioCategory {
    /// Music tracks (streamed, not cached).
    Music,
    /// Ambient sounds (streamed).
    Ambient,
    /// Sound effects (cached in memory).
    Sfx,
    /// UI sounds (cached).
    Ui,
}

impl AudioCategory {
    /// Returns the subdirectory for this category.
    #[must_use]
    pub const fn subdirectory(&self) -> &'static str {
        match self {
            Self::Music => "music",
            Self::Ambient => "ambient",
            Self::Sfx => "sfx",
            Self::Ui => "ui",
        }
    }

    /// Returns whether assets in this category should be cached.
    #[must_use]
    pub const fn should_cache(&self) -> bool {
        match self {
            Self::Music | Self::Ambient => false, // Stream these
            Self::Sfx | Self::Ui => true,         // Cache these
        }
    }
}

/// Audio asset loader with caching.
pub struct AudioAssetLoader {
    /// Base path for assets.
    base_path: PathBuf,
    /// Cache of loaded SFX and UI sounds.
    cache: HashMap<String, CachedAudio>,
    /// Modification times for hot-reload detection.
    mod_times: HashMap<PathBuf, SystemTime>,
    /// Whether hot-reload is enabled.
    hot_reload_enabled: bool,
    /// Stub file extension.
    stub_extension: String,
    /// Statistics.
    stats: LoaderStats,
}

/// Statistics for the asset loader.
#[derive(Debug, Default, Clone)]
pub struct LoaderStats {
    /// Number of cached assets.
    pub cached_count: usize,
    /// Total memory used by cache (bytes).
    pub cache_memory_bytes: u64,
    /// Number of assets loaded this session.
    pub loads_this_session: u64,
    /// Number of cache hits.
    pub cache_hits: u64,
    /// Number of stub files encountered.
    pub stubs_encountered: u64,
    /// Number of missing files.
    pub missing_files: u64,
}

impl AudioAssetLoader {
    /// Creates a new audio asset loader.
    #[must_use]
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let base_path = base_path.into();
        info!("Initializing audio asset loader at: {:?}", base_path);

        Self {
            base_path,
            cache: HashMap::new(),
            mod_times: HashMap::new(),
            hot_reload_enabled: cfg!(debug_assertions), // Enable in debug builds
            stub_extension: ".stub".to_string(),
            stats: LoaderStats::default(),
        }
    }

    /// Creates a loader with default asset path.
    #[must_use]
    pub fn with_default_path() -> Self {
        Self::new(DEFAULT_ASSET_PATH)
    }

    /// Enables or disables hot-reload.
    #[must_use]
    pub fn with_hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload_enabled = enabled;
        self
    }

    /// Returns the base asset path.
    #[must_use]
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Returns loader statistics.
    #[must_use]
    pub fn stats(&self) -> &LoaderStats {
        &self.stats
    }

    /// Loads an audio asset by category and name.
    ///
    /// For SFX/UI, this caches the asset. For Music/Ambient, it loads on demand.
    ///
    /// # Arguments
    ///
    /// * `category` - The asset category.
    /// * `name` - The asset name (without extension).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be found or decoded.
    pub fn load(&mut self, category: AudioCategory, name: &str) -> AudioAssetResult<CachedAudio> {
        let cache_key = format!("{}/{}", category.subdirectory(), name);

        // Check cache first (only for cacheable categories)
        if category.should_cache() {
            if let Some(cached) = self.cache.get(&cache_key) {
                self.stats.cache_hits += 1;
                return Ok(cached.clone());
            }
        }

        // Try to load the file
        let asset = self.load_from_disk(category, name)?;

        // Cache if appropriate
        if category.should_cache() {
            self.stats.cache_memory_bytes += asset.info.size_bytes;
            self.stats.cached_count += 1;
            self.cache.insert(cache_key, asset.clone());
        }

        self.stats.loads_this_session += 1;
        Ok(asset)
    }

    /// Loads an asset directly from disk.
    fn load_from_disk(
        &mut self,
        category: AudioCategory,
        name: &str,
    ) -> AudioAssetResult<CachedAudio> {
        let subdir = category.subdirectory();

        // Try different extensions
        let extensions = ["mp3", "wav", "ogg"];
        let mut found_path: Option<PathBuf> = None;

        for ext in &extensions {
            let path = self.base_path.join(subdir).join(format!("{name}.{ext}"));
            if path.exists() {
                found_path = Some(path);
                break;
            }
        }

        // Check for stub file if no real file found
        if found_path.is_none() {
            for ext in &extensions {
                let stub_path = self
                    .base_path
                    .join(subdir)
                    .join(format!("{name}.{ext}{}", self.stub_extension));
                if stub_path.exists() {
                    warn!("Found stub file for {}/{}: {:?}", subdir, name, stub_path);
                    self.stats.stubs_encountered += 1;
                    return Ok(Self::create_placeholder(stub_path));
                }
            }
        }

        let path = found_path.ok_or_else(|| {
            self.stats.missing_files += 1;
            AudioAssetError::NotFound(self.base_path.join(subdir).join(name))
        })?;

        // Load file data
        let data = fs::read(&path)?;
        let size_bytes = data.len() as u64;

        // Get modification time for hot-reload
        let modified = fs::metadata(&path).ok().and_then(|m| m.modified().ok());

        if self.hot_reload_enabled {
            if let Some(time) = modified {
                self.mod_times.insert(path.clone(), time);
            }
        }

        // Try to get audio info
        let (sample_rate, channels, duration_secs) = Self::extract_audio_info(&data);

        let info = AudioAssetInfo {
            path,
            size_bytes,
            duration_secs,
            sample_rate,
            channels,
            is_stub: false,
            modified,
        };

        debug!(
            "Loaded audio asset: {:?} ({} bytes, {}Hz, {} channels)",
            info.path, size_bytes, sample_rate, channels
        );

        Ok(CachedAudio {
            data: Arc::new(data),
            info,
        })
    }

    /// Extracts audio info from raw data.
    fn extract_audio_info(data: &[u8]) -> (u32, u16, Option<f32>) {
        let cursor = Cursor::new(data.to_vec());
        if let Ok(decoder) = Decoder::new(cursor) {
            let sample_rate = decoder.sample_rate();
            let channels = decoder.channels();
            // Duration estimation from total samples
            let duration = decoder.total_duration().map(|d| d.as_secs_f32());
            (sample_rate, channels, duration)
        } else {
            // Defaults if we can't decode
            (44100, 2, None)
        }
    }

    /// Creates a placeholder asset for missing audio.
    fn create_placeholder(stub_path: PathBuf) -> CachedAudio {
        // Create silent audio data (minimal WAV header + silence)
        let silent_wav = Self::create_silent_wav(0.1); // 100ms of silence

        CachedAudio {
            data: Arc::new(silent_wav),
            info: AudioAssetInfo {
                path: stub_path,
                size_bytes: 0,
                duration_secs: Some(0.1),
                sample_rate: 44100,
                channels: 2,
                is_stub: true,
                modified: None,
            },
        }
    }

    /// Creates a minimal silent WAV file.
    fn create_silent_wav(duration_secs: f32) -> Vec<u8> {
        let sample_rate: u32 = 44100;
        let channels: u16 = 2;
        let bits_per_sample: u16 = 16;
        let num_samples = (sample_rate as f32 * duration_secs) as u32;
        let data_size = num_samples * u32::from(channels) * u32::from(bits_per_sample / 8);

        let mut wav = Vec::with_capacity(44 + data_size as usize);

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample / 8);
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        let block_align = channels * (bits_per_sample / 8);
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());

        // Silent samples (zeros)
        wav.resize(44 + data_size as usize, 0);

        wav
    }

    /// Checks for modified files and reloads them (hot-reload).
    ///
    /// Returns the number of assets reloaded.
    pub fn check_hot_reload(&mut self) -> usize {
        if !self.hot_reload_enabled {
            return 0;
        }

        let mut reloaded = 0;
        let keys_to_reload: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, cached)| {
                if let Some(old_time) = self.mod_times.get(&cached.info.path) {
                    if let Ok(metadata) = fs::metadata(&cached.info.path) {
                        if let Ok(new_time) = metadata.modified() {
                            return new_time > *old_time;
                        }
                    }
                }
                false
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_reload {
            if let Some(cached) = self.cache.get(&key) {
                let path = cached.info.path.clone();
                // Re-parse category and name from key
                if let Some((category_str, name)) = key.split_once('/') {
                    let category = match category_str {
                        "music" => AudioCategory::Music,
                        "ambient" => AudioCategory::Ambient,
                        "sfx" => AudioCategory::Sfx,
                        "ui" => AudioCategory::Ui,
                        _ => continue,
                    };

                    // Update cache memory stats
                    self.stats.cache_memory_bytes -= cached.info.size_bytes;
                    self.cache.remove(&key);

                    // Reload
                    if let Ok(new_asset) = self.load_from_disk(category, name) {
                        info!("Hot-reloaded audio asset: {:?}", path);
                        self.stats.cache_memory_bytes += new_asset.info.size_bytes;
                        self.cache.insert(key, new_asset);
                        reloaded += 1;
                    }
                }
            }
        }

        reloaded
    }

    /// Preloads all SFX and UI assets for immediate playback.
    ///
    /// # Errors
    ///
    /// Logs warnings for failed assets but doesn't fail overall.
    pub fn preload_sfx(&mut self) {
        info!("Preloading SFX assets...");

        let sfx_path = self.base_path.join("sfx");
        let ui_path = self.base_path.join("ui");

        self.preload_directory(&sfx_path, AudioCategory::Sfx);
        self.preload_directory(&ui_path, AudioCategory::Ui);

        info!(
            "Preload complete: {} assets cached ({} KB)",
            self.stats.cached_count,
            self.stats.cache_memory_bytes / 1024
        );
    }

    /// Preloads all assets in a directory.
    fn preload_directory(&mut self, dir_path: &Path, category: AudioCategory) {
        if !dir_path.exists() {
            debug!("Asset directory not found: {:?}", dir_path);
            return;
        }

        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read asset directory {:?}: {}", dir_path, e);
                return;
            },
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Handle subdirectories recursively
            if path.is_dir() {
                self.preload_directory(&path, category);
                continue;
            }

            // Skip stub files for preload
            if path.to_string_lossy().ends_with(&self.stub_extension) {
                continue;
            }

            // Get name without extension
            if let Some(stem) = path.file_stem() {
                let _name = stem.to_string_lossy();
                // Build relative name including subdirectories
                let relative_path = path
                    .strip_prefix(self.base_path.join(category.subdirectory()))
                    .unwrap_or(&path);
                let relative_name = relative_path
                    .with_extension("")
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/");

                if let Err(e) = self.load(category, &relative_name) {
                    warn!("Failed to preload {:?}: {}", path, e);
                }
            }
        }
    }

    /// Clears the asset cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.stats.cached_count = 0;
        self.stats.cache_memory_bytes = 0;
        info!("Audio asset cache cleared");
    }

    /// Returns true if a specific asset is cached.
    #[must_use]
    pub fn is_cached(&self, category: AudioCategory, name: &str) -> bool {
        let cache_key = format!("{}/{}", category.subdirectory(), name);
        self.cache.contains_key(&cache_key)
    }

    /// Returns the cached asset if available.
    #[must_use]
    pub fn get_cached(&self, category: AudioCategory, name: &str) -> Option<&CachedAudio> {
        let cache_key = format!("{}/{}", category.subdirectory(), name);
        self.cache.get(&cache_key)
    }

    /// Lists all available assets in a category.
    #[must_use]
    pub fn list_assets(&self, category: AudioCategory) -> Vec<String> {
        let dir_path = self.base_path.join(category.subdirectory());
        let mut assets = Vec::new();

        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy();
                        // Skip stub extensions in names
                        let clean_name = name.trim_end_matches(".mp3").trim_end_matches(".wav");
                        if !assets.contains(&clean_name.to_string()) {
                            assets.push(clean_name.to_string());
                        }
                    }
                }
            }
        }

        assets
    }
}

impl Default for AudioAssetLoader {
    fn default() -> Self {
        Self::with_default_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_category_subdirectory() {
        assert_eq!(AudioCategory::Music.subdirectory(), "music");
        assert_eq!(AudioCategory::Ambient.subdirectory(), "ambient");
        assert_eq!(AudioCategory::Sfx.subdirectory(), "sfx");
        assert_eq!(AudioCategory::Ui.subdirectory(), "ui");
    }

    #[test]
    fn test_audio_category_should_cache() {
        assert!(!AudioCategory::Music.should_cache());
        assert!(!AudioCategory::Ambient.should_cache());
        assert!(AudioCategory::Sfx.should_cache());
        assert!(AudioCategory::Ui.should_cache());
    }

    #[test]
    fn test_loader_stats_default() {
        let stats = LoaderStats::default();
        assert_eq!(stats.cached_count, 0);
        assert_eq!(stats.cache_memory_bytes, 0);
        assert_eq!(stats.loads_this_session, 0);
    }

    #[test]
    fn test_create_silent_wav() {
        let wav = AudioAssetLoader::create_silent_wav(0.1);
        // Should have RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
    }

    #[test]
    fn test_loader_new() {
        let loader = AudioAssetLoader::new("/test/path");
        assert_eq!(loader.base_path(), Path::new("/test/path"));
        assert_eq!(loader.stats().cached_count, 0);
    }

    #[test]
    fn test_loader_with_hot_reload() {
        let loader = AudioAssetLoader::new("/test/path").with_hot_reload(false);
        assert!(!loader.hot_reload_enabled);
    }

    #[test]
    fn test_loader_cache_key_format() {
        let loader = AudioAssetLoader::new("/test");
        // Cache key should be "category/name"
        assert!(!loader.is_cached(AudioCategory::Sfx, "test_sound"));
    }

    #[test]
    fn test_audio_asset_info() {
        let info = AudioAssetInfo {
            path: PathBuf::from("/test/sound.wav"),
            size_bytes: 1024,
            duration_secs: Some(1.5),
            sample_rate: 44100,
            channels: 2,
            is_stub: false,
            modified: None,
        };

        assert_eq!(info.size_bytes, 1024);
        assert_eq!(info.sample_rate, 44100);
        assert!(!info.is_stub);
    }

    #[test]
    fn test_placeholder_creation() {
        let placeholder =
            AudioAssetLoader::create_placeholder(PathBuf::from("/test/stub.mp3.stub"));

        assert!(placeholder.is_placeholder());
        assert!(placeholder.info.is_stub);
        assert!(!placeholder.data.is_empty());
    }
}
