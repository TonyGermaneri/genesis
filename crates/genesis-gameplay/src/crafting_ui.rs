//! Crafting UI model for rendering.
//!
//! This module provides data structures for presenting crafting recipes
//! and managing crafting queue display in the UI layer.

use genesis_common::{ItemTypeId, RecipeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crafting::{CraftingSystem, Recipe};
use crate::inventory::Inventory;

/// Filter mode for recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecipeFilter {
    /// Show all recipes
    #[default]
    All,
    /// Show only craftable recipes (have all ingredients)
    Craftable,
    /// Show recipes with learned skills
    Learned,
    /// Show recipes by category
    Category(u32),
    /// Show favorited recipes
    Favorites,
}

impl RecipeFilter {
    /// Returns true if this filter matches "all".
    #[must_use]
    pub fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    /// Returns true if this filter is category-based.
    #[must_use]
    pub fn is_category(&self) -> bool {
        matches!(self, Self::Category(_))
    }

    /// Returns the category ID if this is a category filter.
    #[must_use]
    pub fn category_id(&self) -> Option<u32> {
        if let Self::Category(id) = self {
            Some(*id)
        } else {
            None
        }
    }
}

/// Sort order for recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecipeSort {
    /// Sort by name alphabetically
    #[default]
    Name,
    /// Sort by skill requirement
    SkillRequired,
    /// Sort by craft time
    CraftTime,
    /// Sort by ingredient count
    IngredientCount,
    /// Sort by recently crafted
    RecentlyUsed,
}

/// Ingredient display data for UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientUIData {
    /// Item type
    pub item_type: ItemTypeId,
    /// Quantity required
    pub required: u32,
    /// Quantity available in inventory
    pub available: u32,
    /// Whether we have enough
    pub is_satisfied: bool,
    /// Display name (if available)
    pub name: Option<String>,
}

impl IngredientUIData {
    /// Creates new ingredient UI data.
    #[must_use]
    pub fn new(item_type: ItemTypeId, required: u32, available: u32) -> Self {
        Self {
            item_type,
            required,
            available,
            is_satisfied: available >= required,
            name: None,
        }
    }

    /// Sets the display name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Returns how many more items are needed.
    #[must_use]
    pub fn missing_count(&self) -> u32 {
        if self.available >= self.required {
            0
        } else {
            self.required - self.available
        }
    }
}

/// UI data for a single recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeUIData {
    /// Recipe ID
    pub id: RecipeId,
    /// Recipe name
    pub name: String,
    /// Ingredients with availability
    pub ingredients: Vec<IngredientUIData>,
    /// Required tools
    pub tools: Vec<ItemTypeId>,
    /// Tool availability
    pub tools_available: Vec<bool>,
    /// Output item type
    pub output: ItemTypeId,
    /// Output quantity
    pub output_quantity: u32,
    /// Whether this recipe can be crafted (all ingredients and tools available)
    pub can_craft: bool,
    /// Required skill level
    pub skill_required: u32,
    /// Current skill level
    pub skill_current: u32,
    /// Whether skill requirement is met
    pub skill_met: bool,
    /// Craft time in ticks
    pub craft_time: u32,
    /// Whether this recipe is favorited
    pub is_favorite: bool,
    /// Category ID (if any)
    pub category: Option<u32>,
}

impl RecipeUIData {
    /// Creates recipe UI data from a recipe and inventory.
    #[must_use]
    pub fn from_recipe(recipe: &Recipe, inventory: &Inventory, skill_level: u32) -> Self {
        let ingredients: Vec<IngredientUIData> = recipe
            .ingredients
            .iter()
            .map(|ing| {
                let available = inventory.count(ing.item);
                IngredientUIData::new(ing.item, ing.quantity, available)
            })
            .collect();

        let tools_available: Vec<bool> =
            recipe.tools.iter().map(|t| inventory.has(*t, 1)).collect();

        let all_ingredients = ingredients.iter().all(|i| i.is_satisfied);
        let all_tools = tools_available.iter().all(|&t| t);
        let skill_met = skill_level >= recipe.skill_required;

        Self {
            id: recipe.id,
            name: recipe.name.clone(),
            ingredients,
            tools: recipe.tools.clone(),
            tools_available,
            output: recipe.output,
            output_quantity: recipe.output_quantity,
            can_craft: all_ingredients && all_tools && skill_met,
            skill_required: recipe.skill_required,
            skill_current: skill_level,
            skill_met,
            craft_time: recipe.craft_time,
            is_favorite: false,
            category: None,
        }
    }

    /// Sets favorite status.
    #[must_use]
    pub fn with_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }

    /// Sets category.
    #[must_use]
    pub fn with_category(mut self, category: u32) -> Self {
        self.category = Some(category);
        self
    }

    /// Returns the number of missing ingredients.
    #[must_use]
    pub fn missing_ingredient_count(&self) -> usize {
        self.ingredients.iter().filter(|i| !i.is_satisfied).count()
    }

    /// Returns the number of missing tools.
    #[must_use]
    pub fn missing_tool_count(&self) -> usize {
        self.tools_available.iter().filter(|&&t| !t).count()
    }
}

/// Queued crafting item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedCraft {
    /// Recipe being crafted
    pub recipe_id: RecipeId,
    /// Recipe name
    pub name: String,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Elapsed ticks
    pub elapsed_ticks: u32,
    /// Total ticks required
    pub total_ticks: u32,
    /// Remaining ticks
    pub remaining_ticks: u32,
    /// Quantity being crafted
    pub quantity: u32,
}

impl QueuedCraft {
    /// Creates a new queued craft.
    #[must_use]
    pub fn new(recipe_id: RecipeId, name: String, total_ticks: u32, quantity: u32) -> Self {
        Self {
            recipe_id,
            name,
            progress: 0.0,
            elapsed_ticks: 0,
            total_ticks,
            remaining_ticks: total_ticks,
            quantity,
        }
    }

    /// Updates progress by the given number of ticks.
    pub fn tick(&mut self, ticks: u32) {
        self.elapsed_ticks = (self.elapsed_ticks + ticks).min(self.total_ticks);
        self.remaining_ticks = self.total_ticks.saturating_sub(self.elapsed_ticks);

        if self.total_ticks > 0 {
            self.progress = self.elapsed_ticks as f32 / self.total_ticks as f32;
        } else {
            self.progress = 1.0;
        }
    }

    /// Returns true if crafting is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.elapsed_ticks >= self.total_ticks
    }

    /// Returns estimated time remaining in seconds (assuming 60 ticks per second).
    #[must_use]
    pub fn time_remaining_secs(&self, ticks_per_sec: f32) -> f32 {
        if ticks_per_sec > 0.0 {
            self.remaining_ticks as f32 / ticks_per_sec
        } else {
            0.0
        }
    }
}

/// Crafting UI model for rendering.
#[derive(Debug, Clone)]
pub struct CraftingUIModel {
    /// All available recipes as UI data
    pub recipes: Vec<RecipeUIData>,
    /// Filtered and sorted recipes (indices into `recipes`)
    pub filtered_indices: Vec<usize>,
    /// Current filter
    pub filter: RecipeFilter,
    /// Current sort order
    pub sort: RecipeSort,
    /// Reverse sort order
    pub sort_reverse: bool,
    /// Search query
    pub search_query: String,
    /// Selected recipe index (in filtered list)
    pub selected_index: Option<usize>,
    /// Crafting queue
    pub queue: Vec<QueuedCraft>,
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Favorited recipe IDs
    pub favorites: Vec<RecipeId>,
    /// Recipe categories (id -> name)
    pub categories: HashMap<u32, String>,
    /// Current skill level (for display)
    pub skill_level: u32,
}

impl CraftingUIModel {
    /// Creates a new crafting UI model from a crafting system and inventory.
    #[must_use]
    pub fn from_system(system: &CraftingSystem, inventory: &Inventory, skill_level: u32) -> Self {
        let recipes: Vec<RecipeUIData> = system
            .recipes()
            .map(|r| RecipeUIData::from_recipe(r, inventory, skill_level))
            .collect();

        let filtered_indices: Vec<usize> = (0..recipes.len()).collect();

        Self {
            recipes,
            filtered_indices,
            filter: RecipeFilter::default(),
            sort: RecipeSort::default(),
            sort_reverse: false,
            search_query: String::new(),
            selected_index: None,
            queue: Vec::new(),
            max_queue_size: 5,
            favorites: Vec::new(),
            categories: HashMap::new(),
            skill_level,
        }
    }

    /// Creates an empty crafting UI model.
    #[must_use]
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
            filtered_indices: Vec::new(),
            filter: RecipeFilter::default(),
            sort: RecipeSort::default(),
            sort_reverse: false,
            search_query: String::new(),
            selected_index: None,
            queue: Vec::new(),
            max_queue_size: 5,
            favorites: Vec::new(),
            categories: HashMap::new(),
            skill_level: 0,
        }
    }

    /// Registers a category.
    pub fn register_category(&mut self, id: u32, name: impl Into<String>) {
        self.categories.insert(id, name.into());
    }

    /// Sets a recipe's category.
    pub fn set_recipe_category(&mut self, recipe_id: RecipeId, category: u32) {
        for recipe in &mut self.recipes {
            if recipe.id == recipe_id {
                recipe.category = Some(category);
                break;
            }
        }
    }

    /// Sets the filter and re-filters recipes.
    pub fn set_filter(&mut self, filter: RecipeFilter) {
        self.filter = filter;
        self.apply_filter_and_sort();
    }

    /// Sets the sort order and re-sorts recipes.
    pub fn set_sort(&mut self, sort: RecipeSort, reverse: bool) {
        self.sort = sort;
        self.sort_reverse = reverse;
        self.apply_filter_and_sort();
    }

    /// Sets the search query and re-filters.
    pub fn set_search(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
        self.apply_filter_and_sort();
    }

    /// Toggles favorite status for a recipe.
    pub fn toggle_favorite(&mut self, recipe_id: RecipeId) {
        if let Some(pos) = self.favorites.iter().position(|&id| id == recipe_id) {
            self.favorites.remove(pos);
        } else {
            self.favorites.push(recipe_id);
        }

        // Update the recipe's favorite status
        for recipe in &mut self.recipes {
            if recipe.id == recipe_id {
                recipe.is_favorite = self.favorites.contains(&recipe_id);
                break;
            }
        }

        // Re-filter if showing favorites
        if matches!(self.filter, RecipeFilter::Favorites) {
            self.apply_filter_and_sort();
        }
    }

    /// Applies the current filter and sort to the recipe list.
    pub fn apply_filter_and_sort(&mut self) {
        // First, filter
        self.filtered_indices = self
            .recipes
            .iter()
            .enumerate()
            .filter(|(_, r)| self.matches_filter(r))
            .filter(|(_, r)| self.matches_search(r))
            .map(|(i, _)| i)
            .collect();

        // Then sort
        let recipes = &self.recipes;
        let sort = self.sort;
        let reverse = self.sort_reverse;

        self.filtered_indices.sort_by(|&a, &b| {
            let ra = &recipes[a];
            let rb = &recipes[b];

            let ordering = match sort {
                RecipeSort::Name => ra.name.cmp(&rb.name),
                RecipeSort::SkillRequired => ra.skill_required.cmp(&rb.skill_required),
                RecipeSort::CraftTime => ra.craft_time.cmp(&rb.craft_time),
                RecipeSort::IngredientCount => ra.ingredients.len().cmp(&rb.ingredients.len()),
                RecipeSort::RecentlyUsed => std::cmp::Ordering::Equal, // Would need history
            };

            if reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });

        // Reset selection if it's now out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= self.filtered_indices.len() {
                self.selected_index = if self.filtered_indices.is_empty() {
                    None
                } else {
                    Some(self.filtered_indices.len() - 1)
                };
            }
        }
    }

    /// Checks if a recipe matches the current filter.
    fn matches_filter(&self, recipe: &RecipeUIData) -> bool {
        match self.filter {
            RecipeFilter::All => true,
            RecipeFilter::Craftable => recipe.can_craft,
            RecipeFilter::Learned => recipe.skill_met,
            RecipeFilter::Category(cat) => recipe.category == Some(cat),
            RecipeFilter::Favorites => self.favorites.contains(&recipe.id),
        }
    }

    /// Checks if a recipe matches the search query.
    fn matches_search(&self, recipe: &RecipeUIData) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        let query = self.search_query.to_lowercase();
        recipe.name.to_lowercase().contains(&query)
    }

    /// Updates recipe data from inventory (recalculates availability).
    pub fn update_from_inventory(&mut self, system: &CraftingSystem, inventory: &Inventory) {
        self.recipes = system
            .recipes()
            .map(|r| {
                let mut ui_data = RecipeUIData::from_recipe(r, inventory, self.skill_level);
                ui_data.is_favorite = self.favorites.contains(&r.id);
                // Preserve category if set
                if let Some(existing) = self.recipes.iter().find(|rr| rr.id == r.id) {
                    ui_data.category = existing.category;
                }
                ui_data
            })
            .collect();

        self.apply_filter_and_sort();
    }

    /// Selects a recipe by filtered index.
    pub fn select(&mut self, index: usize) {
        if index < self.filtered_indices.len() {
            self.selected_index = Some(index);
        }
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selected_index = None;
    }

    /// Gets the currently selected recipe.
    #[must_use]
    pub fn selected_recipe(&self) -> Option<&RecipeUIData> {
        self.selected_index
            .and_then(|i| self.filtered_indices.get(i))
            .and_then(|&idx| self.recipes.get(idx))
    }

    /// Gets recipe data by filtered index.
    #[must_use]
    pub fn get_filtered(&self, index: usize) -> Option<&RecipeUIData> {
        self.filtered_indices
            .get(index)
            .and_then(|&idx| self.recipes.get(idx))
    }

    /// Returns the number of filtered recipes.
    #[must_use]
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Adds a craft to the queue.
    /// Returns false if queue is full.
    pub fn queue_craft(&mut self, recipe: &Recipe, quantity: u32) -> bool {
        if self.queue.len() >= self.max_queue_size {
            return false;
        }

        self.queue.push(QueuedCraft::new(
            recipe.id,
            recipe.name.clone(),
            recipe.craft_time * quantity,
            quantity,
        ));

        true
    }

    /// Updates the crafting queue by the given number of ticks.
    /// Returns completed craft recipe IDs.
    pub fn tick_queue(&mut self, ticks: u32) -> Vec<RecipeId> {
        let mut completed = Vec::new();

        if let Some(craft) = self.queue.first_mut() {
            craft.tick(ticks);
            if craft.is_complete() {
                completed.push(craft.recipe_id);
                self.queue.remove(0);
            }
        }

        completed
    }

    /// Cancels a queued craft by index.
    pub fn cancel_queued(&mut self, index: usize) -> Option<QueuedCraft> {
        if index < self.queue.len() {
            Some(self.queue.remove(index))
        } else {
            None
        }
    }

    /// Clears the crafting queue.
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Returns the current craft (first in queue).
    #[must_use]
    pub fn current_craft(&self) -> Option<&QueuedCraft> {
        self.queue.first()
    }

    /// Returns queue progress (0.0 to 1.0) if crafting.
    #[must_use]
    pub fn queue_progress(&self) -> Option<f32> {
        self.current_craft().map(|c| c.progress)
    }

    /// Returns true if the queue is full.
    #[must_use]
    pub fn is_queue_full(&self) -> bool {
        self.queue.len() >= self.max_queue_size
    }

    /// Returns true if currently crafting.
    #[must_use]
    pub fn is_crafting(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Gets the number of craftable recipes.
    #[must_use]
    pub fn craftable_count(&self) -> usize {
        self.recipes.iter().filter(|r| r.can_craft).count()
    }

    /// Iterator over filtered recipes.
    pub fn iter_filtered(&self) -> impl Iterator<Item = &RecipeUIData> {
        self.filtered_indices
            .iter()
            .filter_map(|&idx| self.recipes.get(idx))
    }
}

impl Default for CraftingUIModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crafting::Recipe;

    fn create_test_recipe(id: u32, name: &str) -> Recipe {
        Recipe::builder(RecipeId::new(id), name)
            .ingredient(ItemTypeId::new(1), 2)
            .output(ItemTypeId::new(10), 1)
            .build()
    }

    fn create_test_inventory() -> Inventory {
        let mut inv = Inventory::new(10);
        let _ = inv.add(ItemTypeId::new(1), 5);
        let _ = inv.add(ItemTypeId::new(2), 3);
        inv
    }

    #[test]
    fn test_recipe_filter_variants() {
        assert!(RecipeFilter::All.is_all());
        assert!(!RecipeFilter::Craftable.is_all());

        assert!(RecipeFilter::Category(1).is_category());
        assert!(!RecipeFilter::All.is_category());

        assert_eq!(RecipeFilter::Category(5).category_id(), Some(5));
        assert_eq!(RecipeFilter::All.category_id(), None);
    }

    #[test]
    fn test_ingredient_ui_data() {
        let ing = IngredientUIData::new(ItemTypeId::new(1), 5, 3);
        assert!(!ing.is_satisfied);
        assert_eq!(ing.missing_count(), 2);

        let ing2 = IngredientUIData::new(ItemTypeId::new(1), 5, 10);
        assert!(ing2.is_satisfied);
        assert_eq!(ing2.missing_count(), 0);

        let ing3 = ing.with_name("Stone");
        assert_eq!(ing3.name, Some("Stone".to_string()));
    }

    #[test]
    fn test_recipe_ui_data_from_recipe() {
        let recipe = create_test_recipe(1, "Test Recipe");
        let inventory = create_test_inventory();

        let ui_data = RecipeUIData::from_recipe(&recipe, &inventory, 0);

        assert_eq!(ui_data.id, RecipeId::new(1));
        assert_eq!(ui_data.name, "Test Recipe");
        assert_eq!(ui_data.ingredients.len(), 1);
        assert!(ui_data.can_craft); // We have 5 of item 1, need 2
    }

    #[test]
    fn test_recipe_ui_data_missing_ingredient() {
        let recipe = Recipe::builder(RecipeId::new(1), "Test")
            .ingredient(ItemTypeId::new(99), 10) // Item not in inventory
            .output(ItemTypeId::new(10), 1)
            .build();

        let inventory = create_test_inventory();
        let ui_data = RecipeUIData::from_recipe(&recipe, &inventory, 0);

        assert!(!ui_data.can_craft);
        assert_eq!(ui_data.missing_ingredient_count(), 1);
    }

    #[test]
    fn test_recipe_ui_data_skill_requirement() {
        let recipe = Recipe::builder(RecipeId::new(1), "Test")
            .ingredient(ItemTypeId::new(1), 2)
            .output(ItemTypeId::new(10), 1)
            .skill_required(10)
            .build();

        let inventory = create_test_inventory();

        let ui_data_low = RecipeUIData::from_recipe(&recipe, &inventory, 5);
        assert!(!ui_data_low.skill_met);
        assert!(!ui_data_low.can_craft);

        let ui_data_high = RecipeUIData::from_recipe(&recipe, &inventory, 15);
        assert!(ui_data_high.skill_met);
        assert!(ui_data_high.can_craft);
    }

    #[test]
    fn test_queued_craft() {
        let mut craft = QueuedCraft::new(RecipeId::new(1), "Test".to_string(), 100, 1);

        assert_eq!(craft.progress, 0.0);
        assert!(!craft.is_complete());

        craft.tick(50);
        assert!((craft.progress - 0.5).abs() < 0.001);
        assert_eq!(craft.remaining_ticks, 50);

        craft.tick(50);
        assert!(craft.is_complete());
        assert_eq!(craft.remaining_ticks, 0);
    }

    #[test]
    fn test_queued_craft_time_remaining() {
        let craft = QueuedCraft::new(RecipeId::new(1), "Test".to_string(), 60, 1);

        // At 60 ticks per second, 60 ticks = 1 second
        let time = craft.time_remaining_secs(60.0);
        assert!((time - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_crafting_ui_model_new() {
        let model = CraftingUIModel::new();

        assert!(model.recipes.is_empty());
        assert!(model.queue.is_empty());
        assert_eq!(model.max_queue_size, 5);
    }

    #[test]
    fn test_crafting_ui_model_filter_all() {
        let mut model = CraftingUIModel::new();

        // Add some test recipes manually
        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Recipe A".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });
        model.recipes.push(RecipeUIData {
            id: RecipeId::new(2),
            name: "Recipe B".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(2),
            output_quantity: 1,
            can_craft: false,
            skill_required: 10,
            skill_current: 0,
            skill_met: false,
            craft_time: 20,
            is_favorite: false,
            category: None,
        });

        model.apply_filter_and_sort();
        assert_eq!(model.filtered_count(), 2);
    }

    #[test]
    fn test_crafting_ui_model_filter_craftable() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Craftable".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });
        model.recipes.push(RecipeUIData {
            id: RecipeId::new(2),
            name: "Not Craftable".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(2),
            output_quantity: 1,
            can_craft: false,
            skill_required: 10,
            skill_current: 0,
            skill_met: false,
            craft_time: 20,
            is_favorite: false,
            category: None,
        });

        model.set_filter(RecipeFilter::Craftable);
        assert_eq!(model.filtered_count(), 1);
        assert_eq!(
            model.get_filtered(0).map(|r| &r.name),
            Some(&"Craftable".to_string())
        );
    }

    #[test]
    fn test_crafting_ui_model_search() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Iron Sword".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });
        model.recipes.push(RecipeUIData {
            id: RecipeId::new(2),
            name: "Wooden Shield".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(2),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 20,
            is_favorite: false,
            category: None,
        });

        model.set_search("iron");
        assert_eq!(model.filtered_count(), 1);
        assert_eq!(
            model.get_filtered(0).map(|r| &r.name),
            Some(&"Iron Sword".to_string())
        );
    }

    #[test]
    fn test_crafting_ui_model_sort() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Zebra".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 5,
            skill_current: 0,
            skill_met: false,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });
        model.recipes.push(RecipeUIData {
            id: RecipeId::new(2),
            name: "Apple".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(2),
            output_quantity: 1,
            can_craft: true,
            skill_required: 10,
            skill_current: 0,
            skill_met: false,
            craft_time: 20,
            is_favorite: false,
            category: None,
        });

        // Sort by name
        model.set_sort(RecipeSort::Name, false);
        assert_eq!(
            model.get_filtered(0).map(|r| &r.name),
            Some(&"Apple".to_string())
        );
        assert_eq!(
            model.get_filtered(1).map(|r| &r.name),
            Some(&"Zebra".to_string())
        );

        // Sort by name reversed
        model.set_sort(RecipeSort::Name, true);
        assert_eq!(
            model.get_filtered(0).map(|r| &r.name),
            Some(&"Zebra".to_string())
        );

        // Sort by skill
        model.set_sort(RecipeSort::SkillRequired, false);
        assert_eq!(model.get_filtered(0).map(|r| r.skill_required), Some(5));
    }

    #[test]
    fn test_crafting_ui_model_favorites() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Recipe 1".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });

        model.toggle_favorite(RecipeId::new(1));
        assert!(model.favorites.contains(&RecipeId::new(1)));
        assert!(model.recipes[0].is_favorite);

        model.toggle_favorite(RecipeId::new(1));
        assert!(!model.favorites.contains(&RecipeId::new(1)));
        assert!(!model.recipes[0].is_favorite);
    }

    #[test]
    fn test_crafting_ui_model_queue() {
        let mut model = CraftingUIModel::new();
        model.max_queue_size = 2;

        let recipe = create_test_recipe(1, "Test");

        assert!(model.queue_craft(&recipe, 1));
        assert!(model.queue_craft(&recipe, 1));
        assert!(!model.queue_craft(&recipe, 1)); // Queue full

        assert!(model.is_crafting());
        assert!(model.is_queue_full());
        assert_eq!(model.queue.len(), 2);
    }

    #[test]
    fn test_crafting_ui_model_tick_queue() {
        let mut model = CraftingUIModel::new();

        // Create a recipe with craft_time > 0
        let recipe = Recipe::builder(RecipeId::new(1), "Test")
            .ingredient(ItemTypeId::new(1), 2)
            .output(ItemTypeId::new(10), 1)
            .craft_time(100)
            .build();

        model.queue_craft(&recipe, 1);

        // Tick partially
        let completed = model.tick_queue(5);
        assert!(completed.is_empty());
        assert!(model.current_craft().is_some());

        // Tick to completion
        let completed = model.tick_queue(100);
        assert!(!completed.is_empty());
        assert!(model.queue.is_empty());
    }

    #[test]
    fn test_crafting_ui_model_cancel_queued() {
        let mut model = CraftingUIModel::new();
        let recipe = create_test_recipe(1, "Test");

        model.queue_craft(&recipe, 1);
        assert_eq!(model.queue.len(), 1);

        let cancelled = model.cancel_queued(0);
        assert!(cancelled.is_some());
        assert!(model.queue.is_empty());
    }

    #[test]
    fn test_crafting_ui_model_select() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Recipe 1".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });
        model.filtered_indices = vec![0];

        model.select(0);
        assert_eq!(model.selected_index, Some(0));
        assert!(model.selected_recipe().is_some());

        model.clear_selection();
        assert!(model.selected_index.is_none());
    }

    #[test]
    fn test_crafting_ui_model_categories() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Recipe 1".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });

        model.register_category(1, "Weapons");
        model.set_recipe_category(RecipeId::new(1), 1);

        assert_eq!(model.recipes[0].category, Some(1));
        assert_eq!(model.categories.get(&1), Some(&"Weapons".to_string()));
    }

    #[test]
    fn test_crafting_ui_model_craftable_count() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Craftable".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });
        model.recipes.push(RecipeUIData {
            id: RecipeId::new(2),
            name: "Not Craftable".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(2),
            output_quantity: 1,
            can_craft: false,
            skill_required: 10,
            skill_current: 0,
            skill_met: false,
            craft_time: 20,
            is_favorite: false,
            category: None,
        });

        assert_eq!(model.craftable_count(), 1);
    }

    #[test]
    fn test_crafting_ui_model_iter_filtered() {
        let mut model = CraftingUIModel::new();

        model.recipes.push(RecipeUIData {
            id: RecipeId::new(1),
            name: "Recipe 1".to_string(),
            ingredients: vec![],
            tools: vec![],
            tools_available: vec![],
            output: ItemTypeId::new(1),
            output_quantity: 1,
            can_craft: true,
            skill_required: 0,
            skill_current: 0,
            skill_met: true,
            craft_time: 10,
            is_favorite: false,
            category: None,
        });
        model.filtered_indices = vec![0];

        let count = model.iter_filtered().count();
        assert_eq!(count, 1);
    }
}
