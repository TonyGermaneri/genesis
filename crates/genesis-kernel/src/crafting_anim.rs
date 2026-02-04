//! Crafting Animation System
//!
//! This module provides the low-level infrastructure for crafting animations
//! including progress tracking, particle emission, and sound triggers.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐     ┌────────────────┐
//! │ CraftingProgress│────▶│ ParticleEmitter │────▶│ SoundTrigger   │
//! │ (0.0 - 1.0)     │     │ (visual fx)     │     │ (audio cues)   │
//! └─────────────────┘     └─────────────────┘     └────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use genesis_kernel::crafting_anim::{CraftingProgress, AnimationState, SoundTrigger};
//!
//! // Create a crafting progress tracker
//! let mut progress = CraftingProgress::new(2.0); // 2 second craft time
//! progress.start();
//!
//! // Update progress
//! progress.update(0.5); // 500ms elapsed
//! assert!((progress.ratio() - 0.25).abs() < 0.01);
//!
//! // Check for sound triggers
//! let triggers = progress.check_sound_triggers();
//! ```

use tracing::trace;

/// Default craft duration in seconds.
pub const DEFAULT_CRAFT_DURATION: f32 = 1.0;

/// Progress value type (0.0 to 1.0).
pub type ProgressValue = f32;

/// Animation state for crafting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AnimationState {
    /// Not started.
    #[default]
    Idle = 0,
    /// Crafting in progress.
    InProgress = 1,
    /// Paused (interrupted).
    Paused = 2,
    /// Completed successfully.
    Completed = 3,
    /// Failed/cancelled.
    Failed = 4,
}

impl AnimationState {
    /// Check if the state is active (in progress or paused).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::InProgress | Self::Paused)
    }

    /// Check if the state is terminal (completed or failed).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Check if currently crafting.
    #[must_use]
    pub const fn is_crafting(&self) -> bool {
        matches!(self, Self::InProgress)
    }
}

/// Sound trigger point during crafting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundTrigger {
    /// Progress point (0.0 - 1.0) when sound triggers.
    pub progress: f32,
    /// Sound ID to play.
    pub sound_id: u32,
    /// Volume multiplier.
    pub volume: f32,
    /// Pitch variation.
    pub pitch: f32,
    /// Whether this trigger has fired.
    pub fired: bool,
}

impl SoundTrigger {
    /// Create a new sound trigger.
    #[must_use]
    pub const fn new(progress: f32, sound_id: u32) -> Self {
        Self {
            progress,
            sound_id,
            volume: 1.0,
            pitch: 1.0,
            fired: false,
        }
    }

    /// Set volume.
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Set pitch.
    #[must_use]
    pub const fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    /// Check if trigger should fire at given progress.
    #[must_use]
    pub fn should_fire(&self, current_progress: f32) -> bool {
        !self.fired && current_progress >= self.progress
    }

    /// Mark as fired.
    pub fn fire(&mut self) {
        self.fired = true;
    }

    /// Reset the trigger.
    pub fn reset(&mut self) {
        self.fired = false;
    }
}

/// Particle type for crafting effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum CraftingParticleType {
    /// No particles.
    #[default]
    None = 0,
    /// Smoke particles.
    Smoke = 1,
    /// Spark particles.
    Spark = 2,
    /// Fire particles.
    Fire = 3,
    /// Steam particles.
    Steam = 4,
    /// Magic/enchant particles.
    Magic = 5,
    /// Item fragments.
    Fragments = 6,
    /// Dust/powder.
    Dust = 7,
    /// Bubbles (for alchemy).
    Bubbles = 8,
    /// Custom type.
    Custom(u8) = 255,
}

impl CraftingParticleType {
    /// Get the particle type ID.
    #[must_use]
    pub const fn id(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Smoke => 1,
            Self::Spark => 2,
            Self::Fire => 3,
            Self::Steam => 4,
            Self::Magic => 5,
            Self::Fragments => 6,
            Self::Dust => 7,
            Self::Bubbles => 8,
            Self::Custom(id) => *id,
        }
    }

    /// Create from ID.
    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        match id {
            0 => Self::None,
            1 => Self::Smoke,
            2 => Self::Spark,
            3 => Self::Fire,
            4 => Self::Steam,
            5 => Self::Magic,
            6 => Self::Fragments,
            7 => Self::Dust,
            8 => Self::Bubbles,
            other => Self::Custom(other),
        }
    }
}

/// Particle emitter configuration for crafting.
#[derive(Debug, Clone)]
pub struct CraftingParticleEmitter {
    /// Particle type.
    pub particle_type: CraftingParticleType,
    /// Emission position offset from station.
    pub offset: [f32; 3],
    /// Emission rate (particles per second).
    pub rate: f32,
    /// Particle lifetime in seconds.
    pub lifetime: f32,
    /// Initial velocity.
    pub velocity: [f32; 3],
    /// Velocity randomness.
    pub velocity_spread: f32,
    /// Particle size.
    pub size: f32,
    /// Size variation.
    pub size_spread: f32,
    /// Color (RGBA, 0-1).
    pub color: [f32; 4],
    /// Progress range when active (start, end).
    pub progress_range: (f32, f32),
    /// Whether the emitter is active.
    pub active: bool,
}

impl Default for CraftingParticleEmitter {
    fn default() -> Self {
        Self {
            particle_type: CraftingParticleType::None,
            offset: [0.0, 0.5, 0.0],
            rate: 10.0,
            lifetime: 1.0,
            velocity: [0.0, 1.0, 0.0],
            velocity_spread: 0.2,
            size: 0.1,
            size_spread: 0.02,
            color: [1.0, 1.0, 1.0, 1.0],
            progress_range: (0.0, 1.0),
            active: true,
        }
    }
}

impl CraftingParticleEmitter {
    /// Create a new particle emitter.
    #[must_use]
    pub fn new(particle_type: CraftingParticleType) -> Self {
        Self {
            particle_type,
            ..Default::default()
        }
    }

    /// Create a smoke emitter preset.
    #[must_use]
    pub fn smoke() -> Self {
        Self {
            particle_type: CraftingParticleType::Smoke,
            offset: [0.0, 0.8, 0.0],
            rate: 15.0,
            lifetime: 2.0,
            velocity: [0.0, 0.5, 0.0],
            velocity_spread: 0.3,
            size: 0.15,
            size_spread: 0.05,
            color: [0.5, 0.5, 0.5, 0.7],
            progress_range: (0.0, 1.0),
            active: true,
        }
    }

    /// Create a spark emitter preset.
    #[must_use]
    pub fn sparks() -> Self {
        Self {
            particle_type: CraftingParticleType::Spark,
            offset: [0.0, 0.3, 0.0],
            rate: 30.0,
            lifetime: 0.5,
            velocity: [0.0, 2.0, 0.0],
            velocity_spread: 1.5,
            size: 0.03,
            size_spread: 0.01,
            color: [1.0, 0.8, 0.2, 1.0],
            progress_range: (0.0, 1.0),
            active: true,
        }
    }

    /// Create a magic emitter preset.
    #[must_use]
    pub fn magic() -> Self {
        Self {
            particle_type: CraftingParticleType::Magic,
            offset: [0.0, 0.5, 0.0],
            rate: 20.0,
            lifetime: 1.5,
            velocity: [0.0, 0.3, 0.0],
            velocity_spread: 0.5,
            size: 0.08,
            size_spread: 0.03,
            color: [0.5, 0.2, 1.0, 0.9],
            progress_range: (0.0, 1.0),
            active: true,
        }
    }

    /// Set the offset.
    #[must_use]
    pub fn with_offset(mut self, offset: [f32; 3]) -> Self {
        self.offset = offset;
        self
    }

    /// Set the emission rate.
    #[must_use]
    pub fn with_rate(mut self, rate: f32) -> Self {
        self.rate = rate.max(0.0);
        self
    }

    /// Set the particle color.
    #[must_use]
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the progress range.
    #[must_use]
    pub fn with_progress_range(mut self, start: f32, end: f32) -> Self {
        self.progress_range = (start.clamp(0.0, 1.0), end.clamp(0.0, 1.0));
        self
    }

    /// Check if emitter should be active at given progress.
    #[must_use]
    pub fn is_active_at(&self, progress: f32) -> bool {
        self.active && progress >= self.progress_range.0 && progress <= self.progress_range.1
    }

    /// Calculate particles to emit this frame.
    #[must_use]
    pub fn particles_this_frame(&self, delta_time: f32) -> u32 {
        if !self.active || self.rate <= 0.0 {
            return 0;
        }
        (self.rate * delta_time).ceil() as u32
    }
}

/// Crafting progress tracker.
#[derive(Debug, Clone)]
pub struct CraftingProgress {
    /// Current progress (0.0 - 1.0).
    progress: f32,
    /// Total craft duration in seconds.
    duration: f32,
    /// Elapsed time in seconds.
    elapsed: f32,
    /// Current animation state.
    state: AnimationState,
    /// Sound triggers.
    sound_triggers: Vec<SoundTrigger>,
    /// Particle emitters.
    particle_emitters: Vec<CraftingParticleEmitter>,
    /// Speed multiplier (from workbench).
    speed_multiplier: f32,
    /// Whether progress is reversed (uncrafting).
    reversed: bool,
}

impl Default for CraftingProgress {
    fn default() -> Self {
        Self::new(DEFAULT_CRAFT_DURATION)
    }
}

impl CraftingProgress {
    /// Create a new crafting progress tracker.
    #[must_use]
    pub fn new(duration: f32) -> Self {
        Self {
            progress: 0.0,
            duration: duration.max(0.001),
            elapsed: 0.0,
            state: AnimationState::Idle,
            sound_triggers: Vec::new(),
            particle_emitters: Vec::new(),
            speed_multiplier: 1.0,
            reversed: false,
        }
    }

    /// Create with default sounds for a workbench type.
    #[must_use]
    pub fn with_default_sounds(duration: f32, start_sound: u32, complete_sound: u32) -> Self {
        let mut progress = Self::new(duration);
        progress.add_sound_trigger(SoundTrigger::new(0.0, start_sound));
        progress.add_sound_trigger(SoundTrigger::new(1.0, complete_sound));
        progress
    }

    /// Get current progress (0.0 - 1.0).
    #[must_use]
    pub const fn ratio(&self) -> f32 {
        self.progress
    }

    /// Get progress as percentage (0 - 100).
    #[must_use]
    pub fn percentage(&self) -> u8 {
        (self.progress * 100.0).round() as u8
    }

    /// Get elapsed time.
    #[must_use]
    pub const fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Get remaining time.
    #[must_use]
    pub fn remaining(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0) / self.speed_multiplier
    }

    /// Get total duration.
    #[must_use]
    pub const fn duration(&self) -> f32 {
        self.duration
    }

    /// Get current animation state.
    #[must_use]
    pub const fn state(&self) -> AnimationState {
        self.state
    }

    /// Check if crafting is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0 || self.state == AnimationState::Completed
    }

    /// Check if crafting is in progress.
    #[must_use]
    pub const fn is_in_progress(&self) -> bool {
        self.state.is_crafting()
    }

    /// Set the speed multiplier.
    pub fn set_speed_multiplier(&mut self, mult: f32) {
        self.speed_multiplier = mult.max(0.01);
    }

    /// Get the speed multiplier.
    #[must_use]
    pub const fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier
    }

    /// Start crafting.
    pub fn start(&mut self) {
        if self.state == AnimationState::Idle || self.state == AnimationState::Failed {
            self.state = AnimationState::InProgress;
            self.progress = 0.0;
            self.elapsed = 0.0;
            self.reset_triggers();
            trace!("Crafting started, duration: {}s", self.duration);
        }
    }

    /// Pause crafting.
    pub fn pause(&mut self) {
        if self.state == AnimationState::InProgress {
            self.state = AnimationState::Paused;
            trace!("Crafting paused at {}%", self.percentage());
        }
    }

    /// Resume crafting.
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::InProgress;
            trace!("Crafting resumed at {}%", self.percentage());
        }
    }

    /// Cancel crafting.
    pub fn cancel(&mut self) {
        if self.state.is_active() {
            self.state = AnimationState::Failed;
            trace!("Crafting cancelled at {}%", self.percentage());
        }
    }

    /// Reset to idle state.
    pub fn reset(&mut self) {
        self.state = AnimationState::Idle;
        self.progress = 0.0;
        self.elapsed = 0.0;
        self.reset_triggers();
    }

    /// Update progress with delta time.
    ///
    /// Returns true if crafting completed this frame.
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.state != AnimationState::InProgress {
            return false;
        }

        let effective_delta = delta_time * self.speed_multiplier;

        if self.reversed {
            self.elapsed = (self.elapsed - effective_delta).max(0.0);
        } else {
            self.elapsed += effective_delta;
        }

        self.progress = (self.elapsed / self.duration).clamp(0.0, 1.0);

        if self.progress >= 1.0 {
            self.state = AnimationState::Completed;
            trace!("Crafting completed");
            return true;
        }

        false
    }

    /// Set progress directly (for network sync).
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.elapsed = self.progress * self.duration;

        if self.progress >= 1.0 {
            self.state = AnimationState::Completed;
        }
    }

    /// Add a sound trigger.
    pub fn add_sound_trigger(&mut self, trigger: SoundTrigger) {
        self.sound_triggers.push(trigger);
        // Sort by progress for efficient checking
        self.sound_triggers
            .sort_by(|a, b| a.progress.partial_cmp(&b.progress).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Check and return sound triggers that should fire.
    pub fn check_sound_triggers(&mut self) -> Vec<SoundTrigger> {
        let mut fired = Vec::new();

        for trigger in &mut self.sound_triggers {
            if trigger.should_fire(self.progress) {
                trigger.fire();
                fired.push(*trigger);
            }
        }

        fired
    }

    /// Get all sound triggers.
    #[must_use]
    pub fn sound_triggers(&self) -> &[SoundTrigger] {
        &self.sound_triggers
    }

    /// Reset all triggers.
    pub fn reset_triggers(&mut self) {
        for trigger in &mut self.sound_triggers {
            trigger.reset();
        }
    }

    /// Add a particle emitter.
    pub fn add_particle_emitter(&mut self, emitter: CraftingParticleEmitter) {
        self.particle_emitters.push(emitter);
    }

    /// Get active particle emitters.
    #[must_use]
    pub fn active_emitters(&self) -> Vec<&CraftingParticleEmitter> {
        self.particle_emitters
            .iter()
            .filter(|e| e.is_active_at(self.progress))
            .collect()
    }

    /// Get all particle emitters.
    #[must_use]
    pub fn particle_emitters(&self) -> &[CraftingParticleEmitter] {
        &self.particle_emitters
    }

    /// Get mutable particle emitters.
    pub fn particle_emitters_mut(&mut self) -> &mut Vec<CraftingParticleEmitter> {
        &mut self.particle_emitters
    }

    /// Set reversed mode (for uncrafting).
    pub fn set_reversed(&mut self, reversed: bool) {
        self.reversed = reversed;
    }

    /// Check if reversed.
    #[must_use]
    pub const fn is_reversed(&self) -> bool {
        self.reversed
    }
}

/// Crafting animation preset for different workbench types.
#[derive(Debug, Clone)]
pub struct AnimationPreset {
    /// Sound trigger on start.
    pub start_sound: Option<u32>,
    /// Sound trigger on complete.
    pub complete_sound: Option<u32>,
    /// Periodic sound (and interval).
    pub periodic_sound: Option<(u32, f32)>,
    /// Particle emitters.
    pub emitters: Vec<CraftingParticleEmitter>,
}

impl AnimationPreset {
    /// Create an empty preset.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            start_sound: None,
            complete_sound: None,
            periodic_sound: None,
            emitters: Vec::new(),
        }
    }

    /// Create a forge preset (sparks + fire sounds).
    #[must_use]
    pub fn forge() -> Self {
        Self {
            start_sound: Some(100), // fire_start
            complete_sound: Some(101), // anvil_ring
            periodic_sound: Some((102, 0.3)), // hammer_hit
            emitters: vec![
                CraftingParticleEmitter::sparks(),
                CraftingParticleEmitter::smoke().with_progress_range(0.2, 1.0),
            ],
        }
    }

    /// Create an alchemy preset (bubbles + magic).
    #[must_use]
    pub fn alchemy() -> Self {
        Self {
            start_sound: Some(200), // bubble_start
            complete_sound: Some(201), // magic_complete
            periodic_sound: Some((202, 0.5)), // bubble_pop
            emitters: vec![
                CraftingParticleEmitter::new(CraftingParticleType::Bubbles)
                    .with_rate(25.0)
                    .with_color([0.2, 0.5, 1.0, 0.8]),
                CraftingParticleEmitter::magic().with_progress_range(0.8, 1.0),
            ],
        }
    }

    /// Create an enchanting preset.
    #[must_use]
    pub fn enchanting() -> Self {
        Self {
            start_sound: Some(300), // enchant_start
            complete_sound: Some(301), // enchant_complete
            periodic_sound: None,
            emitters: vec![CraftingParticleEmitter::magic()
                .with_color([0.8, 0.3, 1.0, 0.9])
                .with_rate(30.0)],
        }
    }

    /// Create a basic crafting preset.
    #[must_use]
    pub fn basic() -> Self {
        Self {
            start_sound: Some(1), // craft_start
            complete_sound: Some(2), // craft_complete
            periodic_sound: None,
            emitters: vec![],
        }
    }

    /// Apply this preset to a crafting progress tracker.
    pub fn apply_to(&self, progress: &mut CraftingProgress) {
        // Add start sound
        if let Some(sound_id) = self.start_sound {
            progress.add_sound_trigger(SoundTrigger::new(0.0, sound_id));
        }

        // Add complete sound
        if let Some(sound_id) = self.complete_sound {
            progress.add_sound_trigger(SoundTrigger::new(1.0, sound_id));
        }

        // Add periodic sounds
        if let Some((sound_id, interval)) = self.periodic_sound {
            let mut p = interval;
            while p < 1.0 {
                progress.add_sound_trigger(SoundTrigger::new(p, sound_id));
                p += interval;
            }
        }

        // Add emitters
        for emitter in &self.emitters {
            progress.add_particle_emitter(emitter.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_state() {
        assert!(AnimationState::InProgress.is_active());
        assert!(AnimationState::Paused.is_active());
        assert!(!AnimationState::Completed.is_active());

        assert!(AnimationState::Completed.is_terminal());
        assert!(AnimationState::Failed.is_terminal());
        assert!(!AnimationState::InProgress.is_terminal());
    }

    #[test]
    fn test_crafting_progress_creation() {
        let progress = CraftingProgress::new(2.0);
        assert_eq!(progress.state(), AnimationState::Idle);
        assert!((progress.duration() - 2.0).abs() < f32::EPSILON);
        assert!((progress.ratio() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crafting_progress_update() {
        let mut progress = CraftingProgress::new(2.0);
        progress.start();

        assert!(progress.is_in_progress());

        // Update 1 second
        let completed = progress.update(1.0);
        assert!(!completed);
        assert!((progress.ratio() - 0.5).abs() < f32::EPSILON);
        assert_eq!(progress.percentage(), 50);

        // Update another 1 second
        let completed = progress.update(1.0);
        assert!(completed);
        assert!(progress.is_complete());
        assert_eq!(progress.state(), AnimationState::Completed);
    }

    #[test]
    fn test_crafting_progress_pause_resume() {
        let mut progress = CraftingProgress::new(2.0);
        progress.start();
        progress.update(0.5);

        progress.pause();
        assert_eq!(progress.state(), AnimationState::Paused);

        // Progress shouldn't change while paused
        progress.update(1.0);
        assert!((progress.ratio() - 0.25).abs() < f32::EPSILON);

        progress.resume();
        assert_eq!(progress.state(), AnimationState::InProgress);

        progress.update(0.5);
        assert!((progress.ratio() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crafting_progress_cancel() {
        let mut progress = CraftingProgress::new(2.0);
        progress.start();
        progress.update(0.5);

        progress.cancel();
        assert_eq!(progress.state(), AnimationState::Failed);
    }

    #[test]
    fn test_crafting_progress_speed_multiplier() {
        let mut progress = CraftingProgress::new(2.0);
        progress.set_speed_multiplier(2.0);
        progress.start();

        // At 2x speed, 1 second should be 100% progress
        let completed = progress.update(1.0);
        assert!(completed);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_sound_triggers() {
        let mut progress = CraftingProgress::new(1.0);
        progress.add_sound_trigger(SoundTrigger::new(0.0, 1));
        progress.add_sound_trigger(SoundTrigger::new(0.5, 2));
        progress.add_sound_trigger(SoundTrigger::new(1.0, 3));

        progress.start();

        // At start, first trigger should fire
        let fired = progress.check_sound_triggers();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].sound_id, 1);

        // At 50%, second trigger should fire
        progress.update(0.5);
        let fired = progress.check_sound_triggers();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].sound_id, 2);

        // At 100%, third trigger should fire
        progress.update(0.5);
        let fired = progress.check_sound_triggers();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].sound_id, 3);
    }

    #[test]
    fn test_particle_emitter() {
        let emitter = CraftingParticleEmitter::smoke()
            .with_progress_range(0.2, 0.8);

        assert!(!emitter.is_active_at(0.1));
        assert!(emitter.is_active_at(0.5));
        assert!(!emitter.is_active_at(0.9));
    }

    #[test]
    fn test_particle_emitter_presets() {
        let smoke = CraftingParticleEmitter::smoke();
        assert_eq!(smoke.particle_type, CraftingParticleType::Smoke);

        let sparks = CraftingParticleEmitter::sparks();
        assert_eq!(sparks.particle_type, CraftingParticleType::Spark);

        let magic = CraftingParticleEmitter::magic();
        assert_eq!(magic.particle_type, CraftingParticleType::Magic);
    }

    #[test]
    fn test_active_emitters() {
        let mut progress = CraftingProgress::new(1.0);
        progress.add_particle_emitter(
            CraftingParticleEmitter::smoke().with_progress_range(0.0, 0.5)
        );
        progress.add_particle_emitter(
            CraftingParticleEmitter::sparks().with_progress_range(0.5, 1.0)
        );

        progress.start();

        // At 25%, only smoke should be active
        progress.set_progress(0.25);
        let active = progress.active_emitters();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].particle_type, CraftingParticleType::Smoke);

        // At 75%, only sparks should be active
        progress.set_progress(0.75);
        let active = progress.active_emitters();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].particle_type, CraftingParticleType::Spark);
    }

    #[test]
    fn test_animation_preset_forge() {
        let mut progress = CraftingProgress::new(3.0);
        let preset = AnimationPreset::forge();
        preset.apply_to(&mut progress);

        assert!(!progress.sound_triggers().is_empty());
        assert!(!progress.particle_emitters().is_empty());
    }

    #[test]
    fn test_animation_preset_alchemy() {
        let mut progress = CraftingProgress::new(2.0);
        let preset = AnimationPreset::alchemy();
        preset.apply_to(&mut progress);

        assert!(!progress.sound_triggers().is_empty());
        assert_eq!(progress.particle_emitters().len(), 2);
    }

    #[test]
    fn test_reversed_progress() {
        let mut progress = CraftingProgress::new(1.0);
        progress.start();
        progress.update(1.0); // Complete

        // Now reverse (uncraft)
        progress.reset();
        progress.set_progress(1.0);
        progress.set_reversed(true);
        progress.state = AnimationState::InProgress;

        progress.update(0.5);
        assert!((progress.ratio() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_particle_type_roundtrip() {
        for id in 0..=10 {
            let pt = CraftingParticleType::from_id(id);
            let back = CraftingParticleType::from_id(pt.id());
            assert_eq!(pt.id(), back.id());
        }
    }

    #[test]
    fn test_remaining_time() {
        let mut progress = CraftingProgress::new(2.0);
        progress.start();

        assert!((progress.remaining() - 2.0).abs() < f32::EPSILON);

        progress.update(0.5);
        assert!((progress.remaining() - 1.5).abs() < f32::EPSILON);

        // With speed multiplier
        progress.set_speed_multiplier(2.0);
        assert!((progress.remaining() - 0.75).abs() < f32::EPSILON);
    }
}
