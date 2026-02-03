//! Survival needs system (hunger, thirst, energy).

use serde::{Deserialize, Serialize};

/// Effects that can be applied when needs are critical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedEffect {
    /// No effect
    None,
    /// Movement speed reduced
    SlowMovement,
    /// Stamina regeneration reduced
    ReducedStamina,
    /// Taking periodic damage
    Damage,
    /// Vision impaired
    ImpairedVision,
    /// Unconscious (needs depleted)
    Unconscious,
}

/// Status of a need.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedStatus {
    /// Full (80-100%)
    Full,
    /// Satisfied (50-79%)
    Satisfied,
    /// Hungry/Thirsty/Tired (20-49%)
    Low,
    /// Critical (1-19%)
    Critical,
    /// Depleted (0%)
    Depleted,
}

impl NeedStatus {
    /// Returns status from percentage (0.0-1.0).
    #[must_use]
    pub fn from_percentage(pct: f32) -> Self {
        match pct {
            p if p <= 0.0 => Self::Depleted,
            p if p < 0.2 => Self::Critical,
            p if p < 0.5 => Self::Low,
            p if p < 0.8 => Self::Satisfied,
            _ => Self::Full,
        }
    }

    /// Checks if this status is problematic.
    #[must_use]
    pub const fn is_problematic(self) -> bool {
        matches!(self, Self::Critical | Self::Depleted)
    }
}

/// A survival need with current and max values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Need {
    /// Current value (0.0 to max)
    current: f32,
    /// Maximum value
    max: f32,
    /// Decay rate per second
    decay_rate: f32,
    /// Name of the need (for display)
    name: String,
}

impl Need {
    /// Creates a new need.
    #[must_use]
    pub fn new(name: impl Into<String>, max: f32, decay_rate: f32) -> Self {
        Self {
            current: max,
            max,
            decay_rate,
            name: name.into(),
        }
    }

    /// Returns the need's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns current value.
    #[must_use]
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Returns max value.
    #[must_use]
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Returns value as percentage (0.0-1.0).
    #[must_use]
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }

    /// Returns the current status.
    #[must_use]
    pub fn status(&self) -> NeedStatus {
        NeedStatus::from_percentage(self.percentage())
    }

    /// Updates the need by the given delta time (seconds).
    pub fn tick(&mut self, delta_seconds: f32) {
        self.current = (self.current - self.decay_rate * delta_seconds).max(0.0);
    }

    /// Restores the need by the given amount.
    pub fn restore(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Depletes the need by the given amount.
    pub fn deplete(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    /// Sets current value directly.
    pub fn set(&mut self, value: f32) {
        self.current = value.clamp(0.0, self.max);
    }

    /// Checks if critically low (<20%).
    #[must_use]
    pub fn is_critical(&self) -> bool {
        self.percentage() < 0.2
    }

    /// Checks if depleted (0).
    #[must_use]
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }

    /// Checks if full (>=80%).
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.percentage() >= 0.8
    }

    /// Returns the effect for the current status.
    #[must_use]
    pub fn current_effect(&self) -> NeedEffect {
        match self.status() {
            NeedStatus::Full | NeedStatus::Satisfied => NeedEffect::None,
            NeedStatus::Low => NeedEffect::ReducedStamina,
            NeedStatus::Critical => NeedEffect::Damage,
            NeedStatus::Depleted => NeedEffect::Unconscious,
        }
    }
}

/// Collection of survival needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Needs {
    /// Hunger need (restored by eating)
    pub hunger: Need,
    /// Thirst need (restored by drinking)
    pub thirst: Need,
    /// Energy need (restored by resting)
    pub energy: Need,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            // Hunger: decays at 1 per second, 100 max = ~100 seconds to empty (game time)
            hunger: Need::new("Hunger", 100.0, 1.0),
            // Thirst: decays at 1.5 per second = ~67 seconds to empty
            thirst: Need::new("Thirst", 100.0, 1.5),
            // Energy: decays at 0.5 per second = ~200 seconds to empty
            energy: Need::new("Energy", 100.0, 0.5),
        }
    }
}

impl Needs {
    /// Creates a new needs collection with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates needs with custom decay rates.
    #[must_use]
    pub fn with_decay_rates(hunger_rate: f32, thirst_rate: f32, energy_rate: f32) -> Self {
        Self {
            hunger: Need::new("Hunger", 100.0, hunger_rate),
            thirst: Need::new("Thirst", 100.0, thirst_rate),
            energy: Need::new("Energy", 100.0, energy_rate),
        }
    }

    /// Updates all needs by the given delta time (seconds).
    pub fn tick(&mut self, delta_seconds: f32) {
        self.hunger.tick(delta_seconds);
        self.thirst.tick(delta_seconds);
        self.energy.tick(delta_seconds);
    }

    /// Eats food (restores hunger).
    pub fn eat(&mut self, nutrition: f32) {
        self.hunger.restore(nutrition);
    }

    /// Drinks (restores thirst).
    pub fn drink(&mut self, hydration: f32) {
        self.thirst.restore(hydration);
    }

    /// Rests (restores energy).
    pub fn rest(&mut self, rest_amount: f32) {
        self.energy.restore(rest_amount);
    }

    /// Exerts energy (depletes energy faster).
    pub fn exert(&mut self, amount: f32) {
        self.energy.deplete(amount);
    }

    /// Checks if any need is critical.
    #[must_use]
    pub fn any_critical(&self) -> bool {
        self.hunger.is_critical() || self.thirst.is_critical() || self.energy.is_critical()
    }

    /// Checks if any need is depleted.
    #[must_use]
    pub fn any_depleted(&self) -> bool {
        self.hunger.is_depleted() || self.thirst.is_depleted() || self.energy.is_depleted()
    }

    /// Checks if all needs are full.
    #[must_use]
    pub fn all_full(&self) -> bool {
        self.hunger.is_full() && self.thirst.is_full() && self.energy.is_full()
    }

    /// Returns the most urgent need (lowest percentage).
    #[must_use]
    pub fn most_urgent(&self) -> &Need {
        let mut lowest = &self.hunger;
        if self.thirst.percentage() < lowest.percentage() {
            lowest = &self.thirst;
        }
        if self.energy.percentage() < lowest.percentage() {
            lowest = &self.energy;
        }
        lowest
    }

    /// Returns all active effects from critical needs.
    #[must_use]
    pub fn active_effects(&self) -> Vec<NeedEffect> {
        let mut effects = Vec::new();

        let hunger_effect = self.hunger.current_effect();
        if hunger_effect != NeedEffect::None {
            effects.push(hunger_effect);
        }

        let thirst_effect = self.thirst.current_effect();
        if thirst_effect != NeedEffect::None {
            effects.push(thirst_effect);
        }

        let energy_effect = self.energy.current_effect();
        if energy_effect != NeedEffect::None {
            effects.push(energy_effect);
        }

        effects
    }

    /// Calculates damage per second from depleted needs.
    #[must_use]
    pub fn damage_per_second(&self) -> f32 {
        let mut dps = 0.0;

        if self.hunger.is_depleted() {
            dps += 1.0;
        } else if self.hunger.is_critical() {
            dps += 0.5;
        }

        if self.thirst.is_depleted() {
            dps += 2.0; // Thirst is more urgent
        } else if self.thirst.is_critical() {
            dps += 1.0;
        }

        dps
    }

    /// Returns a summary of all need percentages.
    #[must_use]
    pub fn summary(&self) -> NeedsSummary {
        NeedsSummary {
            hunger: self.hunger.percentage(),
            thirst: self.thirst.percentage(),
            energy: self.energy.percentage(),
        }
    }
}

/// Summary of need percentages.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NeedsSummary {
    /// Hunger percentage (0.0-1.0)
    pub hunger: f32,
    /// Thirst percentage (0.0-1.0)
    pub thirst: f32,
    /// Energy percentage (0.0-1.0)
    pub energy: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_need_creation() {
        let need = Need::new("Test", 100.0, 1.0);
        assert_eq!(need.name(), "Test");
        assert_eq!(need.current(), 100.0);
        assert_eq!(need.max(), 100.0);
        assert!((need.percentage() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_need_tick() {
        let mut need = Need::new("Test", 100.0, 10.0);
        need.tick(1.0);
        assert_eq!(need.current(), 90.0);

        need.tick(5.0);
        assert_eq!(need.current(), 40.0);
    }

    #[test]
    fn test_need_restore() {
        let mut need = Need::new("Test", 100.0, 10.0);
        need.tick(5.0); // 50
        need.restore(30.0);
        assert_eq!(need.current(), 80.0);

        // Cannot exceed max
        need.restore(100.0);
        assert_eq!(need.current(), 100.0);
    }

    #[test]
    fn test_need_deplete() {
        let mut need = Need::new("Test", 100.0, 0.0);
        need.deplete(30.0);
        assert_eq!(need.current(), 70.0);

        // Cannot go below 0
        need.deplete(100.0);
        assert_eq!(need.current(), 0.0);
    }

    #[test]
    fn test_need_status() {
        let mut need = Need::new("Test", 100.0, 0.0);

        need.set(100.0);
        assert_eq!(need.status(), NeedStatus::Full);

        need.set(60.0);
        assert_eq!(need.status(), NeedStatus::Satisfied);

        need.set(30.0);
        assert_eq!(need.status(), NeedStatus::Low);

        need.set(10.0);
        assert_eq!(need.status(), NeedStatus::Critical);

        need.set(0.0);
        assert_eq!(need.status(), NeedStatus::Depleted);
    }

    #[test]
    fn test_need_effects() {
        let mut need = Need::new("Test", 100.0, 0.0);

        need.set(100.0);
        assert_eq!(need.current_effect(), NeedEffect::None);

        need.set(10.0);
        assert_eq!(need.current_effect(), NeedEffect::Damage);

        need.set(0.0);
        assert_eq!(need.current_effect(), NeedEffect::Unconscious);
    }

    #[test]
    fn test_needs_default() {
        let needs = Needs::new();
        assert!(needs.hunger.is_full());
        assert!(needs.thirst.is_full());
        assert!(needs.energy.is_full());
    }

    #[test]
    fn test_needs_tick() {
        let mut needs = Needs::with_decay_rates(10.0, 10.0, 10.0);
        needs.tick(5.0);

        assert_eq!(needs.hunger.current(), 50.0);
        assert_eq!(needs.thirst.current(), 50.0);
        assert_eq!(needs.energy.current(), 50.0);
    }

    #[test]
    fn test_needs_eat_drink_rest() {
        let mut needs = Needs::with_decay_rates(10.0, 10.0, 10.0);
        needs.tick(8.0); // All at 20

        needs.eat(30.0);
        assert_eq!(needs.hunger.current(), 50.0);

        needs.drink(30.0);
        assert_eq!(needs.thirst.current(), 50.0);

        needs.rest(30.0);
        assert_eq!(needs.energy.current(), 50.0);
    }

    #[test]
    fn test_needs_exert() {
        let mut needs = Needs::new();
        needs.exert(40.0);
        assert_eq!(needs.energy.current(), 60.0);
    }

    #[test]
    fn test_needs_any_critical() {
        let mut needs = Needs::new();
        assert!(!needs.any_critical());

        needs.hunger.set(10.0);
        assert!(needs.any_critical());
    }

    #[test]
    fn test_needs_any_depleted() {
        let mut needs = Needs::new();
        assert!(!needs.any_depleted());

        needs.thirst.set(0.0);
        assert!(needs.any_depleted());
    }

    #[test]
    fn test_needs_all_full() {
        let mut needs = Needs::new();
        assert!(needs.all_full());

        needs.hunger.set(50.0);
        assert!(!needs.all_full());
    }

    #[test]
    fn test_needs_most_urgent() {
        let mut needs = Needs::new();
        needs.hunger.set(50.0);
        needs.thirst.set(30.0);
        needs.energy.set(70.0);

        assert_eq!(needs.most_urgent().name(), "Thirst");
    }

    #[test]
    fn test_needs_active_effects() {
        let mut needs = Needs::new();
        assert!(needs.active_effects().is_empty());

        needs.hunger.set(10.0); // Critical
        needs.thirst.set(0.0); // Depleted

        let effects = needs.active_effects();
        assert_eq!(effects.len(), 2);
        assert!(effects.contains(&NeedEffect::Damage));
        assert!(effects.contains(&NeedEffect::Unconscious));
    }

    #[test]
    fn test_needs_damage_per_second() {
        let mut needs = Needs::new();
        assert_eq!(needs.damage_per_second(), 0.0);

        needs.hunger.set(10.0); // Critical
        assert_eq!(needs.damage_per_second(), 0.5);

        needs.hunger.set(0.0); // Depleted
        assert_eq!(needs.damage_per_second(), 1.0);

        needs.thirst.set(0.0); // Depleted
        assert_eq!(needs.damage_per_second(), 3.0); // 1.0 + 2.0
    }

    #[test]
    fn test_needs_summary() {
        let mut needs = Needs::new();
        needs.hunger.set(50.0);
        needs.thirst.set(75.0);
        needs.energy.set(25.0);

        let summary = needs.summary();
        assert!((summary.hunger - 0.5).abs() < f32::EPSILON);
        assert!((summary.thirst - 0.75).abs() < f32::EPSILON);
        assert!((summary.energy - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_need_status_problematic() {
        assert!(!NeedStatus::Full.is_problematic());
        assert!(!NeedStatus::Satisfied.is_problematic());
        assert!(!NeedStatus::Low.is_problematic());
        assert!(NeedStatus::Critical.is_problematic());
        assert!(NeedStatus::Depleted.is_problematic());
    }
}
