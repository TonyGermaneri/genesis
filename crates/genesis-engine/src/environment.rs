//! Environment systems for weather and time simulation.
//!
//! Provides game time progression and weather state management.

#![allow(dead_code)]

/// Game time manager for day/night cycles.
#[derive(Debug, Clone)]
pub struct GameTime {
    /// Current time of day (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
    time_of_day: f32,
    /// Speed multiplier for time progression (1.0 = real-time equivalent)
    time_scale: f32,
    /// Length of a full day cycle in real seconds
    day_length_seconds: f32,
    /// Whether time is paused
    paused: bool,
    /// Current day number
    day_count: u32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            time_of_day: 0.25, // Start at dawn (6 AM)
            time_scale: 1.0,
            day_length_seconds: 1200.0, // 20 minutes per day
            paused: false,
            day_count: 1,
        }
    }
}

impl GameTime {
    /// Creates a new game time starting at a specific time of day.
    #[must_use]
    pub fn new(start_time: f32) -> Self {
        Self {
            time_of_day: start_time.rem_euclid(1.0),
            ..Default::default()
        }
    }

    /// Updates the game time.
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds
    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        let time_delta = (dt * self.time_scale) / self.day_length_seconds;
        self.time_of_day += time_delta;

        if self.time_of_day >= 1.0 {
            self.time_of_day -= 1.0;
            self.day_count = self.day_count.saturating_add(1);
        }
    }

    /// Returns the current time of day (0.0-1.0).
    #[must_use]
    pub fn time_of_day(&self) -> f32 {
        self.time_of_day
    }

    /// Returns the current hour (0-23).
    #[must_use]
    pub fn hour(&self) -> u8 {
        (self.time_of_day * 24.0) as u8
    }

    /// Returns the current minute (0-59).
    #[must_use]
    pub fn minute(&self) -> u8 {
        ((self.time_of_day * 24.0 * 60.0) % 60.0) as u8
    }

    /// Returns a formatted time string (HH:MM).
    #[must_use]
    pub fn formatted_time(&self) -> String {
        format!("{:02}:{:02}", self.hour(), self.minute())
    }

    /// Returns the current day count.
    #[must_use]
    pub fn day_count(&self) -> u32 {
        self.day_count
    }

    /// Returns whether it's daytime (6 AM - 6 PM).
    #[must_use]
    pub fn is_daytime(&self) -> bool {
        self.time_of_day >= 0.25 && self.time_of_day < 0.75
    }

    /// Returns whether it's nighttime (6 PM - 6 AM).
    #[must_use]
    pub fn is_nighttime(&self) -> bool {
        !self.is_daytime()
    }

    /// Returns the sun intensity (0.0 at night, 1.0 at noon).
    #[must_use]
    pub fn sun_intensity(&self) -> f32 {
        // Dawn at 0.25, noon at 0.5, dusk at 0.75
        if self.time_of_day < 0.25 {
            0.0 // Night
        } else if self.time_of_day < 0.5 {
            // Dawn to noon: 0->1
            (self.time_of_day - 0.25) * 4.0
        } else if self.time_of_day < 0.75 {
            // Noon to dusk: 1->0
            1.0 - (self.time_of_day - 0.5) * 4.0
        } else {
            0.0 // Night
        }
    }

    /// Sets the time scale multiplier.
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Pauses time progression.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes time progression.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Returns whether time is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Sets the time of day directly.
    pub fn set_time_of_day(&mut self, time: f32) {
        self.time_of_day = time.rem_euclid(1.0);
    }
}

/// Weather type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherType {
    /// Clear skies
    #[default]
    Clear,
    /// Light clouds
    Cloudy,
    /// Rain
    Rain,
    /// Heavy rain/storm
    Storm,
    /// Snow
    Snow,
    /// Fog
    Fog,
}

impl WeatherType {
    /// Returns the rain intensity (0.0-1.0).
    #[must_use]
    pub fn rain_intensity(self) -> f32 {
        match self {
            WeatherType::Clear | WeatherType::Cloudy | WeatherType::Snow | WeatherType::Fog => 0.0,
            WeatherType::Rain => 0.5,
            WeatherType::Storm => 1.0,
        }
    }

    /// Returns whether it's raining.
    #[must_use]
    pub fn is_raining(self) -> bool {
        matches!(self, WeatherType::Rain | WeatherType::Storm)
    }

    /// Returns the ambient light modifier (0.0-1.0, lower = darker).
    #[must_use]
    pub fn ambient_modifier(self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 0.8,
            WeatherType::Rain => 0.6,
            WeatherType::Storm => 0.4,
            WeatherType::Snow => 0.7,
            WeatherType::Fog => 0.5,
        }
    }

    /// Returns a display name for the weather.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            WeatherType::Clear => "Clear",
            WeatherType::Cloudy => "Cloudy",
            WeatherType::Rain => "Rain",
            WeatherType::Storm => "Storm",
            WeatherType::Snow => "Snow",
            WeatherType::Fog => "Fog",
        }
    }
}

/// Weather system managing current and transitioning weather.
#[derive(Debug, Clone)]
pub struct WeatherSystem {
    /// Current weather type
    current_weather: WeatherType,
    /// Target weather type (for transitions)
    target_weather: WeatherType,
    /// Transition progress (0.0-1.0)
    transition_progress: f32,
    /// Duration of weather transitions
    transition_duration: f32,
    /// Time until next weather change
    time_until_change: f32,
    /// Minimum time between weather changes
    min_change_interval: f32,
    /// Maximum time between weather changes
    max_change_interval: f32,
    /// Random seed for weather changes
    seed: u64,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            target_weather: WeatherType::Clear,
            transition_progress: 1.0,
            transition_duration: 30.0,  // 30 seconds to transition
            time_until_change: 300.0,   // 5 minutes until first change
            min_change_interval: 180.0, // 3 minutes minimum
            max_change_interval: 600.0, // 10 minutes maximum
            seed: 12345,
        }
    }
}

impl WeatherSystem {
    /// Creates a new weather system.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a weather system with a specific starting weather.
    #[must_use]
    pub fn with_weather(weather: WeatherType) -> Self {
        Self {
            current_weather: weather,
            target_weather: weather,
            ..Default::default()
        }
    }

    /// Updates the weather system.
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds
    pub fn update(&mut self, dt: f32) {
        // Handle transition
        if self.transition_progress < 1.0 {
            self.transition_progress =
                (self.transition_progress + dt / self.transition_duration).min(1.0);

            if self.transition_progress >= 1.0 {
                self.current_weather = self.target_weather;
            }
        }

        // Count down to next weather change
        self.time_until_change -= dt;
        if self.time_until_change <= 0.0 {
            self.schedule_weather_change();
        }
    }

    /// Schedules a new random weather change.
    fn schedule_weather_change(&mut self) {
        // Simple pseudo-random weather selection
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let rand = (self.seed >> 33) as f32 / u32::MAX as f32;

        self.target_weather = match (rand * 6.0) as u32 {
            0 => WeatherType::Clear,
            1 => WeatherType::Cloudy,
            2 => WeatherType::Rain,
            3 => WeatherType::Storm,
            4 => WeatherType::Snow,
            _ => WeatherType::Fog,
        };

        self.transition_progress = 0.0;

        // Schedule next change
        let range = self.max_change_interval - self.min_change_interval;
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let rand2 = (self.seed >> 33) as f32 / u32::MAX as f32;
        self.time_until_change = self.min_change_interval + rand2 * range;
    }

    /// Returns the current weather type.
    #[must_use]
    pub fn current_weather(&self) -> WeatherType {
        self.current_weather
    }

    /// Returns the target weather type (during transitions).
    #[must_use]
    pub fn target_weather(&self) -> WeatherType {
        self.target_weather
    }

    /// Returns the rain intensity (0.0-1.0), accounting for transitions.
    #[must_use]
    pub fn rain_intensity(&self) -> f32 {
        let current = self.current_weather.rain_intensity();
        let target = self.target_weather.rain_intensity();
        current + (target - current) * self.transition_progress
    }

    /// Returns whether it's currently raining.
    #[must_use]
    pub fn is_raining(&self) -> bool {
        self.rain_intensity() > 0.1
    }

    /// Returns the ambient light modifier (0.0-1.0).
    #[must_use]
    pub fn ambient_modifier(&self) -> f32 {
        let current = self.current_weather.ambient_modifier();
        let target = self.target_weather.ambient_modifier();
        current + (target - current) * self.transition_progress
    }

    /// Forces a specific weather type immediately.
    pub fn set_weather(&mut self, weather: WeatherType) {
        self.current_weather = weather;
        self.target_weather = weather;
        self.transition_progress = 1.0;
    }

    /// Starts a transition to a new weather type.
    pub fn transition_to(&mut self, weather: WeatherType) {
        if self.current_weather != weather {
            self.target_weather = weather;
            self.transition_progress = 0.0;
        }
    }
}

/// Environment state combining time and weather.
#[derive(Debug, Clone)]
pub struct EnvironmentState {
    /// Game time
    pub time: GameTime,
    /// Weather system
    pub weather: WeatherSystem,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentState {
    /// Creates a new environment state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            time: GameTime::default(),
            weather: WeatherSystem::new(),
        }
    }

    /// Updates both time and weather systems.
    pub fn update(&mut self, dt: f32) {
        self.time.update(dt);
        self.weather.update(dt);
    }

    /// Returns the effective ambient light level (0.0-1.0).
    ///
    /// Combines time of day lighting with weather modifiers.
    #[must_use]
    pub fn ambient_light(&self) -> f32 {
        let base = self.time.sun_intensity() * 0.8 + 0.2; // 0.2-1.0 range
        base * self.weather.ambient_modifier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_time_default() {
        let time = GameTime::default();
        assert!((time.time_of_day() - 0.25).abs() < 0.001); // Dawn
        assert_eq!(time.hour(), 6);
        assert!(time.is_daytime());
    }

    #[test]
    fn test_game_time_update() {
        let mut time = GameTime::default();
        // With 1200s day length and 1s dt, time advances by 1/1200
        time.update(1.0);
        assert!(time.time_of_day() > 0.25);
    }

    #[test]
    fn test_game_time_day_rollover() {
        let mut time = GameTime::new(0.99);
        time.day_length_seconds = 100.0;
        time.update(2.0); // Should roll over
        assert_eq!(time.day_count(), 2);
        assert!(time.time_of_day() < 0.1);
    }

    #[test]
    fn test_game_time_formatted() {
        let time = GameTime::new(0.5); // Noon
        assert_eq!(time.formatted_time(), "12:00");
    }

    #[test]
    fn test_weather_type_rain() {
        assert!(WeatherType::Rain.is_raining());
        assert!(WeatherType::Storm.is_raining());
        assert!(!WeatherType::Clear.is_raining());
    }

    #[test]
    fn test_weather_system_transition() {
        let mut weather = WeatherSystem::with_weather(WeatherType::Clear);
        weather.transition_to(WeatherType::Rain);
        assert_eq!(weather.target_weather(), WeatherType::Rain);
        assert!(weather.transition_progress < 1.0);
    }

    #[test]
    fn test_environment_state_ambient() {
        let env = EnvironmentState::new();
        let ambient = env.ambient_light();
        assert!(ambient >= 0.0 && ambient <= 1.0);
    }
}
