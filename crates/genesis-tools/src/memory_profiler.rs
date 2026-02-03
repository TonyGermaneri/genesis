//! Memory profiler integration for tracking allocations.
//!
//! This module provides:
//! - Memory statistics tracking (manual recording)
//! - Per-system memory usage tracking
//! - Leak detection utilities
//! - Memory budget management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::RwLock;

/// Statistics for memory allocations (profiler-specific to avoid name conflict).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfilerMemoryStats {
    /// Current number of live allocations
    pub allocation_count: u64,
    /// Current total allocated bytes
    pub allocated_bytes: u64,
    /// Peak allocation count
    pub peak_allocation_count: u64,
    /// Peak allocated bytes
    pub peak_allocated_bytes: u64,
    /// Total allocations since start
    pub total_allocations: u64,
    /// Total deallocations since start
    pub total_deallocations: u64,
    /// Total bytes allocated since start
    pub total_bytes_allocated: u64,
    /// Total bytes deallocated since start
    pub total_bytes_deallocated: u64,
}

impl ProfilerMemoryStats {
    /// Returns human-readable allocated bytes.
    #[must_use]
    pub fn allocated_str(&self) -> String {
        format_bytes(self.allocated_bytes)
    }

    /// Returns human-readable peak bytes.
    #[must_use]
    pub fn peak_str(&self) -> String {
        format_bytes(self.peak_allocated_bytes)
    }

    /// Returns human-readable total bytes allocated.
    #[must_use]
    pub fn total_allocated_str(&self) -> String {
        format_bytes(self.total_bytes_allocated)
    }

    /// Returns a summary report.
    #[must_use]
    pub fn to_report(&self) -> String {
        let mut report = String::new();
        let _ = writeln!(report, "=== Memory Stats ===");
        let _ = writeln!(
            report,
            "Current: {} ({} allocations)",
            self.allocated_str(),
            self.allocation_count
        );
        let _ = writeln!(
            report,
            "Peak: {} ({} allocations)",
            self.peak_str(),
            self.peak_allocation_count
        );
        let _ = writeln!(
            report,
            "Total allocated: {} ({} allocations)",
            self.total_allocated_str(),
            self.total_allocations
        );
        let _ = writeln!(report, "Total deallocations: {}", self.total_deallocations);
        report
    }
}

/// Formats bytes as human-readable string.
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

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

/// Global atomic counters for manual allocation tracking.
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static PEAK_ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static TOTAL_ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static TOTAL_DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static TOTAL_BYTES_ALLOCATED: AtomicU64 = AtomicU64::new(0);
static TOTAL_BYTES_DEALLOCATED: AtomicU64 = AtomicU64::new(0);

/// Manually record an allocation (for systems without custom allocator).
pub fn record_alloc(bytes: u64) {
    let count = ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    let total_bytes = ALLOCATED_BYTES.fetch_add(bytes, Ordering::Relaxed) + bytes;

    TOTAL_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    TOTAL_BYTES_ALLOCATED.fetch_add(bytes, Ordering::Relaxed);

    // Update peaks
    let _ = PEAK_ALLOCATION_COUNT.fetch_max(count, Ordering::Relaxed);
    let _ = PEAK_ALLOCATED_BYTES.fetch_max(total_bytes, Ordering::Relaxed);
}

/// Manually record a deallocation (for systems without custom allocator).
pub fn record_dealloc(bytes: u64) {
    ALLOCATION_COUNT.fetch_sub(1, Ordering::Relaxed);
    ALLOCATED_BYTES.fetch_sub(bytes, Ordering::Relaxed);
    TOTAL_DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    TOTAL_BYTES_DEALLOCATED.fetch_add(bytes, Ordering::Relaxed);
}

/// Gets current memory statistics from global counters.
#[must_use]
pub fn get_global_stats() -> ProfilerMemoryStats {
    ProfilerMemoryStats {
        allocation_count: ALLOCATION_COUNT.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        peak_allocation_count: PEAK_ALLOCATION_COUNT.load(Ordering::Relaxed),
        peak_allocated_bytes: PEAK_ALLOCATED_BYTES.load(Ordering::Relaxed),
        total_allocations: TOTAL_ALLOCATIONS.load(Ordering::Relaxed),
        total_deallocations: TOTAL_DEALLOCATIONS.load(Ordering::Relaxed),
        total_bytes_allocated: TOTAL_BYTES_ALLOCATED.load(Ordering::Relaxed),
        total_bytes_deallocated: TOTAL_BYTES_DEALLOCATED.load(Ordering::Relaxed),
    }
}

/// Resets all global counters.
pub fn reset_global_stats() {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    PEAK_ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    PEAK_ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    TOTAL_ALLOCATIONS.store(0, Ordering::Relaxed);
    TOTAL_DEALLOCATIONS.store(0, Ordering::Relaxed);
    TOTAL_BYTES_ALLOCATED.store(0, Ordering::Relaxed);
    TOTAL_BYTES_DEALLOCATED.store(0, Ordering::Relaxed);
}

/// A system identifier for per-system tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(u32);

impl SystemId {
    /// Creates a new system ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID.
    #[must_use]
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Per-system memory tracking.
#[derive(Debug)]
pub struct SystemMemoryTracker {
    /// Name of this tracker
    name: String,
    /// Per-system stats
    systems: RwLock<HashMap<SystemId, SystemMemoryStats>>,
    /// System names for display
    system_names: RwLock<HashMap<SystemId, String>>,
    /// Next system ID
    next_id: AtomicUsize,
}

impl Default for SystemMemoryTracker {
    fn default() -> Self {
        Self::new("default")
    }
}

impl SystemMemoryTracker {
    /// Creates a new system memory tracker.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            systems: RwLock::new(HashMap::new()),
            system_names: RwLock::new(HashMap::new()),
            next_id: AtomicUsize::new(1),
        }
    }

    /// Registers a new system and returns its ID.
    pub fn register_system(&self, name: &str) -> SystemId {
        let id = SystemId::new(self.next_id.fetch_add(1, Ordering::Relaxed) as u32);

        if let Ok(mut names) = self.system_names.write() {
            names.insert(id, name.to_string());
        }

        if let Ok(mut systems) = self.systems.write() {
            systems.insert(id, SystemMemoryStats::default());
        }

        id
    }

    /// Records an allocation for a system.
    pub fn record_alloc(&self, system_id: SystemId, bytes: u64) {
        if let Ok(mut systems) = self.systems.write() {
            if let Some(stats) = systems.get_mut(&system_id) {
                stats.allocation_count += 1;
                stats.allocated_bytes += bytes;
                stats.total_allocations += 1;
                stats.total_bytes_allocated += bytes;
                stats.peak_allocated_bytes = stats.peak_allocated_bytes.max(stats.allocated_bytes);
            }
        }
    }

    /// Records a deallocation for a system.
    pub fn record_dealloc(&self, system_id: SystemId, bytes: u64) {
        if let Ok(mut systems) = self.systems.write() {
            if let Some(stats) = systems.get_mut(&system_id) {
                stats.allocation_count = stats.allocation_count.saturating_sub(1);
                stats.allocated_bytes = stats.allocated_bytes.saturating_sub(bytes);
                stats.total_deallocations += 1;
                stats.total_bytes_deallocated += bytes;
            }
        }
    }

    /// Gets stats for a specific system.
    #[must_use]
    pub fn get_system_stats(&self, system_id: SystemId) -> Option<SystemMemoryStats> {
        self.systems
            .read()
            .ok()
            .and_then(|s| s.get(&system_id).copied())
    }

    /// Gets the name of a system.
    #[must_use]
    pub fn get_system_name(&self, system_id: SystemId) -> Option<String> {
        self.system_names
            .read()
            .ok()
            .and_then(|n| n.get(&system_id).cloned())
    }

    /// Gets all registered system IDs.
    #[must_use]
    pub fn get_all_system_ids(&self) -> Vec<SystemId> {
        self.systems
            .read()
            .ok()
            .map(|s| s.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Generates a report of all system memory usage.
    #[must_use]
    pub fn to_report(&self) -> String {
        let mut report = String::new();
        let _ = writeln!(report, "=== {} Memory Report ===", self.name);

        let systems = match self.systems.read() {
            Ok(s) => s,
            Err(_) => return report,
        };

        let names = match self.system_names.read() {
            Ok(n) => n,
            Err(_) => return report,
        };

        let mut entries: Vec<_> = systems.iter().collect();
        entries.sort_by(|a, b| b.1.allocated_bytes.cmp(&a.1.allocated_bytes));

        for (id, stats) in entries {
            let name = names.get(id).map_or("unknown", String::as_str);
            let _ = writeln!(
                report,
                "  {:20} {:>10} ({} allocs)",
                name,
                format_bytes(stats.allocated_bytes),
                stats.allocation_count
            );
        }

        report
    }

    /// Resets all system stats.
    pub fn reset(&self) {
        if let Ok(mut systems) = self.systems.write() {
            for stats in systems.values_mut() {
                *stats = SystemMemoryStats::default();
            }
        }
    }
}

/// Memory statistics for a single system.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SystemMemoryStats {
    /// Current allocation count
    pub allocation_count: u64,
    /// Current allocated bytes
    pub allocated_bytes: u64,
    /// Peak allocated bytes
    pub peak_allocated_bytes: u64,
    /// Total allocations
    pub total_allocations: u64,
    /// Total deallocations
    pub total_deallocations: u64,
    /// Total bytes allocated
    pub total_bytes_allocated: u64,
    /// Total bytes deallocated
    pub total_bytes_deallocated: u64,
}

impl SystemMemoryStats {
    /// Checks for potential leaks.
    #[must_use]
    pub fn has_potential_leak(&self) -> bool {
        self.allocation_count > 0 && self.total_allocations > self.total_deallocations
    }
}

/// Memory leak detection helper.
#[derive(Debug)]
pub struct LeakDetector {
    /// Baseline stats taken at start
    baseline: ProfilerMemoryStats,
    /// Optional system tracker
    system_tracker: Option<SystemMemoryTracker>,
}

impl LeakDetector {
    /// Creates a new leak detector with current memory as baseline.
    #[must_use]
    pub fn new() -> Self {
        Self {
            baseline: get_global_stats(),
            system_tracker: None,
        }
    }

    /// Creates with a system tracker.
    #[must_use]
    pub fn with_system_tracker(tracker: SystemMemoryTracker) -> Self {
        Self {
            baseline: get_global_stats(),
            system_tracker: Some(tracker),
        }
    }

    /// Checks for leaks since baseline.
    #[must_use]
    pub fn check(&self) -> LeakCheckResult {
        let current = get_global_stats();

        let leaked_allocations = current
            .allocation_count
            .saturating_sub(self.baseline.allocation_count);
        let leaked_bytes = current
            .allocated_bytes
            .saturating_sub(self.baseline.allocated_bytes);

        let leaked_systems = self.check_systems();

        LeakCheckResult {
            leaked_allocations,
            leaked_bytes,
            baseline: self.baseline.clone(),
            current,
            leaked_systems,
        }
    }

    fn check_systems(&self) -> Vec<(String, SystemMemoryStats)> {
        let mut leaks = Vec::new();

        if let Some(tracker) = &self.system_tracker {
            for id in tracker.get_all_system_ids() {
                if let Some(stats) = tracker.get_system_stats(id) {
                    if stats.has_potential_leak() {
                        let name = tracker
                            .get_system_name(id)
                            .unwrap_or_else(|| format!("system_{}", id.raw()));
                        leaks.push((name, stats));
                    }
                }
            }
        }

        leaks
    }
}

impl Default for LeakDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a leak check.
#[derive(Debug, Clone)]
pub struct LeakCheckResult {
    /// Number of allocations that appear leaked
    pub leaked_allocations: u64,
    /// Bytes that appear leaked
    pub leaked_bytes: u64,
    /// Baseline stats
    pub baseline: ProfilerMemoryStats,
    /// Current stats
    pub current: ProfilerMemoryStats,
    /// Systems with potential leaks
    pub leaked_systems: Vec<(String, SystemMemoryStats)>,
}

impl LeakCheckResult {
    /// Returns true if no leaks detected.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.leaked_allocations == 0 && self.leaked_bytes == 0 && self.leaked_systems.is_empty()
    }

    /// Generates a report.
    #[must_use]
    pub fn to_report(&self) -> String {
        let mut report = String::new();
        let _ = writeln!(report, "=== Leak Check Report ===");

        if self.is_clean() {
            let _ = writeln!(report, "✓ No leaks detected");
        } else {
            let _ = writeln!(
                report,
                "✗ Potential leaks: {} allocations, {}",
                self.leaked_allocations,
                format_bytes(self.leaked_bytes)
            );

            if !self.leaked_systems.is_empty() {
                let _ = writeln!(report, "\nLeaking systems:");
                for (name, stats) in &self.leaked_systems {
                    let _ = writeln!(
                        report,
                        "  {:20} {:>10} ({} allocs, {} unfreed)",
                        name,
                        format_bytes(stats.allocated_bytes),
                        stats.allocation_count,
                        stats.total_allocations - stats.total_deallocations
                    );
                }
            }
        }

        let _ = writeln!(report, "\nBaseline: {}", self.baseline.allocated_str());
        let _ = writeln!(report, "Current:  {}", self.current.allocated_str());

        report
    }
}

/// Scoped allocation tracker for measuring allocations in a block.
#[derive(Debug)]
pub struct AllocationScope {
    /// Name of this scope
    name: String,
    /// Allocations at start
    start_allocations: u64,
    /// Bytes at start
    start_bytes: u64,
}

impl AllocationScope {
    /// Creates a new allocation scope.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let stats = get_global_stats();
        Self {
            name: name.to_string(),
            start_allocations: stats.total_allocations,
            start_bytes: stats.total_bytes_allocated,
        }
    }

    /// Ends the scope and returns the allocation delta.
    #[must_use]
    pub fn finish(self) -> AllocationDelta {
        let stats = get_global_stats();
        AllocationDelta {
            name: self.name,
            allocations: stats
                .total_allocations
                .saturating_sub(self.start_allocations),
            bytes: stats.total_bytes_allocated.saturating_sub(self.start_bytes),
        }
    }
}

/// Delta of allocations in a scope.
#[derive(Debug, Clone)]
pub struct AllocationDelta {
    /// Scope name
    pub name: String,
    /// Number of allocations made
    pub allocations: u64,
    /// Bytes allocated
    pub bytes: u64,
}

impl AllocationDelta {
    /// Returns a formatted string.
    #[must_use]
    pub fn to_string_pretty(&self) -> String {
        format!(
            "{}: {} allocations, {}",
            self.name,
            self.allocations,
            format_bytes(self.bytes)
        )
    }
}

/// Memory budget tracker for enforcing limits.
#[derive(Debug)]
pub struct MemoryBudget {
    /// Budget name
    name: String,
    /// Maximum allowed bytes
    max_bytes: u64,
    /// Current usage
    current_bytes: AtomicU64,
}

impl MemoryBudget {
    /// Creates a new memory budget.
    #[must_use]
    pub fn new(name: &str, max_bytes: u64) -> Self {
        Self {
            name: name.to_string(),
            max_bytes,
            current_bytes: AtomicU64::new(0),
        }
    }

    /// Attempts to reserve bytes from the budget.
    pub fn try_reserve(&self, bytes: u64) -> Result<(), BudgetExceeded> {
        let current = self.current_bytes.fetch_add(bytes, Ordering::Relaxed);
        let new_total = current + bytes;

        if new_total > self.max_bytes {
            // Rollback
            self.current_bytes.fetch_sub(bytes, Ordering::Relaxed);
            Err(BudgetExceeded {
                budget_name: self.name.clone(),
                requested: bytes,
                available: self.max_bytes.saturating_sub(current),
                max: self.max_bytes,
            })
        } else {
            Ok(())
        }
    }

    /// Releases bytes back to the budget.
    pub fn release(&self, bytes: u64) {
        self.current_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Returns current usage.
    #[must_use]
    pub fn current_usage(&self) -> u64 {
        self.current_bytes.load(Ordering::Relaxed)
    }

    /// Returns available bytes.
    #[must_use]
    pub fn available(&self) -> u64 {
        self.max_bytes
            .saturating_sub(self.current_bytes.load(Ordering::Relaxed))
    }

    /// Returns usage percentage (0.0 - 1.0).
    #[must_use]
    pub fn usage_percent(&self) -> f64 {
        let current = self.current_bytes.load(Ordering::Relaxed) as f64;
        let max = self.max_bytes as f64;
        if max > 0.0 {
            current / max
        } else {
            0.0
        }
    }

    /// Resets the budget.
    pub fn reset(&self) {
        self.current_bytes.store(0, Ordering::Relaxed);
    }
}

/// Error when budget is exceeded.
#[derive(Debug, Clone)]
pub struct BudgetExceeded {
    /// Budget name
    pub budget_name: String,
    /// Requested bytes
    pub requested: u64,
    /// Available bytes
    pub available: u64,
    /// Maximum bytes
    pub max: u64,
}

impl std::fmt::Display for BudgetExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory budget '{}' exceeded: requested {}, available {} (max {})",
            self.budget_name,
            format_bytes(self.requested),
            format_bytes(self.available),
            format_bytes(self.max)
        )
    }
}

impl std::error::Error for BudgetExceeded {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(100), "100 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_memory_stats_default() {
        let stats = ProfilerMemoryStats::default();
        assert_eq!(stats.allocation_count, 0);
        assert_eq!(stats.allocated_bytes, 0);
    }

    #[test]
    fn test_memory_stats_report() {
        let stats = ProfilerMemoryStats {
            allocation_count: 100,
            allocated_bytes: 1024 * 1024,
            peak_allocation_count: 150,
            peak_allocated_bytes: 2 * 1024 * 1024,
            total_allocations: 500,
            total_deallocations: 400,
            total_bytes_allocated: 10 * 1024 * 1024,
            total_bytes_deallocated: 9 * 1024 * 1024,
        };

        let report = stats.to_report();
        assert!(report.contains("Memory Stats"));
        assert!(report.contains("100 allocations"));
    }

    #[test]
    fn test_manual_tracking() {
        reset_global_stats();

        record_alloc(1024);
        record_alloc(2048);
        record_dealloc(1024);

        let stats = get_global_stats();
        assert_eq!(stats.allocation_count, 1);
        assert_eq!(stats.allocated_bytes, 2048);
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_deallocations, 1);
    }

    #[test]
    fn test_system_memory_tracker() {
        let tracker = SystemMemoryTracker::new("test");

        let id1 = tracker.register_system("renderer");
        let id2 = tracker.register_system("physics");

        assert!(tracker.get_system_name(id1).is_some());
        assert_eq!(tracker.get_system_name(id1), Some("renderer".to_string()));

        tracker.record_alloc(id1, 1024);
        tracker.record_alloc(id1, 2048);
        tracker.record_dealloc(id1, 1024);

        let stats = tracker.get_system_stats(id1);
        assert!(stats.is_some());

        let stats = stats.expect("stats should exist");
        assert_eq!(stats.allocation_count, 1);
        assert_eq!(stats.allocated_bytes, 2048);
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_deallocations, 1);

        // Check id2 is independent
        let stats2 = tracker.get_system_stats(id2);
        assert!(stats2.is_some());
        let stats2 = stats2.expect("stats should exist");
        assert_eq!(stats2.allocation_count, 0);
    }

    #[test]
    fn test_system_memory_tracker_report() {
        let tracker = SystemMemoryTracker::new("game");

        let id = tracker.register_system("audio");
        tracker.record_alloc(id, 4096);

        let report = tracker.to_report();
        assert!(report.contains("game Memory Report"));
        assert!(report.contains("audio"));
    }

    #[test]
    fn test_system_memory_stats_leak_detection() {
        let stats = SystemMemoryStats {
            allocation_count: 5,
            total_allocations: 10,
            total_deallocations: 5,
            ..Default::default()
        };
        assert!(stats.has_potential_leak());

        let clean_stats = SystemMemoryStats {
            allocation_count: 0,
            total_allocations: 10,
            total_deallocations: 10,
            ..Default::default()
        };
        assert!(!clean_stats.has_potential_leak());
    }

    #[test]
    fn test_leak_detector() {
        reset_global_stats();
        let detector = LeakDetector::new();

        let result = detector.check();
        assert!(result.is_clean());
    }

    #[test]
    fn test_leak_check_result_report() {
        let result = LeakCheckResult {
            leaked_allocations: 0,
            leaked_bytes: 0,
            baseline: ProfilerMemoryStats::default(),
            current: ProfilerMemoryStats::default(),
            leaked_systems: vec![],
        };

        let report = result.to_report();
        assert!(report.contains("No leaks detected"));

        let leaky_result = LeakCheckResult {
            leaked_allocations: 10,
            leaked_bytes: 1024,
            baseline: ProfilerMemoryStats::default(),
            current: ProfilerMemoryStats::default(),
            leaked_systems: vec![],
        };

        let report = leaky_result.to_report();
        assert!(report.contains("Potential leaks"));
    }

    #[test]
    fn test_allocation_scope() {
        reset_global_stats();
        let scope = AllocationScope::new("test_scope");

        record_alloc(1000);
        record_alloc(2000);

        let delta = scope.finish();

        assert_eq!(delta.name, "test_scope");
        assert_eq!(delta.allocations, 2);
        assert_eq!(delta.bytes, 3000);
    }

    #[test]
    fn test_allocation_delta_format() {
        let delta = AllocationDelta {
            name: "test".to_string(),
            allocations: 50,
            bytes: 2048,
        };

        let s = delta.to_string_pretty();
        assert!(s.contains("test"));
        assert!(s.contains("50 allocations"));
    }

    #[test]
    fn test_memory_budget() {
        let budget = MemoryBudget::new("textures", 1024);

        assert_eq!(budget.available(), 1024);
        assert!(budget.try_reserve(500).is_ok());
        assert_eq!(budget.current_usage(), 500);
        assert_eq!(budget.available(), 524);

        assert!(budget.try_reserve(500).is_ok());
        assert!(budget.try_reserve(100).is_err());

        budget.release(200);
        assert_eq!(budget.current_usage(), 800);
    }

    #[test]
    fn test_memory_budget_usage_percent() {
        let budget = MemoryBudget::new("test", 1000);
        assert!((budget.usage_percent() - 0.0).abs() < 0.001);

        let _ = budget.try_reserve(500);
        assert!((budget.usage_percent() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_budget_exceeded_error() {
        let err = BudgetExceeded {
            budget_name: "test".to_string(),
            requested: 1000,
            available: 100,
            max: 500,
        };

        let msg = format!("{err}");
        assert!(msg.contains("test"));
        assert!(msg.contains("exceeded"));
    }

    #[test]
    fn test_system_id() {
        let id = SystemId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_tracker_reset() {
        let tracker = SystemMemoryTracker::new("test");
        let id = tracker.register_system("sys1");

        tracker.record_alloc(id, 1000);
        let stats = tracker.get_system_stats(id).expect("stats");
        assert_eq!(stats.allocated_bytes, 1000);

        tracker.reset();
        let stats = tracker.get_system_stats(id).expect("stats");
        assert_eq!(stats.allocated_bytes, 0);
    }

    #[test]
    fn test_budget_reset() {
        let budget = MemoryBudget::new("test", 1000);
        let _ = budget.try_reserve(500);
        assert_eq!(budget.current_usage(), 500);

        budget.reset();
        assert_eq!(budget.current_usage(), 0);
    }

    #[test]
    fn test_get_all_system_ids() {
        let tracker = SystemMemoryTracker::new("test");
        let id1 = tracker.register_system("sys1");
        let id2 = tracker.register_system("sys2");

        let ids = tracker.get_all_system_ids();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_peak_tracking() {
        reset_global_stats();

        record_alloc(1000);
        record_alloc(2000);
        record_dealloc(1000);
        record_dealloc(2000);

        let stats = get_global_stats();
        assert_eq!(stats.peak_allocated_bytes, 3000);
        assert_eq!(stats.peak_allocation_count, 2);
    }
}
