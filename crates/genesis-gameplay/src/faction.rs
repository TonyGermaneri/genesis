//! Faction and reputation system with relationships and membership.

use genesis_common::FactionId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Faction error types.
#[derive(Debug, Error)]
pub enum FactionError {
    /// Faction not found
    #[error("Faction not found: {0:?}")]
    FactionNotFound(FactionId),
    /// Already a member
    #[error("Already a member of faction {0:?}")]
    AlreadyMember(FactionId),
    /// Not a member
    #[error("Not a member of faction {0:?}")]
    NotMember(FactionId),
    /// Reputation too low
    #[error("Reputation too low: need {required}, have {current}")]
    ReputationTooLow {
        /// Required reputation
        required: i32,
        /// Current reputation
        current: i32,
    },
}

/// Result type for faction operations.
pub type FactionResult<T> = Result<T, FactionError>;

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

    /// Returns the minimum reputation value for this standing.
    #[must_use]
    pub const fn min_value(self) -> i32 {
        match self {
            Self::Hostile => -100,
            Self::Unfriendly => -49,
            Self::Neutral => -9,
            Self::Friendly => 10,
            Self::Allied => 50,
        }
    }

    /// Checks if this standing allows friendly interactions.
    #[must_use]
    pub const fn is_friendly(self) -> bool {
        matches!(self, Self::Friendly | Self::Allied)
    }

    /// Checks if this standing is hostile.
    #[must_use]
    pub const fn is_hostile(self) -> bool {
        matches!(self, Self::Hostile | Self::Unfriendly)
    }
}

/// Relationship type between factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionRelation {
    /// Allied factions
    Allied,
    /// Friendly factions
    Friendly,
    /// Neutral factions
    Neutral,
    /// Hostile factions
    Enemy,
    /// At war
    AtWar,
}

impl Default for FactionRelation {
    fn default() -> Self {
        Self::Neutral
    }
}

impl FactionRelation {
    /// Checks if this relation allows cooperation.
    #[must_use]
    pub const fn allows_cooperation(self) -> bool {
        matches!(self, Self::Allied | Self::Friendly)
    }

    /// Checks if this relation is hostile.
    #[must_use]
    pub const fn is_hostile(self) -> bool {
        matches!(self, Self::Enemy | Self::AtWar)
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
    relations: HashMap<FactionId, FactionRelation>,
    /// Minimum reputation to join
    pub join_requirement: i32,
    /// Whether the faction is joinable
    pub joinable: bool,
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
            join_requirement: 0,
            joinable: true,
        }
    }

    /// Sets the faction description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the join requirement.
    #[must_use]
    pub const fn with_join_requirement(mut self, rep: i32) -> Self {
        self.join_requirement = rep;
        self
    }

    /// Sets whether the faction is joinable.
    #[must_use]
    pub const fn with_joinable(mut self, joinable: bool) -> Self {
        self.joinable = joinable;
        self
    }

    /// Sets relation with another faction.
    pub fn set_relation(&mut self, other: FactionId, relation: FactionRelation) {
        if other != self.id {
            self.relations.insert(other, relation);
        }
    }

    /// Gets relation with another faction.
    #[must_use]
    pub fn relation(&self, other: FactionId) -> FactionRelation {
        if other == self.id {
            return FactionRelation::Allied;
        }
        self.relations.get(&other).copied().unwrap_or_default()
    }

    /// Checks if friendly with another faction.
    #[must_use]
    pub fn is_friendly_with(&self, other: FactionId) -> bool {
        self.relation(other).allows_cooperation()
    }

    /// Checks if hostile with another faction.
    #[must_use]
    pub fn is_hostile_to(&self, other: FactionId) -> bool {
        self.relation(other).is_hostile()
    }
}

/// Tracks player reputation with factions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationTracker {
    /// Reputation values per faction
    reputation: HashMap<FactionId, i32>,
    /// Faction memberships
    memberships: HashSet<FactionId>,
    /// Reputation decay rate per tick
    decay_rate: f32,
}

impl ReputationTracker {
    /// Creates a new reputation tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a tracker with custom decay rate.
    #[must_use]
    pub fn with_decay_rate(decay_rate: f32) -> Self {
        Self {
            reputation: HashMap::new(),
            memberships: HashSet::new(),
            decay_rate,
        }
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

    /// Applies reputation decay toward neutral.
    pub fn tick(&mut self) {
        if self.decay_rate == 0.0 {
            return;
        }

        for value in self.reputation.values_mut() {
            use std::cmp::Ordering;
            match (*value).cmp(&0) {
                Ordering::Greater => {
                    *value = (*value as f32 - self.decay_rate).max(0.0) as i32;
                },
                Ordering::Less => {
                    *value = (*value as f32 + self.decay_rate).min(0.0) as i32;
                },
                Ordering::Equal => {},
            }
        }
    }

    /// Checks if member of a faction.
    #[must_use]
    pub fn is_member(&self, faction: FactionId) -> bool {
        self.memberships.contains(&faction)
    }

    /// Joins a faction (if requirements met).
    pub fn join(&mut self, faction: &Faction) -> FactionResult<()> {
        if !faction.joinable {
            return Err(FactionError::ReputationTooLow {
                required: i32::MAX,
                current: self.value(faction.id),
            });
        }

        if self.memberships.contains(&faction.id) {
            return Err(FactionError::AlreadyMember(faction.id));
        }

        let current_rep = self.value(faction.id);
        if current_rep < faction.join_requirement {
            return Err(FactionError::ReputationTooLow {
                required: faction.join_requirement,
                current: current_rep,
            });
        }

        self.memberships.insert(faction.id);
        Ok(())
    }

    /// Leaves a faction.
    pub fn leave(&mut self, faction: FactionId) -> FactionResult<()> {
        if !self.memberships.remove(&faction) {
            return Err(FactionError::NotMember(faction));
        }
        Ok(())
    }

    /// Returns all faction memberships.
    pub fn memberships(&self) -> impl Iterator<Item = FactionId> + '_ {
        self.memberships.iter().copied()
    }

    /// Returns all factions with non-zero reputation.
    pub fn all_reputation(&self) -> impl Iterator<Item = (FactionId, i32)> + '_ {
        self.reputation.iter().map(|(&k, &v)| (k, v))
    }

    /// Checks if reputation meets a threshold.
    #[must_use]
    pub fn meets_requirement(&self, faction: FactionId, required: i32) -> bool {
        self.value(faction) >= required
    }

    /// Checks if standing is at least the given level.
    #[must_use]
    pub fn has_standing(&self, faction: FactionId, standing: ReputationStanding) -> bool {
        self.value(faction) >= standing.min_value()
    }
}

/// Faction registry for managing all factions.
#[derive(Debug, Default)]
pub struct FactionRegistry {
    /// All registered factions
    factions: HashMap<FactionId, Faction>,
}

impl FactionRegistry {
    /// Creates a new faction registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a faction.
    pub fn register(&mut self, faction: Faction) {
        self.factions.insert(faction.id, faction);
    }

    /// Gets a faction by ID.
    #[must_use]
    pub fn get(&self, id: FactionId) -> Option<&Faction> {
        self.factions.get(&id)
    }

    /// Gets a mutable faction by ID.
    pub fn get_mut(&mut self, id: FactionId) -> Option<&mut Faction> {
        self.factions.get_mut(&id)
    }

    /// Returns all factions.
    pub fn all(&self) -> impl Iterator<Item = &Faction> {
        self.factions.values()
    }

    /// Returns the number of factions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.factions.len()
    }

    /// Checks if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.factions.is_empty()
    }

    /// Sets mutual relation between two factions.
    pub fn set_mutual_relation(&mut self, a: FactionId, b: FactionId, relation: FactionRelation) {
        if let Some(faction_a) = self.factions.get_mut(&a) {
            faction_a.set_relation(b, relation);
        }
        if let Some(faction_b) = self.factions.get_mut(&b) {
            faction_b.set_relation(a, relation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_faction() -> Faction {
        Faction::new(FactionId::new(1), "Test Faction")
            .with_description("A test faction")
            .with_join_requirement(10)
    }

    #[test]
    fn test_reputation_standing() {
        assert_eq!(
            ReputationStanding::from_value(-100),
            ReputationStanding::Hostile
        );
        assert_eq!(
            ReputationStanding::from_value(-50),
            ReputationStanding::Hostile
        );
        assert_eq!(
            ReputationStanding::from_value(-49),
            ReputationStanding::Unfriendly
        );
        assert_eq!(
            ReputationStanding::from_value(0),
            ReputationStanding::Neutral
        );
        assert_eq!(
            ReputationStanding::from_value(25),
            ReputationStanding::Friendly
        );
        assert_eq!(
            ReputationStanding::from_value(75),
            ReputationStanding::Allied
        );
    }

    #[test]
    fn test_standing_properties() {
        assert!(ReputationStanding::Allied.is_friendly());
        assert!(ReputationStanding::Friendly.is_friendly());
        assert!(!ReputationStanding::Neutral.is_friendly());
        assert!(ReputationStanding::Hostile.is_hostile());
        assert!(!ReputationStanding::Allied.is_hostile());
    }

    #[test]
    fn test_faction_creation() {
        let faction = create_test_faction();
        assert_eq!(faction.name, "Test Faction");
        assert_eq!(faction.join_requirement, 10);
        assert!(faction.joinable);
    }

    #[test]
    fn test_faction_relations() {
        let mut faction_a = Faction::new(FactionId::new(1), "Faction A");
        let faction_b_id = FactionId::new(2);

        faction_a.set_relation(faction_b_id, FactionRelation::Enemy);
        assert!(faction_a.is_hostile_to(faction_b_id));
        assert!(!faction_a.is_friendly_with(faction_b_id));

        faction_a.set_relation(faction_b_id, FactionRelation::Allied);
        assert!(faction_a.is_friendly_with(faction_b_id));
        assert!(!faction_a.is_hostile_to(faction_b_id));
    }

    #[test]
    fn test_reputation_tracker_modify() {
        let mut tracker = ReputationTracker::new();
        let faction = FactionId::new(1);

        tracker.modify(faction, 50);
        assert_eq!(tracker.value(faction), 50);
        assert_eq!(tracker.standing(faction), ReputationStanding::Allied);

        tracker.modify(faction, -100);
        assert_eq!(tracker.value(faction), -50);
        assert_eq!(tracker.standing(faction), ReputationStanding::Hostile);
    }

    #[test]
    fn test_reputation_clamping() {
        let mut tracker = ReputationTracker::new();
        let faction = FactionId::new(1);

        tracker.modify(faction, 200);
        assert_eq!(tracker.value(faction), 100);

        tracker.modify(faction, -300);
        assert_eq!(tracker.value(faction), -100);
    }

    #[test]
    fn test_reputation_decay() {
        let mut tracker = ReputationTracker::with_decay_rate(1.0);
        let faction = FactionId::new(1);

        tracker.set(faction, 10);
        tracker.tick();
        assert_eq!(tracker.value(faction), 9);

        tracker.set(faction, -10);
        tracker.tick();
        assert_eq!(tracker.value(faction), -9);
    }

    #[test]
    fn test_faction_membership_join() {
        let faction = create_test_faction();
        let mut tracker = ReputationTracker::new();

        // Reputation too low
        let result = tracker.join(&faction);
        assert!(matches!(result, Err(FactionError::ReputationTooLow { .. })));

        // Increase reputation and join
        tracker.set(faction.id, 15);
        assert!(tracker.join(&faction).is_ok());
        assert!(tracker.is_member(faction.id));
    }

    #[test]
    fn test_faction_membership_already_member() {
        let faction = create_test_faction();
        let mut tracker = ReputationTracker::new();

        tracker.set(faction.id, 50);
        assert!(tracker.join(&faction).is_ok());

        let result = tracker.join(&faction);
        assert!(matches!(result, Err(FactionError::AlreadyMember(_))));
    }

    #[test]
    fn test_faction_membership_leave() {
        let faction = create_test_faction();
        let mut tracker = ReputationTracker::new();

        tracker.set(faction.id, 50);
        let _ = tracker.join(&faction);

        assert!(tracker.leave(faction.id).is_ok());
        assert!(!tracker.is_member(faction.id));
    }

    #[test]
    fn test_faction_membership_not_member() {
        let mut tracker = ReputationTracker::new();
        let result = tracker.leave(FactionId::new(1));
        assert!(matches!(result, Err(FactionError::NotMember(_))));
    }

    #[test]
    fn test_meets_requirement() {
        let mut tracker = ReputationTracker::new();
        let faction = FactionId::new(1);

        tracker.set(faction, 25);
        assert!(tracker.meets_requirement(faction, 20));
        assert!(!tracker.meets_requirement(faction, 30));
    }

    #[test]
    fn test_has_standing() {
        let mut tracker = ReputationTracker::new();
        let faction = FactionId::new(1);

        tracker.set(faction, 30);
        assert!(tracker.has_standing(faction, ReputationStanding::Friendly));
        assert!(!tracker.has_standing(faction, ReputationStanding::Allied));
    }

    #[test]
    fn test_faction_registry() {
        let mut registry = FactionRegistry::new();
        registry.register(Faction::new(FactionId::new(1), "Faction A"));
        registry.register(Faction::new(FactionId::new(2), "Faction B"));

        assert_eq!(registry.len(), 2);
        assert!(registry.get(FactionId::new(1)).is_some());
    }

    #[test]
    fn test_faction_registry_mutual_relation() {
        let mut registry = FactionRegistry::new();
        let id_a = FactionId::new(1);
        let id_b = FactionId::new(2);

        registry.register(Faction::new(id_a, "Faction A"));
        registry.register(Faction::new(id_b, "Faction B"));

        registry.set_mutual_relation(id_a, id_b, FactionRelation::Enemy);

        assert!(registry.get(id_a).expect("exists").is_hostile_to(id_b));
        assert!(registry.get(id_b).expect("exists").is_hostile_to(id_a));
    }

    #[test]
    fn test_unjoinable_faction() {
        let faction = Faction::new(FactionId::new(1), "Exclusive").with_joinable(false);
        let mut tracker = ReputationTracker::new();
        tracker.set(faction.id, 100);

        let result = tracker.join(&faction);
        assert!(result.is_err());
    }
}
