//! Faction and reputation system.

use genesis_common::FactionId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reputation standing levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReputationStanding {
    /// Hostile (-100 to -50)
    Hostile,
    /// Unfriendly (-49 to -10)
    Unfriendly,
    /// Neutral (-9 to 9)
    Neutral,
    /// Friendly (10 to 49)
    Friendly,
    /// Allied (50 to 100)
    Allied,
}

impl ReputationStanding {
    /// Converts a reputation value to a standing.
    #[must_use]
    pub const fn from_value(value: i32) -> Self {
        match value {
            ..=-50 => Self::Hostile,
            -49..=-10 => Self::Unfriendly,
            -9..=9 => Self::Neutral,
            10..=49 => Self::Friendly,
            50.. => Self::Allied,
        }
    }
}

/// Faction definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    /// Faction ID
    pub id: FactionId,
    /// Faction name
    pub name: String,
    /// Description
    pub description: String,
    /// Relations with other factions
    pub relations: HashMap<FactionId, i32>,
}

impl Faction {
    /// Creates a new faction.
    #[must_use]
    pub fn new(id: FactionId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: String::new(),
            relations: HashMap::new(),
        }
    }

    /// Sets relation with another faction.
    pub fn set_relation(&mut self, other: FactionId, value: i32) {
        self.relations.insert(other, value.clamp(-100, 100));
    }

    /// Gets relation with another faction.
    #[must_use]
    pub fn relation(&self, other: FactionId) -> i32 {
        self.relations.get(&other).copied().unwrap_or(0)
    }
}

/// Tracks player reputation with factions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationTracker {
    /// Reputation values per faction
    reputation: HashMap<FactionId, i32>,
}

impl ReputationTracker {
    /// Creates a new reputation tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets reputation value with a faction.
    #[must_use]
    pub fn value(&self, faction: FactionId) -> i32 {
        self.reputation.get(&faction).copied().unwrap_or(0)
    }

    /// Gets standing with a faction.
    #[must_use]
    pub fn standing(&self, faction: FactionId) -> ReputationStanding {
        ReputationStanding::from_value(self.value(faction))
    }

    /// Modifies reputation with a faction.
    pub fn modify(&mut self, faction: FactionId, delta: i32) {
        let current = self.value(faction);
        let new_value = (current + delta).clamp(-100, 100);
        self.reputation.insert(faction, new_value);
    }

    /// Sets reputation with a faction.
    pub fn set(&mut self, faction: FactionId, value: i32) {
        self.reputation.insert(faction, value.clamp(-100, 100));
    }
}
