//! Settings persistence system.
//!
//! This module provides:
//! - Load/save settings to TOML file
//! - Settings path: ~/.config/genesis/settings.toml
//! - Load on startup, save on change
//! - Reset to defaults option

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{error, info, warn};

/// Default settings file name.
pub const SETTINGS_FILE_NAME: &str = "settings.toml";

/// Default settings directory (relative to config).
pub const SETTINGS_DIR_NAME: &str = "genesis";

/// Errors that can occur during settings operations.
#[derive(Debug, Error)]
pub enum SettingsError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML serialization error.
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// TOML deserialization error.
    #[error("TOML parse error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    /// Settings file not found.
    #[error("Settings file not found: {0}")]
    NotFound(String),

    /// Invalid settings value.
    #[error("Invalid settings value: {0}")]
    InvalidValue(String),
}

/// Result type for settings operations.
pub type SettingsResult<T> = Result<T, SettingsError>;

/// Graphics quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GraphicsQuality {
    /// Low quality for performance.
    Low,
    /// Medium quality (default).
    #[default]
    Medium,
    /// High quality.
    High,
    /// Ultra quality.
    Ultra,
    /// Custom settings.
    Custom,
}

impl GraphicsQuality {
    /// Returns display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
            Self::Custom => "Custom",
        }
    }
}

/// Window mode setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WindowMode {
    /// Windowed mode.
    #[default]
    Windowed,
    /// Borderless fullscreen.
    Borderless,
    /// Exclusive fullscreen.
    Fullscreen,
}

impl WindowMode {
    /// Returns display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless Fullscreen",
            Self::Fullscreen => "Fullscreen",
        }
    }
}

/// Graphics settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Graphics quality preset.
    pub quality: GraphicsQuality,
    /// Window mode.
    pub window_mode: WindowMode,
    /// Resolution width.
    pub resolution_width: u32,
    /// Resolution height.
    pub resolution_height: u32,
    /// VSync enabled.
    pub vsync: bool,
    /// Target FPS (0 = unlimited).
    pub target_fps: u32,
    /// Render scale (0.5 - 2.0).
    pub render_scale: f32,
    /// Shadow quality (0-3).
    pub shadow_quality: u8,
    /// Anti-aliasing level (0-4).
    pub antialiasing: u8,
    /// Bloom enabled.
    pub bloom: bool,
    /// Ambient occlusion enabled.
    pub ambient_occlusion: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            quality: GraphicsQuality::default(),
            window_mode: WindowMode::default(),
            resolution_width: 1920,
            resolution_height: 1080,
            vsync: true,
            target_fps: 60,
            render_scale: 1.0,
            shadow_quality: 2,
            antialiasing: 2,
            bloom: true,
            ambient_occlusion: true,
        }
    }
}

/// Audio settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0 - 1.0).
    pub master_volume: f32,
    /// Music volume (0.0 - 1.0).
    pub music_volume: f32,
    /// Sound effects volume (0.0 - 1.0).
    pub sfx_volume: f32,
    /// Ambient volume (0.0 - 1.0).
    pub ambient_volume: f32,
    /// Voice volume (0.0 - 1.0).
    pub voice_volume: f32,
    /// UI sounds volume (0.0 - 1.0).
    pub ui_volume: f32,
    /// Mute when unfocused.
    pub mute_unfocused: bool,
    /// Subtitles enabled.
    pub subtitles: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,
            ambient_volume: 0.8,
            voice_volume: 1.0,
            ui_volume: 0.8,
            mute_unfocused: false,
            subtitles: false,
        }
    }
}

/// Gameplay settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Difficulty level (0-3).
    pub difficulty: u8,
    /// Auto-save enabled.
    pub autosave_enabled: bool,
    /// Auto-save interval in seconds.
    pub autosave_interval: u32,
    /// Tutorial tips enabled.
    pub tutorial_tips: bool,
    /// Camera shake intensity (0.0 - 1.0).
    pub camera_shake: f32,
    /// HUD scale (0.5 - 2.0).
    pub hud_scale: f32,
    /// Minimap enabled.
    pub show_minimap: bool,
    /// Damage numbers enabled.
    pub show_damage_numbers: bool,
    /// Health bars on enemies.
    pub show_enemy_health: bool,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            difficulty: 1, // Normal
            autosave_enabled: true,
            autosave_interval: 300, // 5 minutes
            tutorial_tips: true,
            camera_shake: 1.0,
            hud_scale: 1.0,
            show_minimap: true,
            show_damage_numbers: true,
            show_enemy_health: true,
        }
    }
}

/// Accessibility settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// Colorblind mode.
    pub colorblind_mode: u8, // 0=off, 1=protanopia, 2=deuteranopia, 3=tritanopia
    /// Screen reader support.
    pub screen_reader: bool,
    /// High contrast mode.
    pub high_contrast: bool,
    /// Large text mode.
    pub large_text: bool,
    /// Reduce motion.
    pub reduce_motion: bool,
    /// Button hold time for actions (ms).
    pub button_hold_time: u32,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            colorblind_mode: 0,
            screen_reader: false,
            high_contrast: false,
            large_text: false,
            reduce_motion: false,
            button_hold_time: 500,
        }
    }
}

/// Complete game settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Settings file version.
    pub version: u32,
    /// Graphics settings.
    pub graphics: GraphicsSettings,
    /// Audio settings.
    pub audio: AudioSettings,
    /// Gameplay settings.
    pub gameplay: GameplaySettings,
    /// Accessibility settings.
    pub accessibility: AccessibilitySettings,
    /// Custom key bindings (action -> key).
    pub key_bindings: HashMap<String, String>,
    /// Language code.
    pub language: String,
    /// First launch flag.
    pub first_launch: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            version: 1,
            graphics: GraphicsSettings::default(),
            audio: AudioSettings::default(),
            gameplay: GameplaySettings::default(),
            accessibility: AccessibilitySettings::default(),
            key_bindings: HashMap::new(),
            language: "en".to_string(),
            first_launch: true,
        }
    }
}

impl GameSettings {
    /// Creates default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates all settings values.
    pub fn validate(&mut self) {
        // Clamp volumes
        self.audio.master_volume = self.audio.master_volume.clamp(0.0, 1.0);
        self.audio.music_volume = self.audio.music_volume.clamp(0.0, 1.0);
        self.audio.sfx_volume = self.audio.sfx_volume.clamp(0.0, 1.0);
        self.audio.ambient_volume = self.audio.ambient_volume.clamp(0.0, 1.0);
        self.audio.voice_volume = self.audio.voice_volume.clamp(0.0, 1.0);
        self.audio.ui_volume = self.audio.ui_volume.clamp(0.0, 1.0);

        // Clamp render scale
        self.graphics.render_scale = self.graphics.render_scale.clamp(0.5, 2.0);

        // Clamp HUD scale
        self.gameplay.hud_scale = self.gameplay.hud_scale.clamp(0.5, 2.0);

        // Clamp camera shake
        self.gameplay.camera_shake = self.gameplay.camera_shake.clamp(0.0, 1.0);

        // Validate difficulty
        self.gameplay.difficulty = self.gameplay.difficulty.min(3);

        // Validate quality settings
        self.graphics.shadow_quality = self.graphics.shadow_quality.min(3);
        self.graphics.antialiasing = self.graphics.antialiasing.min(4);

        // Validate colorblind mode
        self.accessibility.colorblind_mode = self.accessibility.colorblind_mode.min(3);
    }

    /// Serializes to TOML string.
    pub fn to_toml(&self) -> SettingsResult<String> {
        let toml = toml::to_string_pretty(self)?;
        Ok(toml)
    }

    /// Deserializes from TOML string.
    pub fn from_toml(toml: &str) -> SettingsResult<Self> {
        let mut settings: Self = toml::from_str(toml)?;
        settings.validate();
        Ok(settings)
    }
}

/// Manager for settings persistence.
pub struct SettingsManager {
    /// Current settings.
    settings: GameSettings,
    /// Path to settings file.
    settings_path: PathBuf,
    /// Whether settings have been modified since last save.
    dirty: bool,
    /// Auto-save on change.
    auto_save: bool,
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsManager {
    /// Creates a new settings manager with default path.
    #[must_use]
    pub fn new() -> Self {
        let settings_path = Self::default_settings_path();
        Self {
            settings: GameSettings::default(),
            settings_path,
            dirty: false,
            auto_save: true,
        }
    }

    /// Creates a settings manager with a custom path.
    #[must_use]
    pub fn with_path(path: impl AsRef<Path>) -> Self {
        Self {
            settings: GameSettings::default(),
            settings_path: path.as_ref().to_path_buf(),
            dirty: false,
            auto_save: true,
        }
    }

    /// Returns the default settings path.
    #[must_use]
    pub fn default_settings_path() -> PathBuf {
        // Try XDG config first, then fall back to home
        if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(config_dir)
                .join(SETTINGS_DIR_NAME)
                .join(SETTINGS_FILE_NAME);
        }

        if let Some(home) = dirs::home_dir() {
            return home
                .join(".config")
                .join(SETTINGS_DIR_NAME)
                .join(SETTINGS_FILE_NAME);
        }

        // Fallback to current directory
        PathBuf::from(SETTINGS_FILE_NAME)
    }

    /// Returns the current settings path.
    #[must_use]
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    /// Returns a reference to the current settings.
    #[must_use]
    pub fn settings(&self) -> &GameSettings {
        &self.settings
    }

    /// Returns a mutable reference to the current settings.
    pub fn settings_mut(&mut self) -> &mut GameSettings {
        self.dirty = true;
        &mut self.settings
    }

    /// Returns whether settings have been modified.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Sets auto-save on change.
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    /// Loads settings from file.
    pub fn load(&mut self) -> SettingsResult<()> {
        if !self.settings_path.exists() {
            info!("Settings file not found, using defaults");
            self.settings = GameSettings::default();
            self.dirty = true; // Will save defaults
            return Ok(());
        }

        let contents = fs::read_to_string(&self.settings_path)?;
        self.settings = GameSettings::from_toml(&contents)?;
        self.dirty = false;

        info!("Settings loaded from {:?}", self.settings_path);
        Ok(())
    }

    /// Saves settings to file.
    pub fn save(&mut self) -> SettingsResult<()> {
        // Ensure directory exists
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = self.settings.to_toml()?;
        fs::write(&self.settings_path, toml)?;
        self.dirty = false;

        info!("Settings saved to {:?}", self.settings_path);
        Ok(())
    }

    /// Saves if dirty and auto-save is enabled.
    pub fn auto_save_if_dirty(&mut self) -> SettingsResult<()> {
        if self.auto_save && self.dirty {
            self.save()?;
        }
        Ok(())
    }

    /// Resets settings to defaults.
    pub fn reset_to_defaults(&mut self) {
        self.settings = GameSettings::default();
        self.dirty = true;
        info!("Settings reset to defaults");
    }

    /// Resets a specific section to defaults.
    pub fn reset_section(&mut self, section: &str) {
        match section {
            "graphics" => self.settings.graphics = GraphicsSettings::default(),
            "audio" => self.settings.audio = AudioSettings::default(),
            "gameplay" => self.settings.gameplay = GameplaySettings::default(),
            "accessibility" => self.settings.accessibility = AccessibilitySettings::default(),
            "key_bindings" => self.settings.key_bindings.clear(),
            _ => warn!("Unknown settings section: {}", section),
        }
        self.dirty = true;
    }

    /// Applies a graphics preset.
    pub fn apply_graphics_preset(&mut self, quality: GraphicsQuality) {
        self.settings.graphics.quality = quality;

        match quality {
            GraphicsQuality::Low => {
                self.settings.graphics.shadow_quality = 0;
                self.settings.graphics.antialiasing = 0;
                self.settings.graphics.bloom = false;
                self.settings.graphics.ambient_occlusion = false;
                self.settings.graphics.render_scale = 0.75;
            }
            GraphicsQuality::Medium => {
                self.settings.graphics.shadow_quality = 1;
                self.settings.graphics.antialiasing = 1;
                self.settings.graphics.bloom = true;
                self.settings.graphics.ambient_occlusion = false;
                self.settings.graphics.render_scale = 1.0;
            }
            GraphicsQuality::High => {
                self.settings.graphics.shadow_quality = 2;
                self.settings.graphics.antialiasing = 2;
                self.settings.graphics.bloom = true;
                self.settings.graphics.ambient_occlusion = true;
                self.settings.graphics.render_scale = 1.0;
            }
            GraphicsQuality::Ultra => {
                self.settings.graphics.shadow_quality = 3;
                self.settings.graphics.antialiasing = 4;
                self.settings.graphics.bloom = true;
                self.settings.graphics.ambient_occlusion = true;
                self.settings.graphics.render_scale = 1.25;
            }
            GraphicsQuality::Custom => {} // Don't change anything
        }

        self.dirty = true;
    }

    // === Convenience accessors ===

    /// Returns master volume.
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.settings.audio.master_volume
    }

    /// Sets master volume.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.settings.audio.master_volume = volume.clamp(0.0, 1.0);
        self.dirty = true;
    }

    /// Returns music volume.
    #[must_use]
    pub fn music_volume(&self) -> f32 {
        self.settings.audio.music_volume
    }

    /// Sets music volume.
    pub fn set_music_volume(&mut self, volume: f32) {
        self.settings.audio.music_volume = volume.clamp(0.0, 1.0);
        self.dirty = true;
    }

    /// Returns whether VSync is enabled.
    #[must_use]
    pub fn vsync(&self) -> bool {
        self.settings.graphics.vsync
    }

    /// Sets VSync.
    pub fn set_vsync(&mut self, vsync: bool) {
        self.settings.graphics.vsync = vsync;
        self.dirty = true;
    }

    /// Returns target FPS.
    #[must_use]
    pub fn target_fps(&self) -> u32 {
        self.settings.graphics.target_fps
    }

    /// Sets target FPS.
    pub fn set_target_fps(&mut self, fps: u32) {
        self.settings.graphics.target_fps = fps;
        self.dirty = true;
    }

    /// Returns whether this is first launch.
    #[must_use]
    pub fn is_first_launch(&self) -> bool {
        self.settings.first_launch
    }

    /// Marks first launch as complete.
    pub fn complete_first_launch(&mut self) {
        self.settings.first_launch = false;
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_settings_path() -> PathBuf {
        env::temp_dir().join("genesis_test_settings.toml")
    }

    fn cleanup_test_file(path: &Path) {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_graphics_quality_display() {
        assert_eq!(GraphicsQuality::Low.display_name(), "Low");
        assert_eq!(GraphicsQuality::Ultra.display_name(), "Ultra");
    }

    #[test]
    fn test_window_mode_display() {
        assert_eq!(WindowMode::Windowed.display_name(), "Windowed");
        assert_eq!(WindowMode::Fullscreen.display_name(), "Fullscreen");
    }

    #[test]
    fn test_game_settings_default() {
        let settings = GameSettings::default();
        assert_eq!(settings.version, 1);
        assert!(settings.first_launch);
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn test_game_settings_validate() {
        let mut settings = GameSettings::default();
        settings.audio.master_volume = 2.0; // Invalid
        settings.gameplay.difficulty = 10; // Invalid

        settings.validate();

        assert!((settings.audio.master_volume - 1.0).abs() < 0.001);
        assert_eq!(settings.gameplay.difficulty, 3);
    }

    #[test]
    fn test_game_settings_toml_roundtrip() {
        let settings = GameSettings::default();
        let toml = settings.to_toml().expect("Serialize failed");
        let loaded = GameSettings::from_toml(&toml).expect("Deserialize failed");

        assert_eq!(settings.version, loaded.version);
        assert_eq!(settings.language, loaded.language);
        assert!((settings.audio.master_volume - loaded.audio.master_volume).abs() < 0.001);
    }

    #[test]
    fn test_settings_manager_new() {
        let manager = SettingsManager::new();
        assert!(!manager.is_dirty());
        assert!(manager.settings().first_launch);
    }

    #[test]
    fn test_settings_manager_save_load() {
        let path = test_settings_path();
        cleanup_test_file(&path);

        let mut manager = SettingsManager::with_path(&path);
        manager.set_master_volume(0.5);
        manager.save().expect("Save failed");

        let mut manager2 = SettingsManager::with_path(&path);
        manager2.load().expect("Load failed");

        assert!((manager2.master_volume() - 0.5).abs() < 0.001);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_settings_manager_reset() {
        let mut manager = SettingsManager::new();
        manager.set_master_volume(0.1);

        manager.reset_to_defaults();

        assert!((manager.master_volume() - 1.0).abs() < 0.001);
        assert!(manager.is_dirty());
    }

    #[test]
    fn test_settings_manager_reset_section() {
        let mut manager = SettingsManager::new();
        manager.set_master_volume(0.1);

        manager.reset_section("audio");

        assert!((manager.master_volume() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_settings_manager_graphics_preset() {
        let mut manager = SettingsManager::new();

        manager.apply_graphics_preset(GraphicsQuality::Low);
        assert_eq!(manager.settings().graphics.shadow_quality, 0);
        assert!(!manager.settings().graphics.bloom);

        manager.apply_graphics_preset(GraphicsQuality::Ultra);
        assert_eq!(manager.settings().graphics.shadow_quality, 3);
        assert!(manager.settings().graphics.bloom);
    }

    #[test]
    fn test_settings_manager_first_launch() {
        let mut manager = SettingsManager::new();
        assert!(manager.is_first_launch());

        manager.complete_first_launch();
        assert!(!manager.is_first_launch());
    }

    #[test]
    fn test_settings_manager_dirty_tracking() {
        let mut manager = SettingsManager::new();
        assert!(!manager.is_dirty());

        let _ = manager.settings_mut();
        assert!(manager.is_dirty());
    }

    #[test]
    fn test_settings_error_display() {
        let err = SettingsError::NotFound("test.toml".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("not found"));
    }
}
