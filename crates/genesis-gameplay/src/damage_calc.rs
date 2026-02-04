//! Damage calculation system.
//!
//! This module provides:
//! - Base damage from weapon + stats
//! - Defense reduction formula
//! - Critical hit multiplier
//! - Elemental resistances
//! - Damage types (physical, fire, ice, poison)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// G-52: Damage Types
// ============================================================================

/// Types of damage that can be dealt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageCategory {
    /// Physical damage - reduced by armor.
    Physical,
    /// Fire damage.
    Fire,
    /// Ice/frost damage.
    Ice,
    /// Lightning/electric damage.
    Lightning,
    /// Poison damage.
    Poison,
    /// Holy/light damage.
    Holy,
    /// Dark/shadow damage.
    Dark,
    /// True damage - ignores all defenses.
    True,
}

impl DamageCategory {
    /// Check if this damage type bypasses armor.
    #[must_use]
    pub fn bypasses_armor(&self) -> bool {
        !matches!(self, Self::Physical)
    }

    /// Check if this damage type is elemental.
    #[must_use]
    pub fn is_elemental(&self) -> bool {
        matches!(
            self,
            Self::Fire | Self::Ice | Self::Lightning | Self::Poison | Self::Holy | Self::Dark
        )
    }

    /// Check if damage ignores all defenses.
    #[must_use]
    pub fn is_true_damage(&self) -> bool {
        matches!(self, Self::True)
    }

    /// Get color for damage type (for UI).
    #[must_use]
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Self::Physical => (200, 200, 200),  // Gray
            Self::Fire => (255, 100, 50),       // Orange-red
            Self::Ice => (100, 200, 255),       // Light blue
            Self::Lightning => (255, 255, 100), // Yellow
            Self::Poison => (100, 255, 100),    // Green
            Self::Holy => (255, 255, 200),      // Light yellow
            Self::Dark => (100, 50, 150),       // Purple
            Self::True => (255, 255, 255),      // White
        }
    }
}

// ============================================================================
// G-52: Resistance System
// ============================================================================

/// Resistances to different damage types.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Resistances {
    /// Resistance values by damage type (0.0 = none, 1.0 = immune, negative = weakness).
    values: HashMap<DamageCategory, f32>,
}

impl Resistances {
    /// Create empty resistances.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set resistance for a damage type.
    pub fn set(&mut self, damage_type: DamageCategory, value: f32) {
        self.values.insert(damage_type, value.clamp(-1.0, 1.0));
    }

    /// Get resistance for a damage type.
    #[must_use]
    pub fn get(&self, damage_type: DamageCategory) -> f32 {
        self.values.get(&damage_type).copied().unwrap_or(0.0)
    }

    /// Add resistance (builder pattern).
    #[must_use]
    pub fn with_resistance(mut self, damage_type: DamageCategory, value: f32) -> Self {
        self.set(damage_type, value);
        self
    }

    /// Calculate damage multiplier from resistance.
    #[must_use]
    pub fn damage_multiplier(&self, damage_type: DamageCategory) -> f32 {
        let resistance = self.get(damage_type);
        1.0 - resistance
    }

    /// Apply resistance to damage.
    #[must_use]
    pub fn apply(&self, damage: f32, damage_type: DamageCategory) -> f32 {
        if damage_type.is_true_damage() {
            damage // True damage ignores resistance
        } else {
            (damage * self.damage_multiplier(damage_type)).max(0.0)
        }
    }

    /// Check if immune to damage type.
    #[must_use]
    pub fn is_immune(&self, damage_type: DamageCategory) -> bool {
        self.get(damage_type) >= 1.0
    }

    /// Check if weak to damage type.
    #[must_use]
    pub fn is_weak(&self, damage_type: DamageCategory) -> bool {
        self.get(damage_type) < 0.0
    }
}

// ============================================================================
// G-52: Defense Calculation
// ============================================================================

/// Defense statistics for damage reduction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefenseStats {
    /// Armor value (reduces physical damage).
    pub armor: f32,
    /// Elemental resistances.
    pub resistances: Resistances,
    /// Flat damage reduction.
    pub flat_reduction: f32,
    /// Percent damage reduction (0.0-0.75).
    pub percent_reduction: f32,
}

impl Default for DefenseStats {
    fn default() -> Self {
        Self {
            armor: 0.0,
            resistances: Resistances::new(),
            flat_reduction: 0.0,
            percent_reduction: 0.0,
        }
    }
}

impl DefenseStats {
    /// Create new defense stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set armor value.
    #[must_use]
    pub fn with_armor(mut self, armor: f32) -> Self {
        self.armor = armor.max(0.0);
        self
    }

    /// Set resistances.
    #[must_use]
    pub fn with_resistances(mut self, resistances: Resistances) -> Self {
        self.resistances = resistances;
        self
    }

    /// Set flat reduction.
    #[must_use]
    pub fn with_flat_reduction(mut self, reduction: f32) -> Self {
        self.flat_reduction = reduction.max(0.0);
        self
    }

    /// Set percent reduction.
    #[must_use]
    pub fn with_percent_reduction(mut self, reduction: f32) -> Self {
        self.percent_reduction = reduction.clamp(0.0, 0.75);
        self
    }

    /// Calculate armor damage reduction using diminishing returns.
    /// Formula: reduction = armor / (armor + scaling_factor)
    #[must_use]
    pub fn armor_reduction(&self, scaling_factor: f32) -> f32 {
        if self.armor <= 0.0 {
            0.0
        } else {
            self.armor / (self.armor + scaling_factor)
        }
    }

    /// Calculate total reduction for a damage type.
    #[must_use]
    pub fn total_reduction(&self, damage_type: DamageCategory) -> f32 {
        let armor_reduction = if damage_type.bypasses_armor() {
            0.0
        } else {
            self.armor_reduction(100.0)
        };

        let resistance = self.resistances.get(damage_type);

        // Combine multiplicatively
        let multiplier =
            (1.0 - armor_reduction) * (1.0 - resistance) * (1.0 - self.percent_reduction);

        1.0 - multiplier
    }
}

// ============================================================================
// G-52: Damage Instance
// ============================================================================

/// A single instance of damage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageInstance {
    /// Base damage amount.
    pub base_damage: f32,
    /// Damage type.
    pub damage_type: DamageCategory,
    /// Whether this can crit.
    pub can_crit: bool,
    /// Whether this is a crit.
    pub is_crit: bool,
    /// Additional crit multiplier.
    pub crit_multiplier: f32,
    /// Penetration (reduces effective armor/resistance).
    pub penetration: f32,
    /// Flat bonus damage.
    pub flat_bonus: f32,
    /// Percent bonus damage.
    pub percent_bonus: f32,
}

impl DamageInstance {
    /// Create a new damage instance.
    #[must_use]
    pub fn new(damage: f32, damage_type: DamageCategory) -> Self {
        Self {
            base_damage: damage,
            damage_type,
            can_crit: true,
            is_crit: false,
            crit_multiplier: 2.0,
            penetration: 0.0,
            flat_bonus: 0.0,
            percent_bonus: 0.0,
        }
    }

    /// Set as critical hit.
    #[must_use]
    pub fn as_crit(mut self) -> Self {
        self.is_crit = true;
        self
    }

    /// Set crit multiplier.
    #[must_use]
    pub fn with_crit_multiplier(mut self, mult: f32) -> Self {
        self.crit_multiplier = mult.max(1.0);
        self
    }

    /// Set penetration.
    #[must_use]
    pub fn with_penetration(mut self, pen: f32) -> Self {
        self.penetration = pen.clamp(0.0, 1.0);
        self
    }

    /// Add flat bonus damage.
    #[must_use]
    pub fn with_flat_bonus(mut self, bonus: f32) -> Self {
        self.flat_bonus += bonus;
        self
    }

    /// Add percent bonus damage.
    #[must_use]
    pub fn with_percent_bonus(mut self, bonus: f32) -> Self {
        self.percent_bonus += bonus;
        self
    }

    /// Disable crits.
    #[must_use]
    pub fn no_crit(mut self) -> Self {
        self.can_crit = false;
        self
    }

    /// Calculate raw damage before defenses.
    #[must_use]
    pub fn raw_damage(&self) -> f32 {
        let mut damage = self.base_damage;

        // Add flat bonus
        damage += self.flat_bonus;

        // Apply percent bonus
        damage *= 1.0 + self.percent_bonus;

        // Apply crit
        if self.is_crit && self.can_crit {
            damage *= self.crit_multiplier;
        }

        damage.max(0.0)
    }
}

// ============================================================================
// G-52: Damage Result
// ============================================================================

/// Result of a damage calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageResult {
    /// Final damage dealt.
    pub final_damage: f32,
    /// Damage type.
    pub damage_type: DamageCategory,
    /// Raw damage before mitigation.
    pub raw_damage: f32,
    /// Damage blocked by armor.
    pub armor_blocked: f32,
    /// Damage reduced by resistance.
    pub resistance_blocked: f32,
    /// Damage reduced by flat reduction.
    pub flat_blocked: f32,
    /// Whether this was a critical hit.
    pub was_crit: bool,
    /// Whether damage was fully blocked.
    pub was_blocked: bool,
    /// Whether target is immune.
    pub was_immune: bool,
}

impl DamageResult {
    /// Get total damage mitigated.
    #[must_use]
    pub fn total_mitigated(&self) -> f32 {
        self.armor_blocked + self.resistance_blocked + self.flat_blocked
    }

    /// Get mitigation percentage.
    #[must_use]
    pub fn mitigation_percent(&self) -> f32 {
        if self.raw_damage <= 0.0 {
            0.0
        } else {
            (self.total_mitigated() / self.raw_damage * 100.0).clamp(0.0, 100.0)
        }
    }
}

// ============================================================================
// G-52: Damage Calculator
// ============================================================================

/// Configuration for damage calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageConfig {
    /// Armor scaling factor for diminishing returns.
    pub armor_scaling: f32,
    /// Base crit multiplier.
    pub base_crit_mult: f32,
    /// Maximum crit multiplier.
    pub max_crit_mult: f32,
    /// Minimum damage (after all reductions).
    pub min_damage: f32,
    /// Maximum damage percent reduction.
    pub max_damage_reduction: f32,
}

impl Default for DamageConfig {
    fn default() -> Self {
        Self {
            armor_scaling: 100.0,
            base_crit_mult: 2.0,
            max_crit_mult: 5.0,
            min_damage: 1.0,
            max_damage_reduction: 0.9,
        }
    }
}

/// Calculator for damage values.
#[derive(Debug, Clone, Default)]
pub struct DamageCalculator {
    /// Configuration.
    pub config: DamageConfig,
}

impl DamageCalculator {
    /// Create new calculator with default config.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create calculator with config.
    #[must_use]
    pub fn with_config(config: DamageConfig) -> Self {
        Self { config }
    }

    /// Calculate damage dealt.
    #[must_use]
    pub fn calculate(&self, damage: &DamageInstance, defense: &DefenseStats) -> DamageResult {
        let raw = damage.raw_damage();

        // True damage ignores all defenses
        if damage.damage_type.is_true_damage() {
            return DamageResult {
                final_damage: raw.max(self.config.min_damage),
                damage_type: damage.damage_type,
                raw_damage: raw,
                armor_blocked: 0.0,
                resistance_blocked: 0.0,
                flat_blocked: 0.0,
                was_crit: damage.is_crit,
                was_blocked: false,
                was_immune: false,
            };
        }

        // Check immunity
        if defense.resistances.is_immune(damage.damage_type) {
            return DamageResult {
                final_damage: 0.0,
                damage_type: damage.damage_type,
                raw_damage: raw,
                armor_blocked: 0.0,
                resistance_blocked: raw,
                flat_blocked: 0.0,
                was_crit: damage.is_crit,
                was_blocked: true,
                was_immune: true,
            };
        }

        let mut remaining = raw;
        let mut armor_blocked = 0.0;

        // Apply armor (only for physical)
        if !damage.damage_type.bypasses_armor() {
            let effective_armor = defense.armor * (1.0 - damage.penetration);
            let armor_reduction = effective_armor / (effective_armor + self.config.armor_scaling);
            armor_blocked = remaining * armor_reduction;
            remaining -= armor_blocked;
        }

        // Apply resistance
        let resistance = defense.resistances.get(damage.damage_type) * (1.0 - damage.penetration);
        let mut resistance_blocked = remaining * resistance;
        remaining -= resistance_blocked;

        // Apply percent reduction
        let percent_blocked = remaining * defense.percent_reduction;
        remaining -= percent_blocked;
        resistance_blocked += percent_blocked; // Group with resistance for simplicity

        // Apply flat reduction
        let flat_blocked = defense.flat_reduction.min(remaining);
        remaining -= flat_blocked;

        // Ensure minimum damage (unless fully blocked)
        let final_damage = if remaining <= 0.0 {
            0.0
        } else {
            remaining.max(self.config.min_damage)
        };

        DamageResult {
            final_damage,
            damage_type: damage.damage_type,
            raw_damage: raw,
            armor_blocked,
            resistance_blocked,
            flat_blocked,
            was_crit: damage.is_crit,
            was_blocked: final_damage <= 0.0,
            was_immune: false,
        }
    }

    /// Roll for critical hit.
    #[must_use]
    pub fn roll_crit(&self, crit_chance: f32, roll: f32) -> bool {
        roll < crit_chance.clamp(0.0, 1.0)
    }

    /// Get crit multiplier with cap.
    #[must_use]
    pub fn get_crit_mult(&self, base_mult: f32, bonus_mult: f32) -> f32 {
        (base_mult + bonus_mult).clamp(self.config.base_crit_mult, self.config.max_crit_mult)
    }

    /// Calculate effective damage per second.
    #[must_use]
    pub fn calculate_dps(
        &self,
        base_damage: f32,
        attack_speed: f32,
        crit_chance: f32,
        crit_mult: f32,
    ) -> f32 {
        let crit_factor = 1.0 + (crit_chance * (crit_mult - 1.0));
        base_damage * attack_speed * crit_factor
    }
}

// ============================================================================
// G-52: Composite Damage
// ============================================================================

/// Damage composed of multiple types.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CompositeDamage {
    /// Damage instances.
    pub instances: Vec<DamageInstance>,
}

impl CompositeDamage {
    /// Create empty composite damage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a damage instance.
    pub fn add(&mut self, instance: DamageInstance) {
        self.instances.push(instance);
    }

    /// Add damage instance (builder).
    #[must_use]
    pub fn with(mut self, instance: DamageInstance) -> Self {
        self.add(instance);
        self
    }

    /// Create with single damage type.
    #[must_use]
    pub fn single(damage: f32, damage_type: DamageCategory) -> Self {
        Self {
            instances: vec![DamageInstance::new(damage, damage_type)],
        }
    }

    /// Create physical + elemental split.
    #[must_use]
    pub fn split(physical: f32, elemental: f32, element: DamageCategory) -> Self {
        Self {
            instances: vec![
                DamageInstance::new(physical, DamageCategory::Physical),
                DamageInstance::new(elemental, element),
            ],
        }
    }

    /// Mark all instances as crit.
    pub fn set_crit(&mut self, is_crit: bool, multiplier: f32) {
        for instance in &mut self.instances {
            instance.is_crit = is_crit;
            instance.crit_multiplier = multiplier;
        }
    }

    /// Get total raw damage.
    #[must_use]
    pub fn total_raw(&self) -> f32 {
        self.instances.iter().map(DamageInstance::raw_damage).sum()
    }

    /// Check if any component is a crit.
    #[must_use]
    pub fn is_crit(&self) -> bool {
        self.instances.iter().any(|i| i.is_crit)
    }
}

/// Result of composite damage calculation.
#[derive(Debug, Clone, Default)]
pub struct CompositeDamageResult {
    /// Individual results.
    pub results: Vec<DamageResult>,
}

impl CompositeDamageResult {
    /// Get total final damage.
    #[must_use]
    pub fn total_damage(&self) -> f32 {
        self.results.iter().map(|r| r.final_damage).sum()
    }

    /// Get total raw damage.
    #[must_use]
    pub fn total_raw(&self) -> f32 {
        self.results.iter().map(|r| r.raw_damage).sum()
    }

    /// Get total mitigated.
    #[must_use]
    pub fn total_mitigated(&self) -> f32 {
        self.results.iter().map(DamageResult::total_mitigated).sum()
    }

    /// Check if any was crit.
    #[must_use]
    pub fn was_crit(&self) -> bool {
        self.results.iter().any(|r| r.was_crit)
    }

    /// Check if fully blocked.
    #[must_use]
    pub fn was_blocked(&self) -> bool {
        self.total_damage() <= 0.0
    }
}

impl DamageCalculator {
    /// Calculate composite damage.
    #[must_use]
    pub fn calculate_composite(
        &self,
        damage: &CompositeDamage,
        defense: &DefenseStats,
    ) -> CompositeDamageResult {
        CompositeDamageResult {
            results: damage
                .instances
                .iter()
                .map(|i| self.calculate(i, defense))
                .collect(),
        }
    }
}

// ============================================================================
// G-52: Damage Over Time
// ============================================================================

/// A damage over time effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageOverTime {
    /// Damage per tick.
    pub damage_per_tick: f32,
    /// Damage type.
    pub damage_type: DamageCategory,
    /// Total duration.
    pub duration: f32,
    /// Tick interval.
    pub tick_interval: f32,
    /// Time since last tick.
    pub tick_timer: f32,
    /// Remaining duration.
    pub remaining: f32,
    /// Whether to ignore defenses.
    pub ignore_defense: bool,
}

impl DamageOverTime {
    /// Create new DoT.
    #[must_use]
    pub fn new(total_damage: f32, duration: f32, damage_type: DamageCategory) -> Self {
        let tick_interval = 1.0;
        let ticks = (duration / tick_interval).ceil();
        let damage_per_tick = total_damage / ticks;

        Self {
            damage_per_tick,
            damage_type,
            duration,
            tick_interval,
            tick_timer: 0.0,
            remaining: duration,
            ignore_defense: false,
        }
    }

    /// Set tick interval.
    #[must_use]
    pub fn with_tick_interval(mut self, interval: f32) -> Self {
        let ticks = (self.duration / interval).ceil();
        self.damage_per_tick =
            (self.damage_per_tick * (self.duration / self.tick_interval)) / ticks;
        self.tick_interval = interval;
        self
    }

    /// Set to ignore defenses.
    #[must_use]
    pub fn ignore_defenses(mut self) -> Self {
        self.ignore_defense = true;
        self
    }

    /// Update timer, returns damage ticks.
    pub fn tick(&mut self, dt: f32) -> u32 {
        if self.remaining <= 0.0 {
            return 0;
        }

        self.remaining -= dt;
        self.tick_timer += dt;

        let ticks = (self.tick_timer / self.tick_interval) as u32;
        self.tick_timer %= self.tick_interval;

        ticks
    }

    /// Check if expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Get remaining time.
    #[must_use]
    pub fn remaining_time(&self) -> f32 {
        self.remaining.max(0.0)
    }

    /// Get damage per tick.
    #[must_use]
    pub fn tick_damage(&self) -> f32 {
        self.damage_per_tick
    }

    /// Get total remaining damage.
    #[must_use]
    pub fn remaining_damage(&self) -> f32 {
        let remaining_ticks = (self.remaining / self.tick_interval).ceil();
        self.damage_per_tick * remaining_ticks
    }
}

// ============================================================================
// G-52: Attack Modifiers
// ============================================================================

/// Modifiers applied to an attack.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AttackModifiers {
    /// Flat damage bonus.
    pub flat_damage: f32,
    /// Percent damage bonus.
    pub percent_damage: f32,
    /// Crit chance bonus.
    pub crit_chance: f32,
    /// Crit damage bonus.
    pub crit_damage: f32,
    /// Armor penetration.
    pub armor_pen: f32,
    /// Resistance penetration.
    pub resist_pen: f32,
    /// Lifesteal percentage.
    pub lifesteal: f32,
}

impl AttackModifiers {
    /// Create empty modifiers.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add flat damage.
    #[must_use]
    pub fn with_flat_damage(mut self, damage: f32) -> Self {
        self.flat_damage += damage;
        self
    }

    /// Add percent damage.
    #[must_use]
    pub fn with_percent_damage(mut self, percent: f32) -> Self {
        self.percent_damage += percent;
        self
    }

    /// Add crit chance.
    #[must_use]
    pub fn with_crit_chance(mut self, chance: f32) -> Self {
        self.crit_chance += chance;
        self
    }

    /// Add armor penetration.
    #[must_use]
    pub fn with_armor_pen(mut self, pen: f32) -> Self {
        self.armor_pen += pen;
        self
    }

    /// Add lifesteal.
    #[must_use]
    pub fn with_lifesteal(mut self, percent: f32) -> Self {
        self.lifesteal += percent;
        self
    }

    /// Apply modifiers to damage instance.
    pub fn apply_to(&self, damage: &mut DamageInstance) {
        damage.flat_bonus += self.flat_damage;
        damage.percent_bonus += self.percent_damage;
        damage.penetration = (damage.penetration + self.armor_pen).clamp(0.0, 1.0);
    }

    /// Combine with another set of modifiers.
    #[must_use]
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            flat_damage: self.flat_damage + other.flat_damage,
            percent_damage: self.percent_damage + other.percent_damage,
            crit_chance: self.crit_chance + other.crit_chance,
            crit_damage: self.crit_damage + other.crit_damage,
            armor_pen: (self.armor_pen + other.armor_pen).clamp(0.0, 1.0),
            resist_pen: (self.resist_pen + other.resist_pen).clamp(0.0, 1.0),
            lifesteal: self.lifesteal + other.lifesteal,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_type_properties() {
        assert!(!DamageCategory::Physical.bypasses_armor());
        assert!(DamageCategory::Fire.bypasses_armor());
        assert!(DamageCategory::Fire.is_elemental());
        assert!(!DamageCategory::Physical.is_elemental());
        assert!(DamageCategory::True.is_true_damage());
    }

    #[test]
    fn test_resistance_application() {
        let resist = Resistances::new().with_resistance(DamageCategory::Fire, 0.5);

        assert_eq!(resist.damage_multiplier(DamageCategory::Fire), 0.5);
        assert_eq!(resist.apply(100.0, DamageCategory::Fire), 50.0);
        assert_eq!(resist.apply(100.0, DamageCategory::Physical), 100.0);
    }

    #[test]
    fn test_immunity() {
        let resist = Resistances::new().with_resistance(DamageCategory::Ice, 1.0);

        assert!(resist.is_immune(DamageCategory::Ice));
        assert_eq!(resist.apply(100.0, DamageCategory::Ice), 0.0);
    }

    #[test]
    fn test_weakness() {
        let resist = Resistances::new().with_resistance(DamageCategory::Fire, -0.5);

        assert!(resist.is_weak(DamageCategory::Fire));
        assert_eq!(resist.damage_multiplier(DamageCategory::Fire), 1.5);
    }

    #[test]
    fn test_armor_reduction() {
        let defense = DefenseStats::new().with_armor(100.0);

        // With 100 armor and 100 scaling: 100/(100+100) = 0.5
        assert_eq!(defense.armor_reduction(100.0), 0.5);
    }

    #[test]
    fn test_damage_instance_raw() {
        let damage = DamageInstance::new(100.0, DamageCategory::Physical);
        assert_eq!(damage.raw_damage(), 100.0);

        let crit_damage = damage.clone().as_crit().with_crit_multiplier(2.0);
        assert_eq!(crit_damage.raw_damage(), 200.0);
    }

    #[test]
    fn test_damage_bonuses() {
        let damage = DamageInstance::new(100.0, DamageCategory::Physical)
            .with_flat_bonus(20.0)
            .with_percent_bonus(0.5);

        // 100 + 20 = 120, then * 1.5 = 180
        assert_eq!(damage.raw_damage(), 180.0);
    }

    #[test]
    fn test_damage_calculation() {
        let calc = DamageCalculator::new();
        let damage = DamageInstance::new(100.0, DamageCategory::Physical);
        let defense = DefenseStats::new().with_armor(100.0);

        let result = calc.calculate(&damage, &defense);

        // 50% armor reduction
        assert_eq!(result.armor_blocked, 50.0);
        assert_eq!(result.final_damage, 50.0);
    }

    #[test]
    fn test_elemental_bypasses_armor() {
        let calc = DamageCalculator::new();
        let damage = DamageInstance::new(100.0, DamageCategory::Fire);
        let defense = DefenseStats::new().with_armor(200.0);

        let result = calc.calculate(&damage, &defense);

        // Fire bypasses armor
        assert_eq!(result.armor_blocked, 0.0);
        assert_eq!(result.final_damage, 100.0);
    }

    #[test]
    fn test_penetration() {
        let calc = DamageCalculator::new();
        let damage = DamageInstance::new(100.0, DamageCategory::Physical).with_penetration(0.5);
        let defense = DefenseStats::new().with_armor(100.0);

        let result = calc.calculate(&damage, &defense);

        // Effective armor is 50, so reduction is 50/(50+100) = 1/3
        let expected_blocked = 100.0 / 3.0;
        assert!((result.armor_blocked - expected_blocked).abs() < 0.1);
    }

    #[test]
    fn test_true_damage() {
        let calc = DamageCalculator::new();
        let damage = DamageInstance::new(100.0, DamageCategory::True);
        let defense = DefenseStats::new()
            .with_armor(200.0)
            .with_resistances(Resistances::new().with_resistance(DamageCategory::True, 1.0));

        let result = calc.calculate(&damage, &defense);

        // True damage ignores everything
        assert_eq!(result.final_damage, 100.0);
    }

    #[test]
    fn test_composite_damage() {
        let calc = DamageCalculator::new();
        let damage = CompositeDamage::split(50.0, 50.0, DamageCategory::Fire);
        let defense = DefenseStats::new()
            .with_armor(100.0)
            .with_resistances(Resistances::new().with_resistance(DamageCategory::Fire, 0.5));

        let result = calc.calculate_composite(&damage, &defense);

        // Physical: 50 * 0.5 = 25 (50% armor)
        // Fire: 50 * 0.5 = 25 (50% resistance)
        assert_eq!(result.total_damage(), 50.0);
    }

    #[test]
    fn test_dot() {
        let mut dot = DamageOverTime::new(100.0, 10.0, DamageCategory::Poison);

        // 100 damage over 10 seconds = 10 ticks at 10 damage each
        assert_eq!(dot.damage_per_tick, 10.0);

        let ticks = dot.tick(2.5);
        assert_eq!(ticks, 2);
        assert_eq!(dot.remaining_time(), 7.5);
    }

    #[test]
    fn test_attack_modifiers() {
        let mods = AttackModifiers::new()
            .with_flat_damage(10.0)
            .with_percent_damage(0.2);

        let mut damage = DamageInstance::new(100.0, DamageCategory::Physical);
        mods.apply_to(&mut damage);

        // 100 + 10 = 110, then * 1.2 = 132
        assert_eq!(damage.raw_damage(), 132.0);
    }

    #[test]
    fn test_dps_calculation() {
        let calc = DamageCalculator::new();

        // 100 damage, 2 attacks/sec, 20% crit, 2x crit
        let dps = calc.calculate_dps(100.0, 2.0, 0.2, 2.0);

        // Crit factor: 1 + 0.2 * (2 - 1) = 1.2
        // DPS: 100 * 2 * 1.2 = 240
        assert!((dps - 240.0).abs() < 0.001);
    }

    #[test]
    fn test_flat_reduction() {
        let calc = DamageCalculator::new();
        let damage = DamageInstance::new(50.0, DamageCategory::Physical);
        let defense = DefenseStats::new().with_flat_reduction(20.0);

        let result = calc.calculate(&damage, &defense);
        assert_eq!(result.flat_blocked, 20.0);
        assert_eq!(result.final_damage, 30.0);
    }

    #[test]
    fn test_damage_result_stats() {
        let result = DamageResult {
            final_damage: 50.0,
            damage_type: DamageCategory::Physical,
            raw_damage: 100.0,
            armor_blocked: 30.0,
            resistance_blocked: 10.0,
            flat_blocked: 10.0,
            was_crit: true,
            was_blocked: false,
            was_immune: false,
        };

        assert_eq!(result.total_mitigated(), 50.0);
        assert_eq!(result.mitigation_percent(), 50.0);
    }

    #[test]
    fn test_crit_roll() {
        let calc = DamageCalculator::new();

        assert!(calc.roll_crit(0.5, 0.3)); // 0.3 < 0.5
        assert!(!calc.roll_crit(0.5, 0.7)); // 0.7 >= 0.5
    }

    #[test]
    fn test_crit_mult_cap() {
        let calc = DamageCalculator::new();

        assert_eq!(calc.get_crit_mult(2.0, 0.5), 2.5);
        assert_eq!(calc.get_crit_mult(2.0, 10.0), 5.0); // Capped
    }
}
