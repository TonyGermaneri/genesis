//! Recipe asset loading and management.
//!
//! This module provides:
//! - Loading recipes from assets/recipes/*.toml
//! - Recipe validation on load
//! - Hot-reload support for development
//! - Recipe registry with fast lookup by ID, name, and category

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use genesis_common::{ItemTypeId, RecipeId};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Default asset path for recipes.
pub const DEFAULT_RECIPE_PATH: &str = "assets/recipes";

/// Errors that can occur during recipe loading.
#[derive(Debug, Error)]
pub enum RecipeLoadError {
    /// File not found.
    #[error("Recipe file not found: {0}")]
    NotFound(PathBuf),

    /// Failed to read file.
    #[error("Failed to read recipe file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse TOML.
    #[error("Failed to parse recipe TOML: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Validation error.
    #[error("Recipe validation error: {0}")]
    ValidationError(String),

    /// Duplicate recipe ID.
    #[error("Duplicate recipe ID: {0}")]
    DuplicateId(u32),
}

/// Result type for recipe loading operations.
pub type RecipeLoadResult<T> = Result<T, RecipeLoadError>;

/// A recipe ingredient from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    /// Item type ID.
    pub item_id: u32,
    /// Quantity required.
    pub quantity: u32,
}

/// Recipe output definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOutput {
    /// Item type ID produced.
    pub item_id: u32,
    /// Quantity produced.
    pub quantity: u32,
    /// Quality variance (0.0-1.0).
    #[serde(default)]
    pub quality_variance: f32,
}

/// Unlock requirement for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UnlockRequirement {
    /// Requires a skill level.
    #[serde(rename = "skill")]
    Skill {
        /// Skill name.
        skill: String,
        /// Required level.
        level: u32,
    },
    /// Requires another recipe to be learned.
    #[serde(rename = "recipe")]
    Recipe {
        /// Recipe ID.
        recipe_id: u32,
    },
    /// Requires a quest to be completed.
    #[serde(rename = "quest")]
    Quest {
        /// Quest ID.
        quest_id: u32,
    },
    /// Requires an achievement.
    #[serde(rename = "achievement")]
    Achievement {
        /// Achievement ID.
        achievement_id: u32,
    },
}

/// A recipe definition loaded from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDefinition {
    /// Unique recipe identifier.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Recipe description.
    #[serde(default)]
    pub description: String,
    /// Recipe category for UI grouping.
    #[serde(default = "default_category")]
    pub category: String,
    /// Required ingredients (consumed).
    #[serde(default)]
    pub ingredients: Vec<RecipeIngredient>,
    /// Required tools (not consumed).
    #[serde(default)]
    pub tools: Vec<u32>,
    /// Required workstation building ID (None = handcraft).
    #[serde(default)]
    pub workstation: Option<u32>,
    /// Minimum skill level required.
    #[serde(default)]
    pub skill_required: u32,
    /// Skill category.
    #[serde(default = "default_skill_type")]
    pub skill_type: String,
    /// Output item and quantity.
    pub output: RecipeOutput,
    /// Optional secondary outputs.
    #[serde(default)]
    pub byproducts: Vec<RecipeOutput>,
    /// Time to craft in game ticks.
    #[serde(default = "default_craft_time")]
    pub craft_time_ticks: u32,
    /// Skill XP gained on successful craft.
    #[serde(default)]
    pub skill_gain: u32,
    /// Requirements to learn this recipe.
    #[serde(default)]
    pub unlock_requirements: Vec<UnlockRequirement>,
    /// Mod that added this recipe (None = core).
    #[serde(default)]
    pub mod_id: Option<String>,
}

fn default_category() -> String {
    "misc".to_string()
}

fn default_skill_type() -> String {
    "crafting".to_string()
}

const fn default_craft_time() -> u32 {
    60 // 1 second at 60 ticks/second
}

impl RecipeDefinition {
    /// Validates the recipe definition.
    pub fn validate(&self) -> RecipeLoadResult<()> {
        if self.name.is_empty() {
            return Err(RecipeLoadError::ValidationError(format!(
                "Recipe {} has empty name",
                self.id
            )));
        }

        if self.output.quantity == 0 {
            return Err(RecipeLoadError::ValidationError(format!(
                "Recipe {} has zero output quantity",
                self.id
            )));
        }

        for (i, ingredient) in self.ingredients.iter().enumerate() {
            if ingredient.quantity == 0 {
                return Err(RecipeLoadError::ValidationError(format!(
                    "Recipe {} ingredient {} has zero quantity",
                    self.id, i
                )));
            }
        }

        if self.output.quality_variance < 0.0 || self.output.quality_variance > 1.0 {
            return Err(RecipeLoadError::ValidationError(format!(
                "Recipe {} has invalid quality_variance: {}",
                self.id, self.output.quality_variance
            )));
        }

        Ok(())
    }

    /// Converts to gameplay Recipe.
    #[must_use]
    pub fn to_gameplay_recipe(&self) -> genesis_gameplay::crafting::Recipe {
        let mut builder =
            genesis_gameplay::crafting::Recipe::builder(RecipeId::new(self.id), &self.name);

        for ingredient in &self.ingredients {
            builder = builder.ingredient(ItemTypeId::new(ingredient.item_id), ingredient.quantity);
        }

        for tool_id in &self.tools {
            builder = builder.tool(ItemTypeId::new(*tool_id));
        }

        builder = builder
            .output(ItemTypeId::new(self.output.item_id), self.output.quantity)
            .skill_required(self.skill_required)
            .skill_gain(self.skill_gain)
            .craft_time(self.craft_time_ticks);

        builder.build()
    }
}

/// A collection of recipes from a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFile {
    /// File format version.
    #[serde(default = "default_version")]
    pub version: String,
    /// Recipes in this file.
    pub recipes: Vec<RecipeDefinition>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Recipe registry with fast lookup.
pub struct RecipeRegistry {
    /// Recipes by ID.
    by_id: HashMap<u32, RecipeDefinition>,
    /// Recipe IDs by name (lowercase).
    by_name: HashMap<String, u32>,
    /// Recipe IDs by category.
    by_category: HashMap<String, Vec<u32>>,
    /// All categories.
    categories: Vec<String>,
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_name: HashMap::new(),
            by_category: HashMap::new(),
            categories: Vec::new(),
        }
    }

    /// Returns the number of registered recipes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns true if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Registers a recipe.
    pub fn register(&mut self, recipe: RecipeDefinition) -> RecipeLoadResult<()> {
        if self.by_id.contains_key(&recipe.id) {
            return Err(RecipeLoadError::DuplicateId(recipe.id));
        }

        let id = recipe.id;
        let name_lower = recipe.name.to_lowercase();
        let category = recipe.category.clone();

        // Add to category index
        let category_list = self.by_category.entry(category.clone()).or_default();
        category_list.push(id);

        // Track categories
        if !self.categories.contains(&category) {
            self.categories.push(category);
        }

        // Add to name index
        self.by_name.insert(name_lower, id);

        // Add to ID map
        self.by_id.insert(id, recipe);

        Ok(())
    }

    /// Gets a recipe by ID.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<&RecipeDefinition> {
        self.by_id.get(&id)
    }

    /// Gets a recipe by name (case-insensitive).
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&RecipeDefinition> {
        self.by_name
            .get(&name.to_lowercase())
            .and_then(|id| self.by_id.get(id))
    }

    /// Gets all recipes in a category.
    #[must_use]
    pub fn get_by_category(&self, category: &str) -> Vec<&RecipeDefinition> {
        self.by_category
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.by_id.get(id)).collect())
            .unwrap_or_default()
    }

    /// Returns all categories.
    #[must_use]
    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    /// Returns an iterator over all recipes.
    pub fn iter(&self) -> impl Iterator<Item = &RecipeDefinition> {
        self.by_id.values()
    }

    /// Searches recipes by name substring (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&RecipeDefinition> {
        let query_lower = query.to_lowercase();
        self.by_id
            .values()
            .filter(|r| r.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Clears the registry.
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_name.clear();
        self.by_category.clear();
        self.categories.clear();
    }
}

/// Recipe asset loader with hot-reload support.
pub struct RecipeLoader {
    /// Base path for recipe files.
    base_path: PathBuf,
    /// Recipe registry.
    registry: RecipeRegistry,
    /// Modification times for hot-reload detection.
    mod_times: HashMap<PathBuf, SystemTime>,
    /// Whether hot-reload is enabled.
    hot_reload_enabled: bool,
    /// Statistics.
    stats: RecipeLoaderStats,
}

/// Statistics for the recipe loader.
#[derive(Debug, Default, Clone)]
pub struct RecipeLoaderStats {
    /// Number of files loaded.
    pub files_loaded: u32,
    /// Number of recipes loaded.
    pub recipes_loaded: u32,
    /// Number of validation errors.
    pub validation_errors: u32,
    /// Number of hot-reloads performed.
    pub hot_reloads: u32,
}

impl RecipeLoader {
    /// Creates a new recipe loader.
    #[must_use]
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let base_path = base_path.into();
        info!("Initializing recipe loader at: {:?}", base_path);

        Self {
            base_path,
            registry: RecipeRegistry::new(),
            mod_times: HashMap::new(),
            hot_reload_enabled: cfg!(debug_assertions),
            stats: RecipeLoaderStats::default(),
        }
    }

    /// Creates a loader with default path.
    #[must_use]
    pub fn with_default_path() -> Self {
        Self::new(DEFAULT_RECIPE_PATH)
    }

    /// Enables or disables hot-reload.
    #[must_use]
    pub fn with_hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload_enabled = enabled;
        self
    }

    /// Returns the base path.
    #[must_use]
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Returns the recipe registry.
    #[must_use]
    pub fn registry(&self) -> &RecipeRegistry {
        &self.registry
    }

    /// Returns loader statistics.
    #[must_use]
    pub fn stats(&self) -> &RecipeLoaderStats {
        &self.stats
    }

    /// Loads all recipes from the base path.
    pub fn load_all(&mut self) -> RecipeLoadResult<()> {
        if !self.base_path.exists() {
            info!(
                "Recipe directory does not exist, creating: {:?}",
                self.base_path
            );
            fs::create_dir_all(&self.base_path)?;
            return Ok(());
        }

        let entries = fs::read_dir(&self.base_path)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                if let Err(e) = self.load_file(&path) {
                    warn!("Failed to load recipe file {:?}: {}", path, e);
                    self.stats.validation_errors += 1;
                }
            }
        }

        info!(
            "Loaded {} recipes from {} files",
            self.stats.recipes_loaded, self.stats.files_loaded
        );

        Ok(())
    }

    /// Loads recipes from a single file.
    pub fn load_file(&mut self, path: &Path) -> RecipeLoadResult<()> {
        debug!("Loading recipe file: {:?}", path);

        let content = fs::read_to_string(path)?;
        let recipe_file: RecipeFile = toml::from_str(&content)?;

        // Track modification time for hot-reload
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.mod_times.insert(path.to_path_buf(), modified);
            }
        }

        let mut loaded_count = 0;
        for recipe in recipe_file.recipes {
            if let Err(e) = recipe.validate() {
                warn!("Invalid recipe in {:?}: {}", path, e);
                self.stats.validation_errors += 1;
                continue;
            }

            match self.registry.register(recipe) {
                Ok(()) => loaded_count += 1,
                Err(e) => {
                    warn!("Failed to register recipe from {:?}: {}", path, e);
                    self.stats.validation_errors += 1;
                },
            }
        }

        self.stats.files_loaded += 1;
        self.stats.recipes_loaded += loaded_count;
        debug!("Loaded {} recipes from {:?}", loaded_count, path);

        Ok(())
    }

    /// Checks for modified files and reloads them.
    ///
    /// Returns true if any files were reloaded.
    pub fn check_hot_reload(&mut self) -> bool {
        if !self.hot_reload_enabled {
            return false;
        }

        let mut reloaded = false;

        // Check existing files for modifications
        let paths_to_check: Vec<_> = self.mod_times.keys().cloned().collect();
        for path in paths_to_check {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Some(prev_modified) = self.mod_times.get(&path) {
                        if modified > *prev_modified {
                            info!("Hot-reloading recipe file: {:?}", path);
                            // Need to reload - for now, clear and reload all
                            self.registry.clear();
                            self.stats = RecipeLoaderStats::default();
                            if self.load_all().is_ok() {
                                self.stats.hot_reloads += 1;
                                reloaded = true;
                            }
                            break;
                        }
                    }
                }
            }
        }

        reloaded
    }

    /// Registers all loaded recipes with a CraftingSystem.
    pub fn register_with_crafting_system(
        &self,
        crafting_system: &mut genesis_gameplay::crafting::CraftingSystem,
    ) {
        for recipe_def in self.registry.iter() {
            let recipe = recipe_def.to_gameplay_recipe();
            crafting_system.register_recipe(recipe);
        }
        debug!(
            "Registered {} recipes with crafting system",
            self.registry.len()
        );
    }

    /// Gets a recipe by ID.
    #[must_use]
    pub fn get_recipe(&self, id: u32) -> Option<&RecipeDefinition> {
        self.registry.get(id)
    }

    /// Gets a recipe by name.
    #[must_use]
    pub fn get_recipe_by_name(&self, name: &str) -> Option<&RecipeDefinition> {
        self.registry.get_by_name(name)
    }

    /// Searches recipes.
    pub fn search(&self, query: &str) -> Vec<&RecipeDefinition> {
        self.registry.search(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_recipe() -> RecipeDefinition {
        RecipeDefinition {
            id: 1,
            name: "Test Recipe".to_string(),
            description: "A test recipe".to_string(),
            category: "test".to_string(),
            ingredients: vec![RecipeIngredient {
                item_id: 100,
                quantity: 2,
            }],
            tools: vec![200],
            workstation: None,
            skill_required: 0,
            skill_type: "crafting".to_string(),
            output: RecipeOutput {
                item_id: 300,
                quantity: 1,
                quality_variance: 0.0,
            },
            byproducts: vec![],
            craft_time_ticks: 60,
            skill_gain: 5,
            unlock_requirements: vec![],
            mod_id: None,
        }
    }

    #[test]
    fn test_recipe_validation_valid() {
        let recipe = sample_recipe();
        assert!(recipe.validate().is_ok());
    }

    #[test]
    fn test_recipe_validation_empty_name() {
        let mut recipe = sample_recipe();
        recipe.name = String::new();
        assert!(matches!(
            recipe.validate(),
            Err(RecipeLoadError::ValidationError(_))
        ));
    }

    #[test]
    fn test_recipe_validation_zero_output() {
        let mut recipe = sample_recipe();
        recipe.output.quantity = 0;
        assert!(matches!(
            recipe.validate(),
            Err(RecipeLoadError::ValidationError(_))
        ));
    }

    #[test]
    fn test_recipe_validation_invalid_quality() {
        let mut recipe = sample_recipe();
        recipe.output.quality_variance = 1.5;
        assert!(matches!(
            recipe.validate(),
            Err(RecipeLoadError::ValidationError(_))
        ));
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = RecipeRegistry::new();
        let recipe = sample_recipe();

        registry.register(recipe).expect("should register");
        assert_eq!(registry.len(), 1);

        let found = registry.get(1);
        assert!(found.is_some());
        assert_eq!(found.expect("found").name, "Test Recipe");
    }

    #[test]
    fn test_registry_duplicate_id() {
        let mut registry = RecipeRegistry::new();
        let recipe = sample_recipe();

        registry.register(recipe.clone()).expect("should register");
        let result = registry.register(recipe);
        assert!(matches!(result, Err(RecipeLoadError::DuplicateId(1))));
    }

    #[test]
    fn test_registry_get_by_name() {
        let mut registry = RecipeRegistry::new();
        registry.register(sample_recipe()).expect("should register");

        // Case-insensitive lookup
        assert!(registry.get_by_name("test recipe").is_some());
        assert!(registry.get_by_name("TEST RECIPE").is_some());
        assert!(registry.get_by_name("Test Recipe").is_some());
        assert!(registry.get_by_name("unknown").is_none());
    }

    #[test]
    fn test_registry_get_by_category() {
        let mut registry = RecipeRegistry::new();

        let mut recipe1 = sample_recipe();
        recipe1.id = 1;
        recipe1.category = "weapons".to_string();

        let mut recipe2 = sample_recipe();
        recipe2.id = 2;
        recipe2.name = "Recipe 2".to_string();
        recipe2.category = "weapons".to_string();

        let mut recipe3 = sample_recipe();
        recipe3.id = 3;
        recipe3.name = "Recipe 3".to_string();
        recipe3.category = "armor".to_string();

        registry.register(recipe1).expect("register 1");
        registry.register(recipe2).expect("register 2");
        registry.register(recipe3).expect("register 3");

        assert_eq!(registry.get_by_category("weapons").len(), 2);
        assert_eq!(registry.get_by_category("armor").len(), 1);
        assert_eq!(registry.get_by_category("unknown").len(), 0);
    }

    #[test]
    fn test_registry_search() {
        let mut registry = RecipeRegistry::new();

        let mut recipe1 = sample_recipe();
        recipe1.id = 1;
        recipe1.name = "Iron Sword".to_string();

        let mut recipe2 = sample_recipe();
        recipe2.id = 2;
        recipe2.name = "Iron Shield".to_string();

        let mut recipe3 = sample_recipe();
        recipe3.id = 3;
        recipe3.name = "Wooden Staff".to_string();

        registry.register(recipe1).expect("register 1");
        registry.register(recipe2).expect("register 2");
        registry.register(recipe3).expect("register 3");

        assert_eq!(registry.search("iron").len(), 2);
        assert_eq!(registry.search("IRON").len(), 2); // case-insensitive
        assert_eq!(registry.search("sword").len(), 1);
        assert_eq!(registry.search("unknown").len(), 0);
    }

    #[test]
    fn test_recipe_to_gameplay() {
        let recipe = sample_recipe();
        let gameplay_recipe = recipe.to_gameplay_recipe();

        assert_eq!(gameplay_recipe.name, "Test Recipe");
        assert_eq!(gameplay_recipe.output.raw(), 300);
        assert_eq!(gameplay_recipe.output_quantity, 1);
        assert_eq!(gameplay_recipe.skill_required, 0);
        assert_eq!(gameplay_recipe.skill_gain, 5);
        assert_eq!(gameplay_recipe.craft_time, 60);
    }

    #[test]
    fn test_parse_toml() {
        let toml_content = r#"
version = "1.0.0"

[[recipes]]
id = 1
name = "Basic Repair Kit"
category = "tools"
skill_type = "crafting"
craft_time_ticks = 120
skill_gain = 5

[recipes.output]
item_id = 400
quantity = 1

[[recipes.ingredients]]
item_id = 100
quantity = 2

[[recipes.ingredients]]
item_id = 101
quantity = 1
"#;

        let recipe_file: RecipeFile = toml::from_str(toml_content).expect("parse");
        assert_eq!(recipe_file.version, "1.0.0");
        assert_eq!(recipe_file.recipes.len(), 1);

        let recipe = &recipe_file.recipes[0];
        assert_eq!(recipe.id, 1);
        assert_eq!(recipe.name, "Basic Repair Kit");
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.output.item_id, 400);
    }
}
