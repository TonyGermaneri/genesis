//! ID types for entities and resources.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for entity IDs.
static ENTITY_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an entity in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(u64);

impl EntityId {
    /// Creates a new unique entity ID.
    #[must_use]
    pub fn new() -> Self {
        Self(ENTITY_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Creates an entity ID from a raw value (for deserialization).
    #[must_use]
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Null/invalid entity ID.
    pub const NULL: Self = Self(0);

    /// Checks if this is a valid (non-null) entity ID.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkId {
    /// X coordinate of the chunk
    pub x: i32,
    /// Y coordinate of the chunk
    pub y: i32,
    /// Layer (0 = overworld, 1+ = interiors)
    pub layer: u8,
}

impl ChunkId {
    /// Creates a new chunk ID.
    #[must_use]
    pub const fn new(x: i32, y: i32, layer: u8) -> Self {
        Self { x, y, layer }
    }

    /// Creates a chunk ID for the overworld.
    #[must_use]
    pub const fn overworld(x: i32, y: i32) -> Self {
        Self::new(x, y, 0)
    }

    /// Creates a chunk ID for an interior.
    #[must_use]
    pub const fn interior(x: i32, y: i32, interior_id: u8) -> Self {
        Self::new(x, y, interior_id)
    }
}

/// Unique identifier for an item type in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemTypeId(u32);

impl ItemTypeId {
    /// Creates an item type ID from a raw value.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Unique identifier for a recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeId(u32);

impl RecipeId {
    /// Creates a recipe ID from a raw value.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Unique identifier for a faction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactionId(u16);

impl FactionId {
    /// Creates a faction ID from a raw value.
    #[must_use]
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Neutral faction (no allegiance).
    pub const NEUTRAL: Self = Self(0);

    /// Player faction.
    pub const PLAYER: Self = Self(1);
}
