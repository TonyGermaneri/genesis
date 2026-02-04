//! Combat HUD system for real-time combat feedback.
//!
//! Provides health bars, damage numbers, crosshairs, hit indicators,
//! and cooldown displays for action-oriented gameplay.

use egui::{Color32, Pos2, Rect, Vec2};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// Core Types
// ============================================================================

/// Unique identifier for entities in combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CombatEntityId(pub u64);

impl CombatEntityId {
    /// Create a new combat entity ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Unique identifier for damage numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DamageNumberId(pub u64);

impl DamageNumberId {
    /// Create a new damage number ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Unique identifier for hit indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HitIndicatorId(pub u64);

impl HitIndicatorId {
    /// Create a new hit indicator ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Unique identifier for abilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityId(pub u64);

impl AbilityId {
    /// Create a new ability ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

// ============================================================================
// Crosshair System
// ============================================================================

/// Style variants for the crosshair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairStyle {
    /// Simple dot crosshair.
    #[default]
    Dot,
    /// Traditional cross crosshair.
    Cross,
    /// Circular crosshair.
    Circle,
    /// Combined circle and cross.
    CircleCross,
    /// Chevron/arrow pointing up.
    Chevron,
    /// No crosshair visible.
    None,
}

impl CrosshairStyle {
    /// Get all available crosshair styles.
    pub fn all() -> &'static [CrosshairStyle] {
        &[
            CrosshairStyle::Dot,
            CrosshairStyle::Cross,
            CrosshairStyle::Circle,
            CrosshairStyle::CircleCross,
            CrosshairStyle::Chevron,
            CrosshairStyle::None,
        ]
    }

    /// Get the display name for the style.
    pub fn display_name(&self) -> &'static str {
        match self {
            CrosshairStyle::Dot => "Dot",
            CrosshairStyle::Cross => "Cross",
            CrosshairStyle::Circle => "Circle",
            CrosshairStyle::CircleCross => "Circle + Cross",
            CrosshairStyle::Chevron => "Chevron",
            CrosshairStyle::None => "None",
        }
    }
}

/// Configuration for crosshair appearance.
#[derive(Debug, Clone)]
pub struct CrosshairConfig {
    /// The style of crosshair to display.
    pub style: CrosshairStyle,
    /// Base color of the crosshair.
    pub color: Color32,
    /// Color when targeting an enemy.
    pub enemy_color: Color32,
    /// Color when targeting a friendly.
    pub friendly_color: Color32,
    /// Size of the crosshair in pixels.
    pub size: f32,
    /// Thickness of crosshair lines.
    pub thickness: f32,
    /// Gap in the center for cross style.
    pub center_gap: f32,
    /// Whether to show hit marker effect.
    pub show_hit_marker: bool,
    /// Duration of hit marker animation.
    pub hit_marker_duration: Duration,
    /// Whether crosshair expands when shooting.
    pub dynamic_spread: bool,
}

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            style: CrosshairStyle::default(),
            color: Color32::WHITE,
            enemy_color: Color32::from_rgb(255, 100, 100),
            friendly_color: Color32::from_rgb(100, 255, 100),
            size: 16.0,
            thickness: 2.0,
            center_gap: 4.0,
            show_hit_marker: true,
            hit_marker_duration: Duration::from_millis(150),
            dynamic_spread: true,
        }
    }
}

/// State for crosshair animations and targeting.
#[derive(Debug, Clone)]
pub struct CrosshairState {
    /// Current spread multiplier (1.0 = normal).
    pub spread: f32,
    /// Target spread to animate towards.
    pub target_spread: f32,
    /// Time remaining on hit marker animation.
    pub hit_marker_time: f32,
    /// Current target type.
    pub target_type: TargetType,
    /// Whether the crosshair is currently visible.
    pub visible: bool,
}

impl Default for CrosshairState {
    fn default() -> Self {
        Self {
            spread: 1.0,
            target_spread: 1.0,
            hit_marker_time: 0.0,
            target_type: TargetType::None,
            visible: true,
        }
    }
}

impl CrosshairState {
    /// Create a new crosshair state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Trigger the hit marker animation.
    pub fn trigger_hit_marker(&mut self, duration: Duration) {
        self.hit_marker_time = duration.as_secs_f32();
    }

    /// Set spread for shooting/movement.
    pub fn set_spread(&mut self, spread: f32) {
        self.target_spread = spread.max(1.0);
    }

    /// Update crosshair state over time.
    pub fn update(&mut self, delta_time: f32) {
        // Animate hit marker
        if self.hit_marker_time > 0.0 {
            self.hit_marker_time = (self.hit_marker_time - delta_time).max(0.0);
        }

        // Animate spread
        let spread_speed = 8.0;
        self.spread += (self.target_spread - self.spread) * spread_speed * delta_time;
    }

    /// Check if hit marker is currently active.
    pub fn is_hit_marker_active(&self) -> bool {
        self.hit_marker_time > 0.0
    }

    /// Get current color based on target type.
    pub fn get_color(&self, config: &CrosshairConfig) -> Color32 {
        match self.target_type {
            TargetType::None | TargetType::Neutral => config.color,
            TargetType::Enemy => config.enemy_color,
            TargetType::Friendly => config.friendly_color,
            TargetType::Interactable => Color32::from_rgb(255, 255, 100),
        }
    }
}

/// Types of targets the crosshair can be over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetType {
    /// Not targeting anything.
    #[default]
    None,
    /// Targeting an enemy.
    Enemy,
    /// Targeting a friendly.
    Friendly,
    /// Targeting a neutral entity.
    Neutral,
    /// Targeting an interactable object.
    Interactable,
}

// ============================================================================
// Health Bar System
// ============================================================================

/// Style variants for health bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HealthBarStyle {
    /// Simple filled rectangle.
    #[default]
    Simple,
    /// Segmented health chunks.
    Segmented,
    /// Gradient fill based on health.
    Gradient,
    /// Bordered with inner fill.
    Bordered,
}

/// Configuration for entity health bars.
#[derive(Debug, Clone)]
pub struct HealthBarConfig {
    /// Width of the health bar.
    pub width: f32,
    /// Height of the health bar.
    pub height: f32,
    /// Style of the health bar.
    pub style: HealthBarStyle,
    /// Full health color.
    pub full_color: Color32,
    /// Medium health color (50%).
    pub medium_color: Color32,
    /// Low health color (<25%).
    pub low_color: Color32,
    /// Background color.
    pub background_color: Color32,
    /// Border color.
    pub border_color: Color32,
    /// Border thickness.
    pub border_thickness: f32,
    /// Vertical offset from entity position.
    pub y_offset: f32,
    /// Whether to show when full health.
    pub hide_when_full: bool,
    /// Whether to show the entity name.
    pub show_name: bool,
    /// Font size for name.
    pub name_font_size: f32,
    /// Number of segments for segmented style.
    pub segment_count: u32,
    /// Whether to animate damage taken.
    pub animate_damage: bool,
    /// Duration of damage animation.
    pub damage_animation_duration: Duration,
}

impl Default for HealthBarConfig {
    fn default() -> Self {
        Self {
            width: 60.0,
            height: 8.0,
            style: HealthBarStyle::default(),
            full_color: Color32::from_rgb(100, 255, 100),
            medium_color: Color32::from_rgb(255, 255, 100),
            low_color: Color32::from_rgb(255, 100, 100),
            background_color: Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            border_color: Color32::from_rgb(80, 80, 80),
            border_thickness: 1.0,
            y_offset: -30.0,
            hide_when_full: true,
            show_name: false,
            name_font_size: 12.0,
            segment_count: 10,
            animate_damage: true,
            damage_animation_duration: Duration::from_millis(500),
        }
    }
}

/// State for an individual entity's health bar.
#[derive(Debug, Clone)]
pub struct HealthBarState {
    /// Entity this health bar belongs to.
    pub entity_id: CombatEntityId,
    /// Current health value.
    pub current_health: f32,
    /// Maximum health value.
    pub max_health: f32,
    /// Optional shield/armor value.
    pub shield: f32,
    /// Maximum shield value.
    pub max_shield: f32,
    /// Display position in screen space.
    pub screen_position: Pos2,
    /// Entity name to display.
    pub name: Option<String>,
    /// Whether this is a boss health bar.
    pub is_boss: bool,
    /// Animated display health (for damage animation).
    pub display_health: f32,
    /// Time remaining on damage animation.
    pub damage_animation_time: f32,
    /// Whether the health bar is currently visible.
    pub visible: bool,
}

impl HealthBarState {
    /// Create a new health bar state.
    pub fn new(entity_id: CombatEntityId, max_health: f32) -> Self {
        Self {
            entity_id,
            current_health: max_health,
            max_health,
            shield: 0.0,
            max_shield: 0.0,
            screen_position: Pos2::ZERO,
            name: None,
            is_boss: false,
            display_health: max_health,
            damage_animation_time: 0.0,
            visible: true,
        }
    }

    /// Create a boss health bar.
    pub fn new_boss(entity_id: CombatEntityId, max_health: f32, name: &str) -> Self {
        Self {
            entity_id,
            current_health: max_health,
            max_health,
            shield: 0.0,
            max_shield: 0.0,
            screen_position: Pos2::ZERO,
            name: Some(name.to_string()),
            is_boss: true,
            display_health: max_health,
            damage_animation_time: 0.0,
            visible: true,
        }
    }

    /// Set health value.
    pub fn set_health(&mut self, health: f32, animate: bool, duration: Duration) {
        if animate && health < self.current_health {
            self.damage_animation_time = duration.as_secs_f32();
        }
        self.current_health = health.clamp(0.0, self.max_health);
    }

    /// Set shield value.
    pub fn set_shield(&mut self, shield: f32) {
        self.shield = shield.clamp(0.0, self.max_shield);
    }

    /// Get health percentage (0.0 to 1.0).
    pub fn health_percent(&self) -> f32 {
        if self.max_health <= 0.0 {
            return 0.0;
        }
        self.current_health / self.max_health
    }

    /// Get shield percentage (0.0 to 1.0).
    pub fn shield_percent(&self) -> f32 {
        if self.max_shield <= 0.0 {
            return 0.0;
        }
        self.shield / self.max_shield
    }

    /// Check if entity is dead.
    pub fn is_dead(&self) -> bool {
        self.current_health <= 0.0
    }

    /// Check if health is low (<25%).
    pub fn is_low_health(&self) -> bool {
        self.health_percent() < 0.25
    }

    /// Check if health is medium (25-50%).
    pub fn is_medium_health(&self) -> bool {
        let percent = self.health_percent();
        (0.25..0.5).contains(&percent)
    }

    /// Get the appropriate color based on health level.
    pub fn get_health_color(&self, config: &HealthBarConfig) -> Color32 {
        let percent = self.health_percent();
        if percent < 0.25 {
            config.low_color
        } else if percent < 0.5 {
            config.medium_color
        } else {
            config.full_color
        }
    }

    /// Update health bar state over time.
    pub fn update(&mut self, delta_time: f32) {
        // Animate display health catching up to current health
        if self.damage_animation_time > 0.0 {
            self.damage_animation_time = (self.damage_animation_time - delta_time).max(0.0);
        } else {
            // Smoothly animate display health
            let speed = 2.0;
            self.display_health += (self.current_health - self.display_health) * speed * delta_time;
        }
    }
}

// ============================================================================
// Damage Numbers System
// ============================================================================

/// Type of damage for different visual styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DamageType {
    /// Normal physical damage.
    #[default]
    Physical,
    /// Critical hit damage.
    Critical,
    /// Fire/heat damage.
    Fire,
    /// Ice/cold damage.
    Ice,
    /// Electric/lightning damage.
    Electric,
    /// Poison/toxic damage.
    Poison,
    /// Healing (shows green).
    Healing,
    /// Shield damage/absorption.
    Shield,
    /// Experience points gained.
    Experience,
}

impl DamageType {
    /// Get the default color for this damage type.
    pub fn default_color(&self) -> Color32 {
        match self {
            DamageType::Physical => Color32::WHITE,
            DamageType::Critical => Color32::from_rgb(255, 200, 50),
            DamageType::Fire => Color32::from_rgb(255, 100, 50),
            DamageType::Ice => Color32::from_rgb(100, 200, 255),
            DamageType::Electric => Color32::from_rgb(200, 200, 255),
            DamageType::Poison => Color32::from_rgb(100, 255, 100),
            DamageType::Healing => Color32::from_rgb(50, 255, 100),
            DamageType::Shield => Color32::from_rgb(100, 150, 255),
            DamageType::Experience => Color32::from_rgb(255, 215, 0),
        }
    }

    /// Get the icon/prefix for this damage type.
    pub fn prefix(&self) -> &'static str {
        match self {
            DamageType::Physical => "",
            DamageType::Critical => "💥",
            DamageType::Fire => "🔥",
            DamageType::Ice => "❄️",
            DamageType::Electric => "⚡",
            DamageType::Poison => "☠️",
            DamageType::Healing | DamageType::Experience => "+",
            DamageType::Shield => "🛡️",
        }
    }

    /// Get the suffix for this damage type.
    pub fn suffix(&self) -> &'static str {
        match self {
            DamageType::Experience => " XP",
            _ => "",
        }
    }
}

/// Configuration for damage number display.
#[derive(Debug, Clone)]
pub struct DamageNumberConfig {
    /// Base font size.
    pub font_size: f32,
    /// Font size multiplier for critical hits.
    pub critical_scale: f32,
    /// Duration damage numbers are visible.
    pub duration: Duration,
    /// How fast numbers rise.
    pub rise_speed: f32,
    /// Horizontal spread for multiple numbers.
    pub spread: f32,
    /// Whether to combine rapid damage into one number.
    pub combine_rapid: bool,
    /// Time window for combining damage.
    pub combine_window: Duration,
    /// Whether to show damage type icons.
    pub show_icons: bool,
    /// Whether numbers fade out.
    pub fade_out: bool,
    /// Whether numbers scale down as they fade.
    pub scale_down: bool,
    /// Outline color for text.
    pub outline_color: Color32,
    /// Outline thickness.
    pub outline_thickness: f32,
}

impl Default for DamageNumberConfig {
    fn default() -> Self {
        Self {
            font_size: 18.0,
            critical_scale: 1.5,
            duration: Duration::from_millis(1500),
            rise_speed: 40.0,
            spread: 20.0,
            combine_rapid: true,
            combine_window: Duration::from_millis(100),
            show_icons: true,
            fade_out: true,
            scale_down: true,
            outline_color: Color32::BLACK,
            outline_thickness: 1.0,
        }
    }
}

/// An individual floating damage number.
#[derive(Debug, Clone)]
pub struct DamageNumber {
    /// Unique ID for this damage number.
    pub id: DamageNumberId,
    /// The damage value to display.
    pub value: i32,
    /// Type of damage.
    pub damage_type: DamageType,
    /// Current position.
    pub position: Pos2,
    /// Initial spawn position.
    pub spawn_position: Pos2,
    /// Horizontal velocity (for spread).
    pub velocity_x: f32,
    /// Time alive.
    pub elapsed: f32,
    /// Total lifetime.
    pub lifetime: f32,
    /// Custom color override.
    pub color_override: Option<Color32>,
    /// Custom text override.
    pub text_override: Option<String>,
}

impl DamageNumber {
    /// Create a new damage number.
    pub fn new(
        id: DamageNumberId,
        value: i32,
        position: Pos2,
        config: &DamageNumberConfig,
    ) -> Self {
        Self {
            id,
            value,
            damage_type: DamageType::Physical,
            position,
            spawn_position: position,
            velocity_x: 0.0,
            elapsed: 0.0,
            lifetime: config.duration.as_secs_f32(),
            color_override: None,
            text_override: None,
        }
    }

    /// Create a critical damage number.
    pub fn critical(
        id: DamageNumberId,
        value: i32,
        position: Pos2,
        config: &DamageNumberConfig,
    ) -> Self {
        let mut number = Self::new(id, value, position, config);
        number.damage_type = DamageType::Critical;
        number
    }

    /// Create a healing number.
    pub fn healing(
        id: DamageNumberId,
        value: i32,
        position: Pos2,
        config: &DamageNumberConfig,
    ) -> Self {
        let mut number = Self::new(id, value, position, config);
        number.damage_type = DamageType::Healing;
        number
    }

    /// Set damage type.
    pub fn with_damage_type(mut self, damage_type: DamageType) -> Self {
        self.damage_type = damage_type;
        self
    }

    /// Set horizontal velocity.
    pub fn with_velocity(mut self, velocity_x: f32) -> Self {
        self.velocity_x = velocity_x;
        self
    }

    /// Set custom color.
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color_override = Some(color);
        self
    }

    /// Set custom text.
    pub fn with_text(mut self, text: String) -> Self {
        self.text_override = Some(text);
        self
    }

    /// Update the damage number over time.
    pub fn update(&mut self, delta_time: f32, config: &DamageNumberConfig) {
        self.elapsed += delta_time;
        self.position.x += self.velocity_x * delta_time;
        self.position.y -= config.rise_speed * delta_time;
    }

    /// Get the remaining lifetime ratio (1.0 = fresh, 0.0 = expired).
    pub fn life_ratio(&self) -> f32 {
        1.0 - (self.elapsed / self.lifetime).clamp(0.0, 1.0)
    }

    /// Check if the damage number has expired.
    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.lifetime
    }

    /// Get the current alpha based on lifetime.
    pub fn get_alpha(&self, fade_out: bool) -> u8 {
        if fade_out {
            (self.life_ratio() * 255.0) as u8
        } else {
            255
        }
    }

    /// Get the current scale based on lifetime.
    pub fn get_scale(&self, config: &DamageNumberConfig) -> f32 {
        let base_scale = if self.damage_type == DamageType::Critical {
            config.critical_scale
        } else {
            1.0
        };

        if config.scale_down {
            base_scale * (0.5 + 0.5 * self.life_ratio())
        } else {
            base_scale
        }
    }

    /// Get the color for rendering.
    pub fn get_color(&self, config: &DamageNumberConfig) -> Color32 {
        let base_color = self
            .color_override
            .unwrap_or_else(|| self.damage_type.default_color());
        let alpha = self.get_alpha(config.fade_out);
        Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), alpha)
    }

    /// Get the display text.
    pub fn get_text(&self) -> String {
        if let Some(ref text) = self.text_override {
            text.clone()
        } else {
            format!(
                "{}{}{}",
                self.damage_type.prefix(),
                self.value.abs(),
                self.damage_type.suffix()
            )
        }
    }
}

// ============================================================================
// Hit Indicators
// ============================================================================

/// Direction of incoming damage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitDirection {
    /// Damage from the front.
    Front,
    /// Damage from the back.
    Back,
    /// Damage from the left.
    Left,
    /// Damage from the right.
    Right,
    /// Damage from above.
    Above,
    /// Damage from below.
    Below,
    /// Unknown direction.
    Unknown,
}

impl HitDirection {
    /// Get the angle in radians for this direction (0 = right, counter-clockwise).
    pub fn angle(&self) -> f32 {
        match self {
            HitDirection::Front | HitDirection::Above => std::f32::consts::PI * 1.5, // Up
            HitDirection::Back | HitDirection::Below => std::f32::consts::PI * 0.5,  // Down
            HitDirection::Left => std::f32::consts::PI,                              // Left
            HitDirection::Right | HitDirection::Unknown => 0.0,                      // Right
        }
    }

    /// Calculate direction from an angle (in radians, 0 = right).
    pub fn from_angle(angle: f32) -> Self {
        let normalized = angle.rem_euclid(std::f32::consts::TAU);
        let eighth = std::f32::consts::TAU / 8.0;

        if normalized < eighth || normalized >= 7.0 * eighth {
            HitDirection::Right
        } else if normalized < 3.0 * eighth {
            HitDirection::Front
        } else if normalized < 5.0 * eighth {
            HitDirection::Left
        } else {
            HitDirection::Back
        }
    }
}

/// Configuration for hit indicators.
#[derive(Debug, Clone)]
pub struct HitIndicatorConfig {
    /// Size of the indicator.
    pub size: f32,
    /// Distance from screen center.
    pub distance: f32,
    /// Duration of the indicator.
    pub duration: Duration,
    /// Color of the indicator.
    pub color: Color32,
    /// Opacity at full intensity.
    pub max_opacity: u8,
    /// Whether indicators fade out.
    pub fade_out: bool,
    /// Whether to show direction.
    pub directional: bool,
}

impl Default for HitIndicatorConfig {
    fn default() -> Self {
        Self {
            size: 40.0,
            distance: 100.0,
            duration: Duration::from_millis(500),
            color: Color32::from_rgb(255, 50, 50),
            max_opacity: 200,
            fade_out: true,
            directional: true,
        }
    }
}

/// An individual hit indicator.
#[derive(Debug, Clone)]
pub struct HitIndicator {
    /// Unique ID.
    pub id: HitIndicatorId,
    /// Direction of the hit.
    pub direction: HitDirection,
    /// Intensity (affects opacity).
    pub intensity: f32,
    /// Time elapsed.
    pub elapsed: f32,
    /// Total lifetime.
    pub lifetime: f32,
}

impl HitIndicator {
    /// Create a new hit indicator.
    pub fn new(id: HitIndicatorId, direction: HitDirection, config: &HitIndicatorConfig) -> Self {
        Self {
            id,
            direction,
            intensity: 1.0,
            elapsed: 0.0,
            lifetime: config.duration.as_secs_f32(),
        }
    }

    /// Set intensity based on damage amount.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Update the indicator over time.
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
    }

    /// Check if expired.
    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.lifetime
    }

    /// Get current opacity.
    pub fn get_opacity(&self, config: &HitIndicatorConfig) -> u8 {
        let life_ratio = 1.0 - (self.elapsed / self.lifetime).clamp(0.0, 1.0);
        let opacity = if config.fade_out {
            life_ratio * self.intensity
        } else {
            self.intensity
        };
        (opacity * config.max_opacity as f32) as u8
    }

    /// Get screen position for the indicator.
    pub fn get_position(&self, screen_center: Pos2, config: &HitIndicatorConfig) -> Pos2 {
        let angle = self.direction.angle();
        Pos2::new(
            screen_center.x + angle.cos() * config.distance,
            screen_center.y - angle.sin() * config.distance,
        )
    }
}

// ============================================================================
// Cooldown Display
// ============================================================================

/// State for an ability cooldown.
#[derive(Debug, Clone)]
pub struct AbilityCooldown {
    /// Ability ID.
    pub ability_id: AbilityId,
    /// Display name.
    pub name: String,
    /// Icon identifier (for rendering).
    pub icon: String,
    /// Total cooldown duration.
    pub total_duration: f32,
    /// Remaining cooldown time.
    pub remaining: f32,
    /// Number of charges available.
    pub charges: u32,
    /// Maximum charges.
    pub max_charges: u32,
    /// Hotkey binding display.
    pub hotkey: Option<String>,
    /// Whether the ability is ready.
    pub is_ready: bool,
}

impl AbilityCooldown {
    /// Create a new ability cooldown.
    pub fn new(ability_id: AbilityId, name: &str) -> Self {
        Self {
            ability_id,
            name: name.to_string(),
            icon: String::new(),
            total_duration: 0.0,
            remaining: 0.0,
            charges: 1,
            max_charges: 1,
            hotkey: None,
            is_ready: true,
        }
    }

    /// Set cooldown duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.total_duration = duration.as_secs_f32();
        self
    }

    /// Set icon.
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    /// Set hotkey display.
    pub fn with_hotkey(mut self, hotkey: &str) -> Self {
        self.hotkey = Some(hotkey.to_string());
        self
    }

    /// Set charges.
    pub fn with_charges(mut self, charges: u32, max_charges: u32) -> Self {
        self.charges = charges;
        self.max_charges = max_charges;
        self
    }

    /// Start cooldown.
    pub fn start_cooldown(&mut self) {
        self.remaining = self.total_duration;
        self.is_ready = false;
        if self.charges > 0 {
            self.charges -= 1;
        }
    }

    /// Update cooldown over time.
    pub fn update(&mut self, delta_time: f32) {
        if self.remaining > 0.0 {
            self.remaining = (self.remaining - delta_time).max(0.0);
            if self.remaining == 0.0 && self.charges < self.max_charges {
                self.charges += 1;
                if self.charges < self.max_charges {
                    // Start recharging next charge
                    self.remaining = self.total_duration;
                } else {
                    self.is_ready = true;
                }
            }
        }
    }

    /// Get cooldown progress (0.0 = on cooldown, 1.0 = ready).
    pub fn progress(&self) -> f32 {
        if self.total_duration <= 0.0 {
            return 1.0;
        }
        1.0 - (self.remaining / self.total_duration)
    }

    /// Check if ability can be used.
    pub fn can_use(&self) -> bool {
        self.charges > 0
    }
}

/// Configuration for cooldown display.
#[derive(Debug, Clone)]
pub struct CooldownConfig {
    /// Size of ability icons.
    pub icon_size: f32,
    /// Spacing between icons.
    pub spacing: f32,
    /// Position offset from screen bottom.
    pub y_offset: f32,
    /// Background color.
    pub background_color: Color32,
    /// Ready color.
    pub ready_color: Color32,
    /// Cooldown color.
    pub cooldown_color: Color32,
    /// Border color.
    pub border_color: Color32,
    /// Whether to show hotkey labels.
    pub show_hotkeys: bool,
    /// Whether to show cooldown numbers.
    pub show_numbers: bool,
    /// Whether to show charge counts.
    pub show_charges: bool,
}

impl Default for CooldownConfig {
    fn default() -> Self {
        Self {
            icon_size: 48.0,
            spacing: 8.0,
            y_offset: 80.0,
            background_color: Color32::from_rgba_unmultiplied(0, 0, 0, 200),
            ready_color: Color32::from_rgba_unmultiplied(255, 255, 255, 255),
            cooldown_color: Color32::from_rgba_unmultiplied(100, 100, 100, 200),
            border_color: Color32::from_rgb(80, 80, 80),
            show_hotkeys: true,
            show_numbers: true,
            show_charges: true,
        }
    }
}

// ============================================================================
// Combat HUD Model
// ============================================================================

/// Model containing all combat HUD state.
#[derive(Debug)]
pub struct CombatHUDModel {
    /// Crosshair configuration.
    pub crosshair_config: CrosshairConfig,
    /// Crosshair state.
    pub crosshair_state: CrosshairState,
    /// Health bar configuration.
    pub health_bar_config: HealthBarConfig,
    /// Entity health bars.
    pub health_bars: HashMap<CombatEntityId, HealthBarState>,
    /// Boss health bar (special rendering).
    pub boss_health_bar: Option<HealthBarState>,
    /// Damage number configuration.
    pub damage_number_config: DamageNumberConfig,
    /// Active damage numbers.
    pub damage_numbers: Vec<DamageNumber>,
    /// Next damage number ID.
    pub next_damage_id: u64,
    /// Hit indicator configuration.
    pub hit_indicator_config: HitIndicatorConfig,
    /// Active hit indicators.
    pub hit_indicators: Vec<HitIndicator>,
    /// Next hit indicator ID.
    pub next_hit_indicator_id: u64,
    /// Cooldown configuration.
    pub cooldown_config: CooldownConfig,
    /// Ability cooldowns.
    pub cooldowns: Vec<AbilityCooldown>,
    /// Player health (for player-specific HUD).
    pub player_health: Option<HealthBarState>,
    /// Screen size for positioning.
    pub screen_size: Vec2,
    /// Whether the HUD is visible.
    pub visible: bool,
    /// Whether in combat mode.
    pub in_combat: bool,
}

impl Default for CombatHUDModel {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatHUDModel {
    /// Create a new combat HUD model.
    pub fn new() -> Self {
        Self {
            crosshair_config: CrosshairConfig::default(),
            crosshair_state: CrosshairState::default(),
            health_bar_config: HealthBarConfig::default(),
            health_bars: HashMap::new(),
            boss_health_bar: None,
            damage_number_config: DamageNumberConfig::default(),
            damage_numbers: Vec::new(),
            next_damage_id: 1,
            hit_indicator_config: HitIndicatorConfig::default(),
            hit_indicators: Vec::new(),
            next_hit_indicator_id: 1,
            cooldown_config: CooldownConfig::default(),
            cooldowns: Vec::new(),
            player_health: None,
            screen_size: Vec2::new(1920.0, 1080.0),
            visible: true,
            in_combat: false,
        }
    }

    /// Set screen size.
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
    }

    /// Get screen center.
    pub fn screen_center(&self) -> Pos2 {
        Pos2::new(self.screen_size.x / 2.0, self.screen_size.y / 2.0)
    }

    /// Set crosshair target type.
    pub fn set_target_type(&mut self, target_type: TargetType) {
        self.crosshair_state.target_type = target_type;
    }

    /// Set crosshair visibility.
    pub fn set_crosshair_visible(&mut self, visible: bool) {
        self.crosshair_state.visible = visible;
    }

    /// Trigger crosshair hit marker.
    pub fn trigger_hit_marker(&mut self) {
        self.crosshair_state
            .trigger_hit_marker(self.crosshair_config.hit_marker_duration);
    }

    /// Set crosshair spread.
    pub fn set_crosshair_spread(&mut self, spread: f32) {
        self.crosshair_state.set_spread(spread);
    }

    /// Add or update a health bar.
    pub fn update_health_bar(
        &mut self,
        entity_id: CombatEntityId,
        current_health: f32,
        max_health: f32,
        screen_position: Pos2,
    ) {
        let animate = self.health_bar_config.animate_damage;
        let duration = self.health_bar_config.damage_animation_duration;

        if let Some(health_bar) = self.health_bars.get_mut(&entity_id) {
            health_bar.set_health(current_health, animate, duration);
            health_bar.max_health = max_health;
            health_bar.screen_position = screen_position;
        } else {
            let mut health_bar = HealthBarState::new(entity_id, max_health);
            health_bar.current_health = current_health;
            health_bar.screen_position = screen_position;
            self.health_bars.insert(entity_id, health_bar);
        }
    }

    /// Remove a health bar.
    pub fn remove_health_bar(&mut self, entity_id: CombatEntityId) {
        self.health_bars.remove(&entity_id);
    }

    /// Set boss health bar.
    pub fn set_boss_health(
        &mut self,
        entity_id: CombatEntityId,
        name: &str,
        current_health: f32,
        max_health: f32,
    ) {
        if let Some(ref mut boss) = self.boss_health_bar {
            let animate = self.health_bar_config.animate_damage;
            let duration = self.health_bar_config.damage_animation_duration;
            boss.set_health(current_health, animate, duration);
            boss.max_health = max_health;
        } else {
            let mut boss = HealthBarState::new_boss(entity_id, max_health, name);
            boss.current_health = current_health;
            self.boss_health_bar = Some(boss);
        }
    }

    /// Clear boss health bar.
    pub fn clear_boss_health(&mut self) {
        self.boss_health_bar = None;
    }

    /// Set player health.
    pub fn set_player_health(
        &mut self,
        entity_id: CombatEntityId,
        current_health: f32,
        max_health: f32,
    ) {
        if let Some(ref mut player) = self.player_health {
            let animate = self.health_bar_config.animate_damage;
            let duration = self.health_bar_config.damage_animation_duration;
            player.set_health(current_health, animate, duration);
            player.max_health = max_health;
        } else {
            let mut player = HealthBarState::new(entity_id, max_health);
            player.current_health = current_health;
            self.player_health = Some(player);
        }
    }

    /// Spawn a damage number.
    pub fn spawn_damage_number(&mut self, value: i32, position: Pos2) -> DamageNumberId {
        let id = DamageNumberId::new(self.next_damage_id);
        self.next_damage_id += 1;

        let number = DamageNumber::new(id, value, position, &self.damage_number_config);
        self.damage_numbers.push(number);
        id
    }

    /// Spawn a critical damage number.
    pub fn spawn_critical_damage(&mut self, value: i32, position: Pos2) -> DamageNumberId {
        let id = DamageNumberId::new(self.next_damage_id);
        self.next_damage_id += 1;

        let number = DamageNumber::critical(id, value, position, &self.damage_number_config);
        self.damage_numbers.push(number);
        id
    }

    /// Spawn a healing number.
    pub fn spawn_healing_number(&mut self, value: i32, position: Pos2) -> DamageNumberId {
        let id = DamageNumberId::new(self.next_damage_id);
        self.next_damage_id += 1;

        let number = DamageNumber::healing(id, value, position, &self.damage_number_config);
        self.damage_numbers.push(number);
        id
    }

    /// Spawn a custom damage number.
    pub fn spawn_custom_damage(
        &mut self,
        value: i32,
        position: Pos2,
        damage_type: DamageType,
    ) -> DamageNumberId {
        let id = DamageNumberId::new(self.next_damage_id);
        self.next_damage_id += 1;

        let number = DamageNumber::new(id, value, position, &self.damage_number_config)
            .with_damage_type(damage_type);
        self.damage_numbers.push(number);
        id
    }

    /// Spawn a hit indicator.
    pub fn spawn_hit_indicator(&mut self, direction: HitDirection) -> HitIndicatorId {
        let id = HitIndicatorId::new(self.next_hit_indicator_id);
        self.next_hit_indicator_id += 1;

        let indicator = HitIndicator::new(id, direction, &self.hit_indicator_config);
        self.hit_indicators.push(indicator);
        id
    }

    /// Spawn a hit indicator with intensity.
    pub fn spawn_hit_indicator_with_intensity(
        &mut self,
        direction: HitDirection,
        intensity: f32,
    ) -> HitIndicatorId {
        let id = HitIndicatorId::new(self.next_hit_indicator_id);
        self.next_hit_indicator_id += 1;

        let indicator =
            HitIndicator::new(id, direction, &self.hit_indicator_config).with_intensity(intensity);
        self.hit_indicators.push(indicator);
        id
    }

    /// Add an ability cooldown slot.
    pub fn add_cooldown(&mut self, cooldown: AbilityCooldown) {
        self.cooldowns.push(cooldown);
    }

    /// Start cooldown for an ability.
    pub fn start_ability_cooldown(&mut self, ability_id: AbilityId) {
        if let Some(cooldown) = self
            .cooldowns
            .iter_mut()
            .find(|c| c.ability_id == ability_id)
        {
            cooldown.start_cooldown();
        }
    }

    /// Get ability cooldown state.
    pub fn get_cooldown(&self, ability_id: AbilityId) -> Option<&AbilityCooldown> {
        self.cooldowns.iter().find(|c| c.ability_id == ability_id)
    }

    /// Check if ability can be used.
    pub fn can_use_ability(&self, ability_id: AbilityId) -> bool {
        self.cooldowns
            .iter()
            .find(|c| c.ability_id == ability_id)
            .is_some_and(AbilityCooldown::can_use)
    }

    /// Enter combat mode.
    pub fn enter_combat(&mut self) {
        self.in_combat = true;
    }

    /// Exit combat mode.
    pub fn exit_combat(&mut self) {
        self.in_combat = false;
    }

    /// Update all combat HUD elements.
    pub fn update(&mut self, delta_time: f32) {
        // Update crosshair
        self.crosshair_state.update(delta_time);

        // Update health bars
        for health_bar in self.health_bars.values_mut() {
            health_bar.update(delta_time);
        }

        if let Some(ref mut boss) = self.boss_health_bar {
            boss.update(delta_time);
        }

        if let Some(ref mut player) = self.player_health {
            player.update(delta_time);
        }

        // Update damage numbers and remove expired
        let config = &self.damage_number_config;
        for number in &mut self.damage_numbers {
            number.update(delta_time, config);
        }
        self.damage_numbers.retain(|n| !n.is_expired());

        // Update hit indicators and remove expired
        for indicator in &mut self.hit_indicators {
            indicator.update(delta_time);
        }
        self.hit_indicators.retain(|i| !i.is_expired());

        // Update cooldowns
        for cooldown in &mut self.cooldowns {
            cooldown.update(delta_time);
        }
    }

    /// Get count of active damage numbers.
    pub fn damage_number_count(&self) -> usize {
        self.damage_numbers.len()
    }

    /// Get count of active hit indicators.
    pub fn hit_indicator_count(&self) -> usize {
        self.hit_indicators.len()
    }

    /// Clear all health bars.
    pub fn clear_health_bars(&mut self) {
        self.health_bars.clear();
        self.boss_health_bar = None;
    }

    /// Clear all damage numbers.
    pub fn clear_damage_numbers(&mut self) {
        self.damage_numbers.clear();
    }

    /// Clear all hit indicators.
    pub fn clear_hit_indicators(&mut self) {
        self.hit_indicators.clear();
    }

    /// Reset all combat HUD state.
    pub fn reset(&mut self) {
        self.clear_health_bars();
        self.clear_damage_numbers();
        self.clear_hit_indicators();
        self.crosshair_state = CrosshairState::default();
        self.player_health = None;
        self.in_combat = false;
    }
}

// ============================================================================
// Combat HUD UI
// ============================================================================

/// Actions that can be triggered from the combat HUD.
#[derive(Debug, Clone, PartialEq)]
pub enum CombatAction {
    /// Use an ability.
    UseAbility(AbilityId),
    /// Cycle crosshair style.
    CycleCrosshair,
    /// Toggle HUD visibility.
    ToggleHUD,
}

/// Configuration for combat HUD rendering.
#[derive(Debug, Clone)]
pub struct CombatHUDConfig {
    /// Boss health bar width.
    pub boss_health_width: f32,
    /// Boss health bar height.
    pub boss_health_height: f32,
    /// Player health bar width.
    pub player_health_width: f32,
    /// Player health bar height.
    pub player_health_height: f32,
    /// Whether to show player health as bar or orb.
    pub player_health_style: PlayerHealthStyle,
}

impl Default for CombatHUDConfig {
    fn default() -> Self {
        Self {
            boss_health_width: 400.0,
            boss_health_height: 20.0,
            player_health_width: 200.0,
            player_health_height: 16.0,
            player_health_style: PlayerHealthStyle::Bar,
        }
    }
}

/// Style for player health display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerHealthStyle {
    /// Traditional health bar.
    #[default]
    Bar,
    /// Health orb (like Diablo).
    Orb,
    /// Segmented hearts (like Zelda).
    Hearts,
}

/// Main combat HUD renderer.
pub struct CombatHUD {
    /// HUD configuration.
    pub config: CombatHUDConfig,
    /// Whether the HUD is open.
    open: bool,
    /// Pending actions.
    actions: Vec<CombatAction>,
}

impl std::fmt::Debug for CombatHUD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombatHUD")
            .field("config", &self.config)
            .field("open", &self.open)
            .field("actions", &self.actions)
            .finish()
    }
}

impl Default for CombatHUD {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatHUD {
    /// Create a new combat HUD.
    pub fn new() -> Self {
        Self {
            config: CombatHUDConfig::default(),
            open: true,
            actions: Vec::new(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: CombatHUDConfig) -> Self {
        Self {
            config,
            open: true,
            actions: Vec::new(),
        }
    }

    /// Check if HUD is visible.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Set HUD visibility.
    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    /// Toggle HUD visibility.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Drain pending actions.
    pub fn drain_actions(&mut self) -> Vec<CombatAction> {
        std::mem::take(&mut self.actions)
    }

    /// Render the combat HUD.
    pub fn show(&mut self, ctx: &egui::Context, model: &CombatHUDModel) {
        if !self.open || !model.visible {
            return;
        }

        // Render crosshair
        if model.crosshair_state.visible {
            self.render_crosshair(ctx, model);
        }

        // Render entity health bars
        self.render_entity_health_bars(ctx, model);

        // Render boss health bar
        if let Some(ref boss) = model.boss_health_bar {
            self.render_boss_health_bar(ctx, boss, model);
        }

        // Render player health
        if let Some(ref player) = model.player_health {
            self.render_player_health(ctx, player, model);
        }

        // Render damage numbers
        Self::render_damage_numbers(ctx, model);

        // Render hit indicators
        Self::render_hit_indicators(ctx, model);

        // Render cooldowns
        if !model.cooldowns.is_empty() {
            self.render_cooldowns(ctx, model);
        }
    }

    #[allow(clippy::unused_self)]
    fn render_crosshair(&mut self, ctx: &egui::Context, model: &CombatHUDModel) {
        let screen_center = model.screen_center();
        let config = &model.crosshair_config;
        let state = &model.crosshair_state;

        let color = state.get_color(config);
        let size = config.size * state.spread;

        egui::Area::new(egui::Id::new("combat_crosshair"))
            .fixed_pos(Pos2::new(screen_center.x - size, screen_center.y - size))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let painter = ui.painter();
                let center = screen_center;

                match config.style {
                    CrosshairStyle::Dot => {
                        painter.circle_filled(center, config.thickness * 1.5, color);
                    },
                    CrosshairStyle::Cross => {
                        let gap = config.center_gap * state.spread;
                        let half_size = size / 2.0;

                        // Horizontal lines
                        painter.line_segment(
                            [
                                Pos2::new(center.x - half_size, center.y),
                                Pos2::new(center.x - gap, center.y),
                            ],
                            (config.thickness, color),
                        );
                        painter.line_segment(
                            [
                                Pos2::new(center.x + gap, center.y),
                                Pos2::new(center.x + half_size, center.y),
                            ],
                            (config.thickness, color),
                        );

                        // Vertical lines
                        painter.line_segment(
                            [
                                Pos2::new(center.x, center.y - half_size),
                                Pos2::new(center.x, center.y - gap),
                            ],
                            (config.thickness, color),
                        );
                        painter.line_segment(
                            [
                                Pos2::new(center.x, center.y + gap),
                                Pos2::new(center.x, center.y + half_size),
                            ],
                            (config.thickness, color),
                        );
                    },
                    CrosshairStyle::Circle => {
                        painter.circle_stroke(center, size / 2.0, (config.thickness, color));
                    },
                    CrosshairStyle::CircleCross => {
                        painter.circle_stroke(center, size / 2.0, (config.thickness, color));
                        painter.circle_filled(center, config.thickness, color);
                    },
                    CrosshairStyle::Chevron => {
                        let half_size = size / 2.0;
                        painter.line_segment(
                            [
                                Pos2::new(center.x - half_size, center.y + half_size * 0.5),
                                Pos2::new(center.x, center.y - half_size * 0.5),
                            ],
                            (config.thickness, color),
                        );
                        painter.line_segment(
                            [
                                Pos2::new(center.x, center.y - half_size * 0.5),
                                Pos2::new(center.x + half_size, center.y + half_size * 0.5),
                            ],
                            (config.thickness, color),
                        );
                    },
                    CrosshairStyle::None => {},
                }

                // Render hit marker
                if state.is_hit_marker_active() && config.show_hit_marker {
                    let hit_color = Color32::WHITE;
                    let hit_size = size * 0.8;
                    let offset = hit_size / 2.0;

                    // X pattern
                    painter.line_segment(
                        [
                            Pos2::new(center.x - offset, center.y - offset),
                            Pos2::new(center.x + offset, center.y + offset),
                        ],
                        (config.thickness * 1.5, hit_color),
                    );
                    painter.line_segment(
                        [
                            Pos2::new(center.x + offset, center.y - offset),
                            Pos2::new(center.x - offset, center.y + offset),
                        ],
                        (config.thickness * 1.5, hit_color),
                    );
                }
            });
    }

    fn render_entity_health_bars(&mut self, ctx: &egui::Context, model: &CombatHUDModel) {
        let config = &model.health_bar_config;

        for health_bar in model.health_bars.values() {
            if !health_bar.visible {
                continue;
            }

            if config.hide_when_full && health_bar.health_percent() >= 1.0 {
                continue;
            }

            let pos = Pos2::new(
                health_bar.screen_position.x - config.width / 2.0,
                health_bar.screen_position.y + config.y_offset,
            );

            egui::Area::new(egui::Id::new(format!(
                "health_bar_{}",
                health_bar.entity_id.0
            )))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                self.draw_health_bar(ui, health_bar, config);
            });
        }
    }

    fn render_boss_health_bar(
        &mut self,
        ctx: &egui::Context,
        boss: &HealthBarState,
        model: &CombatHUDModel,
    ) {
        let width = self.config.boss_health_width;
        let height = self.config.boss_health_height;

        let pos = Pos2::new(model.screen_size.x / 2.0 - width / 2.0, 50.0);

        egui::Area::new(egui::Id::new("boss_health_bar"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if let Some(ref name) = boss.name {
                        ui.label(
                            egui::RichText::new(name)
                                .size(16.0)
                                .color(Color32::WHITE)
                                .strong(),
                        );
                    }

                    let rect = Rect::from_min_size(ui.cursor().min, Vec2::new(width, height));

                    let painter = ui.painter();

                    // Background
                    painter.rect_filled(rect, 4.0, model.health_bar_config.background_color);

                    // Health fill
                    let health_width = rect.width() * boss.health_percent();
                    let health_rect =
                        Rect::from_min_size(rect.min, Vec2::new(health_width, rect.height()));
                    painter.rect_filled(
                        health_rect,
                        4.0,
                        boss.get_health_color(&model.health_bar_config),
                    );

                    // Damage animation
                    if boss.display_health > boss.current_health {
                        let display_width = rect.width() * (boss.display_health / boss.max_health);
                        let damage_rect = Rect::from_min_size(
                            Pos2::new(rect.min.x + health_width, rect.min.y),
                            Vec2::new(display_width - health_width, rect.height()),
                        );
                        painter.rect_filled(
                            damage_rect,
                            0.0,
                            Color32::from_rgba_unmultiplied(255, 255, 255, 150),
                        );
                    }

                    // Border
                    painter.rect_stroke(rect, 4.0, (2.0, model.health_bar_config.border_color));

                    ui.allocate_space(Vec2::new(width, height + 8.0));
                });
            });
    }

    fn render_player_health(
        &mut self,
        ctx: &egui::Context,
        player: &HealthBarState,
        model: &CombatHUDModel,
    ) {
        let width = self.config.player_health_width;
        let height = self.config.player_health_height;

        let pos = Pos2::new(20.0, model.screen_size.y - height - 20.0);

        egui::Area::new(egui::Id::new("player_health"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                match self.config.player_health_style {
                    PlayerHealthStyle::Bar => {
                        let rect = Rect::from_min_size(pos, Vec2::new(width, height));
                        let painter = ui.painter();

                        // Background
                        painter.rect_filled(rect, 4.0, model.health_bar_config.background_color);

                        // Health fill
                        let health_width = rect.width() * player.health_percent();
                        let health_rect =
                            Rect::from_min_size(rect.min, Vec2::new(health_width, rect.height()));
                        painter.rect_filled(
                            health_rect,
                            4.0,
                            player.get_health_color(&model.health_bar_config),
                        );

                        // Border
                        painter.rect_stroke(rect, 4.0, (2.0, model.health_bar_config.border_color));

                        // Health text
                        let text = format!(
                            "{} / {}",
                            player.current_health as i32, player.max_health as i32
                        );
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    },
                    PlayerHealthStyle::Orb | PlayerHealthStyle::Hearts => {
                        // Simplified - just use bar style for now
                        ui.label(format!(
                            "HP: {} / {}",
                            player.current_health as i32, player.max_health as i32
                        ));
                    },
                }

                ui.allocate_space(Vec2::new(width, height));
            });
    }

    fn render_damage_numbers(ctx: &egui::Context, model: &CombatHUDModel) {
        let config = &model.damage_number_config;

        for number in &model.damage_numbers {
            let scale = number.get_scale(config);
            let font_size = config.font_size * scale;
            let color = number.get_color(config);
            let text = number.get_text();

            egui::Area::new(egui::Id::new(format!("damage_number_{}", number.id.0)))
                .fixed_pos(number.position)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(&text)
                            .size(font_size)
                            .color(color)
                            .strong(),
                    );
                });
        }
    }

    fn render_hit_indicators(ctx: &egui::Context, model: &CombatHUDModel) {
        let config = &model.hit_indicator_config;
        let screen_center = model.screen_center();

        for indicator in &model.hit_indicators {
            if !config.directional && indicator.direction == HitDirection::Unknown {
                continue;
            }

            let pos = indicator.get_position(screen_center, config);
            let opacity = indicator.get_opacity(config);
            let color = Color32::from_rgba_unmultiplied(
                config.color.r(),
                config.color.g(),
                config.color.b(),
                opacity,
            );

            egui::Area::new(egui::Id::new(format!("hit_indicator_{}", indicator.id.0)))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let painter = ui.painter();
                    let angle = indicator.direction.angle();
                    let size = config.size;

                    // Draw arrow/wedge shape
                    let tip = pos;
                    let base_left = Pos2::new(
                        pos.x - (angle + 0.5).cos() * size,
                        pos.y + (angle + 0.5).sin() * size,
                    );
                    let base_right = Pos2::new(
                        pos.x - (angle - 0.5).cos() * size,
                        pos.y + (angle - 0.5).sin() * size,
                    );

                    painter.add(egui::Shape::convex_polygon(
                        vec![tip, base_left, base_right],
                        color,
                        egui::Stroke::NONE,
                    ));
                });
        }
    }

    fn render_cooldowns(&mut self, ctx: &egui::Context, model: &CombatHUDModel) {
        let config = &model.cooldown_config;
        let total_width =
            model.cooldowns.len() as f32 * (config.icon_size + config.spacing) - config.spacing;
        let start_x = model.screen_size.x / 2.0 - total_width / 2.0;
        let y = model.screen_size.y - config.y_offset;

        egui::Area::new(egui::Id::new("ability_cooldowns"))
            .fixed_pos(Pos2::new(start_x, y))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = config.spacing;

                    for cooldown in &model.cooldowns {
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(config.icon_size, config.icon_size),
                            egui::Sense::click(),
                        );

                        if response.clicked() {
                            self.actions
                                .push(CombatAction::UseAbility(cooldown.ability_id));
                        }

                        let painter = ui.painter();

                        // Background
                        painter.rect_filled(rect, 4.0, config.background_color);

                        // Cooldown overlay
                        if !cooldown.is_ready {
                            let progress = cooldown.progress();
                            let overlay_height = rect.height() * (1.0 - progress);
                            let overlay_rect = Rect::from_min_size(
                                rect.min,
                                Vec2::new(rect.width(), overlay_height),
                            );
                            painter.rect_filled(overlay_rect, 0.0, config.cooldown_color);
                        }

                        // Border
                        let border_color = if cooldown.is_ready {
                            config.ready_color
                        } else {
                            config.border_color
                        };
                        painter.rect_stroke(rect, 4.0, (2.0, border_color));

                        // Cooldown number
                        if config.show_numbers && cooldown.remaining > 0.0 {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{:.1}", cooldown.remaining),
                                egui::FontId::proportional(14.0),
                                Color32::WHITE,
                            );
                        }

                        // Hotkey
                        if config.show_hotkeys {
                            if let Some(ref hotkey) = cooldown.hotkey {
                                painter.text(
                                    Pos2::new(rect.right() - 4.0, rect.bottom() - 4.0),
                                    egui::Align2::RIGHT_BOTTOM,
                                    hotkey,
                                    egui::FontId::proportional(10.0),
                                    Color32::LIGHT_GRAY,
                                );
                            }
                        }

                        // Charges
                        if config.show_charges && cooldown.max_charges > 1 {
                            painter.text(
                                Pos2::new(rect.left() + 4.0, rect.bottom() - 4.0),
                                egui::Align2::LEFT_BOTTOM,
                                format!("{}", cooldown.charges),
                                egui::FontId::proportional(10.0),
                                Color32::WHITE,
                            );
                        }
                    }
                });
            });
    }

    #[allow(clippy::unused_self)]
    fn draw_health_bar(
        &mut self,
        ui: &mut egui::Ui,
        health_bar: &HealthBarState,
        config: &HealthBarConfig,
    ) {
        let rect = Rect::from_min_size(ui.cursor().min, Vec2::new(config.width, config.height));
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, 2.0, config.background_color);

        // Health fill
        let health_width = rect.width() * health_bar.health_percent();
        let health_rect = Rect::from_min_size(rect.min, Vec2::new(health_width, rect.height()));
        painter.rect_filled(health_rect, 2.0, health_bar.get_health_color(config));

        // Border
        painter.rect_stroke(rect, 2.0, (config.border_thickness, config.border_color));

        // Name
        if config.show_name {
            if let Some(ref name) = health_bar.name {
                painter.text(
                    Pos2::new(rect.center().x, rect.min.y - 12.0),
                    egui::Align2::CENTER_BOTTOM,
                    name,
                    egui::FontId::proportional(config.name_font_size),
                    Color32::WHITE,
                );
            }
        }

        ui.allocate_space(Vec2::new(config.width, config.height));
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_entity_id() {
        let id = CombatEntityId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_damage_number_id() {
        let id = DamageNumberId::new(123);
        assert_eq!(id.0, 123);
    }

    #[test]
    fn test_crosshair_style_all() {
        let styles = CrosshairStyle::all();
        assert_eq!(styles.len(), 6);
    }

    #[test]
    fn test_crosshair_style_display_name() {
        assert_eq!(CrosshairStyle::Dot.display_name(), "Dot");
        assert_eq!(CrosshairStyle::Cross.display_name(), "Cross");
        assert_eq!(CrosshairStyle::Circle.display_name(), "Circle");
    }

    #[test]
    fn test_crosshair_config_defaults() {
        let config = CrosshairConfig::default();
        assert_eq!(config.style, CrosshairStyle::Dot);
        assert_eq!(config.size, 16.0);
        assert!(config.show_hit_marker);
    }

    #[test]
    fn test_crosshair_state_new() {
        let state = CrosshairState::new();
        assert_eq!(state.spread, 1.0);
        assert!(!state.is_hit_marker_active());
        assert!(state.visible);
    }

    #[test]
    fn test_crosshair_state_hit_marker() {
        let mut state = CrosshairState::new();
        state.trigger_hit_marker(Duration::from_millis(100));
        assert!(state.is_hit_marker_active());
        state.update(0.2);
        assert!(!state.is_hit_marker_active());
    }

    #[test]
    fn test_crosshair_state_spread() {
        let mut state = CrosshairState::new();
        state.set_spread(2.0);
        assert_eq!(state.target_spread, 2.0);
    }

    #[test]
    fn test_target_type_color() {
        let config = CrosshairConfig::default();
        let mut state = CrosshairState::new();

        state.target_type = TargetType::Enemy;
        assert_eq!(state.get_color(&config), config.enemy_color);

        state.target_type = TargetType::Friendly;
        assert_eq!(state.get_color(&config), config.friendly_color);
    }

    #[test]
    fn test_health_bar_config_defaults() {
        let config = HealthBarConfig::default();
        assert_eq!(config.width, 60.0);
        assert_eq!(config.height, 8.0);
        assert!(config.hide_when_full);
    }

    #[test]
    fn test_health_bar_state_new() {
        let state = HealthBarState::new(CombatEntityId::new(1), 100.0);
        assert_eq!(state.current_health, 100.0);
        assert_eq!(state.max_health, 100.0);
        assert_eq!(state.health_percent(), 1.0);
    }

    #[test]
    fn test_health_bar_state_set_health() {
        let mut state = HealthBarState::new(CombatEntityId::new(1), 100.0);
        state.set_health(50.0, false, Duration::ZERO);
        assert_eq!(state.current_health, 50.0);
        assert_eq!(state.health_percent(), 0.5);
    }

    #[test]
    fn test_health_bar_state_levels() {
        let mut state = HealthBarState::new(CombatEntityId::new(1), 100.0);

        state.current_health = 100.0;
        assert!(!state.is_low_health());
        assert!(!state.is_medium_health());

        state.current_health = 40.0;
        assert!(!state.is_low_health());
        assert!(state.is_medium_health());

        state.current_health = 20.0;
        assert!(state.is_low_health());
        assert!(!state.is_medium_health());
    }

    #[test]
    fn test_health_bar_is_dead() {
        let mut state = HealthBarState::new(CombatEntityId::new(1), 100.0);
        assert!(!state.is_dead());
        state.current_health = 0.0;
        assert!(state.is_dead());
    }

    #[test]
    fn test_health_bar_boss() {
        let boss = HealthBarState::new_boss(CombatEntityId::new(1), 1000.0, "Dragon");
        assert!(boss.is_boss);
        assert_eq!(boss.name, Some("Dragon".to_string()));
    }

    #[test]
    fn test_damage_type_default_color() {
        assert_eq!(DamageType::Physical.default_color(), Color32::WHITE);
        assert_eq!(
            DamageType::Critical.default_color(),
            Color32::from_rgb(255, 200, 50)
        );
        assert_eq!(
            DamageType::Healing.default_color(),
            Color32::from_rgb(50, 255, 100)
        );
    }

    #[test]
    fn test_damage_type_prefix_suffix() {
        assert_eq!(DamageType::Physical.prefix(), "");
        assert_eq!(DamageType::Critical.prefix(), "💥");
        assert_eq!(DamageType::Healing.prefix(), "+");
        assert_eq!(DamageType::Experience.suffix(), " XP");
    }

    #[test]
    fn test_damage_number_config_defaults() {
        let config = DamageNumberConfig::default();
        assert_eq!(config.font_size, 18.0);
        assert_eq!(config.critical_scale, 1.5);
        assert!(config.fade_out);
    }

    #[test]
    fn test_damage_number_new() {
        let config = DamageNumberConfig::default();
        let number = DamageNumber::new(
            DamageNumberId::new(1),
            100,
            Pos2::new(100.0, 100.0),
            &config,
        );
        assert_eq!(number.value, 100);
        assert_eq!(number.damage_type, DamageType::Physical);
        assert!(!number.is_expired());
    }

    #[test]
    fn test_damage_number_critical() {
        let config = DamageNumberConfig::default();
        let number = DamageNumber::critical(
            DamageNumberId::new(1),
            200,
            Pos2::new(100.0, 100.0),
            &config,
        );
        assert_eq!(number.damage_type, DamageType::Critical);
    }

    #[test]
    fn test_damage_number_update() {
        let config = DamageNumberConfig::default();
        let mut number = DamageNumber::new(
            DamageNumberId::new(1),
            100,
            Pos2::new(100.0, 100.0),
            &config,
        );
        let initial_y = number.position.y;
        number.update(0.1, &config);
        assert!(number.position.y < initial_y);
        assert!(number.elapsed > 0.0);
    }

    #[test]
    fn test_damage_number_expired() {
        let config = DamageNumberConfig::default();
        let mut number = DamageNumber::new(
            DamageNumberId::new(1),
            100,
            Pos2::new(100.0, 100.0),
            &config,
        );
        number.elapsed = number.lifetime + 1.0;
        assert!(number.is_expired());
    }

    #[test]
    fn test_damage_number_text() {
        let config = DamageNumberConfig::default();
        let number = DamageNumber::new(
            DamageNumberId::new(1),
            100,
            Pos2::new(100.0, 100.0),
            &config,
        );
        assert_eq!(number.get_text(), "100");
    }

    #[test]
    fn test_hit_direction_angle() {
        assert_eq!(HitDirection::Right.angle(), 0.0);
        assert!((HitDirection::Left.angle() - std::f32::consts::PI).abs() < 0.01);
    }

    #[test]
    fn test_hit_direction_from_angle() {
        assert_eq!(HitDirection::from_angle(0.0), HitDirection::Right);
        assert_eq!(
            HitDirection::from_angle(std::f32::consts::PI),
            HitDirection::Left
        );
    }

    #[test]
    fn test_hit_indicator_config_defaults() {
        let config = HitIndicatorConfig::default();
        assert_eq!(config.size, 40.0);
        assert_eq!(config.distance, 100.0);
        assert!(config.fade_out);
    }

    #[test]
    fn test_hit_indicator_new() {
        let config = HitIndicatorConfig::default();
        let indicator = HitIndicator::new(HitIndicatorId::new(1), HitDirection::Front, &config);
        assert_eq!(indicator.direction, HitDirection::Front);
        assert!(!indicator.is_expired());
    }

    #[test]
    fn test_hit_indicator_update() {
        let config = HitIndicatorConfig::default();
        let mut indicator = HitIndicator::new(HitIndicatorId::new(1), HitDirection::Front, &config);
        indicator.update(0.6);
        assert!(indicator.is_expired());
    }

    #[test]
    fn test_ability_cooldown_new() {
        let cooldown = AbilityCooldown::new(AbilityId::new(1), "Fireball")
            .with_duration(Duration::from_secs(5))
            .with_hotkey("Q");
        assert_eq!(cooldown.name, "Fireball");
        assert_eq!(cooldown.total_duration, 5.0);
        assert_eq!(cooldown.hotkey, Some("Q".to_string()));
        assert!(cooldown.is_ready);
    }

    #[test]
    fn test_ability_cooldown_start() {
        let mut cooldown = AbilityCooldown::new(AbilityId::new(1), "Fireball")
            .with_duration(Duration::from_secs(5));
        cooldown.start_cooldown();
        assert!(!cooldown.is_ready);
        assert_eq!(cooldown.remaining, 5.0);
    }

    #[test]
    fn test_ability_cooldown_update() {
        let mut cooldown = AbilityCooldown::new(AbilityId::new(1), "Fireball")
            .with_duration(Duration::from_secs(5));
        cooldown.start_cooldown();
        cooldown.update(3.0);
        assert_eq!(cooldown.remaining, 2.0);
        cooldown.update(3.0);
        assert!(cooldown.is_ready);
    }

    #[test]
    fn test_ability_cooldown_charges() {
        let mut cooldown = AbilityCooldown::new(AbilityId::new(1), "Dash")
            .with_duration(Duration::from_secs(2))
            .with_charges(2, 2);
        assert!(cooldown.can_use());
        cooldown.start_cooldown();
        assert_eq!(cooldown.charges, 1);
        assert!(cooldown.can_use());
        cooldown.start_cooldown();
        assert_eq!(cooldown.charges, 0);
        assert!(!cooldown.can_use());
    }

    #[test]
    fn test_cooldown_config_defaults() {
        let config = CooldownConfig::default();
        assert_eq!(config.icon_size, 48.0);
        assert!(config.show_hotkeys);
        assert!(config.show_numbers);
    }

    #[test]
    fn test_combat_hud_model_new() {
        let model = CombatHUDModel::new();
        assert!(model.visible);
        assert!(!model.in_combat);
        assert_eq!(model.health_bars.len(), 0);
    }

    #[test]
    fn test_combat_hud_model_screen_center() {
        let mut model = CombatHUDModel::new();
        model.set_screen_size(1920.0, 1080.0);
        let center = model.screen_center();
        assert_eq!(center.x, 960.0);
        assert_eq!(center.y, 540.0);
    }

    #[test]
    fn test_combat_hud_model_update_health_bar() {
        let mut model = CombatHUDModel::new();
        let id = CombatEntityId::new(1);
        model.update_health_bar(id, 50.0, 100.0, Pos2::ZERO);
        assert!(model.health_bars.contains_key(&id));
        assert_eq!(model.health_bars[&id].current_health, 50.0);
    }

    #[test]
    fn test_combat_hud_model_spawn_damage_number() {
        let mut model = CombatHUDModel::new();
        let id = model.spawn_damage_number(100, Pos2::new(100.0, 100.0));
        assert_eq!(model.damage_number_count(), 1);
        assert_eq!(id.0, 1);
    }

    #[test]
    fn test_combat_hud_model_spawn_hit_indicator() {
        let mut model = CombatHUDModel::new();
        model.spawn_hit_indicator(HitDirection::Front);
        assert_eq!(model.hit_indicator_count(), 1);
    }

    #[test]
    fn test_combat_hud_model_add_cooldown() {
        let mut model = CombatHUDModel::new();
        model.add_cooldown(AbilityCooldown::new(AbilityId::new(1), "Fireball"));
        assert_eq!(model.cooldowns.len(), 1);
    }

    #[test]
    fn test_combat_hud_model_start_ability_cooldown() {
        let mut model = CombatHUDModel::new();
        let id = AbilityId::new(1);
        model.add_cooldown(
            AbilityCooldown::new(id, "Fireball").with_duration(Duration::from_secs(5)),
        );
        assert!(model.can_use_ability(id));
        model.start_ability_cooldown(id);
        assert!(!model.cooldowns[0].is_ready);
    }

    #[test]
    fn test_combat_hud_model_combat_mode() {
        let mut model = CombatHUDModel::new();
        assert!(!model.in_combat);
        model.enter_combat();
        assert!(model.in_combat);
        model.exit_combat();
        assert!(!model.in_combat);
    }

    #[test]
    fn test_combat_hud_model_update() {
        let mut model = CombatHUDModel::new();
        model.spawn_damage_number(100, Pos2::new(100.0, 100.0));
        model.spawn_hit_indicator(HitDirection::Front);

        // Update with enough time to expire both
        model.update(2.0);
        assert_eq!(model.damage_number_count(), 0);
        assert_eq!(model.hit_indicator_count(), 0);
    }

    #[test]
    fn test_combat_hud_model_reset() {
        let mut model = CombatHUDModel::new();
        model.update_health_bar(CombatEntityId::new(1), 50.0, 100.0, Pos2::ZERO);
        model.spawn_damage_number(100, Pos2::ZERO);
        model.spawn_hit_indicator(HitDirection::Front);
        model.enter_combat();

        model.reset();

        assert_eq!(model.health_bars.len(), 0);
        assert_eq!(model.damage_number_count(), 0);
        assert_eq!(model.hit_indicator_count(), 0);
        assert!(!model.in_combat);
    }

    #[test]
    fn test_combat_action_equality() {
        assert_eq!(
            CombatAction::UseAbility(AbilityId::new(1)),
            CombatAction::UseAbility(AbilityId::new(1))
        );
        assert_ne!(
            CombatAction::UseAbility(AbilityId::new(1)),
            CombatAction::UseAbility(AbilityId::new(2))
        );
        assert_eq!(CombatAction::CycleCrosshair, CombatAction::CycleCrosshair);
    }

    #[test]
    fn test_combat_hud_config_defaults() {
        let config = CombatHUDConfig::default();
        assert_eq!(config.boss_health_width, 400.0);
        assert_eq!(config.player_health_style, PlayerHealthStyle::Bar);
    }

    #[test]
    fn test_combat_hud_new() {
        let hud = CombatHUD::new();
        assert!(hud.is_open());
    }

    #[test]
    fn test_combat_hud_toggle() {
        let mut hud = CombatHUD::new();
        assert!(hud.is_open());
        hud.toggle();
        assert!(!hud.is_open());
        hud.toggle();
        assert!(hud.is_open());
    }

    #[test]
    fn test_combat_hud_drain_actions() {
        let mut hud = CombatHUD::new();
        hud.actions.push(CombatAction::CycleCrosshair);
        hud.actions.push(CombatAction::ToggleHUD);

        let actions = hud.drain_actions();
        assert_eq!(actions.len(), 2);
        assert!(hud.drain_actions().is_empty());
    }

    #[test]
    fn test_boss_health_bar_set() {
        let mut model = CombatHUDModel::new();
        model.set_boss_health(CombatEntityId::new(1), "Dragon Lord", 5000.0, 5000.0);
        assert!(model.boss_health_bar.is_some());
        let boss = model.boss_health_bar.as_ref().unwrap();
        assert_eq!(boss.name, Some("Dragon Lord".to_string()));
    }

    #[test]
    fn test_player_health_set() {
        let mut model = CombatHUDModel::new();
        model.set_player_health(CombatEntityId::new(0), 80.0, 100.0);
        assert!(model.player_health.is_some());
        assert_eq!(model.player_health.as_ref().unwrap().current_health, 80.0);
    }

    #[test]
    fn test_crosshair_hit_marker_trigger() {
        let mut model = CombatHUDModel::new();
        model.trigger_hit_marker();
        assert!(model.crosshair_state.is_hit_marker_active());
    }

    #[test]
    fn test_set_crosshair_spread() {
        let mut model = CombatHUDModel::new();
        model.set_crosshair_spread(1.5);
        assert_eq!(model.crosshair_state.target_spread, 1.5);
    }

    #[test]
    fn test_set_crosshair_visible() {
        let mut model = CombatHUDModel::new();
        model.set_crosshair_visible(false);
        assert!(!model.crosshair_state.visible);
    }

    #[test]
    fn test_damage_number_with_builders() {
        let config = DamageNumberConfig::default();
        let number = DamageNumber::new(DamageNumberId::new(1), 50, Pos2::ZERO, &config)
            .with_damage_type(DamageType::Fire)
            .with_velocity(10.0)
            .with_color(Color32::RED)
            .with_text("BURN!".to_string());

        assert_eq!(number.damage_type, DamageType::Fire);
        assert_eq!(number.velocity_x, 10.0);
        assert_eq!(number.color_override, Some(Color32::RED));
        assert_eq!(number.text_override, Some("BURN!".to_string()));
    }

    #[test]
    fn test_hit_indicator_with_intensity() {
        let config = HitIndicatorConfig::default();
        let indicator = HitIndicator::new(HitIndicatorId::new(1), HitDirection::Back, &config)
            .with_intensity(0.5);
        assert_eq!(indicator.intensity, 0.5);
    }

    #[test]
    fn test_spawn_custom_damage() {
        let mut model = CombatHUDModel::new();
        model.spawn_custom_damage(75, Pos2::ZERO, DamageType::Ice);
        assert_eq!(model.damage_numbers[0].damage_type, DamageType::Ice);
    }

    #[test]
    fn test_spawn_healing_number() {
        let mut model = CombatHUDModel::new();
        model.spawn_healing_number(25, Pos2::ZERO);
        assert_eq!(model.damage_numbers[0].damage_type, DamageType::Healing);
    }

    #[test]
    fn test_spawn_critical_damage() {
        let mut model = CombatHUDModel::new();
        model.spawn_critical_damage(150, Pos2::ZERO);
        assert_eq!(model.damage_numbers[0].damage_type, DamageType::Critical);
    }

    #[test]
    fn test_spawn_hit_indicator_with_intensity() {
        let mut model = CombatHUDModel::new();
        model.spawn_hit_indicator_with_intensity(HitDirection::Left, 0.7);
        assert_eq!(model.hit_indicators[0].intensity, 0.7);
    }

    #[test]
    fn test_get_cooldown() {
        let mut model = CombatHUDModel::new();
        let id = AbilityId::new(42);
        model.add_cooldown(AbilityCooldown::new(id, "Shield Bash"));
        let cooldown = model.get_cooldown(id);
        assert!(cooldown.is_some());
        assert_eq!(cooldown.unwrap().name, "Shield Bash");
    }

    #[test]
    fn test_clear_methods() {
        let mut model = CombatHUDModel::new();
        model.update_health_bar(CombatEntityId::new(1), 50.0, 100.0, Pos2::ZERO);
        model.set_boss_health(CombatEntityId::new(2), "Boss", 1000.0, 1000.0);
        model.spawn_damage_number(100, Pos2::ZERO);
        model.spawn_hit_indicator(HitDirection::Front);

        model.clear_health_bars();
        assert!(model.health_bars.is_empty());
        assert!(model.boss_health_bar.is_none());

        model.clear_damage_numbers();
        assert!(model.damage_numbers.is_empty());

        model.clear_hit_indicators();
        assert!(model.hit_indicators.is_empty());
    }

    #[test]
    fn test_health_bar_shield() {
        let mut state = HealthBarState::new(CombatEntityId::new(1), 100.0);
        state.max_shield = 50.0;
        state.set_shield(30.0);
        assert_eq!(state.shield, 30.0);
        assert_eq!(state.shield_percent(), 0.6);
    }

    #[test]
    fn test_combat_hud_with_config() {
        let config = CombatHUDConfig {
            boss_health_width: 500.0,
            boss_health_height: 25.0,
            player_health_width: 250.0,
            player_health_height: 20.0,
            player_health_style: PlayerHealthStyle::Orb,
        };
        let hud = CombatHUD::with_config(config);
        assert_eq!(hud.config.boss_health_width, 500.0);
        assert_eq!(hud.config.player_health_style, PlayerHealthStyle::Orb);
    }
}
