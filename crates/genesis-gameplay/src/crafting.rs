//! Crafting system for items and buildings.

use genesis_common::{ItemTypeId, RecipeId, WorldCoord};
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
    /// Missing specific ingredient
    #[error("Missing ingredient: item {item:?}, need {needed}, have {have}")]
    MissingIngredient {
        /// Item that's missing
        item: ItemTypeId,
        /// Amount needed
        needed: u32,
        /// Amount available
        have: u32,
    },
    /// Missing ingredients (generic)
    #[error("Missing ingredients")]
    MissingIngredients,
    /// Missing tool
    #[error("Missing required tool: {0:?}")]
    MissingTool(ItemTypeId),
    /// Skill too low
    #[error("Skill too low: need {required}, have {current}")]
    SkillTooLow {
        /// Required skill level
        required: u32,
        /// Current skill level
        current: u32,
    },
    /// Inventory full (can't add output)
    #[error("Inventory full: cannot add crafted item")]
    InventoryFull,
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

impl Ingredient {
    /// Creates a new ingredient requirement.
    #[must_use]
    pub const fn new(item: ItemTypeId, quantity: u32) -> Self {
        Self { item, quantity }
    }
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

impl Recipe {
    /// Creates a new recipe builder.
    #[must_use]
    pub fn builder(id: RecipeId, name: impl Into<String>) -> RecipeBuilder {
        RecipeBuilder::new(id, name)
    }
}

/// Builder for creating recipes.
#[derive(Debug)]
pub struct RecipeBuilder {
    id: RecipeId,
    name: String,
    ingredients: Vec<Ingredient>,
    tools: Vec<ItemTypeId>,
    output: Option<ItemTypeId>,
    output_quantity: u32,
    skill_required: u32,
    skill_gain: u32,
    craft_time: u32,
}

impl RecipeBuilder {
    /// Creates a new recipe builder.
    fn new(id: RecipeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            ingredients: Vec::new(),
            tools: Vec::new(),
            output: None,
            output_quantity: 1,
            skill_required: 0,
            skill_gain: 0,
            craft_time: 0,
        }
    }

    /// Adds an ingredient requirement.
    #[must_use]
    pub fn ingredient(mut self, item: ItemTypeId, quantity: u32) -> Self {
        self.ingredients.push(Ingredient::new(item, quantity));
        self
    }

    /// Adds a tool requirement (not consumed).
    #[must_use]
    pub fn tool(mut self, item: ItemTypeId) -> Self {
        self.tools.push(item);
        self
    }

    /// Sets the output item and quantity.
    #[must_use]
    pub fn output(mut self, item: ItemTypeId, quantity: u32) -> Self {
        self.output = Some(item);
        self.output_quantity = quantity;
        self
    }

    /// Sets the required skill level.
    #[must_use]
    pub const fn skill_required(mut self, level: u32) -> Self {
        self.skill_required = level;
        self
    }

    /// Sets the skill gained on craft.
    #[must_use]
    pub const fn skill_gain(mut self, gain: u32) -> Self {
        self.skill_gain = gain;
        self
    }

    /// Sets the craft time in ticks.
    #[must_use]
    pub const fn craft_time(mut self, ticks: u32) -> Self {
        self.craft_time = ticks;
        self
    }

    /// Builds the recipe.
    ///
    /// # Panics
    /// Panics if output item is not set.
    #[must_use]
    pub fn build(self) -> Recipe {
        Recipe {
            id: self.id,
            name: self.name,
            ingredients: self.ingredients,
            tools: self.tools,
            output: self.output.expect("Recipe must have an output item"),
            output_quantity: self.output_quantity,
            skill_required: self.skill_required,
            skill_gain: self.skill_gain,
            craft_time: self.craft_time,
        }
    }
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

    /// Returns the number of registered recipes.
    #[must_use]
    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }

    /// Validates and checks if a recipe can be crafted.
    ///
    /// Returns `Ok(())` if all requirements are met, or a specific error
    /// describing what's missing.
    pub fn validate_craft(
        &self,
        recipe_id: RecipeId,
        inventory: &Inventory,
        skill_level: u32,
    ) -> CraftingResult<()> {
        let recipe = self
            .recipes
            .get(&recipe_id)
            .ok_or(CraftingError::RecipeNotFound(recipe_id))?;

        // Check skill requirement
        if skill_level < recipe.skill_required {
            return Err(CraftingError::SkillTooLow {
                required: recipe.skill_required,
                current: skill_level,
            });
        }

        // Check each ingredient
        for ingredient in &recipe.ingredients {
            let have = inventory.count(ingredient.item);
            if have < ingredient.quantity {
                return Err(CraftingError::MissingIngredient {
                    item: ingredient.item,
                    needed: ingredient.quantity,
                    have,
                });
            }
        }

        // Check each tool
        for tool in &recipe.tools {
            if !inventory.has(*tool, 1) {
                return Err(CraftingError::MissingTool(*tool));
            }
        }

        // Check if output can be added
        if !inventory.can_add(recipe.output, recipe.output_quantity) {
            return Err(CraftingError::InventoryFull);
        }

        Ok(())
    }

    /// Checks if a recipe can be crafted with the given inventory.
    pub fn can_craft(
        &self,
        recipe_id: RecipeId,
        inventory: &Inventory,
        skill_level: u32,
    ) -> CraftingResult<bool> {
        match self.validate_craft(recipe_id, inventory, skill_level) {
            Ok(()) => Ok(true),
            Err(
                CraftingError::MissingIngredient { .. }
                | CraftingError::MissingTool(_)
                | CraftingError::InventoryFull,
            ) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Executes a craft operation.
    ///
    /// Validates all requirements, consumes ingredients, and produces output.
    pub fn craft(
        &self,
        recipe_id: RecipeId,
        inventory: &mut Inventory,
        skill_level: u32,
    ) -> CraftingResult<()> {
        // Validate first
        self.validate_craft(recipe_id, inventory, skill_level)?;

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

    /// Finds recipes that can be crafted with the given inventory.
    pub fn available_recipes<'a>(
        &'a self,
        inventory: &'a Inventory,
        skill_level: u32,
    ) -> impl Iterator<Item = &'a Recipe> {
        self.recipes.values().filter(move |recipe| {
            self.can_craft(recipe.id, inventory, skill_level)
                .unwrap_or(false)
        })
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

impl BuildingDefinition {
    /// Creates a new building definition builder.
    #[must_use]
    pub fn builder(id: u32, name: impl Into<String>) -> BuildingDefinitionBuilder {
        BuildingDefinitionBuilder::new(id, name)
    }
}

/// Builder for creating building definitions.
#[derive(Debug)]
pub struct BuildingDefinitionBuilder {
    id: u32,
    name: String,
    width: u32,
    height: u32,
    components: Vec<Ingredient>,
    effects: BuildingEffects,
}

impl BuildingDefinitionBuilder {
    /// Creates a new builder.
    fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            width: 1,
            height: 1,
            components: Vec::new(),
            effects: BuildingEffects::default(),
        }
    }

    /// Sets the building dimensions.
    #[must_use]
    pub const fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Adds a required component.
    #[must_use]
    pub fn component(mut self, item: ItemTypeId, quantity: u32) -> Self {
        self.components.push(Ingredient::new(item, quantity));
        self
    }

    /// Sets whether the building blocks movement.
    #[must_use]
    pub const fn blocks_movement(mut self, blocks: bool) -> Self {
        self.effects.blocks_movement = blocks;
        self
    }

    /// Sets the power output.
    #[must_use]
    pub const fn power_output(mut self, watts: i32) -> Self {
        self.effects.power_output = watts;
        self
    }

    /// Sets the power input.
    #[must_use]
    pub const fn power_input(mut self, watts: i32) -> Self {
        self.effects.power_input = watts;
        self
    }

    /// Sets the storage capacity.
    #[must_use]
    pub const fn storage_capacity(mut self, capacity: u32) -> Self {
        self.effects.storage_capacity = capacity;
        self
    }

    /// Sets a production recipe.
    #[must_use]
    pub const fn production_recipe(mut self, recipe: RecipeId) -> Self {
        self.effects.production_recipe = Some(recipe);
        self
    }

    /// Builds the definition.
    #[must_use]
    pub fn build(self) -> BuildingDefinition {
        BuildingDefinition {
            id: self.id,
            name: self.name,
            width: self.width,
            height: self.height,
            components: self.components,
            effects: self.effects,
        }
    }
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

/// Error types for building placement.
#[derive(Debug, Error)]
pub enum BuildingError {
    /// Building definition not found
    #[error("Building definition not found: {0}")]
    DefinitionNotFound(u32),
    /// Missing component
    #[error("Missing component: item {item:?}, need {needed}, have {have}")]
    MissingComponent {
        /// Item that's missing
        item: ItemTypeId,
        /// Amount needed
        needed: u32,
        /// Amount available
        have: u32,
    },
    /// Position is occupied
    #[error("Position occupied at ({x}, {y})")]
    PositionOccupied {
        /// X coordinate
        x: i64,
        /// Y coordinate
        y: i64,
    },
    /// Invalid position
    #[error("Invalid position")]
    InvalidPosition,
    /// Inventory error
    #[error("Inventory error: {0}")]
    Inventory(#[from] InventoryError),
}

/// Result type for building operations.
pub type BuildingResult<T> = Result<T, BuildingError>;

/// Intent to place a building in the world.
///
/// This is sent to the kernel for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingPlacement {
    /// Building definition ID
    pub building_id: u32,
    /// Building name (for display/debug)
    pub building_name: String,
    /// Position in world coordinates
    pub position: WorldCoord,
    /// Building width
    pub width: u32,
    /// Building height
    pub height: u32,
    /// Building effects
    pub effects: BuildingEffects,
}

/// Building placement system.
#[derive(Debug, Default)]
pub struct BuildingSystem {
    /// All building definitions
    definitions: HashMap<u32, BuildingDefinition>,
}

impl BuildingSystem {
    /// Creates a new building system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    /// Registers a building definition.
    pub fn register_building(&mut self, def: BuildingDefinition) {
        self.definitions.insert(def.id, def);
    }

    /// Gets a building definition by ID.
    #[must_use]
    pub fn get_definition(&self, id: u32) -> Option<&BuildingDefinition> {
        self.definitions.get(&id)
    }

    /// Returns the number of registered buildings.
    #[must_use]
    pub fn building_count(&self) -> usize {
        self.definitions.len()
    }

    /// Validates building placement.
    ///
    /// Note: Position validation is a placeholder - actual collision detection
    /// would be done by the kernel/world system.
    pub fn validate_placement(
        &self,
        building_id: u32,
        inventory: &Inventory,
        _position: WorldCoord,
    ) -> BuildingResult<()> {
        let def = self
            .definitions
            .get(&building_id)
            .ok_or(BuildingError::DefinitionNotFound(building_id))?;

        // Check components
        for component in &def.components {
            let have = inventory.count(component.item);
            if have < component.quantity {
                return Err(BuildingError::MissingComponent {
                    item: component.item,
                    needed: component.quantity,
                    have,
                });
            }
        }

        // Position validation would go here - currently a placeholder
        // In a real system, this would check for collisions with other buildings,
        // terrain, etc.

        Ok(())
    }

    /// Checks if a building can be placed.
    #[must_use]
    pub fn can_place(&self, building_id: u32, inventory: &Inventory, position: WorldCoord) -> bool {
        self.validate_placement(building_id, inventory, position)
            .is_ok()
    }

    /// Creates a building placement intent.
    ///
    /// Validates requirements and consumes components from inventory.
    /// Returns an intent that should be sent to the kernel.
    pub fn place(
        &self,
        building_id: u32,
        inventory: &mut Inventory,
        position: WorldCoord,
    ) -> BuildingResult<BuildingPlacement> {
        // Validate first
        self.validate_placement(building_id, inventory, position)?;

        let def = self
            .definitions
            .get(&building_id)
            .ok_or(BuildingError::DefinitionNotFound(building_id))?;

        // Consume components
        for component in &def.components {
            inventory.remove(component.item, component.quantity)?;
        }

        // Create placement intent
        Ok(BuildingPlacement {
            building_id: def.id,
            building_name: def.name.clone(),
            position,
            width: def.width,
            height: def.height,
            effects: def.effects.clone(),
        })
    }

    /// Returns all registered building definitions.
    pub fn buildings(&self) -> impl Iterator<Item = &BuildingDefinition> {
        self.definitions.values()
    }

    /// Finds buildings that can be built with the given inventory.
    pub fn available_buildings<'a>(
        &'a self,
        inventory: &'a Inventory,
    ) -> impl Iterator<Item = &'a BuildingDefinition> {
        self.definitions.values().filter(move |def| {
            def.components
                .iter()
                .all(|c| inventory.has(c.item, c.quantity))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Crafting Tests ===

    fn create_test_recipe() -> Recipe {
        Recipe::builder(RecipeId::new(1), "Wooden Pickaxe")
            .ingredient(ItemTypeId::new(1), 3) // Wood
            .ingredient(ItemTypeId::new(2), 2) // Stick
            .output(ItemTypeId::new(10), 1) // Wooden Pickaxe
            .skill_required(0)
            .build()
    }

    fn create_skill_recipe() -> Recipe {
        Recipe::builder(RecipeId::new(2), "Iron Sword")
            .ingredient(ItemTypeId::new(3), 5) // Iron
            .tool(ItemTypeId::new(20)) // Hammer
            .output(ItemTypeId::new(11), 1) // Iron Sword
            .skill_required(10)
            .skill_gain(1)
            .build()
    }

    #[test]
    fn test_recipe_builder() {
        let recipe = create_test_recipe();
        assert_eq!(recipe.name, "Wooden Pickaxe");
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.output_quantity, 1);
    }

    #[test]
    fn test_crafting_system_register() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_test_recipe());
        assert_eq!(system.recipe_count(), 1);
        assert!(system.get_recipe(RecipeId::new(1)).is_some());
    }

    #[test]
    fn test_can_craft_success() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_test_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 5); // Wood
        let _ = inventory.add(ItemTypeId::new(2), 5); // Stick

        assert!(system
            .can_craft(RecipeId::new(1), &inventory, 0)
            .expect("should not error"));
    }

    #[test]
    fn test_can_craft_missing_ingredient() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_test_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 2); // Not enough wood

        assert!(!system
            .can_craft(RecipeId::new(1), &inventory, 0)
            .expect("should not error"));
    }

    #[test]
    fn test_validate_craft_missing_ingredient() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_test_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 2); // Only 2 wood, need 3

        let result = system.validate_craft(RecipeId::new(1), &inventory, 0);
        assert!(matches!(
            result,
            Err(CraftingError::MissingIngredient {
                needed: 3,
                have: 2,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_craft_missing_tool() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_skill_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(3), 10); // Iron
                                                       // No hammer

        let result = system.validate_craft(RecipeId::new(2), &inventory, 10);
        assert!(matches!(result, Err(CraftingError::MissingTool(_))));
    }

    #[test]
    fn test_validate_craft_skill_too_low() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_skill_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(3), 10); // Iron
        let _ = inventory.add(ItemTypeId::new(20), 1); // Hammer

        let result = system.validate_craft(RecipeId::new(2), &inventory, 5);
        assert!(matches!(
            result,
            Err(CraftingError::SkillTooLow {
                required: 10,
                current: 5
            })
        ));
    }

    #[test]
    fn test_craft_success() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_test_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 5); // Wood
        let _ = inventory.add(ItemTypeId::new(2), 5); // Stick

        let result = system.craft(RecipeId::new(1), &mut inventory, 0);
        assert!(result.is_ok());

        // Check ingredients consumed
        assert_eq!(inventory.count(ItemTypeId::new(1)), 2); // 5 - 3
        assert_eq!(inventory.count(ItemTypeId::new(2)), 3); // 5 - 2

        // Check output produced
        assert_eq!(inventory.count(ItemTypeId::new(10)), 1);
    }

    #[test]
    fn test_craft_with_tool() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_skill_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(3), 10); // Iron
        let _ = inventory.add(ItemTypeId::new(20), 1); // Hammer (tool)

        let result = system.craft(RecipeId::new(2), &mut inventory, 10);
        assert!(result.is_ok());

        // Tool not consumed
        assert_eq!(inventory.count(ItemTypeId::new(20)), 1);
        // Iron consumed
        assert_eq!(inventory.count(ItemTypeId::new(3)), 5);
        // Sword produced
        assert_eq!(inventory.count(ItemTypeId::new(11)), 1);
    }

    #[test]
    fn test_craft_inventory_full() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_test_recipe());

        // Create tiny inventory that's already full
        let mut inventory = Inventory::with_stack_limit(2, 999);
        let _ = inventory.add(ItemTypeId::new(1), 5); // Wood
        let _ = inventory.add(ItemTypeId::new(2), 5); // Stick (fills both slots)

        let result = system.validate_craft(RecipeId::new(1), &inventory, 0);
        assert!(matches!(result, Err(CraftingError::InventoryFull)));
    }

    #[test]
    fn test_craft_recipe_not_found() {
        let system = CraftingSystem::new();
        let inventory = Inventory::new(10);

        let result = system.can_craft(RecipeId::new(999), &inventory, 0);
        assert!(matches!(result, Err(CraftingError::RecipeNotFound(_))));
    }

    #[test]
    fn test_available_recipes() {
        let mut system = CraftingSystem::new();
        system.register_recipe(create_test_recipe());
        system.register_recipe(create_skill_recipe());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 5);
        let _ = inventory.add(ItemTypeId::new(2), 5);

        // Should only find wooden pickaxe (no iron/hammer for sword)
        let available: Vec<_> = system.available_recipes(&inventory, 10).collect();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].name, "Wooden Pickaxe");
    }

    // === Building Tests ===

    fn create_test_building() -> BuildingDefinition {
        BuildingDefinition::builder(1, "Storage Chest")
            .dimensions(1, 1)
            .component(ItemTypeId::new(1), 10) // Wood
            .storage_capacity(100)
            .build()
    }

    fn create_power_building() -> BuildingDefinition {
        BuildingDefinition::builder(2, "Solar Panel")
            .dimensions(2, 2)
            .component(ItemTypeId::new(3), 5) // Iron
            .component(ItemTypeId::new(4), 2) // Glass
            .power_output(100)
            .build()
    }

    #[test]
    fn test_building_definition_builder() {
        let building = create_test_building();
        assert_eq!(building.name, "Storage Chest");
        assert_eq!(building.width, 1);
        assert_eq!(building.height, 1);
        assert_eq!(building.effects.storage_capacity, 100);
    }

    #[test]
    fn test_building_system_register() {
        let mut system = BuildingSystem::new();
        system.register_building(create_test_building());
        assert_eq!(system.building_count(), 1);
        assert!(system.get_definition(1).is_some());
    }

    #[test]
    fn test_can_place_success() {
        let mut system = BuildingSystem::new();
        system.register_building(create_test_building());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 20); // Wood

        assert!(system.can_place(1, &inventory, WorldCoord::new(0, 0)));
    }

    #[test]
    fn test_can_place_missing_component() {
        let mut system = BuildingSystem::new();
        system.register_building(create_test_building());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 5); // Not enough wood

        assert!(!system.can_place(1, &inventory, WorldCoord::new(0, 0)));
    }

    #[test]
    fn test_validate_placement_missing_component() {
        let mut system = BuildingSystem::new();
        system.register_building(create_test_building());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 5); // Only 5, need 10

        let result = system.validate_placement(1, &inventory, WorldCoord::new(0, 0));
        assert!(matches!(
            result,
            Err(BuildingError::MissingComponent {
                needed: 10,
                have: 5,
                ..
            })
        ));
    }

    #[test]
    fn test_place_success() {
        let mut system = BuildingSystem::new();
        system.register_building(create_test_building());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 20); // Wood

        let placement = system
            .place(1, &mut inventory, WorldCoord::new(100, 200))
            .expect("should succeed");

        // Check placement intent
        assert_eq!(placement.building_id, 1);
        assert_eq!(placement.building_name, "Storage Chest");
        assert_eq!(placement.position.x, 100);
        assert_eq!(placement.position.y, 200);
        assert_eq!(placement.effects.storage_capacity, 100);

        // Check components consumed
        assert_eq!(inventory.count(ItemTypeId::new(1)), 10); // 20 - 10
    }

    #[test]
    fn test_place_multiple_components() {
        let mut system = BuildingSystem::new();
        system.register_building(create_power_building());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(3), 10); // Iron
        let _ = inventory.add(ItemTypeId::new(4), 5); // Glass

        let placement = system
            .place(2, &mut inventory, WorldCoord::new(0, 0))
            .expect("should succeed");

        assert_eq!(placement.building_name, "Solar Panel");
        assert_eq!(placement.width, 2);
        assert_eq!(placement.height, 2);
        assert_eq!(placement.effects.power_output, 100);

        // Check components consumed
        assert_eq!(inventory.count(ItemTypeId::new(3)), 5); // 10 - 5
        assert_eq!(inventory.count(ItemTypeId::new(4)), 3); // 5 - 2
    }

    #[test]
    fn test_place_definition_not_found() {
        let system = BuildingSystem::new();
        let mut inventory = Inventory::new(10);

        let result = system.place(999, &mut inventory, WorldCoord::new(0, 0));
        assert!(matches!(
            result,
            Err(BuildingError::DefinitionNotFound(999))
        ));
    }

    #[test]
    fn test_available_buildings() {
        let mut system = BuildingSystem::new();
        system.register_building(create_test_building());
        system.register_building(create_power_building());

        let mut inventory = Inventory::new(10);
        let _ = inventory.add(ItemTypeId::new(1), 20); // Wood only

        // Should only find storage chest (no iron/glass for solar panel)
        let available: Vec<_> = system.available_buildings(&inventory).collect();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].name, "Storage Chest");
    }
}
