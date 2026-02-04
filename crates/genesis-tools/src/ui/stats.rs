//! Player Stats HUD for displaying health, hunger, and stamina.
//!
//! This module provides:
//! - Health bar (red)
//! - Hunger bar (orange)
//! - Stamina bar (green)
//! - Smooth bar animations
//! - Low value flashing warnings

use egui::{Color32, Context, Id, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Default bar width in pixels.
pub const STATS_BAR_WIDTH: f32 = 180.0;

/// Default bar height in pixels.
pub const STATS_BAR_HEIGHT: f32 = 18.0;

/// Low health threshold for flashing warning.
pub const LOW_HEALTH_THRESHOLD: f32 = 0.25;

/// Low hunger threshold for flashing warning.
pub const LOW_HUNGER_THRESHOLD: f32 = 0.20;

/// Low stamina threshold for flashing warning.
pub const LOW_STAMINA_THRESHOLD: f32 = 0.15;

/// Flash rate in seconds.
pub const FLASH_RATE: f32 = 0.5;

/// Animation smoothing factor (lower = smoother).
pub const ANIMATION_SMOOTHING: f32 = 0.1;

/// A stat bar value with animation state.
#[derive(Debug, Clone)]
pub struct StatValue {
    /// Current value (0.0 - 1.0).
    pub current: f32,
    /// Maximum value.
    pub max: f32,
    /// Display value (animated).
    display: f32,
}

impl Default for StatValue {
    fn default() -> Self {
        Self {
            current: 1.0,
            max: 100.0,
            display: 1.0,
        }
    }
}

impl StatValue {
    /// Creates a new stat value.
    #[must_use]
    pub fn new(current: f32, max: f32) -> Self {
        let normalized = if max > 0.0 { current / max } else { 0.0 };
        Self {
            current,
            max,
            display: normalized,
        }
    }

    /// Returns the normalized value (0.0 - 1.0).
    #[must_use]
    pub fn normalized(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Updates the display value with smooth animation.
    pub fn update_animation(&mut self, dt: f32) {
        let target = self.normalized();
        let diff = target - self.display;
        self.display += diff * (ANIMATION_SMOOTHING * dt * 60.0).min(1.0);
    }

    /// Returns the animated display value.
    #[must_use]
    pub fn display_value(&self) -> f32 {
        self.display
    }

    /// Sets the stat values.
    pub fn set(&mut self, current: f32, max: f32) {
        self.current = current;
        self.max = max;
    }
}

/// Player stats data model.
#[derive(Debug, Clone, Default)]
pub struct StatsHudModel {
    /// Health stat.
    pub health: StatValue,
    /// Hunger stat.
    pub hunger: StatValue,
    /// Stamina stat.
    pub stamina: StatValue,
    /// Status effects (name, duration, color).
    pub status_effects: Vec<(String, f32, [u8; 4])>,
}

impl StatsHudModel {
    /// Creates a new stats model with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            health: StatValue::new(100.0, 100.0),
            hunger: StatValue::new(100.0, 100.0),
            stamina: StatValue::new(100.0, 100.0),
            status_effects: Vec::new(),
        }
    }

    /// Creates with specific values.
    #[must_use]
    pub fn with_values(health: (f32, f32), hunger: (f32, f32), stamina: (f32, f32)) -> Self {
        Self {
            health: StatValue::new(health.0, health.1),
            hunger: StatValue::new(hunger.0, hunger.1),
            stamina: StatValue::new(stamina.0, stamina.1),
            status_effects: Vec::new(),
        }
    }

    /// Updates animation state.
    pub fn update(&mut self, dt: f32) {
        self.health.update_animation(dt);
        self.hunger.update_animation(dt);
        self.stamina.update_animation(dt);
    }

    /// Adds a status effect.
    pub fn add_effect(&mut self, name: impl Into<String>, duration: f32, color: [u8; 4]) {
        self.status_effects.push((name.into(), duration, color));
    }

    /// Clears expired status effects.
    pub fn clear_expired_effects(&mut self) {
        self.status_effects
            .retain(|(_, duration, _)| *duration > 0.0);
    }
}

/// Configuration for the stats HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsHudConfig {
    /// Bar width in pixels.
    pub bar_width: f32,
    /// Bar height in pixels.
    pub bar_height: f32,
    /// Spacing between bars.
    pub bar_spacing: f32,
    /// Position from top-left.
    pub position: (f32, f32),
    /// Health bar color.
    pub health_color: [u8; 4],
    /// Hunger bar color.
    pub hunger_color: [u8; 4],
    /// Stamina bar color.
    pub stamina_color: [u8; 4],
    /// Background color.
    pub background_color: [u8; 4],
    /// Show labels.
    pub show_labels: bool,
    /// Show numeric values.
    pub show_values: bool,
}

impl Default for StatsHudConfig {
    fn default() -> Self {
        Self {
            bar_width: STATS_BAR_WIDTH,
            bar_height: STATS_BAR_HEIGHT,
            bar_spacing: 6.0,
            position: (10.0, 80.0),            // Below debug panel
            health_color: [220, 50, 50, 255],  // Red
            hunger_color: [230, 150, 50, 255], // Orange
            stamina_color: [50, 200, 80, 255], // Green
            background_color: [30, 30, 30, 200],
            show_labels: true,
            show_values: true,
        }
    }
}

/// Stats HUD widget.
#[derive(Debug)]
pub struct StatsHud {
    /// Configuration.
    config: StatsHudConfig,
    /// Flash timer for low value warnings.
    flash_timer: f32,
    /// Whether currently in flash-on state.
    flash_on: bool,
    /// Start time for animation.
    start_time: Option<Instant>,
}

impl Default for StatsHud {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsHud {
    /// Creates a new stats HUD.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: StatsHudConfig::default(),
            flash_timer: 0.0,
            flash_on: true,
            start_time: None,
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: StatsHudConfig) -> Self {
        Self {
            config,
            flash_timer: 0.0,
            flash_on: true,
            start_time: None,
        }
    }

    /// Updates the HUD state (call each frame).
    pub fn update(&mut self, dt: f32) {
        self.flash_timer += dt;
        if self.flash_timer >= FLASH_RATE {
            self.flash_timer = 0.0;
            self.flash_on = !self.flash_on;
        }
    }

    /// Shows the stats HUD.
    pub fn show(&mut self, ctx: &Context, model: &StatsHudModel) {
        // Initialize start time if needed
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        egui::Area::new(Id::new("stats_hud"))
            .fixed_pos(Pos2::new(self.config.position.0, self.config.position.1))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Health bar
                    let health_flash = model.health.normalized() < LOW_HEALTH_THRESHOLD;
                    self.render_stat_bar(
                        ui,
                        "Health",
                        model.health.display_value(),
                        model.health.current,
                        model.health.max,
                        self.config.health_color,
                        health_flash,
                    );

                    ui.add_space(self.config.bar_spacing);

                    // Hunger bar
                    let hunger_flash = model.hunger.normalized() < LOW_HUNGER_THRESHOLD;
                    self.render_stat_bar(
                        ui,
                        "Hunger",
                        model.hunger.display_value(),
                        model.hunger.current,
                        model.hunger.max,
                        self.config.hunger_color,
                        hunger_flash,
                    );

                    ui.add_space(self.config.bar_spacing);

                    // Stamina bar
                    let stamina_flash = model.stamina.normalized() < LOW_STAMINA_THRESHOLD;
                    self.render_stat_bar(
                        ui,
                        "Stamina",
                        model.stamina.display_value(),
                        model.stamina.current,
                        model.stamina.max,
                        self.config.stamina_color,
                        stamina_flash,
                    );

                    // Status effects
                    if !model.status_effects.is_empty() {
                        ui.add_space(self.config.bar_spacing * 2.0);
                        Self::render_status_effects(ui, &model.status_effects);
                    }
                });
            });
    }

    /// Renders a single stat bar.
    #[allow(clippy::too_many_arguments)]
    fn render_stat_bar(
        &self,
        ui: &mut Ui,
        label: &str,
        display_value: f32,
        current: f32,
        max: f32,
        color: [u8; 4],
        should_flash: bool,
    ) {
        let bar_width = self.config.bar_width;
        let bar_height = self.config.bar_height;

        ui.horizontal(|ui| {
            // Label
            if self.config.show_labels {
                ui.label(RichText::new(format!("{label}:")).strong().size(12.0));
            }

            // Bar background
            let (rect, _response) =
                ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();

                // Background
                let bg_color = Color32::from_rgba_unmultiplied(
                    self.config.background_color[0],
                    self.config.background_color[1],
                    self.config.background_color[2],
                    self.config.background_color[3],
                );
                painter.rect_filled(rect, Rounding::same(4.0), bg_color);

                // Fill
                let fill_width = bar_width * display_value.clamp(0.0, 1.0);
                if fill_width > 0.0 {
                    let fill_rect =
                        Rect::from_min_size(rect.min, Vec2::new(fill_width, bar_height));

                    // Apply flash effect
                    let alpha = if should_flash && !self.flash_on {
                        (color[3] as f32 * 0.4) as u8
                    } else {
                        color[3]
                    };

                    let fill_color =
                        Color32::from_rgba_unmultiplied(color[0], color[1], color[2], alpha);
                    painter.rect_filled(fill_rect, Rounding::same(4.0), fill_color);
                }

                // Border
                painter.rect_stroke(
                    rect,
                    Rounding::same(4.0),
                    Stroke::new(1.0, Color32::from_gray(80)),
                );

                // Value text inside bar
                if self.config.show_values {
                    let text = format!("{}/{}", current as i32, max as i32);
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::proportional(11.0),
                        Color32::WHITE,
                    );
                }
            }
        });
    }

    /// Renders status effect icons.
    fn render_status_effects(ui: &mut Ui, effects: &[(String, f32, [u8; 4])]) {
        ui.horizontal(|ui| {
            for (name, duration, color) in effects {
                let effect_color =
                    Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);

                ui.vertical(|ui| {
                    // Icon (colored square)
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        ui.painter()
                            .rect_filled(rect, Rounding::same(3.0), effect_color);
                    }

                    // Duration
                    ui.label(
                        RichText::new(format!("{duration:.0}s"))
                            .size(9.0)
                            .color(Color32::GRAY),
                    );
                })
                .response
                .on_hover_text(name);

                ui.add_space(4.0);
            }
        });
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &StatsHudConfig {
        &self.config
    }

    /// Sets the configuration.
    pub fn set_config(&mut self, config: StatsHudConfig) {
        self.config = config;
    }
}

/// Returns a color that transitions from green to red based on value.
#[must_use]
pub fn health_gradient_color(value: f32) -> Color32 {
    let v = value.clamp(0.0, 1.0);
    if v > 0.5 {
        // Green to yellow
        let t = (v - 0.5) * 2.0;
        Color32::from_rgb((255.0 * (1.0 - t)) as u8, 255, 0)
    } else {
        // Yellow to red
        let t = v * 2.0;
        Color32::from_rgb(255, (255.0 * t) as u8, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_value_default() {
        let stat = StatValue::default();
        assert!((stat.current - 1.0).abs() < f32::EPSILON);
        assert!((stat.max - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stat_value_new() {
        let stat = StatValue::new(50.0, 100.0);
        assert!((stat.current - 50.0).abs() < f32::EPSILON);
        assert!((stat.normalized() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stat_value_normalized() {
        let stat = StatValue::new(25.0, 100.0);
        assert!((stat.normalized() - 0.25).abs() < f32::EPSILON);

        let zero_max = StatValue::new(50.0, 0.0);
        assert!((zero_max.normalized()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stat_value_animation() {
        let mut stat = StatValue::new(100.0, 100.0);
        stat.set(50.0, 100.0);

        // Display should still be at initial value
        assert!((stat.display_value() - 1.0).abs() < f32::EPSILON);

        // After update, display should move toward target
        stat.update_animation(0.1);
        assert!(stat.display_value() < 1.0);
    }

    #[test]
    fn test_stats_hud_model_default() {
        let model = StatsHudModel::new();
        assert!((model.health.normalized() - 1.0).abs() < f32::EPSILON);
        assert!((model.hunger.normalized() - 1.0).abs() < f32::EPSILON);
        assert!((model.stamina.normalized() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stats_hud_model_with_values() {
        let model = StatsHudModel::with_values((50.0, 100.0), (75.0, 100.0), (25.0, 100.0));
        assert!((model.health.normalized() - 0.5).abs() < f32::EPSILON);
        assert!((model.hunger.normalized() - 0.75).abs() < f32::EPSILON);
        assert!((model.stamina.normalized() - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stats_hud_model_effects() {
        let mut model = StatsHudModel::new();
        assert!(model.status_effects.is_empty());

        model.add_effect("Poison", 10.0, [100, 200, 50, 255]);
        assert_eq!(model.status_effects.len(), 1);
        assert_eq!(model.status_effects[0].0, "Poison");
    }

    #[test]
    fn test_stats_hud_config_defaults() {
        let config = StatsHudConfig::default();
        assert!((config.bar_width - STATS_BAR_WIDTH).abs() < f32::EPSILON);
        assert!((config.bar_height - STATS_BAR_HEIGHT).abs() < f32::EPSILON);
        assert!(config.show_labels);
        assert!(config.show_values);
    }

    #[test]
    fn test_stats_hud_new() {
        let hud = StatsHud::new();
        assert!(hud.flash_on);
        assert!((hud.flash_timer).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stats_hud_flash_update() {
        let mut hud = StatsHud::new();
        assert!(hud.flash_on);

        // Update past flash rate
        hud.update(FLASH_RATE + 0.1);
        assert!(!hud.flash_on);

        hud.update(FLASH_RATE + 0.1);
        assert!(hud.flash_on);
    }

    #[test]
    fn test_health_gradient_color() {
        let full = health_gradient_color(1.0);
        assert_eq!(full, Color32::from_rgb(0, 255, 0)); // Green

        let half = health_gradient_color(0.5);
        assert_eq!(half, Color32::from_rgb(255, 255, 0)); // Yellow

        let empty = health_gradient_color(0.0);
        assert_eq!(empty, Color32::from_rgb(255, 0, 0)); // Red
    }

    #[test]
    fn test_low_thresholds() {
        assert!(LOW_HEALTH_THRESHOLD > 0.0 && LOW_HEALTH_THRESHOLD < 1.0);
        assert!(LOW_HUNGER_THRESHOLD > 0.0 && LOW_HUNGER_THRESHOLD < 1.0);
        assert!(LOW_STAMINA_THRESHOLD > 0.0 && LOW_STAMINA_THRESHOLD < 1.0);
    }
}
