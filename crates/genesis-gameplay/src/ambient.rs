//! Ambient soundscape system.
//!
//! This module provides:
//! - Ambient layers for biomes and time of day
//! - Multi-layer mixing with smooth transitions
//! - Weather sound integration
//! - Distance-based volume falloff

use crate::biome::BiomeType;
use crate::weather::WeatherState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ambient sound types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmbientSound {
    // === Nature ===
    /// Wind (light breeze).
    WindLight,
    /// Wind (strong).
    WindStrong,
    /// Birds chirping (day).
    Birds,
    /// Crickets (night).
    Crickets,
    /// Owls (night).
    Owls,
    /// Frogs (swamp/water).
    Frogs,
    /// Cicadas (hot biomes).
    Cicadas,
    /// Wolves howling (night, forest).
    Wolves,

    // === Water ===
    /// River/stream flowing.
    River,
    /// Ocean waves.
    Ocean,
    /// Lake/pond gentle.
    Lake,
    /// Waterfall.
    Waterfall,
    /// Rain drops.
    Rain,
    /// Heavy rain/storm.
    Storm,

    // === Environment ===
    /// Cave dripping.
    CaveDrips,
    /// Cave echo/rumble.
    CaveRumble,
    /// Lava bubbling.
    Lava,
    /// Fire crackling.
    Fire,
    /// Leaves rustling.
    Leaves,
    /// Sand wind.
    SandWind,
    /// Snow wind.
    SnowWind,
    /// Thunder.
    Thunder,

    // === Village ===
    /// Crowd murmur.
    Crowd,
    /// Distant hammering.
    Smithy,
    /// Market bustle.
    Market,
    /// Tavern noise.
    Tavern,

    // === Dungeon ===
    /// Distant screams/moans.
    DungeonMoans,
    /// Chains rattling.
    Chains,
    /// Stone grinding.
    StoneGrinding,
}

impl AmbientSound {
    /// Get the asset path for this sound.
    #[must_use]
    pub fn asset_path(self) -> &'static str {
        match self {
            // Nature
            Self::WindLight => "sounds/ambient/wind_light.ogg",
            Self::WindStrong => "sounds/ambient/wind_strong.ogg",
            Self::Birds => "sounds/ambient/birds.ogg",
            Self::Crickets => "sounds/ambient/crickets.ogg",
            Self::Owls => "sounds/ambient/owls.ogg",
            Self::Frogs => "sounds/ambient/frogs.ogg",
            Self::Cicadas => "sounds/ambient/cicadas.ogg",
            Self::Wolves => "sounds/ambient/wolves.ogg",

            // Water
            Self::River => "sounds/ambient/river.ogg",
            Self::Ocean => "sounds/ambient/ocean.ogg",
            Self::Lake => "sounds/ambient/lake.ogg",
            Self::Waterfall => "sounds/ambient/waterfall.ogg",
            Self::Rain => "sounds/ambient/rain.ogg",
            Self::Storm => "sounds/ambient/storm.ogg",

            // Environment
            Self::CaveDrips => "sounds/ambient/cave_drips.ogg",
            Self::CaveRumble => "sounds/ambient/cave_rumble.ogg",
            Self::Lava => "sounds/ambient/lava.ogg",
            Self::Fire => "sounds/ambient/fire.ogg",
            Self::Leaves => "sounds/ambient/leaves.ogg",
            Self::SandWind => "sounds/ambient/sand_wind.ogg",
            Self::SnowWind => "sounds/ambient/snow_wind.ogg",
            Self::Thunder => "sounds/ambient/thunder.ogg",

            // Village
            Self::Crowd => "sounds/ambient/crowd.ogg",
            Self::Smithy => "sounds/ambient/smithy.ogg",
            Self::Market => "sounds/ambient/market.ogg",
            Self::Tavern => "sounds/ambient/tavern.ogg",

            // Dungeon
            Self::DungeonMoans => "sounds/ambient/dungeon_moans.ogg",
            Self::Chains => "sounds/ambient/chains.ogg",
            Self::StoneGrinding => "sounds/ambient/stone_grinding.ogg",
        }
    }

    /// Get default volume for this sound.
    #[must_use]
    pub fn default_volume(self) -> f32 {
        match self {
            Self::WindStrong | Self::Ocean | Self::Rain => 0.5,
            Self::Birds
            | Self::River
            | Self::SandWind
            | Self::SnowWind
            | Self::Lava
            | Self::Fire
            | Self::Tavern => 0.4,
            Self::WindLight
            | Self::Crickets
            | Self::Frogs
            | Self::Lake
            | Self::CaveDrips
            | Self::Crowd
            | Self::Market
            | Self::StoneGrinding => 0.3,
            Self::Cicadas => 0.25,
            Self::Owls
            | Self::Wolves
            | Self::CaveRumble
            | Self::Leaves
            | Self::Smithy
            | Self::DungeonMoans
            | Self::Chains => 0.2,
            Self::Waterfall | Self::Storm => 0.7,
            Self::Thunder => 0.8,
        }
    }

    /// Check if this sound should loop.
    #[must_use]
    pub fn loops(self) -> bool {
        // Thunder is the only non-looping ambient
        !matches!(self, Self::Thunder)
    }
}

/// A single ambient sound layer.
#[derive(Debug, Clone)]
pub struct AmbientLayer {
    /// The ambient sound.
    pub sound: AmbientSound,
    /// Current volume (0.0-1.0).
    pub volume: f32,
    /// Target volume.
    pub target_volume: f32,
    /// Whether currently active.
    pub active: bool,
    /// Fade speed (volume per second).
    pub fade_speed: f32,
}

impl AmbientLayer {
    /// Create a new ambient layer.
    #[must_use]
    pub fn new(sound: AmbientSound) -> Self {
        Self {
            sound,
            volume: 0.0,
            target_volume: sound.default_volume(),
            active: false,
            fade_speed: 0.5,
        }
    }

    /// Create with custom volume.
    #[must_use]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.target_volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Create with custom fade speed.
    #[must_use]
    pub fn with_fade_speed(mut self, speed: f32) -> Self {
        self.fade_speed = speed.max(0.01);
        self
    }

    /// Activate this layer (fade in).
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate this layer (fade out).
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Set target volume.
    pub fn set_target_volume(&mut self, volume: f32) {
        self.target_volume = volume.clamp(0.0, 1.0);
    }

    /// Update volume fade.
    pub fn update(&mut self, delta: f32) {
        let target = if self.active { self.target_volume } else { 0.0 };

        if (self.volume - target).abs() < 0.001 {
            self.volume = target;
            return;
        }

        if self.volume < target {
            self.volume = (self.volume + self.fade_speed * delta).min(target);
        } else {
            self.volume = (self.volume - self.fade_speed * delta).max(target);
        }
    }

    /// Check if layer is audible.
    #[must_use]
    pub fn is_audible(&self) -> bool {
        self.volume > 0.001
    }

    /// Check if layer is fully faded out.
    #[must_use]
    pub fn is_faded_out(&self) -> bool {
        !self.active && self.volume < 0.001
    }
}

/// Biome ambient configuration.
#[derive(Debug, Clone)]
pub struct BiomeAmbientConfig {
    /// Day sounds for this biome.
    pub day_sounds: Vec<(AmbientSound, f32)>,
    /// Night sounds for this biome.
    pub night_sounds: Vec<(AmbientSound, f32)>,
    /// Base sounds (always on).
    pub base_sounds: Vec<(AmbientSound, f32)>,
}

impl BiomeAmbientConfig {
    /// Create empty config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            day_sounds: Vec::new(),
            night_sounds: Vec::new(),
            base_sounds: Vec::new(),
        }
    }

    /// Add a day sound.
    #[must_use]
    pub fn with_day_sound(mut self, sound: AmbientSound, volume: f32) -> Self {
        self.day_sounds.push((sound, volume));
        self
    }

    /// Add a night sound.
    #[must_use]
    pub fn with_night_sound(mut self, sound: AmbientSound, volume: f32) -> Self {
        self.night_sounds.push((sound, volume));
        self
    }

    /// Add a base sound.
    #[must_use]
    pub fn with_base_sound(mut self, sound: AmbientSound, volume: f32) -> Self {
        self.base_sounds.push((sound, volume));
        self
    }
}

impl Default for BiomeAmbientConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Default biome ambient mappings.
pub fn default_biome_ambients() -> HashMap<BiomeType, BiomeAmbientConfig> {
    let mut map = HashMap::new();

    // Forest
    map.insert(
        BiomeType::Forest,
        BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Birds, 0.4)
            .with_day_sound(AmbientSound::Leaves, 0.2)
            .with_night_sound(AmbientSound::Crickets, 0.3)
            .with_night_sound(AmbientSound::Owls, 0.2)
            .with_base_sound(AmbientSound::WindLight, 0.2),
    );

    // Plains
    map.insert(
        BiomeType::Plains,
        BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Birds, 0.3)
            .with_night_sound(AmbientSound::Crickets, 0.4)
            .with_base_sound(AmbientSound::WindLight, 0.3),
    );

    // Desert
    map.insert(
        BiomeType::Desert,
        BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Cicadas, 0.3)
            .with_day_sound(AmbientSound::SandWind, 0.4)
            .with_night_sound(AmbientSound::WindLight, 0.3)
            .with_base_sound(AmbientSound::SandWind, 0.2),
    );

    // Swamp
    map.insert(
        BiomeType::Swamp,
        BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Frogs, 0.4)
            .with_night_sound(AmbientSound::Frogs, 0.3)
            .with_night_sound(AmbientSound::Crickets, 0.3)
            .with_base_sound(AmbientSound::Lake, 0.3),
    );

    // Mountain
    map.insert(
        BiomeType::Mountain,
        BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Birds, 0.2)
            .with_night_sound(AmbientSound::Wolves, 0.3)
            .with_base_sound(AmbientSound::WindStrong, 0.5),
    );

    // Plains
    map.insert(
        BiomeType::Plains,
        BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Birds, 0.3)
            .with_night_sound(AmbientSound::Crickets, 0.3)
            .with_base_sound(AmbientSound::WindLight, 0.3),
    );

    // Lake
    map.insert(
        BiomeType::Lake,
        BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Birds, 0.3)
            .with_night_sound(AmbientSound::Frogs, 0.4)
            .with_base_sound(AmbientSound::Lake, 0.4),
    );

    map
}

/// Weather sound configuration.
#[derive(Debug, Clone)]
pub struct WeatherSoundConfig {
    /// Sound to play.
    pub sound: AmbientSound,
    /// Base volume.
    pub volume: f32,
    /// Intensity multiplier (applied with weather intensity).
    pub intensity_mult: f32,
}

impl WeatherSoundConfig {
    /// Create new config.
    #[must_use]
    pub fn new(sound: AmbientSound, volume: f32) -> Self {
        Self {
            sound,
            volume,
            intensity_mult: 1.0,
        }
    }

    /// With intensity multiplier.
    #[must_use]
    pub fn with_intensity_mult(mut self, mult: f32) -> Self {
        self.intensity_mult = mult;
        self
    }
}

/// Default weather sound mappings.
pub fn default_weather_sounds() -> HashMap<WeatherState, Vec<WeatherSoundConfig>> {
    let mut map = HashMap::new();

    map.insert(
        WeatherState::Raining,
        vec![WeatherSoundConfig::new(AmbientSound::Rain, 0.5).with_intensity_mult(1.0)],
    );

    map.insert(
        WeatherState::Storm,
        vec![
            WeatherSoundConfig::new(AmbientSound::Storm, 0.6).with_intensity_mult(1.0),
            WeatherSoundConfig::new(AmbientSound::Thunder, 0.8).with_intensity_mult(0.5),
        ],
    );

    // Clear, Cloudy have no special sounds
    map.insert(WeatherState::Clear, vec![]);
    map.insert(WeatherState::Cloudy, vec![]);

    map
}

/// Manages ambient soundscape.
#[derive(Debug)]
pub struct AmbientManager {
    /// Active ambient layers.
    layers: HashMap<AmbientSound, AmbientLayer>,
    /// Biome ambient configurations.
    biome_configs: HashMap<BiomeType, BiomeAmbientConfig>,
    /// Weather sound configurations.
    weather_configs: HashMap<WeatherState, Vec<WeatherSoundConfig>>,
    /// Current biome.
    current_biome: Option<BiomeType>,
    /// Previous biome (for transitions).
    previous_biome: Option<BiomeType>,
    /// Is it night time?
    is_night: bool,
    /// Current weather.
    current_weather: WeatherState,
    /// Weather intensity (0.0-1.0).
    weather_intensity: f32,
    /// Master ambient volume.
    master_volume: f32,
    /// Whether ambient is enabled.
    enabled: bool,
    /// Transition progress (0.0-1.0).
    transition_progress: f32,
    /// Transition speed.
    transition_speed: f32,
    /// Indoor flag (mutes outdoor sounds).
    is_indoor: bool,
}

impl AmbientManager {
    /// Create a new ambient manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            biome_configs: default_biome_ambients(),
            weather_configs: default_weather_sounds(),
            current_biome: None,
            previous_biome: None,
            is_night: false,
            current_weather: WeatherState::Clear,
            weather_intensity: 0.0,
            master_volume: 1.0,
            enabled: true,
            transition_progress: 1.0,
            transition_speed: 0.5,
            is_indoor: false,
        }
    }

    /// Set master volume.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Get master volume.
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Enable/disable ambient.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // Fade out all layers
            for layer in self.layers.values_mut() {
                layer.deactivate();
            }
        }
    }

    /// Check if enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set indoor state.
    pub fn set_indoor(&mut self, indoor: bool) {
        if self.is_indoor == indoor {
            return;
        }
        self.is_indoor = indoor;
        self.update_active_layers();
    }

    /// Set current biome.
    pub fn set_biome(&mut self, biome: BiomeType) {
        if self.current_biome == Some(biome) {
            return;
        }

        self.previous_biome = self.current_biome;
        self.current_biome = Some(biome);
        self.transition_progress = 0.0;
        self.update_active_layers();
    }

    /// Set time of day.
    pub fn set_night(&mut self, is_night: bool) {
        if self.is_night == is_night {
            return;
        }
        self.is_night = is_night;
        self.update_active_layers();
    }

    /// Set weather state.
    pub fn set_weather(&mut self, weather: WeatherState, intensity: f32) {
        self.current_weather = weather;
        self.weather_intensity = intensity.clamp(0.0, 1.0);
        self.update_weather_layers();
    }

    /// Update active layers based on current state.
    fn update_active_layers(&mut self) {
        if !self.enabled {
            return;
        }

        // Deactivate all existing layers first
        for layer in self.layers.values_mut() {
            layer.deactivate();
        }

        // If indoor, only play relevant indoor sounds
        if self.is_indoor {
            return;
        }

        // Collect sounds to activate (to avoid borrow issues)
        let mut sounds_to_activate: Vec<(AmbientSound, f32)> = Vec::new();

        // Activate biome layers
        if let Some(biome) = self.current_biome {
            if let Some(config) = self.biome_configs.get(&biome) {
                // Base sounds
                sounds_to_activate.extend(config.base_sounds.iter().copied());

                // Time-based sounds
                let time_sounds = if self.is_night {
                    &config.night_sounds
                } else {
                    &config.day_sounds
                };

                sounds_to_activate.extend(time_sounds.iter().copied());
            }
        }

        // Activate collected sounds
        for (sound, volume) in sounds_to_activate {
            self.activate_layer(sound, volume);
        }

        // Update weather layers
        self.update_weather_layers();
    }

    /// Update weather-based layers.
    fn update_weather_layers(&mut self) {
        if !self.enabled || self.is_indoor {
            return;
        }

        // Collect weather sounds to activate
        let mut sounds_to_activate: Vec<(AmbientSound, f32)> = Vec::new();

        if let Some(configs) = self.weather_configs.get(&self.current_weather) {
            for config in configs {
                let volume =
                    config.volume * (1.0 + (self.weather_intensity - 0.5) * config.intensity_mult);
                sounds_to_activate.push((config.sound, volume));
            }
        }

        // Activate collected sounds
        for (sound, volume) in sounds_to_activate {
            self.activate_layer(sound, volume);
        }
    }

    /// Activate or update a layer.
    fn activate_layer(&mut self, sound: AmbientSound, volume: f32) {
        let layer = self
            .layers
            .entry(sound)
            .or_insert_with(|| AmbientLayer::new(sound));
        layer.set_target_volume(volume);
        layer.activate();
    }

    /// Update all layers.
    pub fn update(&mut self, delta: f32) {
        // Update transition progress
        if self.transition_progress < 1.0 {
            self.transition_progress =
                (self.transition_progress + self.transition_speed * delta).min(1.0);
        }

        // Update all layers
        for layer in self.layers.values_mut() {
            layer.update(delta);
        }

        // Remove fully faded out layers to save memory
        self.layers.retain(|_, layer| !layer.is_faded_out());
    }

    /// Get all active layers for audio backend.
    #[must_use]
    pub fn active_layers(&self) -> Vec<AmbientPlaybackInfo> {
        self.layers
            .values()
            .filter(|l| l.is_audible())
            .map(|l| AmbientPlaybackInfo {
                sound: l.sound,
                volume: l.volume * self.master_volume,
                loops: l.sound.loops(),
            })
            .collect()
    }

    /// Get layer count.
    #[must_use]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get active layer count.
    #[must_use]
    pub fn active_layer_count(&self) -> usize {
        self.layers.values().filter(|l| l.is_audible()).count()
    }

    /// Add custom biome config.
    pub fn add_biome_config(&mut self, biome: BiomeType, config: BiomeAmbientConfig) {
        self.biome_configs.insert(biome, config);
    }

    /// Get biome config for customization.
    #[must_use]
    pub fn biome_config(&self, biome: BiomeType) -> Option<&BiomeAmbientConfig> {
        self.biome_configs.get(&biome)
    }
}

impl Default for AmbientManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Ambient playback info for audio backend.
#[derive(Debug, Clone)]
pub struct AmbientPlaybackInfo {
    /// The sound to play.
    pub sound: AmbientSound,
    /// Current volume (0.0-1.0).
    pub volume: f32,
    /// Whether to loop.
    pub loops: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambient_sound_asset_paths() {
        let sounds = [
            AmbientSound::WindLight,
            AmbientSound::Birds,
            AmbientSound::River,
            AmbientSound::CaveDrips,
        ];

        for sound in sounds {
            assert!(!sound.asset_path().is_empty());
        }
    }

    #[test]
    fn test_ambient_sound_loops() {
        assert!(AmbientSound::Birds.loops());
        assert!(AmbientSound::River.loops());
        assert!(!AmbientSound::Thunder.loops());
    }

    #[test]
    fn test_ambient_layer_fade() {
        let mut layer = AmbientLayer::new(AmbientSound::Birds);
        assert!((layer.volume - 0.0).abs() < 0.001);

        layer.activate();
        layer.update(1.0);
        assert!(layer.volume > 0.0);
    }

    #[test]
    fn test_ambient_layer_deactivate() {
        let mut layer = AmbientLayer::new(AmbientSound::Birds);
        layer.activate();
        layer.volume = 0.4;

        layer.deactivate();
        layer.update(2.0);
        assert!(layer.volume < 0.4);
    }

    #[test]
    fn test_biome_ambient_config() {
        let config = BiomeAmbientConfig::new()
            .with_day_sound(AmbientSound::Birds, 0.4)
            .with_night_sound(AmbientSound::Crickets, 0.3)
            .with_base_sound(AmbientSound::WindLight, 0.2);

        assert_eq!(config.day_sounds.len(), 1);
        assert_eq!(config.night_sounds.len(), 1);
        assert_eq!(config.base_sounds.len(), 1);
    }

    #[test]
    fn test_default_biome_ambients() {
        let map = default_biome_ambients();
        assert!(map.contains_key(&BiomeType::Forest));
        assert!(map.contains_key(&BiomeType::Desert));
        assert!(map.contains_key(&BiomeType::Lake));
    }

    #[test]
    fn test_default_weather_sounds() {
        let map = default_weather_sounds();
        assert!(map.contains_key(&WeatherState::Raining));
        assert!(map.contains_key(&WeatherState::Storm));
        assert!(map.get(&WeatherState::Clear).is_some_and(|v| v.is_empty()));
    }

    #[test]
    fn test_ambient_manager_set_biome() {
        let mut manager = AmbientManager::new();
        manager.set_biome(BiomeType::Forest);

        assert_eq!(manager.current_biome, Some(BiomeType::Forest));
    }

    #[test]
    fn test_ambient_manager_night_change() {
        let mut manager = AmbientManager::new();
        manager.set_biome(BiomeType::Forest);
        manager.set_night(false);
        manager.update(0.1);

        // Day should have birds
        let layers_day: Vec<_> = manager.active_layers();
        let has_birds = layers_day.iter().any(|l| l.sound == AmbientSound::Birds);

        manager.set_night(true);
        manager.update(2.0); // Let fade happen

        let layers_night = manager.active_layers();
        let has_crickets = layers_night
            .iter()
            .any(|l| l.sound == AmbientSound::Crickets);

        // At least one should be true
        assert!(has_birds || has_crickets);
    }

    #[test]
    fn test_ambient_manager_weather() {
        let mut manager = AmbientManager::new();
        manager.set_biome(BiomeType::Forest);
        manager.set_weather(WeatherState::Raining, 0.8);
        manager.update(2.0);

        let layers = manager.active_layers();
        let has_rain = layers.iter().any(|l| l.sound == AmbientSound::Rain);
        assert!(has_rain);
    }

    #[test]
    fn test_ambient_manager_indoor() {
        let mut manager = AmbientManager::new();
        manager.set_biome(BiomeType::Forest);
        manager.update(2.0);

        let outdoor_count = manager.active_layer_count();

        manager.set_indoor(true);
        manager.update(2.0);

        let indoor_count = manager.active_layer_count();
        assert!(indoor_count <= outdoor_count);
    }

    #[test]
    fn test_ambient_manager_disabled() {
        let mut manager = AmbientManager::new();
        manager.set_enabled(false);
        manager.set_biome(BiomeType::Forest);
        manager.update(2.0);

        assert_eq!(manager.active_layer_count(), 0);
    }

    #[test]
    fn test_ambient_manager_volume() {
        let mut manager = AmbientManager::new();
        manager.set_master_volume(0.5);
        manager.set_biome(BiomeType::Forest);
        manager.update(2.0);

        for layer in manager.active_layers() {
            assert!(layer.volume <= 0.5);
        }
    }

    #[test]
    fn test_ambient_playback_info() {
        let info = AmbientPlaybackInfo {
            sound: AmbientSound::Birds,
            volume: 0.4,
            loops: true,
        };

        assert_eq!(info.sound, AmbientSound::Birds);
        assert!(info.loops);
    }
}
