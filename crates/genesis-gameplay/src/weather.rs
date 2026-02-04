//! Weather state system for environmental effects.
//!
//! This module provides weather states that affect gameplay:
//! - Rain speeds up grass regrowth and fills water
//! - Storms slow movement and reduce visibility
//! - Clear weather is ideal for exploration

use serde::{Deserialize, Serialize};

/// Weather states in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum WeatherState {
    /// Clear skies, normal conditions.
    #[default]
    Clear,
    /// Overcast, slightly reduced light.
    Cloudy,
    /// Raining, speeds up plant growth, fills water.
    Raining,
    /// Storm with heavy rain and wind.
    Storm,
}

impl WeatherState {
    /// Get the display name for this weather state.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Cloudy => "Cloudy",
            Self::Raining => "Raining",
            Self::Storm => "Storm",
        }
    }

    /// Check if it's currently raining (includes storm).
    #[must_use]
    pub fn is_raining(self) -> bool {
        matches!(self, Self::Raining | Self::Storm)
    }

    /// Check if there's a storm.
    #[must_use]
    pub fn is_stormy(self) -> bool {
        matches!(self, Self::Storm)
    }

    /// Get the light level modifier (1.0 = normal).
    #[must_use]
    pub fn light_modifier(self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::Cloudy => 0.8,
            Self::Raining => 0.6,
            Self::Storm => 0.4,
        }
    }

    /// Get the movement speed modifier (1.0 = normal).
    #[must_use]
    pub fn movement_modifier(self) -> f32 {
        match self {
            Self::Clear | Self::Cloudy => 1.0,
            Self::Raining => 0.9,
            Self::Storm => 0.75,
        }
    }

    /// Get the plant growth speed modifier (1.0 = normal).
    #[must_use]
    pub fn growth_modifier(self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::Cloudy => 1.1,
            Self::Raining => 1.5,
            Self::Storm => 1.3,
        }
    }

    /// Get all weather states.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Clear, Self::Cloudy, Self::Raining, Self::Storm]
    }
}

/// Configuration for weather transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    /// Minimum duration of a weather state in seconds.
    pub min_duration: f32,
    /// Maximum duration of a weather state in seconds.
    pub max_duration: f32,
    /// Probability of transitioning to each state (indexed by WeatherState).
    /// Each row represents the current state, each column the target state.
    pub transition_weights: [[f32; 4]; 4],
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            min_duration: 120.0, // 2 minutes minimum
            max_duration: 600.0, // 10 minutes maximum
            // Transition probabilities: [Clear, Cloudy, Raining, Storm]
            // From Clear: likely to stay clear or become cloudy
            // From Cloudy: can go to any state
            // From Raining: can clear up or intensify
            // From Storm: usually calms down
            transition_weights: [
                [0.5, 0.4, 0.08, 0.02], // From Clear
                [0.3, 0.3, 0.3, 0.1],   // From Cloudy
                [0.1, 0.3, 0.4, 0.2],   // From Raining
                [0.05, 0.15, 0.5, 0.3], // From Storm
            ],
        }
    }
}

impl WeatherConfig {
    /// Get the transition weights from a given state.
    #[must_use]
    pub fn weights_from(&self, state: WeatherState) -> &[f32; 4] {
        let idx = match state {
            WeatherState::Clear => 0,
            WeatherState::Cloudy => 1,
            WeatherState::Raining => 2,
            WeatherState::Storm => 3,
        };
        &self.transition_weights[idx]
    }
}

/// Weather effect that can be applied to entities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WeatherEffect {
    /// No special effect.
    None,
    /// Getting wet (reduces fire damage, increases cold damage).
    Wet,
    /// Buffeted by wind (knockback chance).
    Windblown,
    /// Struck by lightning (rare, high damage).
    LightningStruck,
}

/// A simple random number generator for weather transitions.
/// Uses a linear congruential generator for deterministic results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherRng {
    state: u64,
}

impl Default for WeatherRng {
    fn default() -> Self {
        Self::new(12345)
    }
}

impl WeatherRng {
    /// Create a new RNG with a seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate a random f32 in [0.0, 1.0).
    pub fn next_f32(&mut self) -> f32 {
        // LCG parameters (same as glibc)
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        // Extract upper bits for better randomness
        let bits = (self.state >> 16) as u32 & 0x7FFF;
        bits as f32 / 32768.0
    }

    /// Generate a random f32 in [min, max).
    pub fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Set the seed.
    pub fn set_seed(&mut self, seed: u64) {
        self.state = seed;
    }
}

/// Manages weather state and transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSystem {
    /// Current weather state.
    current_state: WeatherState,
    /// Previous weather state (for transitions).
    previous_state: WeatherState,
    /// Time remaining in current state (seconds).
    time_remaining: f32,
    /// Configuration for weather behavior.
    config: WeatherConfig,
    /// Random number generator for transitions.
    rng: WeatherRng,
    /// Transition progress (0.0 to 1.0 during transitions).
    transition_progress: f32,
    /// Duration of weather transition effect.
    transition_duration: f32,
    /// Total time in seconds since system started.
    total_time: f32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Default weather transition duration in seconds.
const DEFAULT_TRANSITION_DURATION: f32 = 5.0;

impl WeatherSystem {
    /// Create a new weather system with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_state: WeatherState::Clear,
            previous_state: WeatherState::Clear,
            time_remaining: 300.0, // Start with 5 minutes of clear weather
            config: WeatherConfig::default(),
            rng: WeatherRng::default(),
            transition_progress: 1.0, // Not transitioning
            transition_duration: DEFAULT_TRANSITION_DURATION,
            total_time: 0.0,
        }
    }

    /// Create a weather system with a custom seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        let mut system = Self::new();
        system.rng = WeatherRng::new(seed);
        system
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: WeatherConfig) -> Self {
        let mut system = Self::new();
        system.config = config;
        system
    }

    /// Get the current weather state.
    #[must_use]
    pub fn current(&self) -> WeatherState {
        self.current_state
    }

    /// Get the previous weather state (for transitions).
    #[must_use]
    pub fn previous(&self) -> WeatherState {
        self.previous_state
    }

    /// Check if currently transitioning between states.
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
        self.transition_progress < 1.0
    }

    /// Get transition progress (0.0 to 1.0).
    #[must_use]
    pub fn transition_progress(&self) -> f32 {
        self.transition_progress
    }

    /// Get time remaining in current weather state.
    #[must_use]
    pub fn time_remaining(&self) -> f32 {
        self.time_remaining
    }

    /// Get total elapsed time.
    #[must_use]
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Check if it's currently raining.
    #[must_use]
    pub fn is_raining(&self) -> bool {
        self.current_state.is_raining()
    }

    /// Check if there's a storm.
    #[must_use]
    pub fn is_stormy(&self) -> bool {
        self.current_state.is_stormy()
    }

    /// Get the current light modifier (interpolated during transitions).
    #[must_use]
    pub fn light_modifier(&self) -> f32 {
        if self.is_transitioning() {
            let prev = self.previous_state.light_modifier();
            let curr = self.current_state.light_modifier();
            lerp(prev, curr, self.transition_progress)
        } else {
            self.current_state.light_modifier()
        }
    }

    /// Get the current movement modifier.
    #[must_use]
    pub fn movement_modifier(&self) -> f32 {
        if self.is_transitioning() {
            let prev = self.previous_state.movement_modifier();
            let curr = self.current_state.movement_modifier();
            lerp(prev, curr, self.transition_progress)
        } else {
            self.current_state.movement_modifier()
        }
    }

    /// Get the current growth modifier.
    #[must_use]
    pub fn growth_modifier(&self) -> f32 {
        if self.is_transitioning() {
            let prev = self.previous_state.growth_modifier();
            let curr = self.current_state.growth_modifier();
            lerp(prev, curr, self.transition_progress)
        } else {
            self.current_state.growth_modifier()
        }
    }

    /// Force a specific weather state.
    pub fn set_weather(&mut self, state: WeatherState) {
        self.previous_state = self.current_state;
        self.current_state = state;
        self.transition_progress = 0.0;
        self.time_remaining = self
            .rng
            .next_range(self.config.min_duration, self.config.max_duration);
    }

    /// Force a specific weather state without transition.
    pub fn set_weather_immediate(&mut self, state: WeatherState) {
        self.previous_state = state;
        self.current_state = state;
        self.transition_progress = 1.0;
        self.time_remaining = self
            .rng
            .next_range(self.config.min_duration, self.config.max_duration);
    }

    /// Update the weather system.
    ///
    /// Returns `Some(new_state)` if weather changed, `None` otherwise.
    pub fn update(&mut self, dt: f32) -> Option<WeatherState> {
        self.total_time += dt;

        // Update transition
        if self.is_transitioning() {
            self.transition_progress += dt / self.transition_duration;
            if self.transition_progress >= 1.0 {
                self.transition_progress = 1.0;
            }
        }

        // Update timer
        self.time_remaining -= dt;
        if self.time_remaining <= 0.0 {
            // Time to transition to a new state
            let new_state = self.pick_next_state();
            if new_state != self.current_state {
                self.previous_state = self.current_state;
                self.current_state = new_state;
                self.transition_progress = 0.0;
            }
            self.time_remaining = self
                .rng
                .next_range(self.config.min_duration, self.config.max_duration);
            return Some(new_state);
        }

        None
    }

    /// Pick the next weather state based on transition probabilities.
    fn pick_next_state(&mut self) -> WeatherState {
        let weights = self.config.weights_from(self.current_state);
        let roll = self.rng.next_f32();

        let mut cumulative = 0.0;
        for (idx, &weight) in weights.iter().enumerate() {
            cumulative += weight;
            if roll < cumulative {
                return match idx {
                    0 => WeatherState::Clear,
                    1 => WeatherState::Cloudy,
                    2 => WeatherState::Raining,
                    _ => WeatherState::Storm,
                };
            }
        }

        // Fallback (shouldn't happen with proper weights)
        WeatherState::Clear
    }

    /// Get potential weather effects for an entity at a position.
    /// The `exposure` parameter (0.0 to 1.0) represents how exposed the entity is
    /// (0.0 = fully sheltered, 1.0 = fully exposed).
    #[must_use]
    pub fn get_effects(&mut self, exposure: f32) -> WeatherEffect {
        if exposure <= 0.0 {
            return WeatherEffect::None;
        }

        match self.current_state {
            WeatherState::Clear | WeatherState::Cloudy => WeatherEffect::None,
            WeatherState::Raining => {
                if exposure > 0.5 {
                    WeatherEffect::Wet
                } else {
                    WeatherEffect::None
                }
            },
            WeatherState::Storm => {
                let roll = self.rng.next_f32();
                // 0.1% chance of lightning per update when fully exposed
                if roll < 0.001 * exposure {
                    WeatherEffect::LightningStruck
                } else if roll < 0.05 * exposure {
                    WeatherEffect::Windblown
                } else if exposure > 0.3 {
                    WeatherEffect::Wet
                } else {
                    WeatherEffect::None
                }
            },
        }
    }

    /// Get reference to configuration.
    #[must_use]
    pub fn config(&self) -> &WeatherConfig {
        &self.config
    }

    /// Get mutable reference to configuration.
    pub fn config_mut(&mut self) -> &mut WeatherConfig {
        &mut self.config
    }
}

/// Linear interpolation helper.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Forecast information for upcoming weather.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherForecast {
    /// Current weather state.
    pub current: WeatherState,
    /// Time until next potential change.
    pub time_until_change: f32,
    /// Most likely next states (ordered by probability).
    pub likely_next: Vec<(WeatherState, f32)>,
}

impl WeatherSystem {
    /// Generate a forecast of upcoming weather.
    #[must_use]
    pub fn forecast(&self) -> WeatherForecast {
        let weights = self.config.weights_from(self.current_state);
        let mut likely_next: Vec<_> = WeatherState::all()
            .iter()
            .zip(weights.iter())
            .map(|(&state, &prob)| (state, prob))
            .collect();

        // Sort by probability descending
        likely_next.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        WeatherForecast {
            current: self.current_state,
            time_until_change: self.time_remaining,
            likely_next,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_state_properties() {
        assert!(WeatherState::Raining.is_raining());
        assert!(WeatherState::Storm.is_raining());
        assert!(!WeatherState::Clear.is_raining());
        assert!(!WeatherState::Cloudy.is_raining());

        assert!(WeatherState::Storm.is_stormy());
        assert!(!WeatherState::Raining.is_stormy());
    }

    #[test]
    fn test_weather_state_display_names() {
        assert_eq!(WeatherState::Clear.display_name(), "Clear");
        assert_eq!(WeatherState::Cloudy.display_name(), "Cloudy");
        assert_eq!(WeatherState::Raining.display_name(), "Raining");
        assert_eq!(WeatherState::Storm.display_name(), "Storm");
    }

    #[test]
    fn test_weather_modifiers() {
        assert!((WeatherState::Clear.light_modifier() - 1.0).abs() < 0.01);
        assert!(WeatherState::Storm.light_modifier() < WeatherState::Clear.light_modifier());

        assert!((WeatherState::Clear.movement_modifier() - 1.0).abs() < 0.01);
        assert!(WeatherState::Storm.movement_modifier() < WeatherState::Clear.movement_modifier());

        assert!(WeatherState::Raining.growth_modifier() > WeatherState::Clear.growth_modifier());
    }

    #[test]
    fn test_weather_rng() {
        let mut rng = WeatherRng::new(42);
        let val1 = rng.next_f32();
        let val2 = rng.next_f32();

        assert!(val1 >= 0.0 && val1 < 1.0);
        assert!(val2 >= 0.0 && val2 < 1.0);
        assert!((val1 - val2).abs() > 0.0001); // Different values

        // Same seed should give same sequence
        let mut rng2 = WeatherRng::new(42);
        assert!((rng2.next_f32() - val1).abs() < 0.0001);
    }

    #[test]
    fn test_weather_system_creation() {
        let system = WeatherSystem::new();
        assert_eq!(system.current(), WeatherState::Clear);
        assert!(!system.is_transitioning());
        assert!(system.time_remaining() > 0.0);
    }

    #[test]
    fn test_weather_system_set_weather() {
        let mut system = WeatherSystem::new();
        system.set_weather(WeatherState::Raining);

        assert_eq!(system.current(), WeatherState::Raining);
        assert!(system.is_transitioning());
        assert!(system.is_raining());
    }

    #[test]
    fn test_weather_system_set_immediate() {
        let mut system = WeatherSystem::new();
        system.set_weather_immediate(WeatherState::Storm);

        assert_eq!(system.current(), WeatherState::Storm);
        assert!(!system.is_transitioning());
        assert!(system.is_stormy());
    }

    #[test]
    fn test_weather_system_update() {
        let mut system = WeatherSystem::with_config(WeatherConfig {
            min_duration: 0.5,
            max_duration: 0.5,
            ..Default::default()
        });
        // Set a very short time remaining to trigger change quickly
        system.time_remaining = 0.5;

        // Update until weather changes
        let mut changed = false;
        for _ in 0..20 {
            if system.update(0.1).is_some() {
                changed = true;
                break;
            }
        }
        assert!(changed);
    }

    #[test]
    fn test_weather_transition_interpolation() {
        let mut system = WeatherSystem::new();
        system.set_weather(WeatherState::Storm);

        // During transition, modifiers should be interpolated
        let initial_light = system.light_modifier();
        system.update(2.5); // Half transition
        let mid_light = system.light_modifier();
        system.update(5.0); // Complete transition

        // Mid-transition should be between start and end
        assert!(initial_light >= mid_light);
        assert!(mid_light >= WeatherState::Storm.light_modifier());
    }

    #[test]
    fn test_weather_forecast() {
        let system = WeatherSystem::new();
        let forecast = system.forecast();

        assert_eq!(forecast.current, WeatherState::Clear);
        assert!(forecast.time_until_change > 0.0);
        assert_eq!(forecast.likely_next.len(), 4);

        // Probabilities should sum to approximately 1.0
        let total_prob: f32 = forecast.likely_next.iter().map(|(_, p)| p).sum();
        assert!((total_prob - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_weather_effects_clear() {
        let mut system = WeatherSystem::new();
        system.set_weather_immediate(WeatherState::Clear);
        assert_eq!(system.get_effects(1.0), WeatherEffect::None);
    }

    #[test]
    fn test_weather_effects_raining() {
        let mut system = WeatherSystem::new();
        system.set_weather_immediate(WeatherState::Raining);
        assert_eq!(system.get_effects(1.0), WeatherEffect::Wet);
        assert_eq!(system.get_effects(0.0), WeatherEffect::None);
    }

    #[test]
    fn test_weather_effects_sheltered() {
        let mut system = WeatherSystem::new();
        system.set_weather_immediate(WeatherState::Storm);
        // Fully sheltered should have no effects
        assert_eq!(system.get_effects(0.0), WeatherEffect::None);
    }

    #[test]
    fn test_weather_config_weights() {
        let config = WeatherConfig::default();

        // Each row should sum to 1.0
        for row in &config.transition_weights {
            let sum: f32 = row.iter().sum();
            assert!((sum - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_weather_all_states() {
        let states = WeatherState::all();
        assert_eq!(states.len(), 4);
        assert!(states.contains(&WeatherState::Clear));
        assert!(states.contains(&WeatherState::Cloudy));
        assert!(states.contains(&WeatherState::Raining));
        assert!(states.contains(&WeatherState::Storm));
    }

    #[test]
    fn test_weather_system_total_time() {
        let mut system = WeatherSystem::new();
        assert!((system.total_time() - 0.0).abs() < 0.01);

        system.update(1.0);
        assert!((system.total_time() - 1.0).abs() < 0.01);

        system.update(2.5);
        assert!((system.total_time() - 3.5).abs() < 0.01);
    }
}
