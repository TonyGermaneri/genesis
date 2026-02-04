//! Combat HUD UI components.
//!
//! Provides combat HUD functionality including:
//! - Combo counter display
//! - Damage flash overlay
//! - Low health warning
//! - Status effect icons

use egui::{Color32, Pos2, Rect, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Status effect types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectType {
    /// Poison damage over time.
    Poison,
    /// Fire damage over time.
    Burning,
    /// Ice slow effect.
    Frozen,
    /// Lightning stun.
    Shocked,
    /// Bleeding damage.
    Bleeding,
    /// Regeneration healing.
    Regeneration,
    /// Attack boost.
    Strength,
    /// Defense boost.
    Defense,
    /// Speed boost.
    Haste,
    /// Speed reduction.
    Slow,
    /// Damage reflection.
    Thorns,
    /// Invisibility.
    Invisible,
    /// Invincibility frames.
    Invincible,
    /// Stunned (cannot act).
    Stunned,
    /// Silenced (no abilities).
    Silenced,
}

impl StatusEffectType {
    /// Get all status effect types.
    pub fn all() -> &'static [StatusEffectType] {
        &[
            StatusEffectType::Poison,
            StatusEffectType::Burning,
            StatusEffectType::Frozen,
            StatusEffectType::Shocked,
            StatusEffectType::Bleeding,
            StatusEffectType::Regeneration,
            StatusEffectType::Strength,
            StatusEffectType::Defense,
            StatusEffectType::Haste,
            StatusEffectType::Slow,
            StatusEffectType::Thorns,
            StatusEffectType::Invisible,
            StatusEffectType::Invincible,
            StatusEffectType::Stunned,
            StatusEffectType::Silenced,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            StatusEffectType::Poison => "Poison",
            StatusEffectType::Burning => "Burning",
            StatusEffectType::Frozen => "Frozen",
            StatusEffectType::Shocked => "Shocked",
            StatusEffectType::Bleeding => "Bleeding",
            StatusEffectType::Regeneration => "Regeneration",
            StatusEffectType::Strength => "Strength",
            StatusEffectType::Defense => "Defense",
            StatusEffectType::Haste => "Haste",
            StatusEffectType::Slow => "Slow",
            StatusEffectType::Thorns => "Thorns",
            StatusEffectType::Invisible => "Invisible",
            StatusEffectType::Invincible => "Invincible",
            StatusEffectType::Stunned => "Stunned",
            StatusEffectType::Silenced => "Silenced",
        }
    }

    /// Get icon character.
    pub fn icon(&self) -> &'static str {
        match self {
            StatusEffectType::Poison => "☠",
            StatusEffectType::Burning => "🔥",
            StatusEffectType::Frozen => "❄",
            StatusEffectType::Shocked | StatusEffectType::Haste => "⚡",
            StatusEffectType::Bleeding => "💧",
            StatusEffectType::Regeneration => "💚",
            StatusEffectType::Strength => "💪",
            StatusEffectType::Defense => "🛡",
            StatusEffectType::Slow => "🐌",
            StatusEffectType::Thorns => "🌵",
            StatusEffectType::Invisible => "👻",
            StatusEffectType::Invincible => "⭐",
            StatusEffectType::Stunned => "💫",
            StatusEffectType::Silenced => "🔇",
        }
    }

    /// Get color for the effect.
    pub fn color(&self) -> Color32 {
        match self {
            StatusEffectType::Poison => Color32::from_rgb(100, 200, 100),
            StatusEffectType::Burning => Color32::from_rgb(255, 150, 50),
            StatusEffectType::Frozen => Color32::from_rgb(150, 200, 255),
            StatusEffectType::Shocked => Color32::from_rgb(255, 255, 100),
            StatusEffectType::Bleeding => Color32::from_rgb(200, 50, 50),
            StatusEffectType::Regeneration => Color32::from_rgb(50, 255, 100),
            StatusEffectType::Strength => Color32::from_rgb(255, 100, 100),
            StatusEffectType::Defense => Color32::from_rgb(100, 150, 255),
            StatusEffectType::Haste => Color32::from_rgb(255, 255, 150),
            StatusEffectType::Slow => Color32::from_rgb(150, 100, 200),
            StatusEffectType::Thorns => Color32::from_rgb(200, 150, 100),
            StatusEffectType::Invisible => Color32::from_rgba_unmultiplied(200, 200, 200, 150),
            StatusEffectType::Invincible => Color32::from_rgb(255, 215, 0),
            StatusEffectType::Stunned => Color32::from_rgb(255, 200, 100),
            StatusEffectType::Silenced => Color32::from_rgb(150, 150, 150),
        }
    }

    /// Whether effect is beneficial.
    pub fn is_buff(&self) -> bool {
        matches!(
            self,
            StatusEffectType::Regeneration
                | StatusEffectType::Strength
                | StatusEffectType::Defense
                | StatusEffectType::Haste
                | StatusEffectType::Thorns
                | StatusEffectType::Invisible
                | StatusEffectType::Invincible
        )
    }
}

/// A status effect instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    /// Effect type.
    pub effect_type: StatusEffectType,
    /// Remaining duration in seconds.
    pub duration: f32,
    /// Maximum duration.
    pub max_duration: f32,
    /// Stack count.
    pub stacks: u32,
    /// Maximum stacks.
    pub max_stacks: u32,
    /// Effect source (for display).
    pub source: Option<String>,
}

impl StatusEffect {
    /// Create a new status effect.
    pub fn new(effect_type: StatusEffectType, duration: f32) -> Self {
        Self {
            effect_type,
            duration,
            max_duration: duration,
            stacks: 1,
            max_stacks: 1,
            source: None,
        }
    }

    /// Create with stacks.
    pub fn with_stacks(mut self, stacks: u32, max_stacks: u32) -> Self {
        self.stacks = stacks.min(max_stacks);
        self.max_stacks = max_stacks;
        self
    }

    /// Create with source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Get remaining duration as percentage.
    pub fn duration_percent(&self) -> f32 {
        if self.max_duration > 0.0 {
            (self.duration / self.max_duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if effect is expired.
    pub fn is_expired(&self) -> bool {
        self.duration <= 0.0
    }

    /// Update the effect (reduce duration).
    pub fn update(&mut self, dt: f32) {
        self.duration = (self.duration - dt).max(0.0);
    }

    /// Add a stack.
    pub fn add_stack(&mut self) -> bool {
        if self.stacks < self.max_stacks {
            self.stacks += 1;
            true
        } else {
            false
        }
    }

    /// Refresh duration.
    pub fn refresh(&mut self) {
        self.duration = self.max_duration;
    }
}

/// Damage indicator for floating damage numbers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageIndicator {
    /// Damage amount.
    pub amount: f32,
    /// Position.
    pub position: [f32; 2],
    /// Velocity.
    pub velocity: [f32; 2],
    /// Lifetime remaining.
    pub lifetime: f32,
    /// Maximum lifetime.
    pub max_lifetime: f32,
    /// Damage type for coloring.
    pub damage_type: DamageType,
    /// Whether critical hit.
    pub is_critical: bool,
}

impl DamageIndicator {
    /// Create a new damage indicator.
    pub fn new(amount: f32, x: f32, y: f32, damage_type: DamageType) -> Self {
        Self {
            amount,
            position: [x, y],
            velocity: [0.0, -50.0],
            lifetime: 1.5,
            max_lifetime: 1.5,
            damage_type,
            is_critical: false,
        }
    }

    /// Set as critical hit.
    pub fn with_critical(mut self, is_critical: bool) -> Self {
        self.is_critical = is_critical;
        self
    }

    /// Get position as Pos2.
    pub fn pos(&self) -> Pos2 {
        Pos2::new(self.position[0], self.position[1])
    }

    /// Get alpha based on lifetime.
    pub fn alpha(&self) -> f32 {
        if self.max_lifetime > 0.0 {
            (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if expired.
    pub fn is_expired(&self) -> bool {
        self.lifetime <= 0.0
    }

    /// Update position and lifetime.
    pub fn update(&mut self, dt: f32) {
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.velocity[1] += 30.0 * dt; // gravity
        self.lifetime -= dt;
    }
}

/// Damage types for coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DamageType {
    /// Physical damage.
    #[default]
    Physical,
    /// Fire damage.
    Fire,
    /// Ice damage.
    Ice,
    /// Lightning damage.
    Lightning,
    /// Poison damage.
    Poison,
    /// True damage.
    True,
    /// Healing (negative damage).
    Healing,
}

impl DamageType {
    /// Get color for damage type.
    pub fn color(&self) -> Color32 {
        match self {
            DamageType::Physical => Color32::WHITE,
            DamageType::Fire => Color32::from_rgb(255, 150, 50),
            DamageType::Ice => Color32::from_rgb(150, 200, 255),
            DamageType::Lightning => Color32::from_rgb(255, 255, 100),
            DamageType::Poison => Color32::from_rgb(100, 200, 100),
            DamageType::True => Color32::from_rgb(200, 50, 200),
            DamageType::Healing => Color32::from_rgb(50, 255, 100),
        }
    }
}

/// Combo counter state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboCounter {
    /// Current combo count.
    pub count: u32,
    /// Combo timer (resets if no hits).
    pub timer: f32,
    /// Combo timeout duration.
    pub timeout: f32,
    /// Highest combo this session.
    pub highest: u32,
    /// Total damage in combo.
    pub damage: f32,
    /// Display scale for animation.
    pub display_scale: f32,
}

impl Default for ComboCounter {
    fn default() -> Self {
        Self {
            count: 0,
            timer: 0.0,
            timeout: 2.0,
            highest: 0,
            damage: 0.0,
            display_scale: 1.0,
        }
    }
}

impl ComboCounter {
    /// Create new combo counter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom timeout.
    pub fn with_timeout(mut self, timeout: f32) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a hit to the combo.
    pub fn hit(&mut self, damage: f32) {
        self.count += 1;
        self.timer = self.timeout;
        self.damage += damage;
        self.display_scale = 1.5;

        if self.count > self.highest {
            self.highest = self.count;
        }
    }

    /// Reset the combo.
    pub fn reset(&mut self) {
        self.count = 0;
        self.damage = 0.0;
        self.timer = 0.0;
        self.display_scale = 1.0;
    }

    /// Update the combo timer.
    pub fn update(&mut self, dt: f32) {
        if self.count > 0 {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.reset();
            }
        }

        // Animate scale back to 1.0
        if self.display_scale > 1.0 {
            self.display_scale = (self.display_scale - dt * 2.0).max(1.0);
        }
    }

    /// Get timer percentage.
    pub fn timer_percent(&self) -> f32 {
        if self.timeout > 0.0 {
            (self.timer / self.timeout).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if combo is active.
    pub fn is_active(&self) -> bool {
        self.count > 0
    }

    /// Get combo grade.
    pub fn grade(&self) -> ComboGrade {
        match self.count {
            0 => ComboGrade::None,
            1..=2 => ComboGrade::D,
            3..=5 => ComboGrade::C,
            6..=9 => ComboGrade::B,
            10..=19 => ComboGrade::A,
            20..=49 => ComboGrade::S,
            _ => ComboGrade::SSS,
        }
    }
}

/// Combo grade rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ComboGrade {
    /// No combo.
    #[default]
    None,
    /// Low combo.
    D,
    /// Medium combo.
    C,
    /// Good combo.
    B,
    /// Great combo.
    A,
    /// Excellent combo.
    S,
    /// Perfect combo.
    SSS,
}

impl ComboGrade {
    /// Get display text.
    pub fn display(&self) -> &'static str {
        match self {
            ComboGrade::None => "",
            ComboGrade::D => "D",
            ComboGrade::C => "C",
            ComboGrade::B => "B",
            ComboGrade::A => "A",
            ComboGrade::S => "S",
            ComboGrade::SSS => "SSS",
        }
    }

    /// Get color.
    pub fn color(&self) -> Color32 {
        match self {
            ComboGrade::None => Color32::TRANSPARENT,
            ComboGrade::D => Color32::from_gray(150),
            ComboGrade::C => Color32::from_rgb(100, 200, 100),
            ComboGrade::B => Color32::from_rgb(100, 150, 255),
            ComboGrade::A => Color32::from_rgb(200, 150, 255),
            ComboGrade::S => Color32::from_rgb(255, 215, 0),
            ComboGrade::SSS => Color32::from_rgb(255, 100, 100),
        }
    }
}

/// Screen flash effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenFlash {
    /// Flash color (RGBA).
    pub color: [u8; 4],
    /// Current intensity (0.0 - 1.0).
    pub intensity: f32,
    /// Fade speed.
    pub fade_speed: f32,
}

impl Default for ScreenFlash {
    fn default() -> Self {
        Self {
            color: [255, 0, 0, 255],
            intensity: 0.0,
            fade_speed: 5.0,
        }
    }
}

impl ScreenFlash {
    /// Create new screen flash.
    pub fn new() -> Self {
        Self::default()
    }

    /// Trigger a damage flash.
    pub fn damage_flash(&mut self, intensity: f32) {
        self.color = [255, 0, 0, 255];
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Trigger a heal flash.
    pub fn heal_flash(&mut self, intensity: f32) {
        self.color = [50, 255, 100, 255];
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Trigger a custom flash.
    pub fn custom_flash(&mut self, color: [u8; 4], intensity: f32) {
        self.color = color;
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Get current color with alpha.
    pub fn current_color(&self) -> Color32 {
        let alpha = (self.color[3] as f32 * self.intensity * 0.5) as u8;
        Color32::from_rgba_unmultiplied(self.color[0], self.color[1], self.color[2], alpha)
    }

    /// Update the flash (fade out).
    pub fn update(&mut self, dt: f32) {
        if self.intensity > 0.0 {
            self.intensity = (self.intensity - self.fade_speed * dt).max(0.0);
        }
    }

    /// Check if flash is active.
    pub fn is_active(&self) -> bool {
        self.intensity > 0.01
    }
}

/// Low health warning state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowHealthWarning {
    /// Health threshold to trigger warning.
    pub threshold: f32,
    /// Current health percentage.
    pub health_percent: f32,
    /// Pulse timer.
    pub pulse_timer: f32,
    /// Pulse speed.
    pub pulse_speed: f32,
    /// Whether warning is enabled.
    pub enabled: bool,
}

impl Default for LowHealthWarning {
    fn default() -> Self {
        Self {
            threshold: 0.25,
            health_percent: 1.0,
            pulse_timer: 0.0,
            pulse_speed: 3.0,
            enabled: true,
        }
    }
}

impl LowHealthWarning {
    /// Create new warning.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set health percentage.
    pub fn set_health(&mut self, percent: f32) {
        self.health_percent = percent.clamp(0.0, 1.0);
    }

    /// Check if warning is active.
    pub fn is_active(&self) -> bool {
        self.enabled && self.health_percent <= self.threshold
    }

    /// Get current intensity (pulsing).
    pub fn intensity(&self) -> f32 {
        if self.is_active() {
            let base = 1.0 - (self.health_percent / self.threshold);
            let pulse = (self.pulse_timer * self.pulse_speed).sin() * 0.5 + 0.5;
            base * (0.5 + pulse * 0.5)
        } else {
            0.0
        }
    }

    /// Update the warning.
    pub fn update(&mut self, dt: f32) {
        if self.is_active() {
            self.pulse_timer += dt;
        } else {
            self.pulse_timer = 0.0;
        }
    }
}

/// Combat HUD configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatHudConfig {
    /// Show combo counter.
    pub show_combo: bool,
    /// Show damage numbers.
    pub show_damage_numbers: bool,
    /// Show status effects.
    pub show_status_effects: bool,
    /// Enable screen flash.
    pub enable_flash: bool,
    /// Enable low health warning.
    pub enable_low_health_warning: bool,
    /// Status effect icon size.
    pub status_icon_size: f32,
    /// Max damage indicators.
    pub max_damage_indicators: usize,
    /// Combo display position (normalized 0-1).
    pub combo_position: [f32; 2],
    /// Status effects position.
    pub status_position: [f32; 2],
}

impl Default for CombatHudConfig {
    fn default() -> Self {
        Self {
            show_combo: true,
            show_damage_numbers: true,
            show_status_effects: true,
            enable_flash: true,
            enable_low_health_warning: true,
            status_icon_size: 32.0,
            max_damage_indicators: 20,
            combo_position: [0.5, 0.3],
            status_position: [0.02, 0.15],
        }
    }
}

/// Combat HUD widget.
#[derive(Debug)]
pub struct CombatHud {
    /// Configuration.
    pub config: CombatHudConfig,
    /// Combo counter.
    pub combo: ComboCounter,
    /// Status effects.
    pub status_effects: Vec<StatusEffect>,
    /// Damage indicators.
    pub damage_indicators: Vec<DamageIndicator>,
    /// Screen flash.
    pub screen_flash: ScreenFlash,
    /// Low health warning.
    pub low_health_warning: LowHealthWarning,
    /// Whether HUD is visible.
    pub visible: bool,
}

impl Default for CombatHud {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatHud {
    /// Create new combat HUD.
    pub fn new() -> Self {
        Self {
            config: CombatHudConfig::default(),
            combo: ComboCounter::new(),
            status_effects: Vec::new(),
            damage_indicators: Vec::new(),
            screen_flash: ScreenFlash::new(),
            low_health_warning: LowHealthWarning::new(),
            visible: true,
        }
    }

    /// Create with config.
    pub fn with_config(config: CombatHudConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Register a hit.
    pub fn hit(&mut self, damage: f32) {
        self.combo.hit(damage);
    }

    /// Add damage indicator.
    pub fn add_damage(
        &mut self,
        amount: f32,
        x: f32,
        y: f32,
        damage_type: DamageType,
        critical: bool,
    ) {
        if self.damage_indicators.len() >= self.config.max_damage_indicators {
            self.damage_indicators.remove(0);
        }

        let indicator = DamageIndicator::new(amount, x, y, damage_type).with_critical(critical);
        self.damage_indicators.push(indicator);
    }

    /// Add status effect.
    pub fn add_status(&mut self, effect: StatusEffect) {
        // Check for existing effect
        if let Some(existing) = self
            .status_effects
            .iter_mut()
            .find(|e| e.effect_type == effect.effect_type)
        {
            existing.add_stack();
            existing.refresh();
        } else {
            self.status_effects.push(effect);
        }
    }

    /// Remove status effect by type.
    pub fn remove_status(&mut self, effect_type: StatusEffectType) {
        self.status_effects.retain(|e| e.effect_type != effect_type);
    }

    /// Clear all status effects.
    pub fn clear_status(&mut self) {
        self.status_effects.clear();
    }

    /// Trigger damage flash.
    pub fn damage_flash(&mut self, intensity: f32) {
        if self.config.enable_flash {
            self.screen_flash.damage_flash(intensity);
        }
    }

    /// Set health for low health warning.
    pub fn set_health_percent(&mut self, percent: f32) {
        self.low_health_warning.set_health(percent);
    }

    /// Update all components.
    pub fn update(&mut self, dt: f32) {
        self.combo.update(dt);
        self.screen_flash.update(dt);
        self.low_health_warning.update(dt);

        // Update status effects
        for effect in &mut self.status_effects {
            effect.update(dt);
        }
        self.status_effects.retain(|e| !e.is_expired());

        // Update damage indicators
        for indicator in &mut self.damage_indicators {
            indicator.update(dt);
        }
        self.damage_indicators.retain(|i| !i.is_expired());
    }

    /// Render the combat HUD.
    pub fn show(&self, ui: &mut Ui, screen_size: Vec2) {
        if !self.visible {
            return;
        }

        let painter = ui.painter();

        // Screen flash overlay
        if self.config.enable_flash && self.screen_flash.is_active() {
            let rect = Rect::from_min_size(Pos2::ZERO, screen_size);
            painter.rect_filled(rect, 0.0, self.screen_flash.current_color());
        }

        // Low health warning overlay
        if self.config.enable_low_health_warning && self.low_health_warning.is_active() {
            let intensity = self.low_health_warning.intensity();
            let alpha = (intensity * 80.0) as u8;
            let color = Color32::from_rgba_unmultiplied(200, 0, 0, alpha);
            let rect = Rect::from_min_size(Pos2::ZERO, screen_size);
            painter.rect_filled(rect, 0.0, color);
        }

        // Combo counter
        if self.config.show_combo && self.combo.is_active() {
            self.show_combo(ui, screen_size);
        }

        // Status effects
        if self.config.show_status_effects && !self.status_effects.is_empty() {
            self.show_status_effects(ui, screen_size);
        }

        // Damage indicators
        if self.config.show_damage_numbers {
            self.show_damage_indicators(ui);
        }
    }

    /// Show combo counter.
    fn show_combo(&self, ui: &mut Ui, screen_size: Vec2) {
        let pos = Pos2::new(
            screen_size.x * self.config.combo_position[0],
            screen_size.y * self.config.combo_position[1],
        );

        let painter = ui.painter();
        let scale = self.combo.display_scale;

        // Combo count
        let count_text = format!("{}", self.combo.count);
        let count_size = 48.0 * scale;
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            &count_text,
            egui::FontId::proportional(count_size),
            Color32::WHITE,
        );

        // Grade
        let grade = self.combo.grade();
        if grade != ComboGrade::None {
            let grade_pos = Pos2::new(pos.x + 50.0 * scale, pos.y - 10.0);
            painter.text(
                grade_pos,
                egui::Align2::LEFT_CENTER,
                grade.display(),
                egui::FontId::proportional(32.0 * scale),
                grade.color(),
            );
        }

        // Timer bar
        let bar_width = 100.0;
        let bar_height = 4.0;
        let bar_rect = Rect::from_min_size(
            Pos2::new(pos.x - bar_width / 2.0, pos.y + 30.0),
            Vec2::new(bar_width, bar_height),
        );
        painter.rect_filled(bar_rect, 2.0, Color32::from_gray(60));

        let fill_width = bar_width * self.combo.timer_percent();
        let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_width, bar_height));
        painter.rect_filled(fill_rect, 2.0, grade.color());

        // Total damage
        let damage_text = format!("{:.0} DMG", self.combo.damage);
        painter.text(
            Pos2::new(pos.x, pos.y + 50.0),
            egui::Align2::CENTER_CENTER,
            damage_text,
            egui::FontId::proportional(14.0),
            Color32::from_gray(200),
        );
    }

    /// Show status effects.
    fn show_status_effects(&self, ui: &mut Ui, screen_size: Vec2) {
        let start_pos = Pos2::new(
            screen_size.x * self.config.status_position[0],
            screen_size.y * self.config.status_position[1],
        );

        let painter = ui.painter();
        let icon_size = self.config.status_icon_size;
        let spacing = 4.0;

        // Separate buffs and debuffs
        let buffs: Vec<_> = self
            .status_effects
            .iter()
            .filter(|e| e.effect_type.is_buff())
            .collect();
        let debuffs: Vec<_> = self
            .status_effects
            .iter()
            .filter(|e| !e.effect_type.is_buff())
            .collect();

        // Draw buffs
        for (i, effect) in buffs.iter().enumerate() {
            let pos = Pos2::new(start_pos.x + (icon_size + spacing) * i as f32, start_pos.y);
            self.draw_status_icon(painter, pos, effect, icon_size);
        }

        // Draw debuffs below
        for (i, effect) in debuffs.iter().enumerate() {
            let pos = Pos2::new(
                start_pos.x + (icon_size + spacing) * i as f32,
                start_pos.y + icon_size + spacing,
            );
            self.draw_status_icon(painter, pos, effect, icon_size);
        }
    }

    /// Draw a status effect icon.
    fn draw_status_icon(
        &self,
        painter: &egui::Painter,
        pos: Pos2,
        effect: &StatusEffect,
        size: f32,
    ) {
        let _ = self; // silence unused_self warning - kept for API consistency
        let rect = Rect::from_min_size(pos, Vec2::new(size, size));

        // Background
        let bg_color = if effect.effect_type.is_buff() {
            Color32::from_rgba_unmultiplied(50, 100, 50, 200)
        } else {
            Color32::from_rgba_unmultiplied(100, 50, 50, 200)
        };
        painter.rect_filled(rect, 4.0, bg_color);

        // Icon
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            effect.effect_type.icon(),
            egui::FontId::proportional(size * 0.6),
            effect.effect_type.color(),
        );

        // Duration bar
        let bar_height = 3.0;
        let bar_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, rect.max.y - bar_height),
            Vec2::new(rect.width(), bar_height),
        );
        let fill_width = rect.width() * effect.duration_percent();
        let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_width, bar_height));
        painter.rect_filled(fill_rect, 0.0, Color32::WHITE);

        // Stack count
        if effect.stacks > 1 {
            painter.text(
                Pos2::new(rect.max.x - 2.0, rect.max.y - 2.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("x{}", effect.stacks),
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }

        // Border
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, Color32::from_gray(100)));
    }

    /// Show damage indicators.
    fn show_damage_indicators(&self, ui: &mut Ui) {
        let painter = ui.painter();

        for indicator in &self.damage_indicators {
            let alpha = (indicator.alpha() * 255.0) as u8;
            let base_color = indicator.damage_type.color();
            let color = Color32::from_rgba_unmultiplied(
                base_color.r(),
                base_color.g(),
                base_color.b(),
                alpha,
            );

            let text = if indicator.is_critical {
                format!("{:.0}!", indicator.amount)
            } else {
                format!("{:.0}", indicator.amount)
            };

            let font_size = if indicator.is_critical { 24.0 } else { 18.0 };

            painter.text(
                indicator.pos(),
                egui::Align2::CENTER_CENTER,
                &text,
                egui::FontId::proportional(font_size),
                color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_effect_type_all() {
        assert_eq!(StatusEffectType::all().len(), 15);
    }

    #[test]
    fn test_status_effect_type_display_name() {
        assert_eq!(StatusEffectType::Poison.display_name(), "Poison");
        assert_eq!(StatusEffectType::Burning.display_name(), "Burning");
    }

    #[test]
    fn test_status_effect_type_icon() {
        assert_eq!(StatusEffectType::Poison.icon(), "☠");
        assert_eq!(StatusEffectType::Burning.icon(), "🔥");
    }

    #[test]
    fn test_status_effect_type_is_buff() {
        assert!(StatusEffectType::Regeneration.is_buff());
        assert!(StatusEffectType::Strength.is_buff());
        assert!(!StatusEffectType::Poison.is_buff());
        assert!(!StatusEffectType::Stunned.is_buff());
    }

    #[test]
    fn test_status_effect_new() {
        let effect = StatusEffect::new(StatusEffectType::Poison, 5.0);
        assert_eq!(effect.effect_type, StatusEffectType::Poison);
        assert_eq!(effect.duration, 5.0);
        assert_eq!(effect.stacks, 1);
    }

    #[test]
    fn test_status_effect_with_stacks() {
        let effect = StatusEffect::new(StatusEffectType::Poison, 5.0).with_stacks(3, 5);
        assert_eq!(effect.stacks, 3);
        assert_eq!(effect.max_stacks, 5);
    }

    #[test]
    fn test_status_effect_duration_percent() {
        let mut effect = StatusEffect::new(StatusEffectType::Poison, 10.0);
        assert_eq!(effect.duration_percent(), 1.0);

        effect.duration = 5.0;
        assert_eq!(effect.duration_percent(), 0.5);
    }

    #[test]
    fn test_status_effect_update() {
        let mut effect = StatusEffect::new(StatusEffectType::Poison, 5.0);
        effect.update(2.0);
        assert_eq!(effect.duration, 3.0);
    }

    #[test]
    fn test_status_effect_is_expired() {
        let mut effect = StatusEffect::new(StatusEffectType::Poison, 1.0);
        assert!(!effect.is_expired());
        effect.update(2.0);
        assert!(effect.is_expired());
    }

    #[test]
    fn test_status_effect_add_stack() {
        let mut effect = StatusEffect::new(StatusEffectType::Poison, 5.0).with_stacks(1, 3);
        assert!(effect.add_stack());
        assert_eq!(effect.stacks, 2);
        assert!(effect.add_stack());
        assert_eq!(effect.stacks, 3);
        assert!(!effect.add_stack());
        assert_eq!(effect.stacks, 3);
    }

    #[test]
    fn test_status_effect_refresh() {
        let mut effect = StatusEffect::new(StatusEffectType::Poison, 10.0);
        effect.update(5.0);
        assert_eq!(effect.duration, 5.0);
        effect.refresh();
        assert_eq!(effect.duration, 10.0);
    }

    #[test]
    fn test_damage_indicator_new() {
        let indicator = DamageIndicator::new(50.0, 100.0, 200.0, DamageType::Physical);
        assert_eq!(indicator.amount, 50.0);
        assert_eq!(indicator.position, [100.0, 200.0]);
    }

    #[test]
    fn test_damage_indicator_with_critical() {
        let indicator = DamageIndicator::new(100.0, 0.0, 0.0, DamageType::Fire).with_critical(true);
        assert!(indicator.is_critical);
    }

    #[test]
    fn test_damage_indicator_update() {
        let mut indicator = DamageIndicator::new(50.0, 100.0, 200.0, DamageType::Physical);
        indicator.update(0.5);
        assert!(indicator.position[1] < 200.0);
        assert_eq!(indicator.lifetime, 1.0);
    }

    #[test]
    fn test_damage_indicator_is_expired() {
        let mut indicator = DamageIndicator::new(50.0, 0.0, 0.0, DamageType::Physical);
        assert!(!indicator.is_expired());
        indicator.lifetime = 0.0;
        assert!(indicator.is_expired());
    }

    #[test]
    fn test_damage_type_color() {
        assert_ne!(DamageType::Physical.color(), DamageType::Fire.color());
        assert_ne!(DamageType::Ice.color(), DamageType::Lightning.color());
    }

    #[test]
    fn test_combo_counter_new() {
        let combo = ComboCounter::new();
        assert_eq!(combo.count, 0);
        assert_eq!(combo.highest, 0);
    }

    #[test]
    fn test_combo_counter_hit() {
        let mut combo = ComboCounter::new();
        combo.hit(50.0);
        assert_eq!(combo.count, 1);
        assert_eq!(combo.damage, 50.0);
        assert_eq!(combo.highest, 1);
    }

    #[test]
    fn test_combo_counter_reset() {
        let mut combo = ComboCounter::new();
        combo.hit(50.0);
        combo.hit(30.0);
        combo.reset();
        assert_eq!(combo.count, 0);
        assert_eq!(combo.damage, 0.0);
    }

    #[test]
    fn test_combo_counter_update_timeout() {
        let mut combo = ComboCounter::new().with_timeout(1.0);
        combo.hit(50.0);
        assert!(combo.is_active());
        combo.update(2.0);
        assert!(!combo.is_active());
    }

    #[test]
    fn test_combo_counter_grade() {
        let mut combo = ComboCounter::new();
        assert_eq!(combo.grade(), ComboGrade::None);
        combo.count = 1;
        assert_eq!(combo.grade(), ComboGrade::D);
        combo.count = 5;
        assert_eq!(combo.grade(), ComboGrade::C);
        combo.count = 10;
        assert_eq!(combo.grade(), ComboGrade::A);
        combo.count = 50;
        assert_eq!(combo.grade(), ComboGrade::SSS);
    }

    #[test]
    fn test_combo_grade_display() {
        assert_eq!(ComboGrade::A.display(), "A");
        assert_eq!(ComboGrade::SSS.display(), "SSS");
    }

    #[test]
    fn test_screen_flash_damage() {
        let mut flash = ScreenFlash::new();
        assert!(!flash.is_active());
        flash.damage_flash(0.5);
        assert!(flash.is_active());
    }

    #[test]
    fn test_screen_flash_heal() {
        let mut flash = ScreenFlash::new();
        flash.heal_flash(0.5);
        assert!(flash.is_active());
        assert_eq!(flash.color, [50, 255, 100, 255]);
    }

    #[test]
    fn test_screen_flash_update() {
        let mut flash = ScreenFlash::new();
        flash.damage_flash(1.0);
        flash.update(1.0);
        assert!(flash.intensity < 1.0);
    }

    #[test]
    fn test_low_health_warning_new() {
        let warning = LowHealthWarning::new();
        assert_eq!(warning.threshold, 0.25);
        assert!(warning.enabled);
    }

    #[test]
    fn test_low_health_warning_is_active() {
        let mut warning = LowHealthWarning::new();
        assert!(!warning.is_active());
        warning.set_health(0.2);
        assert!(warning.is_active());
    }

    #[test]
    fn test_low_health_warning_intensity() {
        let mut warning = LowHealthWarning::new();
        warning.set_health(0.1);
        assert!(warning.intensity() > 0.0);
    }

    #[test]
    fn test_combat_hud_config_default() {
        let config = CombatHudConfig::default();
        assert!(config.show_combo);
        assert!(config.show_damage_numbers);
    }

    #[test]
    fn test_combat_hud_new() {
        let hud = CombatHud::new();
        assert!(hud.visible);
        assert!(hud.status_effects.is_empty());
    }

    #[test]
    fn test_combat_hud_hit() {
        let mut hud = CombatHud::new();
        hud.hit(50.0);
        assert_eq!(hud.combo.count, 1);
    }

    #[test]
    fn test_combat_hud_add_damage() {
        let mut hud = CombatHud::new();
        hud.add_damage(50.0, 100.0, 100.0, DamageType::Fire, true);
        assert_eq!(hud.damage_indicators.len(), 1);
    }

    #[test]
    fn test_combat_hud_add_status() {
        let mut hud = CombatHud::new();
        hud.add_status(StatusEffect::new(StatusEffectType::Poison, 5.0));
        assert_eq!(hud.status_effects.len(), 1);
    }

    #[test]
    fn test_combat_hud_add_status_stacks() {
        let mut hud = CombatHud::new();
        hud.add_status(StatusEffect::new(StatusEffectType::Poison, 5.0).with_stacks(1, 3));
        hud.add_status(StatusEffect::new(StatusEffectType::Poison, 5.0));
        assert_eq!(hud.status_effects.len(), 1);
        assert_eq!(hud.status_effects[0].stacks, 2);
    }

    #[test]
    fn test_combat_hud_remove_status() {
        let mut hud = CombatHud::new();
        hud.add_status(StatusEffect::new(StatusEffectType::Poison, 5.0));
        hud.add_status(StatusEffect::new(StatusEffectType::Burning, 5.0));
        assert_eq!(hud.status_effects.len(), 2);
        hud.remove_status(StatusEffectType::Poison);
        assert_eq!(hud.status_effects.len(), 1);
    }

    #[test]
    fn test_combat_hud_clear_status() {
        let mut hud = CombatHud::new();
        hud.add_status(StatusEffect::new(StatusEffectType::Poison, 5.0));
        hud.add_status(StatusEffect::new(StatusEffectType::Burning, 5.0));
        hud.clear_status();
        assert!(hud.status_effects.is_empty());
    }

    #[test]
    fn test_combat_hud_damage_flash() {
        let mut hud = CombatHud::new();
        hud.damage_flash(0.5);
        assert!(hud.screen_flash.is_active());
    }

    #[test]
    fn test_combat_hud_update() {
        let mut hud = CombatHud::new();
        hud.hit(50.0);
        hud.add_damage(50.0, 0.0, 0.0, DamageType::Physical, false);
        hud.add_status(StatusEffect::new(StatusEffectType::Poison, 0.5));
        hud.damage_flash(0.5);

        hud.update(1.0);

        // Effects should have updated/expired
        assert!(hud.screen_flash.intensity < 0.5);
    }

    #[test]
    fn test_status_effect_serialization() {
        let effect = StatusEffect::new(StatusEffectType::Poison, 5.0).with_stacks(2, 5);
        let json = serde_json::to_string(&effect).unwrap();
        let loaded: StatusEffect = serde_json::from_str(&json).unwrap();
        assert_eq!(effect.effect_type, loaded.effect_type);
        assert_eq!(effect.stacks, loaded.stacks);
    }

    #[test]
    fn test_combo_counter_serialization() {
        let mut combo = ComboCounter::new();
        combo.hit(100.0);
        combo.hit(50.0);
        let json = serde_json::to_string(&combo).unwrap();
        let loaded: ComboCounter = serde_json::from_str(&json).unwrap();
        assert_eq!(combo.count, loaded.count);
        assert_eq!(combo.damage, loaded.damage);
    }

    #[test]
    fn test_combat_hud_config_serialization() {
        let config = CombatHudConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: CombatHudConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.show_combo, loaded.show_combo);
    }

    #[test]
    fn test_damage_type_serialization() {
        for damage_type in &[
            DamageType::Physical,
            DamageType::Fire,
            DamageType::Ice,
            DamageType::Lightning,
            DamageType::Poison,
            DamageType::True,
            DamageType::Healing,
        ] {
            let json = serde_json::to_string(damage_type).unwrap();
            let loaded: DamageType = serde_json::from_str(&json).unwrap();
            assert_eq!(*damage_type, loaded);
        }
    }

    #[test]
    fn test_damage_indicator_alpha() {
        let mut indicator = DamageIndicator::new(50.0, 0.0, 0.0, DamageType::Physical);
        assert_eq!(indicator.alpha(), 1.0);
        indicator.lifetime = indicator.max_lifetime / 2.0;
        assert!((indicator.alpha() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_combo_counter_timer_percent() {
        let mut combo = ComboCounter::new().with_timeout(2.0);
        combo.hit(50.0);
        assert_eq!(combo.timer_percent(), 1.0);
        combo.timer = 1.0;
        assert_eq!(combo.timer_percent(), 0.5);
    }

    #[test]
    fn test_status_effect_with_source() {
        let effect =
            StatusEffect::new(StatusEffectType::Poison, 5.0).with_source("Venomous Spider");
        assert_eq!(effect.source, Some("Venomous Spider".to_string()));
    }
}
