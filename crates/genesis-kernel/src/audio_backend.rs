//! Audio Backend with Rodio Integration
//!
//! This module provides the core audio playback infrastructure using rodio.
//! It includes:
//!
//! - `AudioDevice`: Wrapper around rodio's output stream
//! - `AudioSinkPool`: Pool of sinks for managing multiple simultaneous sounds
//! - Streaming support for MP3/WAV/FLAC audio files
//! - Sample caching for frequently-used sound effects
//! - Thread-safe audio playback
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      AudioEngine                             │
//! │  ┌───────────────┐  ┌────────────────┐  ┌────────────────┐  │
//! │  │  AudioDevice  │──│  AudioSinkPool │──│ ResourceManager │  │
//! │  │  (rodio)      │  │  (32 sinks)    │  │ (caching)       │  │
//! │  └───────────────┘  └────────────────┘  └────────────────┘  │
//! │           │                  │                   │          │
//! │           ▼                  ▼                   ▼          │
//! │     OutputStream        Sink[0..N]          BufferCache     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use genesis_kernel::audio::{AudioEngine, AudioConfig};
//!
//! // Create audio engine
//! let config = AudioConfig::default();
//! let engine = AudioEngine::new(config)?;
//!
//! // Load and cache a sound effect
//! let buffer_id = engine.load_sound("assets/sfx/explosion.wav")?;
//!
//! // Play the sound
//! let handle = engine.play_sound(buffer_id)?;
//!
//! // Play music (streaming)
//! let music_handle = engine.play_music("assets/music/theme.mp3")?;
//! ```

use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::audio_resource::{
    AudioBufferCache, AudioCategory, AudioControls, AudioHandle, AudioSource, BufferId,
    HandleGenerator, VolumeSettings,
};
use crate::audio_spatial::{
    AudioEnvironment, EnvironmentParams, SoundSourceData, SpatialAudioProcessor, SpatialParams,
};

// Re-export commonly used types from submodules
pub use crate::audio_resource::{
    AudioBuffer as CachedBuffer, AudioCategory as SoundCategory, AudioHandle as SoundHandle,
    BufferId as SoundBufferId,
};
pub use crate::audio_spatial::{
    AttenuationModel as DistanceModel, AudioEnvironment as Environment, ListenerData as Listener,
};

/// Maximum number of simultaneous audio sinks.
pub const MAX_SINKS: usize = 32;

/// Maximum number of cached audio buffers.
pub const MAX_CACHED_BUFFERS: usize = 256;

/// Default sample rate for audio processing.
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;

/// Default number of channels (stereo).
pub const DEFAULT_CHANNELS: u16 = 2;

/// Audio engine error types.
#[derive(Debug, Error)]
pub enum AudioError {
    /// Failed to initialize audio device.
    #[error("Failed to initialize audio device: {0}")]
    DeviceInitFailed(String),

    /// No audio device available.
    #[error("No audio device available")]
    NoDevice,

    /// Failed to create audio sink.
    #[error("Failed to create audio sink: {0}")]
    SinkCreationFailed(String),

    /// Failed to load audio file.
    #[error("Failed to load audio file '{path}': {message}")]
    LoadFailed {
        /// Path to the file that failed to load.
        path: PathBuf,
        /// Error message.
        message: String,
    },

    /// Failed to decode audio data.
    #[error("Failed to decode audio: {0}")]
    DecodeFailed(String),

    /// No free sinks available.
    #[error("No free audio sinks available (max: {max})")]
    NoFreeSinks {
        /// Maximum number of sinks.
        max: usize,
    },

    /// Invalid audio handle.
    #[error("Invalid audio handle")]
    InvalidHandle,

    /// Buffer not found.
    #[error("Audio buffer not found: {0:?}")]
    BufferNotFound(BufferId),

    /// Audio engine not initialized.
    #[error("Audio engine not initialized")]
    NotInitialized,

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for audio operations.
pub type AudioResult<T> = Result<T, AudioError>;

/// Audio engine configuration.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Maximum number of simultaneous sounds.
    pub max_sounds: usize,
    /// Default volume (0.0-1.0).
    pub default_volume: f32,
    /// Enable spatial audio processing.
    pub spatial_enabled: bool,
    /// Enable Doppler effect.
    pub doppler_enabled: bool,
    /// Doppler effect strength (0.0-2.0).
    pub doppler_factor: f32,
    /// Default crossfade duration for music.
    pub crossfade_duration: Duration,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            max_sounds: MAX_SINKS,
            default_volume: 1.0,
            spatial_enabled: true,
            doppler_enabled: true,
            doppler_factor: 1.0,
            crossfade_duration: Duration::from_millis(500),
        }
    }
}

impl AudioConfig {
    /// Create config with max sounds.
    #[must_use]
    pub const fn with_max_sounds(mut self, max: usize) -> Self {
        self.max_sounds = max;
        self
    }

    /// Create config with default volume.
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.default_volume = volume;
        self
    }

    /// Create config with spatial audio enabled/disabled.
    #[must_use]
    pub const fn with_spatial(mut self, enabled: bool) -> Self {
        self.spatial_enabled = enabled;
        self
    }
}

/// Wraps rodio's output stream for audio playback.
///
/// This struct manages the audio device connection and provides
/// the foundation for all audio playback.
pub struct AudioDevice {
    /// The output stream (must be kept alive).
    _stream: OutputStream,
    /// Handle for creating sinks.
    handle: OutputStreamHandle,
    /// Whether the device is active.
    active: AtomicBool,
}

impl std::fmt::Debug for AudioDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDevice")
            .field("active", &self.active.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl AudioDevice {
    /// Create a new audio device using the default output.
    pub fn new() -> AudioResult<Self> {
        let (stream, handle) =
            OutputStream::try_default().map_err(|e| AudioError::DeviceInitFailed(e.to_string()))?;

        info!("Audio device initialized");

        Ok(Self {
            _stream: stream,
            handle,
            active: AtomicBool::new(true),
        })
    }

    /// Get a reference to the output stream handle.
    #[must_use]
    pub fn handle(&self) -> &OutputStreamHandle {
        &self.handle
    }

    /// Check if the device is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Create a new sink for audio playback.
    pub fn create_sink(&self) -> AudioResult<Sink> {
        Sink::try_new(&self.handle).map_err(|e| AudioError::SinkCreationFailed(e.to_string()))
    }
}

/// State of an audio sink in the pool.
pub(crate) struct SinkState {
    /// The rodio sink.
    sink: Sink,
    /// Associated audio handle.
    handle: Option<AudioHandle>,
    /// Whether the sink is in use.
    in_use: bool,
    /// Volume multiplier.
    volume: f32,
    /// Speed multiplier.
    speed: f32,
    /// Category for volume mixing.
    category: AudioCategory,
}

impl std::fmt::Debug for SinkState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkState")
            .field("handle", &self.handle)
            .field("in_use", &self.in_use)
            .field("volume", &self.volume)
            .field("speed", &self.speed)
            .field("category", &self.category)
            .finish_non_exhaustive()
    }
}

impl SinkState {
    fn new(sink: Sink) -> Self {
        Self {
            sink,
            handle: None,
            in_use: false,
            volume: 1.0,
            speed: 1.0,
            category: AudioCategory::Sfx,
        }
    }

    fn is_available(&self) -> bool {
        !self.in_use || self.sink.empty()
    }

    fn reset(&mut self) {
        self.handle = None;
        self.in_use = false;
        self.volume = 1.0;
        self.speed = 1.0;
        self.category = AudioCategory::Sfx;
        self.sink.set_volume(1.0);
        self.sink.set_speed(1.0);
    }
}

/// Pool of audio sinks for managing multiple simultaneous sounds.
#[derive(Debug)]
pub struct AudioSinkPool {
    /// Sink states.
    sinks: Vec<Mutex<SinkState>>,
    /// Number of active sinks.
    active_count: AtomicU32,
}

impl AudioSinkPool {
    /// Create a new sink pool.
    pub fn new(device: &AudioDevice, capacity: usize) -> AudioResult<Self> {
        let mut sinks = Vec::with_capacity(capacity);

        for i in 0..capacity {
            match device.create_sink() {
                Ok(sink) => {
                    sinks.push(Mutex::new(SinkState::new(sink)));
                },
                Err(e) => {
                    if i == 0 {
                        // Need at least one sink
                        return Err(e);
                    }
                    warn!("Could only create {} audio sinks", i);
                    break;
                },
            }
        }

        debug!("Created audio sink pool with {} sinks", sinks.len());

        Ok(Self {
            sinks,
            active_count: AtomicU32::new(0),
        })
    }

    /// Get the pool capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.sinks.len()
    }

    /// Get the number of active sinks.
    #[must_use]
    pub fn active_count(&self) -> u32 {
        self.active_count.load(Ordering::Relaxed)
    }

    /// Acquire a free sink for playback.
    pub fn acquire(&self, handle: AudioHandle, category: AudioCategory) -> Option<usize> {
        // First pass: look for completely free sinks
        for (idx, sink_mutex) in self.sinks.iter().enumerate() {
            let mut state = sink_mutex.lock();
            if state.is_available() {
                state.reset();
                state.handle = Some(handle);
                state.in_use = true;
                state.category = category;
                self.active_count.fetch_add(1, Ordering::Relaxed);
                return Some(idx);
            }
        }

        // Second pass: look for sinks with lower priority that are empty
        for (idx, sink_mutex) in self.sinks.iter().enumerate() {
            let mut state = sink_mutex.lock();
            if state.sink.empty() {
                state.reset();
                state.handle = Some(handle);
                state.in_use = true;
                state.category = category;
                // Don't increment - we're reusing
                return Some(idx);
            }
        }

        None
    }

    /// Release a sink back to the pool.
    pub fn release(&self, handle: AudioHandle) {
        for sink_mutex in &self.sinks {
            let mut state = sink_mutex.lock();
            if state.handle == Some(handle) {
                state.sink.stop();
                state.reset();
                self.active_count.fetch_sub(1, Ordering::Relaxed);
                return;
            }
        }
    }

    /// Get the sink index for a handle.
    pub fn find_sink(&self, handle: AudioHandle) -> Option<usize> {
        for (idx, sink_mutex) in self.sinks.iter().enumerate() {
            let state = sink_mutex.lock();
            if state.handle == Some(handle) {
                return Some(idx);
            }
        }
        None
    }

    /// Apply an operation to a sink by handle.
    pub(crate) fn with_sink<F, R>(&self, handle: AudioHandle, f: F) -> Option<R>
    where
        F: FnOnce(&mut SinkState) -> R,
    {
        for sink_mutex in &self.sinks {
            let mut state = sink_mutex.lock();
            if state.handle == Some(handle) {
                return Some(f(&mut state));
            }
        }
        None
    }

    /// Apply an operation to a sink by index.
    pub(crate) fn with_sink_idx<F, R>(&self, idx: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut SinkState) -> R,
    {
        self.sinks.get(idx).map(|sink_mutex| {
            let mut state = sink_mutex.lock();
            f(&mut state)
        })
    }

    /// Update all sinks (clean up finished ones).
    pub fn update(&self) {
        for sink_mutex in &self.sinks {
            let mut state = sink_mutex.lock();
            if state.in_use && state.sink.empty() {
                state.reset();
                self.active_count.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    /// Stop all playing sounds.
    pub fn stop_all(&self) {
        for sink_mutex in &self.sinks {
            let mut state = sink_mutex.lock();
            if state.in_use {
                state.sink.stop();
                state.reset();
            }
        }
        self.active_count.store(0, Ordering::Relaxed);
    }

    /// Pause all playing sounds.
    pub fn pause_all(&self) {
        for sink_mutex in &self.sinks {
            let state = sink_mutex.lock();
            if state.in_use {
                state.sink.pause();
            }
        }
    }

    /// Resume all paused sounds.
    pub fn resume_all(&self) {
        for sink_mutex in &self.sinks {
            let state = sink_mutex.lock();
            if state.in_use {
                state.sink.play();
            }
        }
    }
}

/// Playing sound tracking data.
#[derive(Debug)]
struct PlayingSoundData {
    /// Sink index in the pool.
    sink_idx: usize,
    /// Audio source info.
    source: AudioSource,
    /// Whether spatial audio is applied.
    is_spatial: bool,
    /// World position (if spatial).
    world_position: Option<(f32, f32)>,
    /// Calculated spatial parameters.
    spatial_params: Option<SpatialParams>,
}

/// The main audio engine.
///
/// Provides a high-level interface for audio playback including:
/// - Sound effect playback with caching
/// - Music streaming with crossfading
/// - Spatial audio positioning
/// - Volume mixing by category
#[derive(Debug)]
pub struct AudioEngine {
    /// Audio device (must be kept alive for audio to work).
    #[allow(dead_code)]
    device: AudioDevice,
    /// Sink pool.
    pool: AudioSinkPool,
    /// Buffer cache.
    cache: RwLock<AudioBufferCache>,
    /// Handle generator.
    handle_gen: HandleGenerator,
    /// Playing sounds.
    playing: RwLock<HashMap<AudioHandle, PlayingSoundData>>,
    /// Volume settings.
    volumes: RwLock<VolumeSettings>,
    /// Spatial audio processor.
    spatial: RwLock<SpatialAudioProcessor>,
    /// Configuration.
    config: AudioConfig,
    /// Whether the engine is initialized.
    initialized: AtomicBool,
}

impl AudioEngine {
    /// Create a new audio engine with default configuration.
    pub fn new_default() -> AudioResult<Self> {
        Self::new(AudioConfig::default())
    }

    /// Create a new audio engine with the given configuration.
    pub fn new(config: AudioConfig) -> AudioResult<Self> {
        let device = AudioDevice::new()?;
        let pool = AudioSinkPool::new(&device, config.max_sounds)?;

        let mut spatial = SpatialAudioProcessor::new();
        spatial.set_doppler_enabled(config.doppler_enabled);
        spatial.set_doppler_factor(config.doppler_factor);

        info!("Audio engine initialized with {} sinks", pool.capacity());

        Ok(Self {
            device,
            pool,
            cache: RwLock::new(AudioBufferCache::new()),
            handle_gen: HandleGenerator::new(),
            playing: RwLock::new(HashMap::new()),
            volumes: RwLock::new(VolumeSettings::default()),
            spatial: RwLock::new(spatial),
            config,
            initialized: AtomicBool::new(true),
        })
    }

    /// Check if the engine is initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &AudioConfig {
        &self.config
    }

    // ============================================
    // Buffer/Sound Loading
    // ============================================

    /// Load and cache a sound file.
    ///
    /// The sound will be decoded and stored in memory for quick playback.
    /// Returns a buffer ID that can be used to play the sound.
    pub fn load_sound(&self, path: impl AsRef<Path>) -> AudioResult<BufferId> {
        let path = path.as_ref();

        // Check if already cached
        {
            let cache = self.cache.read();
            if let Some(buffer) = cache.get_by_path(path) {
                return Ok(buffer.id);
            }
        }

        // Load and decode the file
        let file = std::fs::File::open(path).map_err(|e| AudioError::LoadFailed {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| AudioError::LoadFailed {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        // Get audio info
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();

        // Collect samples
        let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

        // Cache the buffer
        let mut cache = self.cache.write();
        let id = cache
            .add_with_path(samples, sample_rate, channels, path)
            .ok_or_else(|| AudioError::LoadFailed {
                path: path.to_path_buf(),
                message: "Buffer cache full".to_string(),
            })?;

        debug!("Loaded sound: {:?} -> {:?}", path, id);
        Ok(id)
    }

    /// Load a sound from memory.
    pub fn load_sound_from_memory(&self, data: &[u8], name: Option<&str>) -> AudioResult<BufferId> {
        let cursor = Cursor::new(data.to_vec());
        let decoder = Decoder::new(cursor).map_err(|e| AudioError::DecodeFailed(e.to_string()))?;

        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();
        let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

        let mut cache = self.cache.write();
        let id = if let Some(name) = name {
            cache.add_with_path(samples, sample_rate, channels, name)
        } else {
            cache.add(samples, sample_rate, channels)
        };

        id.ok_or_else(|| AudioError::DecodeFailed("Buffer cache full".to_string()))
    }

    /// Unload a cached sound.
    pub fn unload_sound(&self, id: BufferId) {
        let mut cache = self.cache.write();
        cache.remove(id);
        debug!("Unloaded sound: {:?}", id);
    }

    /// Clear all cached sounds.
    pub fn clear_cache(&self) {
        self.cache.write().clear();
        info!("Cleared audio cache");
    }

    /// Get cache statistics.
    #[must_use]
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read();
        (cache.count(), cache.total_size())
    }

    // ============================================
    // Playback
    // ============================================

    /// Play a cached sound effect.
    pub fn play_sound(&self, buffer_id: BufferId) -> AudioResult<AudioHandle> {
        self.play_sound_with_controls(buffer_id, AudioControls::default())
    }

    /// Play a cached sound with specific controls.
    pub fn play_sound_with_controls(
        &self,
        buffer_id: BufferId,
        controls: AudioControls,
    ) -> AudioResult<AudioHandle> {
        let cache = self.cache.read();
        let buffer = cache
            .get(buffer_id)
            .ok_or(AudioError::BufferNotFound(buffer_id))?;

        let handle = self.handle_gen.next();
        let category = AudioCategory::Sfx;

        // Acquire a sink
        let sink_idx = self
            .pool
            .acquire(handle, category)
            .ok_or(AudioError::NoFreeSinks {
                max: self.pool.capacity(),
            })?;

        // Create source from cached samples
        let source = rodio::buffer::SamplesBuffer::new(
            buffer.channels,
            buffer.sample_rate,
            (*buffer.samples).clone(),
        );

        // Apply controls and play
        self.pool.with_sink_idx(sink_idx, |state| {
            state.volume = controls.volume;
            state.speed = controls.speed;
            state.category = category;

            let effective_vol = self.effective_volume(category) * controls.volume;
            state.sink.set_volume(effective_vol);
            state.sink.set_speed(controls.speed);

            if controls.looping {
                state.sink.append(source.repeat_infinite());
            } else {
                state.sink.append(source);
            }
            state.sink.play();
        });

        // Track the playing sound
        let source_info = AudioSource::buffered(buffer_id)
            .with_controls(controls)
            .with_category(category);

        self.playing.write().insert(
            handle,
            PlayingSoundData {
                sink_idx,
                source: source_info,
                is_spatial: false,
                world_position: None,
                spatial_params: None,
            },
        );

        debug!("Playing sound {:?} on sink {}", buffer_id, sink_idx);
        Ok(handle)
    }

    /// Play a spatial sound at a world position.
    pub fn play_sound_at(&self, buffer_id: BufferId, x: f32, y: f32) -> AudioResult<AudioHandle> {
        self.play_sound_at_with_controls(buffer_id, x, y, AudioControls::default())
    }

    /// Play a spatial sound with controls.
    pub fn play_sound_at_with_controls(
        &self,
        buffer_id: BufferId,
        x: f32,
        y: f32,
        controls: AudioControls,
    ) -> AudioResult<AudioHandle> {
        if !self.config.spatial_enabled {
            return self.play_sound_with_controls(buffer_id, controls);
        }

        let cache = self.cache.read();
        let buffer = cache
            .get(buffer_id)
            .ok_or(AudioError::BufferNotFound(buffer_id))?;

        // Calculate spatial parameters
        let source_data = SoundSourceData::new(x, y).with_volume(controls.volume);
        let spatial_params = self.spatial.read().calculate(&source_data);

        if !spatial_params.audible {
            debug!(
                "Spatial sound {:?} not audible at ({}, {})",
                buffer_id, x, y
            );
            return Ok(AudioHandle::null());
        }

        let handle = self.handle_gen.next();
        let category = AudioCategory::Sfx;

        let sink_idx = self
            .pool
            .acquire(handle, category)
            .ok_or(AudioError::NoFreeSinks {
                max: self.pool.capacity(),
            })?;

        // Create source
        let source = rodio::buffer::SamplesBuffer::new(
            buffer.channels,
            buffer.sample_rate,
            (*buffer.samples).clone(),
        );

        // Apply spatial parameters
        self.pool.with_sink_idx(sink_idx, |state| {
            state.volume = spatial_params.mono_volume;
            state.speed = spatial_params.pitch * controls.speed;
            state.category = category;

            let effective_vol = self.effective_volume(category) * spatial_params.mono_volume;
            state.sink.set_volume(effective_vol);
            state.sink.set_speed(spatial_params.pitch * controls.speed);

            if controls.looping {
                state.sink.append(source.repeat_infinite());
            } else {
                state.sink.append(source);
            }
            state.sink.play();
        });

        // Track the playing sound
        let source_info = AudioSource::buffered(buffer_id)
            .with_controls(controls)
            .with_category(category);

        self.playing.write().insert(
            handle,
            PlayingSoundData {
                sink_idx,
                source: source_info,
                is_spatial: true,
                world_position: Some((x, y)),
                spatial_params: Some(spatial_params),
            },
        );

        debug!("Playing spatial sound {:?} at ({}, {})", buffer_id, x, y);
        Ok(handle)
    }

    /// Stream music from a file.
    pub fn play_music(&self, path: impl AsRef<Path>) -> AudioResult<AudioHandle> {
        self.play_music_with_controls(path, AudioControls::default().with_looping(true))
    }

    /// Stream music with controls.
    pub fn play_music_with_controls(
        &self,
        path: impl AsRef<Path>,
        controls: AudioControls,
    ) -> AudioResult<AudioHandle> {
        let path = path.as_ref();

        let file = std::fs::File::open(path).map_err(|e| AudioError::LoadFailed {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| AudioError::LoadFailed {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let handle = self.handle_gen.next();
        let category = AudioCategory::Music;

        let sink_idx = self
            .pool
            .acquire(handle, category)
            .ok_or(AudioError::NoFreeSinks {
                max: self.pool.capacity(),
            })?;

        self.pool.with_sink_idx(sink_idx, |state| {
            state.volume = controls.volume;
            state.speed = controls.speed;
            state.category = category;

            let effective_vol = self.effective_volume(category) * controls.volume;
            state.sink.set_volume(effective_vol);
            state.sink.set_speed(controls.speed);

            if controls.looping {
                // For streaming, we need to reload - for now just play once
                // TODO: Implement proper music looping with file reload
                state.sink.append(source.convert_samples::<f32>());
            } else {
                state.sink.append(source.convert_samples::<f32>());
            }
            state.sink.play();
        });

        let source_info = AudioSource::streaming(path)
            .with_controls(controls)
            .with_category(category);

        self.playing.write().insert(
            handle,
            PlayingSoundData {
                sink_idx,
                source: source_info,
                is_spatial: false,
                world_position: None,
                spatial_params: None,
            },
        );

        info!("Playing music: {:?}", path);
        Ok(handle)
    }

    // ============================================
    // Playback Control
    // ============================================

    /// Stop a playing sound.
    pub fn stop(&self, handle: AudioHandle) {
        if let Some(data) = self.playing.write().remove(&handle) {
            self.pool.with_sink_idx(data.sink_idx, |state| {
                state.sink.stop();
                state.reset();
            });
            debug!("Stopped sound {:?}", handle);
        }
    }

    /// Pause a playing sound.
    pub fn pause(&self, handle: AudioHandle) {
        self.pool.with_sink(handle, |state| {
            state.sink.pause();
        });
    }

    /// Resume a paused sound.
    pub fn resume(&self, handle: AudioHandle) {
        self.pool.with_sink(handle, |state| {
            state.sink.play();
        });
    }

    /// Set volume for a playing sound.
    pub fn set_volume(&self, handle: AudioHandle, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        self.pool.with_sink(handle, |state| {
            state.volume = volume;
            let effective = self.effective_volume(state.category) * volume;
            state.sink.set_volume(effective);
        });
    }

    /// Set speed for a playing sound.
    pub fn set_speed(&self, handle: AudioHandle, speed: f32) {
        let speed = speed.clamp(0.1, 4.0);
        self.pool.with_sink(handle, |state| {
            state.speed = speed;
            state.sink.set_speed(speed);
        });
    }

    /// Check if a sound is playing.
    #[must_use]
    pub fn is_playing(&self, handle: AudioHandle) -> bool {
        self.pool
            .with_sink(handle, |state| {
                !state.sink.empty() && !state.sink.is_paused()
            })
            .unwrap_or(false)
    }

    /// Check if a sound is paused.
    #[must_use]
    pub fn is_paused(&self, handle: AudioHandle) -> bool {
        self.pool
            .with_sink(handle, |state| state.sink.is_paused())
            .unwrap_or(false)
    }

    /// Stop all playing sounds.
    pub fn stop_all(&self) {
        self.pool.stop_all();
        self.playing.write().clear();
        info!("Stopped all sounds");
    }

    /// Pause all playing sounds.
    pub fn pause_all(&self) {
        self.pool.pause_all();
    }

    /// Resume all paused sounds.
    pub fn resume_all(&self) {
        self.pool.resume_all();
    }

    // ============================================
    // Volume Control
    // ============================================

    /// Set volume for a category.
    pub fn set_category_volume(&self, category: AudioCategory, volume: f32) {
        self.volumes.write().set(category, volume);

        // Update all playing sounds in this category
        for sink_mutex in &self.pool.sinks {
            let state = sink_mutex.lock();
            if state.in_use && state.category == category {
                let effective = self.effective_volume(category) * state.volume;
                state.sink.set_volume(effective);
            }
        }
    }

    /// Get volume for a category.
    #[must_use]
    pub fn category_volume(&self, category: AudioCategory) -> f32 {
        self.volumes.read().get(category)
    }

    /// Get effective volume for a category (includes master).
    #[must_use]
    pub fn effective_volume(&self, category: AudioCategory) -> f32 {
        self.volumes.read().effective(category)
    }

    /// Set master volume.
    pub fn set_master_volume(&self, volume: f32) {
        self.set_category_volume(AudioCategory::Master, volume);
    }

    /// Get master volume.
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.category_volume(AudioCategory::Master)
    }

    // ============================================
    // Spatial Audio
    // ============================================

    /// Update the listener position for spatial audio.
    pub fn set_listener_position(&self, x: f32, y: f32) {
        self.spatial.write().set_listener_position(x, y);
    }

    /// Update the listener velocity for Doppler effect.
    pub fn set_listener_velocity(&self, vx: f32, vy: f32) {
        self.spatial.write().set_listener_velocity(vx, vy);
    }

    /// Update the listener direction (will be normalized).
    pub fn set_listener_direction(&self, dx: f32, dy: f32) {
        self.spatial.write().set_listener_direction(dx, dy);
    }

    /// Set the audio environment.
    pub fn set_environment(&self, env: AudioEnvironment) {
        self.spatial.write().set_environment(env);
    }

    /// Get the current environment.
    #[must_use]
    pub fn environment(&self) -> AudioEnvironment {
        self.spatial.read().environment()
    }

    /// Get environment effect parameters.
    #[must_use]
    pub fn environment_params(&self) -> EnvironmentParams {
        self.spatial.read().environment_params()
    }

    /// Update spatial audio for all playing sounds.
    ///
    /// Call this each frame to update volume and panning for spatial sounds.
    pub fn update_spatial(&self) {
        if !self.config.spatial_enabled {
            return;
        }

        let spatial = self.spatial.read();
        let mut playing = self.playing.write();

        for (handle, data) in playing.iter_mut() {
            if !data.is_spatial {
                continue;
            }

            if let Some((x, y)) = data.world_position {
                let source_data =
                    SoundSourceData::new(x, y).with_volume(data.source.controls.volume);
                let params = spatial.calculate(&source_data);

                if !params.audible {
                    // Stop inaudible sounds
                    self.pool.with_sink(*handle, |state| {
                        state.sink.stop();
                    });
                    continue;
                }

                // Update volume and speed based on spatial params
                self.pool.with_sink(*handle, |state| {
                    let effective = self.effective_volume(state.category) * params.mono_volume;
                    state.sink.set_volume(effective);
                    state.sink.set_speed(params.pitch * state.speed);
                });

                data.spatial_params = Some(params);
            }
        }
    }

    /// Update the position of a spatial sound.
    pub fn update_sound_position(&self, handle: AudioHandle, x: f32, y: f32) {
        if let Some(data) = self.playing.write().get_mut(&handle) {
            data.world_position = Some((x, y));
        }
    }

    // ============================================
    // Update
    // ============================================

    /// Update the audio engine (call each frame).
    ///
    /// This cleans up finished sounds and updates spatial audio.
    pub fn update(&self) {
        // Clean up finished sounds
        self.pool.update();

        // Remove finished sounds from tracking
        let mut playing = self.playing.write();
        let finished: Vec<_> = playing
            .iter()
            .filter(|(_, data)| {
                self.pool
                    .with_sink_idx(data.sink_idx, |state| state.sink.empty())
                    .unwrap_or(true)
            })
            .map(|(h, _)| *h)
            .collect();

        for handle in finished {
            playing.remove(&handle);
        }
        drop(playing);

        // Update spatial audio
        self.update_spatial();
    }

    // ============================================
    // Statistics
    // ============================================

    /// Get the number of currently playing sounds.
    #[must_use]
    pub fn playing_count(&self) -> usize {
        self.pool.active_count() as usize
    }

    /// Get the maximum number of simultaneous sounds.
    #[must_use]
    pub fn max_sounds(&self) -> usize {
        self.pool.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Audio tests require actual audio device, so we test the non-device parts

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.max_sounds, MAX_SINKS);
        assert!((config.default_volume - 1.0).abs() < f32::EPSILON);
        assert!(config.spatial_enabled);
        assert!(config.doppler_enabled);
    }

    #[test]
    fn test_audio_config_builder() {
        let config = AudioConfig::default()
            .with_max_sounds(16)
            .with_volume(0.8)
            .with_spatial(false);

        assert_eq!(config.max_sounds, 16);
        assert!((config.default_volume - 0.8).abs() < f32::EPSILON);
        assert!(!config.spatial_enabled);
    }

    #[test]
    fn test_audio_error_display() {
        let err = AudioError::NoDevice;
        assert!(err.to_string().contains("No audio device"));

        let err = AudioError::BufferNotFound(BufferId::new(42));
        assert!(err.to_string().contains("42"));
    }

    #[test]
    fn test_handle_generator() {
        let gen = HandleGenerator::new();
        let h1 = gen.next();
        let h2 = gen.next();
        assert_ne!(h1.id(), h2.id());
    }

    // Integration tests would require mocking the audio device
    // or running on a system with audio hardware
}
