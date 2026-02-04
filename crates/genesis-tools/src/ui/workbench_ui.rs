//! Workbench UI for station-specific crafting interfaces.
//!
//! Provides specialized crafting station panels:
//! - Forge UI with fuel gauge and heat indicator
//! - Alchemy UI with flask slots and mixing
//! - Progress bars for active crafting
//! - Station-specific bonuses and modifiers

use egui::{Color32, Ui, Vec2};
use serde::{Deserialize, Serialize};

use super::crafting_grid::CraftingItem;
use super::recipe_book::RecipeId;

/// Types of crafting stations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StationType {
    /// Basic hand crafting.
    #[default]
    HandCrafting,
    /// Forge for metalworking.
    Forge,
    /// Alchemy station for potions.
    Alchemy,
    /// Cooking station.
    Cooking,
    /// Woodworking bench.
    Woodworking,
    /// Enchanting table.
    Enchanting,
    /// Tailoring station.
    Tailoring,
}

impl StationType {
    /// Get all station types.
    pub fn all() -> &'static [StationType] {
        &[
            StationType::HandCrafting,
            StationType::Forge,
            StationType::Alchemy,
            StationType::Cooking,
            StationType::Woodworking,
            StationType::Enchanting,
            StationType::Tailoring,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            StationType::HandCrafting => "Hand Crafting",
            StationType::Forge => "Forge",
            StationType::Alchemy => "Alchemy Lab",
            StationType::Cooking => "Cooking Fire",
            StationType::Woodworking => "Woodworking Bench",
            StationType::Enchanting => "Enchanting Table",
            StationType::Tailoring => "Tailoring Station",
        }
    }

    /// Get station icon.
    pub fn icon(&self) -> &'static str {
        match self {
            StationType::HandCrafting => "✋",
            StationType::Forge => "🔥",
            StationType::Alchemy => "🧪",
            StationType::Cooking => "🍳",
            StationType::Woodworking => "🪓",
            StationType::Enchanting => "✨",
            StationType::Tailoring => "🧵",
        }
    }

    /// Get whether station requires fuel.
    pub fn requires_fuel(&self) -> bool {
        matches!(self, StationType::Forge | StationType::Cooking)
    }

    /// Get whether station has heat mechanic.
    pub fn has_heat(&self) -> bool {
        matches!(
            self,
            StationType::Forge | StationType::Cooking | StationType::Alchemy
        )
    }
}

/// Fuel type for stations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuelType {
    /// Wood fuel.
    Wood,
    /// Coal fuel.
    Coal,
    /// Magic fuel crystals.
    MagicCrystal,
}

impl FuelType {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            FuelType::Wood => "Wood",
            FuelType::Coal => "Coal",
            FuelType::MagicCrystal => "Magic Crystal",
        }
    }

    /// Get burn time multiplier.
    pub fn burn_time_multiplier(&self) -> f32 {
        match self {
            FuelType::Wood => 1.0,
            FuelType::Coal => 2.5,
            FuelType::MagicCrystal => 5.0,
        }
    }

    /// Get heat bonus.
    pub fn heat_bonus(&self) -> f32 {
        match self {
            FuelType::Wood => 0.0,
            FuelType::Coal => 0.2,
            FuelType::MagicCrystal => 0.5,
        }
    }
}

/// Fuel gauge for stations requiring fuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelGauge {
    /// Current fuel amount (0.0 - 1.0).
    pub current: f32,
    /// Maximum fuel capacity.
    pub max_capacity: f32,
    /// Burn rate per second.
    pub burn_rate: f32,
    /// Current fuel type.
    pub fuel_type: Option<FuelType>,
    /// Time remaining in seconds.
    pub time_remaining: f32,
}

impl Default for FuelGauge {
    fn default() -> Self {
        Self {
            current: 0.0,
            max_capacity: 100.0,
            burn_rate: 1.0,
            fuel_type: None,
            time_remaining: 0.0,
        }
    }
}

impl FuelGauge {
    /// Create new fuel gauge.
    pub fn new(max_capacity: f32) -> Self {
        Self {
            max_capacity,
            ..Self::default()
        }
    }

    /// Add fuel.
    pub fn add_fuel(&mut self, amount: f32, fuel_type: FuelType) {
        let effective_amount = amount * fuel_type.burn_time_multiplier();
        self.current = (self.current + effective_amount).min(self.max_capacity);
        self.fuel_type = Some(fuel_type);
        self.update_time_remaining();
    }

    /// Consume fuel over delta time.
    pub fn consume(&mut self, dt: f32) -> bool {
        if self.current <= 0.0 {
            return false;
        }
        self.current = (self.current - self.burn_rate * dt).max(0.0);
        self.update_time_remaining();
        if self.current <= 0.0 {
            self.fuel_type = None;
        }
        true
    }

    /// Update time remaining calculation.
    fn update_time_remaining(&mut self) {
        if self.burn_rate > 0.0 {
            self.time_remaining = self.current / self.burn_rate;
        } else {
            self.time_remaining = f32::INFINITY;
        }
    }

    /// Get fuel percentage.
    pub fn percentage(&self) -> f32 {
        if self.max_capacity > 0.0 {
            (self.current / self.max_capacity).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if station has fuel.
    pub fn has_fuel(&self) -> bool {
        self.current > 0.0
    }
}

/// Heat level for stations with temperature mechanics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatLevel {
    /// Current temperature (0.0 - 1.0).
    pub current: f32,
    /// Target temperature.
    pub target: f32,
    /// Heat up rate per second.
    pub heat_rate: f32,
    /// Cool down rate per second.
    pub cool_rate: f32,
    /// Optimal temperature range for crafting.
    pub optimal_min: f32,
    /// Maximum optimal temperature.
    pub optimal_max: f32,
}

impl Default for HeatLevel {
    fn default() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            heat_rate: 0.1,
            cool_rate: 0.05,
            optimal_min: 0.6,
            optimal_max: 0.9,
        }
    }
}

impl HeatLevel {
    /// Create new heat level.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set target temperature.
    pub fn set_target(&mut self, target: f32) {
        self.target = target.clamp(0.0, 1.0);
    }

    /// Update temperature over time.
    pub fn update(&mut self, dt: f32, has_fuel: bool) {
        let actual_target = if has_fuel { self.target } else { 0.0 };

        if self.current < actual_target {
            self.current = (self.current + self.heat_rate * dt).min(actual_target);
        } else if self.current > actual_target {
            self.current = (self.current - self.cool_rate * dt).max(actual_target);
        }
    }

    /// Check if in optimal range.
    pub fn is_optimal(&self) -> bool {
        self.current >= self.optimal_min && self.current <= self.optimal_max
    }

    /// Get heat color based on temperature.
    pub fn color(&self) -> Color32 {
        if self.current < 0.3 {
            Color32::from_rgb(100, 100, 150) // Cold blue
        } else if self.current < 0.6 {
            Color32::from_rgb(200, 150, 50) // Warm yellow
        } else if self.current < 0.8 {
            Color32::from_rgb(255, 100, 50) // Hot orange
        } else {
            Color32::from_rgb(255, 50, 50) // Very hot red
        }
    }

    /// Get quality modifier based on temperature.
    pub fn quality_modifier(&self) -> f32 {
        if self.is_optimal() {
            1.0 + (self.current - self.optimal_min) * 0.5
        } else if self.current < self.optimal_min {
            0.7 + self.current * 0.5
        } else {
            // Overheated
            1.0 - (self.current - self.optimal_max) * 2.0
        }
    }
}

/// Flask slot for alchemy station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlaskSlot {
    /// Slot index.
    pub index: usize,
    /// Contained item.
    pub item: Option<CraftingItem>,
    /// Mixing progress (0.0 - 1.0).
    pub mixing_progress: f32,
    /// Whether slot is active/bubbling.
    pub active: bool,
}

impl FlaskSlot {
    /// Create new flask slot.
    pub fn new(index: usize) -> Self {
        Self {
            index,
            item: None,
            mixing_progress: 0.0,
            active: false,
        }
    }

    /// Set item in slot.
    pub fn set_item(&mut self, item: CraftingItem) {
        self.item = Some(item);
        self.mixing_progress = 0.0;
    }

    /// Clear the slot.
    pub fn clear(&mut self) {
        self.item = None;
        self.mixing_progress = 0.0;
        self.active = false;
    }

    /// Is slot empty.
    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }
}

/// Active crafting job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingJob {
    /// Recipe being crafted.
    pub recipe_id: RecipeId,
    /// Recipe display name.
    pub name: String,
    /// Total time required.
    pub total_time: f32,
    /// Elapsed time.
    pub elapsed_time: f32,
    /// Quantity being crafted.
    pub quantity: u32,
    /// Whether job is paused.
    pub paused: bool,
}

impl CraftingJob {
    /// Create new crafting job.
    pub fn new(
        recipe_id: RecipeId,
        name: impl Into<String>,
        total_time: f32,
        quantity: u32,
    ) -> Self {
        Self {
            recipe_id,
            name: name.into(),
            total_time,
            elapsed_time: 0.0,
            quantity,
            paused: false,
        }
    }

    /// Update job progress.
    pub fn update(&mut self, dt: f32) -> bool {
        if self.paused {
            return false;
        }
        self.elapsed_time += dt;
        self.is_complete()
    }

    /// Get progress percentage.
    pub fn progress(&self) -> f32 {
        if self.total_time > 0.0 {
            (self.elapsed_time / self.total_time).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Check if job is complete.
    pub fn is_complete(&self) -> bool {
        self.elapsed_time >= self.total_time
    }

    /// Get remaining time.
    pub fn remaining_time(&self) -> f32 {
        (self.total_time - self.elapsed_time).max(0.0)
    }
}

/// Actions returned by workbench UI.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkbenchAction {
    /// Add fuel to station.
    AddFuel(FuelType, u32),
    /// Adjust heat target.
    SetHeat(f32),
    /// Start crafting job.
    StartCraft(RecipeId),
    /// Cancel active job.
    CancelJob,
    /// Pause/resume job.
    TogglePause,
    /// Flask slot clicked.
    FlaskClicked(usize),
    /// Take output item.
    TakeOutput,
    /// Close workbench.
    Close,
}

/// Configuration for workbench UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkbenchConfig {
    /// Show fuel gauge.
    pub show_fuel: bool,
    /// Show heat indicator.
    pub show_heat: bool,
    /// Show crafting queue.
    pub show_queue: bool,
    /// Number of flask slots (alchemy).
    pub flask_slots: usize,
    /// Panel width.
    pub width: f32,
}

impl Default for WorkbenchConfig {
    fn default() -> Self {
        Self {
            show_fuel: true,
            show_heat: true,
            show_queue: true,
            flask_slots: 3,
            width: 300.0,
        }
    }
}

/// Workbench UI widget.
#[derive(Debug)]
pub struct WorkbenchUi {
    /// Station type.
    pub station_type: StationType,
    /// Configuration.
    pub config: WorkbenchConfig,
    /// Fuel gauge.
    pub fuel: FuelGauge,
    /// Heat level.
    pub heat: HeatLevel,
    /// Flask slots (alchemy).
    pub flasks: Vec<FlaskSlot>,
    /// Active crafting job.
    pub active_job: Option<CraftingJob>,
    /// Output items ready for pickup.
    pub output_items: Vec<CraftingItem>,
    /// Whether workbench is open.
    pub open: bool,
    /// Pending actions.
    pending_actions: Vec<WorkbenchAction>,
}

impl Default for WorkbenchUi {
    fn default() -> Self {
        Self::new(StationType::HandCrafting)
    }
}

impl WorkbenchUi {
    /// Create new workbench UI.
    pub fn new(station_type: StationType) -> Self {
        let config = WorkbenchConfig::default();
        let flasks = (0..config.flask_slots).map(FlaskSlot::new).collect();

        Self {
            station_type,
            config,
            fuel: FuelGauge::default(),
            heat: HeatLevel::default(),
            flasks,
            active_job: None,
            output_items: Vec::new(),
            open: false,
            pending_actions: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(station_type: StationType, config: WorkbenchConfig) -> Self {
        let flasks = (0..config.flask_slots).map(FlaskSlot::new).collect();

        Self {
            station_type,
            config,
            flasks,
            ..Self::new(station_type)
        }
    }

    /// Open the workbench.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the workbench.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Start a crafting job.
    pub fn start_job(&mut self, job: CraftingJob) {
        self.active_job = Some(job);
    }

    /// Cancel current job.
    pub fn cancel_job(&mut self) {
        self.active_job = None;
    }

    /// Add output item.
    pub fn add_output(&mut self, item: CraftingItem) {
        self.output_items.push(item);
    }

    /// Take all output items.
    pub fn take_outputs(&mut self) -> Vec<CraftingItem> {
        std::mem::take(&mut self.output_items)
    }

    /// Update station state.
    pub fn update(&mut self, dt: f32) {
        // Update fuel consumption if crafting
        if self.active_job.is_some() && self.station_type.requires_fuel() {
            self.fuel.consume(dt);
        }

        // Update heat
        if self.station_type.has_heat() {
            self.heat.update(
                dt,
                self.fuel.has_fuel() || !self.station_type.requires_fuel(),
            );
        }

        // Update crafting job
        if let Some(job) = &mut self.active_job {
            // Check if can craft (has fuel if required)
            let can_craft = !self.station_type.requires_fuel() || self.fuel.has_fuel();
            if can_craft && job.update(dt) {
                // Job complete - would trigger output generation
            }
        }
    }

    /// Render the workbench UI.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<WorkbenchAction> {
        self.pending_actions.clear();

        if !self.open {
            return Vec::new();
        }

        // Clone data needed for UI
        let station_type = self.station_type;
        let has_job = self.active_job.is_some();
        let output_count = self.output_items.len();

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(format!(
                    "{} {}",
                    station_type.icon(),
                    station_type.display_name()
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        self.pending_actions.push(WorkbenchAction::Close);
                    }
                });
            });

            ui.separator();

            // Station-specific UI
            match station_type {
                StationType::Forge => self.show_forge_ui(ui),
                StationType::Alchemy => self.show_alchemy_ui(ui),
                StationType::Cooking => self.show_cooking_ui(ui),
                _ => self.show_generic_ui(ui),
            }

            ui.separator();

            // Active job progress
            if has_job {
                self.show_job_progress(ui);
            }

            // Output items
            if output_count > 0 {
                self.show_output_items(ui);
            }
        });

        std::mem::take(&mut self.pending_actions)
    }

    /// Show forge-specific UI.
    fn show_forge_ui(&mut self, ui: &mut Ui) {
        // Fuel gauge
        if self.config.show_fuel {
            self.show_fuel_gauge(ui);
        }

        // Heat indicator
        if self.config.show_heat {
            self.show_heat_indicator(ui);
        }

        // Fuel buttons
        ui.horizontal(|ui| {
            ui.label("Add fuel:");
            if ui.button("🪵 Wood").clicked() {
                self.pending_actions
                    .push(WorkbenchAction::AddFuel(FuelType::Wood, 1));
            }
            if ui.button("⬛ Coal").clicked() {
                self.pending_actions
                    .push(WorkbenchAction::AddFuel(FuelType::Coal, 1));
            }
        });

        // Heat control
        ui.horizontal(|ui| {
            ui.label("Heat target:");
            let mut target = self.heat.target;
            if ui
                .add(egui::Slider::new(&mut target, 0.0..=1.0).show_value(false))
                .changed()
            {
                self.heat.set_target(target);
                self.pending_actions.push(WorkbenchAction::SetHeat(target));
            }
        });
    }

    /// Show alchemy-specific UI.
    fn show_alchemy_ui(&mut self, ui: &mut Ui) {
        // Heat indicator (for reactions)
        if self.config.show_heat {
            self.show_heat_indicator(ui);
        }

        // Flask slots
        ui.label("Flask Slots:");
        ui.horizontal(|ui| {
            for i in 0..self.flasks.len() {
                let flask = &self.flasks[i];
                let is_empty = flask.is_empty();
                let is_active = flask.active;

                let frame_color = if is_active {
                    Color32::from_rgb(100, 200, 100)
                } else if is_empty {
                    Color32::from_gray(60)
                } else {
                    Color32::from_rgb(100, 150, 200)
                };

                let response = egui::Frame::none()
                    .fill(frame_color)
                    .inner_margin(8.0)
                    .rounding(4.0)
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::new(50.0, 50.0));
                        if let Some(item) = &flask.item {
                            ui.label(&item.name);
                            if flask.active {
                                let progress = flask.mixing_progress;
                                ui.add(egui::ProgressBar::new(progress).show_percentage());
                            }
                        } else {
                            ui.weak("Empty");
                        }
                    })
                    .response;

                if response.clicked() {
                    self.pending_actions.push(WorkbenchAction::FlaskClicked(i));
                }
            }
        });

        // Mix button
        let filled_count = self.flasks.iter().filter(|f| !f.is_empty()).count();
        if ui
            .add_enabled(filled_count >= 2, egui::Button::new("🧪 Mix"))
            .clicked()
        {
            // Would start mixing - requires recipe lookup
        }
    }

    /// Show cooking-specific UI.
    fn show_cooking_ui(&mut self, ui: &mut Ui) {
        // Fuel gauge
        if self.config.show_fuel {
            self.show_fuel_gauge(ui);
        }

        // Heat indicator
        if self.config.show_heat {
            self.show_heat_indicator(ui);
        }

        // Fuel buttons
        ui.horizontal(|ui| {
            ui.label("Add fuel:");
            if ui.button("🪵 Wood").clicked() {
                self.pending_actions
                    .push(WorkbenchAction::AddFuel(FuelType::Wood, 1));
            }
        });
    }

    /// Show generic workbench UI.
    fn show_generic_ui(&mut self, ui: &mut Ui) {
        ui.label("Ready to craft");

        // Simple status
        if let Some(job) = &self.active_job {
            ui.label(format!("Crafting: {}", job.name));
        } else {
            ui.weak("Select a recipe to begin");
        }
    }

    /// Show fuel gauge.
    fn show_fuel_gauge(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("🔥 Fuel:");

            let percentage = self.fuel.percentage();
            let color = if percentage > 0.5 {
                Color32::from_rgb(255, 150, 50)
            } else if percentage > 0.2 {
                Color32::from_rgb(200, 100, 50)
            } else {
                Color32::from_rgb(100, 50, 50)
            };

            let bar_width = 100.0;
            let bar_rect = ui.allocate_space(Vec2::new(bar_width, 16.0)).1;

            ui.painter()
                .rect_filled(bar_rect, 2.0, Color32::from_gray(40));

            let filled_rect = egui::Rect::from_min_size(
                bar_rect.min,
                Vec2::new(bar_rect.width() * percentage, bar_rect.height()),
            );
            ui.painter().rect_filled(filled_rect, 2.0, color);

            // Time remaining
            let time = self.fuel.time_remaining;
            if time > 60.0 {
                ui.label(format!("{:.0}m", time / 60.0));
            } else {
                ui.label(format!("{time:.0}s"));
            }
        });
    }

    /// Show heat indicator.
    fn show_heat_indicator(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("🌡 Heat:");

            let temp = self.heat.current;
            let color = self.heat.color();

            let bar_width = 100.0;
            let bar_rect = ui.allocate_space(Vec2::new(bar_width, 16.0)).1;

            ui.painter()
                .rect_filled(bar_rect, 2.0, Color32::from_gray(40));

            let filled_rect = egui::Rect::from_min_size(
                bar_rect.min,
                Vec2::new(bar_rect.width() * temp, bar_rect.height()),
            );
            ui.painter().rect_filled(filled_rect, 2.0, color);

            // Optimal range indicator
            let optimal_start = bar_rect.min.x + bar_rect.width() * self.heat.optimal_min;
            let optimal_end = bar_rect.min.x + bar_rect.width() * self.heat.optimal_max;
            let optimal_rect = egui::Rect::from_min_max(
                egui::Pos2::new(optimal_start, bar_rect.min.y),
                egui::Pos2::new(optimal_end, bar_rect.max.y),
            );
            ui.painter()
                .rect_stroke(optimal_rect, 0.0, egui::Stroke::new(1.0, Color32::WHITE));

            if self.heat.is_optimal() {
                ui.colored_label(Color32::from_rgb(100, 200, 100), "Optimal");
            } else if temp < self.heat.optimal_min {
                ui.weak("Cold");
            } else {
                ui.colored_label(Color32::from_rgb(255, 100, 100), "Hot!");
            }
        });
    }

    /// Show crafting job progress.
    fn show_job_progress(&mut self, ui: &mut Ui) {
        let Some(job) = &self.active_job else {
            return;
        };

        let name = job.name.clone();
        let progress = job.progress();
        let remaining = job.remaining_time();
        let paused = job.paused;
        let quantity = job.quantity;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("🔨 Crafting: {name} x{quantity}"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").on_hover_text("Cancel").clicked() {
                        self.pending_actions.push(WorkbenchAction::CancelJob);
                    }
                    let pause_text = if paused { "▶" } else { "⏸" };
                    if ui.button(pause_text).clicked() {
                        self.pending_actions.push(WorkbenchAction::TogglePause);
                    }
                });
            });

            ui.add(egui::ProgressBar::new(progress).show_percentage());

            if remaining > 60.0 {
                let minutes = remaining / 60.0;
                ui.small(format!("~{minutes:.0}m remaining"));
            } else {
                ui.small(format!("~{remaining:.0}s remaining"));
            }

            if paused {
                ui.colored_label(Color32::from_rgb(255, 200, 100), "⏸ Paused");
            }
        });
    }

    /// Show output items ready for pickup.
    fn show_output_items(&mut self, ui: &mut Ui) {
        let count = self.output_items.len();

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("📦 Output ({count} items ready)"));
                if ui.button("Take All").clicked() {
                    self.pending_actions.push(WorkbenchAction::TakeOutput);
                }
            });

            // Show preview of outputs
            ui.horizontal_wrapped(|ui| {
                for item in &self.output_items {
                    ui.colored_label(
                        item.rarity.color(),
                        format!("{}x {}", item.count, item.name),
                    );
                }
            });
        });
    }

    /// Drain pending actions.
    pub fn drain_actions(&mut self) -> Vec<WorkbenchAction> {
        std::mem::take(&mut self.pending_actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_station_type() {
        assert_eq!(StationType::all().len(), 7);
        assert_eq!(StationType::Forge.display_name(), "Forge");
        assert_eq!(StationType::Forge.icon(), "🔥");
    }

    #[test]
    fn test_station_requires_fuel() {
        assert!(StationType::Forge.requires_fuel());
        assert!(StationType::Cooking.requires_fuel());
        assert!(!StationType::Alchemy.requires_fuel());
        assert!(!StationType::HandCrafting.requires_fuel());
    }

    #[test]
    fn test_station_has_heat() {
        assert!(StationType::Forge.has_heat());
        assert!(StationType::Alchemy.has_heat());
        assert!(StationType::Cooking.has_heat());
        assert!(!StationType::HandCrafting.has_heat());
    }

    #[test]
    fn test_fuel_type() {
        assert_eq!(FuelType::Wood.display_name(), "Wood");
        assert_eq!(FuelType::Wood.burn_time_multiplier(), 1.0);
        assert!(FuelType::Coal.burn_time_multiplier() > FuelType::Wood.burn_time_multiplier());
    }

    #[test]
    fn test_fuel_gauge_new() {
        let gauge = FuelGauge::new(100.0);
        assert_eq!(gauge.max_capacity, 100.0);
        assert_eq!(gauge.current, 0.0);
        assert!(!gauge.has_fuel());
    }

    #[test]
    fn test_fuel_gauge_add_fuel() {
        let mut gauge = FuelGauge::new(100.0);
        gauge.add_fuel(20.0, FuelType::Wood);

        assert!(gauge.has_fuel());
        assert_eq!(gauge.current, 20.0);
        assert_eq!(gauge.fuel_type, Some(FuelType::Wood));
    }

    #[test]
    fn test_fuel_gauge_add_fuel_with_multiplier() {
        let mut gauge = FuelGauge::new(100.0);
        gauge.add_fuel(20.0, FuelType::Coal);

        // Coal has 2.5x multiplier
        assert_eq!(gauge.current, 50.0);
    }

    #[test]
    fn test_fuel_gauge_consume() {
        let mut gauge = FuelGauge::new(100.0);
        gauge.burn_rate = 10.0;
        gauge.add_fuel(50.0, FuelType::Wood);

        assert!(gauge.consume(1.0));
        assert_eq!(gauge.current, 40.0);

        // Consume all
        assert!(gauge.consume(10.0));
        assert!(!gauge.has_fuel());
    }

    #[test]
    fn test_fuel_gauge_percentage() {
        let mut gauge = FuelGauge::new(100.0);
        gauge.current = 25.0;

        assert_eq!(gauge.percentage(), 0.25);
    }

    #[test]
    fn test_heat_level_new() {
        let heat = HeatLevel::new();
        assert_eq!(heat.current, 0.0);
        assert_eq!(heat.target, 0.0);
        assert!(!heat.is_optimal());
    }

    #[test]
    fn test_heat_level_set_target() {
        let mut heat = HeatLevel::new();
        heat.set_target(0.8);
        assert_eq!(heat.target, 0.8);

        // Test clamping
        heat.set_target(1.5);
        assert_eq!(heat.target, 1.0);
    }

    #[test]
    fn test_heat_level_update() {
        let mut heat = HeatLevel::new();
        heat.heat_rate = 0.5;
        heat.set_target(1.0);

        heat.update(1.0, true);
        assert!(heat.current > 0.0);
        assert!(heat.current <= 0.5);
    }

    #[test]
    fn test_heat_level_is_optimal() {
        let mut heat = HeatLevel::new();
        heat.optimal_min = 0.6;
        heat.optimal_max = 0.9;

        heat.current = 0.5;
        assert!(!heat.is_optimal());

        heat.current = 0.7;
        assert!(heat.is_optimal());

        heat.current = 0.95;
        assert!(!heat.is_optimal());
    }

    #[test]
    fn test_heat_level_quality_modifier() {
        let mut heat = HeatLevel::new();
        heat.optimal_min = 0.6;
        heat.optimal_max = 0.9;

        heat.current = 0.3;
        assert!(heat.quality_modifier() < 1.0);

        heat.current = 0.75;
        assert!(heat.quality_modifier() >= 1.0);
    }

    #[test]
    fn test_flask_slot_new() {
        let slot = FlaskSlot::new(0);
        assert!(slot.is_empty());
        assert!(!slot.active);
        assert_eq!(slot.mixing_progress, 0.0);
    }

    #[test]
    fn test_flask_slot_set_item() {
        let mut slot = FlaskSlot::new(0);
        let item = CraftingItem::new("potion", "Health Potion");
        slot.set_item(item);

        assert!(!slot.is_empty());
    }

    #[test]
    fn test_flask_slot_clear() {
        let mut slot = FlaskSlot::new(0);
        let item = CraftingItem::new("potion", "Health Potion");
        slot.set_item(item);
        slot.active = true;

        slot.clear();
        assert!(slot.is_empty());
        assert!(!slot.active);
    }

    #[test]
    fn test_crafting_job_new() {
        let job = CraftingJob::new(RecipeId::new("sword"), "Iron Sword", 10.0, 1);

        assert_eq!(job.name, "Iron Sword");
        assert_eq!(job.total_time, 10.0);
        assert_eq!(job.elapsed_time, 0.0);
        assert!(!job.paused);
        assert!(!job.is_complete());
    }

    #[test]
    fn test_crafting_job_update() {
        let mut job = CraftingJob::new(RecipeId::new("sword"), "Sword", 10.0, 1);

        job.update(3.0);
        assert_eq!(job.elapsed_time, 3.0);
        assert_eq!(job.progress(), 0.3);
        assert!(!job.is_complete());

        job.update(7.0);
        assert!(job.is_complete());
    }

    #[test]
    fn test_crafting_job_paused() {
        let mut job = CraftingJob::new(RecipeId::new("sword"), "Sword", 10.0, 1);
        job.paused = true;

        job.update(5.0);
        assert_eq!(job.elapsed_time, 0.0); // No progress when paused
    }

    #[test]
    fn test_crafting_job_remaining_time() {
        let mut job = CraftingJob::new(RecipeId::new("sword"), "Sword", 10.0, 1);
        job.elapsed_time = 3.0;

        assert_eq!(job.remaining_time(), 7.0);
    }

    #[test]
    fn test_workbench_ui_new() {
        let ui = WorkbenchUi::new(StationType::Forge);
        assert_eq!(ui.station_type, StationType::Forge);
        assert!(!ui.open);
        assert!(ui.active_job.is_none());
    }

    #[test]
    fn test_workbench_ui_open_close() {
        let mut ui = WorkbenchUi::new(StationType::Forge);
        assert!(!ui.open);

        ui.open();
        assert!(ui.open);

        ui.close();
        assert!(!ui.open);
    }

    #[test]
    fn test_workbench_ui_start_job() {
        let mut ui = WorkbenchUi::new(StationType::Forge);
        let job = CraftingJob::new(RecipeId::new("sword"), "Sword", 10.0, 1);

        ui.start_job(job);
        assert!(ui.active_job.is_some());

        ui.cancel_job();
        assert!(ui.active_job.is_none());
    }

    #[test]
    fn test_workbench_ui_output() {
        let mut ui = WorkbenchUi::new(StationType::Forge);
        let item = CraftingItem::new("sword", "Iron Sword");

        ui.add_output(item);
        assert_eq!(ui.output_items.len(), 1);

        let outputs = ui.take_outputs();
        assert_eq!(outputs.len(), 1);
        assert!(ui.output_items.is_empty());
    }

    #[test]
    fn test_workbench_config_defaults() {
        let config = WorkbenchConfig::default();
        assert!(config.show_fuel);
        assert!(config.show_heat);
        assert!(config.show_queue);
        assert_eq!(config.flask_slots, 3);
    }

    #[test]
    fn test_workbench_action_equality() {
        let action1 = WorkbenchAction::AddFuel(FuelType::Coal, 5);
        let action2 = WorkbenchAction::AddFuel(FuelType::Coal, 5);
        assert_eq!(action1, action2);
    }

    #[test]
    fn test_fuel_gauge_serialization() {
        let mut gauge = FuelGauge::new(100.0);
        gauge.add_fuel(50.0, FuelType::Coal);

        let json = serde_json::to_string(&gauge).unwrap();
        let loaded: FuelGauge = serde_json::from_str(&json).unwrap();

        assert_eq!(gauge.current, loaded.current);
    }

    #[test]
    fn test_heat_level_serialization() {
        let mut heat = HeatLevel::new();
        heat.current = 0.7;
        heat.target = 0.8;

        let json = serde_json::to_string(&heat).unwrap();
        let loaded: HeatLevel = serde_json::from_str(&json).unwrap();

        assert_eq!(heat.current, loaded.current);
        assert_eq!(heat.target, loaded.target);
    }

    #[test]
    fn test_crafting_job_serialization() {
        let job = CraftingJob::new(RecipeId::new("sword"), "Sword", 10.0, 2);

        let json = serde_json::to_string(&job).unwrap();
        let loaded: CraftingJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.name, loaded.name);
        assert_eq!(job.quantity, loaded.quantity);
    }

    #[test]
    fn test_station_type_serialization() {
        for station in StationType::all() {
            let json = serde_json::to_string(station).unwrap();
            let loaded: StationType = serde_json::from_str(&json).unwrap();
            assert_eq!(*station, loaded);
        }
    }

    #[test]
    fn test_workbench_config_serialization() {
        let config = WorkbenchConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: WorkbenchConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.flask_slots, loaded.flask_slots);
    }

    #[test]
    fn test_workbench_drain_actions() {
        let mut ui = WorkbenchUi::new(StationType::Forge);
        ui.pending_actions.push(WorkbenchAction::CancelJob);

        let actions = ui.drain_actions();
        assert_eq!(actions.len(), 1);

        let actions2 = ui.drain_actions();
        assert!(actions2.is_empty());
    }

    #[test]
    fn test_heat_color() {
        let mut heat = HeatLevel::new();

        heat.current = 0.1;
        let cold_color = heat.color();

        heat.current = 0.9;
        let hot_color = heat.color();

        // Colors should be different for different temperatures
        assert_ne!(cold_color, hot_color);
    }

    #[test]
    fn test_workbench_with_config() {
        let config = WorkbenchConfig {
            flask_slots: 5,
            ..WorkbenchConfig::default()
        };
        let ui = WorkbenchUi::with_config(StationType::Alchemy, config);

        assert_eq!(ui.flasks.len(), 5);
        assert_eq!(ui.station_type, StationType::Alchemy);
    }
}
