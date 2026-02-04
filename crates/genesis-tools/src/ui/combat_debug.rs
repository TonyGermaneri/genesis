//! Combat debug UI components.
//!
//! Provides combat debug functionality including:
//! - Hitbox visualization
//! - Damage log with timestamps
//! - Frame data display
//! - I-frame indicator

use egui::{Color32, Pos2, Rect, Ui, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Hitbox type for visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HitboxType {
    /// Hurtbox (can be hit).
    Hurtbox,
    /// Hitbox (deals damage).
    Hitbox,
    /// Pushbox (collision).
    Pushbox,
    /// Trigger zone.
    Trigger,
    /// Grab/throw box.
    Grabbox,
    /// Projectile hitbox.
    Projectile,
    /// Environmental hazard.
    Hazard,
}

impl HitboxType {
    /// Get all hitbox types.
    pub fn all() -> &'static [HitboxType] {
        &[
            HitboxType::Hurtbox,
            HitboxType::Hitbox,
            HitboxType::Pushbox,
            HitboxType::Trigger,
            HitboxType::Grabbox,
            HitboxType::Projectile,
            HitboxType::Hazard,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            HitboxType::Hurtbox => "Hurtbox",
            HitboxType::Hitbox => "Hitbox",
            HitboxType::Pushbox => "Pushbox",
            HitboxType::Trigger => "Trigger",
            HitboxType::Grabbox => "Grabbox",
            HitboxType::Projectile => "Projectile",
            HitboxType::Hazard => "Hazard",
        }
    }

    /// Get color for visualization.
    pub fn color(&self) -> Color32 {
        match self {
            HitboxType::Hurtbox => Color32::from_rgba_unmultiplied(100, 200, 100, 100),
            HitboxType::Hitbox => Color32::from_rgba_unmultiplied(200, 100, 100, 100),
            HitboxType::Pushbox => Color32::from_rgba_unmultiplied(100, 100, 200, 100),
            HitboxType::Trigger => Color32::from_rgba_unmultiplied(200, 200, 100, 100),
            HitboxType::Grabbox => Color32::from_rgba_unmultiplied(200, 100, 200, 100),
            HitboxType::Projectile => Color32::from_rgba_unmultiplied(255, 150, 50, 100),
            HitboxType::Hazard => Color32::from_rgba_unmultiplied(255, 50, 50, 100),
        }
    }

    /// Get border color for visualization.
    pub fn border_color(&self) -> Color32 {
        match self {
            HitboxType::Hurtbox => Color32::from_rgb(100, 200, 100),
            HitboxType::Hitbox => Color32::from_rgb(200, 100, 100),
            HitboxType::Pushbox => Color32::from_rgb(100, 100, 200),
            HitboxType::Trigger => Color32::from_rgb(200, 200, 100),
            HitboxType::Grabbox => Color32::from_rgb(200, 100, 200),
            HitboxType::Projectile => Color32::from_rgb(255, 150, 50),
            HitboxType::Hazard => Color32::from_rgb(255, 50, 50),
        }
    }
}

/// Hitbox shape types.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HitboxShape {
    /// Rectangle/AABB.
    Rectangle {
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
    /// Circle.
    Circle {
        /// Radius.
        radius: f32,
    },
    /// Capsule (vertical).
    Capsule {
        /// Radius.
        radius: f32,
        /// Height (excluding caps).
        height: f32,
    },
}

impl HitboxShape {
    /// Create a rectangle.
    pub fn rect(width: f32, height: f32) -> Self {
        Self::Rectangle { width, height }
    }

    /// Create a circle.
    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    /// Create a capsule.
    pub fn capsule(radius: f32, height: f32) -> Self {
        Self::Capsule { radius, height }
    }

    /// Get bounding width.
    pub fn width(&self) -> f32 {
        match self {
            HitboxShape::Rectangle { width, .. } => *width,
            HitboxShape::Circle { radius } | HitboxShape::Capsule { radius, .. } => radius * 2.0,
        }
    }

    /// Get bounding height.
    pub fn height(&self) -> f32 {
        match self {
            HitboxShape::Rectangle { height, .. } => *height,
            HitboxShape::Circle { radius } => radius * 2.0,
            HitboxShape::Capsule { radius, height } => height + radius * 2.0,
        }
    }
}

/// A hitbox for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hitbox {
    /// Hitbox ID.
    pub id: String,
    /// Owner entity ID.
    pub owner: String,
    /// Hitbox type.
    pub hitbox_type: HitboxType,
    /// Shape.
    pub shape: HitboxShape,
    /// Position (center).
    pub position: [f32; 2],
    /// Whether hitbox is active.
    pub active: bool,
    /// Frame when hitbox becomes active.
    pub active_frame: u32,
    /// Frame when hitbox becomes inactive.
    pub inactive_frame: u32,
    /// Damage (if hitbox).
    pub damage: Option<f32>,
}

impl Hitbox {
    /// Create a new hitbox.
    pub fn new(
        id: impl Into<String>,
        owner: impl Into<String>,
        hitbox_type: HitboxType,
        shape: HitboxShape,
    ) -> Self {
        Self {
            id: id.into(),
            owner: owner.into(),
            hitbox_type,
            shape,
            position: [0.0, 0.0],
            active: true,
            active_frame: 0,
            inactive_frame: u32::MAX,
            damage: None,
        }
    }

    /// Set position.
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    /// Set frame range.
    pub fn with_frames(mut self, start: u32, end: u32) -> Self {
        self.active_frame = start;
        self.inactive_frame = end;
        self
    }

    /// Set damage.
    pub fn with_damage(mut self, damage: f32) -> Self {
        self.damage = Some(damage);
        self
    }

    /// Get position as Pos2.
    pub fn pos(&self) -> Pos2 {
        Pos2::new(self.position[0], self.position[1])
    }

    /// Get bounding rect.
    pub fn bounds(&self) -> Rect {
        let half_w = self.shape.width() / 2.0;
        let half_h = self.shape.height() / 2.0;
        Rect::from_center_size(self.pos(), Vec2::new(half_w * 2.0, half_h * 2.0))
    }

    /// Check if active on given frame.
    pub fn is_active_on_frame(&self, frame: u32) -> bool {
        self.active && frame >= self.active_frame && frame < self.inactive_frame
    }
}

/// Damage log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageLogEntry {
    /// Timestamp (game time).
    pub timestamp: f32,
    /// Frame number.
    pub frame: u32,
    /// Source entity.
    pub source: String,
    /// Target entity.
    pub target: String,
    /// Damage amount.
    pub damage: f32,
    /// Damage type.
    pub damage_type: String,
    /// Was critical hit.
    pub is_critical: bool,
    /// Was blocked.
    pub was_blocked: bool,
    /// Damage after mitigation.
    pub final_damage: f32,
    /// Mitigation amount.
    pub mitigation: f32,
}

impl DamageLogEntry {
    /// Create a new damage log entry.
    pub fn new(
        timestamp: f32,
        frame: u32,
        source: impl Into<String>,
        target: impl Into<String>,
        damage: f32,
    ) -> Self {
        Self {
            timestamp,
            frame,
            source: source.into(),
            target: target.into(),
            damage,
            damage_type: "Physical".to_string(),
            is_critical: false,
            was_blocked: false,
            final_damage: damage,
            mitigation: 0.0,
        }
    }

    /// Set damage type.
    pub fn with_type(mut self, damage_type: impl Into<String>) -> Self {
        self.damage_type = damage_type.into();
        self
    }

    /// Set as critical.
    pub fn with_critical(mut self, is_critical: bool) -> Self {
        self.is_critical = is_critical;
        self
    }

    /// Set as blocked.
    pub fn with_blocked(mut self, was_blocked: bool) -> Self {
        self.was_blocked = was_blocked;
        self
    }

    /// Set mitigation.
    pub fn with_mitigation(mut self, mitigation: f32) -> Self {
        self.mitigation = mitigation;
        self.final_damage = (self.damage - mitigation).max(0.0);
        self
    }

    /// Format timestamp as string.
    pub fn timestamp_str(&self) -> String {
        let minutes = (self.timestamp / 60.0) as u32;
        let seconds = (self.timestamp % 60.0) as u32;
        let millis = ((self.timestamp * 1000.0) % 1000.0) as u32;
        format!("{minutes:02}:{seconds:02}.{millis:03}")
    }

    /// Get summary text.
    pub fn summary(&self) -> String {
        let crit_marker = if self.is_critical { " CRIT" } else { "" };
        let block_marker = if self.was_blocked { " BLOCKED" } else { "" };
        format!(
            "{} -> {}: {:.0} ({}) {crit_marker}{block_marker}",
            self.source, self.target, self.final_damage, self.damage_type
        )
    }
}

/// Frame data for an attack/move.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    /// Move name.
    pub name: String,
    /// Startup frames.
    pub startup: u32,
    /// Active frames.
    pub active: u32,
    /// Recovery frames.
    pub recovery: u32,
    /// Frame advantage on hit.
    pub on_hit: i32,
    /// Frame advantage on block.
    pub on_block: i32,
    /// Cancel frames (can cancel into other moves).
    pub cancel_frames: Vec<u32>,
    /// Whether move is invincible during startup.
    pub invincible_startup: bool,
    /// Invincibility frame range.
    pub invincible_frames: Option<(u32, u32)>,
}

impl FrameData {
    /// Create new frame data.
    pub fn new(name: impl Into<String>, startup: u32, active: u32, recovery: u32) -> Self {
        Self {
            name: name.into(),
            startup,
            active,
            recovery,
            on_hit: 0,
            on_block: 0,
            cancel_frames: Vec::new(),
            invincible_startup: false,
            invincible_frames: None,
        }
    }

    /// Set frame advantage.
    pub fn with_advantage(mut self, on_hit: i32, on_block: i32) -> Self {
        self.on_hit = on_hit;
        self.on_block = on_block;
        self
    }

    /// Add cancel frame.
    pub fn with_cancel(mut self, frame: u32) -> Self {
        self.cancel_frames.push(frame);
        self
    }

    /// Set invincibility frames.
    pub fn with_invincibility(mut self, start: u32, end: u32) -> Self {
        self.invincible_frames = Some((start, end));
        self
    }

    /// Set invincible startup.
    pub fn with_invincible_startup(mut self, invincible: bool) -> Self {
        self.invincible_startup = invincible;
        self
    }

    /// Get total frames.
    pub fn total_frames(&self) -> u32 {
        self.startup + self.active + self.recovery
    }

    /// Get first active frame.
    pub fn first_active(&self) -> u32 {
        self.startup + 1
    }

    /// Get last active frame.
    pub fn last_active(&self) -> u32 {
        self.startup + self.active
    }

    /// Check if frame is in startup.
    pub fn is_startup(&self, frame: u32) -> bool {
        frame <= self.startup
    }

    /// Check if frame is active.
    pub fn is_active(&self, frame: u32) -> bool {
        frame > self.startup && frame <= self.startup + self.active
    }

    /// Check if frame is in recovery.
    pub fn is_recovery(&self, frame: u32) -> bool {
        frame > self.startup + self.active && frame <= self.total_frames()
    }

    /// Check if frame has invincibility.
    pub fn is_invincible(&self, frame: u32) -> bool {
        if self.invincible_startup && frame <= self.startup {
            return true;
        }
        if let Some((start, end)) = self.invincible_frames {
            return frame >= start && frame <= end;
        }
        false
    }

    /// Get phase name for frame.
    pub fn phase_name(&self, frame: u32) -> &'static str {
        if frame == 0 {
            "Idle"
        } else if self.is_startup(frame) {
            "Startup"
        } else if self.is_active(frame) {
            "Active"
        } else if self.is_recovery(frame) {
            "Recovery"
        } else {
            "Complete"
        }
    }
}

/// I-frame state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IFrameState {
    /// No invincibility.
    None,
    /// Dodge i-frames.
    Dodge,
    /// Parry i-frames.
    Parry,
    /// Attack i-frames.
    Attack,
    /// Skill i-frames.
    Skill,
    /// Item i-frames.
    Item,
}

impl IFrameState {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            IFrameState::None => "None",
            IFrameState::Dodge => "Dodge",
            IFrameState::Parry => "Parry",
            IFrameState::Attack => "Attack",
            IFrameState::Skill => "Skill",
            IFrameState::Item => "Item",
        }
    }

    /// Get color.
    pub fn color(&self) -> Color32 {
        match self {
            IFrameState::None => Color32::TRANSPARENT,
            IFrameState::Dodge => Color32::from_rgb(100, 200, 255),
            IFrameState::Parry => Color32::from_rgb(255, 255, 100),
            IFrameState::Attack => Color32::from_rgb(255, 100, 100),
            IFrameState::Skill => Color32::from_rgb(200, 100, 255),
            IFrameState::Item => Color32::from_rgb(100, 255, 100),
        }
    }
}

/// I-frame indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IFrameIndicator {
    /// Current state.
    pub state: IFrameState,
    /// Frames remaining.
    pub frames_remaining: u32,
    /// Total frames.
    pub total_frames: u32,
    /// Whether indicator is visible.
    pub visible: bool,
}

impl Default for IFrameIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl IFrameIndicator {
    /// Create new indicator.
    pub fn new() -> Self {
        Self {
            state: IFrameState::None,
            frames_remaining: 0,
            total_frames: 0,
            visible: true,
        }
    }

    /// Trigger i-frames.
    pub fn trigger(&mut self, state: IFrameState, frames: u32) {
        self.state = state;
        self.frames_remaining = frames;
        self.total_frames = frames;
    }

    /// Clear i-frames.
    pub fn clear(&mut self) {
        self.state = IFrameState::None;
        self.frames_remaining = 0;
        self.total_frames = 0;
    }

    /// Tick one frame.
    pub fn tick(&mut self) {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
            if self.frames_remaining == 0 {
                self.clear();
            }
        }
    }

    /// Check if active.
    pub fn is_active(&self) -> bool {
        self.state != IFrameState::None && self.frames_remaining > 0
    }

    /// Get progress (0.0 - 1.0).
    pub fn progress(&self) -> f32 {
        if self.total_frames > 0 {
            self.frames_remaining as f32 / self.total_frames as f32
        } else {
            0.0
        }
    }
}

/// Combat debug configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatDebugConfig {
    /// Show hitboxes.
    pub show_hitboxes: bool,
    /// Show hurtboxes.
    pub show_hurtboxes: bool,
    /// Show pushboxes.
    pub show_pushboxes: bool,
    /// Show damage log.
    pub show_damage_log: bool,
    /// Show frame data.
    pub show_frame_data: bool,
    /// Show i-frame indicator.
    pub show_iframe_indicator: bool,
    /// Max damage log entries.
    pub max_log_entries: usize,
    /// Hitbox fill alpha.
    pub hitbox_alpha: u8,
    /// Show hitbox labels.
    pub show_hitbox_labels: bool,
}

impl Default for CombatDebugConfig {
    fn default() -> Self {
        Self {
            show_hitboxes: true,
            show_hurtboxes: true,
            show_pushboxes: false,
            show_damage_log: true,
            show_frame_data: true,
            show_iframe_indicator: true,
            max_log_entries: 50,
            hitbox_alpha: 100,
            show_hitbox_labels: true,
        }
    }
}

/// Combat debug overlay widget.
#[derive(Debug)]
pub struct CombatDebug {
    /// Configuration.
    pub config: CombatDebugConfig,
    /// Active hitboxes.
    pub hitboxes: Vec<Hitbox>,
    /// Damage log.
    pub damage_log: VecDeque<DamageLogEntry>,
    /// Current frame data (if in attack).
    pub current_frame_data: Option<FrameData>,
    /// Current frame in animation.
    pub current_frame: u32,
    /// I-frame indicator.
    pub iframe_indicator: IFrameIndicator,
    /// Whether debug is visible.
    pub visible: bool,
    /// Game time.
    pub game_time: f32,
}

impl Default for CombatDebug {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatDebug {
    /// Create new combat debug overlay.
    pub fn new() -> Self {
        Self {
            config: CombatDebugConfig::default(),
            hitboxes: Vec::new(),
            damage_log: VecDeque::new(),
            current_frame_data: None,
            current_frame: 0,
            iframe_indicator: IFrameIndicator::new(),
            visible: true,
            game_time: 0.0,
        }
    }

    /// Create with config.
    pub fn with_config(config: CombatDebugConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Add a hitbox.
    pub fn add_hitbox(&mut self, hitbox: Hitbox) {
        self.hitboxes.push(hitbox);
    }

    /// Remove hitbox by ID.
    pub fn remove_hitbox(&mut self, id: &str) {
        self.hitboxes.retain(|h| h.id != id);
    }

    /// Clear all hitboxes.
    pub fn clear_hitboxes(&mut self) {
        self.hitboxes.clear();
    }

    /// Log damage.
    pub fn log_damage(&mut self, entry: DamageLogEntry) {
        self.damage_log.push_back(entry);
        while self.damage_log.len() > self.config.max_log_entries {
            self.damage_log.pop_front();
        }
    }

    /// Clear damage log.
    pub fn clear_log(&mut self) {
        self.damage_log.clear();
    }

    /// Set current attack frame data.
    pub fn set_frame_data(&mut self, frame_data: FrameData) {
        self.current_frame_data = Some(frame_data);
        self.current_frame = 0;
    }

    /// Clear frame data.
    pub fn clear_frame_data(&mut self) {
        self.current_frame_data = None;
        self.current_frame = 0;
    }

    /// Trigger i-frames.
    pub fn trigger_iframes(&mut self, state: IFrameState, frames: u32) {
        self.iframe_indicator.trigger(state, frames);
    }

    /// Advance one frame.
    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
        self.iframe_indicator.tick();

        // Check if attack is complete
        if let Some(fd) = &self.current_frame_data {
            if self.current_frame > fd.total_frames() {
                self.clear_frame_data();
            }
        }
    }

    /// Update game time.
    pub fn update(&mut self, dt: f32) {
        self.game_time += dt;
    }

    /// Render the debug overlay.
    pub fn show(&self, ui: &mut Ui, world_to_screen: impl Fn(Pos2) -> Pos2) {
        if !self.visible {
            return;
        }

        let painter = ui.painter();

        // Draw hitboxes
        for hitbox in &self.hitboxes {
            if !hitbox.active {
                continue;
            }

            let should_draw = match hitbox.hitbox_type {
                HitboxType::Hurtbox => self.config.show_hurtboxes,
                HitboxType::Pushbox => self.config.show_pushboxes,
                _ => self.config.show_hitboxes,
            };

            if should_draw {
                let screen_pos = world_to_screen(hitbox.pos());
                let mut color = hitbox.hitbox_type.color();
                color = Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    self.config.hitbox_alpha,
                );

                match hitbox.shape {
                    HitboxShape::Rectangle { width, height } => {
                        let rect = Rect::from_center_size(screen_pos, Vec2::new(width, height));
                        painter.rect_filled(rect, 0.0, color);
                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(2.0, hitbox.hitbox_type.border_color()),
                        );
                    },
                    HitboxShape::Circle { radius } => {
                        painter.circle_filled(screen_pos, radius, color);
                        painter.circle_stroke(
                            screen_pos,
                            radius,
                            egui::Stroke::new(2.0, hitbox.hitbox_type.border_color()),
                        );
                    },
                    HitboxShape::Capsule { radius, height } => {
                        // Draw as rounded rectangle approximation
                        let rect = Rect::from_center_size(
                            screen_pos,
                            Vec2::new(radius * 2.0, height + radius * 2.0),
                        );
                        painter.rect_filled(rect, radius, color);
                        painter.rect_stroke(
                            rect,
                            radius,
                            egui::Stroke::new(2.0, hitbox.hitbox_type.border_color()),
                        );
                    },
                }

                // Label
                if self.config.show_hitbox_labels {
                    let label = if let Some(dmg) = hitbox.damage {
                        format!("{} ({dmg:.0})", hitbox.hitbox_type.display_name())
                    } else {
                        hitbox.hitbox_type.display_name().to_string()
                    };
                    painter.text(
                        Pos2::new(
                            screen_pos.x,
                            screen_pos.y - hitbox.shape.height() / 2.0 - 10.0,
                        ),
                        egui::Align2::CENTER_BOTTOM,
                        label,
                        egui::FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
            }
        }
    }

    /// Show damage log panel.
    pub fn show_damage_log(&self, ui: &mut Ui) {
        if !self.config.show_damage_log {
            return;
        }

        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Damage Log").strong());
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for entry in self.damage_log.iter().rev() {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(entry.timestamp_str())
                                    .small()
                                    .color(Color32::from_gray(150)),
                            );

                            let mut summary_text = egui::RichText::new(entry.summary());
                            if entry.is_critical {
                                summary_text = summary_text.color(Color32::from_rgb(255, 200, 100));
                            } else if entry.was_blocked {
                                summary_text = summary_text.color(Color32::from_gray(150));
                            }
                            ui.label(summary_text);
                        });
                    }
                });
        });
    }

    /// Show frame data panel.
    pub fn show_frame_data(&self, ui: &mut Ui) {
        if !self.config.show_frame_data {
            return;
        }

        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Frame Data").strong());
            ui.separator();

            if let Some(fd) = &self.current_frame_data {
                ui.label(egui::RichText::new(&fd.name).strong());

                // Timeline bar
                let total = fd.total_frames() as f32;
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(200.0, 16.0), egui::Sense::hover());
                let painter = ui.painter();

                // Background
                painter.rect_filled(rect, 2.0, Color32::from_gray(40));

                // Startup (yellow)
                let startup_width = (fd.startup as f32 / total) * rect.width();
                let startup_rect =
                    Rect::from_min_size(rect.min, Vec2::new(startup_width, rect.height()));
                painter.rect_filled(startup_rect, 0.0, Color32::from_rgb(200, 200, 100));

                // Active (red)
                let active_width = (fd.active as f32 / total) * rect.width();
                let active_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + startup_width, rect.min.y),
                    Vec2::new(active_width, rect.height()),
                );
                painter.rect_filled(active_rect, 0.0, Color32::from_rgb(200, 100, 100));

                // Recovery (blue)
                let recovery_width = (fd.recovery as f32 / total) * rect.width();
                let recovery_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + startup_width + active_width, rect.min.y),
                    Vec2::new(recovery_width, rect.height()),
                );
                painter.rect_filled(recovery_rect, 0.0, Color32::from_rgb(100, 100, 200));

                // Current frame marker
                let frame_x = rect.min.x + (self.current_frame as f32 / total) * rect.width();
                painter.line_segment(
                    [
                        Pos2::new(frame_x, rect.min.y),
                        Pos2::new(frame_x, rect.max.y),
                    ],
                    egui::Stroke::new(2.0, Color32::WHITE),
                );

                // Invincibility frames overlay
                if let Some((start, end)) = fd.invincible_frames {
                    let inv_start = (start as f32 / total) * rect.width();
                    let inv_end = (end as f32 / total) * rect.width();
                    let inv_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + inv_start, rect.min.y),
                        Vec2::new(inv_end - inv_start, rect.height()),
                    );
                    painter.rect_filled(
                        inv_rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(100, 255, 255, 100),
                    );
                }

                // Frame info
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Frame: {}/{}",
                        self.current_frame,
                        fd.total_frames()
                    ));
                    ui.label(format!("Phase: {}", fd.phase_name(self.current_frame)));
                });

                // Stats
                ui.label(format!(
                    "Startup: {} | Active: {} | Recovery: {}",
                    fd.startup, fd.active, fd.recovery
                ));

                let on_hit_color = if fd.on_hit >= 0 {
                    Color32::from_rgb(100, 200, 100)
                } else {
                    Color32::from_rgb(200, 100, 100)
                };
                let on_block_color = if fd.on_block >= 0 {
                    Color32::from_rgb(100, 200, 100)
                } else {
                    Color32::from_rgb(200, 100, 100)
                };

                ui.horizontal(|ui| {
                    ui.label("On Hit:");
                    ui.label(egui::RichText::new(format!("{:+}", fd.on_hit)).color(on_hit_color));
                    ui.label("On Block:");
                    ui.label(
                        egui::RichText::new(format!("{:+}", fd.on_block)).color(on_block_color),
                    );
                });

                // Invincibility indicator
                if fd.is_invincible(self.current_frame) {
                    ui.label(
                        egui::RichText::new("INVINCIBLE")
                            .color(Color32::from_rgb(100, 255, 255))
                            .strong(),
                    );
                }
            } else {
                ui.label("No active attack");
            }
        });
    }

    /// Show i-frame indicator.
    pub fn show_iframe_indicator(&self, ui: &mut Ui) {
        if !self.config.show_iframe_indicator || !self.iframe_indicator.is_active() {
            return;
        }

        ui.horizontal(|ui| {
            let state = &self.iframe_indicator;
            let color = state.state.color();

            // Progress bar
            let (rect, _) = ui.allocate_exact_size(Vec2::new(100.0, 10.0), egui::Sense::hover());
            let painter = ui.painter();

            painter.rect_filled(rect, 4.0, Color32::from_gray(40));

            let fill_width = rect.width() * state.progress();
            let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
            painter.rect_filled(fill_rect, 4.0, color);

            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, color));

            ui.label(
                egui::RichText::new(format!(
                    "{} ({}f)",
                    state.state.display_name(),
                    state.frames_remaining
                ))
                .color(color),
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hitbox_type_all() {
        assert_eq!(HitboxType::all().len(), 7);
    }

    #[test]
    fn test_hitbox_type_display_name() {
        assert_eq!(HitboxType::Hitbox.display_name(), "Hitbox");
        assert_eq!(HitboxType::Hurtbox.display_name(), "Hurtbox");
    }

    #[test]
    fn test_hitbox_type_color() {
        assert_ne!(HitboxType::Hitbox.color(), HitboxType::Hurtbox.color());
    }

    #[test]
    fn test_hitbox_shape_rect() {
        let shape = HitboxShape::rect(100.0, 50.0);
        assert_eq!(shape.width(), 100.0);
        assert_eq!(shape.height(), 50.0);
    }

    #[test]
    fn test_hitbox_shape_circle() {
        let shape = HitboxShape::circle(25.0);
        assert_eq!(shape.width(), 50.0);
        assert_eq!(shape.height(), 50.0);
    }

    #[test]
    fn test_hitbox_shape_capsule() {
        let shape = HitboxShape::capsule(10.0, 30.0);
        assert_eq!(shape.width(), 20.0);
        assert_eq!(shape.height(), 50.0);
    }

    #[test]
    fn test_hitbox_new() {
        let hitbox = Hitbox::new(
            "hitbox_1",
            "player",
            HitboxType::Hitbox,
            HitboxShape::rect(50.0, 30.0),
        );
        assert_eq!(hitbox.id, "hitbox_1");
        assert_eq!(hitbox.owner, "player");
        assert!(hitbox.active);
    }

    #[test]
    fn test_hitbox_with_position() {
        let hitbox = Hitbox::new("h1", "p1", HitboxType::Hitbox, HitboxShape::circle(10.0))
            .with_position(100.0, 200.0);
        assert_eq!(hitbox.position, [100.0, 200.0]);
    }

    #[test]
    fn test_hitbox_with_frames() {
        let hitbox = Hitbox::new("h1", "p1", HitboxType::Hitbox, HitboxShape::circle(10.0))
            .with_frames(5, 10);
        assert_eq!(hitbox.active_frame, 5);
        assert_eq!(hitbox.inactive_frame, 10);
    }

    #[test]
    fn test_hitbox_is_active_on_frame() {
        let hitbox = Hitbox::new("h1", "p1", HitboxType::Hitbox, HitboxShape::circle(10.0))
            .with_frames(5, 10);

        assert!(!hitbox.is_active_on_frame(4));
        assert!(hitbox.is_active_on_frame(5));
        assert!(hitbox.is_active_on_frame(9));
        assert!(!hitbox.is_active_on_frame(10));
    }

    #[test]
    fn test_damage_log_entry_new() {
        let entry = DamageLogEntry::new(10.5, 630, "player", "enemy", 50.0);
        assert_eq!(entry.source, "player");
        assert_eq!(entry.target, "enemy");
        assert_eq!(entry.damage, 50.0);
    }

    #[test]
    fn test_damage_log_entry_timestamp_str() {
        let entry = DamageLogEntry::new(65.5, 0, "a", "b", 10.0);
        assert_eq!(entry.timestamp_str(), "01:05.500");
    }

    #[test]
    fn test_damage_log_entry_with_mitigation() {
        let entry = DamageLogEntry::new(0.0, 0, "a", "b", 100.0).with_mitigation(30.0);
        assert_eq!(entry.mitigation, 30.0);
        assert_eq!(entry.final_damage, 70.0);
    }

    #[test]
    fn test_frame_data_new() {
        let fd = FrameData::new("Punch", 5, 3, 10);
        assert_eq!(fd.startup, 5);
        assert_eq!(fd.active, 3);
        assert_eq!(fd.recovery, 10);
        assert_eq!(fd.total_frames(), 18);
    }

    #[test]
    fn test_frame_data_phases() {
        let fd = FrameData::new("Kick", 5, 3, 10);

        // Frame 0 is idle
        assert_eq!(fd.phase_name(0), "Idle");

        // Frames 1-5 are startup
        assert!(fd.is_startup(5));
        assert!(!fd.is_active(5));

        // Frames 6-8 are active
        assert!(fd.is_active(6));
        assert!(fd.is_active(8));

        // Frames 9-18 are recovery
        assert!(fd.is_recovery(9));
        assert!(fd.is_recovery(18));

        // Frame 19+ is complete
        assert_eq!(fd.phase_name(19), "Complete");
    }

    #[test]
    fn test_frame_data_invincibility() {
        let fd = FrameData::new("Dodge", 3, 1, 10).with_invincibility(1, 5);

        assert!(fd.is_invincible(1));
        assert!(fd.is_invincible(5));
        assert!(!fd.is_invincible(6));
    }

    #[test]
    fn test_frame_data_invincible_startup() {
        let fd = FrameData::new("DP", 5, 3, 10).with_invincible_startup(true);

        assert!(fd.is_invincible(1));
        assert!(fd.is_invincible(5));
        assert!(!fd.is_invincible(6));
    }

    #[test]
    fn test_iframe_state_color() {
        assert_ne!(IFrameState::Dodge.color(), IFrameState::Parry.color());
    }

    #[test]
    fn test_iframe_indicator_new() {
        let indicator = IFrameIndicator::new();
        assert_eq!(indicator.state, IFrameState::None);
        assert!(!indicator.is_active());
    }

    #[test]
    fn test_iframe_indicator_trigger() {
        let mut indicator = IFrameIndicator::new();
        indicator.trigger(IFrameState::Dodge, 10);
        assert!(indicator.is_active());
        assert_eq!(indicator.frames_remaining, 10);
    }

    #[test]
    fn test_iframe_indicator_tick() {
        let mut indicator = IFrameIndicator::new();
        indicator.trigger(IFrameState::Dodge, 3);

        indicator.tick();
        assert_eq!(indicator.frames_remaining, 2);

        indicator.tick();
        indicator.tick();
        assert!(!indicator.is_active());
    }

    #[test]
    fn test_iframe_indicator_progress() {
        let mut indicator = IFrameIndicator::new();
        indicator.trigger(IFrameState::Dodge, 10);
        assert_eq!(indicator.progress(), 1.0);

        indicator.tick();
        indicator.tick();
        indicator.tick();
        indicator.tick();
        indicator.tick();
        assert_eq!(indicator.progress(), 0.5);
    }

    #[test]
    fn test_combat_debug_config_default() {
        let config = CombatDebugConfig::default();
        assert!(config.show_hitboxes);
        assert!(config.show_damage_log);
    }

    #[test]
    fn test_combat_debug_new() {
        let debug = CombatDebug::new();
        assert!(debug.visible);
        assert!(debug.hitboxes.is_empty());
        assert!(debug.damage_log.is_empty());
    }

    #[test]
    fn test_combat_debug_add_hitbox() {
        let mut debug = CombatDebug::new();
        let hitbox = Hitbox::new("h1", "p1", HitboxType::Hitbox, HitboxShape::circle(10.0));
        debug.add_hitbox(hitbox);
        assert_eq!(debug.hitboxes.len(), 1);
    }

    #[test]
    fn test_combat_debug_remove_hitbox() {
        let mut debug = CombatDebug::new();
        debug.add_hitbox(Hitbox::new(
            "h1",
            "p1",
            HitboxType::Hitbox,
            HitboxShape::circle(10.0),
        ));
        debug.add_hitbox(Hitbox::new(
            "h2",
            "p1",
            HitboxType::Hurtbox,
            HitboxShape::circle(20.0),
        ));

        debug.remove_hitbox("h1");
        assert_eq!(debug.hitboxes.len(), 1);
        assert_eq!(debug.hitboxes[0].id, "h2");
    }

    #[test]
    fn test_combat_debug_log_damage() {
        let mut debug = CombatDebug::new();
        debug.config.max_log_entries = 3;

        for i in 0..5 {
            debug.log_damage(DamageLogEntry::new(i as f32, i, "a", "b", 10.0));
        }

        assert_eq!(debug.damage_log.len(), 3);
    }

    #[test]
    fn test_combat_debug_frame_data() {
        let mut debug = CombatDebug::new();
        let fd = FrameData::new("Punch", 5, 3, 10);
        debug.set_frame_data(fd);

        assert!(debug.current_frame_data.is_some());
        assert_eq!(debug.current_frame, 0);

        // Advance past total frames
        for _ in 0..20 {
            debug.advance_frame();
        }

        assert!(debug.current_frame_data.is_none());
    }

    #[test]
    fn test_combat_debug_iframes() {
        let mut debug = CombatDebug::new();
        debug.trigger_iframes(IFrameState::Dodge, 5);

        assert!(debug.iframe_indicator.is_active());

        for _ in 0..5 {
            debug.advance_frame();
        }

        assert!(!debug.iframe_indicator.is_active());
    }

    #[test]
    fn test_hitbox_serialization() {
        let hitbox = Hitbox::new(
            "h1",
            "p1",
            HitboxType::Hitbox,
            HitboxShape::rect(50.0, 30.0),
        )
        .with_position(100.0, 200.0)
        .with_damage(25.0);

        let json = serde_json::to_string(&hitbox).unwrap();
        let loaded: Hitbox = serde_json::from_str(&json).unwrap();
        assert_eq!(hitbox.id, loaded.id);
        assert_eq!(hitbox.damage, loaded.damage);
    }

    #[test]
    fn test_frame_data_serialization() {
        let fd = FrameData::new("Punch", 5, 3, 10)
            .with_advantage(5, -2)
            .with_invincibility(1, 3);

        let json = serde_json::to_string(&fd).unwrap();
        let loaded: FrameData = serde_json::from_str(&json).unwrap();
        assert_eq!(fd.name, loaded.name);
        assert_eq!(fd.on_hit, loaded.on_hit);
    }

    #[test]
    fn test_damage_log_entry_serialization() {
        let entry = DamageLogEntry::new(10.5, 630, "player", "enemy", 50.0)
            .with_critical(true)
            .with_type("Fire");

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: DamageLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.source, loaded.source);
        assert_eq!(entry.is_critical, loaded.is_critical);
    }

    #[test]
    fn test_combat_debug_config_serialization() {
        let config = CombatDebugConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: CombatDebugConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.show_hitboxes, loaded.show_hitboxes);
    }

    #[test]
    fn test_hitbox_bounds() {
        let hitbox = Hitbox::new(
            "h1",
            "p1",
            HitboxType::Hitbox,
            HitboxShape::rect(100.0, 50.0),
        )
        .with_position(50.0, 25.0);

        let bounds = hitbox.bounds();
        assert_eq!(bounds.min, Pos2::new(0.0, 0.0));
        assert_eq!(bounds.max, Pos2::new(100.0, 50.0));
    }

    #[test]
    fn test_damage_log_entry_summary() {
        let entry = DamageLogEntry::new(0.0, 0, "Player", "Goblin", 50.0)
            .with_type("Fire")
            .with_critical(true);

        let summary = entry.summary();
        assert!(summary.contains("Player"));
        assert!(summary.contains("Goblin"));
        assert!(summary.contains("Fire"));
        assert!(summary.contains("CRIT"));
    }

    #[test]
    fn test_frame_data_first_last_active() {
        let fd = FrameData::new("Test", 5, 3, 10);
        assert_eq!(fd.first_active(), 6);
        assert_eq!(fd.last_active(), 8);
    }
}
