//! Sound test panel for previewing and testing audio.
//!
//! Provides a testing interface including:
//! - Sound browser by category
//! - Play/stop controls
//! - Position controls for spatial testing
//! - Volume/pan preview
//! - Loop toggle

use egui::{Color32, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Unique identifier for a sound asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SoundAssetId(pub String);

impl SoundAssetId {
    /// Create a new sound asset ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for SoundAssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Category for organizing sounds in the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoundBrowserCategory {
    /// All sounds.
    All,
    /// Background music.
    Music,
    /// Sound effects.
    Sfx,
    /// Ambient/environmental.
    Ambient,
    /// UI sounds.
    Ui,
    /// Voice/dialogue.
    Voice,
    /// Favorites.
    Favorites,
}

impl SoundBrowserCategory {
    /// Get all categories.
    pub fn all() -> &'static [SoundBrowserCategory] {
        &[
            SoundBrowserCategory::All,
            SoundBrowserCategory::Music,
            SoundBrowserCategory::Sfx,
            SoundBrowserCategory::Ambient,
            SoundBrowserCategory::Ui,
            SoundBrowserCategory::Voice,
            SoundBrowserCategory::Favorites,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            SoundBrowserCategory::All => "All",
            SoundBrowserCategory::Music => "Music",
            SoundBrowserCategory::Sfx => "SFX",
            SoundBrowserCategory::Ambient => "Ambient",
            SoundBrowserCategory::Ui => "UI",
            SoundBrowserCategory::Voice => "Voice",
            SoundBrowserCategory::Favorites => "★ Favorites",
        }
    }

    /// Get icon.
    pub fn icon(&self) -> &'static str {
        match self {
            SoundBrowserCategory::All => "📁",
            SoundBrowserCategory::Music => "🎵",
            SoundBrowserCategory::Sfx => "💥",
            SoundBrowserCategory::Ambient => "🌿",
            SoundBrowserCategory::Ui => "🖱",
            SoundBrowserCategory::Voice => "🗣",
            SoundBrowserCategory::Favorites => "⭐",
        }
    }
}

/// A sound entry in the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEntry {
    /// Unique asset ID.
    pub id: SoundAssetId,
    /// Display name.
    pub name: String,
    /// Category.
    pub category: SoundBrowserCategory,
    /// Duration in seconds.
    pub duration: f32,
    /// File path or resource path.
    pub path: String,
    /// Whether this is a favorite.
    pub favorite: bool,
    /// Tags for searching.
    pub tags: Vec<String>,
}

impl SoundEntry {
    /// Create a new sound entry.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: SoundBrowserCategory,
    ) -> Self {
        let id_string = id.into();
        Self {
            id: SoundAssetId::new(&id_string),
            name: name.into(),
            category,
            duration: 0.0,
            path: id_string,
            favorite: false,
            tags: Vec::new(),
        }
    }

    /// Set duration.
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Set path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set favorite status.
    pub fn with_favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }

    /// Get formatted duration (MM:SS).
    pub fn formatted_duration(&self) -> String {
        let mins = (self.duration / 60.0).floor() as u32;
        let secs = (self.duration % 60.0).floor() as u32;
        format!("{mins}:{secs:02}")
    }

    /// Check if entry matches search query.
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.path.to_lowercase().contains(&query_lower)
            || self
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
    }

    /// Check if entry matches category filter.
    pub fn matches_category(&self, filter: SoundBrowserCategory) -> bool {
        match filter {
            SoundBrowserCategory::All => true,
            SoundBrowserCategory::Favorites => self.favorite,
            _ => self.category == filter,
        }
    }
}

/// Settings for sound preview/playback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundPreviewSettings {
    /// Volume (0.0 - 1.0).
    pub volume: f32,
    /// Pan (-1.0 left to 1.0 right).
    pub pan: f32,
    /// Pitch multiplier (0.5 - 2.0).
    pub pitch: f32,
    /// Whether to loop.
    pub looping: bool,
    /// Spatial position for 3D testing (None for 2D).
    pub spatial_position: Option<(f32, f32, f32)>,
}

impl Default for SoundPreviewSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
            looping: false,
            spatial_position: None,
        }
    }
}

impl SoundPreviewSettings {
    /// Reset to defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Check if using spatial positioning.
    pub fn is_spatial(&self) -> bool {
        self.spatial_position.is_some()
    }

    /// Enable spatial mode at given position.
    pub fn enable_spatial(&mut self, x: f32, y: f32, z: f32) {
        self.spatial_position = Some((x, y, z));
    }

    /// Disable spatial mode.
    pub fn disable_spatial(&mut self) {
        self.spatial_position = None;
    }
}

/// Actions returned by the sound test panel.
#[derive(Debug, Clone, PartialEq)]
pub enum SoundTestAction {
    /// Play a sound with current settings.
    Play(SoundAssetId, SoundPreviewSettings),
    /// Stop a currently playing sound.
    Stop(SoundAssetId),
    /// Stop all sounds.
    StopAll,
    /// Toggle favorite status for a sound.
    ToggleFavorite(SoundAssetId),
    /// Update preview settings for a playing sound.
    UpdateSettings(SoundAssetId, SoundPreviewSettings),
}

/// Configuration for the sound test panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundTestPanelConfig {
    /// Panel title.
    pub title: String,
    /// Show category tabs.
    pub show_categories: bool,
    /// Show search box.
    pub show_search: bool,
    /// Show preview controls.
    pub show_preview_controls: bool,
    /// Show spatial position controls.
    pub show_spatial_controls: bool,
    /// Maximum sounds to display.
    pub max_sounds_displayed: usize,
    /// Sound list height in pixels.
    pub list_height: f32,
}

impl Default for SoundTestPanelConfig {
    fn default() -> Self {
        Self {
            title: "Sound Test".to_string(),
            show_categories: true,
            show_search: true,
            show_preview_controls: true,
            show_spatial_controls: true,
            max_sounds_displayed: 50,
            list_height: 200.0,
        }
    }
}

/// State of a currently playing preview sound.
#[derive(Debug, Clone)]
pub struct PlayingPreview {
    /// Sound ID.
    pub sound_id: SoundAssetId,
    /// Current position in seconds.
    pub position: f32,
    /// Total duration in seconds.
    pub duration: f32,
    /// Whether paused.
    pub paused: bool,
}

impl PlayingPreview {
    /// Get playback progress (0.0 - 1.0).
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.position / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Sound test panel widget.
#[derive(Debug)]
pub struct SoundTestPanel {
    /// Available sounds.
    sounds: Vec<SoundEntry>,
    /// Configuration.
    pub config: SoundTestPanelConfig,
    /// Whether the panel is open.
    pub open: bool,
    /// Current category filter.
    pub category_filter: SoundBrowserCategory,
    /// Current search query.
    pub search_query: String,
    /// Currently selected sound.
    pub selected_sound: Option<SoundAssetId>,
    /// Preview settings.
    pub preview_settings: SoundPreviewSettings,
    /// Currently playing preview.
    pub playing_preview: Option<PlayingPreview>,
    /// Pending actions.
    pending_actions: Vec<SoundTestAction>,
}

impl Default for SoundTestPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundTestPanel {
    /// Create a new sound test panel.
    pub fn new() -> Self {
        Self {
            sounds: Vec::new(),
            config: SoundTestPanelConfig::default(),
            open: false,
            category_filter: SoundBrowserCategory::All,
            search_query: String::new(),
            selected_sound: None,
            preview_settings: SoundPreviewSettings::default(),
            playing_preview: None,
            pending_actions: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: SoundTestPanelConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set available sounds.
    pub fn set_sounds(&mut self, sounds: Vec<SoundEntry>) {
        self.sounds = sounds;
    }

    /// Add a sound to the list.
    pub fn add_sound(&mut self, sound: SoundEntry) {
        self.sounds.push(sound);
    }

    /// Clear all sounds.
    pub fn clear_sounds(&mut self) {
        self.sounds.clear();
    }

    /// Get a sound by ID.
    pub fn get_sound(&self, id: &SoundAssetId) -> Option<&SoundEntry> {
        self.sounds.iter().find(|s| &s.id == id)
    }

    /// Get a mutable sound by ID.
    pub fn get_sound_mut(&mut self, id: &SoundAssetId) -> Option<&mut SoundEntry> {
        self.sounds.iter_mut().find(|s| &s.id == id)
    }

    /// Toggle favorite status for a sound.
    pub fn toggle_favorite(&mut self, id: &SoundAssetId) {
        if let Some(sound) = self.get_sound_mut(id) {
            sound.favorite = !sound.favorite;
        }
    }

    /// Get filtered sounds based on current category and search.
    pub fn filtered_sounds(&self) -> Vec<&SoundEntry> {
        self.sounds
            .iter()
            .filter(|s| s.matches_category(self.category_filter))
            .filter(|s| self.search_query.is_empty() || s.matches_search(&self.search_query))
            .take(self.config.max_sounds_displayed)
            .collect()
    }

    /// Update playing preview state.
    pub fn update_playing_preview(&mut self, preview: Option<PlayingPreview>) {
        self.playing_preview = preview;
    }

    /// Open the panel.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the panel.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggle panel visibility.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Render the panel and return any actions.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<SoundTestAction> {
        self.pending_actions.clear();

        if !self.open {
            return Vec::new();
        }

        ui.vertical(|ui| {
            // Title
            ui.horizontal(|ui| {
                ui.heading(&self.config.title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        self.open = false;
                    }
                    if ui.small_button("⏹ Stop All").clicked() {
                        self.pending_actions.push(SoundTestAction::StopAll);
                        self.playing_preview = None;
                    }
                });
            });

            ui.separator();

            // Search
            if self.config.show_search {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.search_query);
                    if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                        self.search_query.clear();
                    }
                });
            }

            // Category tabs
            if self.config.show_categories {
                self.show_category_tabs(ui);
            }

            ui.separator();

            // Sound list
            self.show_sound_list(ui);

            ui.separator();

            // Preview controls
            if self.config.show_preview_controls {
                self.show_preview_controls(ui);
            }

            // Spatial controls
            if self.config.show_spatial_controls {
                self.show_spatial_controls(ui);
            }
        });

        std::mem::take(&mut self.pending_actions)
    }

    /// Show category filter tabs.
    fn show_category_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for category in SoundBrowserCategory::all() {
                let selected = self.category_filter == *category;
                let text = format!("{} {}", category.icon(), category.display_name());

                if ui.selectable_label(selected, text).clicked() {
                    self.category_filter = *category;
                }
            }
        });
    }

    /// Show the sound list.
    fn show_sound_list(&mut self, ui: &mut Ui) {
        // Collect all data we need before entering the UI closure
        let sound_data: Vec<_> = self
            .sounds
            .iter()
            .filter(|s| s.matches_category(self.category_filter))
            .filter(|s| self.search_query.is_empty() || s.matches_search(&self.search_query))
            .take(self.config.max_sounds_displayed)
            .map(|s| {
                (
                    s.id.clone(),
                    s.name.clone(),
                    s.formatted_duration(),
                    s.favorite,
                )
            })
            .collect();

        let count = sound_data.len();
        let total = self.sounds.len();
        let is_empty = sound_data.is_empty();
        let list_height = self.config.list_height;

        ui.label(format!("Sounds: {count} / {total}"));

        // Collect state we need for determining selection/playing status
        let selected_id = self.selected_sound.clone();
        let playing_id = self.playing_preview.as_ref().map(|p| p.sound_id.clone());
        let playing_progress = self.playing_preview.as_ref().map(PlayingPreview::progress);

        egui::ScrollArea::vertical()
            .max_height(list_height)
            .show(ui, |ui| {
                for (id, name, duration, favorite) in &sound_data {
                    let is_selected = selected_id.as_ref() == Some(id);
                    let is_playing = playing_id.as_ref() == Some(id);

                    ui.horizontal(|ui| {
                        // Play/stop button
                        let play_btn = if is_playing { "⏹" } else { "▶" };
                        if ui.small_button(play_btn).clicked() {
                            if is_playing {
                                self.pending_actions.push(SoundTestAction::Stop(id.clone()));
                                self.playing_preview = None;
                            } else {
                                self.pending_actions.push(SoundTestAction::Play(
                                    id.clone(),
                                    self.preview_settings.clone(),
                                ));
                            }
                        }

                        // Favorite button
                        let fav_btn = if *favorite { "★" } else { "☆" };
                        let fav_color = if *favorite {
                            Color32::GOLD
                        } else {
                            Color32::GRAY
                        };
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new(fav_btn).color(fav_color))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            self.pending_actions
                                .push(SoundTestAction::ToggleFavorite(id.clone()));
                            // Toggle locally too
                            if let Some(sound) = self.sounds.iter_mut().find(|s| &s.id == id) {
                                sound.favorite = !sound.favorite;
                            }
                        }

                        // Sound name (selectable)
                        let text = if is_playing {
                            egui::RichText::new(name).color(Color32::from_rgb(100, 200, 100))
                        } else {
                            egui::RichText::new(name)
                        };

                        if ui.selectable_label(is_selected, text).clicked() {
                            self.selected_sound = Some(id.clone());
                        }

                        // Duration
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.weak(duration);
                        });
                    });

                    // Show progress bar if playing
                    if is_playing {
                        if let Some(progress) = playing_progress {
                            let bar_height = 4.0;
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), bar_height),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    rect.min,
                                    Vec2::new(rect.width() * progress, bar_height),
                                ),
                                2.0,
                                Color32::from_rgb(100, 180, 100),
                            );
                        }
                    }
                }

                if is_empty {
                    ui.weak("No sounds match filter");
                }
            });
    }

    /// Show preview controls.
    fn show_preview_controls(&mut self, ui: &mut Ui) {
        ui.label("Preview Settings");

        ui.horizontal(|ui| {
            ui.label("Volume:");
            let mut vol_percent = self.preview_settings.volume * 100.0;
            if ui
                .add(
                    egui::Slider::new(&mut vol_percent, 0.0..=100.0)
                        .suffix("%")
                        .show_value(true),
                )
                .changed()
            {
                self.preview_settings.volume = vol_percent / 100.0;
                self.emit_settings_update();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Pan:");
            if ui
                .add(egui::Slider::new(&mut self.preview_settings.pan, -1.0..=1.0).show_value(true))
                .changed()
            {
                self.emit_settings_update();
            }
            if ui.small_button("C").on_hover_text("Center").clicked() {
                self.preview_settings.pan = 0.0;
                self.emit_settings_update();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Pitch:");
            if ui
                .add(
                    egui::Slider::new(&mut self.preview_settings.pitch, 0.5..=2.0).show_value(true),
                )
                .changed()
            {
                self.emit_settings_update();
            }
            if ui.small_button("1x").on_hover_text("Reset pitch").clicked() {
                self.preview_settings.pitch = 1.0;
                self.emit_settings_update();
            }
        });

        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.preview_settings.looping, "Loop")
                .changed()
            {
                self.emit_settings_update();
            }

            if ui.button("Reset All").clicked() {
                self.preview_settings.reset();
                self.emit_settings_update();
            }
        });
    }

    /// Show spatial position controls.
    fn show_spatial_controls(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.label("Spatial Position");

        let is_spatial = self.preview_settings.is_spatial();
        let mut enable_spatial = is_spatial;

        if ui
            .checkbox(&mut enable_spatial, "Enable Spatial Audio")
            .changed()
        {
            if enable_spatial {
                self.preview_settings.enable_spatial(0.0, 0.0, 0.0);
            } else {
                self.preview_settings.disable_spatial();
            }
            self.emit_settings_update();
        }

        if let Some(current_pos) = self.preview_settings.spatial_position {
            // Get current values
            let (mut x, mut y, mut z) = current_pos;
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("X:");
                if ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed() {
                    changed = true;
                }
                ui.label("Y:");
                if ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed() {
                    changed = true;
                }
                ui.label("Z:");
                if ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed() {
                    changed = true;
                }
            });

            // Apply changes from drag values
            if changed {
                self.preview_settings.spatial_position = Some((x, y, z));
                self.emit_settings_update();
            }

            // Preset buttons
            let mut preset_clicked = None;
            ui.horizontal(|ui| {
                if ui.button("Origin").clicked() {
                    preset_clicked = Some((0.0, 0.0, 0.0));
                }
                if ui.button("Left").clicked() {
                    preset_clicked = Some((-5.0, 0.0, 0.0));
                }
                if ui.button("Right").clicked() {
                    preset_clicked = Some((5.0, 0.0, 0.0));
                }
                if ui.button("Behind").clicked() {
                    preset_clicked = Some((0.0, -5.0, 0.0));
                }
            });

            if let Some(pos) = preset_clicked {
                self.preview_settings.spatial_position = Some(pos);
                self.emit_settings_update();
            }
        }
    }

    /// Emit settings update if a sound is playing.
    fn emit_settings_update(&mut self) {
        if let Some(preview) = &self.playing_preview {
            self.pending_actions.push(SoundTestAction::UpdateSettings(
                preview.sound_id.clone(),
                self.preview_settings.clone(),
            ));
        }
    }
}

/// Quick sound player widget for simple play buttons.
pub struct QuickSoundPlayer<'a> {
    sound_id: &'a SoundAssetId,
    label: Option<&'a str>,
    is_playing: bool,
}

impl<'a> QuickSoundPlayer<'a> {
    /// Create a new quick sound player.
    pub fn new(sound_id: &'a SoundAssetId) -> Self {
        Self {
            sound_id,
            label: None,
            is_playing: false,
        }
    }

    /// Set the button label.
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set whether the sound is currently playing.
    pub fn is_playing(mut self, playing: bool) -> Self {
        self.is_playing = playing;
        self
    }

    /// Show the player and return action if clicked.
    pub fn show(self, ui: &mut Ui) -> Option<SoundTestAction> {
        let button_text = if self.is_playing {
            if let Some(label) = self.label {
                format!("⏹ {label}")
            } else {
                "⏹".to_string()
            }
        } else if let Some(label) = self.label {
            format!("▶ {label}")
        } else {
            "▶".to_string()
        };

        if ui.button(button_text).clicked() {
            if self.is_playing {
                Some(SoundTestAction::Stop(self.sound_id.clone()))
            } else {
                Some(SoundTestAction::Play(
                    self.sound_id.clone(),
                    SoundPreviewSettings::default(),
                ))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_asset_id() {
        let id = SoundAssetId::new("explosion.ogg");
        assert_eq!(id.0, "explosion.ogg");
        assert_eq!(format!("{}", id), "explosion.ogg");
    }

    #[test]
    fn test_sound_browser_category() {
        assert_eq!(SoundBrowserCategory::all().len(), 7);
        assert_eq!(SoundBrowserCategory::Music.display_name(), "Music");
        assert_eq!(SoundBrowserCategory::Sfx.icon(), "💥");
    }

    #[test]
    fn test_sound_entry_new() {
        let entry = SoundEntry::new("test.ogg", "Test Sound", SoundBrowserCategory::Sfx);
        assert_eq!(entry.id.0, "test.ogg");
        assert_eq!(entry.name, "Test Sound");
        assert_eq!(entry.category, SoundBrowserCategory::Sfx);
        assert!(!entry.favorite);
    }

    #[test]
    fn test_sound_entry_builder() {
        let entry = SoundEntry::new("music.ogg", "Epic Theme", SoundBrowserCategory::Music)
            .with_duration(180.0)
            .with_path("sounds/music/epic_theme.ogg")
            .with_tag("battle")
            .with_tag("epic")
            .with_favorite(true);

        assert_eq!(entry.duration, 180.0);
        assert_eq!(entry.path, "sounds/music/epic_theme.ogg");
        assert_eq!(entry.tags.len(), 2);
        assert!(entry.favorite);
    }

    #[test]
    fn test_sound_entry_formatted_duration() {
        let mut entry = SoundEntry::new("test.ogg", "Test", SoundBrowserCategory::Sfx);
        entry.duration = 0.0;
        assert_eq!(entry.formatted_duration(), "0:00");

        entry.duration = 65.0;
        assert_eq!(entry.formatted_duration(), "1:05");

        entry.duration = 3661.0;
        assert_eq!(entry.formatted_duration(), "61:01");
    }

    #[test]
    fn test_sound_entry_matches_search() {
        let entry = SoundEntry::new("explosion.ogg", "Big Explosion", SoundBrowserCategory::Sfx)
            .with_tag("combat")
            .with_tag("loud");

        assert!(entry.matches_search("explosion"));
        assert!(entry.matches_search("EXPLOSION")); // case insensitive
        assert!(entry.matches_search("Big"));
        assert!(entry.matches_search("combat")); // by tag
        assert!(entry.matches_search(".ogg")); // by path
        assert!(!entry.matches_search("music"));
    }

    #[test]
    fn test_sound_entry_matches_category() {
        let sfx = SoundEntry::new("boom.ogg", "Boom", SoundBrowserCategory::Sfx);
        let music =
            SoundEntry::new("theme.ogg", "Theme", SoundBrowserCategory::Music).with_favorite(true);

        // All matches everything
        assert!(sfx.matches_category(SoundBrowserCategory::All));
        assert!(music.matches_category(SoundBrowserCategory::All));

        // Specific category
        assert!(sfx.matches_category(SoundBrowserCategory::Sfx));
        assert!(!sfx.matches_category(SoundBrowserCategory::Music));

        // Favorites
        assert!(!sfx.matches_category(SoundBrowserCategory::Favorites));
        assert!(music.matches_category(SoundBrowserCategory::Favorites));
    }

    #[test]
    fn test_sound_preview_settings_default() {
        let settings = SoundPreviewSettings::default();
        assert_eq!(settings.volume, 1.0);
        assert_eq!(settings.pan, 0.0);
        assert_eq!(settings.pitch, 1.0);
        assert!(!settings.looping);
        assert!(!settings.is_spatial());
    }

    #[test]
    fn test_sound_preview_settings_spatial() {
        let mut settings = SoundPreviewSettings::default();
        assert!(!settings.is_spatial());

        settings.enable_spatial(1.0, 2.0, 3.0);
        assert!(settings.is_spatial());
        assert_eq!(settings.spatial_position, Some((1.0, 2.0, 3.0)));

        settings.disable_spatial();
        assert!(!settings.is_spatial());
    }

    #[test]
    fn test_sound_preview_settings_reset() {
        let mut settings = SoundPreviewSettings {
            volume: 0.5,
            pan: -0.5,
            pitch: 1.5,
            looping: true,
            spatial_position: Some((1.0, 2.0, 3.0)),
        };

        settings.reset();
        assert_eq!(settings.volume, 1.0);
        assert_eq!(settings.pan, 0.0);
        assert!(!settings.looping);
        assert!(!settings.is_spatial());
    }

    #[test]
    fn test_playing_preview_progress() {
        let preview = PlayingPreview {
            sound_id: SoundAssetId::new("test.ogg"),
            position: 0.0,
            duration: 10.0,
            paused: false,
        };
        assert_eq!(preview.progress(), 0.0);

        let preview = PlayingPreview {
            sound_id: SoundAssetId::new("test.ogg"),
            position: 5.0,
            duration: 10.0,
            paused: false,
        };
        assert!((preview.progress() - 0.5).abs() < 0.01);

        let preview = PlayingPreview {
            sound_id: SoundAssetId::new("test.ogg"),
            position: 15.0, // Over duration
            duration: 10.0,
            paused: false,
        };
        assert_eq!(preview.progress(), 1.0); // Clamped
    }

    #[test]
    fn test_playing_preview_zero_duration() {
        let preview = PlayingPreview {
            sound_id: SoundAssetId::new("test.ogg"),
            position: 5.0,
            duration: 0.0,
            paused: false,
        };
        assert_eq!(preview.progress(), 0.0);
    }

    #[test]
    fn test_sound_test_panel_new() {
        let panel = SoundTestPanel::new();
        assert!(!panel.open);
        assert!(panel.sounds.is_empty());
        assert_eq!(panel.category_filter, SoundBrowserCategory::All);
        assert!(panel.search_query.is_empty());
    }

    #[test]
    fn test_sound_test_panel_add_sounds() {
        let mut panel = SoundTestPanel::new();

        panel.add_sound(SoundEntry::new(
            "a.ogg",
            "Sound A",
            SoundBrowserCategory::Sfx,
        ));
        panel.add_sound(SoundEntry::new(
            "b.ogg",
            "Sound B",
            SoundBrowserCategory::Music,
        ));

        assert_eq!(panel.sounds.len(), 2);

        // Get by ID
        let sound = panel.get_sound(&SoundAssetId::new("a.ogg"));
        assert!(sound.is_some());
        assert_eq!(sound.unwrap().name, "Sound A");

        // Clear
        panel.clear_sounds();
        assert!(panel.sounds.is_empty());
    }

    #[test]
    fn test_sound_test_panel_set_sounds() {
        let mut panel = SoundTestPanel::new();
        panel.add_sound(SoundEntry::new("old.ogg", "Old", SoundBrowserCategory::Sfx));

        let new_sounds = vec![
            SoundEntry::new("new1.ogg", "New 1", SoundBrowserCategory::Sfx),
            SoundEntry::new("new2.ogg", "New 2", SoundBrowserCategory::Music),
        ];

        panel.set_sounds(new_sounds);
        assert_eq!(panel.sounds.len(), 2);
        assert!(panel.get_sound(&SoundAssetId::new("old.ogg")).is_none());
    }

    #[test]
    fn test_sound_test_panel_toggle_favorite() {
        let mut panel = SoundTestPanel::new();
        panel.add_sound(SoundEntry::new(
            "test.ogg",
            "Test",
            SoundBrowserCategory::Sfx,
        ));

        let id = SoundAssetId::new("test.ogg");
        assert!(!panel.get_sound(&id).unwrap().favorite);

        panel.toggle_favorite(&id);
        assert!(panel.get_sound(&id).unwrap().favorite);

        panel.toggle_favorite(&id);
        assert!(!panel.get_sound(&id).unwrap().favorite);
    }

    #[test]
    fn test_sound_test_panel_filtered_sounds() {
        let mut panel = SoundTestPanel::new();
        panel.add_sound(SoundEntry::new(
            "sfx1.ogg",
            "Explosion",
            SoundBrowserCategory::Sfx,
        ));
        panel.add_sound(SoundEntry::new(
            "sfx2.ogg",
            "Gunshot",
            SoundBrowserCategory::Sfx,
        ));
        panel.add_sound(
            SoundEntry::new("music.ogg", "Theme", SoundBrowserCategory::Music).with_favorite(true),
        );

        // All category
        panel.category_filter = SoundBrowserCategory::All;
        assert_eq!(panel.filtered_sounds().len(), 3);

        // SFX only
        panel.category_filter = SoundBrowserCategory::Sfx;
        assert_eq!(panel.filtered_sounds().len(), 2);

        // Music only
        panel.category_filter = SoundBrowserCategory::Music;
        assert_eq!(panel.filtered_sounds().len(), 1);

        // Favorites
        panel.category_filter = SoundBrowserCategory::Favorites;
        assert_eq!(panel.filtered_sounds().len(), 1);

        // Search filter
        panel.category_filter = SoundBrowserCategory::All;
        panel.search_query = "explosion".to_string();
        assert_eq!(panel.filtered_sounds().len(), 1);
    }

    #[test]
    fn test_sound_test_panel_toggle() {
        let mut panel = SoundTestPanel::new();
        assert!(!panel.open);

        panel.toggle();
        assert!(panel.open);

        panel.toggle();
        assert!(!panel.open);
    }

    #[test]
    fn test_sound_test_panel_config_default() {
        let config = SoundTestPanelConfig::default();
        assert_eq!(config.title, "Sound Test");
        assert!(config.show_categories);
        assert!(config.show_search);
        assert!(config.show_preview_controls);
        assert!(config.show_spatial_controls);
    }

    #[test]
    fn test_sound_test_action_equality() {
        let id = SoundAssetId::new("test.ogg");
        let settings = SoundPreviewSettings::default();

        let action1 = SoundTestAction::Play(id.clone(), settings.clone());
        let action2 = SoundTestAction::Play(id.clone(), settings.clone());
        assert_eq!(action1, action2);

        let action3 = SoundTestAction::Stop(id.clone());
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_sound_entry_serialization() {
        let entry = SoundEntry::new("test.ogg", "Test", SoundBrowserCategory::Sfx)
            .with_duration(5.0)
            .with_tag("loud");

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: SoundEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.id, loaded.id);
        assert_eq!(entry.name, loaded.name);
        assert_eq!(entry.duration, loaded.duration);
    }

    #[test]
    fn test_sound_preview_settings_serialization() {
        let settings = SoundPreviewSettings {
            volume: 0.75,
            pan: -0.5,
            pitch: 1.2,
            looping: true,
            spatial_position: Some((1.0, 2.0, 3.0)),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let loaded: SoundPreviewSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.volume, loaded.volume);
        assert_eq!(settings.spatial_position, loaded.spatial_position);
    }

    #[test]
    fn test_sound_browser_category_serialization() {
        for category in SoundBrowserCategory::all() {
            let json = serde_json::to_string(category).unwrap();
            let loaded: SoundBrowserCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(*category, loaded);
        }
    }

    #[test]
    fn test_sound_test_panel_config_serialization() {
        let config = SoundTestPanelConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: SoundTestPanelConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.title, loaded.title);
        assert_eq!(config.max_sounds_displayed, loaded.max_sounds_displayed);
    }
}
