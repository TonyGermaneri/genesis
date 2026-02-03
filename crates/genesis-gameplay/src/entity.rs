//! Entity system with arena-based storage.

use genesis_common::{EntityId, WorldCoord};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for entity operations.
#[derive(Debug, Error)]
pub enum EntityError {
    /// Entity not found
    #[error("Entity not found: {0:?}")]
    NotFound(EntityId),
    /// Arena is full
    #[error("Entity arena is full: capacity {0}")]
    ArenaFull(usize),
    /// Entity already despawned
    #[error("Entity already despawned: {0:?}")]
    AlreadyDespawned(EntityId),
}

/// Result type for entity operations.
pub type EntityResult<T> = Result<T, EntityError>;

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

/// Arena-based entity storage for efficient allocation and lookup.
///
/// Uses a free list for O(1) allocation and deallocation.
/// Entity lookup by ID uses a HashMap for O(1) access.
#[derive(Debug, Default)]
pub struct EntityArena {
    /// Storage slots for entities
    entities: Vec<Option<Entity>>,
    /// Free slot indices for reuse
    free_list: Vec<usize>,
    /// Map from EntityId to slot index for fast lookup
    id_to_index: std::collections::HashMap<EntityId, usize>,
}

impl EntityArena {
    /// Creates a new empty entity arena.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_list: Vec::new(),
            id_to_index: std::collections::HashMap::new(),
        }
    }

    /// Creates a new entity arena with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            id_to_index: std::collections::HashMap::with_capacity(capacity),
        }
    }

    /// Returns the number of active entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.id_to_index.len()
    }

    /// Returns true if there are no active entities.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.id_to_index.is_empty()
    }

    /// Returns the total capacity (including free slots).
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.entities.len()
    }

    /// Spawns a new entity of the given type.
    ///
    /// Returns the entity's ID on success.
    pub fn spawn(&mut self, entity_type: EntityType) -> EntityId {
        let entity = Entity::new(entity_type);
        let id = entity.id();

        let index = if let Some(free_index) = self.free_list.pop() {
            // Reuse a free slot
            self.entities[free_index] = Some(entity);
            free_index
        } else {
            // Allocate a new slot
            let index = self.entities.len();
            self.entities.push(Some(entity));
            index
        };

        self.id_to_index.insert(id, index);
        id
    }

    /// Spawns a pre-created entity into the arena.
    ///
    /// This is useful for loading entities from save files.
    pub fn spawn_entity(&mut self, entity: Entity) -> EntityId {
        let id = entity.id();

        let index = if let Some(free_index) = self.free_list.pop() {
            self.entities[free_index] = Some(entity);
            free_index
        } else {
            let index = self.entities.len();
            self.entities.push(Some(entity));
            index
        };

        self.id_to_index.insert(id, index);
        id
    }

    /// Despawns an entity by ID.
    ///
    /// Returns the despawned entity on success.
    pub fn despawn(&mut self, id: EntityId) -> EntityResult<Entity> {
        let index = self
            .id_to_index
            .remove(&id)
            .ok_or(EntityError::NotFound(id))?;

        let entity = self.entities[index]
            .take()
            .ok_or(EntityError::AlreadyDespawned(id))?;

        self.free_list.push(index);
        Ok(entity)
    }

    /// Gets a reference to an entity by ID.
    pub fn get(&self, id: EntityId) -> EntityResult<&Entity> {
        let index = self.id_to_index.get(&id).ok_or(EntityError::NotFound(id))?;

        self.entities[*index]
            .as_ref()
            .ok_or(EntityError::NotFound(id))
    }

    /// Gets a mutable reference to an entity by ID.
    pub fn get_mut(&mut self, id: EntityId) -> EntityResult<&mut Entity> {
        let index = self.id_to_index.get(&id).ok_or(EntityError::NotFound(id))?;

        self.entities[*index]
            .as_mut()
            .ok_or(EntityError::NotFound(id))
    }

    /// Checks if an entity with the given ID exists.
    #[must_use]
    pub fn contains(&self, id: EntityId) -> bool {
        self.id_to_index.contains_key(&id)
    }

    /// Returns an iterator over all active entities.
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter().filter_map(|opt| opt.as_ref())
    }

    /// Returns a mutable iterator over all active entities.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.entities.iter_mut().filter_map(|opt| opt.as_mut())
    }

    /// Returns an iterator over entities of a specific type.
    pub fn iter_by_type(&self, entity_type: EntityType) -> impl Iterator<Item = &Entity> {
        self.iter().filter(move |e| e.entity_type() == entity_type)
    }

    /// Returns all entity IDs.
    pub fn ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.id_to_index.keys().copied()
    }

    /// Clears all entities from the arena.
    pub fn clear(&mut self) {
        self.entities.clear();
        self.free_list.clear();
        self.id_to_index.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_spawn_and_get() {
        let mut arena = EntityArena::new();

        let id = arena.spawn(EntityType::Player);
        assert!(id.is_valid());
        assert_eq!(arena.len(), 1);

        let entity = arena.get(id).expect("Entity should exist");
        assert_eq!(entity.entity_type(), EntityType::Player);
    }

    #[test]
    fn test_arena_spawn_multiple() {
        let mut arena = EntityArena::new();

        let id1 = arena.spawn(EntityType::Player);
        let id2 = arena.spawn(EntityType::Npc);
        let id3 = arena.spawn(EntityType::Vehicle);

        assert_eq!(arena.len(), 3);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);

        assert_eq!(
            arena.get(id1).expect("should exist").entity_type(),
            EntityType::Player
        );
        assert_eq!(
            arena.get(id2).expect("should exist").entity_type(),
            EntityType::Npc
        );
        assert_eq!(
            arena.get(id3).expect("should exist").entity_type(),
            EntityType::Vehicle
        );
    }

    #[test]
    fn test_arena_despawn() {
        let mut arena = EntityArena::new();

        let id = arena.spawn(EntityType::Player);
        assert_eq!(arena.len(), 1);

        let entity = arena.despawn(id).expect("Despawn should succeed");
        assert_eq!(entity.entity_type(), EntityType::Player);
        assert_eq!(arena.len(), 0);

        // Entity should no longer be accessible
        assert!(arena.get(id).is_err());
    }

    #[test]
    fn test_arena_reuse_slot() {
        let mut arena = EntityArena::new();

        // Spawn and despawn
        let id1 = arena.spawn(EntityType::Player);
        let _ = arena.despawn(id1);

        // Capacity should remain 1
        assert_eq!(arena.capacity(), 1);
        assert_eq!(arena.len(), 0);

        // Spawn again - should reuse the slot
        let id2 = arena.spawn(EntityType::Npc);
        assert_eq!(arena.capacity(), 1);
        assert_eq!(arena.len(), 1);

        // IDs should be different
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_arena_get_mut() {
        let mut arena = EntityArena::new();

        let id = arena.spawn(EntityType::Player);
        let pos = WorldCoord::new(100, 200);

        {
            let entity = arena.get_mut(id).expect("should exist");
            entity.set_position(pos);
        }

        let entity = arena.get(id).expect("should exist");
        assert_eq!(entity.position(), pos);
    }

    #[test]
    fn test_arena_iter() {
        let mut arena = EntityArena::new();

        arena.spawn(EntityType::Player);
        arena.spawn(EntityType::Npc);
        arena.spawn(EntityType::Npc);

        let count = arena.iter().count();
        assert_eq!(count, 3);

        let npc_count = arena.iter_by_type(EntityType::Npc).count();
        assert_eq!(npc_count, 2);
    }

    #[test]
    fn test_arena_contains() {
        let mut arena = EntityArena::new();

        let id = arena.spawn(EntityType::Player);
        assert!(arena.contains(id));

        let _ = arena.despawn(id);
        assert!(!arena.contains(id));

        assert!(!arena.contains(EntityId::NULL));
    }

    #[test]
    fn test_arena_clear() {
        let mut arena = EntityArena::new();

        arena.spawn(EntityType::Player);
        arena.spawn(EntityType::Npc);
        assert_eq!(arena.len(), 2);

        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_arena_with_capacity() {
        let arena = EntityArena::with_capacity(100);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_arena_despawn_nonexistent() {
        let mut arena = EntityArena::new();

        let result = arena.despawn(EntityId::NULL);
        assert!(result.is_err());

        let id = arena.spawn(EntityType::Player);
        let _ = arena.despawn(id);
        let result = arena.despawn(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_arena_spawn_entity() {
        let mut arena = EntityArena::new();

        let mut entity = Entity::new(EntityType::Player);
        entity.set_position(WorldCoord::new(50, 75));
        let expected_id = entity.id();

        let id = arena.spawn_entity(entity);
        assert_eq!(id, expected_id);

        let retrieved = arena.get(id).expect("should exist");
        assert_eq!(retrieved.position(), WorldCoord::new(50, 75));
    }
}
