//! Crafting system for items and buildings.

use genesis_common::{ItemTypeId, RecipeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::inventory::{Inventory, InventoryError};

/// Crafting error types.
#[derive(Debug, Error)]
pub enum CraftingError {
    /// Recipe not found
    #[error("Recipe not found: {0:?}")]
    RecipeNotFound(RecipeId),
    /// Missing ingredients
    #[error("Missing ingredients")]
    MissingIngredients,
    /// Missing tool
    #[error("Missing required tool")]
    MissingTool,
    /// Skill too low
    #[error("Skill too low: need {required}, have {current}")]
    SkillTooLow {
        /// Required skill level
        required: u32,
        /// Current skill level
        current: u32,
    },
    /// Inventory error
    #[error("Inventory error: {0}")]
    Inventory(#[from] InventoryError),
}

/// Result type for crafting operations.
pub type CraftingResult<T> = Result<T, CraftingError>;

/// An ingredient for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    /// Item type required
    pub item: ItemTypeId,
    /// Quantity required
    pub quantity: u32,
}

/// A crafting recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Recipe identifier
    pub id: RecipeId,
    /// Recipe name
    pub name: String,
    /// Required ingredients
    pub ingredients: Vec<Ingredient>,
    /// Required tools (not consumed)
    pub tools: Vec<ItemTypeId>,
    /// Output item
    pub output: ItemTypeId,
    /// Output quantity
    pub output_quantity: u32,
    /// Required skill level
    pub skill_required: u32,
    /// Skill gained on craft
    pub skill_gain: u32,
    /// Time to craft (in game ticks)
    pub craft_time: u32,
}

/// Crafting system manager.
#[derive(Debug, Default)]
pub struct CraftingSystem {
    /// All available recipes
    recipes: HashMap<RecipeId, Recipe>,
}

impl CraftingSystem {
    /// Creates a new crafting system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            recipes: HashMap::new(),
        }
    }

    /// Registers a recipe.
    pub fn register_recipe(&mut self, recipe: Recipe) {
        self.recipes.insert(recipe.id, recipe);
    }

    /// Gets a recipe by ID.
    #[must_use]
    pub fn get_recipe(&self, id: RecipeId) -> Option<&Recipe> {
        self.recipes.get(&id)
    }

    /// Checks if a recipe can be crafted with the given inventory.
    pub fn can_craft(
        &self,
        recipe_id: RecipeId,
        inventory: &Inventory,
        skill_level: u32,
    ) -> CraftingResult<bool> {
        let recipe = self
            .recipes
            .get(&recipe_id)
            .ok_or(CraftingError::RecipeNotFound(recipe_id))?;

        // Check skill
        if skill_level < recipe.skill_required {
            return Err(CraftingError::SkillTooLow {
                required: recipe.skill_required,
                current: skill_level,
            });
        }

        // Check ingredients
        for ingredient in &recipe.ingredients {
            if !inventory.has(ingredient.item, ingredient.quantity) {
                return Ok(false);
            }
        }

        // Check tools
        for tool in &recipe.tools {
            if !inventory.has(*tool, 1) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Executes a craft operation.
    pub fn craft(
        &self,
        recipe_id: RecipeId,
        inventory: &mut Inventory,
        skill_level: u32,
    ) -> CraftingResult<()> {
        if !self.can_craft(recipe_id, inventory, skill_level)? {
            return Err(CraftingError::MissingIngredients);
        }

        let recipe = self
            .recipes
            .get(&recipe_id)
            .ok_or(CraftingError::RecipeNotFound(recipe_id))?;

        // Consume ingredients
        for ingredient in &recipe.ingredients {
            inventory.remove(ingredient.item, ingredient.quantity)?;
        }

        // Add output
        inventory.add(recipe.output, recipe.output_quantity)?;

        Ok(())
    }

    /// Returns all registered recipes.
    pub fn recipes(&self) -> impl Iterator<Item = &Recipe> {
        self.recipes.values()
    }
}

/// Building definition for world construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDefinition {
    /// Building type ID
    pub id: u32,
    /// Building name
    pub name: String,
    /// Width in cells
    pub width: u32,
    /// Height in cells
    pub height: u32,
    /// Required components to build
    pub components: Vec<Ingredient>,
    /// Effects on world (blocking, power, etc.)
    pub effects: BuildingEffects,
}

/// Effects a building has on the world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildingEffects {
    /// Blocks movement
    pub blocks_movement: bool,
    /// Provides power (watts)
    pub power_output: i32,
    /// Consumes power (watts)
    pub power_input: i32,
    /// Storage capacity
    pub storage_capacity: u32,
    /// Production recipe (if any)
    pub production_recipe: Option<RecipeId>,
}
