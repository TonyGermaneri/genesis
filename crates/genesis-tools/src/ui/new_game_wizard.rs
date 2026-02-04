//! New Game Wizard UI
//!
//! Multi-step wizard for creating a new game with world name input,
//! seed selection, world size, difficulty, and confirmation.

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};

/// Wizard steps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WizardStep {
    /// Step 1: World name and character name
    #[default]
    NameEntry,
    /// Step 2: World settings (seed, size, difficulty)
    WorldSettings,
    /// Step 3: Confirmation summary
    Confirmation,
}

impl WizardStep {
    /// Get all steps in order
    pub fn all() -> &'static [Self] {
        &[Self::NameEntry, Self::WorldSettings, Self::Confirmation]
    }

    /// Get step number (1-based)
    pub fn number(&self) -> u32 {
        match self {
            Self::NameEntry => 1,
            Self::WorldSettings => 2,
            Self::Confirmation => 3,
        }
    }

    /// Get step title
    pub fn title(&self) -> &'static str {
        match self {
            Self::NameEntry => "Name Your World",
            Self::WorldSettings => "World Settings",
            Self::Confirmation => "Confirm Settings",
        }
    }

    /// Get step description
    pub fn description(&self) -> &'static str {
        match self {
            Self::NameEntry => "Choose a name for your world and character",
            Self::WorldSettings => "Configure world generation settings",
            Self::Confirmation => "Review and confirm your choices",
        }
    }

    /// Get next step
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::NameEntry => Some(Self::WorldSettings),
            Self::WorldSettings => Some(Self::Confirmation),
            Self::Confirmation => None,
        }
    }

    /// Get previous step
    pub fn previous(&self) -> Option<Self> {
        match self {
            Self::NameEntry => None,
            Self::WorldSettings => Some(Self::NameEntry),
            Self::Confirmation => Some(Self::WorldSettings),
        }
    }

    /// Check if this is the first step
    pub fn is_first(&self) -> bool {
        matches!(self, Self::NameEntry)
    }

    /// Check if this is the last step
    pub fn is_last(&self) -> bool {
        matches!(self, Self::Confirmation)
    }
}

/// World size option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorldSize {
    /// Small world
    Small,
    /// Medium world
    #[default]
    Medium,
    /// Large world
    Large,
    /// Huge world
    Huge,
}

impl WorldSize {
    /// Get all world sizes
    pub fn all() -> &'static [Self] {
        &[Self::Small, Self::Medium, Self::Large, Self::Huge]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Huge => "Huge",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Small => "Quick exploration, ~1 hour",
            Self::Medium => "Balanced experience, ~5 hours",
            Self::Large => "Extended adventure, ~15 hours",
            Self::Huge => "Epic journey, ~40+ hours",
        }
    }

    /// Get estimated chunk count
    pub fn chunk_count(&self) -> u32 {
        match self {
            Self::Small => 64,
            Self::Medium => 256,
            Self::Large => 1024,
            Self::Huge => 4096,
        }
    }
}

/// Difficulty setting for new game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NewGameDifficulty {
    /// Peaceful mode - no enemies
    Peaceful,
    /// Easy mode
    Easy,
    /// Normal mode
    #[default]
    Normal,
    /// Hard mode
    Hard,
    /// Hardcore mode (permadeath)
    Hardcore,
}

impl NewGameDifficulty {
    /// Get all difficulties
    pub fn all() -> &'static [Self] {
        &[
            Self::Peaceful,
            Self::Easy,
            Self::Normal,
            Self::Hard,
            Self::Hardcore,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Peaceful => "Peaceful",
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Hardcore => "Hardcore",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Peaceful => "No hostile creatures, focus on exploration and building",
            Self::Easy => "Reduced enemy damage, more forgiving survival",
            Self::Normal => "Balanced challenge as intended",
            Self::Hard => "Increased difficulty, tougher enemies",
            Self::Hardcore => "One life only - death deletes the save",
        }
    }

    /// Get color for difficulty
    pub fn color(&self) -> Color32 {
        match self {
            Self::Peaceful => Color32::from_rgb(100, 180, 100),
            Self::Easy => Color32::from_rgb(100, 200, 100),
            Self::Normal => Color32::from_rgb(200, 200, 100),
            Self::Hard => Color32::from_rgb(200, 150, 100),
            Self::Hardcore => Color32::from_rgb(200, 80, 80),
        }
    }

    /// Check if this is a warning difficulty
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Hard | Self::Hardcore)
    }
}

/// Seed input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SeedMode {
    /// Random seed
    #[default]
    Random,
    /// Custom seed input
    Custom,
}

impl SeedMode {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Random => "Random",
            Self::Custom => "Custom",
        }
    }
}

/// New game configuration being created
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewGameConfig {
    /// World name
    pub world_name: String,
    /// Character name
    pub character_name: String,
    /// Seed mode
    pub seed_mode: SeedMode,
    /// Custom seed string (if seed_mode is Custom)
    pub custom_seed: String,
    /// Generated seed value
    pub seed_value: u64,
    /// World size
    pub world_size: WorldSize,
    /// Difficulty
    pub difficulty: NewGameDifficulty,
    /// Enable tutorial
    pub enable_tutorial: bool,
    /// Enable achievements
    pub enable_achievements: bool,
}

impl Default for NewGameConfig {
    fn default() -> Self {
        Self {
            world_name: String::new(),
            character_name: String::new(),
            seed_mode: SeedMode::Random,
            custom_seed: String::new(),
            seed_value: rand_seed(),
            world_size: WorldSize::Medium,
            difficulty: NewGameDifficulty::Normal,
            enable_tutorial: true,
            enable_achievements: true,
        }
    }
}

impl NewGameConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set world name
    pub fn with_world_name(mut self, name: impl Into<String>) -> Self {
        self.world_name = name.into();
        self
    }

    /// Set character name
    pub fn with_character_name(mut self, name: impl Into<String>) -> Self {
        self.character_name = name.into();
        self
    }

    /// Set custom seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed_value = seed;
        self.seed_mode = SeedMode::Custom;
        self
    }

    /// Set world size
    pub fn with_size(mut self, size: WorldSize) -> Self {
        self.world_size = size;
        self
    }

    /// Set difficulty
    pub fn with_difficulty(mut self, difficulty: NewGameDifficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Regenerate random seed
    pub fn regenerate_seed(&mut self) {
        self.seed_value = rand_seed();
    }

    /// Parse custom seed from string
    pub fn parse_custom_seed(&mut self) {
        if self.custom_seed.is_empty() {
            self.seed_value = rand_seed();
            return;
        }

        // Try parsing as number
        if let Ok(n) = self.custom_seed.parse::<u64>() {
            self.seed_value = n;
            return;
        }

        // Try parsing as hex
        if let Some(hex) = self.custom_seed.strip_prefix("0x") {
            if let Ok(n) = u64::from_str_radix(hex, 16) {
                self.seed_value = n;
                return;
            }
        }

        // Hash the string
        self.seed_value = hash_string(&self.custom_seed);
    }

    /// Get formatted seed for display
    pub fn formatted_seed(&self) -> String {
        format!("{:016X}", self.seed_value)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if self.world_name.trim().is_empty() {
            errors.push(ValidationError::WorldNameEmpty);
        } else if self.world_name.len() > 32 {
            errors.push(ValidationError::WorldNameTooLong);
        } else if !is_valid_name(&self.world_name) {
            errors.push(ValidationError::WorldNameInvalid);
        }

        if self.character_name.trim().is_empty() {
            errors.push(ValidationError::CharacterNameEmpty);
        } else if self.character_name.len() > 24 {
            errors.push(ValidationError::CharacterNameTooLong);
        } else if !is_valid_name(&self.character_name) {
            errors.push(ValidationError::CharacterNameInvalid);
        }

        errors
    }

    /// Check if configuration is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

/// Validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    /// World name is empty
    WorldNameEmpty,
    /// World name is too long
    WorldNameTooLong,
    /// World name contains invalid characters
    WorldNameInvalid,
    /// Character name is empty
    CharacterNameEmpty,
    /// Character name is too long
    CharacterNameTooLong,
    /// Character name contains invalid characters
    CharacterNameInvalid,
}

impl ValidationError {
    /// Get error message
    pub fn message(&self) -> &'static str {
        match self {
            Self::WorldNameEmpty => "World name is required",
            Self::WorldNameTooLong => "World name must be 32 characters or less",
            Self::WorldNameInvalid => "World name contains invalid characters",
            Self::CharacterNameEmpty => "Character name is required",
            Self::CharacterNameTooLong => "Character name must be 24 characters or less",
            Self::CharacterNameInvalid => "Character name contains invalid characters",
        }
    }
}

/// Actions generated by the wizard
#[derive(Debug, Clone, PartialEq)]
pub enum NewGameWizardAction {
    /// Go to next step
    Next,
    /// Go to previous step
    Back,
    /// Cancel wizard
    Cancel,
    /// Create the game
    Create(NewGameConfig),
    /// Regenerate seed
    RegenerateSeed,
    /// Close wizard
    Close,
}

/// Configuration for wizard appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGameWizardConfig {
    /// Title text
    pub title: String,
    /// Panel width
    pub panel_width: f32,
    /// Panel height
    pub panel_height: f32,
    /// Background color
    pub background_color: [u8; 4],
    /// Accent color
    pub accent_color: [u8; 4],
    /// Show step indicator
    pub show_step_indicator: bool,
}

impl Default for NewGameWizardConfig {
    fn default() -> Self {
        Self {
            title: String::from("New Game"),
            panel_width: 500.0,
            panel_height: 450.0,
            background_color: [30, 30, 40, 250],
            accent_color: [100, 150, 200, 255],
            show_step_indicator: true,
        }
    }
}

/// New game wizard state
#[derive(Debug, Clone)]
pub struct NewGameWizard {
    /// Configuration
    config: NewGameWizardConfig,
    /// Whether wizard is visible
    visible: bool,
    /// Current step
    current_step: WizardStep,
    /// Game configuration being built
    game_config: NewGameConfig,
    /// Pending actions
    actions: Vec<NewGameWizardAction>,
    /// Validation errors to display
    validation_errors: Vec<ValidationError>,
}

impl NewGameWizard {
    /// Create a new wizard
    pub fn new(config: NewGameWizardConfig) -> Self {
        Self {
            config,
            visible: false,
            current_step: WizardStep::NameEntry,
            game_config: NewGameConfig::new(),
            actions: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(NewGameWizardConfig::default())
    }

    /// Get configuration
    pub fn config(&self) -> &NewGameWizardConfig {
        &self.config
    }

    /// Get game configuration
    pub fn game_config(&self) -> &NewGameConfig {
        &self.game_config
    }

    /// Get mutable game configuration
    pub fn game_config_mut(&mut self) -> &mut NewGameConfig {
        &mut self.game_config
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the wizard
    pub fn show(&mut self) {
        self.visible = true;
        self.current_step = WizardStep::NameEntry;
        self.game_config = NewGameConfig::new();
        self.validation_errors.clear();
    }

    /// Hide the wizard
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Get current step
    pub fn current_step(&self) -> WizardStep {
        self.current_step
    }

    /// Set current step
    pub fn set_step(&mut self, step: WizardStep) {
        self.current_step = step;
        self.validation_errors.clear();
    }

    /// Go to next step
    pub fn next_step(&mut self) -> bool {
        // Validate current step
        self.validation_errors = self.validate_current_step();
        if !self.validation_errors.is_empty() {
            return false;
        }

        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            self.actions.push(NewGameWizardAction::Next);
            true
        } else {
            false
        }
    }

    /// Go to previous step
    pub fn previous_step(&mut self) -> bool {
        if let Some(prev) = self.current_step.previous() {
            self.current_step = prev;
            self.validation_errors.clear();
            self.actions.push(NewGameWizardAction::Back);
            true
        } else {
            false
        }
    }

    /// Validate current step
    fn validate_current_step(&self) -> Vec<ValidationError> {
        match self.current_step {
            WizardStep::NameEntry => {
                let all_errors = self.game_config.validate();
                all_errors
                    .into_iter()
                    .filter(|e| {
                        matches!(
                            e,
                            ValidationError::WorldNameEmpty
                                | ValidationError::WorldNameTooLong
                                | ValidationError::WorldNameInvalid
                                | ValidationError::CharacterNameEmpty
                                | ValidationError::CharacterNameTooLong
                                | ValidationError::CharacterNameInvalid
                        )
                    })
                    .collect()
            },
            WizardStep::WorldSettings | WizardStep::Confirmation => Vec::new(),
        }
    }

    /// Cancel the wizard
    pub fn cancel(&mut self) {
        self.hide();
        self.actions.push(NewGameWizardAction::Cancel);
    }

    /// Create the game
    pub fn create_game(&mut self) {
        self.validation_errors = self.game_config.validate();
        if !self.validation_errors.is_empty() {
            return;
        }

        // Parse seed if custom
        if self.game_config.seed_mode == SeedMode::Custom {
            self.game_config.parse_custom_seed();
        }

        let config = self.game_config.clone();
        self.actions.push(NewGameWizardAction::Create(config));
        self.hide();
    }

    /// Regenerate seed
    pub fn regenerate_seed(&mut self) {
        self.game_config.regenerate_seed();
        self.actions.push(NewGameWizardAction::RegenerateSeed);
    }

    /// Get validation errors
    pub fn validation_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }

    /// Check if has validation errors
    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Drain pending actions
    pub fn drain_actions(&mut self) -> Vec<NewGameWizardAction> {
        std::mem::take(&mut self.actions)
    }

    /// Check if has pending actions
    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    /// Render the wizard
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

        egui::Area::new(egui::Id::new("new_game_wizard"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::none()
                    .fill(bg)
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)))
                    .rounding(8.0)
                    .inner_margin(24.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(
                            self.config.panel_width,
                            self.config.panel_height,
                        ));

                        self.render_header(ui);
                        self.render_step_indicator(ui);
                        self.render_content(ui);
                        self.render_errors(ui);
                        self.render_footer(ui);
                    });
            });
    }

    fn render_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&self.config.title)
                    .size(24.0)
                    .color(Color32::from_gray(220))
                    .strong(),
            );
        });

        ui.label(
            egui::RichText::new(self.current_step.title())
                .size(18.0)
                .color(Color32::from_rgba_unmultiplied(
                    self.config.accent_color[0],
                    self.config.accent_color[1],
                    self.config.accent_color[2],
                    self.config.accent_color[3],
                )),
        );

        ui.label(
            egui::RichText::new(self.current_step.description())
                .size(13.0)
                .color(Color32::from_gray(140)),
        );

        ui.add_space(16.0);
    }

    fn render_step_indicator(&self, ui: &mut Ui) {
        if !self.config.show_step_indicator {
            return;
        }

        let accent = Color32::from_rgba_unmultiplied(
            self.config.accent_color[0],
            self.config.accent_color[1],
            self.config.accent_color[2],
            self.config.accent_color[3],
        );

        ui.horizontal(|ui| {
            for step in WizardStep::all() {
                let is_current = *step == self.current_step;
                let is_completed = step.number() < self.current_step.number();

                let color = if is_current {
                    accent
                } else if is_completed {
                    Color32::from_rgb(100, 180, 100)
                } else {
                    Color32::from_gray(80)
                };

                let text = format!("{}", step.number());
                let size = egui::vec2(24.0, 24.0);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

                ui.painter().circle_filled(rect.center(), 12.0, color);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );

                if !step.is_last() {
                    let line_start = rect.right_center() + egui::vec2(4.0, 0.0);
                    let line_end = line_start + egui::vec2(20.0, 0.0);
                    let line_color = if is_completed {
                        Color32::from_rgb(100, 180, 100)
                    } else {
                        Color32::from_gray(60)
                    };
                    ui.painter()
                        .line_segment([line_start, line_end], egui::Stroke::new(2.0, line_color));
                    ui.add_space(28.0);
                }
            }
        });

        ui.add_space(16.0);
    }

    fn render_content(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .max_height(self.config.panel_height - 200.0)
            .show(ui, |ui| match self.current_step {
                WizardStep::NameEntry => self.render_name_step(ui),
                WizardStep::WorldSettings => self.render_settings_step(ui),
                WizardStep::Confirmation => self.render_confirmation_step(ui),
            });
    }

    fn render_name_step(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("World Name:");
            ui.add(
                egui::TextEdit::singleline(&mut self.game_config.world_name)
                    .desired_width(200.0)
                    .hint_text("My World"),
            );
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Character Name:");
            ui.add(
                egui::TextEdit::singleline(&mut self.game_config.character_name)
                    .desired_width(200.0)
                    .hint_text("Hero"),
            );
        });

        ui.add_space(16.0);

        if ui
            .checkbox(&mut self.game_config.enable_tutorial, "Enable Tutorial")
            .on_hover_text("Show helpful tips for new players")
            .changed()
        {}
    }

    fn render_settings_step(&mut self, ui: &mut Ui) {
        // Seed selection
        ui.horizontal(|ui| {
            ui.label("World Seed:");
            egui::ComboBox::from_id_salt("seed_mode")
                .selected_text(self.game_config.seed_mode.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.game_config.seed_mode,
                        SeedMode::Random,
                        "Random",
                    );
                    ui.selectable_value(
                        &mut self.game_config.seed_mode,
                        SeedMode::Custom,
                        "Custom",
                    );
                });
        });

        match self.game_config.seed_mode {
            SeedMode::Random => {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(self.game_config.formatted_seed())
                            .monospace()
                            .color(Color32::from_gray(180)),
                    );
                    if ui.button("🔄").on_hover_text("Regenerate").clicked() {
                        self.regenerate_seed();
                    }
                });
            },
            SeedMode::Custom => {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.game_config.custom_seed)
                            .desired_width(200.0)
                            .hint_text("Enter seed or text"),
                    );
                });
            },
        }

        ui.add_space(16.0);

        // World size
        ui.label(
            egui::RichText::new("World Size")
                .strong()
                .color(Color32::from_gray(200)),
        );
        for size in WorldSize::all() {
            let selected = self.game_config.world_size == *size;
            if ui
                .selectable_label(
                    selected,
                    format!("{} - {}", size.display_name(), size.description()),
                )
                .clicked()
            {
                self.game_config.world_size = *size;
            }
        }

        ui.add_space(16.0);

        // Difficulty
        ui.label(
            egui::RichText::new("Difficulty")
                .strong()
                .color(Color32::from_gray(200)),
        );
        for diff in NewGameDifficulty::all() {
            let selected = self.game_config.difficulty == *diff;
            let text = egui::RichText::new(diff.display_name()).color(diff.color());
            if ui
                .selectable_label(selected, text)
                .on_hover_text(diff.description())
                .clicked()
            {
                self.game_config.difficulty = *diff;
            }
        }

        if self.game_config.difficulty.is_warning() {
            ui.add_space(8.0);
            ui.colored_label(
                Color32::from_rgb(200, 150, 100),
                format!("⚠ {}", self.game_config.difficulty.description()),
            );
        }
    }

    fn render_confirmation_step(&self, ui: &mut Ui) {
        let game = &self.game_config;

        ui.label(egui::RichText::new("Review your settings:").strong());
        ui.add_space(8.0);

        egui::Grid::new("confirmation_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("World Name:");
                ui.label(egui::RichText::new(&game.world_name).color(Color32::from_gray(200)));
                ui.end_row();

                ui.label("Character:");
                ui.label(egui::RichText::new(&game.character_name).color(Color32::from_gray(200)));
                ui.end_row();

                ui.label("Seed:");
                ui.label(
                    egui::RichText::new(game.formatted_seed())
                        .monospace()
                        .color(Color32::from_gray(180)),
                );
                ui.end_row();

                ui.label("World Size:");
                ui.label(egui::RichText::new(format!(
                    "{} (~{} chunks)",
                    game.world_size.display_name(),
                    game.world_size.chunk_count()
                )));
                ui.end_row();

                ui.label("Difficulty:");
                ui.label(
                    egui::RichText::new(game.difficulty.display_name())
                        .color(game.difficulty.color()),
                );
                ui.end_row();

                ui.label("Tutorial:");
                ui.label(if game.enable_tutorial {
                    "Enabled"
                } else {
                    "Disabled"
                });
                ui.end_row();
            });

        if self.game_config.difficulty == NewGameDifficulty::Hardcore {
            ui.add_space(16.0);
            ui.colored_label(
                Color32::from_rgb(200, 80, 80),
                "⚠ HARDCORE MODE: Your save will be deleted upon death!",
            );
        }
    }

    fn render_errors(&self, ui: &mut Ui) {
        if self.validation_errors.is_empty() {
            return;
        }

        ui.add_space(8.0);
        for error in &self.validation_errors {
            ui.colored_label(
                Color32::from_rgb(200, 80, 80),
                format!("• {}", error.message()),
            );
        }
    }

    fn render_footer(&mut self, ui: &mut Ui) {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                self.cancel();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.current_step.is_last() {
                    if ui
                        .button(egui::RichText::new("Create World").strong())
                        .clicked()
                    {
                        self.create_game();
                    }
                } else if ui.button("Next →").clicked() {
                    self.next_step();
                }

                if !self.current_step.is_first() && ui.button("← Back").clicked() {
                    self.previous_step();
                }
            });
        });
    }
}

/// Generate a random seed
fn rand_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_nanos() as u64 ^ (duration.as_secs() << 32)
}

/// Hash a string to a seed
fn hash_string(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for c in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u64);
    }
    hash
}

/// Check if a name is valid
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_step_all() {
        let steps = WizardStep::all();
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_wizard_step_number() {
        assert_eq!(WizardStep::NameEntry.number(), 1);
        assert_eq!(WizardStep::WorldSettings.number(), 2);
        assert_eq!(WizardStep::Confirmation.number(), 3);
    }

    #[test]
    fn test_wizard_step_title() {
        assert!(!WizardStep::NameEntry.title().is_empty());
        assert!(!WizardStep::WorldSettings.title().is_empty());
    }

    #[test]
    fn test_wizard_step_navigation() {
        let step = WizardStep::NameEntry;
        assert!(step.is_first());
        assert!(!step.is_last());

        let next = step.next().unwrap();
        assert_eq!(next, WizardStep::WorldSettings);

        let back = next.previous().unwrap();
        assert_eq!(back, WizardStep::NameEntry);
    }

    #[test]
    fn test_wizard_step_last() {
        let step = WizardStep::Confirmation;
        assert!(step.is_last());
        assert!(step.next().is_none());
    }

    #[test]
    fn test_world_size_all() {
        let sizes = WorldSize::all();
        assert_eq!(sizes.len(), 4);
    }

    #[test]
    fn test_world_size_display_name() {
        assert_eq!(WorldSize::Small.display_name(), "Small");
        assert_eq!(WorldSize::Huge.display_name(), "Huge");
    }

    #[test]
    fn test_world_size_chunk_count() {
        assert!(WorldSize::Small.chunk_count() < WorldSize::Medium.chunk_count());
        assert!(WorldSize::Medium.chunk_count() < WorldSize::Large.chunk_count());
    }

    #[test]
    fn test_difficulty_all() {
        let diffs = NewGameDifficulty::all();
        assert_eq!(diffs.len(), 5);
    }

    #[test]
    fn test_difficulty_display_name() {
        assert_eq!(NewGameDifficulty::Easy.display_name(), "Easy");
        assert_eq!(NewGameDifficulty::Hardcore.display_name(), "Hardcore");
    }

    #[test]
    fn test_difficulty_is_warning() {
        assert!(!NewGameDifficulty::Normal.is_warning());
        assert!(NewGameDifficulty::Hard.is_warning());
        assert!(NewGameDifficulty::Hardcore.is_warning());
    }

    #[test]
    fn test_seed_mode_display_name() {
        assert_eq!(SeedMode::Random.display_name(), "Random");
        assert_eq!(SeedMode::Custom.display_name(), "Custom");
    }

    #[test]
    fn test_new_game_config_default() {
        let config = NewGameConfig::default();
        assert!(config.world_name.is_empty());
        assert_eq!(config.seed_mode, SeedMode::Random);
        assert!(config.enable_tutorial);
    }

    #[test]
    fn test_new_game_config_builders() {
        let config = NewGameConfig::new()
            .with_world_name("TestWorld")
            .with_character_name("Hero")
            .with_size(WorldSize::Large)
            .with_difficulty(NewGameDifficulty::Hard);

        assert_eq!(config.world_name, "TestWorld");
        assert_eq!(config.character_name, "Hero");
        assert_eq!(config.world_size, WorldSize::Large);
        assert_eq!(config.difficulty, NewGameDifficulty::Hard);
    }

    #[test]
    fn test_new_game_config_with_seed() {
        let config = NewGameConfig::new().with_seed(12345);
        assert_eq!(config.seed_value, 12345);
        assert_eq!(config.seed_mode, SeedMode::Custom);
    }

    #[test]
    fn test_new_game_config_regenerate_seed() {
        let mut config = NewGameConfig::new();
        let original = config.seed_value;
        // Sleep briefly to ensure time-based seed changes
        std::thread::sleep(std::time::Duration::from_millis(1));
        config.regenerate_seed();
        // Seeds should differ (with very high probability)
        // This test could theoretically fail, but it's extremely unlikely
        assert_ne!(config.seed_value, original);
    }

    #[test]
    fn test_new_game_config_parse_custom_seed_number() {
        let mut config = NewGameConfig::new();
        config.custom_seed = String::from("42");
        config.parse_custom_seed();
        assert_eq!(config.seed_value, 42);
    }

    #[test]
    fn test_new_game_config_parse_custom_seed_hex() {
        let mut config = NewGameConfig::new();
        config.custom_seed = String::from("0xFF");
        config.parse_custom_seed();
        assert_eq!(config.seed_value, 255);
    }

    #[test]
    fn test_new_game_config_parse_custom_seed_string() {
        let mut config = NewGameConfig::new();
        config.custom_seed = String::from("hello");
        config.parse_custom_seed();
        // Should be hashed
        assert!(config.seed_value > 0);
    }

    #[test]
    fn test_new_game_config_formatted_seed() {
        let config = NewGameConfig::new().with_seed(0x1234);
        let formatted = config.formatted_seed();
        assert_eq!(formatted, "0000000000001234");
    }

    #[test]
    fn test_new_game_config_validate_empty() {
        let config = NewGameConfig::default();
        let errors = config.validate();
        assert!(errors.contains(&ValidationError::WorldNameEmpty));
        assert!(errors.contains(&ValidationError::CharacterNameEmpty));
    }

    #[test]
    fn test_new_game_config_validate_valid() {
        let config = NewGameConfig::new()
            .with_world_name("Test")
            .with_character_name("Hero");
        assert!(config.is_valid());
    }

    #[test]
    fn test_new_game_config_validate_too_long() {
        let long_name = "a".repeat(50);
        let config = NewGameConfig::new()
            .with_world_name(&long_name)
            .with_character_name("Hero");
        let errors = config.validate();
        assert!(errors.contains(&ValidationError::WorldNameTooLong));
    }

    #[test]
    fn test_validation_error_message() {
        assert!(!ValidationError::WorldNameEmpty.message().is_empty());
        assert!(!ValidationError::CharacterNameInvalid.message().is_empty());
    }

    #[test]
    fn test_wizard_config_defaults() {
        let config = NewGameWizardConfig::default();
        assert_eq!(config.title, "New Game");
        assert!(config.panel_width > 0.0);
        assert!(config.show_step_indicator);
    }

    #[test]
    fn test_new_game_wizard_new() {
        let wizard = NewGameWizard::with_defaults();
        assert!(!wizard.is_visible());
        assert_eq!(wizard.current_step(), WizardStep::NameEntry);
    }

    #[test]
    fn test_new_game_wizard_visibility() {
        let mut wizard = NewGameWizard::with_defaults();

        wizard.show();
        assert!(wizard.is_visible());

        wizard.hide();
        assert!(!wizard.is_visible());

        wizard.toggle();
        assert!(wizard.is_visible());
    }

    #[test]
    fn test_new_game_wizard_show_resets() {
        let mut wizard = NewGameWizard::with_defaults();
        wizard.game_config_mut().world_name = String::from("Test");
        wizard.set_step(WizardStep::Confirmation);

        wizard.show();
        assert_eq!(wizard.current_step(), WizardStep::NameEntry);
        assert!(wizard.game_config().world_name.is_empty());
    }

    #[test]
    fn test_new_game_wizard_next_step_with_validation() {
        let mut wizard = NewGameWizard::with_defaults();
        wizard.show();

        // Try to advance without filling names
        let result = wizard.next_step();
        assert!(!result);
        assert!(wizard.has_errors());

        // Fill names and try again
        wizard.game_config_mut().world_name = String::from("Test");
        wizard.game_config_mut().character_name = String::from("Hero");

        let result = wizard.next_step();
        assert!(result);
        assert_eq!(wizard.current_step(), WizardStep::WorldSettings);
    }

    #[test]
    fn test_new_game_wizard_previous_step() {
        let mut wizard = NewGameWizard::with_defaults();
        wizard.show();
        wizard.game_config_mut().world_name = String::from("Test");
        wizard.game_config_mut().character_name = String::from("Hero");
        wizard.next_step();

        assert_eq!(wizard.current_step(), WizardStep::WorldSettings);

        wizard.previous_step();
        assert_eq!(wizard.current_step(), WizardStep::NameEntry);
    }

    #[test]
    fn test_new_game_wizard_cancel() {
        let mut wizard = NewGameWizard::with_defaults();
        wizard.show();
        wizard.cancel();

        assert!(!wizard.is_visible());
        let actions = wizard.drain_actions();
        assert!(actions.iter().any(|a| *a == NewGameWizardAction::Cancel));
    }

    #[test]
    fn test_new_game_wizard_create_game() {
        let mut wizard = NewGameWizard::with_defaults();
        wizard.show();
        wizard.game_config_mut().world_name = String::from("MyWorld");
        wizard.game_config_mut().character_name = String::from("Hero");

        wizard.create_game();

        assert!(!wizard.is_visible());
        let actions = wizard.drain_actions();
        assert!(actions
            .iter()
            .any(|a| matches!(a, NewGameWizardAction::Create(_))));
    }

    #[test]
    fn test_new_game_wizard_create_game_invalid() {
        let mut wizard = NewGameWizard::with_defaults();
        wizard.show();
        // Don't fill names

        wizard.create_game();

        // Should still be visible with errors
        assert!(wizard.is_visible() || wizard.has_errors());
    }

    #[test]
    fn test_new_game_wizard_regenerate_seed() {
        let mut wizard = NewGameWizard::with_defaults();
        let original = wizard.game_config().seed_value;

        // Sleep briefly to ensure time-based seed changes
        std::thread::sleep(std::time::Duration::from_millis(1));
        wizard.regenerate_seed();

        assert_ne!(wizard.game_config().seed_value, original);
    }

    #[test]
    fn test_is_valid_name() {
        assert!(is_valid_name("Test"));
        assert!(is_valid_name("Test World"));
        assert!(is_valid_name("Test_World"));
        assert!(is_valid_name("Test-World"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("Test!World"));
    }

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("test");
        let hash2 = hash_string("test");
        let hash3 = hash_string("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_wizard_action_equality() {
        assert_eq!(NewGameWizardAction::Next, NewGameWizardAction::Next);
        assert_ne!(NewGameWizardAction::Next, NewGameWizardAction::Back);
    }

    #[test]
    fn test_new_game_config_serialization() {
        let config = NewGameConfig::new()
            .with_world_name("Test")
            .with_character_name("Hero");
        let json = serde_json::to_string(&config).unwrap();
        let parsed: NewGameConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.world_name, config.world_name);
    }

    #[test]
    fn test_wizard_config_serialization() {
        let config = NewGameWizardConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: NewGameWizardConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, config.title);
    }
}
