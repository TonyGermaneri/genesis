//! Debug overlay for development information display.
//!
//! Provides FPS counter, player position, chunk info, and memory usage overlay.
//! Toggled with F3 key (like Minecraft).

use egui::{Color32, Context, Pos2, RichText};
use genesis_gameplay::Player;
use std::collections::VecDeque;
use std::time::Instant;

/// Debug overlay for development and debugging.
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    /// Whether the overlay is visible.
    visible: bool,
    /// Whether to show FPS.
    show_fps: bool,
    /// Whether to show position.
    show_position: bool,
    /// Whether to show chunk info.
    show_chunk_info: bool,
    /// Whether to show memory usage.
    show_memory: bool,
    /// Configuration.
    config: DebugOverlayConfig,
}

/// Configuration for the debug overlay.
#[derive(Debug, Clone)]
pub struct DebugOverlayConfig {
    /// Font size for overlay text.
    pub font_size: f32,
    /// Padding from screen edge.
    pub padding: f32,
    /// Width of the overlay panel.
    pub panel_width: f32,
    /// Background opacity (0.0-1.0).
    pub background_opacity: f32,
}

impl Default for DebugOverlayConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            padding: 10.0,
            panel_width: 280.0,
            background_opacity: 0.7,
        }
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugOverlay {
    /// Create a new debug overlay.
    #[must_use]
    pub fn new() -> Self {
        Self {
            visible: false,
            show_fps: true,
            show_position: true,
            show_chunk_info: true,
            show_memory: true,
            config: DebugOverlayConfig::default(),
        }
    }

    /// Create a new debug overlay with custom config.
    #[must_use]
    pub fn with_config(config: DebugOverlayConfig) -> Self {
        Self {
            visible: false,
            show_fps: true,
            show_position: true,
            show_chunk_info: true,
            show_memory: true,
            config,
        }
    }

    /// Render the debug overlay.
    pub fn render(&self, ctx: &Context, data: &DebugOverlayData<'_>) {
        if !self.visible {
            return;
        }

        let screen_rect = ctx.screen_rect();

        egui::Area::new(egui::Id::new("debug_overlay_area"))
            .fixed_pos(Pos2::new(
                screen_rect.width() - self.config.panel_width - self.config.padding,
                self.config.padding,
            ))
            .show(ctx, |ui| {
                let bg_alpha = (self.config.background_opacity * 255.0) as u8;
                egui::Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, bg_alpha))
                    .inner_margin(8.0)
                    .outer_margin(0.0)
                    .show(ui, |ui| {
                        ui.set_min_width(self.config.panel_width - 16.0);

                        // Title
                        ui.label(
                            RichText::new("Debug Info (F3)")
                                .color(Color32::LIGHT_GRAY)
                                .size(self.config.font_size + 2.0),
                        );
                        ui.separator();

                        if self.show_fps {
                            self.render_fps_section(ui, data.fps, data.frame_time_ms);
                        }

                        if self.show_position {
                            self.render_position_section(ui, data.player);
                        }

                        if self.show_chunk_info {
                            self.render_chunk_section(ui, data.player);
                        }

                        if self.show_memory {
                            self.render_memory_section(ui, data.memory_usage);
                        }
                    });
            });
    }

    /// Render FPS section.
    fn render_fps_section(&self, ui: &mut egui::Ui, fps: f32, frame_time_ms: f32) {
        let fps_color = fps_color(fps);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("FPS:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("{fps:.1}"))
                    .color(fps_color)
                    .size(self.config.font_size),
            );
        });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Frame Time:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("{frame_time_ms:.2}ms"))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });

        ui.add_space(4.0);
    }

    /// Render position section.
    fn render_position_section(&self, ui: &mut egui::Ui, player: &Player) {
        let pos = player.position();
        let vel = player.velocity();

        ui.label(
            RichText::new("Position")
                .color(Color32::LIGHT_BLUE)
                .size(self.config.font_size),
        );

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("X: {:.1}", pos.x))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("Y: {:.1}", pos.y))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });

        ui.label(
            RichText::new("Velocity")
                .color(Color32::LIGHT_BLUE)
                .size(self.config.font_size),
        );

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("VX: {:.1}", vel.x))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("VY: {:.1}", vel.y))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });

        // State info
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("State:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("{:?}", player.state()))
                    .color(Color32::LIGHT_GREEN)
                    .size(self.config.font_size),
            );
        });

        ui.horizontal(|ui| {
            let grounded_text = if player.is_grounded() { "Yes" } else { "No" };
            let grounded_color = if player.is_grounded() {
                Color32::GREEN
            } else {
                Color32::RED
            };
            ui.label(
                RichText::new("Grounded:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(grounded_text)
                    .color(grounded_color)
                    .size(self.config.font_size),
            );
        });

        ui.add_space(4.0);
    }

    /// Render chunk info section.
    fn render_chunk_section(&self, ui: &mut egui::Ui, player: &Player) {
        let pos = player.position();
        // Assuming 512 pixel chunks
        let chunk_x = (pos.x / 512.0).floor() as i32;
        let chunk_y = (pos.y / 512.0).floor() as i32;

        // Local position within chunk
        let local_x = pos.x.rem_euclid(512.0);
        let local_y = pos.y.rem_euclid(512.0);

        ui.label(
            RichText::new("Chunk Info")
                .color(Color32::LIGHT_BLUE)
                .size(self.config.font_size),
        );

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Chunk:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("({chunk_x}, {chunk_y})"))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Local:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("({local_x:.0}, {local_y:.0})"))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });

        ui.add_space(4.0);
    }

    /// Render memory section.
    fn render_memory_section(&self, ui: &mut egui::Ui, memory_usage: usize) {
        ui.label(
            RichText::new("Memory")
                .color(Color32::LIGHT_BLUE)
                .size(self.config.font_size),
        );

        let formatted = format_bytes(memory_usage);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Usage:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(formatted)
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });
    }

    /// Toggle the overlay visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if overlay is visible.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set overlay visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Toggle FPS display.
    pub fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
    }

    /// Toggle position display.
    pub fn toggle_position(&mut self) {
        self.show_position = !self.show_position;
    }

    /// Toggle chunk info display.
    pub fn toggle_chunk_info(&mut self) {
        self.show_chunk_info = !self.show_chunk_info;
    }

    /// Toggle memory display.
    pub fn toggle_memory(&mut self) {
        self.show_memory = !self.show_memory;
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &DebugOverlayConfig {
        &self.config
    }
}

/// Data needed to render the debug overlay.
#[derive(Debug)]
pub struct DebugOverlayData<'a> {
    /// Current FPS.
    pub fps: f32,
    /// Frame time in milliseconds.
    pub frame_time_ms: f32,
    /// Player reference.
    pub player: &'a Player,
    /// Memory usage in bytes.
    pub memory_usage: usize,
}

/// FPS tracking helper with rolling average.
#[derive(Debug)]
pub struct FpsCounter {
    /// Frame timestamps for rolling average.
    frames: VecDeque<Instant>,
    /// Last frame timestamp.
    last_time: Instant,
    /// Maximum number of frames to track.
    max_frames: usize,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    /// Default number of frames to average.
    pub const DEFAULT_FRAME_COUNT: usize = 60;

    /// Create a new FPS counter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: VecDeque::with_capacity(Self::DEFAULT_FRAME_COUNT),
            last_time: Instant::now(),
            max_frames: Self::DEFAULT_FRAME_COUNT,
        }
    }

    /// Create an FPS counter with custom averaging window.
    #[must_use]
    pub fn with_window_size(frames: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(frames),
            last_time: Instant::now(),
            max_frames: frames,
        }
    }

    /// Record a frame and return (fps, frame_time_ms).
    pub fn tick(&mut self) -> (f32, f32) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_time);
        let frame_time_ms = frame_time.as_secs_f32() * 1000.0;

        self.frames.push_back(now);
        self.last_time = now;

        // Keep only the last max_frames
        while self.frames.len() > self.max_frames {
            self.frames.pop_front();
        }

        // Calculate FPS from time span of tracked frames
        let fps = if self.frames.len() > 1 {
            if let (Some(first), Some(last)) = (self.frames.front(), self.frames.back()) {
                let duration = last.duration_since(*first);
                if duration.as_secs_f32() > 0.0 {
                    (self.frames.len() - 1) as f32 / duration.as_secs_f32()
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        (fps, frame_time_ms)
    }

    /// Get the current FPS without recording a frame.
    #[must_use]
    pub fn fps(&self) -> f32 {
        if self.frames.len() > 1 {
            if let (Some(first), Some(last)) = (self.frames.front(), self.frames.back()) {
                let duration = last.duration_since(*first);
                if duration.as_secs_f32() > 0.0 {
                    (self.frames.len() - 1) as f32 / duration.as_secs_f32()
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get the last frame time in milliseconds.
    #[must_use]
    pub fn last_frame_time_ms(&self) -> f32 {
        if self.frames.len() >= 2 {
            let len = self.frames.len();
            if let (Some(prev), Some(curr)) = (self.frames.get(len - 2), self.frames.get(len - 1)) {
                return curr.duration_since(*prev).as_secs_f32() * 1000.0;
            }
        }
        0.0
    }

    /// Reset the FPS counter.
    pub fn reset(&mut self) {
        self.frames.clear();
        self.last_time = Instant::now();
    }

    /// Get the number of frames being tracked.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// Returns a color for FPS display based on value.
fn fps_color(fps: f32) -> Color32 {
    if fps >= 60.0 {
        Color32::GREEN
    } else if fps >= 30.0 {
        Color32::YELLOW
    } else {
        Color32::RED
    }
}

/// Format bytes into human-readable string.
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_gameplay::Vec2;

    fn create_test_player() -> Player {
        Player::new(Vec2::new(1000.0, 500.0))
    }

    #[test]
    fn test_debug_overlay_new() {
        let overlay = DebugOverlay::new();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_debug_overlay_toggle() {
        let mut overlay = DebugOverlay::new();
        assert!(!overlay.is_visible());

        overlay.toggle();
        assert!(overlay.is_visible());

        overlay.toggle();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_debug_overlay_set_visible() {
        let mut overlay = DebugOverlay::new();

        overlay.set_visible(true);
        assert!(overlay.is_visible());

        overlay.set_visible(false);
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_debug_overlay_config_defaults() {
        let config = DebugOverlayConfig::default();
        assert_eq!(config.font_size, 14.0);
        assert_eq!(config.padding, 10.0);
    }

    #[test]
    fn test_debug_overlay_with_config() {
        let config = DebugOverlayConfig {
            font_size: 16.0,
            ..Default::default()
        };
        let overlay = DebugOverlay::with_config(config);
        assert_eq!(overlay.config().font_size, 16.0);
    }

    #[test]
    fn test_debug_overlay_toggle_sections() {
        let mut overlay = DebugOverlay::new();

        assert!(overlay.show_fps);
        overlay.toggle_fps();
        assert!(!overlay.show_fps);

        assert!(overlay.show_position);
        overlay.toggle_position();
        assert!(!overlay.show_position);

        assert!(overlay.show_chunk_info);
        overlay.toggle_chunk_info();
        assert!(!overlay.show_chunk_info);

        assert!(overlay.show_memory);
        overlay.toggle_memory();
        assert!(!overlay.show_memory);
    }

    #[test]
    fn test_fps_counter_new() {
        let counter = FpsCounter::new();
        assert_eq!(counter.frame_count(), 0);
        assert_eq!(counter.fps(), 0.0);
    }

    #[test]
    fn test_fps_counter_tick() {
        let mut counter = FpsCounter::new();

        // First tick returns 0 FPS (need at least 2 frames)
        let (fps, _) = counter.tick();
        assert_eq!(fps, 0.0);

        // After multiple ticks, should have non-zero FPS
        std::thread::sleep(std::time::Duration::from_millis(10));
        counter.tick();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (fps, _) = counter.tick();
        assert!(fps > 0.0);
    }

    #[test]
    fn test_fps_counter_reset() {
        let mut counter = FpsCounter::new();
        counter.tick();
        counter.tick();
        assert!(counter.frame_count() > 0);

        counter.reset();
        assert_eq!(counter.frame_count(), 0);
    }

    #[test]
    fn test_fps_counter_with_window_size() {
        let counter = FpsCounter::with_window_size(30);
        assert_eq!(counter.max_frames, 30);
    }

    #[test]
    fn test_fps_color() {
        assert_eq!(fps_color(60.0), Color32::GREEN);
        assert_eq!(fps_color(120.0), Color32::GREEN);
        assert_eq!(fps_color(45.0), Color32::YELLOW);
        assert_eq!(fps_color(30.0), Color32::YELLOW);
        assert_eq!(fps_color(20.0), Color32::RED);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_debug_overlay_data() {
        let player = create_test_player();
        let data = DebugOverlayData {
            fps: 60.0,
            frame_time_ms: 16.67,
            player: &player,
            memory_usage: 1024 * 1024,
        };
        assert_eq!(data.fps, 60.0);
        assert_eq!(data.memory_usage, 1024 * 1024);
    }
}
