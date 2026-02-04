//! Crafting UI renderer using egui.
//!
//! This module provides:
//! - Recipe list with filtering
//! - Material cost display with availability check
//! - Craft queue with progress bars
//! - Category tabs/filters
//! - Recipe search

use egui::{
    Align, Color32, Context, Layout, ProgressBar, Response, RichText, Rounding, Sense, Stroke,
    TextEdit, Ui, Vec2, Window,
};
use genesis_common::{ItemTypeId, RecipeId};
use serde::{Deserialize, Serialize};

/// Default craft time in seconds.
pub const DEFAULT_CRAFT_TIME: f32 = 2.0;

/// Recipe categories for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RecipeCategory {
    /// All recipes
    #[default]
    All,
    /// Tools and equipment
    Tools,
    /// Weapons
    Weapons,
    /// Building materials
    Building,
    /// Food and consumables
    Consumables,
    /// Misc/Other
    Misc,
}

impl RecipeCategory {
    /// Returns all category variants.
    #[must_use]
    pub fn all() -> &'static [RecipeCategory] {
        &[
            RecipeCategory::All,
            RecipeCategory::Tools,
            RecipeCategory::Weapons,
            RecipeCategory::Building,
            RecipeCategory::Consumables,
            RecipeCategory::Misc,
        ]
    }

    /// Returns the display name for this category.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            RecipeCategory::All => "All",
            RecipeCategory::Tools => "Tools",
            RecipeCategory::Weapons => "Weapons",
            RecipeCategory::Building => "Building",
            RecipeCategory::Consumables => "Consumables",
            RecipeCategory::Misc => "Misc",
        }
    }
}

/// Material cost for a recipe.
#[derive(Debug, Clone)]
pub struct MaterialCost {
    /// Item type ID
    pub item_type: ItemTypeId,
    /// Display name
    pub name: String,
    /// Required quantity
    pub required: u32,
    /// Available in inventory
    pub available: u32,
    /// Icon color (fallback)
    pub icon_color: [u8; 4],
}

impl Default for MaterialCost {
    fn default() -> Self {
        Self {
            item_type: ItemTypeId::new(0),
            name: String::new(),
            required: 0,
            available: 0,
            icon_color: [200, 200, 200, 255],
        }
    }
}

impl MaterialCost {
    /// Creates a new material cost.
    #[must_use]
    pub fn new(
        item_type: ItemTypeId,
        name: impl Into<String>,
        required: u32,
        available: u32,
    ) -> Self {
        Self {
            item_type,
            name: name.into(),
            required,
            available,
            icon_color: [200, 200, 200, 255],
        }
    }

    /// Returns whether we have enough of this material.
    #[must_use]
    pub fn has_enough(&self) -> bool {
        self.available >= self.required
    }
}

/// Recipe card UI data.
#[derive(Debug, Clone)]
pub struct RecipeCard {
    /// Recipe ID
    pub recipe_id: RecipeId,
    /// Recipe name
    pub name: String,
    /// Category
    pub category: RecipeCategory,
    /// Description
    pub description: String,
    /// Material costs
    pub materials: Vec<MaterialCost>,
    /// Tool requirements (display names)
    pub tools: Vec<String>,
    /// Output item name
    pub output_name: String,
    /// Output quantity
    pub output_quantity: u32,
    /// Craft time in seconds
    pub craft_time: f32,
    /// Required skill level (0 = none)
    pub skill_required: u32,
    /// Icon color (fallback)
    pub icon_color: [u8; 4],
    /// Whether this recipe is unlocked
    pub is_unlocked: bool,
}

impl Default for RecipeCard {
    fn default() -> Self {
        Self {
            recipe_id: RecipeId::new(0),
            name: String::new(),
            category: RecipeCategory::Misc,
            description: String::new(),
            materials: Vec::new(),
            tools: Vec::new(),
            output_name: String::new(),
            output_quantity: 1,
            craft_time: DEFAULT_CRAFT_TIME,
            skill_required: 0,
            icon_color: [200, 200, 200, 255],
            is_unlocked: true,
        }
    }
}

impl RecipeCard {
    /// Creates a new recipe card.
    #[must_use]
    pub fn new(recipe_id: RecipeId, name: impl Into<String>) -> Self {
        Self {
            recipe_id,
            name: name.into(),
            category: RecipeCategory::Misc,
            description: String::new(),
            materials: Vec::new(),
            tools: Vec::new(),
            output_name: String::new(),
            output_quantity: 1,
            craft_time: DEFAULT_CRAFT_TIME,
            skill_required: 0,
            icon_color: [200, 200, 200, 255],
            is_unlocked: true,
        }
    }

    /// Returns whether the recipe can be crafted (all materials available).
    #[must_use]
    pub fn can_craft(&self) -> bool {
        self.is_unlocked && self.materials.iter().all(MaterialCost::has_enough)
    }

    /// Returns the number of missing materials.
    #[must_use]
    pub fn missing_material_count(&self) -> usize {
        self.materials.iter().filter(|m| !m.has_enough()).count()
    }
}

/// An item in the crafting queue.
#[derive(Debug, Clone)]
pub struct QueueItem {
    /// Recipe ID
    pub recipe_id: RecipeId,
    /// Recipe name (for display)
    pub recipe_name: String,
    /// Total craft time
    pub craft_time: f32,
    /// Progress (0.0 - 1.0)
    pub progress: f32,
    /// Whether this is currently being crafted
    pub is_active: bool,
}

impl QueueItem {
    /// Creates a new queue item.
    #[must_use]
    pub fn new(recipe_id: RecipeId, name: impl Into<String>, craft_time: f32) -> Self {
        Self {
            recipe_id,
            recipe_name: name.into(),
            craft_time,
            progress: 0.0,
            is_active: false,
        }
    }

    /// Returns remaining time in seconds.
    #[must_use]
    pub fn remaining_time(&self) -> f32 {
        self.craft_time * (1.0 - self.progress)
    }
}

/// Crafting UI model containing all display data.
#[derive(Debug, Clone, Default)]
pub struct CraftingUIModel {
    /// Available recipes
    pub recipes: Vec<RecipeCard>,
    /// Crafting queue
    pub queue: Vec<QueueItem>,
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Whether crafting UI is open
    pub is_open: bool,
}

impl CraftingUIModel {
    /// Creates a new empty crafting model.
    #[must_use]
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            recipes: Vec::new(),
            queue: Vec::new(),
            max_queue_size,
            is_open: false,
        }
    }

    /// Creates with default settings.
    #[must_use]
    pub fn default_sized() -> Self {
        Self::new(5)
    }

    /// Returns whether the queue is full.
    #[must_use]
    pub fn queue_full(&self) -> bool {
        self.queue.len() >= self.max_queue_size
    }

    /// Returns recipes filtered by category and search.
    #[must_use]
    pub fn filtered_recipes(&self, category: RecipeCategory, search: &str) -> Vec<&RecipeCard> {
        self.recipes
            .iter()
            .filter(|r| category == RecipeCategory::All || r.category == category)
            .filter(|r| search.is_empty() || r.name.to_lowercase().contains(&search.to_lowercase()))
            .collect()
    }
}

/// Actions that can be performed on the crafting UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CraftingAction {
    /// Craft a recipe
    Craft(RecipeId),
    /// Cancel a queued craft
    CancelQueue(usize),
    /// Open crafting UI
    Open,
    /// Close crafting UI
    Close,
    /// Select category filter
    SelectCategory(RecipeCategory),
}

/// Configuration for the crafting UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingUIConfig {
    /// Width of the recipe list panel
    pub recipe_list_width: f32,
    /// Height of recipe cards
    pub recipe_card_height: f32,
    /// Progress bar height
    pub progress_bar_height: f32,
    /// Background color
    pub background_color: [u8; 4],
    /// Craftable recipe color
    pub craftable_color: [u8; 4],
    /// Uncraftable recipe color
    pub uncraftable_color: [u8; 4],
    /// Selected recipe color
    pub selected_color: [u8; 4],
}

impl Default for CraftingUIConfig {
    fn default() -> Self {
        Self {
            recipe_list_width: 300.0,
            recipe_card_height: 60.0,
            progress_bar_height: 20.0,
            background_color: [40, 40, 40, 220],
            craftable_color: [60, 80, 60, 255],
            uncraftable_color: [80, 60, 60, 255],
            selected_color: [80, 100, 120, 255],
        }
    }
}

/// Crafting UI renderer.
#[derive(Debug)]
pub struct CraftingUI {
    /// Whether the crafting window is open
    pub is_open: bool,
    /// Configuration
    pub config: CraftingUIConfig,
    /// Current category filter
    pub selected_category: RecipeCategory,
    /// Current search text
    pub search_text: String,
    /// Currently selected recipe index
    pub selected_recipe: Option<usize>,
}

impl Default for CraftingUI {
    fn default() -> Self {
        Self::new()
    }
}

impl CraftingUI {
    /// Creates a new crafting UI.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_open: false,
            config: CraftingUIConfig::default(),
            selected_category: RecipeCategory::All,
            search_text: String::new(),
            selected_recipe: None,
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: CraftingUIConfig) -> Self {
        Self {
            is_open: false,
            config,
            selected_category: RecipeCategory::All,
            search_text: String::new(),
            selected_recipe: None,
        }
    }

    /// Shows the crafting UI and returns any actions.
    pub fn show(&mut self, ctx: &Context, model: &mut CraftingUIModel) -> Vec<CraftingAction> {
        let mut actions = Vec::new();

        // Sync open state
        self.is_open = model.is_open;

        if !self.is_open {
            return actions;
        }

        Window::new("Crafting")
            .resizable(true)
            .collapsible(false)
            .default_width(600.0)
            .show(ctx, |ui| {
                // Category tabs
                ui.horizontal(|ui| {
                    for category in RecipeCategory::all() {
                        let selected = self.selected_category == *category;
                        let btn = ui.selectable_label(selected, category.display_name());
                        if btn.clicked() {
                            self.selected_category = *category;
                            actions.push(CraftingAction::SelectCategory(*category));
                        }
                    }
                });

                // Search bar
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.add(TextEdit::singleline(&mut self.search_text).desired_width(200.0));
                    if ui.button("Clear").clicked() {
                        self.search_text.clear();
                    }
                });

                ui.separator();

                // Main content: recipe list + details panel
                ui.horizontal(|ui| {
                    // Recipe list (left panel)
                    egui::ScrollArea::vertical()
                        .id_salt("recipe_list")
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.set_min_width(self.config.recipe_list_width);
                            let filtered =
                                model.filtered_recipes(self.selected_category, &self.search_text);
                            for (idx, recipe) in filtered.iter().enumerate() {
                                let is_selected = self.selected_recipe == Some(idx);
                                let response = self.render_recipe_card(ui, recipe, is_selected);

                                if response.clicked() {
                                    self.selected_recipe = Some(idx);
                                }

                                if response.double_clicked()
                                    && recipe.can_craft()
                                    && !model.queue_full()
                                {
                                    actions.push(CraftingAction::Craft(recipe.recipe_id));
                                }
                            }
                        });

                    ui.separator();

                    // Details panel (right)
                    ui.vertical(|ui| {
                        ui.set_min_width(250.0);
                        if let Some(idx) = self.selected_recipe {
                            let filtered =
                                model.filtered_recipes(self.selected_category, &self.search_text);
                            if let Some(recipe) = filtered.get(idx) {
                                self.render_recipe_details(ui, recipe, model, &mut actions);
                            }
                        } else {
                            ui.label("Select a recipe");
                        }
                    });
                });

                ui.separator();

                // Crafting queue
                self.render_queue(ui, model, &mut actions);
            });

        actions
    }

    /// Renders a recipe card in the list.
    fn render_recipe_card(&self, ui: &mut Ui, recipe: &RecipeCard, is_selected: bool) -> Response {
        let available_width = ui.available_width();
        let height = self.config.recipe_card_height;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(available_width, height), Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background color based on state
            let bg_color = if is_selected {
                Color32::from_rgba_unmultiplied(
                    self.config.selected_color[0],
                    self.config.selected_color[1],
                    self.config.selected_color[2],
                    self.config.selected_color[3],
                )
            } else if recipe.can_craft() {
                Color32::from_rgba_unmultiplied(
                    self.config.craftable_color[0],
                    self.config.craftable_color[1],
                    self.config.craftable_color[2],
                    self.config.craftable_color[3],
                )
            } else {
                Color32::from_rgba_unmultiplied(
                    self.config.uncraftable_color[0],
                    self.config.uncraftable_color[1],
                    self.config.uncraftable_color[2],
                    self.config.uncraftable_color[3],
                )
            };

            painter.rect_filled(rect, Rounding::same(4.0), bg_color);

            // Border on hover
            if response.hovered() {
                painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0, Color32::WHITE));
            }

            // Icon (placeholder)
            let icon_rect = egui::Rect::from_min_size(
                rect.left_top() + Vec2::new(8.0, 8.0),
                Vec2::splat(height - 16.0),
            );
            let icon_color = Color32::from_rgba_unmultiplied(
                recipe.icon_color[0],
                recipe.icon_color[1],
                recipe.icon_color[2],
                recipe.icon_color[3],
            );
            painter.rect_filled(icon_rect, Rounding::same(2.0), icon_color);

            // Recipe name
            let text_pos = rect.left_top() + Vec2::new(height + 4.0, 8.0);
            let name_color = if recipe.can_craft() {
                Color32::WHITE
            } else {
                Color32::GRAY
            };
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                &recipe.name,
                egui::FontId::proportional(14.0),
                name_color,
            );

            // Output info
            let output_text = format!("→ {}x {}", recipe.output_quantity, recipe.output_name);
            painter.text(
                rect.left_top() + Vec2::new(height + 4.0, 26.0),
                egui::Align2::LEFT_TOP,
                output_text,
                egui::FontId::proportional(11.0),
                Color32::LIGHT_GRAY,
            );

            // Craft time
            let time_text = format!("{:.1}s", recipe.craft_time);
            painter.text(
                rect.right_top() + Vec2::new(-8.0, 8.0),
                egui::Align2::RIGHT_TOP,
                time_text,
                egui::FontId::proportional(10.0),
                Color32::GRAY,
            );

            // Lock icon if not unlocked
            if !recipe.is_unlocked {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔒",
                    egui::FontId::proportional(20.0),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 180),
                );
            }
        }

        response
    }

    /// Renders the recipe details panel.
    fn render_recipe_details(
        &mut self,
        ui: &mut Ui,
        recipe: &RecipeCard,
        model: &CraftingUIModel,
        actions: &mut Vec<CraftingAction>,
    ) {
        // Track the currently viewed recipe
        let _ = &self.config;

        ui.heading(&recipe.name);
        ui.label(&recipe.description);

        ui.separator();

        // Materials
        ui.label(RichText::new("Materials:").strong());
        for mat in &recipe.materials {
            let color = if mat.has_enough() {
                Color32::GREEN
            } else {
                Color32::RED
            };
            ui.horizontal(|ui| {
                ui.colored_label(color, format!("{}/{}", mat.available, mat.required));
                ui.label(&mat.name);
            });
        }

        // Tools
        if !recipe.tools.is_empty() {
            ui.separator();
            ui.label(RichText::new("Required Tools:").strong());
            for tool in &recipe.tools {
                ui.label(format!("• {tool}"));
            }
        }

        // Skill requirement
        if recipe.skill_required > 0 {
            ui.separator();
            ui.label(format!("Skill Required: {}", recipe.skill_required));
        }

        ui.separator();

        // Output
        ui.label(RichText::new("Output:").strong());
        ui.label(format!(
            "{}x {}",
            recipe.output_quantity, recipe.output_name
        ));

        ui.separator();

        // Craft button
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let can_craft = recipe.can_craft() && !model.queue_full();
            let btn = ui.add_enabled(can_craft, egui::Button::new("Craft"));
            if btn.clicked() {
                actions.push(CraftingAction::Craft(recipe.recipe_id));
            }

            if model.queue_full() {
                ui.label(RichText::new("Queue full").color(Color32::YELLOW));
            } else if !recipe.can_craft() {
                let missing = recipe.missing_material_count();
                ui.label(RichText::new(format!("{missing} materials missing")).color(Color32::RED));
            }
        });
    }

    /// Renders the crafting queue.
    fn render_queue(
        &self,
        ui: &mut Ui,
        model: &CraftingUIModel,
        actions: &mut Vec<CraftingAction>,
    ) {
        ui.label(RichText::new("Crafting Queue:").strong());

        if model.queue.is_empty() {
            ui.label("Queue empty");
            return;
        }

        for (idx, item) in model.queue.iter().enumerate() {
            ui.horizontal(|ui| {
                // Cancel button
                if ui.small_button("✕").clicked() {
                    actions.push(CraftingAction::CancelQueue(idx));
                }

                // Recipe name
                ui.label(&item.recipe_name);

                // Progress bar
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let remaining = item.remaining_time();
                    ui.label(format!("{remaining:.1}s"));

                    let bar_width = 100.0;
                    let text = if item.is_active {
                        "Crafting..."
                    } else {
                        "Queued"
                    };
                    ui.add_sized(
                        Vec2::new(bar_width, self.config.progress_bar_height),
                        ProgressBar::new(item.progress).text(text),
                    );
                });
            });
        }

        // Queue capacity
        ui.label(format!(
            "{}/{} slots",
            model.queue.len(),
            model.max_queue_size
        ));
    }

    /// Toggles the crafting UI open/closed.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Opens the crafting UI.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the crafting UI.
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_category_all() {
        let categories = RecipeCategory::all();
        assert_eq!(categories.len(), 6);
        assert!(categories.contains(&RecipeCategory::All));
        assert!(categories.contains(&RecipeCategory::Tools));
    }

    #[test]
    fn test_recipe_category_display_name() {
        assert_eq!(RecipeCategory::All.display_name(), "All");
        assert_eq!(RecipeCategory::Weapons.display_name(), "Weapons");
        assert_eq!(RecipeCategory::Building.display_name(), "Building");
    }

    #[test]
    fn test_material_cost_new() {
        let mat = MaterialCost::new(ItemTypeId::new(1), "Wood", 5, 10);
        assert_eq!(mat.required, 5);
        assert_eq!(mat.available, 10);
        assert!(mat.has_enough());
    }

    #[test]
    fn test_material_cost_not_enough() {
        let mat = MaterialCost::new(ItemTypeId::new(1), "Stone", 10, 5);
        assert!(!mat.has_enough());
    }

    #[test]
    fn test_recipe_card_new() {
        let card = RecipeCard::new(RecipeId::new(1), "Iron Sword");
        assert_eq!(card.name, "Iron Sword");
        assert_eq!(card.category, RecipeCategory::Misc);
        assert!(card.is_unlocked);
    }

    #[test]
    fn test_recipe_card_can_craft() {
        let mut card = RecipeCard::new(RecipeId::new(1), "Test Recipe");
        card.materials
            .push(MaterialCost::new(ItemTypeId::new(1), "A", 1, 5));
        assert!(card.can_craft());

        card.materials
            .push(MaterialCost::new(ItemTypeId::new(2), "B", 10, 1));
        assert!(!card.can_craft());
    }

    #[test]
    fn test_recipe_card_locked() {
        let mut card = RecipeCard::new(RecipeId::new(1), "Secret Recipe");
        card.is_unlocked = false;
        assert!(!card.can_craft());
    }

    #[test]
    fn test_recipe_card_missing_count() {
        let mut card = RecipeCard::new(RecipeId::new(1), "Test");
        card.materials
            .push(MaterialCost::new(ItemTypeId::new(1), "A", 1, 5));
        card.materials
            .push(MaterialCost::new(ItemTypeId::new(2), "B", 10, 1));
        card.materials
            .push(MaterialCost::new(ItemTypeId::new(3), "C", 5, 0));
        assert_eq!(card.missing_material_count(), 2);
    }

    #[test]
    fn test_queue_item_new() {
        let item = QueueItem::new(RecipeId::new(1), "Test", 5.0);
        assert_eq!(item.craft_time, 5.0);
        assert_eq!(item.progress, 0.0);
        assert!(!item.is_active);
    }

    #[test]
    fn test_queue_item_remaining_time() {
        let mut item = QueueItem::new(RecipeId::new(1), "Test", 10.0);
        assert!((item.remaining_time() - 10.0).abs() < 0.001);

        item.progress = 0.5;
        assert!((item.remaining_time() - 5.0).abs() < 0.001);

        item.progress = 1.0;
        assert!((item.remaining_time() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_crafting_ui_model_new() {
        let model = CraftingUIModel::new(10);
        assert_eq!(model.max_queue_size, 10);
        assert!(model.recipes.is_empty());
        assert!(model.queue.is_empty());
    }

    #[test]
    fn test_crafting_ui_model_default_sized() {
        let model = CraftingUIModel::default_sized();
        assert_eq!(model.max_queue_size, 5);
    }

    #[test]
    fn test_crafting_ui_model_queue_full() {
        let mut model = CraftingUIModel::new(2);
        assert!(!model.queue_full());

        model.queue.push(QueueItem::new(RecipeId::new(1), "A", 1.0));
        assert!(!model.queue_full());

        model.queue.push(QueueItem::new(RecipeId::new(2), "B", 1.0));
        assert!(model.queue_full());
    }

    #[test]
    fn test_crafting_ui_model_filtered_recipes() {
        let mut model = CraftingUIModel::new(5);

        let mut r1 = RecipeCard::new(RecipeId::new(1), "Iron Sword");
        r1.category = RecipeCategory::Weapons;
        model.recipes.push(r1);

        let mut r2 = RecipeCard::new(RecipeId::new(2), "Wooden Pickaxe");
        r2.category = RecipeCategory::Tools;
        model.recipes.push(r2);

        let mut r3 = RecipeCard::new(RecipeId::new(3), "Stone Axe");
        r3.category = RecipeCategory::Tools;
        model.recipes.push(r3);

        // All recipes
        let all = model.filtered_recipes(RecipeCategory::All, "");
        assert_eq!(all.len(), 3);

        // Tools only
        let tools = model.filtered_recipes(RecipeCategory::Tools, "");
        assert_eq!(tools.len(), 2);

        // Search
        let search = model.filtered_recipes(RecipeCategory::All, "stone");
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].name, "Stone Axe");
    }

    #[test]
    fn test_crafting_action_equality() {
        assert_eq!(CraftingAction::Open, CraftingAction::Open);
        assert_eq!(
            CraftingAction::Craft(RecipeId::new(1)),
            CraftingAction::Craft(RecipeId::new(1))
        );
        assert_ne!(
            CraftingAction::Craft(RecipeId::new(1)),
            CraftingAction::Craft(RecipeId::new(2))
        );
    }

    #[test]
    fn test_crafting_ui_config_defaults() {
        let config = CraftingUIConfig::default();
        assert_eq!(config.recipe_list_width, 300.0);
        assert_eq!(config.recipe_card_height, 60.0);
    }

    #[test]
    fn test_crafting_ui_new() {
        let ui = CraftingUI::new();
        assert!(!ui.is_open);
        assert_eq!(ui.selected_category, RecipeCategory::All);
        assert!(ui.search_text.is_empty());
    }

    #[test]
    fn test_crafting_ui_toggle() {
        let mut ui = CraftingUI::new();
        assert!(!ui.is_open);
        ui.toggle();
        assert!(ui.is_open);
        ui.toggle();
        assert!(!ui.is_open);
    }

    #[test]
    fn test_crafting_ui_open_close() {
        let mut ui = CraftingUI::new();
        ui.open();
        assert!(ui.is_open);
        ui.close();
        assert!(!ui.is_open);
    }
}
