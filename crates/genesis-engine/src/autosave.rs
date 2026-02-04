//! Auto-save system for automatic game state persistence.
//!
//! This module provides:
//! - AutoSaveConfig: configurable auto-save settings
//! - AutoSaveManager: handles automatic saves at intervals and on events
//! - Support for pausing during combat/cutscenes
//! - Rotating auto-save slots

use std::collections::VecDeque;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::save_manager::{SaveFileData, SaveManager, SaveResult, AUTOSAVE_PREFIX};

/// Default auto-save interval in seconds.
pub const DEFAULT_AUTOSAVE_INTERVAL: f64 = 300.0; // 5 minutes

/// Default number of rotating auto-save slots.
pub const DEFAULT_AUTOSAVE_SLOTS: usize = 3;

/// Maximum auto-save slots allowed.
pub const MAX_AUTOSAVE_SLOTS: usize = 10;

/// Minimum auto-save interval in seconds.
pub const MIN_AUTOSAVE_INTERVAL: f64 = 30.0;

/// Events that can trigger an auto-save.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutoSaveTrigger {
    /// Timer-based auto-save.
    Interval,
    /// Save when entering a new area.
    AreaTransition,
    /// Save after completing a quest.
    QuestComplete,
    /// Save after a boss fight.
    BossFightComplete,
    /// Save after acquiring an important item.
    ImportantItemAcquired,
    /// Save when resting at a save point.
    SavePoint,
    /// Save before a major decision.
    MajorDecision,
    /// Manual trigger from game logic.
    Manual,
}

impl AutoSaveTrigger {
    /// Returns display name for the trigger.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Interval => "Auto-save",
            Self::AreaTransition => "Area Transition",
            Self::QuestComplete => "Quest Complete",
            Self::BossFightComplete => "Boss Defeated",
            Self::ImportantItemAcquired => "Item Acquired",
            Self::SavePoint => "Save Point",
            Self::MajorDecision => "Major Decision",
            Self::Manual => "Manual",
        }
    }
}

/// Conditions that pause auto-saving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutoSavePauseReason {
    /// In active combat.
    Combat,
    /// Cutscene playing.
    Cutscene,
    /// In a menu/dialog.
    Menu,
    /// Loading screen.
    Loading,
    /// Tutorial active.
    Tutorial,
    /// Boss fight in progress.
    BossFight,
    /// Player explicitly paused.
    PlayerPaused,
    /// System pause (minimize, etc).
    SystemPaused,
}

impl AutoSavePauseReason {
    /// Returns display name for the reason.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Combat => "In Combat",
            Self::Cutscene => "Cutscene",
            Self::Menu => "Menu Open",
            Self::Loading => "Loading",
            Self::Tutorial => "Tutorial",
            Self::BossFight => "Boss Fight",
            Self::PlayerPaused => "Paused",
            Self::SystemPaused => "System Paused",
        }
    }
}

/// Configuration for auto-save behavior.
#[derive(Debug, Clone)]
pub struct AutoSaveConfig {
    /// Whether auto-save is enabled.
    pub enabled: bool,
    /// Interval between automatic saves (in seconds).
    pub interval_seconds: f64,
    /// Number of rotating auto-save slots.
    pub rotating_slots: usize,
    /// Save on area transitions.
    pub save_on_area_transition: bool,
    /// Save on quest completion.
    pub save_on_quest_complete: bool,
    /// Save on boss defeat.
    pub save_on_boss_defeat: bool,
    /// Pause auto-save during combat.
    pub pause_during_combat: bool,
    /// Pause auto-save during cutscenes.
    pub pause_during_cutscenes: bool,
    /// Show notification on auto-save.
    pub show_notification: bool,
    /// Notification duration in seconds.
    pub notification_duration: f32,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: DEFAULT_AUTOSAVE_INTERVAL,
            rotating_slots: DEFAULT_AUTOSAVE_SLOTS,
            save_on_area_transition: true,
            save_on_quest_complete: true,
            save_on_boss_defeat: true,
            pause_during_combat: true,
            pause_during_cutscenes: true,
            show_notification: true,
            notification_duration: 2.0,
        }
    }
}

impl AutoSaveConfig {
    /// Creates a new auto-save config with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a config with auto-save disabled.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Creates a config with aggressive saving.
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            enabled: true,
            interval_seconds: 60.0,
            rotating_slots: 5,
            save_on_area_transition: true,
            save_on_quest_complete: true,
            save_on_boss_defeat: true,
            ..Default::default()
        }
    }

    /// Sets the auto-save interval.
    #[must_use]
    pub fn with_interval(mut self, seconds: f64) -> Self {
        self.interval_seconds = seconds.max(MIN_AUTOSAVE_INTERVAL);
        self
    }

    /// Sets the number of rotating slots.
    #[must_use]
    pub fn with_slots(mut self, slots: usize) -> Self {
        self.rotating_slots = slots.min(MAX_AUTOSAVE_SLOTS).max(1);
        self
    }

    /// Enables or disables auto-save.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Validates the configuration.
    #[must_use]
    pub fn validate(&self) -> bool {
        self.interval_seconds >= MIN_AUTOSAVE_INTERVAL
            && self.rotating_slots >= 1
            && self.rotating_slots <= MAX_AUTOSAVE_SLOTS
            && self.notification_duration >= 0.0
    }
}

/// Record of a completed auto-save.
#[derive(Debug, Clone)]
pub struct AutoSaveRecord {
    /// The trigger that caused the save.
    pub trigger: AutoSaveTrigger,
    /// Slot name where saved.
    pub slot_name: String,
    /// When the save occurred.
    pub timestamp: Instant,
    /// Whether the save was successful.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Status of the auto-save system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoSaveStatus {
    /// System is idle, waiting for next trigger.
    Idle,
    /// Auto-save is paused.
    Paused,
    /// Currently saving.
    Saving,
    /// Disabled by config.
    Disabled,
}

/// Manager for automatic saves.
pub struct AutoSaveManager {
    /// Configuration.
    config: AutoSaveConfig,
    /// Current status.
    status: AutoSaveStatus,
    /// Time since last auto-save.
    time_since_save: f64,
    /// Current rotating slot index.
    current_slot_index: usize,
    /// Active pause reasons.
    pause_reasons: Vec<AutoSavePauseReason>,
    /// History of recent auto-saves.
    save_history: VecDeque<AutoSaveRecord>,
    /// Maximum history entries.
    max_history: usize,
    /// Pending triggers to process.
    pending_triggers: Vec<AutoSaveTrigger>,
    /// Last save instant.
    last_save_time: Option<Instant>,
    /// Notification timer.
    notification_timer: f32,
    /// Whether to show notification.
    show_notification: bool,
}

impl Default for AutoSaveManager {
    fn default() -> Self {
        Self::new(AutoSaveConfig::default())
    }
}

impl AutoSaveManager {
    /// Creates a new auto-save manager.
    #[must_use]
    pub fn new(config: AutoSaveConfig) -> Self {
        let status = if config.enabled {
            AutoSaveStatus::Idle
        } else {
            AutoSaveStatus::Disabled
        };

        Self {
            config,
            status,
            time_since_save: 0.0,
            current_slot_index: 0,
            pause_reasons: Vec::new(),
            save_history: VecDeque::new(),
            max_history: 20,
            pending_triggers: Vec::new(),
            last_save_time: None,
            notification_timer: 0.0,
            show_notification: false,
        }
    }

    /// Returns the current configuration.
    #[must_use]
    pub fn config(&self) -> &AutoSaveConfig {
        &self.config
    }

    /// Returns the current status.
    #[must_use]
    pub fn status(&self) -> AutoSaveStatus {
        self.status
    }

    /// Returns whether auto-save is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Returns whether auto-save is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        !self.pause_reasons.is_empty()
    }

    /// Returns the active pause reasons.
    #[must_use]
    pub fn pause_reasons(&self) -> &[AutoSavePauseReason] {
        &self.pause_reasons
    }

    /// Returns time until next auto-save.
    #[must_use]
    pub fn time_until_next_save(&self) -> f64 {
        (self.config.interval_seconds - self.time_since_save).max(0.0)
    }

    /// Returns the save history.
    #[must_use]
    pub fn history(&self) -> &VecDeque<AutoSaveRecord> {
        &self.save_history
    }

    /// Returns whether notification should be shown.
    #[must_use]
    pub fn should_show_notification(&self) -> bool {
        self.show_notification
    }

    /// Updates the configuration.
    pub fn update_config(&mut self, config: AutoSaveConfig) {
        let was_enabled = self.config.enabled;
        self.config = config;

        if self.config.enabled && !was_enabled {
            self.status = AutoSaveStatus::Idle;
            info!("Auto-save enabled");
        } else if !self.config.enabled {
            self.status = AutoSaveStatus::Disabled;
            info!("Auto-save disabled");
        }
    }

    /// Enables auto-save.
    pub fn enable(&mut self) {
        self.config.enabled = true;
        if self.pause_reasons.is_empty() {
            self.status = AutoSaveStatus::Idle;
        } else {
            self.status = AutoSaveStatus::Paused;
        }
        info!("Auto-save enabled");
    }

    /// Disables auto-save.
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.status = AutoSaveStatus::Disabled;
        info!("Auto-save disabled");
    }

    /// Pauses auto-save with a reason.
    pub fn pause(&mut self, reason: AutoSavePauseReason) {
        if !self.pause_reasons.contains(&reason) {
            self.pause_reasons.push(reason);
            debug!("Auto-save paused: {:?}", reason);
        }

        if self.config.enabled {
            self.status = AutoSaveStatus::Paused;
        }
    }

    /// Resumes auto-save by removing a pause reason.
    pub fn resume(&mut self, reason: AutoSavePauseReason) {
        self.pause_reasons.retain(|r| *r != reason);
        debug!("Auto-save pause removed: {:?}", reason);

        if self.pause_reasons.is_empty() && self.config.enabled {
            self.status = AutoSaveStatus::Idle;
        }
    }

    /// Clears all pause reasons.
    pub fn clear_pauses(&mut self) {
        self.pause_reasons.clear();
        if self.config.enabled {
            self.status = AutoSaveStatus::Idle;
        }
    }

    /// Triggers an auto-save event.
    pub fn trigger(&mut self, trigger: AutoSaveTrigger) {
        match trigger {
            AutoSaveTrigger::AreaTransition if !self.config.save_on_area_transition => return,
            AutoSaveTrigger::QuestComplete if !self.config.save_on_quest_complete => return,
            AutoSaveTrigger::BossFightComplete if !self.config.save_on_boss_defeat => return,
            _ => {}
        }

        self.pending_triggers.push(trigger);
        debug!("Auto-save triggered: {:?}", trigger);
    }

    /// Returns the next auto-save slot name.
    #[must_use]
    pub fn next_slot_name(&self) -> String {
        format!("{}_{}", AUTOSAVE_PREFIX, self.current_slot_index)
    }

    /// Advances to the next rotating slot.
    fn advance_slot(&mut self) {
        self.current_slot_index = (self.current_slot_index + 1) % self.config.rotating_slots;
    }

    /// Updates the auto-save manager (call each frame).
    pub fn update(&mut self, delta_time: f64) {
        // Update notification timer
        if self.show_notification {
            self.notification_timer -= delta_time as f32;
            if self.notification_timer <= 0.0 {
                self.show_notification = false;
            }
        }

        // Don't process if disabled
        if !self.config.enabled {
            return;
        }

        // Don't accumulate time if paused
        if self.is_paused() {
            return;
        }

        // Accumulate time
        self.time_since_save += delta_time;
    }

    /// Checks if auto-save should trigger and performs save if needed.
    /// Returns true if a save was initiated.
    pub fn check_and_save(&mut self, save_manager: &mut SaveManager, data: &SaveFileData) -> bool {
        if !self.config.enabled || self.is_paused() {
            return false;
        }

        // Check interval trigger
        if self.time_since_save >= self.config.interval_seconds {
            self.pending_triggers.push(AutoSaveTrigger::Interval);
        }

        // Process pending triggers
        if let Some(trigger) = self.pending_triggers.pop() {
            return self.perform_save(save_manager, data, trigger);
        }

        false
    }

    /// Performs the actual save operation.
    fn perform_save(
        &mut self,
        save_manager: &mut SaveManager,
        data: &SaveFileData,
        trigger: AutoSaveTrigger,
    ) -> bool {
        self.status = AutoSaveStatus::Saving;
        let slot_name = self.next_slot_name();

        // Create auto-save data with updated metadata
        let mut save_data = data.clone();
        save_data.metadata.is_autosave = true;
        save_data.metadata.display_name = format!(
            "{} - {}",
            trigger.display_name(),
            save_data.metadata.location
        );

        let result = save_manager.save(&slot_name, &save_data);

        let record = AutoSaveRecord {
            trigger,
            slot_name: slot_name.clone(),
            timestamp: Instant::now(),
            success: result.is_ok(),
            error: result.as_ref().err().map(|e| e.to_string()),
        };

        // Update history
        self.save_history.push_front(record);
        while self.save_history.len() > self.max_history {
            self.save_history.pop_back();
        }

        match result {
            Ok(()) => {
                self.time_since_save = 0.0;
                self.last_save_time = Some(Instant::now());
                self.advance_slot();
                self.pending_triggers.clear();

                if self.config.show_notification {
                    self.show_notification = true;
                    self.notification_timer = self.config.notification_duration;
                }

                info!("Auto-save complete: {} ({:?})", slot_name, trigger);
                self.status = AutoSaveStatus::Idle;
                true
            }
            Err(e) => {
                warn!("Auto-save failed: {}", e);
                self.status = AutoSaveStatus::Idle;
                false
            }
        }
    }

    /// Forces an immediate auto-save.
    pub fn force_save(&mut self, save_manager: &mut SaveManager, data: &SaveFileData) -> SaveResult<()> {
        let slot_name = self.next_slot_name();

        let mut save_data = data.clone();
        save_data.metadata.is_autosave = true;
        save_data.metadata.display_name = format!("Manual - {}", save_data.metadata.location);

        let result = save_manager.save(&slot_name, &save_data);

        if result.is_ok() {
            self.time_since_save = 0.0;
            self.last_save_time = Some(Instant::now());
            self.advance_slot();
            info!("Forced auto-save complete: {}", slot_name);
        }

        result
    }

    /// Resets the auto-save timer.
    pub fn reset_timer(&mut self) {
        self.time_since_save = 0.0;
    }

    /// Gets the last successful save time.
    #[must_use]
    pub fn last_save_time(&self) -> Option<Instant> {
        self.last_save_time
    }

    /// Gets time since last save in seconds.
    #[must_use]
    pub fn time_since_last_save(&self) -> Option<f64> {
        self.last_save_time
            .map(|t| t.elapsed().as_secs_f64())
    }

    /// Checks if combat pause should be applied.
    fn should_pause_for_combat(&self) -> bool {
        self.config.pause_during_combat
    }

    /// Checks if cutscene pause should be applied.
    fn should_pause_for_cutscene(&self) -> bool {
        self.config.pause_during_cutscenes
    }

    /// Called when combat starts.
    pub fn on_combat_start(&mut self) {
        if self.should_pause_for_combat() {
            self.pause(AutoSavePauseReason::Combat);
        }
    }

    /// Called when combat ends.
    pub fn on_combat_end(&mut self) {
        self.resume(AutoSavePauseReason::Combat);
    }

    /// Called when cutscene starts.
    pub fn on_cutscene_start(&mut self) {
        if self.should_pause_for_cutscene() {
            self.pause(AutoSavePauseReason::Cutscene);
        }
    }

    /// Called when cutscene ends.
    pub fn on_cutscene_end(&mut self) {
        self.resume(AutoSavePauseReason::Cutscene);
    }

    /// Called when boss fight starts.
    pub fn on_boss_fight_start(&mut self) {
        self.pause(AutoSavePauseReason::BossFight);
    }

    /// Called when boss fight ends (triggers save if configured).
    pub fn on_boss_fight_end(&mut self) {
        self.resume(AutoSavePauseReason::BossFight);
        if self.config.save_on_boss_defeat {
            self.trigger(AutoSaveTrigger::BossFightComplete);
        }
    }

    /// Called when entering a new area (triggers save if configured).
    pub fn on_area_transition(&mut self) {
        if self.config.save_on_area_transition {
            self.trigger(AutoSaveTrigger::AreaTransition);
        }
    }

    /// Called when completing a quest (triggers save if configured).
    pub fn on_quest_complete(&mut self) {
        if self.config.save_on_quest_complete {
            self.trigger(AutoSaveTrigger::QuestComplete);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save_manager::SaveFileBuilder;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn test_save_dir() -> PathBuf {
        env::temp_dir().join("genesis_test_autosave")
    }

    fn cleanup_test_dir(path: &std::path::Path) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }

    #[test]
    fn test_autosave_config_default() {
        let config = AutoSaveConfig::default();
        assert!(config.enabled);
        assert!((config.interval_seconds - DEFAULT_AUTOSAVE_INTERVAL).abs() < 0.01);
        assert_eq!(config.rotating_slots, DEFAULT_AUTOSAVE_SLOTS);
        assert!(config.validate());
    }

    #[test]
    fn test_autosave_config_disabled() {
        let config = AutoSaveConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_autosave_config_aggressive() {
        let config = AutoSaveConfig::aggressive();
        assert!(config.enabled);
        assert!((config.interval_seconds - 60.0).abs() < 0.01);
        assert_eq!(config.rotating_slots, 5);
    }

    #[test]
    fn test_autosave_config_builder() {
        let config = AutoSaveConfig::new()
            .with_interval(120.0)
            .with_slots(5)
            .with_enabled(true);

        assert!(config.enabled);
        assert!((config.interval_seconds - 120.0).abs() < 0.01);
        assert_eq!(config.rotating_slots, 5);
    }

    #[test]
    fn test_autosave_config_validation() {
        let valid_config = AutoSaveConfig::default();
        assert!(valid_config.validate());

        let mut invalid_config = AutoSaveConfig::default();
        invalid_config.interval_seconds = 0.0;
        assert!(!invalid_config.validate());
    }

    #[test]
    fn test_autosave_manager_new() {
        let manager = AutoSaveManager::new(AutoSaveConfig::default());
        assert_eq!(manager.status(), AutoSaveStatus::Idle);
        assert!(manager.is_enabled());
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_autosave_manager_disabled() {
        let manager = AutoSaveManager::new(AutoSaveConfig::disabled());
        assert_eq!(manager.status(), AutoSaveStatus::Disabled);
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_autosave_manager_pause_resume() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());

        manager.pause(AutoSavePauseReason::Combat);
        assert!(manager.is_paused());
        assert_eq!(manager.status(), AutoSaveStatus::Paused);
        assert_eq!(manager.pause_reasons().len(), 1);

        manager.pause(AutoSavePauseReason::Cutscene);
        assert_eq!(manager.pause_reasons().len(), 2);

        manager.resume(AutoSavePauseReason::Combat);
        assert!(manager.is_paused()); // Still paused by cutscene
        assert_eq!(manager.pause_reasons().len(), 1);

        manager.resume(AutoSavePauseReason::Cutscene);
        assert!(!manager.is_paused());
        assert_eq!(manager.status(), AutoSaveStatus::Idle);
    }

    #[test]
    fn test_autosave_manager_clear_pauses() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());

        manager.pause(AutoSavePauseReason::Combat);
        manager.pause(AutoSavePauseReason::Cutscene);
        assert!(manager.is_paused());

        manager.clear_pauses();
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_autosave_manager_update_time() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::new().with_interval(60.0));

        manager.update(10.0);
        assert!((manager.time_until_next_save() - 50.0).abs() < 0.01);

        manager.update(30.0);
        assert!((manager.time_until_next_save() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_autosave_manager_update_paused() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::new().with_interval(60.0));

        manager.pause(AutoSavePauseReason::Combat);
        let time_before = manager.time_until_next_save();

        manager.update(10.0);
        let time_after = manager.time_until_next_save();

        // Time should not have changed while paused
        assert!((time_before - time_after).abs() < 0.01);
    }

    #[test]
    fn test_autosave_manager_slot_rotation() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::new().with_slots(3));

        assert_eq!(manager.next_slot_name(), "autosave_0");
        manager.advance_slot();
        assert_eq!(manager.next_slot_name(), "autosave_1");
        manager.advance_slot();
        assert_eq!(manager.next_slot_name(), "autosave_2");
        manager.advance_slot();
        assert_eq!(manager.next_slot_name(), "autosave_0"); // Wraps around
    }

    #[test]
    fn test_autosave_manager_trigger() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());

        manager.trigger(AutoSaveTrigger::AreaTransition);
        assert_eq!(manager.pending_triggers.len(), 1);

        manager.trigger(AutoSaveTrigger::QuestComplete);
        assert_eq!(manager.pending_triggers.len(), 2);
    }

    #[test]
    fn test_autosave_manager_trigger_disabled() {
        let config = AutoSaveConfig {
            save_on_area_transition: false,
            ..Default::default()
        };
        let mut manager = AutoSaveManager::new(config);

        manager.trigger(AutoSaveTrigger::AreaTransition);
        assert_eq!(manager.pending_triggers.len(), 0);

        manager.trigger(AutoSaveTrigger::QuestComplete);
        assert_eq!(manager.pending_triggers.len(), 1);
    }

    #[test]
    fn test_autosave_manager_enable_disable() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());
        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());
        assert_eq!(manager.status(), AutoSaveStatus::Disabled);

        manager.enable();
        assert!(manager.is_enabled());
        assert_eq!(manager.status(), AutoSaveStatus::Idle);
    }

    #[test]
    fn test_autosave_manager_combat_events() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());

        manager.on_combat_start();
        assert!(manager.is_paused());
        assert!(manager.pause_reasons().contains(&AutoSavePauseReason::Combat));

        manager.on_combat_end();
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_autosave_manager_cutscene_events() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());

        manager.on_cutscene_start();
        assert!(manager.is_paused());
        assert!(manager.pause_reasons().contains(&AutoSavePauseReason::Cutscene));

        manager.on_cutscene_end();
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_autosave_manager_boss_fight_events() {
        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());

        manager.on_boss_fight_start();
        assert!(manager.is_paused());
        assert!(manager.pause_reasons().contains(&AutoSavePauseReason::BossFight));

        manager.on_boss_fight_end();
        assert!(!manager.is_paused());
        // Should have triggered boss defeat save
        assert!(manager.pending_triggers.contains(&AutoSaveTrigger::BossFightComplete));
    }

    #[test]
    fn test_autosave_trigger_display_name() {
        assert_eq!(AutoSaveTrigger::Interval.display_name(), "Auto-save");
        assert_eq!(AutoSaveTrigger::AreaTransition.display_name(), "Area Transition");
        assert_eq!(AutoSaveTrigger::BossFightComplete.display_name(), "Boss Defeated");
    }

    #[test]
    fn test_autosave_pause_reason_display_name() {
        assert_eq!(AutoSavePauseReason::Combat.display_name(), "In Combat");
        assert_eq!(AutoSavePauseReason::Cutscene.display_name(), "Cutscene");
    }

    #[test]
    fn test_autosave_manager_check_and_save() {
        let dir = test_save_dir().join("test_check_save");
        cleanup_test_dir(&dir);

        // Use MIN_AUTOSAVE_INTERVAL (30s) since that's the minimum allowed
        let config = AutoSaveConfig::new().with_interval(MIN_AUTOSAVE_INTERVAL);
        let mut manager = AutoSaveManager::new(config);
        let mut save_manager = SaveManager::new(&dir);

        let data = SaveFileBuilder::new("autosave_test")
            .location("Test Area")
            .build();

        // Not enough time elapsed (half the interval)
        manager.update(MIN_AUTOSAVE_INTERVAL / 2.0);
        assert!(!manager.check_and_save(&mut save_manager, &data));

        // Enough time elapsed (another 60% of interval)
        manager.update(MIN_AUTOSAVE_INTERVAL * 0.6);
        assert!(manager.check_and_save(&mut save_manager, &data));

        // Timer should be reset - should be close to full interval
        assert!(manager.time_until_next_save() > MIN_AUTOSAVE_INTERVAL * 0.9);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_autosave_manager_force_save() {
        let dir = test_save_dir().join("test_force_save");
        cleanup_test_dir(&dir);

        let mut manager = AutoSaveManager::new(AutoSaveConfig::default());
        let mut save_manager = SaveManager::new(&dir);

        let data = SaveFileBuilder::new("force_save_test")
            .location("Force Location")
            .build();

        let result = manager.force_save(&mut save_manager, &data);
        assert!(result.is_ok());

        cleanup_test_dir(&dir);
    }
}
