//! Save file management system.
//!
//! This module provides:
//! - SaveManager: orchestrate all save operations
//! - Save slot directory structure
//! - Atomic save operations (temp file + rename)
//! - Error handling and recovery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::combat_save::CombatSaveData;
use crate::crafting_save::CraftingSaveData;
use crate::save_version::CURRENT_SAVE_VERSION;

/// Default save directory name.
pub const DEFAULT_SAVE_DIR: &str = "saves";

/// Maximum number of save slots.
pub const MAX_SAVE_SLOTS: usize = 10;

/// Auto-save slot prefix.
pub const AUTOSAVE_PREFIX: &str = "autosave";

/// Quick-save slot name.
pub const QUICKSAVE_SLOT: &str = "quicksave";

/// Errors that can occur during save operations.
#[derive(Debug, Error)]
pub enum SaveError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Save slot not found.
    #[error("Save slot not found: {0}")]
    SlotNotFound(String),

    /// Invalid slot name.
    #[error("Invalid slot name: {0}")]
    InvalidSlotName(String),

    /// Corrupted save.
    #[error("Corrupted save file: {0}")]
    Corrupted(String),

    /// Version mismatch.
    #[error("Save version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// Expected version.
        expected: u32,
        /// Found version.
        found: u32,
    },

    /// Save in progress.
    #[error("Save operation already in progress")]
    SaveInProgress,

    /// Atomic write failed.
    #[error("Atomic write failed: {0}")]
    AtomicWriteFailed(String),
}

/// Result type for save operations.
pub type SaveResult<T> = Result<T, SaveError>;

/// Metadata about a save slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlotMetadata {
    /// Slot name/identifier.
    pub slot_name: String,
    /// Display name shown to user.
    pub display_name: String,
    /// Save version.
    pub version: u32,
    /// Timestamp when saved (Unix epoch seconds).
    pub timestamp: u64,
    /// Total playtime in seconds.
    pub playtime_seconds: f64,
    /// Player level.
    pub player_level: u32,
    /// Current location/area name.
    pub location: String,
    /// Screenshot thumbnail path (relative to save dir).
    pub thumbnail: Option<String>,
    /// Whether this is an auto-save.
    pub is_autosave: bool,
    /// Checksum for integrity verification.
    pub checksum: u32,
}

impl SaveSlotMetadata {
    /// Creates new metadata for a save slot.
    #[must_use]
    pub fn new(slot_name: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        Self {
            slot_name: slot_name.into(),
            display_name: String::new(),
            version: CURRENT_SAVE_VERSION,
            timestamp,
            playtime_seconds: 0.0,
            player_level: 1,
            location: "Unknown".to_string(),
            thumbnail: None,
            is_autosave: false,
            checksum: 0,
        }
    }

    /// Sets the display name.
    #[must_use]
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    /// Sets playtime.
    #[must_use]
    pub fn with_playtime(mut self, seconds: f64) -> Self {
        self.playtime_seconds = seconds;
        self
    }

    /// Sets player level.
    #[must_use]
    pub fn with_player_level(mut self, level: u32) -> Self {
        self.player_level = level;
        self
    }

    /// Sets location.
    #[must_use]
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }

    /// Marks as autosave.
    #[must_use]
    pub fn as_autosave(mut self) -> Self {
        self.is_autosave = true;
        self
    }

    /// Returns formatted timestamp.
    #[must_use]
    pub fn formatted_timestamp(&self) -> String {
        // Simple formatting - in production would use chrono
        let hours = (self.timestamp / 3600) % 24;
        let minutes = (self.timestamp / 60) % 60;
        format!("{hours:02}:{minutes:02}")
    }

    /// Returns formatted playtime.
    #[must_use]
    pub fn formatted_playtime(&self) -> String {
        let total_seconds = self.playtime_seconds as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

/// Complete save file data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFileData {
    /// Save metadata.
    pub metadata: SaveSlotMetadata,
    /// Crafting state.
    pub crafting: CraftingSaveData,
    /// Combat state.
    pub combat: CombatSaveData,
    /// Player position.
    pub player_position: (f32, f32),
    /// World seed.
    pub world_seed: u64,
    /// Game time (day/time).
    pub game_time: f64,
    /// Custom data for extensibility.
    pub custom_data: HashMap<String, String>,
}

impl SaveFileData {
    /// Creates new save file data.
    #[must_use]
    pub fn new(slot_name: impl Into<String>) -> Self {
        Self {
            metadata: SaveSlotMetadata::new(slot_name),
            crafting: CraftingSaveData::default(),
            combat: CombatSaveData::default(),
            player_position: (0.0, 0.0),
            world_seed: 0,
            game_time: 0.0,
            custom_data: HashMap::new(),
        }
    }

    /// Calculates checksum for the save data.
    #[must_use]
    pub fn calculate_checksum(&self) -> u32 {
        // Simple checksum based on key values
        let mut sum: u32 = 0;
        sum = sum.wrapping_add(self.world_seed as u32);
        sum = sum.wrapping_add((self.player_position.0 * 1000.0) as u32);
        sum = sum.wrapping_add((self.player_position.1 * 1000.0) as u32);
        sum = sum.wrapping_add((self.game_time * 1000.0) as u32);
        sum = sum.wrapping_add(self.combat.combat_level);
        sum
    }

    /// Verifies checksum.
    #[must_use]
    pub fn verify_checksum(&self) -> bool {
        self.metadata.checksum == self.calculate_checksum()
    }
}

/// Save operation state for tracking progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveState {
    /// No operation in progress.
    Idle,
    /// Save in progress.
    Saving,
    /// Load in progress.
    Loading,
    /// Operation completed successfully.
    Complete,
    /// Operation failed.
    Failed,
}

/// Manager for all save operations.
pub struct SaveManager {
    /// Base directory for saves.
    save_dir: PathBuf,
    /// Current save state.
    state: SaveState,
    /// Currently loaded slot name.
    current_slot: Option<String>,
    /// Cached metadata for all slots.
    slot_cache: HashMap<String, SaveSlotMetadata>,
    /// Last error message.
    last_error: Option<String>,
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new(DEFAULT_SAVE_DIR)
    }
}

impl SaveManager {
    /// Creates a new save manager.
    #[must_use]
    pub fn new(save_dir: impl AsRef<Path>) -> Self {
        Self {
            save_dir: save_dir.as_ref().to_path_buf(),
            state: SaveState::Idle,
            current_slot: None,
            slot_cache: HashMap::new(),
            last_error: None,
        }
    }

    /// Returns the save directory path.
    #[must_use]
    pub fn save_dir(&self) -> &Path {
        &self.save_dir
    }

    /// Returns current save state.
    #[must_use]
    pub fn state(&self) -> SaveState {
        self.state
    }

    /// Returns the currently loaded slot.
    #[must_use]
    pub fn current_slot(&self) -> Option<&str> {
        self.current_slot.as_deref()
    }

    /// Returns the last error message.
    #[must_use]
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Ensures the save directory exists.
    pub fn ensure_save_dir(&self) -> SaveResult<()> {
        if !self.save_dir.exists() {
            fs::create_dir_all(&self.save_dir)?;
            info!("Created save directory: {:?}", self.save_dir);
        }
        Ok(())
    }

    /// Returns the path for a save slot.
    fn slot_path(&self, slot_name: &str) -> PathBuf {
        self.save_dir.join(format!("{slot_name}.sav"))
    }

    /// Returns the path for a save slot's metadata.
    fn metadata_path(&self, slot_name: &str) -> PathBuf {
        self.save_dir.join(format!("{slot_name}.meta"))
    }

    /// Returns the temp path for atomic writes.
    fn temp_path(&self, slot_name: &str) -> PathBuf {
        self.save_dir.join(format!("{slot_name}.tmp"))
    }

    /// Validates a slot name.
    fn validate_slot_name(slot_name: &str) -> SaveResult<()> {
        if slot_name.is_empty() {
            return Err(SaveError::InvalidSlotName("Empty slot name".to_string()));
        }

        // Check for invalid characters
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for c in invalid_chars {
            if slot_name.contains(c) {
                return Err(SaveError::InvalidSlotName(format!(
                    "Invalid character '{c}' in slot name"
                )));
            }
        }

        Ok(())
    }

    /// Saves game data to a slot.
    pub fn save(&mut self, slot_name: &str, data: &SaveFileData) -> SaveResult<()> {
        Self::validate_slot_name(slot_name)?;

        if self.state == SaveState::Saving {
            return Err(SaveError::SaveInProgress);
        }

        self.state = SaveState::Saving;
        self.last_error = None;

        // Ensure directory exists
        self.ensure_save_dir()?;

        // Prepare data with checksum
        let mut save_data = data.clone();
        save_data.metadata.checksum = save_data.calculate_checksum();
        save_data.metadata.slot_name = slot_name.to_string();

        // Atomic write: write to temp file first
        let result = self.atomic_write(slot_name, &save_data);

        match &result {
            Ok(()) => {
                self.state = SaveState::Complete;
                self.current_slot = Some(slot_name.to_string());
                self.slot_cache
                    .insert(slot_name.to_string(), save_data.metadata.clone());
                info!("Saved game to slot: {}", slot_name);
            }
            Err(e) => {
                self.state = SaveState::Failed;
                self.last_error = Some(e.to_string());
                error!("Failed to save: {}", e);
            }
        }

        result
    }

    /// Performs atomic write (temp file + rename).
    fn atomic_write(&self, slot_name: &str, data: &SaveFileData) -> SaveResult<()> {
        let temp_path = self.temp_path(slot_name);
        let final_path = self.slot_path(slot_name);
        let meta_path = self.metadata_path(slot_name);

        // Write to temp file
        {
            let file = File::create(&temp_path)?;
            let mut writer = BufWriter::new(file);

            serde_json::to_writer_pretty(&mut writer, data)
                .map_err(|e| SaveError::Serialization(e.to_string()))?;

            writer.flush()?;
        }

        // Atomic rename
        fs::rename(&temp_path, &final_path).map_err(|e| {
            // Clean up temp file on failure
            let _ = fs::remove_file(&temp_path);
            SaveError::AtomicWriteFailed(e.to_string())
        })?;

        // Write metadata separately for quick listing
        {
            let file = File::create(&meta_path)?;
            let mut writer = BufWriter::new(file);

            serde_json::to_writer_pretty(&mut writer, &data.metadata)
                .map_err(|e| SaveError::Serialization(e.to_string()))?;

            writer.flush()?;
        }

        debug!("Atomic write complete for slot: {}", slot_name);
        Ok(())
    }

    /// Loads game data from a slot.
    pub fn load(&mut self, slot_name: &str) -> SaveResult<SaveFileData> {
        Self::validate_slot_name(slot_name)?;

        if self.state == SaveState::Loading {
            return Err(SaveError::SaveInProgress);
        }

        self.state = SaveState::Loading;
        self.last_error = None;

        let path = self.slot_path(slot_name);

        if !path.exists() {
            self.state = SaveState::Failed;
            let err = SaveError::SlotNotFound(slot_name.to_string());
            self.last_error = Some(err.to_string());
            return Err(err);
        }

        let result = self.load_file(&path);

        match &result {
            Ok(data) => {
                // Verify checksum
                if !data.verify_checksum() {
                    warn!("Save checksum mismatch for slot: {}", slot_name);
                }

                self.state = SaveState::Complete;
                self.current_slot = Some(slot_name.to_string());
                info!("Loaded game from slot: {}", slot_name);
            }
            Err(e) => {
                self.state = SaveState::Failed;
                self.last_error = Some(e.to_string());
                error!("Failed to load: {}", e);
            }
        }

        result
    }

    /// Loads a save file from path.
    fn load_file(&self, path: &Path) -> SaveResult<SaveFileData> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let data: SaveFileData = serde_json::from_reader(reader)
            .map_err(|e| SaveError::Deserialization(e.to_string()))?;

        // Check version compatibility
        if data.metadata.version > CURRENT_SAVE_VERSION {
            return Err(SaveError::VersionMismatch {
                expected: CURRENT_SAVE_VERSION,
                found: data.metadata.version,
            });
        }

        Ok(data)
    }

    /// Deletes a save slot.
    pub fn delete_slot(&mut self, slot_name: &str) -> SaveResult<()> {
        Self::validate_slot_name(slot_name)?;

        let save_path = self.slot_path(slot_name);
        let meta_path = self.metadata_path(slot_name);

        if !save_path.exists() {
            return Err(SaveError::SlotNotFound(slot_name.to_string()));
        }

        fs::remove_file(&save_path)?;

        if meta_path.exists() {
            fs::remove_file(&meta_path)?;
        }

        self.slot_cache.remove(slot_name);

        if self.current_slot.as_deref() == Some(slot_name) {
            self.current_slot = None;
        }

        info!("Deleted save slot: {}", slot_name);
        Ok(())
    }

    /// Lists all available save slots.
    pub fn list_slots(&mut self) -> SaveResult<Vec<SaveSlotMetadata>> {
        self.ensure_save_dir()?;

        let mut slots = Vec::new();

        for entry in fs::read_dir(&self.save_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only look at .sav files
            if path.extension().is_some_and(|ext| ext == "sav") {
                if let Some(stem) = path.file_stem() {
                    let slot_name = stem.to_string_lossy().to_string();

                    // Try to load metadata from .meta file first (faster)
                    let meta = self.load_slot_metadata(&slot_name);

                    if let Ok(metadata) = meta {
                        self.slot_cache.insert(slot_name, metadata.clone());
                        slots.push(metadata);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        slots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(slots)
    }

    /// Loads metadata for a specific slot.
    fn load_slot_metadata(&self, slot_name: &str) -> SaveResult<SaveSlotMetadata> {
        // Try metadata file first
        let meta_path = self.metadata_path(slot_name);
        if meta_path.exists() {
            let file = File::open(&meta_path)?;
            let reader = BufReader::new(file);
            let meta: SaveSlotMetadata = serde_json::from_reader(reader)
                .map_err(|e| SaveError::Deserialization(e.to_string()))?;
            return Ok(meta);
        }

        // Fall back to loading full save
        let save_path = self.slot_path(slot_name);
        if save_path.exists() {
            let data = self.load_file(&save_path)?;
            return Ok(data.metadata);
        }

        Err(SaveError::SlotNotFound(slot_name.to_string()))
    }

    /// Checks if a slot exists.
    #[must_use]
    pub fn slot_exists(&self, slot_name: &str) -> bool {
        self.slot_path(slot_name).exists()
    }

    /// Gets metadata for a slot from cache or loads it.
    pub fn get_slot_metadata(&mut self, slot_name: &str) -> SaveResult<SaveSlotMetadata> {
        if let Some(cached) = self.slot_cache.get(slot_name) {
            return Ok(cached.clone());
        }

        let meta = self.load_slot_metadata(slot_name)?;
        self.slot_cache.insert(slot_name.to_string(), meta.clone());
        Ok(meta)
    }

    /// Creates a backup of a save slot.
    pub fn backup_slot(&self, slot_name: &str) -> SaveResult<String> {
        Self::validate_slot_name(slot_name)?;

        let source_path = self.slot_path(slot_name);
        if !source_path.exists() {
            return Err(SaveError::SlotNotFound(slot_name.to_string()));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        let backup_name = format!("{slot_name}_backup_{timestamp}");
        let backup_path = self.slot_path(&backup_name);

        fs::copy(&source_path, &backup_path)?;

        // Also copy metadata if exists
        let meta_source = self.metadata_path(slot_name);
        if meta_source.exists() {
            let meta_backup = self.metadata_path(&backup_name);
            fs::copy(&meta_source, &meta_backup)?;
        }

        info!("Created backup: {} -> {}", slot_name, backup_name);
        Ok(backup_name)
    }

    /// Quick save to the quicksave slot.
    pub fn quicksave(&mut self, data: &SaveFileData) -> SaveResult<()> {
        self.save(QUICKSAVE_SLOT, data)
    }

    /// Quick load from the quicksave slot.
    pub fn quickload(&mut self) -> SaveResult<SaveFileData> {
        self.load(QUICKSAVE_SLOT)
    }

    /// Clears the slot cache.
    pub fn clear_cache(&mut self) {
        self.slot_cache.clear();
    }
}

/// Builder for creating save file data from game state.
pub struct SaveFileBuilder {
    data: SaveFileData,
}

impl SaveFileBuilder {
    /// Creates a new save file builder.
    #[must_use]
    pub fn new(slot_name: impl Into<String>) -> Self {
        Self {
            data: SaveFileData::new(slot_name),
        }
    }

    /// Sets the display name.
    #[must_use]
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.data.metadata.display_name = name.into();
        self
    }

    /// Sets player position.
    #[must_use]
    pub fn player_position(mut self, x: f32, y: f32) -> Self {
        self.data.player_position = (x, y);
        self
    }

    /// Sets world seed.
    #[must_use]
    pub fn world_seed(mut self, seed: u64) -> Self {
        self.data.world_seed = seed;
        self
    }

    /// Sets game time.
    #[must_use]
    pub fn game_time(mut self, time: f64) -> Self {
        self.data.game_time = time;
        self
    }

    /// Sets playtime.
    #[must_use]
    pub fn playtime(mut self, seconds: f64) -> Self {
        self.data.metadata.playtime_seconds = seconds;
        self
    }

    /// Sets player level.
    #[must_use]
    pub fn player_level(mut self, level: u32) -> Self {
        self.data.metadata.player_level = level;
        self
    }

    /// Sets location name.
    #[must_use]
    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.data.metadata.location = location.into();
        self
    }

    /// Sets crafting data.
    #[must_use]
    pub fn crafting(mut self, data: CraftingSaveData) -> Self {
        self.data.crafting = data;
        self
    }

    /// Sets combat data.
    #[must_use]
    pub fn combat(mut self, data: CombatSaveData) -> Self {
        self.data.combat = data;
        self
    }

    /// Adds custom data.
    #[must_use]
    pub fn custom_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.custom_data.insert(key.into(), value.into());
        self
    }

    /// Marks as autosave.
    #[must_use]
    pub fn as_autosave(mut self) -> Self {
        self.data.metadata.is_autosave = true;
        self
    }

    /// Builds the save file data.
    #[must_use]
    pub fn build(mut self) -> SaveFileData {
        self.data.metadata.checksum = self.data.calculate_checksum();
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_save_dir() -> PathBuf {
        env::temp_dir().join("genesis_test_saves")
    }

    fn cleanup_test_dir(path: &Path) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }

    #[test]
    fn test_save_slot_metadata_new() {
        let meta = SaveSlotMetadata::new("test_slot");
        assert_eq!(meta.slot_name, "test_slot");
        assert_eq!(meta.version, CURRENT_SAVE_VERSION);
        assert!(!meta.is_autosave);
    }

    #[test]
    fn test_save_slot_metadata_builder() {
        let meta = SaveSlotMetadata::new("slot1")
            .with_display_name("My Save")
            .with_playtime(3661.0)
            .with_player_level(10)
            .with_location("Forest")
            .as_autosave();

        assert_eq!(meta.display_name, "My Save");
        assert!((meta.playtime_seconds - 3661.0).abs() < 0.01);
        assert_eq!(meta.player_level, 10);
        assert_eq!(meta.location, "Forest");
        assert!(meta.is_autosave);
    }

    #[test]
    fn test_save_slot_metadata_formatted_playtime() {
        let meta = SaveSlotMetadata::new("test").with_playtime(3661.0); // 1h 1m 1s
        assert_eq!(meta.formatted_playtime(), "01:01:01");
    }

    #[test]
    fn test_save_file_data_checksum() {
        let mut data = SaveFileData::new("test");
        data.player_position = (100.0, 200.0);
        data.world_seed = 12345;
        data.game_time = 1000.0;

        let checksum = data.calculate_checksum();
        data.metadata.checksum = checksum;
        assert!(data.verify_checksum());

        // Modify data - checksum should fail
        data.player_position = (0.0, 0.0);
        assert!(!data.verify_checksum());
    }

    #[test]
    fn test_save_file_builder() {
        let data = SaveFileBuilder::new("slot1")
            .display_name("My Game")
            .player_position(100.0, 200.0)
            .world_seed(42)
            .game_time(1000.0)
            .playtime(3600.0)
            .player_level(5)
            .location("Town")
            .custom_data("quest", "main_quest_1")
            .build();

        assert_eq!(data.metadata.slot_name, "slot1");
        assert_eq!(data.metadata.display_name, "My Game");
        assert_eq!(data.player_position, (100.0, 200.0));
        assert_eq!(data.world_seed, 42);
        assert_eq!(data.custom_data.get("quest"), Some(&"main_quest_1".to_string()));
        assert!(data.verify_checksum());
    }

    #[test]
    fn test_validate_slot_name() {
        assert!(SaveManager::validate_slot_name("valid_slot").is_ok());
        assert!(SaveManager::validate_slot_name("slot123").is_ok());
        assert!(SaveManager::validate_slot_name("").is_err());
        assert!(SaveManager::validate_slot_name("invalid/slot").is_err());
        assert!(SaveManager::validate_slot_name("invalid:slot").is_err());
    }

    #[test]
    fn test_save_manager_save_load() {
        let dir = test_save_dir().join("test_save_load");
        cleanup_test_dir(&dir);

        let mut manager = SaveManager::new(&dir);

        let data = SaveFileBuilder::new("test")
            .display_name("Test Save")
            .player_position(10.0, 20.0)
            .build();

        // Save
        manager.save("test", &data).expect("Save failed");
        assert!(manager.slot_exists("test"));
        assert_eq!(manager.current_slot(), Some("test"));

        // Load
        let loaded = manager.load("test").expect("Load failed");
        assert_eq!(loaded.metadata.display_name, "Test Save");
        assert_eq!(loaded.player_position, (10.0, 20.0));

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_save_manager_delete() {
        let dir = test_save_dir().join("test_delete");
        cleanup_test_dir(&dir);

        let mut manager = SaveManager::new(&dir);

        let data = SaveFileBuilder::new("to_delete").build();
        manager.save("to_delete", &data).expect("Save failed");
        assert!(manager.slot_exists("to_delete"));

        manager.delete_slot("to_delete").expect("Delete failed");
        assert!(!manager.slot_exists("to_delete"));

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_save_manager_list_slots() {
        let dir = test_save_dir().join("test_list");
        cleanup_test_dir(&dir);

        let mut manager = SaveManager::new(&dir);

        // Create multiple saves
        for i in 1..=3 {
            let data = SaveFileBuilder::new(format!("slot{i}"))
                .display_name(format!("Save {i}"))
                .build();
            manager.save(&format!("slot{i}"), &data).expect("Save failed");
        }

        let slots = manager.list_slots().expect("List failed");
        assert_eq!(slots.len(), 3);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_save_manager_slot_not_found() {
        let dir = test_save_dir().join("test_not_found");
        cleanup_test_dir(&dir);

        let mut manager = SaveManager::new(&dir);
        let result = manager.load("nonexistent");
        assert!(matches!(result, Err(SaveError::SlotNotFound(_))));

        cleanup_test_dir(&dir);
    }
}
