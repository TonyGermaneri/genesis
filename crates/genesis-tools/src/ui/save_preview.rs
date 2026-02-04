//! Save Slot Preview UI
//!
//! Detailed preview panel for save slots showing screenshot thumbnail,
//! player information, playtime, and world details.

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};

/// Size of thumbnail images
pub const THUMBNAIL_WIDTH: u32 = 256;
/// Height of thumbnail images in pixels
pub const THUMBNAIL_HEIGHT: u32 = 144;

/// Unique identifier for a save file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SaveId(pub u64);

impl SaveId {
    /// Create a new save ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for SaveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Save-{:08X}", self.0)
    }
}

/// Player character class/type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlayerClass {
    /// Default adventurer class
    #[default]
    Adventurer,
    /// Melee combat focused warrior
    Warrior,
    /// Magic focused mage
    Mage,
    /// Stealth focused rogue
    Rogue,
    /// Ranged combat focused ranger
    Ranger,
    /// Healing focused cleric
    Cleric,
    /// User-defined custom class
    Custom,
}

impl PlayerClass {
    /// Get all player classes
    pub fn all() -> &'static [Self] {
        &[
            Self::Adventurer,
            Self::Warrior,
            Self::Mage,
            Self::Rogue,
            Self::Ranger,
            Self::Cleric,
            Self::Custom,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Adventurer => "Adventurer",
            Self::Warrior => "Warrior",
            Self::Mage => "Mage",
            Self::Rogue => "Rogue",
            Self::Ranger => "Ranger",
            Self::Cleric => "Cleric",
            Self::Custom => "Custom",
        }
    }

    /// Get icon for the class
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Adventurer | Self::Rogue => "🗡",
            Self::Warrior => "⚔",
            Self::Mage => "🔮",
            Self::Ranger => "🏹",
            Self::Cleric => "✝",
            Self::Custom => "⚙",
        }
    }

    /// Get color for the class
    pub fn color(&self) -> Color32 {
        match self {
            Self::Adventurer => Color32::from_rgb(200, 200, 200),
            Self::Warrior => Color32::from_rgb(200, 100, 100),
            Self::Mage => Color32::from_rgb(100, 100, 200),
            Self::Rogue => Color32::from_rgb(150, 100, 150),
            Self::Ranger => Color32::from_rgb(100, 180, 100),
            Self::Cleric => Color32::from_rgb(200, 200, 100),
            Self::Custom => Color32::from_rgb(150, 150, 150),
        }
    }
}

/// Game difficulty setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Difficulty {
    /// Easy difficulty
    Easy,
    /// Normal difficulty (default)
    #[default]
    Normal,
    /// Hard difficulty
    Hard,
    /// Nightmare difficulty
    Nightmare,
    /// User-defined custom difficulty
    Custom,
}

impl Difficulty {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Nightmare => "Nightmare",
            Self::Custom => "Custom",
        }
    }

    /// Get color for difficulty
    pub fn color(&self) -> Color32 {
        match self {
            Self::Easy => Color32::from_rgb(100, 200, 100),
            Self::Normal => Color32::from_rgb(200, 200, 100),
            Self::Hard => Color32::from_rgb(200, 150, 100),
            Self::Nightmare => Color32::from_rgb(200, 100, 100),
            Self::Custom => Color32::from_rgb(150, 100, 200),
        }
    }
}

/// Player statistics snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Current health
    pub health: u32,
    /// Maximum health
    pub max_health: u32,
    /// Current experience
    pub experience: u64,
    /// Experience needed for next level
    pub next_level_exp: u64,
    /// Gold/currency amount
    pub gold: u64,
    /// Number of deaths
    pub deaths: u32,
    /// Number of enemies killed
    pub kills: u32,
    /// Distance traveled (in game units)
    pub distance_traveled: f64,
}

impl PlayerStats {
    /// Create new player stats
    pub fn new(health: u32, max_health: u32) -> Self {
        Self {
            health,
            max_health,
            experience: 0,
            next_level_exp: 100,
            gold: 0,
            deaths: 0,
            kills: 0,
            distance_traveled: 0.0,
        }
    }

    /// Set experience values
    pub fn with_experience(mut self, current: u64, next_level: u64) -> Self {
        self.experience = current;
        self.next_level_exp = next_level;
        self
    }

    /// Set gold amount
    pub fn with_gold(mut self, gold: u64) -> Self {
        self.gold = gold;
        self
    }

    /// Set combat stats
    pub fn with_combat_stats(mut self, deaths: u32, kills: u32) -> Self {
        self.deaths = deaths;
        self.kills = kills;
        self
    }

    /// Get health percentage (0.0-1.0)
    pub fn health_percent(&self) -> f32 {
        if self.max_health == 0 {
            return 0.0;
        }
        (self.health as f32 / self.max_health as f32).clamp(0.0, 1.0)
    }

    /// Get experience percentage to next level (0.0-1.0)
    pub fn exp_percent(&self) -> f32 {
        if self.next_level_exp == 0 {
            return 0.0;
        }
        (self.experience as f32 / self.next_level_exp as f32).clamp(0.0, 1.0)
    }

    /// Format gold for display
    pub fn format_gold(&self) -> String {
        if self.gold >= 1_000_000 {
            format!("{:.1}M", self.gold as f64 / 1_000_000.0)
        } else if self.gold >= 1_000 {
            format!("{:.1}K", self.gold as f64 / 1_000.0)
        } else {
            format!("{}", self.gold)
        }
    }

    /// Get kill/death ratio
    pub fn kd_ratio(&self) -> f32 {
        if self.deaths == 0 {
            return self.kills as f32;
        }
        self.kills as f32 / self.deaths as f32
    }
}

/// World information snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    /// World name
    pub name: String,
    /// World seed for generation
    pub seed: u64,
    /// Difficulty setting
    pub difficulty: Difficulty,
    /// In-game day number
    pub day: u32,
    /// Current in-game time (0-24 hours as float)
    pub time_of_day: f32,
    /// Current weather/biome
    pub current_location: String,
    /// Map completion percentage
    pub map_completion: f32,
    /// Whether hardcore mode is enabled
    pub hardcore: bool,
}

impl WorldInfo {
    /// Create new world info
    pub fn new(name: impl Into<String>, seed: u64) -> Self {
        Self {
            name: name.into(),
            seed,
            difficulty: Difficulty::Normal,
            day: 1,
            time_of_day: 12.0,
            current_location: String::from("Unknown"),
            map_completion: 0.0,
            hardcore: false,
        }
    }

    /// Set difficulty
    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Set current game time
    pub fn with_time(mut self, day: u32, time: f32) -> Self {
        self.day = day;
        self.time_of_day = time.clamp(0.0, 24.0);
        self
    }

    /// Set current location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.current_location = location.into();
        self
    }

    /// Set map completion
    pub fn with_map_completion(mut self, percent: f32) -> Self {
        self.map_completion = percent.clamp(0.0, 100.0);
        self
    }

    /// Enable hardcore mode
    pub fn with_hardcore(mut self, enabled: bool) -> Self {
        self.hardcore = enabled;
        self
    }

    /// Format seed for display
    pub fn format_seed(&self) -> String {
        format!("{:016X}", self.seed)
    }

    /// Format time of day (HH:MM format)
    pub fn format_time(&self) -> String {
        let hours = self.time_of_day.floor() as u32 % 24;
        let minutes = ((self.time_of_day.fract()) * 60.0).floor() as u32;
        format!("{hours:02}:{minutes:02}")
    }

    /// Get time of day description
    pub fn time_description(&self) -> &'static str {
        match self.time_of_day as u32 {
            6..=8 => "Dawn",
            9..=11 => "Morning",
            12..=14 => "Noon",
            15..=17 => "Afternoon",
            18..=20 => "Dusk",
            _ => "Night",
        }
    }
}

impl Default for WorldInfo {
    fn default() -> Self {
        Self::new("New World", 0)
    }
}

/// Thumbnail image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailData {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// RGBA pixel data
    pub pixels: Vec<u8>,
}

impl ThumbnailData {
    /// Create new thumbnail with given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            pixels: vec![0; size],
        }
    }

    /// Create from RGBA pixel data
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Option<Self> {
        let expected = (width * height * 4) as usize;
        if pixels.len() != expected {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels,
        })
    }

    /// Create a solid color placeholder
    pub fn placeholder(width: u32, height: u32, color: [u8; 4]) -> Self {
        let size = (width * height) as usize;
        let mut pixels = Vec::with_capacity(size * 4);
        for _ in 0..size {
            pixels.extend_from_slice(&color);
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Check if thumbnail has valid data
    pub fn is_valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.pixels.len() == (self.width * self.height * 4) as usize
    }

    /// Get pixel at coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some([
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ])
    }
}

impl Default for ThumbnailData {
    fn default() -> Self {
        Self::placeholder(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT, [40, 40, 50, 255])
    }
}

/// Complete save preview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePreviewData {
    /// Save identifier
    pub id: SaveId,
    /// Player character name
    pub player_name: String,
    /// Player level
    pub player_level: u32,
    /// Player class
    pub player_class: PlayerClass,
    /// Player statistics
    pub stats: PlayerStats,
    /// World information
    pub world: WorldInfo,
    /// Total playtime in seconds
    pub playtime_seconds: u64,
    /// Last saved timestamp (Unix epoch)
    pub last_saved: u64,
    /// Save file version
    pub save_version: u32,
    /// Optional thumbnail
    pub thumbnail: Option<ThumbnailData>,
    /// Optional notes from player
    pub notes: Option<String>,
}

impl SavePreviewData {
    /// Create new save preview
    pub fn new(id: SaveId, player_name: impl Into<String>, player_level: u32) -> Self {
        Self {
            id,
            player_name: player_name.into(),
            player_level,
            player_class: PlayerClass::Adventurer,
            stats: PlayerStats::default(),
            world: WorldInfo::default(),
            playtime_seconds: 0,
            last_saved: 0,
            save_version: 1,
            thumbnail: None,
            notes: None,
        }
    }

    /// Set player class
    pub fn with_class(mut self, class: PlayerClass) -> Self {
        self.player_class = class;
        self
    }

    /// Set player stats
    pub fn with_stats(mut self, stats: PlayerStats) -> Self {
        self.stats = stats;
        self
    }

    /// Set world info
    pub fn with_world(mut self, world: WorldInfo) -> Self {
        self.world = world;
        self
    }

    /// Set playtime
    pub fn with_playtime(mut self, seconds: u64) -> Self {
        self.playtime_seconds = seconds;
        self
    }

    /// Set last saved timestamp
    pub fn with_last_saved(mut self, timestamp: u64) -> Self {
        self.last_saved = timestamp;
        self
    }

    /// Set thumbnail
    pub fn with_thumbnail(mut self, thumb: ThumbnailData) -> Self {
        self.thumbnail = Some(thumb);
        self
    }

    /// Set notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Format playtime as human-readable string
    pub fn format_playtime(&self) -> String {
        let hours = self.playtime_seconds / 3600;
        let minutes = (self.playtime_seconds % 3600) / 60;
        let seconds = self.playtime_seconds % 60;

        if hours > 0 {
            format!("{hours}h {minutes}m {seconds}s")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        }
    }

    /// Format playtime short (HH:MM:SS)
    pub fn format_playtime_short(&self) -> String {
        let hours = self.playtime_seconds / 3600;
        let minutes = (self.playtime_seconds % 3600) / 60;
        let seconds = self.playtime_seconds % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }

    /// Format last saved as relative time
    pub fn format_last_saved(&self, current_time: u64) -> String {
        if self.last_saved == 0 {
            return String::from("Never");
        }
        if current_time < self.last_saved {
            return String::from("Just now");
        }

        let diff = current_time - self.last_saved;
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
        } else if diff < 2_592_000 {
            let days = diff / 86400;
            if days == 1 {
                String::from("1 day ago")
            } else {
                format!("{days} days ago")
            }
        } else {
            let months = diff / 2_592_000;
            if months == 1 {
                String::from("1 month ago")
            } else {
                format!("{months} months ago")
            }
        }
    }
}

/// Configuration for save preview panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePreviewConfig {
    /// Size of thumbnail display
    pub thumbnail_size: [f32; 2],
    /// Whether to show player stats
    pub show_stats: bool,
    /// Whether to show world info
    pub show_world_info: bool,
    /// Whether to show the seed
    pub show_seed: bool,
    /// Whether to show notes
    pub show_notes: bool,
    /// Background color
    pub background_color: [u8; 4],
    /// Border color
    pub border_color: [u8; 4],
}

impl Default for SavePreviewConfig {
    fn default() -> Self {
        Self {
            thumbnail_size: [256.0, 144.0],
            show_stats: true,
            show_world_info: true,
            show_seed: true,
            show_notes: true,
            background_color: [30, 30, 40, 240],
            border_color: [60, 60, 80, 255],
        }
    }
}

/// Actions generated by the save preview panel
#[derive(Debug, Clone, PartialEq)]
pub enum SavePreviewAction {
    /// Request to load this save
    Load(SaveId),
    /// Request to delete this save
    Delete(SaveId),
    /// Request to copy this save
    Copy(SaveId),
    /// Request to export this save
    Export(SaveId),
    /// Close the preview
    Close,
}

/// Save preview panel widget
#[derive(Debug)]
pub struct SavePreviewPanel {
    /// Whether panel is visible
    visible: bool,
    /// Current preview data
    preview: Option<SavePreviewData>,
    /// Panel configuration
    config: SavePreviewConfig,
    /// Pending actions
    actions: Vec<SavePreviewAction>,
    /// Current time for relative timestamps
    current_time: u64,
}

impl SavePreviewPanel {
    /// Create new preview panel
    pub fn new(config: SavePreviewConfig) -> Self {
        Self {
            visible: false,
            preview: None,
            config,
            actions: Vec::new(),
            current_time: 0,
        }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(SavePreviewConfig::default())
    }

    /// Check if panel is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the panel
    pub fn show_panel(&mut self) {
        self.visible = true;
    }

    /// Hide the panel
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Set preview data
    pub fn set_preview(&mut self, data: SavePreviewData) {
        self.preview = Some(data);
        self.visible = true;
    }

    /// Clear preview
    pub fn clear_preview(&mut self) {
        self.preview = None;
    }

    /// Get current preview data
    pub fn preview(&self) -> Option<&SavePreviewData> {
        self.preview.as_ref()
    }

    /// Set current time for timestamps
    pub fn set_current_time(&mut self, time: u64) {
        self.current_time = time;
    }

    /// Get configuration
    pub fn config(&self) -> &SavePreviewConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: SavePreviewConfig) {
        self.config = config;
    }

    /// Drain pending actions
    pub fn drain_actions(&mut self) -> Vec<SavePreviewAction> {
        std::mem::take(&mut self.actions)
    }

    /// Render the preview panel
    pub fn render(&mut self, ui: &mut Ui) {
        if !self.visible {
            return;
        }

        let Some(preview) = self.preview.clone() else {
            return;
        };

        let bg = Color32::from_rgba_unmultiplied(
            self.config.background_color[0],
            self.config.background_color[1],
            self.config.background_color[2],
            self.config.background_color[3],
        );
        let border = Color32::from_rgba_unmultiplied(
            self.config.border_color[0],
            self.config.border_color[1],
            self.config.border_color[2],
            self.config.border_color[3],
        );

        egui::Frame::none()
            .fill(bg)
            .stroke(egui::Stroke::new(1.0, border))
            .inner_margin(16.0)
            .rounding(8.0)
            .show(ui, |ui| {
                self.render_content(ui, &preview);
            });
    }

    fn render_content(&mut self, ui: &mut Ui, preview: &SavePreviewData) {
        // Header with close button
        ui.horizontal(|ui| {
            ui.heading(&preview.player_name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    self.visible = false;
                    self.actions.push(SavePreviewAction::Close);
                }
            });
        });

        ui.separator();

        // Thumbnail placeholder
        ui.horizontal(|ui| {
            // Thumbnail area
            let thumb_size =
                egui::vec2(self.config.thumbnail_size[0], self.config.thumbnail_size[1]);
            let (rect, _) = ui.allocate_exact_size(thumb_size, egui::Sense::hover());

            if preview.thumbnail.is_some() {
                // Would render actual thumbnail here
                ui.painter().rect_filled(rect, 4.0, Color32::from_gray(50));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📷",
                    egui::FontId::proportional(32.0),
                    Color32::from_gray(120),
                );
            } else {
                ui.painter().rect_filled(rect, 4.0, Color32::from_gray(30));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No Preview",
                    egui::FontId::proportional(14.0),
                    Color32::from_gray(80),
                );
            }

            ui.add_space(16.0);

            // Info column
            ui.vertical(|ui| {
                // Level and class
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Level {}", preview.player_level))
                            .strong()
                            .size(18.0),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            preview.player_class.icon(),
                            preview.player_class.display_name()
                        ))
                        .color(preview.player_class.color()),
                    );
                });

                ui.add_space(4.0);

                // Playtime
                ui.label(format!("⏱ Playtime: {}", preview.format_playtime()));

                // Last saved
                ui.label(format!(
                    "📅 Last saved: {}",
                    preview.format_last_saved(self.current_time)
                ));

                // World info
                if self.config.show_world_info {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("🌍 {}", preview.world.name)).strong());
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Day {} - {}",
                            preview.world.day,
                            preview.world.format_time()
                        ));
                        ui.label(
                            egui::RichText::new(preview.world.difficulty.display_name())
                                .color(preview.world.difficulty.color()),
                        );
                        if preview.world.hardcore {
                            ui.label(egui::RichText::new("☠ Hardcore").color(Color32::RED));
                        }
                    });
                    ui.label(format!("📍 {}", preview.world.current_location));
                }
            });
        });

        // Stats section
        if self.config.show_stats {
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label(egui::RichText::new("Statistics").strong());
            egui::Grid::new("stats_grid")
                .num_columns(4)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(format!(
                        "❤ HP: {}/{}",
                        preview.stats.health, preview.stats.max_health
                    ));
                    ui.label(format!("💰 Gold: {}", preview.stats.format_gold()));
                    ui.label(format!("⚔ Kills: {}", preview.stats.kills));
                    ui.label(format!("💀 Deaths: {}", preview.stats.deaths));
                    ui.end_row();

                    ui.label(format!("📊 K/D: {:.2}", preview.stats.kd_ratio()));
                    ui.label(format!("🗺 Map: {:.1}%", preview.world.map_completion));
                    ui.end_row();
                });
        }

        // Seed (if enabled)
        if self.config.show_seed {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Seed:")
                        .small()
                        .color(Color32::from_gray(120)),
                );
                ui.label(
                    egui::RichText::new(preview.world.format_seed())
                        .small()
                        .monospace()
                        .color(Color32::from_gray(160)),
                );
            });
        }

        // Notes (if present and enabled)
        if self.config.show_notes {
            if let Some(notes) = &preview.notes {
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    egui::RichText::new("Notes:")
                        .small()
                        .color(Color32::from_gray(120)),
                );
                ui.label(egui::RichText::new(notes).small().italics());
            }
        }

        // Action buttons
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("▶ Load").clicked() {
                self.actions.push(SavePreviewAction::Load(preview.id));
            }
            if ui.button("📋 Copy").clicked() {
                self.actions.push(SavePreviewAction::Copy(preview.id));
            }
            if ui.button("📤 Export").clicked() {
                self.actions.push(SavePreviewAction::Export(preview.id));
            }
            ui.separator();
            if ui
                .button(egui::RichText::new("🗑 Delete").color(Color32::from_rgb(200, 100, 100)))
                .clicked()
            {
                self.actions.push(SavePreviewAction::Delete(preview.id));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_id() {
        let id = SaveId::new(12345);
        assert_eq!(id.0, 12345);
        assert!(format!("{id}").contains("Save-"));
    }

    #[test]
    fn test_player_class_all() {
        let classes = PlayerClass::all();
        assert_eq!(classes.len(), 7);
        assert!(classes.contains(&PlayerClass::Adventurer));
        assert!(classes.contains(&PlayerClass::Warrior));
    }

    #[test]
    fn test_player_class_display() {
        assert_eq!(PlayerClass::Warrior.display_name(), "Warrior");
        assert_eq!(PlayerClass::Mage.icon(), "🔮");
        assert_ne!(PlayerClass::Warrior.color(), PlayerClass::Mage.color());
    }

    #[test]
    fn test_difficulty_display() {
        assert_eq!(Difficulty::Normal.display_name(), "Normal");
        assert_eq!(Difficulty::Hard.display_name(), "Hard");
        assert_ne!(Difficulty::Easy.color(), Difficulty::Nightmare.color());
    }

    #[test]
    fn test_player_stats_new() {
        let stats = PlayerStats::new(100, 100);
        assert_eq!(stats.health, 100);
        assert_eq!(stats.max_health, 100);
        assert_eq!(stats.health_percent(), 1.0);
    }

    #[test]
    fn test_player_stats_builders() {
        let stats = PlayerStats::new(50, 100)
            .with_experience(500, 1000)
            .with_gold(5000)
            .with_combat_stats(5, 100);

        assert_eq!(stats.experience, 500);
        assert_eq!(stats.gold, 5000);
        assert_eq!(stats.deaths, 5);
        assert_eq!(stats.kills, 100);
    }

    #[test]
    fn test_player_stats_health_percent() {
        let stats = PlayerStats::new(50, 100);
        assert_eq!(stats.health_percent(), 0.5);

        let zero = PlayerStats::new(0, 0);
        assert_eq!(zero.health_percent(), 0.0);
    }

    #[test]
    fn test_player_stats_exp_percent() {
        let stats = PlayerStats::new(100, 100).with_experience(250, 500);
        assert_eq!(stats.exp_percent(), 0.5);
    }

    #[test]
    fn test_player_stats_format_gold() {
        let small = PlayerStats::new(100, 100).with_gold(500);
        assert_eq!(small.format_gold(), "500");

        let medium = PlayerStats::new(100, 100).with_gold(5000);
        assert_eq!(medium.format_gold(), "5.0K");

        let large = PlayerStats::new(100, 100).with_gold(5000000);
        assert_eq!(large.format_gold(), "5.0M");
    }

    #[test]
    fn test_player_stats_kd_ratio() {
        let stats = PlayerStats::new(100, 100).with_combat_stats(10, 50);
        assert_eq!(stats.kd_ratio(), 5.0);

        let no_deaths = PlayerStats::new(100, 100).with_combat_stats(0, 50);
        assert_eq!(no_deaths.kd_ratio(), 50.0);
    }

    #[test]
    fn test_world_info_new() {
        let world = WorldInfo::new("Test World", 12345);
        assert_eq!(world.name, "Test World");
        assert_eq!(world.seed, 12345);
        assert_eq!(world.day, 1);
    }

    #[test]
    fn test_world_info_builders() {
        let world = WorldInfo::new("Test", 0)
            .with_difficulty(Difficulty::Hard)
            .with_time(5, 14.5)
            .with_location("Forest")
            .with_map_completion(50.0)
            .with_hardcore(true);

        assert_eq!(world.difficulty, Difficulty::Hard);
        assert_eq!(world.day, 5);
        assert_eq!(world.time_of_day, 14.5);
        assert_eq!(world.current_location, "Forest");
        assert_eq!(world.map_completion, 50.0);
        assert!(world.hardcore);
    }

    #[test]
    fn test_world_info_format_seed() {
        let world = WorldInfo::new("Test", 0xDEADBEEF);
        assert!(world.format_seed().contains("DEADBEEF"));
    }

    #[test]
    fn test_world_info_format_time() {
        let world = WorldInfo::new("Test", 0).with_time(1, 14.5);
        assert_eq!(world.format_time(), "14:30");
    }

    #[test]
    fn test_world_info_time_description() {
        let morning = WorldInfo::new("Test", 0).with_time(1, 10.0);
        assert_eq!(morning.time_description(), "Morning");

        let night = WorldInfo::new("Test", 0).with_time(1, 2.0);
        assert_eq!(night.time_description(), "Night");

        let noon = WorldInfo::new("Test", 0).with_time(1, 12.0);
        assert_eq!(noon.time_description(), "Noon");
    }

    #[test]
    fn test_thumbnail_data_new() {
        let thumb = ThumbnailData::new(64, 64);
        assert_eq!(thumb.width, 64);
        assert_eq!(thumb.height, 64);
        assert_eq!(thumb.pixels.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_thumbnail_data_from_rgba() {
        let pixels = vec![0u8; 16 * 16 * 4];
        let thumb = ThumbnailData::from_rgba(16, 16, pixels);
        assert!(thumb.is_some());

        let bad = ThumbnailData::from_rgba(16, 16, vec![0u8; 10]);
        assert!(bad.is_none());
    }

    #[test]
    fn test_thumbnail_data_placeholder() {
        let thumb = ThumbnailData::placeholder(4, 4, [255, 0, 0, 255]);
        assert_eq!(thumb.width, 4);
        assert_eq!(thumb.get_pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_thumbnail_data_is_valid() {
        let valid = ThumbnailData::new(64, 64);
        assert!(valid.is_valid());

        let invalid = ThumbnailData {
            width: 64,
            height: 64,
            pixels: vec![0; 100], // Wrong size
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_thumbnail_get_pixel() {
        let thumb = ThumbnailData::placeholder(4, 4, [100, 150, 200, 255]);
        assert_eq!(thumb.get_pixel(0, 0), Some([100, 150, 200, 255]));
        assert_eq!(thumb.get_pixel(3, 3), Some([100, 150, 200, 255]));
        assert_eq!(thumb.get_pixel(4, 4), None); // Out of bounds
    }

    #[test]
    fn test_save_preview_data_new() {
        let preview = SavePreviewData::new(SaveId::new(1), "Hero", 10);
        assert_eq!(preview.player_name, "Hero");
        assert_eq!(preview.player_level, 10);
        assert_eq!(preview.player_class, PlayerClass::Adventurer);
    }

    #[test]
    fn test_save_preview_data_builders() {
        let preview = SavePreviewData::new(SaveId::new(1), "Hero", 25)
            .with_class(PlayerClass::Warrior)
            .with_playtime(7200)
            .with_last_saved(1000)
            .with_notes("Test notes");

        assert_eq!(preview.player_class, PlayerClass::Warrior);
        assert_eq!(preview.playtime_seconds, 7200);
        assert_eq!(preview.last_saved, 1000);
        assert_eq!(preview.notes, Some("Test notes".to_string()));
    }

    #[test]
    fn test_format_playtime() {
        let preview = SavePreviewData::new(SaveId::new(1), "Test", 1).with_playtime(3661);
        assert_eq!(preview.format_playtime(), "1h 1m 1s");
        assert_eq!(preview.format_playtime_short(), "01:01:01");

        let short = SavePreviewData::new(SaveId::new(1), "Test", 1).with_playtime(65);
        assert_eq!(short.format_playtime(), "1m 5s");

        let very_short = SavePreviewData::new(SaveId::new(1), "Test", 1).with_playtime(30);
        assert_eq!(very_short.format_playtime(), "30s");
    }

    #[test]
    fn test_format_last_saved() {
        let preview = SavePreviewData::new(SaveId::new(1), "Test", 1).with_last_saved(1000);

        assert_eq!(preview.format_last_saved(1030), "Just now");
        assert_eq!(preview.format_last_saved(1120), "2 minutes ago");
        assert_eq!(preview.format_last_saved(4600), "1 hour ago");
        assert_eq!(preview.format_last_saved(90400), "1 day ago");
        assert_eq!(preview.format_last_saved(2600000), "1 month ago");
    }

    #[test]
    fn test_save_preview_config_defaults() {
        let config = SavePreviewConfig::default();
        assert!(config.show_stats);
        assert!(config.show_world_info);
        assert!(config.show_seed);
    }

    #[test]
    fn test_save_preview_panel_new() {
        let panel = SavePreviewPanel::with_defaults();
        assert!(!panel.is_visible());
        assert!(panel.preview().is_none());
    }

    #[test]
    fn test_save_preview_panel_set_preview() {
        let mut panel = SavePreviewPanel::with_defaults();
        let preview = SavePreviewData::new(SaveId::new(1), "Hero", 10);

        panel.set_preview(preview);
        assert!(panel.is_visible());
        assert!(panel.preview().is_some());
    }

    #[test]
    fn test_save_preview_panel_visibility() {
        let mut panel = SavePreviewPanel::with_defaults();

        panel.show_panel();
        assert!(panel.is_visible());

        panel.hide();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_save_preview_panel_clear() {
        let mut panel = SavePreviewPanel::with_defaults();
        panel.set_preview(SavePreviewData::new(SaveId::new(1), "Hero", 10));

        panel.clear_preview();
        assert!(panel.preview().is_none());
    }

    #[test]
    fn test_save_preview_panel_current_time() {
        let mut panel = SavePreviewPanel::with_defaults();
        panel.set_current_time(1000);
        assert_eq!(panel.current_time, 1000);
    }

    #[test]
    fn test_player_stats_serialization() {
        let stats = PlayerStats::new(100, 100)
            .with_gold(5000)
            .with_combat_stats(5, 100);

        let json = serde_json::to_string(&stats).unwrap();
        let parsed: PlayerStats = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.health, stats.health);
        assert_eq!(parsed.gold, stats.gold);
    }

    #[test]
    fn test_world_info_serialization() {
        let world = WorldInfo::new("Test World", 12345).with_difficulty(Difficulty::Hard);

        let json = serde_json::to_string(&world).unwrap();
        let parsed: WorldInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, world.name);
        assert_eq!(parsed.seed, world.seed);
        assert_eq!(parsed.difficulty, Difficulty::Hard);
    }

    #[test]
    fn test_save_preview_data_serialization() {
        let preview = SavePreviewData::new(SaveId::new(1), "Hero", 25)
            .with_class(PlayerClass::Mage)
            .with_playtime(3600);

        let json = serde_json::to_string(&preview).unwrap();
        let parsed: SavePreviewData = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.player_name, "Hero");
        assert_eq!(parsed.player_level, 25);
        assert_eq!(parsed.player_class, PlayerClass::Mage);
    }

    #[test]
    fn test_save_preview_config_serialization() {
        let config = SavePreviewConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SavePreviewConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.show_stats, config.show_stats);
        assert_eq!(parsed.thumbnail_size, config.thumbnail_size);
    }
}
