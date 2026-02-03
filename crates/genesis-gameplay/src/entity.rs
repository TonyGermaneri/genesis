//! Entity system.

use genesis_common::{EntityId, WorldCoord};
use serde::{Deserialize, Serialize};

/// Type of entity in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// Player character
    Player,
    /// Non-player character
    Npc,
    /// Vehicle (can be entered)
    Vehicle,
    /// Projectile
    Projectile,
    /// Item drop in world
    ItemDrop,
    /// Building/structure
    Building,
}

/// An entity in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier
    id: EntityId,
    /// Entity type
    entity_type: EntityType,
    /// World position
    position: WorldCoord,
    /// Health (if applicable)
    health: Option<Health>,
    /// Whether entity is active
    active: bool,
}

impl Entity {
    /// Creates a new entity of the given type.
    #[must_use]
    pub fn new(entity_type: EntityType) -> Self {
        Self {
            id: EntityId::new(),
            entity_type,
            position: WorldCoord::new(0, 0),
            health: Some(Health::new(100)),
            active: true,
        }
    }

    /// Returns the entity's unique ID.
    #[must_use]
    pub const fn id(&self) -> EntityId {
        self.id
    }

    /// Returns the entity type.
    #[must_use]
    pub const fn entity_type(&self) -> EntityType {
        self.entity_type
    }

    /// Returns the entity's world position.
    #[must_use]
    pub const fn position(&self) -> WorldCoord {
        self.position
    }

    /// Sets the entity's world position.
    pub fn set_position(&mut self, pos: WorldCoord) {
        self.position = pos;
    }

    /// Returns the entity's health, if any.
    #[must_use]
    pub const fn health(&self) -> Option<&Health> {
        self.health.as_ref()
    }

    /// Returns whether the entity is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Deactivates the entity.
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Health component for entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    /// Current health
    current: i32,
    /// Maximum health
    max: i32,
}

impl Health {
    /// Creates a new health component.
    #[must_use]
    pub const fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    /// Returns current health.
    #[must_use]
    pub const fn current(&self) -> i32 {
        self.current
    }

    /// Returns maximum health.
    #[must_use]
    pub const fn max(&self) -> i32 {
        self.max
    }

    /// Applies damage.
    pub fn damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }

    /// Applies healing.
    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Checks if dead.
    #[must_use]
    pub const fn is_dead(&self) -> bool {
        self.current <= 0
    }
}
