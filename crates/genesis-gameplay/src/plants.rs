//! Plant growth system for farming and vegetation.
//!
//! This module provides plant growth mechanics:
//! - Growth stages affected by light, water, and weather
//! - Harvestable mature plants
//! - Integration with grass cutting (grass regrowth)
//! - Crop farming system

use genesis_common::{ItemTypeId, WorldCoord};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Growth stage of a plant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GrowthStage {
    /// Just planted (seed).
    #[default]
    Seed,
    /// Small sprout visible.
    Sprout,
    /// Growing plant, not yet mature.
    Growing,
    /// Fully grown, can be harvested.
    Mature,
    /// Overgrown/wilted (missed harvest window).
    Wilted,
    /// Dead (no longer harvestable).
    Dead,
}

impl GrowthStage {
    /// Get the display name of this stage.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Seed => "Seed",
            Self::Sprout => "Sprout",
            Self::Growing => "Growing",
            Self::Mature => "Mature",
            Self::Wilted => "Wilted",
            Self::Dead => "Dead",
        }
    }

    /// Check if this plant can be harvested.
    #[must_use]
    pub fn is_harvestable(self) -> bool {
        matches!(self, Self::Mature | Self::Wilted)
    }

    /// Check if this plant is dead and should be removed.
    #[must_use]
    pub fn is_dead(self) -> bool {
        matches!(self, Self::Dead)
    }

    /// Get the next growth stage.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Seed => Self::Sprout,
            Self::Sprout => Self::Growing,
            Self::Growing => Self::Mature,
            Self::Mature => Self::Wilted,
            Self::Wilted | Self::Dead => Self::Dead,
        }
    }
}

/// Unique identifier for a plant type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlantTypeId(pub u32);

impl PlantTypeId {
    /// Create a new plant type ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Well-known plant type IDs.
pub mod plant_types {
    use super::PlantTypeId;

    /// Wild grass (regrows naturally).
    pub const GRASS: PlantTypeId = PlantTypeId::new(1);
    /// Wheat crop.
    pub const WHEAT: PlantTypeId = PlantTypeId::new(2);
    /// Corn crop.
    pub const CORN: PlantTypeId = PlantTypeId::new(3);
    /// Carrot vegetable.
    pub const CARROT: PlantTypeId = PlantTypeId::new(4);
    /// Potato vegetable.
    pub const POTATO: PlantTypeId = PlantTypeId::new(5);
    /// Tomato fruit.
    pub const TOMATO: PlantTypeId = PlantTypeId::new(6);
    /// Berry bush.
    pub const BERRY_BUSH: PlantTypeId = PlantTypeId::new(7);
    /// Tree (generic).
    pub const TREE: PlantTypeId = PlantTypeId::new(8);
}

/// Definition of a plant type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantDefinition {
    /// Unique identifier.
    pub id: PlantTypeId,
    /// Display name.
    pub name: String,
    /// Time to grow from seed to sprout (game minutes).
    pub seed_to_sprout_time: f32,
    /// Time to grow from sprout to growing (game minutes).
    pub sprout_to_growing_time: f32,
    /// Time to grow from growing to mature (game minutes).
    pub growing_to_mature_time: f32,
    /// Time from mature to wilted (game minutes, 0 = doesn't wilt).
    pub mature_to_wilted_time: f32,
    /// Time from wilted to dead (game minutes).
    pub wilted_to_dead_time: f32,
    /// Minimum light level needed (0.0 to 1.0).
    pub min_light: f32,
    /// Ideal light level for fastest growth.
    pub ideal_light: f32,
    /// Whether this plant needs water (soil moisture).
    pub needs_water: bool,
    /// How much the plant benefits from rain (multiplier).
    pub rain_bonus: f32,
    /// Item dropped when harvested (at mature stage).
    pub harvest_item: ItemTypeId,
    /// Amount harvested per plant.
    pub harvest_amount: u32,
    /// Item dropped when harvested while wilted (reduced yield).
    pub wilted_harvest_amount: u32,
    /// Seed item needed to plant.
    pub seed_item: ItemTypeId,
    /// Whether this plant regrows after harvest.
    pub regrows_after_harvest: bool,
}

impl PlantDefinition {
    /// Create a new plant definition builder.
    #[must_use]
    pub fn builder(id: PlantTypeId, name: &str) -> PlantDefinitionBuilder {
        PlantDefinitionBuilder::new(id, name)
    }

    /// Get the total time from seed to mature (game minutes).
    #[must_use]
    pub fn total_growth_time(&self) -> f32 {
        self.seed_to_sprout_time + self.sprout_to_growing_time + self.growing_to_mature_time
    }
}

/// Builder for plant definitions.
#[derive(Debug)]
pub struct PlantDefinitionBuilder {
    def: PlantDefinition,
}

impl PlantDefinitionBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(id: PlantTypeId, name: &str) -> Self {
        Self {
            def: PlantDefinition {
                id,
                name: name.to_string(),
                seed_to_sprout_time: 60.0, // 1 hour default
                sprout_to_growing_time: 120.0,
                growing_to_mature_time: 240.0,
                mature_to_wilted_time: 0.0, // Doesn't wilt by default
                wilted_to_dead_time: 60.0,
                min_light: 0.3,
                ideal_light: 0.8,
                needs_water: true,
                rain_bonus: 1.2,
                harvest_item: ItemTypeId::new(1),
                harvest_amount: 1,
                wilted_harvest_amount: 1,
                seed_item: ItemTypeId::new(1),
                regrows_after_harvest: false,
            },
        }
    }

    /// Set growth times (seed→sprout, sprout→growing, growing→mature).
    #[must_use]
    pub fn growth_times(mut self, seed: f32, sprout: f32, growing: f32) -> Self {
        self.def.seed_to_sprout_time = seed;
        self.def.sprout_to_growing_time = sprout;
        self.def.growing_to_mature_time = growing;
        self
    }

    /// Set wilting times.
    #[must_use]
    pub fn wilt_times(mut self, mature_to_wilted: f32, wilted_to_dead: f32) -> Self {
        self.def.mature_to_wilted_time = mature_to_wilted;
        self.def.wilted_to_dead_time = wilted_to_dead;
        self
    }

    /// Set light requirements.
    #[must_use]
    pub fn light(mut self, min: f32, ideal: f32) -> Self {
        self.def.min_light = min;
        self.def.ideal_light = ideal;
        self
    }

    /// Set water requirements.
    #[must_use]
    pub fn water(mut self, needs_water: bool, rain_bonus: f32) -> Self {
        self.def.needs_water = needs_water;
        self.def.rain_bonus = rain_bonus;
        self
    }

    /// Set harvest output.
    #[must_use]
    pub fn harvest(mut self, item: ItemTypeId, amount: u32, wilted_amount: u32) -> Self {
        self.def.harvest_item = item;
        self.def.harvest_amount = amount;
        self.def.wilted_harvest_amount = wilted_amount;
        self
    }

    /// Set seed item.
    #[must_use]
    pub fn seed(mut self, item: ItemTypeId) -> Self {
        self.def.seed_item = item;
        self
    }

    /// Set whether plant regrows after harvest.
    #[must_use]
    pub fn regrows(mut self, regrows: bool) -> Self {
        self.def.regrows_after_harvest = regrows;
        self
    }

    /// Build the plant definition.
    #[must_use]
    pub fn build(self) -> PlantDefinition {
        self.def
    }
}

/// State of a planted plant instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantState {
    /// Plant type.
    pub plant_type: PlantTypeId,
    /// World position.
    pub position: WorldCoord,
    /// Current growth stage.
    pub stage: GrowthStage,
    /// Time spent in current stage (game minutes).
    pub time_in_stage: f32,
    /// Whether the plant is watered.
    pub is_watered: bool,
    /// Whether the plant is getting enough light.
    pub has_light: bool,
    /// Growth rate multiplier (affected by conditions).
    pub growth_rate: f32,
}

impl PlantState {
    /// Create a new plant state (just planted).
    #[must_use]
    pub fn new(plant_type: PlantTypeId, position: WorldCoord) -> Self {
        Self {
            plant_type,
            position,
            stage: GrowthStage::Seed,
            time_in_stage: 0.0,
            is_watered: false,
            has_light: true,
            growth_rate: 1.0,
        }
    }

    /// Check if this plant can be harvested.
    #[must_use]
    pub fn can_harvest(&self) -> bool {
        self.stage.is_harvestable()
    }

    /// Get the growth progress in current stage (0.0 to 1.0).
    #[must_use]
    pub fn stage_progress(&self, stage_time: f32) -> f32 {
        if stage_time <= 0.0 {
            return 1.0;
        }
        (self.time_in_stage / stage_time).clamp(0.0, 1.0)
    }
}

/// Result of a harvest attempt.
#[derive(Debug, Clone)]
pub struct HarvestResult {
    /// Plant type that was harvested.
    pub plant_type: PlantTypeId,
    /// Item harvested.
    pub item: ItemTypeId,
    /// Amount harvested.
    pub amount: u32,
    /// Whether the plant will regrow.
    pub will_regrow: bool,
}

/// Environment conditions affecting plant growth.
#[derive(Debug, Clone, Copy, Default)]
pub struct GrowthConditions {
    /// Current light level (0.0 to 1.0).
    pub light_level: f32,
    /// Whether it's currently raining.
    pub is_raining: bool,
    /// Weather growth modifier.
    pub weather_modifier: f32,
    /// Whether soil is watered/irrigated.
    pub soil_moisture: bool,
}

impl GrowthConditions {
    /// Create conditions from time and weather systems.
    #[must_use]
    pub fn from_environment(light_level: f32, is_raining: bool, weather_modifier: f32) -> Self {
        Self {
            light_level,
            is_raining,
            weather_modifier,
            soil_moisture: is_raining, // Rain automatically waters
        }
    }
}

/// Registry of plant types and active plants.
#[derive(Debug, Default)]
pub struct PlantRegistry {
    /// Plant type definitions.
    definitions: HashMap<PlantTypeId, PlantDefinition>,
    /// Active plants in the world.
    plants: HashMap<WorldCoord, PlantState>,
    /// Pending harvest events.
    pending_harvests: Vec<HarvestResult>,
    /// Positions of plants that died.
    dead_plants: Vec<WorldCoord>,
}

impl PlantRegistry {
    /// Create a new empty plant registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry with default plant definitions.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register default plant types.
    pub fn register_defaults(&mut self) {
        // Grass - regrows naturally
        self.register(
            PlantDefinition::builder(plant_types::GRASS, "Grass")
                .growth_times(5.0, 10.0, 15.0) // Fast growing
                .wilt_times(0.0, 0.0) // Doesn't wilt
                .light(0.1, 0.6)
                .water(false, 1.5)
                .harvest(ItemTypeId::new(4), 1, 1) // Grass item
                .seed(ItemTypeId::new(4))
                .regrows(true)
                .build(),
        );

        // Wheat - basic crop
        self.register(
            PlantDefinition::builder(plant_types::WHEAT, "Wheat")
                .growth_times(60.0, 120.0, 180.0) // ~6 hours total
                .wilt_times(120.0, 60.0)
                .light(0.4, 0.9)
                .water(true, 1.3)
                .harvest(ItemTypeId::new(10), 3, 1) // Wheat item
                .seed(ItemTypeId::new(11)) // Wheat seeds
                .build(),
        );

        // Corn - taller crop
        self.register(
            PlantDefinition::builder(plant_types::CORN, "Corn")
                .growth_times(90.0, 180.0, 240.0) // ~8.5 hours total
                .wilt_times(90.0, 60.0)
                .light(0.5, 1.0)
                .water(true, 1.2)
                .harvest(ItemTypeId::new(12), 2, 1) // Corn item
                .seed(ItemTypeId::new(13)) // Corn seeds
                .build(),
        );

        // Carrot - root vegetable
        self.register(
            PlantDefinition::builder(plant_types::CARROT, "Carrot")
                .growth_times(45.0, 90.0, 120.0) // ~4.25 hours
                .wilt_times(180.0, 90.0)
                .light(0.3, 0.7)
                .water(true, 1.4)
                .harvest(ItemTypeId::new(14), 2, 1) // Carrot item
                .seed(ItemTypeId::new(15)) // Carrot seeds
                .build(),
        );

        // Berry bush - regrows after harvest
        self.register(
            PlantDefinition::builder(plant_types::BERRY_BUSH, "Berry Bush")
                .growth_times(120.0, 180.0, 240.0) // ~9 hours
                .wilt_times(0.0, 0.0) // Doesn't wilt
                .light(0.4, 0.8)
                .water(true, 1.2)
                .harvest(ItemTypeId::new(16), 5, 3) // Berries
                .seed(ItemTypeId::new(17)) // Berry seeds
                .regrows(true)
                .build(),
        );
    }

    /// Register a plant definition.
    pub fn register(&mut self, definition: PlantDefinition) {
        self.definitions.insert(definition.id, definition);
    }

    /// Get a plant definition by ID.
    #[must_use]
    pub fn get_definition(&self, id: PlantTypeId) -> Option<&PlantDefinition> {
        self.definitions.get(&id)
    }

    /// Plant a new plant at a position.
    ///
    /// Returns false if the position is already occupied or plant type unknown.
    pub fn plant(&mut self, plant_type: PlantTypeId, position: WorldCoord) -> bool {
        if self.plants.contains_key(&position) {
            return false;
        }
        if !self.definitions.contains_key(&plant_type) {
            return false;
        }
        self.plants
            .insert(position, PlantState::new(plant_type, position));
        true
    }

    /// Remove a plant at a position.
    pub fn remove(&mut self, position: WorldCoord) -> Option<PlantState> {
        self.plants.remove(&position)
    }

    /// Get a plant at a position.
    #[must_use]
    pub fn get(&self, position: WorldCoord) -> Option<&PlantState> {
        self.plants.get(&position)
    }

    /// Get a mutable reference to a plant.
    pub fn get_mut(&mut self, position: WorldCoord) -> Option<&mut PlantState> {
        self.plants.get_mut(&position)
    }

    /// Check if there's a plant at a position.
    #[must_use]
    pub fn has_plant(&self, position: WorldCoord) -> bool {
        self.plants.contains_key(&position)
    }

    /// Get the number of active plants.
    #[must_use]
    pub fn plant_count(&self) -> usize {
        self.plants.len()
    }

    /// Iterate over all plants.
    pub fn iter(&self) -> impl Iterator<Item = (&WorldCoord, &PlantState)> {
        self.plants.iter()
    }

    /// Iterate mutably over all plants.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&WorldCoord, &mut PlantState)> {
        self.plants.iter_mut()
    }

    /// Try to harvest a plant at a position.
    ///
    /// Returns harvest result if successful, None if not harvestable.
    pub fn harvest(&mut self, position: WorldCoord) -> Option<HarvestResult> {
        let plant = self.plants.get(&position)?;
        if !plant.can_harvest() {
            return None;
        }

        let def = self.definitions.get(&plant.plant_type)?;
        let amount = if plant.stage == GrowthStage::Wilted {
            def.wilted_harvest_amount
        } else {
            def.harvest_amount
        };

        let result = HarvestResult {
            plant_type: plant.plant_type,
            item: def.harvest_item,
            amount,
            will_regrow: def.regrows_after_harvest,
        };

        if def.regrows_after_harvest {
            // Reset to growing stage
            if let Some(plant) = self.plants.get_mut(&position) {
                plant.stage = GrowthStage::Growing;
                plant.time_in_stage = 0.0;
            }
        } else {
            // Remove the plant
            self.plants.remove(&position);
        }

        Some(result)
    }

    /// Update all plants based on conditions.
    ///
    /// Call this each game tick with the elapsed game minutes and conditions.
    pub fn update(&mut self, dt_game_minutes: f32, conditions: GrowthConditions) {
        self.dead_plants.clear();
        let positions: Vec<WorldCoord> = self.plants.keys().copied().collect();

        for position in positions {
            // Get the plant type first
            let plant_type = match self.plants.get(&position) {
                Some(p) => p.plant_type,
                None => continue,
            };

            // Get the definition (cloned to avoid borrow conflicts)
            let def = match self.definitions.get(&plant_type).cloned() {
                Some(d) => d,
                None => continue,
            };

            // Now get mutable access to the plant and update it
            if let Some(plant) = self.plants.get_mut(&position) {
                Self::update_single_plant_static(plant, &def, dt_game_minutes, conditions);

                if plant.stage == GrowthStage::Dead {
                    self.dead_plants.push(position);
                }
            }
        }

        // Remove dead plants
        for pos in &self.dead_plants {
            self.plants.remove(pos);
        }
    }

    /// Update a single plant (static version to avoid borrow issues).
    fn update_single_plant_static(
        plant: &mut PlantState,
        def: &PlantDefinition,
        dt: f32,
        conditions: GrowthConditions,
    ) {
        // Update conditions
        plant.has_light = conditions.light_level >= def.min_light;
        plant.is_watered = !def.needs_water || conditions.soil_moisture || conditions.is_raining;

        // Calculate growth rate
        let mut rate = conditions.weather_modifier;

        // Light bonus
        if conditions.light_level >= def.ideal_light {
            rate *= 1.2;
        } else if conditions.light_level >= def.min_light {
            rate *= 0.5 + (conditions.light_level / def.ideal_light) * 0.5;
        } else {
            rate *= 0.1; // Very slow without light
        }

        // Water bonus
        if conditions.is_raining {
            rate *= def.rain_bonus;
        } else if !plant.is_watered && def.needs_water {
            rate *= 0.3; // Slow without water
        }

        plant.growth_rate = rate;

        // Apply growth
        let effective_dt = dt * rate;
        plant.time_in_stage += effective_dt;

        // Check for stage transitions (may advance multiple stages if enough time)
        loop {
            let stage_time = match plant.stage {
                GrowthStage::Seed => def.seed_to_sprout_time,
                GrowthStage::Sprout => def.sprout_to_growing_time,
                GrowthStage::Growing => def.growing_to_mature_time,
                GrowthStage::Mature => def.mature_to_wilted_time,
                GrowthStage::Wilted => def.wilted_to_dead_time,
                GrowthStage::Dead => return, // No transition from Dead
            };

            // If stage time is 0 or not enough time accumulated, stop
            if stage_time <= 0.0 || plant.time_in_stage < stage_time {
                break;
            }

            // Transition to next stage, carrying over excess time
            plant.time_in_stage -= stage_time;
            plant.stage = plant.stage.next();
        }
    }

    /// Take pending harvest results (clears the list).
    pub fn take_pending_harvests(&mut self) -> Vec<HarvestResult> {
        std::mem::take(&mut self.pending_harvests)
    }

    /// Take dead plant positions (clears the list).
    pub fn take_dead_plants(&mut self) -> Vec<WorldCoord> {
        std::mem::take(&mut self.dead_plants)
    }

    /// Get plants in a specific growth stage.
    pub fn get_by_stage(&self, stage: GrowthStage) -> Vec<&PlantState> {
        self.plants.values().filter(|p| p.stage == stage).collect()
    }

    /// Get harvestable plants.
    pub fn get_harvestable(&self) -> Vec<&PlantState> {
        self.plants.values().filter(|p| p.can_harvest()).collect()
    }

    /// Water a plant at a position.
    pub fn water(&mut self, position: WorldCoord) -> bool {
        if let Some(plant) = self.plants.get_mut(&position) {
            plant.is_watered = true;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_growth_stage_properties() {
        assert!(!GrowthStage::Seed.is_harvestable());
        assert!(GrowthStage::Mature.is_harvestable());
        assert!(GrowthStage::Wilted.is_harvestable());
        assert!(!GrowthStage::Dead.is_harvestable());
        assert!(GrowthStage::Dead.is_dead());
    }

    #[test]
    fn test_growth_stage_progression() {
        assert_eq!(GrowthStage::Seed.next(), GrowthStage::Sprout);
        assert_eq!(GrowthStage::Sprout.next(), GrowthStage::Growing);
        assert_eq!(GrowthStage::Growing.next(), GrowthStage::Mature);
        assert_eq!(GrowthStage::Mature.next(), GrowthStage::Wilted);
        assert_eq!(GrowthStage::Wilted.next(), GrowthStage::Dead);
        assert_eq!(GrowthStage::Dead.next(), GrowthStage::Dead);
    }

    #[test]
    fn test_plant_definition_builder() {
        let def = PlantDefinition::builder(PlantTypeId::new(1), "Test Plant")
            .growth_times(10.0, 20.0, 30.0)
            .light(0.2, 0.7)
            .harvest(ItemTypeId::new(5), 3, 1)
            .build();

        assert_eq!(def.name, "Test Plant");
        assert!((def.seed_to_sprout_time - 10.0).abs() < 0.01);
        assert!((def.total_growth_time() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_plant_state_creation() {
        let state = PlantState::new(PlantTypeId::new(1), WorldCoord::new(10, 20));
        assert_eq!(state.stage, GrowthStage::Seed);
        assert!(!state.can_harvest());
    }

    #[test]
    fn test_plant_registry_creation() {
        let registry = PlantRegistry::with_defaults();
        assert!(registry.get_definition(plant_types::GRASS).is_some());
        assert!(registry.get_definition(plant_types::WHEAT).is_some());
    }

    #[test]
    fn test_plant_registry_plant_and_remove() {
        let mut registry = PlantRegistry::with_defaults();
        let pos = WorldCoord::new(10, 20);

        assert!(registry.plant(plant_types::GRASS, pos));
        assert!(registry.has_plant(pos));
        assert!(!registry.plant(plant_types::GRASS, pos)); // Already occupied

        registry.remove(pos);
        assert!(!registry.has_plant(pos));
    }

    #[test]
    fn test_plant_growth() {
        let mut registry = PlantRegistry::with_defaults();
        let pos = WorldCoord::new(10, 20);

        registry.plant(plant_types::GRASS, pos);

        // Good conditions
        let conditions = GrowthConditions {
            light_level: 0.8,
            is_raining: true,
            weather_modifier: 1.5,
            soil_moisture: true,
        };

        // Grass grows fast: 5 + 10 + 15 = 30 minutes total
        // With conditions multiplier, should be even faster
        registry.update(50.0, conditions);

        let plant = registry.get(pos).expect("plant should exist");
        assert_eq!(plant.stage, GrowthStage::Mature);
    }

    #[test]
    fn test_harvest() {
        let mut registry = PlantRegistry::with_defaults();
        let pos = WorldCoord::new(10, 20);

        registry.plant(plant_types::WHEAT, pos);

        // Grow to mature
        if let Some(plant) = registry.get_mut(pos) {
            plant.stage = GrowthStage::Mature;
        }

        // Harvest
        let result = registry.harvest(pos);
        assert!(result.is_some());

        let harvest = result.expect("harvest");
        assert_eq!(harvest.plant_type, plant_types::WHEAT);
        assert!(!harvest.will_regrow);

        // Plant should be removed
        assert!(!registry.has_plant(pos));
    }

    #[test]
    fn test_harvest_regrows() {
        let mut registry = PlantRegistry::with_defaults();
        let pos = WorldCoord::new(10, 20);

        // Berry bush regrows
        registry.plant(plant_types::BERRY_BUSH, pos);

        if let Some(plant) = registry.get_mut(pos) {
            plant.stage = GrowthStage::Mature;
        }

        let result = registry.harvest(pos);
        assert!(result.is_some());
        assert!(result.expect("harvest").will_regrow);

        // Plant should still exist, reset to growing
        let plant = registry.get(pos).expect("plant should exist");
        assert_eq!(plant.stage, GrowthStage::Growing);
    }

    #[test]
    fn test_growth_conditions() {
        let conditions = GrowthConditions::from_environment(0.7, true, 1.5);
        assert!(conditions.is_raining);
        assert!(conditions.soil_moisture); // Rain waters soil
    }

    #[test]
    fn test_plant_without_light() {
        let mut registry = PlantRegistry::with_defaults();
        let pos = WorldCoord::new(10, 20);

        registry.plant(plant_types::WHEAT, pos);

        // No light
        let conditions = GrowthConditions {
            light_level: 0.1,
            is_raining: false,
            weather_modifier: 1.0,
            soil_moisture: false,
        };

        registry.update(60.0, conditions);

        let plant = registry.get(pos).expect("plant");
        // Should still be seed or sprout due to low growth rate
        assert!(plant.stage == GrowthStage::Seed || plant.stage == GrowthStage::Sprout);
    }

    #[test]
    fn test_get_harvestable() {
        let mut registry = PlantRegistry::with_defaults();

        registry.plant(plant_types::WHEAT, WorldCoord::new(0, 0));
        registry.plant(plant_types::WHEAT, WorldCoord::new(1, 0));
        registry.plant(plant_types::WHEAT, WorldCoord::new(2, 0));

        // Make one mature, one wilted
        if let Some(plant) = registry.get_mut(WorldCoord::new(0, 0)) {
            plant.stage = GrowthStage::Mature;
        }
        if let Some(plant) = registry.get_mut(WorldCoord::new(1, 0)) {
            plant.stage = GrowthStage::Wilted;
        }

        let harvestable = registry.get_harvestable();
        assert_eq!(harvestable.len(), 2);
    }

    #[test]
    fn test_water_plant() {
        let mut registry = PlantRegistry::with_defaults();
        let pos = WorldCoord::new(10, 20);

        registry.plant(plant_types::WHEAT, pos);
        assert!(!registry.get(pos).expect("plant").is_watered);

        registry.water(pos);
        assert!(registry.get(pos).expect("plant").is_watered);
    }

    #[test]
    fn test_plant_wilting() {
        let mut registry = PlantRegistry::with_defaults();
        let pos = WorldCoord::new(10, 20);

        registry.plant(plant_types::WHEAT, pos);

        // Set to mature
        if let Some(plant) = registry.get_mut(pos) {
            plant.stage = GrowthStage::Mature;
        }

        // Wait long enough to wilt (wheat wilt time = 120 minutes)
        let conditions = GrowthConditions {
            light_level: 0.8,
            is_raining: false,
            weather_modifier: 1.0,
            soil_moisture: true,
        };

        registry.update(150.0, conditions);

        let plant = registry.get(pos).expect("plant");
        assert_eq!(plant.stage, GrowthStage::Wilted);
    }

    #[test]
    fn test_stage_progress() {
        let mut plant = PlantState::new(PlantTypeId::new(1), WorldCoord::new(0, 0));
        plant.time_in_stage = 5.0;

        assert!((plant.stage_progress(10.0) - 0.5).abs() < 0.01);
        assert!((plant.stage_progress(0.0) - 1.0).abs() < 0.01);
    }
}
