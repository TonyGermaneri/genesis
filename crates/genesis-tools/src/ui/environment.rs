//! Environment HUD for displaying time and weather.
//!
//! This module provides:
//! - Clock display (HH:MM)
//! - Day counter
//! - Weather icons (sun/cloud/rain/storm)
//! - Background tint based on time of day

use egui::{Color32, Context, Id, Pos2, RichText, Rounding, Stroke, Ui};
use serde::{Deserialize, Serialize};

/// Weather condition types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WeatherType {
    /// Clear sunny weather.
    #[default]
    Clear,
    /// Partly cloudy.
    PartlyCloudy,
    /// Overcast/cloudy.
    Cloudy,
    /// Rainy weather.
    Rain,
    /// Thunderstorm.
    Storm,
    /// Snowy weather.
    Snow,
    /// Foggy weather.
    Fog,
}

impl WeatherType {
    /// Returns the weather icon (emoji/unicode).
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            WeatherType::Clear => "☀",
            WeatherType::PartlyCloudy => "⛅",
            WeatherType::Cloudy => "☁",
            WeatherType::Rain => "🌧",
            WeatherType::Storm => "⛈",
            WeatherType::Snow => "❄",
            WeatherType::Fog => "🌫",
        }
    }

    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            WeatherType::Clear => "Clear",
            WeatherType::PartlyCloudy => "Partly Cloudy",
            WeatherType::Cloudy => "Cloudy",
            WeatherType::Rain => "Rain",
            WeatherType::Storm => "Storm",
            WeatherType::Snow => "Snow",
            WeatherType::Fog => "Fog",
        }
    }

    /// Returns the icon color.
    #[must_use]
    pub fn icon_color(&self) -> Color32 {
        match self {
            WeatherType::Clear => Color32::GOLD,
            WeatherType::PartlyCloudy => Color32::from_rgb(200, 200, 100),
            WeatherType::Cloudy => Color32::LIGHT_GRAY,
            WeatherType::Rain => Color32::from_rgb(100, 150, 220),
            WeatherType::Storm => Color32::from_rgb(80, 80, 150),
            WeatherType::Snow => Color32::WHITE,
            WeatherType::Fog => Color32::from_rgb(180, 180, 180),
        }
    }

    /// Returns all weather types.
    #[must_use]
    pub fn all() -> &'static [WeatherType] {
        &[
            WeatherType::Clear,
            WeatherType::PartlyCloudy,
            WeatherType::Cloudy,
            WeatherType::Rain,
            WeatherType::Storm,
            WeatherType::Snow,
            WeatherType::Fog,
        ]
    }
}

/// Time of day periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimeOfDay {
    /// Early morning (5:00 - 7:59).
    Dawn,
    /// Morning (8:00 - 11:59).
    Morning,
    /// Midday (12:00 - 13:59).
    #[default]
    Noon,
    /// Afternoon (14:00 - 17:59).
    Afternoon,
    /// Evening (18:00 - 20:59).
    Dusk,
    /// Night (21:00 - 4:59).
    Night,
}

impl TimeOfDay {
    /// Determines the time of day from hour (0-23).
    #[must_use]
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            5..=7 => TimeOfDay::Dawn,
            8..=11 => TimeOfDay::Morning,
            12..=13 => TimeOfDay::Noon,
            14..=17 => TimeOfDay::Afternoon,
            18..=20 => TimeOfDay::Dusk,
            _ => TimeOfDay::Night,
        }
    }

    /// Returns the background tint color.
    #[must_use]
    pub fn tint_color(&self) -> Color32 {
        match self {
            TimeOfDay::Dawn => Color32::from_rgba_unmultiplied(255, 200, 150, 30),
            TimeOfDay::Morning => Color32::from_rgba_unmultiplied(255, 255, 200, 15),
            TimeOfDay::Noon => Color32::from_rgba_unmultiplied(255, 255, 255, 10),
            TimeOfDay::Afternoon => Color32::from_rgba_unmultiplied(255, 230, 180, 20),
            TimeOfDay::Dusk => Color32::from_rgba_unmultiplied(255, 150, 100, 40),
            TimeOfDay::Night => Color32::from_rgba_unmultiplied(50, 50, 100, 50),
        }
    }

    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeOfDay::Dawn => "Dawn",
            TimeOfDay::Morning => "Morning",
            TimeOfDay::Noon => "Noon",
            TimeOfDay::Afternoon => "Afternoon",
            TimeOfDay::Dusk => "Dusk",
            TimeOfDay::Night => "Night",
        }
    }
}

/// Environment HUD data model.
#[derive(Debug, Clone)]
pub struct EnvironmentHudModel {
    /// Current hour (0-23).
    pub hour: u8,
    /// Current minute (0-59).
    pub minute: u8,
    /// Current day number.
    pub day: u32,
    /// Current weather.
    pub weather: WeatherType,
    /// Temperature (optional, in degrees).
    pub temperature: Option<i8>,
    /// Weather intensity (0.0 - 1.0).
    pub weather_intensity: f32,
}

impl Default for EnvironmentHudModel {
    fn default() -> Self {
        Self {
            hour: 12,
            minute: 0,
            day: 1,
            weather: WeatherType::Clear,
            temperature: None,
            weather_intensity: 0.5,
        }
    }
}

impl EnvironmentHudModel {
    /// Creates a new environment model.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates with specific time and weather.
    #[must_use]
    pub fn with_time_weather(hour: u8, minute: u8, day: u32, weather: WeatherType) -> Self {
        Self {
            hour: hour.min(23),
            minute: minute.min(59),
            day,
            weather,
            temperature: None,
            weather_intensity: 0.5,
        }
    }

    /// Returns the time of day.
    #[must_use]
    pub fn time_of_day(&self) -> TimeOfDay {
        TimeOfDay::from_hour(self.hour)
    }

    /// Returns formatted time string (HH:MM).
    #[must_use]
    pub fn formatted_time(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    /// Sets the time.
    pub fn set_time(&mut self, hour: u8, minute: u8) {
        self.hour = hour.min(23);
        self.minute = minute.min(59);
    }

    /// Advances time by one minute.
    pub fn advance_minute(&mut self) {
        self.minute += 1;
        if self.minute >= 60 {
            self.minute = 0;
            self.hour += 1;
            if self.hour >= 24 {
                self.hour = 0;
                self.day += 1;
            }
        }
    }

    /// Returns total minutes elapsed today.
    #[must_use]
    pub fn total_minutes(&self) -> u16 {
        self.hour as u16 * 60 + self.minute as u16
    }
}

/// Configuration for the environment HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentHudConfig {
    /// Position from top-right corner.
    pub position_offset: (f32, f32),
    /// Panel width.
    pub panel_width: f32,
    /// Background color.
    pub background_color: [u8; 4],
    /// Show temperature.
    pub show_temperature: bool,
    /// Use 24-hour format.
    pub use_24h_format: bool,
    /// Show weather description.
    pub show_weather_text: bool,
}

impl Default for EnvironmentHudConfig {
    fn default() -> Self {
        Self {
            position_offset: (10.0, 10.0),
            panel_width: 120.0,
            background_color: [30, 30, 30, 200],
            show_temperature: true,
            use_24h_format: true,
            show_weather_text: true,
        }
    }
}

/// Environment HUD widget.
#[derive(Debug)]
pub struct EnvironmentHud {
    /// Configuration.
    config: EnvironmentHudConfig,
}

impl Default for EnvironmentHud {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentHud {
    /// Creates a new environment HUD.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: EnvironmentHudConfig::default(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: EnvironmentHudConfig) -> Self {
        Self { config }
    }

    /// Shows the environment HUD in the top-right corner.
    pub fn show(&self, ctx: &Context, model: &EnvironmentHudModel) {
        let screen_rect = ctx.screen_rect();
        let pos = Pos2::new(
            screen_rect.right() - self.config.panel_width - self.config.position_offset.0,
            self.config.position_offset.1,
        );

        egui::Area::new(Id::new("environment_hud"))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let bg_color = Color32::from_rgba_unmultiplied(
                    self.config.background_color[0],
                    self.config.background_color[1],
                    self.config.background_color[2],
                    self.config.background_color[3],
                );

                egui::Frame::none()
                    .fill(bg_color)
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0, Color32::from_gray(60)))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_width(self.config.panel_width - 16.0);
                        self.render_content(ui, model);
                    });
            });
    }

    /// Renders the HUD content.
    fn render_content(&self, ui: &mut Ui, model: &EnvironmentHudModel) {
        // Time display
        ui.horizontal(|ui| {
            let time_str = if self.config.use_24h_format {
                model.formatted_time()
            } else {
                let (hour12, period) = if model.hour == 0 {
                    (12, "AM")
                } else if model.hour < 12 {
                    (model.hour, "AM")
                } else if model.hour == 12 {
                    (12, "PM")
                } else {
                    (model.hour - 12, "PM")
                };
                format!("{:02}:{:02} {}", hour12, model.minute, period)
            };

            ui.label(RichText::new(time_str).size(18.0).strong());
        });

        // Day counter
        ui.label(
            RichText::new(format!("Day {}", model.day))
                .size(12.0)
                .color(Color32::LIGHT_GRAY),
        );

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Weather display
        ui.horizontal(|ui| {
            // Weather icon
            ui.label(
                RichText::new(model.weather.icon())
                    .size(24.0)
                    .color(model.weather.icon_color()),
            );

            if self.config.show_weather_text {
                ui.label(model.weather.display_name());
            }
        });

        // Temperature (optional)
        if self.config.show_temperature {
            if let Some(temp) = model.temperature {
                ui.label(
                    RichText::new(format!("{temp}°"))
                        .size(14.0)
                        .color(temperature_color(temp)),
                );
            }
        }

        // Time of day indicator
        ui.add_space(2.0);
        let tod = model.time_of_day();
        ui.label(
            RichText::new(tod.display_name())
                .size(10.0)
                .color(Color32::GRAY),
        );
    }

    /// Returns the background tint color for the current time.
    #[must_use]
    pub fn get_tint_color(&self, model: &EnvironmentHudModel) -> Color32 {
        model.time_of_day().tint_color()
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &EnvironmentHudConfig {
        &self.config
    }

    /// Sets the configuration.
    pub fn set_config(&mut self, config: EnvironmentHudConfig) {
        self.config = config;
    }
}

/// Returns a color for temperature display.
#[must_use]
pub fn temperature_color(temp: i8) -> Color32 {
    if temp <= 0 {
        Color32::from_rgb(100, 150, 255) // Cold (blue)
    } else if temp <= 15 {
        Color32::from_rgb(150, 200, 255) // Cool (light blue)
    } else if temp <= 25 {
        Color32::from_rgb(200, 255, 200) // Mild (light green)
    } else if temp <= 35 {
        Color32::from_rgb(255, 200, 100) // Warm (orange)
    } else {
        Color32::from_rgb(255, 100, 100) // Hot (red)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_type_icon() {
        assert_eq!(WeatherType::Clear.icon(), "☀");
        assert_eq!(WeatherType::Rain.icon(), "🌧");
        assert_eq!(WeatherType::Storm.icon(), "⛈");
    }

    #[test]
    fn test_weather_type_all() {
        let all = WeatherType::all();
        assert_eq!(all.len(), 7);
        assert!(all.contains(&WeatherType::Clear));
        assert!(all.contains(&WeatherType::Storm));
    }

    #[test]
    fn test_time_of_day_from_hour() {
        assert_eq!(TimeOfDay::from_hour(6), TimeOfDay::Dawn);
        assert_eq!(TimeOfDay::from_hour(10), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(12), TimeOfDay::Noon);
        assert_eq!(TimeOfDay::from_hour(15), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(19), TimeOfDay::Dusk);
        assert_eq!(TimeOfDay::from_hour(23), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(3), TimeOfDay::Night);
    }

    #[test]
    fn test_environment_model_default() {
        let model = EnvironmentHudModel::default();
        assert_eq!(model.hour, 12);
        assert_eq!(model.minute, 0);
        assert_eq!(model.day, 1);
        assert_eq!(model.weather, WeatherType::Clear);
    }

    #[test]
    fn test_environment_model_formatted_time() {
        let model = EnvironmentHudModel::with_time_weather(9, 5, 1, WeatherType::Clear);
        assert_eq!(model.formatted_time(), "09:05");

        let model2 = EnvironmentHudModel::with_time_weather(23, 59, 1, WeatherType::Clear);
        assert_eq!(model2.formatted_time(), "23:59");
    }

    #[test]
    fn test_environment_model_advance_minute() {
        let mut model = EnvironmentHudModel::with_time_weather(23, 59, 1, WeatherType::Clear);
        model.advance_minute();
        assert_eq!(model.hour, 0);
        assert_eq!(model.minute, 0);
        assert_eq!(model.day, 2);
    }

    #[test]
    fn test_environment_model_total_minutes() {
        let model = EnvironmentHudModel::with_time_weather(2, 30, 1, WeatherType::Clear);
        assert_eq!(model.total_minutes(), 150);
    }

    #[test]
    fn test_environment_model_set_time() {
        let mut model = EnvironmentHudModel::new();
        model.set_time(25, 70); // Out of range values
        assert_eq!(model.hour, 23);
        assert_eq!(model.minute, 59);
    }

    #[test]
    fn test_environment_hud_config_defaults() {
        let config = EnvironmentHudConfig::default();
        assert!(config.use_24h_format);
        assert!(config.show_temperature);
        assert!(config.show_weather_text);
    }

    #[test]
    fn test_environment_hud_new() {
        let hud = EnvironmentHud::new();
        assert!(hud.config.use_24h_format);
    }

    #[test]
    fn test_temperature_color() {
        let cold = temperature_color(-10);
        assert_eq!(cold, Color32::from_rgb(100, 150, 255));

        let hot = temperature_color(40);
        assert_eq!(hot, Color32::from_rgb(255, 100, 100));
    }

    #[test]
    fn test_time_of_day_tint() {
        let dawn = TimeOfDay::Dawn.tint_color();
        let night = TimeOfDay::Night.tint_color();
        // Night should be darker (higher alpha)
        assert!(night.a() > dawn.a());
    }

    #[test]
    fn test_weather_intensity_range() {
        let model = EnvironmentHudModel::default();
        assert!(model.weather_intensity >= 0.0 && model.weather_intensity <= 1.0);
    }
}
