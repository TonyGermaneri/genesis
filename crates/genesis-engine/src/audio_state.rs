//! Audio state management.
//!
//! This module provides:
//! - `AudioState`: current volumes, mute states
//! - `MusicState`: current track, crossfade progress
//! - `AmbientState`: active layers, transition progress
//! - Save/restore audio state

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum number of ambient layers.
pub const MAX_AMBIENT_LAYERS: usize = 8;

/// Default crossfade duration in seconds.
pub const DEFAULT_CROSSFADE_SECS: f32 = 2.0;

/// Audio volume settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSettings {
    /// Master volume (0.0 - 1.0).
    pub master: f32,
    /// Music volume (0.0 - 1.0).
    pub music: f32,
    /// Sound effects volume (0.0 - 1.0).
    pub sfx: f32,
    /// Ambient sounds volume (0.0 - 1.0).
    pub ambient: f32,
    /// UI sounds volume (0.0 - 1.0).
    pub ui: f32,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.7,
            sfx: 1.0,
            ambient: 0.6,
            ui: 0.8,
        }
    }
}

impl VolumeSettings {
    /// Returns the effective volume for music.
    #[must_use]
    pub fn effective_music(&self) -> f32 {
        self.master * self.music
    }

    /// Returns the effective volume for SFX.
    #[must_use]
    pub fn effective_sfx(&self) -> f32 {
        self.master * self.sfx
    }

    /// Returns the effective volume for ambient.
    #[must_use]
    pub fn effective_ambient(&self) -> f32 {
        self.master * self.ambient
    }

    /// Returns the effective volume for UI.
    #[must_use]
    pub fn effective_ui(&self) -> f32 {
        self.master * self.ui
    }
}

/// Mute state for different audio channels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MuteState {
    /// Master mute (mutes everything).
    pub master: bool,
    /// Mute music only.
    pub music: bool,
    /// Mute sound effects only.
    pub sfx: bool,
    /// Mute ambient sounds only.
    pub ambient: bool,
    /// Mute UI sounds only.
    pub ui: bool,
}

impl MuteState {
    /// Returns true if music is effectively muted.
    #[must_use]
    pub fn is_music_muted(&self) -> bool {
        self.master || self.music
    }

    /// Returns true if SFX are effectively muted.
    #[must_use]
    pub fn is_sfx_muted(&self) -> bool {
        self.master || self.sfx
    }

    /// Returns true if ambient is effectively muted.
    #[must_use]
    pub fn is_ambient_muted(&self) -> bool {
        self.master || self.ambient
    }

    /// Returns true if UI is effectively muted.
    #[must_use]
    pub fn is_ui_muted(&self) -> bool {
        self.master || self.ui
    }

    /// Toggles master mute.
    pub fn toggle_master(&mut self) {
        self.master = !self.master;
    }
}

/// Current state of a music track.
#[derive(Debug, Clone, PartialEq)]
pub enum MusicPlayState {
    /// No music playing.
    Stopped,
    /// Music is playing normally.
    Playing,
    /// Music is paused.
    Paused,
    /// Fading in from silence.
    FadingIn {
        /// Target volume.
        target_volume: f32,
        /// Progress (0.0 - 1.0).
        progress: f32,
        /// Total fade duration.
        duration: f32,
    },
    /// Fading out to silence.
    FadingOut {
        /// Starting volume.
        start_volume: f32,
        /// Progress (0.0 - 1.0).
        progress: f32,
        /// Total fade duration.
        duration: f32,
    },
    /// Crossfading to a new track.
    Crossfading {
        /// Name of the next track.
        next_track: String,
        /// Progress (0.0 - 1.0).
        progress: f32,
        /// Total crossfade duration.
        duration: f32,
    },
}

impl Default for MusicPlayState {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Music track state.
#[derive(Debug, Clone, Default)]
pub struct MusicState {
    /// Currently playing track name.
    pub current_track: Option<String>,
    /// Play state.
    pub play_state: MusicPlayState,
    /// Track-specific volume multiplier.
    pub track_volume: f32,
    /// Playback position in seconds.
    pub position_secs: f32,
    /// Total track duration (if known).
    pub duration_secs: Option<f32>,
    /// Whether to loop the current track.
    pub looping: bool,
    /// Queue of tracks to play next.
    pub queue: Vec<String>,
}

impl MusicState {
    /// Creates new music state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_track: None,
            play_state: MusicPlayState::Stopped,
            track_volume: 1.0,
            position_secs: 0.0,
            duration_secs: None,
            looping: true,
            queue: Vec::new(),
        }
    }

    /// Returns true if music is currently playing or fading.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        matches!(
            self.play_state,
            MusicPlayState::Playing
                | MusicPlayState::FadingIn { .. }
                | MusicPlayState::Crossfading { .. }
        )
    }

    /// Returns true if a crossfade is in progress.
    #[must_use]
    pub fn is_crossfading(&self) -> bool {
        matches!(self.play_state, MusicPlayState::Crossfading { .. })
    }

    /// Returns the current effective volume (0.0 - 1.0).
    #[must_use]
    pub fn current_volume(&self) -> f32 {
        match &self.play_state {
            MusicPlayState::Stopped | MusicPlayState::Paused => 0.0,
            MusicPlayState::Playing => self.track_volume,
            MusicPlayState::FadingIn {
                target_volume,
                progress,
                ..
            } => target_volume * progress,
            MusicPlayState::FadingOut {
                start_volume,
                progress,
                ..
            } => start_volume * (1.0 - progress),
            MusicPlayState::Crossfading { progress, .. } => {
                // Old track fades out
                self.track_volume * (1.0 - progress)
            },
        }
    }

    /// Updates music state for the frame.
    ///
    /// Returns true if state changed significantly.
    pub fn update(&mut self, dt: f32) -> bool {
        let mut changed = false;

        match &mut self.play_state {
            MusicPlayState::FadingIn {
                target_volume,
                progress,
                duration,
            } => {
                *progress += dt / *duration;
                if *progress >= 1.0 {
                    self.track_volume = *target_volume;
                    self.play_state = MusicPlayState::Playing;
                    changed = true;
                }
            },
            MusicPlayState::FadingOut {
                progress, duration, ..
            } => {
                *progress += dt / *duration;
                if *progress >= 1.0 {
                    self.current_track = None;
                    self.play_state = MusicPlayState::Stopped;
                    changed = true;
                }
            },
            MusicPlayState::Crossfading {
                next_track,
                progress,
                duration,
            } => {
                *progress += dt / *duration;
                if *progress >= 1.0 {
                    self.current_track = Some(next_track.clone());
                    self.play_state = MusicPlayState::Playing;
                    self.position_secs = 0.0;
                    changed = true;
                }
            },
            MusicPlayState::Playing => {
                self.position_secs += dt;
            },
            _ => {},
        }

        changed
    }

    /// Starts playing a track.
    pub fn play(&mut self, track_name: &str, fade_in: Option<f32>) {
        self.current_track = Some(track_name.to_string());
        self.position_secs = 0.0;

        if let Some(duration) = fade_in {
            self.play_state = MusicPlayState::FadingIn {
                target_volume: self.track_volume,
                progress: 0.0,
                duration,
            };
        } else {
            self.play_state = MusicPlayState::Playing;
        }
    }

    /// Stops the current track.
    pub fn stop(&mut self, fade_out: Option<f32>) {
        if let Some(duration) = fade_out {
            self.play_state = MusicPlayState::FadingOut {
                start_volume: self.current_volume(),
                progress: 0.0,
                duration,
            };
        } else {
            self.current_track = None;
            self.play_state = MusicPlayState::Stopped;
            self.position_secs = 0.0;
        }
    }

    /// Crossfades to a new track.
    pub fn crossfade_to(&mut self, track_name: &str, duration: f32) {
        if self.current_track.as_deref() == Some(track_name) {
            return; // Already playing this track
        }

        self.play_state = MusicPlayState::Crossfading {
            next_track: track_name.to_string(),
            progress: 0.0,
            duration,
        };
    }

    /// Adds a track to the queue.
    pub fn queue_track(&mut self, track_name: &str) {
        self.queue.push(track_name.to_string());
    }

    /// Advances to the next track in queue.
    pub fn next_in_queue(&mut self, crossfade: bool) {
        if let Some(next) = self.queue.first().cloned() {
            self.queue.remove(0);
            if crossfade {
                self.crossfade_to(&next, DEFAULT_CROSSFADE_SECS);
            } else {
                self.play(&next, None);
            }
        }
    }
}

/// State of an ambient sound layer.
#[derive(Debug, Clone)]
pub struct AmbientLayer {
    /// Layer name/identifier.
    pub name: String,
    /// Asset name to play.
    pub asset_name: String,
    /// Current volume (0.0 - 1.0).
    pub volume: f32,
    /// Target volume for transitions.
    pub target_volume: f32,
    /// Whether this layer is active.
    pub active: bool,
    /// Transition progress (0.0 - 1.0, 1.0 = no transition).
    pub transition_progress: f32,
    /// Transition duration.
    pub transition_duration: f32,
}

impl AmbientLayer {
    /// Creates a new ambient layer.
    #[must_use]
    pub fn new(name: &str, asset_name: &str) -> Self {
        Self {
            name: name.to_string(),
            asset_name: asset_name.to_string(),
            volume: 0.0,
            target_volume: 0.0,
            active: false,
            transition_progress: 1.0,
            transition_duration: 1.0,
        }
    }

    /// Starts fading in the layer.
    pub fn fade_in(&mut self, target_volume: f32, duration: f32) {
        self.target_volume = target_volume;
        self.transition_progress = 0.0;
        self.transition_duration = duration;
        self.active = true;
    }

    /// Starts fading out the layer.
    pub fn fade_out(&mut self, duration: f32) {
        self.target_volume = 0.0;
        self.transition_progress = 0.0;
        self.transition_duration = duration;
    }

    /// Updates the layer state.
    ///
    /// Returns true if the layer became inactive.
    pub fn update(&mut self, dt: f32) -> bool {
        if self.transition_progress < 1.0 {
            self.transition_progress += dt / self.transition_duration;
            if self.transition_progress >= 1.0 {
                self.transition_progress = 1.0;
                self.volume = self.target_volume;

                // Deactivate if faded to zero
                if self.volume <= 0.001 {
                    self.active = false;
                    return true;
                }
            } else {
                // Interpolate volume
                let start_volume = if self.target_volume > self.volume {
                    0.0 // Fading in
                } else {
                    self.volume // Fading out
                };
                self.volume =
                    start_volume + (self.target_volume - start_volume) * self.transition_progress;
            }
        }
        false
    }

    /// Returns true if a transition is in progress.
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
        self.transition_progress < 1.0
    }
}

/// Ambient audio state with multiple layers.
#[derive(Debug, Clone, Default)]
pub struct AmbientState {
    /// Active ambient layers.
    pub layers: HashMap<String, AmbientLayer>,
    /// Current biome (affects which layers are active).
    pub current_biome: Option<String>,
    /// Time of day factor (0.0 = midnight, 0.5 = noon, 1.0 = midnight).
    pub time_of_day: f32,
    /// Indoor/outdoor state (affects ambient).
    pub is_indoors: bool,
}

impl AmbientState {
    /// Creates new ambient state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            current_biome: None,
            time_of_day: 0.5, // Default to noon
            is_indoors: false,
        }
    }

    /// Adds or updates an ambient layer.
    pub fn set_layer(&mut self, layer: AmbientLayer) {
        self.layers.insert(layer.name.clone(), layer);
    }

    /// Gets a layer by name.
    #[must_use]
    pub fn get_layer(&self, name: &str) -> Option<&AmbientLayer> {
        self.layers.get(name)
    }

    /// Gets a mutable layer by name.
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut AmbientLayer> {
        self.layers.get_mut(name)
    }

    /// Removes a layer.
    pub fn remove_layer(&mut self, name: &str) {
        self.layers.remove(name);
    }

    /// Fades in a layer.
    pub fn fade_in_layer(&mut self, name: &str, asset_name: &str, volume: f32, duration: f32) {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.fade_in(volume, duration);
        } else {
            let mut layer = AmbientLayer::new(name, asset_name);
            layer.fade_in(volume, duration);
            self.layers.insert(name.to_string(), layer);
        }
    }

    /// Fades out a layer.
    pub fn fade_out_layer(&mut self, name: &str, duration: f32) {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.fade_out(duration);
        }
    }

    /// Updates all layers.
    ///
    /// Returns names of layers that became inactive.
    pub fn update(&mut self, dt: f32) -> Vec<String> {
        let mut inactive = Vec::new();

        for (name, layer) in &mut self.layers {
            if layer.update(dt) {
                inactive.push(name.clone());
            }
        }

        // Remove inactive layers
        for name in &inactive {
            self.layers.remove(name);
        }

        inactive
    }

    /// Returns the number of active layers.
    #[must_use]
    pub fn active_layer_count(&self) -> usize {
        self.layers.values().filter(|l| l.active).count()
    }

    /// Sets the current biome.
    pub fn set_biome(&mut self, biome: &str) {
        if self.current_biome.as_deref() != Some(biome) {
            self.current_biome = Some(biome.to_string());
        }
    }
}

/// Complete audio system state.
#[derive(Debug, Clone)]
pub struct AudioState {
    /// Volume settings.
    pub volumes: VolumeSettings,
    /// Mute states.
    pub mutes: MuteState,
    /// Music state.
    pub music: MusicState,
    /// Ambient state.
    pub ambient: AmbientState,
    /// Whether audio system is initialized.
    pub initialized: bool,
    /// Whether audio device is available.
    pub device_available: bool,
    /// Number of active sound effects.
    pub active_sfx_count: u32,
    /// Missing audio indicator (for debug UI).
    pub missing_audio_count: u32,
    /// Tracks that failed to load (prevents repeated attempts).
    pub failed_tracks: std::collections::HashSet<String>,
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioState {
    /// Creates new audio state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            volumes: VolumeSettings::default(),
            mutes: MuteState::default(),
            music: MusicState::new(),
            ambient: AmbientState::new(),
            initialized: false,
            device_available: false,
            active_sfx_count: 0,
            missing_audio_count: 0,
            failed_tracks: std::collections::HashSet::new(),
        }
    }

    /// Updates audio state for the frame.
    pub fn update(&mut self, dt: f32) {
        self.music.update(dt);
        self.ambient.update(dt);
    }

    /// Returns true if audio is effectively enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.initialized && self.device_available && !self.mutes.master
    }

    /// Saves audio state to serializable format.
    #[must_use]
    pub fn save(&self) -> SavedAudioState {
        SavedAudioState {
            volumes: self.volumes.clone(),
            mutes: self.mutes.clone(),
            current_music_track: self.music.current_track.clone(),
            music_position: self.music.position_secs,
            music_looping: self.music.looping,
        }
    }

    /// Restores audio state from saved data.
    pub fn restore(&mut self, saved: &SavedAudioState) {
        self.volumes = saved.volumes.clone();
        self.mutes = saved.mutes.clone();
        self.music.looping = saved.music_looping;
        // Note: We don't restore music position/track directly
        // The caller should trigger playback with the restored track name
    }
}

/// Saved audio state for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAudioState {
    /// Volume settings.
    pub volumes: VolumeSettings,
    /// Mute states.
    pub mutes: MuteState,
    /// Currently playing music track.
    pub current_music_track: Option<String>,
    /// Music playback position.
    pub music_position: f32,
    /// Whether music was looping.
    pub music_looping: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_settings_default() {
        let vol = VolumeSettings::default();
        assert!((vol.master - 1.0).abs() < 0.001);
        assert!((vol.music - 0.7).abs() < 0.001);
        assert!((vol.sfx - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_volume_settings_effective() {
        let vol = VolumeSettings {
            master: 0.5,
            music: 0.8,
            sfx: 1.0,
            ambient: 0.6,
            ui: 0.8,
        };
        assert!((vol.effective_music() - 0.4).abs() < 0.001);
        assert!((vol.effective_sfx() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mute_state() {
        let mut mute = MuteState::default();
        assert!(!mute.is_music_muted());

        mute.master = true;
        assert!(mute.is_music_muted());
        assert!(mute.is_sfx_muted());
    }

    #[test]
    fn test_mute_state_toggle() {
        let mut mute = MuteState::default();
        assert!(!mute.master);

        mute.toggle_master();
        assert!(mute.master);

        mute.toggle_master();
        assert!(!mute.master);
    }

    #[test]
    fn test_music_state_new() {
        let music = MusicState::new();
        assert!(music.current_track.is_none());
        assert!(!music.is_playing());
        assert_eq!(music.play_state, MusicPlayState::Stopped);
    }

    #[test]
    fn test_music_state_play() {
        let mut music = MusicState::new();
        music.play("test_track", None);

        assert_eq!(music.current_track.as_deref(), Some("test_track"));
        assert!(music.is_playing());
    }

    #[test]
    fn test_music_state_play_with_fade() {
        let mut music = MusicState::new();
        music.play("test_track", Some(2.0));

        assert!(matches!(music.play_state, MusicPlayState::FadingIn { .. }));
    }

    #[test]
    fn test_music_state_stop() {
        let mut music = MusicState::new();
        music.play("test_track", None);
        music.stop(None);

        assert!(music.current_track.is_none());
        assert!(!music.is_playing());
    }

    #[test]
    fn test_music_state_crossfade() {
        let mut music = MusicState::new();
        music.play("track1", None);
        music.crossfade_to("track2", 2.0);

        assert!(music.is_crossfading());
    }

    #[test]
    fn test_music_state_update_fade_in() {
        let mut music = MusicState::new();
        music.track_volume = 1.0;
        music.play("test", Some(1.0)); // 1 second fade

        // Simulate half the fade
        music.update(0.5);
        assert!(music.current_volume() > 0.4);
        assert!(music.current_volume() < 0.6);

        // Complete the fade
        music.update(0.6);
        assert_eq!(music.play_state, MusicPlayState::Playing);
    }

    #[test]
    fn test_ambient_layer_new() {
        let layer = AmbientLayer::new("wind", "ambient/wind");
        assert_eq!(layer.name, "wind");
        assert!(!layer.active);
        assert!((layer.volume - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_ambient_layer_fade_in() {
        let mut layer = AmbientLayer::new("wind", "ambient/wind");
        layer.fade_in(0.8, 1.0);

        assert!(layer.active);
        assert!(layer.is_transitioning());
        assert!((layer.target_volume - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_ambient_layer_update() {
        let mut layer = AmbientLayer::new("wind", "ambient/wind");
        layer.fade_in(1.0, 1.0);

        // Half the transition
        layer.update(0.5);
        assert!(layer.volume > 0.4);
        assert!(layer.is_transitioning());

        // Complete
        layer.update(0.6);
        assert!(!layer.is_transitioning());
        assert!((layer.volume - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ambient_state_layers() {
        let mut ambient = AmbientState::new();
        ambient.fade_in_layer("wind", "ambient/wind", 0.8, 1.0);

        assert_eq!(ambient.active_layer_count(), 1);
        assert!(ambient.get_layer("wind").is_some());
    }

    #[test]
    fn test_audio_state_new() {
        let state = AudioState::new();
        assert!(!state.initialized);
        assert!(!state.device_available);
        assert!(!state.is_enabled());
    }

    #[test]
    fn test_audio_state_enabled() {
        let mut state = AudioState::new();
        state.initialized = true;
        state.device_available = true;

        assert!(state.is_enabled());

        state.mutes.master = true;
        assert!(!state.is_enabled());
    }

    #[test]
    fn test_audio_state_save_restore() {
        let mut state = AudioState::new();
        state.volumes.master = 0.5;
        state.mutes.music = true;
        state.music.looping = false;

        let saved = state.save();

        let mut restored = AudioState::new();
        restored.restore(&saved);

        assert!((restored.volumes.master - 0.5).abs() < 0.001);
        assert!(restored.mutes.music);
        assert!(!restored.music.looping);
    }

    #[test]
    fn test_music_queue() {
        let mut music = MusicState::new();
        music.queue_track("track1");
        music.queue_track("track2");

        assert_eq!(music.queue.len(), 2);

        music.next_in_queue(false);
        assert_eq!(music.current_track.as_deref(), Some("track1"));
        assert_eq!(music.queue.len(), 1);
    }
}
