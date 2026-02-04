//! Performance metrics and profiling.
//!
//! Collects and reports frame timing, render stats, and world metrics.

#![allow(dead_code)]

use std::collections::VecDeque;
use std::time::Instant;

/// Performance metrics collector.
#[derive(Debug)]
pub struct PerfMetrics {
    /// Frame times (total)
    frame_times: VecDeque<f32>,
    /// Update times (gameplay/physics)
    update_times: VecDeque<f32>,
    /// Render times (GPU)
    render_times: VecDeque<f32>,
    /// History size for averaging
    history_size: usize,
    /// Number of loaded chunks
    chunk_count: u32,
    /// Number of cells being simulated
    cell_count: u64,
    /// Current camera position
    camera_position: (f32, f32),
    /// Current zoom level
    zoom: f32,
    /// Player position
    player_position: (f32, f32),
    /// Player velocity
    player_velocity: (f32, f32),
}

impl Default for PerfMetrics {
    fn default() -> Self {
        Self::new(120)
    }
}

impl PerfMetrics {
    /// Create a new performance metrics collector.
    ///
    /// # Arguments
    /// * `history_size` - Number of samples to keep for averaging
    #[must_use]
    pub fn new(history_size: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(history_size),
            update_times: VecDeque::with_capacity(history_size),
            render_times: VecDeque::with_capacity(history_size),
            history_size,
            chunk_count: 0,
            cell_count: 0,
            camera_position: (0.0, 0.0),
            zoom: 1.0,
            player_position: (0.0, 0.0),
            player_velocity: (0.0, 0.0),
        }
    }

    /// Record frame timing.
    ///
    /// # Arguments
    /// * `total` - Total frame time in seconds
    /// * `update` - Update/gameplay time in seconds
    /// * `render` - Render time in seconds
    pub fn record_frame(&mut self, total: f32, update: f32, render: f32) {
        self.frame_times.push_back(total);
        if self.frame_times.len() > self.history_size {
            self.frame_times.pop_front();
        }

        self.update_times.push_back(update);
        if self.update_times.len() > self.history_size {
            self.update_times.pop_front();
        }

        self.render_times.push_back(render);
        if self.render_times.len() > self.history_size {
            self.render_times.pop_front();
        }
    }

    /// Get average FPS.
    #[must_use]
    pub fn avg_fps(&self) -> f32 {
        let avg = self.avg_frame_time();
        if avg > 0.0 {
            1.0 / avg
        } else {
            0.0
        }
    }

    /// Get 1% low FPS (99th percentile frame time).
    #[must_use]
    pub fn low_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f32> = self.frame_times.iter().copied().collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Get the 1% worst frame times
        let idx = (sorted.len() / 100).max(1).min(sorted.len() - 1);
        let worst = sorted[idx];

        if worst > 0.0 {
            1.0 / worst
        } else {
            0.0
        }
    }

    /// Get average frame time in seconds.
    #[must_use]
    pub fn avg_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    /// Get average update time in seconds.
    #[must_use]
    pub fn avg_update_time(&self) -> f32 {
        if self.update_times.is_empty() {
            return 0.0;
        }
        self.update_times.iter().sum::<f32>() / self.update_times.len() as f32
    }

    /// Get average render time in seconds.
    #[must_use]
    pub fn avg_render_time(&self) -> f32 {
        if self.render_times.is_empty() {
            return 0.0;
        }
        self.render_times.iter().sum::<f32>() / self.render_times.len() as f32
    }

    /// Update world statistics.
    pub fn set_world_stats(&mut self, chunks: u32, cells: u64) {
        self.chunk_count = chunks;
        self.cell_count = cells;
    }

    /// Update camera information.
    pub fn set_camera(&mut self, position: (f32, f32), zoom: f32) {
        self.camera_position = position;
        self.zoom = zoom;
    }

    /// Update player information.
    pub fn set_player(&mut self, position: (f32, f32), velocity: (f32, f32)) {
        self.player_position = position;
        self.player_velocity = velocity;
    }

    /// Get a summary of all performance metrics.
    #[must_use]
    pub fn summary(&self) -> PerfSummary {
        PerfSummary {
            fps: self.avg_fps(),
            fps_1_percent_low: self.low_fps(),
            frame_time_ms: self.avg_frame_time() * 1000.0,
            update_time_ms: self.avg_update_time() * 1000.0,
            render_time_ms: self.avg_render_time() * 1000.0,
            chunks_loaded: self.chunk_count,
            cells_simulated: self.cell_count,
            camera_position: self.camera_position,
            zoom: self.zoom,
            player_position: self.player_position,
            player_velocity: self.player_velocity,
        }
    }

    /// Clear all recorded metrics.
    pub fn clear(&mut self) {
        self.frame_times.clear();
        self.update_times.clear();
        self.render_times.clear();
    }
}

/// Summary of performance metrics for display.
#[derive(Debug, Clone, Default)]
pub struct PerfSummary {
    /// Average FPS
    pub fps: f32,
    /// 1% low FPS
    pub fps_1_percent_low: f32,
    /// Average frame time in milliseconds
    pub frame_time_ms: f32,
    /// Average update time in milliseconds
    pub update_time_ms: f32,
    /// Average render time in milliseconds
    pub render_time_ms: f32,
    /// Number of loaded chunks
    pub chunks_loaded: u32,
    /// Number of cells being simulated
    pub cells_simulated: u64,
    /// Camera position
    pub camera_position: (f32, f32),
    /// Zoom level
    pub zoom: f32,
    /// Player position
    pub player_position: (f32, f32),
    /// Player velocity
    pub player_velocity: (f32, f32),
}

impl PerfSummary {
    /// Format as multi-line debug text.
    #[must_use]
    pub fn format_debug(&self) -> String {
        format!(
            "FPS: {:.0} (1% low: {:.0})\n\
             Frame: {:.1}ms (Update: {:.1}ms, Render: {:.1}ms)\n\
             Chunks: {} ({} cells)\n\
             Camera: ({:.1}, {:.1}) Zoom: {:.1}x\n\
             Player: ({:.1}, {:.1}) vel: ({:.1}, {:.1})",
            self.fps,
            self.fps_1_percent_low,
            self.frame_time_ms,
            self.update_time_ms,
            self.render_time_ms,
            self.chunks_loaded,
            format_number(self.cells_simulated),
            self.camera_position.0,
            self.camera_position.1,
            self.zoom,
            self.player_position.0,
            self.player_position.1,
            self.player_velocity.0,
            self.player_velocity.1
        )
    }
}

/// Format a large number with commas.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Scoped timer for measuring code sections.
#[derive(Debug)]
pub struct ScopedTimer {
    start: Instant,
    name: &'static str,
}

impl ScopedTimer {
    /// Start a new scoped timer.
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self {
            start: Instant::now(),
            name,
        }
    }

    /// Get elapsed time without stopping.
    #[must_use]
    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Stop and return elapsed time in seconds.
    #[must_use]
    pub fn stop(self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Get the timer name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_metrics_creation() {
        let metrics = PerfMetrics::new(60);
        assert_eq!(metrics.avg_fps(), 0.0);
        assert_eq!(metrics.chunk_count, 0);
    }

    #[test]
    fn test_perf_metrics_record() {
        let mut metrics = PerfMetrics::new(60);

        // Record some frames at ~60 FPS (16.67ms)
        for _ in 0..30 {
            metrics.record_frame(0.0167, 0.002, 0.008);
        }

        let fps = metrics.avg_fps();
        assert!(fps > 55.0 && fps < 65.0, "FPS should be ~60, got {fps}");
    }

    #[test]
    fn test_perf_metrics_1_percent_low() {
        let mut metrics = PerfMetrics::new(100);

        // Most frames at 60 FPS
        for _ in 0..90 {
            metrics.record_frame(0.0167, 0.002, 0.008);
        }
        // 10 slow frames at 30 FPS (10% of frames to ensure they're captured)
        for _ in 0..10 {
            metrics.record_frame(0.0333, 0.004, 0.016);
        }

        let low = metrics.low_fps();
        // With 10% slow frames, the 1% low should pick up some slow frames
        assert!(
            low < 50.0,
            "1% low should be impacted by slow frames, got {low}"
        );
    }

    #[test]
    fn test_perf_summary_format() {
        let summary = PerfSummary {
            fps: 60.0,
            fps_1_percent_low: 55.0,
            frame_time_ms: 16.67,
            update_time_ms: 2.0,
            render_time_ms: 8.0,
            chunks_loaded: 9,
            cells_simulated: 589824,
            camera_position: (128.5, 100.2),
            zoom: 4.0,
            player_position: (128.5, 100.2),
            player_velocity: (0.0, 0.0),
        };

        let text = summary.format_debug();
        assert!(text.contains("FPS: 60"));
        assert!(text.contains("1% low: 55"));
        assert!(text.contains("589,824"));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(999), "999");
    }

    #[test]
    fn test_scoped_timer() {
        let timer = ScopedTimer::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.stop();
        assert!(elapsed >= 0.009, "Timer should have measured ~10ms");
    }

    #[test]
    fn test_perf_metrics_world_stats() {
        let mut metrics = PerfMetrics::new(60);
        metrics.set_world_stats(9, 589824);

        let summary = metrics.summary();
        assert_eq!(summary.chunks_loaded, 9);
        assert_eq!(summary.cells_simulated, 589824);
    }

    #[test]
    fn test_perf_metrics_clear() {
        let mut metrics = PerfMetrics::new(60);
        metrics.record_frame(0.016, 0.002, 0.008);
        assert!(metrics.avg_fps() > 0.0);

        metrics.clear();
        assert_eq!(metrics.avg_fps(), 0.0);
    }
}
