//! NPC debug and editor tools.
//!
//! This module provides:
//! - NPC debug overlay (T-37) - visualize AI state, collision, targets
//! - NPC spawn editor (T-38) - spawn/remove NPCs for debugging
//! - NPC list panel (T-39) - list and manage NPCs in loaded chunks

use crate::ui::{ConstrainedWindow, ScreenConstraints};
use egui::{Color32, Context, Id, Key, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

// ============================================================================
// Common Types
// ============================================================================

/// Unique identifier for NPCs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct NpcId(pub u64);

impl NpcId {
    /// Creates a new NPC ID.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for NpcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NPC#{}", self.0)
    }
}

/// NPC type/species.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NpcType {
    /// Generic villager NPC.
    #[default]
    Villager,
    /// Merchant/shopkeeper.
    Merchant,
    /// Guard/soldier.
    Guard,
    /// Animal (passive).
    Animal,
    /// Monster (hostile).
    Monster,
    /// Quest giver.
    QuestGiver,
    /// Companion/follower.
    Companion,
    /// Boss enemy.
    Boss,
}

impl NpcType {
    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            NpcType::Villager => "Villager",
            NpcType::Merchant => "Merchant",
            NpcType::Guard => "Guard",
            NpcType::Animal => "Animal",
            NpcType::Monster => "Monster",
            NpcType::QuestGiver => "Quest Giver",
            NpcType::Companion => "Companion",
            NpcType::Boss => "Boss",
        }
    }

    /// Returns the display color.
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            NpcType::Villager => Color32::from_rgb(100, 180, 100),
            NpcType::Merchant => Color32::from_rgb(200, 180, 100),
            NpcType::Guard => Color32::from_rgb(100, 150, 200),
            NpcType::Animal => Color32::from_rgb(180, 140, 100),
            NpcType::Monster => Color32::from_rgb(200, 80, 80),
            NpcType::QuestGiver => Color32::GOLD,
            NpcType::Companion => Color32::from_rgb(100, 200, 200),
            NpcType::Boss => Color32::from_rgb(180, 50, 180),
        }
    }

    /// Returns all NPC types.
    #[must_use]
    pub fn all() -> &'static [NpcType] {
        &[
            NpcType::Villager,
            NpcType::Merchant,
            NpcType::Guard,
            NpcType::Animal,
            NpcType::Monster,
            NpcType::QuestGiver,
            NpcType::Companion,
            NpcType::Boss,
        ]
    }
}

/// NPC state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NpcState {
    /// Standing still.
    #[default]
    Idle,
    /// Walking/moving.
    Walking,
    /// Running.
    Running,
    /// In combat.
    Combat,
    /// Dead.
    Dead,
    /// Talking/dialogue.
    Talking,
    /// Sleeping.
    Sleeping,
    /// Working (job activity).
    Working,
}

impl NpcState {
    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            NpcState::Idle => "Idle",
            NpcState::Walking => "Walking",
            NpcState::Running => "Running",
            NpcState::Combat => "Combat",
            NpcState::Dead => "Dead",
            NpcState::Talking => "Talking",
            NpcState::Sleeping => "Sleeping",
            NpcState::Working => "Working",
        }
    }

    /// Returns the display color.
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            NpcState::Idle => Color32::LIGHT_GRAY,
            NpcState::Walking => Color32::WHITE,
            NpcState::Running => Color32::from_rgb(100, 200, 255),
            NpcState::Combat => Color32::RED,
            NpcState::Dead => Color32::DARK_GRAY,
            NpcState::Talking => Color32::YELLOW,
            NpcState::Sleeping => Color32::from_rgb(100, 100, 180),
            NpcState::Working => Color32::from_rgb(180, 180, 100),
        }
    }

    /// Returns all NPC states.
    #[must_use]
    pub fn all() -> &'static [NpcState] {
        &[
            NpcState::Idle,
            NpcState::Walking,
            NpcState::Running,
            NpcState::Combat,
            NpcState::Dead,
            NpcState::Talking,
            NpcState::Sleeping,
            NpcState::Working,
        ]
    }
}

/// NPC AI behavior type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NpcBehavior {
    /// No active behavior.
    #[default]
    None,
    /// Patrolling a route.
    Patrol,
    /// Wandering randomly.
    Wander,
    /// Following a target.
    Follow,
    /// Guarding a position.
    Guard,
    /// Fleeing from threat.
    Flee,
    /// Attacking a target.
    Attack,
    /// Going to a location.
    GoTo,
}

impl NpcBehavior {
    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            NpcBehavior::None => "None",
            NpcBehavior::Patrol => "Patrol",
            NpcBehavior::Wander => "Wander",
            NpcBehavior::Follow => "Follow",
            NpcBehavior::Guard => "Guard",
            NpcBehavior::Flee => "Flee",
            NpcBehavior::Attack => "Attack",
            NpcBehavior::GoTo => "GoTo",
        }
    }

    /// Returns the display color.
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            NpcBehavior::None => Color32::DARK_GRAY,
            NpcBehavior::Patrol => Color32::from_rgb(100, 150, 200),
            NpcBehavior::Wander => Color32::from_rgb(150, 180, 150),
            NpcBehavior::Follow => Color32::from_rgb(200, 180, 100),
            NpcBehavior::Guard => Color32::from_rgb(100, 200, 200),
            NpcBehavior::Flee => Color32::from_rgb(200, 150, 100),
            NpcBehavior::Attack => Color32::RED,
            NpcBehavior::GoTo => Color32::WHITE,
        }
    }
}

/// Data for a single NPC used in debug displays.
#[derive(Debug, Clone, Default)]
pub struct NpcDebugData {
    /// NPC ID.
    pub id: NpcId,
    /// NPC name.
    pub name: String,
    /// NPC type.
    pub npc_type: NpcType,
    /// Current state.
    pub state: NpcState,
    /// Current AI behavior.
    pub behavior: NpcBehavior,
    /// World position.
    pub position: (f32, f32),
    /// Target position (if any).
    pub target_position: Option<(f32, f32)>,
    /// Collision radius.
    pub collision_radius: f32,
    /// Interaction radius.
    pub interaction_radius: f32,
    /// Current health (0.0-1.0).
    pub health: f32,
    /// Maximum health.
    pub max_health: f32,
    /// Is this NPC selected for debug.
    pub selected: bool,
    /// Is this a debug-spawned NPC.
    pub debug_spawned: bool,
}

impl NpcDebugData {
    /// Creates new NPC debug data.
    #[must_use]
    pub fn new(id: NpcId, name: impl Into<String>, npc_type: NpcType) -> Self {
        Self {
            id,
            name: name.into(),
            npc_type,
            state: NpcState::Idle,
            behavior: NpcBehavior::None,
            position: (0.0, 0.0),
            target_position: None,
            collision_radius: 16.0,
            interaction_radius: 48.0,
            health: 1.0,
            max_health: 100.0,
            selected: false,
            debug_spawned: false,
        }
    }

    /// Sets position.
    #[must_use]
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Sets health.
    #[must_use]
    pub fn with_health(mut self, health: f32, max_health: f32) -> Self {
        self.health = health;
        self.max_health = max_health;
        self
    }

    /// Returns health as normalized value (0.0-1.0).
    #[must_use]
    pub fn health_normalized(&self) -> f32 {
        if self.max_health > 0.0 {
            (self.health / self.max_health).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Calculates distance to a position.
    #[must_use]
    pub fn distance_to(&self, x: f32, y: f32) -> f32 {
        let dx = self.position.0 - x;
        let dy = self.position.1 - y;
        (dx * dx + dy * dy).sqrt()
    }
}

// ============================================================================
// T-37: NPC Debug Overlay
// ============================================================================

/// Configuration for NPC debug overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDebugOverlayConfig {
    /// Show NPC ID labels.
    pub show_ids: bool,
    /// Show collision radius circles.
    pub show_collision: bool,
    /// Show interaction radius circles.
    pub show_interaction: bool,
    /// Show health bars.
    pub show_health: bool,
    /// Show target lines.
    pub show_targets: bool,
    /// Show state labels.
    pub show_state: bool,
    /// Show behavior labels.
    pub show_behavior: bool,
    /// Label font size.
    pub font_size: f32,
    /// Health bar width.
    pub health_bar_width: f32,
    /// Health bar height.
    pub health_bar_height: f32,
    /// Toggle key name.
    #[serde(default = "default_overlay_toggle_key")]
    pub toggle_key_name: String,
}

fn default_overlay_toggle_key() -> String {
    "F9".to_string()
}

impl Default for NpcDebugOverlayConfig {
    fn default() -> Self {
        Self {
            show_ids: true,
            show_collision: true,
            show_interaction: true,
            show_health: true,
            show_targets: true,
            show_state: true,
            show_behavior: false,
            font_size: 10.0,
            health_bar_width: 40.0,
            health_bar_height: 4.0,
            toggle_key_name: default_overlay_toggle_key(),
        }
    }
}

impl NpcDebugOverlayConfig {
    /// Returns the toggle key.
    #[must_use]
    pub fn toggle_key(&self) -> Option<Key> {
        match self.toggle_key_name.to_uppercase().as_str() {
            "F9" => Some(Key::F9),
            "F10" => Some(Key::F10),
            "F11" => Some(Key::F11),
            "N" => Some(Key::N),
            _ => None,
        }
    }
}

/// NPC debug overlay statistics.
#[derive(Debug, Clone, Default)]
pub struct NpcDebugStats {
    /// Total NPC count.
    pub total_npcs: usize,
    /// NPCs per chunk (chunk coords -> count).
    pub npcs_per_chunk: Vec<((i32, i32), usize)>,
    /// Last AI update time in ms.
    pub ai_update_time_ms: f32,
    /// Selected NPC ID.
    pub selected_npc: Option<NpcId>,
}

/// NPC debug overlay widget.
#[derive(Debug)]
pub struct NpcDebugOverlay {
    /// Configuration.
    config: NpcDebugOverlayConfig,
    /// Whether the overlay is visible.
    visible: bool,
}

impl Default for NpcDebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcDebugOverlay {
    /// Creates a new NPC debug overlay.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: NpcDebugOverlayConfig::default(),
            visible: false,
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: NpcDebugOverlayConfig) -> Self {
        Self {
            config,
            visible: false,
        }
    }

    /// Toggles visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Returns whether overlay is visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Renders the overlay for world-space NPC visualization.
    /// `world_to_screen` converts world coords to screen coords.
    pub fn render_world_overlay<F>(
        &mut self,
        ctx: &Context,
        npcs: &[NpcDebugData],
        world_to_screen: F,
    ) where
        F: Fn(f32, f32) -> Option<(f32, f32)>,
    {
        // Handle toggle key
        if let Some(key) = self.config.toggle_key() {
            if ctx.input(|i| i.key_pressed(key)) {
                self.toggle();
            }
        }

        if !self.visible {
            return;
        }

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            Id::new("npc_debug_overlay"),
        ));

        for npc in npcs {
            if let Some((screen_x, screen_y)) = world_to_screen(npc.position.0, npc.position.1) {
                let center = Pos2::new(screen_x, screen_y);

                // Collision radius circle
                if self.config.show_collision {
                    let radius = npc.collision_radius;
                    painter.circle_stroke(
                        center,
                        radius,
                        Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 100, 100, 180)),
                    );
                }

                // Interaction radius circle
                if self.config.show_interaction {
                    let radius = npc.interaction_radius;
                    painter.circle_stroke(
                        center,
                        radius,
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 200, 255, 120)),
                    );
                }

                // Target line
                if self.config.show_targets {
                    if let Some((tx, ty)) = npc.target_position {
                        if let Some((target_sx, target_sy)) = world_to_screen(tx, ty) {
                            painter.line_segment(
                                [center, Pos2::new(target_sx, target_sy)],
                                Stroke::new(
                                    1.0,
                                    Color32::from_rgba_unmultiplied(255, 200, 100, 150),
                                ),
                            );
                        }
                    }
                }

                // Health bar above NPC
                if self.config.show_health && npc.health_normalized() < 1.0 {
                    let bar_y = screen_y - npc.collision_radius - 8.0;
                    let bar_rect = Rect::from_min_size(
                        Pos2::new(screen_x - self.config.health_bar_width / 2.0, bar_y),
                        Vec2::new(self.config.health_bar_width, self.config.health_bar_height),
                    );

                    // Background
                    painter.rect_filled(bar_rect, Rounding::ZERO, Color32::from_gray(40));

                    // Health fill
                    let fill_width = bar_rect.width() * npc.health_normalized();
                    let fill_rect = Rect::from_min_size(
                        bar_rect.min,
                        Vec2::new(fill_width, self.config.health_bar_height),
                    );
                    let health_color = health_color(npc.health_normalized());
                    painter.rect_filled(fill_rect, Rounding::ZERO, health_color);

                    // Border
                    painter.rect_stroke(
                        bar_rect,
                        Rounding::ZERO,
                        Stroke::new(1.0, Color32::DARK_GRAY),
                    );
                }

                // Labels
                let mut label_y = screen_y - npc.collision_radius - 20.0;

                if self.config.show_ids {
                    painter.text(
                        Pos2::new(screen_x, label_y),
                        egui::Align2::CENTER_BOTTOM,
                        format!("{}", npc.id),
                        egui::FontId::proportional(self.config.font_size),
                        npc.npc_type.color(),
                    );
                    label_y -= self.config.font_size + 2.0;
                }

                if self.config.show_state {
                    painter.text(
                        Pos2::new(screen_x, label_y),
                        egui::Align2::CENTER_BOTTOM,
                        npc.state.display_name(),
                        egui::FontId::proportional(self.config.font_size - 1.0),
                        npc.state.color(),
                    );
                    label_y -= self.config.font_size;
                }

                if self.config.show_behavior && npc.behavior != NpcBehavior::None {
                    painter.text(
                        Pos2::new(screen_x, label_y),
                        egui::Align2::CENTER_BOTTOM,
                        npc.behavior.display_name(),
                        egui::FontId::proportional(self.config.font_size - 1.0),
                        npc.behavior.color(),
                    );
                }

                // Selection highlight
                if npc.selected {
                    painter.circle_stroke(
                        center,
                        npc.collision_radius + 4.0,
                        Stroke::new(2.0, Color32::GOLD),
                    );
                }
            }
        }
    }

    /// Renders the debug stats panel.
    pub fn render_stats_panel(
        &self,
        ui: &mut Ui,
        stats: &NpcDebugStats,
        selected_npc: Option<&NpcDebugData>,
    ) {
        ui.label(
            RichText::new("NPC Debug")
                .color(Color32::LIGHT_GREEN)
                .size(14.0),
        );

        ui.horizontal(|ui| {
            ui.label(RichText::new("Total NPCs:").color(Color32::GRAY));
            ui.label(RichText::new(stats.total_npcs.to_string()).color(Color32::WHITE));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("AI Update:").color(Color32::GRAY));
            ui.label(
                RichText::new(format!("{:.2}ms", stats.ai_update_time_ms)).color(
                    if stats.ai_update_time_ms > 5.0 {
                        Color32::RED
                    } else {
                        Color32::WHITE
                    },
                ),
            );
        });

        // NPCs per chunk (top 3)
        if !stats.npcs_per_chunk.is_empty() {
            ui.add_space(4.0);
            ui.label(
                RichText::new("Per Chunk:")
                    .color(Color32::LIGHT_BLUE)
                    .size(12.0),
            );
            for (i, ((cx, cy), count)) in stats.npcs_per_chunk.iter().take(3).enumerate() {
                ui.label(
                    RichText::new(format!("  ({cx},{cy}): {count}"))
                        .color(Color32::GRAY)
                        .size(11.0),
                );
                if i >= 2 {
                    break;
                }
            }
        }

        // Selected NPC details
        if let Some(npc) = selected_npc {
            ui.add_space(8.0);
            ui.separator();
            ui.label(
                RichText::new("Selected NPC")
                    .color(Color32::GOLD)
                    .size(13.0),
            );

            ui.horizontal(|ui| {
                ui.label(RichText::new("ID:").color(Color32::GRAY));
                ui.label(RichText::new(format!("{}", npc.id)).color(Color32::WHITE));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Name:").color(Color32::GRAY));
                ui.label(RichText::new(&npc.name).color(Color32::WHITE));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Type:").color(Color32::GRAY));
                ui.label(RichText::new(npc.npc_type.display_name()).color(npc.npc_type.color()));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("State:").color(Color32::GRAY));
                ui.label(RichText::new(npc.state.display_name()).color(npc.state.color()));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Behavior:").color(Color32::GRAY));
                ui.label(RichText::new(npc.behavior.display_name()).color(npc.behavior.color()));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Position:").color(Color32::GRAY));
                ui.label(
                    RichText::new(format!("({:.1}, {:.1})", npc.position.0, npc.position.1))
                        .color(Color32::WHITE),
                );
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Health:").color(Color32::GRAY));
                ui.label(
                    RichText::new(format!("{:.0}/{:.0}", npc.health, npc.max_health))
                        .color(health_color(npc.health_normalized())),
                );
            });
        }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &NpcDebugOverlayConfig {
        &self.config
    }

    /// Sets configuration.
    pub fn set_config(&mut self, config: NpcDebugOverlayConfig) {
        self.config = config;
    }
}

/// Returns health bar color based on health percentage.
fn health_color(health_pct: f32) -> Color32 {
    if health_pct > 0.6 {
        Color32::from_rgb(80, 200, 80)
    } else if health_pct > 0.3 {
        Color32::from_rgb(200, 200, 80)
    } else {
        Color32::from_rgb(200, 80, 80)
    }
}

// ============================================================================
// T-38: NPC Spawn Editor
// ============================================================================

/// Actions emitted by the spawn editor.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcSpawnAction {
    /// Spawn an NPC at position.
    Spawn {
        /// NPC type to spawn.
        npc_type: NpcType,
        /// World position.
        position: (f32, f32),
        /// Optional custom name.
        name: Option<String>,
    },
    /// Remove an NPC by ID.
    Remove(NpcId),
    /// Select an NPC.
    Select(NpcId),
    /// Clear selection.
    Deselect,
    /// Spawn at player position.
    SpawnAtPlayer(NpcType),
}

/// Configuration for the spawn editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSpawnEditorConfig {
    /// Font size.
    pub font_size: f32,
    /// Panel width.
    pub panel_width: f32,
    /// Panel position from top-left.
    pub position: (f32, f32),
}

impl Default for NpcSpawnEditorConfig {
    fn default() -> Self {
        Self {
            font_size: 13.0,
            panel_width: 220.0,
            position: (10.0, 300.0),
        }
    }
}

/// NPC spawn editor widget.
#[derive(Debug)]
pub struct NpcSpawnEditor {
    /// Configuration.
    config: NpcSpawnEditorConfig,
    /// Whether the editor is visible.
    visible: bool,
    /// Selected NPC type to spawn.
    selected_type: NpcType,
    /// Spawn mode active (click to spawn).
    spawn_mode: bool,
    /// Pending actions.
    actions: Vec<NpcSpawnAction>,
    /// Custom name input.
    custom_name: String,
}

impl Default for NpcSpawnEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcSpawnEditor {
    /// Creates a new spawn editor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: NpcSpawnEditorConfig::default(),
            visible: false,
            selected_type: NpcType::Villager,
            spawn_mode: false,
            actions: Vec::new(),
            custom_name: String::new(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: NpcSpawnEditorConfig) -> Self {
        Self {
            config,
            visible: false,
            selected_type: NpcType::Villager,
            spawn_mode: false,
            actions: Vec::new(),
            custom_name: String::new(),
        }
    }

    /// Toggles visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Returns whether editor is visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns whether spawn mode is active.
    #[must_use]
    pub fn is_spawn_mode(&self) -> bool {
        self.spawn_mode
    }

    /// Sets spawn mode.
    pub fn set_spawn_mode(&mut self, active: bool) {
        self.spawn_mode = active;
    }

    /// Called when world is clicked (in spawn mode).
    pub fn on_world_click(&mut self, world_x: f32, world_y: f32) {
        if self.spawn_mode {
            let name = if self.custom_name.is_empty() {
                None
            } else {
                Some(self.custom_name.clone())
            };
            self.actions.push(NpcSpawnAction::Spawn {
                npc_type: self.selected_type,
                position: (world_x, world_y),
                name,
            });
        }
    }

    /// Called when an NPC is clicked.
    pub fn on_npc_click(&mut self, npc_id: NpcId) {
        self.actions.push(NpcSpawnAction::Select(npc_id));
    }

    /// Drains pending actions.
    pub fn drain_actions(&mut self) -> Vec<NpcSpawnAction> {
        std::mem::take(&mut self.actions)
    }

    /// Returns pending actions.
    #[must_use]
    pub fn actions(&self) -> &[NpcSpawnAction] {
        &self.actions
    }

    /// Renders the spawn editor panel.
    pub fn render(
        &mut self,
        ctx: &Context,
        debug_npcs: &[NpcDebugData],
        selected_npc: Option<NpcId>,
    ) {
        if !self.visible {
            return;
        }

        // Calculate max window size based on screen with margin
        let constraints = ScreenConstraints::from_context(ctx);

        egui::Window::new("NPC Spawn Editor")
            .id(Id::new("npc_spawn_editor"))
            .default_pos(Pos2::new(self.config.position.0, self.config.position.1))
            .default_width(constraints.constrained_width(self.config.panel_width))
            .with_screen_constraints(&constraints)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // NPC Type selector
                ui.label(
                    RichText::new("Spawn NPC")
                        .size(self.config.font_size)
                        .strong(),
                );
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("npc_type_combo")
                        .selected_text(self.selected_type.display_name())
                        .show_ui(ui, |ui| {
                            for npc_type in NpcType::all() {
                                ui.selectable_value(
                                    &mut self.selected_type,
                                    *npc_type,
                                    npc_type.display_name(),
                                );
                            }
                        });
                });

                // Custom name
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.custom_name)
                            .desired_width(100.0)
                            .hint_text("(optional)"),
                    );
                });

                ui.add_space(8.0);

                // Spawn buttons
                ui.horizontal(|ui| {
                    let spawn_btn_text = if self.spawn_mode {
                        "🎯 Click to Spawn (ON)"
                    } else {
                        "🎯 Click to Spawn"
                    };
                    if ui
                        .button(spawn_btn_text)
                        .on_hover_text("Toggle click-to-spawn mode")
                        .clicked()
                    {
                        self.spawn_mode = !self.spawn_mode;
                    }
                });

                if ui
                    .button("📍 Spawn at Player")
                    .on_hover_text("Spawn NPC at player's current position")
                    .clicked()
                {
                    self.actions
                        .push(NpcSpawnAction::SpawnAtPlayer(self.selected_type));
                }

                ui.add_space(8.0);
                ui.separator();

                // Selected NPC actions
                if let Some(npc_id) = selected_npc {
                    ui.label(
                        RichText::new(format!("Selected: {npc_id}"))
                            .color(Color32::GOLD)
                            .size(self.config.font_size),
                    );

                    ui.horizontal(|ui| {
                        if ui.button("🗑 Remove").clicked() {
                            self.actions.push(NpcSpawnAction::Remove(npc_id));
                        }
                        if ui.button("✖ Deselect").clicked() {
                            self.actions.push(NpcSpawnAction::Deselect);
                        }
                    });
                } else {
                    ui.label(RichText::new("No NPC selected").color(Color32::GRAY));
                }

                ui.add_space(8.0);
                ui.separator();

                // Debug-spawned NPCs list
                let debug_spawned: Vec<_> = debug_npcs.iter().filter(|n| n.debug_spawned).collect();
                ui.label(
                    RichText::new(format!("Debug NPCs ({})", debug_spawned.len()))
                        .size(self.config.font_size),
                );

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for npc in debug_spawned {
                            ui.horizontal(|ui| {
                                let is_selected = selected_npc == Some(npc.id);
                                let color = if is_selected {
                                    Color32::GOLD
                                } else {
                                    npc.npc_type.color()
                                };

                                if ui
                                    .add(
                                        egui::Label::new(
                                            RichText::new(format!(
                                                "{} - {}",
                                                npc.id,
                                                npc.npc_type.display_name()
                                            ))
                                            .color(color),
                                        )
                                        .sense(egui::Sense::click()),
                                    )
                                    .clicked()
                                {
                                    self.actions.push(NpcSpawnAction::Select(npc.id));
                                }
                            });
                        }
                    });
            });
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &NpcSpawnEditorConfig {
        &self.config
    }
}

// ============================================================================
// T-39: NPC List Panel
// ============================================================================

/// Actions emitted by the NPC list panel.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcListAction {
    /// Select an NPC.
    Select(NpcId),
    /// Teleport player to NPC.
    TeleportTo(NpcId),
    /// Center camera on NPC.
    CenterOn(NpcId),
}

/// Filter options for NPC list.
#[derive(Debug, Clone, Default)]
pub struct NpcListFilter {
    /// Filter by NPC type (None = all).
    pub npc_type: Option<NpcType>,
    /// Filter by state (None = all).
    pub state: Option<NpcState>,
    /// Search text for name.
    pub search: String,
    /// Maximum distance from player (None = no limit).
    pub max_distance: Option<f32>,
}

impl NpcListFilter {
    /// Returns whether an NPC matches the filter.
    #[must_use]
    pub fn matches(&self, npc: &NpcDebugData, player_pos: (f32, f32)) -> bool {
        // Type filter
        if let Some(t) = self.npc_type {
            if npc.npc_type != t {
                return false;
            }
        }

        // State filter
        if let Some(s) = self.state {
            if npc.state != s {
                return false;
            }
        }

        // Search filter
        if !self.search.is_empty() {
            let search_lower = self.search.to_lowercase();
            if !npc.name.to_lowercase().contains(&search_lower)
                && !format!("{}", npc.id).contains(&search_lower)
            {
                return false;
            }
        }

        // Distance filter
        if let Some(max_dist) = self.max_distance {
            if npc.distance_to(player_pos.0, player_pos.1) > max_dist {
                return false;
            }
        }

        true
    }
}

/// Configuration for the NPC list panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcListPanelConfig {
    /// Font size.
    pub font_size: f32,
    /// Panel width.
    pub panel_width: f32,
    /// Panel height.
    pub panel_height: f32,
}

impl Default for NpcListPanelConfig {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            panel_width: 300.0,
            panel_height: 400.0,
        }
    }
}

/// NPC list panel widget.
#[derive(Debug)]
pub struct NpcListPanel {
    /// Configuration.
    config: NpcListPanelConfig,
    /// Whether the panel is visible.
    visible: bool,
    /// Current filter.
    filter: NpcListFilter,
    /// Pending actions.
    actions: Vec<NpcListAction>,
    /// Sort by distance.
    sort_by_distance: bool,
}

impl Default for NpcListPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcListPanel {
    /// Creates a new NPC list panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: NpcListPanelConfig::default(),
            visible: false,
            filter: NpcListFilter::default(),
            actions: Vec::new(),
            sort_by_distance: true,
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: NpcListPanelConfig) -> Self {
        Self {
            config,
            visible: false,
            filter: NpcListFilter::default(),
            actions: Vec::new(),
            sort_by_distance: true,
        }
    }

    /// Toggles visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Returns whether panel is visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Drains pending actions.
    pub fn drain_actions(&mut self) -> Vec<NpcListAction> {
        std::mem::take(&mut self.actions)
    }

    /// Returns pending actions.
    #[must_use]
    pub fn actions(&self) -> &[NpcListAction] {
        &self.actions
    }

    /// Returns the current filter.
    #[must_use]
    pub fn filter(&self) -> &NpcListFilter {
        &self.filter
    }

    /// Sets the filter.
    pub fn set_filter(&mut self, filter: NpcListFilter) {
        self.filter = filter;
    }

    /// Renders the NPC list panel.
    pub fn render(
        &mut self,
        ctx: &Context,
        npcs: &[NpcDebugData],
        player_pos: (f32, f32),
        selected_npc: Option<NpcId>,
    ) {
        if !self.visible {
            return;
        }

        // Calculate max window size based on screen with margin
        let constraints = ScreenConstraints::from_context(ctx);

        egui::Window::new("NPC List")
            .id(Id::new("npc_list_panel"))
            .with_constrained_defaults(&constraints, self.config.panel_width, self.config.panel_height)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Filters
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.filter.search)
                            .desired_width(80.0)
                            .hint_text("Name/ID..."),
                    );

                    ui.separator();

                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("npc_list_type_filter")
                        .width(70.0)
                        .selected_text(self.filter.npc_type.map_or("All", |t| t.display_name()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.filter.npc_type, None, "All");
                            for npc_type in NpcType::all() {
                                ui.selectable_value(
                                    &mut self.filter.npc_type,
                                    Some(*npc_type),
                                    npc_type.display_name(),
                                );
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("State:");
                    egui::ComboBox::from_id_salt("npc_list_state_filter")
                        .width(70.0)
                        .selected_text(self.filter.state.map_or("All", |s| s.display_name()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.filter.state, None, "All");
                            for state in NpcState::all() {
                                ui.selectable_value(
                                    &mut self.filter.state,
                                    Some(*state),
                                    state.display_name(),
                                );
                            }
                        });

                    ui.separator();

                    ui.checkbox(&mut self.sort_by_distance, "Sort by distance");
                });

                ui.separator();

                // Filter and sort NPCs
                let mut filtered: Vec<_> = npcs
                    .iter()
                    .filter(|npc| self.filter.matches(npc, player_pos))
                    .collect();

                if self.sort_by_distance {
                    filtered.sort_by(|a, b| {
                        let dist_a = a.distance_to(player_pos.0, player_pos.1);
                        let dist_b = b.distance_to(player_pos.0, player_pos.1);
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }

                // Count display
                ui.label(
                    RichText::new(format!("Showing {} of {} NPCs", filtered.len(), npcs.len()))
                        .size(self.config.font_size)
                        .color(Color32::GRAY),
                );

                ui.separator();

                // NPC list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for npc in filtered {
                        let is_selected = selected_npc == Some(npc.id);
                        let distance = npc.distance_to(player_pos.0, player_pos.1);

                        let bg_color = if is_selected {
                            Color32::from_rgba_unmultiplied(100, 80, 40, 100)
                        } else {
                            Color32::TRANSPARENT
                        };

                        egui::Frame::none()
                            .fill(bg_color)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Select on click
                                    let response = ui.add(
                                        egui::Label::new(
                                            RichText::new(format!("{}", npc.id))
                                                .size(self.config.font_size)
                                                .color(npc.npc_type.color()),
                                        )
                                        .sense(egui::Sense::click()),
                                    );

                                    if response.clicked() {
                                        self.actions.push(NpcListAction::Select(npc.id));
                                    }

                                    ui.label(
                                        RichText::new(&npc.name)
                                            .size(self.config.font_size)
                                            .color(Color32::WHITE),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Distance
                                            ui.label(
                                                RichText::new(format!("{distance:.0}"))
                                                    .size(self.config.font_size - 1.0)
                                                    .color(Color32::GRAY),
                                            );

                                            // State
                                            ui.label(
                                                RichText::new(npc.state.display_name())
                                                    .size(self.config.font_size - 1.0)
                                                    .color(npc.state.color()),
                                            );
                                        },
                                    );
                                });

                                // Actions for selected NPC
                                if is_selected {
                                    ui.horizontal(|ui| {
                                        ui.add_space(20.0);
                                        if ui.small_button("📍 Teleport").clicked() {
                                            self.actions.push(NpcListAction::TeleportTo(npc.id));
                                        }
                                        if ui.small_button("🎯 Center").clicked() {
                                            self.actions.push(NpcListAction::CenterOn(npc.id));
                                        }
                                    });
                                }
                            });
                    }
                });
            });
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &NpcListPanelConfig {
        &self.config
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Common type tests
    #[test]
    fn test_npc_id() {
        let id = NpcId::new(42);
        assert_eq!(id.raw(), 42);
        assert_eq!(format!("{id}"), "NPC#42");
    }

    #[test]
    fn test_npc_type_all() {
        let types = NpcType::all();
        assert_eq!(types.len(), 8);
        assert!(types.contains(&NpcType::Villager));
        assert!(types.contains(&NpcType::Boss));
    }

    #[test]
    fn test_npc_type_color() {
        let villager_color = NpcType::Villager.color();
        let monster_color = NpcType::Monster.color();
        assert_ne!(villager_color, monster_color);
    }

    #[test]
    fn test_npc_state_all() {
        let states = NpcState::all();
        assert_eq!(states.len(), 8);
        assert!(states.contains(&NpcState::Idle));
        assert!(states.contains(&NpcState::Combat));
    }

    #[test]
    fn test_npc_behavior_display_name() {
        assert_eq!(NpcBehavior::Patrol.display_name(), "Patrol");
        assert_eq!(NpcBehavior::Attack.display_name(), "Attack");
    }

    #[test]
    fn test_npc_debug_data_new() {
        let npc = NpcDebugData::new(NpcId::new(1), "Test NPC", NpcType::Villager);
        assert_eq!(npc.id, NpcId::new(1));
        assert_eq!(npc.name, "Test NPC");
        assert_eq!(npc.npc_type, NpcType::Villager);
        assert_eq!(npc.state, NpcState::Idle);
    }

    #[test]
    fn test_npc_debug_data_with_position() {
        let npc =
            NpcDebugData::new(NpcId::new(1), "Test", NpcType::Guard).with_position(100.0, 200.0);
        assert_eq!(npc.position, (100.0, 200.0));
    }

    #[test]
    fn test_npc_debug_data_health_normalized() {
        let npc =
            NpcDebugData::new(NpcId::new(1), "Test", NpcType::Monster).with_health(50.0, 100.0);
        assert!((npc.health_normalized() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_npc_debug_data_distance_to() {
        let npc =
            NpcDebugData::new(NpcId::new(1), "Test", NpcType::Villager).with_position(0.0, 0.0);
        let dist = npc.distance_to(3.0, 4.0);
        assert!((dist - 5.0).abs() < f32::EPSILON);
    }

    // T-37 Tests
    #[test]
    fn test_npc_debug_overlay_new() {
        let overlay = NpcDebugOverlay::new();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_npc_debug_overlay_toggle() {
        let mut overlay = NpcDebugOverlay::new();
        assert!(!overlay.is_visible());

        overlay.toggle();
        assert!(overlay.is_visible());

        overlay.toggle();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_npc_debug_overlay_config_defaults() {
        let config = NpcDebugOverlayConfig::default();
        assert!(config.show_ids);
        assert!(config.show_collision);
        assert!(config.show_health);
        assert_eq!(config.toggle_key(), Some(Key::F9));
    }

    #[test]
    fn test_npc_debug_stats_default() {
        let stats = NpcDebugStats::default();
        assert_eq!(stats.total_npcs, 0);
        assert!(stats.npcs_per_chunk.is_empty());
    }

    // T-38 Tests
    #[test]
    fn test_npc_spawn_editor_new() {
        let editor = NpcSpawnEditor::new();
        assert!(!editor.is_visible());
        assert!(!editor.is_spawn_mode());
    }

    #[test]
    fn test_npc_spawn_editor_toggle() {
        let mut editor = NpcSpawnEditor::new();
        editor.toggle();
        assert!(editor.is_visible());
    }

    #[test]
    fn test_npc_spawn_editor_spawn_mode() {
        let mut editor = NpcSpawnEditor::new();
        editor.set_spawn_mode(true);
        assert!(editor.is_spawn_mode());

        editor.on_world_click(100.0, 200.0);
        let actions = editor.drain_actions();
        assert_eq!(actions.len(), 1);

        match &actions[0] {
            NpcSpawnAction::Spawn {
                npc_type, position, ..
            } => {
                assert_eq!(*npc_type, NpcType::Villager);
                assert_eq!(*position, (100.0, 200.0));
            },
            _ => panic!("Expected Spawn action"),
        }
    }

    #[test]
    fn test_npc_spawn_editor_on_npc_click() {
        let mut editor = NpcSpawnEditor::new();
        editor.on_npc_click(NpcId::new(42));

        let actions = editor.drain_actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], NpcSpawnAction::Select(NpcId::new(42)));
    }

    #[test]
    fn test_npc_spawn_action_equality() {
        let a1 = NpcSpawnAction::Select(NpcId::new(1));
        let a2 = NpcSpawnAction::Select(NpcId::new(1));
        let a3 = NpcSpawnAction::Select(NpcId::new(2));
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }

    // T-39 Tests
    #[test]
    fn test_npc_list_panel_new() {
        let panel = NpcListPanel::new();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_npc_list_panel_toggle() {
        let mut panel = NpcListPanel::new();
        panel.toggle();
        assert!(panel.is_visible());
    }

    #[test]
    fn test_npc_list_filter_matches_all() {
        let filter = NpcListFilter::default();
        let npc = NpcDebugData::new(NpcId::new(1), "Test", NpcType::Villager);
        assert!(filter.matches(&npc, (0.0, 0.0)));
    }

    #[test]
    fn test_npc_list_filter_by_type() {
        let filter = NpcListFilter {
            npc_type: Some(NpcType::Guard),
            ..Default::default()
        };

        let villager = NpcDebugData::new(NpcId::new(1), "V", NpcType::Villager);
        let guard = NpcDebugData::new(NpcId::new(2), "G", NpcType::Guard);

        assert!(!filter.matches(&villager, (0.0, 0.0)));
        assert!(filter.matches(&guard, (0.0, 0.0)));
    }

    #[test]
    fn test_npc_list_filter_by_state() {
        let filter = NpcListFilter {
            state: Some(NpcState::Combat),
            ..Default::default()
        };

        let mut idle_npc = NpcDebugData::new(NpcId::new(1), "I", NpcType::Monster);
        idle_npc.state = NpcState::Idle;

        let mut combat_npc = NpcDebugData::new(NpcId::new(2), "C", NpcType::Monster);
        combat_npc.state = NpcState::Combat;

        assert!(!filter.matches(&idle_npc, (0.0, 0.0)));
        assert!(filter.matches(&combat_npc, (0.0, 0.0)));
    }

    #[test]
    fn test_npc_list_filter_by_search() {
        let filter = NpcListFilter {
            search: "Bob".to_string(),
            ..Default::default()
        };

        let bob = NpcDebugData::new(NpcId::new(1), "Bob the Guard", NpcType::Guard);
        let alice = NpcDebugData::new(NpcId::new(2), "Alice", NpcType::Villager);

        assert!(filter.matches(&bob, (0.0, 0.0)));
        assert!(!filter.matches(&alice, (0.0, 0.0)));
    }

    #[test]
    fn test_npc_list_filter_by_distance() {
        let filter = NpcListFilter {
            max_distance: Some(100.0),
            ..Default::default()
        };

        let near =
            NpcDebugData::new(NpcId::new(1), "Near", NpcType::Villager).with_position(50.0, 0.0);
        let far =
            NpcDebugData::new(NpcId::new(2), "Far", NpcType::Villager).with_position(200.0, 0.0);

        assert!(filter.matches(&near, (0.0, 0.0)));
        assert!(!filter.matches(&far, (0.0, 0.0)));
    }

    #[test]
    fn test_npc_list_action_equality() {
        let a1 = NpcListAction::Select(NpcId::new(1));
        let a2 = NpcListAction::Select(NpcId::new(1));
        assert_eq!(a1, a2);

        let t1 = NpcListAction::TeleportTo(NpcId::new(5));
        let t2 = NpcListAction::CenterOn(NpcId::new(5));
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_npc_list_panel_drain_actions() {
        let mut panel = NpcListPanel::new();
        panel.actions.push(NpcListAction::Select(NpcId::new(1)));
        panel.actions.push(NpcListAction::TeleportTo(NpcId::new(2)));

        let actions = panel.drain_actions();
        assert_eq!(actions.len(), 2);
        assert!(panel.actions.is_empty());
    }

    #[test]
    fn test_health_color() {
        let high = health_color(0.8);
        let mid = health_color(0.5);
        let low = health_color(0.2);
        assert_ne!(high, mid);
        assert_ne!(mid, low);
    }
}
