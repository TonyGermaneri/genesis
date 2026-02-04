//! Game HUD renderer for displaying player vitals, hotbar, and debug info.
//!
//! This module provides the main gameplay HUD overlay using egui.

use egui::{Color32, Context, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use genesis_gameplay::{Health, Inventory, Need, Player};

/// Data needed to render the HUD each frame.
#[derive(Debug)]
pub struct HUDRenderData<'a> {
    /// The egui context.
    pub ctx: &'a Context,
    /// The player entity.
    pub player: &'a Player,
    /// Player health.
    pub health: &'a Health,
    /// Player stamina (optional).
    pub stamina: Option<&'a Need>,
    /// Player inventory.
    pub inventory: &'a Inventory,
    /// Current FPS.
    pub fps: f32,
    /// Frame time in milliseconds.
    pub frame_time: f32,
}

/// Main gameplay HUD renderer.
#[derive(Debug, Clone)]
pub struct GameHUD {
    /// Whether to show debug information.
    show_debug: bool,
    /// Whether to show the inventory panel.
    show_inventory: bool,
    /// Configuration for the HUD.
    config: GameHUDConfig,
}

/// Configuration for the Game HUD.
#[derive(Debug, Clone)]
pub struct GameHUDConfig {
    /// Width of health/stamina bars in pixels.
    pub bar_width: f32,
    /// Height of health/stamina bars in pixels.
    pub bar_height: f32,
    /// Spacing between bars.
    pub bar_spacing: f32,
    /// Padding from screen edge for vitals.
    pub vitals_padding: f32,
    /// Size of hotbar slots in pixels.
    pub hotbar_slot_size: f32,
    /// Spacing between hotbar slots.
    pub hotbar_spacing: f32,
    /// Padding from bottom of screen for hotbar.
    pub hotbar_padding: f32,
    /// Font size for debug text.
    pub debug_font_size: f32,
}

impl Default for GameHUDConfig {
    fn default() -> Self {
        Self {
            bar_width: 200.0,
            bar_height: 20.0,
            bar_spacing: 8.0,
            vitals_padding: 20.0,
            hotbar_slot_size: 48.0,
            hotbar_spacing: 4.0,
            hotbar_padding: 20.0,
            debug_font_size: 14.0,
        }
    }
}

impl Default for GameHUD {
    fn default() -> Self {
        Self::new()
    }
}

impl GameHUD {
    /// Create a new Game HUD with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            show_debug: false,
            show_inventory: false,
            config: GameHUDConfig::default(),
        }
    }

    /// Create a new Game HUD with custom configuration.
    #[must_use]
    pub fn with_config(config: GameHUDConfig) -> Self {
        Self {
            show_debug: false,
            show_inventory: false,
            config,
        }
    }

    /// Render the full HUD using the provided render data.
    pub fn render(&mut self, data: &HUDRenderData<'_>) {
        self.render_vitals(data.ctx, data.health, data.stamina);
        self.render_hotbar(data.ctx, data.inventory);

        if self.show_debug {
            self.render_debug(data.ctx, data.fps, data.frame_time, data.player);
        }
    }

    /// Render health/stamina bars (top-left).
    pub fn render_vitals(&self, ctx: &Context, health: &Health, stamina: Option<&Need>) {
        egui::Area::new(egui::Id::new("vitals_area"))
            .fixed_pos(Pos2::new(
                self.config.vitals_padding,
                self.config.vitals_padding,
            ))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Health bar
                    self.render_bar(
                        ui,
                        "HP",
                        health.current() as f32,
                        health.max() as f32,
                        health_color_gradient,
                    );

                    ui.add_space(self.config.bar_spacing);

                    // Stamina bar (if provided)
                    if let Some(stamina) = stamina {
                        self.render_bar(
                            ui,
                            stamina.name(),
                            stamina.current(),
                            stamina.max(),
                            stamina_color,
                        );
                    }
                });
            });
    }

    /// Render a progress bar with label and custom color function.
    fn render_bar(
        &self,
        ui: &mut Ui,
        label: &str,
        current: f32,
        max: f32,
        color_fn: fn(f32) -> Color32,
    ) {
        let percentage = if max > 0.0 { current / max } else { 0.0 };
        let color = color_fn(percentage);

        // Background
        let (response, painter) = ui.allocate_painter(
            Vec2::new(self.config.bar_width, self.config.bar_height),
            egui::Sense::hover(),
        );
        let rect = response.rect;

        // Background fill
        painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 30));

        // Progress fill
        let fill_width = rect.width() * percentage.clamp(0.0, 1.0);
        let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
        painter.rect_filled(fill_rect, 4.0, color);

        // Border
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(60, 60, 60)));

        // Text overlay
        let text = format!("{label}: {current:.0}/{max:.0}");
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

    /// Render hotbar (bottom-center).
    pub fn render_hotbar(&self, ctx: &Context, inventory: &Inventory) {
        let screen_rect = ctx.screen_rect();
        let hotbar_width = 10.0 * (self.config.hotbar_slot_size + self.config.hotbar_spacing)
            - self.config.hotbar_spacing;

        let pos = Pos2::new(
            (screen_rect.width() - hotbar_width) / 2.0,
            screen_rect.height() - self.config.hotbar_padding - self.config.hotbar_slot_size,
        );

        egui::Area::new(egui::Id::new("hotbar_area"))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for i in 0..10 {
                        self.render_hotbar_slot(ui, i, inventory);
                    }
                });
            });
    }

    /// Render a single hotbar slot.
    fn render_hotbar_slot(&self, ui: &mut Ui, slot: usize, _inventory: &Inventory) {
        let size = Vec2::splat(self.config.hotbar_slot_size);
        let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
        let rect = response.rect;

        // Background
        let bg_color = if slot == 0 {
            // First slot is typically selected
            Color32::from_rgb(80, 80, 100)
        } else {
            Color32::from_rgb(40, 40, 50)
        };
        painter.rect_filled(rect, 4.0, bg_color);

        // Border
        let border_color = if slot == 0 {
            Color32::from_rgb(200, 200, 255)
        } else {
            Color32::from_rgb(80, 80, 80)
        };
        painter.rect_stroke(rect, 4.0, Stroke::new(2.0, border_color));

        // Key number hint (1-9, 0)
        let key_text = if slot == 9 {
            "0"
        } else {
            &format!("{}", slot + 1)
        };
        painter.text(
            rect.min + Vec2::new(4.0, 4.0),
            egui::Align2::LEFT_TOP,
            key_text,
            egui::FontId::proportional(10.0),
            Color32::from_rgb(150, 150, 150),
        );
    }

    /// Render debug info (top-right, toggleable).
    pub fn render_debug(&self, ctx: &Context, fps: f32, frame_time: f32, player: &Player) {
        let screen_rect = ctx.screen_rect();

        egui::Area::new(egui::Id::new("debug_overlay"))
            .fixed_pos(Pos2::new(
                screen_rect.width() - 250.0 - self.config.vitals_padding,
                self.config.vitals_padding,
            ))
            .show(ctx, |ui| {
                egui::Frame::dark_canvas(ui.style())
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 180))
                    .show(ui, |ui| {
                        ui.set_min_width(230.0);
                        ui.vertical(|ui| {
                            // FPS
                            let fps_color = fps_color(fps);
                            ui.label(
                                RichText::new(format!("FPS: {fps:.1}"))
                                    .color(fps_color)
                                    .size(self.config.debug_font_size),
                            );

                            // Frame time
                            ui.label(
                                RichText::new(format!("Frame: {frame_time:.2}ms"))
                                    .size(self.config.debug_font_size),
                            );

                            ui.separator();

                            // Position
                            let pos = player.position();
                            ui.label(
                                RichText::new(format!("Pos: ({:.1}, {:.1})", pos.x, pos.y))
                                    .size(self.config.debug_font_size),
                            );

                            // Velocity
                            let vel = player.velocity();
                            ui.label(
                                RichText::new(format!("Vel: ({:.1}, {:.1})", vel.x, vel.y))
                                    .size(self.config.debug_font_size),
                            );

                            // Chunk coordinates
                            let chunk_x = (pos.x / 512.0).floor() as i32;
                            let chunk_y = (pos.y / 512.0).floor() as i32;
                            ui.label(
                                RichText::new(format!("Chunk: ({chunk_x}, {chunk_y})"))
                                    .size(self.config.debug_font_size),
                            );

                            ui.separator();

                            // Player state
                            ui.label(
                                RichText::new(format!("State: {:?}", player.state()))
                                    .size(self.config.debug_font_size),
                            );

                            // Grounded status
                            ui.label(
                                RichText::new(format!("Grounded: {}", player.is_grounded()))
                                    .size(self.config.debug_font_size),
                            );
                        });
                    });
            });
    }

    /// Toggle debug overlay visibility.
    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }

    /// Set debug overlay visibility.
    pub fn set_debug_visible(&mut self, visible: bool) {
        self.show_debug = visible;
    }

    /// Check if debug overlay is visible.
    #[must_use]
    pub fn is_debug_visible(&self) -> bool {
        self.show_debug
    }

    /// Toggle inventory panel visibility.
    pub fn toggle_inventory(&mut self) {
        self.show_inventory = !self.show_inventory;
    }

    /// Set inventory panel visibility.
    pub fn set_inventory_visible(&mut self, visible: bool) {
        self.show_inventory = visible;
    }

    /// Check if inventory panel is visible.
    #[must_use]
    pub fn is_inventory_visible(&self) -> bool {
        self.show_inventory
    }

    /// Get the HUD configuration.
    #[must_use]
    pub fn config(&self) -> &GameHUDConfig {
        &self.config
    }

    /// Set the HUD configuration.
    pub fn set_config(&mut self, config: GameHUDConfig) {
        self.config = config;
    }
}

/// Returns a color for health based on percentage (0.0-1.0).
fn health_color_gradient(percentage: f32) -> Color32 {
    if percentage > 0.6 {
        // Green to yellow
        let t = (percentage - 0.6) / 0.4;
        Color32::from_rgb((255.0 * (1.0 - t)) as u8, 255, 0)
    } else if percentage > 0.3 {
        // Yellow to orange
        let t = (percentage - 0.3) / 0.3;
        Color32::from_rgb(255, (200.0 + 55.0 * t) as u8, 0)
    } else {
        // Orange to red
        let t = percentage / 0.3;
        Color32::from_rgb(255, (100.0 * t) as u8, 0)
    }
}

/// Returns a color for stamina (always blue-ish).
fn stamina_color(_percentage: f32) -> Color32 {
    Color32::from_rgb(60, 150, 200)
}

/// Returns a color for FPS display based on value.
fn fps_color(fps: f32) -> Color32 {
    if fps >= 60.0 {
        Color32::GREEN
    } else if fps >= 30.0 {
        Color32::YELLOW
    } else {
        Color32::RED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_hud_new() {
        let hud = GameHUD::new();
        assert!(!hud.is_debug_visible());
        assert!(!hud.is_inventory_visible());
    }

    #[test]
    fn test_game_hud_toggle_debug() {
        let mut hud = GameHUD::new();
        assert!(!hud.is_debug_visible());
        hud.toggle_debug();
        assert!(hud.is_debug_visible());
        hud.toggle_debug();
        assert!(!hud.is_debug_visible());
    }

    #[test]
    fn test_game_hud_toggle_inventory() {
        let mut hud = GameHUD::new();
        assert!(!hud.is_inventory_visible());
        hud.toggle_inventory();
        assert!(hud.is_inventory_visible());
        hud.toggle_inventory();
        assert!(!hud.is_inventory_visible());
    }

    #[test]
    fn test_game_hud_config_defaults() {
        let config = GameHUDConfig::default();
        assert_eq!(config.bar_width, 200.0);
        assert_eq!(config.bar_height, 20.0);
        assert_eq!(config.hotbar_slot_size, 48.0);
    }

    #[test]
    fn test_game_hud_with_config() {
        let config = GameHUDConfig {
            bar_width: 300.0,
            ..Default::default()
        };
        let hud = GameHUD::with_config(config);
        assert_eq!(hud.config().bar_width, 300.0);
    }

    #[test]
    fn test_health_color_gradient_high() {
        let color = health_color_gradient(0.9);
        // Should be greenish
        assert!(color.g() > color.r());
    }

    #[test]
    fn test_health_color_gradient_low() {
        let color = health_color_gradient(0.1);
        // Should be reddish
        assert!(color.r() > color.g());
    }

    #[test]
    fn test_fps_color_good() {
        assert_eq!(fps_color(60.0), Color32::GREEN);
        assert_eq!(fps_color(120.0), Color32::GREEN);
    }

    #[test]
    fn test_fps_color_medium() {
        assert_eq!(fps_color(45.0), Color32::YELLOW);
    }

    #[test]
    fn test_fps_color_bad() {
        assert_eq!(fps_color(20.0), Color32::RED);
    }

    #[test]
    fn test_game_hud_set_visibility() {
        let mut hud = GameHUD::new();

        hud.set_debug_visible(true);
        assert!(hud.is_debug_visible());

        hud.set_inventory_visible(true);
        assert!(hud.is_inventory_visible());

        hud.set_debug_visible(false);
        assert!(!hud.is_debug_visible());
    }
}
