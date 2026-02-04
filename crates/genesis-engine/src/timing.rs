//! Frame timing and performance tracking.
//!
//! Provides smooth delta time calculation, fixed timestep for physics,
//! and FPS limiting.

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
}
