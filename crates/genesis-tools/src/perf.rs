//! Performance tracking and HUD.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Performance statistics.
#[derive(Debug, Clone, Default)]
pub struct PerfStats {
    /// Frames per second
    pub fps: f64,
    /// Frame time in milliseconds
    pub frame_time_ms: f64,
    /// GPU time in milliseconds (if available)
    pub gpu_time_ms: Option<f64>,
    /// Loaded chunk count
    pub loaded_chunks: usize,
    /// Entity count
    pub entity_count: usize,
    /// Memory usage in bytes
    pub memory_bytes: usize,
}

/// Tracks performance metrics.
#[derive(Debug)]
pub struct PerfTracker {
    /// Frame time history
    frame_times: VecDeque<Duration>,
    /// Target FPS for calculations
    target_fps: u32,
    /// Maximum history length
    history_length: usize,
    /// Last frame start time
    frame_start: Option<Instant>,
    /// Total frame count
    frame_count: u64,
}

impl PerfTracker {
    /// Creates a new tracker with target FPS.
    #[must_use]
    pub fn new(target_fps: u32) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            target_fps,
            history_length: 120,
            frame_start: None,
            frame_count: 0,
        }
    }

    /// Marks the start of a frame.
    pub fn frame_start(&mut self) {
        self.frame_start = Some(Instant::now());
    }

    /// Marks the end of a frame.
    pub fn frame_end(&mut self) {
        if let Some(start) = self.frame_start.take() {
            let duration = start.elapsed();
            if self.frame_times.len() >= self.history_length {
                self.frame_times.pop_front();
            }
            self.frame_times.push_back(duration);
            self.frame_count += 1;
        }
    }

    /// Returns the average FPS over recent frames.
    #[must_use]
    pub fn average_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.frame_times.iter().sum();
        let avg_frame_time = total.as_secs_f64() / self.frame_times.len() as f64;
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Returns the average frame time in milliseconds.
    #[must_use]
    pub fn average_frame_time_ms(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.frame_times.iter().sum();
        total.as_secs_f64() * 1000.0 / self.frame_times.len() as f64
    }

    /// Returns the worst frame time in milliseconds.
    #[must_use]
    pub fn worst_frame_time_ms(&self) -> f64 {
        self.frame_times
            .iter()
            .max()
            .map_or(0.0, |d| d.as_secs_f64() * 1000.0)
    }

    /// Returns whether we're hitting target FPS.
    #[must_use]
    pub fn is_hitting_target(&self) -> bool {
        self.average_fps() >= self.target_fps as f64 * 0.95
    }

    /// Returns total frame count.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Generates current performance stats.
    #[must_use]
    pub fn stats(&self) -> PerfStats {
        PerfStats {
            fps: self.average_fps(),
            frame_time_ms: self.average_frame_time_ms(),
            gpu_time_ms: None,
            loaded_chunks: 0,
            entity_count: 0,
            memory_bytes: 0,
        }
    }
}
