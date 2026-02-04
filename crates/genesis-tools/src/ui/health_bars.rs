//! Health and stamina bar UI components.
//!
//! Provides health bar functionality including:
//! - Player health bar (top-left position)
//! - Stamina bar below health
//! - Target health bar (when locked on)
//! - Smooth interpolation on damage

use egui::{Color32, Pos2, Rect, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Unique identifier for an entity with health.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    /// Create a new entity ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A stat value with current, max, and animated display value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedStatValue {
    /// Current value.
    pub current: f32,
    /// Maximum value.
    pub max: f32,
    /// Display value (animated towards current).
    pub display: f32,
    /// Previous value for damage tracking.
    pub previous: f32,
    /// Animation speed (units per second).
    pub animation_speed: f32,
    /// Time since last change.
    pub change_time: f32,
}

impl Default for AnimatedStatValue {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            display: 100.0,
            previous: 100.0,
            animation_speed: 50.0,
            change_time: 0.0,
        }
    }
}

impl AnimatedStatValue {
    /// Create a new animated stat value.
    pub fn new(current: f32, max: f32) -> Self {
        Self {
            current,
            max,
            display: current,
            previous: current,
            animation_speed: 50.0,
            change_time: 0.0,
        }
    }

    /// Set the current value.
    pub fn set(&mut self, value: f32) {
        self.previous = self.current;
        self.current = value.clamp(0.0, self.max);
        self.change_time = 0.0;
    }

    /// Set the maximum value.
    pub fn set_max(&mut self, max: f32) {
        self.max = max.max(1.0);
        self.current = self.current.min(self.max);
        self.display = self.display.min(self.max);
    }

    /// Apply damage (reduces current).
    pub fn damage(&mut self, amount: f32) {
        self.set(self.current - amount);
    }

    /// Apply healing (increases current).
    pub fn heal(&mut self, amount: f32) {
        self.set(self.current + amount);
    }

    /// Get normalized value (0.0 - 1.0).
    pub fn normalized(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Get normalized display value (0.0 - 1.0).
    pub fn display_normalized(&self) -> f32 {
        if self.max > 0.0 {
            (self.display / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if at full.
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Check if empty (zero).
    pub fn is_empty(&self) -> bool {
        self.current <= 0.0
    }

    /// Get damage taken since last change.
    pub fn recent_damage(&self) -> f32 {
        (self.previous - self.current).max(0.0)
    }

    /// Update animation.
    pub fn update(&mut self, dt: f32) {
        self.change_time += dt;

        // Animate display towards current
        if (self.display - self.current).abs() > 0.01 {
            let direction = if self.current > self.display {
                1.0
            } else {
                -1.0
            };
            let step = self.animation_speed * dt;

            if direction > 0.0 {
                self.display = (self.display + step).min(self.current);
            } else {
                self.display = (self.display - step).max(self.current);
            }
        } else {
            self.display = self.current;
        }
    }
}

/// Health bar style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HealthBarStyle {
    /// Standard horizontal bar.
    #[default]
    Standard,
    /// Segmented bar (like souls games).
    Segmented,
    /// Curved/arc style.
    Curved,
    /// Minimal thin bar.
    Minimal,
}

impl HealthBarStyle {
    /// Get all styles.
    pub fn all() -> &'static [HealthBarStyle] {
        &[
            HealthBarStyle::Standard,
            HealthBarStyle::Segmented,
            HealthBarStyle::Curved,
            HealthBarStyle::Minimal,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            HealthBarStyle::Standard => "Standard",
            HealthBarStyle::Segmented => "Segmented",
            HealthBarStyle::Curved => "Curved",
            HealthBarStyle::Minimal => "Minimal",
        }
    }
}

/// Configuration for health bar colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthBarColors {
    /// Full health color (RGBA).
    pub full: [u8; 4],
    /// Low health color (RGBA).
    pub low: [u8; 4],
    /// Critical health color (RGBA).
    pub critical: [u8; 4],
    /// Background color (RGBA).
    pub background: [u8; 4],
    /// Border color (RGBA).
    pub border: [u8; 4],
    /// Damage preview color (RGBA).
    pub damage_preview: [u8; 4],
}

impl Default for HealthBarColors {
    fn default() -> Self {
        Self {
            full: [100, 200, 100, 255],           // Green
            low: [200, 200, 100, 255],            // Yellow
            critical: [200, 80, 80, 255],         // Red
            background: [40, 40, 40, 200],        // Dark gray
            border: [80, 80, 80, 255],            // Gray
            damage_preview: [255, 100, 100, 180], // Light red
        }
    }
}

impl HealthBarColors {
    /// Get full health color.
    pub fn full_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.full[0], self.full[1], self.full[2], self.full[3])
    }

    /// Get low health color.
    pub fn low_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.low[0], self.low[1], self.low[2], self.low[3])
    }

    /// Get critical health color.
    pub fn critical_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.critical[0],
            self.critical[1],
            self.critical[2],
            self.critical[3],
        )
    }

    /// Get background color.
    pub fn background_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.background[0],
            self.background[1],
            self.background[2],
            self.background[3],
        )
    }

    /// Get border color.
    pub fn border_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.border[0],
            self.border[1],
            self.border[2],
            self.border[3],
        )
    }

    /// Get damage preview color.
    pub fn damage_preview_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.damage_preview[0],
            self.damage_preview[1],
            self.damage_preview[2],
            self.damage_preview[3],
        )
    }

    /// Get interpolated color based on health percentage.
    pub fn health_color(&self, health_percent: f32) -> Color32 {
        if health_percent <= 0.25 {
            self.critical_color()
        } else if health_percent <= 0.5 {
            // Lerp between critical and low
            let t = (health_percent - 0.25) / 0.25;
            lerp_color(self.critical_color(), self.low_color(), t)
        } else if health_percent <= 0.75 {
            // Lerp between low and full
            let t = (health_percent - 0.5) / 0.25;
            lerp_color(self.low_color(), self.full_color(), t)
        } else {
            self.full_color()
        }
    }
}

/// Lerp between two colors.
fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

/// Configuration for stamina bar colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaminaBarColors {
    /// Full stamina color (RGBA).
    pub full: [u8; 4],
    /// Depleted stamina color (RGBA).
    pub depleted: [u8; 4],
    /// Regenerating color (RGBA).
    pub regenerating: [u8; 4],
    /// Background color (RGBA).
    pub background: [u8; 4],
}

impl Default for StaminaBarColors {
    fn default() -> Self {
        Self {
            full: [100, 200, 100, 255],         // Green
            depleted: [100, 100, 100, 255],     // Gray
            regenerating: [150, 200, 100, 255], // Light green
            background: [40, 40, 40, 200],      // Dark gray
        }
    }
}

impl StaminaBarColors {
    /// Get full stamina color.
    pub fn full_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.full[0], self.full[1], self.full[2], self.full[3])
    }

    /// Get depleted color.
    pub fn depleted_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.depleted[0],
            self.depleted[1],
            self.depleted[2],
            self.depleted[3],
        )
    }

    /// Get regenerating color.
    pub fn regenerating_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.regenerating[0],
            self.regenerating[1],
            self.regenerating[2],
            self.regenerating[3],
        )
    }

    /// Get background color.
    pub fn background_color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.background[0],
            self.background[1],
            self.background[2],
            self.background[3],
        )
    }
}

/// Player health bar data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerHealthData {
    /// Health value.
    pub health: AnimatedStatValue,
    /// Stamina value.
    pub stamina: AnimatedStatValue,
    /// Shield/armor value (optional).
    pub shield: Option<AnimatedStatValue>,
    /// Player name.
    pub name: String,
    /// Player level.
    pub level: u32,
}

impl Default for PlayerHealthData {
    fn default() -> Self {
        Self {
            health: AnimatedStatValue::new(100.0, 100.0),
            stamina: AnimatedStatValue::new(100.0, 100.0),
            shield: None,
            name: "Player".to_string(),
            level: 1,
        }
    }
}

impl PlayerHealthData {
    /// Create new player health data.
    pub fn new(name: impl Into<String>, health: f32, stamina: f32) -> Self {
        Self {
            health: AnimatedStatValue::new(health, health),
            stamina: AnimatedStatValue::new(stamina, stamina),
            shield: None,
            name: name.into(),
            level: 1,
        }
    }

    /// Set health.
    pub fn set_health(&mut self, value: f32) {
        self.health.set(value);
    }

    /// Set stamina.
    pub fn set_stamina(&mut self, value: f32) {
        self.stamina.set(value);
    }

    /// Set shield value.
    pub fn set_shield(&mut self, value: f32, max: f32) {
        if let Some(shield) = &mut self.shield {
            shield.set(value);
            shield.set_max(max);
        } else {
            self.shield = Some(AnimatedStatValue::new(value, max));
        }
    }

    /// Remove shield.
    pub fn clear_shield(&mut self) {
        self.shield = None;
    }

    /// Update animations.
    pub fn update(&mut self, dt: f32) {
        self.health.update(dt);
        self.stamina.update(dt);
        if let Some(shield) = &mut self.shield {
            shield.update(dt);
        }
    }

    /// Check if player is dead.
    pub fn is_dead(&self) -> bool {
        self.health.is_empty()
    }

    /// Check if health is critical.
    pub fn is_critical(&self) -> bool {
        self.health.normalized() <= 0.25
    }

    /// Check if health is low.
    pub fn is_low(&self) -> bool {
        self.health.normalized() <= 0.5
    }
}

/// Target health bar data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetHealthData {
    /// Entity ID.
    pub id: EntityId,
    /// Target name.
    pub name: String,
    /// Target level.
    pub level: u32,
    /// Health value.
    pub health: AnimatedStatValue,
    /// Whether target is a boss.
    pub is_boss: bool,
    /// Target type for color coding.
    pub target_type: TargetType,
    /// Time since target was acquired.
    pub lock_time: f32,
}

impl TargetHealthData {
    /// Create new target health data.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        health: f32,
        max_health: f32,
    ) -> Self {
        Self {
            id: EntityId::new(id),
            name: name.into(),
            level: 1,
            health: AnimatedStatValue::new(health, max_health),
            is_boss: false,
            target_type: TargetType::Enemy,
            lock_time: 0.0,
        }
    }

    /// Set as boss.
    pub fn with_boss(mut self, is_boss: bool) -> Self {
        self.is_boss = is_boss;
        self
    }

    /// Set target type.
    pub fn with_type(mut self, target_type: TargetType) -> Self {
        self.target_type = target_type;
        self
    }

    /// Set level.
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    /// Update animations.
    pub fn update(&mut self, dt: f32) {
        self.health.update(dt);
        self.lock_time += dt;
    }

    /// Check if target is dead.
    pub fn is_dead(&self) -> bool {
        self.health.is_empty()
    }
}

/// Target type for color coding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetType {
    /// Enemy target.
    #[default]
    Enemy,
    /// Friendly target.
    Friendly,
    /// Neutral target.
    Neutral,
    /// Boss enemy.
    Boss,
}

impl TargetType {
    /// Get color for target type.
    pub fn color(&self) -> Color32 {
        match self {
            TargetType::Enemy => Color32::from_rgb(200, 80, 80),
            TargetType::Friendly => Color32::from_rgb(80, 200, 80),
            TargetType::Neutral => Color32::from_rgb(200, 200, 80),
            TargetType::Boss => Color32::from_rgb(200, 80, 200),
        }
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetType::Enemy => "Enemy",
            TargetType::Friendly => "Friendly",
            TargetType::Neutral => "Neutral",
            TargetType::Boss => "Boss",
        }
    }
}

/// Configuration for health bars UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthBarsConfig {
    /// Health bar width.
    pub health_bar_width: f32,
    /// Health bar height.
    pub health_bar_height: f32,
    /// Stamina bar height.
    pub stamina_bar_height: f32,
    /// Target bar width.
    pub target_bar_width: f32,
    /// Target bar height.
    pub target_bar_height: f32,
    /// Health bar style.
    pub style: HealthBarStyle,
    /// Number of segments (for segmented style).
    pub segments: u32,
    /// Show numeric values.
    pub show_values: bool,
    /// Show percentage.
    pub show_percentage: bool,
    /// Health bar colors.
    pub health_colors: HealthBarColors,
    /// Stamina bar colors.
    pub stamina_colors: StaminaBarColors,
    /// Damage preview delay (seconds).
    pub damage_preview_delay: f32,
    /// Corner rounding.
    pub corner_rounding: f32,
}

impl Default for HealthBarsConfig {
    fn default() -> Self {
        Self {
            health_bar_width: 250.0,
            health_bar_height: 24.0,
            stamina_bar_height: 12.0,
            target_bar_width: 300.0,
            target_bar_height: 20.0,
            style: HealthBarStyle::Standard,
            segments: 10,
            show_values: true,
            show_percentage: false,
            health_colors: HealthBarColors::default(),
            stamina_colors: StaminaBarColors::default(),
            damage_preview_delay: 0.5,
            corner_rounding: 4.0,
        }
    }
}

/// Health bars UI widget.
#[derive(Debug)]
pub struct HealthBars {
    /// Player health data.
    pub player: PlayerHealthData,
    /// Target health data (if locked on).
    pub target: Option<TargetHealthData>,
    /// Configuration.
    pub config: HealthBarsConfig,
    /// Whether the UI is visible.
    pub visible: bool,
    /// Low health pulse timer.
    pulse_timer: f32,
}

impl Default for HealthBars {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthBars {
    /// Create new health bars UI.
    pub fn new() -> Self {
        Self {
            player: PlayerHealthData::default(),
            target: None,
            config: HealthBarsConfig::default(),
            visible: true,
            pulse_timer: 0.0,
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: HealthBarsConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set player health.
    pub fn set_player_health(&mut self, current: f32, max: f32) {
        self.player.health.set_max(max);
        self.player.health.set(current);
    }

    /// Set player stamina.
    pub fn set_player_stamina(&mut self, current: f32, max: f32) {
        self.player.stamina.set_max(max);
        self.player.stamina.set(current);
    }

    /// Apply damage to player.
    pub fn damage_player(&mut self, amount: f32) {
        self.player.health.damage(amount);
    }

    /// Heal player.
    pub fn heal_player(&mut self, amount: f32) {
        self.player.health.heal(amount);
    }

    /// Set target.
    pub fn set_target(&mut self, target: TargetHealthData) {
        self.target = Some(target);
    }

    /// Clear target.
    pub fn clear_target(&mut self) {
        self.target = None;
    }

    /// Update target health.
    pub fn update_target_health(&mut self, current: f32) {
        if let Some(target) = &mut self.target {
            target.health.set(current);
        }
    }

    /// Damage target.
    pub fn damage_target(&mut self, amount: f32) {
        if let Some(target) = &mut self.target {
            target.health.damage(amount);
        }
    }

    /// Update animations.
    pub fn update(&mut self, dt: f32) {
        self.player.update(dt);
        if let Some(target) = &mut self.target {
            target.update(dt);
        }

        // Update pulse timer for low health effect
        if self.player.is_critical() {
            self.pulse_timer += dt * 4.0;
        } else {
            self.pulse_timer = 0.0;
        }
    }

    /// Render the health bars.
    pub fn show(&mut self, ui: &mut Ui) {
        if !self.visible {
            return;
        }

        // Player health bar (top-left area)
        self.show_player_bars(ui);

        // Target health bar (top-center)
        if let Some(target) = &self.target {
            self.show_target_bar(ui, target);
        }
    }

    /// Show player health and stamina bars.
    fn show_player_bars(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Player name and level
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&self.player.name)
                        .color(Color32::WHITE)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("Lv.{}", self.player.level))
                        .color(Color32::from_rgb(200, 200, 100))
                        .small(),
                );
            });

            // Health bar
            self.draw_health_bar(
                ui,
                &self.player.health,
                self.config.health_bar_width,
                self.config.health_bar_height,
                &self.config.health_colors,
                true,
            );

            // Shield bar (if present)
            if let Some(shield) = &self.player.shield {
                self.draw_shield_bar(ui, shield);
            }

            // Stamina bar
            self.draw_stamina_bar(ui, &self.player.stamina);
        });
    }

    /// Show target health bar.
    fn show_target_bar(&self, ui: &mut Ui, target: &TargetHealthData) {
        ui.vertical(|ui| {
            // Target name and level
            ui.horizontal(|ui| {
                let name_color = target.target_type.color();
                ui.label(egui::RichText::new(&target.name).color(name_color).strong());
                ui.label(
                    egui::RichText::new(format!("Lv.{}", target.level))
                        .color(Color32::from_gray(180))
                        .small(),
                );
                if target.is_boss {
                    ui.label(egui::RichText::new("👑").small());
                }
            });

            // Target health bar
            let mut colors = self.config.health_colors.clone();
            colors.full = [
                target.target_type.color().r(),
                target.target_type.color().g(),
                target.target_type.color().b(),
                255,
            ];

            self.draw_health_bar(
                ui,
                &target.health,
                self.config.target_bar_width,
                self.config.target_bar_height,
                &colors,
                false,
            );
        });
    }

    /// Draw a health bar.
    fn draw_health_bar(
        &self,
        ui: &mut Ui,
        stat: &AnimatedStatValue,
        width: f32,
        height: f32,
        colors: &HealthBarColors,
        show_pulse: bool,
    ) {
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, self.config.corner_rounding, colors.background_color());

        // Damage preview (shows recent damage as fading bar)
        if stat.display > stat.current && stat.change_time < self.config.damage_preview_delay {
            let preview_width = rect.width() * stat.display_normalized();
            let preview_rect =
                Rect::from_min_size(rect.min, Vec2::new(preview_width, rect.height()));
            painter.rect_filled(
                preview_rect,
                self.config.corner_rounding,
                colors.damage_preview_color(),
            );
        }

        // Current health
        let health_percent = stat.normalized();
        let health_width = rect.width() * health_percent;
        let health_rect = Rect::from_min_size(rect.min, Vec2::new(health_width, rect.height()));

        let mut health_color = colors.health_color(health_percent);

        // Pulse effect for critical health
        if show_pulse && self.player.is_critical() {
            let pulse = (self.pulse_timer.sin() * 0.5 + 0.5) * 0.3;
            health_color = Color32::from_rgba_unmultiplied(
                (health_color.r() as f32 * (1.0 + pulse)).min(255.0) as u8,
                health_color.g(),
                health_color.b(),
                health_color.a(),
            );
        }

        painter.rect_filled(health_rect, self.config.corner_rounding, health_color);

        // Segmented style overlay
        if self.config.style == HealthBarStyle::Segmented && self.config.segments > 1 {
            let segment_width = rect.width() / self.config.segments as f32;
            for i in 1..self.config.segments {
                let x = rect.min.x + segment_width * i as f32;
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    egui::Stroke::new(1.0, Color32::from_gray(60)),
                );
            }
        }

        // Border
        painter.rect_stroke(
            rect,
            self.config.corner_rounding,
            egui::Stroke::new(1.0, colors.border_color()),
        );

        // Text overlay
        if self.config.show_values || self.config.show_percentage {
            let text = if self.config.show_values && self.config.show_percentage {
                format!(
                    "{:.0}/{:.0} ({:.0}%)",
                    stat.current,
                    stat.max,
                    health_percent * 100.0
                )
            } else if self.config.show_values {
                format!("{:.0}/{:.0}", stat.current, stat.max)
            } else {
                format!("{:.0}%", health_percent * 100.0)
            };

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(height * 0.6),
                Color32::WHITE,
            );
        }
    }

    /// Draw shield bar.
    fn draw_shield_bar(&self, ui: &mut Ui, shield: &AnimatedStatValue) {
        let height = self.config.stamina_bar_height * 0.8;
        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(self.config.health_bar_width, height),
            egui::Sense::hover(),
        );

        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, self.config.corner_rounding, Color32::from_gray(30));

        // Shield fill
        let shield_width = rect.width() * shield.normalized();
        let shield_rect = Rect::from_min_size(rect.min, Vec2::new(shield_width, rect.height()));
        painter.rect_filled(
            shield_rect,
            self.config.corner_rounding,
            Color32::from_rgb(100, 150, 255),
        );

        // Border
        painter.rect_stroke(
            rect,
            self.config.corner_rounding,
            egui::Stroke::new(1.0, Color32::from_gray(80)),
        );
    }

    /// Draw stamina bar.
    fn draw_stamina_bar(&self, ui: &mut Ui, stamina: &AnimatedStatValue) {
        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(self.config.health_bar_width, self.config.stamina_bar_height),
            egui::Sense::hover(),
        );

        let painter = ui.painter();
        let colors = &self.config.stamina_colors;

        // Background
        painter.rect_filled(rect, self.config.corner_rounding, colors.background_color());

        // Stamina fill
        let stamina_percent = stamina.normalized();
        let stamina_width = rect.width() * stamina_percent;
        let stamina_rect = Rect::from_min_size(rect.min, Vec2::new(stamina_width, rect.height()));

        let stamina_color = if stamina_percent < 0.25 {
            colors.depleted_color()
        } else if stamina.display > stamina.current {
            colors.regenerating_color()
        } else {
            colors.full_color()
        };

        painter.rect_filled(stamina_rect, self.config.corner_rounding, stamina_color);

        // Border
        painter.rect_stroke(
            rect,
            self.config.corner_rounding,
            egui::Stroke::new(1.0, colors.background_color()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id() {
        let id = EntityId::new("player_1");
        assert_eq!(id.0, "player_1");
        assert_eq!(format!("{id}"), "player_1");
    }

    #[test]
    fn test_animated_stat_value_new() {
        let stat = AnimatedStatValue::new(50.0, 100.0);
        assert_eq!(stat.current, 50.0);
        assert_eq!(stat.max, 100.0);
        assert_eq!(stat.display, 50.0);
    }

    #[test]
    fn test_animated_stat_value_set() {
        let mut stat = AnimatedStatValue::new(100.0, 100.0);
        stat.set(75.0);
        assert_eq!(stat.current, 75.0);
        assert_eq!(stat.previous, 100.0);
    }

    #[test]
    fn test_animated_stat_value_damage() {
        let mut stat = AnimatedStatValue::new(100.0, 100.0);
        stat.damage(30.0);
        assert_eq!(stat.current, 70.0);
    }

    #[test]
    fn test_animated_stat_value_heal() {
        let mut stat = AnimatedStatValue::new(50.0, 100.0);
        stat.heal(30.0);
        assert_eq!(stat.current, 80.0);
    }

    #[test]
    fn test_animated_stat_value_clamp() {
        let mut stat = AnimatedStatValue::new(100.0, 100.0);
        stat.damage(150.0);
        assert_eq!(stat.current, 0.0);

        stat.heal(200.0);
        assert_eq!(stat.current, 100.0);
    }

    #[test]
    fn test_animated_stat_value_normalized() {
        let stat = AnimatedStatValue::new(50.0, 100.0);
        assert_eq!(stat.normalized(), 0.5);
    }

    #[test]
    fn test_animated_stat_value_is_full() {
        let stat = AnimatedStatValue::new(100.0, 100.0);
        assert!(stat.is_full());

        let stat2 = AnimatedStatValue::new(99.0, 100.0);
        assert!(!stat2.is_full());
    }

    #[test]
    fn test_animated_stat_value_is_empty() {
        let stat = AnimatedStatValue::new(0.0, 100.0);
        assert!(stat.is_empty());

        let stat2 = AnimatedStatValue::new(1.0, 100.0);
        assert!(!stat2.is_empty());
    }

    #[test]
    fn test_animated_stat_value_update() {
        let mut stat = AnimatedStatValue::new(100.0, 100.0);
        stat.set(50.0);
        assert_eq!(stat.display, 100.0);

        stat.update(1.0);
        assert!(stat.display < 100.0);
    }

    #[test]
    fn test_animated_stat_value_recent_damage() {
        let mut stat = AnimatedStatValue::new(100.0, 100.0);
        stat.set(70.0);
        assert_eq!(stat.recent_damage(), 30.0);
    }

    #[test]
    fn test_health_bar_style() {
        assert_eq!(HealthBarStyle::all().len(), 4);
        assert_eq!(HealthBarStyle::Standard.display_name(), "Standard");
    }

    #[test]
    fn test_health_bar_colors_default() {
        let colors = HealthBarColors::default();
        assert_eq!(colors.full, [100, 200, 100, 255]);
    }

    #[test]
    fn test_health_bar_colors_health_color() {
        let colors = HealthBarColors::default();
        // Full health
        let full_color = colors.health_color(1.0);
        assert_eq!(full_color, colors.full_color());
        // Critical health
        let critical_color = colors.health_color(0.1);
        assert_eq!(critical_color, colors.critical_color());
    }

    #[test]
    fn test_stamina_bar_colors_default() {
        let colors = StaminaBarColors::default();
        assert_eq!(colors.full, [100, 200, 100, 255]);
    }

    #[test]
    fn test_player_health_data_default() {
        let data = PlayerHealthData::default();
        assert_eq!(data.name, "Player");
        assert_eq!(data.level, 1);
    }

    #[test]
    fn test_player_health_data_new() {
        let data = PlayerHealthData::new("Hero", 150.0, 100.0);
        assert_eq!(data.name, "Hero");
        assert_eq!(data.health.max, 150.0);
    }

    #[test]
    fn test_player_health_data_is_dead() {
        let mut data = PlayerHealthData::default();
        assert!(!data.is_dead());
        data.set_health(0.0);
        assert!(data.is_dead());
    }

    #[test]
    fn test_player_health_data_is_critical() {
        let mut data = PlayerHealthData::default();
        assert!(!data.is_critical());
        data.set_health(20.0);
        assert!(data.is_critical());
    }

    #[test]
    fn test_player_health_data_is_low() {
        let mut data = PlayerHealthData::default();
        assert!(!data.is_low());
        data.set_health(40.0);
        assert!(data.is_low());
    }

    #[test]
    fn test_player_health_data_shield() {
        let mut data = PlayerHealthData::default();
        assert!(data.shield.is_none());
        data.set_shield(50.0, 100.0);
        assert!(data.shield.is_some());
        data.clear_shield();
        assert!(data.shield.is_none());
    }

    #[test]
    fn test_target_health_data_new() {
        let target = TargetHealthData::new("enemy_1", "Goblin", 50.0, 50.0);
        assert_eq!(target.name, "Goblin");
        assert_eq!(target.health.max, 50.0);
    }

    #[test]
    fn test_target_health_data_builders() {
        let target = TargetHealthData::new("boss_1", "Dragon", 1000.0, 1000.0)
            .with_boss(true)
            .with_type(TargetType::Boss)
            .with_level(50);

        assert!(target.is_boss);
        assert_eq!(target.target_type, TargetType::Boss);
        assert_eq!(target.level, 50);
    }

    #[test]
    fn test_target_type_color() {
        assert_ne!(TargetType::Enemy.color(), TargetType::Friendly.color());
        assert_ne!(TargetType::Neutral.color(), TargetType::Boss.color());
    }

    #[test]
    fn test_target_type_display_name() {
        assert_eq!(TargetType::Enemy.display_name(), "Enemy");
        assert_eq!(TargetType::Boss.display_name(), "Boss");
    }

    #[test]
    fn test_health_bars_config_default() {
        let config = HealthBarsConfig::default();
        assert_eq!(config.health_bar_width, 250.0);
        assert_eq!(config.segments, 10);
    }

    #[test]
    fn test_health_bars_new() {
        let bars = HealthBars::new();
        assert!(bars.visible);
        assert!(bars.target.is_none());
    }

    #[test]
    fn test_health_bars_set_player_health() {
        let mut bars = HealthBars::new();
        bars.set_player_health(75.0, 100.0);
        assert_eq!(bars.player.health.current, 75.0);
        assert_eq!(bars.player.health.max, 100.0);
    }

    #[test]
    fn test_health_bars_damage_player() {
        let mut bars = HealthBars::new();
        bars.damage_player(30.0);
        assert_eq!(bars.player.health.current, 70.0);
    }

    #[test]
    fn test_health_bars_heal_player() {
        let mut bars = HealthBars::new();
        bars.damage_player(50.0);
        bars.heal_player(20.0);
        assert_eq!(bars.player.health.current, 70.0);
    }

    #[test]
    fn test_health_bars_target() {
        let mut bars = HealthBars::new();
        assert!(bars.target.is_none());

        let target = TargetHealthData::new("enemy", "Enemy", 100.0, 100.0);
        bars.set_target(target);
        assert!(bars.target.is_some());

        bars.damage_target(25.0);
        assert_eq!(bars.target.as_ref().unwrap().health.current, 75.0);

        bars.clear_target();
        assert!(bars.target.is_none());
    }

    #[test]
    fn test_health_bars_update() {
        let mut bars = HealthBars::new();
        bars.damage_player(30.0);
        bars.update(0.1);
        // Animation should progress
        assert!(bars.player.health.display > bars.player.health.current);
    }

    #[test]
    fn test_lerp_color() {
        let a = Color32::BLACK;
        let b = Color32::WHITE;
        let mid = lerp_color(a, b, 0.5);
        assert_eq!(mid.r(), 127);
        assert_eq!(mid.g(), 127);
        assert_eq!(mid.b(), 127);
    }

    #[test]
    fn test_animated_stat_serialization() {
        let stat = AnimatedStatValue::new(75.0, 100.0);
        let json = serde_json::to_string(&stat).unwrap();
        let loaded: AnimatedStatValue = serde_json::from_str(&json).unwrap();
        assert_eq!(stat.current, loaded.current);
        assert_eq!(stat.max, loaded.max);
    }

    #[test]
    fn test_health_bar_colors_serialization() {
        let colors = HealthBarColors::default();
        let json = serde_json::to_string(&colors).unwrap();
        let loaded: HealthBarColors = serde_json::from_str(&json).unwrap();
        assert_eq!(colors.full, loaded.full);
    }

    #[test]
    fn test_health_bars_config_serialization() {
        let config = HealthBarsConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: HealthBarsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.health_bar_width, loaded.health_bar_width);
    }

    #[test]
    fn test_target_type_serialization() {
        for target_type in &[
            TargetType::Enemy,
            TargetType::Friendly,
            TargetType::Neutral,
            TargetType::Boss,
        ] {
            let json = serde_json::to_string(target_type).unwrap();
            let loaded: TargetType = serde_json::from_str(&json).unwrap();
            assert_eq!(*target_type, loaded);
        }
    }

    #[test]
    fn test_health_bar_style_serialization() {
        for style in HealthBarStyle::all() {
            let json = serde_json::to_string(style).unwrap();
            let loaded: HealthBarStyle = serde_json::from_str(&json).unwrap();
            assert_eq!(*style, loaded);
        }
    }
}
