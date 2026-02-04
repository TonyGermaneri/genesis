//! Workbench types and station definitions.
//!
//! This module provides:
//! - Workbench definitions with crafting capabilities
//! - Station tiers and progression
//! - Recipe unlocks per station
//! - Specialized crafting stations (Forge, Anvil, Alchemy)

use genesis_common::RecipeId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::recipes::StationType;

// ============================================================================
// G-47: Workbench Tiers
// ============================================================================

/// Tier level for workbenches.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum WorkbenchTier {
    /// Basic tier (starter).
    #[default]
    Basic,
    /// Improved tier (mid-game).
    Improved,
    /// Advanced tier (late-game).
    Advanced,
    /// Master tier (end-game).
    Master,
}

impl WorkbenchTier {
    /// Get display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Improved => "Improved",
            Self::Advanced => "Advanced",
            Self::Master => "Master",
        }
    }

    /// Get the tier level (0-3).
    #[must_use]
    pub fn level(self) -> u8 {
        match self {
            Self::Basic => 0,
            Self::Improved => 1,
            Self::Advanced => 2,
            Self::Master => 3,
        }
    }

    /// Get all tiers in order.
    #[must_use]
    pub fn all() -> &'static [WorkbenchTier] {
        &[Self::Basic, Self::Improved, Self::Advanced, Self::Master]
    }

    /// Get next tier (if any).
    #[must_use]
    pub fn next(self) -> Option<WorkbenchTier> {
        match self {
            Self::Basic => Some(Self::Improved),
            Self::Improved => Some(Self::Advanced),
            Self::Advanced => Some(Self::Master),
            Self::Master => None,
        }
    }
}

// ============================================================================
// G-47: Workbench Definition
// ============================================================================

/// Unique identifier for a workbench.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkbenchId(pub u32);

impl WorkbenchId {
    /// Create new workbench ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Definition for a workbench/crafting station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkbenchDefinition {
    /// Unique identifier.
    pub id: WorkbenchId,
    /// Station type.
    pub station_type: StationType,
    /// Workbench name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Tier level.
    pub tier: WorkbenchTier,
    /// Recipes unlocked at this station.
    pub unlocked_recipes: HashSet<RecipeId>,
    /// Required recipes to be learned first.
    pub prerequisite_recipes: HashSet<RecipeId>,
    /// Crafting speed multiplier (1.0 = normal).
    pub speed_multiplier: f32,
    /// Quality bonus (increases chance of better output).
    pub quality_bonus: f32,
    /// Can process multiple items at once.
    pub batch_size: u32,
    /// Fuel consumption per craft (0 = no fuel).
    pub fuel_per_craft: u32,
    /// Grid size for shaped recipes.
    pub grid_size: (u8, u8),
}

impl WorkbenchDefinition {
    /// Create a builder for workbench definition.
    #[must_use]
    pub fn builder(
        id: WorkbenchId,
        station_type: StationType,
        name: impl Into<String>,
    ) -> WorkbenchBuilder {
        WorkbenchBuilder::new(id, station_type, name)
    }

    /// Check if this workbench can craft a recipe.
    #[must_use]
    pub fn can_craft_recipe(&self, recipe_id: RecipeId) -> bool {
        self.unlocked_recipes.contains(&recipe_id)
    }

    /// Check if prerequisites are met.
    #[must_use]
    pub fn prerequisites_met(&self, learned_recipes: &HashSet<RecipeId>) -> bool {
        self.prerequisite_recipes
            .iter()
            .all(|r| learned_recipes.contains(r))
    }
}

/// Builder for WorkbenchDefinition.
#[derive(Debug)]
pub struct WorkbenchBuilder {
    id: WorkbenchId,
    station_type: StationType,
    name: String,
    description: String,
    tier: WorkbenchTier,
    unlocked_recipes: HashSet<RecipeId>,
    prerequisite_recipes: HashSet<RecipeId>,
    speed_multiplier: f32,
    quality_bonus: f32,
    batch_size: u32,
    fuel_per_craft: u32,
    grid_size: (u8, u8),
}

impl WorkbenchBuilder {
    /// Create new builder.
    fn new(id: WorkbenchId, station_type: StationType, name: impl Into<String>) -> Self {
        Self {
            id,
            station_type,
            name: name.into(),
            description: String::new(),
            tier: WorkbenchTier::default(),
            unlocked_recipes: HashSet::new(),
            prerequisite_recipes: HashSet::new(),
            speed_multiplier: 1.0,
            quality_bonus: 0.0,
            batch_size: 1,
            fuel_per_craft: 0,
            grid_size: (3, 3),
        }
    }

    /// Set description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set tier.
    #[must_use]
    pub fn tier(mut self, tier: WorkbenchTier) -> Self {
        self.tier = tier;
        self
    }

    /// Add an unlocked recipe.
    #[must_use]
    pub fn unlock_recipe(mut self, recipe: RecipeId) -> Self {
        self.unlocked_recipes.insert(recipe);
        self
    }

    /// Add multiple unlocked recipes.
    #[must_use]
    pub fn unlock_recipes(mut self, recipes: impl IntoIterator<Item = RecipeId>) -> Self {
        self.unlocked_recipes.extend(recipes);
        self
    }

    /// Add a prerequisite recipe.
    #[must_use]
    pub fn prerequisite(mut self, recipe: RecipeId) -> Self {
        self.prerequisite_recipes.insert(recipe);
        self
    }

    /// Set speed multiplier.
    #[must_use]
    pub fn speed_multiplier(mut self, mult: f32) -> Self {
        self.speed_multiplier = mult;
        self
    }

    /// Set quality bonus.
    #[must_use]
    pub fn quality_bonus(mut self, bonus: f32) -> Self {
        self.quality_bonus = bonus;
        self
    }

    /// Set batch size.
    #[must_use]
    pub fn batch_size(mut self, size: u32) -> Self {
        self.batch_size = size;
        self
    }

    /// Set fuel consumption.
    #[must_use]
    pub fn fuel_per_craft(mut self, fuel: u32) -> Self {
        self.fuel_per_craft = fuel;
        self
    }

    /// Set grid size.
    #[must_use]
    pub fn grid_size(mut self, width: u8, height: u8) -> Self {
        self.grid_size = (width, height);
        self
    }

    /// Build the definition.
    #[must_use]
    pub fn build(self) -> WorkbenchDefinition {
        WorkbenchDefinition {
            id: self.id,
            station_type: self.station_type,
            name: self.name,
            description: self.description,
            tier: self.tier,
            unlocked_recipes: self.unlocked_recipes,
            prerequisite_recipes: self.prerequisite_recipes,
            speed_multiplier: self.speed_multiplier,
            quality_bonus: self.quality_bonus,
            batch_size: self.batch_size,
            fuel_per_craft: self.fuel_per_craft,
            grid_size: self.grid_size,
        }
    }
}

// ============================================================================
// G-47: Specialized Stations
// ============================================================================

/// Forge-specific data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForgeData {
    /// Current temperature (affects recipe availability).
    pub temperature: f32,
    /// Maximum temperature.
    pub max_temperature: f32,
    /// Fuel level (0.0-1.0).
    pub fuel_level: f32,
    /// Heat rate per tick.
    pub heat_rate: f32,
    /// Cool rate per tick.
    pub cool_rate: f32,
    /// Is actively heating.
    pub is_heating: bool,
}

impl ForgeData {
    /// Create new forge data.
    #[must_use]
    pub fn new(max_temp: f32) -> Self {
        Self {
            temperature: 0.0,
            max_temperature: max_temp,
            fuel_level: 0.0,
            heat_rate: 10.0,
            cool_rate: 2.0,
            is_heating: false,
        }
    }

    /// Update forge state.
    pub fn update(&mut self, delta: f32) {
        if self.is_heating && self.fuel_level > 0.0 {
            self.temperature =
                (self.temperature + self.heat_rate * delta).min(self.max_temperature);
            self.fuel_level = (self.fuel_level - 0.01 * delta).max(0.0);
        } else {
            self.temperature = (self.temperature - self.cool_rate * delta).max(0.0);
        }
    }

    /// Add fuel (normalized 0.0-1.0).
    pub fn add_fuel(&mut self, amount: f32) {
        self.fuel_level = (self.fuel_level + amount).min(1.0);
    }

    /// Check if hot enough for a recipe.
    #[must_use]
    pub fn is_hot_enough(&self, required_temp: f32) -> bool {
        self.temperature >= required_temp
    }

    /// Get temperature as percentage.
    #[must_use]
    pub fn temperature_percent(&self) -> f32 {
        if self.max_temperature > 0.0 {
            (self.temperature / self.max_temperature) * 100.0
        } else {
            0.0
        }
    }
}

/// Anvil-specific data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnvilData {
    /// Current durability (uses before repair needed).
    pub durability: u32,
    /// Maximum durability.
    pub max_durability: u32,
    /// Upgrade quality bonus.
    pub upgrade_bonus: f32,
    /// Repair efficiency.
    pub repair_efficiency: f32,
}

impl AnvilData {
    /// Create new anvil data.
    #[must_use]
    pub fn new(max_durability: u32) -> Self {
        Self {
            durability: max_durability,
            max_durability,
            upgrade_bonus: 0.0,
            repair_efficiency: 1.0,
        }
    }

    /// Use the anvil (reduces durability).
    pub fn use_anvil(&mut self) {
        self.durability = self.durability.saturating_sub(1);
    }

    /// Repair the anvil.
    pub fn repair(&mut self, amount: u32) {
        self.durability = (self.durability + amount).min(self.max_durability);
    }

    /// Check if anvil is usable.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.durability > 0
    }

    /// Get durability percentage.
    #[must_use]
    pub fn durability_percent(&self) -> f32 {
        if self.max_durability > 0 {
            (self.durability as f32 / self.max_durability as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Alchemy table-specific data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlchemyData {
    /// Current brewing progress (0.0-1.0).
    pub brew_progress: f32,
    /// Brewing speed multiplier.
    pub brew_speed: f32,
    /// Potion potency bonus.
    pub potency_bonus: f32,
    /// Chance for extra output.
    pub bonus_output_chance: f32,
    /// Currently brewing recipe.
    pub active_recipe: Option<RecipeId>,
}

impl AlchemyData {
    /// Create new alchemy data.
    #[must_use]
    pub fn new() -> Self {
        Self {
            brew_progress: 0.0,
            brew_speed: 1.0,
            potency_bonus: 0.0,
            bonus_output_chance: 0.0,
            active_recipe: None,
        }
    }

    /// Start brewing a recipe.
    pub fn start_brewing(&mut self, recipe: RecipeId) {
        self.active_recipe = Some(recipe);
        self.brew_progress = 0.0;
    }

    /// Update brewing progress.
    /// Returns true if brewing is complete.
    pub fn update(&mut self, delta: f32) -> bool {
        if self.active_recipe.is_some() {
            self.brew_progress += delta * self.brew_speed;
            if self.brew_progress >= 1.0 {
                self.brew_progress = 1.0;
                return true;
            }
        }
        false
    }

    /// Complete brewing and reset.
    pub fn complete_brewing(&mut self) -> Option<RecipeId> {
        let recipe = self.active_recipe.take();
        self.brew_progress = 0.0;
        recipe
    }

    /// Check if currently brewing.
    #[must_use]
    pub fn is_brewing(&self) -> bool {
        self.active_recipe.is_some()
    }
}

// ============================================================================
// G-47: Workbench Registry
// ============================================================================

/// Registry for all workbench definitions.
#[derive(Debug, Default)]
pub struct WorkbenchRegistry {
    /// All workbench definitions by ID.
    workbenches: HashMap<WorkbenchId, WorkbenchDefinition>,
    /// Workbenches by station type.
    by_station_type: HashMap<StationType, Vec<WorkbenchId>>,
    /// Workbenches by tier.
    by_tier: HashMap<WorkbenchTier, Vec<WorkbenchId>>,
}

impl WorkbenchRegistry {
    /// Create new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a workbench definition.
    pub fn register(&mut self, workbench: WorkbenchDefinition) {
        let id = workbench.id;

        // Index by station type
        self.by_station_type
            .entry(workbench.station_type)
            .or_default()
            .push(id);

        // Index by tier
        self.by_tier.entry(workbench.tier).or_default().push(id);

        self.workbenches.insert(id, workbench);
    }

    /// Get workbench by ID.
    #[must_use]
    pub fn get(&self, id: WorkbenchId) -> Option<&WorkbenchDefinition> {
        self.workbenches.get(&id)
    }

    /// Get all workbenches.
    pub fn all(&self) -> impl Iterator<Item = &WorkbenchDefinition> {
        self.workbenches.values()
    }

    /// Get workbenches by station type.
    #[must_use]
    pub fn by_station_type(&self, station_type: StationType) -> Vec<&WorkbenchDefinition> {
        self.by_station_type
            .get(&station_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workbenches.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get workbenches by tier.
    #[must_use]
    pub fn by_tier(&self, tier: WorkbenchTier) -> Vec<&WorkbenchDefinition> {
        self.by_tier
            .get(&tier)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.workbenches.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find workbench that can craft a specific recipe.
    #[must_use]
    pub fn find_for_recipe(&self, recipe: RecipeId) -> Option<&WorkbenchDefinition> {
        self.workbenches
            .values()
            .find(|w| w.can_craft_recipe(recipe))
    }

    /// Get all workbenches that can craft a recipe.
    #[must_use]
    pub fn all_for_recipe(&self, recipe: RecipeId) -> Vec<&WorkbenchDefinition> {
        self.workbenches
            .values()
            .filter(|w| w.can_craft_recipe(recipe))
            .collect()
    }

    /// Get registry size.
    #[must_use]
    pub fn len(&self) -> usize {
        self.workbenches.len()
    }

    /// Check if registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.workbenches.is_empty()
    }
}

// ============================================================================
// G-47: Default Workbench Definitions
// ============================================================================

/// Create default hand crafting station (no workbench).
#[must_use]
pub fn create_hand_crafting() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(0), StationType::None, "Hand Crafting")
        .description("Basic crafting using only your hands")
        .tier(WorkbenchTier::Basic)
        .grid_size(2, 2)
        .build()
}

/// Create basic crafting table.
#[must_use]
pub fn create_crafting_table() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(
        WorkbenchId::new(1),
        StationType::CraftingTable,
        "Crafting Table",
    )
    .description("Standard crafting table with 3x3 grid")
    .tier(WorkbenchTier::Basic)
    .grid_size(3, 3)
    .build()
}

/// Create basic forge.
#[must_use]
pub fn create_basic_forge() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(10), StationType::Forge, "Basic Forge")
        .description("A simple forge for smelting ore and crafting metal items")
        .tier(WorkbenchTier::Basic)
        .fuel_per_craft(1)
        .grid_size(2, 2)
        .build()
}

/// Create improved forge.
#[must_use]
pub fn create_improved_forge() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(11), StationType::Forge, "Improved Forge")
        .description("An upgraded forge with better fuel efficiency")
        .tier(WorkbenchTier::Improved)
        .speed_multiplier(1.25)
        .fuel_per_craft(1)
        .grid_size(3, 3)
        .build()
}

/// Create basic anvil.
#[must_use]
pub fn create_basic_anvil() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(20), StationType::Anvil, "Basic Anvil")
        .description("An anvil for smithing weapons and armor")
        .tier(WorkbenchTier::Basic)
        .grid_size(2, 2)
        .build()
}

/// Create master anvil.
#[must_use]
pub fn create_master_anvil() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(21), StationType::Anvil, "Master Anvil")
        .description("A master-quality anvil with superior crafting capabilities")
        .tier(WorkbenchTier::Master)
        .quality_bonus(0.25)
        .speed_multiplier(1.5)
        .grid_size(3, 3)
        .build()
}

/// Create basic alchemy table.
#[must_use]
pub fn create_basic_alchemy_table() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(
        WorkbenchId::new(30),
        StationType::AlchemyTable,
        "Alchemy Table",
    )
    .description("A table for brewing potions and elixirs")
    .tier(WorkbenchTier::Basic)
    .grid_size(2, 3)
    .build()
}

/// Create advanced alchemy table.
#[must_use]
pub fn create_advanced_alchemy_table() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(
        WorkbenchId::new(31),
        StationType::AlchemyTable,
        "Advanced Alchemy Lab",
    )
    .description("An advanced alchemy laboratory with enhanced capabilities")
    .tier(WorkbenchTier::Advanced)
    .speed_multiplier(1.5)
    .quality_bonus(0.15)
    .batch_size(2)
    .grid_size(3, 3)
    .build()
}

/// Create cooking pot.
#[must_use]
pub fn create_cooking_pot() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(40), StationType::CookingPot, "Cooking Pot")
        .description("A pot for cooking food and preparing meals")
        .tier(WorkbenchTier::Basic)
        .fuel_per_craft(1)
        .grid_size(2, 2)
        .build()
}

/// Create sawmill.
#[must_use]
pub fn create_sawmill() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(50), StationType::Sawmill, "Sawmill")
        .description("A sawmill for processing wood and lumber")
        .tier(WorkbenchTier::Improved)
        .speed_multiplier(2.0)
        .batch_size(4)
        .build()
}

/// Create loom.
#[must_use]
pub fn create_loom() -> WorkbenchDefinition {
    WorkbenchDefinition::builder(WorkbenchId::new(60), StationType::Loom, "Loom")
        .description("A loom for weaving cloth and fabric")
        .tier(WorkbenchTier::Basic)
        .grid_size(3, 2)
        .build()
}

/// Create all default workbenches and register them.
pub fn register_default_workbenches(registry: &mut WorkbenchRegistry) {
    registry.register(create_hand_crafting());
    registry.register(create_crafting_table());
    registry.register(create_basic_forge());
    registry.register(create_improved_forge());
    registry.register(create_basic_anvil());
    registry.register(create_master_anvil());
    registry.register(create_basic_alchemy_table());
    registry.register(create_advanced_alchemy_table());
    registry.register(create_cooking_pot());
    registry.register(create_sawmill());
    registry.register(create_loom());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_recipe_id(id: u32) -> RecipeId {
        RecipeId::new(id)
    }

    #[test]
    fn test_workbench_tier_display() {
        assert_eq!(WorkbenchTier::Basic.display_name(), "Basic");
        assert_eq!(WorkbenchTier::Master.display_name(), "Master");
    }

    #[test]
    fn test_workbench_tier_level() {
        assert_eq!(WorkbenchTier::Basic.level(), 0);
        assert_eq!(WorkbenchTier::Improved.level(), 1);
        assert_eq!(WorkbenchTier::Advanced.level(), 2);
        assert_eq!(WorkbenchTier::Master.level(), 3);
    }

    #[test]
    fn test_workbench_tier_next() {
        assert_eq!(WorkbenchTier::Basic.next(), Some(WorkbenchTier::Improved));
        assert_eq!(WorkbenchTier::Master.next(), None);
    }

    #[test]
    fn test_workbench_builder() {
        let workbench =
            WorkbenchDefinition::builder(WorkbenchId::new(1), StationType::Forge, "Test Forge")
                .description("A test forge")
                .tier(WorkbenchTier::Improved)
                .unlock_recipe(test_recipe_id(1))
                .unlock_recipe(test_recipe_id(2))
                .speed_multiplier(1.5)
                .fuel_per_craft(2)
                .build();

        assert_eq!(workbench.id, WorkbenchId::new(1));
        assert_eq!(workbench.name, "Test Forge");
        assert_eq!(workbench.tier, WorkbenchTier::Improved);
        assert_eq!(workbench.unlocked_recipes.len(), 2);
        assert_eq!(workbench.speed_multiplier, 1.5);
        assert_eq!(workbench.fuel_per_craft, 2);
    }

    #[test]
    fn test_workbench_can_craft() {
        let workbench =
            WorkbenchDefinition::builder(WorkbenchId::new(1), StationType::CraftingTable, "Table")
                .unlock_recipe(test_recipe_id(10))
                .build();

        assert!(workbench.can_craft_recipe(test_recipe_id(10)));
        assert!(!workbench.can_craft_recipe(test_recipe_id(20)));
    }

    #[test]
    fn test_workbench_prerequisites() {
        let workbench =
            WorkbenchDefinition::builder(WorkbenchId::new(1), StationType::Anvil, "Anvil")
                .prerequisite(test_recipe_id(1))
                .prerequisite(test_recipe_id(2))
                .build();

        let mut learned = HashSet::new();
        assert!(!workbench.prerequisites_met(&learned));

        learned.insert(test_recipe_id(1));
        assert!(!workbench.prerequisites_met(&learned));

        learned.insert(test_recipe_id(2));
        assert!(workbench.prerequisites_met(&learned));
    }

    #[test]
    fn test_forge_data() {
        let mut forge = ForgeData::new(1000.0);
        assert_eq!(forge.temperature, 0.0);
        assert!(!forge.is_hot_enough(500.0));

        forge.add_fuel(0.5);
        forge.is_heating = true;
        forge.update(10.0); // Heat for 10 seconds

        assert!(forge.temperature > 0.0);
    }

    #[test]
    fn test_forge_temperature_percent() {
        let mut forge = ForgeData::new(1000.0);
        forge.temperature = 500.0;
        assert_eq!(forge.temperature_percent(), 50.0);
    }

    #[test]
    fn test_anvil_data() {
        let mut anvil = AnvilData::new(100);
        assert!(anvil.is_usable());
        assert_eq!(anvil.durability_percent(), 100.0);

        anvil.use_anvil();
        assert_eq!(anvil.durability, 99);

        anvil.repair(10);
        assert_eq!(anvil.durability, 100); // Capped at max
    }

    #[test]
    fn test_alchemy_data() {
        let mut alchemy = AlchemyData::new();
        assert!(!alchemy.is_brewing());

        alchemy.start_brewing(test_recipe_id(1));
        assert!(alchemy.is_brewing());
        assert_eq!(alchemy.brew_progress, 0.0);

        let complete = alchemy.update(0.5);
        assert!(!complete);
        assert!(alchemy.brew_progress > 0.0);

        let complete = alchemy.update(1.0);
        assert!(complete);

        let recipe = alchemy.complete_brewing();
        assert_eq!(recipe, Some(test_recipe_id(1)));
        assert!(!alchemy.is_brewing());
    }

    #[test]
    fn test_workbench_registry() {
        let mut registry = WorkbenchRegistry::new();

        let forge = WorkbenchDefinition::builder(WorkbenchId::new(1), StationType::Forge, "Forge")
            .tier(WorkbenchTier::Basic)
            .build();

        let anvil = WorkbenchDefinition::builder(WorkbenchId::new(2), StationType::Anvil, "Anvil")
            .tier(WorkbenchTier::Improved)
            .build();

        registry.register(forge);
        registry.register(anvil);

        assert_eq!(registry.len(), 2);
        assert!(registry.get(WorkbenchId::new(1)).is_some());
    }

    #[test]
    fn test_registry_by_station_type() {
        let mut registry = WorkbenchRegistry::new();

        registry.register(
            WorkbenchDefinition::builder(WorkbenchId::new(1), StationType::Forge, "Forge 1")
                .build(),
        );
        registry.register(
            WorkbenchDefinition::builder(WorkbenchId::new(2), StationType::Forge, "Forge 2")
                .build(),
        );
        registry.register(
            WorkbenchDefinition::builder(WorkbenchId::new(3), StationType::Anvil, "Anvil").build(),
        );

        let forges = registry.by_station_type(StationType::Forge);
        assert_eq!(forges.len(), 2);

        let anvils = registry.by_station_type(StationType::Anvil);
        assert_eq!(anvils.len(), 1);
    }

    #[test]
    fn test_registry_by_tier() {
        let mut registry = WorkbenchRegistry::new();

        registry.register(
            WorkbenchDefinition::builder(WorkbenchId::new(1), StationType::Forge, "Basic")
                .tier(WorkbenchTier::Basic)
                .build(),
        );
        registry.register(
            WorkbenchDefinition::builder(WorkbenchId::new(2), StationType::Forge, "Advanced")
                .tier(WorkbenchTier::Advanced)
                .build(),
        );

        let basic = registry.by_tier(WorkbenchTier::Basic);
        assert_eq!(basic.len(), 1);
        assert_eq!(basic[0].name, "Basic");
    }

    #[test]
    fn test_registry_find_for_recipe() {
        let mut registry = WorkbenchRegistry::new();

        registry.register(
            WorkbenchDefinition::builder(WorkbenchId::new(1), StationType::Forge, "Forge")
                .unlock_recipe(test_recipe_id(100))
                .build(),
        );

        let found = registry.find_for_recipe(test_recipe_id(100));
        assert!(found.is_some());
        assert_eq!(found.map(|w| &w.name), Some(&"Forge".to_string()));

        let not_found = registry.find_for_recipe(test_recipe_id(200));
        assert!(not_found.is_none());
    }

    #[test]
    fn test_default_workbenches() {
        let mut registry = WorkbenchRegistry::new();
        register_default_workbenches(&mut registry);

        assert!(registry.len() >= 10);
        assert!(registry.get(WorkbenchId::new(0)).is_some()); // Hand crafting
        assert!(registry.get(WorkbenchId::new(1)).is_some()); // Crafting table
        assert!(registry.get(WorkbenchId::new(10)).is_some()); // Basic forge
    }

    #[test]
    fn test_create_hand_crafting() {
        let hand = create_hand_crafting();
        assert_eq!(hand.station_type, StationType::None);
        assert_eq!(hand.grid_size, (2, 2));
    }

    #[test]
    fn test_create_crafting_table() {
        let table = create_crafting_table();
        assert_eq!(table.station_type, StationType::CraftingTable);
        assert_eq!(table.grid_size, (3, 3));
    }
}
