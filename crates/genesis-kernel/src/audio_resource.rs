//! Audio Resource Management
//!
//! Provides abstractions for audio resources including:
//! - `AudioHandle`: Unique identifier for playing sounds
//! - `AudioBuffer`: Cached audio samples for quick playback
//! - `AudioSource`: Streaming or buffered audio data
//! - Per-handle volume, pan, speed controls
//!
//! # Architecture
//!
//! Audio resources are managed through a handle-based system:
//!
//! ```text
//! ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
//! │ AudioBuffer │──────│ AudioSource │──────│ AudioHandle │
//! │  (cached)   │      │ (playable)  │      │ (controls)  │
//! └─────────────┘      └─────────────┘      └─────────────┘
//! ```
//!
//! Short sound effects are loaded into `AudioBuffer` for low-latency playback.
//! Long-form audio (music, ambience) streams from disk via `AudioSource`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tracing::{debug, warn};

/// Maximum number of cached audio buffers.
pub const MAX_CACHED_BUFFERS: usize = 256;

/// Maximum buffer size for caching (samples > this stream from disk).
pub const MAX_CACHEABLE_SIZE: usize = 5 * 1024 * 1024; // 5MB

/// Default crossfade duration for music transitions.
pub const DEFAULT_CROSSFADE_DURATION: Duration = Duration::from_millis(500);

/// Unique identifier for a playing sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioHandle {
    id: u64,
    generation: u32,
}

impl AudioHandle {
    /// Create a new handle with the given ID and generation.
    #[must_use]
    pub const fn new(id: u64, generation: u32) -> Self {
        Self { id, generation }
    }

    /// Get the raw ID.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Get the generation (for detecting stale handles).
    #[must_use]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Create a null/invalid handle.
    #[must_use]
    pub const fn null() -> Self {
        Self {
            id: u64::MAX,
            generation: 0,
        }
    }

    /// Check if this is a null handle.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.id == u64::MAX
    }
}

impl Default for AudioHandle {
    fn default() -> Self {
        Self::null()
    }
}

/// Audio handle generator for unique IDs.
#[derive(Debug)]
pub struct HandleGenerator {
    next_id: AtomicU64,
}

impl Default for HandleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleGenerator {
    /// Create a new handle generator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
        }
    }

    /// Generate a new unique handle.
    pub fn next(&self) -> AudioHandle {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        AudioHandle::new(id, 0)
    }

    /// Generate a new handle with a specific generation.
    pub fn next_with_generation(&self, generation: u32) -> AudioHandle {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        AudioHandle::new(id, generation)
    }
}

/// Audio playback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// Sound is playing normally.
    Playing,
    /// Sound is paused.
    Paused,
    /// Sound has finished playing.
    #[default]
    Stopped,
    /// Sound is fading out before stopping.
    FadingOut,
    /// Sound is fading in.
    FadingIn,
}

impl PlaybackState {
    /// Check if currently playing (not paused/stopped).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Playing | Self::FadingIn | Self::FadingOut)
    }

    /// Check if the sound has ended.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        matches!(self, Self::Stopped)
    }
}

/// Per-handle audio controls.
#[derive(Debug, Clone)]
pub struct AudioControls {
    /// Volume multiplier (0.0-1.0).
    pub volume: f32,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Stereo pan (-1.0 = full left, 0.0 = center, 1.0 = full right).
    pub pan: f32,
    /// Whether to loop the audio.
    pub looping: bool,
    /// Current playback state.
    pub state: PlaybackState,
    /// Fade duration for transitions.
    pub fade_duration: Duration,
}

impl Default for AudioControls {
    fn default() -> Self {
        Self {
            volume: 1.0,
            speed: 1.0,
            pan: 0.0,
            looping: false,
            state: PlaybackState::Stopped,
            fade_duration: DEFAULT_CROSSFADE_DURATION,
        }
    }
}

impl AudioControls {
    /// Create new controls with volume.
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Create new controls with speed.
    #[must_use]
    pub const fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Create new controls with pan.
    #[must_use]
    pub const fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan;
        self
    }

    /// Create new controls with looping.
    #[must_use]
    pub const fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Create new controls with fade duration.
    #[must_use]
    pub const fn with_fade_duration(mut self, duration: Duration) -> Self {
        self.fade_duration = duration;
        self
    }

    /// Clamp values to valid ranges.
    pub fn normalize(&mut self) {
        self.volume = self.volume.clamp(0.0, 1.0);
        self.speed = self.speed.clamp(0.1, 4.0);
        self.pan = self.pan.clamp(-1.0, 1.0);
    }
}

/// Audio buffer identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(u32);

impl BufferId {
    /// Create a new buffer ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID.
    #[must_use]
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Cached audio buffer for quick playback.
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Unique buffer identifier.
    pub id: BufferId,
    /// Audio sample data (interleaved stereo f32).
    pub samples: Arc<Vec<f32>>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Duration of the audio.
    pub duration: Duration,
    /// Original file path (if loaded from file).
    pub source_path: Option<PathBuf>,
}

impl AudioBuffer {
    /// Create a new audio buffer from samples.
    #[must_use]
    pub fn new(
        id: BufferId,
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
    ) -> Self {
        let num_samples = samples.len() / channels as usize;
        let duration_secs = num_samples as f64 / sample_rate as f64;

        Self {
            id,
            samples: Arc::new(samples),
            sample_rate,
            channels,
            duration: Duration::from_secs_f64(duration_secs),
            source_path: None,
        }
    }

    /// Create with source path.
    #[must_use]
    pub fn with_source_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_path = Some(path.into());
        self
    }

    /// Get the size in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.samples.len() * std::mem::size_of::<f32>()
    }

    /// Check if this buffer is suitable for caching.
    #[must_use]
    pub fn is_cacheable(&self) -> bool {
        self.size_bytes() <= MAX_CACHEABLE_SIZE
    }

    /// Get the number of samples per channel.
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }
}

/// Audio source type.
#[derive(Debug, Clone)]
pub enum AudioSourceType {
    /// Buffered audio (fully loaded in memory).
    Buffer(BufferId),
    /// Streaming audio from file.
    Stream {
        /// Path to the audio file.
        path: PathBuf,
        /// Current read position in samples.
        position: usize,
    },
}

/// Audio source configuration.
#[derive(Debug, Clone)]
pub struct AudioSource {
    /// Source type (buffered or streaming).
    pub source_type: AudioSourceType,
    /// Initial playback controls.
    pub controls: AudioControls,
    /// Priority for channel allocation (higher = more important).
    pub priority: u8,
    /// Category for volume mixing.
    pub category: AudioCategory,
}

impl AudioSource {
    /// Create a buffered audio source.
    #[must_use]
    pub fn buffered(buffer_id: BufferId) -> Self {
        Self {
            source_type: AudioSourceType::Buffer(buffer_id),
            controls: AudioControls::default(),
            priority: 128,
            category: AudioCategory::Sfx,
        }
    }

    /// Create a streaming audio source.
    #[must_use]
    pub fn streaming(path: impl Into<PathBuf>) -> Self {
        Self {
            source_type: AudioSourceType::Stream {
                path: path.into(),
                position: 0,
            },
            controls: AudioControls::default(),
            priority: 128,
            category: AudioCategory::Music,
        }
    }

    /// Set controls.
    #[must_use]
    pub fn with_controls(mut self, controls: AudioControls) -> Self {
        self.controls = controls;
        self
    }

    /// Set priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set category.
    #[must_use]
    pub const fn with_category(mut self, category: AudioCategory) -> Self {
        self.category = category;
        self
    }

    /// Check if this is a streaming source.
    #[must_use]
    pub const fn is_streaming(&self) -> bool {
        matches!(self.source_type, AudioSourceType::Stream { .. })
    }
}

/// Audio category for volume mixing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioCategory {
    /// Master volume (affects all audio).
    Master,
    /// Background music.
    Music,
    /// Sound effects.
    #[default]
    Sfx,
    /// Ambient/environmental sounds.
    Ambient,
    /// UI/menu sounds.
    Ui,
    /// Voice/dialogue.
    Voice,
}

impl AudioCategory {
    /// Get the default volume for this category.
    #[must_use]
    pub const fn default_volume(&self) -> f32 {
        match self {
            Self::Master | Self::Sfx | Self::Voice => 1.0,
            Self::Music => 0.7,
            Self::Ambient => 0.5,
            Self::Ui => 0.8,
        }
    }
}

/// Volume settings for audio categories.
#[derive(Debug, Clone)]
pub struct VolumeSettings {
    volumes: HashMap<AudioCategory, f32>,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        let mut volumes = HashMap::new();
        volumes.insert(AudioCategory::Master, 1.0);
        volumes.insert(AudioCategory::Music, AudioCategory::Music.default_volume());
        volumes.insert(AudioCategory::Sfx, AudioCategory::Sfx.default_volume());
        volumes.insert(AudioCategory::Ambient, AudioCategory::Ambient.default_volume());
        volumes.insert(AudioCategory::Ui, AudioCategory::Ui.default_volume());
        volumes.insert(AudioCategory::Voice, AudioCategory::Voice.default_volume());
        Self { volumes }
    }
}

impl VolumeSettings {
    /// Get volume for a category.
    #[must_use]
    pub fn get(&self, category: AudioCategory) -> f32 {
        self.volumes.get(&category).copied().unwrap_or(1.0)
    }

    /// Set volume for a category.
    pub fn set(&mut self, category: AudioCategory, volume: f32) {
        self.volumes.insert(category, volume.clamp(0.0, 1.0));
    }

    /// Get effective volume for a category (includes master).
    #[must_use]
    pub fn effective(&self, category: AudioCategory) -> f32 {
        let master = self.get(AudioCategory::Master);
        let category_vol = self.get(category);
        master * category_vol
    }
}

/// Audio buffer cache.
#[derive(Debug)]
pub struct AudioBufferCache {
    /// Cached buffers by ID.
    buffers: HashMap<BufferId, AudioBuffer>,
    /// Path to buffer ID mapping.
    path_to_id: HashMap<PathBuf, BufferId>,
    /// Next buffer ID.
    next_id: u32,
    /// Total cached size in bytes.
    total_size: usize,
}

impl Default for AudioBufferCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBufferCache {
    /// Create a new buffer cache.
    #[must_use]
    pub fn new() -> Self {
        debug!("Created audio buffer cache");
        Self {
            buffers: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 0,
            total_size: 0,
        }
    }

    /// Get a cached buffer by ID.
    #[must_use]
    pub fn get(&self, id: BufferId) -> Option<&AudioBuffer> {
        self.buffers.get(&id)
    }

    /// Get a cached buffer by path.
    #[must_use]
    pub fn get_by_path(&self, path: &Path) -> Option<&AudioBuffer> {
        self.path_to_id
            .get(path)
            .and_then(|id| self.buffers.get(id))
    }

    /// Check if a path is already cached.
    #[must_use]
    pub fn is_cached(&self, path: &Path) -> bool {
        self.path_to_id.contains_key(path)
    }

    /// Add a buffer to the cache.
    ///
    /// Returns the buffer ID, or `None` if the cache is full.
    pub fn add(&mut self, samples: Vec<f32>, sample_rate: u32, channels: u16) -> Option<BufferId> {
        if self.buffers.len() >= MAX_CACHED_BUFFERS {
            warn!("Audio buffer cache full");
            return None;
        }

        let id = BufferId::new(self.next_id);
        self.next_id += 1;

        let buffer = AudioBuffer::new(id, samples, sample_rate, channels);
        self.total_size += buffer.size_bytes();
        self.buffers.insert(id, buffer);

        debug!("Cached audio buffer {:?}", id);
        Some(id)
    }

    /// Add a buffer with source path.
    pub fn add_with_path(
        &mut self,
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
        path: impl Into<PathBuf>,
    ) -> Option<BufferId> {
        let path = path.into();

        // Check if already cached
        if let Some(&id) = self.path_to_id.get(&path) {
            return Some(id);
        }

        if self.buffers.len() >= MAX_CACHED_BUFFERS {
            warn!("Audio buffer cache full");
            return None;
        }

        let id = BufferId::new(self.next_id);
        self.next_id += 1;

        let buffer = AudioBuffer::new(id, samples, sample_rate, channels)
            .with_source_path(path.clone());
        self.total_size += buffer.size_bytes();

        self.path_to_id.insert(path, id);
        self.buffers.insert(id, buffer);

        debug!("Cached audio buffer {:?}", id);
        Some(id)
    }

    /// Remove a buffer from the cache.
    pub fn remove(&mut self, id: BufferId) -> Option<AudioBuffer> {
        if let Some(buffer) = self.buffers.remove(&id) {
            self.total_size -= buffer.size_bytes();
            if let Some(path) = &buffer.source_path {
                self.path_to_id.remove(path);
            }
            debug!("Removed audio buffer {:?}", id);
            return Some(buffer);
        }
        None
    }

    /// Clear all cached buffers.
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.path_to_id.clear();
        self.total_size = 0;
        debug!("Cleared audio buffer cache");
    }

    /// Get total cached size in bytes.
    #[must_use]
    pub const fn total_size(&self) -> usize {
        self.total_size
    }

    /// Get number of cached buffers.
    #[must_use]
    pub fn count(&self) -> usize {
        self.buffers.len()
    }
}

/// Playing sound instance data.
#[derive(Debug)]
pub struct PlayingSound {
    /// Handle for this sound.
    pub handle: AudioHandle,
    /// Audio source.
    pub source: AudioSource,
    /// Current controls.
    pub controls: AudioControls,
    /// Current playback position (samples).
    pub position: usize,
    /// Whether spatial audio is applied.
    pub is_spatial: bool,
    /// World position (if spatial).
    pub world_position: Option<(f32, f32)>,
    /// Start time (for timing).
    pub start_time: std::time::Instant,
}

impl PlayingSound {
    /// Create a new playing sound.
    #[must_use]
    pub fn new(handle: AudioHandle, source: AudioSource) -> Self {
        let mut controls = source.controls.clone();
        controls.state = PlaybackState::Playing;
        Self {
            handle,
            source,
            controls,
            position: 0,
            is_spatial: false,
            world_position: None,
            start_time: std::time::Instant::now(),
        }
    }

    /// Create with spatial position.
    #[must_use]
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.is_spatial = true;
        self.world_position = Some((x, y));
        self
    }

    /// Get elapsed time since playback started.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if the sound is currently playing.
    #[must_use]
    pub const fn is_playing(&self) -> bool {
        self.controls.state.is_active()
    }

    /// Check if the sound has finished.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.controls.state.is_finished()
    }
}

/// Thread-safe audio resource manager.
#[derive(Debug)]
pub struct AudioResourceManager {
    /// Buffer cache.
    cache: RwLock<AudioBufferCache>,
    /// Handle generator.
    handle_gen: HandleGenerator,
    /// Playing sounds.
    playing: RwLock<HashMap<AudioHandle, PlayingSound>>,
    /// Volume settings.
    volumes: RwLock<VolumeSettings>,
    /// Maximum simultaneous sounds.
    max_sounds: usize,
}

impl Default for AudioResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioResourceManager {
    /// Create a new audio resource manager.
    #[must_use]
    pub fn new() -> Self {
        Self::with_max_sounds(32)
    }

    /// Create with a specific max sound limit.
    #[must_use]
    pub fn with_max_sounds(max_sounds: usize) -> Self {
        debug!("Created audio resource manager (max {} sounds)", max_sounds);
        Self {
            cache: RwLock::new(AudioBufferCache::new()),
            handle_gen: HandleGenerator::new(),
            playing: RwLock::new(HashMap::new()),
            volumes: RwLock::new(VolumeSettings::default()),
            max_sounds,
        }
    }

    /// Get the buffer cache (read access).
    pub fn cache(&self) -> parking_lot::RwLockReadGuard<'_, AudioBufferCache> {
        self.cache.read()
    }

    /// Get the buffer cache (write access).
    pub fn cache_mut(&self) -> parking_lot::RwLockWriteGuard<'_, AudioBufferCache> {
        self.cache.write()
    }

    /// Cache a buffer.
    pub fn cache_buffer(
        &self,
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
    ) -> Option<BufferId> {
        self.cache.write().add(samples, sample_rate, channels)
    }

    /// Cache a buffer with path.
    pub fn cache_buffer_with_path(
        &self,
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
        path: impl Into<PathBuf>,
    ) -> Option<BufferId> {
        self.cache.write().add_with_path(samples, sample_rate, channels, path)
    }

    /// Start playing a sound.
    pub fn play(&self, source: AudioSource) -> Option<AudioHandle> {
        let mut playing = self.playing.write();

        if playing.len() >= self.max_sounds {
            // Try to evict lowest priority stopped sound
            let lowest_stopped = playing
                .iter()
                .filter(|(_, s)| s.is_finished())
                .min_by_key(|(_, s)| s.source.priority)
                .map(|(h, _)| *h);

            if let Some(handle) = lowest_stopped {
                playing.remove(&handle);
            } else {
                warn!("Maximum sounds reached, cannot play new sound");
                return None;
            }
        }

        let handle = self.handle_gen.next();
        let sound = PlayingSound::new(handle, source);
        playing.insert(handle, sound);

        debug!("Started playing sound {:?}", handle);
        Some(handle)
    }

    /// Start playing a spatial sound.
    pub fn play_at(&self, source: AudioSource, x: f32, y: f32) -> Option<AudioHandle> {
        let mut playing = self.playing.write();

        if playing.len() >= self.max_sounds {
            warn!("Maximum sounds reached");
            return None;
        }

        let handle = self.handle_gen.next();
        let sound = PlayingSound::new(handle, source).with_position(x, y);
        playing.insert(handle, sound);

        debug!("Started playing spatial sound {:?} at ({}, {})", handle, x, y);
        Some(handle)
    }

    /// Stop a playing sound.
    pub fn stop(&self, handle: AudioHandle) {
        let mut playing = self.playing.write();
        if let Some(sound) = playing.get_mut(&handle) {
            sound.controls.state = PlaybackState::Stopped;
            debug!("Stopped sound {:?}", handle);
        }
    }

    /// Pause a playing sound.
    pub fn pause(&self, handle: AudioHandle) {
        let mut playing = self.playing.write();
        if let Some(sound) = playing.get_mut(&handle) {
            if sound.controls.state == PlaybackState::Playing {
                sound.controls.state = PlaybackState::Paused;
                debug!("Paused sound {:?}", handle);
            }
        }
    }

    /// Resume a paused sound.
    pub fn resume(&self, handle: AudioHandle) {
        let mut playing = self.playing.write();
        if let Some(sound) = playing.get_mut(&handle) {
            if sound.controls.state == PlaybackState::Paused {
                sound.controls.state = PlaybackState::Playing;
                debug!("Resumed sound {:?}", handle);
            }
        }
    }

    /// Set volume for a playing sound.
    pub fn set_volume(&self, handle: AudioHandle, volume: f32) {
        let mut playing = self.playing.write();
        if let Some(sound) = playing.get_mut(&handle) {
            sound.controls.volume = volume.clamp(0.0, 1.0);
        }
    }

    /// Set pan for a playing sound.
    pub fn set_pan(&self, handle: AudioHandle, pan: f32) {
        let mut playing = self.playing.write();
        if let Some(sound) = playing.get_mut(&handle) {
            sound.controls.pan = pan.clamp(-1.0, 1.0);
        }
    }

    /// Set speed for a playing sound.
    pub fn set_speed(&self, handle: AudioHandle, speed: f32) {
        let mut playing = self.playing.write();
        if let Some(sound) = playing.get_mut(&handle) {
            sound.controls.speed = speed.clamp(0.1, 4.0);
        }
    }

    /// Update position of a spatial sound.
    pub fn update_position(&self, handle: AudioHandle, x: f32, y: f32) {
        let mut playing = self.playing.write();
        if let Some(sound) = playing.get_mut(&handle) {
            sound.world_position = Some((x, y));
        }
    }

    /// Get the current state of a sound.
    #[must_use]
    pub fn state(&self, handle: AudioHandle) -> Option<PlaybackState> {
        self.playing.read().get(&handle).map(|s| s.controls.state)
    }

    /// Check if a sound is playing.
    #[must_use]
    pub fn is_playing(&self, handle: AudioHandle) -> bool {
        self.playing
            .read()
            .get(&handle)
            .is_some_and(PlayingSound::is_playing)
    }

    /// Stop all playing sounds.
    pub fn stop_all(&self) {
        let mut playing = self.playing.write();
        for sound in playing.values_mut() {
            sound.controls.state = PlaybackState::Stopped;
        }
        debug!("Stopped all sounds");
    }

    /// Stop all sounds in a category.
    pub fn stop_category(&self, category: AudioCategory) {
        let mut playing = self.playing.write();
        for sound in playing.values_mut() {
            if sound.source.category == category {
                sound.controls.state = PlaybackState::Stopped;
            }
        }
        debug!("Stopped all {:?} sounds", category);
    }

    /// Clean up finished sounds.
    pub fn cleanup_finished(&self) {
        let mut playing = self.playing.write();
        playing.retain(|_, sound| !sound.is_finished());
    }

    /// Get volume settings.
    pub fn volumes(&self) -> parking_lot::RwLockReadGuard<'_, VolumeSettings> {
        self.volumes.read()
    }

    /// Set category volume.
    pub fn set_category_volume(&self, category: AudioCategory, volume: f32) {
        self.volumes.write().set(category, volume);
    }

    /// Get effective volume for a category.
    #[must_use]
    pub fn effective_volume(&self, category: AudioCategory) -> f32 {
        self.volumes.read().effective(category)
    }

    /// Get number of currently playing sounds.
    #[must_use]
    pub fn playing_count(&self) -> usize {
        self.playing.read().len()
    }

    /// Get number of active (not finished) sounds.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.playing.read().values().filter(|s| s.is_playing()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_handle() {
        let handle = AudioHandle::new(42, 1);
        assert_eq!(handle.id(), 42);
        assert_eq!(handle.generation(), 1);
        assert!(!handle.is_null());

        let null = AudioHandle::null();
        assert!(null.is_null());
    }

    #[test]
    fn test_handle_generator() {
        let gen = HandleGenerator::new();
        let h1 = gen.next();
        let h2 = gen.next();
        let h3 = gen.next();

        assert_ne!(h1.id(), h2.id());
        assert_ne!(h2.id(), h3.id());
    }

    #[test]
    fn test_playback_state() {
        assert!(PlaybackState::Playing.is_active());
        assert!(PlaybackState::FadingIn.is_active());
        assert!(PlaybackState::FadingOut.is_active());
        assert!(!PlaybackState::Paused.is_active());
        assert!(!PlaybackState::Stopped.is_active());

        assert!(PlaybackState::Stopped.is_finished());
        assert!(!PlaybackState::Playing.is_finished());
    }

    #[test]
    fn test_audio_controls() {
        let mut controls = AudioControls::default()
            .with_volume(1.5) // Will be clamped
            .with_pan(0.5)
            .with_looping(true);

        controls.normalize();
        assert!((controls.volume - 1.0).abs() < f32::EPSILON); // Clamped to 1.0
        assert!((controls.pan - 0.5).abs() < f32::EPSILON);
        assert!(controls.looping);
    }

    #[test]
    fn test_audio_buffer() {
        let samples = vec![0.0f32; 44100 * 2]; // 1 second stereo
        let buffer = AudioBuffer::new(BufferId::new(0), samples, 44100, 2);

        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.channels, 2);
        assert!(buffer.duration >= Duration::from_millis(999));
        assert!(buffer.duration <= Duration::from_millis(1001));
    }

    #[test]
    fn test_audio_source() {
        let source = AudioSource::buffered(BufferId::new(1))
            .with_priority(200)
            .with_category(AudioCategory::Sfx);

        assert!(!source.is_streaming());
        assert_eq!(source.priority, 200);
        assert_eq!(source.category, AudioCategory::Sfx);

        let stream = AudioSource::streaming("/path/to/music.mp3");
        assert!(stream.is_streaming());
    }

    #[test]
    fn test_volume_settings() {
        let mut settings = VolumeSettings::default();

        settings.set(AudioCategory::Master, 0.8);
        settings.set(AudioCategory::Music, 0.5);

        assert!((settings.get(AudioCategory::Master) - 0.8).abs() < f32::EPSILON);
        assert!((settings.effective(AudioCategory::Music) - 0.4).abs() < f32::EPSILON); // 0.8 * 0.5
    }

    #[test]
    fn test_buffer_cache() {
        let mut cache = AudioBufferCache::new();

        let samples = vec![0.0f32; 1000];
        let id = cache.add(samples, 44100, 2);
        assert!(id.is_some());

        let id = id.expect("should have id");
        assert!(cache.get(id).is_some());
        assert_eq!(cache.count(), 1);

        cache.remove(id);
        assert!(cache.get(id).is_none());
        assert_eq!(cache.count(), 0);
    }

    #[test]
    fn test_buffer_cache_path() {
        let mut cache = AudioBufferCache::new();

        let samples = vec![0.0f32; 1000];
        let path = PathBuf::from("/test/sound.wav");
        let id = cache.add_with_path(samples.clone(), 44100, 2, path.clone());
        assert!(id.is_some());

        // Should return same ID for same path
        let id2 = cache.add_with_path(samples, 44100, 2, path.clone());
        assert_eq!(id, id2);

        // Should find by path
        assert!(cache.get_by_path(&path).is_some());
        assert!(cache.is_cached(&path));
    }

    #[test]
    fn test_resource_manager_play() {
        let manager = AudioResourceManager::new();

        let source = AudioSource::buffered(BufferId::new(0));
        let handle = manager.play(source);
        assert!(handle.is_some());

        let handle = handle.expect("should have handle");
        assert!(manager.is_playing(handle));
        assert_eq!(manager.playing_count(), 1);
    }

    #[test]
    fn test_resource_manager_controls() {
        let manager = AudioResourceManager::new();

        let source = AudioSource::buffered(BufferId::new(0));
        let handle = manager.play(source).expect("should have handle");

        manager.set_volume(handle, 0.5);
        manager.set_pan(handle, -0.3);
        manager.pause(handle);

        assert_eq!(manager.state(handle), Some(PlaybackState::Paused));

        manager.resume(handle);
        assert_eq!(manager.state(handle), Some(PlaybackState::Playing));

        manager.stop(handle);
        assert_eq!(manager.state(handle), Some(PlaybackState::Stopped));
    }

    #[test]
    fn test_resource_manager_spatial() {
        let manager = AudioResourceManager::new();

        let source = AudioSource::buffered(BufferId::new(0));
        let handle = manager.play_at(source, 100.0, 200.0);
        assert!(handle.is_some());

        let handle = handle.expect("should have handle");
        manager.update_position(handle, 150.0, 250.0);
    }

    #[test]
    fn test_resource_manager_stop_all() {
        let manager = AudioResourceManager::new();

        for i in 0..5 {
            let source = AudioSource::buffered(BufferId::new(i));
            let _ = manager.play(source);
        }

        assert_eq!(manager.playing_count(), 5);
        assert_eq!(manager.active_count(), 5);

        manager.stop_all();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_category_volumes() {
        let manager = AudioResourceManager::new();

        manager.set_category_volume(AudioCategory::Music, 0.5);
        let vol = manager.effective_volume(AudioCategory::Music);
        assert!((vol - 0.5).abs() < f32::EPSILON); // Master is 1.0

        manager.set_category_volume(AudioCategory::Master, 0.8);
        let vol = manager.effective_volume(AudioCategory::Music);
        assert!((vol - 0.4).abs() < f32::EPSILON); // 0.8 * 0.5
    }
}
