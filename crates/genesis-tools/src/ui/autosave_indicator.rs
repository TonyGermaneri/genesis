//! Auto-save Indicator UI
//!
//! Visual indicator for automatic save operations with spinning icon,
//! configurable position, fade animations, and error handling.

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Corner position for the indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CornerPosition {
    /// Top-left corner
    TopLeft,
    /// Top-right corner
    #[default]
    TopRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner
    BottomRight,
}

impl CornerPosition {
    /// Get all corner positions
    pub fn all() -> &'static [Self] {
        &[
            Self::TopLeft,
            Self::TopRight,
            Self::BottomLeft,
            Self::BottomRight,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::TopLeft => "Top Left",
            Self::TopRight => "Top Right",
            Self::BottomLeft => "Bottom Left",
            Self::BottomRight => "Bottom Right",
        }
    }

    /// Get egui alignment for this corner
    pub fn alignment(&self) -> egui::Align2 {
        match self {
            Self::TopLeft => egui::Align2::LEFT_TOP,
            Self::TopRight => egui::Align2::RIGHT_TOP,
            Self::BottomLeft => egui::Align2::LEFT_BOTTOM,
            Self::BottomRight => egui::Align2::RIGHT_BOTTOM,
        }
    }

    /// Check if this is a left position
    pub fn is_left(&self) -> bool {
        matches!(self, Self::TopLeft | Self::BottomLeft)
    }

    /// Check if this is a top position
    pub fn is_top(&self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }
}

/// State of the auto-save operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SaveState {
    /// No save operation in progress
    #[default]
    Idle,
    /// Save is in progress
    Saving,
    /// Save completed successfully
    Success,
    /// Save failed with error
    Error,
}

impl SaveState {
    /// Get display text for the state
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Saving => "Saving...",
            Self::Success => "Saved!",
            Self::Error => "Save Failed!",
        }
    }

    /// Get icon for the state
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Saving => "💾",
            Self::Success => "✓",
            Self::Error => "✕",
        }
    }

    /// Get color for the state
    pub fn color(&self) -> Color32 {
        match self {
            Self::Idle => Color32::TRANSPARENT,
            Self::Saving => Color32::from_rgb(100, 180, 255),
            Self::Success => Color32::from_rgb(100, 200, 100),
            Self::Error => Color32::from_rgb(200, 100, 100),
        }
    }

    /// Check if this state should show the indicator
    pub fn is_visible(&self) -> bool {
        !matches!(self, Self::Idle)
    }

    /// Check if this is an active/spinning state
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Saving)
    }
}

/// Animation style for the indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnimationStyle {
    /// Simple spinning icon
    #[default]
    Spin,
    /// Pulsing opacity
    Pulse,
    /// Bouncing motion
    Bounce,
    /// No animation (static)
    None,
}

impl AnimationStyle {
    /// Get all animation styles
    pub fn all() -> &'static [Self] {
        &[Self::Spin, Self::Pulse, Self::Bounce, Self::None]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Spin => "Spin",
            Self::Pulse => "Pulse",
            Self::Bounce => "Bounce",
            Self::None => "None",
        }
    }
}

/// Fade animation state
#[derive(Debug, Clone, Copy, Default)]
pub struct FadeState {
    /// Current opacity (0.0-1.0)
    pub opacity: f32,
    /// Target opacity
    pub target: f32,
    /// Fade speed (units per second)
    pub speed: f32,
}

impl FadeState {
    /// Create new fade state
    pub fn new(speed: f32) -> Self {
        Self {
            opacity: 0.0,
            target: 0.0,
            speed,
        }
    }

    /// Start fading in
    pub fn fade_in(&mut self) {
        self.target = 1.0;
    }

    /// Start fading out
    pub fn fade_out(&mut self) {
        self.target = 0.0;
    }

    /// Set immediate opacity (no fade)
    pub fn set_immediate(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
        self.target = self.opacity;
    }

    /// Update fade animation
    pub fn update(&mut self, delta_time: f32) {
        if (self.opacity - self.target).abs() < 0.001 {
            self.opacity = self.target;
            return;
        }

        let direction = if self.target > self.opacity {
            1.0
        } else {
            -1.0
        };
        self.opacity += direction * self.speed * delta_time;
        self.opacity = self.opacity.clamp(0.0, 1.0);

        // Snap to target if very close
        if (self.opacity - self.target).abs() < 0.01 {
            self.opacity = self.target;
        }
    }

    /// Check if currently fading
    pub fn is_fading(&self) -> bool {
        (self.opacity - self.target).abs() > 0.001
    }

    /// Check if fully visible
    pub fn is_visible(&self) -> bool {
        self.opacity > 0.01
    }

    /// Check if fully hidden
    pub fn is_hidden(&self) -> bool {
        self.opacity < 0.01
    }
}

/// Configuration for auto-save indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveIndicatorConfig {
    /// Corner position
    pub position: CornerPosition,
    /// Margin from screen edge
    pub margin: f32,
    /// Icon size
    pub icon_size: f32,
    /// Animation style
    pub animation_style: AnimationStyle,
    /// Spin speed (rotations per second)
    pub spin_speed: f32,
    /// Fade in duration (seconds)
    pub fade_in_duration: f32,
    /// Fade out duration (seconds)
    pub fade_out_duration: f32,
    /// How long to show success state (seconds)
    pub success_duration: f32,
    /// How long to show error state (seconds)
    pub error_duration: f32,
    /// Whether to show text label
    pub show_label: bool,
    /// Background color
    pub background_color: [u8; 4],
    /// Whether to show background
    pub show_background: bool,
}

impl Default for AutosaveIndicatorConfig {
    fn default() -> Self {
        Self {
            position: CornerPosition::TopRight,
            margin: 20.0,
            icon_size: 24.0,
            animation_style: AnimationStyle::Spin,
            spin_speed: 1.0,
            fade_in_duration: 0.2,
            fade_out_duration: 0.5,
            success_duration: 2.0,
            error_duration: 5.0,
            show_label: true,
            background_color: [30, 30, 40, 200],
            show_background: true,
        }
    }
}

/// Error information for failed saves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveError {
    /// Error message
    pub message: String,
    /// Error code (if applicable)
    pub code: Option<i32>,
    /// Timestamp when error occurred
    pub timestamp: u64,
    /// Whether the error is recoverable
    pub recoverable: bool,
}

impl SaveError {
    /// Create new save error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            timestamp: 0,
            recoverable: true,
        }
    }

    /// Set error code
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Mark as non-recoverable
    pub fn non_recoverable(mut self) -> Self {
        self.recoverable = false;
        self
    }

    /// Get short description
    pub fn short_description(&self) -> &str {
        if self.message.len() > 50 {
            &self.message[..50]
        } else {
            &self.message
        }
    }
}

/// Auto-save indicator widget
#[derive(Debug)]
pub struct AutosaveIndicator {
    /// Current save state
    state: SaveState,
    /// Fade animation state
    fade: FadeState,
    /// Current rotation angle (radians)
    rotation: f32,
    /// Animation time accumulator
    anim_time: f32,
    /// Time remaining in current state
    state_timer: f32,
    /// Configuration
    config: AutosaveIndicatorConfig,
    /// Last error (if any)
    last_error: Option<SaveError>,
    /// Save progress (0.0-1.0, if applicable)
    progress: Option<f32>,
}

impl AutosaveIndicator {
    /// Create new auto-save indicator
    pub fn new(config: AutosaveIndicatorConfig) -> Self {
        let fade_speed = 1.0 / config.fade_in_duration.max(0.01);
        Self {
            state: SaveState::Idle,
            fade: FadeState::new(fade_speed),
            rotation: 0.0,
            anim_time: 0.0,
            state_timer: 0.0,
            config,
            last_error: None,
            progress: None,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AutosaveIndicatorConfig::default())
    }

    /// Get current state
    pub fn state(&self) -> SaveState {
        self.state
    }

    /// Check if indicator is visible
    pub fn is_visible(&self) -> bool {
        self.fade.is_visible()
    }

    /// Check if currently saving
    pub fn is_saving(&self) -> bool {
        self.state == SaveState::Saving
    }

    /// Start save operation
    pub fn start_save(&mut self) {
        self.state = SaveState::Saving;
        self.progress = None;
        self.last_error = None;
        self.fade.speed = 1.0 / self.config.fade_in_duration.max(0.01);
        self.fade.fade_in();
    }

    /// Start save with progress tracking
    pub fn start_save_with_progress(&mut self) {
        self.start_save();
        self.progress = Some(0.0);
    }

    /// Update save progress
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = Some(progress.clamp(0.0, 1.0));
    }

    /// Complete save successfully
    pub fn complete_success(&mut self) {
        self.state = SaveState::Success;
        self.state_timer = self.config.success_duration;
        self.progress = None;
    }

    /// Complete save with error
    pub fn complete_error(&mut self, error: SaveError) {
        self.state = SaveState::Error;
        self.state_timer = self.config.error_duration;
        self.last_error = Some(error);
        self.progress = None;
    }

    /// Get last error
    pub fn last_error(&self) -> Option<&SaveError> {
        self.last_error.as_ref()
    }

    /// Clear last error
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// Get current progress
    pub fn progress(&self) -> Option<f32> {
        self.progress
    }

    /// Get configuration
    pub fn config(&self) -> &AutosaveIndicatorConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: AutosaveIndicatorConfig) {
        self.config = config;
    }

    /// Set position
    pub fn set_position(&mut self, position: CornerPosition) {
        self.config.position = position;
    }

    /// Update the indicator
    pub fn update(&mut self, delta_time: f32) {
        // Update animation time
        self.anim_time += delta_time;

        // Update rotation for spinning animation
        if self.state.is_active() {
            self.rotation += TAU * self.config.spin_speed * delta_time;
            self.rotation %= TAU;
        }

        // Update fade
        self.fade.update(delta_time);

        // Update state timer
        if self.state_timer > 0.0 {
            self.state_timer -= delta_time;
            if self.state_timer <= 0.0 {
                // Timer expired, start fading out
                self.fade.speed = 1.0 / self.config.fade_out_duration.max(0.01);
                self.fade.fade_out();
            }
        }

        // Return to idle when fully faded out
        if self.fade.is_hidden() && !matches!(self.state, SaveState::Idle | SaveState::Saving) {
            self.state = SaveState::Idle;
        }
    }

    /// Render the indicator
    pub fn render(&mut self, ui: &mut Ui) {
        if !self.fade.is_visible() {
            return;
        }

        let opacity = (self.fade.opacity * 255.0) as u8;

        // Calculate position based on corner
        let screen_rect = ui.ctx().screen_rect();
        let margin = self.config.margin;

        let pos = match self.config.position {
            CornerPosition::TopLeft => egui::pos2(margin, margin),
            CornerPosition::TopRight => egui::pos2(screen_rect.max.x - margin - 100.0, margin),
            CornerPosition::BottomLeft => egui::pos2(margin, screen_rect.max.y - margin - 40.0),
            CornerPosition::BottomRight => egui::pos2(
                screen_rect.max.x - margin - 100.0,
                screen_rect.max.y - margin - 40.0,
            ),
        };

        // Create a small window at the corner position
        egui::Area::new(egui::Id::new("autosave_indicator"))
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                self.render_content(ui, opacity);
            });
    }

    fn render_content(&self, ui: &mut Ui, opacity: u8) {
        let bg_color = if self.config.show_background {
            Color32::from_rgba_unmultiplied(
                self.config.background_color[0],
                self.config.background_color[1],
                self.config.background_color[2],
                (self.config.background_color[3] as f32 * self.fade.opacity) as u8,
            )
        } else {
            Color32::TRANSPARENT
        };

        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(8.0)
            .rounding(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Icon with animation
                    self.render_icon(ui, opacity);

                    // Label
                    if self.config.show_label {
                        ui.add_space(4.0);
                        let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, opacity);
                        ui.label(egui::RichText::new(self.state.display_text()).color(text_color));
                    }

                    // Progress bar (if applicable)
                    if let Some(progress) = self.progress {
                        ui.add_space(4.0);
                        let progress_bar = egui::ProgressBar::new(progress)
                            .desired_width(60.0)
                            .show_percentage();
                        ui.add(progress_bar);
                    }
                });

                // Error details (if error state)
                if self.state == SaveState::Error {
                    if let Some(error) = &self.last_error {
                        ui.add_space(4.0);
                        let error_color = Color32::from_rgba_unmultiplied(255, 150, 150, opacity);
                        ui.label(
                            egui::RichText::new(error.short_description())
                                .small()
                                .color(error_color),
                        );
                    }
                }
            });
    }

    fn render_icon(&self, ui: &mut Ui, opacity: u8) {
        let icon_color = Color32::from_rgba_unmultiplied(
            self.state.color().r(),
            self.state.color().g(),
            self.state.color().b(),
            opacity,
        );

        let icon_text = match self.state {
            SaveState::Saving => {
                // Animated icon based on style
                match self.config.animation_style {
                    AnimationStyle::Spin | AnimationStyle::Bounce => "💾",
                    AnimationStyle::Pulse | AnimationStyle::None => self.state.icon(),
                }
            },
            _ => self.state.icon(),
        };

        // Apply animation effects
        let scale = match self.config.animation_style {
            AnimationStyle::Pulse if self.state.is_active() => {
                1.0 + 0.2 * (self.anim_time * 4.0).sin()
            },
            AnimationStyle::Bounce if self.state.is_active() => {
                1.0 + 0.1 * (self.anim_time * 8.0).sin().abs()
            },
            _ => 1.0,
        };

        let font_size = self.config.icon_size * scale;
        ui.label(
            egui::RichText::new(icon_text)
                .size(font_size)
                .color(icon_color),
        );
    }
}

/// Manages multiple save operations (for manual + auto saves)
#[derive(Debug)]
pub struct SaveOperationManager {
    /// Auto-save indicator
    autosave: AutosaveIndicator,
    /// Manual save indicator
    manual_save: AutosaveIndicator,
    /// Time until next auto-save (seconds)
    autosave_timer: f32,
    /// Auto-save interval (seconds)
    autosave_interval: f32,
    /// Whether auto-save is enabled
    autosave_enabled: bool,
    /// Number of saves performed this session
    save_count: u32,
    /// Total time spent saving (seconds)
    total_save_time: f32,
}

impl SaveOperationManager {
    /// Create new save operation manager
    pub fn new(autosave_interval: f32) -> Self {
        let autosave_config = AutosaveIndicatorConfig {
            position: CornerPosition::TopRight,
            ..Default::default()
        };

        let manual_config = AutosaveIndicatorConfig {
            position: CornerPosition::BottomRight,
            ..Default::default()
        };

        Self {
            autosave: AutosaveIndicator::new(autosave_config),
            manual_save: AutosaveIndicator::new(manual_config),
            autosave_timer: autosave_interval,
            autosave_interval,
            autosave_enabled: true,
            save_count: 0,
            total_save_time: 0.0,
        }
    }

    /// Create with default 5-minute interval
    pub fn with_defaults() -> Self {
        Self::new(300.0)
    }

    /// Get auto-save indicator
    pub fn autosave_indicator(&self) -> &AutosaveIndicator {
        &self.autosave
    }

    /// Get auto-save indicator mutably
    pub fn autosave_indicator_mut(&mut self) -> &mut AutosaveIndicator {
        &mut self.autosave
    }

    /// Get manual save indicator
    pub fn manual_save_indicator(&self) -> &AutosaveIndicator {
        &self.manual_save
    }

    /// Get manual save indicator mutably
    pub fn manual_save_indicator_mut(&mut self) -> &mut AutosaveIndicator {
        &mut self.manual_save
    }

    /// Check if auto-save is enabled
    pub fn is_autosave_enabled(&self) -> bool {
        self.autosave_enabled
    }

    /// Enable/disable auto-save
    pub fn set_autosave_enabled(&mut self, enabled: bool) {
        self.autosave_enabled = enabled;
    }

    /// Get auto-save interval
    pub fn autosave_interval(&self) -> f32 {
        self.autosave_interval
    }

    /// Set auto-save interval
    pub fn set_autosave_interval(&mut self, interval: f32) {
        self.autosave_interval = interval.max(10.0);
    }

    /// Get time until next auto-save
    pub fn time_until_autosave(&self) -> f32 {
        self.autosave_timer
    }

    /// Reset auto-save timer
    pub fn reset_autosave_timer(&mut self) {
        self.autosave_timer = self.autosave_interval;
    }

    /// Get total save count
    pub fn save_count(&self) -> u32 {
        self.save_count
    }

    /// Start manual save
    pub fn start_manual_save(&mut self) {
        self.manual_save.start_save();
        self.save_count += 1;
    }

    /// Complete manual save
    pub fn complete_manual_save(&mut self, success: bool, error: Option<SaveError>) {
        if success {
            self.manual_save.complete_success();
        } else if let Some(err) = error {
            self.manual_save.complete_error(err);
        }
    }

    /// Start auto-save
    pub fn start_autosave(&mut self) {
        self.autosave.start_save();
        self.save_count += 1;
    }

    /// Complete auto-save
    pub fn complete_autosave(&mut self, success: bool, error: Option<SaveError>) {
        if success {
            self.autosave.complete_success();
            self.reset_autosave_timer();
        } else if let Some(err) = error {
            self.autosave.complete_error(err);
        }
    }

    /// Update the manager
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.autosave.update(delta_time);
        self.manual_save.update(delta_time);

        // Track save time
        if self.autosave.is_saving() || self.manual_save.is_saving() {
            self.total_save_time += delta_time;
        }

        // Auto-save timer
        let mut should_autosave = false;
        if self.autosave_enabled && !self.autosave.is_saving() && !self.manual_save.is_saving() {
            self.autosave_timer -= delta_time;
            if self.autosave_timer <= 0.0 {
                should_autosave = true;
            }
        }

        should_autosave
    }

    /// Render both indicators
    pub fn render(&mut self, ui: &mut Ui) {
        self.autosave.render(ui);
        self.manual_save.render(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corner_position_all() {
        let positions = CornerPosition::all();
        assert_eq!(positions.len(), 4);
    }

    #[test]
    fn test_corner_position_display_name() {
        assert_eq!(CornerPosition::TopLeft.display_name(), "Top Left");
        assert_eq!(CornerPosition::TopRight.display_name(), "Top Right");
        assert_eq!(CornerPosition::BottomLeft.display_name(), "Bottom Left");
        assert_eq!(CornerPosition::BottomRight.display_name(), "Bottom Right");
    }

    #[test]
    fn test_corner_position_is_left() {
        assert!(CornerPosition::TopLeft.is_left());
        assert!(CornerPosition::BottomLeft.is_left());
        assert!(!CornerPosition::TopRight.is_left());
        assert!(!CornerPosition::BottomRight.is_left());
    }

    #[test]
    fn test_corner_position_is_top() {
        assert!(CornerPosition::TopLeft.is_top());
        assert!(CornerPosition::TopRight.is_top());
        assert!(!CornerPosition::BottomLeft.is_top());
        assert!(!CornerPosition::BottomRight.is_top());
    }

    #[test]
    fn test_save_state_display() {
        assert_eq!(SaveState::Idle.display_text(), "");
        assert_eq!(SaveState::Saving.display_text(), "Saving...");
        assert_eq!(SaveState::Success.display_text(), "Saved!");
        assert_eq!(SaveState::Error.display_text(), "Save Failed!");
    }

    #[test]
    fn test_save_state_icon() {
        assert_eq!(SaveState::Saving.icon(), "💾");
        assert_eq!(SaveState::Success.icon(), "✓");
        assert_eq!(SaveState::Error.icon(), "✕");
    }

    #[test]
    fn test_save_state_is_visible() {
        assert!(!SaveState::Idle.is_visible());
        assert!(SaveState::Saving.is_visible());
        assert!(SaveState::Success.is_visible());
        assert!(SaveState::Error.is_visible());
    }

    #[test]
    fn test_save_state_is_active() {
        assert!(!SaveState::Idle.is_active());
        assert!(SaveState::Saving.is_active());
        assert!(!SaveState::Success.is_active());
        assert!(!SaveState::Error.is_active());
    }

    #[test]
    fn test_animation_style_all() {
        let styles = AnimationStyle::all();
        assert_eq!(styles.len(), 4);
    }

    #[test]
    fn test_animation_style_display_name() {
        assert_eq!(AnimationStyle::Spin.display_name(), "Spin");
        assert_eq!(AnimationStyle::Pulse.display_name(), "Pulse");
        assert_eq!(AnimationStyle::Bounce.display_name(), "Bounce");
        assert_eq!(AnimationStyle::None.display_name(), "None");
    }

    #[test]
    fn test_fade_state_new() {
        let fade = FadeState::new(2.0);
        assert_eq!(fade.opacity, 0.0);
        assert_eq!(fade.target, 0.0);
        assert_eq!(fade.speed, 2.0);
    }

    #[test]
    fn test_fade_state_fade_in_out() {
        let mut fade = FadeState::new(2.0);

        fade.fade_in();
        assert_eq!(fade.target, 1.0);

        fade.fade_out();
        assert_eq!(fade.target, 0.0);
    }

    #[test]
    fn test_fade_state_set_immediate() {
        let mut fade = FadeState::new(2.0);
        fade.set_immediate(0.5);

        assert_eq!(fade.opacity, 0.5);
        assert_eq!(fade.target, 0.5);
    }

    #[test]
    fn test_fade_state_update() {
        let mut fade = FadeState::new(2.0);
        fade.fade_in();

        fade.update(0.3);
        assert!(fade.opacity > 0.0);
        assert!(fade.opacity < 1.0);

        fade.update(1.0);
        assert_eq!(fade.opacity, 1.0);
    }

    #[test]
    fn test_fade_state_is_fading() {
        let mut fade = FadeState::new(2.0);
        assert!(!fade.is_fading());

        fade.fade_in();
        assert!(fade.is_fading());

        fade.set_immediate(1.0);
        assert!(!fade.is_fading());
    }

    #[test]
    fn test_fade_state_visibility() {
        let mut fade = FadeState::new(2.0);
        assert!(fade.is_hidden());
        assert!(!fade.is_visible());

        fade.set_immediate(1.0);
        assert!(fade.is_visible());
        assert!(!fade.is_hidden());
    }

    #[test]
    fn test_autosave_config_defaults() {
        let config = AutosaveIndicatorConfig::default();
        assert_eq!(config.position, CornerPosition::TopRight);
        assert_eq!(config.spin_speed, 1.0);
        assert!(config.show_label);
        assert!(config.show_background);
    }

    #[test]
    fn test_save_error_new() {
        let error = SaveError::new("Disk full");
        assert_eq!(error.message, "Disk full");
        assert!(error.code.is_none());
        assert!(error.recoverable);
    }

    #[test]
    fn test_save_error_builders() {
        let error = SaveError::new("Error")
            .with_code(42)
            .with_timestamp(1000)
            .non_recoverable();

        assert_eq!(error.code, Some(42));
        assert_eq!(error.timestamp, 1000);
        assert!(!error.recoverable);
    }

    #[test]
    fn test_save_error_short_description() {
        let short = SaveError::new("Short");
        assert_eq!(short.short_description(), "Short");

        let long = SaveError::new("A".repeat(100));
        assert_eq!(long.short_description().len(), 50);
    }

    #[test]
    fn test_autosave_indicator_new() {
        let indicator = AutosaveIndicator::with_defaults();
        assert_eq!(indicator.state(), SaveState::Idle);
        assert!(!indicator.is_visible());
        assert!(!indicator.is_saving());
    }

    #[test]
    fn test_autosave_indicator_start_save() {
        let mut indicator = AutosaveIndicator::with_defaults();
        indicator.start_save();

        assert_eq!(indicator.state(), SaveState::Saving);
        assert!(indicator.is_saving());
    }

    #[test]
    fn test_autosave_indicator_with_progress() {
        let mut indicator = AutosaveIndicator::with_defaults();
        indicator.start_save_with_progress();

        assert_eq!(indicator.progress(), Some(0.0));

        indicator.set_progress(0.5);
        assert_eq!(indicator.progress(), Some(0.5));
    }

    #[test]
    fn test_autosave_indicator_complete_success() {
        let mut indicator = AutosaveIndicator::with_defaults();
        indicator.start_save();
        indicator.complete_success();

        assert_eq!(indicator.state(), SaveState::Success);
        assert!(indicator.progress().is_none());
    }

    #[test]
    fn test_autosave_indicator_complete_error() {
        let mut indicator = AutosaveIndicator::with_defaults();
        indicator.start_save();
        indicator.complete_error(SaveError::new("Failed"));

        assert_eq!(indicator.state(), SaveState::Error);
        assert!(indicator.last_error().is_some());
        assert_eq!(indicator.last_error().unwrap().message, "Failed");
    }

    #[test]
    fn test_autosave_indicator_clear_error() {
        let mut indicator = AutosaveIndicator::with_defaults();
        indicator.complete_error(SaveError::new("Failed"));

        indicator.clear_error();
        assert!(indicator.last_error().is_none());
    }

    #[test]
    fn test_autosave_indicator_set_position() {
        let mut indicator = AutosaveIndicator::with_defaults();
        indicator.set_position(CornerPosition::BottomLeft);

        assert_eq!(indicator.config().position, CornerPosition::BottomLeft);
    }

    #[test]
    fn test_autosave_indicator_update() {
        let mut indicator = AutosaveIndicator::with_defaults();
        indicator.start_save();

        indicator.update(0.1);
        // Should be animating
        assert!(indicator.rotation > 0.0 || indicator.anim_time > 0.0);
    }

    #[test]
    fn test_save_operation_manager_new() {
        let manager = SaveOperationManager::new(300.0);
        assert!(manager.is_autosave_enabled());
        assert_eq!(manager.autosave_interval(), 300.0);
        assert_eq!(manager.save_count(), 0);
    }

    #[test]
    fn test_save_operation_manager_autosave_toggle() {
        let mut manager = SaveOperationManager::with_defaults();

        manager.set_autosave_enabled(false);
        assert!(!manager.is_autosave_enabled());

        manager.set_autosave_enabled(true);
        assert!(manager.is_autosave_enabled());
    }

    #[test]
    fn test_save_operation_manager_interval() {
        let mut manager = SaveOperationManager::with_defaults();

        manager.set_autosave_interval(600.0);
        assert_eq!(manager.autosave_interval(), 600.0);

        // Minimum interval is 10 seconds
        manager.set_autosave_interval(5.0);
        assert_eq!(manager.autosave_interval(), 10.0);
    }

    #[test]
    fn test_save_operation_manager_manual_save() {
        let mut manager = SaveOperationManager::with_defaults();

        manager.start_manual_save();
        assert_eq!(manager.save_count(), 1);

        manager.complete_manual_save(true, None);
        assert_eq!(manager.manual_save_indicator().state(), SaveState::Success);
    }

    #[test]
    fn test_save_operation_manager_autosave() {
        let mut manager = SaveOperationManager::with_defaults();

        manager.start_autosave();
        assert_eq!(manager.save_count(), 1);

        manager.complete_autosave(true, None);
        assert_eq!(manager.autosave_indicator().state(), SaveState::Success);
    }

    #[test]
    fn test_save_operation_manager_reset_timer() {
        let mut manager = SaveOperationManager::new(300.0);

        // Simulate some time passing
        manager.autosave_timer = 50.0;

        manager.reset_autosave_timer();
        assert_eq!(manager.time_until_autosave(), 300.0);
    }

    #[test]
    fn test_save_operation_manager_update_triggers_autosave() {
        let mut manager = SaveOperationManager::new(1.0);

        // Update past the interval
        let should_save = manager.update(1.5);
        assert!(should_save);
    }

    #[test]
    fn test_save_state_color() {
        // Just verify colors are different
        assert_ne!(SaveState::Saving.color(), SaveState::Success.color());
        assert_ne!(SaveState::Success.color(), SaveState::Error.color());
    }

    #[test]
    fn test_corner_position_serialization() {
        let pos = CornerPosition::TopRight;
        let json = serde_json::to_string(&pos).unwrap();
        let parsed: CornerPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pos);
    }

    #[test]
    fn test_save_state_serialization() {
        let state = SaveState::Saving;
        let json = serde_json::to_string(&state).unwrap();
        let parsed: SaveState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_autosave_config_serialization() {
        let config = AutosaveIndicatorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AutosaveIndicatorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.position, config.position);
        assert_eq!(parsed.spin_speed, config.spin_speed);
    }

    #[test]
    fn test_save_error_serialization() {
        let error = SaveError::new("Test").with_code(42);
        let json = serde_json::to_string(&error).unwrap();
        let parsed: SaveError = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.message, error.message);
        assert_eq!(parsed.code, error.code);
    }
}
