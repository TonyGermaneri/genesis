//! Frame timing and performance tracking.
//!
//! Provides smooth delta time calculation, fixed timestep for physics,
//! FPS limiting, and chunk-specific metrics.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame timing manager.
#[derive(Debug)]
#[allow(dead_code)]
pub struct FrameTiming {
    /// Target frames per second
    target_fps: u32,
    /// Time budget per frame
    frame_budget: Duration,
    /// Time of last frame start
    last_frame: Instant,
    /// Accumulator for fixed timestep
    accumulator: f32,
    /// Fixed timestep delta (for physics)
    fixed_dt: f32,
    /// Maximum delta time to prevent spiral of death
    max_dt: f32,
    /// Whether VSync is enabled (disables manual frame limiting)
    vsync: bool,
    /// Recent frame times for averaging
    frame_times: VecDeque<f32>,
    /// Maximum samples for averaging
    max_samples: usize,
}

impl Default for FrameTiming {
    fn default() -> Self {
        Self::new(60)
    }
}

#[allow(dead_code)]
impl FrameTiming {
    /// Create a new frame timing manager.
    ///
    /// # Arguments
    /// * `target_fps` - Target frames per second for frame limiting
    #[must_use]
    pub fn new(target_fps: u32) -> Self {
        let target_fps = target_fps.max(1);
        Self {
            target_fps,
            frame_budget: Duration::from_secs_f64(1.0 / f64::from(target_fps)),
            last_frame: Instant::now(),
            accumulator: 0.0,
            fixed_dt: 1.0 / 60.0, // Fixed 60Hz physics
            max_dt: 0.25,         // Max 250ms delta (prevents spiral of death)
            vsync: true,
            frame_times: VecDeque::with_capacity(120),
            max_samples: 120,
        }
    }

    /// Create with VSync setting.
    #[must_use]
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Set the fixed timestep for physics updates.
    pub fn set_fixed_dt(&mut self, dt: f32) {
        self.fixed_dt = dt.max(0.001); // Minimum 1ms
    }

    /// Get the fixed timestep value.
    #[must_use]
    pub fn fixed_dt(&self) -> f32 {
        self.fixed_dt
    }

    /// Calculate delta time since last frame.
    /// Also stores the frame time for FPS calculation.
    pub fn delta_time(&mut self) -> f32 {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Clamp to prevent spiral of death
        let clamped_dt = dt.min(self.max_dt);

        // Store for averaging
        self.frame_times.push_back(clamped_dt);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }

        clamped_dt
    }

    /// Accumulate time for fixed timestep updates.
    /// Returns the number of fixed updates that should be performed.
    pub fn accumulate(&mut self, dt: f32) -> u32 {
        self.accumulator += dt;
        let mut count = 0;

        // Limit to prevent spiral of death
        let max_updates = 10;
        while self.accumulator >= self.fixed_dt && count < max_updates {
            self.accumulator -= self.fixed_dt;
            count += 1;
        }

        // If we're still behind, reset accumulator
        if self.accumulator > self.fixed_dt * 2.0 {
            self.accumulator = 0.0;
        }

        count
    }

    /// Check if a fixed update should be performed.
    /// Decrements the accumulator if true.
    pub fn should_update_fixed(&mut self) -> bool {
        if self.accumulator >= self.fixed_dt {
            self.accumulator -= self.fixed_dt;
            true
        } else {
            false
        }
    }

    /// Sleep for the remainder of the frame budget (if VSync is off).
    pub fn sleep_remainder(&self) {
        if self.vsync {
            return;
        }

        let elapsed = self.last_frame.elapsed();
        if elapsed < self.frame_budget {
            let sleep_time = self.frame_budget - elapsed;
            // Use spin sleep for more accurate timing on short durations
            if sleep_time > Duration::from_millis(1) {
                std::thread::sleep(sleep_time - Duration::from_millis(1));
            }
            // Spin for the remainder
            while self.last_frame.elapsed() < self.frame_budget {
                std::hint::spin_loop();
            }
        }
    }

    /// Get the current FPS (averaged over recent frames).
    #[must_use]
    pub fn current_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let avg_frame_time: f32 =
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Get the average frame time in milliseconds.
    #[must_use]
    pub fn average_frame_time_ms(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        (self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32) * 1000.0
    }

    /// Get the target FPS.
    #[must_use]
    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    /// Set the target FPS.
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps.max(1);
        self.frame_budget = Duration::from_secs_f64(1.0 / f64::from(self.target_fps));
    }

    /// Reset timing (call after pause or loading).
    pub fn reset(&mut self) {
        self.last_frame = Instant::now();
        self.accumulator = 0.0;
        self.frame_times.clear();
    }
}

/// Simple FPS counter for HUD display.
#[derive(Debug)]
pub struct FpsCounter {
    /// Frame count since last update
    frame_count: u32,
    /// Time of last FPS calculation
    last_update: Instant,
    /// Update interval
    update_interval: Duration,
    /// Current FPS value
    current_fps: f32,
    /// Current frame time in ms
    current_frame_time: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl FpsCounter {
    /// Create a new FPS counter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500),
            current_fps: 0.0,
            current_frame_time: 0.0,
        }
    }

    /// Tick the counter. Returns (fps, frame_time_ms) if updated this frame.
    pub fn tick(&mut self) -> (f32, f32) {
        self.frame_count += 1;

        let elapsed = self.last_update.elapsed();
        if elapsed >= self.update_interval {
            let secs = elapsed.as_secs_f32();
            self.current_fps = self.frame_count as f32 / secs;
            self.current_frame_time = (secs / self.frame_count as f32) * 1000.0;
            self.frame_count = 0;
            self.last_update = Instant::now();
        }

        (self.current_fps, self.current_frame_time)
    }

    /// Get current FPS.
    #[must_use]
    pub fn fps(&self) -> f32 {
        self.current_fps
    }

    /// Get current frame time in milliseconds.
    #[must_use]
    pub fn frame_time_ms(&self) -> f32 {
        self.current_frame_time
    }
}

/// Metrics for NPC system performance.
#[derive(Debug, Clone)]
pub struct NpcMetrics {
    /// Recent NPC AI update times
    ai_times: VecDeque<Duration>,
    /// Peak AI update time in window
    peak_ai_time: Duration,
    /// Maximum samples to keep
    max_samples: usize,
    /// Current NPC count
    npc_count: usize,
    /// Frame time budget in milliseconds for NPC updates
    frame_budget_ms: f64,
}

impl Default for NpcMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl NpcMetrics {
    /// Creates a new NPC metrics tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ai_times: VecDeque::with_capacity(60),
            peak_ai_time: Duration::ZERO,
            max_samples: 60,
            npc_count: 0,
            frame_budget_ms: 2.0, // 2ms budget for NPC updates
        }
    }

    /// Records an AI update time.
    pub fn record_ai_time(&mut self, duration: Duration) {
        // Track peak
        if duration > self.peak_ai_time {
            self.peak_ai_time = duration;
        }

        self.ai_times.push_back(duration);
        if self.ai_times.len() > self.max_samples {
            // When removing old samples, recalculate peak
            self.ai_times.pop_front();
            self.recalculate_peak();
        }
    }

    /// Recalculates the peak AI time from current samples.
    fn recalculate_peak(&mut self) {
        self.peak_ai_time = self.ai_times.iter().copied().max().unwrap_or(Duration::ZERO);
    }

    /// Sets the current NPC count.
    pub fn set_npc_count(&mut self, count: usize) {
        self.npc_count = count;
    }

    /// Returns the current NPC count.
    #[must_use]
    pub fn npc_count(&self) -> usize {
        self.npc_count
    }

    /// Returns the average AI update time in milliseconds.
    #[must_use]
    pub fn avg_ai_time_ms(&self) -> f64 {
        if self.ai_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.ai_times.iter().sum();
        total.as_secs_f64() * 1000.0 / self.ai_times.len() as f64
    }

    /// Returns the peak AI update time in milliseconds.
    #[must_use]
    pub fn peak_ai_time_ms(&self) -> f64 {
        self.peak_ai_time.as_secs_f64() * 1000.0
    }

    /// Returns the average time per NPC in microseconds.
    #[must_use]
    pub fn avg_time_per_npc_us(&self) -> f64 {
        if self.npc_count == 0 {
            return 0.0;
        }
        self.avg_ai_time_ms() * 1000.0 / self.npc_count as f64
    }

    /// Returns whether NPC processing exceeds the budget.
    #[must_use]
    pub fn exceeds_budget(&self) -> bool {
        self.avg_ai_time_ms() > self.frame_budget_ms
    }

    /// Sets the frame budget for NPC updates.
    pub fn set_budget(&mut self, budget_ms: f64) {
        self.frame_budget_ms = budget_ms;
    }

    /// Clears all recorded metrics.
    pub fn clear(&mut self) {
        self.ai_times.clear();
        self.peak_ai_time = Duration::ZERO;
    }
}

/// Metrics for chunk loading and simulation performance.
#[derive(Debug, Clone)]
pub struct ChunkMetrics {
    /// Recent chunk load times
    load_times: VecDeque<Duration>,
    /// Recent simulation times per frame
    sim_times: VecDeque<Duration>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Current chunk count
    chunk_count: u32,
    /// Frame time budget in milliseconds
    frame_budget_ms: f64,
}

impl Default for ChunkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ChunkMetrics {
    /// Creates a new chunk metrics tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            load_times: VecDeque::with_capacity(60),
            sim_times: VecDeque::with_capacity(60),
            max_samples: 60,
            chunk_count: 0,
            frame_budget_ms: 16.67, // 60 FPS target
        }
    }

    /// Records a chunk load time.
    pub fn record_load_time(&mut self, duration: Duration) {
        self.load_times.push_back(duration);
        if self.load_times.len() > self.max_samples {
            self.load_times.pop_front();
        }
    }

    /// Records simulation time for a frame.
    pub fn record_sim_time(&mut self, duration: Duration) {
        self.sim_times.push_back(duration);
        if self.sim_times.len() > self.max_samples {
            self.sim_times.pop_front();
        }
    }

    /// Sets the current chunk count.
    pub fn set_chunk_count(&mut self, count: u32) {
        self.chunk_count = count;
    }

    /// Returns the current chunk count.
    #[must_use]
    pub fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    /// Returns the average load time in milliseconds.
    #[must_use]
    pub fn avg_load_time_ms(&self) -> f64 {
        if self.load_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.load_times.iter().sum();
        total.as_secs_f64() * 1000.0 / self.load_times.len() as f64
    }

    /// Returns the average simulation time in milliseconds.
    #[must_use]
    pub fn avg_sim_time_ms(&self) -> f64 {
        if self.sim_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.sim_times.iter().sum();
        total.as_secs_f64() * 1000.0 / self.sim_times.len() as f64
    }

    /// Returns the total average chunk processing time in milliseconds.
    #[must_use]
    pub fn total_chunk_time_ms(&self) -> f64 {
        self.avg_load_time_ms() + self.avg_sim_time_ms()
    }

    /// Returns whether chunk processing exceeds the frame budget.
    #[must_use]
    pub fn exceeds_budget(&self) -> bool {
        self.total_chunk_time_ms() > self.frame_budget_ms * 0.5 // 50% of frame budget
    }

    /// Sets the frame budget (based on target FPS).
    pub fn set_frame_budget(&mut self, target_fps: u32) {
        self.frame_budget_ms = 1000.0 / f64::from(target_fps.max(1));
    }

    /// Clears all recorded metrics.
    pub fn clear(&mut self) {
        self.load_times.clear();
        self.sim_times.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_timing_creation() {
        let timing = FrameTiming::new(60);
        assert_eq!(timing.target_fps(), 60);
        assert!((timing.fixed_dt() - 1.0 / 60.0).abs() < 0.001);
    }

    #[test]
    fn test_frame_timing_delta() {
        let mut timing = FrameTiming::new(60);

        // First delta should be very small
        std::thread::sleep(Duration::from_millis(16));
        let dt = timing.delta_time();
        assert!(dt >= 0.015); // At least 15ms
        assert!(dt < 0.5); // Less than max_dt
    }

    #[test]
    fn test_frame_timing_max_dt() {
        let mut timing = FrameTiming::new(60);

        // Sleep longer than max_dt
        std::thread::sleep(Duration::from_millis(300));
        let dt = timing.delta_time();

        // Should be clamped to max_dt
        assert!(dt <= timing.max_dt);
    }

    #[test]
    fn test_fixed_timestep() {
        let mut timing = FrameTiming::new(60);
        timing.set_fixed_dt(1.0 / 60.0);

        // Simulate 32ms frame (should trigger ~2 fixed updates)
        let updates = timing.accumulate(0.032);
        assert!(updates == 1 || updates == 2);
    }

    #[test]
    fn test_fps_counter() {
        let mut counter = FpsCounter::new();

        // Tick several times
        for _ in 0..10 {
            counter.tick();
            std::thread::sleep(Duration::from_millis(10));
        }

        // FPS should be calculable
        assert!(counter.fps() >= 0.0);
    }

    #[test]
    fn test_accumulate_spiral_prevention() {
        let mut timing = FrameTiming::new(60);
        timing.set_fixed_dt(1.0 / 60.0);

        // Simulate huge lag spike
        let updates = timing.accumulate(1.0); // 1 second

        // Should be capped to prevent spiral of death
        assert!(updates <= 10);
    }

    #[test]
    fn test_vsync_setting() {
        let timing = FrameTiming::new(60).with_vsync(true);
        assert!(timing.vsync);

        let timing = FrameTiming::new(60).with_vsync(false);
        assert!(!timing.vsync);
    }

    #[test]
    fn test_reset_timing() {
        let mut timing = FrameTiming::new(60);
        timing.accumulator = 0.5;
        timing.frame_times.push_back(0.016);

        timing.reset();

        assert_eq!(timing.accumulator, 0.0);
        assert!(timing.frame_times.is_empty());
    }

    #[test]
    fn test_chunk_metrics_default() {
        let metrics = ChunkMetrics::new();
        assert_eq!(metrics.chunk_count(), 0);
        assert_eq!(metrics.avg_load_time_ms(), 0.0);
        assert_eq!(metrics.avg_sim_time_ms(), 0.0);
        assert!(!metrics.exceeds_budget());
    }

    #[test]
    fn test_chunk_metrics_recording() {
        let mut metrics = ChunkMetrics::new();

        metrics.record_load_time(Duration::from_millis(5));
        metrics.record_load_time(Duration::from_millis(10));
        metrics.record_load_time(Duration::from_millis(15));

        // Average should be 10ms
        let avg = metrics.avg_load_time_ms();
        assert!((avg - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_chunk_metrics_sim_time() {
        let mut metrics = ChunkMetrics::new();

        metrics.record_sim_time(Duration::from_millis(2));
        metrics.record_sim_time(Duration::from_millis(4));

        // Average should be 3ms
        let avg = metrics.avg_sim_time_ms();
        assert!((avg - 3.0).abs() < 0.1);
    }

    #[test]
    fn test_chunk_metrics_chunk_count() {
        let mut metrics = ChunkMetrics::new();
        metrics.set_chunk_count(9);
        assert_eq!(metrics.chunk_count(), 9);
    }

    #[test]
    fn test_chunk_metrics_exceeds_budget() {
        let mut metrics = ChunkMetrics::new();

        // Record many long times to exceed budget (8.33ms = 50% of 16.67ms)
        for _ in 0..10 {
            metrics.record_load_time(Duration::from_millis(5));
            metrics.record_sim_time(Duration::from_millis(5));
        }

        // Total = 10ms > 8.33ms threshold
        assert!(metrics.exceeds_budget());
    }

    #[test]
    fn test_chunk_metrics_clear() {
        let mut metrics = ChunkMetrics::new();
        metrics.record_load_time(Duration::from_millis(10));
        metrics.record_sim_time(Duration::from_millis(5));

        metrics.clear();

        assert_eq!(metrics.avg_load_time_ms(), 0.0);
        assert_eq!(metrics.avg_sim_time_ms(), 0.0);
    }

    #[test]
    fn test_chunk_metrics_total_time() {
        let mut metrics = ChunkMetrics::new();
        metrics.record_load_time(Duration::from_millis(4));
        metrics.record_sim_time(Duration::from_millis(6));

        let total = metrics.total_chunk_time_ms();
        assert!((total - 10.0).abs() < 0.1);
    }
}
