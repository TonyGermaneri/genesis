//! World state serialization.
//!
//! This module provides world state saving:
//! - Day/night cycle and time
//! - Weather state and forecast
//! - Moon phase
//! - Season and seasonal events
//! - Environment timers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// G-55: Time Save
// ============================================================================

/// Day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    /// First day.
    Sunday,
    /// Second day.
    Monday,
    /// Third day.
    Tuesday,
    /// Fourth day.
    Wednesday,
    /// Fifth day.
    Thursday,
    /// Sixth day.
    Friday,
    /// Seventh day.
    Saturday,
}

impl DayOfWeek {
    /// Get day from index (0-6).
    #[must_use]
    pub fn from_index(index: u32) -> Self {
        match index % 7 {
            0 => Self::Sunday,
            1 => Self::Monday,
            2 => Self::Tuesday,
            3 => Self::Wednesday,
            4 => Self::Thursday,
            5 => Self::Friday,
            _ => Self::Saturday,
        }
    }

    /// Get day index (0-6).
    #[must_use]
    pub fn index(self) -> u32 {
        match self {
            Self::Sunday => 0,
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
        }
    }

    /// Get next day.
    #[must_use]
    pub fn next(self) -> Self {
        Self::from_index(self.index() + 1)
    }

    /// Get previous day.
    #[must_use]
    pub fn previous(self) -> Self {
        Self::from_index(self.index() + 6) // +6 instead of -1 to avoid underflow
    }
}

impl Default for DayOfWeek {
    fn default() -> Self {
        Self::Monday
    }
}

/// Time of day period for save data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SaveTimePeriod {
    /// Early morning (5:00 - 8:00).
    Dawn,
    /// Morning (8:00 - 12:00).
    Morning,
    /// Afternoon (12:00 - 17:00).
    Afternoon,
    /// Evening (17:00 - 20:00).
    Dusk,
    /// Night (20:00 - 5:00).
    Night,
}

impl SaveTimePeriod {
    /// Get period from hour.
    #[must_use]
    pub fn from_hour(hour: u32) -> Self {
        match hour % 24 {
            5..=7 => Self::Dawn,
            8..=11 => Self::Morning,
            12..=16 => Self::Afternoon,
            17..=19 => Self::Dusk,
            _ => Self::Night,
        }
    }

    /// Check if daytime.
    #[must_use]
    pub fn is_day(self) -> bool {
        matches!(self, Self::Dawn | Self::Morning | Self::Afternoon)
    }

    /// Check if nighttime.
    #[must_use]
    pub fn is_night(self) -> bool {
        matches!(self, Self::Dusk | Self::Night)
    }

    /// Get light level (0.0-1.0).
    #[must_use]
    pub fn light_level(self) -> f32 {
        match self {
            Self::Dawn => 0.5,
            Self::Morning => 0.9,
            Self::Afternoon => 1.0,
            Self::Dusk => 0.6,
            Self::Night => 0.2,
        }
    }
}

/// World time save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldTimeSave {
    /// Current day number (since world creation).
    pub day: u32,
    /// Current hour (0-23).
    pub hour: u32,
    /// Current minute (0-59).
    pub minute: u32,
    /// Current second (0-59).
    pub second: u32,
    /// Fractional seconds for smooth transitions.
    pub fraction: f32,
    /// Total elapsed game seconds.
    pub total_seconds: f64,
    /// Time scale (1.0 = normal speed).
    pub time_scale: f32,
    /// Whether time is paused.
    pub paused: bool,
}

impl Default for WorldTimeSave {
    fn default() -> Self {
        Self {
            day: 1,
            hour: 8,
            minute: 0,
            second: 0,
            fraction: 0.0,
            total_seconds: 0.0,
            time_scale: 1.0,
            paused: false,
        }
    }
}

impl WorldTimeSave {
    /// Create new world time.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set specific time.
    #[must_use]
    pub fn with_time(mut self, day: u32, hour: u32, minute: u32) -> Self {
        self.day = day;
        self.hour = hour.min(23);
        self.minute = minute.min(59);
        self.second = 0;
        self.fraction = 0.0;
        self
    }

    /// Set time scale.
    #[must_use]
    pub fn with_time_scale(mut self, scale: f32) -> Self {
        self.time_scale = scale.max(0.0);
        self
    }

    /// Get current time period.
    #[must_use]
    pub fn time_period(&self) -> SaveTimePeriod {
        SaveTimePeriod::from_hour(self.hour)
    }

    /// Get day of week.
    #[must_use]
    pub fn day_of_week(&self) -> DayOfWeek {
        DayOfWeek::from_index(self.day)
    }

    /// Check if daytime.
    #[must_use]
    pub fn is_day(&self) -> bool {
        self.time_period().is_day()
    }

    /// Check if nighttime.
    #[must_use]
    pub fn is_night(&self) -> bool {
        self.time_period().is_night()
    }

    /// Get current light level.
    #[must_use]
    pub fn light_level(&self) -> f32 {
        self.time_period().light_level()
    }

    /// Get formatted time string (HH:MM).
    #[must_use]
    pub fn formatted_time(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    /// Get formatted date string (Day X).
    #[must_use]
    pub fn formatted_date(&self) -> String {
        format!("Day {}, {:?}", self.day, self.day_of_week())
    }

    /// Advance time by delta (in game seconds).
    pub fn advance(&mut self, delta_seconds: f64) {
        if self.paused {
            return;
        }

        let scaled_delta = delta_seconds * self.time_scale as f64;
        self.total_seconds += scaled_delta;

        self.fraction += scaled_delta as f32;
        while self.fraction >= 1.0 {
            self.fraction -= 1.0;
            self.second += 1;
        }

        while self.second >= 60 {
            self.second -= 60;
            self.minute += 1;
        }

        while self.minute >= 60 {
            self.minute -= 60;
            self.hour += 1;
        }

        while self.hour >= 24 {
            self.hour -= 24;
            self.day += 1;
        }
    }

    /// Set to specific time instantly.
    pub fn set_time(&mut self, hour: u32, minute: u32) {
        self.hour = hour.min(23);
        self.minute = minute.min(59);
        self.second = 0;
        self.fraction = 0.0;
    }

    /// Skip to next day at specified hour.
    pub fn skip_to_next_day(&mut self, hour: u32) {
        self.day += 1;
        self.hour = hour.min(23);
        self.minute = 0;
        self.second = 0;
        self.fraction = 0.0;
    }
}

// ============================================================================
// G-55: Weather Save
// ============================================================================

/// Weather type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatherType {
    /// Clear skies.
    Clear,
    /// Partly cloudy.
    PartlyCloudy,
    /// Overcast.
    Cloudy,
    /// Light rain.
    LightRain,
    /// Heavy rain.
    HeavyRain,
    /// Thunderstorm.
    Storm,
    /// Fog.
    Fog,
    /// Light snow.
    LightSnow,
    /// Heavy snow/blizzard.
    Blizzard,
    /// Sandstorm.
    Sandstorm,
    /// Windy.
    Windy,
}

impl Default for WeatherType {
    fn default() -> Self {
        Self::Clear
    }
}

impl WeatherType {
    /// Get visibility modifier (0.0-1.0).
    #[must_use]
    pub fn visibility(self) -> f32 {
        match self {
            Self::Clear | Self::Windy => 1.0,
            Self::PartlyCloudy => 0.95,
            Self::Cloudy => 0.85,
            Self::LightRain | Self::LightSnow => 0.7,
            Self::HeavyRain => 0.5,
            Self::Fog => 0.3,
            Self::Storm => 0.4,
            Self::Blizzard | Self::Sandstorm => 0.2,
        }
    }

    /// Get movement speed modifier.
    #[must_use]
    pub fn movement_modifier(self) -> f32 {
        match self {
            Self::Clear | Self::PartlyCloudy | Self::Cloudy => 1.0,
            Self::Windy => 0.95,
            Self::LightRain | Self::LightSnow | Self::Fog => 0.9,
            Self::HeavyRain => 0.8,
            Self::Storm => 0.7,
            Self::Blizzard | Self::Sandstorm => 0.5,
        }
    }

    /// Check if precipitation.
    #[must_use]
    pub fn has_precipitation(self) -> bool {
        matches!(
            self,
            Self::LightRain | Self::HeavyRain | Self::Storm | Self::LightSnow | Self::Blizzard
        )
    }

    /// Get precipitation intensity (0.0-1.0).
    #[must_use]
    pub fn precipitation_intensity(self) -> f32 {
        match self {
            Self::LightRain | Self::LightSnow => 0.3,
            Self::HeavyRain => 0.7,
            Self::Storm | Self::Blizzard => 1.0,
            _ => 0.0,
        }
    }
}

/// Weather forecast entry for save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherForecastSave {
    /// Weather type.
    pub weather: WeatherType,
    /// Start time (total game seconds).
    pub start_time: f64,
    /// Duration in game seconds.
    pub duration: f64,
    /// Transition time from previous weather.
    pub transition_time: f64,
}

impl WeatherForecastSave {
    /// Create new forecast entry.
    #[must_use]
    pub fn new(weather: WeatherType, start_time: f64, duration: f64) -> Self {
        Self {
            weather,
            start_time,
            duration,
            transition_time: 300.0, // 5 minutes default
        }
    }

    /// Set transition time.
    #[must_use]
    pub fn with_transition(mut self, time: f64) -> Self {
        self.transition_time = time;
        self
    }

    /// Get end time.
    #[must_use]
    pub fn end_time(&self) -> f64 {
        self.start_time + self.duration
    }

    /// Check if active at time.
    #[must_use]
    pub fn is_active(&self, time: f64) -> bool {
        time >= self.start_time && time < self.end_time()
    }
}
/// Weather save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherSave {
    /// Current weather.
    pub current: WeatherType,
    /// Previous weather (for transitions).
    pub previous: WeatherType,
    /// Transition progress (0.0-1.0).
    pub transition: f32,
    /// Wind direction (degrees, 0 = north).
    pub wind_direction: f32,
    /// Wind speed.
    pub wind_speed: f32,
    /// Temperature.
    pub temperature: f32,
    /// Humidity (0.0-1.0).
    pub humidity: f32,
    /// Weather forecast.
    pub forecast: Vec<WeatherForecastSave>,
    /// Time current weather started.
    pub weather_start_time: f64,
    /// Duration of current weather.
    pub weather_duration: f64,
}

impl Default for WeatherSave {
    fn default() -> Self {
        Self {
            current: WeatherType::Clear,
            previous: WeatherType::Clear,
            transition: 1.0,
            wind_direction: 0.0,
            wind_speed: 5.0,
            temperature: 20.0,
            humidity: 0.5,
            forecast: Vec::new(),
            weather_start_time: 0.0,
            weather_duration: 3600.0, // 1 hour default
        }
    }
}

impl WeatherSave {
    /// Create new weather save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set current weather.
    #[must_use]
    pub fn with_weather(mut self, weather: WeatherType) -> Self {
        self.current = weather;
        self
    }

    /// Set wind.
    #[must_use]
    pub fn with_wind(mut self, direction: f32, speed: f32) -> Self {
        self.wind_direction = direction % 360.0;
        self.wind_speed = speed.max(0.0);
        self
    }

    /// Set temperature.
    #[must_use]
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Set humidity.
    #[must_use]
    pub fn with_humidity(mut self, humidity: f32) -> Self {
        self.humidity = humidity.clamp(0.0, 1.0);
        self
    }

    /// Add forecast entry.
    pub fn add_forecast(&mut self, forecast: WeatherForecastSave) {
        self.forecast.push(forecast);
    }

    /// Get current visibility.
    #[must_use]
    pub fn visibility(&self) -> f32 {
        if self.transition >= 1.0 {
            self.current.visibility()
        } else {
            let prev = self.previous.visibility();
            let curr = self.current.visibility();
            prev + (curr - prev) * self.transition
        }
    }

    /// Get movement modifier.
    #[must_use]
    pub fn movement_modifier(&self) -> f32 {
        if self.transition >= 1.0 {
            self.current.movement_modifier()
        } else {
            let prev = self.previous.movement_modifier();
            let curr = self.current.movement_modifier();
            prev + (curr - prev) * self.transition
        }
    }

    /// Start weather transition.
    pub fn transition_to(&mut self, weather: WeatherType, start_time: f64, duration: f64) {
        self.previous = self.current;
        self.current = weather;
        self.transition = 0.0;
        self.weather_start_time = start_time;
        self.weather_duration = duration;
    }

    /// Update transition progress.
    pub fn update_transition(&mut self, delta: f32, transition_speed: f32) {
        if self.transition < 1.0 {
            self.transition = (self.transition + delta * transition_speed).min(1.0);
        }
    }

    /// Get wind direction as cardinal.
    #[must_use]
    pub fn wind_cardinal(&self) -> &'static str {
        let dir = self.wind_direction % 360.0;
        match dir as u32 {
            0..=22 | 338..=360 => "N",
            23..=67 => "NE",
            68..=112 => "E",
            113..=157 => "SE",
            158..=202 => "S",
            203..=247 => "SW",
            248..=292 => "W",
            _ => "NW",
        }
    }
}

// ============================================================================
// G-55: Moon Phase Save
// ============================================================================

/// Moon phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoonPhase {
    /// New moon (no visible moon).
    New,
    /// Waxing crescent.
    WaxingCrescent,
    /// First quarter.
    FirstQuarter,
    /// Waxing gibbous.
    WaxingGibbous,
    /// Full moon.
    Full,
    /// Waning gibbous.
    WaningGibbous,
    /// Last quarter.
    LastQuarter,
    /// Waning crescent.
    WaningCrescent,
}

impl Default for MoonPhase {
    fn default() -> Self {
        Self::New
    }
}

impl MoonPhase {
    /// Get moon phase from day.
    #[must_use]
    pub fn from_day(day: u32, lunar_cycle_days: u32) -> Self {
        let phase_day = day % lunar_cycle_days;
        let phase_progress = phase_day as f32 / lunar_cycle_days as f32;

        match (phase_progress * 8.0) as u32 {
            0 => Self::New,
            1 => Self::WaxingCrescent,
            2 => Self::FirstQuarter,
            3 => Self::WaxingGibbous,
            4 => Self::Full,
            5 => Self::WaningGibbous,
            6 => Self::LastQuarter,
            _ => Self::WaningCrescent,
        }
    }

    /// Get illumination level (0.0-1.0).
    #[must_use]
    pub fn illumination(self) -> f32 {
        match self {
            Self::New => 0.0,
            Self::WaxingCrescent | Self::WaningCrescent => 0.25,
            Self::FirstQuarter | Self::LastQuarter => 0.5,
            Self::WaxingGibbous | Self::WaningGibbous => 0.75,
            Self::Full => 1.0,
        }
    }

    /// Get next phase.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::New => Self::WaxingCrescent,
            Self::WaxingCrescent => Self::FirstQuarter,
            Self::FirstQuarter => Self::WaxingGibbous,
            Self::WaxingGibbous => Self::Full,
            Self::Full => Self::WaningGibbous,
            Self::WaningGibbous => Self::LastQuarter,
            Self::LastQuarter => Self::WaningCrescent,
            Self::WaningCrescent => Self::New,
        }
    }

    /// Check if waxing (growing).
    #[must_use]
    pub fn is_waxing(self) -> bool {
        matches!(
            self,
            Self::New | Self::WaxingCrescent | Self::FirstQuarter | Self::WaxingGibbous
        )
    }

    /// Check if waning (shrinking).
    #[must_use]
    pub fn is_waning(self) -> bool {
        !self.is_waxing()
    }
}

/// Moon save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoonSave {
    /// Current phase.
    pub phase: MoonPhase,
    /// Lunar cycle length in days.
    pub cycle_days: u32,
    /// Day within current cycle.
    pub cycle_day: u32,
    /// Moon rise hour.
    pub rise_hour: u32,
    /// Moon set hour.
    pub set_hour: u32,
}

impl Default for MoonSave {
    fn default() -> Self {
        Self {
            phase: MoonPhase::New,
            cycle_days: 28,
            cycle_day: 0,
            rise_hour: 19,
            set_hour: 6,
        }
    }
}

impl MoonSave {
    /// Create new moon save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set cycle length.
    #[must_use]
    pub fn with_cycle_days(mut self, days: u32) -> Self {
        self.cycle_days = days.max(1);
        self
    }

    /// Update for current day.
    pub fn update_for_day(&mut self, day: u32) {
        self.cycle_day = day % self.cycle_days;
        self.phase = MoonPhase::from_day(day, self.cycle_days);
    }

    /// Check if moon is visible at hour.
    #[must_use]
    pub fn is_visible(&self, hour: u32) -> bool {
        let hour = hour % 24;
        if self.rise_hour <= self.set_hour {
            hour >= self.rise_hour && hour < self.set_hour
        } else {
            hour >= self.rise_hour || hour < self.set_hour
        }
    }

    /// Get night light level from moon.
    #[must_use]
    pub fn night_light(&self, hour: u32) -> f32 {
        if self.is_visible(hour) {
            self.phase.illumination() * 0.3 // Moon provides up to 30% light
        } else {
            0.0
        }
    }
}

// ============================================================================
// G-55: Season Save
// ============================================================================

/// Season.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    /// Spring.
    Spring,
    /// Summer.
    Summer,
    /// Fall/Autumn.
    Fall,
    /// Winter.
    Winter,
}

impl Default for Season {
    fn default() -> Self {
        Self::Spring
    }
}

impl Season {
    /// Get season from day.
    #[must_use]
    pub fn from_day(day: u32, days_per_season: u32) -> Self {
        let season_index = (day / days_per_season) % 4;
        match season_index {
            0 => Self::Spring,
            1 => Self::Summer,
            2 => Self::Fall,
            _ => Self::Winter,
        }
    }

    /// Get next season.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Spring => Self::Summer,
            Self::Summer => Self::Fall,
            Self::Fall => Self::Winter,
            Self::Winter => Self::Spring,
        }
    }

    /// Get temperature modifier.
    #[must_use]
    pub fn temperature_modifier(self) -> f32 {
        match self {
            Self::Spring => 0.0,
            Self::Summer => 15.0,
            Self::Fall => -5.0,
            Self::Winter => -20.0,
        }
    }

    /// Get daylight hours.
    #[must_use]
    pub fn daylight_hours(self) -> u32 {
        match self {
            Self::Spring | Self::Fall => 12,
            Self::Summer => 16,
            Self::Winter => 8,
        }
    }

    /// Get typical weather weights.
    #[must_use]
    pub fn weather_weights(self) -> [(WeatherType, f32); 5] {
        match self {
            Self::Spring => [
                (WeatherType::Clear, 0.3),
                (WeatherType::PartlyCloudy, 0.3),
                (WeatherType::LightRain, 0.2),
                (WeatherType::Cloudy, 0.15),
                (WeatherType::Storm, 0.05),
            ],
            Self::Summer => [
                (WeatherType::Clear, 0.5),
                (WeatherType::PartlyCloudy, 0.2),
                (WeatherType::HeavyRain, 0.1),
                (WeatherType::Storm, 0.1),
                (WeatherType::Cloudy, 0.1),
            ],
            Self::Fall => [
                (WeatherType::Cloudy, 0.3),
                (WeatherType::PartlyCloudy, 0.25),
                (WeatherType::LightRain, 0.2),
                (WeatherType::Fog, 0.15),
                (WeatherType::Clear, 0.1),
            ],
            Self::Winter => [
                (WeatherType::Cloudy, 0.3),
                (WeatherType::LightSnow, 0.25),
                (WeatherType::Blizzard, 0.15),
                (WeatherType::Clear, 0.2),
                (WeatherType::Fog, 0.1),
            ],
        }
    }
}

/// Season save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonSave {
    /// Current season.
    pub current: Season,
    /// Days per season.
    pub days_per_season: u32,
    /// Day within current season.
    pub season_day: u32,
    /// Year number.
    pub year: u32,
}

impl Default for SeasonSave {
    fn default() -> Self {
        Self {
            current: Season::Spring,
            days_per_season: 30,
            season_day: 1,
            year: 1,
        }
    }
}

impl SeasonSave {
    /// Create new season save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set days per season.
    #[must_use]
    pub fn with_days_per_season(mut self, days: u32) -> Self {
        self.days_per_season = days.max(1);
        self
    }

    /// Update for current day.
    pub fn update_for_day(&mut self, day: u32) {
        let total_seasons = day / self.days_per_season;
        self.year = total_seasons / 4 + 1;
        self.current = Season::from_day(day, self.days_per_season);
        self.season_day = (day % self.days_per_season) + 1;
    }

    /// Get progress through season (0.0-1.0).
    #[must_use]
    pub fn progress(&self) -> f32 {
        self.season_day as f32 / self.days_per_season as f32
    }

    /// Get days until next season.
    #[must_use]
    pub fn days_until_next(&self) -> u32 {
        self.days_per_season - self.season_day + 1
    }

    /// Get formatted year and season.
    #[must_use]
    pub fn formatted(&self) -> String {
        format!(
            "Year {}, {:?} Day {}",
            self.year, self.current, self.season_day
        )
    }
}

// ============================================================================
// G-55: Environment Timer Save
// ============================================================================

/// Environment event type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvironmentEvent {
    /// Eclipse.
    Eclipse,
    /// Meteor shower.
    MeteorShower,
    /// Aurora.
    Aurora,
    /// Blood moon.
    BloodMoon,
    /// Harvest festival.
    HarvestFestival,
    /// Custom event.
    Custom(String),
}

/// Environment timer save.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentTimerSave {
    /// Timer ID.
    pub id: String,
    /// Event type.
    pub event: EnvironmentEvent,
    /// Start time.
    pub start_time: f64,
    /// Duration.
    pub duration: f64,
    /// Whether active.
    pub active: bool,
    /// Repeat interval (None = one-time).
    pub repeat_interval: Option<f64>,
}

impl EnvironmentTimerSave {
    /// Create new timer.
    #[must_use]
    pub fn new(id: impl Into<String>, event: EnvironmentEvent, start: f64, duration: f64) -> Self {
        Self {
            id: id.into(),
            event,
            start_time: start,
            duration,
            active: false,
            repeat_interval: None,
        }
    }

    /// Set repeat interval.
    #[must_use]
    pub fn with_repeat(mut self, interval: f64) -> Self {
        self.repeat_interval = Some(interval);
        self
    }

    /// Get end time.
    #[must_use]
    pub fn end_time(&self) -> f64 {
        self.start_time + self.duration
    }

    /// Check if should be active at time.
    #[must_use]
    pub fn should_be_active(&self, time: f64) -> bool {
        time >= self.start_time && time < self.end_time()
    }

    /// Update for current time.
    pub fn update(&mut self, time: f64) {
        self.active = self.should_be_active(time);

        // Handle repeat
        if let Some(interval) = self.repeat_interval {
            if time >= self.end_time() {
                self.start_time += interval;
            }
        }
    }
}

// ============================================================================
// G-55: Complete World State Save
// ============================================================================

/// Complete world state save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldStateSave {
    /// Save format version.
    pub version: u32,
    /// World time.
    pub time: WorldTimeSave,
    /// Weather state.
    pub weather: WeatherSave,
    /// Moon state.
    pub moon: MoonSave,
    /// Season state.
    pub season: SeasonSave,
    /// Environment timers.
    pub timers: Vec<EnvironmentTimerSave>,
    /// World seed (for regeneration).
    pub seed: u64,
    /// Custom world flags.
    pub flags: HashMap<String, String>,
}

impl Default for WorldStateSave {
    fn default() -> Self {
        Self {
            version: 1,
            time: WorldTimeSave::default(),
            weather: WeatherSave::default(),
            moon: MoonSave::default(),
            season: SeasonSave::default(),
            timers: Vec::new(),
            seed: 0,
            flags: HashMap::new(),
        }
    }
}

impl WorldStateSave {
    /// Create new world state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set world seed.
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set time.
    #[must_use]
    pub fn with_time(mut self, time: WorldTimeSave) -> Self {
        self.time = time;
        self
    }

    /// Set weather.
    #[must_use]
    pub fn with_weather(mut self, weather: WeatherSave) -> Self {
        self.weather = weather;
        self
    }

    /// Add environment timer.
    pub fn add_timer(&mut self, timer: EnvironmentTimerSave) {
        self.timers.push(timer);
    }

    /// Get active timers.
    #[must_use]
    pub fn active_timers(&self) -> Vec<&EnvironmentTimerSave> {
        self.timers.iter().filter(|t| t.active).collect()
    }

    /// Set world flag.
    pub fn set_flag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.flags.insert(key.into(), value.into());
    }

    /// Get world flag.
    #[must_use]
    pub fn get_flag(&self, key: &str) -> Option<&str> {
        self.flags.get(key).map(String::as_str)
    }

    /// Update all systems for current time.
    pub fn update(&mut self, delta_seconds: f64) {
        // Update time
        self.time.advance(delta_seconds);

        // Update season and moon based on day
        self.season.update_for_day(self.time.day);
        self.moon.update_for_day(self.time.day);

        // Update timers
        let current_time = self.time.total_seconds;
        for timer in &mut self.timers {
            timer.update(current_time);
        }
    }

    /// Get combined light level.
    #[must_use]
    pub fn light_level(&self) -> f32 {
        let base = self.time.light_level();
        let weather_modifier = self.weather.visibility();
        let moon_bonus = if self.time.is_night() {
            self.moon.night_light(self.time.hour)
        } else {
            0.0
        };

        (base * weather_modifier + moon_bonus).clamp(0.0, 1.0)
    }

    /// Get current temperature.
    #[must_use]
    pub fn temperature(&self) -> f32 {
        let base = self.weather.temperature;
        let season_mod = self.season.current.temperature_modifier();
        let time_mod = if self.time.is_night() { -5.0 } else { 0.0 };

        base + season_mod + time_mod
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_of_week() {
        assert_eq!(DayOfWeek::from_index(0), DayOfWeek::Sunday);
        assert_eq!(DayOfWeek::from_index(7), DayOfWeek::Sunday);
        assert_eq!(DayOfWeek::Monday.next(), DayOfWeek::Tuesday);
    }

    #[test]
    fn test_time_period() {
        assert_eq!(SaveTimePeriod::from_hour(6), SaveTimePeriod::Dawn);
        assert_eq!(SaveTimePeriod::from_hour(12), SaveTimePeriod::Afternoon);
        assert_eq!(SaveTimePeriod::from_hour(22), SaveTimePeriod::Night);
        assert!(SaveTimePeriod::Morning.is_day());
        assert!(SaveTimePeriod::Night.is_night());
    }

    #[test]
    fn test_world_time() {
        let mut time = WorldTimeSave::new().with_time(1, 8, 30);
        assert_eq!(time.hour, 8);
        assert_eq!(time.minute, 30);
        assert_eq!(time.formatted_time(), "08:30");

        time.advance(3600.0); // 1 hour
        assert_eq!(time.hour, 9);
    }

    #[test]
    fn test_time_advance_day() {
        let mut time = WorldTimeSave::new().with_time(1, 23, 30);
        time.advance(3600.0); // 1 hour

        assert_eq!(time.day, 2);
        assert_eq!(time.hour, 0);
    }

    #[test]
    fn test_weather_type() {
        assert_eq!(WeatherType::Clear.visibility(), 1.0);
        assert!(WeatherType::HeavyRain.has_precipitation());
        assert!(!WeatherType::Fog.has_precipitation());
    }

    #[test]
    fn test_weather_save() {
        let weather = WeatherSave::new()
            .with_weather(WeatherType::LightRain)
            .with_wind(90.0, 15.0)
            .with_temperature(18.0);

        assert_eq!(weather.current, WeatherType::LightRain);
        assert_eq!(weather.wind_cardinal(), "E");
    }

    #[test]
    fn test_weather_transition() {
        let mut weather = WeatherSave::new();
        weather.transition_to(WeatherType::Storm, 0.0, 3600.0);

        assert_eq!(weather.current, WeatherType::Storm);
        assert_eq!(weather.previous, WeatherType::Clear);
        assert_eq!(weather.transition, 0.0);
    }

    #[test]
    fn test_moon_phase() {
        let phase = MoonPhase::from_day(14, 28);
        assert_eq!(phase, MoonPhase::Full);
        assert_eq!(phase.illumination(), 1.0);
        assert!(!phase.is_waxing());
    }

    #[test]
    fn test_moon_visibility() {
        let moon = MoonSave::default();
        assert!(moon.is_visible(20));
        assert!(moon.is_visible(2));
        assert!(!moon.is_visible(12));
    }

    #[test]
    fn test_season() {
        assert_eq!(Season::from_day(0, 30), Season::Spring);
        assert_eq!(Season::from_day(30, 30), Season::Summer);
        assert_eq!(Season::from_day(120, 30), Season::Spring); // Wrapped
    }

    #[test]
    fn test_season_save() {
        let mut season = SeasonSave::new().with_days_per_season(30);
        season.update_for_day(45);

        assert_eq!(season.current, Season::Summer);
        assert_eq!(season.season_day, 16);
    }

    #[test]
    fn test_environment_timer() {
        let mut timer =
            EnvironmentTimerSave::new("eclipse", EnvironmentEvent::Eclipse, 100.0, 60.0);

        assert!(!timer.should_be_active(50.0));
        assert!(timer.should_be_active(120.0));
        assert!(!timer.should_be_active(200.0));

        timer.update(120.0);
        assert!(timer.active);
    }

    #[test]
    fn test_world_state_save() {
        let state = WorldStateSave::new().with_seed(12345);

        assert_eq!(state.seed, 12345);
        assert_eq!(state.time.day, 1);
    }

    #[test]
    fn test_world_state_update() {
        let mut state = WorldStateSave::new();
        state.update(86400.0); // 1 day

        assert_eq!(state.time.day, 2);
    }

    #[test]
    fn test_combined_light_level() {
        let state = WorldStateSave::new()
            .with_time(WorldTimeSave::new().with_time(1, 12, 0))
            .with_weather(WeatherSave::new());

        assert!(state.light_level() > 0.8); // Midday clear should be bright
    }
}
