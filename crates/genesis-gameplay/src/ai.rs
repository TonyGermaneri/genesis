//! AI behavior system for NPCs.
//!
//! This module provides:
//! - Extended NPC type classifications (animals, monsters)
//! - Biome-based spawn rules
//! - NPC spawning system that respects chunk boundaries

use crate::biome::BiomeType;
use crate::npc::{NPCManager, NPCType};
use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Extended NPC Classifications (G-37 Enhancement)
// ============================================================================

/// Animal types that can appear in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimalType {
    /// Produces eggs
    Chicken,
    /// Produces milk
    Cow,
    /// Produces meat
    Pig,
    /// Produces wool
    Sheep,
    /// Predator, hostile at night
    Wolf,
    /// Large predator
    Bear,
    /// Mountain animal
    Goat,
    /// Forest animal, flees
    Deer,
    /// Swamp creature
    Frog,
}

impl AnimalType {
    /// Get display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Chicken => "Chicken",
            Self::Cow => "Cow",
            Self::Pig => "Pig",
            Self::Sheep => "Sheep",
            Self::Wolf => "Wolf",
            Self::Bear => "Bear",
            Self::Goat => "Goat",
            Self::Deer => "Deer",
            Self::Frog => "Frog",
        }
    }

    /// Check if this animal is hostile.
    #[must_use]
    pub fn is_hostile(self) -> bool {
        matches!(self, Self::Wolf | Self::Bear)
    }

    /// Check if this animal is tameable.
    #[must_use]
    pub fn is_tameable(self) -> bool {
        matches!(
            self,
            Self::Chicken | Self::Cow | Self::Pig | Self::Sheep | Self::Wolf | Self::Goat
        )
    }

    /// Get the base NPC type for this animal.
    #[must_use]
    pub fn base_npc_type(self) -> NPCType {
        if self.is_hostile() {
            NPCType::Hostile
        } else {
            NPCType::Passive
        }
    }

    /// Get all animal types.
    #[must_use]
    pub const fn all() -> [Self; 9] {
        [
            Self::Chicken,
            Self::Cow,
            Self::Pig,
            Self::Sheep,
            Self::Wolf,
            Self::Bear,
            Self::Goat,
            Self::Deer,
            Self::Frog,
        ]
    }
}

/// Monster types that can appear in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterType {
    /// Weak, common enemy
    Slime,
    /// Undead ranged attacker
    Skeleton,
    /// Small aggressive humanoid
    Goblin,
    /// Larger aggressive humanoid
    Orc,
    /// Desert creature
    Scorpion,
    /// Cave creature
    Spider,
    /// Night creature
    Bat,
}

impl MonsterType {
    /// Get display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Slime => "Slime",
            Self::Skeleton => "Skeleton",
            Self::Goblin => "Goblin",
            Self::Orc => "Orc",
            Self::Scorpion => "Scorpion",
            Self::Spider => "Spider",
            Self::Bat => "Bat",
        }
    }

    /// Get base health for this monster.
    #[must_use]
    pub fn base_health(self) -> f32 {
        match self {
            Self::Slime | Self::Spider => 20.0,
            Self::Skeleton => 30.0,
            Self::Goblin => 25.0,
            Self::Orc => 50.0,
            Self::Scorpion => 15.0,
            Self::Bat => 10.0,
        }
    }

    /// Get base damage for this monster.
    #[must_use]
    pub fn base_damage(self) -> f32 {
        match self {
            Self::Slime => 5.0,
            Self::Skeleton => 10.0,
            Self::Goblin | Self::Spider => 8.0,
            Self::Orc => 15.0,
            Self::Scorpion => 12.0,
            Self::Bat => 3.0,
        }
    }

    /// Get all monster types.
    #[must_use]
    pub const fn all() -> [Self; 7] {
        [
            Self::Slime,
            Self::Skeleton,
            Self::Goblin,
            Self::Orc,
            Self::Scorpion,
            Self::Spider,
            Self::Bat,
        ]
    }
}

/// Villager profession types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VillagerProfession {
    /// No specific profession
    None,
    /// Sells/buys items
    Merchant,
    /// Heals players
    Healer,
    /// Repairs equipment
    Blacksmith,
    /// Sells food
    Farmer,
    /// Patrols and protects
    Guard,
    /// Gives quests
    QuestGiver,
}

impl VillagerProfession {
    /// Get display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::None => "Villager",
            Self::Merchant => "Merchant",
            Self::Healer => "Healer",
            Self::Blacksmith => "Blacksmith",
            Self::Farmer => "Farmer",
            Self::Guard => "Guard",
            Self::QuestGiver => "Quest Giver",
        }
    }

    /// Get base NPC type for this profession.
    #[must_use]
    pub fn base_npc_type(self) -> NPCType {
        match self {
            Self::Guard => NPCType::Guard,
            Self::Merchant => NPCType::Merchant,
            _ => NPCType::Neutral,
        }
    }
}

/// Extended NPC classification combining all types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NPCClassification {
    /// A villager with optional profession
    Villager(VillagerProfession),
    /// An animal
    Animal(AnimalType),
    /// A monster
    Monster(MonsterType),
}

impl NPCClassification {
    /// Get display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Villager(prof) => prof.display_name(),
            Self::Animal(animal) => animal.display_name(),
            Self::Monster(monster) => monster.display_name(),
        }
    }

    /// Get the base NPC type for behavior.
    #[must_use]
    pub fn base_npc_type(self) -> NPCType {
        match self {
            Self::Villager(prof) => prof.base_npc_type(),
            Self::Animal(animal) => animal.base_npc_type(),
            Self::Monster(_) => NPCType::Hostile,
        }
    }
}

// ============================================================================
// G-39: NPC Spawning System
// ============================================================================

/// Rule for spawning a specific NPC type.
#[derive(Debug, Clone)]
pub struct NPCSpawnRule {
    /// What to spawn
    pub classification: NPCClassification,
    /// Which biomes this can spawn in
    pub biomes: Vec<BiomeType>,
    /// Minimum NPCs per chunk
    pub min_density: f32,
    /// Maximum NPCs per chunk
    pub max_density: f32,
    /// Group size range (min, max)
    pub group_size: (u32, u32),
    /// Time of day range when active (0.0-1.0), None = always
    pub time_of_day: Option<(f32, f32)>,
    /// Spawn weight (higher = more common)
    pub weight: f32,
}

impl NPCSpawnRule {
    /// Create a new spawn rule.
    #[must_use]
    pub fn new(classification: NPCClassification) -> Self {
        Self {
            classification,
            biomes: Vec::new(),
            min_density: 0.0,
            max_density: 1.0,
            group_size: (1, 1),
            time_of_day: None,
            weight: 1.0,
        }
    }

    /// Set biomes where this can spawn.
    #[must_use]
    pub fn in_biomes(mut self, biomes: Vec<BiomeType>) -> Self {
        self.biomes = biomes;
        self
    }

    /// Set density range.
    #[must_use]
    pub fn with_density(mut self, min: f32, max: f32) -> Self {
        self.min_density = min;
        self.max_density = max;
        self
    }

    /// Set group size.
    #[must_use]
    pub fn with_group_size(mut self, min: u32, max: u32) -> Self {
        self.group_size = (min, max);
        self
    }

    /// Set active time of day (0.0 = midnight, 0.5 = noon).
    #[must_use]
    pub fn during_time(mut self, start: f32, end: f32) -> Self {
        self.time_of_day = Some((start, end));
        self
    }

    /// Set spawn weight.
    #[must_use]
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Check if this rule applies to a biome.
    #[must_use]
    pub fn applies_to_biome(&self, biome: BiomeType) -> bool {
        self.biomes.is_empty() || self.biomes.contains(&biome)
    }

    /// Check if this rule is active at a given time.
    #[must_use]
    pub fn is_active_at_time(&self, time: f32) -> bool {
        match self.time_of_day {
            None => true,
            Some((start, end)) => {
                if start <= end {
                    time >= start && time <= end
                } else {
                    // Wraps around midnight
                    time >= start || time <= end
                }
            },
        }
    }
}

/// Manager for NPC spawn rules.
#[derive(Debug, Clone)]
pub struct NPCSpawnRuleRegistry {
    rules: Vec<NPCSpawnRule>,
}

impl NPCSpawnRuleRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create registry with default spawn rules.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Forest spawns
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Deer))
                .in_biomes(vec![BiomeType::Forest])
                .with_density(0.5, 2.0)
                .with_group_size(1, 3)
                .with_weight(2.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Wolf))
                .in_biomes(vec![BiomeType::Forest])
                .with_density(0.1, 0.5)
                .with_group_size(2, 4)
                .during_time(0.75, 0.25) // Night
                .with_weight(1.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Villager(VillagerProfession::None))
                .in_biomes(vec![BiomeType::Forest, BiomeType::Plains])
                .with_density(0.1, 0.3)
                .with_group_size(1, 2)
                .with_weight(0.5),
        );

        // Desert spawns
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Monster(MonsterType::Scorpion))
                .in_biomes(vec![BiomeType::Desert])
                .with_density(0.3, 1.0)
                .with_group_size(1, 2)
                .with_weight(2.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Villager(VillagerProfession::Merchant))
                .in_biomes(vec![BiomeType::Desert])
                .with_density(0.0, 0.1)
                .with_group_size(1, 1)
                .with_weight(0.1),
        );

        // Plains spawns
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Cow))
                .in_biomes(vec![BiomeType::Plains])
                .with_density(0.5, 2.0)
                .with_group_size(2, 5)
                .with_weight(2.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Sheep))
                .in_biomes(vec![BiomeType::Plains])
                .with_density(0.5, 2.0)
                .with_group_size(3, 6)
                .with_weight(2.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Chicken))
                .in_biomes(vec![BiomeType::Plains])
                .with_density(0.3, 1.5)
                .with_group_size(2, 4)
                .with_weight(1.5),
        );

        // Mountain spawns
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Goat))
                .in_biomes(vec![BiomeType::Mountain])
                .with_density(0.3, 1.0)
                .with_group_size(2, 4)
                .with_weight(2.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Bear))
                .in_biomes(vec![BiomeType::Mountain, BiomeType::Forest])
                .with_density(0.05, 0.2)
                .with_group_size(1, 2)
                .with_weight(0.5),
        );

        // Swamp spawns
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Monster(MonsterType::Slime))
                .in_biomes(vec![BiomeType::Swamp])
                .with_density(0.5, 2.0)
                .with_group_size(1, 4)
                .with_weight(3.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Frog))
                .in_biomes(vec![BiomeType::Swamp, BiomeType::Lake])
                .with_density(0.3, 1.0)
                .with_group_size(2, 5)
                .with_weight(2.0),
        );

        // Generic monster spawns (night only)
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Monster(MonsterType::Skeleton))
                .in_biomes(vec![
                    BiomeType::Forest,
                    BiomeType::Plains,
                    BiomeType::Mountain,
                ])
                .with_density(0.1, 0.5)
                .with_group_size(1, 3)
                .during_time(0.75, 0.25) // Night
                .with_weight(1.0),
        );
        registry.add_rule(
            NPCSpawnRule::new(NPCClassification::Monster(MonsterType::Goblin))
                .in_biomes(vec![BiomeType::Forest, BiomeType::Swamp])
                .with_density(0.1, 0.4)
                .with_group_size(2, 4)
                .during_time(0.75, 0.25) // Night
                .with_weight(1.0),
        );

        registry
    }

    /// Add a spawn rule.
    pub fn add_rule(&mut self, rule: NPCSpawnRule) {
        self.rules.push(rule);
    }

    /// Get all rules for a biome.
    pub fn rules_for_biome(&self, biome: BiomeType) -> Vec<&NPCSpawnRule> {
        self.rules
            .iter()
            .filter(|r| r.applies_to_biome(biome))
            .collect()
    }

    /// Get active rules for a biome and time.
    pub fn active_rules(&self, biome: BiomeType, time: f32) -> Vec<&NPCSpawnRule> {
        self.rules
            .iter()
            .filter(|r| r.applies_to_biome(biome) && r.is_active_at_time(time))
            .collect()
    }

    /// Get all rules.
    pub fn all_rules(&self) -> &[NPCSpawnRule] {
        &self.rules
    }
}

impl Default for NPCSpawnRuleRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Simple LCG random number generator for deterministic spawning.
#[derive(Debug, Clone)]
pub struct SpawnRng {
    state: u64,
}

impl SpawnRng {
    /// Create a new RNG with seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Create RNG for a specific chunk.
    #[must_use]
    pub fn for_chunk(world_seed: u64, chunk_x: i32, chunk_y: i32) -> Self {
        let mut state = world_seed;
        state = state.wrapping_mul(31).wrapping_add(chunk_x as u64);
        state = state.wrapping_mul(31).wrapping_add(chunk_y as u64);
        Self::new(state)
    }

    /// Get next random u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Get random f32 in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f32) / (u64::MAX as f32)
    }

    /// Get random value in range [min, max].
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Get random u32 in range [min, max].
    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % (max - min + 1) as u64) as u32
    }

    /// Choose random item from slice.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            let idx = (self.next_u64() % items.len() as u64) as usize;
            Some(&items[idx])
        }
    }
}

/// Spawned NPC record for tracking.
#[derive(Debug, Clone)]
pub struct SpawnedNPC {
    /// Entity ID
    pub entity_id: EntityId,
    /// Classification
    pub classification: NPCClassification,
    /// Position
    pub position: (f32, f32),
    /// Chunk coordinates
    pub chunk: (i32, i32),
}

/// NPC spawning system.
#[derive(Debug)]
pub struct NPCSpawner {
    /// Spawn rules
    rules: NPCSpawnRuleRegistry,
    /// World seed for deterministic spawning
    world_seed: u64,
    /// Spawned NPCs by chunk
    spawned_by_chunk: HashMap<(i32, i32), Vec<SpawnedNPC>>,
    /// Maximum NPCs per chunk
    max_per_chunk: usize,
    /// Chunk size in world units
    chunk_size: f32,
}

impl NPCSpawner {
    /// Create a new spawner with default rules.
    #[must_use]
    pub fn new(world_seed: u64) -> Self {
        Self {
            rules: NPCSpawnRuleRegistry::with_defaults(),
            world_seed,
            spawned_by_chunk: HashMap::new(),
            max_per_chunk: 20,
            chunk_size: 64.0,
        }
    }

    /// Create spawner with custom rules.
    #[must_use]
    pub fn with_rules(world_seed: u64, rules: NPCSpawnRuleRegistry) -> Self {
        Self {
            rules,
            world_seed,
            spawned_by_chunk: HashMap::new(),
            max_per_chunk: 20,
            chunk_size: 64.0,
        }
    }

    /// Set maximum NPCs per chunk.
    pub fn set_max_per_chunk(&mut self, max: usize) {
        self.max_per_chunk = max;
    }

    /// Set chunk size.
    pub fn set_chunk_size(&mut self, size: f32) {
        self.chunk_size = size;
    }

    /// Get spawn rules.
    #[must_use]
    pub fn rules(&self) -> &NPCSpawnRuleRegistry {
        &self.rules
    }

    /// Spawn NPCs for a chunk when it loads.
    pub fn spawn_chunk(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        biome: BiomeType,
        time_of_day: f32,
        npc_manager: &mut NPCManager,
    ) -> Vec<SpawnedNPC> {
        let chunk_key = (chunk_x, chunk_y);

        // Don't respawn if already loaded
        if self.spawned_by_chunk.contains_key(&chunk_key) {
            return Vec::new();
        }

        let mut rng = SpawnRng::for_chunk(self.world_seed, chunk_x, chunk_y);
        let active_rules = self.rules.active_rules(biome, time_of_day);

        if active_rules.is_empty() {
            self.spawned_by_chunk.insert(chunk_key, Vec::new());
            return Vec::new();
        }

        let mut spawned = Vec::new();
        let base_x = chunk_x as f32 * self.chunk_size;
        let base_y = chunk_y as f32 * self.chunk_size;

        // Calculate total weight
        let total_weight: f32 = active_rules.iter().map(|r| r.weight).sum();

        // Spawn based on rules
        for rule in &active_rules {
            // Determine how many to spawn
            let density = rng.range(rule.min_density, rule.max_density);
            let spawn_count =
                (density * (rule.weight / total_weight) * self.max_per_chunk as f32).ceil() as u32;

            if spawn_count == 0 {
                continue;
            }

            // Spawn in groups
            let mut remaining = spawn_count;
            while remaining > 0 && spawned.len() < self.max_per_chunk {
                let group_size = rng
                    .range_u32(rule.group_size.0, rule.group_size.1)
                    .min(remaining);

                // Random position within chunk
                let group_x = base_x + rng.next_f32() * self.chunk_size;
                let group_y = base_y + rng.next_f32() * self.chunk_size;

                for i in 0..group_size {
                    if spawned.len() >= self.max_per_chunk {
                        break;
                    }

                    // Offset within group
                    let offset_x = (i as f32 - group_size as f32 / 2.0) * 2.0;
                    let offset_y = rng.range(-2.0, 2.0);
                    let pos = (group_x + offset_x, group_y + offset_y);

                    // Spawn the NPC
                    let npc_type = rule.classification.base_npc_type();
                    let entity_id = npc_manager.spawn_npc(npc_type, pos);

                    spawned.push(SpawnedNPC {
                        entity_id,
                        classification: rule.classification,
                        position: pos,
                        chunk: chunk_key,
                    });
                }

                remaining = remaining.saturating_sub(group_size);
            }
        }

        self.spawned_by_chunk.insert(chunk_key, spawned.clone());
        spawned
    }

    /// Despawn NPCs when a chunk unloads.
    pub fn despawn_chunk(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        npc_manager: &mut NPCManager,
    ) -> Vec<EntityId> {
        let chunk_key = (chunk_x, chunk_y);

        let mut despawned = Vec::new();
        if let Some(npcs) = self.spawned_by_chunk.remove(&chunk_key) {
            for npc in npcs {
                if npc_manager.despawn_npc(npc.entity_id).is_ok() {
                    despawned.push(npc.entity_id);
                }
            }
        }
        despawned
    }

    /// Get NPCs in a chunk.
    #[must_use]
    pub fn get_chunk_npcs(&self, chunk_x: i32, chunk_y: i32) -> &[SpawnedNPC] {
        self.spawned_by_chunk
            .get(&(chunk_x, chunk_y))
            .map_or(&[], Vec::as_slice)
    }

    /// Check if chunk has been spawned.
    #[must_use]
    pub fn is_chunk_spawned(&self, chunk_x: i32, chunk_y: i32) -> bool {
        self.spawned_by_chunk.contains_key(&(chunk_x, chunk_y))
    }

    /// Get total spawned NPC count.
    #[must_use]
    pub fn total_spawned(&self) -> usize {
        self.spawned_by_chunk.values().map(Vec::len).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Animal type tests
    #[test]
    fn test_animal_type_display_names() {
        assert_eq!(AnimalType::Chicken.display_name(), "Chicken");
        assert_eq!(AnimalType::Wolf.display_name(), "Wolf");
    }

    #[test]
    fn test_animal_type_hostility() {
        assert!(!AnimalType::Cow.is_hostile());
        assert!(AnimalType::Wolf.is_hostile());
        assert!(AnimalType::Bear.is_hostile());
    }

    #[test]
    fn test_animal_type_tameable() {
        assert!(AnimalType::Chicken.is_tameable());
        assert!(AnimalType::Wolf.is_tameable());
        assert!(!AnimalType::Deer.is_tameable());
    }

    #[test]
    fn test_animal_all() {
        let all = AnimalType::all();
        assert_eq!(all.len(), 9);
    }

    // Monster type tests
    #[test]
    fn test_monster_type_display_names() {
        assert_eq!(MonsterType::Slime.display_name(), "Slime");
        assert_eq!(MonsterType::Orc.display_name(), "Orc");
    }

    #[test]
    fn test_monster_base_stats() {
        assert!(MonsterType::Slime.base_health() < MonsterType::Orc.base_health());
        assert!(MonsterType::Slime.base_damage() < MonsterType::Orc.base_damage());
    }

    #[test]
    fn test_monster_all() {
        let all = MonsterType::all();
        assert_eq!(all.len(), 7);
    }

    // Villager profession tests
    #[test]
    fn test_villager_profession_display() {
        assert_eq!(VillagerProfession::Merchant.display_name(), "Merchant");
        assert_eq!(VillagerProfession::None.display_name(), "Villager");
    }

    #[test]
    fn test_villager_base_npc_type() {
        assert_eq!(VillagerProfession::Guard.base_npc_type(), NPCType::Guard);
        assert_eq!(
            VillagerProfession::Merchant.base_npc_type(),
            NPCType::Merchant
        );
        assert_eq!(VillagerProfession::Farmer.base_npc_type(), NPCType::Neutral);
    }

    // NPC Classification tests
    #[test]
    fn test_npc_classification_display() {
        let villager = NPCClassification::Villager(VillagerProfession::Blacksmith);
        assert_eq!(villager.display_name(), "Blacksmith");

        let animal = NPCClassification::Animal(AnimalType::Cow);
        assert_eq!(animal.display_name(), "Cow");

        let monster = NPCClassification::Monster(MonsterType::Goblin);
        assert_eq!(monster.display_name(), "Goblin");
    }

    #[test]
    fn test_npc_classification_base_type() {
        let villager = NPCClassification::Villager(VillagerProfession::None);
        assert_eq!(villager.base_npc_type(), NPCType::Neutral);

        let hostile_animal = NPCClassification::Animal(AnimalType::Wolf);
        assert_eq!(hostile_animal.base_npc_type(), NPCType::Hostile);

        let passive_animal = NPCClassification::Animal(AnimalType::Sheep);
        assert_eq!(passive_animal.base_npc_type(), NPCType::Passive);

        let monster = NPCClassification::Monster(MonsterType::Slime);
        assert_eq!(monster.base_npc_type(), NPCType::Hostile);
    }

    // Spawn rule tests
    #[test]
    fn test_spawn_rule_creation() {
        let rule = NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Cow))
            .in_biomes(vec![BiomeType::Plains])
            .with_density(0.5, 2.0)
            .with_group_size(2, 5);

        assert!(rule.applies_to_biome(BiomeType::Plains));
        assert!(!rule.applies_to_biome(BiomeType::Desert));
        assert_eq!(rule.group_size, (2, 5));
    }

    #[test]
    fn test_spawn_rule_time_of_day() {
        let day_rule =
            NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Cow)).during_time(0.25, 0.75); // Day only

        assert!(day_rule.is_active_at_time(0.5));
        assert!(!day_rule.is_active_at_time(0.0));
        assert!(!day_rule.is_active_at_time(0.9));
    }

    #[test]
    fn test_spawn_rule_night_wrap() {
        let night_rule = NPCSpawnRule::new(NPCClassification::Monster(MonsterType::Skeleton))
            .during_time(0.75, 0.25); // Night (wraps around)

        assert!(night_rule.is_active_at_time(0.8));
        assert!(night_rule.is_active_at_time(0.1));
        assert!(!night_rule.is_active_at_time(0.5));
    }

    #[test]
    fn test_spawn_rule_always_active() {
        let rule = NPCSpawnRule::new(NPCClassification::Animal(AnimalType::Cow));

        assert!(rule.is_active_at_time(0.0));
        assert!(rule.is_active_at_time(0.5));
        assert!(rule.is_active_at_time(1.0));
    }

    // Spawn rule registry tests
    #[test]
    fn test_spawn_rule_registry_defaults() {
        let registry = NPCSpawnRuleRegistry::with_defaults();
        assert!(!registry.all_rules().is_empty());
    }

    #[test]
    fn test_spawn_rule_registry_biome_filter() {
        let registry = NPCSpawnRuleRegistry::with_defaults();

        let forest_rules = registry.rules_for_biome(BiomeType::Forest);
        assert!(!forest_rules.is_empty());

        let desert_rules = registry.rules_for_biome(BiomeType::Desert);
        assert!(!desert_rules.is_empty());
    }

    #[test]
    fn test_spawn_rule_registry_time_filter() {
        let registry = NPCSpawnRuleRegistry::with_defaults();

        let day_rules = registry.active_rules(BiomeType::Forest, 0.5);
        let night_rules = registry.active_rules(BiomeType::Forest, 0.9);

        // Should have different sets (night has skeletons, etc.)
        assert!(!day_rules.is_empty());
        assert!(!night_rules.is_empty());
    }

    // SpawnRng tests
    #[test]
    fn test_spawn_rng_deterministic() {
        let mut rng1 = SpawnRng::new(12345);
        let mut rng2 = SpawnRng::new(12345);

        for _ in 0..10 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_spawn_rng_chunk_deterministic() {
        let rng1 = SpawnRng::for_chunk(42, 10, 20);
        let rng2 = SpawnRng::for_chunk(42, 10, 20);

        assert_eq!(rng1.state, rng2.state);
    }

    #[test]
    fn test_spawn_rng_different_chunks() {
        let rng1 = SpawnRng::for_chunk(42, 10, 20);
        let rng2 = SpawnRng::for_chunk(42, 11, 20);

        assert_ne!(rng1.state, rng2.state);
    }

    #[test]
    fn test_spawn_rng_range() {
        let mut rng = SpawnRng::new(12345);

        for _ in 0..100 {
            let val = rng.range(5.0, 10.0);
            assert!((5.0..=10.0).contains(&val));
        }
    }

    #[test]
    fn test_spawn_rng_range_u32() {
        let mut rng = SpawnRng::new(12345);

        for _ in 0..100 {
            let val = rng.range_u32(5, 10);
            assert!((5..=10).contains(&val));
        }
    }

    #[test]
    fn test_spawn_rng_choose() {
        let mut rng = SpawnRng::new(12345);
        let items = vec![1, 2, 3, 4, 5];

        let chosen = rng.choose(&items);
        assert!(chosen.is_some());
        assert!(items.contains(chosen.unwrap()));
    }

    #[test]
    fn test_spawn_rng_choose_empty() {
        let mut rng = SpawnRng::new(12345);
        let items: Vec<i32> = vec![];

        assert!(rng.choose(&items).is_none());
    }

    // NPC Spawner tests
    #[test]
    fn test_npc_spawner_creation() {
        let spawner = NPCSpawner::new(42);
        assert_eq!(spawner.total_spawned(), 0);
    }

    #[test]
    fn test_npc_spawner_spawn_chunk() {
        let mut spawner = NPCSpawner::new(42);
        let mut npc_manager = NPCManager::new();

        let spawned = spawner.spawn_chunk(0, 0, BiomeType::Plains, 0.5, &mut npc_manager);

        // Should spawn some NPCs
        assert!(!spawned.is_empty());
        assert!(spawner.is_chunk_spawned(0, 0));
    }

    #[test]
    fn test_npc_spawner_deterministic() {
        let mut spawner1 = NPCSpawner::new(42);
        let mut npc_manager1 = NPCManager::new();
        let spawned1 = spawner1.spawn_chunk(5, 5, BiomeType::Forest, 0.5, &mut npc_manager1);

        let mut spawner2 = NPCSpawner::new(42);
        let mut npc_manager2 = NPCManager::new();
        let spawned2 = spawner2.spawn_chunk(5, 5, BiomeType::Forest, 0.5, &mut npc_manager2);

        assert_eq!(spawned1.len(), spawned2.len());
        for (s1, s2) in spawned1.iter().zip(spawned2.iter()) {
            assert_eq!(s1.classification, s2.classification);
            assert_eq!(s1.position.0, s2.position.0);
            assert_eq!(s1.position.1, s2.position.1);
        }
    }

    #[test]
    fn test_npc_spawner_no_respawn() {
        let mut spawner = NPCSpawner::new(42);
        let mut npc_manager = NPCManager::new();

        let spawned1 = spawner.spawn_chunk(0, 0, BiomeType::Plains, 0.5, &mut npc_manager);
        let spawned2 = spawner.spawn_chunk(0, 0, BiomeType::Plains, 0.5, &mut npc_manager);

        assert!(!spawned1.is_empty());
        assert!(spawned2.is_empty()); // Already spawned
    }

    #[test]
    fn test_npc_spawner_despawn_chunk() {
        let mut spawner = NPCSpawner::new(42);
        let mut npc_manager = NPCManager::new();

        spawner.spawn_chunk(0, 0, BiomeType::Plains, 0.5, &mut npc_manager);
        let count_before = npc_manager.len();

        let despawned = spawner.despawn_chunk(0, 0, &mut npc_manager);

        assert!(!despawned.is_empty());
        assert!(npc_manager.len() < count_before);
        assert!(!spawner.is_chunk_spawned(0, 0));
    }

    #[test]
    fn test_npc_spawner_max_per_chunk() {
        let mut spawner = NPCSpawner::new(42);
        spawner.set_max_per_chunk(5);
        let mut npc_manager = NPCManager::new();

        let spawned = spawner.spawn_chunk(0, 0, BiomeType::Plains, 0.5, &mut npc_manager);

        assert!(spawned.len() <= 5);
    }

    #[test]
    fn test_npc_spawner_get_chunk_npcs() {
        let mut spawner = NPCSpawner::new(42);
        let mut npc_manager = NPCManager::new();

        spawner.spawn_chunk(0, 0, BiomeType::Plains, 0.5, &mut npc_manager);

        let npcs = spawner.get_chunk_npcs(0, 0);
        assert!(!npcs.is_empty());

        let empty = spawner.get_chunk_npcs(99, 99);
        assert!(empty.is_empty());
    }
}
