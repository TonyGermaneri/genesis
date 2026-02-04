//! Spatial audio integration for 2D game environments.
//!
//! This module provides a spatial audio system that calculates stereo panning
//! and distance attenuation for sound sources in 2D space. It integrates with
//! the kernel's world coordinates to provide realistic audio positioning.

use std::collections::HashMap;

use tracing::debug;

/// Maximum number of simultaneous sound sources.
pub const MAX_SOUND_SOURCES: usize = 128;

/// Default maximum hearing distance in world units.
pub const DEFAULT_MAX_DISTANCE: f32 = 1000.0;

/// Default reference distance for volume calculations.
pub const DEFAULT_REFERENCE_DISTANCE: f32 = 100.0;

/// Speed of sound in world units per second (for Doppler effect).
pub const SPEED_OF_SOUND: f32 = 343.0;

/// Unique identifier for a sound source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundSourceId(u32);

impl SoundSourceId {
    /// Creates a new sound source ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Type of audio environment for reverb/effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum AudioEnvironment {
    /// Open outdoor space (minimal reverb).
    #[default]
    Outdoor = 0,
    /// Cave or enclosed underground (heavy reverb).
    Cave = 1,
    /// Small room (moderate reverb).
    Room = 2,
    /// Large hall (long reverb).
    Hall = 3,
    /// Underwater (muffled, low-pass).
    Underwater = 4,
}

impl AudioEnvironment {
    /// Converts from raw u32 value.
    #[must_use]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Cave,
            2 => Self::Room,
            3 => Self::Hall,
            4 => Self::Underwater,
            _ => Self::Outdoor,
        }
    }

    /// Returns reverb mix level (0.0-1.0) for this environment.
    #[must_use]
    pub const fn reverb_mix(&self) -> f32 {
        match self {
            Self::Outdoor => 0.05,
            Self::Cave => 0.7,
            Self::Room => 0.3,
            Self::Hall => 0.5,
            Self::Underwater => 0.4,
        }
    }

    /// Returns reverb decay time in seconds.
    #[must_use]
    pub const fn reverb_decay(&self) -> f32 {
        match self {
            Self::Outdoor => 0.1,
            Self::Cave => 2.5,
            Self::Room => 0.8,
            Self::Hall => 1.5,
            Self::Underwater => 1.0,
        }
    }

    /// Returns low-pass filter cutoff frequency (Hz) for this environment.
    #[must_use]
    pub const fn lowpass_cutoff(&self) -> f32 {
        match self {
            Self::Outdoor => 20000.0,
            Self::Cave => 8000.0,
            Self::Room => 15000.0,
            Self::Hall => 12000.0,
            Self::Underwater => 500.0,
        }
    }
}

/// Sound attenuation model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttenuationModel {
    /// No attenuation (constant volume).
    None,
    /// Linear falloff to zero at max distance.
    Linear,
    /// Inverse distance (1/d) falloff.
    #[default]
    Inverse,
    /// Exponential falloff.
    Exponential,
}

impl AttenuationModel {
    /// Calculates volume multiplier based on distance.
    #[must_use]
    pub fn calculate(&self, distance: f32, ref_distance: f32, max_distance: f32) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Linear => {
                if distance >= max_distance {
                    0.0
                } else if distance <= ref_distance {
                    1.0
                } else {
                    1.0 - (distance - ref_distance) / (max_distance - ref_distance)
                }
            },
            Self::Inverse => {
                if distance < ref_distance {
                    1.0
                } else {
                    ref_distance / distance
                }
            },
            Self::Exponential => {
                if distance < ref_distance {
                    1.0
                } else {
                    (-((distance - ref_distance) / ref_distance)).exp()
                }
            },
        }
    }
}

/// A positioned sound source in the world.
#[derive(Debug, Clone, Copy)]
pub struct SoundSource {
    /// Position in world coordinates.
    pub position: (f32, f32),
    /// Velocity for Doppler effect (world units per second).
    pub velocity: (f32, f32),
    /// Base volume (0.0-1.0).
    pub volume: f32,
    /// Pitch multiplier (1.0 = normal).
    pub pitch: f32,
    /// Reference distance for attenuation.
    pub ref_distance: f32,
    /// Maximum hearing distance.
    pub max_distance: f32,
    /// Attenuation model to use.
    pub attenuation: AttenuationModel,
    /// Whether the source is currently playing.
    pub playing: bool,
    /// Whether the source loops.
    pub looping: bool,
    /// Sound priority (higher = more important).
    pub priority: u8,
}

impl Default for SoundSource {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            volume: 1.0,
            pitch: 1.0,
            ref_distance: DEFAULT_REFERENCE_DISTANCE,
            max_distance: DEFAULT_MAX_DISTANCE,
            attenuation: AttenuationModel::Inverse,
            playing: false,
            looping: false,
            priority: 128,
        }
    }
}

impl SoundSource {
    /// Creates a new sound source at a position.
    #[must_use]
    pub fn new(position: (f32, f32)) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Sets the volume.
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Sets the position.
    #[must_use]
    pub const fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Sets the velocity for Doppler effect.
    #[must_use]
    pub const fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.velocity = (vx, vy);
        self
    }

    /// Sets the reference distance.
    #[must_use]
    pub const fn with_ref_distance(mut self, distance: f32) -> Self {
        self.ref_distance = distance;
        self
    }

    /// Sets the max distance.
    #[must_use]
    pub const fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance;
        self
    }

    /// Sets the attenuation model.
    #[must_use]
    pub const fn with_attenuation(mut self, model: AttenuationModel) -> Self {
        self.attenuation = model;
        self
    }

    /// Sets whether the source loops.
    #[must_use]
    pub const fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Sets the priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Starts playing the sound.
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stops playing the sound.
    pub fn stop(&mut self) {
        self.playing = false;
    }
}

/// Calculated spatial audio parameters for a sound source.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpatialAudioParams {
    /// Left channel volume (0.0-1.0).
    pub left_volume: f32,
    /// Right channel volume (0.0-1.0).
    pub right_volume: f32,
    /// Pitch multiplier (including Doppler).
    pub pitch: f32,
    /// Distance from listener.
    pub distance: f32,
    /// Whether the sound is audible.
    pub audible: bool,
}

/// Listener position and orientation.
#[derive(Debug, Clone, Copy)]
pub struct AudioListener {
    /// Position in world coordinates.
    pub position: (f32, f32),
    /// Velocity for Doppler effect.
    pub velocity: (f32, f32),
    /// Facing direction (normalized).
    pub direction: (f32, f32),
    /// Master volume multiplier.
    pub volume: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            direction: (1.0, 0.0),
            volume: 1.0,
        }
    }
}

impl AudioListener {
    /// Creates a new listener at a position.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            velocity: (0.0, 0.0),
            direction: (1.0, 0.0),
            volume: 1.0,
        }
    }

    /// Sets the listener position.
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    /// Sets the listener velocity.
    pub fn set_velocity(&mut self, vx: f32, vy: f32) {
        self.velocity = (vx, vy);
    }

    /// Sets the facing direction (will be normalized).
    pub fn set_direction(&mut self, dx: f32, dy: f32) {
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            self.direction = (dx / len, dy / len);
        }
    }
}

/// Spatial audio manager.
///
/// Calculates stereo panning and distance attenuation for sound sources
/// relative to a listener position.
#[derive(Debug)]
pub struct SpatialAudioManager {
    /// Active sound sources.
    sources: HashMap<SoundSourceId, SoundSource>,
    /// Next source ID.
    next_source_id: u32,
    /// Listener state.
    listener: AudioListener,
    /// Current audio environment.
    environment: AudioEnvironment,
    /// Whether Doppler effect is enabled.
    doppler_enabled: bool,
    /// Doppler factor (0.0 = none, 1.0 = normal).
    doppler_factor: f32,
}

impl Default for SpatialAudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialAudioManager {
    /// Creates a new spatial audio manager.
    #[must_use]
    pub fn new() -> Self {
        debug!("Created spatial audio manager");
        Self {
            sources: HashMap::new(),
            next_source_id: 0,
            listener: AudioListener::default(),
            environment: AudioEnvironment::Outdoor,
            doppler_enabled: true,
            doppler_factor: 1.0,
        }
    }

    /// Adds a sound source to the manager.
    ///
    /// Returns a unique ID for the source, or `None` if at capacity.
    pub fn add_source(&mut self, source: SoundSource) -> Option<SoundSourceId> {
        if self.sources.len() >= MAX_SOUND_SOURCES {
            return None;
        }

        let id = SoundSourceId::new(self.next_source_id);
        self.next_source_id += 1;
        self.sources.insert(id, source);

        debug!("Added sound source {:?}", id);
        Some(id)
    }

    /// Removes a sound source from the manager.
    pub fn remove_source(&mut self, id: SoundSourceId) -> Option<SoundSource> {
        let removed = self.sources.remove(&id);
        if removed.is_some() {
            debug!("Removed sound source {:?}", id);
        }
        removed
    }

    /// Gets a sound source by ID.
    #[must_use]
    pub fn get_source(&self, id: SoundSourceId) -> Option<&SoundSource> {
        self.sources.get(&id)
    }

    /// Gets a mutable sound source by ID.
    pub fn get_source_mut(&mut self, id: SoundSourceId) -> Option<&mut SoundSource> {
        self.sources.get_mut(&id)
    }

    /// Returns the number of active sources.
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Gets the listener.
    #[must_use]
    pub const fn listener(&self) -> &AudioListener {
        &self.listener
    }

    /// Gets a mutable reference to the listener.
    pub fn listener_mut(&mut self) -> &mut AudioListener {
        &mut self.listener
    }

    /// Sets the listener position.
    pub fn set_listener_position(&mut self, x: f32, y: f32) {
        self.listener.set_position(x, y);
    }

    /// Sets the audio environment.
    pub fn set_environment(&mut self, env: AudioEnvironment) {
        self.environment = env;
    }

    /// Gets the current environment.
    #[must_use]
    pub const fn environment(&self) -> AudioEnvironment {
        self.environment
    }

    /// Enables or disables Doppler effect.
    pub fn set_doppler_enabled(&mut self, enabled: bool) {
        self.doppler_enabled = enabled;
    }

    /// Sets the Doppler factor.
    pub fn set_doppler_factor(&mut self, factor: f32) {
        self.doppler_factor = factor.max(0.0);
    }

    /// Calculates spatial audio parameters for a source.
    #[must_use]
    pub fn calculate_spatial_params(&self, source: &SoundSource) -> SpatialAudioParams {
        // Calculate distance
        let dx = source.position.0 - self.listener.position.0;
        let dy = source.position.1 - self.listener.position.1;
        let distance = (dx * dx + dy * dy).sqrt();

        // Check if audible
        if distance > source.max_distance || !source.playing {
            return SpatialAudioParams {
                audible: false,
                distance,
                ..Default::default()
            };
        }

        // Calculate attenuation
        let attenuation =
            source
                .attenuation
                .calculate(distance, source.ref_distance, source.max_distance);
        let base_volume = source.volume * attenuation * self.listener.volume;

        // Calculate panning (stereo)
        // Angle from listener to source
        let angle = if distance > 0.01 {
            // Calculate relative angle from listener's facing direction
            let to_source = (dx / distance, dy / distance);
            let cross =
                self.listener.direction.0 * to_source.1 - self.listener.direction.1 * to_source.0;
            let dot =
                self.listener.direction.0 * to_source.0 + self.listener.direction.1 * to_source.1;
            cross.atan2(dot)
        } else {
            0.0
        };

        // Convert angle to panning (-1 = full left, +1 = full right)
        let pan = angle.sin().clamp(-1.0, 1.0);

        // Calculate per-channel volumes using constant power panning
        let pan_angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
        let left_volume = base_volume * pan_angle.cos();
        let right_volume = base_volume * pan_angle.sin();

        // Calculate Doppler pitch shift
        let pitch = if self.doppler_enabled && distance > 0.01 {
            // Relative velocity along the line between listener and source
            let direction = (dx / distance, dy / distance);
            let listener_vel =
                self.listener.velocity.0 * direction.0 + self.listener.velocity.1 * direction.1;
            let source_vel = source.velocity.0 * direction.0 + source.velocity.1 * direction.1;

            // Doppler formula: f' = f * (c + v_l) / (c + v_s)
            // Where c = speed of sound, v_l = listener velocity toward source, v_s = source velocity toward listener
            let numerator = SPEED_OF_SOUND + listener_vel * self.doppler_factor;
            let denominator = SPEED_OF_SOUND + source_vel * self.doppler_factor;

            if denominator.abs() > 0.01 {
                source.pitch * (numerator / denominator)
            } else {
                source.pitch
            }
        } else {
            source.pitch
        };

        SpatialAudioParams {
            left_volume,
            right_volume,
            pitch,
            distance,
            audible: base_volume > 0.001,
        }
    }

    /// Calculates spatial parameters for all playing sources.
    #[must_use]
    pub fn calculate_all_spatial_params(&self) -> Vec<(SoundSourceId, SpatialAudioParams)> {
        self.sources
            .iter()
            .filter(|(_, source)| source.playing)
            .map(|(&id, source)| (id, self.calculate_spatial_params(source)))
            .filter(|(_, params)| params.audible)
            .collect()
    }

    /// Gets the N loudest audible sources by distance.
    #[must_use]
    pub fn get_loudest_sources(&self, count: usize) -> Vec<(SoundSourceId, SpatialAudioParams)> {
        let mut params: Vec<_> = self.calculate_all_spatial_params();
        params.sort_by(|a, b| {
            // Sort by priority first, then by volume
            let priority_cmp = self
                .sources
                .get(&b.0)
                .map_or(0, |s| s.priority)
                .cmp(&self.sources.get(&a.0).map_or(0, |s| s.priority));

            if priority_cmp == std::cmp::Ordering::Equal {
                // Sort by combined volume (higher first)
                let vol_a = a.1.left_volume + a.1.right_volume;
                let vol_b = b.1.left_volume + b.1.right_volume;
                vol_b
                    .partial_cmp(&vol_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                priority_cmp
            }
        });
        params.truncate(count);
        params
    }

    /// Updates source positions and velocities from world data.
    pub fn update_source_position(&mut self, id: SoundSourceId, x: f32, y: f32) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.position = (x, y);
        }
    }

    /// Stops all playing sources.
    pub fn stop_all(&mut self) {
        for source in self.sources.values_mut() {
            source.playing = false;
        }
    }

    /// Clears all sources.
    pub fn clear(&mut self) {
        self.sources.clear();
    }

    /// Gets environment audio parameters for effects processing.
    #[must_use]
    pub fn get_environment_params(&self) -> EnvironmentParams {
        EnvironmentParams {
            reverb_mix: self.environment.reverb_mix(),
            reverb_decay: self.environment.reverb_decay(),
            lowpass_cutoff: self.environment.lowpass_cutoff(),
            environment: self.environment,
        }
    }
}

/// Environment audio parameters for effects processing.
#[derive(Debug, Clone, Copy)]
pub struct EnvironmentParams {
    /// Reverb mix level (0.0-1.0).
    pub reverb_mix: f32,
    /// Reverb decay time in seconds.
    pub reverb_decay: f32,
    /// Low-pass filter cutoff frequency.
    pub lowpass_cutoff: f32,
    /// Current environment type.
    pub environment: AudioEnvironment,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_source_id() {
        let id = SoundSourceId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_audio_environment_conversion() {
        assert_eq!(AudioEnvironment::from_u32(0), AudioEnvironment::Outdoor);
        assert_eq!(AudioEnvironment::from_u32(1), AudioEnvironment::Cave);
        assert_eq!(AudioEnvironment::from_u32(2), AudioEnvironment::Room);
        assert_eq!(AudioEnvironment::from_u32(3), AudioEnvironment::Hall);
        assert_eq!(AudioEnvironment::from_u32(4), AudioEnvironment::Underwater);
        assert_eq!(AudioEnvironment::from_u32(99), AudioEnvironment::Outdoor);
    }

    #[test]
    fn test_environment_reverb() {
        let cave = AudioEnvironment::Cave;
        assert!(cave.reverb_mix() > 0.5);
        assert!(cave.reverb_decay() > 2.0);

        let outdoor = AudioEnvironment::Outdoor;
        assert!(outdoor.reverb_mix() < 0.1);
    }

    #[test]
    fn test_attenuation_models() {
        // None - always 1.0
        let none = AttenuationModel::None;
        assert!((none.calculate(500.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);

        // Linear
        let linear = AttenuationModel::Linear;
        assert!((linear.calculate(50.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON); // Under ref
        assert!(linear.calculate(500.0, 100.0, 1000.0) < 1.0); // Partial
        assert!((linear.calculate(1000.0, 100.0, 1000.0) - 0.0).abs() < f32::EPSILON); // At max

        // Inverse
        let inverse = AttenuationModel::Inverse;
        assert!((inverse.calculate(50.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON); // Under ref
        assert!((inverse.calculate(200.0, 100.0, 1000.0) - 0.5).abs() < f32::EPSILON); // 2x = 0.5

        // Exponential
        let exp = AttenuationModel::Exponential;
        assert!((exp.calculate(50.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);
        assert!(exp.calculate(200.0, 100.0, 1000.0) < 0.5);
    }

    #[test]
    fn test_sound_source_builder() {
        let source = SoundSource::new((100.0, 200.0))
            .with_volume(0.5)
            .with_max_distance(500.0)
            .with_looping(true)
            .with_priority(200);

        assert!((source.position.0 - 100.0).abs() < f32::EPSILON);
        assert!((source.position.1 - 200.0).abs() < f32::EPSILON);
        assert!((source.volume - 0.5).abs() < f32::EPSILON);
        assert!((source.max_distance - 500.0).abs() < f32::EPSILON);
        assert!(source.looping);
        assert_eq!(source.priority, 200);
    }

    #[test]
    fn test_listener_creation() {
        let listener = AudioListener::new(50.0, 75.0);
        assert!((listener.position.0 - 50.0).abs() < f32::EPSILON);
        assert!((listener.position.1 - 75.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_listener_direction_normalization() {
        let mut listener = AudioListener::default();
        listener.set_direction(3.0, 4.0);

        let len = (listener.direction.0 * listener.direction.0
            + listener.direction.1 * listener.direction.1)
            .sqrt();
        assert!((len - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_manager_creation() {
        let manager = SpatialAudioManager::new();
        assert_eq!(manager.source_count(), 0);
    }

    #[test]
    fn test_add_remove_source() {
        let mut manager = SpatialAudioManager::new();

        let source = SoundSource::new((100.0, 100.0));
        let id = manager.add_source(source);
        assert!(id.is_some());
        assert_eq!(manager.source_count(), 1);

        let id = id.expect("should have id");
        let removed = manager.remove_source(id);
        assert!(removed.is_some());
        assert_eq!(manager.source_count(), 0);
    }

    #[test]
    fn test_spatial_params_calculation() {
        let mut manager = SpatialAudioManager::new();
        manager.set_listener_position(0.0, 0.0);
        // Listener is facing (1.0, 0.0) = right by default

        // Source to the listener's right (positive Y when facing +X)
        let mut source = SoundSource::new((0.0, 100.0));
        source.playing = true;

        let params = manager.calculate_spatial_params(&source);
        assert!(params.audible);
        assert!(params.right_volume > params.left_volume); // Panned right
    }

    #[test]
    fn test_distance_attenuation() {
        let mut manager = SpatialAudioManager::new();
        manager.set_listener_position(0.0, 0.0);

        // Near source
        let mut near = SoundSource::new((50.0, 0.0));
        near.playing = true;

        // Far source
        let mut far = SoundSource::new((500.0, 0.0));
        far.playing = true;

        let near_params = manager.calculate_spatial_params(&near);
        let far_params = manager.calculate_spatial_params(&far);

        let near_volume = near_params.left_volume + near_params.right_volume;
        let far_volume = far_params.left_volume + far_params.right_volume;
        assert!(near_volume > far_volume);
    }

    #[test]
    fn test_environment_params() {
        let mut manager = SpatialAudioManager::new();

        manager.set_environment(AudioEnvironment::Cave);
        let params = manager.get_environment_params();
        assert!(params.reverb_mix > 0.5);
        assert_eq!(params.environment, AudioEnvironment::Cave);

        manager.set_environment(AudioEnvironment::Underwater);
        let params = manager.get_environment_params();
        assert!(params.lowpass_cutoff < 1000.0); // Heavy filtering
    }

    #[test]
    fn test_out_of_range_not_audible() {
        let mut manager = SpatialAudioManager::new();
        manager.set_listener_position(0.0, 0.0);

        let mut source = SoundSource::new((2000.0, 0.0)); // Beyond default max distance
        source.playing = true;

        let params = manager.calculate_spatial_params(&source);
        assert!(!params.audible);
    }

    #[test]
    fn test_stopped_source_not_audible() {
        let mut manager = SpatialAudioManager::new();
        manager.set_listener_position(0.0, 0.0);

        let source = SoundSource::new((50.0, 0.0)); // Not playing

        let params = manager.calculate_spatial_params(&source);
        assert!(!params.audible);
    }
}
