//! Weapon data loading and management.
//!
//! This module provides:
//! - Loading weapons from assets/weapons/*.toml
//! - Weapon validation on load
//! - Hot-reload support for development
//! - Weapon registry with fast lookup by ID, name, and type

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Default asset path for weapons.
pub const DEFAULT_WEAPON_PATH: &str = "assets/weapons";

/// Errors that can occur during weapon loading.
#[derive(Debug, Error)]
pub enum WeaponLoadError {
    /// File not found.
    #[error("Weapon file not found: {0}")]
    NotFound(PathBuf),

    /// Failed to read file.
    #[error("Failed to read weapon file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse TOML.
    #[error("Failed to parse weapon TOML: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Validation error.
    #[error("Weapon validation error: {0}")]
    ValidationError(String),

    /// Duplicate weapon ID.
    #[error("Duplicate weapon ID: {0}")]
    DuplicateId(u32),
}

/// Result type for weapon loading operations.
pub type WeaponLoadResult<T> = Result<T, WeaponLoadError>;

/// Weapon damage type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeaponDamageType {
    /// Physical damage.
    Physical,
    /// Fire damage.
    Fire,
    /// Ice damage.
    Ice,
    /// Electric damage.
    Electric,
    /// Poison damage.
    Poison,
}

impl Default for WeaponDamageType {
    fn default() -> Self {
        Self::Physical
    }
}

/// Weapon category/type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponCategory {
    /// One-handed swords.
    Sword,
    /// Two-handed swords/greatswords.
    Greatsword,
    /// Axes.
    Axe,
    /// Two-handed axes.
    Greataxe,
    /// Maces/hammers.
    Mace,
    /// Two-handed hammers.
    Warhammer,
    /// Daggers/knives.
    Dagger,
    /// Spears/polearms.
    Spear,
    /// Staves.
    Staff,
    /// Bows.
    Bow,
    /// Crossbows.
    Crossbow,
    /// Guns/firearms.
    Gun,
    /// Thrown weapons.
    Thrown,
    /// Unarmed/fists.
    Unarmed,
    /// Shields (for blocking).
    Shield,
}

impl Default for WeaponCategory {
    fn default() -> Self {
        Self::Sword
    }
}

impl WeaponCategory {
    /// Returns true if this is a ranged weapon.
    #[must_use]
    pub const fn is_ranged(&self) -> bool {
        matches!(self, Self::Bow | Self::Crossbow | Self::Gun | Self::Thrown)
    }

    /// Returns true if this is a two-handed weapon.
    #[must_use]
    pub const fn is_two_handed(&self) -> bool {
        matches!(
            self,
            Self::Greatsword
                | Self::Greataxe
                | Self::Warhammer
                | Self::Bow
                | Self::Crossbow
                | Self::Staff
        )
    }

    /// Returns the default reach for this weapon type.
    #[must_use]
    pub const fn default_reach(&self) -> f32 {
        match self {
            Self::Dagger => 0.8,
            Self::Sword | Self::Axe | Self::Mace | Self::Unarmed => 1.0,
            Self::Greatsword | Self::Greataxe | Self::Warhammer => 1.5,
            Self::Spear | Self::Staff => 2.0,
            Self::Bow | Self::Crossbow | Self::Gun => 20.0,
            Self::Thrown => 15.0,
            Self::Shield => 0.5,
        }
    }

    /// Returns the default attack speed for this weapon type.
    #[must_use]
    pub const fn default_speed(&self) -> f32 {
        match self {
            Self::Dagger => 1.5,
            Self::Sword | Self::Mace | Self::Unarmed => 1.0,
            Self::Axe | Self::Spear => 0.9,
            Self::Greatsword | Self::Greataxe | Self::Staff => 0.7,
            Self::Warhammer => 0.6,
            Self::Bow => 0.8,
            Self::Crossbow => 0.5,
            Self::Gun => 0.4,
            Self::Thrown => 1.2,
            Self::Shield => 0.8,
        }
    }
}

/// Status effect that a weapon can apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponStatusEffect {
    /// Effect type.
    pub effect: String,
    /// Chance to apply (0.0-1.0).
    pub chance: f32,
    /// Duration in seconds.
    pub duration: f32,
    /// Stack count.
    #[serde(default = "default_stacks")]
    pub stacks: u32,
}

const fn default_stacks() -> u32 {
    1
}

/// Special ability for a weapon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponAbility {
    /// Ability name.
    pub name: String,
    /// Ability description.
    #[serde(default)]
    pub description: String,
    /// Cooldown in seconds.
    #[serde(default)]
    pub cooldown: f32,
    /// Stamina/mana cost.
    #[serde(default)]
    pub cost: f32,
    /// Damage multiplier.
    #[serde(default = "default_multiplier")]
    pub damage_multiplier: f32,
}

const fn default_multiplier() -> f32 {
    1.0
}

/// Projectile configuration for ranged weapons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileConfig {
    /// Projectile type identifier.
    pub projectile_type: String,
    /// Projectile speed.
    #[serde(default = "default_projectile_speed")]
    pub speed: f32,
    /// Gravity affect (0.0 = no gravity).
    #[serde(default)]
    pub gravity: f32,
    /// Whether it can pierce through targets.
    #[serde(default)]
    pub piercing: bool,
    /// Maximum pierce count (if piercing).
    #[serde(default)]
    pub max_pierce: u32,
}

const fn default_projectile_speed() -> f32 {
    20.0
}

/// A weapon definition loaded from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDefinition {
    /// Unique weapon identifier.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Weapon description.
    #[serde(default)]
    pub description: String,
    /// Weapon category.
    #[serde(default)]
    pub category: WeaponCategory,
    /// Base damage.
    pub base_damage: f32,
    /// Damage variance (randomness).
    #[serde(default)]
    pub damage_variance: f32,
    /// Primary damage type.
    #[serde(default)]
    pub damage_type: WeaponDamageType,
    /// Attack speed multiplier (1.0 = normal).
    #[serde(default = "default_speed")]
    pub attack_speed: f32,
    /// Reach/range in units.
    #[serde(default)]
    pub reach: Option<f32>,
    /// Critical hit chance bonus.
    #[serde(default)]
    pub crit_chance: f32,
    /// Critical damage multiplier.
    #[serde(default = "default_crit_multiplier")]
    pub crit_multiplier: f32,
    /// Knockback strength.
    #[serde(default)]
    pub knockback: f32,
    /// Armor penetration (ignores this much armor).
    #[serde(default)]
    pub armor_penetration: f32,
    /// Stamina cost per attack.
    #[serde(default = "default_stamina_cost")]
    pub stamina_cost: f32,
    /// Durability (0 = indestructible).
    #[serde(default)]
    pub durability: u32,
    /// Required level to equip.
    #[serde(default)]
    pub required_level: u32,
    /// Required strength to equip.
    #[serde(default)]
    pub required_strength: u32,
    /// Required dexterity to equip.
    #[serde(default)]
    pub required_dexterity: u32,
    /// Status effects this weapon can apply.
    #[serde(default)]
    pub status_effects: Vec<WeaponStatusEffect>,
    /// Special abilities.
    #[serde(default)]
    pub abilities: Vec<WeaponAbility>,
    /// Projectile configuration (for ranged).
    #[serde(default)]
    pub projectile: Option<ProjectileConfig>,
    /// Item ID this weapon corresponds to.
    #[serde(default)]
    pub item_id: Option<u32>,
    /// Rarity tier.
    #[serde(default = "default_rarity")]
    pub rarity: String,
    /// Asset path for visuals.
    #[serde(default)]
    pub asset_path: Option<String>,
    /// Sound set name.
    #[serde(default)]
    pub sound_set: Option<String>,
    /// Mod that added this weapon (None = core).
    #[serde(default)]
    pub mod_id: Option<String>,
}

const fn default_speed() -> f32 {
    1.0
}

const fn default_crit_multiplier() -> f32 {
    2.0
}

const fn default_stamina_cost() -> f32 {
    10.0
}

fn default_rarity() -> String {
    "common".to_string()
}

impl WeaponDefinition {
    /// Validates the weapon definition.
    pub fn validate(&self) -> WeaponLoadResult<()> {
        if self.name.is_empty() {
            return Err(WeaponLoadError::ValidationError(format!(
                "Weapon {} has empty name",
                self.id
            )));
        }

        if self.base_damage < 0.0 {
            return Err(WeaponLoadError::ValidationError(format!(
                "Weapon {} has negative base damage: {}",
                self.id, self.base_damage
            )));
        }

        if self.attack_speed <= 0.0 {
            return Err(WeaponLoadError::ValidationError(format!(
                "Weapon {} has invalid attack speed: {}",
                self.id, self.attack_speed
            )));
        }

        if self.crit_chance < 0.0 || self.crit_chance > 1.0 {
            return Err(WeaponLoadError::ValidationError(format!(
                "Weapon {} has invalid crit_chance: {} (must be 0.0-1.0)",
                self.id, self.crit_chance
            )));
        }

        if self.crit_multiplier < 1.0 {
            return Err(WeaponLoadError::ValidationError(format!(
                "Weapon {} has invalid crit_multiplier: {} (must be >= 1.0)",
                self.id, self.crit_multiplier
            )));
        }

        for (i, effect) in self.status_effects.iter().enumerate() {
            if effect.chance < 0.0 || effect.chance > 1.0 {
                return Err(WeaponLoadError::ValidationError(format!(
                    "Weapon {} status effect {} has invalid chance: {}",
                    self.id, i, effect.chance
                )));
            }
        }

        // Ranged weapons must have projectile config
        if self.category.is_ranged() && self.projectile.is_none() {
            warn!("Weapon {} is ranged but has no projectile config", self.id);
        }

        Ok(())
    }

    /// Returns the effective reach for this weapon.
    #[must_use]
    pub fn effective_reach(&self) -> f32 {
        self.reach.unwrap_or_else(|| self.category.default_reach())
    }

    /// Calculates damage with variance.
    #[must_use]
    pub fn calculate_damage(&self, roll: f32) -> f32 {
        let variance = self.damage_variance * self.base_damage;
        self.base_damage + (roll * 2.0 - 1.0) * variance
    }

    /// Returns true if this weapon can apply a critical hit.
    #[must_use]
    pub fn roll_critical(&self, roll: f32, bonus: f32) -> bool {
        roll < (self.crit_chance + bonus)
    }
}

/// A collection of weapons from a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponFile {
    /// File format version.
    #[serde(default = "default_version")]
    pub version: String,
    /// Weapons in this file.
    pub weapons: Vec<WeaponDefinition>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Weapon registry with fast lookup.
pub struct WeaponRegistry {
    /// Weapons by ID.
    by_id: HashMap<u32, WeaponDefinition>,
    /// Weapon IDs by name (lowercase).
    by_name: HashMap<String, u32>,
    /// Weapon IDs by category.
    by_category: HashMap<WeaponCategory, Vec<u32>>,
    /// Weapon IDs by rarity.
    by_rarity: HashMap<String, Vec<u32>>,
}

impl Default for WeaponRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WeaponRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_name: HashMap::new(),
            by_category: HashMap::new(),
            by_rarity: HashMap::new(),
        }
    }

    /// Returns the number of registered weapons.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns true if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Registers a weapon.
    pub fn register(&mut self, weapon: WeaponDefinition) -> WeaponLoadResult<()> {
        if self.by_id.contains_key(&weapon.id) {
            return Err(WeaponLoadError::DuplicateId(weapon.id));
        }

        let id = weapon.id;
        let name_lower = weapon.name.to_lowercase();
        let category = weapon.category;
        let rarity = weapon.rarity.clone();

        // Add to category index
        self.by_category.entry(category).or_default().push(id);

        // Add to rarity index
        self.by_rarity.entry(rarity).or_default().push(id);

        // Add to name index
        self.by_name.insert(name_lower, id);

        // Add to ID map
        self.by_id.insert(id, weapon);

        Ok(())
    }

    /// Gets a weapon by ID.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<&WeaponDefinition> {
        self.by_id.get(&id)
    }

    /// Gets a weapon by name (case-insensitive).
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&WeaponDefinition> {
        self.by_name
            .get(&name.to_lowercase())
            .and_then(|id| self.by_id.get(id))
    }

    /// Gets all weapons of a category.
    #[must_use]
    pub fn get_by_category(&self, category: WeaponCategory) -> Vec<&WeaponDefinition> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.by_id.get(id)).collect())
            .unwrap_or_default()
    }

    /// Gets all weapons of a rarity.
    #[must_use]
    pub fn get_by_rarity(&self, rarity: &str) -> Vec<&WeaponDefinition> {
        self.by_rarity
            .get(rarity)
            .map(|ids| ids.iter().filter_map(|id| self.by_id.get(id)).collect())
            .unwrap_or_default()
    }

    /// Returns an iterator over all weapons.
    pub fn iter(&self) -> impl Iterator<Item = &WeaponDefinition> {
        self.by_id.values()
    }

    /// Searches weapons by name substring (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&WeaponDefinition> {
        let query_lower = query.to_lowercase();
        self.by_id
            .values()
            .filter(|w| w.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Clears the registry.
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_name.clear();
        self.by_category.clear();
        self.by_rarity.clear();
    }

    /// Returns all melee weapons.
    #[must_use]
    pub fn melee_weapons(&self) -> Vec<&WeaponDefinition> {
        self.by_id
            .values()
            .filter(|w| !w.category.is_ranged())
            .collect()
    }

    /// Returns all ranged weapons.
    #[must_use]
    pub fn ranged_weapons(&self) -> Vec<&WeaponDefinition> {
        self.by_id
            .values()
            .filter(|w| w.category.is_ranged())
            .collect()
    }

    /// Returns all two-handed weapons.
    #[must_use]
    pub fn two_handed_weapons(&self) -> Vec<&WeaponDefinition> {
        self.by_id
            .values()
            .filter(|w| w.category.is_two_handed())
            .collect()
    }
}

/// Weapon loader with hot-reload support.
pub struct WeaponLoader {
    /// Base path for weapon files.
    base_path: PathBuf,
    /// Weapon registry.
    registry: WeaponRegistry,
    /// File modification times for hot-reload.
    file_times: HashMap<PathBuf, SystemTime>,
    /// Whether hot-reload is enabled.
    hot_reload_enabled: bool,
}

impl WeaponLoader {
    /// Creates a new weapon loader.
    #[must_use]
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            registry: WeaponRegistry::new(),
            file_times: HashMap::new(),
            hot_reload_enabled: true,
        }
    }

    /// Creates a weapon loader with default path.
    #[must_use]
    pub fn with_default_path() -> Self {
        Self::new(DEFAULT_WEAPON_PATH)
    }

    /// Returns a reference to the weapon registry.
    #[must_use]
    pub fn registry(&self) -> &WeaponRegistry {
        &self.registry
    }

    /// Returns a mutable reference to the weapon registry.
    pub fn registry_mut(&mut self) -> &mut WeaponRegistry {
        &mut self.registry
    }

    /// Enables or disables hot-reload.
    pub fn set_hot_reload(&mut self, enabled: bool) {
        self.hot_reload_enabled = enabled;
    }

    /// Loads all weapons from the base path.
    pub fn load_all(&mut self) -> WeaponLoadResult<usize> {
        let path = &self.base_path;
        if !path.exists() {
            info!("Weapon path does not exist, creating: {:?}", path);
            fs::create_dir_all(path)?;
            return Ok(0);
        }

        let mut count = 0;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().is_some_and(|ext| ext == "toml") {
                match self.load_file(&file_path) {
                    Ok(n) => {
                        count += n;
                        debug!("Loaded {} weapons from {:?}", n, file_path);
                    },
                    Err(e) => {
                        warn!("Failed to load weapon file {:?}: {}", file_path, e);
                    },
                }
            }
        }

        info!("Loaded {} weapons total", count);
        Ok(count)
    }

    /// Loads weapons from a single file.
    pub fn load_file(&mut self, path: &Path) -> WeaponLoadResult<usize> {
        let content = fs::read_to_string(path)?;
        let weapon_file: WeaponFile = toml::from_str(&content)?;

        // Track file modification time
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.file_times.insert(path.to_path_buf(), modified);
            }
        }

        let mut count = 0;
        for weapon in weapon_file.weapons {
            weapon.validate()?;
            self.registry.register(weapon)?;
            count += 1;
        }

        Ok(count)
    }

    /// Checks for file changes and reloads if necessary.
    pub fn check_hot_reload(&mut self) -> WeaponLoadResult<bool> {
        if !self.hot_reload_enabled {
            return Ok(false);
        }

        let mut needs_reload = false;

        for (path, last_time) in &self.file_times {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > *last_time {
                        needs_reload = true;
                        break;
                    }
                }
            }
        }

        if needs_reload {
            info!("Weapon files changed, reloading...");
            self.registry.clear();
            self.file_times.clear();
            self.load_all()?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Reloads all weapons.
    pub fn reload(&mut self) -> WeaponLoadResult<usize> {
        self.registry.clear();
        self.file_times.clear();
        self.load_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_category_is_ranged() {
        assert!(WeaponCategory::Bow.is_ranged());
        assert!(WeaponCategory::Gun.is_ranged());
        assert!(!WeaponCategory::Sword.is_ranged());
        assert!(!WeaponCategory::Dagger.is_ranged());
    }

    #[test]
    fn test_weapon_category_is_two_handed() {
        assert!(WeaponCategory::Greatsword.is_two_handed());
        assert!(WeaponCategory::Bow.is_two_handed());
        assert!(!WeaponCategory::Sword.is_two_handed());
        assert!(!WeaponCategory::Dagger.is_two_handed());
    }

    #[test]
    fn test_weapon_definition_validate() {
        let weapon = WeaponDefinition {
            id: 1,
            name: "Iron Sword".to_string(),
            description: "A basic sword".to_string(),
            category: WeaponCategory::Sword,
            base_damage: 10.0,
            damage_variance: 0.1,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 0.1,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 1,
            required_strength: 5,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: Some(100),
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        assert!(weapon.validate().is_ok());
    }

    #[test]
    fn test_weapon_definition_validate_empty_name() {
        let weapon = WeaponDefinition {
            id: 1,
            name: String::new(),
            description: String::new(),
            category: WeaponCategory::Sword,
            base_damage: 10.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        assert!(matches!(
            weapon.validate(),
            Err(WeaponLoadError::ValidationError(_))
        ));
    }

    #[test]
    fn test_weapon_definition_validate_invalid_crit() {
        let weapon = WeaponDefinition {
            id: 1,
            name: "Test".to_string(),
            description: String::new(),
            category: WeaponCategory::Sword,
            base_damage: 10.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 1.5, // Invalid
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        assert!(matches!(
            weapon.validate(),
            Err(WeaponLoadError::ValidationError(_))
        ));
    }

    #[test]
    fn test_weapon_registry_register() {
        let mut registry = WeaponRegistry::new();

        let weapon = WeaponDefinition {
            id: 1,
            name: "Iron Sword".to_string(),
            description: String::new(),
            category: WeaponCategory::Sword,
            base_damage: 10.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        assert!(registry.register(weapon).is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_weapon_registry_get() {
        let mut registry = WeaponRegistry::new();

        let weapon = WeaponDefinition {
            id: 42,
            name: "Test Weapon".to_string(),
            description: String::new(),
            category: WeaponCategory::Dagger,
            base_damage: 5.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.5,
            reach: None,
            crit_chance: 0.2,
            crit_multiplier: 3.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 5.0,
            durability: 50,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        registry.register(weapon).unwrap();

        assert!(registry.get(42).is_some());
        assert!(registry.get(999).is_none());
    }

    #[test]
    fn test_weapon_registry_get_by_name() {
        let mut registry = WeaponRegistry::new();

        let weapon = WeaponDefinition {
            id: 1,
            name: "Flame Blade".to_string(),
            description: String::new(),
            category: WeaponCategory::Sword,
            base_damage: 15.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Fire,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "rare".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        registry.register(weapon).unwrap();

        assert!(registry.get_by_name("flame blade").is_some());
        assert!(registry.get_by_name("FLAME BLADE").is_some());
        assert!(registry.get_by_name("unknown").is_none());
    }

    #[test]
    fn test_weapon_registry_get_by_category() {
        let mut registry = WeaponRegistry::new();

        let sword = WeaponDefinition {
            id: 1,
            name: "Sword".to_string(),
            description: String::new(),
            category: WeaponCategory::Sword,
            base_damage: 10.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        let dagger = WeaponDefinition {
            id: 2,
            name: "Dagger".to_string(),
            description: String::new(),
            category: WeaponCategory::Dagger,
            base_damage: 5.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.5,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 5.0,
            durability: 50,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        registry.register(sword).unwrap();
        registry.register(dagger).unwrap();

        assert_eq!(registry.get_by_category(WeaponCategory::Sword).len(), 1);
        assert_eq!(registry.get_by_category(WeaponCategory::Dagger).len(), 1);
        assert_eq!(registry.get_by_category(WeaponCategory::Bow).len(), 0);
    }

    #[test]
    fn test_weapon_calculate_damage() {
        let weapon = WeaponDefinition {
            id: 1,
            name: "Test".to_string(),
            description: String::new(),
            category: WeaponCategory::Sword,
            base_damage: 100.0,
            damage_variance: 0.1,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        // Roll 0.5 = no variance
        assert!((weapon.calculate_damage(0.5) - 100.0).abs() < 0.01);

        // Roll 0.0 = minimum damage
        assert!((weapon.calculate_damage(0.0) - 90.0).abs() < 0.01);

        // Roll 1.0 = maximum damage
        assert!((weapon.calculate_damage(1.0) - 110.0).abs() < 0.01);
    }

    #[test]
    fn test_weapon_effective_reach() {
        let mut weapon = WeaponDefinition {
            id: 1,
            name: "Test".to_string(),
            description: String::new(),
            category: WeaponCategory::Dagger,
            base_damage: 5.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.5,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 5.0,
            durability: 50,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        // No reach specified, use category default
        assert!((weapon.effective_reach() - 0.8).abs() < 0.01);

        // With explicit reach
        weapon.reach = Some(1.2);
        assert!((weapon.effective_reach() - 1.2).abs() < 0.01);
    }

    #[test]
    fn test_weapon_registry_duplicate() {
        let mut registry = WeaponRegistry::new();

        let weapon1 = WeaponDefinition {
            id: 1,
            name: "Weapon".to_string(),
            description: String::new(),
            category: WeaponCategory::Sword,
            base_damage: 10.0,
            damage_variance: 0.0,
            damage_type: WeaponDamageType::Physical,
            attack_speed: 1.0,
            reach: None,
            crit_chance: 0.0,
            crit_multiplier: 2.0,
            knockback: 0.0,
            armor_penetration: 0.0,
            stamina_cost: 10.0,
            durability: 100,
            required_level: 0,
            required_strength: 0,
            required_dexterity: 0,
            status_effects: vec![],
            abilities: vec![],
            projectile: None,
            item_id: None,
            rarity: "common".to_string(),
            asset_path: None,
            sound_set: None,
            mod_id: None,
        };

        let weapon2 = weapon1.clone();

        assert!(registry.register(weapon1).is_ok());
        assert!(matches!(
            registry.register(weapon2),
            Err(WeaponLoadError::DuplicateId(1))
        ));
    }
}
