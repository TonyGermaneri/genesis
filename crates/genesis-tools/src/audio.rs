//! Audio engine integration using rodio.
//!
//! This module provides:
//! - Cross-platform audio playback
//! - Spatial audio positioning
//! - Separate volume controls (master, SFX, music)
//! - Music crossfading
//! - Sound effect pooling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

// Re-export rodio types for integration
pub use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

/// Unique identifier for audio sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioSourceId(u64);

impl AudioSourceId {
    /// Creates a new audio source ID.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

static NEXT_AUDIO_ID: AtomicU64 = AtomicU64::new(1);

impl AudioSourceId {
    fn next() -> Self {
        Self(NEXT_AUDIO_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Unique identifier for sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SoundId(u64);

impl SoundId {
    /// Creates a new sound ID.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// A sound effect that can be played.
#[derive(Debug, Clone)]
pub struct SoundEffect {
    /// Unique identifier
    pub id: SoundId,
    /// Raw audio data (WAV/OGG bytes)
    pub data: Vec<u8>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Base volume (0.0 - 1.0)
    pub volume: f32,
    /// Whether this sound loops
    pub looping: bool,
}

impl SoundEffect {
    /// Creates a new sound effect.
    #[must_use]
    pub fn new(id: SoundId, data: Vec<u8>, sample_rate: u32, channels: u16) -> Self {
        Self {
            id,
            data,
            sample_rate,
            channels,
            volume: 1.0,
            looping: false,
        }
    }

    /// Sets the volume.
    #[must_use]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Sets whether the sound loops.
    #[must_use]
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

/// Spatial audio position and velocity.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpatialPosition {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Z position (optional, for 3D)
    pub z: f32,
}

impl SpatialPosition {
    /// Creates a new 2D position.
    #[must_use]
    pub fn new_2d(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }

    /// Creates a new 3D position.
    #[must_use]
    pub fn new_3d(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculates distance to another position.
    #[must_use]
    pub fn distance_to(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Spatial audio manager data (from kernel).
#[derive(Debug, Clone, Default)]
pub struct SpatialAudioManager {
    /// Listener position
    pub listener_position: SpatialPosition,
    /// Listener forward direction
    pub listener_forward: (f32, f32, f32),
    /// Active spatial sources
    pub sources: HashMap<AudioSourceId, SpatialPosition>,
    /// Maximum hearing distance
    pub max_distance: f32,
    /// Rolloff factor for distance attenuation
    pub rolloff_factor: f32,
}

impl SpatialAudioManager {
    /// Creates a new spatial audio manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            listener_position: SpatialPosition::default(),
            listener_forward: (0.0, 0.0, -1.0),
            sources: HashMap::new(),
            max_distance: 100.0,
            rolloff_factor: 1.0,
        }
    }

    /// Sets the listener position.
    pub fn set_listener_position(&mut self, pos: SpatialPosition) {
        self.listener_position = pos;
    }

    /// Adds a spatial source.
    pub fn add_source(&mut self, id: AudioSourceId, pos: SpatialPosition) {
        self.sources.insert(id, pos);
    }

    /// Updates a source position.
    pub fn update_source(&mut self, id: AudioSourceId, pos: SpatialPosition) {
        if let Some(source) = self.sources.get_mut(&id) {
            *source = pos;
        }
    }

    /// Removes a spatial source.
    pub fn remove_source(&mut self, id: AudioSourceId) {
        self.sources.remove(&id);
    }

    /// Calculates volume attenuation for a source.
    #[must_use]
    pub fn calculate_attenuation(&self, source_id: AudioSourceId) -> f32 {
        if let Some(pos) = self.sources.get(&source_id) {
            let distance = self.listener_position.distance_to(pos);
            if distance >= self.max_distance {
                0.0
            } else {
                let attenuation = 1.0 / (1.0 + self.rolloff_factor * distance / self.max_distance);
                attenuation.clamp(0.0, 1.0)
            }
        } else {
            1.0
        }
    }
}

/// Audio playback errors.
#[derive(Debug, Error)]
pub enum AudioError {
    /// No audio output device available.
    #[error("No audio output device available")]
    NoOutputDevice,

    /// Error decoding audio data.
    #[error("Audio decoding error: {0}")]
    DecodingError(String),

    /// Error during playback.
    #[error("Playback error: {0}")]
    PlaybackError(String),

    /// Audio stream error.
    #[error("Stream error: {0}")]
    StreamError(String),
}

/// Result type for audio operations.
pub type AudioResult<T> = Result<T, AudioError>;

/// Music playback state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicState {
    /// No music playing
    Stopped,
    /// Music is playing
    Playing,
    /// Fading in with target volume and remaining time in seconds
    FadingIn {
        /// Target volume to fade to
        target_volume: f32,
        /// Remaining fade time in seconds
        remaining: f32,
    },
    /// Fading out with remaining time in seconds
    FadingOut {
        /// Remaining fade time in seconds
        remaining: f32,
    },
    /// Paused
    Paused,
}

impl Default for MusicState {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Audio configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume (0.0 - 1.0)
    pub master_volume: f32,
    /// Sound effects volume (0.0 - 1.0)
    pub sfx_volume: f32,
    /// Music volume (0.0 - 1.0)
    pub music_volume: f32,
    /// Whether audio is muted
    pub muted: bool,
    /// Maximum simultaneous sounds
    pub max_sounds: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 0.7,
            muted: false,
            max_sounds: 32,
        }
    }
}

/// State of an active sound.
struct ActiveSound {
    /// The sink playing this sound
    sink: Sink,
    /// Whether this is a spatial sound
    is_spatial: bool,
    /// Base volume before spatial attenuation
    base_volume: f32,
}

/// Audio engine for sound playback.
pub struct AudioEngine {
    /// Audio output stream (must be kept alive)
    _stream: OutputStream,
    /// Stream handle for creating sinks
    stream_handle: OutputStreamHandle,
    /// Active sound effects
    sinks: HashMap<AudioSourceId, ActiveSound>,
    /// Music sink
    music_sink: Option<Sink>,
    /// Music state
    music_state: MusicState,
    /// Configuration
    config: AudioConfig,
    /// Spatial audio manager
    spatial: SpatialAudioManager,
}

impl fmt::Debug for AudioEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioEngine")
            .field("active_sounds", &self.sinks.len())
            .field("music_state", &self.music_state)
            .field("config", &self.config)
            .field("spatial", &self.spatial)
            .finish_non_exhaustive()
    }
}

impl AudioEngine {
    /// Creates a new audio engine.
    ///
    /// # Errors
    ///
    /// Returns an error if no audio output device is available.
    pub fn new() -> AudioResult<Self> {
        let (stream, stream_handle) =
            OutputStream::try_default().map_err(|_e| AudioError::NoOutputDevice)?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sinks: HashMap::new(),
            music_sink: None,
            music_state: MusicState::Stopped,
            config: AudioConfig::default(),
            spatial: SpatialAudioManager::new(),
        })
    }

    /// Creates with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if no audio output device is available.
    pub fn with_config(config: AudioConfig) -> AudioResult<Self> {
        let mut engine = Self::new()?;
        engine.config = config;
        Ok(engine)
    }

    /// Plays a sound effect.
    ///
    /// # Arguments
    ///
    /// * `sound` - The sound effect to play
    /// * `position` - Optional spatial position (None for non-spatial)
    ///
    /// Returns the audio source ID for later control.
    pub fn play_sound(
        &mut self,
        sound: &SoundEffect,
        position: Option<(f32, f32)>,
    ) -> AudioSourceId {
        let id = AudioSourceId::next();

        // Check if we've hit the max sounds limit
        if self.sinks.len() >= self.config.max_sounds {
            self.cleanup_finished();
        }

        // Create sink
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            // Calculate volume
            let spatial_attenuation = if let Some((x, y)) = position {
                self.spatial.add_source(id, SpatialPosition::new_2d(x, y));
                self.spatial.calculate_attenuation(id)
            } else {
                1.0
            };

            let volume = self.calculate_sfx_volume(sound.volume * spatial_attenuation);
            sink.set_volume(volume);

            // Try to decode and play
            let cursor = Cursor::new(sound.data.clone());
            if let Ok(source) = rodio::Decoder::new(cursor) {
                if sound.looping {
                    sink.append(source.repeat_infinite());
                } else {
                    sink.append(source);
                }

                self.sinks.insert(
                    id,
                    ActiveSound {
                        sink,
                        is_spatial: position.is_some(),
                        base_volume: sound.volume,
                    },
                );
            }
        }

        id
    }

    /// Plays music with optional fade-in.
    ///
    /// # Arguments
    ///
    /// * `music` - The music to play
    /// * `fade_in` - Fade-in duration in seconds (0 for instant)
    pub fn play_music(&mut self, music: &SoundEffect, fade_in: f32) {
        // Stop any existing music
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }

        // Create new music sink
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            let cursor = Cursor::new(music.data.clone());
            if let Ok(source) = rodio::Decoder::new(cursor) {
                // Always loop music
                sink.append(source.repeat_infinite());

                if fade_in > 0.0 {
                    sink.set_volume(0.0);
                    self.music_state = MusicState::FadingIn {
                        target_volume: self.calculate_music_volume(music.volume),
                        remaining: fade_in,
                    };
                } else {
                    sink.set_volume(self.calculate_music_volume(music.volume));
                    self.music_state = MusicState::Playing;
                }

                self.music_sink = Some(sink);
            }
        }
    }

    /// Stops music with optional fade-out.
    ///
    /// # Arguments
    ///
    /// * `fade_out` - Fade-out duration in seconds (0 for instant)
    pub fn stop_music(&mut self, fade_out: f32) {
        if fade_out > 0.0 {
            self.music_state = MusicState::FadingOut {
                remaining: fade_out,
            };
        } else {
            if let Some(sink) = self.music_sink.take() {
                sink.stop();
            }
            self.music_state = MusicState::Stopped;
        }
    }

    /// Pauses the music.
    pub fn pause_music(&mut self) {
        if let Some(sink) = &self.music_sink {
            sink.pause();
            self.music_state = MusicState::Paused;
        }
    }

    /// Resumes paused music.
    pub fn resume_music(&mut self) {
        if let Some(sink) = &self.music_sink {
            sink.play();
            self.music_state = MusicState::Playing;
        }
    }

    /// Updates spatial audio for all sources.
    pub fn update_spatial(&mut self, spatial: &SpatialAudioManager) {
        self.spatial = spatial.clone();

        // Pre-calculate values to avoid borrow conflicts
        let muted = self.config.muted;
        let sfx_volume = self.config.sfx_volume;
        let master_volume = self.config.master_volume;

        // Update volumes for spatial sounds
        for (id, active) in &mut self.sinks {
            if active.is_spatial {
                let attenuation = self.spatial.calculate_attenuation(*id);
                let base = active.base_volume * attenuation;
                let volume = if muted {
                    0.0
                } else {
                    base * sfx_volume * master_volume
                };
                active.sink.set_volume(volume);
            }
        }
    }

    /// Updates the audio engine (call once per frame).
    pub fn update(&mut self, dt: f32) {
        // Handle music fading
        match self.music_state {
            MusicState::FadingIn {
                target_volume,
                remaining,
            } => {
                let new_remaining = remaining - dt;
                if new_remaining <= 0.0 {
                    if let Some(sink) = &self.music_sink {
                        sink.set_volume(target_volume);
                    }
                    self.music_state = MusicState::Playing;
                } else {
                    let progress = 1.0 - (new_remaining / remaining);
                    if let Some(sink) = &self.music_sink {
                        sink.set_volume(target_volume * progress);
                    }
                    self.music_state = MusicState::FadingIn {
                        target_volume,
                        remaining: new_remaining,
                    };
                }
            },
            MusicState::FadingOut { remaining } => {
                let new_remaining = remaining - dt;
                if new_remaining <= 0.0 {
                    if let Some(sink) = self.music_sink.take() {
                        sink.stop();
                    }
                    self.music_state = MusicState::Stopped;
                } else {
                    let current_volume = if let Some(sink) = &self.music_sink {
                        sink.volume()
                    } else {
                        0.0
                    };
                    let new_volume = current_volume * (new_remaining / remaining);
                    if let Some(sink) = &self.music_sink {
                        sink.set_volume(new_volume);
                    }
                    self.music_state = MusicState::FadingOut {
                        remaining: new_remaining,
                    };
                }
            },
            _ => {},
        }

        // Cleanup finished sounds
        self.cleanup_finished();
    }

    /// Sets the master volume.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.config.master_volume = volume.clamp(0.0, 1.0);
        self.update_all_volumes();
    }

    /// Sets the sound effects volume.
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.config.sfx_volume = volume.clamp(0.0, 1.0);
        self.update_all_volumes();
    }

    /// Sets the music volume.
    pub fn set_music_volume(&mut self, volume: f32) {
        self.config.music_volume = volume.clamp(0.0, 1.0);
        self.update_all_volumes();
    }

    /// Sets whether audio is muted.
    pub fn set_muted(&mut self, muted: bool) {
        self.config.muted = muted;
        self.update_all_volumes();
    }

    /// Toggles mute state.
    pub fn toggle_mute(&mut self) {
        self.set_muted(!self.config.muted);
    }

    /// Returns the current configuration.
    #[must_use]
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Stops a specific sound.
    pub fn stop_sound(&mut self, id: AudioSourceId) {
        if let Some(active) = self.sinks.remove(&id) {
            active.sink.stop();
            self.spatial.remove_source(id);
        }
    }

    /// Pauses a specific sound.
    pub fn pause_sound(&mut self, id: AudioSourceId) {
        if let Some(active) = self.sinks.get(&id) {
            active.sink.pause();
        }
    }

    /// Resumes a paused sound.
    pub fn resume_sound(&mut self, id: AudioSourceId) {
        if let Some(active) = self.sinks.get(&id) {
            active.sink.play();
        }
    }

    /// Stops all sounds and music.
    pub fn stop_all(&mut self) {
        for (id, active) in self.sinks.drain() {
            active.sink.stop();
            self.spatial.remove_source(id);
        }

        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
        self.music_state = MusicState::Stopped;
    }

    /// Returns the number of active sounds.
    #[must_use]
    pub fn active_sound_count(&self) -> usize {
        self.sinks.len()
    }

    /// Returns whether music is currently playing.
    #[must_use]
    pub fn is_music_playing(&self) -> bool {
        matches!(
            self.music_state,
            MusicState::Playing | MusicState::FadingIn { .. }
        )
    }

    /// Returns the music state.
    #[must_use]
    pub fn music_state(&self) -> MusicState {
        self.music_state
    }

    fn calculate_sfx_volume(&self, base: f32) -> f32 {
        if self.config.muted {
            0.0
        } else {
            base * self.config.sfx_volume * self.config.master_volume
        }
    }

    fn calculate_music_volume(&self, base: f32) -> f32 {
        if self.config.muted {
            0.0
        } else {
            base * self.config.music_volume * self.config.master_volume
        }
    }

    fn update_all_volumes(&mut self) {
        // Pre-calculate values to avoid borrow conflicts
        let muted = self.config.muted;
        let sfx_volume = self.config.sfx_volume;
        let master_volume = self.config.master_volume;
        let music_volume = self.config.music_volume;

        // Update SFX volumes
        for (id, active) in &mut self.sinks {
            let attenuation = if active.is_spatial {
                self.spatial.calculate_attenuation(*id)
            } else {
                1.0
            };
            let base = active.base_volume * attenuation;
            let volume = if muted {
                0.0
            } else {
                base * sfx_volume * master_volume
            };
            active.sink.set_volume(volume);
        }

        // Update music volume
        if let Some(sink) = &self.music_sink {
            if self.music_state == MusicState::Playing {
                let vol = if muted {
                    0.0
                } else {
                    music_volume * master_volume
                };
                sink.set_volume(vol);
            }
        }
    }

    fn cleanup_finished(&mut self) {
        let finished: Vec<AudioSourceId> = self
            .sinks
            .iter()
            .filter(|(_, active)| active.sink.empty())
            .map(|(id, _)| *id)
            .collect();

        for id in finished {
            self.sinks.remove(&id);
            self.spatial.remove_source(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_source_id() {
        let id = AudioSourceId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_sound_id() {
        let id = SoundId::new(123);
        assert_eq!(id.raw(), 123);
    }

    #[test]
    fn test_sound_effect_new() {
        let sound = SoundEffect::new(SoundId::new(1), vec![0, 1, 2], 44100, 2);
        assert_eq!(sound.sample_rate, 44100);
        assert_eq!(sound.channels, 2);
        assert!((sound.volume - 1.0).abs() < 0.001);
        assert!(!sound.looping);
    }

    #[test]
    fn test_sound_effect_with_volume() {
        let sound = SoundEffect::new(SoundId::new(1), vec![], 44100, 2).with_volume(0.5);
        assert!((sound.volume - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_sound_effect_with_looping() {
        let sound = SoundEffect::new(SoundId::new(1), vec![], 44100, 2).with_looping(true);
        assert!(sound.looping);
    }

    #[test]
    fn test_spatial_position_new_2d() {
        let pos = SpatialPosition::new_2d(10.0, 20.0);
        assert!((pos.x - 10.0).abs() < 0.001);
        assert!((pos.y - 20.0).abs() < 0.001);
        assert!((pos.z - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_position_new_3d() {
        let pos = SpatialPosition::new_3d(10.0, 20.0, 30.0);
        assert!((pos.x - 10.0).abs() < 0.001);
        assert!((pos.y - 20.0).abs() < 0.001);
        assert!((pos.z - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_position_distance() {
        let pos1 = SpatialPosition::new_2d(0.0, 0.0);
        let pos2 = SpatialPosition::new_2d(3.0, 4.0);
        let dist = pos1.distance_to(&pos2);
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_audio_manager_new() {
        let manager = SpatialAudioManager::new();
        assert!(manager.sources.is_empty());
        assert!((manager.max_distance - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_audio_manager_add_remove_source() {
        let mut manager = SpatialAudioManager::new();
        let id = AudioSourceId::new(1);
        let pos = SpatialPosition::new_2d(10.0, 20.0);

        manager.add_source(id, pos);
        assert_eq!(manager.sources.len(), 1);

        manager.remove_source(id);
        assert!(manager.sources.is_empty());
    }

    #[test]
    fn test_spatial_audio_manager_attenuation() {
        let mut manager = SpatialAudioManager::new();
        manager.set_listener_position(SpatialPosition::new_2d(0.0, 0.0));
        manager.max_distance = 100.0;
        manager.rolloff_factor = 1.0;

        // Near source
        let near_id = AudioSourceId::new(1);
        manager.add_source(near_id, SpatialPosition::new_2d(10.0, 0.0));
        let near_atten = manager.calculate_attenuation(near_id);
        assert!(near_atten > 0.5);

        // Far source (at max distance)
        let far_id = AudioSourceId::new(2);
        manager.add_source(far_id, SpatialPosition::new_2d(100.0, 0.0));
        let far_atten = manager.calculate_attenuation(far_id);
        assert!(far_atten < near_atten);

        // Beyond max distance
        let beyond_id = AudioSourceId::new(3);
        manager.add_source(beyond_id, SpatialPosition::new_2d(200.0, 0.0));
        let beyond_atten = manager.calculate_attenuation(beyond_id);
        assert!((beyond_atten - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_audio_config_defaults() {
        let config = AudioConfig::default();
        assert!((config.master_volume - 1.0).abs() < 0.001);
        assert!((config.sfx_volume - 1.0).abs() < 0.001);
        assert!((config.music_volume - 0.7).abs() < 0.001);
        assert!(!config.muted);
        assert_eq!(config.max_sounds, 32);
    }

    #[test]
    fn test_music_state_default() {
        let state = MusicState::default();
        assert_eq!(state, MusicState::Stopped);
    }

    // Note: AudioEngine tests require audio hardware, so we test the logic components above
    // Integration tests with actual audio would go in a separate test binary
}
