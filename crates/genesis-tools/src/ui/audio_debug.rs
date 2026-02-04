//! Audio debug UI for monitoring and visualizing audio state.
//!
//! Provides debug visualization including:
//! - Currently playing sounds list
//! - Active music track display
//! - Ambient layers status
//! - Spatial audio sources visualization
//! - Audio channel usage meters
//! - Peak level indicators

use egui::{Color32, Pos2, Rect, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Unique identifier for a sound instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SoundInstanceId(pub u64);

impl SoundInstanceId {
    /// Create a new sound instance ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Category of sound for the debug display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoundCategory {
    /// Background music.
    Music,
    /// Sound effects.
    Sfx,
    /// Ambient/environmental sounds.
    Ambient,
    /// UI sounds.
    Ui,
    /// Voice/dialogue.
    Voice,
}

impl SoundCategory {
    /// Get display name for the category.
    pub fn display_name(&self) -> &'static str {
        match self {
            SoundCategory::Music => "Music",
            SoundCategory::Sfx => "SFX",
            SoundCategory::Ambient => "Ambient",
            SoundCategory::Ui => "UI",
            SoundCategory::Voice => "Voice",
        }
    }

    /// Get display color for the category.
    pub fn color(&self) -> Color32 {
        match self {
            SoundCategory::Music => Color32::from_rgb(100, 180, 220),
            SoundCategory::Sfx => Color32::from_rgb(220, 180, 100),
            SoundCategory::Ambient => Color32::from_rgb(100, 220, 150),
            SoundCategory::Ui => Color32::from_rgb(180, 150, 220),
            SoundCategory::Voice => Color32::from_rgb(220, 150, 150),
        }
    }
}

/// State of a playing sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// Sound is currently playing.
    Playing,
    /// Sound is paused.
    Paused,
    /// Sound is fading in.
    FadingIn,
    /// Sound is fading out.
    FadingOut,
    /// Sound is stopped.
    Stopped,
}

impl PlaybackState {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            PlaybackState::Playing => "Playing",
            PlaybackState::Paused => "Paused",
            PlaybackState::FadingIn => "Fading In",
            PlaybackState::FadingOut => "Fading Out",
            PlaybackState::Stopped => "Stopped",
        }
    }

    /// Get display symbol.
    pub fn symbol(&self) -> &'static str {
        match self {
            PlaybackState::Playing => "▶",
            PlaybackState::Paused => "⏸",
            PlaybackState::FadingIn => "↗",
            PlaybackState::FadingOut => "↘",
            PlaybackState::Stopped => "⏹",
        }
    }
}

/// Information about a currently playing sound.
#[derive(Debug, Clone)]
pub struct PlayingSoundInfo {
    /// Unique instance ID.
    pub id: SoundInstanceId,
    /// Sound name/path.
    pub name: String,
    /// Sound category.
    pub category: SoundCategory,
    /// Current playback state.
    pub state: PlaybackState,
    /// Current volume (0.0 - 1.0).
    pub volume: f32,
    /// Current playback position in seconds.
    pub position: f32,
    /// Total duration in seconds.
    pub duration: f32,
    /// Whether sound is looping.
    pub looping: bool,
    /// World position for spatial sounds (None for non-spatial).
    pub world_position: Option<(f32, f32, f32)>,
    /// Current pan value (-1.0 left to 1.0 right).
    pub pan: f32,
}

impl PlayingSoundInfo {
    /// Create a new playing sound info.
    pub fn new(id: SoundInstanceId, name: String, category: SoundCategory) -> Self {
        Self {
            id,
            name,
            category,
            state: PlaybackState::Playing,
            volume: 1.0,
            position: 0.0,
            duration: 0.0,
            looping: false,
            world_position: None,
            pan: 0.0,
        }
    }

    /// Get playback progress (0.0 - 1.0).
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.position / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if this is a spatial sound.
    pub fn is_spatial(&self) -> bool {
        self.world_position.is_some()
    }
}

/// Information about the currently active music track.
#[derive(Debug, Clone)]
pub struct MusicTrackInfo {
    /// Track name.
    pub name: String,
    /// Artist/composer (optional).
    pub artist: Option<String>,
    /// Current playback position in seconds.
    pub position: f32,
    /// Total duration in seconds.
    pub duration: f32,
    /// Current volume.
    pub volume: f32,
    /// Whether track is looping.
    pub looping: bool,
    /// Crossfade progress (0.0 = not crossfading, 1.0 = complete).
    pub crossfade_progress: Option<f32>,
}

impl MusicTrackInfo {
    /// Create new music track info.
    pub fn new(name: String) -> Self {
        Self {
            name,
            artist: None,
            position: 0.0,
            duration: 0.0,
            volume: 1.0,
            looping: true,
            crossfade_progress: None,
        }
    }

    /// Get formatted position (MM:SS).
    pub fn formatted_position(&self) -> String {
        format_time(self.position)
    }

    /// Get formatted duration (MM:SS).
    pub fn formatted_duration(&self) -> String {
        format_time(self.duration)
    }

    /// Get playback progress (0.0 - 1.0).
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.position / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Information about an ambient layer.
#[derive(Debug, Clone)]
pub struct AmbientLayerInfo {
    /// Layer name.
    pub name: String,
    /// Current volume (0.0 - 1.0).
    pub volume: f32,
    /// Target volume for blending.
    pub target_volume: f32,
    /// Whether layer is active.
    pub active: bool,
}

impl AmbientLayerInfo {
    /// Create new ambient layer info.
    pub fn new(name: String) -> Self {
        Self {
            name,
            volume: 0.0,
            target_volume: 0.0,
            active: false,
        }
    }

    /// Check if layer is transitioning.
    pub fn is_transitioning(&self) -> bool {
        (self.volume - self.target_volume).abs() > 0.01
    }
}

/// Audio channel usage information.
#[derive(Debug, Clone, Copy)]
pub struct ChannelUsage {
    /// Number of active channels.
    pub active: u32,
    /// Maximum available channels.
    pub max: u32,
    /// Peak level (0.0 - 1.0).
    pub peak_level: f32,
    /// Average level (0.0 - 1.0).
    pub average_level: f32,
}

impl Default for ChannelUsage {
    fn default() -> Self {
        Self {
            active: 0,
            max: 32,
            peak_level: 0.0,
            average_level: 0.0,
        }
    }
}

impl ChannelUsage {
    /// Get usage percentage.
    pub fn usage_percent(&self) -> f32 {
        if self.max > 0 {
            (self.active as f32 / self.max as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Check if channels are near capacity.
    pub fn is_near_capacity(&self) -> bool {
        self.usage_percent() >= 80.0
    }
}

/// Spatial audio source for minimap visualization.
#[derive(Debug, Clone)]
pub struct SpatialAudioSource {
    /// Source ID.
    pub id: SoundInstanceId,
    /// Sound name.
    pub name: String,
    /// World position (x, y, z).
    pub position: (f32, f32, f32),
    /// Sound radius/range.
    pub radius: f32,
    /// Current volume.
    pub volume: f32,
    /// Sound category.
    pub category: SoundCategory,
}

/// Complete audio debug state snapshot.
#[derive(Debug, Clone, Default)]
pub struct AudioDebugState {
    /// Currently playing sounds.
    pub playing_sounds: Vec<PlayingSoundInfo>,
    /// Current music track.
    pub music_track: Option<MusicTrackInfo>,
    /// Active ambient layers.
    pub ambient_layers: Vec<AmbientLayerInfo>,
    /// Spatial audio sources.
    pub spatial_sources: Vec<SpatialAudioSource>,
    /// Channel usage.
    pub channel_usage: ChannelUsage,
    /// Listener position (x, y, z).
    pub listener_position: (f32, f32, f32),
    /// Listener rotation (yaw in radians).
    pub listener_rotation: f32,
}

/// Configuration for the audio debug panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDebugConfig {
    /// Show playing sounds list.
    pub show_playing_sounds: bool,
    /// Show music track info.
    pub show_music_track: bool,
    /// Show ambient layers.
    pub show_ambient_layers: bool,
    /// Show channel usage.
    pub show_channel_usage: bool,
    /// Show peak levels.
    pub show_peak_levels: bool,
    /// Maximum sounds to show in list.
    pub max_sounds_displayed: usize,
    /// Meter width in pixels.
    pub meter_width: f32,
    /// Peak level warning threshold.
    pub peak_warning_threshold: f32,
}

impl Default for AudioDebugConfig {
    fn default() -> Self {
        Self {
            show_playing_sounds: true,
            show_music_track: true,
            show_ambient_layers: true,
            show_channel_usage: true,
            show_peak_levels: true,
            max_sounds_displayed: 10,
            meter_width: 100.0,
            peak_warning_threshold: 0.9,
        }
    }
}

/// Audio debug panel widget.
#[derive(Debug)]
pub struct AudioDebugPanel {
    /// Current audio state.
    pub state: AudioDebugState,
    /// Panel configuration.
    pub config: AudioDebugConfig,
    /// Whether the panel is open.
    pub open: bool,
}

impl Default for AudioDebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDebugPanel {
    /// Create a new audio debug panel.
    pub fn new() -> Self {
        Self {
            state: AudioDebugState::default(),
            config: AudioDebugConfig::default(),
            open: false,
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: AudioDebugConfig) -> Self {
        Self {
            state: AudioDebugState::default(),
            config,
            open: false,
        }
    }

    /// Update the audio state.
    pub fn update_state(&mut self, state: AudioDebugState) {
        self.state = state;
    }

    /// Open the panel.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the panel.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggle panel visibility.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Render the debug panel.
    pub fn show(&mut self, ui: &mut Ui) {
        if !self.open {
            return;
        }

        ui.vertical(|ui| {
            ui.heading("Audio Debug");
            ui.separator();

            // Channel usage
            if self.config.show_channel_usage {
                self.show_channel_usage(ui);
                ui.separator();
            }

            // Peak levels
            if self.config.show_peak_levels {
                self.show_peak_levels(ui);
                ui.separator();
            }

            // Music track
            if self.config.show_music_track {
                self.show_music_track(ui);
                ui.separator();
            }

            // Ambient layers
            if self.config.show_ambient_layers {
                self.show_ambient_layers(ui);
                ui.separator();
            }

            // Playing sounds
            if self.config.show_playing_sounds {
                self.show_playing_sounds(ui);
            }
        });
    }

    /// Show channel usage section.
    fn show_channel_usage(&self, ui: &mut Ui) {
        let usage = &self.state.channel_usage;

        ui.label(format!(
            "Channels: {}/{} ({:.0}%)",
            usage.active,
            usage.max,
            usage.usage_percent()
        ));

        // Channel usage bar
        let bar_rect = ui.available_rect_before_wrap();
        let bar_height = 10.0;
        let bar_rect =
            Rect::from_min_size(bar_rect.min, Vec2::new(self.config.meter_width, bar_height));

        let fill_width = bar_rect.width() * (usage.usage_percent() / 100.0);
        let fill_color = if usage.is_near_capacity() {
            Color32::from_rgb(220, 100, 100)
        } else {
            Color32::from_rgb(100, 180, 100)
        };

        ui.painter()
            .rect_filled(bar_rect, 2.0, Color32::from_gray(40));
        ui.painter().rect_filled(
            Rect::from_min_size(bar_rect.min, Vec2::new(fill_width, bar_height)),
            2.0,
            fill_color,
        );

        ui.allocate_space(Vec2::new(self.config.meter_width, bar_height + 4.0));
    }

    /// Show peak levels section.
    fn show_peak_levels(&self, ui: &mut Ui) {
        let usage = &self.state.channel_usage;

        ui.horizontal(|ui| {
            ui.label("Peak:");
            self.show_level_meter(ui, usage.peak_level);

            ui.label("Avg:");
            self.show_level_meter(ui, usage.average_level);
        });
    }

    /// Show a single level meter.
    fn show_level_meter(&self, ui: &mut Ui, level: f32) {
        let meter_width = 60.0;
        let meter_height = 12.0;

        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(meter_width, meter_height), egui::Sense::hover());

        // Background
        ui.painter().rect_filled(rect, 2.0, Color32::from_gray(30));

        // Level fill
        let fill_width = rect.width() * level;
        let color = if level >= self.config.peak_warning_threshold {
            Color32::from_rgb(220, 80, 80)
        } else if level >= 0.7 {
            Color32::from_rgb(220, 180, 80)
        } else {
            Color32::from_rgb(80, 180, 80)
        };

        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(fill_width, meter_height)),
            2.0,
            color,
        );

        // Level text
        let text = format!("{:.0}%", level * 100.0);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
    }

    /// Show music track section.
    fn show_music_track(&self, ui: &mut Ui) {
        ui.label("🎵 Music");

        if let Some(track) = &self.state.music_track {
            ui.horizontal(|ui| {
                ui.label(&track.name);
                if let Some(artist) = &track.artist {
                    ui.weak(format!("- {artist}"));
                }
            });

            // Progress bar
            ui.horizontal(|ui| {
                ui.label(track.formatted_position());

                let bar_width = 100.0;
                let bar_height = 8.0;
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

                ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        rect.min,
                        Vec2::new(rect.width() * track.progress(), bar_height),
                    ),
                    2.0,
                    SoundCategory::Music.color(),
                );

                ui.label(track.formatted_duration());

                if track.looping {
                    ui.label("🔁");
                }
            });

            if let Some(crossfade) = track.crossfade_progress {
                ui.label(format!("Crossfading: {:.0}%", crossfade * 100.0));
            }
        } else {
            ui.weak("No music playing");
        }
    }

    /// Show ambient layers section.
    fn show_ambient_layers(&self, ui: &mut Ui) {
        ui.label("🌿 Ambient Layers");

        if self.state.ambient_layers.is_empty() {
            ui.weak("No ambient layers");
            return;
        }

        for layer in &self.state.ambient_layers {
            ui.horizontal(|ui| {
                let status = if layer.active { "●" } else { "○" };
                let color = if layer.active {
                    SoundCategory::Ambient.color()
                } else {
                    Color32::GRAY
                };
                ui.colored_label(color, status);
                ui.label(&layer.name);

                // Volume indicator
                let vol_text = format!("{:.0}%", layer.volume * 100.0);
                ui.weak(vol_text);

                if layer.is_transitioning() {
                    ui.weak(format!("→ {:.0}%", layer.target_volume * 100.0));
                }
            });
        }
    }

    /// Show playing sounds section.
    fn show_playing_sounds(&self, ui: &mut Ui) {
        ui.label(format!(
            "🔊 Playing Sounds ({})",
            self.state.playing_sounds.len()
        ));

        if self.state.playing_sounds.is_empty() {
            ui.weak("No sounds playing");
            return;
        }

        let sounds_to_show = self
            .state
            .playing_sounds
            .iter()
            .take(self.config.max_sounds_displayed);

        for sound in sounds_to_show {
            ui.horizontal(|ui| {
                // State indicator
                ui.colored_label(sound.category.color(), sound.state.symbol());

                // Category badge
                ui.colored_label(sound.category.color(), sound.category.display_name());

                // Sound name (truncated)
                let name = truncate_string(&sound.name, 20);
                ui.label(name);

                // Volume
                ui.weak(format!("{:.0}%", sound.volume * 100.0));

                // Spatial indicator
                if sound.is_spatial() {
                    ui.weak("📍");
                }

                // Loop indicator
                if sound.looping {
                    ui.weak("🔁");
                }
            });
        }

        let remaining = self
            .state
            .playing_sounds
            .len()
            .saturating_sub(self.config.max_sounds_displayed);
        if remaining > 0 {
            ui.weak(format!("... and {remaining} more"));
        }
    }
}

/// Configuration for spatial audio minimap overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAudioMinimapConfig {
    /// Whether to show spatial sources.
    pub enabled: bool,
    /// Source marker size.
    pub marker_size: f32,
    /// Show source ranges.
    pub show_ranges: bool,
    /// Range circle alpha.
    pub range_alpha: u8,
    /// Show source labels.
    pub show_labels: bool,
}

impl Default for SpatialAudioMinimapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            marker_size: 6.0,
            show_ranges: true,
            range_alpha: 40,
            show_labels: false,
        }
    }
}

/// Spatial audio minimap overlay widget.
#[derive(Debug)]
pub struct SpatialAudioMinimapOverlay {
    /// Configuration.
    pub config: SpatialAudioMinimapConfig,
    /// Spatial sources to display.
    sources: Vec<SpatialAudioSource>,
    /// Listener position.
    listener_pos: (f32, f32, f32),
    /// Listener rotation.
    listener_rot: f32,
}

impl Default for SpatialAudioMinimapOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialAudioMinimapOverlay {
    /// Create a new spatial audio minimap overlay.
    pub fn new() -> Self {
        Self {
            config: SpatialAudioMinimapConfig::default(),
            sources: Vec::new(),
            listener_pos: (0.0, 0.0, 0.0),
            listener_rot: 0.0,
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: SpatialAudioMinimapConfig) -> Self {
        Self {
            config,
            sources: Vec::new(),
            listener_pos: (0.0, 0.0, 0.0),
            listener_rot: 0.0,
        }
    }

    /// Update sources and listener.
    pub fn update(
        &mut self,
        sources: Vec<SpatialAudioSource>,
        listener_pos: (f32, f32, f32),
        listener_rot: f32,
    ) {
        self.sources = sources;
        self.listener_pos = listener_pos;
        self.listener_rot = listener_rot;
    }

    /// Render the overlay onto a minimap.
    ///
    /// `world_to_minimap` converts world (x, y) to minimap screen position.
    /// `scale` is the world units per pixel for range circles.
    pub fn show(
        &self,
        ui: &mut Ui,
        minimap_rect: Rect,
        world_to_minimap: impl Fn(f32, f32) -> Pos2,
        scale: f32,
    ) {
        if !self.config.enabled {
            return;
        }

        let painter = ui.painter_at(minimap_rect);

        // Draw range circles and source markers
        for source in &self.sources {
            let pos = world_to_minimap(source.position.0, source.position.1);

            // Skip if outside minimap
            if !minimap_rect.contains(pos) {
                continue;
            }

            let color = source.category.color();

            // Draw range circle
            if self.config.show_ranges && source.radius > 0.0 {
                let range_pixels = source.radius / scale;
                let range_color = Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    self.config.range_alpha,
                );
                painter.circle_stroke(pos, range_pixels, egui::Stroke::new(1.0, range_color));
            }

            // Draw source marker
            painter.circle_filled(pos, self.config.marker_size, color);

            // Draw volume indicator (smaller inner circle)
            let inner_size = self.config.marker_size * source.volume * 0.7;
            painter.circle_filled(pos, inner_size, Color32::WHITE);

            // Draw label
            if self.config.show_labels {
                let label_pos = pos + Vec2::new(self.config.marker_size + 2.0, 0.0);
                painter.text(
                    label_pos,
                    egui::Align2::LEFT_CENTER,
                    truncate_string(&source.name, 10),
                    egui::FontId::proportional(9.0),
                    Color32::WHITE,
                );
            }
        }

        // Draw listener position and direction
        let listener_screen = world_to_minimap(self.listener_pos.0, self.listener_pos.1);
        if minimap_rect.contains(listener_screen) {
            // Listener marker (triangle pointing in direction)
            let dir_x = self.listener_rot.cos();
            let dir_y = self.listener_rot.sin();
            let marker_size = 8.0;

            let tip = listener_screen + Vec2::new(dir_x, dir_y) * marker_size;
            let left = listener_screen
                + Vec2::new(dir_x * -0.5 - dir_y * 0.5, dir_y * -0.5 + dir_x * 0.5) * marker_size;
            let right = listener_screen
                + Vec2::new(dir_x * -0.5 + dir_y * 0.5, dir_y * -0.5 - dir_x * 0.5) * marker_size;

            painter.add(egui::Shape::convex_polygon(
                vec![tip, left, right],
                Color32::WHITE,
                egui::Stroke::new(1.0, Color32::BLACK),
            ));
        }
    }
}

/// Helper function to format time in MM:SS.
fn format_time(seconds: f32) -> String {
    let mins = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    format!("{mins}:{secs:02}")
}

/// Helper function to truncate strings.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_instance_id() {
        let id = SoundInstanceId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_sound_category() {
        assert_eq!(SoundCategory::Music.display_name(), "Music");
        assert_eq!(SoundCategory::Sfx.display_name(), "SFX");
        assert_eq!(SoundCategory::Ambient.display_name(), "Ambient");
        assert_eq!(SoundCategory::Ui.display_name(), "UI");
        assert_eq!(SoundCategory::Voice.display_name(), "Voice");
    }

    #[test]
    fn test_sound_category_colors() {
        // Just verify they return different colors
        let colors: Vec<_> = [
            SoundCategory::Music,
            SoundCategory::Sfx,
            SoundCategory::Ambient,
            SoundCategory::Ui,
            SoundCategory::Voice,
        ]
        .iter()
        .map(|c| c.color())
        .collect();

        // All should be distinct
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j]);
            }
        }
    }

    #[test]
    fn test_playback_state() {
        assert_eq!(PlaybackState::Playing.display_name(), "Playing");
        assert_eq!(PlaybackState::Playing.symbol(), "▶");
        assert_eq!(PlaybackState::Paused.symbol(), "⏸");
        assert_eq!(PlaybackState::Stopped.symbol(), "⏹");
    }

    #[test]
    fn test_playing_sound_info() {
        let mut sound = PlayingSoundInfo::new(
            SoundInstanceId::new(1),
            "explosion.ogg".to_string(),
            SoundCategory::Sfx,
        );

        assert!(!sound.is_spatial());
        assert_eq!(sound.progress(), 0.0);

        sound.duration = 2.0;
        sound.position = 1.0;
        assert!((sound.progress() - 0.5).abs() < 0.01);

        sound.world_position = Some((10.0, 20.0, 0.0));
        assert!(sound.is_spatial());
    }

    #[test]
    fn test_playing_sound_progress_no_duration() {
        let sound = PlayingSoundInfo::new(
            SoundInstanceId::new(1),
            "test.ogg".to_string(),
            SoundCategory::Sfx,
        );
        assert_eq!(sound.progress(), 0.0);
    }

    #[test]
    fn test_music_track_info() {
        let mut track = MusicTrackInfo::new("epic_theme.ogg".to_string());
        track.duration = 180.0; // 3 minutes
        track.position = 90.0; // 1:30

        assert_eq!(track.formatted_position(), "1:30");
        assert_eq!(track.formatted_duration(), "3:00");
        assert!((track.progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_music_track_progress_no_duration() {
        let track = MusicTrackInfo::new("test.ogg".to_string());
        assert_eq!(track.progress(), 0.0);
    }

    #[test]
    fn test_ambient_layer_info() {
        let mut layer = AmbientLayerInfo::new("forest".to_string());
        assert!(!layer.active);
        assert!(!layer.is_transitioning());

        layer.volume = 0.5;
        layer.target_volume = 1.0;
        assert!(layer.is_transitioning());

        layer.target_volume = 0.5;
        assert!(!layer.is_transitioning());
    }

    #[test]
    fn test_channel_usage() {
        let mut usage = ChannelUsage::default();
        assert_eq!(usage.usage_percent(), 0.0);
        assert!(!usage.is_near_capacity());

        usage.active = 16;
        usage.max = 32;
        assert!((usage.usage_percent() - 50.0).abs() < 0.01);

        usage.active = 28;
        assert!(usage.is_near_capacity());
    }

    #[test]
    fn test_channel_usage_zero_max() {
        let usage = ChannelUsage {
            active: 0,
            max: 0,
            peak_level: 0.0,
            average_level: 0.0,
        };
        assert_eq!(usage.usage_percent(), 0.0);
    }

    #[test]
    fn test_audio_debug_state_default() {
        let state = AudioDebugState::default();
        assert!(state.playing_sounds.is_empty());
        assert!(state.music_track.is_none());
        assert!(state.ambient_layers.is_empty());
        assert!(state.spatial_sources.is_empty());
    }

    #[test]
    fn test_audio_debug_panel_new() {
        let panel = AudioDebugPanel::new();
        assert!(!panel.open);
    }

    #[test]
    fn test_audio_debug_panel_toggle() {
        let mut panel = AudioDebugPanel::new();
        assert!(!panel.open);

        panel.toggle();
        assert!(panel.open);

        panel.toggle();
        assert!(!panel.open);
    }

    #[test]
    fn test_audio_debug_panel_update_state() {
        let mut panel = AudioDebugPanel::new();
        let mut state = AudioDebugState::default();
        state.channel_usage.active = 10;

        panel.update_state(state);
        assert_eq!(panel.state.channel_usage.active, 10);
    }

    #[test]
    fn test_audio_debug_config_default() {
        let config = AudioDebugConfig::default();
        assert!(config.show_playing_sounds);
        assert!(config.show_music_track);
        assert!(config.show_ambient_layers);
        assert!(config.show_channel_usage);
        assert!(config.show_peak_levels);
        assert_eq!(config.max_sounds_displayed, 10);
    }

    #[test]
    fn test_spatial_audio_minimap_config_default() {
        let config = SpatialAudioMinimapConfig::default();
        assert!(config.enabled);
        assert!(config.show_ranges);
        assert!(!config.show_labels);
    }

    #[test]
    fn test_spatial_audio_minimap_overlay_new() {
        let overlay = SpatialAudioMinimapOverlay::new();
        assert!(overlay.sources.is_empty());
    }

    #[test]
    fn test_spatial_audio_minimap_overlay_update() {
        let mut overlay = SpatialAudioMinimapOverlay::new();
        let sources = vec![SpatialAudioSource {
            id: SoundInstanceId::new(1),
            name: "fire.ogg".to_string(),
            position: (10.0, 20.0, 0.0),
            radius: 5.0,
            volume: 0.8,
            category: SoundCategory::Ambient,
        }];

        overlay.update(sources, (0.0, 0.0, 0.0), 0.5);
        assert_eq!(overlay.sources.len(), 1);
        assert_eq!(overlay.listener_pos, (0.0, 0.0, 0.0));
        assert!((overlay.listener_rot - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(0.0), "0:00");
        assert_eq!(format_time(30.0), "0:30");
        assert_eq!(format_time(60.0), "1:00");
        assert_eq!(format_time(90.0), "1:30");
        assert_eq!(format_time(3661.0), "61:01");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is...");
        assert_eq!(truncate_string("abc", 3), "abc");
    }

    #[test]
    fn test_truncate_string_edge_cases() {
        assert_eq!(truncate_string("", 5), "");
        assert_eq!(truncate_string("ab", 2), "ab");
    }

    #[test]
    fn test_sound_category_serialization() {
        let category = SoundCategory::Ambient;
        let json = serde_json::to_string(&category).unwrap();
        let loaded: SoundCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, loaded);
    }

    #[test]
    fn test_playback_state_serialization() {
        let state = PlaybackState::FadingOut;
        let json = serde_json::to_string(&state).unwrap();
        let loaded: PlaybackState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, loaded);
    }

    #[test]
    fn test_audio_debug_config_serialization() {
        let config = AudioDebugConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: AudioDebugConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.max_sounds_displayed, loaded.max_sounds_displayed);
    }

    #[test]
    fn test_spatial_audio_minimap_config_serialization() {
        let config = SpatialAudioMinimapConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: SpatialAudioMinimapConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.enabled, loaded.enabled);
    }
}
