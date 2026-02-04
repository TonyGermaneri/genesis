//! Audio settings panel with volume controls and preferences.
//!
//! Provides a complete audio settings UI with:
//! - Master, music, SFX, and ambient volume sliders (0-100%)
//! - Mute toggles per category
//! - Spatial audio and mono audio options
//! - Apply/Reset/Defaults buttons
//! - Settings persistence via serialization

use egui::{Color32, Response, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Audio category for volume control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioCategory {
    /// Master volume affecting all audio.
    Master,
    /// Background music.
    Music,
    /// Sound effects.
    Sfx,
    /// Ambient/environmental sounds.
    Ambient,
}

impl AudioCategory {
    /// Get all audio categories.
    pub fn all() -> &'static [AudioCategory] {
        &[
            AudioCategory::Master,
            AudioCategory::Music,
            AudioCategory::Sfx,
            AudioCategory::Ambient,
        ]
    }

    /// Get display name for the category.
    pub fn display_name(&self) -> &'static str {
        match self {
            AudioCategory::Master => "Master Volume",
            AudioCategory::Music => "Music Volume",
            AudioCategory::Sfx => "SFX Volume",
            AudioCategory::Ambient => "Ambient Volume",
        }
    }

    /// Get short name for the category.
    pub fn short_name(&self) -> &'static str {
        match self {
            AudioCategory::Master => "Master",
            AudioCategory::Music => "Music",
            AudioCategory::Sfx => "SFX",
            AudioCategory::Ambient => "Ambient",
        }
    }
}

/// Volume settings for a single audio category.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CategoryVolume {
    /// Volume level (0.0 to 1.0, displayed as 0-100%).
    pub volume: f32,
    /// Whether this category is muted.
    pub muted: bool,
}

impl Default for CategoryVolume {
    fn default() -> Self {
        Self {
            volume: 1.0,
            muted: false,
        }
    }
}

impl CategoryVolume {
    /// Create a new category volume setting.
    pub fn new(volume: f32, muted: bool) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
            muted,
        }
    }

    /// Get effective volume (0 if muted).
    pub fn effective_volume(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume
        }
    }

    /// Get volume as percentage (0-100).
    pub fn as_percent(&self) -> u32 {
        (self.volume * 100.0).round() as u32
    }

    /// Set volume from percentage (0-100).
    pub fn set_percent(&mut self, percent: u32) {
        self.volume = (percent.min(100) as f32) / 100.0;
    }
}

/// Complete audio settings configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume settings.
    pub master: CategoryVolume,
    /// Music volume settings.
    pub music: CategoryVolume,
    /// SFX volume settings.
    pub sfx: CategoryVolume,
    /// Ambient volume settings.
    pub ambient: CategoryVolume,
    /// Enable spatial/3D audio.
    pub spatial_audio: bool,
    /// Force mono audio output.
    pub mono_audio: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: CategoryVolume::new(0.8, false),
            music: CategoryVolume::new(0.6, false),
            sfx: CategoryVolume::new(0.8, false),
            ambient: CategoryVolume::new(1.0, false),
            spatial_audio: true,
            mono_audio: false,
        }
    }
}

impl AudioSettings {
    /// Create new audio settings with all volumes at 100%.
    pub fn full_volume() -> Self {
        Self {
            master: CategoryVolume::new(1.0, false),
            music: CategoryVolume::new(1.0, false),
            sfx: CategoryVolume::new(1.0, false),
            ambient: CategoryVolume::new(1.0, false),
            spatial_audio: true,
            mono_audio: false,
        }
    }

    /// Create silent audio settings.
    pub fn silent() -> Self {
        Self {
            master: CategoryVolume::new(0.0, true),
            music: CategoryVolume::new(0.0, true),
            sfx: CategoryVolume::new(0.0, true),
            ambient: CategoryVolume::new(0.0, true),
            spatial_audio: false,
            mono_audio: false,
        }
    }

    /// Get volume settings for a category.
    pub fn get_category(&self, category: AudioCategory) -> &CategoryVolume {
        match category {
            AudioCategory::Master => &self.master,
            AudioCategory::Music => &self.music,
            AudioCategory::Sfx => &self.sfx,
            AudioCategory::Ambient => &self.ambient,
        }
    }

    /// Get mutable volume settings for a category.
    pub fn get_category_mut(&mut self, category: AudioCategory) -> &mut CategoryVolume {
        match category {
            AudioCategory::Master => &mut self.master,
            AudioCategory::Music => &mut self.music,
            AudioCategory::Sfx => &mut self.sfx,
            AudioCategory::Ambient => &mut self.ambient,
        }
    }

    /// Calculate effective volume for a category (applying master).
    pub fn effective_volume(&self, category: AudioCategory) -> f32 {
        let master = self.master.effective_volume();
        if category == AudioCategory::Master {
            master
        } else {
            master * self.get_category(category).effective_volume()
        }
    }

    /// Mute all audio categories.
    pub fn mute_all(&mut self) {
        self.master.muted = true;
        self.music.muted = true;
        self.sfx.muted = true;
        self.ambient.muted = true;
    }

    /// Unmute all audio categories.
    pub fn unmute_all(&mut self) {
        self.master.muted = false;
        self.music.muted = false;
        self.sfx.muted = false;
        self.ambient.muted = false;
    }

    /// Check if all audio is effectively muted.
    pub fn is_all_muted(&self) -> bool {
        self.master.muted
            || (self.music.muted && self.sfx.muted && self.ambient.muted)
            || self.master.volume == 0.0
    }
}

/// Configuration for the audio settings panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettingsPanelConfig {
    /// Panel title.
    pub title: String,
    /// Slider width in pixels.
    pub slider_width: f32,
    /// Show percentage labels.
    pub show_percentages: bool,
    /// Show mute toggles.
    pub show_mute_toggles: bool,
    /// Show audio options (spatial, mono).
    pub show_audio_options: bool,
    /// Show apply/reset buttons.
    pub show_buttons: bool,
    /// Volume slider color RGB values.
    pub slider_color_rgb: [u8; 3],
    /// Muted indicator color RGB values.
    pub muted_color_rgb: [u8; 3],
}

impl AudioSettingsPanelConfig {
    /// Get slider color as Color32.
    pub fn slider_color(&self) -> Color32 {
        Color32::from_rgb(
            self.slider_color_rgb[0],
            self.slider_color_rgb[1],
            self.slider_color_rgb[2],
        )
    }

    /// Get muted color as Color32.
    pub fn muted_color(&self) -> Color32 {
        Color32::from_rgb(
            self.muted_color_rgb[0],
            self.muted_color_rgb[1],
            self.muted_color_rgb[2],
        )
    }
}

impl Default for AudioSettingsPanelConfig {
    fn default() -> Self {
        Self {
            title: "Audio Settings".to_string(),
            slider_width: 200.0,
            show_percentages: true,
            show_mute_toggles: true,
            show_audio_options: true,
            show_buttons: true,
            slider_color_rgb: [100, 180, 100],
            muted_color_rgb: [180, 100, 100],
        }
    }
}

/// Actions returned by the audio settings panel.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioSettingsAction {
    /// A volume was changed.
    VolumeChanged(AudioCategory, f32),
    /// Mute state was toggled.
    MuteToggled(AudioCategory, bool),
    /// Spatial audio was toggled.
    SpatialAudioToggled(bool),
    /// Mono audio was toggled.
    MonoAudioToggled(bool),
    /// Apply button clicked - apply current settings.
    Apply,
    /// Reset button clicked - revert to saved settings.
    Reset,
    /// Defaults button clicked - reset to default settings.
    Defaults,
    /// Close button clicked.
    Close,
}

/// Audio settings panel widget.
#[derive(Debug)]
pub struct AudioSettingsPanel {
    /// Current settings being edited.
    pub settings: AudioSettings,
    /// Saved settings to revert to on reset.
    saved_settings: AudioSettings,
    /// Panel configuration.
    pub config: AudioSettingsPanelConfig,
    /// Whether the panel is open.
    pub open: bool,
    /// Pending actions from user interaction.
    pending_actions: Vec<AudioSettingsAction>,
}

impl AudioSettingsPanel {
    /// Create a new audio settings panel.
    pub fn new(settings: AudioSettings) -> Self {
        Self {
            saved_settings: settings.clone(),
            settings,
            config: AudioSettingsPanelConfig::default(),
            open: true,
            pending_actions: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(settings: AudioSettings, config: AudioSettingsPanelConfig) -> Self {
        Self {
            saved_settings: settings.clone(),
            settings,
            config,
            open: true,
            pending_actions: Vec::new(),
        }
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

    /// Check if settings have been modified.
    pub fn has_changes(&self) -> bool {
        self.settings != self.saved_settings
    }

    /// Apply current settings (marks them as saved).
    pub fn apply_settings(&mut self) {
        self.saved_settings = self.settings.clone();
    }

    /// Reset to saved settings.
    pub fn reset_settings(&mut self) {
        self.settings = self.saved_settings.clone();
    }

    /// Reset to default settings.
    pub fn reset_to_defaults(&mut self) {
        self.settings = AudioSettings::default();
    }

    /// Get current settings.
    pub fn current_settings(&self) -> &AudioSettings {
        &self.settings
    }

    /// Get saved settings.
    pub fn saved_settings(&self) -> &AudioSettings {
        &self.saved_settings
    }

    /// Update saved settings (e.g., after loading from file).
    pub fn set_saved_settings(&mut self, settings: AudioSettings) {
        self.saved_settings = settings.clone();
        self.settings = settings;
    }

    /// Render the panel and return any actions.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<AudioSettingsAction> {
        self.pending_actions.clear();

        if !self.open {
            return Vec::new();
        }

        ui.vertical(|ui| {
            // Title bar with close button
            ui.horizontal(|ui| {
                ui.heading(&self.config.title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        self.pending_actions.push(AudioSettingsAction::Close);
                        self.open = false;
                    }
                });
            });

            ui.separator();

            // Volume sliders
            for category in AudioCategory::all() {
                self.show_volume_slider(ui, *category);
            }

            ui.separator();

            // Audio options
            if self.config.show_audio_options {
                self.show_audio_options(ui);
                ui.separator();
            }

            // Buttons
            if self.config.show_buttons {
                self.show_buttons(ui);
            }
        });

        std::mem::take(&mut self.pending_actions)
    }

    /// Render a volume slider for a category.
    fn show_volume_slider(&mut self, ui: &mut Ui, category: AudioCategory) {
        let category_settings = self.settings.get_category_mut(category);
        let mut volume_percent = category_settings.as_percent() as f32;

        ui.horizontal(|ui| {
            // Category name
            ui.label(category.display_name());

            // Mute toggle
            if self.config.show_mute_toggles {
                let mute_text = if category_settings.muted {
                    "🔇"
                } else {
                    "🔊"
                };
                if ui.small_button(mute_text).clicked() {
                    category_settings.muted = !category_settings.muted;
                    self.pending_actions.push(AudioSettingsAction::MuteToggled(
                        category,
                        category_settings.muted,
                    ));
                }
            }

            // Volume slider
            let slider_color = if category_settings.muted {
                self.config.muted_color()
            } else {
                self.config.slider_color()
            };

            let slider = egui::Slider::new(&mut volume_percent, 0.0..=100.0)
                .show_value(false)
                .custom_formatter(|v, _| format!("{v:.0}%"));

            let response = ui.add_sized(
                Vec2::new(self.config.slider_width, ui.spacing().interact_size.y),
                slider,
            );

            // Apply slider color via painter
            if !category_settings.muted {
                let rect = response.rect;
                let fill_width = rect.width() * (volume_percent / 100.0);
                let fill_rect =
                    egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
                ui.painter()
                    .rect_filled(fill_rect, 2.0, slider_color.linear_multiply(0.3));
            }

            if response.changed() {
                category_settings.set_percent(volume_percent as u32);
                self.pending_actions
                    .push(AudioSettingsAction::VolumeChanged(
                        category,
                        category_settings.volume,
                    ));
            }

            // Percentage label
            if self.config.show_percentages {
                let text = if category_settings.muted {
                    "Muted".to_string()
                } else {
                    format!("{}%", volume_percent as u32)
                };
                ui.label(text);
            }
        });
    }

    /// Render audio options.
    fn show_audio_options(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.settings.spatial_audio, "Enable Spatial Audio")
                .changed()
            {
                self.pending_actions
                    .push(AudioSettingsAction::SpatialAudioToggled(
                        self.settings.spatial_audio,
                    ));
            }
        });

        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.settings.mono_audio, "Mono Audio")
                .changed()
            {
                self.pending_actions
                    .push(AudioSettingsAction::MonoAudioToggled(
                        self.settings.mono_audio,
                    ));
            }
        });
    }

    /// Render action buttons.
    fn show_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let has_changes = self.has_changes();

            // Apply button
            if ui
                .add_enabled(has_changes, egui::Button::new("Apply"))
                .clicked()
            {
                self.apply_settings();
                self.pending_actions.push(AudioSettingsAction::Apply);
            }

            // Reset button
            if ui
                .add_enabled(has_changes, egui::Button::new("Reset"))
                .clicked()
            {
                self.reset_settings();
                self.pending_actions.push(AudioSettingsAction::Reset);
            }

            // Defaults button
            if ui.button("Defaults").clicked() {
                self.reset_to_defaults();
                self.pending_actions.push(AudioSettingsAction::Defaults);
            }
        });
    }
}

/// Simple volume slider widget for inline use.
pub struct VolumeSlider<'a> {
    volume: &'a mut f32,
    label: &'a str,
    show_percentage: bool,
}

impl<'a> VolumeSlider<'a> {
    /// Create a new volume slider.
    pub fn new(volume: &'a mut f32, label: &'a str) -> Self {
        Self {
            volume,
            label,
            show_percentage: true,
        }
    }

    /// Set whether to show percentage label.
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Show the slider and return whether it changed.
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut percent = *self.volume * 100.0;

        let response = ui
            .horizontal(|ui| {
                ui.label(self.label);
                let slider_response =
                    ui.add(egui::Slider::new(&mut percent, 0.0..=100.0).show_value(false));

                if self.show_percentage {
                    ui.label(format!("{}%", percent as u32));
                }

                slider_response
            })
            .inner;

        if response.changed() {
            *self.volume = percent / 100.0;
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_category() {
        assert_eq!(AudioCategory::all().len(), 4);
        assert_eq!(AudioCategory::Master.display_name(), "Master Volume");
        assert_eq!(AudioCategory::Music.short_name(), "Music");
        assert_eq!(AudioCategory::Sfx.short_name(), "SFX");
        assert_eq!(AudioCategory::Ambient.display_name(), "Ambient Volume");
    }

    #[test]
    fn test_category_volume() {
        let mut vol = CategoryVolume::default();
        assert_eq!(vol.volume, 1.0);
        assert!(!vol.muted);
        assert_eq!(vol.effective_volume(), 1.0);
        assert_eq!(vol.as_percent(), 100);

        vol.set_percent(50);
        assert_eq!(vol.as_percent(), 50);
        assert!((vol.volume - 0.5).abs() < 0.01);

        vol.muted = true;
        assert_eq!(vol.effective_volume(), 0.0);
    }

    #[test]
    fn test_category_volume_clamping() {
        let vol = CategoryVolume::new(1.5, false);
        assert_eq!(vol.volume, 1.0);

        let vol = CategoryVolume::new(-0.5, false);
        assert_eq!(vol.volume, 0.0);
    }

    #[test]
    fn test_audio_settings_defaults() {
        let settings = AudioSettings::default();
        assert!((settings.master.volume - 0.8).abs() < 0.01);
        assert!((settings.music.volume - 0.6).abs() < 0.01);
        assert!((settings.sfx.volume - 0.8).abs() < 0.01);
        assert!((settings.ambient.volume - 1.0).abs() < 0.01);
        assert!(settings.spatial_audio);
        assert!(!settings.mono_audio);
    }

    #[test]
    fn test_audio_settings_full_volume() {
        let settings = AudioSettings::full_volume();
        assert_eq!(settings.master.volume, 1.0);
        assert_eq!(settings.music.volume, 1.0);
        assert_eq!(settings.sfx.volume, 1.0);
        assert_eq!(settings.ambient.volume, 1.0);
    }

    #[test]
    fn test_audio_settings_silent() {
        let settings = AudioSettings::silent();
        assert_eq!(settings.master.volume, 0.0);
        assert!(settings.master.muted);
        assert!(settings.is_all_muted());
    }

    #[test]
    fn test_audio_settings_get_category() {
        let settings = AudioSettings::default();
        assert_eq!(
            settings.get_category(AudioCategory::Master).volume,
            settings.master.volume
        );
        assert_eq!(
            settings.get_category(AudioCategory::Music).volume,
            settings.music.volume
        );
    }

    #[test]
    fn test_audio_settings_effective_volume() {
        let mut settings = AudioSettings::default();
        settings.master.volume = 0.5;
        settings.music.volume = 0.8;

        // Master returns its own volume
        assert!((settings.effective_volume(AudioCategory::Master) - 0.5).abs() < 0.01);
        // Music is master * music = 0.5 * 0.8 = 0.4
        assert!((settings.effective_volume(AudioCategory::Music) - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_audio_settings_mute_all() {
        let mut settings = AudioSettings::default();
        settings.mute_all();
        assert!(settings.master.muted);
        assert!(settings.music.muted);
        assert!(settings.sfx.muted);
        assert!(settings.ambient.muted);
        assert!(settings.is_all_muted());

        settings.unmute_all();
        assert!(!settings.master.muted);
        assert!(!settings.is_all_muted());
    }

    #[test]
    fn test_audio_settings_panel_new() {
        let panel = AudioSettingsPanel::new(AudioSettings::default());
        assert!(panel.open);
        assert!(!panel.has_changes());
    }

    #[test]
    fn test_audio_settings_panel_changes() {
        let mut panel = AudioSettingsPanel::new(AudioSettings::default());
        assert!(!panel.has_changes());

        panel.settings.master.volume = 0.5;
        assert!(panel.has_changes());

        panel.apply_settings();
        assert!(!panel.has_changes());
    }

    #[test]
    fn test_audio_settings_panel_reset() {
        let mut panel = AudioSettingsPanel::new(AudioSettings::default());
        let original_volume = panel.settings.master.volume;

        panel.settings.master.volume = 0.1;
        assert!(panel.has_changes());

        panel.reset_settings();
        assert_eq!(panel.settings.master.volume, original_volume);
        assert!(!panel.has_changes());
    }

    #[test]
    fn test_audio_settings_panel_reset_to_defaults() {
        let mut panel = AudioSettingsPanel::new(AudioSettings::silent());
        panel.reset_to_defaults();

        let defaults = AudioSettings::default();
        assert_eq!(panel.settings.master.volume, defaults.master.volume);
    }

    #[test]
    fn test_audio_settings_panel_toggle() {
        let mut panel = AudioSettingsPanel::new(AudioSettings::default());
        assert!(panel.open);

        panel.toggle();
        assert!(!panel.open);

        panel.toggle();
        assert!(panel.open);
    }

    #[test]
    fn test_audio_settings_panel_config() {
        let config = AudioSettingsPanelConfig::default();
        assert_eq!(config.title, "Audio Settings");
        assert!(config.show_percentages);
        assert!(config.show_mute_toggles);
        assert!(config.show_audio_options);
        assert!(config.show_buttons);
    }

    #[test]
    fn test_audio_settings_serialization() {
        let settings = AudioSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let loaded: AudioSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, loaded);
    }

    #[test]
    fn test_category_volume_serialization() {
        let vol = CategoryVolume::new(0.75, true);
        let json = serde_json::to_string(&vol).unwrap();
        let loaded: CategoryVolume = serde_json::from_str(&json).unwrap();
        assert_eq!(vol, loaded);
    }

    #[test]
    fn test_audio_category_serialization() {
        let category = AudioCategory::Sfx;
        let json = serde_json::to_string(&category).unwrap();
        let loaded: AudioCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, loaded);
    }

    #[test]
    fn test_audio_settings_panel_set_saved() {
        let mut panel = AudioSettingsPanel::new(AudioSettings::default());
        let new_settings = AudioSettings::full_volume();

        panel.set_saved_settings(new_settings.clone());
        assert_eq!(panel.settings, new_settings);
        assert_eq!(panel.saved_settings, new_settings);
        assert!(!panel.has_changes());
    }

    #[test]
    fn test_effective_volume_with_muted_master() {
        let mut settings = AudioSettings::default();
        settings.master.muted = true;
        assert_eq!(settings.effective_volume(AudioCategory::Master), 0.0);
        assert_eq!(settings.effective_volume(AudioCategory::Music), 0.0);
    }

    #[test]
    fn test_is_all_muted_variations() {
        let mut settings = AudioSettings::default();

        // Master muted = all muted
        settings.master.muted = true;
        assert!(settings.is_all_muted());
        settings.master.muted = false;

        // Master volume 0 = all muted
        settings.master.volume = 0.0;
        assert!(settings.is_all_muted());
        settings.master.volume = 1.0;

        // All categories muted (but not master)
        settings.music.muted = true;
        settings.sfx.muted = true;
        settings.ambient.muted = true;
        assert!(settings.is_all_muted());

        // One category unmuted = not all muted
        settings.music.muted = false;
        assert!(!settings.is_all_muted());
    }
}
