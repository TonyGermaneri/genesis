//! Crafting persistence system.
//!
//! This module provides:
//! - Saving learned recipes to player save
//! - Persisting workbench contents on exit
//! - Loading crafting state on game load
//! - Migration for recipe format changes

use genesis_common::{ItemTypeId, RecipeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use tracing::{debug, info};

use crate::crafting_events::CraftingStats;

/// Current crafting save format version.
pub const CRAFTING_SAVE_VERSION: u32 = 1;

/// Errors that can occur during crafting persistence.
#[derive(Debug, Error)]
pub enum CraftingSaveError {
    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Version mismatch.
    #[error("Incompatible crafting save version: expected {expected}, found {found}")]
    VersionMismatch {
        /// Expected version.
        expected: u32,
        /// Found version.
        found: u32,
    },

    /// Corrupted data.
    #[error("Corrupted crafting data: {0}")]
    Corrupted(String),

    /// Migration failed.
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

/// Result type for crafting save operations.
pub type CraftingSaveResult<T> = Result<T, CraftingSaveError>;

/// Item stack for workbench contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkbenchItemStack {
    /// Item type ID.
    pub item_id: u32,
    /// Quantity.
    pub quantity: u32,
}

impl WorkbenchItemStack {
    /// Creates a new item stack.
    #[must_use]
    pub const fn new(item_id: u32, quantity: u32) -> Self {
        Self { item_id, quantity }
    }

    /// Gets the item type ID.
    #[must_use]
    pub fn item_type(&self) -> ItemTypeId {
        ItemTypeId::new(self.item_id)
    }
}

/// Saved workbench state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkbenchSaveData {
    /// Workbench building ID.
    pub building_id: u32,
    /// World position (x, y).
    pub position: (i64, i64),
    /// Input items in the workbench.
    pub inputs: Vec<WorkbenchItemStack>,
    /// Output items waiting to be collected.
    pub outputs: Vec<WorkbenchItemStack>,
    /// Recipe currently being crafted (if any).
    pub current_recipe: Option<u32>,
    /// Progress on current recipe (0.0 - 1.0).
    pub craft_progress: f32,
    /// Fuel remaining (for fuel-based workstations).
    pub fuel_remaining: f32,
}

impl WorkbenchSaveData {
    /// Creates a new empty workbench save.
    #[must_use]
    pub fn new(building_id: u32, position: (i64, i64)) -> Self {
        Self {
            building_id,
            position,
            inputs: Vec::new(),
            outputs: Vec::new(),
            current_recipe: None,
            craft_progress: 0.0,
            fuel_remaining: 0.0,
        }
    }

    /// Adds an input item.
    pub fn add_input(&mut self, item_id: u32, quantity: u32) {
        // Try to stack with existing
        for stack in &mut self.inputs {
            if stack.item_id == item_id {
                stack.quantity += quantity;
                return;
            }
        }
        self.inputs.push(WorkbenchItemStack::new(item_id, quantity));
    }

    /// Adds an output item.
    pub fn add_output(&mut self, item_id: u32, quantity: u32) {
        for stack in &mut self.outputs {
            if stack.item_id == item_id {
                stack.quantity += quantity;
                return;
            }
        }
        self.outputs
            .push(WorkbenchItemStack::new(item_id, quantity));
    }

    /// Returns true if the workbench has any contents.
    #[must_use]
    pub fn has_contents(&self) -> bool {
        !self.inputs.is_empty() || !self.outputs.is_empty() || self.current_recipe.is_some()
    }
}

/// Saved crafting statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CraftingStatsSaveData {
    /// Total items crafted.
    pub items_crafted: u64,
    /// Total crafts completed.
    pub crafts_completed: u64,
    /// Total crafts failed.
    pub crafts_failed: u64,
    /// Total skill gained from crafting.
    pub skill_gained: u64,
    /// Crafts per recipe ID.
    pub crafts_by_recipe: HashMap<u32, u32>,
}

impl From<&CraftingStats> for CraftingStatsSaveData {
    fn from(stats: &CraftingStats) -> Self {
        Self {
            items_crafted: stats.items_crafted,
            crafts_completed: stats.crafts_completed,
            crafts_failed: stats.crafts_failed,
            skill_gained: stats.skill_gained,
            crafts_by_recipe: stats.crafts_by_recipe.clone(),
        }
    }
}

impl CraftingStatsSaveData {
    /// Restores to a CraftingStats instance.
    #[must_use]
    pub fn restore(&self) -> CraftingStats {
        CraftingStats {
            items_crafted: self.items_crafted,
            crafts_completed: self.crafts_completed,
            crafts_failed: self.crafts_failed,
            skill_gained: self.skill_gained,
            crafts_by_recipe: self.crafts_by_recipe.clone(),
        }
    }
}

/// Complete crafting save data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingSaveData {
    /// Save format version.
    pub version: u32,
    /// Learned recipe IDs.
    pub learned_recipes: Vec<u32>,
    /// Crafting statistics.
    pub stats: CraftingStatsSaveData,
    /// Workbench contents by position key.
    pub workbenches: HashMap<String, WorkbenchSaveData>,
    /// Crafting skill levels by skill name.
    pub skill_levels: HashMap<String, u32>,
    /// Recipe favorites.
    pub favorite_recipes: Vec<u32>,
    /// Recently crafted recipes (for quick access).
    pub recent_recipes: Vec<u32>,
}

impl Default for CraftingSaveData {
    fn default() -> Self {
        Self {
            version: CRAFTING_SAVE_VERSION,
            learned_recipes: Vec::new(),
            stats: CraftingStatsSaveData::default(),
            workbenches: HashMap::new(),
            skill_levels: HashMap::new(),
            favorite_recipes: Vec::new(),
            recent_recipes: Vec::new(),
        }
    }
}

impl CraftingSaveData {
    /// Creates a new empty crafting save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a learned recipe.
    pub fn learn_recipe(&mut self, recipe_id: u32) {
        if !self.learned_recipes.contains(&recipe_id) {
            self.learned_recipes.push(recipe_id);
            debug!("Learned recipe: {}", recipe_id);
        }
    }

    /// Checks if a recipe is learned.
    #[must_use]
    pub fn is_recipe_learned(&self, recipe_id: u32) -> bool {
        self.learned_recipes.contains(&recipe_id)
    }

    /// Adds a recipe to favorites.
    pub fn add_favorite(&mut self, recipe_id: u32) {
        if !self.favorite_recipes.contains(&recipe_id) {
            self.favorite_recipes.push(recipe_id);
        }
    }

    /// Removes a recipe from favorites.
    pub fn remove_favorite(&mut self, recipe_id: u32) {
        self.favorite_recipes.retain(|&id| id != recipe_id);
    }

    /// Adds a recipe to recent list.
    pub fn add_recent(&mut self, recipe_id: u32) {
        // Remove if already present
        self.recent_recipes.retain(|&id| id != recipe_id);
        // Add to front
        self.recent_recipes.insert(0, recipe_id);
        // Limit to 10 recent
        self.recent_recipes.truncate(10);
    }

    /// Saves a workbench state.
    pub fn save_workbench(&mut self, workbench: WorkbenchSaveData) {
        let key = workbench_key(workbench.position.0, workbench.position.1);
        if workbench.has_contents() {
            self.workbenches.insert(key, workbench);
        } else {
            self.workbenches.remove(&key);
        }
    }

    /// Gets a workbench state by position.
    #[must_use]
    pub fn get_workbench(&self, x: i64, y: i64) -> Option<&WorkbenchSaveData> {
        let key = workbench_key(x, y);
        self.workbenches.get(&key)
    }

    /// Sets a skill level.
    pub fn set_skill_level(&mut self, skill: impl Into<String>, level: u32) {
        self.skill_levels.insert(skill.into(), level);
    }

    /// Gets a skill level.
    #[must_use]
    pub fn get_skill_level(&self, skill: &str) -> u32 {
        self.skill_levels.get(skill).copied().unwrap_or(0)
    }

    /// Migrates save data from an older version.
    pub fn migrate(self) -> CraftingSaveResult<Self> {
        if self.version == CRAFTING_SAVE_VERSION {
            return Ok(self);
        }

        info!(
            "Migrating crafting save from v{} to v{}",
            self.version, CRAFTING_SAVE_VERSION
        );

        // Future migrations would go here
        // Example:
        // if self.version == 0 {
        //     // Migrate from v0 to v1
        //     self.version = 1;
        // }

        if self.version != CRAFTING_SAVE_VERSION {
            return Err(CraftingSaveError::VersionMismatch {
                expected: CRAFTING_SAVE_VERSION,
                found: self.version,
            });
        }

        Ok(self)
    }

    /// Serializes to JSON.
    pub fn to_json(&self) -> CraftingSaveResult<String> {
        serde_json::to_string(self).map_err(|e| CraftingSaveError::Serialization(e.to_string()))
    }

    /// Serializes to JSON (pretty).
    pub fn to_json_pretty(&self) -> CraftingSaveResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CraftingSaveError::Serialization(e.to_string()))
    }

    /// Deserializes from JSON.
    pub fn from_json(json: &str) -> CraftingSaveResult<Self> {
        let data: Self = serde_json::from_str(json)
            .map_err(|e| CraftingSaveError::Serialization(e.to_string()))?;
        data.migrate()
    }

    /// Serializes to binary (MessagePack-style compact format).
    pub fn to_bytes(&self) -> CraftingSaveResult<Vec<u8>> {
        // Use JSON for now, could use bincode/messagepack later
        let json = self.to_json()?;
        Ok(json.into_bytes())
    }

    /// Deserializes from binary.
    pub fn from_bytes(bytes: &[u8]) -> CraftingSaveResult<Self> {
        let json =
            std::str::from_utf8(bytes).map_err(|e| CraftingSaveError::Corrupted(e.to_string()))?;
        Self::from_json(json)
    }
}

/// Generates a unique key for a workbench position.
fn workbench_key(x: i64, y: i64) -> String {
    format!("{x},{y}")
}

/// Manager for crafting persistence.
pub struct CraftingPersistence {
    /// Current crafting state.
    data: CraftingSaveData,
    /// Recipes that start unlocked.
    starter_recipes: HashSet<u32>,
    /// Whether there are unsaved changes.
    dirty: bool,
}

impl Default for CraftingPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl CraftingPersistence {
    /// Creates a new persistence manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: CraftingSaveData::new(),
            starter_recipes: HashSet::new(),
            dirty: false,
        }
    }

    /// Creates with starter recipes.
    #[must_use]
    pub fn with_starter_recipes(starter_recipes: impl IntoIterator<Item = u32>) -> Self {
        let starter_recipes: HashSet<u32> = starter_recipes.into_iter().collect();
        let mut data = CraftingSaveData::new();

        // Learn all starter recipes
        for &recipe_id in &starter_recipes {
            data.learn_recipe(recipe_id);
        }

        Self {
            data,
            starter_recipes,
            dirty: false,
        }
    }

    /// Loads from save data.
    pub fn load(&mut self, data: CraftingSaveData) -> CraftingSaveResult<()> {
        self.data = data.migrate()?;

        // Ensure starter recipes are learned
        for &recipe_id in &self.starter_recipes {
            self.data.learn_recipe(recipe_id);
        }

        self.dirty = false;
        info!(
            "Loaded crafting state: {} recipes learned",
            self.data.learned_recipes.len()
        );
        Ok(())
    }

    /// Returns the current save data.
    #[must_use]
    pub fn save_data(&self) -> &CraftingSaveData {
        &self.data
    }

    /// Creates a copy of save data for saving.
    #[must_use]
    pub fn to_save_data(&self) -> CraftingSaveData {
        self.data.clone()
    }

    /// Loads save data from a save file.
    pub fn load_data(&mut self, data: &CraftingSaveData) {
        self.data = data.clone();
        // Re-learn starter recipes (ensure they're always available)
        for &recipe_id in &self.starter_recipes {
            self.data.learn_recipe(recipe_id);
        }
        self.dirty = false;
    }

    /// Returns whether there are unsaved changes.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks changes as saved.
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    /// Learns a recipe.
    pub fn learn_recipe(&mut self, recipe_id: RecipeId) {
        if !self.data.is_recipe_learned(recipe_id.raw()) {
            self.data.learn_recipe(recipe_id.raw());
            self.dirty = true;
        }
    }

    /// Checks if a recipe is learned.
    #[must_use]
    pub fn is_recipe_learned(&self, recipe_id: RecipeId) -> bool {
        self.data.is_recipe_learned(recipe_id.raw())
    }

    /// Returns all learned recipe IDs.
    #[must_use]
    pub fn learned_recipes(&self) -> &[u32] {
        &self.data.learned_recipes
    }

    /// Saves a workbench state.
    pub fn save_workbench(&mut self, workbench: WorkbenchSaveData) {
        self.data.save_workbench(workbench);
        self.dirty = true;
    }

    /// Gets a workbench state.
    #[must_use]
    pub fn get_workbench(&self, x: i64, y: i64) -> Option<&WorkbenchSaveData> {
        self.data.get_workbench(x, y)
    }

    /// Returns all workbenches with contents.
    pub fn workbenches(&self) -> impl Iterator<Item = &WorkbenchSaveData> {
        self.data.workbenches.values()
    }

    /// Updates crafting statistics.
    pub fn update_stats(&mut self, stats: &CraftingStats) {
        self.data.stats = CraftingStatsSaveData::from(stats);
        self.dirty = true;
    }

    /// Gets crafting statistics.
    #[must_use]
    pub fn stats(&self) -> &CraftingStatsSaveData {
        &self.data.stats
    }

    /// Gets a skill level.
    #[must_use]
    pub fn get_skill_level(&self, skill: &str) -> u32 {
        self.data.get_skill_level(skill)
    }

    /// Sets a skill level.
    pub fn set_skill_level(&mut self, skill: impl Into<String>, level: u32) {
        self.data.set_skill_level(skill, level);
        self.dirty = true;
    }

    /// Adds a recipe to recent.
    pub fn add_recent(&mut self, recipe_id: RecipeId) {
        self.data.add_recent(recipe_id.raw());
        self.dirty = true;
    }

    /// Returns recent recipes.
    #[must_use]
    pub fn recent_recipes(&self) -> &[u32] {
        &self.data.recent_recipes
    }

    /// Toggles favorite status for a recipe.
    pub fn toggle_favorite(&mut self, recipe_id: RecipeId) {
        let id = recipe_id.raw();
        if self.data.favorite_recipes.contains(&id) {
            self.data.remove_favorite(id);
        } else {
            self.data.add_favorite(id);
        }
        self.dirty = true;
    }

    /// Checks if a recipe is favorited.
    #[must_use]
    pub fn is_favorite(&self, recipe_id: RecipeId) -> bool {
        self.data.favorite_recipes.contains(&recipe_id.raw())
    }

    /// Returns favorite recipes.
    #[must_use]
    pub fn favorite_recipes(&self) -> &[u32] {
        &self.data.favorite_recipes
    }

    /// Resets to initial state (keeps starter recipes).
    pub fn reset(&mut self) {
        self.data = CraftingSaveData::new();
        for &recipe_id in &self.starter_recipes {
            self.data.learn_recipe(recipe_id);
        }
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workbench_save_data() {
        let mut wb = WorkbenchSaveData::new(100, (10, 20));
        assert!(!wb.has_contents());

        wb.add_input(1, 5);
        assert!(wb.has_contents());

        wb.add_input(1, 3);
        assert_eq!(wb.inputs.len(), 1);
        assert_eq!(wb.inputs[0].quantity, 8);

        wb.add_input(2, 2);
        assert_eq!(wb.inputs.len(), 2);
    }

    #[test]
    fn test_crafting_save_data_recipes() {
        let mut data = CraftingSaveData::new();

        assert!(!data.is_recipe_learned(1));
        data.learn_recipe(1);
        assert!(data.is_recipe_learned(1));

        // Learning again doesn't duplicate
        data.learn_recipe(1);
        assert_eq!(data.learned_recipes.len(), 1);
    }

    #[test]
    fn test_crafting_save_data_favorites() {
        let mut data = CraftingSaveData::new();

        data.add_favorite(1);
        data.add_favorite(2);
        assert_eq!(data.favorite_recipes.len(), 2);

        data.remove_favorite(1);
        assert_eq!(data.favorite_recipes.len(), 1);
        assert!(!data.favorite_recipes.contains(&1));
    }

    #[test]
    fn test_crafting_save_data_recent() {
        let mut data = CraftingSaveData::new();

        for i in 1..=15 {
            data.add_recent(i);
        }

        // Should be limited to 10
        assert_eq!(data.recent_recipes.len(), 10);
        // Most recent should be first
        assert_eq!(data.recent_recipes[0], 15);
    }

    #[test]
    fn test_crafting_save_data_serialization() {
        let mut data = CraftingSaveData::new();
        data.learn_recipe(1);
        data.learn_recipe(2);
        data.set_skill_level("crafting", 5);

        let json = data.to_json().expect("serialize");
        let restored = CraftingSaveData::from_json(&json).expect("deserialize");

        assert_eq!(restored.learned_recipes, data.learned_recipes);
        assert_eq!(restored.get_skill_level("crafting"), 5);
    }

    #[test]
    fn test_crafting_persistence_starter_recipes() {
        let persistence = CraftingPersistence::with_starter_recipes([1, 2, 3]);

        assert!(persistence.is_recipe_learned(RecipeId::new(1)));
        assert!(persistence.is_recipe_learned(RecipeId::new(2)));
        assert!(persistence.is_recipe_learned(RecipeId::new(3)));
        assert!(!persistence.is_recipe_learned(RecipeId::new(4)));
    }

    #[test]
    fn test_crafting_persistence_dirty_tracking() {
        let mut persistence = CraftingPersistence::new();
        assert!(!persistence.is_dirty());

        persistence.learn_recipe(RecipeId::new(1));
        assert!(persistence.is_dirty());

        persistence.mark_saved();
        assert!(!persistence.is_dirty());

        persistence.set_skill_level("crafting", 5);
        assert!(persistence.is_dirty());
    }

    #[test]
    fn test_crafting_persistence_reset() {
        let mut persistence = CraftingPersistence::with_starter_recipes([1, 2]);

        persistence.learn_recipe(RecipeId::new(3));
        persistence.learn_recipe(RecipeId::new(4));
        assert_eq!(persistence.learned_recipes().len(), 4);

        persistence.reset();
        assert_eq!(persistence.learned_recipes().len(), 2);
        assert!(persistence.is_recipe_learned(RecipeId::new(1)));
        assert!(persistence.is_recipe_learned(RecipeId::new(2)));
        assert!(!persistence.is_recipe_learned(RecipeId::new(3)));
    }

    #[test]
    fn test_workbench_key() {
        assert_eq!(workbench_key(10, 20), "10,20");
        assert_eq!(workbench_key(-5, -10), "-5,-10");
    }

    #[test]
    fn test_stats_roundtrip() {
        let mut stats = CraftingStats::new();
        stats.record_craft(RecipeId::new(1), 5, 10);
        stats.record_craft(RecipeId::new(2), 3, 5);
        stats.record_failure();

        let save_data = CraftingStatsSaveData::from(&stats);
        let restored = save_data.restore();

        assert_eq!(restored.items_crafted, stats.items_crafted);
        assert_eq!(restored.crafts_completed, stats.crafts_completed);
        assert_eq!(restored.crafts_failed, stats.crafts_failed);
        assert_eq!(restored.skill_gained, stats.skill_gained);
    }
}
