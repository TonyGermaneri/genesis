//! Save/Load Menu UI
//!
//! Main menu interface for save/load functionality with slot grid,
//! new game button, and continue last save button.

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};

/// Unique identifier for a save slot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SaveSlotId(pub u32);

impl SaveSlotId {
    /// Create a new save slot ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the slot number (1-indexed for display)
    pub fn slot_number(&self) -> u32 {
        self.0 + 1
    }
}

impl std::fmt::Display for SaveSlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot {}", self.slot_number())
    }
}

/// State of a save slot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SlotState {
    /// Slot is empty and available
    #[default]
    Empty,
    /// Slot contains a valid save
    Occupied,
    /// Save is corrupted or invalid
    Corrupted,
    /// Slot is currently being written to
    Saving,
    /// Slot is locked (e.g., DLC not owned)
    Locked,
}

impl SlotState {
    /// Get display name for the state
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Occupied => "Occupied",
            Self::Corrupted => "Corrupted",
            Self::Saving => "Saving...",
            Self::Locked => "Locked",
        }
    }

    /// Get color for the state
    pub fn color(&self) -> Color32 {
        match self {
            Self::Empty => Color32::from_rgb(100, 100, 100),
            Self::Occupied => Color32::from_rgb(100, 200, 100),
            Self::Corrupted => Color32::from_rgb(200, 100, 100),
            Self::Saving => Color32::from_rgb(200, 200, 100),
            Self::Locked => Color32::from_rgb(150, 100, 150),
        }
    }

    /// Check if slot can be saved to
    pub fn can_save(&self) -> bool {
        matches!(self, Self::Empty | Self::Occupied)
    }

    /// Check if slot can be loaded from
    pub fn can_load(&self) -> bool {
        matches!(self, Self::Occupied)
    }
}

/// Brief information about a save slot for the grid display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlotInfo {
    /// Slot identifier
    pub id: SaveSlotId,
    /// Current state of the slot
    pub state: SlotState,
    /// Player character name (if occupied)
    pub player_name: Option<String>,
    /// Player level (if occupied)
    pub player_level: Option<u32>,
    /// Total playtime in seconds
    pub playtime_seconds: Option<u64>,
    /// Last saved timestamp (Unix epoch seconds)
    pub last_saved: Option<u64>,
    /// World/save name
    pub world_name: Option<String>,
    /// Whether this is the most recent save
    pub is_most_recent: bool,
    /// Optional thumbnail data (RGBA bytes, 64x64)
    pub thumbnail: Option<Vec<u8>>,
}

impl SaveSlotInfo {
    /// Create a new empty save slot
    pub fn empty(id: SaveSlotId) -> Self {
        Self {
            id,
            state: SlotState::Empty,
            player_name: None,
            player_level: None,
            playtime_seconds: None,
            last_saved: None,
            world_name: None,
            is_most_recent: false,
            thumbnail: None,
        }
    }

    /// Create an occupied save slot
    pub fn occupied(
        id: SaveSlotId,
        player_name: impl Into<String>,
        player_level: u32,
        playtime_seconds: u64,
    ) -> Self {
        Self {
            id,
            state: SlotState::Occupied,
            player_name: Some(player_name.into()),
            player_level: Some(player_level),
            playtime_seconds: Some(playtime_seconds),
            last_saved: None,
            world_name: None,
            is_most_recent: false,
            thumbnail: None,
        }
    }

    /// Set the world name
    pub fn with_world_name(mut self, name: impl Into<String>) -> Self {
        self.world_name = Some(name.into());
        self
    }

    /// Set the last saved timestamp
    pub fn with_last_saved(mut self, timestamp: u64) -> Self {
        self.last_saved = Some(timestamp);
        self
    }

    /// Mark as most recent save
    pub fn with_most_recent(mut self, is_recent: bool) -> Self {
        self.is_most_recent = is_recent;
        self
    }

    /// Set thumbnail data
    pub fn with_thumbnail(mut self, data: Vec<u8>) -> Self {
        self.thumbnail = Some(data);
        self
    }

    /// Format playtime as HH:MM:SS
    pub fn format_playtime(&self) -> String {
        match self.playtime_seconds {
            Some(secs) => {
                let hours = secs / 3600;
                let minutes = (secs % 3600) / 60;
                let seconds = secs % 60;
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            },
            None => String::from("--:--:--"),
        }
    }

    /// Format last saved as relative time
    pub fn format_last_saved(&self, current_time: u64) -> String {
        match self.last_saved {
            Some(saved) => {
                if current_time < saved {
                    return String::from("Just now");
                }
                let diff = current_time - saved;
                if diff < 60 {
                    String::from("Just now")
                } else if diff < 3600 {
                    let mins = diff / 60;
                    if mins == 1 {
                        String::from("1 minute ago")
                    } else {
                        format!("{mins} minutes ago")
                    }
                } else if diff < 86400 {
                    let hours = diff / 3600;
                    if hours == 1 {
                        String::from("1 hour ago")
                    } else {
                        format!("{hours} hours ago")
                    }
                } else {
                    let days = diff / 86400;
                    if days == 1 {
                        String::from("1 day ago")
                    } else {
                        format!("{days} days ago")
                    }
                }
            },
            None => String::from("Never"),
        }
    }

    /// Get display title for the slot
    pub fn display_title(&self) -> String {
        match &self.player_name {
            Some(name) => name.clone(),
            None => format!("Slot {}", self.id.slot_number()),
        }
    }

    /// Get display subtitle for the slot
    pub fn display_subtitle(&self) -> String {
        match self.state {
            SlotState::Occupied => {
                let level = self.player_level.unwrap_or(1);
                let world = self.world_name.as_deref().unwrap_or("Unknown World");
                format!("Level {level} - {world}")
            },
            _ => self.state.display_name().to_string(),
        }
    }
}

/// Menu mode for save/load operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MenuMode {
    /// Main menu (no specific action)
    #[default]
    Main,
    /// Save mode - selecting slot to save to
    Save,
    /// Load mode - selecting slot to load from
    Load,
}

impl MenuMode {
    /// Get the title for this mode
    pub fn title(&self) -> &'static str {
        match self {
            Self::Main => "Main Menu",
            Self::Save => "Save Game",
            Self::Load => "Load Game",
        }
    }

    /// Get the action button text
    pub fn action_text(&self) -> &'static str {
        match self {
            Self::Main => "Select",
            Self::Save => "Save",
            Self::Load => "Load",
        }
    }
}

/// Actions generated by the save menu
#[derive(Debug, Clone, PartialEq)]
pub enum SaveMenuAction {
    /// Request to start a new game
    NewGame,
    /// Request to continue from most recent save
    ContinueGame,
    /// Request to save to a specific slot
    SaveToSlot(SaveSlotId),
    /// Request to load from a specific slot
    LoadFromSlot(SaveSlotId),
    /// Request to show save management for a slot
    ManageSlot(SaveSlotId),
    /// Request to close the menu
    Close,
    /// Changed menu mode
    ModeChanged(MenuMode),
    /// Slot was selected (for preview)
    SlotSelected(SaveSlotId),
    /// Slot was deselected
    SlotDeselected,
}

/// Configuration for save menu appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMenuConfig {
    /// Number of save slots
    pub slot_count: usize,
    /// Grid columns for slot display
    pub grid_columns: usize,
    /// Size of slot thumbnails
    pub thumbnail_size: [f32; 2],
    /// Slot card size
    pub slot_size: [f32; 2],
    /// Spacing between slots
    pub slot_spacing: f32,
    /// Whether to show thumbnails
    pub show_thumbnails: bool,
    /// Whether to show playtime
    pub show_playtime: bool,
    /// Whether to show last saved time
    pub show_last_saved: bool,
    /// Color for selected slot border
    pub selected_color: [u8; 4],
    /// Color for most recent save highlight
    pub recent_color: [u8; 4],
    /// Whether to allow overwriting saves without confirmation
    pub allow_quick_overwrite: bool,
}

impl Default for SaveMenuConfig {
    fn default() -> Self {
        Self {
            slot_count: 8,
            grid_columns: 4,
            thumbnail_size: [128.0, 72.0],
            slot_size: [160.0, 140.0],
            slot_spacing: 10.0,
            show_thumbnails: true,
            show_playtime: true,
            show_last_saved: true,
            selected_color: [100, 180, 255, 255],
            recent_color: [255, 215, 0, 255],
            allow_quick_overwrite: false,
        }
    }
}

/// Save menu state and UI
#[derive(Debug)]
pub struct SaveMenu {
    /// Whether the menu is open
    open: bool,
    /// Current menu mode
    mode: MenuMode,
    /// Available save slots
    slots: Vec<SaveSlotInfo>,
    /// Currently selected slot
    selected_slot: Option<SaveSlotId>,
    /// Pending actions to be processed
    actions: Vec<SaveMenuAction>,
    /// Menu configuration
    config: SaveMenuConfig,
    /// Current time for relative timestamps
    current_time: u64,
    /// Whether there's a game in progress that can be saved
    can_save_current: bool,
    /// Whether there's a recent save to continue
    has_recent_save: bool,
}

impl SaveMenu {
    /// Create a new save menu
    pub fn new(config: SaveMenuConfig) -> Self {
        let slot_count = config.slot_count;
        Self {
            open: false,
            mode: MenuMode::Main,
            slots: (0..slot_count)
                .map(|i| SaveSlotInfo::empty(SaveSlotId::new(i as u32)))
                .collect(),
            selected_slot: None,
            actions: Vec::new(),
            config,
            current_time: 0,
            can_save_current: false,
            has_recent_save: false,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SaveMenuConfig::default())
    }

    /// Check if menu is open
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Open the menu in a specific mode
    pub fn open(&mut self, mode: MenuMode) {
        self.open = true;
        self.mode = mode;
        self.actions.push(SaveMenuAction::ModeChanged(mode));
    }

    /// Open for saving
    pub fn open_save(&mut self) {
        self.open(MenuMode::Save);
    }

    /// Open for loading
    pub fn open_load(&mut self) {
        self.open(MenuMode::Load);
    }

    /// Close the menu
    pub fn close(&mut self) {
        self.open = false;
        self.selected_slot = None;
        self.actions.push(SaveMenuAction::Close);
    }

    /// Toggle menu visibility
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open(MenuMode::Main);
        }
    }

    /// Get current mode
    pub fn mode(&self) -> MenuMode {
        self.mode
    }

    /// Set menu mode
    pub fn set_mode(&mut self, mode: MenuMode) {
        self.mode = mode;
        self.actions.push(SaveMenuAction::ModeChanged(mode));
    }

    /// Get selected slot
    pub fn selected_slot(&self) -> Option<SaveSlotId> {
        self.selected_slot
    }

    /// Select a slot
    pub fn select_slot(&mut self, id: SaveSlotId) {
        self.selected_slot = Some(id);
        self.actions.push(SaveMenuAction::SlotSelected(id));
    }

    /// Deselect current slot
    pub fn deselect_slot(&mut self) {
        if self.selected_slot.is_some() {
            self.selected_slot = None;
            self.actions.push(SaveMenuAction::SlotDeselected);
        }
    }

    /// Update slot information
    pub fn set_slot(&mut self, slot: SaveSlotInfo) {
        if let Some(existing) = self.slots.iter_mut().find(|s| s.id == slot.id) {
            *existing = slot;
        }
    }

    /// Update all slots
    pub fn set_slots(&mut self, slots: Vec<SaveSlotInfo>) {
        self.slots = slots;
    }

    /// Get slot by ID
    pub fn get_slot(&self, id: SaveSlotId) -> Option<&SaveSlotInfo> {
        self.slots.iter().find(|s| s.id == id)
    }

    /// Get all slots
    pub fn slots(&self) -> &[SaveSlotInfo] {
        &self.slots
    }

    /// Get most recent save slot
    pub fn most_recent_slot(&self) -> Option<&SaveSlotInfo> {
        self.slots.iter().find(|s| s.is_most_recent)
    }

    /// Set current time for relative timestamps
    pub fn set_current_time(&mut self, time: u64) {
        self.current_time = time;
    }

    /// Set whether current game can be saved
    pub fn set_can_save(&mut self, can_save: bool) {
        self.can_save_current = can_save;
    }

    /// Set whether there's a recent save to continue
    pub fn set_has_recent_save(&mut self, has_save: bool) {
        self.has_recent_save = has_save;
    }

    /// Get configuration
    pub fn config(&self) -> &SaveMenuConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: SaveMenuConfig) {
        self.config = config;
    }

    /// Drain pending actions
    pub fn drain_actions(&mut self) -> Vec<SaveMenuAction> {
        std::mem::take(&mut self.actions)
    }

    /// Confirm action on selected slot
    pub fn confirm_action(&mut self) {
        if let Some(id) = self.selected_slot {
            if let Some(slot) = self.get_slot(id) {
                match self.mode {
                    MenuMode::Save => {
                        if slot.state.can_save() {
                            self.actions.push(SaveMenuAction::SaveToSlot(id));
                        }
                    },
                    MenuMode::Load => {
                        if slot.state.can_load() {
                            self.actions.push(SaveMenuAction::LoadFromSlot(id));
                        }
                    },
                    MenuMode::Main => {
                        self.actions.push(SaveMenuAction::ManageSlot(id));
                    },
                }
            }
        }
    }

    /// Request to start a new game
    pub fn new_game(&mut self) {
        self.actions.push(SaveMenuAction::NewGame);
    }

    /// Request to continue from most recent save
    pub fn continue_game(&mut self) {
        self.actions.push(SaveMenuAction::ContinueGame);
    }

    /// Render the save menu
    pub fn show(&mut self, ui: &mut Ui) {
        if !self.open {
            return;
        }

        egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(20, 20, 30, 240))
            .inner_margin(20.0)
            .rounding(8.0)
            .show(ui, |ui| {
                self.show_content(ui);
            });
    }

    fn show_content(&mut self, ui: &mut Ui) {
        // Title
        ui.heading(self.mode.title());
        ui.add_space(10.0);

        // Main menu buttons (only in main mode)
        if self.mode == MenuMode::Main {
            self.show_main_buttons(ui);
            ui.add_space(20.0);
        }

        // Mode selector (when not in main mode)
        if self.mode != MenuMode::Main {
            self.show_mode_selector(ui);
            ui.add_space(10.0);
        }

        // Slot grid
        self.show_slot_grid(ui);

        ui.add_space(20.0);

        // Action buttons
        self.show_action_buttons(ui);
    }

    fn show_main_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("🎮 New Game").clicked() {
                self.actions.push(SaveMenuAction::NewGame);
            }

            ui.add_enabled_ui(self.has_recent_save, |ui| {
                if ui.button("▶ Continue").clicked() {
                    self.actions.push(SaveMenuAction::ContinueGame);
                }
            });

            ui.add_enabled_ui(self.can_save_current, |ui| {
                if ui.button("💾 Save Game").clicked() {
                    self.set_mode(MenuMode::Save);
                }
            });

            if ui.button("📂 Load Game").clicked() {
                self.set_mode(MenuMode::Load);
            }
        });
    }

    fn show_mode_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.mode == MenuMode::Save, "💾 Save")
                .clicked()
            {
                self.set_mode(MenuMode::Save);
            }
            if ui
                .selectable_label(self.mode == MenuMode::Load, "📂 Load")
                .clicked()
            {
                self.set_mode(MenuMode::Load);
            }
            ui.separator();
            if ui.button("← Back").clicked() {
                self.set_mode(MenuMode::Main);
            }
        });
    }

    fn show_slot_grid(&mut self, ui: &mut Ui) {
        let columns = self.config.grid_columns;
        let slot_size = self.config.slot_size;

        egui::Grid::new("save_slot_grid")
            .spacing([self.config.slot_spacing, self.config.slot_spacing])
            .show(ui, |ui| {
                for (i, slot) in self.slots.clone().iter().enumerate() {
                    self.show_slot_card(ui, slot, slot_size);

                    if (i + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn show_slot_card(&mut self, ui: &mut Ui, slot: &SaveSlotInfo, size: [f32; 2]) {
        let is_selected = self.selected_slot == Some(slot.id);
        let can_interact = match self.mode {
            MenuMode::Save => slot.state.can_save(),
            MenuMode::Load => slot.state.can_load(),
            MenuMode::Main => true,
        };

        let border_color = if is_selected {
            Color32::from_rgba_unmultiplied(
                self.config.selected_color[0],
                self.config.selected_color[1],
                self.config.selected_color[2],
                self.config.selected_color[3],
            )
        } else if slot.is_most_recent {
            Color32::from_rgba_unmultiplied(
                self.config.recent_color[0],
                self.config.recent_color[1],
                self.config.recent_color[2],
                self.config.recent_color[3],
            )
        } else {
            Color32::from_gray(60)
        };

        let bg_color = if can_interact {
            Color32::from_gray(40)
        } else {
            Color32::from_gray(25)
        };

        let response = egui::Frame::none()
            .fill(bg_color)
            .stroke(egui::Stroke::new(
                if is_selected { 2.0 } else { 1.0 },
                border_color,
            ))
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(size[0], size[1]));
                ui.set_max_size(egui::vec2(size[0], size[1]));

                // Slot number badge
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("#{}", slot.id.slot_number()))
                            .color(Color32::from_gray(150))
                            .small(),
                    );
                    if slot.is_most_recent {
                        ui.label(egui::RichText::new("★").color(Color32::GOLD).small());
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(slot.state.display_name())
                                .color(slot.state.color())
                                .small(),
                        );
                    });
                });

                ui.add_space(4.0);

                // Title
                let title_color = if can_interact {
                    Color32::WHITE
                } else {
                    Color32::from_gray(120)
                };
                ui.label(
                    egui::RichText::new(slot.display_title())
                        .color(title_color)
                        .strong(),
                );

                // Subtitle
                ui.label(
                    egui::RichText::new(slot.display_subtitle())
                        .color(Color32::from_gray(160))
                        .small(),
                );

                // Playtime and last saved
                if slot.state == SlotState::Occupied {
                    ui.add_space(4.0);
                    if self.config.show_playtime {
                        ui.label(
                            egui::RichText::new(format!("⏱ {}", slot.format_playtime()))
                                .color(Color32::from_gray(140))
                                .small(),
                        );
                    }
                    if self.config.show_last_saved {
                        ui.label(
                            egui::RichText::new(format!(
                                "📅 {}",
                                slot.format_last_saved(self.current_time)
                            ))
                            .color(Color32::from_gray(140))
                            .small(),
                        );
                    }
                }
            })
            .response;

        if can_interact && response.interact(egui::Sense::click()).clicked() {
            if is_selected {
                self.confirm_action();
            } else {
                self.select_slot(slot.id);
            }
        }
    }

    fn show_action_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Action button based on mode
            let can_action = self.selected_slot.is_some_and(|id| {
                self.get_slot(id).is_some_and(|slot| match self.mode {
                    MenuMode::Save => slot.state.can_save(),
                    MenuMode::Load => slot.state.can_load(),
                    MenuMode::Main => slot.state == SlotState::Occupied,
                })
            });

            ui.add_enabled_ui(can_action, |ui| {
                let action_text = match self.mode {
                    MenuMode::Main => "Manage",
                    MenuMode::Save => "Save Here",
                    MenuMode::Load => "Load",
                };
                if ui.button(action_text).clicked() {
                    self.confirm_action();
                }
            });

            ui.separator();

            if ui.button("Close").clicked() {
                self.close();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_slot_id() {
        let id = SaveSlotId::new(0);
        assert_eq!(id.0, 0);
        assert_eq!(id.slot_number(), 1);
        assert_eq!(format!("{id}"), "Slot 1");
    }

    #[test]
    fn test_slot_state_display_name() {
        assert_eq!(SlotState::Empty.display_name(), "Empty");
        assert_eq!(SlotState::Occupied.display_name(), "Occupied");
        assert_eq!(SlotState::Corrupted.display_name(), "Corrupted");
        assert_eq!(SlotState::Saving.display_name(), "Saving...");
        assert_eq!(SlotState::Locked.display_name(), "Locked");
    }

    #[test]
    fn test_slot_state_can_save() {
        assert!(SlotState::Empty.can_save());
        assert!(SlotState::Occupied.can_save());
        assert!(!SlotState::Corrupted.can_save());
        assert!(!SlotState::Saving.can_save());
        assert!(!SlotState::Locked.can_save());
    }

    #[test]
    fn test_slot_state_can_load() {
        assert!(!SlotState::Empty.can_load());
        assert!(SlotState::Occupied.can_load());
        assert!(!SlotState::Corrupted.can_load());
        assert!(!SlotState::Saving.can_load());
        assert!(!SlotState::Locked.can_load());
    }

    #[test]
    fn test_save_slot_info_empty() {
        let slot = SaveSlotInfo::empty(SaveSlotId::new(0));
        assert_eq!(slot.state, SlotState::Empty);
        assert!(slot.player_name.is_none());
        assert!(slot.player_level.is_none());
        assert!(!slot.is_most_recent);
    }

    #[test]
    fn test_save_slot_info_occupied() {
        let slot = SaveSlotInfo::occupied(SaveSlotId::new(1), "Hero", 25, 3600);
        assert_eq!(slot.state, SlotState::Occupied);
        assert_eq!(slot.player_name, Some("Hero".to_string()));
        assert_eq!(slot.player_level, Some(25));
        assert_eq!(slot.playtime_seconds, Some(3600));
    }

    #[test]
    fn test_save_slot_info_builders() {
        let slot = SaveSlotInfo::occupied(SaveSlotId::new(0), "Test", 10, 7200)
            .with_world_name("Test World")
            .with_last_saved(1000)
            .with_most_recent(true);

        assert_eq!(slot.world_name, Some("Test World".to_string()));
        assert_eq!(slot.last_saved, Some(1000));
        assert!(slot.is_most_recent);
    }

    #[test]
    fn test_format_playtime() {
        let slot = SaveSlotInfo::occupied(SaveSlotId::new(0), "Test", 1, 3661);
        assert_eq!(slot.format_playtime(), "01:01:01");

        let slot2 = SaveSlotInfo::empty(SaveSlotId::new(0));
        assert_eq!(slot2.format_playtime(), "--:--:--");
    }

    #[test]
    fn test_format_last_saved() {
        let slot = SaveSlotInfo::occupied(SaveSlotId::new(0), "Test", 1, 100).with_last_saved(1000);

        assert_eq!(slot.format_last_saved(1030), "Just now");
        assert_eq!(slot.format_last_saved(1120), "2 minutes ago");
        assert_eq!(slot.format_last_saved(4600), "1 hour ago");
        assert_eq!(slot.format_last_saved(8200), "2 hours ago");
        assert_eq!(slot.format_last_saved(90400), "1 day ago");
        assert_eq!(slot.format_last_saved(180000), "2 days ago");

        let empty = SaveSlotInfo::empty(SaveSlotId::new(0));
        assert_eq!(empty.format_last_saved(1000), "Never");
    }

    #[test]
    fn test_display_title_subtitle() {
        let occupied = SaveSlotInfo::occupied(SaveSlotId::new(0), "Hero", 15, 1000)
            .with_world_name("Sanctuary");
        assert_eq!(occupied.display_title(), "Hero");
        assert_eq!(occupied.display_subtitle(), "Level 15 - Sanctuary");

        let empty = SaveSlotInfo::empty(SaveSlotId::new(2));
        assert_eq!(empty.display_title(), "Slot 3");
        assert_eq!(empty.display_subtitle(), "Empty");
    }

    #[test]
    fn test_menu_mode() {
        assert_eq!(MenuMode::Main.title(), "Main Menu");
        assert_eq!(MenuMode::Save.title(), "Save Game");
        assert_eq!(MenuMode::Load.title(), "Load Game");

        assert_eq!(MenuMode::Main.action_text(), "Select");
        assert_eq!(MenuMode::Save.action_text(), "Save");
        assert_eq!(MenuMode::Load.action_text(), "Load");
    }

    #[test]
    fn test_save_menu_config_defaults() {
        let config = SaveMenuConfig::default();
        assert_eq!(config.slot_count, 8);
        assert_eq!(config.grid_columns, 4);
        assert!(config.show_thumbnails);
        assert!(config.show_playtime);
        assert!(!config.allow_quick_overwrite);
    }

    #[test]
    fn test_save_menu_new() {
        let menu = SaveMenu::with_defaults();
        assert!(!menu.is_open());
        assert_eq!(menu.mode(), MenuMode::Main);
        assert!(menu.selected_slot().is_none());
        assert_eq!(menu.slots().len(), 8);
    }

    #[test]
    fn test_save_menu_open_close() {
        let mut menu = SaveMenu::with_defaults();

        menu.open_save();
        assert!(menu.is_open());
        assert_eq!(menu.mode(), MenuMode::Save);

        menu.close();
        assert!(!menu.is_open());

        menu.open_load();
        assert!(menu.is_open());
        assert_eq!(menu.mode(), MenuMode::Load);
    }

    #[test]
    fn test_save_menu_toggle() {
        let mut menu = SaveMenu::with_defaults();

        menu.toggle();
        assert!(menu.is_open());

        menu.toggle();
        assert!(!menu.is_open());
    }

    #[test]
    fn test_save_menu_select_slot() {
        let mut menu = SaveMenu::with_defaults();

        menu.select_slot(SaveSlotId::new(2));
        assert_eq!(menu.selected_slot(), Some(SaveSlotId::new(2)));

        menu.deselect_slot();
        assert!(menu.selected_slot().is_none());
    }

    #[test]
    fn test_save_menu_set_slot() {
        let mut menu = SaveMenu::with_defaults();

        let slot = SaveSlotInfo::occupied(SaveSlotId::new(0), "Updated", 50, 10000);
        menu.set_slot(slot);

        let retrieved = menu.get_slot(SaveSlotId::new(0)).unwrap();
        assert_eq!(retrieved.player_name, Some("Updated".to_string()));
        assert_eq!(retrieved.player_level, Some(50));
    }

    #[test]
    fn test_save_menu_most_recent() {
        let mut menu = SaveMenu::with_defaults();

        assert!(menu.most_recent_slot().is_none());

        let slot =
            SaveSlotInfo::occupied(SaveSlotId::new(1), "Recent", 10, 500).with_most_recent(true);
        menu.set_slot(slot);

        let recent = menu.most_recent_slot().unwrap();
        assert_eq!(recent.id, SaveSlotId::new(1));
    }

    #[test]
    fn test_save_menu_actions() {
        let mut menu = SaveMenu::with_defaults();

        menu.new_game();
        menu.continue_game();

        let actions = menu.drain_actions();
        assert!(actions.contains(&SaveMenuAction::NewGame));
        assert!(actions.contains(&SaveMenuAction::ContinueGame));
    }

    #[test]
    fn test_save_menu_confirm_action_save() {
        let mut menu = SaveMenu::with_defaults();
        menu.open_save();
        menu.select_slot(SaveSlotId::new(0));
        menu.drain_actions(); // Clear setup actions

        menu.confirm_action();

        let actions = menu.drain_actions();
        assert!(actions.contains(&SaveMenuAction::SaveToSlot(SaveSlotId::new(0))));
    }

    #[test]
    fn test_save_menu_confirm_action_load() {
        let mut menu = SaveMenu::with_defaults();

        // Set slot as occupied so it can be loaded
        let slot = SaveSlotInfo::occupied(SaveSlotId::new(0), "Test", 1, 100);
        menu.set_slot(slot);

        menu.open_load();
        menu.select_slot(SaveSlotId::new(0));
        menu.drain_actions();

        menu.confirm_action();

        let actions = menu.drain_actions();
        assert!(actions.contains(&SaveMenuAction::LoadFromSlot(SaveSlotId::new(0))));
    }

    #[test]
    fn test_save_menu_set_current_time() {
        let mut menu = SaveMenu::with_defaults();
        menu.set_current_time(1000);
        // Time is used for formatting, stored internally
        assert_eq!(menu.current_time, 1000);
    }

    #[test]
    fn test_save_menu_set_flags() {
        let mut menu = SaveMenu::with_defaults();

        menu.set_can_save(true);
        assert!(menu.can_save_current);

        menu.set_has_recent_save(true);
        assert!(menu.has_recent_save);
    }

    #[test]
    fn test_save_slot_info_serialization() {
        let slot = SaveSlotInfo::occupied(SaveSlotId::new(0), "Hero", 25, 3600)
            .with_world_name("Test")
            .with_last_saved(1000);

        let json = serde_json::to_string(&slot).unwrap();
        let parsed: SaveSlotInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.player_name, slot.player_name);
        assert_eq!(parsed.player_level, slot.player_level);
        assert_eq!(parsed.world_name, slot.world_name);
    }

    #[test]
    fn test_save_menu_config_serialization() {
        let config = SaveMenuConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SaveMenuConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.slot_count, config.slot_count);
        assert_eq!(parsed.grid_columns, config.grid_columns);
    }

    #[test]
    fn test_slot_state_color() {
        // Just verify colors are returned
        assert_ne!(SlotState::Empty.color(), SlotState::Occupied.color());
        assert_ne!(SlotState::Corrupted.color(), SlotState::Saving.color());
    }

    #[test]
    fn test_format_last_saved_edge_cases() {
        let slot = SaveSlotInfo::occupied(SaveSlotId::new(0), "Test", 1, 100).with_last_saved(1000);

        // Current time before saved time
        assert_eq!(slot.format_last_saved(500), "Just now");

        // Exactly 1 minute
        assert_eq!(slot.format_last_saved(1060), "1 minute ago");

        // Exactly 1 hour
        assert_eq!(slot.format_last_saved(4600), "1 hour ago");
    }
}
