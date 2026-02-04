//! 2D Spatial Audio System
//!
//! Implements distance-based volume attenuation and stereo panning for
//! positioned sound sources relative to a listener.
//!
//! # Overview
//!
//! The spatial audio system calculates how sounds should be heard based on:
//! - **Distance attenuation**: Sounds get quieter as they move away
//! - **Stereo panning**: Sounds pan left/right based on relative position
//! - **Doppler effect**: Pitch shifts based on relative velocity
//! - **Environment effects**: Reverb and filtering based on location
//!
//! # Example
//!
//! ```
//! use genesis_kernel::audio_spatial::{
//!     SpatialAudioProcessor, SoundSourceData, ListenerData, AttenuationModel
//! };
//!
//! let mut processor = SpatialAudioProcessor::new();
//!
//! // Set listener at center
//! processor.set_listener(ListenerData {
//!     position: (0.0, 0.0),
//!     velocity: (0.0, 0.0),
//!     direction: (1.0, 0.0), // Facing right
//!     volume: 1.0,
//! });
//!
//! // Calculate spatial params for a source
//! let source = SoundSourceData {
//!     position: (100.0, 50.0),
//!     velocity: (0.0, 0.0),
//!     volume: 1.0,
//!     pitch: 1.0,
//!     ref_distance: 100.0,
//!     max_distance: 1000.0,
//!     attenuation_model: AttenuationModel::Inverse,
//! };
//!
//! let params = processor.calculate(&source);
//! assert!(params.audible);
//! ```

use std::f32::consts::FRAC_PI_4;

use tracing::debug;

/// Speed of sound in world units per second (for Doppler calculations).
pub const SPEED_OF_SOUND: f32 = 343.0;

/// Default reference distance for attenuation.
pub const DEFAULT_REFERENCE_DISTANCE: f32 = 100.0;

/// Default maximum hearing distance.
pub const DEFAULT_MAX_DISTANCE: f32 = 1000.0;

/// Minimum volume threshold for audibility.
pub const MIN_AUDIBLE_VOLUME: f32 = 0.001;

/// Distance attenuation models for 2D spatial audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AttenuationModel {
    /// No distance attenuation - constant volume regardless of distance.
    None,
    /// Linear falloff from reference to max distance.
    Linear,
    /// Inverse distance (1/d) falloff - realistic but strong dropoff.
    #[default]
    Inverse,
    /// Exponential falloff - very steep dropoff.
    Exponential,
    /// Custom rolloff factor for inverse distance.
    InverseCustom {
        /// Rolloff factor (higher = steeper falloff).
        rolloff: u8,
    },
}

impl AttenuationModel {
    /// Calculate attenuation factor (0.0-1.0) for a given distance.
    ///
    /// # Arguments
    /// * `distance` - Distance from listener to source
    /// * `ref_distance` - Reference distance (full volume)
    /// * `max_distance` - Maximum hearing distance
    ///
    /// # Returns
    /// Attenuation factor between 0.0 and 1.0
    #[must_use]
    pub fn calculate(&self, distance: f32, ref_distance: f32, max_distance: f32) -> f32 {
        // Clamp distance to valid range
        let d = distance.clamp(0.0, max_distance);

        // Under reference distance = full volume
        if d <= ref_distance {
            return 1.0;
        }

        match self {
            Self::None => 1.0,
            Self::Linear => {
                // Linear interpolation from ref to max
                let range = max_distance - ref_distance;
                if range > 0.0 {
                    1.0 - ((d - ref_distance) / range)
                } else {
                    1.0
                }
            },
            Self::Inverse => {
                // Classic inverse distance
                ref_distance / d
            },
            Self::Exponential => {
                // Exponential decay: e^(-k * (d - ref) / ref)
                // At d=ref: 1.0, at d=2*ref: ~0.37, at d=3*ref: ~0.14
                let normalized = (d - ref_distance) / ref_distance;
                (-normalized).exp()
            },
            Self::InverseCustom { rolloff } => {
                // Inverse with custom rolloff factor
                let r = f32::from(*rolloff);
                1.0 / (1.0 + r * ((d - ref_distance) / ref_distance))
            },
        }
    }

    /// Get a human-readable name for this model.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Linear => "Linear",
            Self::Inverse => "Inverse",
            Self::Exponential => "Exponential",
            Self::InverseCustom { .. } => "Inverse (Custom)",
        }
    }
}

/// Audio environment types for reverb and filtering effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum AudioEnvironment {
    /// Open outdoor environment - minimal reverb.
    #[default]
    Outdoor = 0,
    /// Cave or underground - heavy reverb, long decay.
    Cave = 1,
    /// Small room - moderate reverb, short decay.
    Room = 2,
    /// Large hall - moderate reverb, medium decay.
    Hall = 3,
    /// Underwater - muffled sound, heavy low-pass filtering.
    Underwater = 4,
    /// Forest - natural reverb, some high-frequency absorption.
    Forest = 5,
    /// Metal interior - sharp reflections, bright reverb.
    Metal = 6,
}

impl AudioEnvironment {
    /// Create from u32 value with fallback to Outdoor.
    #[must_use]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Cave,
            2 => Self::Room,
            3 => Self::Hall,
            4 => Self::Underwater,
            5 => Self::Forest,
            6 => Self::Metal,
            _ => Self::Outdoor,
        }
    }

    /// Get reverb wet/dry mix (0.0-1.0).
    #[must_use]
    pub const fn reverb_mix(&self) -> f32 {
        match self {
            Self::Outdoor => 0.05,
            Self::Cave => 0.7,
            Self::Room => 0.3,
            Self::Hall => 0.5,
            Self::Underwater => 0.2,
            Self::Forest => 0.15,
            Self::Metal => 0.6,
        }
    }

    /// Get reverb decay time in seconds.
    #[must_use]
    pub const fn reverb_decay(&self) -> f32 {
        match self {
            Self::Outdoor => 0.3,
            Self::Cave => 3.5,
            Self::Room => 0.5,
            Self::Hall => 1.8,
            Self::Underwater => 1.0,
            Self::Forest => 0.8,
            Self::Metal => 2.0,
        }
    }

    /// Get low-pass filter cutoff frequency in Hz.
    #[must_use]
    pub const fn lowpass_cutoff(&self) -> f32 {
        match self {
            Self::Outdoor => 20000.0,
            Self::Cave => 8000.0,
            Self::Room => 16000.0,
            Self::Hall => 14000.0,
            Self::Underwater => 800.0,
            Self::Forest => 12000.0,
            Self::Metal => 18000.0,
        }
    }

    /// Get high-pass filter cutoff frequency in Hz.
    #[must_use]
    pub const fn highpass_cutoff(&self) -> f32 {
        match self {
            Self::Outdoor => 20.0,
            Self::Room | Self::Hall | Self::Forest => 30.0,
            Self::Cave | Self::Metal => 40.0,
            Self::Underwater => 100.0,
        }
    }

    /// Get pre-delay time in milliseconds for reverb.
    #[must_use]
    pub const fn predelay_ms(&self) -> f32 {
        match self {
            Self::Outdoor => 5.0,
            Self::Cave => 30.0,
            Self::Room => 10.0,
            Self::Hall => 25.0,
            Self::Underwater => 15.0,
            Self::Forest => 20.0,
            Self::Metal => 8.0,
        }
    }
}

/// Environment effect parameters for audio processing.
#[derive(Debug, Clone, Copy)]
pub struct EnvironmentParams {
    /// Reverb wet/dry mix (0.0-1.0).
    pub reverb_mix: f32,
    /// Reverb decay time in seconds.
    pub reverb_decay: f32,
    /// Low-pass filter cutoff frequency in Hz.
    pub lowpass_cutoff: f32,
    /// High-pass filter cutoff frequency in Hz.
    pub highpass_cutoff: f32,
    /// Pre-delay time in milliseconds.
    pub predelay_ms: f32,
    /// The environment type.
    pub environment: AudioEnvironment,
}

impl From<AudioEnvironment> for EnvironmentParams {
    fn from(env: AudioEnvironment) -> Self {
        Self {
            reverb_mix: env.reverb_mix(),
            reverb_decay: env.reverb_decay(),
            lowpass_cutoff: env.lowpass_cutoff(),
            highpass_cutoff: env.highpass_cutoff(),
            predelay_ms: env.predelay_ms(),
            environment: env,
        }
    }
}

/// Listener position and orientation for spatial calculations.
#[derive(Debug, Clone, Copy)]
pub struct ListenerData {
    /// Position in world coordinates.
    pub position: (f32, f32),
    /// Velocity for Doppler effect (world units per second).
    pub velocity: (f32, f32),
    /// Facing direction (normalized vector).
    pub direction: (f32, f32),
    /// Master volume multiplier (0.0-1.0).
    pub volume: f32,
}

impl Default for ListenerData {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            direction: (1.0, 0.0), // Facing right by default
            volume: 1.0,
        }
    }
}

impl ListenerData {
    /// Create a new listener at a position.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            velocity: (0.0, 0.0),
            direction: (1.0, 0.0),
            volume: 1.0,
        }
    }

    /// Create with position and direction.
    #[must_use]
    pub fn with_direction(mut self, dx: f32, dy: f32) -> Self {
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            self.direction = (dx / len, dy / len);
        }
        self
    }

    /// Create with velocity.
    #[must_use]
    pub const fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.velocity = (vx, vy);
        self
    }

    /// Create with volume.
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
}

/// Sound source data for spatial calculations.
#[derive(Debug, Clone, Copy)]
pub struct SoundSourceData {
    /// Position in world coordinates.
    pub position: (f32, f32),
    /// Velocity for Doppler effect (world units per second).
    pub velocity: (f32, f32),
    /// Base volume (0.0-1.0).
    pub volume: f32,
    /// Base pitch multiplier (1.0 = normal).
    pub pitch: f32,
    /// Reference distance where attenuation begins.
    pub ref_distance: f32,
    /// Maximum hearing distance.
    pub max_distance: f32,
    /// Distance attenuation model.
    pub attenuation_model: AttenuationModel,
}

impl Default for SoundSourceData {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            volume: 1.0,
            pitch: 1.0,
            ref_distance: DEFAULT_REFERENCE_DISTANCE,
            max_distance: DEFAULT_MAX_DISTANCE,
            attenuation_model: AttenuationModel::Inverse,
        }
    }
}

impl SoundSourceData {
    /// Create a new source at a position.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            velocity: (0.0, 0.0),
            volume: 1.0,
            pitch: 1.0,
            ref_distance: DEFAULT_REFERENCE_DISTANCE,
            max_distance: DEFAULT_MAX_DISTANCE,
            attenuation_model: AttenuationModel::Inverse,
        }
    }

    /// Set volume.
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Set velocity for Doppler effect.
    #[must_use]
    pub const fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.velocity = (vx, vy);
        self
    }

    /// Set attenuation parameters.
    #[must_use]
    pub const fn with_attenuation(
        mut self,
        model: AttenuationModel,
        ref_distance: f32,
        max_distance: f32,
    ) -> Self {
        self.attenuation_model = model;
        self.ref_distance = ref_distance;
        self.max_distance = max_distance;
        self
    }
}

/// Calculated spatial audio parameters for playback.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpatialParams {
    /// Left channel volume (0.0-1.0).
    pub left_volume: f32,
    /// Right channel volume (0.0-1.0).
    pub right_volume: f32,
    /// Final pitch multiplier (including Doppler).
    pub pitch: f32,
    /// Distance from listener.
    pub distance: f32,
    /// Pan position (-1.0 = full left, 0.0 = center, 1.0 = full right).
    pub pan: f32,
    /// Combined mono volume before panning.
    pub mono_volume: f32,
    /// Whether the sound is audible at all.
    pub audible: bool,
}

impl SpatialParams {
    /// Create non-audible params.
    #[must_use]
    pub const fn silent() -> Self {
        Self {
            left_volume: 0.0,
            right_volume: 0.0,
            pitch: 1.0,
            distance: 0.0,
            pan: 0.0,
            mono_volume: 0.0,
            audible: false,
        }
    }

    /// Get combined stereo volume.
    #[must_use]
    pub fn stereo_volume(&self) -> f32 {
        self.left_volume + self.right_volume
    }
}

/// 2D spatial audio processor.
///
/// Calculates stereo panning, distance attenuation, and Doppler effects
/// for sound sources relative to a listener.
#[derive(Debug)]
pub struct SpatialAudioProcessor {
    /// Current listener state.
    listener: ListenerData,
    /// Current environment.
    environment: AudioEnvironment,
    /// Whether Doppler effect is enabled.
    doppler_enabled: bool,
    /// Doppler effect strength (0.0 = none, 1.0 = realistic).
    doppler_factor: f32,
    /// Speed of sound for Doppler calculations.
    speed_of_sound: f32,
}

impl Default for SpatialAudioProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialAudioProcessor {
    /// Create a new spatial audio processor.
    #[must_use]
    pub fn new() -> Self {
        debug!("Created spatial audio processor");
        Self {
            listener: ListenerData::default(),
            environment: AudioEnvironment::Outdoor,
            doppler_enabled: true,
            doppler_factor: 1.0,
            speed_of_sound: SPEED_OF_SOUND,
        }
    }

    /// Set the listener state.
    pub fn set_listener(&mut self, listener: ListenerData) {
        self.listener = listener;
    }

    /// Update listener position.
    pub fn set_listener_position(&mut self, x: f32, y: f32) {
        self.listener.position = (x, y);
    }

    /// Update listener velocity.
    pub fn set_listener_velocity(&mut self, vx: f32, vy: f32) {
        self.listener.velocity = (vx, vy);
    }

    /// Update listener direction (will be normalized).
    pub fn set_listener_direction(&mut self, dx: f32, dy: f32) {
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            self.listener.direction = (dx / len, dy / len);
        }
    }

    /// Set master volume.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.listener.volume = volume.clamp(0.0, 1.0);
    }

    /// Get the current listener state.
    #[must_use]
    pub const fn listener(&self) -> &ListenerData {
        &self.listener
    }

    /// Set the audio environment.
    pub fn set_environment(&mut self, env: AudioEnvironment) {
        self.environment = env;
    }

    /// Get the current environment.
    #[must_use]
    pub const fn environment(&self) -> AudioEnvironment {
        self.environment
    }

    /// Enable or disable Doppler effect.
    pub fn set_doppler_enabled(&mut self, enabled: bool) {
        self.doppler_enabled = enabled;
    }

    /// Set Doppler effect strength.
    pub fn set_doppler_factor(&mut self, factor: f32) {
        self.doppler_factor = factor.clamp(0.0, 2.0);
    }

    /// Set speed of sound for Doppler calculations.
    pub fn set_speed_of_sound(&mut self, speed: f32) {
        self.speed_of_sound = speed.max(1.0);
    }

    /// Get environment effect parameters.
    #[must_use]
    pub fn environment_params(&self) -> EnvironmentParams {
        EnvironmentParams::from(self.environment)
    }

    /// Calculate spatial parameters for a sound source.
    #[must_use]
    pub fn calculate(&self, source: &SoundSourceData) -> SpatialParams {
        // Calculate distance
        let dx = source.position.0 - self.listener.position.0;
        let dy = source.position.1 - self.listener.position.1;
        let distance = (dx * dx + dy * dy).sqrt();

        // Check if out of range
        if distance > source.max_distance {
            return SpatialParams {
                distance,
                ..SpatialParams::silent()
            };
        }

        // Calculate distance attenuation
        let attenuation =
            source
                .attenuation_model
                .calculate(distance, source.ref_distance, source.max_distance);

        // Calculate base volume
        let mono_volume = source.volume * attenuation * self.listener.volume;

        // Check if audible
        if mono_volume < MIN_AUDIBLE_VOLUME {
            return SpatialParams {
                distance,
                mono_volume,
                audible: false,
                ..SpatialParams::silent()
            };
        }

        // Calculate panning angle
        let pan = if distance > 0.01 {
            // Calculate direction to source
            let to_source = (dx / distance, dy / distance);

            // Cross product gives sin of angle (positive = right, negative = left)
            let cross =
                self.listener.direction.0 * to_source.1 - self.listener.direction.1 * to_source.0;

            // Clamp to valid pan range
            cross.clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Calculate per-channel volumes using constant power panning
        // This preserves perceived loudness across the stereo field
        let pan_angle = (pan + 1.0) * FRAC_PI_4; // Map -1..1 to 0..PI/2
        let left_volume = mono_volume * pan_angle.cos();
        let right_volume = mono_volume * pan_angle.sin();

        // Calculate Doppler pitch shift
        let pitch = if self.doppler_enabled && distance > 0.01 {
            self.calculate_doppler(source, dx, dy, distance)
        } else {
            source.pitch
        };

        SpatialParams {
            left_volume,
            right_volume,
            pitch,
            distance,
            pan,
            mono_volume,
            audible: true,
        }
    }

    /// Calculate Doppler pitch shift.
    fn calculate_doppler(&self, source: &SoundSourceData, dx: f32, dy: f32, distance: f32) -> f32 {
        // Direction from listener to source
        let direction = (dx / distance, dy / distance);

        // Project velocities onto the direction vector
        // Positive = moving toward each other, negative = moving apart
        let listener_vel =
            self.listener.velocity.0 * direction.0 + self.listener.velocity.1 * direction.1;
        let source_vel = source.velocity.0 * direction.0 + source.velocity.1 * direction.1;

        // Apply Doppler formula: f' = f * (c + v_listener) / (c + v_source)
        // v_listener > 0 when listener moves toward source (higher pitch)
        // v_source > 0 when source moves toward listener (higher pitch)
        let numerator = self.speed_of_sound + listener_vel * self.doppler_factor;
        let denominator = self.speed_of_sound + source_vel * self.doppler_factor;

        // Avoid division by very small numbers
        if denominator.abs() > 0.01 {
            source.pitch * (numerator / denominator)
        } else {
            source.pitch
        }
    }

    /// Batch calculate spatial parameters for multiple sources.
    ///
    /// Returns only audible sources, sorted by volume (loudest first).
    pub fn calculate_batch(&self, sources: &[SoundSourceData]) -> Vec<SpatialParams> {
        let mut results: Vec<_> = sources
            .iter()
            .map(|s| self.calculate(s))
            .filter(|p| p.audible)
            .collect();

        // Sort by combined volume (loudest first)
        results.sort_by(|a, b| {
            b.stereo_volume()
                .partial_cmp(&a.stereo_volume())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Calculate spatial params and return the N loudest.
    pub fn calculate_loudest(
        &self,
        sources: &[SoundSourceData],
        count: usize,
    ) -> Vec<SpatialParams> {
        let mut results = self.calculate_batch(sources);
        results.truncate(count);
        results
    }
}

/// Calculate interpolated pan for crossfading between two positions.
#[must_use]
pub fn lerp_pan(from_pan: f32, to_pan: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    from_pan + (to_pan - from_pan) * t
}

/// Calculate interpolated volume for crossfading.
#[must_use]
pub fn lerp_volume(from_vol: f32, to_vol: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // Use sqrt for perceptually linear crossfade
    let from_gain = from_vol * (1.0 - t).sqrt();
    let to_gain = to_vol * t.sqrt();
    from_gain + to_gain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attenuation_none() {
        let model = AttenuationModel::None;
        assert!((model.calculate(0.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);
        assert!((model.calculate(500.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);
        assert!((model.calculate(1000.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_attenuation_linear() {
        let model = AttenuationModel::Linear;

        // Under reference = full volume
        assert!((model.calculate(50.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);

        // At reference = full volume
        assert!((model.calculate(100.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);

        // Midway between ref and max
        let mid = model.calculate(550.0, 100.0, 1000.0);
        assert!((mid - 0.5).abs() < 0.01);

        // At max = silent
        assert!((model.calculate(1000.0, 100.0, 1000.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_attenuation_inverse() {
        let model = AttenuationModel::Inverse;

        // Under reference = full volume
        assert!((model.calculate(50.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);

        // At 2x reference = half volume
        assert!((model.calculate(200.0, 100.0, 1000.0) - 0.5).abs() < f32::EPSILON);

        // At 4x reference = quarter volume
        assert!((model.calculate(400.0, 100.0, 1000.0) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_attenuation_exponential() {
        let model = AttenuationModel::Exponential;

        // Under reference = full volume
        assert!((model.calculate(50.0, 100.0, 1000.0) - 1.0).abs() < f32::EPSILON);

        // Exponential falls off faster than inverse
        let exp_vol = model.calculate(200.0, 100.0, 1000.0);
        let inv_vol = AttenuationModel::Inverse.calculate(200.0, 100.0, 1000.0);
        assert!(exp_vol < inv_vol);
    }

    #[test]
    fn test_environment_params() {
        let cave = AudioEnvironment::Cave;
        assert!(cave.reverb_mix() > 0.5);
        assert!(cave.reverb_decay() > 2.0);

        let underwater = AudioEnvironment::Underwater;
        assert!(underwater.lowpass_cutoff() < 1000.0);
    }

    #[test]
    fn test_listener_creation() {
        let listener = ListenerData::new(100.0, 200.0)
            .with_direction(0.0, 1.0)
            .with_volume(0.8);

        assert!((listener.position.0 - 100.0).abs() < f32::EPSILON);
        assert!((listener.position.1 - 200.0).abs() < f32::EPSILON);
        assert!((listener.direction.1 - 1.0).abs() < f32::EPSILON);
        assert!((listener.volume - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_source_creation() {
        let source = SoundSourceData::new(50.0, 75.0)
            .with_volume(0.5)
            .with_velocity(10.0, 0.0);

        assert!((source.position.0 - 50.0).abs() < f32::EPSILON);
        assert!((source.position.1 - 75.0).abs() < f32::EPSILON);
        assert!((source.volume - 0.5).abs() < f32::EPSILON);
        assert!((source.velocity.0 - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_processor_creation() {
        let processor = SpatialAudioProcessor::new();
        assert!(processor.doppler_enabled);
        assert_eq!(processor.environment(), AudioEnvironment::Outdoor);
    }

    #[test]
    fn test_spatial_panning_right() {
        let mut processor = SpatialAudioProcessor::new();
        processor.set_listener(ListenerData::new(0.0, 0.0).with_direction(1.0, 0.0));

        // Source to the right (positive Y when facing +X)
        let source = SoundSourceData::new(0.0, 100.0);
        let params = processor.calculate(&source);

        assert!(params.audible);
        assert!(params.right_volume > params.left_volume);
        assert!(params.pan > 0.0);
    }

    #[test]
    fn test_spatial_panning_left() {
        let mut processor = SpatialAudioProcessor::new();
        processor.set_listener(ListenerData::new(0.0, 0.0).with_direction(1.0, 0.0));

        // Source to the left (negative Y when facing +X)
        let source = SoundSourceData::new(0.0, -100.0);
        let params = processor.calculate(&source);

        assert!(params.audible);
        assert!(params.left_volume > params.right_volume);
        assert!(params.pan < 0.0);
    }

    #[test]
    fn test_spatial_center() {
        let mut processor = SpatialAudioProcessor::new();
        processor.set_listener(ListenerData::new(0.0, 0.0).with_direction(1.0, 0.0));

        // Source directly in front
        let source = SoundSourceData::new(100.0, 0.0);
        let params = processor.calculate(&source);

        assert!(params.audible);
        assert!((params.pan).abs() < 0.01); // Center
        assert!((params.left_volume - params.right_volume).abs() < 0.01);
    }

    #[test]
    fn test_distance_attenuation() {
        let processor = SpatialAudioProcessor::new();

        let near = SoundSourceData::new(50.0, 0.0);
        let far = SoundSourceData::new(500.0, 0.0);

        let near_params = processor.calculate(&near);
        let far_params = processor.calculate(&far);

        assert!(near_params.mono_volume > far_params.mono_volume);
    }

    #[test]
    fn test_out_of_range() {
        let processor = SpatialAudioProcessor::new();

        let source = SoundSourceData::new(2000.0, 0.0); // Beyond default max
        let params = processor.calculate(&source);

        assert!(!params.audible);
    }

    #[test]
    fn test_doppler_approaching() {
        let mut processor = SpatialAudioProcessor::new();
        processor.set_listener(ListenerData::new(0.0, 0.0));

        // Source approaching listener
        let source = SoundSourceData::new(200.0, 0.0).with_velocity(-50.0, 0.0);
        let params = processor.calculate(&source);

        // Approaching = higher pitch
        assert!(params.pitch > 1.0);
    }

    #[test]
    fn test_doppler_receding() {
        let mut processor = SpatialAudioProcessor::new();
        processor.set_listener(ListenerData::new(0.0, 0.0));

        // Source moving away from listener
        let source = SoundSourceData::new(200.0, 0.0).with_velocity(50.0, 0.0);
        let params = processor.calculate(&source);

        // Receding = lower pitch
        assert!(params.pitch < 1.0);
    }

    #[test]
    fn test_doppler_disabled() {
        let mut processor = SpatialAudioProcessor::new();
        processor.set_doppler_enabled(false);

        let source = SoundSourceData::new(200.0, 0.0).with_velocity(-100.0, 0.0);
        let params = processor.calculate(&source);

        // Should be normal pitch when Doppler disabled
        assert!((params.pitch - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_batch_calculate() {
        let processor = SpatialAudioProcessor::new();

        let sources = vec![
            SoundSourceData::new(100.0, 0.0),  // Loudest (closest)
            SoundSourceData::new(500.0, 0.0),  // Medium
            SoundSourceData::new(2000.0, 0.0), // Out of range
        ];

        let results = processor.calculate_batch(&sources);

        // Should only include 2 audible sources
        assert_eq!(results.len(), 2);
        // Should be sorted by volume (closest first)
        assert!(results[0].mono_volume > results[1].mono_volume);
    }

    #[test]
    fn test_lerp_pan() {
        assert!((lerp_pan(-1.0, 1.0, 0.0) - (-1.0)).abs() < f32::EPSILON);
        assert!((lerp_pan(-1.0, 1.0, 0.5) - 0.0).abs() < f32::EPSILON);
        assert!((lerp_pan(-1.0, 1.0, 1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lerp_volume() {
        // At t=0, should be from_vol
        let v0 = lerp_volume(1.0, 0.0, 0.0);
        assert!((v0 - 1.0).abs() < f32::EPSILON);

        // At t=1, should be to_vol
        let v1 = lerp_volume(1.0, 0.0, 1.0);
        assert!((v1 - 0.0).abs() < f32::EPSILON);

        // At t=0.5, should be roughly equal (constant power)
        let v_mid = lerp_volume(1.0, 1.0, 0.5);
        assert!(v_mid > 0.9 && v_mid < 1.5); // Roughly preserves energy
    }
}
