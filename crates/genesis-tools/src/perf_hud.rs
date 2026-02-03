//! Performance HUD overlay for real-time metrics display.
//!
//! This module provides an egui-based performance overlay showing:
//! - Frame times and FPS (with graph)
//! - Simulation tick duration
//! - Memory usage
//! - Compute dispatch counts
//! - GPU timing (when available)

use crate::perf::{PerfStats, PerfTracker};
use egui::{Color32, Grid, RichText, Ui, Vec2};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Target frame time for 60 FPS (in ms)
const TARGET_FRAME_TIME_MS: f64 = 16.67;

/// Warning threshold for frame time (in ms)
const WARNING_FRAME_TIME_MS: f64 = 20.0;

/// Critical threshold for frame time (in ms)
const CRITICAL_FRAME_TIME_MS: f64 = 33.33;

/// Configuration for the performance HUD.
#[derive(Debug, Clone)]
pub struct PerfHudConfig {
    /// Whether to show the FPS counter
    pub show_fps: bool,
    /// Whether to show the frame time graph
    pub show_frame_graph: bool,
    /// Whether to show simulation stats
    pub show_sim_stats: bool,
    /// Whether to show memory usage
    pub show_memory: bool,
    /// Whether to show compute stats
    pub show_compute: bool,
    /// Whether to show GPU timing
    pub show_gpu: bool,
    /// Number of samples to display in the graph
    pub graph_samples: usize,
    /// HUD opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Graph height in pixels
    pub graph_height: f32,
    /// Whether the HUD is collapsed
    pub collapsed: bool,
}

impl Default for PerfHudConfig {
    fn default() -> Self {
        Self {
            show_fps: true,
            show_frame_graph: true,
            show_sim_stats: true,
            show_memory: true,
            show_compute: true,
            show_gpu: true,
            graph_samples: 120,
            opacity: 0.85,
            graph_height: 60.0,
            collapsed: false,
        }
    }
}

/// Simulation timing statistics.
#[derive(Debug, Clone, Default)]
pub struct SimStats {
    /// Time spent in physics/simulation tick (ms)
    pub tick_time_ms: f64,
    /// Number of ticks per second
    pub ticks_per_second: f64,
    /// Accumulated tick time for averaging
    pub accumulated_tick_time: Duration,
    /// Number of ticks this frame
    pub ticks_this_frame: u32,
}

/// Compute dispatch statistics.
#[derive(Debug, Clone, Default)]
pub struct ComputeStats {
    /// Number of compute shader dispatches
    pub dispatch_count: u32,
    /// Total workgroups dispatched
    pub total_workgroups: u64,
    /// Active cell updates per frame
    pub active_cells: u64,
    /// Compute time in ms
    pub compute_time_ms: f64,
}

/// Memory usage statistics.
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Total heap memory usage in bytes
    pub heap_bytes: usize,
    /// GPU buffer memory in bytes
    pub gpu_buffer_bytes: usize,
    /// Loaded chunk count
    pub loaded_chunks: usize,
    /// Cached chunk count
    pub cached_chunks: usize,
    /// Entity count
    pub entity_count: usize,
}

impl MemoryStats {
    /// Formats bytes as human-readable string.
    #[must_use]
    pub fn format_bytes(bytes: usize) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{bytes} B")
        }
    }
}

/// Frame time sample for the graph.
#[derive(Debug, Clone, Copy)]
pub struct FrameSample {
    /// Frame time in milliseconds
    pub frame_time_ms: f64,
    /// Simulation time in milliseconds (subset of frame time)
    pub sim_time_ms: f64,
    /// GPU time in milliseconds (if available)
    pub gpu_time_ms: Option<f64>,
}

impl Default for FrameSample {
    fn default() -> Self {
        Self {
            frame_time_ms: 0.0,
            sim_time_ms: 0.0,
            gpu_time_ms: None,
        }
    }
}

/// Performance HUD with egui rendering.
#[derive(Debug)]
pub struct PerfHud {
    /// Configuration
    pub config: PerfHudConfig,
    /// Frame time history for graph
    frame_history: VecDeque<FrameSample>,
    /// Simulation statistics
    sim_stats: SimStats,
    /// Compute statistics
    compute_stats: ComputeStats,
    /// Memory statistics
    memory_stats: MemoryStats,
    /// GPU time in ms (last frame)
    gpu_time_ms: Option<f64>,
    /// Current FPS
    current_fps: f64,
    /// Current frame time
    current_frame_time_ms: f64,
    /// Best frame time (1% low)
    best_frame_time_ms: f64,
    /// Worst frame time (99th percentile)
    worst_frame_time_ms: f64,
    /// Last update time
    last_update: Instant,
    /// Update interval for stats
    update_interval: Duration,
}

impl Default for PerfHud {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfHud {
    /// Creates a new Performance HUD.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: PerfHudConfig::default(),
            frame_history: VecDeque::with_capacity(120),
            sim_stats: SimStats::default(),
            compute_stats: ComputeStats::default(),
            memory_stats: MemoryStats::default(),
            gpu_time_ms: None,
            current_fps: 0.0,
            current_frame_time_ms: 0.0,
            best_frame_time_ms: 0.0,
            worst_frame_time_ms: 0.0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
        }
    }

    /// Creates a new Performance HUD with custom config.
    #[must_use]
    pub fn with_config(config: PerfHudConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Updates the HUD with a new frame sample.
    pub fn update(&mut self, frame_time_ms: f64, sim_time_ms: f64) {
        // Add sample to history
        let sample = FrameSample {
            frame_time_ms,
            sim_time_ms,
            gpu_time_ms: self.gpu_time_ms,
        };

        if self.frame_history.len() >= self.config.graph_samples {
            self.frame_history.pop_front();
        }
        self.frame_history.push_back(sample);

        // Update stats periodically
        if self.last_update.elapsed() >= self.update_interval {
            self.recalculate_stats();
            self.last_update = Instant::now();
        }
    }

    /// Updates from a `PerfTracker` instance.
    pub fn update_from_tracker(&mut self, tracker: &PerfTracker) {
        let stats = tracker.stats();
        self.update(stats.frame_time_ms, 0.0);
        self.current_fps = stats.fps;
        self.memory_stats.loaded_chunks = stats.loaded_chunks;
        self.memory_stats.entity_count = stats.entity_count;
        self.memory_stats.heap_bytes = stats.memory_bytes;
        if let Some(gpu) = stats.gpu_time_ms {
            self.gpu_time_ms = Some(gpu);
        }
    }

    /// Updates simulation statistics.
    pub fn update_sim_stats(&mut self, tick_time: Duration, ticks: u32) {
        self.sim_stats.tick_time_ms = tick_time.as_secs_f64() * 1000.0;
        self.sim_stats.ticks_this_frame = ticks;
        self.sim_stats.accumulated_tick_time += tick_time;
    }

    /// Updates compute statistics.
    pub fn update_compute_stats(
        &mut self,
        dispatches: u32,
        workgroups: u64,
        active_cells: u64,
        compute_time_ms: f64,
    ) {
        self.compute_stats.dispatch_count = dispatches;
        self.compute_stats.total_workgroups = workgroups;
        self.compute_stats.active_cells = active_cells;
        self.compute_stats.compute_time_ms = compute_time_ms;
    }

    /// Updates memory statistics.
    pub fn update_memory_stats(&mut self, stats: MemoryStats) {
        self.memory_stats = stats;
    }

    /// Sets GPU timing for the last frame.
    pub fn set_gpu_time(&mut self, time_ms: f64) {
        self.gpu_time_ms = Some(time_ms);
    }

    /// Recalculates derived statistics.
    fn recalculate_stats(&mut self) {
        if self.frame_history.is_empty() {
            return;
        }

        // Calculate current values
        if let Some(last) = self.frame_history.back() {
            self.current_frame_time_ms = last.frame_time_ms;
        }

        // Calculate FPS from average frame time
        let total_time: f64 = self.frame_history.iter().map(|s| s.frame_time_ms).sum();
        let avg_frame_time = total_time / self.frame_history.len() as f64;
        if avg_frame_time > 0.0 {
            self.current_fps = 1000.0 / avg_frame_time;
        }

        // Calculate percentiles
        let mut sorted: Vec<f64> = self.frame_history.iter().map(|s| s.frame_time_ms).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        if !sorted.is_empty() {
            let len = sorted.len();
            self.best_frame_time_ms = sorted[len / 100]; // ~1%
            self.worst_frame_time_ms = sorted[len * 99 / 100]; // ~99%
        }
    }

    /// Returns the color for a frame time value.
    #[must_use]
    pub fn frame_time_color(frame_time_ms: f64) -> Color32 {
        if frame_time_ms <= TARGET_FRAME_TIME_MS {
            Color32::from_rgb(100, 255, 100) // Green
        } else if frame_time_ms <= WARNING_FRAME_TIME_MS {
            Color32::from_rgb(255, 255, 100) // Yellow
        } else if frame_time_ms <= CRITICAL_FRAME_TIME_MS {
            Color32::from_rgb(255, 165, 0) // Orange
        } else {
            Color32::from_rgb(255, 100, 100) // Red
        }
    }

    /// Renders the HUD.
    pub fn render_ui(&self, ui: &mut Ui) {
        let opacity = (self.config.opacity * 255.0) as u8;
        let bg_color = Color32::from_rgba_unmultiplied(20, 20, 20, opacity);

        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(8.0)
            .rounding(4.0)
            .show(ui, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📊 Performance").strong().size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(if self.config.collapsed { "▼" } else { "▲" })
                            .clicked()
                        {
                            // Note: Would need &mut self to actually toggle
                        }
                    });
                });

                if self.config.collapsed {
                    // Compact view - just FPS
                    ui.label(
                        RichText::new(format!("{:.0} FPS", self.current_fps))
                            .color(Self::frame_time_color(self.current_frame_time_ms)),
                    );
                    return;
                }

                ui.separator();

                // FPS Section
                if self.config.show_fps {
                    self.render_fps_section(ui);
                }

                // Frame Graph
                if self.config.show_frame_graph {
                    ui.add_space(4.0);
                    self.render_frame_graph(ui);
                }

                // Simulation Stats
                if self.config.show_sim_stats {
                    ui.separator();
                    self.render_sim_section(ui);
                }

                // Memory Stats
                if self.config.show_memory {
                    ui.separator();
                    self.render_memory_section(ui);
                }

                // Compute Stats
                if self.config.show_compute {
                    ui.separator();
                    self.render_compute_section(ui);
                }

                // GPU Stats
                if self.config.show_gpu && self.gpu_time_ms.is_some() {
                    ui.separator();
                    self.render_gpu_section(ui);
                }
            });
    }

    /// Renders the FPS section.
    fn render_fps_section(&self, ui: &mut Ui) {
        let fps_color = Self::frame_time_color(self.current_frame_time_ms);

        ui.horizontal(|ui| {
            ui.label("FPS:");
            ui.label(
                RichText::new(format!("{:.0}", self.current_fps))
                    .color(fps_color)
                    .strong(),
            );
            ui.label("|");
            ui.label("Frame:");
            ui.label(
                RichText::new(format!("{:.2} ms", self.current_frame_time_ms)).color(fps_color),
            );
        });

        // Percentiles
        ui.horizontal(|ui| {
            ui.label("1% low:");
            ui.label(format!("{:.2} ms", self.best_frame_time_ms));
            ui.label("|");
            ui.label("99%:");
            ui.label(format!("{:.2} ms", self.worst_frame_time_ms));
        });
    }

    /// Renders the frame time graph.
    fn render_frame_graph(&self, ui: &mut Ui) {
        let height = self.config.graph_height;
        let max_time = CRITICAL_FRAME_TIME_MS * 1.5; // Scale to ~50ms max

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 30));

        // Target line (16.67ms)
        let target_y = rect.max.y - (TARGET_FRAME_TIME_MS / max_time * height as f64) as f32;
        painter.line_segment(
            [
                egui::Pos2::new(rect.min.x, target_y),
                egui::Pos2::new(rect.max.x, target_y),
            ],
            egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
        );

        // Draw frame time bars
        if !self.frame_history.is_empty() {
            let bar_width = rect.width() / self.config.graph_samples as f32;
            let offset = if self.frame_history.len() < self.config.graph_samples {
                (self.config.graph_samples - self.frame_history.len()) as f32 * bar_width
            } else {
                0.0
            };

            for (i, sample) in self.frame_history.iter().enumerate() {
                let x = rect.min.x + offset + i as f32 * bar_width;
                let bar_height = (sample.frame_time_ms / max_time * height as f64) as f32;
                let bar_height = bar_height.min(height);

                let color = Self::frame_time_color(sample.frame_time_ms);

                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::Pos2::new(x, rect.max.y - bar_height),
                        Vec2::new(bar_width - 1.0, bar_height),
                    ),
                    0.0,
                    color,
                );

                // Sim time overlay
                if sample.sim_time_ms > 0.0 {
                    let sim_height = (sample.sim_time_ms / max_time * height as f64) as f32;
                    let sim_height = sim_height.min(bar_height);
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::Pos2::new(x, rect.max.y - sim_height),
                            Vec2::new(bar_width - 1.0, sim_height),
                        ),
                        0.0,
                        Color32::from_rgba_unmultiplied(100, 100, 255, 180),
                    );
                }
            }
        }

        // Scale labels
        let small_font = egui::FontId::proportional(9.0);
        painter.text(
            egui::Pos2::new(rect.max.x - 20.0, target_y - 2.0),
            egui::Align2::RIGHT_BOTTOM,
            "16ms",
            small_font.clone(),
            Color32::GRAY,
        );
        painter.text(
            egui::Pos2::new(rect.max.x - 2.0, rect.min.y + 2.0),
            egui::Align2::RIGHT_TOP,
            format!("{max_time:.0}ms"),
            small_font,
            Color32::GRAY,
        );
    }

    /// Renders the simulation section.
    fn render_sim_section(&self, ui: &mut Ui) {
        ui.label(RichText::new("⚙ Simulation").size(12.0));
        Grid::new("sim_stats_grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .show(ui, |ui| {
                ui.label("Tick time:");
                ui.label(format!("{:.2} ms", self.sim_stats.tick_time_ms));
                ui.end_row();

                ui.label("Ticks/frame:");
                ui.label(format!("{}", self.sim_stats.ticks_this_frame));
                ui.end_row();
            });
    }

    /// Renders the memory section.
    fn render_memory_section(&self, ui: &mut Ui) {
        ui.label(RichText::new("💾 Memory").size(12.0));
        Grid::new("memory_stats_grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .show(ui, |ui| {
                ui.label("Heap:");
                ui.label(MemoryStats::format_bytes(self.memory_stats.heap_bytes));
                ui.end_row();

                ui.label("GPU buffers:");
                ui.label(MemoryStats::format_bytes(
                    self.memory_stats.gpu_buffer_bytes,
                ));
                ui.end_row();

                ui.label("Chunks:");
                ui.label(format!(
                    "{} loaded / {} cached",
                    self.memory_stats.loaded_chunks, self.memory_stats.cached_chunks
                ));
                ui.end_row();

                ui.label("Entities:");
                ui.label(format!("{}", self.memory_stats.entity_count));
                ui.end_row();
            });
    }

    /// Renders the compute section.
    fn render_compute_section(&self, ui: &mut Ui) {
        ui.label(RichText::new("🔧 Compute").size(12.0));
        Grid::new("compute_stats_grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .show(ui, |ui| {
                ui.label("Dispatches:");
                ui.label(format!("{}", self.compute_stats.dispatch_count));
                ui.end_row();

                ui.label("Workgroups:");
                ui.label(format!("{}", self.compute_stats.total_workgroups));
                ui.end_row();

                ui.label("Active cells:");
                ui.label(Self::format_large_number(self.compute_stats.active_cells));
                ui.end_row();

                ui.label("Compute time:");
                ui.label(format!("{:.2} ms", self.compute_stats.compute_time_ms));
                ui.end_row();
            });
    }

    /// Renders the GPU section.
    fn render_gpu_section(&self, ui: &mut Ui) {
        ui.label(RichText::new("🎮 GPU").size(12.0));
        if let Some(gpu_time) = self.gpu_time_ms {
            let gpu_color = Self::frame_time_color(gpu_time);
            ui.horizontal(|ui| {
                ui.label("GPU time:");
                ui.label(RichText::new(format!("{gpu_time:.2} ms")).color(gpu_color));
            });
        }
    }

    /// Formats large numbers with K/M suffixes.
    fn format_large_number(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{:.2}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            format!("{n}")
        }
    }

    /// Returns the current stats as a `PerfStats` struct.
    #[must_use]
    pub fn to_perf_stats(&self) -> PerfStats {
        PerfStats {
            fps: self.current_fps,
            frame_time_ms: self.current_frame_time_ms,
            gpu_time_ms: self.gpu_time_ms,
            loaded_chunks: self.memory_stats.loaded_chunks,
            entity_count: self.memory_stats.entity_count,
            memory_bytes: self.memory_stats.heap_bytes,
        }
    }
}

/// A simple timer for measuring code sections.
#[derive(Debug)]
pub struct ScopedTimer {
    name: String,
    start: Instant,
}

impl ScopedTimer {
    /// Creates a new timer with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// Returns the elapsed time.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Returns the elapsed time in milliseconds.
    #[must_use]
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Finishes the timer and returns the duration.
    #[must_use]
    pub fn finish(self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        tracing::debug!(
            target: "perf",
            "{}: {:.2}ms",
            self.name,
            elapsed.as_secs_f64() * 1000.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_hud_creation() {
        let hud = PerfHud::new();
        assert_eq!(hud.current_fps, 0.0);
        assert!(hud.frame_history.is_empty());
    }

    #[test]
    fn test_perf_hud_update() {
        let mut hud = PerfHud::new();

        // Add some samples
        for i in 0..10 {
            hud.update(16.0 + i as f64 * 0.1, 5.0);
        }

        assert_eq!(hud.frame_history.len(), 10);
    }

    #[test]
    fn test_frame_time_color() {
        // Green for good frame times
        let color = PerfHud::frame_time_color(10.0);
        assert_eq!(color, Color32::from_rgb(100, 255, 100));

        // Yellow for warning
        let color = PerfHud::frame_time_color(18.0);
        assert_eq!(color, Color32::from_rgb(255, 255, 100));

        // Orange for bad
        let color = PerfHud::frame_time_color(25.0);
        assert_eq!(color, Color32::from_rgb(255, 165, 0));

        // Red for critical
        let color = PerfHud::frame_time_color(50.0);
        assert_eq!(color, Color32::from_rgb(255, 100, 100));
    }

    #[test]
    fn test_memory_stats_format() {
        assert_eq!(MemoryStats::format_bytes(500), "500 B");
        assert_eq!(MemoryStats::format_bytes(1024), "1.00 KB");
        assert_eq!(MemoryStats::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(MemoryStats::format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_large_number() {
        assert_eq!(PerfHud::format_large_number(500), "500");
        assert_eq!(PerfHud::format_large_number(1500), "1.5K");
        assert_eq!(PerfHud::format_large_number(1_500_000), "1.50M");
    }

    #[test]
    fn test_perf_hud_config_defaults() {
        let config = PerfHudConfig::default();
        assert!(config.show_fps);
        assert!(config.show_frame_graph);
        assert!(config.show_sim_stats);
        assert!(config.show_memory);
        assert!(config.show_compute);
        assert!(config.show_gpu);
        assert_eq!(config.graph_samples, 120);
        assert!(!config.collapsed);
    }

    #[test]
    fn test_compute_stats_update() {
        let mut hud = PerfHud::new();
        hud.update_compute_stats(5, 1000, 50000, 2.5);

        assert_eq!(hud.compute_stats.dispatch_count, 5);
        assert_eq!(hud.compute_stats.total_workgroups, 1000);
        assert_eq!(hud.compute_stats.active_cells, 50000);
    }

    #[test]
    fn test_sim_stats_update() {
        let mut hud = PerfHud::new();
        hud.update_sim_stats(Duration::from_millis(5), 2);

        assert!((hud.sim_stats.tick_time_ms - 5.0).abs() < 0.1);
        assert_eq!(hud.sim_stats.ticks_this_frame, 2);
    }

    #[test]
    fn test_scoped_timer() {
        let timer = ScopedTimer::new("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10.0);
    }

    #[test]
    fn test_to_perf_stats() {
        let mut hud = PerfHud::new();
        hud.current_fps = 60.0;
        hud.current_frame_time_ms = 16.67;
        hud.memory_stats.loaded_chunks = 10;
        hud.memory_stats.entity_count = 100;

        let stats = hud.to_perf_stats();
        assert_eq!(stats.fps, 60.0);
        assert_eq!(stats.loaded_chunks, 10);
    }
}
