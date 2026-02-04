//! Audio system integration.
//!
//! This module wires up all audio components:
//! - Initialize audio device on startup
//! - Connect gameplay events to sound system
//! - Apply volume settings from UI
//! - Update spatial audio listener position
//! - Handle audio device changes

use std::collections::{HashMap, VecDeque};
use std::io::Cursor;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::audio_assets::{AudioAssetLoader, AudioCategory};
use crate::audio_state::{AmbientState, AudioState, MusicPlayState, MusicState, VolumeSettings};

/// Maximum queued sound events per frame.
pub const MAX_QUEUED_EVENTS: usize = 32;

/// Maximum simultaneous sound effects.
pub const MAX_ACTIVE_SFX: usize = 64;

/// Errors that can occur in audio integration.
#[derive(Debug, Error)]
pub enum AudioIntegrationError {
    /// No audio output device available.
    #[error("No audio output device available")]
    NoOutputDevice,

    /// Failed to create audio stream.
    #[error("Failed to create audio stream: {0}")]
    StreamError(String),

    /// Failed to decode audio.
    #[error("Failed to decode audio: {0}")]
    DecodeError(String),

    /// Asset loading error.
    #[error("Asset error: {0}")]
    AssetError(String),
}

/// Result type for audio integration operations.
pub type AudioResult<T> = Result<T, AudioIntegrationError>;

/// Unique identifier for a playing sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(u64);

impl SoundHandle {
    /// Returns the raw ID.
    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.0
    }
}

/// A queued sound event to be processed.
#[derive(Debug, Clone)]
pub struct SoundEvent {
    /// Asset category.
    pub category: AudioCategory,
    /// Asset name.
    pub name: String,
    /// Optional spatial position (x, y).
    pub position: Option<(f32, f32)>,
    /// Volume multiplier.
    pub volume: f32,
    /// Pitch multiplier.
    pub pitch: f32,
    /// Whether the sound loops.
    pub looping: bool,
}

impl SoundEvent {
    /// Creates a new sound event.
    #[must_use]
    pub fn new(category: AudioCategory, name: &str) -> Self {
        Self {
            category,
            name: name.to_string(),
            position: None,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
        }
    }

    /// Sets the spatial position.
    #[must_use]
    pub fn at_position(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    /// Sets the volume multiplier.
    #[must_use]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }

    /// Sets the pitch multiplier.
    #[must_use]
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 4.0);
        self
    }

    /// Sets whether the sound loops.
    #[must_use]
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

/// An active playing sound.
struct ActiveSound {
    /// The rodio sink.
    sink: Sink,
    /// Category of sound.
    category: AudioCategory,
    /// Base volume before spatial/category adjustments.
    base_volume: f32,
    /// Spatial position if any.
    position: Option<(f32, f32)>,
    /// Whether this is looping.
    looping: bool,
}

/// Listener position for spatial audio.
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioListener {
    /// X position in world coordinates.
    pub x: f32,
    /// Y position in world coordinates.
    pub y: f32,
    /// Maximum hearing distance.
    pub max_distance: f32,
    /// Rolloff factor for distance attenuation.
    pub rolloff: f32,
}

impl AudioListener {
    /// Creates a new listener.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            max_distance: 500.0,
            rolloff: 1.0,
        }
    }

    /// Calculates volume attenuation for a sound at position.
    #[must_use]
    pub fn calculate_attenuation(&self, sound_x: f32, sound_y: f32) -> f32 {
        let dx = sound_x - self.x;
        let dy = sound_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance >= self.max_distance {
            0.0
        } else if distance < 1.0 {
            1.0
        } else {
            let attenuation = 1.0 / (1.0 + self.rolloff * distance / self.max_distance);
            attenuation.clamp(0.0, 1.0)
        }
    }

    /// Calculates stereo panning (-1.0 = full left, 1.0 = full right).
    #[must_use]
    pub fn calculate_panning(&self, sound_x: f32, _sound_y: f32) -> f32 {
        let dx = sound_x - self.x;
        // Simple linear panning based on X offset
        (dx / self.max_distance).clamp(-1.0, 1.0)
    }
}

/// Main audio integration system.
pub struct AudioIntegration {
    /// Audio output stream (must stay alive - dropping this stops all audio).
    #[allow(dead_code)]
    output_stream: Option<OutputStream>,
    /// Stream handle for creating sinks.
    stream_handle: Option<OutputStreamHandle>,
    /// Asset loader.
    asset_loader: AudioAssetLoader,
    /// Audio state.
    state: AudioState,
    /// Listener position.
    listener: AudioListener,

    /// Active sound effects.
    active_sfx: HashMap<SoundHandle, ActiveSound>,
    /// Music sink.
    music_sink: Option<Sink>,
    /// Next music sink (for crossfade).
    next_music_sink: Option<Sink>,
    /// Ambient layer sinks.
    ambient_sinks: HashMap<String, Sink>,

    /// Event queue for deferred processing.
    event_queue: VecDeque<SoundEvent>,

    /// Next sound handle ID.
    next_handle_id: u64,

    /// Frame counter for hot-reload checks (debug builds only).
    #[cfg(debug_assertions)]
    hot_reload_counter: u32,
}

impl std::fmt::Debug for AudioIntegration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioIntegration")
            .field("initialized", &self.state.initialized)
            .field("device_available", &self.state.device_available)
            .field("active_sfx", &self.active_sfx.len())
            .field("ambient_layers", &self.ambient_sinks.len())
            .finish_non_exhaustive()
    }
}

impl AudioIntegration {
    /// Creates a new audio integration system.
    ///
    /// This initializes the audio device and asset loader.
    #[must_use]
    pub fn new(asset_base_path: &str) -> Self {
        let asset_loader = AudioAssetLoader::new(asset_base_path);
        let mut state = AudioState::new();

        // Try to initialize audio device
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((stream, handle)) => {
                info!("Audio device initialized successfully");
                state.initialized = true;
                state.device_available = true;
                (Some(stream), Some(handle))
            },
            Err(e) => {
                warn!(
                    "Failed to initialize audio device: {}. Audio will be disabled.",
                    e
                );
                state.initialized = true;
                state.device_available = false;
                (None, None)
            },
        };

        Self {
            output_stream: stream,
            stream_handle,
            asset_loader,
            state,
            listener: AudioListener::default(),
            active_sfx: HashMap::new(),
            music_sink: None,
            next_music_sink: None,
            ambient_sinks: HashMap::new(),
            event_queue: VecDeque::with_capacity(MAX_QUEUED_EVENTS),
            next_handle_id: 1,
            #[cfg(debug_assertions)]
            hot_reload_counter: 0,
        }
    }

    /// Creates with default asset path.
    #[must_use]
    pub fn with_default_assets() -> Self {
        Self::new("assets/sounds")
    }

    /// Returns whether audio is available.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.state.device_available && self.stream_handle.is_some()
    }

    /// Returns a reference to the audio state.
    #[must_use]
    pub fn state(&self) -> &AudioState {
        &self.state
    }

    /// Returns a mutable reference to the audio state.
    pub fn state_mut(&mut self) -> &mut AudioState {
        &mut self.state
    }

    /// Returns volume settings.
    #[must_use]
    pub fn volumes(&self) -> &VolumeSettings {
        &self.state.volumes
    }

    /// Sets volume settings.
    pub fn set_volumes(&mut self, volumes: VolumeSettings) {
        self.state.volumes = volumes;
        self.update_all_volumes();
    }

    /// Sets master volume.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.state.volumes.master = volume.clamp(0.0, 1.0);
        self.update_all_volumes();
    }

    /// Sets music volume.
    pub fn set_music_volume(&mut self, volume: f32) {
        self.state.volumes.music = volume.clamp(0.0, 1.0);
        self.update_music_volume();
    }

    /// Sets SFX volume.
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.state.volumes.sfx = volume.clamp(0.0, 1.0);
        self.update_sfx_volumes();
    }

    /// Sets ambient volume.
    pub fn set_ambient_volume(&mut self, volume: f32) {
        self.state.volumes.ambient = volume.clamp(0.0, 1.0);
        self.update_ambient_volumes();
    }

    /// Toggles master mute.
    pub fn toggle_mute(&mut self) {
        self.state.mutes.toggle_master();
        self.update_all_volumes();
    }

    /// Sets mute state.
    pub fn set_muted(&mut self, muted: bool) {
        self.state.mutes.master = muted;
        self.update_all_volumes();
    }

    /// Updates the listener position (typically from player position).
    pub fn set_listener_position(&mut self, x: f32, y: f32) {
        self.listener.x = x;
        self.listener.y = y;
        self.update_sfx_volumes(); // Update spatial sounds
    }

    /// Sets the listener configuration.
    pub fn set_listener(&mut self, listener: AudioListener) {
        self.listener = listener;
        self.update_sfx_volumes();
    }

    /// Queues a sound event to be processed on the next update.
    pub fn queue_sound(&mut self, event: SoundEvent) {
        if self.event_queue.len() < MAX_QUEUED_EVENTS {
            self.event_queue.push_back(event);
        } else {
            debug!("Sound event queue full, dropping event: {}", event.name);
        }
    }

    /// Plays a sound immediately.
    ///
    /// Returns a handle to control the sound, or None if playback failed.
    pub fn play_sound(&mut self, event: &SoundEvent) -> Option<SoundHandle> {
        if !self.is_available() {
            return None;
        }

        let stream_handle = self.stream_handle.as_ref()?;

        // Load asset
        let asset = match self.asset_loader.load(event.category, &event.name) {
            Ok(asset) => {
                if asset.is_placeholder() {
                    self.state.missing_audio_count += 1;
                    debug!("Playing placeholder for: {}", event.name);
                }
                asset
            },
            Err(e) => {
                warn!("Failed to load audio asset {}: {}", event.name, e);
                self.state.missing_audio_count += 1;
                return None;
            },
        };

        // Create sink
        let sink = match Sink::try_new(stream_handle) {
            Ok(sink) => sink,
            Err(e) => {
                error!("Failed to create audio sink: {}", e);
                return None;
            },
        };

        // Decode and play
        let cursor = Cursor::new((*asset.data).clone());
        let decoder = match Decoder::new(cursor) {
            Ok(decoder) => decoder,
            Err(e) => {
                error!("Failed to decode audio: {}", e);
                return None;
            },
        };

        // Calculate effective volume
        let category_volume = match event.category {
            AudioCategory::Sfx => self.state.volumes.effective_sfx(),
            AudioCategory::Ui => self.state.volumes.effective_ui(),
            AudioCategory::Music => self.state.volumes.effective_music(),
            AudioCategory::Ambient => self.state.volumes.effective_ambient(),
        };

        let spatial_attenuation = if let Some((x, y)) = event.position {
            self.listener.calculate_attenuation(x, y)
        } else {
            1.0
        };

        let effective_volume = if self.state.mutes.master {
            0.0
        } else {
            event.volume * category_volume * spatial_attenuation
        };

        sink.set_volume(effective_volume);

        // Apply pitch if not 1.0
        if (event.pitch - 1.0).abs() > 0.01 {
            // Note: rodio doesn't have direct pitch control, we'd use speed
            sink.set_speed(event.pitch);
        }

        // Append source
        if event.looping {
            sink.append(decoder.repeat_infinite());
        } else {
            sink.append(decoder);
        }

        // Generate handle
        let handle = SoundHandle(self.next_handle_id);
        self.next_handle_id += 1;

        // Store active sound
        self.active_sfx.insert(
            handle,
            ActiveSound {
                sink,
                category: event.category,
                base_volume: event.volume,
                position: event.position,
                looping: event.looping,
            },
        );

        self.state.active_sfx_count = self.active_sfx.len() as u32;

        Some(handle)
    }

    /// Stops a playing sound.
    pub fn stop_sound(&mut self, handle: SoundHandle) {
        if let Some(sound) = self.active_sfx.remove(&handle) {
            sound.sink.stop();
            self.state.active_sfx_count = self.active_sfx.len() as u32;
        }
    }

    /// Pauses a playing sound.
    pub fn pause_sound(&mut self, handle: SoundHandle) {
        if let Some(sound) = self.active_sfx.get(&handle) {
            sound.sink.pause();
        }
    }

    /// Resumes a paused sound.
    pub fn resume_sound(&mut self, handle: SoundHandle) {
        if let Some(sound) = self.active_sfx.get(&handle) {
            sound.sink.play();
        }
    }

    /// Plays music track.
    pub fn play_music(&mut self, track_name: &str, fade_in: Option<f32>) {
        if !self.is_available() {
            return;
        }

        // Skip if this track already failed to load
        if self.state.failed_tracks.contains(track_name) {
            return;
        }

        let stream_handle = match &self.stream_handle {
            Some(h) => h,
            None => return,
        };

        // Stop current music
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }

        // Load music asset
        let asset = match self.asset_loader.load(AudioCategory::Music, track_name) {
            Ok(asset) => asset,
            Err(e) => {
                warn!("Failed to load music {}: {}", track_name, e);
                self.state.missing_audio_count += 1;
                self.state.failed_tracks.insert(track_name.to_string());
                return;
            },
        };

        // Create sink
        let sink = match Sink::try_new(stream_handle) {
            Ok(sink) => sink,
            Err(e) => {
                error!("Failed to create music sink: {}", e);
                return;
            },
        };

        // Decode
        let cursor = Cursor::new((*asset.data).clone());
        if let Ok(decoder) = Decoder::new(cursor) {
            sink.append(decoder.repeat_infinite());

            // Set initial volume
            if fade_in.is_some() {
                sink.set_volume(0.0);
            } else {
                let vol = self.state.volumes.effective_music();
                sink.set_volume(if self.state.mutes.is_music_muted() {
                    0.0
                } else {
                    vol
                });
            }

            self.music_sink = Some(sink);
            self.state.music.play(track_name, fade_in);

            info!("Playing music: {}", track_name);
        }
    }

    /// Stops music.
    pub fn stop_music(&mut self, fade_out: Option<f32>) {
        self.state.music.stop(fade_out);

        if fade_out.is_none() {
            if let Some(sink) = self.music_sink.take() {
                sink.stop();
            }
        }
    }

    /// Crossfades to new music track.
    pub fn crossfade_music(&mut self, track_name: &str, duration: f32) {
        if !self.is_available() {
            return;
        }

        // Skip if this track already failed to load
        if self.state.failed_tracks.contains(track_name) {
            return;
        }

        let stream_handle = match &self.stream_handle {
            Some(h) => h,
            None => return,
        };

        // Load next track
        let asset = match self.asset_loader.load(AudioCategory::Music, track_name) {
            Ok(asset) => asset,
            Err(e) => {
                warn!("Failed to load music for crossfade {}: {}", track_name, e);
                self.state.failed_tracks.insert(track_name.to_string());
                return;
            },
        };

        // Create next sink
        let sink = match Sink::try_new(stream_handle) {
            Ok(sink) => sink,
            Err(e) => {
                error!("Failed to create music sink for crossfade: {}", e);
                return;
            },
        };

        // Decode and start at 0 volume
        let cursor = Cursor::new((*asset.data).clone());
        if let Ok(decoder) = Decoder::new(cursor) {
            sink.append(decoder.repeat_infinite());
            sink.set_volume(0.0);

            self.next_music_sink = Some(sink);
            self.state.music.crossfade_to(track_name, duration);

            info!("Crossfading to music: {}", track_name);
        }
    }

    /// Sets the current biome for ambient audio.
    pub fn set_biome(&mut self, biome: &str) {
        self.state.ambient.set_biome(biome);
        // Biome change will trigger ambient layer updates in update()
    }

    /// Fades in an ambient layer.
    pub fn fade_in_ambient(
        &mut self,
        layer_name: &str,
        asset_name: &str,
        volume: f32,
        duration: f32,
    ) {
        if !self.is_available() {
            return;
        }

        let stream_handle = match &self.stream_handle {
            Some(h) => h,
            None => return,
        };

        // Create sink if not exists
        if !self.ambient_sinks.contains_key(layer_name) {
            // Load asset
            let asset = match self.asset_loader.load(AudioCategory::Ambient, asset_name) {
                Ok(asset) => asset,
                Err(e) => {
                    warn!("Failed to load ambient {}: {}", asset_name, e);
                    return;
                },
            };

            // Create sink
            if let Ok(sink) = Sink::try_new(stream_handle) {
                let cursor = Cursor::new((*asset.data).clone());
                if let Ok(decoder) = Decoder::new(cursor) {
                    sink.append(decoder.repeat_infinite());
                    sink.set_volume(0.0);
                    self.ambient_sinks.insert(layer_name.to_string(), sink);
                }
            }
        }

        self.state
            .ambient
            .fade_in_layer(layer_name, asset_name, volume, duration);
    }

    /// Fades out an ambient layer.
    pub fn fade_out_ambient(&mut self, layer_name: &str, duration: f32) {
        self.state.ambient.fade_out_layer(layer_name, duration);
    }

    /// Updates the audio system (call once per frame).
    pub fn update(&mut self, dt: f32) {
        // Process event queue
        while let Some(event) = self.event_queue.pop_front() {
            self.play_sound(&event);
        }

        // Update state
        self.state.update(dt);

        // Update music fading
        self.update_music_fade(dt);

        // Update ambient fading
        self.update_ambient_fade();

        // Cleanup finished sounds
        self.cleanup_finished_sounds();

        // Check for hot-reload in debug builds
        #[cfg(debug_assertions)]
        {
            // Only check occasionally to avoid performance impact
            self.hot_reload_counter += 1;
            if self.hot_reload_counter >= 60 {
                self.hot_reload_counter = 0;
                let reloaded = self.asset_loader.check_hot_reload();
                if reloaded > 0 {
                    info!("Hot-reloaded {} audio assets", reloaded);
                }
            }
        }
    }

    /// Updates music fade/crossfade.
    fn update_music_fade(&mut self, _dt: f32) {
        let effective_volume = if self.state.mutes.is_music_muted() {
            0.0
        } else {
            self.state.volumes.effective_music()
        };

        match &self.state.music.play_state {
            MusicPlayState::FadingIn {
                progress,
                target_volume,
                ..
            } => {
                if let Some(sink) = &self.music_sink {
                    sink.set_volume(target_volume * progress * effective_volume);
                }
            },
            MusicPlayState::FadingOut {
                progress,
                start_volume,
                ..
            } => {
                if let Some(sink) = &self.music_sink {
                    sink.set_volume(start_volume * (1.0 - progress) * effective_volume);
                }
                if *progress >= 1.0 {
                    if let Some(sink) = self.music_sink.take() {
                        sink.stop();
                    }
                }
            },
            MusicPlayState::Crossfading { progress, .. } => {
                // Fade out old
                if let Some(sink) = &self.music_sink {
                    sink.set_volume((1.0 - progress) * effective_volume);
                }
                // Fade in new
                if let Some(sink) = &self.next_music_sink {
                    sink.set_volume(*progress * effective_volume);
                }
                // Swap when complete
                if *progress >= 1.0 {
                    if let Some(old) = self.music_sink.take() {
                        old.stop();
                    }
                    self.music_sink = self.next_music_sink.take();
                }
            },
            MusicPlayState::Playing => {
                if let Some(sink) = &self.music_sink {
                    sink.set_volume(effective_volume);
                }
            },
            MusicPlayState::Paused => {
                if let Some(sink) = &self.music_sink {
                    sink.pause();
                }
            },
            MusicPlayState::Stopped => {},
        }
    }

    /// Updates ambient layer fading.
    fn update_ambient_fade(&mut self) {
        let effective_volume = if self.state.mutes.is_ambient_muted() {
            0.0
        } else {
            self.state.volumes.effective_ambient()
        };

        // Update each layer's sink volume
        for (name, layer) in &self.state.ambient.layers {
            if let Some(sink) = self.ambient_sinks.get(name) {
                sink.set_volume(layer.volume * effective_volume);
            }
        }

        // Remove sinks for removed layers
        let layer_names: Vec<_> = self.state.ambient.layers.keys().cloned().collect();
        self.ambient_sinks.retain(|name, sink| {
            if layer_names.contains(name) {
                true
            } else {
                sink.stop();
                false
            }
        });
    }

    /// Cleans up finished sounds.
    fn cleanup_finished_sounds(&mut self) {
        let finished: Vec<_> = self
            .active_sfx
            .iter()
            .filter(|(_, sound)| sound.sink.empty() && !sound.looping)
            .map(|(handle, _)| *handle)
            .collect();

        for handle in finished {
            self.active_sfx.remove(&handle);
        }

        self.state.active_sfx_count = self.active_sfx.len() as u32;
    }

    /// Updates all volume levels.
    fn update_all_volumes(&mut self) {
        self.update_music_volume();
        self.update_sfx_volumes();
        self.update_ambient_volumes();
    }

    /// Updates music volume.
    fn update_music_volume(&mut self) {
        let vol = if self.state.mutes.is_music_muted() {
            0.0
        } else {
            self.state.volumes.effective_music()
        };

        if let Some(sink) = &self.music_sink {
            // Only update if playing (not fading)
            if self.state.music.play_state == MusicPlayState::Playing {
                sink.set_volume(vol);
            }
        }
    }

    /// Updates SFX volumes.
    fn update_sfx_volumes(&mut self) {
        let sfx_vol = self.state.volumes.effective_sfx();
        let ui_vol = self.state.volumes.effective_ui();
        let is_muted = self.state.mutes.master;

        for sound in self.active_sfx.values_mut() {
            let category_vol = match sound.category {
                AudioCategory::Sfx => sfx_vol,
                AudioCategory::Ui => ui_vol,
                _ => 1.0,
            };

            let spatial = if let Some((x, y)) = sound.position {
                self.listener.calculate_attenuation(x, y)
            } else {
                1.0
            };

            let vol = if is_muted {
                0.0
            } else {
                sound.base_volume * category_vol * spatial
            };

            sound.sink.set_volume(vol);
        }
    }

    /// Updates ambient volumes.
    fn update_ambient_volumes(&mut self) {
        self.update_ambient_fade();
    }

    /// Stops all audio.
    pub fn stop_all(&mut self) {
        // Stop SFX
        for (_, sound) in self.active_sfx.drain() {
            sound.sink.stop();
        }

        // Stop music
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
        if let Some(sink) = self.next_music_sink.take() {
            sink.stop();
        }

        // Stop ambient
        for (_, sink) in self.ambient_sinks.drain() {
            sink.stop();
        }

        // Reset state
        self.state.music = MusicState::default();
        self.state.ambient = AmbientState::default();
        self.state.active_sfx_count = 0;

        info!("All audio stopped");
    }

    /// Preloads SFX and UI assets.
    pub fn preload_sfx(&mut self) {
        self.asset_loader.preload_sfx();
    }

    /// Returns asset loader statistics.
    #[must_use]
    pub fn asset_stats(&self) -> &crate::audio_assets::LoaderStats {
        self.asset_loader.stats()
    }

    /// Attempts to reinitialize the audio device.
    ///
    /// Call this if the audio device was lost or changed.
    pub fn reinitialize_device(&mut self) -> bool {
        // Clean up existing
        self.stop_all();
        self.output_stream = None;
        self.stream_handle = None;

        // Try to reinitialize
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                info!("Audio device reinitialized successfully");
                self.output_stream = Some(stream);
                self.stream_handle = Some(handle);
                self.state.device_available = true;
                true
            },
            Err(e) => {
                warn!("Failed to reinitialize audio device: {}", e);
                self.state.device_available = false;
                false
            },
        }
    }
}

impl Default for AudioIntegration {
    fn default() -> Self {
        Self::with_default_assets()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_event_new() {
        let event = SoundEvent::new(AudioCategory::Sfx, "test_sound");
        assert_eq!(event.name, "test_sound");
        assert_eq!(event.category, AudioCategory::Sfx);
        assert!(event.position.is_none());
        assert!((event.volume - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_sound_event_builder() {
        let event = SoundEvent::new(AudioCategory::Sfx, "test")
            .at_position(10.0, 20.0)
            .with_volume(0.5)
            .with_pitch(1.5)
            .with_looping(true);

        assert_eq!(event.position, Some((10.0, 20.0)));
        assert!((event.volume - 0.5).abs() < 0.001);
        assert!((event.pitch - 1.5).abs() < 0.001);
        assert!(event.looping);
    }

    #[test]
    fn test_sound_handle() {
        let handle = SoundHandle(42);
        assert_eq!(handle.raw(), 42);
    }

    #[test]
    fn test_audio_listener_new() {
        let listener = AudioListener::new(100.0, 200.0);
        assert!((listener.x - 100.0).abs() < 0.001);
        assert!((listener.y - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_audio_listener_attenuation() {
        let listener = AudioListener {
            x: 0.0,
            y: 0.0,
            max_distance: 100.0,
            rolloff: 1.0,
        };

        // At listener position
        let atten = listener.calculate_attenuation(0.0, 0.0);
        assert!((atten - 1.0).abs() < 0.001);

        // At max distance
        let atten = listener.calculate_attenuation(100.0, 0.0);
        assert!((atten - 0.0).abs() < 0.001);

        // Halfway
        let atten = listener.calculate_attenuation(50.0, 0.0);
        assert!(atten > 0.3 && atten < 0.7);
    }

    #[test]
    fn test_audio_listener_panning() {
        let listener = AudioListener {
            x: 0.0,
            y: 0.0,
            max_distance: 100.0,
            rolloff: 1.0,
        };

        // Center
        let pan = listener.calculate_panning(0.0, 0.0);
        assert!((pan - 0.0).abs() < 0.001);

        // Full left
        let pan = listener.calculate_panning(-100.0, 0.0);
        assert!((pan - (-1.0)).abs() < 0.001);

        // Full right
        let pan = listener.calculate_panning(100.0, 0.0);
        assert!((pan - 1.0).abs() < 0.001);
    }

    // Note: AudioIntegration tests that require actual audio hardware
    // are skipped in unit tests. Integration tests would go separately.
}
