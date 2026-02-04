//! Save/Load game state system.
//!
//! This module provides serialization and persistence of game state,
//! including player data, entities, and world metadata.

use genesis_common::{EntityId, ItemTypeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Current save file format version.
pub const SAVE_VERSION: u32 = 1;

/// Magic bytes for save file identification.
const SAVE_MAGIC: [u8; 4] = *b"GNSV";

/// Errors that can occur during save/load operations.
#[derive(Debug, Error)]
pub enum SaveError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid magic bytes
    #[error("Invalid save file format")]
    InvalidFormat,

    /// Version mismatch
    #[error("Incompatible save version: expected {expected}, found {found}")]
    VersionMismatch {
        /// Expected version
        expected: u32,
        /// Found version
        found: u32,
    },

    /// Save file not found
    #[error("Save not found: {0}")]
    NotFound(String),

    /// Save file corrupted
    #[error("Save file corrupted: {0}")]
    Corrupted(String),
}

/// Result type for save operations.
pub type SaveResult<T> = Result<T, SaveError>;

/// Item stack save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStackSaveData {
    /// Item type ID
    pub item_type: u32,
    /// Quantity
    pub quantity: u32,
    /// Custom data (for unique items)
    pub custom_data: Option<Vec<u8>>,
}

impl ItemStackSaveData {
    /// Creates a new item stack save data.
    #[must_use]
    pub fn new(item_type: ItemTypeId, quantity: u32) -> Self {
        Self {
            item_type: item_type.raw(),
            quantity,
            custom_data: None,
        }
    }

    /// Creates item stack save data with custom data.
    #[must_use]
    pub fn with_custom_data(mut self, data: Vec<u8>) -> Self {
        self.custom_data = Some(data);
        self
    }

    /// Gets the item type ID.
    #[must_use]
    pub fn item_type(&self) -> ItemTypeId {
        ItemTypeId::new(self.item_type)
    }
}

/// Player save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSaveData {
    /// Player position (x, y)
    pub position: (f32, f32),
    /// Player health
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
    /// Inventory items
    pub inventory: Vec<ItemStackSaveData>,
    /// Equipped items (indexed by slot)
    pub equipped: Vec<Option<ItemStackSaveData>>,
    /// Reputation with factions (faction_id -> reputation)
    pub faction_reputation: HashMap<u32, i32>,
    /// Unlocked recipes
    pub unlocked_recipes: Vec<u32>,
    /// Player name
    pub name: Option<String>,
}

impl Default for PlayerSaveData {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            health: 100.0,
            max_health: 100.0,
            inventory: Vec::new(),
            equipped: vec![None; 10], // 10 equipment slots
            faction_reputation: HashMap::new(),
            unlocked_recipes: Vec::new(),
            name: None,
        }
    }
}

impl PlayerSaveData {
    /// Creates a new player save data at the given position.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            ..Default::default()
        }
    }

    /// Sets player health.
    #[must_use]
    pub fn with_health(mut self, health: f32, max_health: f32) -> Self {
        self.health = health;
        self.max_health = max_health;
        self
    }

    /// Adds an item to inventory.
    pub fn add_inventory_item(&mut self, item: ItemStackSaveData) {
        self.inventory.push(item);
    }

    /// Equips an item to a slot.
    pub fn equip_item(&mut self, slot: usize, item: ItemStackSaveData) {
        if slot < self.equipped.len() {
            self.equipped[slot] = Some(item);
        }
    }

    /// Sets faction reputation.
    pub fn set_faction_reputation(&mut self, faction_id: u32, reputation: i32) {
        self.faction_reputation.insert(faction_id, reputation);
    }

    /// Adds an unlocked recipe.
    pub fn unlock_recipe(&mut self, recipe_id: u32) {
        if !self.unlocked_recipes.contains(&recipe_id) {
            self.unlocked_recipes.push(recipe_id);
        }
    }
}

/// Entity save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySaveData {
    /// Entity type identifier
    pub entity_type: u32,
    /// Entity unique ID
    pub entity_id: u64,
    /// Position (x, y)
    pub position: (f32, f32),
    /// Health (if applicable)
    pub health: Option<f32>,
    /// Entity-specific data
    pub data: HashMap<String, Vec<u8>>,
}

impl EntitySaveData {
    /// Creates a new entity save data.
    #[must_use]
    pub fn new(entity_type: u32, entity_id: EntityId, x: f32, y: f32) -> Self {
        Self {
            entity_type,
            entity_id: entity_id.raw(),
            position: (x, y),
            health: None,
            data: HashMap::new(),
        }
    }

    /// Sets entity health.
    #[must_use]
    pub fn with_health(mut self, health: f32) -> Self {
        self.health = Some(health);
        self
    }

    /// Adds custom data.
    pub fn set_data(&mut self, key: impl Into<String>, value: Vec<u8>) {
        self.data.insert(key.into(), value);
    }

    /// Gets custom data.
    #[must_use]
    pub fn get_data(&self, key: &str) -> Option<&Vec<u8>> {
        self.data.get(key)
    }
}

/// Complete game save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGame {
    /// Save format version
    pub version: u32,
    /// Save timestamp (Unix seconds)
    pub timestamp: u64,
    /// Player data
    pub player: PlayerSaveData,
    /// All entities
    pub entities: Vec<EntitySaveData>,
    /// World generation seed
    pub world_seed: u64,
    /// Game time (in-game seconds)
    pub game_time: f64,
    /// Total playtime (real seconds)
    pub playtime: f64,
    /// Save name/description
    pub name: String,
    /// Game difficulty
    pub difficulty: u32,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl Default for SaveGame {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            timestamp: current_timestamp(),
            player: PlayerSaveData::default(),
            entities: Vec::new(),
            world_seed: 0,
            game_time: 0.0,
            playtime: 0.0,
            name: String::new(),
            difficulty: 1,
            metadata: HashMap::new(),
        }
    }
}

impl SaveGame {
    /// Creates a new save game with default values.
    #[must_use]
    pub fn new(name: impl Into<String>, world_seed: u64) -> Self {
        Self {
            name: name.into(),
            world_seed,
            timestamp: current_timestamp(),
            ..Default::default()
        }
    }

    /// Sets the player data.
    #[must_use]
    pub fn with_player(mut self, player: PlayerSaveData) -> Self {
        self.player = player;
        self
    }

    /// Sets game time.
    #[must_use]
    pub fn with_game_time(mut self, game_time: f64) -> Self {
        self.game_time = game_time;
        self
    }

    /// Sets playtime.
    #[must_use]
    pub fn with_playtime(mut self, playtime: f64) -> Self {
        self.playtime = playtime;
        self
    }

    /// Adds an entity.
    pub fn add_entity(&mut self, entity: EntitySaveData) {
        self.entities.push(entity);
    }

    /// Sets metadata.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Gets metadata.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }

    /// Updates timestamp to current time.
    pub fn update_timestamp(&mut self) {
        self.timestamp = current_timestamp();
    }

    /// Serializes to binary format.
    pub fn to_bytes(&self) -> SaveResult<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write magic bytes
        buffer.extend_from_slice(&SAVE_MAGIC);

        // Serialize data
        let data = bincode::serialize(self).map_err(|e| SaveError::Serialization(e.to_string()))?;

        buffer.extend(data);

        Ok(buffer)
    }

    /// Deserializes from binary format.
    pub fn from_bytes(bytes: &[u8]) -> SaveResult<Self> {
        // Check magic bytes
        if bytes.len() < 4 || bytes[0..4] != SAVE_MAGIC {
            return Err(SaveError::InvalidFormat);
        }

        // Deserialize data
        let save: SaveGame =
            bincode::deserialize(&bytes[4..]).map_err(|e| SaveError::Corrupted(e.to_string()))?;

        // Check version
        if save.version > SAVE_VERSION {
            return Err(SaveError::VersionMismatch {
                expected: SAVE_VERSION,
                found: save.version,
            });
        }

        Ok(save)
    }
}

/// Save metadata for listing saves without loading full data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Save file name
    pub name: String,
    /// Save timestamp
    pub timestamp: u64,
    /// Total playtime
    pub playtime: f64,
    /// Save file size in bytes
    pub file_size: u64,
    /// Player name (if set)
    pub player_name: Option<String>,
    /// Game time
    pub game_time: f64,
}

impl SaveMetadata {
    /// Creates metadata from a save game.
    #[must_use]
    pub fn from_save(save: &SaveGame, file_size: u64) -> Self {
        Self {
            name: save.name.clone(),
            timestamp: save.timestamp,
            playtime: save.playtime,
            file_size,
            player_name: save.player.name.clone(),
            game_time: save.game_time,
        }
    }

    /// Returns formatted playtime as HH:MM:SS.
    #[must_use]
    pub fn formatted_playtime(&self) -> String {
        let total_secs = self.playtime as u64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }

    /// Returns formatted game time.
    #[must_use]
    pub fn formatted_game_time(&self) -> String {
        let total_secs = self.game_time as u64;
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        let minutes = (total_secs % 3600) / 60;

        if days > 0 {
            format!("Day {days}, {hours:02}:{minutes:02}")
        } else {
            format!("{hours:02}:{minutes:02}")
        }
    }
}

/// Save manager for handling save files.
#[derive(Debug)]
pub struct SaveManager {
    /// Directory for save files
    save_dir: PathBuf,
}

impl SaveManager {
    /// Creates a new save manager with the given save directory.
    #[must_use]
    pub fn new(save_dir: impl Into<PathBuf>) -> Self {
        Self {
            save_dir: save_dir.into(),
        }
    }

    /// Gets the save directory path.
    #[must_use]
    pub fn save_dir(&self) -> &Path {
        &self.save_dir
    }

    /// Ensures the save directory exists.
    pub fn ensure_dir(&self) -> SaveResult<()> {
        fs::create_dir_all(&self.save_dir)?;
        Ok(())
    }

    /// Gets the path for a save file.
    fn save_path(&self, name: &str) -> PathBuf {
        self.save_dir.join(format!("{name}.sav"))
    }

    /// Gets the path for a temporary save file.
    fn temp_path(&self, name: &str) -> PathBuf {
        self.save_dir.join(format!("{name}.sav.tmp"))
    }

    /// Saves a game to disk.
    ///
    /// Uses atomic write (write to temp, then rename) for safety.
    pub fn save(&self, name: &str, data: &SaveGame) -> SaveResult<()> {
        self.ensure_dir()?;

        let bytes = data.to_bytes()?;
        let temp_path = self.temp_path(name);
        let final_path = self.save_path(name);

        // Write to temp file
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);

        // Atomic rename
        fs::rename(&temp_path, &final_path)?;

        Ok(())
    }

    /// Loads a game from disk.
    pub fn load(&self, name: &str) -> SaveResult<SaveGame> {
        let path = self.save_path(name);

        if !path.exists() {
            return Err(SaveError::NotFound(name.to_string()));
        }

        let mut file = fs::File::open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        SaveGame::from_bytes(&bytes)
    }

    /// Lists all available saves with metadata.
    pub fn list_saves(&self) -> SaveResult<Vec<SaveMetadata>> {
        self.ensure_dir()?;

        let mut saves = Vec::new();

        for entry in fs::read_dir(&self.save_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "sav") {
                if let Ok(save) = self.load(
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default(),
                ) {
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    saves.push(SaveMetadata::from_save(&save, file_size));
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    /// Gets metadata for a specific save without loading full data.
    pub fn get_metadata(&self, name: &str) -> SaveResult<SaveMetadata> {
        let save = self.load(name)?;
        let path = self.save_path(name);
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Ok(SaveMetadata::from_save(&save, file_size))
    }

    /// Deletes a save file.
    pub fn delete(&self, name: &str) -> SaveResult<()> {
        let path = self.save_path(name);

        if !path.exists() {
            return Err(SaveError::NotFound(name.to_string()));
        }

        fs::remove_file(&path)?;

        Ok(())
    }

    /// Checks if a save exists.
    #[must_use]
    pub fn exists(&self, name: &str) -> bool {
        self.save_path(name).exists()
    }

    /// Creates a backup of an existing save.
    pub fn backup(&self, name: &str) -> SaveResult<String> {
        let timestamp = current_timestamp();
        let backup_name = format!("{name}_backup_{timestamp}");

        let data = self.load(name)?;
        self.save(&backup_name, &data)?;

        Ok(backup_name)
    }

    /// Imports a save from bytes.
    pub fn import(&self, name: &str, bytes: &[u8]) -> SaveResult<()> {
        // Validate the save first
        let _ = SaveGame::from_bytes(bytes)?;

        self.ensure_dir()?;
        let path = self.save_path(name);

        fs::write(&path, bytes)?;

        Ok(())
    }

    /// Exports a save to bytes.
    pub fn export(&self, name: &str) -> SaveResult<Vec<u8>> {
        let path = self.save_path(name);

        if !path.exists() {
            return Err(SaveError::NotFound(name.to_string()));
        }

        let bytes = fs::read(&path)?;
        Ok(bytes)
    }
}

/// Returns current Unix timestamp.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_save_dir() -> PathBuf {
        let unique_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        env::temp_dir().join(format!(
            "genesis_save_test_{}_{unique_id}",
            current_timestamp()
        ))
    }

    #[test]
    fn test_item_stack_save_data() {
        let item = ItemStackSaveData::new(ItemTypeId::new(42), 10);
        assert_eq!(item.item_type, 42);
        assert_eq!(item.quantity, 10);
        assert!(item.custom_data.is_none());

        let item_with_data = item.with_custom_data(vec![1, 2, 3]);
        assert_eq!(item_with_data.custom_data, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_player_save_data() {
        let mut player = PlayerSaveData::new(100.0, 200.0);
        assert_eq!(player.position, (100.0, 200.0));
        assert_eq!(player.health, 100.0);

        player = player.with_health(50.0, 100.0);
        assert_eq!(player.health, 50.0);
        assert_eq!(player.max_health, 100.0);

        player.add_inventory_item(ItemStackSaveData::new(ItemTypeId::new(1), 5));
        assert_eq!(player.inventory.len(), 1);

        player.equip_item(0, ItemStackSaveData::new(ItemTypeId::new(10), 1));
        assert!(player.equipped[0].is_some());

        player.set_faction_reputation(1, 100);
        assert_eq!(player.faction_reputation.get(&1), Some(&100));

        player.unlock_recipe(1);
        player.unlock_recipe(1); // Duplicate
        assert_eq!(player.unlocked_recipes.len(), 1);
    }

    #[test]
    fn test_entity_save_data() {
        let mut entity = EntitySaveData::new(1, EntityId::new(), 50.0, 75.0);
        assert_eq!(entity.entity_type, 1);
        assert_eq!(entity.position, (50.0, 75.0));

        entity = entity.with_health(100.0);
        assert_eq!(entity.health, Some(100.0));

        entity.set_data("custom", vec![1, 2, 3]);
        assert_eq!(entity.get_data("custom"), Some(&vec![1, 2, 3]));
    }

    #[test]
    fn test_save_game_creation() {
        let save = SaveGame::new("Test Save", 12345);
        assert_eq!(save.name, "Test Save");
        assert_eq!(save.world_seed, 12345);
        assert_eq!(save.version, SAVE_VERSION);
    }

    #[test]
    fn test_save_game_with_player() {
        let player = PlayerSaveData::new(100.0, 200.0);
        let save = SaveGame::new("Test", 0).with_player(player);

        assert_eq!(save.player.position, (100.0, 200.0));
    }

    #[test]
    fn test_save_game_entities() {
        let mut save = SaveGame::new("Test", 0);
        save.add_entity(EntitySaveData::new(1, EntityId::new(), 0.0, 0.0));
        save.add_entity(EntitySaveData::new(2, EntityId::new(), 10.0, 10.0));

        assert_eq!(save.entities.len(), 2);
    }

    #[test]
    fn test_save_game_metadata() {
        let mut save = SaveGame::new("Test", 0);
        save.set_metadata("version", "1.0.0");
        save.set_metadata("mod_list", "mod1,mod2");

        assert_eq!(save.get_metadata("version"), Some("1.0.0"));
        assert_eq!(save.get_metadata("mod_list"), Some("mod1,mod2"));
        assert_eq!(save.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_save_game_serialization() {
        let mut save = SaveGame::new("Test Save", 12345);
        save.player = PlayerSaveData::new(100.0, 200.0);
        save.add_entity(EntitySaveData::new(1, EntityId::new(), 50.0, 50.0));

        let bytes = save.to_bytes().expect("serialization should succeed");
        assert!(!bytes.is_empty());

        // Check magic bytes
        assert_eq!(&bytes[0..4], &SAVE_MAGIC);

        let loaded = SaveGame::from_bytes(&bytes).expect("deserialization should succeed");
        assert_eq!(loaded.name, "Test Save");
        assert_eq!(loaded.world_seed, 12345);
        assert_eq!(loaded.player.position, (100.0, 200.0));
        assert_eq!(loaded.entities.len(), 1);
    }

    #[test]
    fn test_save_game_invalid_format() {
        let bad_bytes = vec![0, 1, 2, 3, 4, 5];
        let result = SaveGame::from_bytes(&bad_bytes);
        assert!(matches!(result, Err(SaveError::InvalidFormat)));
    }

    #[test]
    fn test_save_metadata() {
        let save = SaveGame::new("Test", 0)
            .with_game_time(3665.0) // 1 hour, 1 minute, 5 seconds
            .with_playtime(7325.0); // 2 hours, 2 minutes, 5 seconds

        let meta = SaveMetadata::from_save(&save, 1024);

        assert_eq!(meta.name, "Test");
        assert_eq!(meta.file_size, 1024);
        assert_eq!(meta.formatted_playtime(), "02:02:05");
        assert_eq!(meta.formatted_game_time(), "01:01");
    }

    #[test]
    fn test_save_metadata_multi_day() {
        let save = SaveGame::new("Test", 0).with_game_time(90061.0); // Day 1, 01:01

        let meta = SaveMetadata::from_save(&save, 0);
        assert_eq!(meta.formatted_game_time(), "Day 1, 01:01");
    }

    #[test]
    fn test_save_manager_save_load() {
        let dir = temp_save_dir();
        let manager = SaveManager::new(&dir);

        let save = SaveGame::new("Test Save", 12345)
            .with_player(PlayerSaveData::new(100.0, 200.0))
            .with_game_time(1000.0);

        manager.save("test1", &save).expect("save should succeed");
        assert!(manager.exists("test1"));

        let loaded = manager.load("test1").expect("load should succeed");
        assert_eq!(loaded.name, "Test Save");
        assert_eq!(loaded.world_seed, 12345);
        assert_eq!(loaded.player.position, (100.0, 200.0));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_manager_not_found() {
        let dir = temp_save_dir();
        let manager = SaveManager::new(&dir);

        let result = manager.load("nonexistent");
        assert!(matches!(result, Err(SaveError::NotFound(_))));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_manager_delete() {
        let dir = temp_save_dir();
        let manager = SaveManager::new(&dir);

        let save = SaveGame::new("Test", 0);
        manager
            .save("to_delete", &save)
            .expect("save should succeed");
        assert!(manager.exists("to_delete"));

        manager.delete("to_delete").expect("delete should succeed");
        assert!(!manager.exists("to_delete"));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_manager_list_saves() {
        let dir = temp_save_dir();
        let manager = SaveManager::new(&dir);

        let save1 = SaveGame::new("Save 1", 1);
        let save2 = SaveGame::new("Save 2", 2);

        manager.save("save1", &save1).expect("save should succeed");
        manager.save("save2", &save2).expect("save should succeed");

        let saves = manager.list_saves().expect("list should succeed");
        assert_eq!(saves.len(), 2);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_manager_backup() {
        let dir = temp_save_dir();
        let manager = SaveManager::new(&dir);

        let save = SaveGame::new("Original", 12345);
        manager
            .save("original", &save)
            .expect("save should succeed");

        let backup_name = manager.backup("original").expect("backup should succeed");
        assert!(manager.exists(&backup_name));

        let backup = manager.load(&backup_name).expect("load should succeed");
        assert_eq!(backup.name, "Original");
        assert_eq!(backup.world_seed, 12345);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_manager_export_import() {
        let dir = temp_save_dir();
        let manager = SaveManager::new(&dir);

        let save = SaveGame::new("Export Test", 99999);
        manager
            .save("export_test", &save)
            .expect("save should succeed");

        let bytes = manager
            .export("export_test")
            .expect("export should succeed");

        manager
            .delete("export_test")
            .expect("delete should succeed");
        assert!(!manager.exists("export_test"));

        manager
            .import("imported", &bytes)
            .expect("import should succeed");
        assert!(manager.exists("imported"));

        let imported = manager.load("imported").expect("load should succeed");
        assert_eq!(imported.name, "Export Test");

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_manager_get_metadata() {
        let dir = temp_save_dir();
        let manager = SaveManager::new(&dir);

        let mut save = SaveGame::new("Metadata Test", 0);
        save.player.name = Some("Hero".to_string());
        save.playtime = 3600.0;

        manager
            .save("meta_test", &save)
            .expect("save should succeed");

        let meta = manager
            .get_metadata("meta_test")
            .expect("get_metadata should succeed");
        assert_eq!(meta.name, "Metadata Test");
        assert_eq!(meta.player_name, Some("Hero".to_string()));
        assert_eq!(meta.playtime, 3600.0);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }
}
