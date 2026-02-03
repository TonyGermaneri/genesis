//! Benchmarking utilities for GPU compute performance.
//!
//! This module provides tools to measure and report compute dispatch performance
//! for various chunk sizes. Use these benchmarks to verify performance targets
//! are met (e.g., <1ms dispatch for 256x256 grids).

use std::time::{Duration, Instant};
use tracing::info;

/// Benchmark result for a single compute dispatch.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Chunk size (width and height in cells)
    pub chunk_size: u32,
    /// Total number of cells
    pub cell_count: usize,
    /// Dispatch time (CPU-side command recording)
    pub dispatch_time: Duration,
    /// GPU execution time (if available via timestamp queries)
    pub gpu_time: Option<Duration>,
    /// Whether the target was met
    pub target_met: bool,
    /// Target time for this chunk size
    pub target_time: Duration,
}

impl BenchmarkResult {
    /// Creates a new benchmark result.
    #[must_use]
    pub fn new(chunk_size: u32, dispatch_time: Duration, target_time: Duration) -> Self {
        Self {
            chunk_size,
            cell_count: (chunk_size * chunk_size) as usize,
            dispatch_time,
            gpu_time: None,
            target_met: dispatch_time <= target_time,
            target_time,
        }
    }

    /// Sets the GPU execution time.
    pub fn with_gpu_time(mut self, gpu_time: Duration) -> Self {
        self.gpu_time = Some(gpu_time);
        self
    }

    /// Returns the dispatch time in milliseconds.
    #[must_use]
    pub fn dispatch_ms(&self) -> f64 {
        self.dispatch_time.as_secs_f64() * 1000.0
    }

    /// Returns the GPU time in milliseconds (if available).
    #[must_use]
    pub fn gpu_ms(&self) -> Option<f64> {
        self.gpu_time.map(|d| d.as_secs_f64() * 1000.0)
    }

    /// Formats the result as a human-readable string.
    #[must_use]
    pub fn format(&self) -> String {
        let status = if self.target_met { "✅" } else { "❌" };
        let gpu_str = self
            .gpu_ms()
            .map(|ms| format!(", gpu: {ms:.3}ms"))
            .unwrap_or_default();

        format!(
            "{status} {}x{} ({} cells): {:.3}ms{gpu_str} (target: {:.1}ms)",
            self.chunk_size,
            self.chunk_size,
            self.cell_count,
            self.dispatch_ms(),
            self.target_time.as_secs_f64() * 1000.0
        )
    }
}

/// Default performance targets for various chunk sizes.
#[must_use]
pub fn default_targets() -> Vec<(u32, Duration)> {
    vec![
        (256, Duration::from_millis(1)),   // 256x256: target <1ms
        (512, Duration::from_millis(4)),   // 512x512: target <4ms
        (1024, Duration::from_millis(16)), // 1024x1024: target <16ms
    ]
}

/// A simple timer for measuring durations.
#[derive(Debug)]
pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    /// Creates and starts a new timer.
    #[must_use]
    pub fn start(label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    /// Returns the elapsed time since the timer started.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Returns the elapsed time in milliseconds.
    #[must_use]
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }

    /// Stops the timer and logs the elapsed time.
    pub fn stop_and_log(self) -> Duration {
        let elapsed = self.elapsed();
        info!("{}: {:.3}ms", self.label, elapsed.as_secs_f64() * 1000.0);
        elapsed
    }
}

/// Benchmark suite for compute dispatch performance.
#[derive(Debug, Default)]
pub struct BenchmarkSuite {
    /// Collected results
    results: Vec<BenchmarkResult>,
}

impl BenchmarkSuite {
    /// Creates a new empty benchmark suite.
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Adds a benchmark result.
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    /// Records a benchmark measurement.
    pub fn record(&mut self, chunk_size: u32, dispatch_time: Duration, target_time: Duration) {
        self.results
            .push(BenchmarkResult::new(chunk_size, dispatch_time, target_time));
    }

    /// Returns all results.
    #[must_use]
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Returns the number of benchmarks that met their targets.
    #[must_use]
    pub fn targets_met(&self) -> usize {
        self.results.iter().filter(|r| r.target_met).count()
    }

    /// Returns whether all benchmarks met their targets.
    #[must_use]
    pub fn all_targets_met(&self) -> bool {
        self.results.iter().all(|r| r.target_met)
    }

    /// Prints a summary of all results.
    pub fn print_summary(&self) {
        info!("=== Benchmark Results ===");
        for result in &self.results {
            info!("{}", result.format());
        }
        info!("Targets met: {}/{}", self.targets_met(), self.results.len());
    }

    /// Formats results as a string for output.
    #[must_use]
    pub fn format_summary(&self) -> String {
        let mut lines = vec!["=== Benchmark Results ===".to_string()];
        for result in &self.results {
            lines.push(result.format());
        }
        lines.push(format!(
            "Targets met: {}/{}",
            self.targets_met(),
            self.results.len()
        ));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_target_met() {
        let result =
            BenchmarkResult::new(256, Duration::from_micros(500), Duration::from_millis(1));
        assert!(result.target_met);
        assert_eq!(result.chunk_size, 256);
        assert_eq!(result.cell_count, 65536);
    }

    #[test]
    fn test_benchmark_result_target_missed() {
        let result = BenchmarkResult::new(256, Duration::from_millis(2), Duration::from_millis(1));
        assert!(!result.target_met);
    }

    #[test]
    fn test_timer_elapsed() {
        let timer = Timer::start("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new();
        suite.record(256, Duration::from_micros(500), Duration::from_millis(1));
        suite.record(512, Duration::from_millis(5), Duration::from_millis(4));

        assert_eq!(suite.results().len(), 2);
        assert_eq!(suite.targets_met(), 1);
        assert!(!suite.all_targets_met());
    }

    #[test]
    fn test_default_targets() {
        let targets = default_targets();
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].0, 256);
        assert_eq!(targets[1].0, 512);
        assert_eq!(targets[2].0, 1024);
    }
}
