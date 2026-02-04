//! NPC state persistence.
//!
//! This module provides comprehensive NPC state saving:
//! - Position, rotation, movement state
//! - AI state (current behavior, targets, patrol data)
//! - Health and combat state
//! - Inventory and equipment
//! - Dialogue progress and flags
//! - Respawn timers and death state

use genesis_common::{EntityId, FactionId, ItemTypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// G-54: AI State Save
// ============================================================================

/// AI behavior state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiBehaviorState {
    /// Idle/waiting.
    Idle,
    /// Patrolling a route.
    Patrol,
    /// Wandering randomly.
    Wander,
    /// Following a target.
    Follow,
    /// Chasing a hostile target.
    Chase,
    /// Engaging in combat.
    Combat,
    /// Fleeing from danger.
    Flee,
    /// Returning to home position.
    ReturnHome,
    /// Performing scripted action.
    Scripted,
    /// Dead/inactive.
    Dead,
    /// Talking to player.
    Dialogue,
    /// Working (crafting, farming, etc.).
    Working,
    /// Sleeping/resting.
    Sleeping,
}

impl Default for AiBehaviorState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Patrol route save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatrolRouteSave {
    /// Patrol waypoints (x, y).
    pub waypoints: Vec<(f32, f32)>,
    /// Current waypoint index.
    pub current_index: usize,
    /// Whether patrol loops.
    pub loops: bool,
    /// Whether currently going forward (for non-looping).
    pub forward: bool,
    /// Wait time at each waypoint.
    pub wait_time: f32,
    /// Current wait timer.
    pub current_wait: f32,
}

impl Default for PatrolRouteSave {
    fn default() -> Self {
        Self {
            waypoints: Vec::new(),
            current_index: 0,
            loops: true,
            forward: true,
            wait_time: 2.0,
            current_wait: 0.0,
        }
    }
}

impl PatrolRouteSave {
    /// Create new patrol route.
    #[must_use]
    pub fn new(waypoints: Vec<(f32, f32)>) -> Self {
        Self {
            waypoints,
            ..Default::default()
        }
    }

    /// Set looping behavior.
    #[must_use]
    pub fn with_loop(mut self, loops: bool) -> Self {
        self.loops = loops;
        self
    }

    /// Set wait time.
    #[must_use]
    pub fn with_wait_time(mut self, time: f32) -> Self {
        self.wait_time = time;
        self
    }

    /// Get current waypoint.
    #[must_use]
    pub fn current_waypoint(&self) -> Option<(f32, f32)> {
        self.waypoints.get(self.current_index).copied()
    }

    /// Check if patrol is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }
}

/// AI target save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiTargetSave {
    /// Target entity ID (if entity).
    pub entity_id: Option<u64>,
    /// Target position.
    pub position: (f32, f32),
    /// Target priority.
    pub priority: u32,
    /// Time target was acquired.
    pub acquired_at: f64,
    /// Whether target is hostile.
    pub hostile: bool,
}

impl AiTargetSave {
    /// Create target from entity.
    #[must_use]
    pub fn from_entity(entity_id: EntityId, position: (f32, f32)) -> Self {
        Self {
            entity_id: Some(entity_id.raw()),
            position,
            priority: 1,
            acquired_at: 0.0,
            hostile: false,
        }
    }

    /// Create target from position.
    #[must_use]
    pub fn from_position(position: (f32, f32)) -> Self {
        Self {
            entity_id: None,
            position,
            priority: 0,
            acquired_at: 0.0,
            hostile: false,
        }
    }

    /// Set priority.
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set hostile.
    #[must_use]
    pub fn with_hostile(mut self, hostile: bool) -> Self {
        self.hostile = hostile;
        self
    }

    /// Get entity ID.
    #[must_use]
    pub fn entity_id(&self) -> Option<EntityId> {
        self.entity_id.map(EntityId::from_raw)
    }
}

/// Complete AI state save.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AiStateSave {
    /// Current behavior state.
    pub behavior: AiBehaviorState,
    /// Previous behavior (for resuming).
    pub previous_behavior: Option<AiBehaviorState>,
    /// Current target.
    pub target: Option<AiTargetSave>,
    /// Secondary targets.
    pub secondary_targets: Vec<AiTargetSave>,
    /// Patrol route (if patrolling).
    pub patrol: Option<PatrolRouteSave>,
    /// Home position.
    pub home_position: (f32, f32),
    /// Max distance from home.
    pub leash_distance: f32,
    /// Aggro range.
    pub aggro_range: f32,
    /// Time in current state.
    pub state_time: f64,
    /// Whether AI is active.
    pub active: bool,
    /// Custom AI flags.
    pub flags: HashMap<String, bool>,
}

impl AiStateSave {
    /// Create new AI state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            leash_distance: 50.0,
            aggro_range: 15.0,
            active: true,
            ..Default::default()
        }
    }

    /// Set behavior state.
    #[must_use]
    pub fn with_behavior(mut self, behavior: AiBehaviorState) -> Self {
        self.behavior = behavior;
        self
    }

    /// Set home position.
    #[must_use]
    pub fn with_home(mut self, x: f32, y: f32) -> Self {
        self.home_position = (x, y);
        self
    }

    /// Set patrol route.
    #[must_use]
    pub fn with_patrol(mut self, patrol: PatrolRouteSave) -> Self {
        self.patrol = Some(patrol);
        self
    }

    /// Set target.
    pub fn set_target(&mut self, target: AiTargetSave) {
        self.target = Some(target);
    }

    /// Clear target.
    pub fn clear_target(&mut self) {
        self.target = None;
    }

    /// Set AI flag.
    pub fn set_flag(&mut self, flag: impl Into<String>, value: bool) {
        self.flags.insert(flag.into(), value);
    }

    /// Get AI flag.
    #[must_use]
    pub fn get_flag(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    /// Check if in combat.
    #[must_use]
    pub fn is_in_combat(&self) -> bool {
        matches!(
            self.behavior,
            AiBehaviorState::Combat | AiBehaviorState::Chase
        )
    }
}

// ============================================================================
// G-54: NPC Combat State Save
// ============================================================================

/// Combat state save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NpcCombatSave {
    /// Current health.
    pub health: f32,
    /// Maximum health.
    pub max_health: f32,
    /// Current stamina.
    pub stamina: f32,
    /// Maximum stamina.
    pub max_stamina: f32,
    /// Last damage taken timestamp.
    pub last_damage_time: f64,
    /// Entities that damaged this NPC (for aggro/loot).
    pub damage_sources: HashMap<u64, f32>,
    /// Attack cooldown remaining.
    pub attack_cooldown: f32,
    /// Whether currently attacking.
    pub is_attacking: bool,
    /// Stagger time remaining.
    pub stagger_time: f32,
}

impl NpcCombatSave {
    /// Create new combat state.
    #[must_use]
    pub fn new(max_health: f32) -> Self {
        Self {
            health: max_health,
            max_health,
            stamina: 100.0,
            max_stamina: 100.0,
            ..Default::default()
        }
    }

    /// Set health.
    #[must_use]
    pub fn with_health(mut self, health: f32) -> Self {
        self.health = health;
        self
    }

    /// Set stamina.
    #[must_use]
    pub fn with_stamina(mut self, stamina: f32, max: f32) -> Self {
        self.stamina = stamina;
        self.max_stamina = max;
        self
    }

    /// Record damage from source.
    pub fn record_damage(&mut self, source: EntityId, amount: f32, time: f64) {
        *self.damage_sources.entry(source.raw()).or_insert(0.0) += amount;
        self.last_damage_time = time;
    }

    /// Get health percentage.
    #[must_use]
    pub fn health_percent(&self) -> f32 {
        if self.max_health > 0.0 {
            self.health / self.max_health * 100.0
        } else {
            0.0
        }
    }

    /// Check if dead.
    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Get top damage dealer.
    #[must_use]
    pub fn top_damage_dealer(&self) -> Option<EntityId> {
        self.damage_sources
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| EntityId::from_raw(*id))
    }
}

// ============================================================================
// G-54: NPC Inventory Save
// ============================================================================

/// NPC inventory item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcItemSave {
    /// Item type ID.
    pub item_type: u32,
    /// Quantity.
    pub quantity: u32,
    /// Drop chance (0.0-1.0).
    pub drop_chance: f32,
    /// Whether item is equipped.
    pub equipped: bool,
}

impl NpcItemSave {
    /// Create new NPC item.
    #[must_use]
    pub fn new(item_type: ItemTypeId, quantity: u32) -> Self {
        Self {
            item_type: item_type.raw(),
            quantity,
            drop_chance: 1.0,
            equipped: false,
        }
    }

    /// Set drop chance.
    #[must_use]
    pub fn with_drop_chance(mut self, chance: f32) -> Self {
        self.drop_chance = chance.clamp(0.0, 1.0);
        self
    }

    /// Set equipped.
    #[must_use]
    pub fn with_equipped(mut self, equipped: bool) -> Self {
        self.equipped = equipped;
        self
    }

    /// Get item type ID.
    #[must_use]
    pub fn item_type(&self) -> ItemTypeId {
        ItemTypeId::new(self.item_type)
    }
}

/// NPC inventory save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NpcInventorySave {
    /// Items in inventory.
    pub items: Vec<NpcItemSave>,
    /// Currency amount.
    pub currency: u32,
    /// Currency drop chance.
    pub currency_drop_chance: f32,
}

impl NpcInventorySave {
    /// Create new inventory.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add item.
    pub fn add_item(&mut self, item: NpcItemSave) {
        self.items.push(item);
    }

    /// Set currency.
    #[must_use]
    pub fn with_currency(mut self, amount: u32, drop_chance: f32) -> Self {
        self.currency = amount;
        self.currency_drop_chance = drop_chance.clamp(0.0, 1.0);
        self
    }

    /// Get equipped items.
    #[must_use]
    pub fn equipped_items(&self) -> Vec<&NpcItemSave> {
        self.items.iter().filter(|i| i.equipped).collect()
    }

    /// Count total items.
    #[must_use]
    pub fn total_items(&self) -> u32 {
        self.items.iter().map(|i| i.quantity).sum()
    }
}

// ============================================================================
// G-54: Dialogue State Save
// ============================================================================

/// Dialogue progress save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogueSave {
    /// Conversation IDs that have been started.
    pub started_conversations: HashSet<String>,
    /// Conversation IDs that have been completed.
    pub completed_conversations: HashSet<String>,
    /// Selected dialogue choices (conversation_id -> choice_ids).
    pub choices_made: HashMap<String, Vec<String>>,
    /// Dialogue flags (for conditional dialogue).
    pub flags: HashMap<String, bool>,
    /// Relationship/affinity level.
    pub affinity: i32,
    /// Times player has talked to this NPC.
    pub interaction_count: u32,
    /// Last interaction timestamp.
    pub last_interaction: f64,
}

impl DialogueSave {
    /// Create new dialogue save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a conversation.
    pub fn start_conversation(&mut self, conversation_id: impl Into<String>) {
        self.started_conversations.insert(conversation_id.into());
        self.interaction_count += 1;
    }

    /// Complete a conversation.
    pub fn complete_conversation(&mut self, conversation_id: impl Into<String>) {
        let id = conversation_id.into();
        self.completed_conversations.insert(id);
    }

    /// Record a dialogue choice.
    pub fn record_choice(
        &mut self,
        conversation_id: impl Into<String>,
        choice_id: impl Into<String>,
    ) {
        self.choices_made
            .entry(conversation_id.into())
            .or_default()
            .push(choice_id.into());
    }

    /// Check if conversation was completed.
    #[must_use]
    pub fn is_completed(&self, conversation_id: &str) -> bool {
        self.completed_conversations.contains(conversation_id)
    }

    /// Set dialogue flag.
    pub fn set_flag(&mut self, flag: impl Into<String>, value: bool) {
        self.flags.insert(flag.into(), value);
    }

    /// Get dialogue flag.
    #[must_use]
    pub fn get_flag(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    /// Modify affinity.
    pub fn modify_affinity(&mut self, delta: i32) {
        self.affinity = self.affinity.saturating_add(delta);
    }

    /// Get affinity level name.
    #[must_use]
    pub fn affinity_level(&self) -> &'static str {
        match self.affinity {
            i32::MIN..=-50 => "Hated",
            -49..=-20 => "Disliked",
            -19..=19 => "Neutral",
            20..=49 => "Friendly",
            50..=79 => "Trusted",
            80..=i32::MAX => "Beloved",
        }
    }
}

// ============================================================================
// G-54: Respawn State Save
// ============================================================================

/// Respawn state save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RespawnSave {
    /// Whether NPC is currently dead.
    pub is_dead: bool,
    /// Time of death (game time).
    pub death_time: f64,
    /// Respawn delay in seconds.
    pub respawn_delay: f64,
    /// Position to respawn at.
    pub respawn_position: (f32, f32),
    /// Whether NPC can respawn.
    pub can_respawn: bool,
    /// Number of times respawned.
    pub respawn_count: u32,
    /// Maximum respawns allowed (None = infinite).
    pub max_respawns: Option<u32>,
    /// Position where NPC died.
    pub death_position: Option<(f32, f32)>,
}

impl RespawnSave {
    /// Create new respawn state.
    #[must_use]
    pub fn new(respawn_position: (f32, f32)) -> Self {
        Self {
            respawn_position,
            respawn_delay: 300.0, // 5 minutes default
            can_respawn: true,
            ..Default::default()
        }
    }

    /// Set respawn delay.
    #[must_use]
    pub fn with_delay(mut self, delay: f64) -> Self {
        self.respawn_delay = delay;
        self
    }

    /// Set max respawns.
    #[must_use]
    pub fn with_max_respawns(mut self, max: u32) -> Self {
        self.max_respawns = Some(max);
        self
    }

    /// Record death.
    pub fn record_death(&mut self, position: (f32, f32), time: f64) {
        self.is_dead = true;
        self.death_time = time;
        self.death_position = Some(position);
    }

    /// Check if ready to respawn.
    #[must_use]
    pub fn is_ready_to_respawn(&self, current_time: f64) -> bool {
        if !self.is_dead || !self.can_respawn {
            return false;
        }

        // Check max respawns
        if let Some(max) = self.max_respawns {
            if self.respawn_count >= max {
                return false;
            }
        }

        current_time >= self.death_time + self.respawn_delay
    }

    /// Get time until respawn.
    #[must_use]
    pub fn time_until_respawn(&self, current_time: f64) -> f64 {
        if !self.is_dead {
            return 0.0;
        }

        let ready_time = self.death_time + self.respawn_delay;
        (ready_time - current_time).max(0.0)
    }

    /// Perform respawn.
    pub fn respawn(&mut self) {
        self.is_dead = false;
        self.respawn_count += 1;
        self.death_position = None;
    }
}

// ============================================================================
// G-54: Complete NPC Save
// ============================================================================

/// NPC type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcType {
    /// Friendly NPC (villager, merchant).
    Friendly,
    /// Neutral NPC (wildlife).
    Neutral,
    /// Hostile NPC (enemy).
    Hostile,
    /// Quest-related NPC.
    Quest,
    /// Merchant/vendor.
    Merchant,
    /// Guard/protector.
    Guard,
    /// Boss enemy.
    Boss,
    /// Companion/follower.
    Companion,
}

impl Default for NpcType {
    fn default() -> Self {
        Self::Neutral
    }
}

/// Complete NPC save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcFullSave {
    /// Save format version.
    pub version: u32,
    /// NPC unique ID.
    pub npc_id: u64,
    /// NPC template/definition ID.
    pub template_id: u32,
    /// NPC type.
    pub npc_type: NpcType,
    /// Display name.
    pub name: String,
    /// Current position.
    pub position: (f32, f32),
    /// Rotation/facing.
    pub rotation: f32,
    /// Velocity.
    pub velocity: (f32, f32),
    /// Faction ID.
    pub faction_id: u32,
    /// AI state.
    pub ai_state: AiStateSave,
    /// Combat state.
    pub combat: NpcCombatSave,
    /// Inventory.
    pub inventory: NpcInventorySave,
    /// Dialogue progress (keyed by player ID for multi-player).
    pub dialogue: HashMap<u64, DialogueSave>,
    /// Single-player dialogue progress.
    pub dialogue_single: DialogueSave,
    /// Respawn state.
    pub respawn: RespawnSave,
    /// Custom data/flags.
    pub custom_data: HashMap<String, String>,
    /// Whether NPC is persistent (saved with world).
    pub persistent: bool,
}

impl NpcFullSave {
    /// Create new NPC save.
    #[must_use]
    pub fn new(npc_id: EntityId, template_id: u32, name: impl Into<String>) -> Self {
        Self {
            version: 1,
            npc_id: npc_id.raw(),
            template_id,
            npc_type: NpcType::default(),
            name: name.into(),
            position: (0.0, 0.0),
            rotation: 0.0,
            velocity: (0.0, 0.0),
            faction_id: 0,
            ai_state: AiStateSave::new(),
            combat: NpcCombatSave::new(100.0),
            inventory: NpcInventorySave::new(),
            dialogue: HashMap::new(),
            dialogue_single: DialogueSave::new(),
            respawn: RespawnSave::new((0.0, 0.0)),
            custom_data: HashMap::new(),
            persistent: true,
        }
    }

    /// Set NPC type.
    #[must_use]
    pub fn with_type(mut self, npc_type: NpcType) -> Self {
        self.npc_type = npc_type;
        self
    }

    /// Set position.
    #[must_use]
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self.respawn.respawn_position = (x, y);
        self.ai_state.home_position = (x, y);
        self
    }

    /// Set rotation.
    #[must_use]
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set faction.
    #[must_use]
    pub fn with_faction(mut self, faction_id: FactionId) -> Self {
        self.faction_id = faction_id.raw() as u32;
        self
    }

    /// Set AI state.
    #[must_use]
    pub fn with_ai_state(mut self, ai_state: AiStateSave) -> Self {
        self.ai_state = ai_state;
        self
    }

    /// Set combat state.
    #[must_use]
    pub fn with_combat(mut self, combat: NpcCombatSave) -> Self {
        self.combat = combat;
        self
    }

    /// Set inventory.
    #[must_use]
    pub fn with_inventory(mut self, inventory: NpcInventorySave) -> Self {
        self.inventory = inventory;
        self
    }

    /// Set respawn state.
    #[must_use]
    pub fn with_respawn(mut self, respawn: RespawnSave) -> Self {
        self.respawn = respawn;
        self
    }

    /// Get NPC entity ID.
    #[must_use]
    pub fn npc_id(&self) -> EntityId {
        EntityId::from_raw(self.npc_id)
    }

    /// Get faction ID.
    #[must_use]
    pub fn faction_id(&self) -> FactionId {
        FactionId::new(self.faction_id as u16)
    }

    /// Check if NPC is alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        !self.respawn.is_dead && self.combat.health > 0.0
    }

    /// Check if NPC is hostile.
    #[must_use]
    pub fn is_hostile(&self) -> bool {
        matches!(self.npc_type, NpcType::Hostile | NpcType::Boss)
    }

    /// Set custom data.
    pub fn set_data(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom_data.insert(key.into(), value.into());
    }

    /// Get custom data.
    #[must_use]
    pub fn get_data(&self, key: &str) -> Option<&str> {
        self.custom_data.get(key).map(String::as_str)
    }

    /// Get dialogue for player.
    #[must_use]
    pub fn get_dialogue(&self, player_id: Option<EntityId>) -> &DialogueSave {
        match player_id {
            Some(id) => self
                .dialogue
                .get(&id.raw())
                .unwrap_or(&self.dialogue_single),
            None => &self.dialogue_single,
        }
    }

    /// Get mutable dialogue for player.
    pub fn get_dialogue_mut(&mut self, player_id: Option<EntityId>) -> &mut DialogueSave {
        match player_id {
            Some(id) => self.dialogue.entry(id.raw()).or_default(),
            None => &mut self.dialogue_single,
        }
    }
}

// ============================================================================
// G-54: NPC Collection Save
// ============================================================================

/// Collection of NPCs to save.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NpcCollectionSave {
    /// Version.
    pub version: u32,
    /// All NPC saves.
    pub npcs: Vec<NpcFullSave>,
    /// NPCs that have been permanently killed.
    pub permanently_dead: HashSet<u64>,
}

impl NpcCollectionSave {
    /// Create new NPC collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: 1,
            npcs: Vec::new(),
            permanently_dead: HashSet::new(),
        }
    }

    /// Add NPC.
    pub fn add(&mut self, npc: NpcFullSave) {
        self.npcs.push(npc);
    }

    /// Get NPC by ID.
    #[must_use]
    pub fn get(&self, npc_id: EntityId) -> Option<&NpcFullSave> {
        self.npcs.iter().find(|n| n.npc_id == npc_id.raw())
    }

    /// Get mutable NPC by ID.
    pub fn get_mut(&mut self, npc_id: EntityId) -> Option<&mut NpcFullSave> {
        self.npcs.iter_mut().find(|n| n.npc_id == npc_id.raw())
    }

    /// Remove NPC.
    pub fn remove(&mut self, npc_id: EntityId) {
        self.npcs.retain(|n| n.npc_id != npc_id.raw());
    }

    /// Mark NPC as permanently dead.
    pub fn mark_permanently_dead(&mut self, npc_id: EntityId) {
        self.permanently_dead.insert(npc_id.raw());
        self.remove(npc_id);
    }

    /// Check if NPC is permanently dead.
    #[must_use]
    pub fn is_permanently_dead(&self, npc_id: EntityId) -> bool {
        self.permanently_dead.contains(&npc_id.raw())
    }

    /// Count living NPCs.
    #[must_use]
    pub fn living_count(&self) -> usize {
        self.npcs.iter().filter(|n| n.is_alive()).count()
    }

    /// Count all NPCs.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.npcs.len()
    }

    /// Get NPCs by type.
    #[must_use]
    pub fn by_type(&self, npc_type: NpcType) -> Vec<&NpcFullSave> {
        self.npcs
            .iter()
            .filter(|n| n.npc_type == npc_type)
            .collect()
    }

    /// Get NPCs ready for respawn.
    #[must_use]
    pub fn ready_for_respawn(&self, current_time: f64) -> Vec<&NpcFullSave> {
        self.npcs
            .iter()
            .filter(|n| n.respawn.is_ready_to_respawn(current_time))
            .collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_behavior_default() {
        let state = AiBehaviorState::default();
        assert_eq!(state, AiBehaviorState::Idle);
    }

    #[test]
    fn test_patrol_route() {
        let patrol = PatrolRouteSave::new(vec![(0.0, 0.0), (10.0, 10.0), (20.0, 0.0)])
            .with_loop(true)
            .with_wait_time(5.0);

        assert_eq!(patrol.current_waypoint(), Some((0.0, 0.0)));
        assert!(!patrol.is_empty());
        assert_eq!(patrol.wait_time, 5.0);
    }

    #[test]
    fn test_ai_target() {
        let target = AiTargetSave::from_entity(EntityId::from_raw(1), (50.0, 50.0))
            .with_priority(5)
            .with_hostile(true);

        assert_eq!(target.entity_id(), Some(EntityId::from_raw(1)));
        assert!(target.hostile);
        assert_eq!(target.priority, 5);
    }

    #[test]
    fn test_ai_state() {
        let mut state = AiStateSave::new()
            .with_behavior(AiBehaviorState::Patrol)
            .with_home(100.0, 100.0);

        state.set_flag("alerted", true);

        assert_eq!(state.behavior, AiBehaviorState::Patrol);
        assert!(state.get_flag("alerted"));
        assert!(!state.is_in_combat());
    }

    #[test]
    fn test_npc_combat_save() {
        let mut combat = NpcCombatSave::new(100.0);
        combat.record_damage(EntityId::from_raw(1), 25.0, 100.0);

        assert_eq!(combat.health_percent(), 100.0);
        assert_eq!(combat.top_damage_dealer(), Some(EntityId::from_raw(1)));
    }

    #[test]
    fn test_npc_item() {
        let item = NpcItemSave::new(ItemTypeId::new(5), 3)
            .with_drop_chance(0.5)
            .with_equipped(true);

        assert_eq!(item.item_type(), ItemTypeId::new(5));
        assert!(item.equipped);
        assert_eq!(item.drop_chance, 0.5);
    }

    #[test]
    fn test_npc_inventory() {
        let mut inv = NpcInventorySave::new().with_currency(100, 0.75);
        inv.add_item(NpcItemSave::new(ItemTypeId::new(1), 5));
        inv.add_item(NpcItemSave::new(ItemTypeId::new(2), 3).with_equipped(true));

        assert_eq!(inv.total_items(), 8);
        assert_eq!(inv.equipped_items().len(), 1);
    }

    #[test]
    fn test_dialogue_save() {
        let mut dialogue = DialogueSave::new();
        dialogue.start_conversation("greeting");
        dialogue.record_choice("greeting", "friendly");
        dialogue.complete_conversation("greeting");
        dialogue.modify_affinity(25);

        assert!(dialogue.is_completed("greeting"));
        assert_eq!(dialogue.affinity, 25);
        assert_eq!(dialogue.affinity_level(), "Friendly");
    }

    #[test]
    fn test_respawn_save() {
        let mut respawn = RespawnSave::new((0.0, 0.0))
            .with_delay(60.0)
            .with_max_respawns(3);

        respawn.record_death((10.0, 20.0), 100.0);

        assert!(respawn.is_dead);
        assert!(!respawn.is_ready_to_respawn(150.0)); // 100 + 60 = 160
        assert!(respawn.is_ready_to_respawn(161.0));
    }

    #[test]
    fn test_respawn_time_calculation() {
        let mut respawn = RespawnSave::new((0.0, 0.0)).with_delay(30.0);
        respawn.record_death((0.0, 0.0), 100.0);

        assert_eq!(respawn.time_until_respawn(110.0), 20.0);
        assert_eq!(respawn.time_until_respawn(130.0), 0.0);
    }

    #[test]
    fn test_npc_full_save() {
        let npc = NpcFullSave::new(EntityId::from_raw(1), 100, "Guard")
            .with_type(NpcType::Guard)
            .with_position(50.0, 50.0)
            .with_faction(FactionId::new(1));

        assert_eq!(npc.name, "Guard");
        assert_eq!(npc.npc_type, NpcType::Guard);
        assert!(npc.is_alive());
        assert!(!npc.is_hostile());
    }

    #[test]
    fn test_npc_collection() {
        let mut collection = NpcCollectionSave::new();
        collection.add(NpcFullSave::new(EntityId::from_raw(1), 1, "NPC1"));
        collection
            .add(NpcFullSave::new(EntityId::from_raw(2), 2, "NPC2").with_type(NpcType::Hostile));

        assert_eq!(collection.total_count(), 2);
        assert_eq!(collection.by_type(NpcType::Hostile).len(), 1);
    }

    #[test]
    fn test_npc_permanent_death() {
        let mut collection = NpcCollectionSave::new();
        collection.add(NpcFullSave::new(EntityId::from_raw(1), 1, "Boss"));
        collection.mark_permanently_dead(EntityId::from_raw(1));

        assert!(collection.is_permanently_dead(EntityId::from_raw(1)));
        assert_eq!(collection.total_count(), 0);
    }
}
