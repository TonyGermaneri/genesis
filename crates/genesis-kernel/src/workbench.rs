//! Workbench Zone System
//!
//! This module provides the workbench and crafting station infrastructure.
//! It handles:
//!
//! - Different workbench types (Basic, Forge, Anvil, Alchemy)
//! - Interaction radius and player detection
//! - Station capabilities and recipe filtering
//! - Zone-based crafting permissions
//!
//! # Architecture
//!
//! ```text
//! ┌───────────────┐     ┌────────────────┐     ┌─────────────────┐
//! │ WorkbenchType │────▶│ WorkbenchZone  │────▶│ CraftingStation │
//! │ (enum)        │     │ (world pos)    │     │ (full config)   │
//! └───────────────┘     └────────────────┘     └─────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use genesis_kernel::workbench::{WorkbenchType, WorkbenchZone, CraftingStation};
//!
//! // Create a forge workbench zone
//! let zone = WorkbenchZone::new(WorkbenchType::Forge, [10.0, 5.0, 10.0]);
//!
//! // Check if player is in range
//! let player_pos = [11.0, 5.0, 10.0];
//! assert!(zone.is_in_range(player_pos));
//!
//! // Create a full crafting station
//! let station = CraftingStation::builder(WorkbenchType::Forge)
//!     .position([10.0, 5.0, 10.0])
//!     .interaction_radius(3.0)
//!     .build();
//! ```

use std::collections::HashSet;

use tracing::debug;

/// Default interaction radius for workbenches.
pub const DEFAULT_INTERACTION_RADIUS: f32 = 2.5;

/// Maximum interaction radius allowed.
pub const MAX_INTERACTION_RADIUS: f32 = 10.0;

/// Workbench type identifier.
pub type WorkbenchId = u32;

/// Recipe category identifier.
pub type RecipeCategory = u32;

/// Station capability flags.
pub type CapabilityFlags = u32;

/// Type of crafting workbench/station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum WorkbenchType {
    /// Basic crafting table (3x3 grid).
    #[default]
    Basic = 0,
    /// Forge for metalworking (smelting, tool crafting).
    Forge = 1,
    /// Anvil for repairs and tool enhancement.
    Anvil = 2,
    /// Alchemy table for potions and magical items.
    Alchemy = 3,
    /// Cooking station for food preparation.
    Cooking = 4,
    /// Loom for textile crafting.
    Loom = 5,
    /// Woodworking station.
    Woodworking = 6,
    /// Stonecutting station.
    Stonecutting = 7,
    /// Enchanting table for magical enhancement.
    Enchanting = 8,
    /// Custom workbench type.
    Custom(u8) = 255,
}

impl WorkbenchType {
    /// Get the default grid size for this workbench type.
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub const fn default_grid_size(&self) -> (usize, usize) {
        match self {
            Self::Basic => (3, 3),
            Self::Forge => (3, 3),
            Self::Anvil => (2, 1), // Item + material
            Self::Alchemy => (3, 3),
            Self::Cooking => (3, 3),
            Self::Loom => (2, 2),
            Self::Woodworking => (3, 3),
            Self::Stonecutting => (1, 1), // Single input
            Self::Enchanting => (1, 1),   // Single input
            Self::Custom(_) => (3, 3),
        }
    }

    /// Get the default interaction radius.
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub const fn default_radius(&self) -> f32 {
        match self {
            Self::Basic => 2.5,
            Self::Forge => 3.0,
            Self::Anvil => 2.0,
            Self::Alchemy => 2.5,
            Self::Cooking => 3.0,
            Self::Loom => 2.5,
            Self::Woodworking => 3.0,
            Self::Stonecutting => 2.5,
            Self::Enchanting => 2.0,
            Self::Custom(_) => DEFAULT_INTERACTION_RADIUS,
        }
    }

    /// Get the default capabilities for this workbench type.
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub const fn default_capabilities(&self) -> CapabilityFlags {
        match self {
            Self::Basic => Capability::CRAFTING,
            Self::Forge => Capability::CRAFTING | Capability::SMELTING | Capability::FUEL_REQUIRED,
            Self::Anvil => Capability::REPAIR | Capability::ENHANCEMENT,
            Self::Alchemy => Capability::CRAFTING | Capability::BREWING,
            Self::Cooking => Capability::CRAFTING | Capability::COOKING | Capability::FUEL_REQUIRED,
            Self::Loom => Capability::CRAFTING,
            Self::Woodworking => Capability::CRAFTING,
            Self::Stonecutting => Capability::CRAFTING,
            Self::Enchanting => Capability::ENHANCEMENT | Capability::ENCHANTING,
            Self::Custom(_) => Capability::CRAFTING,
        }
    }

    /// Check if this workbench requires fuel.
    #[must_use]
    pub const fn requires_fuel(&self) -> bool {
        (self.default_capabilities() & Capability::FUEL_REQUIRED) != 0
    }

    /// Get the workbench type ID.
    #[must_use]
    pub const fn id(&self) -> u8 {
        match self {
            Self::Basic => 0,
            Self::Forge => 1,
            Self::Anvil => 2,
            Self::Alchemy => 3,
            Self::Cooking => 4,
            Self::Loom => 5,
            Self::Woodworking => 6,
            Self::Stonecutting => 7,
            Self::Enchanting => 8,
            Self::Custom(id) => *id,
        }
    }

    /// Create from ID.
    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        match id {
            0 => Self::Basic,
            1 => Self::Forge,
            2 => Self::Anvil,
            3 => Self::Alchemy,
            4 => Self::Cooking,
            5 => Self::Loom,
            6 => Self::Woodworking,
            7 => Self::Stonecutting,
            8 => Self::Enchanting,
            other => Self::Custom(other),
        }
    }

    /// Get a human-readable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Basic => "Crafting Table",
            Self::Forge => "Forge",
            Self::Anvil => "Anvil",
            Self::Alchemy => "Alchemy Table",
            Self::Cooking => "Cooking Station",
            Self::Loom => "Loom",
            Self::Woodworking => "Woodworking Bench",
            Self::Stonecutting => "Stonecutter",
            Self::Enchanting => "Enchanting Table",
            Self::Custom(_) => "Custom Station",
        }
    }
}

/// Station capability flags.
#[allow(non_snake_case)]
pub mod Capability {
    use super::CapabilityFlags;

    /// Can perform basic crafting.
    pub const CRAFTING: CapabilityFlags = 1 << 0;
    /// Can smelt ores.
    pub const SMELTING: CapabilityFlags = 1 << 1;
    /// Can repair items.
    pub const REPAIR: CapabilityFlags = 1 << 2;
    /// Can enhance/upgrade items.
    pub const ENHANCEMENT: CapabilityFlags = 1 << 3;
    /// Can brew potions.
    pub const BREWING: CapabilityFlags = 1 << 4;
    /// Can cook food.
    pub const COOKING: CapabilityFlags = 1 << 5;
    /// Can enchant items.
    pub const ENCHANTING: CapabilityFlags = 1 << 6;
    /// Requires fuel to operate.
    pub const FUEL_REQUIRED: CapabilityFlags = 1 << 7;
    /// Has fluid input.
    pub const FLUID_INPUT: CapabilityFlags = 1 << 8;
    /// Has fluid output.
    pub const FLUID_OUTPUT: CapabilityFlags = 1 << 9;
    /// Produces byproducts.
    pub const BYPRODUCTS: CapabilityFlags = 1 << 10;
}

/// A workbench zone in the world.
#[derive(Debug, Clone)]
pub struct WorkbenchZone {
    /// Workbench type.
    workbench_type: WorkbenchType,
    /// World position [x, y, z].
    position: [f32; 3],
    /// Interaction radius.
    radius: f32,
    /// Whether the workbench is active.
    active: bool,
    /// Unique zone ID.
    zone_id: u64,
}

impl WorkbenchZone {
    /// Create a new workbench zone.
    #[must_use]
    pub fn new(workbench_type: WorkbenchType, position: [f32; 3]) -> Self {
        Self {
            workbench_type,
            position,
            radius: workbench_type.default_radius(),
            active: true,
            zone_id: 0,
        }
    }

    /// Create with custom radius.
    #[must_use]
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius.min(MAX_INTERACTION_RADIUS);
        self
    }

    /// Set the zone ID.
    #[must_use]
    pub const fn with_id(mut self, id: u64) -> Self {
        self.zone_id = id;
        self
    }

    /// Get the workbench type.
    #[must_use]
    pub const fn workbench_type(&self) -> WorkbenchType {
        self.workbench_type
    }

    /// Get the position.
    #[must_use]
    pub const fn position(&self) -> [f32; 3] {
        self.position
    }

    /// Get the interaction radius.
    #[must_use]
    pub const fn radius(&self) -> f32 {
        self.radius
    }

    /// Get the zone ID.
    #[must_use]
    pub const fn zone_id(&self) -> u64 {
        self.zone_id
    }

    /// Check if the workbench is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Set active state.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Check if a position is within interaction range.
    #[must_use]
    pub fn is_in_range(&self, other: [f32; 3]) -> bool {
        let dx = self.position[0] - other[0];
        let dy = self.position[1] - other[1];
        let dz = self.position[2] - other[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;
        dist_sq <= self.radius * self.radius
    }

    /// Get the distance to a position.
    #[must_use]
    pub fn distance_to(&self, other: [f32; 3]) -> f32 {
        let dx = self.position[0] - other[0];
        let dy = self.position[1] - other[1];
        let dz = self.position[2] - other[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Get squared distance (for comparisons).
    #[must_use]
    pub fn distance_sq(&self, other: [f32; 3]) -> f32 {
        let dx = self.position[0] - other[0];
        let dy = self.position[1] - other[1];
        let dz = self.position[2] - other[2];
        dx * dx + dy * dy + dz * dz
    }
}

/// Full crafting station configuration.
#[derive(Debug, Clone)]
pub struct CraftingStation {
    /// Workbench type.
    workbench_type: WorkbenchType,
    /// World position.
    position: [f32; 3],
    /// Interaction radius.
    radius: f32,
    /// Crafting grid size (width, height).
    grid_size: (usize, usize),
    /// Capability flags.
    capabilities: CapabilityFlags,
    /// Allowed recipe categories.
    allowed_categories: HashSet<RecipeCategory>,
    /// Blocked recipe categories.
    blocked_categories: HashSet<RecipeCategory>,
    /// Crafting speed multiplier.
    speed_multiplier: f32,
    /// Quality bonus (0.0 = none, 1.0 = 100% better).
    quality_bonus: f32,
    /// Current fuel level (0.0 - 1.0).
    fuel_level: f32,
    /// Whether the station is currently active.
    active: bool,
    /// Owner entity ID (0 = unowned).
    owner_id: u64,
}

impl CraftingStation {
    /// Create a builder for a crafting station.
    #[must_use]
    pub fn builder(workbench_type: WorkbenchType) -> CraftingStationBuilder {
        CraftingStationBuilder::new(workbench_type)
    }

    /// Create a simple station with defaults.
    #[must_use]
    pub fn new(workbench_type: WorkbenchType, position: [f32; 3]) -> Self {
        Self::builder(workbench_type).position(position).build()
    }

    /// Get the workbench type.
    #[must_use]
    pub const fn workbench_type(&self) -> WorkbenchType {
        self.workbench_type
    }

    /// Get the position.
    #[must_use]
    pub const fn position(&self) -> [f32; 3] {
        self.position
    }

    /// Set the position.
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }

    /// Get the interaction radius.
    #[must_use]
    pub const fn radius(&self) -> f32 {
        self.radius
    }

    /// Get the grid size.
    #[must_use]
    pub const fn grid_size(&self) -> (usize, usize) {
        self.grid_size
    }

    /// Get capabilities.
    #[must_use]
    pub const fn capabilities(&self) -> CapabilityFlags {
        self.capabilities
    }

    /// Check if station has a capability.
    #[must_use]
    pub const fn has_capability(&self, cap: CapabilityFlags) -> bool {
        (self.capabilities & cap) != 0
    }

    /// Check if a recipe category is allowed.
    #[must_use]
    pub fn is_category_allowed(&self, category: RecipeCategory) -> bool {
        if self.blocked_categories.contains(&category) {
            return false;
        }
        self.allowed_categories.is_empty() || self.allowed_categories.contains(&category)
    }

    /// Get the speed multiplier.
    #[must_use]
    pub const fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier
    }

    /// Get the quality bonus.
    #[must_use]
    pub const fn quality_bonus(&self) -> f32 {
        self.quality_bonus
    }

    /// Get the fuel level.
    #[must_use]
    pub const fn fuel_level(&self) -> f32 {
        self.fuel_level
    }

    /// Set the fuel level.
    pub fn set_fuel_level(&mut self, level: f32) {
        self.fuel_level = level.clamp(0.0, 1.0);
    }

    /// Add fuel.
    pub fn add_fuel(&mut self, amount: f32) {
        self.fuel_level = (self.fuel_level + amount).min(1.0);
    }

    /// Consume fuel.
    ///
    /// Returns true if there was enough fuel.
    pub fn consume_fuel(&mut self, amount: f32) -> bool {
        if self.fuel_level >= amount {
            self.fuel_level -= amount;
            true
        } else {
            false
        }
    }

    /// Check if station requires fuel and has some.
    #[must_use]
    pub fn has_fuel(&self) -> bool {
        if !self.has_capability(Capability::FUEL_REQUIRED) {
            return true; // No fuel needed
        }
        self.fuel_level > 0.0
    }

    /// Check if the station is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Set active state.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        debug!(
            "Station {:?} active: {}",
            self.workbench_type.name(),
            active
        );
    }

    /// Check if a position is in range.
    #[must_use]
    pub fn is_in_range(&self, position: [f32; 3]) -> bool {
        let dx = self.position[0] - position[0];
        let dy = self.position[1] - position[1];
        let dz = self.position[2] - position[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;
        dist_sq <= self.radius * self.radius
    }

    /// Get the owner entity ID.
    #[must_use]
    pub const fn owner_id(&self) -> u64 {
        self.owner_id
    }

    /// Set the owner entity ID.
    pub fn set_owner(&mut self, owner_id: u64) {
        self.owner_id = owner_id;
    }

    /// Check if entity can use this station.
    #[must_use]
    pub fn can_use(&self, entity_id: u64, entity_position: [f32; 3]) -> bool {
        if !self.active {
            return false;
        }
        if !self.is_in_range(entity_position) {
            return false;
        }
        // If owned, only owner can use (0 = public)
        if self.owner_id != 0 && self.owner_id != entity_id {
            return false;
        }
        true
    }

    /// Create a WorkbenchZone from this station.
    #[must_use]
    pub fn to_zone(&self) -> WorkbenchZone {
        WorkbenchZone {
            workbench_type: self.workbench_type,
            position: self.position,
            radius: self.radius,
            active: self.active,
            zone_id: 0,
        }
    }
}

/// Builder for crafting stations.
#[derive(Debug, Clone)]
pub struct CraftingStationBuilder {
    station: CraftingStation,
}

impl CraftingStationBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(workbench_type: WorkbenchType) -> Self {
        Self {
            station: CraftingStation {
                workbench_type,
                position: [0.0, 0.0, 0.0],
                radius: workbench_type.default_radius(),
                grid_size: workbench_type.default_grid_size(),
                capabilities: workbench_type.default_capabilities(),
                allowed_categories: HashSet::new(),
                blocked_categories: HashSet::new(),
                speed_multiplier: 1.0,
                quality_bonus: 0.0,
                fuel_level: 0.0,
                active: true,
                owner_id: 0,
            },
        }
    }

    /// Set the position.
    #[must_use]
    pub fn position(mut self, pos: [f32; 3]) -> Self {
        self.station.position = pos;
        self
    }

    /// Set the interaction radius.
    #[must_use]
    pub fn interaction_radius(mut self, radius: f32) -> Self {
        self.station.radius = radius.min(MAX_INTERACTION_RADIUS);
        self
    }

    /// Set the grid size.
    #[must_use]
    pub fn grid_size(mut self, width: usize, height: usize) -> Self {
        self.station.grid_size = (width, height);
        self
    }

    /// Set capabilities.
    #[must_use]
    pub fn capabilities(mut self, caps: CapabilityFlags) -> Self {
        self.station.capabilities = caps;
        self
    }

    /// Add a capability.
    #[must_use]
    pub fn add_capability(mut self, cap: CapabilityFlags) -> Self {
        self.station.capabilities |= cap;
        self
    }

    /// Remove a capability.
    #[must_use]
    pub fn remove_capability(mut self, cap: CapabilityFlags) -> Self {
        self.station.capabilities &= !cap;
        self
    }

    /// Allow a recipe category.
    #[must_use]
    pub fn allow_category(mut self, category: RecipeCategory) -> Self {
        self.station.allowed_categories.insert(category);
        self
    }

    /// Block a recipe category.
    #[must_use]
    pub fn block_category(mut self, category: RecipeCategory) -> Self {
        self.station.blocked_categories.insert(category);
        self
    }

    /// Set speed multiplier.
    #[must_use]
    pub fn speed_multiplier(mut self, mult: f32) -> Self {
        self.station.speed_multiplier = mult.max(0.01);
        self
    }

    /// Set quality bonus.
    #[must_use]
    pub fn quality_bonus(mut self, bonus: f32) -> Self {
        self.station.quality_bonus = bonus;
        self
    }

    /// Set initial fuel level.
    #[must_use]
    pub fn fuel_level(mut self, level: f32) -> Self {
        self.station.fuel_level = level.clamp(0.0, 1.0);
        self
    }

    /// Set active state.
    #[must_use]
    pub fn active(mut self, active: bool) -> Self {
        self.station.active = active;
        self
    }

    /// Set owner.
    #[must_use]
    pub fn owner(mut self, owner_id: u64) -> Self {
        self.station.owner_id = owner_id;
        self
    }

    /// Build the crafting station.
    #[must_use]
    pub fn build(self) -> CraftingStation {
        self.station
    }
}

/// Station registry for world management.
#[derive(Debug, Default)]
pub struct StationRegistry {
    /// Registered stations by ID.
    stations: Vec<CraftingStation>,
    /// Next station ID.
    next_id: u64,
}

impl StationRegistry {
    /// Create a new registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a station and return its ID.
    pub fn register(&mut self, station: CraftingStation) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.stations.push(station);
        debug!("Registered station {} at index {}", id, self.stations.len() - 1);
        id
    }

    /// Get a station by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&CraftingStation> {
        self.stations.get(index)
    }

    /// Get mutable station by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut CraftingStation> {
        self.stations.get_mut(index)
    }

    /// Find stations in range of a position.
    #[must_use]
    pub fn find_in_range(&self, position: [f32; 3]) -> Vec<(usize, &CraftingStation)> {
        self.stations
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_in_range(position))
            .collect()
    }

    /// Find nearest station of a type.
    #[must_use]
    pub fn find_nearest(
        &self,
        position: [f32; 3],
        workbench_type: WorkbenchType,
    ) -> Option<(usize, &CraftingStation)> {
        self.stations
            .iter()
            .enumerate()
            .filter(|(_, s)| s.workbench_type() == workbench_type && s.is_active())
            .min_by(|(_, a), (_, b)| {
                let da = distance_sq(position, a.position());
                let db = distance_sq(position, b.position());
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Find all stations with a capability.
    #[must_use]
    pub fn find_with_capability(&self, cap: CapabilityFlags) -> Vec<(usize, &CraftingStation)> {
        self.stations
            .iter()
            .enumerate()
            .filter(|(_, s)| s.has_capability(cap))
            .collect()
    }

    /// Remove a station by index.
    pub fn remove(&mut self, index: usize) -> Option<CraftingStation> {
        if index < self.stations.len() {
            Some(self.stations.remove(index))
        } else {
            None
        }
    }

    /// Get all stations.
    #[must_use]
    pub fn stations(&self) -> &[CraftingStation] {
        &self.stations
    }

    /// Get station count.
    #[must_use]
    pub fn count(&self) -> usize {
        self.stations.len()
    }

    /// Clear all stations.
    pub fn clear(&mut self) {
        self.stations.clear();
    }
}

/// Calculate squared distance between two positions.
#[inline]
fn distance_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workbench_type() {
        assert_eq!(WorkbenchType::Basic.default_grid_size(), (3, 3));
        assert_eq!(WorkbenchType::Anvil.default_grid_size(), (2, 1));
        assert!(WorkbenchType::Forge.requires_fuel());
        assert!(!WorkbenchType::Basic.requires_fuel());
    }

    #[test]
    fn test_workbench_type_roundtrip() {
        for id in 0..=10 {
            let wb_type = WorkbenchType::from_id(id);
            let back = WorkbenchType::from_id(wb_type.id());
            assert_eq!(wb_type.id(), back.id());
        }
    }

    #[test]
    fn test_workbench_zone_creation() {
        let zone = WorkbenchZone::new(WorkbenchType::Forge, [10.0, 5.0, 10.0]);
        assert_eq!(zone.workbench_type(), WorkbenchType::Forge);
        assert_eq!(zone.position(), [10.0, 5.0, 10.0]);
        assert!((zone.radius() - 3.0).abs() < f32::EPSILON);
        assert!(zone.is_active());
    }

    #[test]
    fn test_workbench_zone_range() {
        let zone = WorkbenchZone::new(WorkbenchType::Basic, [0.0, 0.0, 0.0]);

        // Within range
        assert!(zone.is_in_range([1.0, 0.0, 0.0]));
        assert!(zone.is_in_range([0.0, 2.0, 0.0]));

        // Out of range
        assert!(!zone.is_in_range([5.0, 0.0, 0.0]));
        assert!(!zone.is_in_range([0.0, 0.0, 10.0]));
    }

    #[test]
    fn test_crafting_station_builder() {
        let station = CraftingStation::builder(WorkbenchType::Forge)
            .position([10.0, 5.0, 20.0])
            .interaction_radius(4.0)
            .speed_multiplier(1.5)
            .quality_bonus(0.1)
            .fuel_level(0.8)
            .owner(12345)
            .build();

        assert_eq!(station.workbench_type(), WorkbenchType::Forge);
        assert_eq!(station.position(), [10.0, 5.0, 20.0]);
        assert!((station.radius() - 4.0).abs() < f32::EPSILON);
        assert!((station.speed_multiplier() - 1.5).abs() < f32::EPSILON);
        assert!((station.quality_bonus() - 0.1).abs() < f32::EPSILON);
        assert!((station.fuel_level() - 0.8).abs() < f32::EPSILON);
        assert_eq!(station.owner_id(), 12345);
    }

    #[test]
    fn test_station_capabilities() {
        let station = CraftingStation::new(WorkbenchType::Forge, [0.0, 0.0, 0.0]);

        assert!(station.has_capability(Capability::CRAFTING));
        assert!(station.has_capability(Capability::SMELTING));
        assert!(station.has_capability(Capability::FUEL_REQUIRED));
        assert!(!station.has_capability(Capability::REPAIR));
    }

    #[test]
    fn test_station_fuel() {
        let mut station = CraftingStation::builder(WorkbenchType::Forge)
            .position([0.0, 0.0, 0.0])
            .fuel_level(0.5)
            .build();

        assert!(station.has_fuel());
        assert!(station.consume_fuel(0.3));
        assert!((station.fuel_level() - 0.2).abs() < f32::EPSILON);

        assert!(!station.consume_fuel(0.5)); // Not enough
        assert!((station.fuel_level() - 0.2).abs() < f32::EPSILON);

        station.add_fuel(0.5);
        assert!((station.fuel_level() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_station_can_use() {
        let station = CraftingStation::builder(WorkbenchType::Basic)
            .position([0.0, 0.0, 0.0])
            .interaction_radius(3.0)
            .owner(100)
            .build();

        // Owner in range
        assert!(station.can_use(100, [1.0, 0.0, 0.0]));

        // Non-owner in range
        assert!(!station.can_use(200, [1.0, 0.0, 0.0]));

        // Owner out of range
        assert!(!station.can_use(100, [10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_station_public_access() {
        let station = CraftingStation::builder(WorkbenchType::Basic)
            .position([0.0, 0.0, 0.0])
            .interaction_radius(3.0)
            // No owner (owner_id = 0)
            .build();

        // Anyone in range can use
        assert!(station.can_use(100, [1.0, 0.0, 0.0]));
        assert!(station.can_use(200, [1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_recipe_categories() {
        let station = CraftingStation::builder(WorkbenchType::Basic)
            .position([0.0, 0.0, 0.0])
            .allow_category(1)
            .allow_category(2)
            .block_category(3)
            .build();

        assert!(station.is_category_allowed(1));
        assert!(station.is_category_allowed(2));
        assert!(!station.is_category_allowed(3)); // Blocked
        assert!(!station.is_category_allowed(4)); // Not in allowed list
    }

    #[test]
    fn test_station_registry() {
        let mut registry = StationRegistry::new();

        let id1 = registry.register(CraftingStation::new(
            WorkbenchType::Basic,
            [0.0, 0.0, 0.0],
        ));
        let id2 = registry.register(CraftingStation::new(
            WorkbenchType::Forge,
            [10.0, 0.0, 0.0],
        ));

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(registry.count(), 2);
    }

    #[test]
    fn test_registry_find_in_range() {
        let mut registry = StationRegistry::new();

        registry.register(CraftingStation::new(WorkbenchType::Basic, [0.0, 0.0, 0.0]));
        registry.register(CraftingStation::new(WorkbenchType::Forge, [10.0, 0.0, 0.0]));
        registry.register(CraftingStation::new(WorkbenchType::Anvil, [1.0, 0.0, 0.0]));

        let in_range = registry.find_in_range([0.5, 0.0, 0.0]);
        assert_eq!(in_range.len(), 2); // Basic and Anvil
    }

    #[test]
    fn test_registry_find_nearest() {
        let mut registry = StationRegistry::new();

        registry.register(CraftingStation::new(WorkbenchType::Forge, [0.0, 0.0, 0.0]));
        registry.register(CraftingStation::new(WorkbenchType::Forge, [10.0, 0.0, 0.0]));
        registry.register(CraftingStation::new(WorkbenchType::Forge, [5.0, 0.0, 0.0]));

        let nearest = registry.find_nearest([4.0, 0.0, 0.0], WorkbenchType::Forge);
        assert!(nearest.is_some());
        let (idx, _) = nearest.unwrap();
        assert_eq!(idx, 2); // The one at [5.0, 0.0, 0.0]
    }

    #[test]
    fn test_to_zone() {
        let station = CraftingStation::builder(WorkbenchType::Alchemy)
            .position([5.0, 5.0, 5.0])
            .interaction_radius(3.5)
            .active(true)
            .build();

        let zone = station.to_zone();
        assert_eq!(zone.workbench_type(), WorkbenchType::Alchemy);
        assert_eq!(zone.position(), [5.0, 5.0, 5.0]);
        assert!((zone.radius() - 3.5).abs() < f32::EPSILON);
        assert!(zone.is_active());
    }
}
