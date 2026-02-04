//! Combat profiling and performance metrics.
//!
//! This module provides:
//! - Hitbox check performance measurement
//! - Projectile update timing
//! - Combat event processing metrics
//! - Memory usage tracking for combat data

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tracing::debug;

/// Performance metrics for hitbox checks.
#[derive(Debug, Clone, Default)]
pub struct HitboxMetrics {
    /// Total hitbox checks performed.
    pub total_checks: u64,
    /// Total time spent on hitbox checks.
    pub total_time: Duration,
    /// Maximum check time observed.
    pub max_time: Duration,
    /// Minimum check time observed.
    pub min_time: Option<Duration>,
    /// Checks that found a hit.
    pub hits_found: u64,
    /// Checks by entity count in range.
    pub by_entity_count: HashMap<usize, CheckBucket>,
}

/// Metrics bucket for grouped checks.
#[derive(Debug, Clone, Default)]
pub struct CheckBucket {
    /// Number of checks in this bucket.
    pub count: u64,
    /// Total time for this bucket.
    pub total_time: Duration,
}

impl HitboxMetrics {
    /// Creates new empty metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a hitbox check.
    pub fn record_check(&mut self, entity_count: usize, duration: Duration, hit_found: bool) {
        self.total_checks += 1;
        self.total_time += duration;

        if duration > self.max_time {
            self.max_time = duration;
        }

        match self.min_time {
            Some(min) if duration < min => self.min_time = Some(duration),
            None => self.min_time = Some(duration),
            _ => {},
        }

        if hit_found {
            self.hits_found += 1;
        }

        let bucket = self.by_entity_count.entry(entity_count).or_default();
        bucket.count += 1;
        bucket.total_time += duration;
    }

    /// Returns average check time.
    #[must_use]
    pub fn average_time(&self) -> Duration {
        if self.total_checks == 0 {
            return Duration::ZERO;
        }
        self.total_time / self.total_checks as u32
    }

    /// Returns hit rate (hits / checks).
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        self.hits_found as f64 / self.total_checks as f64
    }

    /// Returns checks per second (throughput).
    #[must_use]
    pub fn checks_per_second(&self) -> f64 {
        if self.total_time.is_zero() {
            return 0.0;
        }
        self.total_checks as f64 / self.total_time.as_secs_f64()
    }

    /// Resets all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Performance metrics for projectile updates.
#[derive(Debug, Clone, Default)]
pub struct ProjectileMetrics {
    /// Total update frames.
    pub total_updates: u64,
    /// Total time spent on projectile updates.
    pub total_time: Duration,
    /// Maximum update time.
    pub max_time: Duration,
    /// Minimum update time.
    pub min_time: Option<Duration>,
    /// Total projectiles processed.
    pub total_projectiles: u64,
    /// Projectiles that hit a target.
    pub hits: u64,
    /// Projectiles that expired/despawned.
    pub expired: u64,
    /// Updates by projectile count.
    pub by_projectile_count: HashMap<usize, CheckBucket>,
}

impl ProjectileMetrics {
    /// Creates new empty metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a projectile update frame.
    pub fn record_update(
        &mut self,
        projectile_count: usize,
        duration: Duration,
        hits: u64,
        expired: u64,
    ) {
        self.total_updates += 1;
        self.total_time += duration;
        self.total_projectiles += projectile_count as u64;
        self.hits += hits;
        self.expired += expired;

        if duration > self.max_time {
            self.max_time = duration;
        }

        match self.min_time {
            Some(min) if duration < min => self.min_time = Some(duration),
            None if projectile_count > 0 => self.min_time = Some(duration),
            _ => {},
        }

        let bucket = self
            .by_projectile_count
            .entry(projectile_count)
            .or_default();
        bucket.count += 1;
        bucket.total_time += duration;
    }

    /// Returns average update time.
    #[must_use]
    pub fn average_time(&self) -> Duration {
        if self.total_updates == 0 {
            return Duration::ZERO;
        }
        self.total_time / self.total_updates as u32
    }

    /// Returns average projectiles per frame.
    #[must_use]
    pub fn average_projectiles_per_frame(&self) -> f64 {
        if self.total_updates == 0 {
            return 0.0;
        }
        self.total_projectiles as f64 / self.total_updates as f64
    }

    /// Returns time per projectile.
    #[must_use]
    pub fn time_per_projectile(&self) -> Duration {
        if self.total_projectiles == 0 {
            return Duration::ZERO;
        }
        self.total_time / self.total_projectiles as u32
    }

    /// Returns hit rate (hits / total projectiles).
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.total_projectiles == 0 {
            return 0.0;
        }
        self.hits as f64 / self.total_projectiles as f64
    }

    /// Resets all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Performance metrics for combat event processing.
#[derive(Debug, Clone, Default)]
pub struct CombatEventMetrics {
    /// Total event batches processed.
    pub total_batches: u64,
    /// Total events processed.
    pub total_events: u64,
    /// Total time spent processing events.
    pub total_time: Duration,
    /// Maximum batch time.
    pub max_batch_time: Duration,
    /// Events by type.
    pub by_event_type: HashMap<String, u64>,
    /// Processing time by event type.
    pub time_by_event_type: HashMap<String, Duration>,
}

impl CombatEventMetrics {
    /// Creates new empty metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an event batch.
    pub fn record_batch(&mut self, event_count: usize, duration: Duration) {
        self.total_batches += 1;
        self.total_events += event_count as u64;
        self.total_time += duration;

        if duration > self.max_batch_time {
            self.max_batch_time = duration;
        }
    }

    /// Records an individual event.
    pub fn record_event(&mut self, event_type: &str, duration: Duration) {
        *self
            .by_event_type
            .entry(event_type.to_string())
            .or_insert(0) += 1;
        *self
            .time_by_event_type
            .entry(event_type.to_string())
            .or_insert(Duration::ZERO) += duration;
    }

    /// Returns average events per batch.
    #[must_use]
    pub fn average_batch_size(&self) -> f64 {
        if self.total_batches == 0 {
            return 0.0;
        }
        self.total_events as f64 / self.total_batches as f64
    }

    /// Returns average time per event.
    #[must_use]
    pub fn average_event_time(&self) -> Duration {
        if self.total_events == 0 {
            return Duration::ZERO;
        }
        self.total_time / self.total_events as u32
    }

    /// Returns events per second.
    #[must_use]
    pub fn events_per_second(&self) -> f64 {
        if self.total_time.is_zero() {
            return 0.0;
        }
        self.total_events as f64 / self.total_time.as_secs_f64()
    }

    /// Returns count for a specific event type.
    #[must_use]
    pub fn event_count(&self, event_type: &str) -> u64 {
        self.by_event_type.get(event_type).copied().unwrap_or(0)
    }

    /// Resets all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Performance metrics for damage calculations.
#[derive(Debug, Clone, Default)]
pub struct DamageCalculationMetrics {
    /// Total calculations performed.
    pub total_calculations: u64,
    /// Total time spent.
    pub total_time: Duration,
    /// Calculations by damage type.
    pub by_damage_type: HashMap<String, u64>,
    /// Average damage dealt.
    pub total_damage: f64,
    /// Critical hit count.
    pub criticals: u64,
    /// Block count.
    pub blocks: u64,
}

impl DamageCalculationMetrics {
    /// Creates new empty metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a damage calculation.
    pub fn record_calculation(
        &mut self,
        damage_type: &str,
        damage: f32,
        critical: bool,
        blocked: bool,
        duration: Duration,
    ) {
        self.total_calculations += 1;
        self.total_time += duration;
        self.total_damage += f64::from(damage);

        *self
            .by_damage_type
            .entry(damage_type.to_string())
            .or_insert(0) += 1;

        if critical {
            self.criticals += 1;
        }
        if blocked {
            self.blocks += 1;
        }
    }

    /// Returns average damage per hit.
    #[must_use]
    pub fn average_damage(&self) -> f64 {
        if self.total_calculations == 0 {
            return 0.0;
        }
        self.total_damage / self.total_calculations as f64
    }

    /// Returns critical rate.
    #[must_use]
    pub fn critical_rate(&self) -> f64 {
        if self.total_calculations == 0 {
            return 0.0;
        }
        self.criticals as f64 / self.total_calculations as f64
    }

    /// Returns block rate.
    #[must_use]
    pub fn block_rate(&self) -> f64 {
        if self.total_calculations == 0 {
            return 0.0;
        }
        self.blocks as f64 / self.total_calculations as f64
    }

    /// Resets all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Memory usage tracking for combat data.
#[derive(Debug, Clone, Default)]
pub struct CombatMemoryUsage {
    /// Number of active entities with combat data.
    pub entity_count: usize,
    /// Estimated memory for entity combat states (bytes).
    pub entity_states_bytes: usize,
    /// Active projectile count.
    pub projectile_count: usize,
    /// Memory for projectiles (bytes).
    pub projectiles_bytes: usize,
    /// Active status effects count.
    pub status_effect_count: usize,
    /// Memory for status effects (bytes).
    pub status_effects_bytes: usize,
    /// Weapon registry size.
    pub weapon_count: usize,
    /// Memory for weapon definitions (bytes).
    pub weapons_bytes: usize,
    /// Total estimated memory (bytes).
    pub total_bytes: usize,
}

impl CombatMemoryUsage {
    /// Creates new empty usage tracking.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates memory estimates.
    pub fn update(
        &mut self,
        entity_count: usize,
        projectile_count: usize,
        status_effect_count: usize,
        weapon_count: usize,
    ) {
        self.entity_count = entity_count;
        self.projectile_count = projectile_count;
        self.status_effect_count = status_effect_count;
        self.weapon_count = weapon_count;

        // Rough estimates:
        // - EntityCombatState: ~200 bytes base + status effects
        self.entity_states_bytes = entity_count * 200;

        // - Projectile: ~100 bytes (position, velocity, owner, type, etc.)
        self.projectiles_bytes = projectile_count * 100;

        // - StatusEffect: ~80 bytes (type, duration, stacks, source)
        self.status_effects_bytes = status_effect_count * 80;

        // - WeaponDefinition: ~400 bytes (stats, effects, abilities)
        self.weapons_bytes = weapon_count * 400;

        self.total_bytes = self.entity_states_bytes
            + self.projectiles_bytes
            + self.status_effects_bytes
            + self.weapons_bytes;
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
}

/// Frame timing for combat systems.
#[derive(Debug, Clone, Default)]
pub struct CombatFrameTimings {
    /// Time spent on hitbox checks.
    pub hitbox_time: Duration,
    /// Time spent on projectile updates.
    pub projectile_time: Duration,
    /// Time spent on event processing.
    pub event_time: Duration,
    /// Time spent on status effect updates.
    pub status_effect_time: Duration,
    /// Time spent on damage calculations.
    pub damage_calc_time: Duration,
    /// Time spent on AI/behavior updates.
    pub ai_time: Duration,
    /// Total combat frame time.
    pub total_time: Duration,
}

impl CombatFrameTimings {
    /// Creates new zero timings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates total time from components.
    pub fn calculate_total(&mut self) {
        self.total_time = self.hitbox_time
            + self.projectile_time
            + self.event_time
            + self.status_effect_time
            + self.damage_calc_time
            + self.ai_time;
    }

    /// Returns the percentage of frame time used by a component.
    #[must_use]
    pub fn hitbox_percentage(&self) -> f64 {
        if self.total_time.is_zero() {
            return 0.0;
        }
        self.hitbox_time.as_secs_f64() / self.total_time.as_secs_f64() * 100.0
    }

    /// Resets all timings.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Complete combat profiler.
pub struct CombatProfiler {
    /// Hitbox check metrics.
    pub hitbox_metrics: HitboxMetrics,
    /// Projectile update metrics.
    pub projectile_metrics: ProjectileMetrics,
    /// Event processing metrics.
    pub event_metrics: CombatEventMetrics,
    /// Damage calculation metrics.
    pub damage_metrics: DamageCalculationMetrics,
    /// Memory usage tracking.
    pub memory_usage: CombatMemoryUsage,
    /// Current frame timings.
    pub frame_timings: CombatFrameTimings,
    /// Historical frame timings (ring buffer).
    frame_history: Vec<CombatFrameTimings>,
    /// Maximum frame history size.
    max_history: usize,
    /// Active timers.
    active_timers: HashMap<String, Instant>,
    /// Profiling enabled flag.
    enabled: bool,
}

impl Default for CombatProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatProfiler {
    /// Creates a new combat profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            hitbox_metrics: HitboxMetrics::new(),
            projectile_metrics: ProjectileMetrics::new(),
            event_metrics: CombatEventMetrics::new(),
            damage_metrics: DamageCalculationMetrics::new(),
            memory_usage: CombatMemoryUsage::new(),
            frame_timings: CombatFrameTimings::new(),
            frame_history: Vec::new(),
            max_history: 120, // 2 seconds at 60fps
            active_timers: HashMap::new(),
            enabled: cfg!(debug_assertions),
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

    /// Starts a named timer.
    pub fn start_timer(&mut self, name: &str) {
        if self.enabled {
            self.active_timers.insert(name.to_string(), Instant::now());
        }
    }

    /// Ends a named timer and returns the duration.
    pub fn end_timer(&mut self, name: &str) -> Duration {
        if self.enabled {
            if let Some(start) = self.active_timers.remove(name) {
                return start.elapsed();
            }
        }
        Duration::ZERO
    }

    /// Times a function.
    pub fn time<T, F: FnOnce() -> T>(&mut self, name: &str, f: F) -> T {
        self.start_timer(name);
        let result = f();
        let duration = self.end_timer(name);
        debug!("{} took {:?}", name, duration);
        result
    }

    /// Starts timing hitbox checks.
    pub fn start_hitbox_check(&mut self) {
        self.start_timer("hitbox");
    }

    /// Ends timing hitbox checks.
    pub fn end_hitbox_check(&mut self, entity_count: usize, hit_found: bool) {
        if self.enabled {
            let duration = self.end_timer("hitbox");
            self.hitbox_metrics
                .record_check(entity_count, duration, hit_found);
            self.frame_timings.hitbox_time += duration;
        }
    }

    /// Starts timing projectile updates.
    pub fn start_projectile_update(&mut self) {
        self.start_timer("projectile");
    }

    /// Ends timing projectile updates.
    pub fn end_projectile_update(&mut self, projectile_count: usize, hits: u64, expired: u64) {
        if self.enabled {
            let duration = self.end_timer("projectile");
            self.projectile_metrics
                .record_update(projectile_count, duration, hits, expired);
            self.frame_timings.projectile_time += duration;
        }
    }

    /// Starts timing event processing.
    pub fn start_event_processing(&mut self) {
        self.start_timer("events");
    }

    /// Ends timing event processing.
    pub fn end_event_processing(&mut self, event_count: usize) {
        if self.enabled {
            let duration = self.end_timer("events");
            self.event_metrics.record_batch(event_count, duration);
            self.frame_timings.event_time += duration;
        }
    }

    /// Records a combat event of a specific type.
    pub fn record_event(&mut self, event_type: &str) {
        if self.enabled {
            self.event_metrics.record_event(event_type, Duration::ZERO);
        }
    }

    /// Starts timing damage calculation.
    pub fn start_damage_calc(&mut self) {
        self.start_timer("damage");
    }

    /// Ends timing damage calculation.
    pub fn end_damage_calc(
        &mut self,
        damage_type: &str,
        damage: f32,
        critical: bool,
        blocked: bool,
    ) {
        if self.enabled {
            let duration = self.end_timer("damage");
            self.damage_metrics.record_calculation(
                damage_type,
                damage,
                critical,
                blocked,
                duration,
            );
            self.frame_timings.damage_calc_time += duration;
        }
    }

    /// Updates memory usage.
    pub fn update_memory(
        &mut self,
        entity_count: usize,
        projectile_count: usize,
        status_effect_count: usize,
        weapon_count: usize,
    ) {
        if self.enabled {
            self.memory_usage.update(
                entity_count,
                projectile_count,
                status_effect_count,
                weapon_count,
            );
        }
    }

    /// Ends the current frame and saves to history.
    pub fn end_frame(&mut self) {
        if self.enabled {
            self.frame_timings.calculate_total();

            // Save to history
            if self.frame_history.len() >= self.max_history {
                self.frame_history.remove(0);
            }
            self.frame_history.push(self.frame_timings.clone());

            // Reset for next frame
            self.frame_timings.reset();
        }
    }

    /// Returns average frame time over history.
    #[must_use]
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_history.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.frame_history.iter().map(|f| f.total_time).sum();
        total / self.frame_history.len() as u32
    }

    /// Returns maximum frame time in history.
    #[must_use]
    pub fn max_frame_time(&self) -> Duration {
        self.frame_history
            .iter()
            .map(|f| f.total_time)
            .max()
            .unwrap_or(Duration::ZERO)
    }

    /// Returns a summary of profiling data.
    #[must_use]
    pub fn summary(&self) -> CombatProfilingSummary {
        CombatProfilingSummary {
            total_hitbox_checks: self.hitbox_metrics.total_checks,
            avg_hitbox_time: self.hitbox_metrics.average_time(),
            hitbox_hit_rate: self.hitbox_metrics.hit_rate(),
            total_projectile_updates: self.projectile_metrics.total_updates,
            avg_projectiles_per_frame: self.projectile_metrics.average_projectiles_per_frame(),
            projectile_hit_rate: self.projectile_metrics.hit_rate(),
            total_events: self.event_metrics.total_events,
            avg_events_per_batch: self.event_metrics.average_batch_size(),
            total_damage_calcs: self.damage_metrics.total_calculations,
            avg_damage: self.damage_metrics.average_damage(),
            critical_rate: self.damage_metrics.critical_rate(),
            block_rate: self.damage_metrics.block_rate(),
            avg_frame_time: self.average_frame_time(),
            max_frame_time: self.max_frame_time(),
            memory_usage_kb: self.memory_usage.total_kb(),
        }
    }

    /// Resets all profiling data.
    pub fn reset(&mut self) {
        self.hitbox_metrics.reset();
        self.projectile_metrics.reset();
        self.event_metrics.reset();
        self.damage_metrics.reset();
        self.frame_timings.reset();
        self.frame_history.clear();
    }
}

/// Summary of combat profiling data.
#[derive(Debug, Clone)]
pub struct CombatProfilingSummary {
    /// Total hitbox checks.
    pub total_hitbox_checks: u64,
    /// Average hitbox check time.
    pub avg_hitbox_time: Duration,
    /// Hitbox hit rate.
    pub hitbox_hit_rate: f64,
    /// Total projectile update frames.
    pub total_projectile_updates: u64,
    /// Average projectiles per frame.
    pub avg_projectiles_per_frame: f64,
    /// Projectile hit rate.
    pub projectile_hit_rate: f64,
    /// Total events processed.
    pub total_events: u64,
    /// Average events per batch.
    pub avg_events_per_batch: f64,
    /// Total damage calculations.
    pub total_damage_calcs: u64,
    /// Average damage per hit.
    pub avg_damage: f64,
    /// Critical hit rate.
    pub critical_rate: f64,
    /// Block rate.
    pub block_rate: f64,
    /// Average frame time.
    pub avg_frame_time: Duration,
    /// Maximum frame time.
    pub max_frame_time: Duration,
    /// Memory usage in KB.
    pub memory_usage_kb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hitbox_metrics() {
        let mut metrics = HitboxMetrics::new();

        metrics.record_check(10, Duration::from_micros(100), true);
        metrics.record_check(10, Duration::from_micros(200), false);
        metrics.record_check(10, Duration::from_micros(150), true);

        assert_eq!(metrics.total_checks, 3);
        assert_eq!(metrics.hits_found, 2);
        assert!((metrics.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_projectile_metrics() {
        let mut metrics = ProjectileMetrics::new();

        metrics.record_update(5, Duration::from_micros(50), 1, 0);
        metrics.record_update(8, Duration::from_micros(80), 2, 1);

        assert_eq!(metrics.total_updates, 2);
        assert_eq!(metrics.total_projectiles, 13);
        assert_eq!(metrics.hits, 3);
        assert_eq!(metrics.expired, 1);
    }

    #[test]
    fn test_event_metrics() {
        let mut metrics = CombatEventMetrics::new();

        metrics.record_batch(10, Duration::from_micros(100));
        metrics.record_batch(20, Duration::from_micros(200));

        assert_eq!(metrics.total_batches, 2);
        assert_eq!(metrics.total_events, 30);
        assert!((metrics.average_batch_size() - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_damage_metrics() {
        let mut metrics = DamageCalculationMetrics::new();

        metrics.record_calculation("physical", 50.0, false, false, Duration::from_micros(10));
        metrics.record_calculation("physical", 100.0, true, false, Duration::from_micros(10));
        metrics.record_calculation("fire", 30.0, false, true, Duration::from_micros(10));

        assert_eq!(metrics.total_calculations, 3);
        assert!((metrics.average_damage() - 60.0).abs() < 0.01);
        assert!((metrics.critical_rate() - 0.333).abs() < 0.01);
        assert!((metrics.block_rate() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_memory_usage() {
        let mut usage = CombatMemoryUsage::new();

        usage.update(100, 20, 50, 10);

        assert_eq!(usage.entity_count, 100);
        assert_eq!(usage.projectile_count, 20);
        assert!(usage.total_bytes > 0);
        assert!(usage.total_kb() > 0.0);
    }

    #[test]
    fn test_frame_timings() {
        let mut timings = CombatFrameTimings::new();

        timings.hitbox_time = Duration::from_micros(100);
        timings.projectile_time = Duration::from_micros(50);
        timings.event_time = Duration::from_micros(30);
        timings.calculate_total();

        assert_eq!(timings.total_time, Duration::from_micros(180));
    }

    #[test]
    fn test_combat_profiler_timers() {
        let mut profiler = CombatProfiler::with_enabled(true);

        profiler.start_timer("test");
        std::thread::sleep(Duration::from_millis(1));
        let duration = profiler.end_timer("test");

        assert!(duration >= Duration::from_millis(1));
    }

    #[test]
    fn test_combat_profiler_hitbox() {
        let mut profiler = CombatProfiler::with_enabled(true);

        profiler.start_hitbox_check();
        std::thread::sleep(Duration::from_micros(100));
        profiler.end_hitbox_check(5, true);

        assert_eq!(profiler.hitbox_metrics.total_checks, 1);
        assert_eq!(profiler.hitbox_metrics.hits_found, 1);
    }

    #[test]
    fn test_combat_profiler_frame_history() {
        let mut profiler = CombatProfiler::with_enabled(true);

        for _ in 0..5 {
            // Set component times which will be summed by calculate_total() in end_frame()
            profiler.frame_timings.hitbox_time = Duration::from_millis(5);
            profiler.frame_timings.event_time = Duration::from_millis(5);
            profiler.end_frame();
        }

        assert_eq!(profiler.frame_history.len(), 5);
        // Average should be 10ms (5ms hitbox + 5ms event)
        assert_eq!(profiler.average_frame_time(), Duration::from_millis(10));
    }

    #[test]
    fn test_combat_profiler_summary() {
        let mut profiler = CombatProfiler::with_enabled(true);

        profiler.hitbox_metrics.total_checks = 100;
        profiler.hitbox_metrics.hits_found = 50;
        profiler.damage_metrics.total_calculations = 50;
        profiler.damage_metrics.total_damage = 5000.0;

        let summary = profiler.summary();

        assert_eq!(summary.total_hitbox_checks, 100);
        assert!((summary.hitbox_hit_rate - 0.5).abs() < 0.01);
        assert!((summary.avg_damage - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_combat_profiler_disabled() {
        let mut profiler = CombatProfiler::with_enabled(false);

        profiler.start_hitbox_check();
        profiler.end_hitbox_check(5, true);

        // Should not record when disabled
        assert_eq!(profiler.hitbox_metrics.total_checks, 0);
    }
}
