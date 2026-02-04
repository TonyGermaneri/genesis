//! Quest UI rendering for tracking and displaying quests.
//!
//! This module provides:
//! - Quest tracker HUD widget
//! - Quest log panel
//! - Objective display and progress tracking
//! - Quest markers and waypoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for quests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct QuestId(u64);

impl QuestId {
    /// Creates a new quest ID.
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

/// Current status of a quest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum QuestStatus {
    /// Quest is available but not started
    #[default]
    Available,
    /// Quest is active and in progress
    Active,
    /// Quest has been completed
    Completed,
    /// Quest has failed
    Failed,
    /// Quest is locked (requirements not met)
    Locked,
}

impl QuestStatus {
    /// Returns the display name for this status.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Active => "In Progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Locked => "Locked",
        }
    }

    /// Returns whether this status represents an active quest.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns whether the quest is finished (completed or failed).
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

/// Type of quest objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveType {
    /// Kill a certain number of enemies
    Kill,
    /// Collect items
    Collect,
    /// Talk to an NPC
    Talk,
    /// Reach a location
    Explore,
    /// Defend a position
    Defend,
    /// Escort an NPC
    Escort,
    /// Custom objective type
    Custom,
}

impl ObjectiveType {
    /// Returns an icon string for this objective type.
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Kill => "⚔",
            Self::Collect => "📦",
            Self::Talk => "💬",
            Self::Explore => "🗺",
            Self::Defend => "🛡",
            Self::Escort => "👥",
            Self::Custom => "◆",
        }
    }
}

/// A single quest objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    /// Unique ID within the quest
    pub id: u32,
    /// Type of objective
    pub objective_type: ObjectiveType,
    /// Description text
    pub description: String,
    /// Current progress count
    pub current: u32,
    /// Target count to complete
    pub target: u32,
    /// Whether this objective is optional
    pub optional: bool,
    /// Whether this objective is complete
    pub complete: bool,
    /// Whether this objective is hidden until revealed
    pub hidden: bool,
}

impl QuestObjective {
    /// Creates a new objective.
    #[must_use]
    pub fn new(id: u32, objective_type: ObjectiveType, description: impl Into<String>) -> Self {
        Self {
            id,
            objective_type,
            description: description.into(),
            current: 0,
            target: 1,
            optional: false,
            complete: false,
            hidden: false,
        }
    }

    /// Sets the target count.
    #[must_use]
    pub fn with_target(mut self, target: u32) -> Self {
        self.target = target;
        self
    }

    /// Sets whether this objective is optional.
    #[must_use]
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Sets whether this objective is hidden.
    #[must_use]
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Returns the progress percentage (0.0 - 1.0).
    #[must_use]
    pub fn progress(&self) -> f32 {
        if self.target == 0 {
            return 1.0;
        }
        (self.current as f32 / self.target as f32).clamp(0.0, 1.0)
    }

    /// Returns whether this objective is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.complete || self.current >= self.target
    }

    /// Returns the progress display string.
    #[must_use]
    pub fn progress_text(&self) -> String {
        if self.target <= 1 {
            if self.is_complete() {
                "✓".to_string()
            } else {
                "○".to_string()
            }
        } else {
            format!("{}/{}", self.current, self.target)
        }
    }
}

/// Quest difficulty level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuestDifficulty {
    /// Easy quest
    Easy,
    /// Normal quest
    #[default]
    Normal,
    /// Hard quest
    Hard,
    /// Epic quest
    Epic,
}

impl QuestDifficulty {
    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Epic => "Epic",
        }
    }

    /// Returns the color for this difficulty (RGBA).
    #[must_use]
    pub fn color(&self) -> [u8; 4] {
        match self {
            Self::Easy => [100, 200, 100, 255],   // Green
            Self::Normal => [200, 200, 200, 255], // Gray
            Self::Hard => [200, 150, 50, 255],    // Orange
            Self::Epic => [150, 100, 200, 255],   // Purple
        }
    }
}

/// Quest category for organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum QuestCategory {
    /// Main story quests
    #[default]
    Main,
    /// Side quests
    Side,
    /// Faction-specific quests
    Faction,
    /// Daily quests
    Daily,
    /// Achievement-style quests
    Achievement,
}

impl QuestCategory {
    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Main => "Main Quest",
            Self::Side => "Side Quest",
            Self::Faction => "Faction Quest",
            Self::Daily => "Daily Quest",
            Self::Achievement => "Achievement",
        }
    }

    /// Returns an icon for this category.
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Main => "⭐",
            Self::Side => "◇",
            Self::Faction => "🏴",
            Self::Daily => "📅",
            Self::Achievement => "🏆",
        }
    }

    /// Returns all categories.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Main,
            Self::Side,
            Self::Faction,
            Self::Daily,
            Self::Achievement,
        ]
    }
}

/// Quest data for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDisplayData {
    /// Quest ID
    pub id: QuestId,
    /// Quest title
    pub title: String,
    /// Quest description
    pub description: String,
    /// Current status
    pub status: QuestStatus,
    /// Category
    pub category: QuestCategory,
    /// Difficulty
    pub difficulty: QuestDifficulty,
    /// Objectives
    pub objectives: Vec<QuestObjective>,
    /// Rewards description
    pub rewards: Vec<String>,
    /// Quest giver name
    pub quest_giver: Option<String>,
    /// Recommended level
    pub recommended_level: Option<u32>,
    /// Whether this quest is tracked on HUD
    pub tracked: bool,
}

impl QuestDisplayData {
    /// Creates new quest display data.
    #[must_use]
    pub fn new(id: QuestId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: String::new(),
            status: QuestStatus::Available,
            category: QuestCategory::Side,
            difficulty: QuestDifficulty::Normal,
            objectives: Vec::new(),
            rewards: Vec::new(),
            quest_giver: None,
            recommended_level: None,
            tracked: false,
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Sets the category.
    #[must_use]
    pub fn with_category(mut self, category: QuestCategory) -> Self {
        self.category = category;
        self
    }

    /// Sets the difficulty.
    #[must_use]
    pub fn with_difficulty(mut self, difficulty: QuestDifficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Adds an objective.
    #[must_use]
    pub fn with_objective(mut self, objective: QuestObjective) -> Self {
        self.objectives.push(objective);
        self
    }

    /// Adds a reward.
    #[must_use]
    pub fn with_reward(mut self, reward: impl Into<String>) -> Self {
        self.rewards.push(reward.into());
        self
    }

    /// Returns the overall progress (0.0 - 1.0).
    #[must_use]
    pub fn progress(&self) -> f32 {
        let required: Vec<_> = self.objectives.iter().filter(|o| !o.optional).collect();
        if required.is_empty() {
            return if self.status == QuestStatus::Completed {
                1.0
            } else {
                0.0
            };
        }
        let completed = required.iter().filter(|o| o.is_complete()).count();
        completed as f32 / required.len() as f32
    }

    /// Returns the number of completed objectives.
    #[must_use]
    pub fn completed_objectives(&self) -> usize {
        self.objectives.iter().filter(|o| o.is_complete()).count()
    }

    /// Returns visible objectives (non-hidden or revealed).
    #[must_use]
    pub fn visible_objectives(&self) -> Vec<&QuestObjective> {
        self.objectives.iter().filter(|o| !o.hidden).collect()
    }
}

/// Quest progress from gameplay.
#[derive(Debug, Clone, Default)]
pub struct QuestProgress {
    /// Current objective index
    pub current_objective: u32,
    /// Objectives progress map (objective_id -> current count)
    pub objective_progress: HashMap<u32, u32>,
    /// Total time spent on quest in seconds
    pub time_spent: f32,
    /// Whether the quest has been completed
    pub completed: bool,
}

impl QuestProgress {
    /// Creates new quest progress.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets progress for an objective.
    #[must_use]
    pub fn get_objective_progress(&self, id: u32) -> u32 {
        self.objective_progress.get(&id).copied().unwrap_or(0)
    }

    /// Increments objective progress.
    pub fn increment_objective(&mut self, id: u32, amount: u32) {
        *self.objective_progress.entry(id).or_insert(0) += amount;
    }
}

/// UI action triggered by quest UI.
#[derive(Debug, Clone, PartialEq)]
pub enum QuestAction {
    /// Accept a quest
    Accept(QuestId),
    /// Abandon a quest
    Abandon(QuestId),
    /// Track/untrack a quest
    ToggleTracking(QuestId),
    /// View quest details
    ViewDetails(QuestId),
    /// Turn in a completed quest
    TurnIn(QuestId),
    /// Close the quest log
    CloseLog,
    /// Open the quest log
    OpenLog,
    /// Filter by category
    FilterCategory(Option<QuestCategory>),
    /// Filter by status
    FilterStatus(Option<QuestStatus>),
}

/// Quest tracker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestTrackerConfig {
    /// Maximum quests to track on HUD
    pub max_tracked: usize,
    /// Show objective progress bars
    pub show_progress_bars: bool,
    /// Show optional objectives
    pub show_optional: bool,
    /// Tracker opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Tracker width in pixels
    pub width: f32,
    /// Tracker position (X, Y from top-right)
    pub position: (f32, f32),
}

impl Default for QuestTrackerConfig {
    fn default() -> Self {
        Self {
            max_tracked: 3,
            show_progress_bars: true,
            show_optional: false,
            opacity: 0.9,
            width: 300.0,
            position: (20.0, 100.0),
        }
    }
}

/// Quest log configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestLogConfig {
    /// Panel width
    pub width: f32,
    /// Panel height
    pub height: f32,
    /// Show completed quests
    pub show_completed: bool,
    /// Show failed quests
    pub show_failed: bool,
    /// Sort by category first
    pub group_by_category: bool,
}

impl Default for QuestLogConfig {
    fn default() -> Self {
        Self {
            width: 600.0,
            height: 500.0,
            show_completed: true,
            show_failed: false,
            group_by_category: true,
        }
    }
}

/// Quest UI model (state).
#[derive(Debug, Clone)]
pub struct QuestUIModel {
    /// All known quests
    quests: HashMap<QuestId, QuestDisplayData>,
    /// Quest progress
    progress: HashMap<QuestId, QuestProgress>,
    /// Currently selected quest
    selected: Option<QuestId>,
    /// Category filter
    category_filter: Option<QuestCategory>,
    /// Status filter
    status_filter: Option<QuestStatus>,
    /// Tracker configuration
    tracker_config: QuestTrackerConfig,
    /// Log configuration
    log_config: QuestLogConfig,
}

impl Default for QuestUIModel {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestUIModel {
    /// Creates a new quest UI model.
    #[must_use]
    pub fn new() -> Self {
        Self {
            quests: HashMap::new(),
            progress: HashMap::new(),
            selected: None,
            category_filter: None,
            status_filter: None,
            tracker_config: QuestTrackerConfig::default(),
            log_config: QuestLogConfig::default(),
        }
    }

    /// Adds a quest.
    pub fn add_quest(&mut self, quest: QuestDisplayData) {
        let id = quest.id;
        self.quests.insert(id, quest);
        self.progress.insert(id, QuestProgress::new());
    }

    /// Removes a quest.
    pub fn remove_quest(&mut self, id: QuestId) {
        self.quests.remove(&id);
        self.progress.remove(&id);
        if self.selected == Some(id) {
            self.selected = None;
        }
    }

    /// Gets a quest by ID.
    #[must_use]
    pub fn get_quest(&self, id: QuestId) -> Option<&QuestDisplayData> {
        self.quests.get(&id)
    }

    /// Gets a mutable quest by ID.
    pub fn get_quest_mut(&mut self, id: QuestId) -> Option<&mut QuestDisplayData> {
        self.quests.get_mut(&id)
    }

    /// Gets progress for a quest.
    #[must_use]
    pub fn get_progress(&self, id: QuestId) -> Option<&QuestProgress> {
        self.progress.get(&id)
    }

    /// Gets mutable progress for a quest.
    pub fn get_progress_mut(&mut self, id: QuestId) -> Option<&mut QuestProgress> {
        self.progress.get_mut(&id)
    }

    /// Updates objective progress.
    pub fn update_objective(&mut self, quest_id: QuestId, objective_id: u32, current: u32) {
        if let Some(quest) = self.quests.get_mut(&quest_id) {
            if let Some(obj) = quest.objectives.iter_mut().find(|o| o.id == objective_id) {
                obj.current = current;
                obj.complete = current >= obj.target;
            }
        }
        if let Some(progress) = self.progress.get_mut(&quest_id) {
            progress.objective_progress.insert(objective_id, current);
        }
    }

    /// Sets quest status.
    pub fn set_status(&mut self, id: QuestId, status: QuestStatus) {
        if let Some(quest) = self.quests.get_mut(&id) {
            quest.status = status;
        }
        if status == QuestStatus::Completed {
            if let Some(progress) = self.progress.get_mut(&id) {
                progress.completed = true;
            }
        }
    }

    /// Toggles quest tracking.
    pub fn toggle_tracking(&mut self, id: QuestId) {
        // Count tracked before modifying
        let max_tracked = self.tracker_config.max_tracked;
        let current_tracked_count = self
            .quests
            .values()
            .filter(|q| q.tracked && q.status == QuestStatus::Active)
            .count();

        if let Some(quest) = self.quests.get_mut(&id) {
            if quest.tracked {
                // Untracking is always allowed
                quest.tracked = false;
            } else {
                // Only track if under the limit
                if current_tracked_count < max_tracked {
                    quest.tracked = true;
                }
            }
        }
    }

    /// Sets quest tracking.
    pub fn set_tracking(&mut self, id: QuestId, tracked: bool) {
        if let Some(quest) = self.quests.get_mut(&id) {
            quest.tracked = tracked;
        }
    }

    /// Selects a quest.
    pub fn select(&mut self, id: Option<QuestId>) {
        self.selected = id;
    }

    /// Returns the currently selected quest.
    #[must_use]
    pub fn selected(&self) -> Option<QuestId> {
        self.selected
    }

    /// Returns the selected quest data.
    #[must_use]
    pub fn selected_quest(&self) -> Option<&QuestDisplayData> {
        self.selected.and_then(|id| self.quests.get(&id))
    }

    /// Sets category filter.
    pub fn set_category_filter(&mut self, category: Option<QuestCategory>) {
        self.category_filter = category;
    }

    /// Sets status filter.
    pub fn set_status_filter(&mut self, status: Option<QuestStatus>) {
        self.status_filter = status;
    }

    /// Returns filtered quests.
    #[must_use]
    pub fn filtered_quests(&self) -> Vec<&QuestDisplayData> {
        self.quests
            .values()
            .filter(|q| {
                if let Some(cat) = self.category_filter {
                    if q.category != cat {
                        return false;
                    }
                }
                if let Some(status) = self.status_filter {
                    if q.status != status {
                        return false;
                    }
                }
                // Apply log config filters
                if !self.log_config.show_completed && q.status == QuestStatus::Completed {
                    return false;
                }
                if !self.log_config.show_failed && q.status == QuestStatus::Failed {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Returns tracked quests.
    #[must_use]
    pub fn tracked_quests(&self) -> Vec<&QuestDisplayData> {
        self.quests
            .values()
            .filter(|q| q.tracked && q.status == QuestStatus::Active)
            .take(self.tracker_config.max_tracked)
            .collect()
    }

    /// Returns active quests.
    #[must_use]
    pub fn active_quests(&self) -> Vec<&QuestDisplayData> {
        self.quests
            .values()
            .filter(|q| q.status == QuestStatus::Active)
            .collect()
    }

    /// Returns all quests.
    #[must_use]
    pub fn all_quests(&self) -> Vec<&QuestDisplayData> {
        self.quests.values().collect()
    }

    /// Returns quests count by status.
    #[must_use]
    pub fn quest_counts(&self) -> HashMap<QuestStatus, usize> {
        let mut counts = HashMap::new();
        for quest in self.quests.values() {
            *counts.entry(quest.status).or_insert(0) += 1;
        }
        counts
    }

    /// Returns quests grouped by category.
    #[must_use]
    pub fn quests_by_category(&self) -> HashMap<QuestCategory, Vec<&QuestDisplayData>> {
        let mut grouped: HashMap<QuestCategory, Vec<&QuestDisplayData>> = HashMap::new();
        for quest in self.quests.values() {
            grouped.entry(quest.category).or_default().push(quest);
        }
        grouped
    }

    /// Returns tracker config.
    #[must_use]
    pub fn tracker_config(&self) -> &QuestTrackerConfig {
        &self.tracker_config
    }

    /// Returns mutable tracker config.
    pub fn tracker_config_mut(&mut self) -> &mut QuestTrackerConfig {
        &mut self.tracker_config
    }

    /// Returns log config.
    #[must_use]
    pub fn log_config(&self) -> &QuestLogConfig {
        &self.log_config
    }

    /// Returns mutable log config.
    pub fn log_config_mut(&mut self) -> &mut QuestLogConfig {
        &mut self.log_config
    }

    /// Updates time spent on active quests.
    pub fn update(&mut self, dt: f32) {
        for (id, quest) in &self.quests {
            if quest.status == QuestStatus::Active {
                if let Some(progress) = self.progress.get_mut(id) {
                    progress.time_spent += dt;
                }
            }
        }
    }
}

/// Quest UI (egui widget).
#[derive(Debug)]
pub struct QuestUI {
    /// Whether quest log is open
    log_open: bool,
    /// Whether tracker is visible
    tracker_visible: bool,
    /// Pending actions
    actions: Vec<QuestAction>,
}

impl Default for QuestUI {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestUI {
    /// Creates a new quest UI.
    #[must_use]
    pub fn new() -> Self {
        Self {
            log_open: false,
            tracker_visible: true,
            actions: Vec::new(),
        }
    }

    /// Opens the quest log.
    pub fn open_log(&mut self) {
        self.log_open = true;
    }

    /// Closes the quest log.
    pub fn close_log(&mut self) {
        self.log_open = false;
    }

    /// Toggles the quest log.
    pub fn toggle_log(&mut self) {
        self.log_open = !self.log_open;
    }

    /// Returns whether the log is open.
    #[must_use]
    pub fn is_log_open(&self) -> bool {
        self.log_open
    }

    /// Shows/hides the tracker.
    pub fn set_tracker_visible(&mut self, visible: bool) {
        self.tracker_visible = visible;
    }

    /// Returns whether the tracker is visible.
    #[must_use]
    pub fn is_tracker_visible(&self) -> bool {
        self.tracker_visible
    }

    /// Drains pending actions.
    pub fn drain_actions(&mut self) -> Vec<QuestAction> {
        std::mem::take(&mut self.actions)
    }

    /// Renders the quest tracker HUD.
    pub fn render_tracker(&mut self, ctx: &egui::Context, model: &QuestUIModel) {
        if !self.tracker_visible {
            return;
        }

        let config = model.tracker_config();
        let tracked = model.tracked_quests();

        if tracked.is_empty() {
            return;
        }

        egui::Area::new(egui::Id::new("quest_tracker"))
            .anchor(
                egui::Align2::RIGHT_TOP,
                egui::vec2(-config.position.0, config.position.1),
            )
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(
                        30,
                        30,
                        30,
                        (config.opacity * 255.0) as u8,
                    ))
                    .rounding(egui::Rounding::same(5.0))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_width(config.width);

                        for quest in tracked {
                            self.render_tracked_quest(ui, quest, config);
                            ui.add_space(8.0);
                        }
                    });
            });
    }

    #[allow(clippy::unused_self)]
    fn render_tracked_quest(
        &self,
        ui: &mut egui::Ui,
        quest: &QuestDisplayData,
        config: &QuestTrackerConfig,
    ) {
        // Quest title
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&quest.title)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
        });

        // Objectives
        for obj in quest.visible_objectives() {
            if obj.optional && !config.show_optional {
                continue;
            }

            ui.horizontal(|ui| {
                // Checkbox-style completion indicator
                let icon = if obj.is_complete() { "☑" } else { "☐" };
                ui.label(egui::RichText::new(icon).size(12.0));

                // Objective text
                let text_color = if obj.is_complete() {
                    egui::Color32::GRAY
                } else {
                    egui::Color32::LIGHT_GRAY
                };

                let mut text = obj.description.clone();
                if obj.optional {
                    text = format!("(Optional) {text}");
                }

                ui.label(egui::RichText::new(&text).size(12.0).color(text_color));

                // Progress if target > 1
                if obj.target > 1 {
                    ui.label(
                        egui::RichText::new(obj.progress_text())
                            .size(12.0)
                            .color(egui::Color32::LIGHT_BLUE),
                    );
                }
            });

            // Progress bar
            if config.show_progress_bars && obj.target > 1 && !obj.is_complete() {
                let progress = obj.progress();
                ui.add(
                    egui::ProgressBar::new(progress)
                        .desired_height(4.0)
                        .fill(egui::Color32::from_rgb(100, 150, 255)),
                );
            }
        }
    }

    /// Renders the quest log panel.
    pub fn render_log(&mut self, ctx: &egui::Context, model: &mut QuestUIModel) {
        if !self.log_open {
            return;
        }

        let config = model.log_config().clone();

        egui::Window::new("Quest Log")
            .default_size(egui::vec2(config.width, config.height))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Header with filters
                ui.horizontal(|ui| {
                    ui.label("Filter:");

                    // Category filter
                    egui::ComboBox::from_id_salt("quest_category_filter")
                        .selected_text(
                            model
                                .category_filter
                                .map_or("All Categories", |c| c.display_name()),
                        )
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(model.category_filter.is_none(), "All Categories")
                                .clicked()
                            {
                                model.set_category_filter(None);
                            }
                            for cat in QuestCategory::all() {
                                if ui
                                    .selectable_label(
                                        model.category_filter == Some(*cat),
                                        cat.display_name(),
                                    )
                                    .clicked()
                                {
                                    model.set_category_filter(Some(*cat));
                                }
                            }
                        });

                    // Status filter
                    egui::ComboBox::from_id_salt("quest_status_filter")
                        .selected_text(
                            model
                                .status_filter
                                .map_or("All Status", |s| s.display_name()),
                        )
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(model.status_filter.is_none(), "All Status")
                                .clicked()
                            {
                                model.set_status_filter(None);
                            }
                            for status in [
                                QuestStatus::Active,
                                QuestStatus::Available,
                                QuestStatus::Completed,
                            ] {
                                if ui
                                    .selectable_label(
                                        model.status_filter == Some(status),
                                        status.display_name(),
                                    )
                                    .clicked()
                                {
                                    model.set_status_filter(Some(status));
                                }
                            }
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            self.actions.push(QuestAction::CloseLog);
                        }
                    });
                });

                ui.separator();

                // Quest list and details
                ui.columns(2, |columns| {
                    // Left: Quest list
                    egui::ScrollArea::vertical().id_salt("quest_list").show(
                        &mut columns[0],
                        |ui| {
                            let quests = model.filtered_quests();

                            if config.group_by_category {
                                let grouped = model.quests_by_category();
                                for cat in QuestCategory::all() {
                                    if let Some(cat_quests) = grouped.get(cat) {
                                        let filtered: Vec<_> = cat_quests
                                            .iter()
                                            .filter(|q| quests.iter().any(|fq| fq.id == q.id))
                                            .collect();

                                        if filtered.is_empty() {
                                            continue;
                                        }

                                        ui.collapsing(
                                            format!(
                                                "{} {} ({})",
                                                cat.icon(),
                                                cat.display_name(),
                                                filtered.len()
                                            ),
                                            |ui| {
                                                for quest in filtered {
                                                    self.render_quest_list_item(
                                                        ui,
                                                        quest,
                                                        model.selected(),
                                                    );
                                                }
                                            },
                                        );
                                    }
                                }
                            } else {
                                for quest in &quests {
                                    self.render_quest_list_item(ui, quest, model.selected());
                                }
                            }
                        },
                    );

                    // Right: Quest details
                    egui::ScrollArea::vertical().id_salt("quest_details").show(
                        &mut columns[1],
                        |ui| {
                            if let Some(quest) = model.selected_quest() {
                                self.render_quest_details(ui, quest);
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.label("Select a quest to view details");
                                });
                            }
                        },
                    );
                });
            });

        // Handle close action
        if self
            .actions
            .iter()
            .any(|a| matches!(a, QuestAction::CloseLog))
        {
            self.log_open = false;
        }
    }

    fn render_quest_list_item(
        &mut self,
        ui: &mut egui::Ui,
        quest: &QuestDisplayData,
        selected: Option<QuestId>,
    ) {
        let is_selected = selected == Some(quest.id);

        let frame = if is_selected {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(60, 80, 120))
                .rounding(egui::Rounding::same(3.0))
                .inner_margin(egui::Margin::same(5.0))
        } else {
            egui::Frame::none().inner_margin(egui::Margin::same(5.0))
        };

        frame.show(ui, |ui| {
            if ui
                .add(
                    egui::Label::new(egui::RichText::new(&quest.title).color(if is_selected {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::LIGHT_GRAY
                    }))
                    .sense(egui::Sense::click()),
                )
                .clicked()
            {
                self.actions.push(QuestAction::ViewDetails(quest.id));
            }

            ui.horizontal(|ui| {
                // Status badge
                let status_color = match quest.status {
                    QuestStatus::Active => egui::Color32::from_rgb(100, 200, 100),
                    QuestStatus::Completed => egui::Color32::from_rgb(200, 200, 100),
                    QuestStatus::Failed => egui::Color32::from_rgb(200, 100, 100),
                    _ => egui::Color32::GRAY,
                };
                ui.label(
                    egui::RichText::new(quest.status.display_name())
                        .size(10.0)
                        .color(status_color),
                );

                // Progress
                if quest.status == QuestStatus::Active {
                    let progress = (quest.progress() * 100.0) as u32;
                    ui.label(egui::RichText::new(format!("{progress}%")).size(10.0));
                }

                // Tracking indicator
                if quest.tracked {
                    ui.label(egui::RichText::new("📍").size(10.0));
                }
            });
        });
    }

    fn render_quest_details(&mut self, ui: &mut egui::Ui, quest: &QuestDisplayData) {
        // Title
        ui.heading(&quest.title);

        // Category and difficulty
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} {}",
                quest.category.icon(),
                quest.category.display_name()
            ));
            ui.separator();
            let diff_color = quest.difficulty.color();
            ui.label(egui::RichText::new(quest.difficulty.display_name()).color(
                egui::Color32::from_rgb(diff_color[0], diff_color[1], diff_color[2]),
            ));
        });

        ui.separator();

        // Description
        ui.label(&quest.description);

        ui.separator();

        // Objectives
        ui.label(egui::RichText::new("Objectives").strong());
        for obj in &quest.objectives {
            if obj.hidden {
                continue;
            }

            ui.horizontal(|ui| {
                let icon = if obj.is_complete() { "✓" } else { "○" };
                let color = if obj.is_complete() {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::WHITE
                };
                ui.label(egui::RichText::new(icon).color(color));

                let mut text = format!("{} {}", obj.objective_type.icon(), obj.description);
                if obj.optional {
                    text = format!("(Optional) {text}");
                }
                ui.label(text);

                if obj.target > 1 {
                    ui.label(obj.progress_text());
                }
            });
        }

        // Rewards
        if !quest.rewards.is_empty() {
            ui.separator();
            ui.label(egui::RichText::new("Rewards").strong());
            for reward in &quest.rewards {
                ui.label(format!("• {reward}"));
            }
        }

        ui.separator();

        // Actions
        ui.horizontal(|ui| match quest.status {
            QuestStatus::Available => {
                if ui.button("Accept Quest").clicked() {
                    self.actions.push(QuestAction::Accept(quest.id));
                }
            },
            QuestStatus::Active => {
                let track_text = if quest.tracked { "Untrack" } else { "Track" };
                if ui.button(track_text).clicked() {
                    self.actions.push(QuestAction::ToggleTracking(quest.id));
                }
                if ui.button("Abandon").clicked() {
                    self.actions.push(QuestAction::Abandon(quest.id));
                }
            },
            QuestStatus::Completed => {
                if ui.button("Turn In").clicked() {
                    self.actions.push(QuestAction::TurnIn(quest.id));
                }
            },
            _ => {},
        });
    }

    /// Handles an action and returns it for processing.
    pub fn handle_action(&mut self, action: QuestAction, model: &mut QuestUIModel) {
        match &action {
            QuestAction::ViewDetails(id) => {
                model.select(Some(*id));
            },
            QuestAction::ToggleTracking(id) => {
                model.toggle_tracking(*id);
            },
            QuestAction::FilterCategory(cat) => {
                model.set_category_filter(*cat);
            },
            QuestAction::FilterStatus(status) => {
                model.set_status_filter(*status);
            },
            QuestAction::CloseLog => {
                self.log_open = false;
            },
            QuestAction::OpenLog => {
                self.log_open = true;
            },
            _ => {},
        }
        self.actions.push(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_id() {
        let id = QuestId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_quest_status_properties() {
        assert!(QuestStatus::Active.is_active());
        assert!(!QuestStatus::Completed.is_active());
        assert!(QuestStatus::Completed.is_finished());
        assert!(QuestStatus::Failed.is_finished());
        assert!(!QuestStatus::Active.is_finished());
    }

    #[test]
    fn test_quest_status_display() {
        assert_eq!(QuestStatus::Active.display_name(), "In Progress");
        assert_eq!(QuestStatus::Completed.display_name(), "Completed");
    }

    #[test]
    fn test_objective_type_icons() {
        assert_eq!(ObjectiveType::Kill.icon(), "⚔");
        assert_eq!(ObjectiveType::Collect.icon(), "📦");
        assert_eq!(ObjectiveType::Talk.icon(), "💬");
    }

    #[test]
    fn test_quest_objective_new() {
        let obj = QuestObjective::new(1, ObjectiveType::Kill, "Defeat enemies");
        assert_eq!(obj.id, 1);
        assert_eq!(obj.description, "Defeat enemies");
        assert_eq!(obj.current, 0);
        assert_eq!(obj.target, 1);
        assert!(!obj.optional);
    }

    #[test]
    fn test_quest_objective_progress() {
        let mut obj =
            QuestObjective::new(1, ObjectiveType::Collect, "Collect items").with_target(10);

        assert!((obj.progress() - 0.0).abs() < 0.001);

        obj.current = 5;
        assert!((obj.progress() - 0.5).abs() < 0.001);

        obj.current = 10;
        assert!((obj.progress() - 1.0).abs() < 0.001);
        assert!(obj.is_complete());
    }

    #[test]
    fn test_quest_objective_progress_text() {
        let obj = QuestObjective::new(1, ObjectiveType::Talk, "Talk to NPC");
        assert_eq!(obj.progress_text(), "○");

        let mut obj2 =
            QuestObjective::new(2, ObjectiveType::Collect, "Collect items").with_target(5);
        obj2.current = 2;
        assert_eq!(obj2.progress_text(), "2/5");
    }

    #[test]
    fn test_quest_difficulty_colors() {
        let easy = QuestDifficulty::Easy.color();
        assert_eq!(easy[3], 255); // Alpha is full

        let epic = QuestDifficulty::Epic.color();
        assert_eq!(epic[0], 150); // Purple-ish
    }

    #[test]
    fn test_quest_category_all() {
        let all = QuestCategory::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&QuestCategory::Main));
        assert!(all.contains(&QuestCategory::Side));
    }

    #[test]
    fn test_quest_display_data_new() {
        let quest = QuestDisplayData::new(QuestId::new(1), "Test Quest")
            .with_description("A test quest")
            .with_category(QuestCategory::Main)
            .with_difficulty(QuestDifficulty::Hard);

        assert_eq!(quest.title, "Test Quest");
        assert_eq!(quest.description, "A test quest");
        assert_eq!(quest.category, QuestCategory::Main);
        assert_eq!(quest.difficulty, QuestDifficulty::Hard);
    }

    #[test]
    fn test_quest_display_data_progress() {
        let quest = QuestDisplayData::new(QuestId::new(1), "Test")
            .with_objective(QuestObjective::new(1, ObjectiveType::Kill, "Kill").with_target(5))
            .with_objective(
                QuestObjective::new(2, ObjectiveType::Collect, "Collect").with_target(5),
            );

        assert!((quest.progress() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_quest_progress() {
        let mut progress = QuestProgress::new();
        assert_eq!(progress.get_objective_progress(1), 0);

        progress.increment_objective(1, 3);
        assert_eq!(progress.get_objective_progress(1), 3);

        progress.increment_objective(1, 2);
        assert_eq!(progress.get_objective_progress(1), 5);
    }

    #[test]
    fn test_quest_action_equality() {
        let a1 = QuestAction::Accept(QuestId::new(1));
        let a2 = QuestAction::Accept(QuestId::new(1));
        assert_eq!(a1, a2);

        let a3 = QuestAction::Accept(QuestId::new(2));
        assert_ne!(a1, a3);
    }

    #[test]
    fn test_quest_tracker_config_defaults() {
        let config = QuestTrackerConfig::default();
        assert_eq!(config.max_tracked, 3);
        assert!(config.show_progress_bars);
        assert!(!config.show_optional);
    }

    #[test]
    fn test_quest_log_config_defaults() {
        let config = QuestLogConfig::default();
        assert!(config.show_completed);
        assert!(!config.show_failed);
        assert!(config.group_by_category);
    }

    #[test]
    fn test_quest_ui_model_new() {
        let model = QuestUIModel::new();
        assert!(model.all_quests().is_empty());
        assert!(model.selected().is_none());
    }

    #[test]
    fn test_quest_ui_model_add_remove() {
        let mut model = QuestUIModel::new();
        let quest = QuestDisplayData::new(QuestId::new(1), "Test Quest");

        model.add_quest(quest);
        assert_eq!(model.all_quests().len(), 1);
        assert!(model.get_quest(QuestId::new(1)).is_some());

        model.remove_quest(QuestId::new(1));
        assert!(model.all_quests().is_empty());
    }

    #[test]
    fn test_quest_ui_model_tracking() {
        let mut model = QuestUIModel::new();
        let mut quest = QuestDisplayData::new(QuestId::new(1), "Test");
        quest.status = QuestStatus::Active;
        model.add_quest(quest);

        assert!(model.tracked_quests().is_empty());

        model.toggle_tracking(QuestId::new(1));
        assert_eq!(model.tracked_quests().len(), 1);

        model.toggle_tracking(QuestId::new(1));
        assert!(model.tracked_quests().is_empty());
    }

    #[test]
    fn test_quest_ui_model_filter() {
        let mut model = QuestUIModel::new();

        let q1 =
            QuestDisplayData::new(QuestId::new(1), "Main Quest").with_category(QuestCategory::Main);
        let mut q2 =
            QuestDisplayData::new(QuestId::new(2), "Side Quest").with_category(QuestCategory::Side);
        q2.status = QuestStatus::Active;

        model.add_quest(q1);
        model.add_quest(q2);

        assert_eq!(model.filtered_quests().len(), 2);

        model.set_category_filter(Some(QuestCategory::Main));
        assert_eq!(model.filtered_quests().len(), 1);
        assert_eq!(model.filtered_quests()[0].title, "Main Quest");
    }

    #[test]
    fn test_quest_ui_model_update_objective() {
        let mut model = QuestUIModel::new();
        let quest = QuestDisplayData::new(QuestId::new(1), "Test").with_objective(
            QuestObjective::new(1, ObjectiveType::Collect, "Collect").with_target(5),
        );

        model.add_quest(quest);
        model.update_objective(QuestId::new(1), 1, 3);

        let quest = model.get_quest(QuestId::new(1)).expect("quest");
        assert_eq!(quest.objectives[0].current, 3);
    }

    #[test]
    fn test_quest_ui_model_set_status() {
        let mut model = QuestUIModel::new();
        let quest = QuestDisplayData::new(QuestId::new(1), "Test");
        model.add_quest(quest);

        model.set_status(QuestId::new(1), QuestStatus::Active);
        assert_eq!(
            model.get_quest(QuestId::new(1)).expect("quest").status,
            QuestStatus::Active
        );

        model.set_status(QuestId::new(1), QuestStatus::Completed);
        let progress = model.get_progress(QuestId::new(1)).expect("progress");
        assert!(progress.completed);
    }

    #[test]
    fn test_quest_ui_model_quest_counts() {
        let mut model = QuestUIModel::new();

        let mut q1 = QuestDisplayData::new(QuestId::new(1), "Q1");
        q1.status = QuestStatus::Active;
        let mut q2 = QuestDisplayData::new(QuestId::new(2), "Q2");
        q2.status = QuestStatus::Active;
        let q3 = QuestDisplayData::new(QuestId::new(3), "Q3"); // Available

        model.add_quest(q1);
        model.add_quest(q2);
        model.add_quest(q3);

        let counts = model.quest_counts();
        assert_eq!(counts.get(&QuestStatus::Active), Some(&2));
        assert_eq!(counts.get(&QuestStatus::Available), Some(&1));
    }

    #[test]
    fn test_quest_ui_new() {
        let ui = QuestUI::new();
        assert!(!ui.is_log_open());
        assert!(ui.is_tracker_visible());
    }

    #[test]
    fn test_quest_ui_toggle_log() {
        let mut ui = QuestUI::new();
        assert!(!ui.is_log_open());

        ui.toggle_log();
        assert!(ui.is_log_open());

        ui.toggle_log();
        assert!(!ui.is_log_open());
    }

    #[test]
    fn test_quest_ui_open_close_log() {
        let mut ui = QuestUI::new();

        ui.open_log();
        assert!(ui.is_log_open());

        ui.close_log();
        assert!(!ui.is_log_open());
    }

    #[test]
    fn test_quest_ui_tracker_visibility() {
        let mut ui = QuestUI::new();
        assert!(ui.is_tracker_visible());

        ui.set_tracker_visible(false);
        assert!(!ui.is_tracker_visible());
    }

    #[test]
    fn test_quest_ui_drain_actions() {
        let mut ui = QuestUI::new();
        let mut model = QuestUIModel::new();

        ui.handle_action(QuestAction::OpenLog, &mut model);
        ui.handle_action(
            QuestAction::FilterCategory(Some(QuestCategory::Main)),
            &mut model,
        );

        let actions = ui.drain_actions();
        assert_eq!(actions.len(), 2);

        let actions2 = ui.drain_actions();
        assert!(actions2.is_empty());
    }

    #[test]
    fn test_quest_ui_handle_view_details() {
        let mut ui = QuestUI::new();
        let mut model = QuestUIModel::new();
        let quest = QuestDisplayData::new(QuestId::new(1), "Test");
        model.add_quest(quest);

        ui.handle_action(QuestAction::ViewDetails(QuestId::new(1)), &mut model);
        assert_eq!(model.selected(), Some(QuestId::new(1)));
    }

    #[test]
    fn test_quest_ui_model_update_time() {
        let mut model = QuestUIModel::new();
        let mut quest = QuestDisplayData::new(QuestId::new(1), "Test");
        quest.status = QuestStatus::Active;
        model.add_quest(quest);

        model.update(1.0);
        let progress = model.get_progress(QuestId::new(1)).expect("progress");
        assert!((progress.time_spent - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quest_visible_objectives() {
        let quest = QuestDisplayData::new(QuestId::new(1), "Test")
            .with_objective(QuestObjective::new(1, ObjectiveType::Kill, "Visible"))
            .with_objective(
                QuestObjective::new(2, ObjectiveType::Kill, "Hidden").with_hidden(true),
            );

        let visible = quest.visible_objectives();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].description, "Visible");
    }

    #[test]
    fn test_quest_completed_objectives() {
        let mut quest = QuestDisplayData::new(QuestId::new(1), "Test")
            .with_objective(QuestObjective::new(1, ObjectiveType::Kill, "A").with_target(1))
            .with_objective(QuestObjective::new(2, ObjectiveType::Kill, "B").with_target(1));

        quest.objectives[0].complete = true;
        assert_eq!(quest.completed_objectives(), 1);
    }
}
