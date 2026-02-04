//! Game settings data model.
//!
//! This module provides settings structures for:
//! - Graphics settings (resolution, fullscreen, quality)
//! - Audio settings (volume levels)
//! - Control settings (sensitivity, key bindings)
//! - Gameplay settings (difficulty, auto-save)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// G-58: Graphics Settings
// ============================================================================

/// Screen resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Resolution {
    /// Create a new resolution.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 720p HD resolution.
    pub const HD: Self = Self::new(1280, 720);
    /// 1080p Full HD resolution.
    pub const FULL_HD: Self = Self::new(1920, 1080);
    /// 1440p QHD resolution.
    pub const QHD: Self = Self::new(2560, 1440);
    /// 4K UHD resolution.
    pub const UHD: Self = Self::new(3840, 2160);

    /// Get aspect ratio.
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Get common resolutions.
    #[must_use]
    pub fn common() -> Vec<Self> {
        vec![
            Self::new(1280, 720),
            Self::new(1366, 768),
            Self::new(1600, 900),
            Self::new(1920, 1080),
            Self::new(2560, 1440),
            Self::new(3840, 2160),
        ]
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self::FULL_HD
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Window mode setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WindowMode {
    /// Windowed mode.
    Windowed,
    /// Borderless fullscreen.
    #[default]
    Borderless,
    /// Exclusive fullscreen.
    Fullscreen,
}

impl WindowMode {
    /// Check if fullscreen.
    #[must_use]
    pub const fn is_fullscreen(&self) -> bool {
        matches!(self, Self::Borderless | Self::Fullscreen)
    }
}

/// Graphics quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum QualityLevel {
    /// Low quality for performance.
    Low,
    /// Medium quality.
    #[default]
    Medium,
    /// High quality.
    High,
    /// Ultra quality.
    Ultra,
    /// Custom settings.
    Custom,
}

impl QualityLevel {
    /// Get render distance for quality level.
    #[must_use]
    pub const fn render_distance(&self) -> u32 {
        match self {
            Self::Low => 8,
            Self::Medium | Self::Custom => 12,
            Self::High => 16,
            Self::Ultra => 24,
        }
    }

    /// Get shadow quality multiplier.
    #[must_use]
    pub const fn shadow_quality(&self) -> f32 {
        match self {
            Self::Low => 0.5,
            Self::Medium | Self::Custom => 1.0,
            Self::High => 1.5,
            Self::Ultra => 2.0,
        }
    }
}

/// VSync mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VsyncMode {
    /// VSync disabled.
    Off,
    /// VSync enabled.
    #[default]
    On,
    /// Adaptive VSync.
    Adaptive,
}

/// Anti-aliasing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AntiAliasing {
    /// No anti-aliasing.
    #[default]
    None,
    /// FXAA.
    Fxaa,
    /// MSAA 2x.
    Msaa2x,
    /// MSAA 4x.
    Msaa4x,
    /// MSAA 8x.
    Msaa8x,
    /// TAA.
    Taa,
}

/// Graphics settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Screen resolution.
    pub resolution: Resolution,
    /// Window mode.
    pub window_mode: WindowMode,
    /// VSync setting.
    pub vsync: VsyncMode,
    /// Quality preset.
    pub quality: QualityLevel,
    /// Render distance in chunks.
    pub render_distance: u32,
    /// Anti-aliasing mode.
    pub anti_aliasing: AntiAliasing,
    /// Shadow quality (0.0 to 2.0).
    pub shadow_quality: f32,
    /// Texture quality (0.0 to 1.0).
    pub texture_quality: f32,
    /// Enable bloom effects.
    pub bloom_enabled: bool,
    /// Enable ambient occlusion.
    pub ambient_occlusion: bool,
    /// Field of view in degrees.
    pub fov: f32,
    /// Brightness adjustment (-1.0 to 1.0).
    pub brightness: f32,
    /// Gamma adjustment (0.5 to 3.0).
    pub gamma: f32,
    /// Maximum framerate (0 = unlimited).
    pub max_fps: u32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: Resolution::default(),
            window_mode: WindowMode::default(),
            vsync: VsyncMode::default(),
            quality: QualityLevel::default(),
            render_distance: 12,
            anti_aliasing: AntiAliasing::default(),
            shadow_quality: 1.0,
            texture_quality: 1.0,
            bloom_enabled: true,
            ambient_occlusion: true,
            fov: 75.0,
            brightness: 0.0,
            gamma: 1.0,
            max_fps: 0,
        }
    }
}

impl GraphicsSettings {
    /// Apply quality preset.
    pub fn apply_quality_preset(&mut self, quality: QualityLevel) {
        self.quality = quality;
        match quality {
            QualityLevel::Low => {
                self.render_distance = 8;
                self.shadow_quality = 0.5;
                self.texture_quality = 0.5;
                self.bloom_enabled = false;
                self.ambient_occlusion = false;
                self.anti_aliasing = AntiAliasing::None;
            },
            QualityLevel::Medium => {
                self.render_distance = 12;
                self.shadow_quality = 1.0;
                self.texture_quality = 1.0;
                self.bloom_enabled = true;
                self.ambient_occlusion = false;
                self.anti_aliasing = AntiAliasing::Fxaa;
            },
            QualityLevel::High => {
                self.render_distance = 16;
                self.shadow_quality = 1.5;
                self.texture_quality = 1.0;
                self.bloom_enabled = true;
                self.ambient_occlusion = true;
                self.anti_aliasing = AntiAliasing::Msaa4x;
            },
            QualityLevel::Ultra => {
                self.render_distance = 24;
                self.shadow_quality = 2.0;
                self.texture_quality = 1.0;
                self.bloom_enabled = true;
                self.ambient_occlusion = true;
                self.anti_aliasing = AntiAliasing::Msaa8x;
            },
            QualityLevel::Custom => {},
        }
    }

    /// Validate settings.
    #[must_use]
    pub fn validate(&self) -> SettingsValidation {
        let mut validation = SettingsValidation::new();

        if self.resolution.width < 640 || self.resolution.height < 480 {
            validation.add_warning("Resolution is very low");
        }

        if self.fov < 60.0 || self.fov > 120.0 {
            validation.add_warning("FOV is outside recommended range (60-120)");
        }

        if self.gamma < 0.5 || self.gamma > 3.0 {
            validation.add_error("Gamma must be between 0.5 and 3.0");
        }

        validation
    }
}

// ============================================================================
// G-58: Audio Settings
// ============================================================================

/// Audio settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0 to 1.0).
    pub master_volume: f32,
    /// Music volume (0.0 to 1.0).
    pub music_volume: f32,
    /// Sound effects volume (0.0 to 1.0).
    pub sfx_volume: f32,
    /// Ambient sounds volume (0.0 to 1.0).
    pub ambient_volume: f32,
    /// Voice/dialogue volume (0.0 to 1.0).
    pub voice_volume: f32,
    /// UI sounds volume (0.0 to 1.0).
    pub ui_volume: f32,
    /// Enable audio.
    pub audio_enabled: bool,
    /// Mute when game loses focus.
    pub mute_on_focus_loss: bool,
    /// Enable subtitles.
    pub subtitles_enabled: bool,
    /// Audio output device name.
    pub output_device: Option<String>,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 1.0,
            ambient_volume: 0.6,
            voice_volume: 1.0,
            ui_volume: 0.8,
            audio_enabled: true,
            mute_on_focus_loss: true,
            subtitles_enabled: false,
            output_device: None,
        }
    }
}

impl AudioSettings {
    /// Get effective volume for a channel.
    #[must_use]
    pub fn effective_volume(&self, channel_volume: f32) -> f32 {
        if self.audio_enabled {
            self.master_volume * channel_volume
        } else {
            0.0
        }
    }

    /// Get effective music volume.
    #[must_use]
    pub fn effective_music_volume(&self) -> f32 {
        self.effective_volume(self.music_volume)
    }

    /// Get effective SFX volume.
    #[must_use]
    pub fn effective_sfx_volume(&self) -> f32 {
        self.effective_volume(self.sfx_volume)
    }

    /// Get effective ambient volume.
    #[must_use]
    pub fn effective_ambient_volume(&self) -> f32 {
        self.effective_volume(self.ambient_volume)
    }

    /// Validate settings.
    #[must_use]
    pub fn validate(&self) -> SettingsValidation {
        let mut validation = SettingsValidation::new();

        let volumes = [
            ("master", self.master_volume),
            ("music", self.music_volume),
            ("sfx", self.sfx_volume),
            ("ambient", self.ambient_volume),
            ("voice", self.voice_volume),
            ("ui", self.ui_volume),
        ];

        for (name, vol) in volumes {
            if !(0.0..=1.0).contains(&vol) {
                validation.add_error(&format!("{name} volume must be between 0.0 and 1.0"));
            }
        }

        validation
    }
}

// ============================================================================
// G-58: Control Settings
// ============================================================================

/// Input action that can be bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputAction {
    /// Move forward.
    MoveForward,
    /// Move backward.
    MoveBackward,
    /// Move left.
    MoveLeft,
    /// Move right.
    MoveRight,
    /// Jump.
    Jump,
    /// Crouch.
    Crouch,
    /// Sprint.
    Sprint,
    /// Attack.
    Attack,
    /// Block.
    Block,
    /// Dodge.
    Dodge,
    /// Special ability.
    SpecialAbility,
    /// Interact with object.
    Interact,
    /// Use item.
    Use,
    /// Pick up item.
    PickUp,
    /// Open inventory.
    Inventory,
    /// Open map.
    Map,
    /// Open journal.
    Journal,
    /// Pause game.
    Pause,
    /// Quick save.
    QuickSave,
    /// Quick load.
    QuickLoad,
    /// Zoom in camera.
    ZoomIn,
    /// Zoom out camera.
    ZoomOut,
    /// Toggle first person view.
    ToggleFirstPerson,
    /// Hotbar slot 1.
    Hotbar1,
    /// Hotbar slot 2.
    Hotbar2,
    /// Hotbar slot 3.
    Hotbar3,
    /// Hotbar slot 4.
    Hotbar4,
    /// Hotbar slot 5.
    Hotbar5,
    /// Hotbar slot 6.
    Hotbar6,
    /// Hotbar slot 7.
    Hotbar7,
    /// Hotbar slot 8.
    Hotbar8,
    /// Hotbar slot 9.
    Hotbar9,
    /// Hotbar slot 0.
    Hotbar0,
}

impl InputAction {
    /// Get default key binding.
    #[must_use]
    pub fn default_key(&self) -> &'static str {
        match self {
            Self::MoveForward => "W",
            Self::MoveBackward => "S",
            Self::MoveLeft => "A",
            Self::MoveRight => "D",
            Self::Jump => "Space",
            Self::Crouch => "LCtrl",
            Self::Sprint => "LShift",
            Self::Attack => "Mouse1",
            Self::Block => "Mouse2",
            Self::Dodge => "LAlt",
            Self::SpecialAbility => "Q",
            Self::Interact => "E",
            Self::Use => "F",
            Self::PickUp => "G",
            Self::Inventory => "I",
            Self::Map => "M",
            Self::Journal => "J",
            Self::Pause => "Escape",
            Self::QuickSave => "F5",
            Self::QuickLoad => "F9",
            Self::ZoomIn => "ScrollUp",
            Self::ZoomOut => "ScrollDown",
            Self::ToggleFirstPerson => "V",
            Self::Hotbar1 => "1",
            Self::Hotbar2 => "2",
            Self::Hotbar3 => "3",
            Self::Hotbar4 => "4",
            Self::Hotbar5 => "5",
            Self::Hotbar6 => "6",
            Self::Hotbar7 => "7",
            Self::Hotbar8 => "8",
            Self::Hotbar9 => "9",
            Self::Hotbar0 => "0",
        }
    }

    /// Get action category.
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::MoveForward
            | Self::MoveBackward
            | Self::MoveLeft
            | Self::MoveRight
            | Self::Jump
            | Self::Crouch
            | Self::Sprint => "Movement",

            Self::Attack | Self::Block | Self::Dodge | Self::SpecialAbility => "Combat",

            Self::Interact | Self::Use | Self::PickUp => "Interaction",

            Self::Inventory
            | Self::Map
            | Self::Journal
            | Self::Pause
            | Self::QuickSave
            | Self::QuickLoad => "Interface",

            Self::ZoomIn | Self::ZoomOut | Self::ToggleFirstPerson => "Camera",

            Self::Hotbar1
            | Self::Hotbar2
            | Self::Hotbar3
            | Self::Hotbar4
            | Self::Hotbar5
            | Self::Hotbar6
            | Self::Hotbar7
            | Self::Hotbar8
            | Self::Hotbar9
            | Self::Hotbar0 => "Hotbar",
        }
    }

    /// Get all actions.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::MoveForward,
            Self::MoveBackward,
            Self::MoveLeft,
            Self::MoveRight,
            Self::Jump,
            Self::Crouch,
            Self::Sprint,
            Self::Attack,
            Self::Block,
            Self::Dodge,
            Self::SpecialAbility,
            Self::Interact,
            Self::Use,
            Self::PickUp,
            Self::Inventory,
            Self::Map,
            Self::Journal,
            Self::Pause,
            Self::QuickSave,
            Self::QuickLoad,
            Self::ZoomIn,
            Self::ZoomOut,
            Self::ToggleFirstPerson,
            Self::Hotbar1,
            Self::Hotbar2,
            Self::Hotbar3,
            Self::Hotbar4,
            Self::Hotbar5,
            Self::Hotbar6,
            Self::Hotbar7,
            Self::Hotbar8,
            Self::Hotbar9,
            Self::Hotbar0,
        ]
    }
}

/// Control settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlSettings {
    /// Mouse sensitivity (0.1 to 5.0).
    pub mouse_sensitivity: f32,
    /// Invert Y axis.
    pub invert_y: bool,
    /// Invert X axis.
    pub invert_x: bool,
    /// Toggle crouch vs hold.
    pub toggle_crouch: bool,
    /// Toggle sprint vs hold.
    pub toggle_sprint: bool,
    /// Camera smoothing (0.0 to 1.0).
    pub camera_smoothing: f32,
    /// Controller deadzone (0.0 to 0.5).
    pub controller_deadzone: f32,
    /// Key bindings.
    pub key_bindings: HashMap<InputAction, String>,
    /// Enable controller.
    pub controller_enabled: bool,
    /// Controller vibration enabled.
    pub vibration_enabled: bool,
}

impl Default for ControlSettings {
    fn default() -> Self {
        let mut key_bindings = HashMap::new();
        for action in InputAction::all() {
            key_bindings.insert(*action, action.default_key().to_string());
        }

        Self {
            mouse_sensitivity: 1.0,
            invert_y: false,
            invert_x: false,
            toggle_crouch: false,
            toggle_sprint: false,
            camera_smoothing: 0.3,
            controller_deadzone: 0.15,
            key_bindings,
            controller_enabled: true,
            vibration_enabled: true,
        }
    }
}

impl ControlSettings {
    /// Get key binding for action.
    #[must_use]
    pub fn get_binding(&self, action: InputAction) -> &str {
        self.key_bindings
            .get(&action)
            .map_or(action.default_key(), String::as_str)
    }

    /// Set key binding for action.
    pub fn set_binding(&mut self, action: InputAction, key: String) {
        self.key_bindings.insert(action, key);
    }

    /// Reset binding to default.
    pub fn reset_binding(&mut self, action: InputAction) {
        self.key_bindings
            .insert(action, action.default_key().to_string());
    }

    /// Reset all bindings to default.
    pub fn reset_all_bindings(&mut self) {
        for action in InputAction::all() {
            self.key_bindings
                .insert(*action, action.default_key().to_string());
        }
    }

    /// Check for conflicting bindings.
    #[must_use]
    pub fn find_conflicts(&self) -> Vec<(InputAction, InputAction, String)> {
        let mut conflicts = Vec::new();
        let actions: Vec<_> = InputAction::all().iter().collect();

        for (i, action1) in actions.iter().enumerate() {
            for action2 in actions.iter().skip(i + 1) {
                let key1 = self.get_binding(**action1);
                let key2 = self.get_binding(**action2);

                if key1 == key2 && action1.category() == action2.category() {
                    conflicts.push((**action1, **action2, key1.to_string()));
                }
            }
        }

        conflicts
    }

    /// Validate settings.
    #[must_use]
    pub fn validate(&self) -> SettingsValidation {
        let mut validation = SettingsValidation::new();

        if !(0.1..=5.0).contains(&self.mouse_sensitivity) {
            validation.add_warning("Mouse sensitivity is outside recommended range (0.1-5.0)");
        }

        if !(0.0..=0.5).contains(&self.controller_deadzone) {
            validation.add_error("Controller deadzone must be between 0.0 and 0.5");
        }

        let conflicts = self.find_conflicts();
        for (a1, a2, key) in conflicts {
            validation.add_warning(&format!("{a1:?} and {a2:?} both bound to {key}"));
        }

        validation
    }
}

// ============================================================================
// G-58: Gameplay Settings
// ============================================================================

/// Game difficulty level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GameDifficulty {
    /// Peaceful mode - no enemies.
    Peaceful,
    /// Easy difficulty.
    Easy,
    /// Normal difficulty.
    #[default]
    Normal,
    /// Hard difficulty.
    Hard,
    /// Hardcore difficulty (permadeath).
    Hardcore,
}

impl GameDifficulty {
    /// Get damage multiplier for player.
    #[must_use]
    pub const fn player_damage_multiplier(&self) -> f32 {
        match self {
            Self::Peaceful => 1.5,
            Self::Easy => 1.25,
            Self::Normal => 1.0,
            Self::Hard => 0.8,
            Self::Hardcore => 0.6,
        }
    }

    /// Get damage multiplier for enemies.
    #[must_use]
    pub const fn enemy_damage_multiplier(&self) -> f32 {
        match self {
            Self::Peaceful => 0.0,
            Self::Easy => 0.5,
            Self::Normal => 1.0,
            Self::Hard => 1.5,
            Self::Hardcore => 2.0,
        }
    }

    /// Check if permadeath is enabled.
    #[must_use]
    pub const fn is_permadeath(&self) -> bool {
        matches!(self, Self::Hardcore)
    }

    /// Check if enemies spawn.
    #[must_use]
    pub const fn enemies_spawn(&self) -> bool {
        !matches!(self, Self::Peaceful)
    }
}

/// HUD visibility preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HudVisibility {
    /// Minimal HUD.
    Minimal,
    /// Standard HUD.
    #[default]
    Standard,
    /// Full HUD with all info.
    Full,
    /// Completely hidden.
    Hidden,
}

/// Gameplay settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Game difficulty.
    pub difficulty: GameDifficulty,
    /// Auto-save interval in minutes (0 = disabled).
    pub auto_save_interval: u32,
    /// Show tutorial prompts.
    pub show_tutorials: bool,
    /// Show damage numbers.
    pub show_damage_numbers: bool,
    /// Show item tooltips.
    pub show_tooltips: bool,
    /// HUD visibility.
    pub hud_visibility: HudVisibility,
    /// Enable screenshake.
    pub screenshake_enabled: bool,
    /// Screenshake intensity (0.0 to 2.0).
    pub screenshake_intensity: f32,
    /// Camera shake on hit.
    pub hit_feedback: bool,
    /// Show minimap.
    pub show_minimap: bool,
    /// Show compass.
    pub show_compass: bool,
    /// Show quest markers.
    pub show_quest_markers: bool,
    /// Enable auto-pickup of items.
    pub auto_pickup: bool,
    /// Auto-equip better gear.
    pub auto_equip: bool,
    /// Language code (e.g., "en", "es", "fr").
    pub language: String,
    /// Text size multiplier (0.5 to 2.0).
    pub text_size: f32,
    /// Colorblind mode.
    pub colorblind_mode: ColorblindMode,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            difficulty: GameDifficulty::default(),
            auto_save_interval: 5,
            show_tutorials: true,
            show_damage_numbers: true,
            show_tooltips: true,
            hud_visibility: HudVisibility::default(),
            screenshake_enabled: true,
            screenshake_intensity: 1.0,
            hit_feedback: true,
            show_minimap: true,
            show_compass: true,
            show_quest_markers: true,
            auto_pickup: false,
            auto_equip: false,
            language: "en".to_string(),
            text_size: 1.0,
            colorblind_mode: ColorblindMode::default(),
        }
    }
}

impl GameplaySettings {
    /// Validate settings.
    #[must_use]
    pub fn validate(&self) -> SettingsValidation {
        let mut validation = SettingsValidation::new();

        if !(0.5..=2.0).contains(&self.text_size) {
            validation.add_warning("Text size is outside recommended range (0.5-2.0)");
        }

        if !(0.0..=2.0).contains(&self.screenshake_intensity) {
            validation.add_error("Screenshake intensity must be between 0.0 and 2.0");
        }

        validation
    }
}

/// Colorblind assistance mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColorblindMode {
    /// No colorblind assistance.
    #[default]
    None,
    /// Deuteranopia (green-blind).
    Deuteranopia,
    /// Protanopia (red-blind).
    Protanopia,
    /// Tritanopia (blue-blind).
    Tritanopia,
}

// ============================================================================
// G-58: Settings Validation
// ============================================================================

/// Validation result for settings.
#[derive(Debug, Clone, Default)]
pub struct SettingsValidation {
    /// Error messages.
    pub errors: Vec<String>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl SettingsValidation {
    /// Create empty validation result.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error.
    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    /// Add a warning.
    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    /// Check if validation passed (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Merge another validation result.
    pub fn merge(&mut self, other: Self) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

// ============================================================================
// G-58: Combined Game Settings
// ============================================================================

/// All game settings combined.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    /// Graphics settings.
    pub graphics: GraphicsSettings,
    /// Audio settings.
    pub audio: AudioSettings,
    /// Control settings.
    pub controls: ControlSettings,
    /// Gameplay settings.
    pub gameplay: GameplaySettings,
    /// Settings version for migration.
    pub version: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            audio: AudioSettings::default(),
            controls: ControlSettings::default(),
            gameplay: GameplaySettings::default(),
            version: 1,
        }
    }
}

impl GameSettings {
    /// Create new default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate all settings.
    #[must_use]
    pub fn validate(&self) -> SettingsValidation {
        let mut validation = SettingsValidation::new();
        validation.merge(self.graphics.validate());
        validation.merge(self.audio.validate());
        validation.merge(self.controls.validate());
        validation.merge(self.gameplay.validate());
        validation
    }

    /// Reset all settings to defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Reset graphics settings.
    pub fn reset_graphics(&mut self) {
        self.graphics = GraphicsSettings::default();
    }

    /// Reset audio settings.
    pub fn reset_audio(&mut self) {
        self.audio = AudioSettings::default();
    }

    /// Reset control settings.
    pub fn reset_controls(&mut self) {
        self.controls = ControlSettings::default();
    }

    /// Reset gameplay settings.
    pub fn reset_gameplay(&mut self) {
        self.gameplay = GameplaySettings::default();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution() {
        let res = Resolution::new(1920, 1080);
        assert_eq!(res.width, 1920);
        assert_eq!(res.height, 1080);
        assert!((res.aspect_ratio() - 16.0 / 9.0).abs() < 0.01);
        assert_eq!(res.to_string(), "1920x1080");
    }

    #[test]
    fn test_resolution_presets() {
        assert_eq!(Resolution::HD, Resolution::new(1280, 720));
        assert_eq!(Resolution::FULL_HD, Resolution::new(1920, 1080));
    }

    #[test]
    fn test_graphics_settings_default() {
        let settings = GraphicsSettings::default();
        assert_eq!(settings.resolution, Resolution::FULL_HD);
        assert!(settings.bloom_enabled);
    }

    #[test]
    fn test_graphics_quality_preset() {
        let mut settings = GraphicsSettings::default();
        settings.apply_quality_preset(QualityLevel::Low);

        assert_eq!(settings.render_distance, 8);
        assert!(!settings.bloom_enabled);
        assert_eq!(settings.anti_aliasing, AntiAliasing::None);
    }

    #[test]
    fn test_audio_effective_volume() {
        let mut settings = AudioSettings::default();
        settings.master_volume = 0.5;
        settings.music_volume = 0.8;

        assert!((settings.effective_music_volume() - 0.4).abs() < 0.001);

        settings.audio_enabled = false;
        assert_eq!(settings.effective_music_volume(), 0.0);
    }

    #[test]
    fn test_control_settings_bindings() {
        let mut settings = ControlSettings::default();
        assert_eq!(settings.get_binding(InputAction::MoveForward), "W");

        settings.set_binding(InputAction::MoveForward, "Up".to_string());
        assert_eq!(settings.get_binding(InputAction::MoveForward), "Up");

        settings.reset_binding(InputAction::MoveForward);
        assert_eq!(settings.get_binding(InputAction::MoveForward), "W");
    }

    #[test]
    fn test_control_conflicts() {
        let mut settings = ControlSettings::default();
        // No conflicts by default
        assert!(settings.find_conflicts().is_empty());

        // Create a conflict
        settings.set_binding(InputAction::MoveBackward, "W".to_string());
        let conflicts = settings.find_conflicts();
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_difficulty_multipliers() {
        assert!(!GameDifficulty::Peaceful.enemies_spawn());
        assert!(GameDifficulty::Normal.enemies_spawn());
        assert!(GameDifficulty::Hardcore.is_permadeath());
        assert!(!GameDifficulty::Hard.is_permadeath());
    }

    #[test]
    fn test_gameplay_settings_default() {
        let settings = GameplaySettings::default();
        assert!(settings.show_tutorials);
        assert_eq!(settings.auto_save_interval, 5);
    }

    #[test]
    fn test_settings_validation() {
        let mut validation = SettingsValidation::new();
        assert!(validation.is_valid());

        validation.add_warning("Test warning");
        assert!(validation.is_valid());
        assert!(validation.has_warnings());

        validation.add_error("Test error");
        assert!(!validation.is_valid());
    }

    #[test]
    fn test_game_settings() {
        let settings = GameSettings::new();
        let validation = settings.validate();
        assert!(validation.is_valid());
    }

    #[test]
    fn test_game_settings_reset() {
        let mut settings = GameSettings::new();
        settings.audio.master_volume = 0.0;
        settings.reset_audio();
        assert!((settings.audio.master_volume - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_input_action_categories() {
        assert_eq!(InputAction::MoveForward.category(), "Movement");
        assert_eq!(InputAction::Attack.category(), "Combat");
        assert_eq!(InputAction::Inventory.category(), "Interface");
    }

    #[test]
    fn test_window_mode() {
        assert!(!WindowMode::Windowed.is_fullscreen());
        assert!(WindowMode::Borderless.is_fullscreen());
        assert!(WindowMode::Fullscreen.is_fullscreen());
    }
}
