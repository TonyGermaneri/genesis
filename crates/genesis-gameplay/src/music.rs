//! Music management system.
//!
//! This module provides:
//! - Music track definitions and asset mappings
//! - Biome-to-track mappings
//! - Crossfade transitions
//! - Combat music triggers
//! - Day/night music variants

use crate::biome::BiomeType;
use serde::{Deserialize, Serialize};

/// Music tracks available in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MusicTrack {
    /// Main menu theme.
    Menu,
    /// Default exploration music.
    Exploration,
    /// Forest biome music.
    Forest,
    /// Desert biome music.
    Desert,
    /// Snow/tundra biome music.
    Snow,
    /// Swamp biome music.
    Swamp,
    /// Mountain biome music.
    Mountain,
    /// Ocean/beach music.
    Ocean,
    /// Village/town music.
    Village,
    /// Dungeon music.
    Dungeon,
    /// Cave music.
    Cave,
    /// Night time ambient music.
    Night,
    /// Combat music.
    Combat,
    /// Boss battle music.
    Boss,
    /// Victory fanfare (short).
    Victory,
    /// Death/game over.
    GameOver,
    /// Peaceful/rest music.
    Peaceful,
    /// Mysterious/discovery music.
    Mystery,
    /// No music (silence).
    None,
}

impl MusicTrack {
    /// Get the asset path for this track.
    #[must_use]
    pub fn asset_path(self) -> Option<&'static str> {
        match self {
            Self::Menu => Some("sounds/music/menu_theme.mp3"),
            Self::Exploration => Some("sounds/music/exploration.mp3"),
            Self::Forest => Some("sounds/music/forest.mp3"),
            Self::Desert => Some("sounds/music/desert.mp3"),
            Self::Snow => Some("sounds/music/snow.mp3"),
            Self::Swamp => Some("sounds/music/swamp.mp3"),
            Self::Mountain => Some("sounds/music/mountain.mp3"),
            Self::Ocean => Some("sounds/music/ocean.mp3"),
            Self::Village => Some("sounds/music/village.mp3"),
            Self::Dungeon => Some("sounds/music/dungeon.mp3"),
            Self::Cave => Some("sounds/music/cave.mp3"),
            Self::Night => Some("sounds/music/night.mp3"),
            Self::Combat => Some("sounds/music/combat.mp3"),
            Self::Boss => Some("sounds/music/boss.mp3"),
            Self::Victory => Some("sounds/music/victory.mp3"),
            Self::GameOver => Some("sounds/music/game_over.mp3"),
            Self::Peaceful => Some("sounds/music/peaceful.mp3"),
            Self::Mystery => Some("sounds/music/mystery.mp3"),
            Self::None => None,
        }
    }

    /// Get the display name for this track.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Menu => "Menu Theme",
            Self::Exploration => "Exploration",
            Self::Forest => "Forest",
            Self::Desert => "Desert",
            Self::Snow => "Snow",
            Self::Swamp => "Swamp",
            Self::Mountain => "Mountain",
            Self::Ocean => "Ocean",
            Self::Village => "Village",
            Self::Dungeon => "Dungeon",
            Self::Cave => "Cave",
            Self::Night => "Night",
            Self::Combat => "Combat",
            Self::Boss => "Boss Battle",
            Self::Victory => "Victory",
            Self::GameOver => "Game Over",
            Self::Peaceful => "Peaceful",
            Self::Mystery => "Mystery",
            Self::None => "None",
        }
    }

    /// Check if this track should loop.
    #[must_use]
    pub fn loops(self) -> bool {
        !matches!(self, Self::Victory | Self::GameOver | Self::None)
    }

    /// Check if this is a combat track.
    #[must_use]
    pub fn is_combat(self) -> bool {
        matches!(self, Self::Combat | Self::Boss)
    }

    /// Get default volume (0.0-1.0).
    #[must_use]
    pub fn default_volume(self) -> f32 {
        match self {
            Self::Combat | Self::Boss => 0.7,
            Self::Victory => 0.8,
            Self::GameOver => 0.6,
            Self::Night => 0.4,
            _ => 0.5,
        }
    }

    /// Get all tracks.
    #[must_use]
    pub const fn all() -> [Self; 19] {
        [
            Self::Menu,
            Self::Exploration,
            Self::Forest,
            Self::Desert,
            Self::Snow,
            Self::Swamp,
            Self::Mountain,
            Self::Ocean,
            Self::Village,
            Self::Dungeon,
            Self::Cave,
            Self::Night,
            Self::Combat,
            Self::Boss,
            Self::Victory,
            Self::GameOver,
            Self::Peaceful,
            Self::Mystery,
            Self::None,
        ]
    }
}

impl Default for MusicTrack {
    fn default() -> Self {
        Self::Exploration
    }
}

/// Music playback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicState {
    /// Not playing.
    Stopped,
    /// Playing normally.
    Playing,
    /// Paused.
    Paused,
    /// Fading in.
    FadingIn,
    /// Fading out.
    FadingOut,
    /// Crossfading to another track.
    Crossfading,
}

/// Configuration for music transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionConfig {
    /// Fade in duration in seconds.
    pub fade_in: f32,
    /// Fade out duration in seconds.
    pub fade_out: f32,
    /// Crossfade duration in seconds.
    pub crossfade: f32,
    /// Combat music fade in (faster).
    pub combat_fade_in: f32,
    /// Combat music fade out.
    pub combat_fade_out: f32,
}

impl TransitionConfig {
    /// Create default transition config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fade_in: 2.0,
            fade_out: 2.0,
            crossfade: 3.0,
            combat_fade_in: 0.5,
            combat_fade_out: 3.0,
        }
    }

    /// Create quick transitions.
    #[must_use]
    pub fn quick() -> Self {
        Self {
            fade_in: 0.5,
            fade_out: 0.5,
            crossfade: 1.0,
            combat_fade_in: 0.3,
            combat_fade_out: 1.5,
        }
    }

    /// Create slow/cinematic transitions.
    #[must_use]
    pub fn cinematic() -> Self {
        Self {
            fade_in: 4.0,
            fade_out: 4.0,
            crossfade: 5.0,
            combat_fade_in: 1.0,
            combat_fade_out: 5.0,
        }
    }
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Current track being played.
#[derive(Debug, Clone)]
pub struct ActiveTrack {
    /// The track.
    pub track: MusicTrack,
    /// Current volume (0.0-1.0).
    pub volume: f32,
    /// Target volume.
    pub target_volume: f32,
    /// Playback position in seconds.
    pub position: f32,
    /// Playback state.
    pub state: MusicState,
}

impl ActiveTrack {
    /// Create a new active track.
    #[must_use]
    pub fn new(track: MusicTrack) -> Self {
        Self {
            track,
            volume: 0.0,
            target_volume: track.default_volume(),
            position: 0.0,
            state: MusicState::Stopped,
        }
    }

    /// Start playing with fade in.
    pub fn play(&mut self) {
        self.state = MusicState::FadingIn;
    }

    /// Start playing immediately at full volume.
    pub fn play_immediate(&mut self) {
        self.volume = self.target_volume;
        self.state = MusicState::Playing;
    }

    /// Pause playback.
    pub fn pause(&mut self) {
        if self.state == MusicState::Playing {
            self.state = MusicState::Paused;
        }
    }

    /// Resume playback.
    pub fn resume(&mut self) {
        if self.state == MusicState::Paused {
            self.state = MusicState::Playing;
        }
    }

    /// Stop with fade out.
    pub fn stop(&mut self) {
        self.state = MusicState::FadingOut;
    }

    /// Stop immediately.
    pub fn stop_immediate(&mut self) {
        self.volume = 0.0;
        self.state = MusicState::Stopped;
    }

    /// Check if track is active (playing or transitioning).
    #[must_use]
    pub fn is_active(&self) -> bool {
        !matches!(self.state, MusicState::Stopped)
    }

    /// Update fade progress.
    pub fn update_fade(&mut self, delta: f32, fade_speed: f32) {
        match self.state {
            MusicState::FadingIn => {
                self.volume += fade_speed * delta;
                if self.volume >= self.target_volume {
                    self.volume = self.target_volume;
                    self.state = MusicState::Playing;
                }
            },
            MusicState::FadingOut | MusicState::Crossfading => {
                self.volume -= fade_speed * delta;
                if self.volume <= 0.0 {
                    self.volume = 0.0;
                    self.state = MusicState::Stopped;
                }
            },
            _ => {},
        }
    }
}

/// Biome music mapping.
#[derive(Debug, Clone)]
pub struct BiomeMusicMap {
    /// Day tracks for each biome.
    day_tracks: Vec<(BiomeType, MusicTrack)>,
    /// Night tracks for each biome.
    night_tracks: Vec<(BiomeType, MusicTrack)>,
}

impl BiomeMusicMap {
    /// Create default biome music mapping.
    #[must_use]
    pub fn new() -> Self {
        let day_tracks = vec![
            (BiomeType::Forest, MusicTrack::Forest),
            (BiomeType::Desert, MusicTrack::Desert),
            (BiomeType::Swamp, MusicTrack::Swamp),
            (BiomeType::Mountain, MusicTrack::Mountain),
            (BiomeType::Plains, MusicTrack::Exploration),
            (BiomeType::Lake, MusicTrack::Peaceful),
        ];

        let night_tracks = vec![
            (BiomeType::Forest, MusicTrack::Night),
            (BiomeType::Desert, MusicTrack::Night),
            (BiomeType::Swamp, MusicTrack::Mystery),
            (BiomeType::Mountain, MusicTrack::Night),
            (BiomeType::Plains, MusicTrack::Night),
            (BiomeType::Lake, MusicTrack::Night),
        ];

        Self {
            day_tracks,
            night_tracks,
        }
    }

    /// Get track for biome during day.
    #[must_use]
    pub fn day_track(&self, biome: BiomeType) -> MusicTrack {
        self.day_tracks
            .iter()
            .find(|(b, _)| *b == biome)
            .map_or(MusicTrack::Exploration, |(_, t)| *t)
    }

    /// Get track for biome during night.
    #[must_use]
    pub fn night_track(&self, biome: BiomeType) -> MusicTrack {
        self.night_tracks
            .iter()
            .find(|(b, _)| *b == biome)
            .map_or(MusicTrack::Night, |(_, t)| *t)
    }

    /// Get track for biome based on time of day.
    #[must_use]
    pub fn track_for_biome(&self, biome: BiomeType, is_night: bool) -> MusicTrack {
        if is_night {
            self.night_track(biome)
        } else {
            self.day_track(biome)
        }
    }

    /// Set custom day track for biome.
    pub fn set_day_track(&mut self, biome: BiomeType, track: MusicTrack) {
        if let Some(entry) = self.day_tracks.iter_mut().find(|(b, _)| *b == biome) {
            entry.1 = track;
        } else {
            self.day_tracks.push((biome, track));
        }
    }

    /// Set custom night track for biome.
    pub fn set_night_track(&mut self, biome: BiomeType, track: MusicTrack) {
        if let Some(entry) = self.night_tracks.iter_mut().find(|(b, _)| *b == biome) {
            entry.1 = track;
        } else {
            self.night_tracks.push((biome, track));
        }
    }
}

impl Default for BiomeMusicMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Music manager for the game.
#[derive(Debug)]
pub struct MusicManager {
    /// Currently playing track.
    current: Option<ActiveTrack>,
    /// Track being faded in during crossfade.
    incoming: Option<ActiveTrack>,
    /// Previous track (for returning from combat).
    previous: Option<MusicTrack>,
    /// Biome music mapping.
    biome_map: BiomeMusicMap,
    /// Transition configuration.
    transition: TransitionConfig,
    /// Master music volume.
    master_volume: f32,
    /// Whether music is enabled.
    enabled: bool,
    /// Whether in combat mode.
    in_combat: bool,
    /// Combat intensity (for boss music).
    combat_intensity: f32,
    /// Current biome.
    current_biome: Option<BiomeType>,
    /// Is it night time?
    is_night: bool,
    /// Is player in village?
    in_village: bool,
    /// Is player in dungeon?
    in_dungeon: bool,
}

impl MusicManager {
    /// Create a new music manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: None,
            incoming: None,
            previous: None,
            biome_map: BiomeMusicMap::new(),
            transition: TransitionConfig::new(),
            master_volume: 1.0,
            enabled: true,
            in_combat: false,
            combat_intensity: 0.0,
            current_biome: None,
            is_night: false,
            in_village: false,
            in_dungeon: false,
        }
    }

    /// Get biome map for customization.
    #[must_use]
    pub fn biome_map(&self) -> &BiomeMusicMap {
        &self.biome_map
    }

    /// Get mutable biome map.
    pub fn biome_map_mut(&mut self) -> &mut BiomeMusicMap {
        &mut self.biome_map
    }

    /// Get transition config.
    #[must_use]
    pub fn transition_config(&self) -> &TransitionConfig {
        &self.transition
    }

    /// Set transition config.
    pub fn set_transition_config(&mut self, config: TransitionConfig) {
        self.transition = config;
    }

    /// Get master volume.
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Set master volume.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Enable/disable music.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.stop_immediate();
        }
    }

    /// Check if music is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get current track.
    #[must_use]
    pub fn current_track(&self) -> Option<MusicTrack> {
        self.current.as_ref().map(|t| t.track)
    }

    /// Get effective volume (current track volume * master).
    #[must_use]
    pub fn effective_volume(&self) -> f32 {
        self.current
            .as_ref()
            .map_or(0.0, |t| t.volume * self.master_volume)
    }

    /// Play a track with crossfade.
    pub fn play(&mut self, track: MusicTrack) {
        if !self.enabled {
            return;
        }

        // Already playing this track
        if self.current.as_ref().is_some_and(|c| c.track == track) {
            return;
        }

        // Start crossfade
        if let Some(mut current) = self.current.take() {
            current.state = MusicState::Crossfading;
            self.incoming = Some(current);
        }

        let mut new_track = ActiveTrack::new(track);
        new_track.play();
        self.current = Some(new_track);
    }

    /// Play a track immediately (no fade).
    pub fn play_immediate(&mut self, track: MusicTrack) {
        if !self.enabled {
            return;
        }

        self.stop_immediate();
        let mut new_track = ActiveTrack::new(track);
        new_track.play_immediate();
        self.current = Some(new_track);
    }

    /// Stop current track with fade.
    pub fn stop(&mut self) {
        if let Some(ref mut current) = self.current {
            current.stop();
        }
    }

    /// Stop current track immediately.
    pub fn stop_immediate(&mut self) {
        self.current = None;
        self.incoming = None;
    }

    /// Pause music.
    pub fn pause(&mut self) {
        if let Some(ref mut current) = self.current {
            current.pause();
        }
    }

    /// Resume music.
    pub fn resume(&mut self) {
        if let Some(ref mut current) = self.current {
            current.resume();
        }
    }

    /// Enter combat mode.
    pub fn enter_combat(&mut self, is_boss: bool) {
        if self.in_combat {
            // Already in combat, check if boss upgrade needed
            if is_boss && self.current_track() != Some(MusicTrack::Boss) {
                self.combat_intensity = 1.0;
                self.play(MusicTrack::Boss);
            }
            return;
        }

        self.in_combat = true;
        self.combat_intensity = if is_boss { 1.0 } else { 0.5 };

        // Store previous track
        self.previous = self.current_track();

        // Play combat music
        let track = if is_boss {
            MusicTrack::Boss
        } else {
            MusicTrack::Combat
        };
        self.play(track);
    }

    /// Exit combat mode.
    pub fn exit_combat(&mut self) {
        if !self.in_combat {
            return;
        }

        self.in_combat = false;
        self.combat_intensity = 0.0;

        // Return to previous track
        if let Some(prev) = self.previous.take() {
            self.play(prev);
        } else {
            self.update_ambient_music();
        }
    }

    /// Set current biome.
    pub fn set_biome(&mut self, biome: BiomeType) {
        if self.current_biome == Some(biome) {
            return;
        }
        self.current_biome = Some(biome);

        if !self.in_combat {
            self.update_ambient_music();
        }
    }

    /// Set time of day.
    pub fn set_night(&mut self, is_night: bool) {
        if self.is_night == is_night {
            return;
        }
        self.is_night = is_night;

        if !self.in_combat {
            self.update_ambient_music();
        }
    }

    /// Set village state.
    pub fn set_in_village(&mut self, in_village: bool) {
        if self.in_village == in_village {
            return;
        }
        self.in_village = in_village;

        if !self.in_combat {
            self.update_ambient_music();
        }
    }

    /// Set dungeon state.
    pub fn set_in_dungeon(&mut self, in_dungeon: bool) {
        if self.in_dungeon == in_dungeon {
            return;
        }
        self.in_dungeon = in_dungeon;

        if !self.in_combat {
            self.update_ambient_music();
        }
    }

    /// Update ambient music based on current state.
    fn update_ambient_music(&mut self) {
        let track = self.determine_ambient_track();
        self.play(track);
    }

    /// Determine which ambient track to play.
    fn determine_ambient_track(&self) -> MusicTrack {
        // Priority: dungeon > village > biome
        if self.in_dungeon {
            return MusicTrack::Dungeon;
        }

        if self.in_village {
            return MusicTrack::Village;
        }

        if let Some(biome) = self.current_biome {
            return self.biome_map.track_for_biome(biome, self.is_night);
        }

        if self.is_night {
            MusicTrack::Night
        } else {
            MusicTrack::Exploration
        }
    }

    /// Update music system.
    pub fn update(&mut self, delta: f32) {
        // Update current track fades
        if let Some(ref mut current) = self.current {
            let fade_speed = if current.track.is_combat() {
                1.0 / self.transition.combat_fade_in
            } else {
                1.0 / self.transition.fade_in
            };
            current.update_fade(delta, fade_speed);

            // Update position
            if matches!(current.state, MusicState::Playing) {
                current.position += delta;
            }

            // Clean up stopped track
            if matches!(current.state, MusicState::Stopped) {
                self.current = None;
            }
        }

        // Update outgoing track during crossfade
        if let Some(ref mut incoming) = self.incoming {
            let fade_speed = 1.0 / self.transition.crossfade;
            incoming.update_fade(delta, fade_speed);

            if matches!(incoming.state, MusicState::Stopped) {
                self.incoming = None;
            }
        }
    }

    /// Get current playback info for audio backend.
    #[must_use]
    pub fn playback_info(&self) -> Option<MusicPlaybackInfo> {
        self.current.as_ref().map(|track| MusicPlaybackInfo {
            track: track.track,
            volume: track.volume * self.master_volume,
            position: track.position,
            state: track.state,
            loops: track.track.loops(),
        })
    }

    /// Get outgoing track info (during crossfade).
    #[must_use]
    pub fn outgoing_info(&self) -> Option<MusicPlaybackInfo> {
        self.incoming.as_ref().map(|track| MusicPlaybackInfo {
            track: track.track,
            volume: track.volume * self.master_volume,
            position: track.position,
            state: track.state,
            loops: track.track.loops(),
        })
    }
}

impl Default for MusicManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Music playback information for audio backend.
#[derive(Debug, Clone)]
pub struct MusicPlaybackInfo {
    /// The track to play.
    pub track: MusicTrack,
    /// Current volume (0.0-1.0).
    pub volume: f32,
    /// Playback position in seconds.
    pub position: f32,
    /// Playback state.
    pub state: MusicState,
    /// Whether to loop.
    pub loops: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_music_track_asset_paths() {
        for track in MusicTrack::all() {
            if track != MusicTrack::None {
                assert!(
                    track.asset_path().is_some(),
                    "{:?} should have asset path",
                    track
                );
            }
        }
    }

    #[test]
    fn test_music_track_none_has_no_path() {
        assert!(MusicTrack::None.asset_path().is_none());
    }

    #[test]
    fn test_music_track_loops() {
        assert!(MusicTrack::Forest.loops());
        assert!(MusicTrack::Combat.loops());
        assert!(!MusicTrack::Victory.loops());
        assert!(!MusicTrack::GameOver.loops());
    }

    #[test]
    fn test_music_track_is_combat() {
        assert!(MusicTrack::Combat.is_combat());
        assert!(MusicTrack::Boss.is_combat());
        assert!(!MusicTrack::Forest.is_combat());
    }

    #[test]
    fn test_transition_config_defaults() {
        let config = TransitionConfig::new();
        assert!(config.fade_in > 0.0);
        assert!(config.fade_out > 0.0);
        assert!(config.crossfade > 0.0);
    }

    #[test]
    fn test_active_track_fade_in() {
        let mut track = ActiveTrack::new(MusicTrack::Forest);
        track.play();
        assert_eq!(track.state, MusicState::FadingIn);

        // Simulate fade
        track.update_fade(1.0, 1.0);
        assert!(track.volume > 0.0);
    }

    #[test]
    fn test_active_track_stop() {
        let mut track = ActiveTrack::new(MusicTrack::Forest);
        track.play_immediate();
        assert_eq!(track.state, MusicState::Playing);

        track.stop();
        assert_eq!(track.state, MusicState::FadingOut);
    }

    #[test]
    fn test_biome_music_map() {
        let map = BiomeMusicMap::new();
        assert_eq!(map.day_track(BiomeType::Forest), MusicTrack::Forest);
        assert_eq!(map.day_track(BiomeType::Desert), MusicTrack::Desert);
    }

    #[test]
    fn test_biome_music_map_night() {
        let map = BiomeMusicMap::new();
        assert_eq!(map.night_track(BiomeType::Forest), MusicTrack::Night);
    }

    #[test]
    fn test_biome_music_map_custom() {
        let mut map = BiomeMusicMap::new();
        map.set_day_track(BiomeType::Forest, MusicTrack::Mystery);
        assert_eq!(map.day_track(BiomeType::Forest), MusicTrack::Mystery);
    }

    #[test]
    fn test_music_manager_play() {
        let mut manager = MusicManager::new();
        manager.play(MusicTrack::Forest);
        assert_eq!(manager.current_track(), Some(MusicTrack::Forest));
    }

    #[test]
    fn test_music_manager_disabled() {
        let mut manager = MusicManager::new();
        manager.set_enabled(false);
        manager.play(MusicTrack::Forest);
        assert!(manager.current_track().is_none());
    }

    #[test]
    fn test_music_manager_combat() {
        let mut manager = MusicManager::new();
        manager.play(MusicTrack::Forest);

        manager.enter_combat(false);
        assert!(manager.in_combat);
        assert_eq!(manager.current_track(), Some(MusicTrack::Combat));

        manager.exit_combat();
        assert!(!manager.in_combat);
        assert_eq!(manager.current_track(), Some(MusicTrack::Forest));
    }

    #[test]
    fn test_music_manager_boss_combat() {
        let mut manager = MusicManager::new();
        manager.enter_combat(true);
        assert_eq!(manager.current_track(), Some(MusicTrack::Boss));
    }

    #[test]
    fn test_music_manager_biome_change() {
        let mut manager = MusicManager::new();
        manager.set_biome(BiomeType::Desert);
        assert_eq!(manager.current_track(), Some(MusicTrack::Desert));

        manager.set_biome(BiomeType::Forest);
        assert_eq!(manager.current_track(), Some(MusicTrack::Forest));
    }

    #[test]
    fn test_music_manager_day_night() {
        let mut manager = MusicManager::new();
        manager.set_biome(BiomeType::Forest);
        assert_eq!(manager.current_track(), Some(MusicTrack::Forest));

        manager.set_night(true);
        assert_eq!(manager.current_track(), Some(MusicTrack::Night));
    }

    #[test]
    fn test_music_manager_village_override() {
        let mut manager = MusicManager::new();
        manager.set_biome(BiomeType::Forest);
        manager.set_in_village(true);
        assert_eq!(manager.current_track(), Some(MusicTrack::Village));
    }

    #[test]
    fn test_music_manager_dungeon_override() {
        let mut manager = MusicManager::new();
        manager.set_in_village(true);
        manager.set_in_dungeon(true);
        assert_eq!(manager.current_track(), Some(MusicTrack::Dungeon));
    }

    #[test]
    fn test_music_manager_update() {
        let mut manager = MusicManager::new();
        manager.play_immediate(MusicTrack::Forest);
        manager.update(0.1);

        let info = manager.playback_info().expect("should have info");
        assert!(info.position > 0.0);
    }

    #[test]
    fn test_music_manager_volume() {
        let mut manager = MusicManager::new();
        manager.set_master_volume(0.5);
        manager.play_immediate(MusicTrack::Forest);

        let effective = manager.effective_volume();
        assert!(effective <= 0.5);
    }
}
