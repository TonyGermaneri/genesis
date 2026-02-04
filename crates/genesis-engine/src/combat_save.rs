//! Combat persistence system.
//!
//! This module provides:
//! - Saving HP, stamina, and combat stats
//! - Persisting status effects and equipped weapon
//! - Loading combat state on game load
//! - Migration for combat save format changes

use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info};

use crate::combat_events::{CombatStats, StatusEffect};

/// Current combat save format version.
pub const COMBAT_SAVE_VERSION: u32 = 1;

/// Errors that can occur during combat persistence.
#[derive(Debug, Error)]
pub enum CombatSaveError {
    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Version mismatch.
    #[error("Incompatible combat save version: expected {expected}, found {found}")]
    VersionMismatch {
        /// Expected version.
        expected: u32,
        /// Found version.
        found: u32,
    },

    /// Corrupted data.
    #[error("Corrupted combat data: {0}")]
    Corrupted(String),

    /// Migration failed.
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

/// Result type for combat save operations.
pub type CombatSaveResult<T> = Result<T, CombatSaveError>;

/// Saved status effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffectSaveData {
    /// Effect type name.
    pub effect_type: String,
    /// Duration remaining.
    pub duration: f32,
    /// Stack count.
    pub stacks: u32,
    /// Source entity (if any).
    pub source: Option<u64>,
}

impl StatusEffectSaveData {
    /// Creates save data from a status effect.
    #[must_use]
    pub fn from_effect(effect: StatusEffect, duration: f32, stacks: u32, source: Option<EntityId>) -> Self {
        Self {
            effect_type: format!("{effect:?}"),
            duration,
            stacks,
            source: source.map(|e| e.raw()),
        }
    }

    /// Converts effect type string back to enum.
    #[must_use]
    pub fn to_effect(&self) -> Option<StatusEffect> {
        match self.effect_type.as_str() {
            "Burning" => Some(StatusEffect::Burning),
            "Poisoned" => Some(StatusEffect::Poisoned),
            "Frozen" => Some(StatusEffect::Frozen),
            "Stunned" => Some(StatusEffect::Stunned),
            "Bleeding" => Some(StatusEffect::Bleeding),
            "Weakened" => Some(StatusEffect::Weakened),
            "Strengthened" => Some(StatusEffect::Strengthened),
            "Regenerating" => Some(StatusEffect::Regenerating),
            "Shielded" => Some(StatusEffect::Shielded),
            _ => None,
        }
    }
}

/// Saved entity combat state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCombatSaveData {
    /// Entity ID.
    pub entity_id: u64,
    /// Current health.
    pub health: f32,
    /// Maximum health.
    pub max_health: f32,
    /// Current stamina.
    pub stamina: f32,
    /// Maximum stamina.
    pub max_stamina: f32,
    /// Armor rating.
    pub armor: f32,
    /// Active status effects.
    pub status_effects: Vec<StatusEffectSaveData>,
    /// Equipped weapon ID.
    pub equipped_weapon: Option<u32>,
    /// Equipped offhand ID (shield, etc.).
    pub equipped_offhand: Option<u32>,
    /// Block stamina remaining (for active blocking).
    pub block_stamina: f32,
    /// Time until next attack is ready.
    pub attack_cooldown: f32,
    /// Current combo count.
    pub combo_count: u32,
    /// Damage resistances by type.
    pub resistances: HashMap<String, f32>,
    /// Whether the entity is dead.
    pub is_dead: bool,
    /// Respawn timer (if dead).
    pub respawn_timer: f32,
}

impl Default for EntityCombatSaveData {
    fn default() -> Self {
        Self {
            entity_id: 0,
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            armor: 0.0,
            status_effects: Vec::new(),
            equipped_weapon: None,
            equipped_offhand: None,
            block_stamina: 50.0,
            attack_cooldown: 0.0,
            combo_count: 0,
            resistances: HashMap::new(),
            is_dead: false,
            respawn_timer: 0.0,
        }
    }
}

impl EntityCombatSaveData {
    /// Creates new combat save data for an entity.
    #[must_use]
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id: entity_id.raw(),
            ..Default::default()
        }
    }

    /// Gets the entity ID.
    #[must_use]
    pub fn entity(&self) -> EntityId {
        EntityId::from_raw(self.entity_id)
    }

    /// Sets health, clamping to 0..max_health.
    pub fn set_health(&mut self, health: f32) {
        self.health = health.clamp(0.0, self.max_health);
        self.is_dead = self.health <= 0.0;
    }

    /// Sets stamina, clamping to 0..max_stamina.
    pub fn set_stamina(&mut self, stamina: f32) {
        self.stamina = stamina.clamp(0.0, self.max_stamina);
    }

    /// Returns health as percentage (0.0-1.0).
    #[must_use]
    pub fn health_percent(&self) -> f32 {
        if self.max_health <= 0.0 {
            0.0
        } else {
            self.health / self.max_health
        }
    }

    /// Returns stamina as percentage (0.0-1.0).
    #[must_use]
    pub fn stamina_percent(&self) -> f32 {
        if self.max_stamina <= 0.0 {
            0.0
        } else {
            self.stamina / self.max_stamina
        }
    }

    /// Adds a status effect.
    pub fn add_status_effect(&mut self, effect: StatusEffectSaveData) {
        // Check if already has this effect
        for existing in &mut self.status_effects {
            if existing.effect_type == effect.effect_type {
                // Refresh duration and add stacks
                existing.duration = existing.duration.max(effect.duration);
                existing.stacks = existing.stacks.saturating_add(effect.stacks);
                return;
            }
        }
        self.status_effects.push(effect);
    }

    /// Removes a status effect by type.
    pub fn remove_status_effect(&mut self, effect_type: &str) {
        self.status_effects.retain(|e| e.effect_type != effect_type);
    }

    /// Updates status effect durations and removes expired ones.
    pub fn update_status_effects(&mut self, delta_time: f32) -> Vec<StatusEffectSaveData> {
        let mut expired = Vec::new();

        self.status_effects.retain_mut(|effect| {
            effect.duration -= delta_time;
            if effect.duration <= 0.0 {
                expired.push(effect.clone());
                false
            } else {
                true
            }
        });

        expired
    }

    /// Sets a damage resistance.
    pub fn set_resistance(&mut self, damage_type: impl Into<String>, value: f32) {
        self.resistances.insert(damage_type.into(), value);
    }

    /// Gets a damage resistance.
    #[must_use]
    pub fn get_resistance(&self, damage_type: &str) -> f32 {
        self.resistances.get(damage_type).copied().unwrap_or(0.0)
    }

    /// Returns true if entity has a specific status effect.
    #[must_use]
    pub fn has_status_effect(&self, effect_type: &str) -> bool {
        self.status_effects.iter().any(|e| e.effect_type == effect_type)
    }

    /// Returns true if entity can attack (not dead, no cooldown, has stamina).
    #[must_use]
    pub fn can_attack(&self, stamina_cost: f32) -> bool {
        !self.is_dead
            && self.attack_cooldown <= 0.0
            && self.stamina >= stamina_cost
            && !self.has_status_effect("Stunned")
            && !self.has_status_effect("Frozen")
    }
}

/// Saved combat statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CombatStatsSaveData {
    /// Total attacks made.
    pub attacks_made: u64,
    /// Total hits landed.
    pub hits_landed: u64,
    /// Total damage dealt.
    pub damage_dealt: f64,
    /// Total damage taken.
    pub damage_taken: f64,
    /// Total damage blocked.
    pub damage_blocked: f64,
    /// Total kills.
    pub kills: u64,
    /// Total deaths.
    pub deaths: u64,
    /// Critical hits landed.
    pub critical_hits: u64,
    /// Perfect blocks performed.
    pub perfect_blocks: u64,
}

impl From<&CombatStats> for CombatStatsSaveData {
    fn from(stats: &CombatStats) -> Self {
        Self {
            attacks_made: stats.attacks_made,
            hits_landed: stats.hits_landed,
            damage_dealt: stats.damage_dealt,
            damage_taken: stats.damage_taken,
            damage_blocked: stats.damage_blocked,
            kills: stats.kills,
            deaths: stats.deaths,
            critical_hits: stats.critical_hits,
            perfect_blocks: stats.perfect_blocks,
        }
    }
}

impl CombatStatsSaveData {
    /// Restores to a CombatStats instance.
    #[must_use]
    pub fn restore(&self) -> CombatStats {
        CombatStats {
            attacks_made: self.attacks_made,
            hits_landed: self.hits_landed,
            damage_dealt: self.damage_dealt,
            damage_taken: self.damage_taken,
            damage_blocked: self.damage_blocked,
            kills: self.kills,
            deaths: self.deaths,
            critical_hits: self.critical_hits,
            perfect_blocks: self.perfect_blocks,
        }
    }
}

/// Kill record for tracking specific kills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRecord {
    /// Entity type killed.
    pub entity_type: String,
    /// Kill count.
    pub count: u64,
    /// First kill timestamp.
    pub first_kill: u64,
    /// Last kill timestamp.
    pub last_kill: u64,
}

impl KillRecord {
    /// Creates a new kill record.
    #[must_use]
    pub fn new(entity_type: impl Into<String>, timestamp: u64) -> Self {
        let entity_type = entity_type.into();
        Self {
            entity_type,
            count: 1,
            first_kill: timestamp,
            last_kill: timestamp,
        }
    }

    /// Records another kill.
    pub fn record_kill(&mut self, timestamp: u64) {
        self.count += 1;
        self.last_kill = timestamp;
    }
}

/// Complete combat save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSaveData {
    /// Save format version.
    pub version: u32,
    /// Player combat state.
    pub player: EntityCombatSaveData,
    /// Combat statistics.
    pub stats: CombatStatsSaveData,
    /// Kill records by entity type.
    pub kill_records: HashMap<String, KillRecord>,
    /// Unlocked weapon proficiencies.
    pub weapon_proficiencies: HashMap<String, u32>,
    /// Unlocked combat abilities.
    pub unlocked_abilities: Vec<String>,
    /// Combat skill level.
    pub combat_level: u32,
    /// Combat experience.
    pub combat_experience: u64,
    /// Experience needed for next level.
    pub experience_to_next_level: u64,
    /// Active buffs/debuffs from items.
    pub item_effects: Vec<StatusEffectSaveData>,
}

impl Default for CombatSaveData {
    fn default() -> Self {
        Self {
            version: COMBAT_SAVE_VERSION,
            player: EntityCombatSaveData::new(EntityId::from_raw(1)),
            stats: CombatStatsSaveData::default(),
            kill_records: HashMap::new(),
            weapon_proficiencies: HashMap::new(),
            unlocked_abilities: Vec::new(),
            combat_level: 1,
            combat_experience: 0,
            experience_to_next_level: 100,
            item_effects: Vec::new(),
        }
    }
}

impl CombatSaveData {
    /// Creates new combat save data.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a kill.
    pub fn record_kill(&mut self, entity_type: impl Into<String>, timestamp: u64) {
        let entity_type = entity_type.into();
        if let Some(record) = self.kill_records.get_mut(&entity_type) {
            record.record_kill(timestamp);
        } else {
            self.kill_records
                .insert(entity_type.clone(), KillRecord::new(entity_type, timestamp));
        }
    }

    /// Gets total kills for an entity type.
    #[must_use]
    pub fn get_kills(&self, entity_type: &str) -> u64 {
        self.kill_records.get(entity_type).map(|r| r.count).unwrap_or(0)
    }

    /// Adds combat experience and handles leveling.
    pub fn add_experience(&mut self, amount: u64) -> Option<u32> {
        self.combat_experience += amount;

        let mut leveled_up = None;
        while self.combat_experience >= self.experience_to_next_level {
            self.combat_experience -= self.experience_to_next_level;
            self.combat_level += 1;
            // Experience curve: level * 100 * 1.5^(level/5)
            let level_factor = 1.5_f64.powf(f64::from(self.combat_level) / 5.0);
            self.experience_to_next_level = (f64::from(self.combat_level) * 100.0 * level_factor) as u64;
            leveled_up = Some(self.combat_level);
            info!("Combat level up: {}", self.combat_level);
        }

        leveled_up
    }

    /// Gets weapon proficiency level.
    #[must_use]
    pub fn get_proficiency(&self, weapon_type: &str) -> u32 {
        self.weapon_proficiencies.get(weapon_type).copied().unwrap_or(0)
    }

    /// Adds weapon proficiency experience.
    pub fn add_proficiency(&mut self, weapon_type: impl Into<String>, amount: u32) {
        let weapon_type = weapon_type.into();
        *self.weapon_proficiencies.entry(weapon_type).or_insert(0) += amount;
    }

    /// Unlocks a combat ability.
    pub fn unlock_ability(&mut self, ability: impl Into<String>) {
        let ability = ability.into();
        if !self.unlocked_abilities.contains(&ability) {
            self.unlocked_abilities.push(ability);
        }
    }

    /// Checks if an ability is unlocked.
    #[must_use]
    pub fn has_ability(&self, ability: &str) -> bool {
        self.unlocked_abilities.iter().any(|a| a == ability)
    }

    /// Migrates save data from an older version.
    pub fn migrate(self) -> CombatSaveResult<Self> {
        if self.version == COMBAT_SAVE_VERSION {
            return Ok(self);
        }

        info!(
            "Migrating combat save from v{} to v{}",
            self.version, COMBAT_SAVE_VERSION
        );

        // Future migrations would go here

        if self.version != COMBAT_SAVE_VERSION {
            return Err(CombatSaveError::VersionMismatch {
                expected: COMBAT_SAVE_VERSION,
                found: self.version,
            });
        }

        Ok(self)
    }

    /// Serializes to JSON.
    pub fn to_json(&self) -> CombatSaveResult<String> {
        serde_json::to_string(self).map_err(|e| CombatSaveError::Serialization(e.to_string()))
    }

    /// Serializes to JSON (pretty).
    pub fn to_json_pretty(&self) -> CombatSaveResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| CombatSaveError::Serialization(e.to_string()))
    }

    /// Deserializes from JSON.
    pub fn from_json(json: &str) -> CombatSaveResult<Self> {
        let data: Self =
            serde_json::from_str(json).map_err(|e| CombatSaveError::Serialization(e.to_string()))?;
        data.migrate()
    }

    /// Serializes to bytes.
    pub fn to_bytes(&self) -> CombatSaveResult<Vec<u8>> {
        let json = self.to_json()?;
        Ok(json.into_bytes())
    }

    /// Deserializes from bytes.
    pub fn from_bytes(bytes: &[u8]) -> CombatSaveResult<Self> {
        let json =
            std::str::from_utf8(bytes).map_err(|e| CombatSaveError::Corrupted(e.to_string()))?;
        Self::from_json(json)
    }
}

/// Manager for combat persistence.
pub struct CombatPersistence {
    /// Current combat state.
    data: CombatSaveData,
    /// Entity combat states.
    entities: HashMap<u64, EntityCombatSaveData>,
    /// Whether there are unsaved changes.
    dirty: bool,
}

impl Default for CombatPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatPersistence {
    /// Creates a new combat persistence manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: CombatSaveData::new(),
            entities: HashMap::new(),
            dirty: false,
        }
    }

    /// Returns a reference to the combat save data.
    #[must_use]
    pub fn data(&self) -> &CombatSaveData {
        &self.data
    }

    /// Returns a mutable reference to the combat save data.
    pub fn data_mut(&mut self) -> &mut CombatSaveData {
        self.dirty = true;
        &mut self.data
    }

    /// Gets the player combat state.
    #[must_use]
    pub fn player(&self) -> &EntityCombatSaveData {
        &self.data.player
    }

    /// Gets mutable player combat state.
    pub fn player_mut(&mut self) -> &mut EntityCombatSaveData {
        self.dirty = true;
        &mut self.data.player
    }

    /// Gets an entity's combat state.
    #[must_use]
    pub fn get_entity(&self, entity_id: EntityId) -> Option<&EntityCombatSaveData> {
        self.entities.get(&entity_id.raw())
    }

    /// Gets or creates an entity's combat state.
    pub fn get_or_create_entity(&mut self, entity_id: EntityId) -> &mut EntityCombatSaveData {
        self.dirty = true;
        self.entities
            .entry(entity_id.raw())
            .or_insert_with(|| EntityCombatSaveData::new(entity_id))
    }

    /// Removes an entity's combat state (e.g., on death/despawn).
    pub fn remove_entity(&mut self, entity_id: EntityId) -> Option<EntityCombatSaveData> {
        self.dirty = true;
        self.entities.remove(&entity_id.raw())
    }

    /// Updates all entity combat states.
    pub fn update(&mut self, delta_time: f32) {
        // Update player status effects
        let _ = self.data.player.update_status_effects(delta_time);

        // Update player attack cooldown
        if self.data.player.attack_cooldown > 0.0 {
            self.data.player.attack_cooldown = (self.data.player.attack_cooldown - delta_time).max(0.0);
        }

        // Update respawn timer
        if self.data.player.is_dead && self.data.player.respawn_timer > 0.0 {
            self.data.player.respawn_timer -= delta_time;
        }

        // Update all tracked entities
        for entity in self.entities.values_mut() {
            let _ = entity.update_status_effects(delta_time);

            if entity.attack_cooldown > 0.0 {
                entity.attack_cooldown = (entity.attack_cooldown - delta_time).max(0.0);
            }

            if entity.is_dead && entity.respawn_timer > 0.0 {
                entity.respawn_timer -= delta_time;
            }
        }

        self.dirty = true;
    }

    /// Returns true if there are unsaved changes.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the data as saved.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Loads from save data.
    pub fn load(&mut self, data: CombatSaveData) {
        self.data = data;
        self.dirty = false;
        debug!("Loaded combat save data");
    }

    /// Exports save data.
    #[must_use]
    pub fn export(&self) -> CombatSaveData {
        self.data.clone()
    }

    /// Resets to default state.
    pub fn reset(&mut self) {
        self.data = CombatSaveData::new();
        self.entities.clear();
        self.dirty = true;
    }

    /// Records damage dealt.
    pub fn record_damage_dealt(&mut self, damage: f32) {
        self.data.stats.damage_dealt += f64::from(damage);
        self.dirty = true;
    }

    /// Records damage taken.
    pub fn record_damage_taken(&mut self, damage: f32) {
        self.data.stats.damage_taken += f64::from(damage);
        self.dirty = true;
    }

    /// Records an attack.
    pub fn record_attack(&mut self, hit: bool, critical: bool) {
        self.data.stats.attacks_made += 1;
        if hit {
            self.data.stats.hits_landed += 1;
            if critical {
                self.data.stats.critical_hits += 1;
            }
        }
        self.dirty = true;
    }

    /// Records a kill.
    pub fn record_kill(&mut self, entity_type: impl Into<String>, timestamp: u64, experience: u64) {
        self.data.stats.kills += 1;
        self.data.record_kill(entity_type, timestamp);
        self.data.add_experience(experience);
        self.dirty = true;
    }

    /// Records a death.
    pub fn record_death(&mut self, respawn_time: f32) {
        self.data.stats.deaths += 1;
        self.data.player.is_dead = true;
        self.data.player.respawn_timer = respawn_time;
        // Reset combo on death
        self.data.player.combo_count = 0;
        self.dirty = true;
    }

    /// Records a block.
    pub fn record_block(&mut self, damage_blocked: f32, perfect: bool) {
        self.data.stats.damage_blocked += f64::from(damage_blocked);
        if perfect {
            self.data.stats.perfect_blocks += 1;
        }
        self.dirty = true;
    }

    /// Respawns the player.
    pub fn respawn_player(&mut self, health_percent: f32) {
        self.data.player.is_dead = false;
        self.data.player.respawn_timer = 0.0;
        self.data.player.health = self.data.player.max_health * health_percent;
        self.data.player.stamina = self.data.player.max_stamina * 0.5;
        self.data.player.status_effects.clear();
        self.data.player.attack_cooldown = 0.0;
        self.dirty = true;
        info!("Player respawned with {}% health", (health_percent * 100.0) as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_combat_save_data_health() {
        let mut entity = EntityCombatSaveData::new(EntityId::from_raw(1));
        entity.max_health = 100.0;

        entity.set_health(50.0);
        assert!((entity.health - 50.0).abs() < 0.01);
        assert!(!entity.is_dead);

        entity.set_health(150.0);
        assert!((entity.health - 100.0).abs() < 0.01);

        entity.set_health(-10.0);
        assert!((entity.health - 0.0).abs() < 0.01);
        assert!(entity.is_dead);
    }

    #[test]
    fn test_entity_combat_save_data_stamina() {
        let mut entity = EntityCombatSaveData::new(EntityId::from_raw(1));
        entity.max_stamina = 100.0;

        entity.set_stamina(75.0);
        assert!((entity.stamina_percent() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_entity_combat_save_data_status_effects() {
        let mut entity = EntityCombatSaveData::new(EntityId::from_raw(1));

        entity.add_status_effect(StatusEffectSaveData {
            effect_type: "Burning".to_string(),
            duration: 5.0,
            stacks: 1,
            source: None,
        });

        assert!(entity.has_status_effect("Burning"));
        assert!(!entity.has_status_effect("Frozen"));

        // Add same effect - should stack
        entity.add_status_effect(StatusEffectSaveData {
            effect_type: "Burning".to_string(),
            duration: 3.0,
            stacks: 2,
            source: None,
        });

        assert_eq!(entity.status_effects.len(), 1);
        assert_eq!(entity.status_effects[0].stacks, 3);
        assert!((entity.status_effects[0].duration - 5.0).abs() < 0.01); // Max duration kept

        entity.remove_status_effect("Burning");
        assert!(!entity.has_status_effect("Burning"));
    }

    #[test]
    fn test_entity_combat_save_data_update_effects() {
        let mut entity = EntityCombatSaveData::new(EntityId::from_raw(1));

        entity.add_status_effect(StatusEffectSaveData {
            effect_type: "Burning".to_string(),
            duration: 2.0,
            stacks: 1,
            source: None,
        });

        let expired = entity.update_status_effects(1.0);
        assert!(expired.is_empty());
        assert!(entity.has_status_effect("Burning"));

        let expired = entity.update_status_effects(1.5);
        assert_eq!(expired.len(), 1);
        assert!(!entity.has_status_effect("Burning"));
    }

    #[test]
    fn test_entity_combat_save_data_can_attack() {
        let mut entity = EntityCombatSaveData::new(EntityId::from_raw(1));
        entity.stamina = 50.0;

        assert!(entity.can_attack(10.0));

        entity.attack_cooldown = 0.5;
        assert!(!entity.can_attack(10.0));

        entity.attack_cooldown = 0.0;
        entity.stamina = 5.0;
        assert!(!entity.can_attack(10.0));

        entity.stamina = 50.0;
        entity.is_dead = true;
        assert!(!entity.can_attack(10.0));
    }

    #[test]
    fn test_combat_save_data_experience() {
        let mut save = CombatSaveData::new();
        save.experience_to_next_level = 100;

        let level_up = save.add_experience(50);
        assert!(level_up.is_none());
        assert_eq!(save.combat_level, 1);

        let level_up = save.add_experience(60);
        assert!(level_up.is_some());
        assert_eq!(save.combat_level, 2);
    }

    #[test]
    fn test_combat_save_data_kills() {
        let mut save = CombatSaveData::new();

        save.record_kill("goblin", 1000);
        save.record_kill("goblin", 2000);
        save.record_kill("orc", 3000);

        assert_eq!(save.get_kills("goblin"), 2);
        assert_eq!(save.get_kills("orc"), 1);
        assert_eq!(save.get_kills("troll"), 0);
    }

    #[test]
    fn test_combat_save_data_proficiency() {
        let mut save = CombatSaveData::new();

        save.add_proficiency("sword", 10);
        save.add_proficiency("sword", 5);
        save.add_proficiency("bow", 8);

        assert_eq!(save.get_proficiency("sword"), 15);
        assert_eq!(save.get_proficiency("bow"), 8);
        assert_eq!(save.get_proficiency("axe"), 0);
    }

    #[test]
    fn test_combat_save_data_abilities() {
        let mut save = CombatSaveData::new();

        save.unlock_ability("power_strike");
        save.unlock_ability("block_counter");
        save.unlock_ability("power_strike"); // Duplicate

        assert!(save.has_ability("power_strike"));
        assert!(save.has_ability("block_counter"));
        assert!(!save.has_ability("spin_attack"));
        assert_eq!(save.unlocked_abilities.len(), 2);
    }

    #[test]
    fn test_combat_save_data_serialization() {
        let mut save = CombatSaveData::new();
        save.player.set_health(75.0);
        save.combat_level = 5;

        let json = save.to_json().unwrap();
        let loaded = CombatSaveData::from_json(&json).unwrap();

        assert!((loaded.player.health - 75.0).abs() < 0.01);
        assert_eq!(loaded.combat_level, 5);
    }

    #[test]
    fn test_combat_persistence_update() {
        let mut persistence = CombatPersistence::new();
        persistence.data.player.attack_cooldown = 1.0;
        persistence.data.player.add_status_effect(StatusEffectSaveData {
            effect_type: "Burning".to_string(),
            duration: 0.5,
            stacks: 1,
            source: None,
        });

        persistence.update(0.3);
        assert!((persistence.data.player.attack_cooldown - 0.7).abs() < 0.01);
        assert!(persistence.data.player.has_status_effect("Burning"));

        persistence.update(0.3);
        assert!(!persistence.data.player.has_status_effect("Burning"));
    }

    #[test]
    fn test_combat_persistence_respawn() {
        let mut persistence = CombatPersistence::new();
        persistence.record_death(10.0);

        assert!(persistence.data.player.is_dead);
        assert!((persistence.data.player.respawn_timer - 10.0).abs() < 0.01);

        persistence.respawn_player(0.5);

        assert!(!persistence.data.player.is_dead);
        assert!((persistence.data.player.health - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_status_effect_save_data_conversion() {
        let save = StatusEffectSaveData {
            effect_type: "Burning".to_string(),
            duration: 5.0,
            stacks: 2,
            source: Some(42),
        };

        assert_eq!(save.to_effect(), Some(StatusEffect::Burning));

        let unknown = StatusEffectSaveData {
            effect_type: "Unknown".to_string(),
            duration: 1.0,
            stacks: 1,
            source: None,
        };
        assert_eq!(unknown.to_effect(), None);
    }
}
