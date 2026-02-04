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

/// Complete HUD state for rendering.
#[derive(Debug, Clone)]
pub struct HUDState {
    /// Player position (x, y).
    pub player_position: (f32, f32),
    /// Player velocity (x, y).
    pub player_velocity: (f32, f32),
    /// Current health.
    pub health_current: f32,
    /// Maximum health.
    pub health_max: f32,
    /// Current stamina (optional).
    pub stamina_current: Option<f32>,
    /// Maximum stamina (optional).
    pub stamina_max: Option<f32>,
    /// Currently selected hotbar slot (0-9).
    pub hotbar_selection: u8,
    /// Current FPS.
    pub fps: f32,
    /// Frame time in milliseconds.
    pub frame_time_ms: f32,
    /// Current material/tool ID.
    pub current_material: u16,
    /// Whether player is grounded.
    pub is_grounded: bool,
}

impl Default for HUDState {
    fn default() -> Self {
        Self {
            player_position: (0.0, 0.0),
            player_velocity: (0.0, 0.0),
            health_current: 100.0,
            health_max: 100.0,
            stamina_current: Some(100.0),
            stamina_max: Some(100.0),
            hotbar_selection: 0,
            fps: 60.0,
            frame_time_ms: 16.67,
            current_material: 0,
            is_grounded: true,
        }
    }
}

/// Main gameplay HUD renderer.
#[derive(Debug, Clone)]
pub struct GameHUD {
    /// Whether to show debug information.
    show_debug: bool,
    /// Whether to show the inventory panel.
    show_inventory: bool,
    /// Whether to show the crafting panel.
    show_crafting: bool,
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
    /// Size of the minimap in pixels.
    pub minimap_size: f32,
    /// Size of the tool indicator.
    pub tool_indicator_size: f32,
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
            minimap_size: 150.0,
            tool_indicator_size: 48.0,
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
            show_crafting: false,
            config: GameHUDConfig::default(),
        }
    }

    /// Create a new Game HUD with custom configuration.
    #[must_use]
    pub fn with_config(config: GameHUDConfig) -> Self {
        Self {
            show_debug: false,
            show_inventory: false,
            show_crafting: false,
            config,
        }
    }

    /// Render the full HUD using the provided render data.
    pub fn render(&mut self, data: &HUDRenderData<'_>) {
        self.render_vitals(data.ctx, data.health, data.stamina);
        self.render_hotbar(data.ctx, data.inventory);
        self.render_minimap(data.ctx, data.player.position().x, data.player.position().y);

        if self.show_debug {
            self.render_debug(data.ctx, data.fps, data.frame_time, data.player);
        }
    }

    /// Render the full HUD using HUDState (simplified interface).
    pub fn render_from_state(&mut self, ctx: &Context, state: &HUDState) {
        self.render_vitals_from_state(ctx, state);
        self.render_hotbar_from_state(ctx, state);
        self.render_minimap(ctx, state.player_position.0, state.player_position.1);
        self.render_tool_indicator(ctx, state.current_material);

        if self.show_debug {
            self.render_debug_from_state(ctx, state);
        }
    }

    /// Render vitals from HUDState.
    fn render_vitals_from_state(&self, ctx: &Context, state: &HUDState) {
        egui::Area::new(egui::Id::new("vitals_state_area"))
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
                        state.health_current,
                        state.health_max,
                        health_color_gradient,
                    );

                    ui.add_space(self.config.bar_spacing);

                    // Stamina bar (if available)
                    if let (Some(current), Some(max)) = (state.stamina_current, state.stamina_max) {
                        self.render_bar(ui, "Stamina", current, max, stamina_color);
                    }
                });
            });
    }

    /// Render hotbar from HUDState.
    fn render_hotbar_from_state(&self, ctx: &Context, state: &HUDState) {
        let screen_rect = ctx.screen_rect();
        let hotbar_width = 10.0 * (self.config.hotbar_slot_size + self.config.hotbar_spacing)
            - self.config.hotbar_spacing;

        let pos = Pos2::new(
            (screen_rect.width() - hotbar_width) / 2.0,
            screen_rect.height() - self.config.hotbar_padding - self.config.hotbar_slot_size,
        );

        egui::Area::new(egui::Id::new("hotbar_state_area"))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for i in 0..10 {
                        self.render_hotbar_slot_with_selection(
                            ui,
                            i,
                            i == state.hotbar_selection as usize,
                        );
                    }
                });
            });
    }

    /// Render debug from HUDState.
    fn render_debug_from_state(&self, ctx: &Context, state: &HUDState) {
        let screen_rect = ctx.screen_rect();

        egui::Area::new(egui::Id::new("debug_state_overlay"))
            .fixed_pos(Pos2::new(
                screen_rect.width() - 250.0 - self.config.vitals_padding,
                self.config.vitals_padding + self.config.minimap_size + 20.0,
            ))
            .show(ctx, |ui| {
                egui::Frame::dark_canvas(ui.style())
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 180))
                    .show(ui, |ui| {
                        ui.set_min_width(230.0);
                        ui.vertical(|ui| {
                            // FPS
                            let fps_col = fps_color(state.fps);
                            ui.label(
                                RichText::new(format!("FPS: {:.1}", state.fps))
                                    .color(fps_col)
                                    .size(self.config.debug_font_size),
                            );

                            ui.label(
                                RichText::new(format!("Frame: {:.2}ms", state.frame_time_ms))
                                    .size(self.config.debug_font_size),
                            );

                            ui.separator();

                            ui.label(
                                RichText::new(format!(
                                    "Pos: ({:.1}, {:.1})",
                                    state.player_position.0, state.player_position.1
                                ))
                                .size(self.config.debug_font_size),
                            );

                            ui.label(
                                RichText::new(format!(
                                    "Vel: ({:.1}, {:.1})",
                                    state.player_velocity.0, state.player_velocity.1
                                ))
                                .size(self.config.debug_font_size),
                            );

                            let chunk_x = (state.player_position.0 / 512.0).floor() as i32;
                            let chunk_y = (state.player_position.1 / 512.0).floor() as i32;
                            ui.label(
                                RichText::new(format!("Chunk: ({chunk_x}, {chunk_y})"))
                                    .size(self.config.debug_font_size),
                            );

                            ui.separator();

                            let grounded_text = if state.is_grounded {
                                "Grounded"
                            } else {
                                "Airborne"
                            };
                            ui.label(
                                RichText::new(grounded_text).size(self.config.debug_font_size),
                            );

                            ui.label(
                                RichText::new(format!("Material: {}", state.current_material))
                                    .size(self.config.debug_font_size),
                            );
                        });
                    });
            });
    }

    /// Render minimap (top-right).
    pub fn render_minimap(&self, ctx: &Context, player_x: f32, player_y: f32) {
        let screen_rect = ctx.screen_rect();
        let minimap_pos = Pos2::new(
            screen_rect.width() - self.config.minimap_size - self.config.vitals_padding,
            self.config.vitals_padding,
        );

        egui::Area::new(egui::Id::new("minimap_area"))
            .fixed_pos(minimap_pos)
            .show(ctx, |ui| {
                let size = Vec2::splat(self.config.minimap_size);
                let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
                let rect = response.rect;

                // Background
                painter.rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied(20, 30, 40, 200));

                // Border
                painter.rect_stroke(rect, 4.0, Stroke::new(2.0, Color32::from_rgb(60, 80, 100)));

                // Grid lines
                let grid_spacing = self.config.minimap_size / 4.0;
                for i in 1..4 {
                    let offset = i as f32 * grid_spacing;
                    // Vertical line
                    painter.line_segment(
                        [
                            rect.min + Vec2::new(offset, 0.0),
                            rect.min + Vec2::new(offset, self.config.minimap_size),
                        ],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 80, 100, 100)),
                    );
                    // Horizontal line
                    painter.line_segment(
                        [
                            rect.min + Vec2::new(0.0, offset),
                            rect.min + Vec2::new(self.config.minimap_size, offset),
                        ],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 80, 100, 100)),
                    );
                }

                // Player marker (center)
                let center = rect.center();
                painter.circle_filled(center, 4.0, Color32::from_rgb(255, 200, 100));
                painter.circle_stroke(center, 4.0, Stroke::new(1.0, Color32::WHITE));

                // Coordinates label
                painter.text(
                    rect.min + Vec2::new(4.0, 4.0),
                    egui::Align2::LEFT_TOP,
                    format!("({player_x:.0}, {player_y:.0})"),
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(180, 180, 180),
                );
            });
    }

    /// Render tool/material indicator (bottom-left, above hotbar).
    pub fn render_tool_indicator(&self, ctx: &Context, material_id: u16) {
        let screen_rect = ctx.screen_rect();
        let pos = Pos2::new(
            self.config.vitals_padding,
            screen_rect.height()
                - self.config.hotbar_padding
                - self.config.hotbar_slot_size
                - self.config.tool_indicator_size
                - 10.0,
        );

        egui::Area::new(egui::Id::new("tool_indicator_area"))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let size = Vec2::splat(self.config.tool_indicator_size);
                let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
                let rect = response.rect;

                // Background
                painter.rect_filled(rect, 4.0, Color32::from_rgb(50, 50, 60));

                // Border
                painter.rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(2.0, Color32::from_rgb(100, 100, 120)),
                );

                // Material color (placeholder based on ID)
                let inner_rect = Rect::from_center_size(
                    rect.center(),
                    Vec2::splat(self.config.tool_indicator_size - 12.0),
                );
                #[allow(clippy::cast_possible_truncation)]
                let material_color = material_color(material_id as u8);
                painter.rect_filled(inner_rect, 2.0, material_color);

                // Material ID label
                painter.text(
                    rect.max - Vec2::new(4.0, 4.0),
                    egui::Align2::RIGHT_BOTTOM,
                    format!("{material_id}"),
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            });
    }

    /// Render a hotbar slot with explicit selection state.
    fn render_hotbar_slot_with_selection(&self, ui: &mut Ui, slot: usize, is_selected: bool) {
        let size = Vec2::splat(self.config.hotbar_slot_size);
        let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
        let rect = response.rect;

        // Background
        let bg_color = if is_selected {
            Color32::from_rgb(80, 80, 100)
        } else {
            Color32::from_rgb(40, 40, 50)
        };
        painter.rect_filled(rect, 4.0, bg_color);

        // Border
        let border_color = if is_selected {
            Color32::from_rgb(200, 200, 255)
        } else {
            Color32::from_rgb(80, 80, 80)
        };
        painter.rect_stroke(rect, 4.0, Stroke::new(2.0, border_color));

        // Key number hint
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

    /// Toggle crafting panel visibility.
    pub fn toggle_crafting(&mut self) {
        self.show_crafting = !self.show_crafting;
    }

    /// Set crafting panel visibility.
    pub fn set_crafting_visible(&mut self, visible: bool) {
        self.show_crafting = visible;
    }

    /// Check if crafting panel is visible.
    #[must_use]
    pub fn is_crafting_visible(&self) -> bool {
        self.show_crafting
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

/// Returns a color for a material type.
/// Each material ID maps to a unique color.
#[must_use]
pub fn material_color(material_id: u8) -> Color32 {
    match material_id {
        0 => Color32::from_rgb(139, 90, 43),    // Dirt (brown)
        1 => Color32::from_rgb(128, 128, 128),  // Stone (gray)
        2 => Color32::from_rgb(34, 139, 34),    // Grass (green)
        3 => Color32::from_rgb(210, 180, 140),  // Sand (tan)
        4 => Color32::from_rgb(64, 164, 223),   // Water (blue)
        5 => Color32::from_rgb(255, 140, 0),    // Lava (orange)
        6 => Color32::from_rgb(139, 69, 19),    // Wood (saddle brown)
        7 => Color32::from_rgb(192, 192, 192),  // Iron (silver)
        8 => Color32::from_rgb(255, 215, 0),    // Gold (gold)
        9 => Color32::from_rgb(0, 191, 255),    // Diamond (deep sky blue)
        10 => Color32::from_rgb(50, 50, 50),    // Coal (dark gray)
        11 => Color32::from_rgb(255, 99, 71),   // Copper (tomato)
        12 => Color32::from_rgb(224, 224, 224), // Snow (white-ish)
        13 => Color32::from_rgb(144, 238, 144), // Leaf (light green)
        14 => Color32::from_rgb(178, 102, 255), // Crystal (purple)
        15 => Color32::from_rgb(255, 182, 193), // Clay (light pink)
        _ => Color32::from_rgb(255, 0, 255),    // Unknown (magenta)
    }
}

/// Returns the name for a material type.
#[must_use]
pub fn material_name(material_id: u8) -> &'static str {
    match material_id {
        0 => "Dirt",
        1 => "Stone",
        2 => "Grass",
        3 => "Sand",
        4 => "Water",
        5 => "Lava",
        6 => "Wood",
        7 => "Iron",
        8 => "Gold",
        9 => "Diamond",
        10 => "Coal",
        11 => "Copper",
        12 => "Snow",
        13 => "Leaf",
        14 => "Crystal",
        15 => "Clay",
        _ => "Unknown",
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

    #[test]
    fn test_game_hud_toggle_crafting() {
        let mut hud = GameHUD::new();
        assert!(!hud.is_crafting_visible());
        hud.toggle_crafting();
        assert!(hud.is_crafting_visible());
        hud.toggle_crafting();
        assert!(!hud.is_crafting_visible());
    }

    #[test]
    fn test_game_hud_set_crafting_visible() {
        let mut hud = GameHUD::new();
        assert!(!hud.is_crafting_visible());
        hud.set_crafting_visible(true);
        assert!(hud.is_crafting_visible());
        hud.set_crafting_visible(false);
        assert!(!hud.is_crafting_visible());
    }

    #[test]
    fn test_hud_state_default() {
        let state = HUDState::default();
        assert_eq!(state.player_position, (0.0, 0.0));
        assert_eq!(state.health_current, 100.0);
        assert_eq!(state.health_max, 100.0);
        assert_eq!(state.hotbar_selection, 0);
        assert_eq!(state.fps, 60.0);
        assert!(state.is_grounded);
    }

    #[test]
    fn test_material_color_known() {
        assert_eq!(material_color(0), Color32::from_rgb(139, 90, 43)); // Dirt
        assert_eq!(material_color(1), Color32::from_rgb(128, 128, 128)); // Stone
        assert_eq!(material_color(8), Color32::from_rgb(255, 215, 0)); // Gold
    }

    #[test]
    fn test_material_color_unknown() {
        assert_eq!(material_color(255), Color32::from_rgb(255, 0, 255)); // Magenta
    }

    #[test]
    fn test_material_name_known() {
        assert_eq!(material_name(0), "Dirt");
        assert_eq!(material_name(1), "Stone");
        assert_eq!(material_name(8), "Gold");
    }

    #[test]
    fn test_material_name_unknown() {
        assert_eq!(material_name(255), "Unknown");
    }

    #[test]
    fn test_game_hud_config_minimap_and_tool_indicator() {
        let config = GameHUDConfig::default();
        assert_eq!(config.minimap_size, 150.0);
        assert_eq!(config.tool_indicator_size, 48.0);
    }
}
