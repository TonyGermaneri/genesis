//! Pause state handling.
//!
//! This module manages game pause functionality:
//! - Freeze world updates when paused
//! - Track total paused time
//! - Continue rendering while paused

use serde::{Deserialize, Serialize};
use std::time::Instant;

// ============================================================================
// G-60: Pause State
// ============================================================================

/// Current pause state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PauseReason {
    /// Not paused.
    #[default]
    NotPaused,
    /// Paused by player (escape menu).
    PlayerPaused,
    /// Paused by inventory/menu.
    MenuOpen,
    /// Paused due to window losing focus.
    FocusLost,
    /// Paused during cutscene.
    Cutscene,
    /// Paused during loading.
    Loading,
    /// Paused by system (e.g., low battery).
    System,
}

impl PauseReason {
    /// Check if actually paused.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        !matches!(self, Self::NotPaused)
    }

    /// Check if world should freeze.
    #[must_use]
    pub const fn should_freeze_world(&self) -> bool {
        match self {
            Self::NotPaused | Self::Cutscene => false, // Cutscenes may have scripted events
            Self::PlayerPaused
            | Self::MenuOpen
            | Self::FocusLost
            | Self::Loading
            | Self::System => true,
        }
    }

    /// Check if game should render.
    #[must_use]
    pub const fn should_render(&self) -> bool {
        match self {
            Self::FocusLost => false, // Don't render when unfocused
            Self::NotPaused
            | Self::PlayerPaused
            | Self::MenuOpen
            | Self::Cutscene
            | Self::Loading
            | Self::System => true,
        }
    }

    /// Check if UI should be interactive.
    #[must_use]
    pub const fn ui_interactive(&self) -> bool {
        match self {
            Self::FocusLost | Self::Cutscene | Self::Loading => false,
            Self::NotPaused | Self::PlayerPaused | Self::MenuOpen | Self::System => true,
        }
    }

    /// Get pause message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::NotPaused | Self::MenuOpen | Self::Cutscene => "",
            Self::PlayerPaused | Self::System => "PAUSED",
            Self::FocusLost => "PAUSED - Click to resume",
            Self::Loading => "Loading...",
        }
    }
}

// ============================================================================
// G-60: Pause Manager
// ============================================================================

/// Manages pause state and timing.
#[derive(Debug)]
pub struct PauseManager {
    /// Current pause reason.
    reason: PauseReason,
    /// Stack of pause reasons (for nested pauses).
    pause_stack: Vec<PauseReason>,
    /// When current pause started.
    pause_start: Option<Instant>,
    /// Total accumulated paused time in seconds.
    total_paused_time: f64,
    /// Whether pause on focus loss is enabled.
    pause_on_focus_loss: bool,
    /// Whether menu opens cause pause.
    pause_on_menu: bool,
}

impl Default for PauseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PauseManager {
    /// Create a new pause manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            reason: PauseReason::NotPaused,
            pause_stack: Vec::new(),
            pause_start: None,
            total_paused_time: 0.0,
            pause_on_focus_loss: true,
            pause_on_menu: true,
        }
    }

    /// Get current pause reason.
    #[must_use]
    pub fn reason(&self) -> PauseReason {
        self.reason
    }

    /// Check if game is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.reason.is_paused()
    }

    /// Check if world should be updated.
    #[must_use]
    pub fn should_update_world(&self) -> bool {
        !self.reason.should_freeze_world()
    }

    /// Check if game should render.
    #[must_use]
    pub fn should_render(&self) -> bool {
        self.reason.should_render()
    }

    /// Check if UI should be interactive.
    #[must_use]
    pub fn ui_interactive(&self) -> bool {
        self.reason.ui_interactive()
    }

    /// Get total paused time in seconds.
    #[must_use]
    pub fn total_paused_time(&self) -> f64 {
        let current_pause = self.pause_start.map_or(0.0, |s| s.elapsed().as_secs_f64());
        self.total_paused_time + current_pause
    }

    /// Get current pause duration in seconds.
    #[must_use]
    pub fn current_pause_duration(&self) -> f64 {
        self.pause_start.map_or(0.0, |s| s.elapsed().as_secs_f64())
    }

    /// Set pause on focus loss setting.
    pub fn set_pause_on_focus_loss(&mut self, enabled: bool) {
        self.pause_on_focus_loss = enabled;
    }

    /// Set pause on menu setting.
    pub fn set_pause_on_menu(&mut self, enabled: bool) {
        self.pause_on_menu = enabled;
    }

    /// Pause the game.
    pub fn pause(&mut self, reason: PauseReason) {
        // Don't pause if already not paused reason or same reason
        if !reason.is_paused() {
            return;
        }

        // Check settings
        if reason == PauseReason::FocusLost && !self.pause_on_focus_loss {
            return;
        }
        if reason == PauseReason::MenuOpen && !self.pause_on_menu {
            return;
        }

        // If not already paused, start timing
        if !self.is_paused() {
            self.pause_start = Some(Instant::now());
        }

        // Push current reason to stack if different
        if self.reason != reason && self.reason.is_paused() {
            self.pause_stack.push(self.reason);
        }

        self.reason = reason;
    }

    /// Resume from pause.
    pub fn resume(&mut self) {
        self.resume_from(self.reason);
    }

    /// Resume from a specific pause reason.
    pub fn resume_from(&mut self, reason: PauseReason) {
        // Only resume if this is the current reason
        if self.reason != reason {
            // Try to remove from stack
            self.pause_stack.retain(|r| *r != reason);
            return;
        }

        // Pop from stack or fully resume
        if let Some(prev_reason) = self.pause_stack.pop() {
            self.reason = prev_reason;
        } else {
            self.complete_resume();
        }
    }

    /// Force resume, clearing all pause states.
    pub fn force_resume(&mut self) {
        self.pause_stack.clear();
        self.complete_resume();
    }

    /// Complete the resume process.
    fn complete_resume(&mut self) {
        // Accumulate paused time
        if let Some(start) = self.pause_start.take() {
            self.total_paused_time += start.elapsed().as_secs_f64();
        }

        self.reason = PauseReason::NotPaused;
    }

    /// Toggle pause (player initiated).
    pub fn toggle_pause(&mut self) {
        if self.reason == PauseReason::PlayerPaused {
            self.resume_from(PauseReason::PlayerPaused);
        } else if !self.is_paused() {
            self.pause(PauseReason::PlayerPaused);
        }
    }

    /// Handle window focus change.
    pub fn on_focus_change(&mut self, focused: bool) {
        if focused {
            self.resume_from(PauseReason::FocusLost);
        } else if self.pause_on_focus_loss {
            self.pause(PauseReason::FocusLost);
        }
    }

    /// Open a menu (potentially pausing).
    pub fn open_menu(&mut self) {
        if self.pause_on_menu {
            self.pause(PauseReason::MenuOpen);
        }
    }

    /// Close a menu.
    pub fn close_menu(&mut self) {
        self.resume_from(PauseReason::MenuOpen);
    }

    /// Start a cutscene.
    pub fn start_cutscene(&mut self) {
        self.pause(PauseReason::Cutscene);
    }

    /// End a cutscene.
    pub fn end_cutscene(&mut self) {
        self.resume_from(PauseReason::Cutscene);
    }

    /// Start loading.
    pub fn start_loading(&mut self) {
        self.pause(PauseReason::Loading);
    }

    /// End loading.
    pub fn end_loading(&mut self) {
        self.resume_from(PauseReason::Loading);
    }

    /// Reset pause timing (e.g., when starting new game).
    pub fn reset_timing(&mut self) {
        self.total_paused_time = 0.0;
        self.pause_start = None;
    }

    /// Get adjusted game time (excluding paused time).
    #[must_use]
    pub fn adjust_time(&self, raw_elapsed: f64) -> f64 {
        (raw_elapsed - self.total_paused_time()).max(0.0)
    }
}

// ============================================================================
// G-60: Pause State Snapshot
// ============================================================================

/// Serializable pause state snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PauseStateSnapshot {
    /// Total accumulated paused time.
    pub total_paused_time: f64,
    /// Pause on focus loss setting.
    pub pause_on_focus_loss: bool,
    /// Pause on menu setting.
    pub pause_on_menu: bool,
}

impl PauseStateSnapshot {
    /// Create snapshot from pause manager.
    #[must_use]
    pub fn from_manager(manager: &PauseManager) -> Self {
        Self {
            total_paused_time: manager.total_paused_time(),
            pause_on_focus_loss: manager.pause_on_focus_loss,
            pause_on_menu: manager.pause_on_menu,
        }
    }

    /// Apply snapshot to pause manager.
    pub fn apply_to(&self, manager: &mut PauseManager) {
        manager.total_paused_time = self.total_paused_time;
        manager.pause_on_focus_loss = self.pause_on_focus_loss;
        manager.pause_on_menu = self.pause_on_menu;
    }
}

impl Default for PauseStateSnapshot {
    fn default() -> Self {
        Self {
            total_paused_time: 0.0,
            pause_on_focus_loss: true,
            pause_on_menu: true,
        }
    }
}

// ============================================================================
// G-60: Time Scale Manager
// ============================================================================

/// Manages game time scaling (slow-mo, fast-forward).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeScaleManager {
    /// Current time scale (1.0 = normal).
    time_scale: f32,
    /// Target time scale (for smooth transitions).
    target_scale: f32,
    /// Transition speed (scale per second).
    transition_speed: f32,
    /// Minimum time scale.
    min_scale: f32,
    /// Maximum time scale.
    max_scale: f32,
}

impl Default for TimeScaleManager {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            target_scale: 1.0,
            transition_speed: 5.0,
            min_scale: 0.0,
            max_scale: 4.0,
        }
    }
}

impl TimeScaleManager {
    /// Create a new time scale manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current time scale.
    #[must_use]
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Get target time scale.
    #[must_use]
    pub fn target_scale(&self) -> f32 {
        self.target_scale
    }

    /// Set time scale immediately.
    pub fn set_scale(&mut self, scale: f32) {
        let clamped = scale.clamp(self.min_scale, self.max_scale);
        self.time_scale = clamped;
        self.target_scale = clamped;
    }

    /// Set target time scale for smooth transition.
    pub fn set_target(&mut self, target: f32) {
        self.target_scale = target.clamp(self.min_scale, self.max_scale);
    }

    /// Set transition speed.
    pub fn set_transition_speed(&mut self, speed: f32) {
        self.transition_speed = speed.max(0.1);
    }

    /// Update time scale (call each frame).
    pub fn update(&mut self, delta_time: f32) {
        if (self.time_scale - self.target_scale).abs() > 0.001 {
            let direction = (self.target_scale - self.time_scale).signum();
            let change = self.transition_speed * delta_time * direction;

            self.time_scale += change;

            // Clamp to not overshoot target
            if direction > 0.0 {
                self.time_scale = self.time_scale.min(self.target_scale);
            } else {
                self.time_scale = self.time_scale.max(self.target_scale);
            }
        }
    }

    /// Apply time scale to delta time.
    #[must_use]
    pub fn scale_delta(&self, delta_time: f32) -> f32 {
        delta_time * self.time_scale
    }

    /// Freeze time (scale = 0).
    pub fn freeze(&mut self) {
        self.set_scale(0.0);
    }

    /// Reset to normal speed.
    pub fn reset(&mut self) {
        self.set_scale(1.0);
    }

    /// Enable slow motion.
    pub fn slow_motion(&mut self, scale: f32) {
        self.set_target(scale.clamp(0.1, 0.9));
    }

    /// Enable fast forward.
    pub fn fast_forward(&mut self, scale: f32) {
        self.set_target(scale.clamp(1.1, self.max_scale));
    }

    /// Check if at normal speed.
    #[must_use]
    pub fn is_normal_speed(&self) -> bool {
        (self.time_scale - 1.0).abs() < 0.01
    }

    /// Check if frozen.
    #[must_use]
    pub fn is_frozen(&self) -> bool {
        self.time_scale < 0.01
    }
}

// ============================================================================
// G-60: Combined Pause and Time Controller
// ============================================================================

/// Combined pause and time scale controller.
#[derive(Debug)]
pub struct GameTimeController {
    /// Pause manager.
    pub pause: PauseManager,
    /// Time scale manager.
    pub time_scale: TimeScaleManager,
}

impl Default for GameTimeController {
    fn default() -> Self {
        Self::new()
    }
}

impl GameTimeController {
    /// Create a new game time controller.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pause: PauseManager::new(),
            time_scale: TimeScaleManager::new(),
        }
    }

    /// Get effective delta time considering pause and time scale.
    #[must_use]
    pub fn effective_delta(&self, raw_delta: f32) -> f32 {
        if self.pause.should_update_world() {
            self.time_scale.scale_delta(raw_delta)
        } else {
            0.0
        }
    }

    /// Update the controller.
    pub fn update(&mut self, delta_time: f32) {
        self.time_scale.update(delta_time);
    }

    /// Check if game world should update.
    #[must_use]
    pub fn should_update(&self) -> bool {
        self.pause.should_update_world() && !self.time_scale.is_frozen()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_reason_default() {
        let reason = PauseReason::default();
        assert!(!reason.is_paused());
        assert!(!reason.should_freeze_world());
    }

    #[test]
    fn test_pause_reason_states() {
        assert!(PauseReason::PlayerPaused.is_paused());
        assert!(PauseReason::PlayerPaused.should_freeze_world());
        assert!(PauseReason::PlayerPaused.should_render());

        assert!(PauseReason::FocusLost.is_paused());
        assert!(!PauseReason::FocusLost.should_render());
    }

    #[test]
    fn test_pause_manager_new() {
        let manager = PauseManager::new();
        assert!(!manager.is_paused());
        assert!(manager.should_update_world());
    }

    #[test]
    fn test_pause_and_resume() {
        let mut manager = PauseManager::new();

        manager.pause(PauseReason::PlayerPaused);
        assert!(manager.is_paused());
        assert!(!manager.should_update_world());

        manager.resume();
        assert!(!manager.is_paused());
        assert!(manager.should_update_world());
    }

    #[test]
    fn test_toggle_pause() {
        let mut manager = PauseManager::new();

        manager.toggle_pause();
        assert!(manager.is_paused());
        assert_eq!(manager.reason(), PauseReason::PlayerPaused);

        manager.toggle_pause();
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_pause_stack() {
        let mut manager = PauseManager::new();

        manager.pause(PauseReason::PlayerPaused);
        assert_eq!(manager.reason(), PauseReason::PlayerPaused);

        manager.pause(PauseReason::MenuOpen);
        assert_eq!(manager.reason(), PauseReason::MenuOpen);

        manager.resume_from(PauseReason::MenuOpen);
        assert_eq!(manager.reason(), PauseReason::PlayerPaused);

        manager.resume_from(PauseReason::PlayerPaused);
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_force_resume() {
        let mut manager = PauseManager::new();

        manager.pause(PauseReason::PlayerPaused);
        manager.pause(PauseReason::MenuOpen);

        manager.force_resume();
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_focus_change() {
        let mut manager = PauseManager::new();

        manager.on_focus_change(false);
        assert!(manager.is_paused());
        assert_eq!(manager.reason(), PauseReason::FocusLost);

        manager.on_focus_change(true);
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_focus_change_disabled() {
        let mut manager = PauseManager::new();
        manager.set_pause_on_focus_loss(false);

        manager.on_focus_change(false);
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_menu_pause() {
        let mut manager = PauseManager::new();

        manager.open_menu();
        assert!(manager.is_paused());

        manager.close_menu();
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_pause_state_snapshot() {
        let mut manager = PauseManager::new();
        manager.pause(PauseReason::PlayerPaused);

        let snapshot = PauseStateSnapshot::from_manager(&manager);
        assert!(snapshot.pause_on_focus_loss);

        let mut new_manager = PauseManager::new();
        new_manager.set_pause_on_focus_loss(false);
        snapshot.apply_to(&mut new_manager);
        assert!(new_manager.pause_on_focus_loss);
    }

    #[test]
    fn test_time_scale_default() {
        let manager = TimeScaleManager::new();
        assert_eq!(manager.time_scale(), 1.0);
        assert!(manager.is_normal_speed());
    }

    #[test]
    fn test_time_scale_set() {
        let mut manager = TimeScaleManager::new();

        manager.set_scale(0.5);
        assert_eq!(manager.time_scale(), 0.5);
        assert!(!manager.is_normal_speed());

        manager.freeze();
        assert!(manager.is_frozen());

        manager.reset();
        assert!(manager.is_normal_speed());
    }

    #[test]
    fn test_time_scale_delta() {
        let mut manager = TimeScaleManager::new();
        manager.set_scale(0.5);

        let scaled = manager.scale_delta(1.0);
        assert_eq!(scaled, 0.5);
    }

    #[test]
    fn test_time_scale_clamping() {
        let mut manager = TimeScaleManager::new();

        manager.set_scale(10.0);
        assert_eq!(manager.time_scale(), 4.0); // Clamped to max

        manager.set_scale(-1.0);
        assert_eq!(manager.time_scale(), 0.0); // Clamped to min
    }

    #[test]
    fn test_game_time_controller() {
        let mut controller = GameTimeController::new();

        // Normal operation
        assert!(controller.should_update());
        let delta = controller.effective_delta(1.0);
        assert_eq!(delta, 1.0);

        // Paused
        controller.pause.pause(PauseReason::PlayerPaused);
        assert!(!controller.should_update());
        let delta = controller.effective_delta(1.0);
        assert_eq!(delta, 0.0);

        // Resumed with slow motion
        controller.pause.resume();
        controller.time_scale.set_scale(0.5);
        assert!(controller.should_update());
        let delta = controller.effective_delta(1.0);
        assert_eq!(delta, 0.5);
    }
}
