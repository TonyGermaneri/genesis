//! Time and day/night cycle system.
//!
//! This module provides game time management:
//! - Day/night cycle with configurable length
//! - Time-based events (dawn, dusk, midnight, noon)
//! - Light level calculations based on time

use serde::{Deserialize, Serialize};

/// Represents game time with day/night cycle.
///
/// Time is represented as:
/// - `time_of_day`: Normalized value from 0.0 (midnight) to 1.0 (next midnight)
/// - `day_count`: Number of complete days elapsed
///
/// By default, 1 real second = 1 game minute (configurable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTime {
    /// Current time of day (0.0 = midnight, 0.5 = noon, 1.0 = next midnight).
    time_of_day: f32,
    /// Number of complete days that have passed.
    day_count: u32,
    /// Real seconds per game minute.
    real_seconds_per_game_minute: f32,
    /// Total elapsed game time in game minutes.
    total_game_minutes: f64,
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

/// Default starting hour (6:00 AM).
const DEFAULT_START_HOUR: u32 = 6;
/// Default real seconds per game minute.
const DEFAULT_REAL_SECONDS_PER_MINUTE: f32 = 1.0;
/// Minutes in a game day.
const MINUTES_PER_DAY: f32 = 24.0 * 60.0;
/// Minutes in a game hour.
const MINUTES_PER_HOUR: f32 = 60.0;

impl GameTime {
    /// Create a new game time starting at 6:00 AM (dawn).
    #[must_use]
    pub fn new() -> Self {
        Self {
            time_of_day: DEFAULT_START_HOUR as f32 * MINUTES_PER_HOUR / MINUTES_PER_DAY,
            day_count: 0,
            real_seconds_per_game_minute: DEFAULT_REAL_SECONDS_PER_MINUTE,
            total_game_minutes: DEFAULT_START_HOUR as f64 * MINUTES_PER_HOUR as f64,
        }
    }

    /// Create with a specific starting time.
    ///
    /// # Arguments
    /// * `hour` - Starting hour (0-23)
    /// * `minute` - Starting minute (0-59)
    /// * `day` - Starting day number
    #[must_use]
    pub fn with_time(hour: u32, minute: u32, day: u32) -> Self {
        let hour = hour.min(23);
        let minute = minute.min(59);
        let minutes_today = hour as f32 * MINUTES_PER_HOUR + minute as f32;

        Self {
            time_of_day: minutes_today / MINUTES_PER_DAY,
            day_count: day,
            real_seconds_per_game_minute: DEFAULT_REAL_SECONDS_PER_MINUTE,
            total_game_minutes: day as f64 * MINUTES_PER_DAY as f64 + minutes_today as f64,
        }
    }

    /// Set the time scale (real seconds per game minute).
    ///
    /// # Arguments
    /// * `seconds_per_minute` - Real seconds per game minute (1.0 = default)
    pub fn set_time_scale(&mut self, seconds_per_minute: f32) {
        self.real_seconds_per_game_minute = seconds_per_minute.max(0.01);
    }

    /// Get the time scale.
    #[must_use]
    pub fn time_scale(&self) -> f32 {
        self.real_seconds_per_game_minute
    }

    /// Get the current time of day (0.0 to 1.0).
    #[must_use]
    pub fn time_of_day(&self) -> f32 {
        self.time_of_day
    }

    /// Get the current day count.
    #[must_use]
    pub fn day_count(&self) -> u32 {
        self.day_count
    }

    /// Get the current hour (0-23).
    #[must_use]
    pub fn hour(&self) -> u32 {
        let minutes_today = self.time_of_day * MINUTES_PER_DAY;
        (minutes_today / MINUTES_PER_HOUR) as u32 % 24
    }

    /// Get the current minute (0-59).
    #[must_use]
    pub fn minute(&self) -> u32 {
        let minutes_today = self.time_of_day * MINUTES_PER_DAY;
        (minutes_today % MINUTES_PER_HOUR).round() as u32 % 60
    }

    /// Get the total elapsed game minutes.
    #[must_use]
    pub fn total_game_minutes(&self) -> f64 {
        self.total_game_minutes
    }

    /// Get the total elapsed game hours.
    #[must_use]
    pub fn total_game_hours(&self) -> f64 {
        self.total_game_minutes / MINUTES_PER_HOUR as f64
    }

    /// Get the total elapsed game days.
    #[must_use]
    pub fn total_game_days(&self) -> f64 {
        self.total_game_minutes / MINUTES_PER_DAY as f64
    }

    /// Check if it's daytime (6:00 to 18:00).
    #[must_use]
    pub fn is_day(&self) -> bool {
        let hour = self.hour();
        (6..18).contains(&hour)
    }

    /// Check if it's nighttime (18:00 to 6:00).
    #[must_use]
    pub fn is_night(&self) -> bool {
        !self.is_day()
    }

    /// Check if it's dawn (5:00 to 7:00).
    #[must_use]
    pub fn is_dawn(&self) -> bool {
        let hour = self.hour();
        (5..7).contains(&hour)
    }

    /// Check if it's dusk (17:00 to 19:00).
    #[must_use]
    pub fn is_dusk(&self) -> bool {
        let hour = self.hour();
        (17..19).contains(&hour)
    }

    /// Check if it's noon (11:00 to 13:00).
    #[must_use]
    pub fn is_noon(&self) -> bool {
        let hour = self.hour();
        (11..13).contains(&hour)
    }

    /// Check if it's midnight (23:00 to 1:00).
    #[must_use]
    pub fn is_midnight(&self) -> bool {
        let hour = self.hour();
        !(1..23).contains(&hour)
    }

    /// Get the current period of day.
    #[must_use]
    pub fn period(&self) -> TimePeriod {
        let hour = self.hour();
        match hour {
            5..=6 => TimePeriod::Dawn,
            7..=10 => TimePeriod::Morning,
            11..=13 => TimePeriod::Noon,
            14..=16 => TimePeriod::Afternoon,
            17..=18 => TimePeriod::Dusk,
            _ => TimePeriod::Night, // 0-4 and 19-23
        }
    }

    /// Get the ambient light level (0.0 = pitch black, 1.0 = full daylight).
    ///
    /// Light levels:
    /// - Night (0-4, 20-23): 0.1 - 0.2
    /// - Dawn/Dusk (5-6, 17-19): 0.3 - 0.7
    /// - Day (7-16): 0.8 - 1.0
    /// - Noon (11-13): 1.0
    #[must_use]
    pub fn light_level(&self) -> f32 {
        // Use sine wave for smooth transitions
        // Peak at noon (0.5), minimum at midnight (0.0 and 1.0)
        let angle = self.time_of_day * std::f32::consts::PI * 2.0 - std::f32::consts::FRAC_PI_2;
        let base_light = (angle.sin() + 1.0) / 2.0;

        // Clamp to reasonable range (even at night there's some moonlight)
        0.1 + base_light * 0.9
    }

    /// Get a formatted time string (HH:MM).
    #[must_use]
    pub fn format_time(&self) -> String {
        format!("{:02}:{:02}", self.hour(), self.minute())
    }

    /// Get a formatted time string with period (HH:MM AM/PM).
    #[must_use]
    pub fn format_time_12h(&self) -> String {
        let hour = self.hour();
        let (hour_12, period) = if hour == 0 {
            (12, "AM")
        } else if hour < 12 {
            (hour, "AM")
        } else if hour == 12 {
            (12, "PM")
        } else {
            (hour - 12, "PM")
        };
        format!("{:02}:{:02} {}", hour_12, self.minute(), period)
    }

    /// Update game time based on real elapsed time.
    ///
    /// Returns `Some(event)` if a time event occurred.
    pub fn update(&mut self, dt_real_seconds: f32) -> Option<TimeEvent> {
        // Convert real time to game minutes
        let game_minutes = dt_real_seconds / self.real_seconds_per_game_minute;
        self.total_game_minutes += game_minutes as f64;

        // Update time of day
        let old_time = self.time_of_day;
        let old_hour = self.hour();
        self.time_of_day += game_minutes / MINUTES_PER_DAY;

        // Handle day rollover
        let mut event = None;
        while self.time_of_day >= 1.0 {
            self.time_of_day -= 1.0;
            self.day_count += 1;
            event = Some(TimeEvent::NewDay(self.day_count));
        }

        // Check for hour change events
        let new_hour = self.hour();
        if old_hour != new_hour && event.is_none() {
            event = match new_hour {
                6 => Some(TimeEvent::Dawn),
                12 => Some(TimeEvent::Noon),
                18 => Some(TimeEvent::Dusk),
                0 => Some(TimeEvent::Midnight),
                _ => Some(TimeEvent::HourChanged(new_hour)),
            };
        }

        // Check for day/night transition
        if event.is_none() {
            let was_day = (0.25..0.75).contains(&old_time);
            let is_day = (0.25..0.75).contains(&self.time_of_day);
            if !was_day && is_day {
                event = Some(TimeEvent::DayStarted);
            } else if was_day && !is_day {
                event = Some(TimeEvent::NightStarted);
            }
        }

        event
    }

    /// Set the time to a specific hour and minute.
    pub fn set_time(&mut self, hour: u32, minute: u32) {
        let hour = hour.min(23);
        let minute = minute.min(59);
        let minutes_today = hour as f32 * MINUTES_PER_HOUR + minute as f32;
        self.time_of_day = minutes_today / MINUTES_PER_DAY;
    }

    /// Advance time by a specific number of game hours.
    pub fn advance_hours(&mut self, hours: f32) {
        let minutes = hours * MINUTES_PER_HOUR;
        self.total_game_minutes += minutes as f64;
        self.time_of_day += minutes / MINUTES_PER_DAY;

        while self.time_of_day >= 1.0 {
            self.time_of_day -= 1.0;
            self.day_count += 1;
        }
    }

    /// Skip to the next occurrence of a specific hour.
    pub fn skip_to_hour(&mut self, target_hour: u32) {
        let target_hour = target_hour.min(23);
        let current_hour = self.hour();

        let hours_to_advance = if target_hour > current_hour {
            target_hour - current_hour
        } else {
            24 - current_hour + target_hour
        };

        self.advance_hours(hours_to_advance as f32);
        self.set_time(target_hour, 0);
    }
}

/// Period of the day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimePeriod {
    /// Early morning (5-6)
    Dawn,
    /// Morning (7-10)
    Morning,
    /// Midday (11-13)
    Noon,
    /// Afternoon (14-16)
    Afternoon,
    /// Evening (17-18)
    Dusk,
    /// Night (19-4)
    Night,
}

impl TimePeriod {
    /// Get the display name of this period.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Dawn => "Dawn",
            Self::Morning => "Morning",
            Self::Noon => "Noon",
            Self::Afternoon => "Afternoon",
            Self::Dusk => "Dusk",
            Self::Night => "Night",
        }
    }

    /// Check if this period is considered daytime.
    #[must_use]
    pub fn is_daytime(self) -> bool {
        matches!(
            self,
            Self::Dawn | Self::Morning | Self::Noon | Self::Afternoon | Self::Dusk
        )
    }
}

/// Events that can occur during time updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeEvent {
    /// A new day has started.
    NewDay(u32),
    /// It's now dawn (6:00).
    Dawn,
    /// It's now noon (12:00).
    Noon,
    /// It's now dusk (18:00).
    Dusk,
    /// It's now midnight (0:00).
    Midnight,
    /// Day has started (transition from night).
    DayStarted,
    /// Night has started (transition from day).
    NightStarted,
    /// Hour has changed.
    HourChanged(u32),
}

impl TimeEvent {
    /// Get a description of this event.
    #[must_use]
    pub fn description(self) -> String {
        match self {
            Self::NewDay(day) => format!("Day {day} has begun"),
            Self::Dawn => "The sun rises".to_string(),
            Self::Noon => "It's high noon".to_string(),
            Self::Dusk => "The sun sets".to_string(),
            Self::Midnight => "Midnight".to_string(),
            Self::DayStarted => "Daytime".to_string(),
            Self::NightStarted => "Nighttime".to_string(),
            Self::HourChanged(h) => format!("It's {h}:00"),
        }
    }
}

/// Configuration for time-based effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEffects {
    /// Temperature modifier by time period.
    pub temperature_modifiers: [(TimePeriod, f32); 6],
    /// Enemy spawn rate modifier by time period (1.0 = normal).
    pub enemy_spawn_modifiers: [(TimePeriod, f32); 6],
    /// NPC activity level by time period (1.0 = active).
    pub npc_activity_modifiers: [(TimePeriod, f32); 6],
}

impl Default for TimeEffects {
    fn default() -> Self {
        Self {
            temperature_modifiers: [
                (TimePeriod::Dawn, 0.7),
                (TimePeriod::Morning, 0.85),
                (TimePeriod::Noon, 1.0),
                (TimePeriod::Afternoon, 0.95),
                (TimePeriod::Dusk, 0.8),
                (TimePeriod::Night, 0.6),
            ],
            enemy_spawn_modifiers: [
                (TimePeriod::Dawn, 0.5),
                (TimePeriod::Morning, 0.3),
                (TimePeriod::Noon, 0.2),
                (TimePeriod::Afternoon, 0.3),
                (TimePeriod::Dusk, 0.7),
                (TimePeriod::Night, 1.5),
            ],
            npc_activity_modifiers: [
                (TimePeriod::Dawn, 0.3),
                (TimePeriod::Morning, 0.9),
                (TimePeriod::Noon, 1.0),
                (TimePeriod::Afternoon, 0.9),
                (TimePeriod::Dusk, 0.5),
                (TimePeriod::Night, 0.1),
            ],
        }
    }
}

impl TimeEffects {
    /// Get the temperature modifier for a time period.
    #[must_use]
    pub fn temperature_modifier(&self, period: TimePeriod) -> f32 {
        self.temperature_modifiers
            .iter()
            .find(|(p, _)| *p == period)
            .map_or(1.0, |(_, m)| *m)
    }

    /// Get the enemy spawn modifier for a time period.
    #[must_use]
    pub fn enemy_spawn_modifier(&self, period: TimePeriod) -> f32 {
        self.enemy_spawn_modifiers
            .iter()
            .find(|(p, _)| *p == period)
            .map_or(1.0, |(_, m)| *m)
    }

    /// Get the NPC activity modifier for a time period.
    #[must_use]
    pub fn npc_activity_modifier(&self, period: TimePeriod) -> f32 {
        self.npc_activity_modifiers
            .iter()
            .find(|(p, _)| *p == period)
            .map_or(1.0, |(_, m)| *m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_time_creation() {
        let time = GameTime::new();
        assert_eq!(time.hour(), 6);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.day_count(), 0);
    }

    #[test]
    fn test_game_time_with_time() {
        let time = GameTime::with_time(14, 30, 5);
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.day_count(), 5);
    }

    #[test]
    fn test_time_of_day() {
        let time = GameTime::with_time(12, 0, 0);
        assert!((time.time_of_day() - 0.5).abs() < 0.01);

        let midnight = GameTime::with_time(0, 0, 0);
        assert!(midnight.time_of_day() < 0.01);
    }

    #[test]
    fn test_is_day_night() {
        let day_time = GameTime::with_time(12, 0, 0);
        assert!(day_time.is_day());
        assert!(!day_time.is_night());

        let night_time = GameTime::with_time(22, 0, 0);
        assert!(!night_time.is_day());
        assert!(night_time.is_night());
    }

    #[test]
    fn test_is_dawn_dusk() {
        let dawn = GameTime::with_time(6, 0, 0);
        assert!(dawn.is_dawn());
        assert!(!dawn.is_dusk());

        let dusk = GameTime::with_time(18, 0, 0);
        assert!(!dusk.is_dawn());
        assert!(dusk.is_dusk());
    }

    #[test]
    fn test_is_noon_midnight() {
        let noon = GameTime::with_time(12, 0, 0);
        assert!(noon.is_noon());
        assert!(!noon.is_midnight());

        let midnight = GameTime::with_time(0, 0, 0);
        assert!(!midnight.is_noon());
        assert!(midnight.is_midnight());
    }

    #[test]
    fn test_time_period() {
        assert_eq!(GameTime::with_time(3, 0, 0).period(), TimePeriod::Night);
        assert_eq!(GameTime::with_time(6, 0, 0).period(), TimePeriod::Dawn);
        assert_eq!(GameTime::with_time(9, 0, 0).period(), TimePeriod::Morning);
        assert_eq!(GameTime::with_time(12, 0, 0).period(), TimePeriod::Noon);
        assert_eq!(
            GameTime::with_time(15, 0, 0).period(),
            TimePeriod::Afternoon
        );
        assert_eq!(GameTime::with_time(18, 0, 0).period(), TimePeriod::Dusk);
        assert_eq!(GameTime::with_time(21, 0, 0).period(), TimePeriod::Night);
    }

    #[test]
    fn test_light_level() {
        let noon = GameTime::with_time(12, 0, 0);
        let midnight = GameTime::with_time(0, 0, 0);

        // Noon should have higher light than midnight
        assert!(noon.light_level() > midnight.light_level());
        // All light levels should be within valid range
        assert!(noon.light_level() >= 0.0 && noon.light_level() <= 1.0);
        assert!(midnight.light_level() >= 0.0 && midnight.light_level() <= 1.0);
    }

    #[test]
    fn test_format_time() {
        let time = GameTime::with_time(14, 30, 0);
        assert_eq!(time.format_time(), "14:30");

        let early = GameTime::with_time(6, 5, 0);
        assert_eq!(early.format_time(), "06:05");
    }

    #[test]
    fn test_format_time_12h() {
        let noon = GameTime::with_time(12, 0, 0);
        assert_eq!(noon.format_time_12h(), "12:00 PM");

        let midnight = GameTime::with_time(0, 0, 0);
        assert_eq!(midnight.format_time_12h(), "12:00 AM");

        let afternoon = GameTime::with_time(14, 30, 0);
        assert_eq!(afternoon.format_time_12h(), "02:30 PM");

        let morning = GameTime::with_time(9, 15, 0);
        assert_eq!(morning.format_time_12h(), "09:15 AM");
    }

    #[test]
    fn test_update_basic() {
        let mut time = GameTime::new();
        time.update(60.0); // 60 real seconds = 60 game minutes = 1 hour

        assert_eq!(time.hour(), 7); // Started at 6, now 7
    }

    #[test]
    fn test_update_day_rollover() {
        let mut time = GameTime::with_time(23, 30, 0);
        let event = time.update(60.0); // 1 hour should cross midnight

        assert_eq!(time.day_count(), 1);
        assert!(matches!(event, Some(TimeEvent::NewDay(1))));
    }

    #[test]
    fn test_update_events() {
        let mut time = GameTime::with_time(5, 59, 0);
        time.update(60.0); // Should trigger dawn

        // Note: depends on exact timing, just check hour changed
        assert_eq!(time.hour(), 6);
    }

    #[test]
    fn test_set_time() {
        let mut time = GameTime::new();
        time.set_time(15, 45);

        assert_eq!(time.hour(), 15);
        assert_eq!(time.minute(), 45);
    }

    #[test]
    fn test_advance_hours() {
        let mut time = GameTime::with_time(10, 0, 0);
        time.advance_hours(5.0);

        assert_eq!(time.hour(), 15);
        assert_eq!(time.day_count(), 0);

        time.advance_hours(15.0); // Should cross midnight
        assert_eq!(time.day_count(), 1);
    }

    #[test]
    fn test_skip_to_hour() {
        let mut time = GameTime::with_time(10, 30, 0);
        time.skip_to_hour(15);

        assert_eq!(time.hour(), 15);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.day_count(), 0);

        time.skip_to_hour(8); // Next day
        assert_eq!(time.hour(), 8);
        assert_eq!(time.day_count(), 1);
    }

    #[test]
    fn test_time_scale() {
        let mut time = GameTime::new();
        assert!((time.time_scale() - 1.0).abs() < 0.01);

        time.set_time_scale(2.0); // 2 real seconds per game minute
        assert!((time.time_scale() - 2.0).abs() < 0.01);

        let initial_hour = time.hour();
        time.update(120.0); // 120 real seconds = 60 game minutes with 2x scale
        assert_eq!(time.hour(), initial_hour + 1);
    }

    #[test]
    fn test_total_game_time() {
        let time = GameTime::with_time(12, 30, 2);

        // Day 2, 12:30 = 2 * 24 * 60 + 12 * 60 + 30 = 2880 + 720 + 30 = 3630 minutes
        assert!((time.total_game_minutes() - 3630.0).abs() < 1.0);
        assert!((time.total_game_hours() - 60.5).abs() < 0.1);
        assert!((time.total_game_days() - 2.52).abs() < 0.1);
    }

    #[test]
    fn test_time_period_display() {
        assert_eq!(TimePeriod::Dawn.display_name(), "Dawn");
        assert_eq!(TimePeriod::Night.display_name(), "Night");
        assert!(TimePeriod::Morning.is_daytime());
        assert!(!TimePeriod::Night.is_daytime());
    }

    #[test]
    fn test_time_event_description() {
        assert!(!TimeEvent::NewDay(1).description().is_empty());
        assert!(!TimeEvent::Dawn.description().is_empty());
        assert!(!TimeEvent::HourChanged(15).description().is_empty());
    }

    #[test]
    fn test_time_effects() {
        let effects = TimeEffects::default();

        // Night should have higher enemy spawn rate
        assert!(
            effects.enemy_spawn_modifier(TimePeriod::Night)
                > effects.enemy_spawn_modifier(TimePeriod::Noon)
        );

        // Noon should have higher NPC activity
        assert!(
            effects.npc_activity_modifier(TimePeriod::Noon)
                > effects.npc_activity_modifier(TimePeriod::Night)
        );

        // Noon should be warmest
        assert!(
            effects.temperature_modifier(TimePeriod::Noon)
                >= effects.temperature_modifier(TimePeriod::Night)
        );
    }
}
