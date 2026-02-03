//! Survival needs system (food, water).

use serde::{Deserialize, Serialize};

/// A survival need with current and max values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Need {
    /// Current value
    current: f32,
    /// Maximum value
    max: f32,
    /// Decay rate per tick
    decay_rate: f32,
}

impl Need {
    /// Creates a new need.
    #[must_use]
    pub fn new(max: f32, decay_rate: f32) -> Self {
        Self {
            current: max,
            max,
            decay_rate,
        }
    }

    /// Returns current value.
    #[must_use]
    pub const fn current(&self) -> f32 {
        self.current
    }

    /// Returns max value.
    #[must_use]
    pub const fn max(&self) -> f32 {
        self.max
    }

    /// Returns value as percentage (0-100).
    #[must_use]
    pub fn percentage(&self) -> f32 {
        (self.current / self.max) * 100.0
    }

    /// Updates the need (applies decay).
    pub fn tick(&mut self) {
        self.current = (self.current - self.decay_rate).max(0.0);
    }

    /// Restores the need by the given amount.
    pub fn restore(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Checks if critically low (<20%).
    #[must_use]
    pub fn is_critical(&self) -> bool {
        self.percentage() < 20.0
    }

    /// Checks if depleted.
    #[must_use]
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }
}

/// Collection of survival needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Needs {
    /// Hunger need
    pub hunger: Need,
    /// Thirst need
    pub thirst: Need,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: Need::new(100.0, 0.01),  // ~166 minutes to empty
            thirst: Need::new(100.0, 0.015), // ~111 minutes to empty
        }
    }
}

impl Needs {
    /// Creates a new needs collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates all needs.
    pub fn tick(&mut self) {
        self.hunger.tick();
        self.thirst.tick();
    }

    /// Checks if any need is critical.
    #[must_use]
    pub fn any_critical(&self) -> bool {
        self.hunger.is_critical() || self.thirst.is_critical()
    }

    /// Checks if any need is depleted.
    #[must_use]
    pub fn any_depleted(&self) -> bool {
        self.hunger.is_depleted() || self.thirst.is_depleted()
    }
}
