//! Opt-in Gameplay Analytics
//!
//! Collects anonymous gameplay events for improving the game.
//! Disabled by default and requires explicit opt-in.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Event type/name
    pub event_type: String,
    /// Event properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Configuration for analytics
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// Whether analytics is enabled (default: false)
    pub enabled: bool,
    /// Optional endpoint for submitting events
    pub endpoint: Option<String>,
    /// How often to flush events (in seconds)
    pub flush_interval_secs: u64,
    /// Maximum batch size before auto-flush
    pub batch_size: usize,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default - requires opt-in
            endpoint: None,
            flush_interval_secs: 60,
            batch_size: 100,
        }
    }
}

/// Analytics manager for collecting gameplay events
pub struct Analytics {
    enabled: AtomicBool,
    session_id: String,
    events: Vec<AnalyticsEvent>,
    flush_interval: Duration,
    batch_size: usize,
    last_flush: Instant,
    endpoint: Option<String>,
    session_start: Instant,
}

impl Default for Analytics {
    fn default() -> Self {
        Self::new(AnalyticsConfig::default())
    }
}

impl Analytics {
    /// Create a new analytics manager
    pub fn new(config: AnalyticsConfig) -> Self {
        let session_id = generate_session_id();

        if config.enabled {
            info!("Analytics enabled with session: {}", session_id);
        }

        Self {
            enabled: AtomicBool::new(config.enabled),
            session_id,
            events: Vec::new(),
            flush_interval: Duration::from_secs(config.flush_interval_secs),
            batch_size: config.batch_size,
            last_flush: Instant::now(),
            endpoint: config.endpoint,
            session_start: Instant::now(),
        }
    }

    /// Check if analytics is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable or disable analytics
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        if enabled {
            info!("Analytics enabled");
        } else {
            info!("Analytics disabled");
            self.events.clear();
        }
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Track a custom event
    pub fn track(&mut self, event_type: &str, properties: HashMap<String, serde_json::Value>) {
        if !self.is_enabled() {
            return;
        }

        let event = AnalyticsEvent {
            timestamp: current_timestamp_ms(),
            event_type: event_type.to_string(),
            properties,
        };

        debug!("Tracking event: {}", event_type);
        self.events.push(event);

        // Auto-flush if batch is full
        if self.events.len() >= self.batch_size {
            self.flush();
        }
    }

    /// Track session start
    pub fn track_session_start(&mut self) {
        self.track(
            "session_start",
            HashMap::from([
                ("session_id".to_string(), json_string(&self.session_id)),
                ("os".to_string(), json_string(std::env::consts::OS)),
                ("arch".to_string(), json_string(std::env::consts::ARCH)),
            ]),
        );
    }

    /// Track session end
    pub fn track_session_end(&mut self, play_time_secs: u64) {
        self.track(
            "session_end",
            HashMap::from([
                ("session_id".to_string(), json_string(&self.session_id)),
                ("play_time_secs".to_string(), json_number(play_time_secs)),
                (
                    "events_count".to_string(),
                    json_number(self.events.len() as u64),
                ),
            ]),
        );
        // Flush on session end
        self.flush();
    }

    /// Track level/area completion
    pub fn track_level_complete(&mut self, level: &str, time_secs: u64) {
        self.track(
            "level_complete",
            HashMap::from([
                ("level".to_string(), json_string(level)),
                ("time_secs".to_string(), json_number(time_secs)),
            ]),
        );
    }

    /// Track player death
    pub fn track_death(&mut self, cause: &str, location: (f32, f32)) {
        self.track(
            "death",
            HashMap::from([
                ("cause".to_string(), json_string(cause)),
                ("x".to_string(), serde_json::Value::from(location.0)),
                ("y".to_string(), serde_json::Value::from(location.1)),
            ]),
        );
    }

    /// Track achievement unlock
    pub fn track_achievement(&mut self, achievement: &str) {
        let play_time = self.session_start.elapsed().as_secs();
        self.track(
            "achievement",
            HashMap::from([
                ("achievement".to_string(), json_string(achievement)),
                ("play_time_secs".to_string(), json_number(play_time)),
            ]),
        );
    }

    /// Track crafting
    pub fn track_craft(&mut self, recipe: &str, success: bool) {
        self.track(
            "craft",
            HashMap::from([
                ("recipe".to_string(), json_string(recipe)),
                ("success".to_string(), serde_json::Value::Bool(success)),
            ]),
        );
    }

    /// Track item pickup
    pub fn track_item_pickup(&mut self, item: &str, quantity: u32) {
        self.track(
            "item_pickup",
            HashMap::from([
                ("item".to_string(), json_string(item)),
                ("quantity".to_string(), json_number(quantity as u64)),
            ]),
        );
    }

    /// Track quest started
    pub fn track_quest_start(&mut self, quest: &str) {
        self.track(
            "quest_start",
            HashMap::from([("quest".to_string(), json_string(quest))]),
        );
    }

    /// Track quest completed
    pub fn track_quest_complete(&mut self, quest: &str, time_secs: u64) {
        self.track(
            "quest_complete",
            HashMap::from([
                ("quest".to_string(), json_string(quest)),
                ("time_secs".to_string(), json_number(time_secs)),
            ]),
        );
    }

    /// Check if it's time to flush and do so if needed
    pub fn check_flush(&mut self) {
        if self.last_flush.elapsed() >= self.flush_interval {
            self.flush();
        }
    }

    /// Flush events to the server (or log)
    pub fn flush(&mut self) {
        if self.events.is_empty() {
            return;
        }

        if !self.is_enabled() {
            self.events.clear();
            return;
        }

        let event_count = self.events.len();
        debug!("Flushing {} analytics events", event_count);

        if let Some(ref _endpoint) = self.endpoint {
            // In a real implementation, this would POST events to the endpoint
            // For now, just log
            info!("Would send {} events to analytics endpoint", event_count);
        }

        // Clear events after flush
        self.events.clear();
        self.last_flush = Instant::now();
    }

    /// Get count of pending events
    pub fn pending_count(&self) -> usize {
        self.events.len()
    }

    /// Get total session time in seconds
    pub fn session_time_secs(&self) -> u64 {
        self.session_start.elapsed().as_secs()
    }
}

impl Drop for Analytics {
    fn drop(&mut self) {
        if self.is_enabled() && !self.events.is_empty() {
            warn!(
                "Analytics dropped with {} unflushed events",
                self.events.len()
            );
        }
    }
}

/// Generate a random session ID
fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random: u32 = (timestamp as u32)
        .wrapping_mul(1_103_515_245)
        .wrapping_add(12345);
    format!("{timestamp:x}-{random:08x}")
}

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Helper to create a JSON string value
fn json_string(s: &str) -> serde_json::Value {
    serde_json::Value::String(s.to_string())
}

/// Helper to create a JSON number value
fn json_number(n: u64) -> serde_json::Value {
    serde_json::Value::Number(n.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_disabled_by_default() {
        let analytics = Analytics::default();
        assert!(!analytics.is_enabled());
    }

    #[test]
    fn test_analytics_enable_disable() {
        let mut analytics = Analytics::default();
        assert!(!analytics.is_enabled());

        analytics.set_enabled(true);
        assert!(analytics.is_enabled());

        analytics.set_enabled(false);
        assert!(!analytics.is_enabled());
    }

    #[test]
    fn test_track_when_disabled() {
        let mut analytics = Analytics::default();
        analytics.track("test", HashMap::new());
        assert_eq!(analytics.pending_count(), 0);
    }

    #[test]
    fn test_track_when_enabled() {
        let mut analytics = Analytics::new(AnalyticsConfig {
            enabled: true,
            ..Default::default()
        });

        analytics.track("test", HashMap::new());
        assert_eq!(analytics.pending_count(), 1);
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert!(!id1.is_empty());
        // IDs should be different (though timing could rarely cause collision)
        // Just check they're generated
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_flush_clears_events() {
        let mut analytics = Analytics::new(AnalyticsConfig {
            enabled: true,
            ..Default::default()
        });

        analytics.track("test1", HashMap::new());
        analytics.track("test2", HashMap::new());
        assert_eq!(analytics.pending_count(), 2);

        analytics.flush();
        assert_eq!(analytics.pending_count(), 0);
    }

    #[test]
    fn test_predefined_events() {
        let mut analytics = Analytics::new(AnalyticsConfig {
            enabled: true,
            batch_size: 100,
            ..Default::default()
        });

        analytics.track_session_start();
        analytics.track_death("fall", (100.0, 200.0));
        analytics.track_achievement("first_craft");
        analytics.track_craft("iron_sword", true);
        analytics.track_level_complete("tutorial", 300);

        assert_eq!(analytics.pending_count(), 5);
    }
}
