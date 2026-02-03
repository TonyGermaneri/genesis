//! Event log viewer for debugging and analysis.
//!
//! This module provides a scrollable event log with:
//! - Filtering by event type and source
//! - Pause/play for live capture
//! - Text search
//! - Color-coded severity levels
//! - Event details expansion

use egui::{Color32, Grid, RichText, ScrollArea, TextEdit, Ui};
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

/// Event severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventLevel {
    /// Debug information
    Debug,
    /// Informational message
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
}

impl EventLevel {
    /// Returns all event levels.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Debug, Self::Info, Self::Warning, Self::Error]
    }

    /// Returns the display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        }
    }

    /// Returns the icon for this level.
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Debug => "🔍",
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Error => "❌",
        }
    }

    /// Returns the color for this level.
    #[must_use]
    pub const fn color(&self) -> Color32 {
        match self {
            Self::Debug => Color32::from_rgb(150, 150, 150),
            Self::Info => Color32::from_rgb(100, 180, 255),
            Self::Warning => Color32::from_rgb(255, 200, 50),
            Self::Error => Color32::from_rgb(255, 100, 100),
        }
    }
}

/// Event category for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    /// World/chunk events
    World,
    /// Entity events
    Entity,
    /// Physics/simulation events
    Physics,
    /// Rendering events
    Render,
    /// Input events
    Input,
    /// Network events
    Network,
    /// System/engine events
    System,
    /// Custom/user events
    Custom,
}

impl EventCategory {
    /// Returns all categories.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::World,
            Self::Entity,
            Self::Physics,
            Self::Render,
            Self::Input,
            Self::Network,
            Self::System,
            Self::Custom,
        ]
    }

    /// Returns the display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::World => "World",
            Self::Entity => "Entity",
            Self::Physics => "Physics",
            Self::Render => "Render",
            Self::Input => "Input",
            Self::Network => "Network",
            Self::System => "System",
            Self::Custom => "Custom",
        }
    }

    /// Returns the icon for this category.
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::World => "🌍",
            Self::Entity => "👤",
            Self::Physics => "⚡",
            Self::Render => "🎨",
            Self::Input => "🎮",
            Self::Network => "🌐",
            Self::System => "⚙",
            Self::Custom => "📝",
        }
    }
}

/// A single log event.
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// Unique event ID
    pub id: u64,
    /// Timestamp when the event occurred
    pub timestamp: Instant,
    /// Frame number when event occurred
    pub frame: u64,
    /// Event severity level
    pub level: EventLevel,
    /// Event category
    pub category: EventCategory,
    /// Event source (module/system name)
    pub source: String,
    /// Event message
    pub message: String,
    /// Optional detailed information
    pub details: Option<String>,
}

impl LogEvent {
    /// Creates a new log event.
    #[must_use]
    pub fn new(
        level: EventLevel,
        category: EventCategory,
        source: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            timestamp: Instant::now(),
            frame: 0,
            level,
            category,
            source: source.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Sets the frame number.
    #[must_use]
    pub fn with_frame(mut self, frame: u64) -> Self {
        self.frame = frame;
        self
    }

    /// Sets detailed information.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Checks if this event matches the search query.
    #[must_use]
    pub fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let query_lower = query.to_lowercase();
        self.message.to_lowercase().contains(&query_lower)
            || self.source.to_lowercase().contains(&query_lower)
            || self
                .details
                .as_ref()
                .is_some_and(|d| d.to_lowercase().contains(&query_lower))
    }
}

/// Filter configuration for the event log.
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Enabled event levels
    pub levels: HashSet<EventLevel>,
    /// Enabled categories
    pub categories: HashSet<EventCategory>,
    /// Text search query
    pub search: String,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            levels: EventLevel::all().iter().copied().collect(),
            categories: EventCategory::all().iter().copied().collect(),
            search: String::new(),
        }
    }
}

impl EventFilter {
    /// Checks if an event passes this filter.
    #[must_use]
    pub fn matches(&self, event: &LogEvent) -> bool {
        self.levels.contains(&event.level)
            && self.categories.contains(&event.category)
            && event.matches_search(&self.search)
    }
}

/// Configuration for the event log viewer.
#[derive(Debug, Clone)]
pub struct EventLogConfig {
    /// Maximum number of events to keep in memory
    pub max_events: usize,
    /// Whether to auto-scroll to new events
    pub auto_scroll: bool,
    /// Whether to show timestamps
    pub show_timestamps: bool,
    /// Whether to show frame numbers
    pub show_frames: bool,
    /// Whether to show event source
    pub show_source: bool,
    /// Row height in pixels
    pub row_height: f32,
    /// Maximum visible rows
    pub max_visible_rows: usize,
}

impl Default for EventLogConfig {
    fn default() -> Self {
        Self {
            max_events: 10000,
            auto_scroll: true,
            show_timestamps: true,
            show_frames: true,
            show_source: true,
            row_height: 20.0,
            max_visible_rows: 20,
        }
    }
}

/// Event log viewer with egui rendering.
#[derive(Debug)]
pub struct EventLogViewer {
    /// Configuration
    pub config: EventLogConfig,
    /// Event filter
    pub filter: EventFilter,
    /// All logged events
    events: VecDeque<LogEvent>,
    /// Whether capture is paused
    paused: bool,
    /// Currently selected event ID
    selected_event: Option<u64>,
    /// Reference time for relative timestamps
    start_time: Instant,
    /// Current frame number
    current_frame: u64,
    /// Cached filtered event count
    filtered_count: usize,
}

impl Default for EventLogViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLogViewer {
    /// Creates a new event log viewer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: EventLogConfig::default(),
            filter: EventFilter::default(),
            events: VecDeque::with_capacity(10000),
            paused: false,
            selected_event: None,
            start_time: Instant::now(),
            current_frame: 0,
            filtered_count: 0,
        }
    }

    /// Creates a new event log viewer with custom config.
    #[must_use]
    pub fn with_config(config: EventLogConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Logs a new event.
    pub fn log(&mut self, event: LogEvent) {
        if self.paused {
            return;
        }

        // Update frame number
        let mut event = event;
        if event.frame == 0 {
            event.frame = self.current_frame;
        }

        // Check capacity
        if self.events.len() >= self.config.max_events {
            self.events.pop_front();
        }

        self.events.push_back(event);
        self.update_filtered_count();
    }

    /// Logs a debug event.
    pub fn debug(
        &mut self,
        category: EventCategory,
        source: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.log(LogEvent::new(EventLevel::Debug, category, source, message));
    }

    /// Logs an info event.
    pub fn info(
        &mut self,
        category: EventCategory,
        source: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.log(LogEvent::new(EventLevel::Info, category, source, message));
    }

    /// Logs a warning event.
    pub fn warn(
        &mut self,
        category: EventCategory,
        source: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.log(LogEvent::new(
            EventLevel::Warning,
            category,
            source,
            message,
        ));
    }

    /// Logs an error event.
    pub fn error(
        &mut self,
        category: EventCategory,
        source: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.log(LogEvent::new(EventLevel::Error, category, source, message));
    }

    /// Sets the current frame number.
    pub fn set_frame(&mut self, frame: u64) {
        self.current_frame = frame;
    }

    /// Pauses event capture.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes event capture.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggles pause state.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Returns whether capture is paused.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Clears all events.
    pub fn clear(&mut self) {
        self.events.clear();
        self.selected_event = None;
        self.filtered_count = 0;
    }

    /// Returns total event count.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns filtered event count.
    #[must_use]
    pub const fn filtered_event_count(&self) -> usize {
        self.filtered_count
    }

    /// Updates the filtered count.
    fn update_filtered_count(&mut self) {
        self.filtered_count = self
            .events
            .iter()
            .filter(|e| self.filter.matches(e))
            .count();
    }

    /// Formats a duration as a timestamp string.
    fn format_timestamp(&self, timestamp: Instant) -> String {
        let elapsed = timestamp.duration_since(self.start_time);
        let secs = elapsed.as_secs();
        let millis = elapsed.subsec_millis();
        format!("{secs:02}.{millis:03}")
    }

    /// Renders the event log UI.
    pub fn render_ui(&mut self, ui: &mut Ui) {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("📋 Event Log").strong().size(14.0));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Event count
                ui.label(format!(
                    "{}/{} events",
                    self.filtered_count,
                    self.events.len()
                ));

                // Clear button
                if ui.small_button("🗑 Clear").clicked() {
                    self.clear();
                }

                // Pause/Play button
                let pause_text = if self.paused { "▶ Play" } else { "⏸ Pause" };
                if ui.small_button(pause_text).clicked() {
                    self.toggle_pause();
                }

                if self.paused {
                    ui.label(RichText::new("PAUSED").color(Color32::YELLOW));
                }
            });
        });

        ui.separator();

        // Filter bar
        self.render_filter_bar(ui);

        ui.separator();

        // Event list
        let height = self.config.row_height * self.config.max_visible_rows as f32;
        ScrollArea::vertical()
            .max_height(height)
            .auto_shrink([false, false])
            .stick_to_bottom(self.config.auto_scroll && !self.paused)
            .show(ui, |ui| {
                self.render_event_list(ui);
            });

        // Selected event details
        if let Some(selected_id) = self.selected_event {
            if let Some(event) = self.events.iter().find(|e| e.id == selected_id) {
                ui.separator();
                self.render_event_details(ui, event);
            }
        }
    }

    /// Renders the filter bar.
    fn render_filter_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Search box
            ui.label("🔍");
            let search_response = ui.add(
                TextEdit::singleline(&mut self.filter.search)
                    .hint_text("Search...")
                    .desired_width(150.0),
            );
            if search_response.changed() {
                self.update_filtered_count();
            }

            ui.separator();

            // Level filters
            for &level in EventLevel::all() {
                let mut enabled = self.filter.levels.contains(&level);
                if ui
                    .checkbox(
                        &mut enabled,
                        RichText::new(level.name()).color(level.color()),
                    )
                    .changed()
                {
                    if enabled {
                        self.filter.levels.insert(level);
                    } else {
                        self.filter.levels.remove(&level);
                    }
                    self.update_filtered_count();
                }
            }
        });

        // Category filters (collapsible)
        ui.collapsing("Categories", |ui| {
            ui.horizontal_wrapped(|ui| {
                for &cat in EventCategory::all() {
                    let mut enabled = self.filter.categories.contains(&cat);
                    if ui
                        .checkbox(&mut enabled, format!("{} {}", cat.icon(), cat.name()))
                        .changed()
                    {
                        if enabled {
                            self.filter.categories.insert(cat);
                        } else {
                            self.filter.categories.remove(&cat);
                        }
                        self.update_filtered_count();
                    }
                }
            });
        });
    }

    /// Renders the event list.
    fn render_event_list(&mut self, ui: &mut Ui) {
        let mut new_selection = self.selected_event;

        for event in &self.events {
            if !self.filter.matches(event) {
                continue;
            }

            let is_selected = self.selected_event == Some(event.id);
            let response = self.render_event_row(ui, event, is_selected);

            if response.clicked() {
                new_selection = Some(event.id);
            }
        }

        self.selected_event = new_selection;
    }

    /// Renders a single event row.
    fn render_event_row(&self, ui: &mut Ui, event: &LogEvent, is_selected: bool) -> egui::Response {
        let bg_color = if is_selected {
            Color32::from_rgb(50, 50, 80)
        } else {
            Color32::TRANSPARENT
        };

        let response = egui::Frame::none()
            .fill(bg_color)
            .inner_margin(2.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Level icon
                    ui.label(RichText::new(event.level.icon()).size(12.0));

                    // Timestamp
                    if self.config.show_timestamps {
                        ui.label(
                            RichText::new(self.format_timestamp(event.timestamp))
                                .monospace()
                                .size(11.0)
                                .color(Color32::GRAY),
                        );
                    }

                    // Frame
                    if self.config.show_frames {
                        ui.label(
                            RichText::new(format!("F{}", event.frame))
                                .monospace()
                                .size(11.0)
                                .color(Color32::GRAY),
                        );
                    }

                    // Category
                    ui.label(RichText::new(event.category.icon()).size(11.0));

                    // Source
                    if self.config.show_source {
                        ui.label(
                            RichText::new(format!("[{}]", event.source))
                                .size(11.0)
                                .color(Color32::from_rgb(150, 150, 200)),
                        );
                    }

                    // Message
                    ui.label(RichText::new(&event.message).color(event.level.color()));

                    // Details indicator
                    if event.details.is_some() {
                        ui.label(RichText::new("...").color(Color32::GRAY));
                    }
                });
            })
            .response;

        response.interact(egui::Sense::click())
    }

    /// Renders event details panel.
    fn render_event_details(&self, ui: &mut Ui, event: &LogEvent) {
        ui.label(RichText::new("Event Details").strong());

        Grid::new("event_details_grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .show(ui, |ui| {
                ui.label("ID:");
                ui.label(format!("{}", event.id));
                ui.end_row();

                ui.label("Time:");
                ui.label(self.format_timestamp(event.timestamp));
                ui.end_row();

                ui.label("Frame:");
                ui.label(format!("{}", event.frame));
                ui.end_row();

                ui.label("Level:");
                ui.label(RichText::new(event.level.name()).color(event.level.color()));
                ui.end_row();

                ui.label("Category:");
                ui.label(format!(
                    "{} {}",
                    event.category.icon(),
                    event.category.name()
                ));
                ui.end_row();

                ui.label("Source:");
                ui.label(&event.source);
                ui.end_row();

                ui.label("Message:");
                ui.label(&event.message);
                ui.end_row();
            });

        if let Some(ref details) = event.details {
            ui.add_space(4.0);
            ui.label(RichText::new("Details:").strong());
            ui.label(details);
        }
    }

    /// Returns events matching a predicate.
    #[must_use]
    pub fn find_events<F>(&self, predicate: F) -> Vec<&LogEvent>
    where
        F: Fn(&LogEvent) -> bool,
    {
        self.events.iter().filter(|e| predicate(e)).collect()
    }

    /// Returns events of a specific level.
    #[must_use]
    pub fn events_by_level(&self, level: EventLevel) -> Vec<&LogEvent> {
        self.find_events(|e| e.level == level)
    }

    /// Returns events of a specific category.
    #[must_use]
    pub fn events_by_category(&self, category: EventCategory) -> Vec<&LogEvent> {
        self.find_events(|e| e.category == category)
    }

    /// Returns error count.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.events
            .iter()
            .filter(|e| e.level == EventLevel::Error)
            .count()
    }

    /// Returns warning count.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.events
            .iter()
            .filter(|e| e.level == EventLevel::Warning)
            .count()
    }
}

/// Builder for creating log events.
pub struct LogEventBuilder {
    level: EventLevel,
    category: EventCategory,
    source: String,
    message: String,
    details: Option<String>,
    frame: u64,
}

impl LogEventBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new(level: EventLevel, category: EventCategory) -> Self {
        Self {
            level,
            category,
            source: String::new(),
            message: String::new(),
            details: None,
            frame: 0,
        }
    }

    /// Sets the event source.
    #[must_use]
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Sets the event message.
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Sets the event details.
    #[must_use]
    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Sets the frame number.
    #[must_use]
    pub const fn frame(mut self, frame: u64) -> Self {
        self.frame = frame;
        self
    }

    /// Builds the log event.
    #[must_use]
    pub fn build(self) -> LogEvent {
        let mut event = LogEvent::new(self.level, self.category, self.source, self.message);
        event.frame = self.frame;
        event.details = self.details;
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_level_properties() {
        assert_eq!(EventLevel::Debug.name(), "DEBUG");
        assert_eq!(EventLevel::Error.icon(), "❌");
        assert_eq!(EventLevel::all().len(), 4);
    }

    #[test]
    fn test_event_category_properties() {
        assert_eq!(EventCategory::World.name(), "World");
        assert_eq!(EventCategory::Physics.icon(), "⚡");
        assert_eq!(EventCategory::all().len(), 8);
    }

    #[test]
    fn test_log_event_creation() {
        let event = LogEvent::new(
            EventLevel::Info,
            EventCategory::System,
            "test",
            "Hello world",
        );
        assert_eq!(event.level, EventLevel::Info);
        assert_eq!(event.category, EventCategory::System);
        assert_eq!(event.source, "test");
        assert_eq!(event.message, "Hello world");
    }

    #[test]
    fn test_log_event_search() {
        let event = LogEvent::new(
            EventLevel::Info,
            EventCategory::System,
            "renderer",
            "Frame rendered",
        )
        .with_details("Rendered 1000 triangles");

        assert!(event.matches_search(""));
        assert!(event.matches_search("frame"));
        assert!(event.matches_search("FRAME")); // Case insensitive
        assert!(event.matches_search("renderer"));
        assert!(event.matches_search("triangles"));
        assert!(!event.matches_search("physics"));
    }

    #[test]
    fn test_event_filter() {
        let filter = EventFilter::default();

        let event = LogEvent::new(EventLevel::Info, EventCategory::World, "test", "Test");
        assert!(filter.matches(&event));

        let mut filter_no_info = EventFilter::default();
        filter_no_info.levels.remove(&EventLevel::Info);
        assert!(!filter_no_info.matches(&event));

        let mut filter_no_world = EventFilter::default();
        filter_no_world.categories.remove(&EventCategory::World);
        assert!(!filter_no_world.matches(&event));
    }

    #[test]
    fn test_event_log_viewer() {
        let mut viewer = EventLogViewer::new();
        assert_eq!(viewer.event_count(), 0);

        viewer.info(EventCategory::System, "test", "Event 1");
        viewer.warn(EventCategory::Physics, "test", "Event 2");
        viewer.error(EventCategory::World, "test", "Event 3");

        assert_eq!(viewer.event_count(), 3);
        assert_eq!(viewer.error_count(), 1);
        assert_eq!(viewer.warning_count(), 1);
    }

    #[test]
    fn test_event_log_pause() {
        let mut viewer = EventLogViewer::new();

        viewer.info(EventCategory::System, "test", "Before pause");
        assert_eq!(viewer.event_count(), 1);

        viewer.pause();
        viewer.info(EventCategory::System, "test", "While paused");
        assert_eq!(viewer.event_count(), 1); // Should not add while paused

        viewer.resume();
        viewer.info(EventCategory::System, "test", "After resume");
        assert_eq!(viewer.event_count(), 2);
    }

    #[test]
    fn test_event_log_clear() {
        let mut viewer = EventLogViewer::new();

        viewer.info(EventCategory::System, "test", "Event 1");
        viewer.info(EventCategory::System, "test", "Event 2");
        assert_eq!(viewer.event_count(), 2);

        viewer.clear();
        assert_eq!(viewer.event_count(), 0);
    }

    #[test]
    fn test_event_log_max_capacity() {
        let mut config = EventLogConfig::default();
        config.max_events = 5;

        let mut viewer = EventLogViewer::with_config(config);

        for i in 0..10 {
            viewer.info(EventCategory::System, "test", format!("Event {i}"));
        }

        assert_eq!(viewer.event_count(), 5);
    }

    #[test]
    fn test_event_log_filter_count() {
        let mut viewer = EventLogViewer::new();

        viewer.debug(EventCategory::System, "test", "Debug event");
        viewer.info(EventCategory::System, "test", "Info event");
        viewer.error(EventCategory::System, "test", "Error event");

        assert_eq!(viewer.filtered_event_count(), 3);

        viewer.filter.levels.remove(&EventLevel::Debug);
        viewer.update_filtered_count();
        assert_eq!(viewer.filtered_event_count(), 2);
    }

    #[test]
    fn test_log_event_builder() {
        let event = LogEventBuilder::new(EventLevel::Warning, EventCategory::Physics)
            .source("collision")
            .message("Collision detected")
            .details("Entity A collided with Entity B")
            .frame(100)
            .build();

        assert_eq!(event.level, EventLevel::Warning);
        assert_eq!(event.category, EventCategory::Physics);
        assert_eq!(event.source, "collision");
        assert_eq!(event.message, "Collision detected");
        assert_eq!(event.frame, 100);
        assert!(event.details.is_some());
    }

    #[test]
    fn test_events_by_level() {
        let mut viewer = EventLogViewer::new();

        viewer.info(EventCategory::System, "a", "Info 1");
        viewer.error(EventCategory::System, "b", "Error 1");
        viewer.info(EventCategory::System, "c", "Info 2");

        let infos = viewer.events_by_level(EventLevel::Info);
        assert_eq!(infos.len(), 2);

        let errors = viewer.events_by_level(EventLevel::Error);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_events_by_category() {
        let mut viewer = EventLogViewer::new();

        viewer.info(EventCategory::World, "a", "World event");
        viewer.info(EventCategory::Physics, "b", "Physics event");
        viewer.info(EventCategory::World, "c", "Another world event");

        let world_events = viewer.events_by_category(EventCategory::World);
        assert_eq!(world_events.len(), 2);

        let physics_events = viewer.events_by_category(EventCategory::Physics);
        assert_eq!(physics_events.len(), 1);
    }

    #[test]
    fn test_event_log_config_defaults() {
        let config = EventLogConfig::default();
        assert_eq!(config.max_events, 10000);
        assert!(config.auto_scroll);
        assert!(config.show_timestamps);
        assert!(config.show_frames);
        assert!(config.show_source);
    }
}
