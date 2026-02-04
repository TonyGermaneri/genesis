//! Screen transition effects infrastructure.
//!
//! Provides GPU-based transition effects for scene changes.
//! Features include:
//! - Fade to/from black
//! - Crossfade between scenes
//! - Configurable duration (default 0.3s)
//! - GPU-based alpha blending

use bytemuck::{Pod, Zeroable};

/// Default transition duration in seconds.
pub const DEFAULT_TRANSITION_DURATION: f32 = 0.3;

/// Minimum transition duration in seconds.
pub const MIN_TRANSITION_DURATION: f32 = 0.05;

/// Maximum transition duration in seconds.
pub const MAX_TRANSITION_DURATION: f32 = 5.0;

/// Type of screen transition effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionType {
    /// Fade to black then fade in.
    #[default]
    FadeToBlack,
    /// Fade from black.
    FadeFromBlack,
    /// Crossfade between two scenes.
    Crossfade,
    /// Fade to white then fade in.
    FadeToWhite,
    /// Fade from white.
    FadeFromWhite,
    /// Instant cut (no transition).
    Cut,
    /// Wipe from left to right.
    WipeLeft,
    /// Wipe from right to left.
    WipeRight,
    /// Wipe from top to bottom.
    WipeDown,
    /// Wipe from bottom to top.
    WipeUp,
}

impl TransitionType {
    /// Converts to GPU shader enum value.
    #[must_use]
    pub fn to_shader_value(&self) -> u32 {
        match self {
            Self::FadeToBlack => 0,
            Self::FadeFromBlack => 1,
            Self::Crossfade => 2,
            Self::FadeToWhite => 3,
            Self::FadeFromWhite => 4,
            Self::Cut => 5,
            Self::WipeLeft => 6,
            Self::WipeRight => 7,
            Self::WipeDown => 8,
            Self::WipeUp => 9,
        }
    }

    /// Creates from GPU shader enum value.
    #[must_use]
    pub fn from_shader_value(value: u32) -> Self {
        match value {
            1 => Self::FadeFromBlack,
            2 => Self::Crossfade,
            3 => Self::FadeToWhite,
            4 => Self::FadeFromWhite,
            5 => Self::Cut,
            6 => Self::WipeLeft,
            7 => Self::WipeRight,
            8 => Self::WipeDown,
            9 => Self::WipeUp,
            _ => Self::FadeToBlack,
        }
    }

    /// Whether this transition requires a source texture.
    #[must_use]
    pub fn requires_source(&self) -> bool {
        matches!(self, Self::Crossfade)
    }

    /// Whether this transition fades to a solid color.
    #[must_use]
    pub fn is_color_fade(&self) -> bool {
        matches!(
            self,
            Self::FadeToBlack | Self::FadeFromBlack | Self::FadeToWhite | Self::FadeFromWhite
        )
    }

    /// Whether this transition is a wipe effect.
    #[must_use]
    pub fn is_wipe(&self) -> bool {
        matches!(
            self,
            Self::WipeLeft | Self::WipeRight | Self::WipeDown | Self::WipeUp
        )
    }
}

/// Current state of a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionState {
    /// No transition active.
    #[default]
    Idle,
    /// Transition is fading out (first half).
    FadingOut,
    /// At midpoint (fully covered).
    AtMidpoint,
    /// Transition is fading in (second half).
    FadingIn,
    /// Transition completed.
    Completed,
}

impl TransitionState {
    /// Whether the transition is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Idle | Self::Completed)
    }

    /// Whether the transition has finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Easing function for transition timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionEasing {
    /// Linear interpolation.
    Linear,
    /// Ease in (slow start).
    EaseIn,
    /// Ease out (slow end).
    #[default]
    EaseOut,
    /// Ease in and out (slow start and end).
    EaseInOut,
    /// Smooth step (Hermite interpolation).
    SmoothStep,
}

impl TransitionEasing {
    /// Applies the easing function to a normalized time value.
    #[must_use]
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::SmoothStep => t * t * (3.0 - 2.0 * t),
        }
    }

    /// Converts to GPU shader enum value.
    #[must_use]
    pub fn to_shader_value(&self) -> u32 {
        match self {
            Self::Linear => 0,
            Self::EaseIn => 1,
            Self::EaseOut => 2,
            Self::EaseInOut => 3,
            Self::SmoothStep => 4,
        }
    }
}

/// GPU-ready uniform buffer for transition rendering.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct TransitionUniforms {
    /// Current transition progress (0.0-1.0).
    pub progress: f32,
    /// Transition type (shader enum).
    pub transition_type: u32,
    /// Easing function (shader enum).
    pub easing: u32,
    /// Screen width.
    pub screen_width: f32,
    /// Screen height.
    pub screen_height: f32,
    /// Fade color (packed RGBA).
    pub fade_color: u32,
    /// Whether transition is active (0 or 1).
    pub active: u32,
    /// Padding for 16-byte alignment.
    _pad: u32,
}

impl Default for TransitionUniforms {
    fn default() -> Self {
        Self {
            progress: 0.0,
            transition_type: 0,
            easing: TransitionEasing::EaseOut.to_shader_value(),
            screen_width: 1920.0,
            screen_height: 1080.0,
            fade_color: 0xFF00_0000, // Black
            active: 0,
            _pad: 0,
        }
    }
}

impl TransitionUniforms {
    /// Creates uniforms for a specific transition type.
    #[must_use]
    pub fn new(transition_type: TransitionType) -> Self {
        let fade_color = match transition_type {
            TransitionType::FadeToWhite | TransitionType::FadeFromWhite => 0xFFFF_FFFF,
            _ => 0xFF00_0000,
        };
        Self {
            transition_type: transition_type.to_shader_value(),
            fade_color,
            ..Default::default()
        }
    }

    /// Sets the screen dimensions.
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    /// Sets the fade color.
    pub fn set_fade_color(&mut self, color: u32) {
        self.fade_color = color;
    }

    /// Updates the progress value with easing applied.
    pub fn update_progress(&mut self, raw_progress: f32, easing: TransitionEasing) {
        self.progress = easing.apply(raw_progress);
        self.easing = easing.to_shader_value();
        self.active = u32::from(raw_progress > 0.0 && raw_progress < 1.0);
    }
}

/// Configuration for a transition effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionConfig {
    /// Type of transition.
    pub transition_type: TransitionType,
    /// Duration in seconds.
    pub duration: f32,
    /// Easing function.
    pub easing: TransitionEasing,
    /// Fade color (for color-based transitions).
    pub fade_color: u32,
    /// Whether to pause at midpoint.
    pub pause_at_midpoint: bool,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            transition_type: TransitionType::FadeToBlack,
            duration: DEFAULT_TRANSITION_DURATION,
            easing: TransitionEasing::EaseOut,
            fade_color: 0xFF00_0000,
            pause_at_midpoint: false,
        }
    }
}

impl TransitionConfig {
    /// Creates a fade to black transition.
    #[must_use]
    pub fn fade_to_black() -> Self {
        Self::default()
    }

    /// Creates a fade from black transition.
    #[must_use]
    pub fn fade_from_black() -> Self {
        Self {
            transition_type: TransitionType::FadeFromBlack,
            ..Default::default()
        }
    }

    /// Creates a crossfade transition.
    #[must_use]
    pub fn crossfade() -> Self {
        Self {
            transition_type: TransitionType::Crossfade,
            ..Default::default()
        }
    }

    /// Creates a fade to white transition.
    #[must_use]
    pub fn fade_to_white() -> Self {
        Self {
            transition_type: TransitionType::FadeToWhite,
            fade_color: 0xFFFF_FFFF,
            ..Default::default()
        }
    }

    /// Sets the duration.
    #[must_use]
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration.clamp(MIN_TRANSITION_DURATION, MAX_TRANSITION_DURATION);
        self
    }

    /// Sets the easing function.
    #[must_use]
    pub fn with_easing(mut self, easing: TransitionEasing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the fade color.
    #[must_use]
    pub fn with_color(mut self, color: u32) -> Self {
        self.fade_color = color;
        self
    }

    /// Enables pause at midpoint (for two-phase transitions).
    #[must_use]
    pub fn with_midpoint_pause(mut self) -> Self {
        self.pause_at_midpoint = true;
        self
    }
}

/// Active transition state manager.
#[derive(Debug, Clone)]
pub struct TransitionManager {
    /// Current configuration.
    config: TransitionConfig,
    /// Current state.
    state: TransitionState,
    /// Elapsed time in the current transition.
    elapsed: f32,
    /// GPU uniform buffer data.
    uniforms: TransitionUniforms,
    /// Callback ID for midpoint notification.
    midpoint_callback_pending: bool,
    /// Callback ID for completion notification.
    completion_callback_pending: bool,
}

impl Default for TransitionManager {
    fn default() -> Self {
        Self {
            config: TransitionConfig::default(),
            state: TransitionState::Idle,
            elapsed: 0.0,
            uniforms: TransitionUniforms::default(),
            midpoint_callback_pending: false,
            completion_callback_pending: false,
        }
    }
}

impl TransitionManager {
    /// Creates a new transition manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a new transition with the given configuration.
    pub fn start(&mut self, config: TransitionConfig) {
        self.config = config;
        self.state = TransitionState::FadingOut;
        self.elapsed = 0.0;
        self.midpoint_callback_pending = true;
        self.completion_callback_pending = true;
        self.uniforms = TransitionUniforms::new(config.transition_type);
        self.uniforms.set_fade_color(config.fade_color);
    }

    /// Starts a quick fade to black.
    pub fn fade_to_black(&mut self) {
        self.start(TransitionConfig::fade_to_black());
    }

    /// Starts a quick fade from black.
    pub fn fade_from_black(&mut self) {
        self.start(TransitionConfig::fade_from_black());
    }

    /// Starts a crossfade.
    pub fn crossfade(&mut self) {
        self.start(TransitionConfig::crossfade());
    }

    /// Updates the transition state.
    pub fn update(&mut self, dt: f32) {
        if !self.state.is_active() {
            return;
        }

        self.elapsed += dt;
        let progress = (self.elapsed / self.config.duration).clamp(0.0, 1.0);

        // Determine state based on progress
        match self.config.transition_type {
            TransitionType::FadeToBlack | TransitionType::FadeToWhite => {
                if progress < 0.5 {
                    self.state = TransitionState::FadingOut;
                } else if self.config.pause_at_midpoint && (0.5..0.51).contains(&progress) {
                    self.state = TransitionState::AtMidpoint;
                } else if progress < 1.0 {
                    self.state = TransitionState::FadingIn;
                } else {
                    self.state = TransitionState::Completed;
                }
            }
            TransitionType::FadeFromBlack | TransitionType::FadeFromWhite => {
                if progress < 1.0 {
                    self.state = TransitionState::FadingIn;
                } else {
                    self.state = TransitionState::Completed;
                }
            }
            TransitionType::Cut => {
                self.state = TransitionState::Completed;
            }
            _ => {
                if progress < 1.0 {
                    self.state = TransitionState::FadingOut;
                } else {
                    self.state = TransitionState::Completed;
                }
            }
        }

        // Update uniforms
        self.uniforms.update_progress(progress, self.config.easing);
    }

    /// Gets the current transition state.
    #[must_use]
    pub fn state(&self) -> TransitionState {
        self.state
    }

    /// Gets the current progress (0.0-1.0).
    #[must_use]
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.config.duration).clamp(0.0, 1.0)
    }

    /// Gets the eased progress value.
    #[must_use]
    pub fn eased_progress(&self) -> f32 {
        self.config.easing.apply(self.progress())
    }

    /// Gets the current alpha value for overlay rendering.
    #[must_use]
    pub fn alpha(&self) -> f32 {
        let progress = self.eased_progress();
        match self.config.transition_type {
            TransitionType::FadeToBlack | TransitionType::FadeToWhite => {
                if progress < 0.5 {
                    progress * 2.0
                } else {
                    (1.0 - progress) * 2.0
                }
            }
            TransitionType::FadeFromBlack | TransitionType::FadeFromWhite => 1.0 - progress,
            TransitionType::Cut => 0.0,
            _ => progress,
        }
    }

    /// Checks if the transition is at its midpoint.
    #[must_use]
    pub fn at_midpoint(&self) -> bool {
        matches!(self.state, TransitionState::AtMidpoint)
    }

    /// Checks if a midpoint callback is pending.
    #[must_use]
    pub fn take_midpoint_callback(&mut self) -> bool {
        if self.midpoint_callback_pending && self.progress() >= 0.5 {
            self.midpoint_callback_pending = false;
            true
        } else {
            false
        }
    }

    /// Checks if a completion callback is pending.
    #[must_use]
    pub fn take_completion_callback(&mut self) -> bool {
        if self.completion_callback_pending && self.state.is_finished() {
            self.completion_callback_pending = false;
            true
        } else {
            false
        }
    }

    /// Checks if any transition is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Checks if the transition has completed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.state.is_finished()
    }

    /// Cancels the current transition.
    pub fn cancel(&mut self) {
        self.state = TransitionState::Idle;
        self.elapsed = 0.0;
        self.midpoint_callback_pending = false;
        self.completion_callback_pending = false;
        self.uniforms.active = 0;
        self.uniforms.progress = 0.0;
    }

    /// Resets to idle state after completion.
    pub fn reset(&mut self) {
        self.state = TransitionState::Idle;
        self.elapsed = 0.0;
        self.uniforms.active = 0;
    }

    /// Resumes from midpoint pause.
    pub fn resume_from_midpoint(&mut self) {
        if matches!(self.state, TransitionState::AtMidpoint) {
            self.state = TransitionState::FadingIn;
        }
    }

    /// Gets the GPU uniform buffer data.
    #[must_use]
    pub fn uniforms(&self) -> &TransitionUniforms {
        &self.uniforms
    }

    /// Gets mutable GPU uniform buffer data.
    #[must_use]
    pub fn uniforms_mut(&mut self) -> &mut TransitionUniforms {
        &mut self.uniforms
    }

    /// Sets the screen size for the uniforms.
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.uniforms.set_screen_size(width, height);
    }

    /// Gets the current transition type.
    #[must_use]
    pub fn transition_type(&self) -> TransitionType {
        self.config.transition_type
    }

    /// Gets the configured duration.
    #[must_use]
    pub fn duration(&self) -> f32 {
        self.config.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_type_shader_value() {
        assert_eq!(TransitionType::FadeToBlack.to_shader_value(), 0);
        assert_eq!(TransitionType::Crossfade.to_shader_value(), 2);
        assert_eq!(TransitionType::WipeRight.to_shader_value(), 7);
    }

    #[test]
    fn test_transition_type_from_shader_value() {
        assert_eq!(
            TransitionType::from_shader_value(0),
            TransitionType::FadeToBlack
        );
        assert_eq!(
            TransitionType::from_shader_value(2),
            TransitionType::Crossfade
        );
        assert_eq!(
            TransitionType::from_shader_value(99),
            TransitionType::FadeToBlack
        );
    }

    #[test]
    fn test_transition_type_requires_source() {
        assert!(TransitionType::Crossfade.requires_source());
        assert!(!TransitionType::FadeToBlack.requires_source());
    }

    #[test]
    fn test_transition_type_is_color_fade() {
        assert!(TransitionType::FadeToBlack.is_color_fade());
        assert!(TransitionType::FadeFromWhite.is_color_fade());
        assert!(!TransitionType::Crossfade.is_color_fade());
    }

    #[test]
    fn test_transition_type_is_wipe() {
        assert!(TransitionType::WipeLeft.is_wipe());
        assert!(TransitionType::WipeUp.is_wipe());
        assert!(!TransitionType::FadeToBlack.is_wipe());
    }

    #[test]
    fn test_transition_state_is_active() {
        assert!(!TransitionState::Idle.is_active());
        assert!(TransitionState::FadingOut.is_active());
        assert!(TransitionState::FadingIn.is_active());
        assert!(!TransitionState::Completed.is_active());
    }

    #[test]
    fn test_transition_state_is_finished() {
        assert!(!TransitionState::Idle.is_finished());
        assert!(!TransitionState::FadingOut.is_finished());
        assert!(TransitionState::Completed.is_finished());
    }

    #[test]
    fn test_easing_linear() {
        let easing = TransitionEasing::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = TransitionEasing::EaseIn;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) < 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = TransitionEasing::EaseOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) > 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_smooth_step() {
        let easing = TransitionEasing::SmoothStep;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_clamp() {
        let easing = TransitionEasing::Linear;
        assert_eq!(easing.apply(-0.5), 0.0);
        assert_eq!(easing.apply(1.5), 1.0);
    }

    #[test]
    fn test_transition_uniforms_default() {
        let uniforms = TransitionUniforms::default();
        assert_eq!(uniforms.progress, 0.0);
        assert_eq!(uniforms.active, 0);
        assert_eq!(uniforms.fade_color, 0xFF000000);
    }

    #[test]
    fn test_transition_uniforms_new() {
        let uniforms = TransitionUniforms::new(TransitionType::FadeToWhite);
        assert_eq!(uniforms.fade_color, 0xFFFFFFFF);
    }

    #[test]
    fn test_transition_uniforms_screen_size() {
        let mut uniforms = TransitionUniforms::default();
        uniforms.set_screen_size(1280.0, 720.0);
        assert_eq!(uniforms.screen_width, 1280.0);
        assert_eq!(uniforms.screen_height, 720.0);
    }

    #[test]
    fn test_transition_uniforms_update_progress() {
        let mut uniforms = TransitionUniforms::default();
        uniforms.update_progress(0.5, TransitionEasing::Linear);
        assert_eq!(uniforms.progress, 0.5);
        assert_eq!(uniforms.active, 1);
    }

    #[test]
    fn test_transition_config_default() {
        let config = TransitionConfig::default();
        assert_eq!(config.transition_type, TransitionType::FadeToBlack);
        assert_eq!(config.duration, DEFAULT_TRANSITION_DURATION);
    }

    #[test]
    fn test_transition_config_builders() {
        let fade_black = TransitionConfig::fade_to_black();
        assert_eq!(fade_black.transition_type, TransitionType::FadeToBlack);

        let fade_from = TransitionConfig::fade_from_black();
        assert_eq!(fade_from.transition_type, TransitionType::FadeFromBlack);

        let crossfade = TransitionConfig::crossfade();
        assert_eq!(crossfade.transition_type, TransitionType::Crossfade);

        let fade_white = TransitionConfig::fade_to_white();
        assert_eq!(fade_white.fade_color, 0xFFFFFFFF);
    }

    #[test]
    fn test_transition_config_with_duration() {
        let config = TransitionConfig::default().with_duration(0.5);
        assert_eq!(config.duration, 0.5);
    }

    #[test]
    fn test_transition_config_duration_clamp() {
        let too_short = TransitionConfig::default().with_duration(0.001);
        assert_eq!(too_short.duration, MIN_TRANSITION_DURATION);

        let too_long = TransitionConfig::default().with_duration(100.0);
        assert_eq!(too_long.duration, MAX_TRANSITION_DURATION);
    }

    #[test]
    fn test_transition_config_with_easing() {
        let config = TransitionConfig::default().with_easing(TransitionEasing::EaseInOut);
        assert_eq!(config.easing, TransitionEasing::EaseInOut);
    }

    #[test]
    fn test_transition_manager_default() {
        let manager = TransitionManager::default();
        assert_eq!(manager.state(), TransitionState::Idle);
        assert!(!manager.is_active());
    }

    #[test]
    fn test_transition_manager_start() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionConfig::fade_to_black());
        assert!(manager.is_active());
        assert_eq!(manager.state(), TransitionState::FadingOut);
    }

    #[test]
    fn test_transition_manager_fade_to_black() {
        let mut manager = TransitionManager::new();
        manager.fade_to_black();
        assert!(manager.is_active());
        assert_eq!(manager.transition_type(), TransitionType::FadeToBlack);
    }

    #[test]
    fn test_transition_manager_update() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionConfig::fade_to_black().with_duration(1.0));
        manager.update(0.5);
        assert!(manager.progress() >= 0.5);
    }

    #[test]
    fn test_transition_manager_completion() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionConfig::fade_to_black().with_duration(0.1));
        manager.update(1.0);
        assert!(manager.is_complete());
    }

    #[test]
    fn test_transition_manager_alpha_fade_to_black() {
        let mut manager = TransitionManager::new();
        // Use linear easing for predictable test values
        manager.start(
            TransitionConfig::fade_to_black()
                .with_duration(1.0)
                .with_easing(TransitionEasing::Linear),
        );

        // At start, alpha should be 0
        assert_eq!(manager.alpha(), 0.0);

        // At midpoint, alpha should be 1
        manager.update(0.5);
        let midpoint_alpha = manager.alpha();
        assert!(midpoint_alpha > 0.9);

        // At end, alpha should be back to 0
        manager.update(0.5);
        let end_alpha = manager.alpha();
        assert!(end_alpha < 0.1);
    }

    #[test]
    fn test_transition_manager_cancel() {
        let mut manager = TransitionManager::new();
        manager.fade_to_black();
        manager.cancel();
        assert!(!manager.is_active());
        assert_eq!(manager.state(), TransitionState::Idle);
    }

    #[test]
    fn test_transition_manager_reset() {
        let mut manager = TransitionManager::new();
        manager.fade_to_black();
        manager.update(1.0);
        manager.reset();
        assert_eq!(manager.state(), TransitionState::Idle);
    }

    #[test]
    fn test_transition_manager_midpoint_callback() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionConfig::fade_to_black().with_duration(1.0));
        manager.update(0.3);
        assert!(!manager.take_midpoint_callback());
        manager.update(0.3);
        assert!(manager.take_midpoint_callback());
        assert!(!manager.take_midpoint_callback()); // Already taken
    }

    #[test]
    fn test_transition_manager_completion_callback() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionConfig::fade_to_black().with_duration(0.1));
        assert!(!manager.take_completion_callback());
        manager.update(1.0);
        assert!(manager.take_completion_callback());
        assert!(!manager.take_completion_callback()); // Already taken
    }

    #[test]
    fn test_transition_manager_screen_size() {
        let mut manager = TransitionManager::new();
        manager.set_screen_size(1280.0, 720.0);
        assert_eq!(manager.uniforms().screen_width, 1280.0);
        assert_eq!(manager.uniforms().screen_height, 720.0);
    }

    #[test]
    fn test_transition_manager_duration() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionConfig::fade_to_black().with_duration(0.5));
        assert_eq!(manager.duration(), 0.5);
    }

    #[test]
    fn test_transition_uniforms_size() {
        assert_eq!(std::mem::size_of::<TransitionUniforms>(), 32);
    }

    #[test]
    fn test_cut_transition() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionConfig {
            transition_type: TransitionType::Cut,
            ..Default::default()
        });
        manager.update(0.0);
        assert!(manager.is_complete());
    }

    #[test]
    fn test_fade_from_black() {
        let mut manager = TransitionManager::new();
        manager.fade_from_black();
        assert_eq!(manager.transition_type(), TransitionType::FadeFromBlack);

        // Alpha should start at 1 and decrease
        let start_alpha = manager.alpha();
        assert!(start_alpha > 0.9);

        manager.update(0.5);
        let mid_alpha = manager.alpha();
        assert!(mid_alpha < start_alpha);
    }
}
