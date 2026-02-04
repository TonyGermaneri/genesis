//! Options Menu UI
//!
//! Tabbed options interface for Graphics, Audio, Controls, and Gameplay settings
//! with Apply/Cancel/Reset functionality.

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};

/// Options menu tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub enum OptionsTab {
    /// Graphics settings
    #[default]
    Graphics,
    /// Audio settings
    Audio,
    /// Control settings
    Controls,
    /// Gameplay settings
    Gameplay,
}

impl OptionsTab {
    /// Get all tabs in order
    pub fn all() -> &'static [Self] {
        &[Self::Graphics, Self::Audio, Self::Controls, Self::Gameplay]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Graphics => "Graphics",
            Self::Audio => "Audio",
            Self::Controls => "Controls",
            Self::Gameplay => "Gameplay",
        }
    }

    /// Get tab icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Graphics => "🖥",
            Self::Audio => "🔊",
            Self::Controls => "🎮",
            Self::Gameplay => "⚙",
        }
    }
}

/// Display resolution option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Common resolution values
static COMMON_RESOLUTIONS: [Resolution; 6] = [
    Resolution {
        width: 1280,
        height: 720,
    },
    Resolution {
        width: 1366,
        height: 768,
    },
    Resolution {
        width: 1600,
        height: 900,
    },
    Resolution {
        width: 1920,
        height: 1080,
    },
    Resolution {
        width: 2560,
        height: 1440,
    },
    Resolution {
        width: 3840,
        height: 2160,
    },
];

impl Resolution {
    /// Create a new resolution
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get common resolutions
    pub fn common() -> &'static [Self] {
        &COMMON_RESOLUTIONS
    }

    /// Get display string
    pub fn display_string(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

/// Shadow quality setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShadowQuality {
    /// No shadows
    Off,
    /// Low quality shadows
    Low,
    /// Medium quality shadows
    #[default]
    Medium,
    /// High quality shadows
    High,
    /// Ultra quality shadows
    Ultra,
}

impl ShadowQuality {
    /// Get all shadow quality options
    pub fn all() -> &'static [Self] {
        &[Self::Off, Self::Low, Self::Medium, Self::High, Self::Ultra]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }
}

/// Graphics settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Display resolution
    pub resolution: Resolution,
    /// Fullscreen mode
    pub fullscreen: bool,
    /// Borderless window mode
    pub borderless: bool,
    /// VSync enabled
    pub vsync: bool,
    /// FPS limit (0 = unlimited)
    pub fps_limit: u32,
    /// Render distance (chunks)
    pub render_distance: u32,
    /// Shadow quality
    pub shadow_quality: ShadowQuality,
    /// Anti-aliasing level (0 = off, 2, 4, 8)
    pub anti_aliasing: u32,
    /// Texture quality (0-100)
    pub texture_quality: u32,
    /// Enable bloom effects
    pub bloom_enabled: bool,
    /// Ambient occlusion enabled
    pub ambient_occlusion: bool,
    /// Motion blur enabled
    pub motion_blur: bool,
    /// Gamma correction (0.5 - 2.5)
    pub gamma: f32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: Resolution::default(),
            fullscreen: false,
            borderless: true,
            vsync: true,
            fps_limit: 0,
            render_distance: 12,
            shadow_quality: ShadowQuality::Medium,
            anti_aliasing: 4,
            texture_quality: 100,
            bloom_enabled: true,
            ambient_occlusion: true,
            motion_blur: false,
            gamma: 1.0,
        }
    }
}

impl GraphicsSettings {
    /// Clamp all values to valid ranges
    pub fn clamp(&mut self) {
        self.render_distance = self.render_distance.clamp(4, 32);
        self.anti_aliasing = match self.anti_aliasing {
            0 => 0,
            1..=2 => 2,
            3..=4 => 4,
            _ => 8,
        };
        self.texture_quality = self.texture_quality.min(100);
        self.gamma = self.gamma.clamp(0.5, 2.5);
    }
}

/// Audio settings for options menu
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionsAudioSettings {
    /// Master volume (0-100)
    pub master_volume: u32,
    /// Music volume (0-100)
    pub music_volume: u32,
    /// Sound effects volume (0-100)
    pub sfx_volume: u32,
    /// Ambient sounds volume (0-100)
    pub ambient_volume: u32,
    /// Voice volume (0-100)
    pub voice_volume: u32,
    /// UI sounds volume (0-100)
    pub ui_volume: u32,
    /// Master mute
    pub muted: bool,
    /// Enable spatial audio
    pub spatial_audio: bool,
    /// Dynamic range compression
    pub dynamic_range: bool,
}

impl Default for OptionsAudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 80,
            music_volume: 70,
            sfx_volume: 100,
            ambient_volume: 60,
            voice_volume: 100,
            ui_volume: 80,
            muted: false,
            spatial_audio: true,
            dynamic_range: false,
        }
    }
}

impl OptionsAudioSettings {
    /// Clamp all values to valid ranges
    pub fn clamp(&mut self) {
        self.master_volume = self.master_volume.min(100);
        self.music_volume = self.music_volume.min(100);
        self.sfx_volume = self.sfx_volume.min(100);
        self.ambient_volume = self.ambient_volume.min(100);
        self.voice_volume = self.voice_volume.min(100);
        self.ui_volume = self.ui_volume.min(100);
    }

    /// Get effective volume for a category
    pub fn effective_volume(&self, category_volume: u32) -> f32 {
        if self.muted {
            0.0
        } else {
            (self.master_volume as f32 / 100.0) * (category_volume as f32 / 100.0)
        }
    }
}

/// Key binding action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAction {
    /// Move forward
    MoveForward,
    /// Move backward
    MoveBackward,
    /// Move left
    MoveLeft,
    /// Move right
    MoveRight,
    /// Jump
    Jump,
    /// Crouch
    Crouch,
    /// Sprint
    Sprint,
    /// Primary action/attack
    PrimaryAction,
    /// Secondary action
    SecondaryAction,
    /// Interact
    Interact,
    /// Open inventory
    Inventory,
    /// Open map
    Map,
    /// Open quest log
    QuestLog,
    /// Pause menu
    Pause,
}

impl KeyAction {
    /// Get all key actions
    pub fn all() -> &'static [Self] {
        &[
            Self::MoveForward,
            Self::MoveBackward,
            Self::MoveLeft,
            Self::MoveRight,
            Self::Jump,
            Self::Crouch,
            Self::Sprint,
            Self::PrimaryAction,
            Self::SecondaryAction,
            Self::Interact,
            Self::Inventory,
            Self::Map,
            Self::QuestLog,
            Self::Pause,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MoveForward => "Move Forward",
            Self::MoveBackward => "Move Backward",
            Self::MoveLeft => "Move Left",
            Self::MoveRight => "Move Right",
            Self::Jump => "Jump",
            Self::Crouch => "Crouch",
            Self::Sprint => "Sprint",
            Self::PrimaryAction => "Primary Action",
            Self::SecondaryAction => "Secondary Action",
            Self::Interact => "Interact",
            Self::Inventory => "Inventory",
            Self::Map => "Map",
            Self::QuestLog => "Quest Log",
            Self::Pause => "Pause",
        }
    }

    /// Get default key binding
    pub fn default_key(&self) -> &'static str {
        match self {
            Self::MoveForward => "W",
            Self::MoveBackward => "S",
            Self::MoveLeft => "A",
            Self::MoveRight => "D",
            Self::Jump => "Space",
            Self::Crouch => "Ctrl",
            Self::Sprint => "Shift",
            Self::PrimaryAction => "Mouse1",
            Self::SecondaryAction => "Mouse2",
            Self::Interact => "E",
            Self::Inventory => "I",
            Self::Map => "M",
            Self::QuestLog => "J",
            Self::Pause => "Escape",
        }
    }
}

/// Key binding entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    /// The action
    pub action: KeyAction,
    /// Primary key
    pub primary: String,
    /// Secondary/alternate key
    pub secondary: Option<String>,
}

impl KeyBinding {
    /// Create new key binding
    pub fn new(action: KeyAction) -> Self {
        Self {
            action,
            primary: action.default_key().to_string(),
            secondary: None,
        }
    }

    /// Set primary key
    pub fn with_primary(mut self, key: impl Into<String>) -> Self {
        self.primary = key.into();
        self
    }

    /// Set secondary key
    pub fn with_secondary(mut self, key: impl Into<String>) -> Self {
        self.secondary = Some(key.into());
        self
    }
}

/// Control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSettings {
    /// Mouse sensitivity (0.1 - 5.0)
    pub mouse_sensitivity: f32,
    /// Invert Y axis
    pub invert_y: bool,
    /// Invert X axis
    pub invert_x: bool,
    /// Toggle sprint vs hold
    pub toggle_sprint: bool,
    /// Toggle crouch vs hold
    pub toggle_crouch: bool,
    /// Key bindings
    pub key_bindings: Vec<KeyBinding>,
    /// Gamepad sensitivity
    pub gamepad_sensitivity: f32,
    /// Gamepad deadzone
    pub gamepad_deadzone: f32,
}

impl Default for ControlSettings {
    fn default() -> Self {
        let key_bindings = KeyAction::all()
            .iter()
            .map(|a| KeyBinding::new(*a))
            .collect();

        Self {
            mouse_sensitivity: 1.0,
            invert_y: false,
            invert_x: false,
            toggle_sprint: false,
            toggle_crouch: true,
            key_bindings,
            gamepad_sensitivity: 1.0,
            gamepad_deadzone: 0.15,
        }
    }
}

impl ControlSettings {
    /// Clamp values to valid ranges
    pub fn clamp(&mut self) {
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(0.1, 5.0);
        self.gamepad_sensitivity = self.gamepad_sensitivity.clamp(0.1, 5.0);
        self.gamepad_deadzone = self.gamepad_deadzone.clamp(0.0, 0.5);
    }

    /// Get binding for action
    pub fn get_binding(&self, action: KeyAction) -> Option<&KeyBinding> {
        self.key_bindings.iter().find(|b| b.action == action)
    }

    /// Set binding for action
    pub fn set_binding(&mut self, action: KeyAction, primary: impl Into<String>) {
        if let Some(binding) = self.key_bindings.iter_mut().find(|b| b.action == action) {
            binding.primary = primary.into();
        }
    }
}

/// Difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DifficultyLevel {
    /// Easy mode
    Easy,
    /// Normal mode
    #[default]
    Normal,
    /// Hard mode
    Hard,
    /// Hardcore mode
    Hardcore,
}

impl DifficultyLevel {
    /// Get all difficulty levels
    pub fn all() -> &'static [Self] {
        &[Self::Easy, Self::Normal, Self::Hard, Self::Hardcore]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Hardcore => "Hardcore",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Easy => "Relaxed experience with reduced enemy damage and easier survival",
            Self::Normal => "Balanced experience as intended by the developers",
            Self::Hard => "Challenging experience with increased enemy damage and tougher survival",
            Self::Hardcore => "Permanent death - when you die, the save is deleted",
        }
    }
}

/// Gameplay settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Difficulty level
    pub difficulty: DifficultyLevel,
    /// Auto-save enabled
    pub auto_save: bool,
    /// Auto-save interval in minutes
    pub auto_save_interval: u32,
    /// Show tutorials
    pub show_tutorials: bool,
    /// Show damage numbers
    pub show_damage_numbers: bool,
    /// Show enemy health bars
    pub show_enemy_health: bool,
    /// Show minimap
    pub show_minimap: bool,
    /// Show quest markers
    pub show_quest_markers: bool,
    /// Camera shake intensity (0-100)
    pub camera_shake: u32,
    /// Language code
    pub language: String,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            difficulty: DifficultyLevel::Normal,
            auto_save: true,
            auto_save_interval: 5,
            show_tutorials: true,
            show_damage_numbers: true,
            show_enemy_health: true,
            show_minimap: true,
            show_quest_markers: true,
            camera_shake: 100,
            language: String::from("en"),
        }
    }
}

impl GameplaySettings {
    /// Clamp values to valid ranges
    pub fn clamp(&mut self) {
        self.auto_save_interval = self.auto_save_interval.clamp(1, 30);
        self.camera_shake = self.camera_shake.min(100);
    }
}

/// Combined options settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptionsSettings {
    /// Graphics settings
    pub graphics: GraphicsSettings,
    /// Audio settings
    pub audio: OptionsAudioSettings,
    /// Control settings
    pub controls: ControlSettings,
    /// Gameplay settings
    pub gameplay: GameplaySettings,
}

impl OptionsSettings {
    /// Clamp all values to valid ranges
    pub fn clamp(&mut self) {
        self.graphics.clamp();
        self.audio.clamp();
        self.controls.clamp();
        self.gameplay.clamp();
    }
}

/// Actions generated by options menu
#[derive(Debug, Clone, PartialEq)]
pub enum OptionsMenuAction {
    /// Apply current settings
    Apply,
    /// Cancel and revert changes
    Cancel,
    /// Reset to defaults
    ResetToDefaults,
    /// Reset current tab to defaults
    ResetTab(OptionsTab),
    /// Close the menu
    Close,
    /// Switch to a tab
    SwitchTab(OptionsTab),
    /// Settings were changed
    SettingsChanged,
    /// Start key rebinding
    StartRebind(KeyAction),
}

/// Configuration for options menu appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsMenuConfig {
    /// Title text
    pub title: String,
    /// Panel width
    pub panel_width: f32,
    /// Panel height
    pub panel_height: f32,
    /// Background color
    pub background_color: [u8; 4],
    /// Tab bar color
    pub tab_bar_color: [u8; 4],
    /// Active tab color
    pub active_tab_color: [u8; 4],
    /// Show icons in tabs
    pub show_tab_icons: bool,
}

impl Default for OptionsMenuConfig {
    fn default() -> Self {
        Self {
            title: String::from("Options"),
            panel_width: 600.0,
            panel_height: 500.0,
            background_color: [30, 30, 40, 245],
            tab_bar_color: [40, 40, 55, 255],
            active_tab_color: [60, 60, 80, 255],
            show_tab_icons: true,
        }
    }
}

/// Options menu state
#[derive(Debug, Clone)]
pub struct OptionsMenu {
    /// Configuration
    config: OptionsMenuConfig,
    /// Whether menu is visible
    visible: bool,
    /// Current active tab
    active_tab: OptionsTab,
    /// Current settings (being edited)
    current_settings: OptionsSettings,
    /// Original settings (before changes)
    original_settings: OptionsSettings,
    /// Whether settings have been modified
    has_changes: bool,
    /// Pending actions
    actions: Vec<OptionsMenuAction>,
    /// Currently rebinding key action
    rebinding_action: Option<KeyAction>,
}

impl OptionsMenu {
    /// Create a new options menu
    pub fn new(config: OptionsMenuConfig, settings: OptionsSettings) -> Self {
        Self {
            config,
            visible: false,
            active_tab: OptionsTab::Graphics,
            current_settings: settings.clone(),
            original_settings: settings,
            has_changes: false,
            actions: Vec::new(),
            rebinding_action: None,
        }
    }

    /// Create with default config and settings
    pub fn with_defaults() -> Self {
        Self::new(OptionsMenuConfig::default(), OptionsSettings::default())
    }

    /// Get configuration
    pub fn config(&self) -> &OptionsMenuConfig {
        &self.config
    }

    /// Get current settings
    pub fn settings(&self) -> &OptionsSettings {
        &self.current_settings
    }

    /// Get mutable current settings
    pub fn settings_mut(&mut self) -> &mut OptionsSettings {
        self.has_changes = true;
        &mut self.current_settings
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the menu
    pub fn show(&mut self) {
        self.visible = true;
        self.active_tab = OptionsTab::Graphics;
    }

    /// Hide the menu
    pub fn hide(&mut self) {
        self.visible = false;
        self.rebinding_action = None;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Open with specific settings
    pub fn open_with_settings(&mut self, settings: OptionsSettings) {
        self.current_settings = settings.clone();
        self.original_settings = settings;
        self.has_changes = false;
        self.show();
    }

    /// Get active tab
    pub fn active_tab(&self) -> OptionsTab {
        self.active_tab
    }

    /// Set active tab
    pub fn set_active_tab(&mut self, tab: OptionsTab) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.actions.push(OptionsMenuAction::SwitchTab(tab));
        }
    }

    /// Check if has unsaved changes
    pub fn has_changes(&self) -> bool {
        self.has_changes
    }

    /// Mark settings as changed
    pub fn mark_changed(&mut self) {
        self.has_changes = true;
        self.actions.push(OptionsMenuAction::SettingsChanged);
    }

    /// Apply current settings
    pub fn apply(&mut self) {
        self.current_settings.clamp();
        self.original_settings = self.current_settings.clone();
        self.has_changes = false;
        self.actions.push(OptionsMenuAction::Apply);
    }

    /// Cancel and revert changes
    pub fn cancel(&mut self) {
        self.current_settings = self.original_settings.clone();
        self.has_changes = false;
        self.hide();
        self.actions.push(OptionsMenuAction::Cancel);
    }

    /// Reset all settings to defaults
    pub fn reset_to_defaults(&mut self) {
        self.current_settings = OptionsSettings::default();
        self.has_changes = true;
        self.actions.push(OptionsMenuAction::ResetToDefaults);
    }

    /// Reset current tab to defaults
    pub fn reset_current_tab(&mut self) {
        match self.active_tab {
            OptionsTab::Graphics => {
                self.current_settings.graphics = GraphicsSettings::default();
            },
            OptionsTab::Audio => {
                self.current_settings.audio = OptionsAudioSettings::default();
            },
            OptionsTab::Controls => {
                self.current_settings.controls = ControlSettings::default();
            },
            OptionsTab::Gameplay => {
                self.current_settings.gameplay = GameplaySettings::default();
            },
        }
        self.has_changes = true;
        self.actions
            .push(OptionsMenuAction::ResetTab(self.active_tab));
    }

    /// Check if currently rebinding
    pub fn is_rebinding(&self) -> bool {
        self.rebinding_action.is_some()
    }

    /// Get currently rebinding action
    pub fn rebinding_action(&self) -> Option<KeyAction> {
        self.rebinding_action
    }

    /// Start rebinding a key
    pub fn start_rebind(&mut self, action: KeyAction) {
        self.rebinding_action = Some(action);
        self.actions.push(OptionsMenuAction::StartRebind(action));
    }

    /// Complete rebinding with a key
    pub fn complete_rebind(&mut self, key: impl Into<String>) {
        if let Some(action) = self.rebinding_action.take() {
            self.current_settings.controls.set_binding(action, key);
            self.has_changes = true;
        }
    }

    /// Cancel rebinding
    pub fn cancel_rebind(&mut self) {
        self.rebinding_action = None;
    }

    /// Drain pending actions
    pub fn drain_actions(&mut self) -> Vec<OptionsMenuAction> {
        std::mem::take(&mut self.actions)
    }

    /// Check if has pending actions
    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    /// Render the options menu
    pub fn render(&mut self, ui: &mut Ui) {
        if !self.visible {
            return;
        }

        let bg = Color32::from_rgba_unmultiplied(
            self.config.background_color[0],
            self.config.background_color[1],
            self.config.background_color[2],
            self.config.background_color[3],
        );

        egui::Area::new(egui::Id::new("options_menu"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::none()
                    .fill(bg)
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)))
                    .rounding(8.0)
                    .inner_margin(0.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(
                            self.config.panel_width,
                            self.config.panel_height,
                        ));

                        self.render_header(ui);
                        self.render_tabs(ui);
                        self.render_content(ui);
                        self.render_footer(ui);
                    });
            });
    }

    fn render_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(&self.config.title)
                    .size(24.0)
                    .color(Color32::from_gray(220))
                    .strong(),
            );
        });
        ui.add_space(8.0);
    }

    fn render_tabs(&mut self, ui: &mut Ui) {
        let tab_bar_color = Color32::from_rgba_unmultiplied(
            self.config.tab_bar_color[0],
            self.config.tab_bar_color[1],
            self.config.tab_bar_color[2],
            self.config.tab_bar_color[3],
        );

        let active_color = Color32::from_rgba_unmultiplied(
            self.config.active_tab_color[0],
            self.config.active_tab_color[1],
            self.config.active_tab_color[2],
            self.config.active_tab_color[3],
        );

        egui::Frame::none()
            .fill(tab_bar_color)
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in OptionsTab::all() {
                        let is_active = self.active_tab == *tab;
                        let bg = if is_active {
                            active_color
                        } else {
                            Color32::TRANSPARENT
                        };
                        let text_color = if is_active {
                            Color32::from_gray(255)
                        } else {
                            Color32::from_gray(160)
                        };

                        let mut text = String::new();
                        if self.config.show_tab_icons {
                            text.push_str(tab.icon());
                            text.push(' ');
                        }
                        text.push_str(tab.display_name());

                        let response = ui.add(
                            egui::Button::new(egui::RichText::new(text).color(text_color))
                                .fill(bg)
                                .rounding(4.0),
                        );

                        if response.clicked() {
                            self.set_active_tab(*tab);
                        }
                    }
                });
            });
    }

    fn render_content(&mut self, ui: &mut Ui) {
        egui::Frame::none().inner_margin(16.0).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(self.config.panel_height - 150.0)
                .show(ui, |ui| match self.active_tab {
                    OptionsTab::Graphics => self.render_graphics_tab(ui),
                    OptionsTab::Audio => self.render_audio_tab(ui),
                    OptionsTab::Controls => self.render_controls_tab(ui),
                    OptionsTab::Gameplay => self.render_gameplay_tab(ui),
                });
        });
    }

    fn render_graphics_tab(&mut self, ui: &mut Ui) {
        let graphics = &mut self.current_settings.graphics;

        ui.horizontal(|ui| {
            ui.label("Resolution:");
            egui::ComboBox::from_id_salt("resolution")
                .selected_text(graphics.resolution.display_string())
                .show_ui(ui, |ui| {
                    for res in Resolution::common() {
                        if ui
                            .selectable_value(&mut graphics.resolution, *res, res.display_string())
                            .changed()
                        {
                            self.has_changes = true;
                        }
                    }
                });
        });

        if ui
            .checkbox(&mut graphics.fullscreen, "Fullscreen")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut graphics.borderless, "Borderless")
            .changed()
        {
            self.has_changes = true;
        }

        if ui.checkbox(&mut graphics.vsync, "VSync").changed() {
            self.has_changes = true;
        }

        ui.horizontal(|ui| {
            ui.label("Render Distance:");
            if ui
                .add(egui::Slider::new(&mut graphics.render_distance, 4..=32))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Shadow Quality:");
            egui::ComboBox::from_id_salt("shadows")
                .selected_text(graphics.shadow_quality.display_name())
                .show_ui(ui, |ui| {
                    for quality in ShadowQuality::all() {
                        if ui
                            .selectable_value(
                                &mut graphics.shadow_quality,
                                *quality,
                                quality.display_name(),
                            )
                            .changed()
                        {
                            self.has_changes = true;
                        }
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Anti-Aliasing:");
            let aa_text = if graphics.anti_aliasing == 0 {
                "Off".to_string()
            } else {
                format!("{}x", graphics.anti_aliasing)
            };
            egui::ComboBox::from_id_salt("aa")
                .selected_text(aa_text)
                .show_ui(ui, |ui| {
                    for aa in [0, 2, 4, 8] {
                        let text = if aa == 0 {
                            "Off".to_string()
                        } else {
                            format!("{aa}x")
                        };
                        if ui
                            .selectable_value(&mut graphics.anti_aliasing, aa, text)
                            .changed()
                        {
                            self.has_changes = true;
                        }
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Gamma:");
            if ui
                .add(egui::Slider::new(&mut graphics.gamma, 0.5..=2.5))
                .changed()
            {
                self.has_changes = true;
            }
        });

        if ui.checkbox(&mut graphics.bloom_enabled, "Bloom").changed() {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut graphics.ambient_occlusion, "Ambient Occlusion")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut graphics.motion_blur, "Motion Blur")
            .changed()
        {
            self.has_changes = true;
        }
    }

    fn render_audio_tab(&mut self, ui: &mut Ui) {
        let audio = &mut self.current_settings.audio;

        if ui.checkbox(&mut audio.muted, "Mute All").changed() {
            self.has_changes = true;
        }

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Master Volume:");
            if ui
                .add(egui::Slider::new(&mut audio.master_volume, 0..=100))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Music Volume:");
            if ui
                .add(egui::Slider::new(&mut audio.music_volume, 0..=100))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("SFX Volume:");
            if ui
                .add(egui::Slider::new(&mut audio.sfx_volume, 0..=100))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Ambient Volume:");
            if ui
                .add(egui::Slider::new(&mut audio.ambient_volume, 0..=100))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Voice Volume:");
            if ui
                .add(egui::Slider::new(&mut audio.voice_volume, 0..=100))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("UI Volume:");
            if ui
                .add(egui::Slider::new(&mut audio.ui_volume, 0..=100))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.add_space(8.0);

        if ui
            .checkbox(&mut audio.spatial_audio, "Spatial Audio")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut audio.dynamic_range, "Dynamic Range Compression")
            .changed()
        {
            self.has_changes = true;
        }
    }

    fn render_controls_tab(&mut self, ui: &mut Ui) {
        let controls = &mut self.current_settings.controls;

        ui.horizontal(|ui| {
            ui.label("Mouse Sensitivity:");
            if ui
                .add(egui::Slider::new(
                    &mut controls.mouse_sensitivity,
                    0.1..=5.0,
                ))
                .changed()
            {
                self.has_changes = true;
            }
        });

        if ui
            .checkbox(&mut controls.invert_y, "Invert Y Axis")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut controls.invert_x, "Invert X Axis")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut controls.toggle_sprint, "Toggle Sprint")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut controls.toggle_crouch, "Toggle Crouch")
            .changed()
        {
            self.has_changes = true;
        }

        ui.add_space(16.0);
        ui.label(egui::RichText::new("Key Bindings").strong());
        ui.separator();

        // Clone to avoid borrow issues
        let bindings = controls.key_bindings.clone();
        for binding in bindings {
            ui.horizontal(|ui| {
                ui.label(binding.action.display_name());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.rebinding_action == Some(binding.action) {
                        ui.label(
                            egui::RichText::new("Press a key...")
                                .color(Color32::from_rgb(200, 200, 100)),
                        );
                    } else if ui.button(&binding.primary).clicked() {
                        self.rebinding_action = Some(binding.action);
                    }
                });
            });
        }

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            ui.label("Gamepad Sensitivity:");
            if ui
                .add(egui::Slider::new(
                    &mut controls.gamepad_sensitivity,
                    0.1..=5.0,
                ))
                .changed()
            {
                self.has_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Gamepad Deadzone:");
            if ui
                .add(egui::Slider::new(&mut controls.gamepad_deadzone, 0.0..=0.5))
                .changed()
            {
                self.has_changes = true;
            }
        });
    }

    fn render_gameplay_tab(&mut self, ui: &mut Ui) {
        let gameplay = &mut self.current_settings.gameplay;

        ui.horizontal(|ui| {
            ui.label("Difficulty:");
            egui::ComboBox::from_id_salt("difficulty")
                .selected_text(gameplay.difficulty.display_name())
                .show_ui(ui, |ui| {
                    for diff in DifficultyLevel::all() {
                        if ui
                            .selectable_value(&mut gameplay.difficulty, *diff, diff.display_name())
                            .on_hover_text(diff.description())
                            .changed()
                        {
                            self.has_changes = true;
                        }
                    }
                });
        });

        ui.add_space(8.0);

        if ui.checkbox(&mut gameplay.auto_save, "Auto-Save").changed() {
            self.has_changes = true;
        }

        if gameplay.auto_save {
            ui.horizontal(|ui| {
                ui.label("Auto-Save Interval (minutes):");
                if ui
                    .add(egui::Slider::new(&mut gameplay.auto_save_interval, 1..=30))
                    .changed()
                {
                    self.has_changes = true;
                }
            });
        }

        ui.add_space(8.0);

        if ui
            .checkbox(&mut gameplay.show_tutorials, "Show Tutorials")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut gameplay.show_damage_numbers, "Show Damage Numbers")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut gameplay.show_enemy_health, "Show Enemy Health Bars")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut gameplay.show_minimap, "Show Minimap")
            .changed()
        {
            self.has_changes = true;
        }

        if ui
            .checkbox(&mut gameplay.show_quest_markers, "Show Quest Markers")
            .changed()
        {
            self.has_changes = true;
        }

        ui.horizontal(|ui| {
            ui.label("Camera Shake:");
            if ui
                .add(egui::Slider::new(&mut gameplay.camera_shake, 0..=100).suffix("%"))
                .changed()
            {
                self.has_changes = true;
            }
        });
    }

    fn render_footer(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_space(16.0);

            if ui.button("Reset Tab").clicked() {
                self.reset_current_tab();
            }

            if ui.button("Reset All").clicked() {
                self.reset_to_defaults();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(16.0);

                let apply_enabled = self.has_changes;
                if ui
                    .add_enabled(apply_enabled, egui::Button::new("Apply"))
                    .clicked()
                {
                    self.apply();
                }

                if ui.button("Cancel").clicked() {
                    self.cancel();
                }
            });
        });
        ui.add_space(8.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_tab_all() {
        let tabs = OptionsTab::all();
        assert_eq!(tabs.len(), 4);
        assert_eq!(tabs[0], OptionsTab::Graphics);
    }

    #[test]
    fn test_options_tab_display_name() {
        assert_eq!(OptionsTab::Graphics.display_name(), "Graphics");
        assert_eq!(OptionsTab::Audio.display_name(), "Audio");
    }

    #[test]
    fn test_options_tab_icon() {
        assert_eq!(OptionsTab::Graphics.icon(), "🖥");
        assert_eq!(OptionsTab::Audio.icon(), "🔊");
    }

    #[test]
    fn test_resolution_common() {
        let resolutions = Resolution::common();
        assert!(!resolutions.is_empty());
        assert!(resolutions
            .iter()
            .any(|r| r.width == 1920 && r.height == 1080));
    }

    #[test]
    fn test_resolution_display_string() {
        let res = Resolution::new(1920, 1080);
        assert_eq!(res.display_string(), "1920x1080");
    }

    #[test]
    fn test_resolution_aspect_ratio() {
        let res = Resolution::new(1920, 1080);
        let ratio = res.aspect_ratio();
        assert!((ratio - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_shadow_quality_all() {
        let qualities = ShadowQuality::all();
        assert_eq!(qualities.len(), 5);
    }

    #[test]
    fn test_shadow_quality_display_name() {
        assert_eq!(ShadowQuality::Off.display_name(), "Off");
        assert_eq!(ShadowQuality::Ultra.display_name(), "Ultra");
    }

    #[test]
    fn test_graphics_settings_defaults() {
        let settings = GraphicsSettings::default();
        assert!(settings.vsync);
        assert_eq!(settings.render_distance, 12);
    }

    #[test]
    fn test_graphics_settings_clamp() {
        let mut settings = GraphicsSettings::default();
        settings.render_distance = 100;
        settings.gamma = 5.0;
        settings.clamp();
        assert_eq!(settings.render_distance, 32);
        assert!((settings.gamma - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_audio_settings_defaults() {
        let settings = OptionsAudioSettings::default();
        assert_eq!(settings.master_volume, 80);
        assert!(!settings.muted);
    }

    #[test]
    fn test_audio_settings_effective_volume() {
        let settings = OptionsAudioSettings::default();
        let effective = settings.effective_volume(100);
        assert!((effective - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_audio_settings_effective_volume_muted() {
        let mut settings = OptionsAudioSettings::default();
        settings.muted = true;
        let effective = settings.effective_volume(100);
        assert!((effective).abs() < f32::EPSILON);
    }

    #[test]
    fn test_key_action_all() {
        let actions = KeyAction::all();
        assert!(!actions.is_empty());
        assert!(actions.contains(&KeyAction::MoveForward));
    }

    #[test]
    fn test_key_action_display_name() {
        assert_eq!(KeyAction::MoveForward.display_name(), "Move Forward");
        assert_eq!(KeyAction::Jump.display_name(), "Jump");
    }

    #[test]
    fn test_key_action_default_key() {
        assert_eq!(KeyAction::MoveForward.default_key(), "W");
        assert_eq!(KeyAction::Jump.default_key(), "Space");
    }

    #[test]
    fn test_key_binding_new() {
        let binding = KeyBinding::new(KeyAction::Jump);
        assert_eq!(binding.action, KeyAction::Jump);
        assert_eq!(binding.primary, "Space");
        assert!(binding.secondary.is_none());
    }

    #[test]
    fn test_key_binding_with_secondary() {
        let binding = KeyBinding::new(KeyAction::Jump)
            .with_primary("W")
            .with_secondary("Up");
        assert_eq!(binding.primary, "W");
        assert_eq!(binding.secondary, Some(String::from("Up")));
    }

    #[test]
    fn test_control_settings_defaults() {
        let settings = ControlSettings::default();
        assert!((settings.mouse_sensitivity - 1.0).abs() < f32::EPSILON);
        assert!(!settings.invert_y);
    }

    #[test]
    fn test_control_settings_get_binding() {
        let settings = ControlSettings::default();
        let binding = settings.get_binding(KeyAction::Jump);
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().primary, "Space");
    }

    #[test]
    fn test_control_settings_set_binding() {
        let mut settings = ControlSettings::default();
        settings.set_binding(KeyAction::Jump, "F");
        let binding = settings.get_binding(KeyAction::Jump).unwrap();
        assert_eq!(binding.primary, "F");
    }

    #[test]
    fn test_difficulty_level_all() {
        let levels = DifficultyLevel::all();
        assert_eq!(levels.len(), 4);
    }

    #[test]
    fn test_difficulty_level_display_name() {
        assert_eq!(DifficultyLevel::Easy.display_name(), "Easy");
        assert_eq!(DifficultyLevel::Hardcore.display_name(), "Hardcore");
    }

    #[test]
    fn test_difficulty_level_description() {
        assert!(!DifficultyLevel::Normal.description().is_empty());
    }

    #[test]
    fn test_gameplay_settings_defaults() {
        let settings = GameplaySettings::default();
        assert!(settings.auto_save);
        assert_eq!(settings.auto_save_interval, 5);
    }

    #[test]
    fn test_options_settings_clamp() {
        let mut settings = OptionsSettings::default();
        settings.graphics.render_distance = 100;
        settings.clamp();
        assert_eq!(settings.graphics.render_distance, 32);
    }

    #[test]
    fn test_options_menu_config_defaults() {
        let config = OptionsMenuConfig::default();
        assert_eq!(config.title, "Options");
        assert!(config.panel_width > 0.0);
    }

    #[test]
    fn test_options_menu_new() {
        let menu = OptionsMenu::with_defaults();
        assert!(!menu.is_visible());
        assert_eq!(menu.active_tab(), OptionsTab::Graphics);
    }

    #[test]
    fn test_options_menu_visibility() {
        let mut menu = OptionsMenu::with_defaults();

        menu.show();
        assert!(menu.is_visible());

        menu.hide();
        assert!(!menu.is_visible());

        menu.toggle();
        assert!(menu.is_visible());
    }

    #[test]
    fn test_options_menu_set_tab() {
        let mut menu = OptionsMenu::with_defaults();
        menu.set_active_tab(OptionsTab::Audio);
        assert_eq!(menu.active_tab(), OptionsTab::Audio);

        let actions = menu.drain_actions();
        assert!(actions
            .iter()
            .any(|a| *a == OptionsMenuAction::SwitchTab(OptionsTab::Audio)));
    }

    #[test]
    fn test_options_menu_has_changes() {
        let mut menu = OptionsMenu::with_defaults();
        assert!(!menu.has_changes());

        menu.settings_mut().graphics.vsync = false;
        assert!(menu.has_changes());
    }

    #[test]
    fn test_options_menu_apply() {
        let mut menu = OptionsMenu::with_defaults();
        menu.settings_mut().audio.master_volume = 50;
        assert!(menu.has_changes());

        menu.apply();
        assert!(!menu.has_changes());

        let actions = menu.drain_actions();
        assert!(actions.iter().any(|a| *a == OptionsMenuAction::Apply));
    }

    #[test]
    fn test_options_menu_cancel() {
        let mut menu = OptionsMenu::with_defaults();
        let original_volume = menu.settings().audio.master_volume;

        menu.settings_mut().audio.master_volume = 50;
        menu.cancel();

        assert_eq!(menu.settings().audio.master_volume, original_volume);
        assert!(!menu.is_visible());
    }

    #[test]
    fn test_options_menu_reset_to_defaults() {
        let mut menu = OptionsMenu::with_defaults();
        menu.settings_mut().graphics.render_distance = 4;

        menu.reset_to_defaults();

        assert_eq!(menu.settings().graphics.render_distance, 12);
        assert!(menu.has_changes());
    }

    #[test]
    fn test_options_menu_reset_current_tab() {
        let mut menu = OptionsMenu::with_defaults();
        menu.set_active_tab(OptionsTab::Graphics);
        menu.settings_mut().graphics.render_distance = 4;

        menu.reset_current_tab();

        assert_eq!(menu.settings().graphics.render_distance, 12);
    }

    #[test]
    fn test_options_menu_rebinding() {
        let mut menu = OptionsMenu::with_defaults();
        assert!(!menu.is_rebinding());

        menu.start_rebind(KeyAction::Jump);
        assert!(menu.is_rebinding());
        assert_eq!(menu.rebinding_action(), Some(KeyAction::Jump));

        menu.complete_rebind("X");
        assert!(!menu.is_rebinding());

        let binding = menu
            .settings()
            .controls
            .get_binding(KeyAction::Jump)
            .unwrap();
        assert_eq!(binding.primary, "X");
    }

    #[test]
    fn test_options_menu_cancel_rebind() {
        let mut menu = OptionsMenu::with_defaults();
        menu.start_rebind(KeyAction::Jump);
        menu.cancel_rebind();
        assert!(!menu.is_rebinding());
    }

    #[test]
    fn test_options_menu_action_equality() {
        assert_eq!(OptionsMenuAction::Apply, OptionsMenuAction::Apply);
        assert_ne!(OptionsMenuAction::Apply, OptionsMenuAction::Cancel);
    }

    #[test]
    fn test_options_settings_serialization() {
        let settings = OptionsSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: OptionsSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.graphics.render_distance,
            settings.graphics.render_distance
        );
    }

    #[test]
    fn test_options_menu_config_serialization() {
        let config = OptionsMenuConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: OptionsMenuConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, config.title);
    }
}
