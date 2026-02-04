//! Combat stats system.
//!
//! This module provides:
//! - Core combat statistics (hp, attack, defense, etc.)
//! - Stat modifiers from equipment
//! - Temporary buffs and debuffs
//! - Stamina for attacks

use genesis_common::EntityId;
use serde::{Deserialize, Serialize};

// ============================================================================
// G-49: Base Stats
// ============================================================================

/// Core combat statistics for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseCombatStats {
    /// Current health points.
    pub hp: f32,
    /// Maximum health points.
    pub max_hp: f32,
    /// Base attack power.
    pub attack: f32,
    /// Base defense value.
    pub defense: f32,
    /// Critical hit chance (0.0-1.0).
    pub crit_chance: f32,
    /// Critical hit damage multiplier.
    pub crit_multiplier: f32,
    /// Dodge chance (0.0-1.0).
    pub dodge: f32,
    /// Current stamina.
    pub stamina: f32,
    /// Maximum stamina.
    pub max_stamina: f32,
    /// Stamina regeneration per second.
    pub stamina_regen: f32,
    /// Attack speed multiplier.
    pub attack_speed: f32,
    /// Movement speed multiplier.
    pub move_speed: f32,
}

impl Default for BaseCombatStats {
    fn default() -> Self {
        Self {
            hp: 100.0,
            max_hp: 100.0,
            attack: 10.0,
            defense: 5.0,
            crit_chance: 0.05,
            crit_multiplier: 2.0,
            dodge: 0.0,
            stamina: 100.0,
            max_stamina: 100.0,
            stamina_regen: 10.0,
            attack_speed: 1.0,
            move_speed: 1.0,
        }
    }
}

impl BaseCombatStats {
    /// Create new stats with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create stats with specified HP.
    #[must_use]
    pub fn with_hp(mut self, hp: f32) -> Self {
        self.hp = hp;
        self.max_hp = hp;
        self
    }

    /// Set attack power.
    #[must_use]
    pub fn with_attack(mut self, attack: f32) -> Self {
        self.attack = attack;
        self
    }

    /// Set defense value.
    #[must_use]
    pub fn with_defense(mut self, defense: f32) -> Self {
        self.defense = defense;
        self
    }

    /// Set crit chance.
    #[must_use]
    pub fn with_crit_chance(mut self, chance: f32) -> Self {
        self.crit_chance = chance.clamp(0.0, 1.0);
        self
    }

    /// Set dodge chance.
    #[must_use]
    pub fn with_dodge(mut self, dodge: f32) -> Self {
        self.dodge = dodge.clamp(0.0, 0.75); // Cap at 75%
        self
    }

    /// Set stamina.
    #[must_use]
    pub fn with_stamina(mut self, stamina: f32) -> Self {
        self.stamina = stamina;
        self.max_stamina = stamina;
        self
    }

    /// Check if dead.
    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Check if alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
    }

    /// Get HP percentage (0.0-1.0).
    #[must_use]
    pub fn hp_percent(&self) -> f32 {
        if self.max_hp <= 0.0 {
            0.0
        } else {
            (self.hp / self.max_hp).clamp(0.0, 1.0)
        }
    }

    /// Get stamina percentage (0.0-1.0).
    #[must_use]
    pub fn stamina_percent(&self) -> f32 {
        if self.max_stamina <= 0.0 {
            0.0
        } else {
            (self.stamina / self.max_stamina).clamp(0.0, 1.0)
        }
    }

    /// Heal HP.
    pub fn heal(&mut self, amount: f32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Take damage (returns actual damage taken).
    pub fn take_damage(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.hp);
        self.hp = (self.hp - actual).max(0.0);
        actual
    }

    /// Consume stamina (returns true if successful).
    pub fn consume_stamina(&mut self, amount: f32) -> bool {
        if self.stamina >= amount {
            self.stamina -= amount;
            true
        } else {
            false
        }
    }

    /// Try to consume stamina, allowing partial consumption.
    pub fn try_consume_stamina(&mut self, amount: f32) -> f32 {
        let consumed = amount.min(self.stamina);
        self.stamina -= consumed;
        consumed
    }

    /// Regenerate stamina over time.
    pub fn regen_stamina(&mut self, dt: f32) {
        self.stamina = (self.stamina + self.stamina_regen * dt).min(self.max_stamina);
    }

    /// Check if has enough stamina.
    #[must_use]
    pub fn has_stamina(&self, amount: f32) -> bool {
        self.stamina >= amount
    }
}

// ============================================================================
// G-49: Stat Modifiers
// ============================================================================

/// Source of a stat modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierSource {
    /// From equipped weapon.
    Weapon,
    /// From equipped armor.
    Armor,
    /// From accessory/trinket.
    Accessory,
    /// From consumable buff.
    Consumable,
    /// From skill/ability.
    Skill,
    /// From environmental effect.
    Environment,
    /// From status effect.
    Status,
    /// From passive ability.
    Passive,
}

/// Type of stat being modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatType {
    /// Maximum HP.
    MaxHp,
    /// Attack power.
    Attack,
    /// Defense value.
    Defense,
    /// Critical hit chance.
    CritChance,
    /// Critical hit multiplier.
    CritMultiplier,
    /// Dodge chance.
    Dodge,
    /// Maximum stamina.
    MaxStamina,
    /// Stamina regeneration.
    StaminaRegen,
    /// Attack speed.
    AttackSpeed,
    /// Movement speed.
    MoveSpeed,
}

/// How a modifier is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierType {
    /// Add flat value.
    Flat,
    /// Multiply by percentage (1.0 = 100%).
    Percent,
    /// Add after percentage modifiers.
    FlatFinal,
}

/// A stat modifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatModifier {
    /// Unique identifier for this modifier.
    pub id: u32,
    /// Source of the modifier.
    pub source: ModifierSource,
    /// Stat being modified.
    pub stat: StatType,
    /// Type of modification.
    pub modifier_type: ModifierType,
    /// Value of the modifier.
    pub value: f32,
    /// Optional duration (None = permanent).
    pub duration: Option<f32>,
    /// Whether currently active.
    pub active: bool,
}

impl StatModifier {
    /// Create a new flat modifier.
    #[must_use]
    pub fn flat(id: u32, source: ModifierSource, stat: StatType, value: f32) -> Self {
        Self {
            id,
            source,
            stat,
            modifier_type: ModifierType::Flat,
            value,
            duration: None,
            active: true,
        }
    }

    /// Create a new percentage modifier.
    #[must_use]
    pub fn percent(id: u32, source: ModifierSource, stat: StatType, value: f32) -> Self {
        Self {
            id,
            source,
            stat,
            modifier_type: ModifierType::Percent,
            value,
            duration: None,
            active: true,
        }
    }

    /// Set duration.
    #[must_use]
    pub fn with_duration(mut self, seconds: f32) -> Self {
        self.duration = Some(seconds);
        self
    }

    /// Check if expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        matches!(self.duration, Some(d) if d <= 0.0)
    }

    /// Update duration.
    pub fn tick(&mut self, dt: f32) {
        if let Some(ref mut dur) = self.duration {
            *dur -= dt;
        }
    }
}

// ============================================================================
// G-49: Buffs and Debuffs
// ============================================================================

/// Type of buff/debuff effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuffType {
    // Buffs
    /// Increased attack.
    AttackUp,
    /// Increased defense.
    DefenseUp,
    /// Increased speed.
    SpeedUp,
    /// Increased crit chance.
    CritUp,
    /// Health regeneration.
    Regeneration,
    /// Stamina regeneration.
    Endurance,
    /// Damage shield.
    Shield,
    /// Invulnerability.
    Invincible,

    // Debuffs
    /// Decreased attack.
    AttackDown,
    /// Decreased defense.
    DefenseDown,
    /// Decreased speed.
    Slowed,
    /// Cannot move.
    Rooted,
    /// Cannot attack.
    Disarmed,
    /// Cannot use abilities.
    Silenced,
    /// Stunned (cannot act).
    Stunned,
    /// Taking damage over time.
    Burning,
    /// Taking damage over time (cold).
    Frozen,
    /// Taking damage over time (poison).
    Poisoned,
    /// Taking damage over time (bleed).
    Bleeding,
}

impl BuffType {
    /// Check if this is a debuff.
    #[must_use]
    pub fn is_debuff(&self) -> bool {
        matches!(
            self,
            Self::AttackDown
                | Self::DefenseDown
                | Self::Slowed
                | Self::Rooted
                | Self::Disarmed
                | Self::Silenced
                | Self::Stunned
                | Self::Burning
                | Self::Frozen
                | Self::Poisoned
                | Self::Bleeding
        )
    }

    /// Check if this is a buff.
    #[must_use]
    pub fn is_buff(&self) -> bool {
        !self.is_debuff()
    }

    /// Check if this is a crowd control effect.
    #[must_use]
    pub fn is_cc(&self) -> bool {
        matches!(
            self,
            Self::Slowed | Self::Rooted | Self::Disarmed | Self::Silenced | Self::Stunned
        )
    }

    /// Check if this is a damage over time effect.
    #[must_use]
    pub fn is_dot(&self) -> bool {
        matches!(
            self,
            Self::Burning | Self::Frozen | Self::Poisoned | Self::Bleeding
        )
    }
}

/// An active buff or debuff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveBuff {
    /// Buff type.
    pub buff_type: BuffType,
    /// Source entity (if any).
    pub source: Option<EntityId>,
    /// Remaining duration.
    pub duration: f32,
    /// Effect strength/stacks.
    pub stacks: u32,
    /// Value per stack (for DoT, stat changes, etc.).
    pub value: f32,
    /// Time since last tick (for DoT).
    pub tick_timer: f32,
    /// Tick interval (for DoT).
    pub tick_interval: f32,
}

impl ActiveBuff {
    /// Create a new buff.
    #[must_use]
    pub fn new(buff_type: BuffType, duration: f32) -> Self {
        Self {
            buff_type,
            source: None,
            duration,
            stacks: 1,
            value: 1.0,
            tick_timer: 0.0,
            tick_interval: 1.0,
        }
    }

    /// Set source entity.
    #[must_use]
    pub fn with_source(mut self, source: EntityId) -> Self {
        self.source = Some(source);
        self
    }

    /// Set value.
    #[must_use]
    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Set stacks.
    #[must_use]
    pub fn with_stacks(mut self, stacks: u32) -> Self {
        self.stacks = stacks;
        self
    }

    /// Set tick interval for DoT effects.
    #[must_use]
    pub fn with_tick_interval(mut self, interval: f32) -> Self {
        self.tick_interval = interval;
        self
    }

    /// Add stacks (up to max).
    pub fn add_stacks(&mut self, amount: u32, max: u32) {
        self.stacks = (self.stacks + amount).min(max);
    }

    /// Refresh duration.
    pub fn refresh(&mut self, duration: f32) {
        self.duration = self.duration.max(duration);
    }

    /// Check if expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.duration <= 0.0
    }

    /// Update timer, returns DoT damage ticks.
    pub fn tick(&mut self, dt: f32) -> u32 {
        self.duration -= dt;

        if self.buff_type.is_dot() {
            self.tick_timer += dt;
            let ticks = (self.tick_timer / self.tick_interval) as u32;
            self.tick_timer %= self.tick_interval;
            ticks
        } else {
            0
        }
    }

    /// Get total DoT damage per tick.
    #[must_use]
    pub fn dot_damage(&self) -> f32 {
        self.value * self.stacks as f32
    }
}

// ============================================================================
// G-49: Combat Stats Manager
// ============================================================================

/// Complete combat stats with modifiers and buffs.
#[derive(Debug, Clone, Default)]
pub struct CombatStatsManager {
    /// Base stats.
    pub base: BaseCombatStats,
    /// Active stat modifiers.
    modifiers: Vec<StatModifier>,
    /// Active buffs/debuffs.
    buffs: Vec<ActiveBuff>,
    /// Modifier ID counter.
    next_modifier_id: u32,
    /// Cached calculated stats (invalidated on changes).
    cached_stats: Option<CalculatedStats>,
}

/// Calculated stats after applying modifiers.
#[derive(Debug, Clone, PartialEq)]
pub struct CalculatedStats {
    /// Calculated max HP.
    pub max_hp: f32,
    /// Calculated attack.
    pub attack: f32,
    /// Calculated defense.
    pub defense: f32,
    /// Calculated crit chance.
    pub crit_chance: f32,
    /// Calculated crit multiplier.
    pub crit_multiplier: f32,
    /// Calculated dodge.
    pub dodge: f32,
    /// Calculated max stamina.
    pub max_stamina: f32,
    /// Calculated stamina regen.
    pub stamina_regen: f32,
    /// Calculated attack speed.
    pub attack_speed: f32,
    /// Calculated move speed.
    pub move_speed: f32,
}

impl CombatStatsManager {
    /// Create new stats manager.
    #[must_use]
    pub fn new(base: BaseCombatStats) -> Self {
        Self {
            base,
            modifiers: Vec::new(),
            buffs: Vec::new(),
            next_modifier_id: 1,
            cached_stats: None,
        }
    }

    /// Add a stat modifier.
    pub fn add_modifier(&mut self, mut modifier: StatModifier) -> u32 {
        modifier.id = self.next_modifier_id;
        self.next_modifier_id += 1;
        let id = modifier.id;
        self.modifiers.push(modifier);
        self.invalidate_cache();
        id
    }

    /// Remove a modifier by ID.
    pub fn remove_modifier(&mut self, id: u32) -> bool {
        let len = self.modifiers.len();
        self.modifiers.retain(|m| m.id != id);
        if self.modifiers.len() == len {
            false
        } else {
            self.invalidate_cache();
            true
        }
    }

    /// Remove all modifiers from a source.
    pub fn remove_modifiers_from(&mut self, source: ModifierSource) {
        let len = self.modifiers.len();
        self.modifiers.retain(|m| m.source != source);
        if self.modifiers.len() != len {
            self.invalidate_cache();
        }
    }

    /// Add a buff.
    pub fn add_buff(&mut self, buff: ActiveBuff) {
        // Check for existing buff of same type
        if let Some(existing) = self
            .buffs
            .iter_mut()
            .find(|b| b.buff_type == buff.buff_type)
        {
            // Refresh duration and add stacks
            existing.refresh(buff.duration);
            existing.add_stacks(buff.stacks, 10);
        } else {
            self.buffs.push(buff);
        }
        self.invalidate_cache();
    }

    /// Remove a buff by type.
    pub fn remove_buff(&mut self, buff_type: BuffType) -> bool {
        let len = self.buffs.len();
        self.buffs.retain(|b| b.buff_type != buff_type);
        if self.buffs.len() == len {
            false
        } else {
            self.invalidate_cache();
            true
        }
    }

    /// Remove all debuffs.
    pub fn cleanse_debuffs(&mut self) {
        let len = self.buffs.len();
        self.buffs.retain(|b| !b.buff_type.is_debuff());
        if self.buffs.len() != len {
            self.invalidate_cache();
        }
    }

    /// Remove all buffs.
    pub fn purge_buffs(&mut self) {
        let len = self.buffs.len();
        self.buffs.retain(|b| b.buff_type.is_debuff());
        if self.buffs.len() != len {
            self.invalidate_cache();
        }
    }

    /// Check if has buff.
    #[must_use]
    pub fn has_buff(&self, buff_type: BuffType) -> bool {
        self.buffs.iter().any(|b| b.buff_type == buff_type)
    }

    /// Get buff stacks.
    #[must_use]
    pub fn buff_stacks(&self, buff_type: BuffType) -> u32 {
        self.buffs
            .iter()
            .find(|b| b.buff_type == buff_type)
            .map_or(0, |b| b.stacks)
    }

    /// Check if can act (not stunned).
    #[must_use]
    pub fn can_act(&self) -> bool {
        !self.has_buff(BuffType::Stunned)
    }

    /// Check if can move.
    #[must_use]
    pub fn can_move(&self) -> bool {
        !self.has_buff(BuffType::Stunned) && !self.has_buff(BuffType::Rooted)
    }

    /// Check if can attack.
    #[must_use]
    pub fn can_attack(&self) -> bool {
        !self.has_buff(BuffType::Stunned) && !self.has_buff(BuffType::Disarmed)
    }

    /// Check if invincible.
    #[must_use]
    pub fn is_invincible(&self) -> bool {
        self.has_buff(BuffType::Invincible)
    }

    /// Get shield amount.
    #[must_use]
    pub fn shield_amount(&self) -> f32 {
        self.buffs
            .iter()
            .filter(|b| b.buff_type == BuffType::Shield)
            .map(|b| b.value * b.stacks as f32)
            .sum()
    }

    /// Damage shield (returns remaining damage).
    pub fn damage_shield(&mut self, damage: f32) -> f32 {
        let mut remaining = damage;

        for buff in &mut self.buffs {
            if buff.buff_type == BuffType::Shield && remaining > 0.0 {
                let shield = buff.value * buff.stacks as f32;
                if shield >= remaining {
                    // Shield absorbs all damage
                    buff.value -= remaining / buff.stacks as f32;
                    remaining = 0.0;
                } else {
                    // Shield breaks
                    remaining -= shield;
                    buff.stacks = 0;
                }
            }
        }

        // Remove broken shields
        self.buffs
            .retain(|b| b.buff_type != BuffType::Shield || b.stacks > 0);
        remaining
    }

    /// Update timers, returns DoT damage events.
    pub fn tick(&mut self, dt: f32) -> Vec<(BuffType, f32)> {
        let mut dot_damage = Vec::new();

        // Update modifiers
        for modifier in &mut self.modifiers {
            modifier.tick(dt);
        }
        let had_modifiers = !self.modifiers.is_empty();
        self.modifiers.retain(|m| !m.is_expired());
        if had_modifiers && self.modifiers.is_empty() {
            self.invalidate_cache();
        }

        // Update buffs
        for buff in &mut self.buffs {
            let ticks = buff.tick(dt);
            if ticks > 0 && buff.buff_type.is_dot() {
                dot_damage.push((buff.buff_type, buff.dot_damage() * ticks as f32));
            }
        }
        let had_buffs = !self.buffs.is_empty();
        self.buffs.retain(|b| !b.is_expired());
        if had_buffs && self.buffs.is_empty() {
            self.invalidate_cache();
        }

        // Regenerate stamina
        let regen = self.get_stats().stamina_regen;
        self.base
            .regen_stamina(dt * regen / self.base.stamina_regen);

        // Apply regeneration buff
        if self.has_buff(BuffType::Regeneration) {
            let regen_value: f32 = self
                .buffs
                .iter()
                .filter(|b| b.buff_type == BuffType::Regeneration)
                .map(|b| b.value * b.stacks as f32)
                .sum();
            self.base.heal(regen_value * dt);
        }

        dot_damage
    }

    /// Invalidate cached stats.
    fn invalidate_cache(&mut self) {
        self.cached_stats = None;
    }

    /// Calculate stats from base + modifiers.
    fn calculate_stats(&self) -> CalculatedStats {
        let mut stats = CalculatedStats {
            max_hp: self.base.max_hp,
            attack: self.base.attack,
            defense: self.base.defense,
            crit_chance: self.base.crit_chance,
            crit_multiplier: self.base.crit_multiplier,
            dodge: self.base.dodge,
            max_stamina: self.base.max_stamina,
            stamina_regen: self.base.stamina_regen,
            attack_speed: self.base.attack_speed,
            move_speed: self.base.move_speed,
        };

        // Apply flat modifiers first
        for modifier in &self.modifiers {
            if modifier.active && modifier.modifier_type == ModifierType::Flat {
                Self::apply_modifier(&mut stats, modifier);
            }
        }

        // Apply percent modifiers
        for modifier in &self.modifiers {
            if modifier.active && modifier.modifier_type == ModifierType::Percent {
                Self::apply_modifier(&mut stats, modifier);
            }
        }

        // Apply final flat modifiers
        for modifier in &self.modifiers {
            if modifier.active && modifier.modifier_type == ModifierType::FlatFinal {
                Self::apply_modifier(&mut stats, modifier);
            }
        }

        // Apply buff stat changes
        for buff in &self.buffs {
            let multiplier = buff.stacks as f32;
            match buff.buff_type {
                BuffType::AttackUp => stats.attack *= 1.0 + 0.2 * multiplier,
                BuffType::AttackDown => stats.attack *= 1.0 - 0.15 * multiplier,
                BuffType::DefenseUp => stats.defense *= 1.0 + 0.2 * multiplier,
                BuffType::DefenseDown => stats.defense *= 1.0 - 0.15 * multiplier,
                BuffType::SpeedUp => {
                    stats.attack_speed *= 1.0 + 0.15 * multiplier;
                    stats.move_speed *= 1.0 + 0.2 * multiplier;
                },
                BuffType::Slowed => {
                    stats.move_speed *= 1.0 - 0.3 * multiplier;
                },
                BuffType::CritUp => stats.crit_chance += 0.1 * multiplier,
                BuffType::Endurance => stats.stamina_regen *= 1.0 + 0.5 * multiplier,
                BuffType::Frozen => {
                    stats.attack_speed *= 0.5;
                    stats.move_speed *= 0.5;
                },
                _ => {},
            }
        }

        // Clamp values
        stats.max_hp = stats.max_hp.max(1.0);
        stats.attack = stats.attack.max(0.0);
        stats.defense = stats.defense.max(0.0);
        stats.crit_chance = stats.crit_chance.clamp(0.0, 1.0);
        stats.crit_multiplier = stats.crit_multiplier.max(1.0);
        stats.dodge = stats.dodge.clamp(0.0, 0.75);
        stats.max_stamina = stats.max_stamina.max(0.0);
        stats.stamina_regen = stats.stamina_regen.max(0.0);
        stats.attack_speed = stats.attack_speed.max(0.1);
        stats.move_speed = stats.move_speed.max(0.1);

        stats
    }

    /// Apply a single modifier to stats.
    fn apply_modifier(stats: &mut CalculatedStats, modifier: &StatModifier) {
        let value = modifier.value;
        match (modifier.stat, modifier.modifier_type) {
            (StatType::MaxHp, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.max_hp += value;
            },
            (StatType::MaxHp, ModifierType::Percent) => stats.max_hp *= value,
            (StatType::Attack, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.attack += value;
            },
            (StatType::Attack, ModifierType::Percent) => stats.attack *= value,
            (StatType::Defense, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.defense += value;
            },
            (StatType::Defense, ModifierType::Percent) => stats.defense *= value,
            (StatType::CritChance, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.crit_chance += value;
            },
            (StatType::CritChance, ModifierType::Percent) => stats.crit_chance *= value,
            (StatType::CritMultiplier, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.crit_multiplier += value;
            },
            (StatType::CritMultiplier, ModifierType::Percent) => stats.crit_multiplier *= value,
            (StatType::Dodge, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.dodge += value;
            },
            (StatType::Dodge, ModifierType::Percent) => stats.dodge *= value,
            (StatType::MaxStamina, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.max_stamina += value;
            },
            (StatType::MaxStamina, ModifierType::Percent) => stats.max_stamina *= value,
            (StatType::StaminaRegen, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.stamina_regen += value;
            },
            (StatType::StaminaRegen, ModifierType::Percent) => stats.stamina_regen *= value,
            (StatType::AttackSpeed, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.attack_speed += value;
            },
            (StatType::AttackSpeed, ModifierType::Percent) => stats.attack_speed *= value,
            (StatType::MoveSpeed, ModifierType::Flat | ModifierType::FlatFinal) => {
                stats.move_speed += value;
            },
            (StatType::MoveSpeed, ModifierType::Percent) => stats.move_speed *= value,
        }
    }

    /// Get calculated stats.
    #[must_use]
    pub fn get_stats(&self) -> CalculatedStats {
        self.cached_stats
            .clone()
            .unwrap_or_else(|| self.calculate_stats())
    }

    /// Refresh cache.
    pub fn refresh_cache(&mut self) {
        self.cached_stats = Some(self.calculate_stats());
    }

    /// Get active buffs.
    pub fn active_buffs(&self) -> impl Iterator<Item = &ActiveBuff> {
        self.buffs.iter()
    }

    /// Get active modifiers.
    pub fn active_modifiers(&self) -> impl Iterator<Item = &StatModifier> {
        self.modifiers.iter()
    }
}

// ============================================================================
// G-49: Equipment Stat Bonuses
// ============================================================================

/// Stats provided by a piece of equipment.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EquipmentStats {
    /// Flat attack bonus.
    pub attack: f32,
    /// Flat defense bonus.
    pub defense: f32,
    /// Flat max HP bonus.
    pub max_hp: f32,
    /// Flat max stamina bonus.
    pub max_stamina: f32,
    /// Crit chance bonus.
    pub crit_chance: f32,
    /// Dodge bonus.
    pub dodge: f32,
    /// Attack speed multiplier (1.0 = no change).
    pub attack_speed: f32,
    /// Move speed multiplier (1.0 = no change).
    pub move_speed: f32,
}

impl EquipmentStats {
    /// Create empty equipment stats.
    #[must_use]
    pub fn new() -> Self {
        Self {
            attack_speed: 1.0,
            move_speed: 1.0,
            ..Default::default()
        }
    }

    /// Convert to modifiers.
    #[must_use]
    pub fn to_modifiers(&self, source: ModifierSource) -> Vec<StatModifier> {
        let mut mods = Vec::new();
        let id = 0; // Will be assigned by manager

        if self.attack != 0.0 {
            mods.push(StatModifier::flat(
                id,
                source,
                StatType::Attack,
                self.attack,
            ));
        }
        if self.defense != 0.0 {
            mods.push(StatModifier::flat(
                id,
                source,
                StatType::Defense,
                self.defense,
            ));
        }
        if self.max_hp != 0.0 {
            mods.push(StatModifier::flat(id, source, StatType::MaxHp, self.max_hp));
        }
        if self.max_stamina != 0.0 {
            mods.push(StatModifier::flat(
                id,
                source,
                StatType::MaxStamina,
                self.max_stamina,
            ));
        }
        if self.crit_chance != 0.0 {
            mods.push(StatModifier::flat(
                id,
                source,
                StatType::CritChance,
                self.crit_chance,
            ));
        }
        if self.dodge != 0.0 {
            mods.push(StatModifier::flat(id, source, StatType::Dodge, self.dodge));
        }
        if (self.attack_speed - 1.0).abs() > 0.001 {
            mods.push(StatModifier::percent(
                id,
                source,
                StatType::AttackSpeed,
                self.attack_speed,
            ));
        }
        if (self.move_speed - 1.0).abs() > 0.001 {
            mods.push(StatModifier::percent(
                id,
                source,
                StatType::MoveSpeed,
                self.move_speed,
            ));
        }

        mods
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_stats_default() {
        let stats = BaseCombatStats::new();
        assert_eq!(stats.hp, 100.0);
        assert_eq!(stats.max_hp, 100.0);
        assert!(!stats.is_dead());
    }

    #[test]
    fn test_base_stats_damage() {
        let mut stats = BaseCombatStats::new().with_hp(50.0);
        let taken = stats.take_damage(30.0);
        assert_eq!(taken, 30.0);
        assert_eq!(stats.hp, 20.0);

        let taken = stats.take_damage(100.0);
        assert_eq!(taken, 20.0);
        assert!(stats.is_dead());
    }

    #[test]
    fn test_stamina_consumption() {
        let mut stats = BaseCombatStats::new().with_stamina(50.0);

        assert!(stats.consume_stamina(30.0));
        assert_eq!(stats.stamina, 20.0);

        assert!(!stats.consume_stamina(30.0));
        assert_eq!(stats.stamina, 20.0);
    }

    #[test]
    fn test_stamina_regen() {
        let mut stats = BaseCombatStats::new().with_stamina(100.0);
        stats.stamina = 50.0;

        stats.regen_stamina(1.0);
        assert_eq!(stats.stamina, 60.0);
    }

    #[test]
    fn test_stat_modifier() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new().with_attack(10.0));

        // Add flat attack bonus
        manager.add_modifier(StatModifier::flat(
            0,
            ModifierSource::Weapon,
            StatType::Attack,
            5.0,
        ));

        let stats = manager.get_stats();
        assert_eq!(stats.attack, 15.0);
    }

    #[test]
    fn test_percent_modifier() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new().with_attack(100.0));

        // Add 50% attack bonus
        manager.add_modifier(StatModifier::percent(
            0,
            ModifierSource::Skill,
            StatType::Attack,
            1.5,
        ));

        let stats = manager.get_stats();
        assert_eq!(stats.attack, 150.0);
    }

    #[test]
    fn test_modifier_stacking() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new().with_attack(100.0));

        // Flat + percent stacking
        manager.add_modifier(StatModifier::flat(
            0,
            ModifierSource::Weapon,
            StatType::Attack,
            20.0,
        ));
        manager.add_modifier(StatModifier::percent(
            0,
            ModifierSource::Skill,
            StatType::Attack,
            1.5,
        ));

        // 100 + 20 = 120, then * 1.5 = 180
        let stats = manager.get_stats();
        assert_eq!(stats.attack, 180.0);
    }

    #[test]
    fn test_buff_application() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new().with_attack(100.0));

        manager.add_buff(ActiveBuff::new(BuffType::AttackUp, 10.0));

        let stats = manager.get_stats();
        assert!((stats.attack - 120.0).abs() < 0.001); // 20% buff
    }

    #[test]
    fn test_buff_stacking() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new().with_attack(100.0));

        manager.add_buff(ActiveBuff::new(BuffType::AttackUp, 10.0));
        manager.add_buff(ActiveBuff::new(BuffType::AttackUp, 10.0));

        assert_eq!(manager.buff_stacks(BuffType::AttackUp), 2);

        let stats = manager.get_stats();
        assert_eq!(stats.attack, 140.0); // 40% buff (2 stacks)
    }

    #[test]
    fn test_debuff_detection() {
        assert!(BuffType::Stunned.is_debuff());
        assert!(BuffType::Poisoned.is_debuff());
        assert!(!BuffType::AttackUp.is_debuff());
    }

    #[test]
    fn test_cc_detection() {
        assert!(BuffType::Stunned.is_cc());
        assert!(BuffType::Rooted.is_cc());
        assert!(!BuffType::AttackUp.is_cc());
        assert!(!BuffType::Burning.is_cc());
    }

    #[test]
    fn test_dot_detection() {
        assert!(BuffType::Burning.is_dot());
        assert!(BuffType::Poisoned.is_dot());
        assert!(!BuffType::Stunned.is_dot());
    }

    #[test]
    fn test_can_act_when_stunned() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new());

        assert!(manager.can_act());
        assert!(manager.can_move());
        assert!(manager.can_attack());

        manager.add_buff(ActiveBuff::new(BuffType::Stunned, 5.0));

        assert!(!manager.can_act());
        assert!(!manager.can_move());
        assert!(!manager.can_attack());
    }

    #[test]
    fn test_shield_damage() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new());

        manager.add_buff(ActiveBuff::new(BuffType::Shield, 10.0).with_value(50.0));

        assert_eq!(manager.shield_amount(), 50.0);

        let remaining = manager.damage_shield(30.0);
        assert_eq!(remaining, 0.0);
        assert_eq!(manager.shield_amount(), 20.0);

        let remaining = manager.damage_shield(50.0);
        assert_eq!(remaining, 30.0);
        assert_eq!(manager.shield_amount(), 0.0);
    }

    #[test]
    fn test_dot_tick() {
        let mut buff = ActiveBuff::new(BuffType::Burning, 5.0)
            .with_value(10.0)
            .with_tick_interval(1.0);

        let ticks = buff.tick(2.5);
        assert_eq!(ticks, 2);
        assert_eq!(buff.dot_damage(), 10.0);
    }

    #[test]
    fn test_cleanse_debuffs() {
        let mut manager = CombatStatsManager::new(BaseCombatStats::new());

        manager.add_buff(ActiveBuff::new(BuffType::AttackUp, 10.0));
        manager.add_buff(ActiveBuff::new(BuffType::Poisoned, 10.0));
        manager.add_buff(ActiveBuff::new(BuffType::Stunned, 5.0));

        manager.cleanse_debuffs();

        assert!(manager.has_buff(BuffType::AttackUp));
        assert!(!manager.has_buff(BuffType::Poisoned));
        assert!(!manager.has_buff(BuffType::Stunned));
    }

    #[test]
    fn test_equipment_stats_to_modifiers() {
        let equip = EquipmentStats {
            attack: 10.0,
            defense: 5.0,
            ..EquipmentStats::new()
        };

        let mods = equip.to_modifiers(ModifierSource::Weapon);
        assert_eq!(mods.len(), 2);
    }

    #[test]
    fn test_timed_modifier_expiry() {
        let mut modifier =
            StatModifier::flat(1, ModifierSource::Consumable, StatType::Attack, 10.0)
                .with_duration(5.0);

        assert!(!modifier.is_expired());

        modifier.tick(3.0);
        assert!(!modifier.is_expired());

        modifier.tick(3.0);
        assert!(modifier.is_expired());
    }

    #[test]
    fn test_hp_and_stamina_percent() {
        let stats = BaseCombatStats::new().with_hp(100.0).with_stamina(50.0);
        assert_eq!(stats.hp_percent(), 1.0);
        assert_eq!(stats.stamina_percent(), 1.0);

        let mut stats = stats;
        stats.hp = 50.0;
        stats.stamina = 25.0;
        assert_eq!(stats.hp_percent(), 0.5);
        assert_eq!(stats.stamina_percent(), 0.5);
    }
}
