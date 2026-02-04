//! Recipe definitions and data structures.
//!
//! This module provides:
//! - Recipe data structures with ingredients and patterns
//! - Recipe categories and tags for organization
//! - Shaped vs shapeless recipe support
//! - Recipe filtering and lookup

use genesis_common::{ItemTypeId, RecipeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// G-45: Recipe Categories and Tags
// ============================================================================

/// Categories for organizing recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecipeCategory {
    /// Tools (pickaxe, axe, shovel, etc.).
    Tools,
    /// Weapons (sword, bow, etc.).
    Weapons,
    /// Armor (helmet, chestplate, etc.).
    Armor,
    /// Potions and consumables.
    Potions,
    /// Building materials and structures.
    Building,
    /// Food and cooking recipes.
    Food,
    /// Furniture and decorations.
    Furniture,
    /// Components and materials.
    Materials,
    /// Miscellaneous items.
    Misc,
}

impl RecipeCategory {
    /// Get display name for this category.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Tools => "Tools",
            Self::Weapons => "Weapons",
            Self::Armor => "Armor",
            Self::Potions => "Potions",
            Self::Building => "Building",
            Self::Food => "Food",
            Self::Furniture => "Furniture",
            Self::Materials => "Materials",
            Self::Misc => "Miscellaneous",
        }
    }

    /// Get all categories.
    #[must_use]
    pub fn all() -> &'static [RecipeCategory] {
        &[
            Self::Tools,
            Self::Weapons,
            Self::Armor,
            Self::Potions,
            Self::Building,
            Self::Food,
            Self::Furniture,
            Self::Materials,
            Self::Misc,
        ]
    }
}

impl Default for RecipeCategory {
    fn default() -> Self {
        Self::Misc
    }
}

/// Tags for recipe filtering and grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecipeTag {
    /// Requires starter/basic materials only.
    Starter,
    /// Intermediate complexity recipe.
    Intermediate,
    /// Advanced/end-game recipe.
    Advanced,
    /// Requires smelting/forge.
    RequiresSmelting,
    /// Requires alchemy.
    RequiresAlchemy,
    /// Quest-related recipe.
    Quest,
    /// Limited time/event recipe.
    Event,
    /// Requires rare materials.
    Rare,
    /// Can be discovered through experimentation.
    Discoverable,
    /// Hidden until unlocked.
    Hidden,
}

impl RecipeTag {
    /// Get display name for this tag.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Starter => "Starter",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
            Self::RequiresSmelting => "Requires Smelting",
            Self::RequiresAlchemy => "Requires Alchemy",
            Self::Quest => "Quest",
            Self::Event => "Event",
            Self::Rare => "Rare",
            Self::Discoverable => "Discoverable",
            Self::Hidden => "Hidden",
        }
    }
}

// ============================================================================
// G-45: Recipe Ingredients
// ============================================================================

/// An ingredient requirement for a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeIngredient {
    /// Item type required.
    pub item: ItemTypeId,
    /// Quantity required.
    pub quantity: u32,
}

impl RecipeIngredient {
    /// Create a new ingredient requirement.
    #[must_use]
    pub const fn new(item: ItemTypeId, quantity: u32) -> Self {
        Self { item, quantity }
    }

    /// Create ingredient requiring 1 of an item.
    #[must_use]
    pub const fn one(item: ItemTypeId) -> Self {
        Self::new(item, 1)
    }
}

/// Alternative ingredients (any of these can satisfy the requirement).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngredientAlternatives {
    /// List of alternative items (any one can be used).
    pub alternatives: Vec<ItemTypeId>,
    /// Quantity required.
    pub quantity: u32,
}

impl IngredientAlternatives {
    /// Create alternatives from a list of items.
    #[must_use]
    pub fn new(alternatives: Vec<ItemTypeId>, quantity: u32) -> Self {
        Self {
            alternatives,
            quantity,
        }
    }

    /// Check if an item satisfies this requirement.
    #[must_use]
    pub fn accepts(&self, item: ItemTypeId) -> bool {
        self.alternatives.contains(&item)
    }
}

// ============================================================================
// G-45: Recipe Patterns (Shaped Crafting)
// ============================================================================

/// Grid slot for shaped recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternSlot {
    /// Empty slot.
    Empty,
    /// Requires specific item.
    Item(ItemTypeId),
    /// Requires any item from a group (referenced by index).
    Group(u8),
}

impl Default for PatternSlot {
    fn default() -> Self {
        Self::Empty
    }
}

/// Pattern for shaped recipes (grid-based).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipePattern {
    /// Pattern width.
    pub width: u8,
    /// Pattern height.
    pub height: u8,
    /// Grid slots (row-major order).
    pub slots: Vec<PatternSlot>,
    /// Item groups for Group slots.
    pub groups: HashMap<u8, Vec<ItemTypeId>>,
}

impl RecipePattern {
    /// Create a new pattern with the given dimensions.
    #[must_use]
    pub fn new(width: u8, height: u8) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            slots: vec![PatternSlot::Empty; size],
            groups: HashMap::new(),
        }
    }

    /// Create a 2x2 pattern.
    #[must_use]
    pub fn grid_2x2() -> Self {
        Self::new(2, 2)
    }

    /// Create a 3x3 pattern.
    #[must_use]
    pub fn grid_3x3() -> Self {
        Self::new(3, 3)
    }

    /// Set a slot at the given position.
    pub fn set_slot(&mut self, x: u8, y: u8, slot: PatternSlot) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.slots[idx] = slot;
        }
    }

    /// Get slot at position.
    #[must_use]
    pub fn get_slot(&self, x: u8, y: u8) -> PatternSlot {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.slots[idx]
        } else {
            PatternSlot::Empty
        }
    }

    /// Define an item group for Group slots.
    pub fn define_group(&mut self, group_id: u8, items: Vec<ItemTypeId>) {
        self.groups.insert(group_id, items);
    }

    /// Check if an input grid matches this pattern.
    #[must_use]
    pub fn matches(&self, input: &CraftingGrid) -> bool {
        // Try all valid positions where pattern could fit
        let max_x = input.width.saturating_sub(self.width);
        let max_y = input.height.saturating_sub(self.height);

        for start_y in 0..=max_y {
            for start_x in 0..=max_x {
                if self.matches_at(input, start_x, start_y) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if pattern matches at specific position.
    fn matches_at(&self, input: &CraftingGrid, start_x: u8, start_y: u8) -> bool {
        // Check pattern slots match
        for py in 0..self.height {
            for px in 0..self.width {
                let pattern_slot = self.get_slot(px, py);
                let input_slot = input.get_slot(start_x + px, start_y + py);

                if !self.slot_matches(pattern_slot, input_slot) {
                    return false;
                }
            }
        }

        // Check that slots outside pattern are empty
        for iy in 0..input.height {
            for ix in 0..input.width {
                let in_pattern = ix >= start_x
                    && ix < start_x + self.width
                    && iy >= start_y
                    && iy < start_y + self.height;

                if !in_pattern && input.get_slot(ix, iy).is_some() {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a pattern slot matches an input slot.
    fn slot_matches(&self, pattern: PatternSlot, input: Option<ItemTypeId>) -> bool {
        match (pattern, input) {
            (PatternSlot::Empty, None) => true,
            (PatternSlot::Empty, Some(_))
            | (PatternSlot::Item(_) | PatternSlot::Group(_), None) => false,
            (PatternSlot::Item(expected), Some(actual)) => expected == actual,
            (PatternSlot::Group(group_id), Some(actual)) => self
                .groups
                .get(&group_id)
                .is_some_and(|items| items.contains(&actual)),
        }
    }

    /// Get required items from this pattern.
    #[must_use]
    pub fn required_items(&self) -> Vec<ItemTypeId> {
        let mut items = Vec::new();
        for slot in &self.slots {
            if let PatternSlot::Item(item) = slot {
                items.push(*item);
            }
        }
        items
    }
}

/// Input crafting grid.
#[derive(Debug, Clone, Default)]
pub struct CraftingGrid {
    /// Grid width.
    pub width: u8,
    /// Grid height.
    pub height: u8,
    /// Items in slots (None = empty).
    pub slots: Vec<Option<ItemTypeId>>,
    /// Quantities per slot.
    pub quantities: Vec<u32>,
}

impl CraftingGrid {
    /// Create a new crafting grid.
    #[must_use]
    pub fn new(width: u8, height: u8) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            slots: vec![None; size],
            quantities: vec![0; size],
        }
    }

    /// Create a 2x2 grid.
    #[must_use]
    pub fn grid_2x2() -> Self {
        Self::new(2, 2)
    }

    /// Create a 3x3 grid.
    #[must_use]
    pub fn grid_3x3() -> Self {
        Self::new(3, 3)
    }

    /// Set item at position.
    pub fn set_slot(&mut self, x: u8, y: u8, item: Option<ItemTypeId>, quantity: u32) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.slots[idx] = item;
            self.quantities[idx] = quantity;
        }
    }

    /// Get item at position.
    #[must_use]
    pub fn get_slot(&self, x: u8, y: u8) -> Option<ItemTypeId> {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.slots[idx]
        } else {
            None
        }
    }

    /// Get quantity at position.
    #[must_use]
    pub fn get_quantity(&self, x: u8, y: u8) -> u32 {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.quantities[idx]
        } else {
            0
        }
    }

    /// Count items of a specific type.
    #[must_use]
    pub fn count_item(&self, item: ItemTypeId) -> u32 {
        self.slots
            .iter()
            .zip(self.quantities.iter())
            .filter(|(slot, _)| **slot == Some(item))
            .map(|(_, qty)| *qty)
            .sum()
    }

    /// Get all unique items in the grid.
    #[must_use]
    pub fn unique_items(&self) -> HashSet<ItemTypeId> {
        self.slots.iter().filter_map(|s| *s).collect()
    }

    /// Check if grid is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    /// Clear the grid.
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
        for qty in &mut self.quantities {
            *qty = 0;
        }
    }
}

// ============================================================================
// G-45: Recipe Definition
// ============================================================================

/// Type of recipe (shaped or shapeless).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeType {
    /// Shaped recipe requiring specific pattern.
    Shaped(RecipePattern),
    /// Shapeless recipe - ingredients can be anywhere.
    Shapeless,
}

impl Default for RecipeType {
    fn default() -> Self {
        Self::Shapeless
    }
}

/// Station type required for crafting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum StationType {
    /// No station required (hand crafting).
    #[default]
    None,
    /// Basic crafting table.
    CraftingTable,
    /// Forge for smelting.
    Forge,
    /// Anvil for smithing.
    Anvil,
    /// Alchemy table for potions.
    AlchemyTable,
    /// Cooking station.
    CookingPot,
    /// Sawmill for wood processing.
    Sawmill,
    /// Loom for cloth/fabric.
    Loom,
    /// Enchanting table.
    EnchantingTable,
}

impl StationType {
    /// Get display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::None => "Hand Crafting",
            Self::CraftingTable => "Crafting Table",
            Self::Forge => "Forge",
            Self::Anvil => "Anvil",
            Self::AlchemyTable => "Alchemy Table",
            Self::CookingPot => "Cooking Pot",
            Self::Sawmill => "Sawmill",
            Self::Loom => "Loom",
            Self::EnchantingTable => "Enchanting Table",
        }
    }

    /// Check if this is a basic station (no specialized equipment).
    #[must_use]
    pub fn is_basic(self) -> bool {
        matches!(self, Self::None | Self::CraftingTable)
    }
}

/// Recipe output definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeOutput {
    /// Output item type.
    pub item: ItemTypeId,
    /// Base quantity produced.
    pub quantity: u32,
    /// Bonus quantity chance (0.0-1.0).
    pub bonus_chance: f32,
    /// Bonus quantity amount.
    pub bonus_quantity: u32,
}

impl RecipeOutput {
    /// Create simple output.
    #[must_use]
    pub const fn new(item: ItemTypeId, quantity: u32) -> Self {
        Self {
            item,
            quantity,
            bonus_chance: 0.0,
            bonus_quantity: 0,
        }
    }

    /// Create output with bonus chance.
    #[must_use]
    pub const fn with_bonus(
        item: ItemTypeId,
        quantity: u32,
        bonus_chance: f32,
        bonus_quantity: u32,
    ) -> Self {
        Self {
            item,
            quantity,
            bonus_chance,
            bonus_quantity,
        }
    }
}

/// Complete recipe data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeData {
    /// Unique recipe identifier.
    pub id: RecipeId,
    /// Recipe name.
    pub name: String,
    /// Recipe description.
    pub description: String,
    /// Category for organization.
    pub category: RecipeCategory,
    /// Tags for filtering.
    pub tags: HashSet<RecipeTag>,
    /// Recipe type (shaped/shapeless).
    pub recipe_type: RecipeType,
    /// Required ingredients (for shapeless).
    pub ingredients: Vec<RecipeIngredient>,
    /// Alternative ingredient options.
    pub alternatives: Vec<IngredientAlternatives>,
    /// Required station.
    pub station: StationType,
    /// Output item(s).
    pub output: RecipeOutput,
    /// Secondary outputs (byproducts).
    pub secondary_outputs: Vec<RecipeOutput>,
    /// Required skill type (if any).
    pub skill_type: Option<String>,
    /// Required skill level.
    pub skill_level: u32,
    /// Skill experience gained.
    pub skill_xp: u32,
    /// Crafting time in ticks.
    pub craft_time: u32,
    /// Whether recipe is unlocked by default.
    pub unlocked_by_default: bool,
}

impl RecipeData {
    /// Create a new recipe builder.
    #[must_use]
    pub fn builder(id: RecipeId, name: impl Into<String>) -> RecipeDataBuilder {
        RecipeDataBuilder::new(id, name)
    }

    /// Check if recipe has a specific tag.
    #[must_use]
    pub fn has_tag(&self, tag: RecipeTag) -> bool {
        self.tags.contains(&tag)
    }

    /// Check if recipe matches search text.
    #[must_use]
    pub fn matches_search(&self, search: &str) -> bool {
        let search_lower = search.to_lowercase();
        self.name.to_lowercase().contains(&search_lower)
            || self.description.to_lowercase().contains(&search_lower)
    }

    /// Get total ingredient count.
    #[must_use]
    pub fn total_ingredient_count(&self) -> u32 {
        self.ingredients.iter().map(|i| i.quantity).sum()
    }

    /// Check if this is a shaped recipe.
    #[must_use]
    pub fn is_shaped(&self) -> bool {
        matches!(self.recipe_type, RecipeType::Shaped(_))
    }
}

/// Builder for RecipeData.
#[derive(Debug)]
pub struct RecipeDataBuilder {
    id: RecipeId,
    name: String,
    description: String,
    category: RecipeCategory,
    tags: HashSet<RecipeTag>,
    recipe_type: RecipeType,
    ingredients: Vec<RecipeIngredient>,
    alternatives: Vec<IngredientAlternatives>,
    station: StationType,
    output: Option<RecipeOutput>,
    secondary_outputs: Vec<RecipeOutput>,
    skill_type: Option<String>,
    skill_level: u32,
    skill_xp: u32,
    craft_time: u32,
    unlocked_by_default: bool,
}

impl RecipeDataBuilder {
    /// Create new builder.
    fn new(id: RecipeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: String::new(),
            category: RecipeCategory::default(),
            tags: HashSet::new(),
            recipe_type: RecipeType::default(),
            ingredients: Vec::new(),
            alternatives: Vec::new(),
            station: StationType::default(),
            output: None,
            secondary_outputs: Vec::new(),
            skill_type: None,
            skill_level: 0,
            skill_xp: 0,
            craft_time: 60,
            unlocked_by_default: true,
        }
    }

    /// Set description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set category.
    #[must_use]
    pub fn category(mut self, category: RecipeCategory) -> Self {
        self.category = category;
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn tag(mut self, tag: RecipeTag) -> Self {
        self.tags.insert(tag);
        self
    }

    /// Set as shaped recipe with pattern.
    #[must_use]
    pub fn shaped(mut self, pattern: RecipePattern) -> Self {
        self.recipe_type = RecipeType::Shaped(pattern);
        self
    }

    /// Add an ingredient.
    #[must_use]
    pub fn ingredient(mut self, item: ItemTypeId, quantity: u32) -> Self {
        self.ingredients.push(RecipeIngredient::new(item, quantity));
        self
    }

    /// Add alternative ingredients.
    #[must_use]
    pub fn alternatives(mut self, items: Vec<ItemTypeId>, quantity: u32) -> Self {
        self.alternatives
            .push(IngredientAlternatives::new(items, quantity));
        self
    }

    /// Set required station.
    #[must_use]
    pub fn station(mut self, station: StationType) -> Self {
        self.station = station;
        self
    }

    /// Set output.
    #[must_use]
    pub fn output(mut self, item: ItemTypeId, quantity: u32) -> Self {
        self.output = Some(RecipeOutput::new(item, quantity));
        self
    }

    /// Set output with bonus chance.
    #[must_use]
    pub fn output_with_bonus(
        mut self,
        item: ItemTypeId,
        quantity: u32,
        bonus_chance: f32,
        bonus_quantity: u32,
    ) -> Self {
        self.output = Some(RecipeOutput::with_bonus(
            item,
            quantity,
            bonus_chance,
            bonus_quantity,
        ));
        self
    }

    /// Add secondary output.
    #[must_use]
    pub fn secondary_output(mut self, item: ItemTypeId, quantity: u32) -> Self {
        self.secondary_outputs
            .push(RecipeOutput::new(item, quantity));
        self
    }

    /// Set skill requirement.
    #[must_use]
    pub fn skill(mut self, skill_type: impl Into<String>, level: u32, xp: u32) -> Self {
        self.skill_type = Some(skill_type.into());
        self.skill_level = level;
        self.skill_xp = xp;
        self
    }

    /// Set craft time.
    #[must_use]
    pub const fn craft_time(mut self, ticks: u32) -> Self {
        self.craft_time = ticks;
        self
    }

    /// Set whether unlocked by default.
    #[must_use]
    pub const fn unlocked_by_default(mut self, unlocked: bool) -> Self {
        self.unlocked_by_default = unlocked;
        self
    }

    /// Build the recipe.
    ///
    /// # Panics
    /// Panics if output is not set.
    #[must_use]
    pub fn build(self) -> RecipeData {
        RecipeData {
            id: self.id,
            name: self.name,
            description: self.description,
            category: self.category,
            tags: self.tags,
            recipe_type: self.recipe_type,
            ingredients: self.ingredients,
            alternatives: self.alternatives,
            station: self.station,
            output: self.output.expect("Recipe must have output"),
            secondary_outputs: self.secondary_outputs,
            skill_type: self.skill_type,
            skill_level: self.skill_level,
            skill_xp: self.skill_xp,
            craft_time: self.craft_time,
            unlocked_by_default: self.unlocked_by_default,
        }
    }
}

// ============================================================================
// G-45: Recipe Registry
// ============================================================================

/// Registry for all recipes.
#[derive(Debug, Default)]
pub struct RecipeRegistry {
    /// All recipes by ID.
    recipes: HashMap<RecipeId, RecipeData>,
    /// Recipes by category.
    by_category: HashMap<RecipeCategory, Vec<RecipeId>>,
    /// Recipes by tag.
    by_tag: HashMap<RecipeTag, Vec<RecipeId>>,
    /// Recipes by station.
    by_station: HashMap<StationType, Vec<RecipeId>>,
    /// Recipes by output item.
    by_output: HashMap<ItemTypeId, Vec<RecipeId>>,
}

impl RecipeRegistry {
    /// Create new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a recipe.
    pub fn register(&mut self, recipe: RecipeData) {
        let id = recipe.id;

        // Index by category
        self.by_category
            .entry(recipe.category)
            .or_default()
            .push(id);

        // Index by tags
        for tag in &recipe.tags {
            self.by_tag.entry(*tag).or_default().push(id);
        }

        // Index by station
        self.by_station.entry(recipe.station).or_default().push(id);

        // Index by output
        self.by_output
            .entry(recipe.output.item)
            .or_default()
            .push(id);

        self.recipes.insert(id, recipe);
    }

    /// Get recipe by ID.
    #[must_use]
    pub fn get(&self, id: RecipeId) -> Option<&RecipeData> {
        self.recipes.get(&id)
    }

    /// Get all recipes.
    pub fn all(&self) -> impl Iterator<Item = &RecipeData> {
        self.recipes.values()
    }

    /// Get recipes by category.
    #[must_use]
    pub fn by_category(&self, category: RecipeCategory) -> Vec<&RecipeData> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get recipes by tag.
    #[must_use]
    pub fn by_tag(&self, tag: RecipeTag) -> Vec<&RecipeData> {
        self.by_tag
            .get(&tag)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get recipes by station.
    #[must_use]
    pub fn by_station(&self, station: StationType) -> Vec<&RecipeData> {
        self.by_station
            .get(&station)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get recipes that produce a specific item.
    #[must_use]
    pub fn by_output(&self, item: ItemTypeId) -> Vec<&RecipeData> {
        self.by_output
            .get(&item)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Search recipes by name/description.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&RecipeData> {
        self.recipes
            .values()
            .filter(|r| r.matches_search(query))
            .collect()
    }

    /// Get recipe count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    /// Filter recipes by predicate.
    pub fn filter<F>(&self, predicate: F) -> Vec<&RecipeData>
    where
        F: Fn(&RecipeData) -> bool,
    {
        self.recipes.values().filter(|r| predicate(r)).collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item(id: u32) -> ItemTypeId {
        ItemTypeId::new(id)
    }

    fn test_recipe_id(id: u32) -> RecipeId {
        RecipeId::new(id)
    }

    #[test]
    fn test_recipe_category_display() {
        assert_eq!(RecipeCategory::Tools.display_name(), "Tools");
        assert_eq!(RecipeCategory::Weapons.display_name(), "Weapons");
        assert_eq!(RecipeCategory::Potions.display_name(), "Potions");
    }

    #[test]
    fn test_recipe_tag_display() {
        assert_eq!(RecipeTag::Starter.display_name(), "Starter");
        assert_eq!(RecipeTag::Advanced.display_name(), "Advanced");
    }

    #[test]
    fn test_recipe_ingredient() {
        let ing = RecipeIngredient::new(test_item(1), 5);
        assert_eq!(ing.item, test_item(1));
        assert_eq!(ing.quantity, 5);
    }

    #[test]
    fn test_ingredient_alternatives() {
        let alt = IngredientAlternatives::new(vec![test_item(1), test_item(2)], 2);
        assert!(alt.accepts(test_item(1)));
        assert!(alt.accepts(test_item(2)));
        assert!(!alt.accepts(test_item(3)));
    }

    #[test]
    fn test_pattern_slot_default() {
        assert_eq!(PatternSlot::default(), PatternSlot::Empty);
    }

    #[test]
    fn test_recipe_pattern_creation() {
        let pattern = RecipePattern::grid_3x3();
        assert_eq!(pattern.width, 3);
        assert_eq!(pattern.height, 3);
        assert_eq!(pattern.slots.len(), 9);
    }

    #[test]
    fn test_recipe_pattern_set_slot() {
        let mut pattern = RecipePattern::grid_2x2();
        pattern.set_slot(0, 0, PatternSlot::Item(test_item(1)));
        pattern.set_slot(1, 1, PatternSlot::Item(test_item(2)));

        assert_eq!(pattern.get_slot(0, 0), PatternSlot::Item(test_item(1)));
        assert_eq!(pattern.get_slot(1, 1), PatternSlot::Item(test_item(2)));
        assert_eq!(pattern.get_slot(0, 1), PatternSlot::Empty);
    }

    #[test]
    fn test_crafting_grid() {
        let mut grid = CraftingGrid::grid_3x3();
        grid.set_slot(0, 0, Some(test_item(1)), 5);
        grid.set_slot(1, 1, Some(test_item(2)), 3);

        assert_eq!(grid.get_slot(0, 0), Some(test_item(1)));
        assert_eq!(grid.get_quantity(0, 0), 5);
        assert_eq!(grid.count_item(test_item(1)), 5);
        assert!(!grid.is_empty());
    }

    #[test]
    fn test_crafting_grid_clear() {
        let mut grid = CraftingGrid::grid_2x2();
        grid.set_slot(0, 0, Some(test_item(1)), 1);
        grid.clear();
        assert!(grid.is_empty());
    }

    #[test]
    fn test_station_type() {
        assert!(StationType::None.is_basic());
        assert!(StationType::CraftingTable.is_basic());
        assert!(!StationType::Forge.is_basic());
    }

    #[test]
    fn test_recipe_output() {
        let output = RecipeOutput::new(test_item(10), 2);
        assert_eq!(output.item, test_item(10));
        assert_eq!(output.quantity, 2);
        assert_eq!(output.bonus_chance, 0.0);
    }

    #[test]
    fn test_recipe_output_with_bonus() {
        let output = RecipeOutput::with_bonus(test_item(10), 1, 0.5, 1);
        assert_eq!(output.bonus_chance, 0.5);
        assert_eq!(output.bonus_quantity, 1);
    }

    #[test]
    fn test_recipe_data_builder() {
        let recipe = RecipeData::builder(test_recipe_id(1), "Test Recipe")
            .description("A test recipe")
            .category(RecipeCategory::Tools)
            .tag(RecipeTag::Starter)
            .ingredient(test_item(1), 2)
            .ingredient(test_item(2), 1)
            .station(StationType::CraftingTable)
            .output(test_item(10), 1)
            .skill("crafting", 1, 10)
            .craft_time(60)
            .build();

        assert_eq!(recipe.id, test_recipe_id(1));
        assert_eq!(recipe.name, "Test Recipe");
        assert_eq!(recipe.category, RecipeCategory::Tools);
        assert!(recipe.has_tag(RecipeTag::Starter));
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.station, StationType::CraftingTable);
        assert_eq!(recipe.skill_level, 1);
    }

    #[test]
    fn test_recipe_data_shaped() {
        let mut pattern = RecipePattern::grid_2x2();
        pattern.set_slot(0, 0, PatternSlot::Item(test_item(1)));
        pattern.set_slot(1, 0, PatternSlot::Item(test_item(1)));

        let recipe = RecipeData::builder(test_recipe_id(2), "Shaped Recipe")
            .shaped(pattern)
            .output(test_item(20), 1)
            .build();

        assert!(recipe.is_shaped());
    }

    #[test]
    fn test_recipe_matches_search() {
        let recipe = RecipeData::builder(test_recipe_id(1), "Iron Sword")
            .description("A basic iron sword")
            .output(test_item(1), 1)
            .build();

        assert!(recipe.matches_search("Iron"));
        assert!(recipe.matches_search("iron"));
        assert!(recipe.matches_search("sword"));
        assert!(recipe.matches_search("basic"));
        assert!(!recipe.matches_search("gold"));
    }

    #[test]
    fn test_recipe_registry() {
        let mut registry = RecipeRegistry::new();

        let recipe1 = RecipeData::builder(test_recipe_id(1), "Tool A")
            .category(RecipeCategory::Tools)
            .tag(RecipeTag::Starter)
            .station(StationType::None)
            .output(test_item(10), 1)
            .build();

        let recipe2 = RecipeData::builder(test_recipe_id(2), "Weapon B")
            .category(RecipeCategory::Weapons)
            .tag(RecipeTag::Starter)
            .station(StationType::Forge)
            .output(test_item(20), 1)
            .build();

        registry.register(recipe1);
        registry.register(recipe2);

        assert_eq!(registry.len(), 2);
        assert!(registry.get(test_recipe_id(1)).is_some());
    }

    #[test]
    fn test_recipe_registry_by_category() {
        let mut registry = RecipeRegistry::new();

        registry.register(
            RecipeData::builder(test_recipe_id(1), "Tool")
                .category(RecipeCategory::Tools)
                .output(test_item(1), 1)
                .build(),
        );
        registry.register(
            RecipeData::builder(test_recipe_id(2), "Weapon")
                .category(RecipeCategory::Weapons)
                .output(test_item(2), 1)
                .build(),
        );

        let tools = registry.by_category(RecipeCategory::Tools);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "Tool");
    }

    #[test]
    fn test_recipe_registry_by_tag() {
        let mut registry = RecipeRegistry::new();

        registry.register(
            RecipeData::builder(test_recipe_id(1), "Starter Item")
                .tag(RecipeTag::Starter)
                .output(test_item(1), 1)
                .build(),
        );
        registry.register(
            RecipeData::builder(test_recipe_id(2), "Advanced Item")
                .tag(RecipeTag::Advanced)
                .output(test_item(2), 1)
                .build(),
        );

        let starters = registry.by_tag(RecipeTag::Starter);
        assert_eq!(starters.len(), 1);
    }

    #[test]
    fn test_recipe_registry_by_station() {
        let mut registry = RecipeRegistry::new();

        registry.register(
            RecipeData::builder(test_recipe_id(1), "Hand Craft")
                .station(StationType::None)
                .output(test_item(1), 1)
                .build(),
        );
        registry.register(
            RecipeData::builder(test_recipe_id(2), "Forge Craft")
                .station(StationType::Forge)
                .output(test_item(2), 1)
                .build(),
        );

        let forge_recipes = registry.by_station(StationType::Forge);
        assert_eq!(forge_recipes.len(), 1);
    }

    #[test]
    fn test_recipe_registry_by_output() {
        let mut registry = RecipeRegistry::new();

        registry.register(
            RecipeData::builder(test_recipe_id(1), "Recipe A")
                .output(test_item(100), 1)
                .build(),
        );
        registry.register(
            RecipeData::builder(test_recipe_id(2), "Recipe B")
                .output(test_item(100), 2)
                .build(),
        );

        let recipes_for_item = registry.by_output(test_item(100));
        assert_eq!(recipes_for_item.len(), 2);
    }

    #[test]
    fn test_recipe_registry_search() {
        let mut registry = RecipeRegistry::new();

        registry.register(
            RecipeData::builder(test_recipe_id(1), "Iron Pickaxe")
                .output(test_item(1), 1)
                .build(),
        );
        registry.register(
            RecipeData::builder(test_recipe_id(2), "Gold Sword")
                .output(test_item(2), 1)
                .build(),
        );

        let results = registry.search("iron");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Iron Pickaxe");
    }

    #[test]
    fn test_pattern_matches_shapeless_in_grid() {
        let mut pattern = RecipePattern::grid_2x2();
        pattern.set_slot(0, 0, PatternSlot::Item(test_item(1)));
        pattern.set_slot(1, 0, PatternSlot::Item(test_item(1)));
        pattern.set_slot(0, 1, PatternSlot::Empty);
        pattern.set_slot(1, 1, PatternSlot::Empty);

        // Create matching input
        let mut grid = CraftingGrid::grid_2x2();
        grid.set_slot(0, 0, Some(test_item(1)), 1);
        grid.set_slot(1, 0, Some(test_item(1)), 1);

        assert!(pattern.matches(&grid));
    }

    #[test]
    fn test_pattern_no_match_wrong_item() {
        let mut pattern = RecipePattern::grid_2x2();
        pattern.set_slot(0, 0, PatternSlot::Item(test_item(1)));

        let mut grid = CraftingGrid::grid_2x2();
        grid.set_slot(0, 0, Some(test_item(2)), 1); // Wrong item

        assert!(!pattern.matches(&grid));
    }

    #[test]
    fn test_total_ingredient_count() {
        let recipe = RecipeData::builder(test_recipe_id(1), "Multi Ingredient")
            .ingredient(test_item(1), 3)
            .ingredient(test_item(2), 2)
            .output(test_item(10), 1)
            .build();

        assert_eq!(recipe.total_ingredient_count(), 5);
    }
}
