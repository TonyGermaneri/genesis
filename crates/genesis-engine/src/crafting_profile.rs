//! Crafting profiling and performance metrics.
//!
//! This module provides:
//! - Recipe search performance measurement
//! - Crafting frequency statistics
//! - Memory usage tracking for recipe database

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tracing::debug;

/// Performance metrics for recipe operations.
#[derive(Debug, Clone, Default)]
pub struct RecipeSearchMetrics {
    /// Total searches performed.
    pub total_searches: u64,
    /// Total time spent searching.
    pub total_search_time: Duration,
    /// Maximum search time observed.
    pub max_search_time: Duration,
    /// Minimum search time observed (None if no searches yet).
    pub min_search_time: Option<Duration>,
    /// Search times by query length bucket.
    pub by_query_length: HashMap<usize, QueryLengthBucket>,
}

/// Metrics for a specific query length bucket.
#[derive(Debug, Clone, Default)]
pub struct QueryLengthBucket {
    /// Number of searches in this bucket.
    pub count: u64,
    /// Total time for this bucket.
    pub total_time: Duration,
}

impl RecipeSearchMetrics {
    /// Creates new empty metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a search operation.
    pub fn record_search(&mut self, query_length: usize, duration: Duration) {
        self.total_searches += 1;
        self.total_search_time += duration;

        if duration > self.max_search_time {
            self.max_search_time = duration;
        }

        match self.min_search_time {
            Some(min) if duration < min => self.min_search_time = Some(duration),
            None => self.min_search_time = Some(duration),
            _ => {}
        }

        let bucket = self.by_query_length.entry(query_length).or_default();
        bucket.count += 1;
        bucket.total_time += duration;
    }

    /// Returns average search time.
    #[must_use]
    pub fn average_search_time(&self) -> Duration {
        if self.total_searches == 0 {
            return Duration::ZERO;
        }
        self.total_search_time / self.total_searches as u32
    }

    /// Returns searches per second (throughput).
    #[must_use]
    pub fn searches_per_second(&self) -> f64 {
        if self.total_search_time.is_zero() {
            return 0.0;
        }
        self.total_searches as f64 / self.total_search_time.as_secs_f64()
    }

    /// Resets all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Crafting frequency statistics.
#[derive(Debug, Clone, Default)]
pub struct CraftingFrequencyStats {
    /// Crafts per category.
    pub by_category: HashMap<String, u64>,
    /// Crafts per hour of playtime.
    pub crafts_per_hour: f64,
    /// Total playtime tracked (seconds).
    pub playtime_tracked: f64,
    /// Recipe craft counts.
    pub recipe_counts: HashMap<u32, u64>,
    /// Most popular recipe ID and count.
    pub most_popular: Option<(u32, u64)>,
    /// Least crafted recipe ID and count (among those crafted at least once).
    pub least_popular: Option<(u32, u64)>,
}

impl CraftingFrequencyStats {
    /// Creates new empty stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a craft.
    pub fn record_craft(&mut self, recipe_id: u32, category: &str) {
        *self.by_category.entry(category.to_string()).or_insert(0) += 1;
        *self.recipe_counts.entry(recipe_id).or_insert(0) += 1;
        self.update_popularity();
    }

    /// Updates playtime and recalculates crafts per hour.
    pub fn update_playtime(&mut self, playtime_seconds: f64) {
        self.playtime_tracked = playtime_seconds;
        let total_crafts: u64 = self.recipe_counts.values().sum();
        if playtime_seconds > 0.0 {
            self.crafts_per_hour = (total_crafts as f64 / playtime_seconds) * 3600.0;
        }
    }

    /// Updates most/least popular recipes.
    fn update_popularity(&mut self) {
        self.most_popular = self
            .recipe_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(id, count)| (*id, *count));

        self.least_popular = self
            .recipe_counts
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(id, count)| (*id, *count));
    }

    /// Returns top N most crafted recipes.
    #[must_use]
    pub fn top_recipes(&self, n: usize) -> Vec<(u32, u64)> {
        let mut sorted: Vec<_> = self.recipe_counts.iter().map(|(k, v)| (*k, *v)).collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(n);
        sorted
    }

    /// Returns crafts in a specific category.
    #[must_use]
    pub fn crafts_in_category(&self, category: &str) -> u64 {
        self.by_category.get(category).copied().unwrap_or(0)
    }

    /// Resets all stats.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Memory usage tracking for recipe database.
#[derive(Debug, Clone, Default)]
pub struct RecipeMemoryUsage {
    /// Number of recipes loaded.
    pub recipe_count: usize,
    /// Estimated memory for recipe definitions (bytes).
    pub recipe_definitions_bytes: usize,
    /// Memory for name index (bytes).
    pub name_index_bytes: usize,
    /// Memory for category index (bytes).
    pub category_index_bytes: usize,
    /// Total estimated memory (bytes).
    pub total_bytes: usize,
}

impl RecipeMemoryUsage {
    /// Creates new empty usage tracking.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Estimates memory usage for a recipe registry.
    pub fn estimate_from_registry(
        &mut self,
        recipe_count: usize,
        avg_name_length: usize,
        category_count: usize,
    ) {
        self.recipe_count = recipe_count;

        // Rough estimates:
        // - Each RecipeDefinition: ~200 bytes base + ingredients/tools
        // - Average 3 ingredients per recipe: 3 * 12 bytes = 36 bytes
        // - Average 1 tool: 4 bytes
        // - String allocations for name/description: ~100 bytes average
        self.recipe_definitions_bytes = recipe_count * (200 + 36 + 4 + 100);

        // Name index: HashMap overhead + string keys
        self.name_index_bytes = recipe_count * (avg_name_length + 32);

        // Category index: HashMap with Vec values
        self.category_index_bytes = category_count * 64 + recipe_count * 8;

        self.total_bytes =
            self.recipe_definitions_bytes + self.name_index_bytes + self.category_index_bytes;
    }

    /// Returns memory in kilobytes.
    #[must_use]
    pub fn total_kb(&self) -> f64 {
        self.total_bytes as f64 / 1024.0
    }

    /// Returns memory in megabytes.
    #[must_use]
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Returns memory per recipe (average bytes).
    #[must_use]
    pub fn bytes_per_recipe(&self) -> f64 {
        if self.recipe_count == 0 {
            return 0.0;
        }
        self.total_bytes as f64 / self.recipe_count as f64
    }
}

/// Complete crafting profiler.
pub struct CraftingProfiler {
    /// Search performance metrics.
    pub search_metrics: RecipeSearchMetrics,
    /// Crafting frequency stats.
    pub frequency_stats: CraftingFrequencyStats,
    /// Memory usage tracking.
    pub memory_usage: RecipeMemoryUsage,
    /// Active search timer (for in-progress searches).
    active_search_start: Option<Instant>,
    /// Active search query length.
    active_search_query_len: usize,
    /// Profiling enabled flag.
    enabled: bool,
}

impl Default for CraftingProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl CraftingProfiler {
    /// Creates a new profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            search_metrics: RecipeSearchMetrics::new(),
            frequency_stats: CraftingFrequencyStats::new(),
            memory_usage: RecipeMemoryUsage::new(),
            active_search_start: None,
            active_search_query_len: 0,
            enabled: cfg!(debug_assertions), // Enable in debug builds
        }
    }

    /// Creates a profiler with explicit enabled state.
    #[must_use]
    pub fn with_enabled(enabled: bool) -> Self {
        let mut profiler = Self::new();
        profiler.enabled = enabled;
        profiler
    }

    /// Returns whether profiling is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables or disables profiling.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Starts timing a search operation.
    pub fn start_search(&mut self, query_length: usize) {
        if self.enabled {
            self.active_search_start = Some(Instant::now());
            self.active_search_query_len = query_length;
        }
    }

    /// Ends timing a search operation.
    pub fn end_search(&mut self) {
        if self.enabled {
            if let Some(start) = self.active_search_start.take() {
                let duration = start.elapsed();
                self.search_metrics
                    .record_search(self.active_search_query_len, duration);
                debug!(
                    "Recipe search took {:?} (query len: {})",
                    duration, self.active_search_query_len
                );
            }
        }
    }

    /// Records a timed search (convenience method).
    pub fn time_search<T, F: FnOnce() -> T>(&mut self, query_length: usize, f: F) -> T {
        self.start_search(query_length);
        let result = f();
        self.end_search();
        result
    }

    /// Records a craft event.
    pub fn record_craft(&mut self, recipe_id: u32, category: &str) {
        if self.enabled {
            self.frequency_stats.record_craft(recipe_id, category);
        }
    }

    /// Updates playtime for frequency calculations.
    pub fn update_playtime(&mut self, playtime_seconds: f64) {
        if self.enabled {
            self.frequency_stats.update_playtime(playtime_seconds);
        }
    }

    /// Updates memory usage estimates.
    pub fn update_memory_usage(
        &mut self,
        recipe_count: usize,
        avg_name_length: usize,
        category_count: usize,
    ) {
        if self.enabled {
            self.memory_usage
                .estimate_from_registry(recipe_count, avg_name_length, category_count);
        }
    }

    /// Resets all profiling data.
    pub fn reset(&mut self) {
        self.search_metrics.reset();
        self.frequency_stats.reset();
        self.memory_usage = RecipeMemoryUsage::new();
    }

    /// Generates a summary report.
    #[must_use]
    #[allow(clippy::format_push_string)]
    pub fn summary_report(&self) -> String {
        use std::fmt::Write;
        let mut report = String::new();

        report.push_str("=== Crafting Profiler Report ===\n\n");

        // Search metrics
        report.push_str("Search Performance:\n");
        let _ = writeln!(report, "  Total searches: {}", self.search_metrics.total_searches);
        let _ = writeln!(report, "  Average search time: {:?}", self.search_metrics.average_search_time());
        let _ = writeln!(report, "  Max search time: {:?}", self.search_metrics.max_search_time);
        if let Some(min) = self.search_metrics.min_search_time {
            let _ = writeln!(report, "  Min search time: {min:?}");
        }
        let _ = writeln!(report, "  Searches/second: {:.2}", self.search_metrics.searches_per_second());
        report.push('\n');

        // Frequency stats
        report.push_str("Crafting Frequency:\n");
        let _ = writeln!(report, "  Crafts per hour: {:.2}", self.frequency_stats.crafts_per_hour);
        if let Some((id, count)) = self.frequency_stats.most_popular {
            let _ = writeln!(report, "  Most popular recipe: {id} ({count} crafts)");
        }
        report.push_str("  Top categories:\n");
        for (category, count) in &self.frequency_stats.by_category {
            let _ = writeln!(report, "    {category}: {count}");
        }
        report.push('\n');

        // Memory usage
        report.push_str("Memory Usage:\n");
        let _ = writeln!(report, "  Recipe count: {}", self.memory_usage.recipe_count);
        let _ = writeln!(report, "  Total memory: {:.2} KB ({:.3} MB)", self.memory_usage.total_kb(), self.memory_usage.total_mb());
        let _ = writeln!(report, "  Bytes per recipe: {:.1}", self.memory_usage.bytes_per_recipe());

        report
    }
}

/// RAII guard for timing search operations.
pub struct SearchTimer<'a> {
    profiler: &'a mut CraftingProfiler,
}

impl<'a> SearchTimer<'a> {
    /// Creates a new search timer.
    #[must_use]
    pub fn new(profiler: &'a mut CraftingProfiler, query_length: usize) -> Self {
        profiler.start_search(query_length);
        Self { profiler }
    }
}

impl Drop for SearchTimer<'_> {
    fn drop(&mut self) {
        self.profiler.end_search();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_metrics_record() {
        let mut metrics = RecipeSearchMetrics::new();

        metrics.record_search(5, Duration::from_micros(100));
        metrics.record_search(5, Duration::from_micros(200));
        metrics.record_search(10, Duration::from_micros(150));

        assert_eq!(metrics.total_searches, 3);
        assert_eq!(metrics.max_search_time, Duration::from_micros(200));
        assert_eq!(metrics.min_search_time, Some(Duration::from_micros(100)));
    }

    #[test]
    fn test_search_metrics_average() {
        let mut metrics = RecipeSearchMetrics::new();

        metrics.record_search(5, Duration::from_micros(100));
        metrics.record_search(5, Duration::from_micros(200));

        let avg = metrics.average_search_time();
        assert_eq!(avg, Duration::from_micros(150));
    }

    #[test]
    fn test_frequency_stats_record() {
        let mut stats = CraftingFrequencyStats::new();

        stats.record_craft(1, "weapons");
        stats.record_craft(1, "weapons");
        stats.record_craft(2, "armor");

        assert_eq!(stats.crafts_in_category("weapons"), 2);
        assert_eq!(stats.crafts_in_category("armor"), 1);
        assert_eq!(stats.recipe_counts.get(&1), Some(&2));
    }

    #[test]
    fn test_frequency_stats_top_recipes() {
        let mut stats = CraftingFrequencyStats::new();

        for _ in 0..5 {
            stats.record_craft(1, "weapons");
        }
        for _ in 0..3 {
            stats.record_craft(2, "armor");
        }
        stats.record_craft(3, "tools");

        let top = stats.top_recipes(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], (1, 5));
        assert_eq!(top[1], (2, 3));
    }

    #[test]
    fn test_frequency_stats_crafts_per_hour() {
        let mut stats = CraftingFrequencyStats::new();

        stats.record_craft(1, "weapons");
        stats.record_craft(2, "armor");

        // Simulate 30 minutes of playtime
        stats.update_playtime(1800.0);

        // 2 crafts in 30 minutes = 4 crafts per hour
        assert!((stats.crafts_per_hour - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_memory_usage_estimate() {
        let mut usage = RecipeMemoryUsage::new();

        usage.estimate_from_registry(100, 20, 10);

        assert_eq!(usage.recipe_count, 100);
        assert!(usage.total_bytes > 0);
        assert!(usage.total_kb() > 0.0);
    }

    #[test]
    fn test_profiler_search_timing() {
        let mut profiler = CraftingProfiler::with_enabled(true);

        profiler.start_search(5);
        std::thread::sleep(Duration::from_millis(1));
        profiler.end_search();

        assert_eq!(profiler.search_metrics.total_searches, 1);
        assert!(profiler.search_metrics.max_search_time >= Duration::from_millis(1));
    }

    #[test]
    fn test_profiler_time_search() {
        let mut profiler = CraftingProfiler::with_enabled(true);

        let result = profiler.time_search(5, || 42);

        assert_eq!(result, 42);
        assert_eq!(profiler.search_metrics.total_searches, 1);
    }

    #[test]
    fn test_profiler_disabled() {
        let mut profiler = CraftingProfiler::with_enabled(false);

        profiler.record_craft(1, "weapons");
        profiler.start_search(5);
        profiler.end_search();

        // Nothing should be recorded when disabled
        assert_eq!(profiler.search_metrics.total_searches, 0);
        assert!(profiler.frequency_stats.recipe_counts.is_empty());
    }

    #[test]
    fn test_profiler_summary_report() {
        let mut profiler = CraftingProfiler::with_enabled(true);

        profiler.record_craft(1, "weapons");
        profiler.update_memory_usage(50, 15, 5);

        let report = profiler.summary_report();
        assert!(report.contains("Crafting Profiler Report"));
        assert!(report.contains("weapons"));
    }
}
