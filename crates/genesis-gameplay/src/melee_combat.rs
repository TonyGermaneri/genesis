//! Melee attack logic.
//!
//! This module provides:
//! - Attack timing windows (windup, active, recovery)
//! - Combo chains with timing bonuses
//! - Stamina cost per swing
//! - Weapon reach and arc

use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// G-50: Attack Phases
// ============================================================================

/// Phase of a melee attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackPhase {
    /// Preparing to strike.
    Windup,
    /// Active damage window.
    Active,
    /// Recovering after attack.
    Recovery,
    /// Attack complete.
    Complete,
    /// Attack was cancelled.
    Cancelled,
}

impl AttackPhase {
    /// Check if attack can deal damage.
    #[must_use]
    pub fn can_damage(&self) -> bool {
        *self == Self::Active
    }

    /// Check if attack is finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Complete | Self::Cancelled)
    }

    /// Check if attack can be cancelled.
    #[must_use]
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Windup | Self::Recovery)
    }
}

// ============================================================================
// G-50: Melee Weapon Types
// ============================================================================

/// Type of melee weapon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MeleeWeaponType {
    /// Unarmed combat.
    Fists,
    /// One-handed sword.
    Sword,
    /// Two-handed greatsword.
    Greatsword,
    /// Axe (one or two handed).
    Axe,
    /// Mace/hammer.
    Mace,
    /// Dagger/knife.
    Dagger,
    /// Spear/polearm.
    Spear,
    /// Staff.
    Staff,
    /// Whip.
    Whip,
    /// Shield bash.
    Shield,
}

impl MeleeWeaponType {
    /// Get default reach for this weapon type.
    #[must_use]
    pub fn default_reach(&self) -> f32 {
        match self {
            Self::Fists => 0.8,
            Self::Dagger | Self::Shield => 1.0,
            Self::Sword => 1.5,
            Self::Axe => 1.4,
            Self::Mace => 1.3,
            Self::Staff => 1.8,
            Self::Greatsword => 2.0,
            Self::Spear => 2.5,
            Self::Whip => 3.0,
        }
    }

    /// Get default arc angle (radians) for this weapon type.
    #[must_use]
    pub fn default_arc(&self) -> f32 {
        match self {
            Self::Fists | Self::Shield => std::f32::consts::PI * 0.4, // 72 degrees
            Self::Dagger | Self::Whip => std::f32::consts::PI * 0.3,  // 54 degrees (narrow)
            Self::Sword | Self::Mace => std::f32::consts::PI * 0.5,   // 90 degrees
            Self::Axe => std::f32::consts::PI * 0.6,                  // 108 degrees (wide swing)
            Self::Staff => std::f32::consts::PI * 0.7,                // 126 degrees (sweeping)
            Self::Greatsword => std::f32::consts::PI * 0.75,          // 135 degrees (very wide)
            Self::Spear => std::f32::consts::PI * 0.2,                // 36 degrees (thrust)
        }
    }

    /// Get default base stamina cost.
    #[must_use]
    pub fn base_stamina_cost(&self) -> f32 {
        match self {
            Self::Fists => 5.0,
            Self::Dagger => 8.0,
            Self::Staff | Self::Whip => 10.0,
            Self::Sword => 12.0,
            Self::Spear => 14.0,
            Self::Axe | Self::Shield => 15.0,
            Self::Mace => 18.0,
            Self::Greatsword => 25.0,
        }
    }
}

// ============================================================================
// G-50: Attack Timing
// ============================================================================

/// Timing configuration for a melee attack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackTiming {
    /// Duration of windup phase (seconds).
    pub windup: f32,
    /// Duration of active phase (seconds).
    pub active: f32,
    /// Duration of recovery phase (seconds).
    pub recovery: f32,
}

impl Default for AttackTiming {
    fn default() -> Self {
        Self {
            windup: 0.15,
            active: 0.1,
            recovery: 0.2,
        }
    }
}

impl AttackTiming {
    /// Create new attack timing.
    #[must_use]
    pub fn new(windup: f32, active: f32, recovery: f32) -> Self {
        Self {
            windup: windup.max(0.0),
            active: active.max(0.01),
            recovery: recovery.max(0.0),
        }
    }

    /// Get total attack duration.
    #[must_use]
    pub fn total_duration(&self) -> f32 {
        self.windup + self.active + self.recovery
    }

    /// Create fast attack timing.
    #[must_use]
    pub fn fast() -> Self {
        Self::new(0.08, 0.05, 0.12)
    }

    /// Create medium attack timing.
    #[must_use]
    pub fn medium() -> Self {
        Self::new(0.15, 0.1, 0.2)
    }

    /// Create slow attack timing.
    #[must_use]
    pub fn slow() -> Self {
        Self::new(0.3, 0.15, 0.35)
    }

    /// Create heavy attack timing.
    #[must_use]
    pub fn heavy() -> Self {
        Self::new(0.5, 0.2, 0.5)
    }

    /// Scale timing by speed multiplier.
    #[must_use]
    pub fn scaled(&self, speed: f32) -> Self {
        let multiplier = 1.0 / speed.max(0.1);
        Self {
            windup: self.windup * multiplier,
            active: self.active * multiplier,
            recovery: self.recovery * multiplier,
        }
    }
}

// ============================================================================
// G-50: Melee Weapon Stats
// ============================================================================

/// Statistics for a melee weapon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeleeWeaponStats {
    /// Weapon type.
    pub weapon_type: MeleeWeaponType,
    /// Base damage.
    pub damage: f32,
    /// Attack reach.
    pub reach: f32,
    /// Attack arc (radians).
    pub arc: f32,
    /// Attack timing.
    pub timing: AttackTiming,
    /// Stamina cost per swing.
    pub stamina_cost: f32,
    /// Knockback force.
    pub knockback: f32,
    /// Critical chance bonus.
    pub crit_bonus: f32,
    /// Number of attacks in combo chain.
    pub combo_length: u8,
    /// Damage multiplier for combo finisher.
    pub combo_finisher_mult: f32,
}

impl Default for MeleeWeaponStats {
    fn default() -> Self {
        Self {
            weapon_type: MeleeWeaponType::Sword,
            damage: 10.0,
            reach: MeleeWeaponType::Sword.default_reach(),
            arc: MeleeWeaponType::Sword.default_arc(),
            timing: AttackTiming::medium(),
            stamina_cost: MeleeWeaponType::Sword.base_stamina_cost(),
            knockback: 3.0,
            crit_bonus: 0.0,
            combo_length: 3,
            combo_finisher_mult: 1.5,
        }
    }
}

impl MeleeWeaponStats {
    /// Create weapon stats for a weapon type.
    #[must_use]
    pub fn for_type(weapon_type: MeleeWeaponType) -> Self {
        Self {
            weapon_type,
            reach: weapon_type.default_reach(),
            arc: weapon_type.default_arc(),
            stamina_cost: weapon_type.base_stamina_cost(),
            ..Default::default()
        }
    }

    /// Set damage.
    #[must_use]
    pub fn with_damage(mut self, damage: f32) -> Self {
        self.damage = damage;
        self
    }

    /// Set reach.
    #[must_use]
    pub fn with_reach(mut self, reach: f32) -> Self {
        self.reach = reach;
        self
    }

    /// Set arc.
    #[must_use]
    pub fn with_arc(mut self, arc: f32) -> Self {
        self.arc = arc;
        self
    }

    /// Set timing.
    #[must_use]
    pub fn with_timing(mut self, timing: AttackTiming) -> Self {
        self.timing = timing;
        self
    }

    /// Set stamina cost.
    #[must_use]
    pub fn with_stamina_cost(mut self, cost: f32) -> Self {
        self.stamina_cost = cost;
        self
    }

    /// Set knockback.
    #[must_use]
    pub fn with_knockback(mut self, knockback: f32) -> Self {
        self.knockback = knockback;
        self
    }

    /// Set combo configuration.
    #[must_use]
    pub fn with_combo(mut self, length: u8, finisher_mult: f32) -> Self {
        self.combo_length = length.max(1);
        self.combo_finisher_mult = finisher_mult;
        self
    }
}

// ============================================================================
// G-50: Active Melee Attack
// ============================================================================

/// An active melee attack in progress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveMeleeAttack {
    /// Attacking entity.
    pub attacker: EntityId,
    /// Target direction (radians).
    pub direction: f32,
    /// Current phase.
    pub phase: AttackPhase,
    /// Time in current phase.
    pub phase_time: f32,
    /// Attack timing.
    pub timing: AttackTiming,
    /// Weapon stats.
    pub weapon: MeleeWeaponStats,
    /// Current combo index (0 = first attack).
    pub combo_index: u8,
    /// Whether hit has been registered.
    pub hit_registered: bool,
    /// Entities already hit (no multi-hit).
    pub hit_entities: Vec<EntityId>,
    /// Total damage for this attack.
    pub total_damage: f32,
}

impl ActiveMeleeAttack {
    /// Create a new melee attack.
    #[must_use]
    pub fn new(attacker: EntityId, direction: f32, weapon: MeleeWeaponStats) -> Self {
        Self {
            attacker,
            direction,
            phase: AttackPhase::Windup,
            phase_time: 0.0,
            timing: weapon.timing.clone(),
            weapon,
            combo_index: 0,
            hit_registered: false,
            hit_entities: Vec::new(),
            total_damage: 0.0,
        }
    }

    /// Set combo index.
    #[must_use]
    pub fn with_combo_index(mut self, index: u8) -> Self {
        self.combo_index = index;
        self
    }

    /// Apply attack speed multiplier.
    #[must_use]
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.timing = self.weapon.timing.scaled(speed);
        self
    }

    /// Update attack state.
    pub fn tick(&mut self, dt: f32) {
        if self.phase.is_finished() {
            return;
        }

        self.phase_time += dt;

        // Transition between phases
        match self.phase {
            AttackPhase::Windup => {
                if self.phase_time >= self.timing.windup {
                    self.phase = AttackPhase::Active;
                    self.phase_time -= self.timing.windup;
                }
            },
            AttackPhase::Active => {
                if self.phase_time >= self.timing.active {
                    self.phase = AttackPhase::Recovery;
                    self.phase_time -= self.timing.active;
                }
            },
            AttackPhase::Recovery => {
                if self.phase_time >= self.timing.recovery {
                    self.phase = AttackPhase::Complete;
                }
            },
            _ => {},
        }
    }

    /// Cancel the attack.
    pub fn cancel(&mut self) {
        if self.phase.can_cancel() {
            self.phase = AttackPhase::Cancelled;
        }
    }

    /// Check if attack is in damage window.
    #[must_use]
    pub fn can_damage(&self) -> bool {
        self.phase.can_damage()
    }

    /// Get progress through current phase (0.0-1.0).
    #[must_use]
    pub fn phase_progress(&self) -> f32 {
        let duration = match self.phase {
            AttackPhase::Windup => self.timing.windup,
            AttackPhase::Active => self.timing.active,
            AttackPhase::Recovery => self.timing.recovery,
            _ => 1.0,
        };
        if duration <= 0.0 {
            1.0
        } else {
            (self.phase_time / duration).clamp(0.0, 1.0)
        }
    }

    /// Check if position is in attack range and arc.
    #[must_use]
    pub fn is_in_range(&self, attacker_pos: (f32, f32), target_pos: (f32, f32)) -> bool {
        let dx = target_pos.0 - attacker_pos.0;
        let dy = target_pos.1 - attacker_pos.1;
        let dist = (dx * dx + dy * dy).sqrt();

        // Check range
        if dist > self.weapon.reach {
            return false;
        }

        // Check arc
        let angle_to_target = dy.atan2(dx);
        let angle_diff = (angle_to_target - self.direction).abs();
        let normalized_diff = if angle_diff > std::f32::consts::PI {
            std::f32::consts::TAU - angle_diff
        } else {
            angle_diff
        };

        normalized_diff <= self.weapon.arc / 2.0
    }

    /// Register a hit on an entity.
    pub fn register_hit(&mut self, target: EntityId, damage: f32) -> bool {
        if self.hit_entities.contains(&target) {
            return false;
        }
        self.hit_entities.push(target);
        self.total_damage += damage;
        self.hit_registered = true;
        true
    }

    /// Get damage multiplier for current combo.
    #[must_use]
    pub fn combo_damage_multiplier(&self) -> f32 {
        if self.combo_index + 1 >= self.weapon.combo_length {
            self.weapon.combo_finisher_mult
        } else {
            1.0 + (self.combo_index as f32 * 0.1) // 10% per combo hit
        }
    }

    /// Check if this is combo finisher.
    #[must_use]
    pub fn is_finisher(&self) -> bool {
        self.combo_index + 1 >= self.weapon.combo_length
    }
}

// ============================================================================
// G-50: Combo System
// ============================================================================

/// Combo timing window configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComboConfig {
    /// Time window to continue combo (seconds).
    pub window: f32,
    /// Bonus damage per combo hit (percentage).
    pub damage_bonus: f32,
    /// Stamina discount per combo hit (percentage).
    pub stamina_discount: f32,
    /// Speed increase per combo hit (percentage).
    pub speed_bonus: f32,
}

impl Default for ComboConfig {
    fn default() -> Self {
        Self {
            window: 0.5,
            damage_bonus: 0.1,
            stamina_discount: 0.05,
            speed_bonus: 0.05,
        }
    }
}

/// State of an entity's combo.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ComboState {
    /// Current combo count.
    pub count: u8,
    /// Time since last attack.
    pub timer: f32,
    /// Total damage in combo.
    pub total_damage: f32,
    /// Total hits in combo.
    pub total_hits: u32,
    /// Best combo achieved.
    pub best_combo: u8,
}

impl ComboState {
    /// Create new combo state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update timer.
    pub fn tick(&mut self, dt: f32, window: f32) {
        self.timer += dt;
        if self.timer > window {
            self.reset();
        }
    }

    /// Register attack, returns combo index.
    pub fn register_attack(&mut self, damage: f32, hits: u32) -> u8 {
        let index = self.count;
        self.count = self.count.saturating_add(1);
        self.total_damage += damage;
        self.total_hits += hits;
        self.timer = 0.0;
        if self.count > self.best_combo {
            self.best_combo = self.count;
        }
        index
    }

    /// Reset combo.
    pub fn reset(&mut self) {
        self.count = 0;
        self.timer = 0.0;
        self.total_damage = 0.0;
        self.total_hits = 0;
    }

    /// Get damage multiplier from combo.
    #[must_use]
    pub fn damage_multiplier(&self, config: &ComboConfig) -> f32 {
        1.0 + (self.count as f32 * config.damage_bonus)
    }

    /// Get stamina cost multiplier from combo.
    #[must_use]
    pub fn stamina_multiplier(&self, config: &ComboConfig) -> f32 {
        (1.0 - (self.count as f32 * config.stamina_discount)).max(0.5)
    }

    /// Get speed multiplier from combo.
    #[must_use]
    pub fn speed_multiplier(&self, config: &ComboConfig) -> f32 {
        1.0 + (self.count as f32 * config.speed_bonus)
    }

    /// Check if in combo.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.count > 0
    }
}

// ============================================================================
// G-50: Melee Combat System
// ============================================================================

/// Result of a melee attack attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum MeleeAttackResult {
    /// Attack started successfully.
    Started(ActiveMeleeAttack),
    /// Not enough stamina.
    InsufficientStamina {
        /// Stamina required for the attack.
        required: f32,
        /// Stamina currently available.
        available: f32,
    },
    /// Attack on cooldown.
    OnCooldown {
        /// Time remaining on cooldown.
        remaining: f32,
    },
    /// Cannot attack (stunned, etc.).
    CannotAttack,
    /// No weapon equipped.
    NoWeapon,
}

/// A hit from a melee attack.
#[derive(Debug, Clone, PartialEq)]
pub struct MeleeHit {
    /// Target entity.
    pub target: EntityId,
    /// Damage dealt.
    pub damage: f32,
    /// Knockback direction and force.
    pub knockback: (f32, f32),
    /// Whether this was a critical hit.
    pub critical: bool,
    /// Whether this was a combo finisher.
    pub finisher: bool,
    /// Combo count at hit.
    pub combo_count: u8,
}

/// Manages melee combat for entities.
#[derive(Debug, Default)]
pub struct MeleeCombatSystem {
    /// Active melee attacks by entity.
    active_attacks: HashMap<EntityId, ActiveMeleeAttack>,
    /// Combo states by entity.
    combo_states: HashMap<EntityId, ComboState>,
    /// Attack cooldowns by entity.
    cooldowns: HashMap<EntityId, f32>,
    /// Combo configuration.
    pub combo_config: ComboConfig,
}

impl MeleeCombatSystem {
    /// Create new melee combat system.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempt to start a melee attack.
    #[allow(clippy::too_many_arguments)]
    pub fn start_attack(
        &mut self,
        attacker: EntityId,
        direction: f32,
        weapon: MeleeWeaponStats,
        stamina: f32,
        attack_speed: f32,
        can_attack: bool,
    ) -> MeleeAttackResult {
        // Check if can attack
        if !can_attack {
            return MeleeAttackResult::CannotAttack;
        }

        // Check cooldown
        if let Some(&cooldown) = self.cooldowns.get(&attacker) {
            if cooldown > 0.0 {
                return MeleeAttackResult::OnCooldown {
                    remaining: cooldown,
                };
            }
        }

        // Check if already attacking
        if let Some(attack) = self.active_attacks.get(&attacker) {
            if !attack.phase.is_finished() {
                return MeleeAttackResult::OnCooldown {
                    remaining: attack.timing.recovery,
                };
            }
        }

        // Get combo state
        let combo = self.combo_states.entry(attacker).or_default();
        let combo_index = combo.count % weapon.combo_length;

        // Calculate stamina cost
        let stamina_mult = combo.stamina_multiplier(&self.combo_config);
        let stamina_cost = weapon.stamina_cost * stamina_mult;

        if stamina < stamina_cost {
            return MeleeAttackResult::InsufficientStamina {
                required: stamina_cost,
                available: stamina,
            };
        }

        // Calculate speed
        let speed_mult = combo.speed_multiplier(&self.combo_config);
        let total_speed = attack_speed * speed_mult;

        // Create attack
        let attack = ActiveMeleeAttack::new(attacker, direction, weapon)
            .with_combo_index(combo_index)
            .with_speed(total_speed);

        self.active_attacks.insert(attacker, attack.clone());

        MeleeAttackResult::Started(attack)
    }

    /// Update all active attacks.
    pub fn tick(&mut self, dt: f32) {
        // Update cooldowns
        for cooldown in self.cooldowns.values_mut() {
            *cooldown = (*cooldown - dt).max(0.0);
        }

        // Update combo timers
        let window = self.combo_config.window;
        for combo in self.combo_states.values_mut() {
            combo.tick(dt, window);
        }

        // Update attacks
        let mut completed = Vec::new();
        for (entity, attack) in &mut self.active_attacks {
            attack.tick(dt);
            if attack.phase.is_finished() {
                completed.push(*entity);
            }
        }

        // Clean up completed attacks
        for entity in completed {
            if let Some(attack) = self.active_attacks.remove(&entity) {
                // Set cooldown based on recovery
                self.cooldowns.insert(entity, 0.1); // Small buffer

                // Update combo
                if let Some(combo) = self.combo_states.get_mut(&entity) {
                    if attack.hit_registered {
                        combo
                            .register_attack(attack.total_damage, attack.hit_entities.len() as u32);
                    }
                }
            }
        }
    }

    /// Get active attack for entity.
    #[must_use]
    pub fn get_attack(&self, entity: EntityId) -> Option<&ActiveMeleeAttack> {
        self.active_attacks.get(&entity)
    }

    /// Get mutable attack for entity.
    pub fn get_attack_mut(&mut self, entity: EntityId) -> Option<&mut ActiveMeleeAttack> {
        self.active_attacks.get_mut(&entity)
    }

    /// Get combo state for entity.
    #[must_use]
    pub fn get_combo(&self, entity: EntityId) -> Option<&ComboState> {
        self.combo_states.get(&entity)
    }

    /// Cancel attack for entity.
    pub fn cancel_attack(&mut self, entity: EntityId) {
        if let Some(attack) = self.active_attacks.get_mut(&entity) {
            attack.cancel();
        }
    }

    /// Check if entity is attacking.
    #[must_use]
    pub fn is_attacking(&self, entity: EntityId) -> bool {
        self.active_attacks
            .get(&entity)
            .is_some_and(|a| !a.phase.is_finished())
    }

    /// Check if entity is in damage window.
    #[must_use]
    pub fn in_damage_window(&self, entity: EntityId) -> bool {
        self.active_attacks
            .get(&entity)
            .is_some_and(ActiveMeleeAttack::can_damage)
    }

    /// Process hit for an attack.
    pub fn process_hit(
        &mut self,
        attacker: EntityId,
        target: EntityId,
        base_damage: f32,
        critical: bool,
    ) -> Option<MeleeHit> {
        let attack = self.active_attacks.get_mut(&attacker)?;

        if !attack.can_damage() {
            return None;
        }

        // Calculate damage with combo multiplier
        let combo = self.combo_states.get(&attacker);
        let combo_mult = combo.map_or(1.0, |c| c.damage_multiplier(&self.combo_config));
        let attack_combo_mult = attack.combo_damage_multiplier();

        let final_damage = base_damage * combo_mult * attack_combo_mult;

        // Register hit
        if !attack.register_hit(target, final_damage) {
            return None; // Already hit this target
        }

        // Calculate knockback
        let kb_force = attack.weapon.knockback;
        let kb_dir = attack.direction;
        let knockback = (kb_dir.cos() * kb_force, kb_dir.sin() * kb_force);

        Some(MeleeHit {
            target,
            damage: final_damage,
            knockback,
            critical,
            finisher: attack.is_finisher(),
            combo_count: attack.combo_index + 1,
        })
    }

    /// Reset combo for entity.
    pub fn reset_combo(&mut self, entity: EntityId) {
        if let Some(combo) = self.combo_states.get_mut(&entity) {
            combo.reset();
        }
    }

    /// Remove entity from system.
    pub fn remove_entity(&mut self, entity: EntityId) {
        self.active_attacks.remove(&entity);
        self.combo_states.remove(&entity);
        self.cooldowns.remove(&entity);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entity() -> EntityId {
        EntityId::from_raw(1)
    }

    fn test_target() -> EntityId {
        EntityId::from_raw(2)
    }

    #[test]
    fn test_attack_phase_damage() {
        assert!(!AttackPhase::Windup.can_damage());
        assert!(AttackPhase::Active.can_damage());
        assert!(!AttackPhase::Recovery.can_damage());
    }

    #[test]
    fn test_attack_phase_cancel() {
        assert!(AttackPhase::Windup.can_cancel());
        assert!(!AttackPhase::Active.can_cancel());
        assert!(AttackPhase::Recovery.can_cancel());
    }

    #[test]
    fn test_weapon_type_defaults() {
        assert!(MeleeWeaponType::Spear.default_reach() > MeleeWeaponType::Dagger.default_reach());
        assert!(
            MeleeWeaponType::Greatsword.base_stamina_cost()
                > MeleeWeaponType::Dagger.base_stamina_cost()
        );
    }

    #[test]
    fn test_attack_timing() {
        let timing = AttackTiming::new(0.1, 0.1, 0.2);
        assert_eq!(timing.total_duration(), 0.4);

        let scaled = timing.scaled(2.0);
        assert_eq!(scaled.total_duration(), 0.2);
    }

    #[test]
    fn test_active_attack_progression() {
        let weapon = MeleeWeaponStats::default();
        let mut attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon);

        assert_eq!(attack.phase, AttackPhase::Windup);

        // Progress through windup
        attack.tick(0.2);
        assert_eq!(attack.phase, AttackPhase::Active);

        // Progress through active
        attack.tick(0.15);
        assert_eq!(attack.phase, AttackPhase::Recovery);

        // Progress through recovery
        attack.tick(0.3);
        assert_eq!(attack.phase, AttackPhase::Complete);
    }

    #[test]
    fn test_attack_cancel() {
        let weapon = MeleeWeaponStats::default();
        let mut attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon);

        assert!(attack.phase.can_cancel());
        attack.cancel();
        assert_eq!(attack.phase, AttackPhase::Cancelled);
    }

    #[test]
    fn test_attack_cannot_cancel_during_active() {
        let weapon = MeleeWeaponStats::default();
        let mut attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon);

        attack.tick(0.2); // Into active phase
        assert!(!attack.phase.can_cancel());
        attack.cancel();
        assert_eq!(attack.phase, AttackPhase::Active); // Not cancelled
    }

    #[test]
    fn test_range_check() {
        let weapon = MeleeWeaponStats::default()
            .with_reach(2.0)
            .with_arc(std::f32::consts::PI);
        let attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon);

        // In range and arc
        assert!(attack.is_in_range((0.0, 0.0), (1.0, 0.0)));

        // Out of range
        assert!(!attack.is_in_range((0.0, 0.0), (3.0, 0.0)));

        // Out of arc (behind)
        assert!(!attack.is_in_range((0.0, 0.0), (-1.0, 0.0)));
    }

    #[test]
    fn test_combo_state() {
        let config = ComboConfig::default();
        let mut combo = ComboState::new();

        assert_eq!(combo.count, 0);

        combo.register_attack(10.0, 1);
        assert_eq!(combo.count, 1);

        combo.register_attack(15.0, 1);
        assert_eq!(combo.count, 2);

        assert!(combo.damage_multiplier(&config) > 1.0);
    }

    #[test]
    fn test_combo_expiry() {
        let mut combo = ComboState::new();
        combo.register_attack(10.0, 1);

        combo.tick(1.0, 0.5); // Exceeds window
        assert_eq!(combo.count, 0);
    }

    #[test]
    fn test_melee_system_start_attack() {
        let mut system = MeleeCombatSystem::new();
        let weapon = MeleeWeaponStats::default();

        let result = system.start_attack(test_entity(), 0.0, weapon, 100.0, 1.0, true);
        assert!(matches!(result, MeleeAttackResult::Started(_)));
    }

    #[test]
    fn test_melee_system_insufficient_stamina() {
        let mut system = MeleeCombatSystem::new();
        let weapon = MeleeWeaponStats::default().with_stamina_cost(50.0);

        let result = system.start_attack(test_entity(), 0.0, weapon, 10.0, 1.0, true);
        assert!(matches!(
            result,
            MeleeAttackResult::InsufficientStamina { .. }
        ));
    }

    #[test]
    fn test_melee_system_cannot_attack() {
        let mut system = MeleeCombatSystem::new();
        let weapon = MeleeWeaponStats::default();

        let result = system.start_attack(test_entity(), 0.0, weapon, 100.0, 1.0, false);
        assert!(matches!(result, MeleeAttackResult::CannotAttack));
    }

    #[test]
    fn test_hit_registration() {
        let weapon = MeleeWeaponStats::default();
        let mut attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon);

        assert!(attack.register_hit(test_target(), 10.0));
        assert!(!attack.register_hit(test_target(), 10.0)); // Already hit
    }

    #[test]
    fn test_combo_damage_multiplier() {
        let weapon = MeleeWeaponStats::default().with_combo(3, 1.5);

        let mut attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon.clone());
        assert_eq!(attack.combo_damage_multiplier(), 1.0); // First hit

        let attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon.clone()).with_combo_index(1);
        assert_eq!(attack.combo_damage_multiplier(), 1.1); // Second hit

        let attack = ActiveMeleeAttack::new(test_entity(), 0.0, weapon).with_combo_index(2);
        assert_eq!(attack.combo_damage_multiplier(), 1.5); // Finisher
    }

    #[test]
    fn test_system_process_hit() {
        let mut system = MeleeCombatSystem::new();
        let weapon = MeleeWeaponStats::default();

        // Start attack
        system.start_attack(test_entity(), 0.0, weapon, 100.0, 1.0, true);

        // Progress to active phase
        system.tick(0.2);
        assert!(system.in_damage_window(test_entity()));

        // Process hit
        let hit = system.process_hit(test_entity(), test_target(), 10.0, false);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().target, test_target());
    }
}
