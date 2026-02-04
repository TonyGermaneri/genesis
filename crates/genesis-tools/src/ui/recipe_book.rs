//! Recipe book UI for browsing and discovering crafting recipes.
//!
//! Provides a categorized recipe browser with:
//! - Category-based organization
//! - Search and filter by name or ingredient
//! - Required materials display
//! - Craftable vs locked recipe highlighting

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};

use super::crafting_grid::{CraftingItemId, ItemRarity};

/// Unique identifier for a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeId(pub String);

impl RecipeId {
    /// Create a new recipe ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for RecipeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Recipe category for organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RecipeCategory {
    /// All recipes.
    #[default]
    All,
    /// Weapon recipes.
    Weapons,
    /// Armor recipes.
    Armor,
    /// Tool recipes.
    Tools,
    /// Building material recipes.
    Building,
    /// Consumable recipes (food, potions).
    Consumables,
    /// Decoration recipes.
    Decorations,
    /// Miscellaneous recipes.
    Misc,
}

impl RecipeCategory {
    /// Get all categories.
    pub fn all() -> &'static [RecipeCategory] {
        &[
            RecipeCategory::All,
            RecipeCategory::Weapons,
            RecipeCategory::Armor,
            RecipeCategory::Tools,
            RecipeCategory::Building,
            RecipeCategory::Consumables,
            RecipeCategory::Decorations,
            RecipeCategory::Misc,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            RecipeCategory::All => "All",
            RecipeCategory::Weapons => "Weapons",
            RecipeCategory::Armor => "Armor",
            RecipeCategory::Tools => "Tools",
            RecipeCategory::Building => "Building",
            RecipeCategory::Consumables => "Consumables",
            RecipeCategory::Decorations => "Decorations",
            RecipeCategory::Misc => "Misc",
        }
    }

    /// Get icon.
    pub fn icon(&self) -> &'static str {
        match self {
            RecipeCategory::All => "📋",
            RecipeCategory::Weapons => "⚔",
            RecipeCategory::Armor => "🛡",
            RecipeCategory::Tools => "🔧",
            RecipeCategory::Building => "🏠",
            RecipeCategory::Consumables => "🧪",
            RecipeCategory::Decorations => "🎨",
            RecipeCategory::Misc => "📦",
        }
    }
}

/// An ingredient required for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    /// Item ID required.
    pub item_id: CraftingItemId,
    /// Display name.
    pub name: String,
    /// Quantity required.
    pub quantity: u32,
    /// Current quantity available (for display).
    pub available: u32,
}

impl RecipeIngredient {
    /// Create a new ingredient requirement.
    pub fn new(item_id: impl Into<String>, name: impl Into<String>, quantity: u32) -> Self {
        Self {
            item_id: CraftingItemId::new(item_id),
            name: name.into(),
            quantity,
            available: 0,
        }
    }

    /// Set available quantity.
    pub fn with_available(mut self, available: u32) -> Self {
        self.available = available;
        self
    }

    /// Check if enough materials are available.
    pub fn has_enough(&self) -> bool {
        self.available >= self.quantity
    }

    /// Get missing quantity.
    pub fn missing(&self) -> u32 {
        self.quantity.saturating_sub(self.available)
    }
}

/// Output item from a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOutput {
    /// Item ID produced.
    pub item_id: CraftingItemId,
    /// Display name.
    pub name: String,
    /// Quantity produced.
    pub quantity: u32,
    /// Item rarity.
    pub rarity: ItemRarity,
    /// Item description.
    pub description: String,
}

impl RecipeOutput {
    /// Create a new recipe output.
    pub fn new(item_id: impl Into<String>, name: impl Into<String>, quantity: u32) -> Self {
        Self {
            item_id: CraftingItemId::new(item_id),
            name: name.into(),
            quantity,
            rarity: ItemRarity::Common,
            description: String::new(),
        }
    }

    /// Set rarity.
    pub fn with_rarity(mut self, rarity: ItemRarity) -> Self {
        self.rarity = rarity;
        self
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Recipe unlock status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RecipeStatus {
    /// Recipe is locked and not discovered.
    #[default]
    Locked,
    /// Recipe is discovered but requirements not met.
    Discovered,
    /// Recipe can be crafted.
    Craftable,
    /// Recipe is a favorite.
    Favorite,
}

impl RecipeStatus {
    /// Get status color.
    pub fn color(&self) -> Color32 {
        match self {
            RecipeStatus::Locked => Color32::from_gray(80),
            RecipeStatus::Discovered => Color32::from_rgb(150, 150, 150),
            RecipeStatus::Craftable => Color32::from_rgb(100, 200, 100),
            RecipeStatus::Favorite => Color32::from_rgb(255, 200, 100),
        }
    }

    /// Get status icon.
    pub fn icon(&self) -> &'static str {
        match self {
            RecipeStatus::Locked => "🔒",
            RecipeStatus::Discovered => "📖",
            RecipeStatus::Craftable => "✓",
            RecipeStatus::Favorite => "★",
        }
    }
}

/// A crafting recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe ID.
    pub id: RecipeId,
    /// Display name.
    pub name: String,
    /// Recipe category.
    pub category: RecipeCategory,
    /// Required ingredients.
    pub ingredients: Vec<RecipeIngredient>,
    /// Output item.
    pub output: RecipeOutput,
    /// Crafting time in seconds.
    pub craft_time: f32,
    /// Required crafting station (None = hand crafting).
    pub station: Option<String>,
    /// Recipe status.
    pub status: RecipeStatus,
    /// Search tags.
    pub tags: Vec<String>,
}

impl Recipe {
    /// Create a new recipe.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: RecipeCategory,
        output: RecipeOutput,
    ) -> Self {
        Self {
            id: RecipeId::new(id),
            name: name.into(),
            category,
            ingredients: Vec::new(),
            output,
            craft_time: 1.0,
            station: None,
            status: RecipeStatus::Locked,
            tags: Vec::new(),
        }
    }

    /// Add an ingredient.
    pub fn with_ingredient(mut self, ingredient: RecipeIngredient) -> Self {
        self.ingredients.push(ingredient);
        self
    }

    /// Set craft time.
    pub fn with_craft_time(mut self, time: f32) -> Self {
        self.craft_time = time;
        self
    }

    /// Set required station.
    pub fn with_station(mut self, station: impl Into<String>) -> Self {
        self.station = Some(station.into());
        self
    }

    /// Set status.
    pub fn with_status(mut self, status: RecipeStatus) -> Self {
        self.status = status;
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Check if all ingredients are available.
    pub fn can_craft(&self) -> bool {
        self.status != RecipeStatus::Locked
            && self.ingredients.iter().all(RecipeIngredient::has_enough)
    }

    /// Check if recipe matches search query.
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.output.name.to_lowercase().contains(&query_lower)
            || self
                .ingredients
                .iter()
                .any(|i| i.name.to_lowercase().contains(&query_lower))
            || self
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
    }

    /// Check if recipe matches category filter.
    pub fn matches_category(&self, category: RecipeCategory) -> bool {
        category == RecipeCategory::All || self.category == category
    }

    /// Update ingredient availability.
    pub fn update_availability(&mut self, inventory: &[(CraftingItemId, u32)]) {
        for ingredient in &mut self.ingredients {
            ingredient.available = inventory
                .iter()
                .find(|(id, _)| id == &ingredient.item_id)
                .map_or(0, |(_, count)| *count);
        }
    }
}

/// Actions returned by the recipe book.
#[derive(Debug, Clone, PartialEq)]
pub enum RecipeBookAction {
    /// Recipe selected for viewing.
    RecipeSelected(RecipeId),
    /// Recipe double-clicked to craft.
    RecipeCraft(RecipeId),
    /// Recipe favorited/unfavorited.
    RecipeToggleFavorite(RecipeId),
    /// Category changed.
    CategoryChanged(RecipeCategory),
    /// Search query changed.
    SearchChanged(String),
}

/// Configuration for the recipe book UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeBookConfig {
    /// Show category tabs.
    pub show_categories: bool,
    /// Show search bar.
    pub show_search: bool,
    /// Show ingredient details.
    pub show_ingredients: bool,
    /// Show locked recipes.
    pub show_locked: bool,
    /// Recipe list height.
    pub list_height: f32,
    /// Recipe card height.
    pub card_height: f32,
}

impl Default for RecipeBookConfig {
    fn default() -> Self {
        Self {
            show_categories: true,
            show_search: true,
            show_ingredients: true,
            show_locked: true,
            list_height: 300.0,
            card_height: 60.0,
        }
    }
}

/// Recipe book widget.
#[derive(Debug)]
pub struct RecipeBook {
    /// Available recipes.
    recipes: Vec<Recipe>,
    /// Configuration.
    pub config: RecipeBookConfig,
    /// Current category filter.
    pub category_filter: RecipeCategory,
    /// Current search query.
    pub search_query: String,
    /// Currently selected recipe.
    pub selected_recipe: Option<RecipeId>,
    /// Whether the book is open.
    pub open: bool,
    /// Pending actions.
    pending_actions: Vec<RecipeBookAction>,
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeBook {
    /// Create a new recipe book.
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
            config: RecipeBookConfig::default(),
            category_filter: RecipeCategory::All,
            search_query: String::new(),
            selected_recipe: None,
            open: false,
            pending_actions: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: RecipeBookConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Open the recipe book.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the recipe book.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggle visibility.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Add a recipe.
    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Set all recipes.
    pub fn set_recipes(&mut self, recipes: Vec<Recipe>) {
        self.recipes = recipes;
    }

    /// Get a recipe by ID.
    pub fn get_recipe(&self, id: &RecipeId) -> Option<&Recipe> {
        self.recipes.iter().find(|r| &r.id == id)
    }

    /// Get a mutable recipe by ID.
    pub fn get_recipe_mut(&mut self, id: &RecipeId) -> Option<&mut Recipe> {
        self.recipes.iter_mut().find(|r| &r.id == id)
    }

    /// Get filtered recipes.
    pub fn filtered_recipes(&self) -> Vec<&Recipe> {
        self.recipes
            .iter()
            .filter(|r| r.matches_category(self.category_filter))
            .filter(|r| self.search_query.is_empty() || r.matches_search(&self.search_query))
            .filter(|r| self.config.show_locked || r.status != RecipeStatus::Locked)
            .collect()
    }

    /// Get recipe count by category.
    pub fn count_by_category(&self, category: RecipeCategory) -> usize {
        self.recipes
            .iter()
            .filter(|r| r.matches_category(category))
            .filter(|r| self.config.show_locked || r.status != RecipeStatus::Locked)
            .count()
    }

    /// Get craftable recipe count.
    pub fn craftable_count(&self) -> usize {
        self.recipes.iter().filter(|r| r.can_craft()).count()
    }

    /// Update all recipe availability.
    pub fn update_availability(&mut self, inventory: &[(CraftingItemId, u32)]) {
        for recipe in &mut self.recipes {
            recipe.update_availability(inventory);
        }
    }

    /// Unlock a recipe.
    pub fn unlock_recipe(&mut self, id: &RecipeId) {
        if let Some(recipe) = self.get_recipe_mut(id) {
            if recipe.status == RecipeStatus::Locked {
                recipe.status = RecipeStatus::Discovered;
            }
        }
    }

    /// Toggle recipe favorite status.
    pub fn toggle_favorite(&mut self, id: &RecipeId) {
        if let Some(recipe) = self.get_recipe_mut(id) {
            recipe.status = match recipe.status {
                RecipeStatus::Favorite => RecipeStatus::Discovered,
                RecipeStatus::Discovered | RecipeStatus::Craftable => RecipeStatus::Favorite,
                RecipeStatus::Locked => RecipeStatus::Locked, // Can't favorite locked
            };
        }
    }

    /// Render the recipe book and return actions.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<RecipeBookAction> {
        self.pending_actions.clear();

        if !self.open {
            return Vec::new();
        }

        ui.vertical(|ui| {
            ui.heading("📖 Recipe Book");
            ui.separator();

            // Search bar
            if self.config.show_search {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    let response = ui.text_edit_singleline(&mut self.search_query);
                    if response.changed() {
                        self.pending_actions
                            .push(RecipeBookAction::SearchChanged(self.search_query.clone()));
                    }
                    if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                        self.search_query.clear();
                        self.pending_actions
                            .push(RecipeBookAction::SearchChanged(String::new()));
                    }
                });
            }

            // Category tabs
            if self.config.show_categories {
                self.show_category_tabs(ui);
            }

            ui.separator();

            // Recipe count
            let filtered = self.filtered_recipes();
            ui.label(format!(
                "Recipes: {} ({} craftable)",
                filtered.len(),
                self.craftable_count()
            ));

            // Recipe list
            self.show_recipe_list(ui);
        });

        std::mem::take(&mut self.pending_actions)
    }

    /// Show category filter tabs.
    fn show_category_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for category in RecipeCategory::all() {
                let count = self.count_by_category(*category);
                let selected = self.category_filter == *category;
                let text = format!(
                    "{} {} ({})",
                    category.icon(),
                    category.display_name(),
                    count
                );

                if ui.selectable_label(selected, text).clicked() {
                    self.category_filter = *category;
                    self.pending_actions
                        .push(RecipeBookAction::CategoryChanged(*category));
                }
            }
        });
    }

    /// Show the recipe list.
    fn show_recipe_list(&mut self, ui: &mut Ui) {
        // Collect recipe data to avoid borrow issues
        let recipe_data: Vec<_> = self
            .recipes
            .iter()
            .filter(|r| r.matches_category(self.category_filter))
            .filter(|r| self.search_query.is_empty() || r.matches_search(&self.search_query))
            .filter(|r| self.config.show_locked || r.status != RecipeStatus::Locked)
            .map(|r| {
                (
                    r.id.clone(),
                    r.name.clone(),
                    r.output.name.clone(),
                    r.output.quantity,
                    r.output.rarity,
                    r.status,
                    r.can_craft(),
                    r.ingredients
                        .iter()
                        .map(|i| (i.name.clone(), i.quantity, i.available, i.has_enough()))
                        .collect::<Vec<_>>(),
                    r.craft_time,
                )
            })
            .collect();

        let selected_id = self.selected_recipe.clone();
        let card_height = self.config.card_height;
        let show_ingredients = self.config.show_ingredients;

        egui::ScrollArea::vertical()
            .max_height(self.config.list_height)
            .show(ui, |ui| {
                for (
                    id,
                    name,
                    output_name,
                    output_qty,
                    rarity,
                    status,
                    can_craft,
                    ingredients,
                    craft_time,
                ) in &recipe_data
                {
                    let is_selected = selected_id.as_ref() == Some(id);
                    let is_locked = *status == RecipeStatus::Locked;

                    // Recipe card
                    let frame_color = if is_selected {
                        Color32::from_rgb(80, 120, 180)
                    } else {
                        status.color().linear_multiply(0.3)
                    };

                    egui::Frame::none()
                        .fill(frame_color)
                        .inner_margin(8.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.set_min_height(card_height);

                            ui.horizontal(|ui| {
                                // Status icon
                                ui.label(status.icon());

                                // Recipe name and output
                                ui.vertical(|ui| {
                                    let name_text = if is_locked {
                                        egui::RichText::new(name).color(Color32::GRAY)
                                    } else {
                                        egui::RichText::new(name).color(rarity.color())
                                    };
                                    ui.label(name_text);

                                    if !is_locked {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("→ {output_qty}x {output_name}"));
                                            ui.weak(format!("({craft_time:.1}s)"));
                                        });
                                    }
                                });

                                // Craftable indicator
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if *can_craft {
                                            ui.colored_label(
                                                Color32::from_rgb(100, 200, 100),
                                                "✓ Craft",
                                            );
                                        } else if !is_locked {
                                            ui.colored_label(Color32::from_rgb(200, 100, 100), "✗");
                                        }
                                    },
                                );
                            });

                            // Ingredients
                            if show_ingredients && !is_locked && !ingredients.is_empty() {
                                ui.horizontal_wrapped(|ui| {
                                    for (ing_name, required, available, has_enough) in ingredients {
                                        let color = if *has_enough {
                                            Color32::from_rgb(150, 200, 150)
                                        } else {
                                            Color32::from_rgb(200, 150, 150)
                                        };
                                        ui.colored_label(
                                            color,
                                            format!("{available}/{required} {ing_name}"),
                                        );
                                    }
                                });
                            }
                        });

                    // Handle click
                    let response = ui.interact(
                        ui.min_rect(),
                        egui::Id::new(format!("recipe_{}", id.0)),
                        egui::Sense::click(),
                    );

                    if response.clicked() {
                        self.selected_recipe = Some(id.clone());
                        self.pending_actions
                            .push(RecipeBookAction::RecipeSelected(id.clone()));
                    }

                    if response.double_clicked() && *can_craft {
                        self.pending_actions
                            .push(RecipeBookAction::RecipeCraft(id.clone()));
                    }

                    ui.add_space(4.0);
                }

                if recipe_data.is_empty() {
                    ui.weak("No recipes match filter");
                }
            });
    }

    /// Drain pending actions.
    pub fn drain_actions(&mut self) -> Vec<RecipeBookAction> {
        std::mem::take(&mut self.pending_actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_id() {
        let id = RecipeId::new("iron_sword");
        assert_eq!(id.0, "iron_sword");
        assert_eq!(format!("{id}"), "iron_sword");
    }

    #[test]
    fn test_recipe_category() {
        assert_eq!(RecipeCategory::all().len(), 8);
        assert_eq!(RecipeCategory::Weapons.display_name(), "Weapons");
        assert_eq!(RecipeCategory::Tools.icon(), "🔧");
    }

    #[test]
    fn test_recipe_ingredient() {
        let ing = RecipeIngredient::new("iron", "Iron Ingot", 3).with_available(5);
        assert!(ing.has_enough());
        assert_eq!(ing.missing(), 0);

        let ing2 = RecipeIngredient::new("gold", "Gold Ingot", 5).with_available(2);
        assert!(!ing2.has_enough());
        assert_eq!(ing2.missing(), 3);
    }

    #[test]
    fn test_recipe_output() {
        let output = RecipeOutput::new("sword", "Iron Sword", 1)
            .with_rarity(ItemRarity::Rare)
            .with_description("A sturdy sword");

        assert_eq!(output.quantity, 1);
        assert_eq!(output.rarity, ItemRarity::Rare);
        assert_eq!(output.description, "A sturdy sword");
    }

    #[test]
    fn test_recipe_status() {
        assert_eq!(RecipeStatus::Locked.icon(), "🔒");
        assert_eq!(RecipeStatus::Craftable.icon(), "✓");
        assert_ne!(
            RecipeStatus::Locked.color(),
            RecipeStatus::Craftable.color()
        );
    }

    #[test]
    fn test_recipe_new() {
        let output = RecipeOutput::new("sword", "Iron Sword", 1);
        let recipe = Recipe::new(
            "iron_sword",
            "Iron Sword Recipe",
            RecipeCategory::Weapons,
            output,
        )
        .with_ingredient(RecipeIngredient::new("iron", "Iron", 2))
        .with_craft_time(2.0)
        .with_station("forge")
        .with_tag("metal");

        assert_eq!(recipe.name, "Iron Sword Recipe");
        assert_eq!(recipe.category, RecipeCategory::Weapons);
        assert_eq!(recipe.ingredients.len(), 1);
        assert_eq!(recipe.craft_time, 2.0);
        assert_eq!(recipe.station, Some("forge".to_string()));
        assert_eq!(recipe.tags.len(), 1);
    }

    #[test]
    fn test_recipe_can_craft() {
        let output = RecipeOutput::new("sword", "Sword", 1);
        let mut recipe = Recipe::new("sword", "Sword", RecipeCategory::Weapons, output)
            .with_ingredient(RecipeIngredient::new("iron", "Iron", 2).with_available(5))
            .with_status(RecipeStatus::Discovered);

        assert!(recipe.can_craft());

        recipe.status = RecipeStatus::Locked;
        assert!(!recipe.can_craft());

        recipe.status = RecipeStatus::Discovered;
        recipe.ingredients[0].available = 1;
        assert!(!recipe.can_craft());
    }

    #[test]
    fn test_recipe_matches_search() {
        let output = RecipeOutput::new("sword", "Iron Sword", 1);
        let recipe = Recipe::new("iron_sword", "Iron Sword", RecipeCategory::Weapons, output)
            .with_ingredient(RecipeIngredient::new("iron", "Iron Ingot", 2))
            .with_tag("metal");

        assert!(recipe.matches_search("sword"));
        assert!(recipe.matches_search("IRON")); // case insensitive
        assert!(recipe.matches_search("Ingot")); // ingredient name
        assert!(recipe.matches_search("metal")); // tag
        assert!(!recipe.matches_search("gold"));
    }

    #[test]
    fn test_recipe_matches_category() {
        let output = RecipeOutput::new("sword", "Sword", 1);
        let recipe = Recipe::new("sword", "Sword", RecipeCategory::Weapons, output);

        assert!(recipe.matches_category(RecipeCategory::All));
        assert!(recipe.matches_category(RecipeCategory::Weapons));
        assert!(!recipe.matches_category(RecipeCategory::Tools));
    }

    #[test]
    fn test_recipe_update_availability() {
        let output = RecipeOutput::new("sword", "Sword", 1);
        let mut recipe = Recipe::new("sword", "Sword", RecipeCategory::Weapons, output)
            .with_ingredient(RecipeIngredient::new("iron", "Iron", 2))
            .with_ingredient(RecipeIngredient::new("wood", "Wood", 1));

        let inventory = vec![
            (CraftingItemId::new("iron"), 5),
            (CraftingItemId::new("wood"), 10),
        ];

        recipe.update_availability(&inventory);
        assert_eq!(recipe.ingredients[0].available, 5);
        assert_eq!(recipe.ingredients[1].available, 10);
    }

    #[test]
    fn test_recipe_book_new() {
        let book = RecipeBook::new();
        assert!(!book.open);
        assert!(book.recipes.is_empty());
        assert_eq!(book.category_filter, RecipeCategory::All);
    }

    #[test]
    fn test_recipe_book_toggle() {
        let mut book = RecipeBook::new();
        assert!(!book.open);
        book.toggle();
        assert!(book.open);
        book.toggle();
        assert!(!book.open);
    }

    #[test]
    fn test_recipe_book_add_recipe() {
        let mut book = RecipeBook::new();
        let output = RecipeOutput::new("sword", "Sword", 1);
        let recipe = Recipe::new("sword", "Sword", RecipeCategory::Weapons, output);

        book.add_recipe(recipe);
        assert_eq!(book.recipes.len(), 1);
    }

    #[test]
    fn test_recipe_book_get_recipe() {
        let mut book = RecipeBook::new();
        let output = RecipeOutput::new("sword", "Sword", 1);
        let recipe = Recipe::new("sword", "Sword", RecipeCategory::Weapons, output);
        book.add_recipe(recipe);

        let found = book.get_recipe(&RecipeId::new("sword"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Sword");

        assert!(book.get_recipe(&RecipeId::new("axe")).is_none());
    }

    #[test]
    fn test_recipe_book_filtered_recipes() {
        let mut book = RecipeBook::new();
        book.add_recipe(
            Recipe::new(
                "sword",
                "Sword",
                RecipeCategory::Weapons,
                RecipeOutput::new("sword", "Sword", 1),
            )
            .with_status(RecipeStatus::Discovered),
        );
        book.add_recipe(
            Recipe::new(
                "axe",
                "Axe",
                RecipeCategory::Tools,
                RecipeOutput::new("axe", "Axe", 1),
            )
            .with_status(RecipeStatus::Discovered),
        );
        book.add_recipe(
            Recipe::new(
                "pick",
                "Pickaxe",
                RecipeCategory::Tools,
                RecipeOutput::new("pick", "Pickaxe", 1),
            )
            .with_status(RecipeStatus::Locked),
        );

        // All category
        assert_eq!(book.filtered_recipes().len(), 3);

        // Weapons only
        book.category_filter = RecipeCategory::Weapons;
        assert_eq!(book.filtered_recipes().len(), 1);

        // Tools only
        book.category_filter = RecipeCategory::Tools;
        assert_eq!(book.filtered_recipes().len(), 2);

        // Hide locked
        book.config.show_locked = false;
        assert_eq!(book.filtered_recipes().len(), 1);
    }

    #[test]
    fn test_recipe_book_count_by_category() {
        let mut book = RecipeBook::new();
        book.add_recipe(Recipe::new(
            "sword",
            "Sword",
            RecipeCategory::Weapons,
            RecipeOutput::new("sword", "Sword", 1),
        ));
        book.add_recipe(Recipe::new(
            "axe",
            "Axe",
            RecipeCategory::Tools,
            RecipeOutput::new("axe", "Axe", 1),
        ));
        book.add_recipe(Recipe::new(
            "pick",
            "Pick",
            RecipeCategory::Tools,
            RecipeOutput::new("pick", "Pick", 1),
        ));

        assert_eq!(book.count_by_category(RecipeCategory::All), 3);
        assert_eq!(book.count_by_category(RecipeCategory::Weapons), 1);
        assert_eq!(book.count_by_category(RecipeCategory::Tools), 2);
        assert_eq!(book.count_by_category(RecipeCategory::Armor), 0);
    }

    #[test]
    fn test_recipe_book_unlock_recipe() {
        let mut book = RecipeBook::new();
        book.add_recipe(Recipe::new(
            "sword",
            "Sword",
            RecipeCategory::Weapons,
            RecipeOutput::new("sword", "Sword", 1),
        ));

        let id = RecipeId::new("sword");
        assert_eq!(book.get_recipe(&id).unwrap().status, RecipeStatus::Locked);

        book.unlock_recipe(&id);
        assert_eq!(
            book.get_recipe(&id).unwrap().status,
            RecipeStatus::Discovered
        );
    }

    #[test]
    fn test_recipe_book_toggle_favorite() {
        let mut book = RecipeBook::new();
        book.add_recipe(
            Recipe::new(
                "sword",
                "Sword",
                RecipeCategory::Weapons,
                RecipeOutput::new("sword", "Sword", 1),
            )
            .with_status(RecipeStatus::Discovered),
        );

        let id = RecipeId::new("sword");
        book.toggle_favorite(&id);
        assert_eq!(book.get_recipe(&id).unwrap().status, RecipeStatus::Favorite);

        book.toggle_favorite(&id);
        assert_eq!(
            book.get_recipe(&id).unwrap().status,
            RecipeStatus::Discovered
        );
    }

    #[test]
    fn test_recipe_book_config_defaults() {
        let config = RecipeBookConfig::default();
        assert!(config.show_categories);
        assert!(config.show_search);
        assert!(config.show_ingredients);
        assert!(config.show_locked);
    }

    #[test]
    fn test_recipe_book_action_equality() {
        let action1 = RecipeBookAction::RecipeSelected(RecipeId::new("test"));
        let action2 = RecipeBookAction::RecipeSelected(RecipeId::new("test"));
        assert_eq!(action1, action2);
    }

    #[test]
    fn test_recipe_serialization() {
        let output = RecipeOutput::new("sword", "Sword", 1).with_rarity(ItemRarity::Rare);
        let recipe = Recipe::new("sword", "Sword Recipe", RecipeCategory::Weapons, output)
            .with_ingredient(RecipeIngredient::new("iron", "Iron", 2))
            .with_craft_time(2.5);

        let json = serde_json::to_string(&recipe).unwrap();
        let loaded: Recipe = serde_json::from_str(&json).unwrap();

        assert_eq!(recipe.id, loaded.id);
        assert_eq!(recipe.craft_time, loaded.craft_time);
    }

    #[test]
    fn test_recipe_book_config_serialization() {
        let config = RecipeBookConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: RecipeBookConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.list_height, loaded.list_height);
    }

    #[test]
    fn test_recipe_category_serialization() {
        for category in RecipeCategory::all() {
            let json = serde_json::to_string(category).unwrap();
            let loaded: RecipeCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(*category, loaded);
        }
    }

    #[test]
    fn test_recipe_book_drain_actions() {
        let mut book = RecipeBook::new();
        book.pending_actions
            .push(RecipeBookAction::RecipeSelected(RecipeId::new("test")));

        let actions = book.drain_actions();
        assert_eq!(actions.len(), 1);

        let actions2 = book.drain_actions();
        assert!(actions2.is_empty());
    }
}
