//! World creation options.
//!
//! This module provides configuration for new world creation:
//! - World name validation
//! - Seed input and generation
//! - Difficulty selection
//! - World size options
//! - Starting items toggle

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

// ============================================================================
// G-59: World Size
// ============================================================================

/// World size preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorldSize {
    /// Small world (faster generation, less exploration).
    Small,
    /// Medium world (balanced).
    #[default]
    Medium,
    /// Large world (more content, longer generation).
    Large,
    /// Huge world (maximum exploration).
    Huge,
}

impl WorldSize {
    /// Get world dimensions in chunks.
    #[must_use]
    pub const fn chunks(&self) -> (u32, u32) {
        match self {
            Self::Small => (64, 64),
            Self::Medium => (128, 128),
            Self::Large => (256, 256),
            Self::Huge => (512, 512),
        }
    }

    /// Get total chunk count.
    #[must_use]
    pub const fn total_chunks(&self) -> u32 {
        let (w, h) = self.chunks();
        w * h
    }

    /// Get approximate world size in blocks.
    #[must_use]
    pub const fn blocks(&self) -> (u32, u32) {
        let (cw, ch) = self.chunks();
        // Assuming 16x16 blocks per chunk
        (cw * 16, ch * 16)
    }

    /// Get estimated generation time in seconds.
    #[must_use]
    pub const fn estimated_generation_time(&self) -> u32 {
        match self {
            Self::Small => 5,
            Self::Medium => 15,
            Self::Large => 45,
            Self::Huge => 120,
        }
    }

    /// Get display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Huge => "Huge",
        }
    }

    /// Get description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Small => "A compact world, ideal for quick playthroughs.",
            Self::Medium => "A balanced world with plenty to explore.",
            Self::Large => "A vast world with extensive exploration opportunities.",
            Self::Huge => "A massive world for those seeking endless adventure.",
        }
    }

    /// Get all sizes.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Small, Self::Medium, Self::Large, Self::Huge]
    }
}

// ============================================================================
// G-59: World Difficulty
// ============================================================================

/// World generation difficulty affecting resource distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorldDifficulty {
    /// Peaceful - abundant resources, no hostile spawns.
    Peaceful,
    /// Easy - more resources, fewer hazards.
    Easy,
    /// Normal - balanced resources and hazards.
    #[default]
    Normal,
    /// Hard - scarce resources, more hazards.
    Hard,
    /// Hardcore - permadeath, minimal resources.
    Hardcore,
}

impl WorldDifficulty {
    /// Get resource multiplier.
    #[must_use]
    pub const fn resource_multiplier(&self) -> f32 {
        match self {
            Self::Peaceful => 2.0,
            Self::Easy => 1.5,
            Self::Normal => 1.0,
            Self::Hard => 0.7,
            Self::Hardcore => 0.5,
        }
    }

    /// Get enemy spawn rate multiplier.
    #[must_use]
    pub const fn enemy_spawn_rate(&self) -> f32 {
        match self {
            Self::Peaceful => 0.0,
            Self::Easy => 0.5,
            Self::Normal => 1.0,
            Self::Hard => 1.5,
            Self::Hardcore => 2.0,
        }
    }

    /// Get hazard frequency multiplier.
    #[must_use]
    pub const fn hazard_multiplier(&self) -> f32 {
        match self {
            Self::Peaceful => 0.0,
            Self::Easy => 0.5,
            Self::Normal => 1.0,
            Self::Hard => 1.5,
            Self::Hardcore => 2.0,
        }
    }

    /// Check if permadeath is enabled.
    #[must_use]
    pub const fn is_permadeath(&self) -> bool {
        matches!(self, Self::Hardcore)
    }

    /// Get display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Peaceful => "Peaceful",
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Hardcore => "Hardcore",
        }
    }

    /// Get description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Peaceful => "A relaxing experience with no combat threats.",
            Self::Easy => "A gentler adventure with more resources.",
            Self::Normal => "The standard experience as intended.",
            Self::Hard => "A challenging journey for experienced players.",
            Self::Hardcore => "Permadeath enabled. One life, no second chances.",
        }
    }

    /// Get all difficulties.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Peaceful,
            Self::Easy,
            Self::Normal,
            Self::Hard,
            Self::Hardcore,
        ]
    }
}

// ============================================================================
// G-59: Starting Items
// ============================================================================

/// Starting item preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StartingItems {
    /// No starting items (challenge mode).
    Nothing,
    /// Minimal starting items.
    Minimal,
    /// Standard starting kit.
    #[default]
    Standard,
    /// Bonus starting items.
    Bonus,
}

impl StartingItems {
    /// Get display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Nothing => "Nothing",
            Self::Minimal => "Minimal",
            Self::Standard => "Standard",
            Self::Bonus => "Bonus",
        }
    }

    /// Get description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Nothing => "Start with nothing. True survival.",
            Self::Minimal => "Basic tools to get started.",
            Self::Standard => "A reasonable starting kit.",
            Self::Bonus => "Extra items to help you along.",
        }
    }

    /// Get all presets.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Nothing, Self::Minimal, Self::Standard, Self::Bonus]
    }
}

// ============================================================================
// G-59: World Type
// ============================================================================

/// World generation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorldType {
    /// Standard mixed biome world.
    #[default]
    Standard,
    /// Flat world for building.
    Flat,
    /// Island world surrounded by ocean.
    Island,
    /// Desert world with oasis.
    Desert,
    /// Frozen world with snow and ice.
    Frozen,
    /// Jungle world with dense vegetation.
    Jungle,
    /// Volcanic world with lava and ash.
    Volcanic,
    /// Custom world (use advanced options).
    Custom,
}

impl WorldType {
    /// Get display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Flat => "Flat",
            Self::Island => "Island",
            Self::Desert => "Desert",
            Self::Frozen => "Frozen",
            Self::Jungle => "Jungle",
            Self::Volcanic => "Volcanic",
            Self::Custom => "Custom",
        }
    }

    /// Get description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Standard => "A balanced world with varied biomes.",
            Self::Flat => "A flat world ideal for creative building.",
            Self::Island => "Start on an island surrounded by ocean.",
            Self::Desert => "An arid world with scattered oases.",
            Self::Frozen => "A frozen landscape of ice and snow.",
            Self::Jungle => "Dense tropical vegetation everywhere.",
            Self::Volcanic => "A dangerous world of lava and ash.",
            Self::Custom => "Customize all generation parameters.",
        }
    }

    /// Get all types.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Standard,
            Self::Flat,
            Self::Island,
            Self::Desert,
            Self::Frozen,
            Self::Jungle,
            Self::Volcanic,
            Self::Custom,
        ]
    }
}

// ============================================================================
// G-59: Seed
// ============================================================================

/// World seed for generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSeed {
    /// The numeric seed value.
    value: u64,
    /// Original input (if from string).
    original: Option<String>,
}

impl WorldSeed {
    /// Create from numeric value.
    #[must_use]
    pub const fn from_value(value: u64) -> Self {
        Self {
            value,
            original: None,
        }
    }

    /// Create from string (hashed).
    #[must_use]
    pub fn from_string(s: &str) -> Self {
        let value = Self::hash_string(s);
        Self {
            value,
            original: Some(s.to_string()),
        }
    }

    /// Generate a random seed.
    #[must_use]
    pub fn random() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let value = duration.as_nanos() as u64;
        Self {
            value,
            original: None,
        }
    }

    /// Get the numeric seed value.
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.value
    }

    /// Get the original string input, if any.
    #[must_use]
    pub fn original(&self) -> Option<&str> {
        self.original.as_deref()
    }

    /// Get display string.
    #[must_use]
    pub fn display(&self) -> String {
        self.original
            .clone()
            .unwrap_or_else(|| self.value.to_string())
    }

    /// Hash a string to a seed value.
    fn hash_string(s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for WorldSeed {
    fn default() -> Self {
        Self::random()
    }
}

impl From<u64> for WorldSeed {
    fn from(value: u64) -> Self {
        Self::from_value(value)
    }
}

impl From<&str> for WorldSeed {
    fn from(s: &str) -> Self {
        // Try to parse as number first
        if let Ok(value) = s.parse::<u64>() {
            Self::from_value(value)
        } else {
            Self::from_string(s)
        }
    }
}

impl From<String> for WorldSeed {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

// ============================================================================
// G-59: World Name Validation
// ============================================================================

/// World name validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldNameValidation {
    /// Name is valid.
    Valid,
    /// Name is empty.
    Empty,
    /// Name is too short.
    TooShort {
        /// Minimum required length.
        minimum: usize,
        /// Actual length.
        actual: usize,
    },
    /// Name is too long.
    TooLong {
        /// Maximum allowed length.
        maximum: usize,
        /// Actual length.
        actual: usize,
    },
    /// Name contains invalid characters.
    InvalidCharacters {
        /// The invalid characters found.
        invalid: Vec<char>,
    },
    /// Name is reserved.
    Reserved,
}

impl WorldNameValidation {
    /// Check if valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Get error message.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Valid => String::new(),
            Self::Empty => "World name cannot be empty.".to_string(),
            Self::TooShort { minimum, actual } => {
                format!("World name must be at least {minimum} characters (currently {actual}).")
            },
            Self::TooLong { maximum, actual } => {
                format!("World name cannot exceed {maximum} characters (currently {actual}).")
            },
            Self::InvalidCharacters { invalid } => {
                let chars: String = invalid.iter().collect();
                format!("World name contains invalid characters: {chars}")
            },
            Self::Reserved => "This world name is reserved.".to_string(),
        }
    }
}

/// Validate a world name.
#[must_use]
pub fn validate_world_name(name: &str) -> WorldNameValidation {
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: usize = 32;
    const RESERVED_NAMES: &[&str] = &["con", "prn", "aux", "nul", "com1", "lpt1"];

    let name = name.trim();

    if name.is_empty() {
        return WorldNameValidation::Empty;
    }

    if name.len() < MIN_LENGTH {
        return WorldNameValidation::TooShort {
            minimum: MIN_LENGTH,
            actual: name.len(),
        };
    }

    if name.len() > MAX_LENGTH {
        return WorldNameValidation::TooLong {
            maximum: MAX_LENGTH,
            actual: name.len(),
        };
    }

    // Check for invalid characters
    let invalid: Vec<char> = name
        .chars()
        .filter(|c| !c.is_alphanumeric() && *c != ' ' && *c != '-' && *c != '_')
        .collect();

    if !invalid.is_empty() {
        return WorldNameValidation::InvalidCharacters { invalid };
    }

    // Check for reserved names
    if RESERVED_NAMES.contains(&name.to_lowercase().as_str()) {
        return WorldNameValidation::Reserved;
    }

    WorldNameValidation::Valid
}

/// Sanitize a world name.
#[must_use]
pub fn sanitize_world_name(name: &str) -> String {
    name.trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .take(32)
        .collect::<String>()
        .trim()
        .to_string()
}

// ============================================================================
// G-59: World Creation Options
// ============================================================================

/// World creation options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldCreationOptions {
    /// World name.
    pub name: String,
    /// World seed.
    pub seed: WorldSeed,
    /// World size.
    pub size: WorldSize,
    /// World difficulty.
    pub difficulty: WorldDifficulty,
    /// World type.
    pub world_type: WorldType,
    /// Starting items preset.
    pub starting_items: StartingItems,
    /// Enable bonus chest at spawn.
    pub bonus_chest: bool,
    /// Enable structures generation.
    pub generate_structures: bool,
    /// Enable cave generation.
    pub generate_caves: bool,
    /// Enable ore generation.
    pub generate_ores: bool,
    /// Enable weather.
    pub enable_weather: bool,
    /// Day/night cycle enabled.
    pub day_night_cycle: bool,
    /// Day length in real-time minutes.
    pub day_length_minutes: f32,
    /// Player keep inventory on death.
    pub keep_inventory: bool,
    /// Player natural regeneration.
    pub natural_regeneration: bool,
    /// Fire spreads.
    pub fire_spread: bool,
    /// Allow cheats/commands.
    pub cheats_enabled: bool,
}

impl Default for WorldCreationOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            seed: WorldSeed::random(),
            size: WorldSize::default(),
            difficulty: WorldDifficulty::default(),
            world_type: WorldType::default(),
            starting_items: StartingItems::default(),
            bonus_chest: false,
            generate_structures: true,
            generate_caves: true,
            generate_ores: true,
            enable_weather: true,
            day_night_cycle: true,
            day_length_minutes: 20.0,
            keep_inventory: false,
            natural_regeneration: true,
            fire_spread: true,
            cheats_enabled: false,
        }
    }
}

impl WorldCreationOptions {
    /// Create new default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Create with seed.
    #[must_use]
    pub fn with_seed(mut self, seed: impl Into<WorldSeed>) -> Self {
        self.seed = seed.into();
        self
    }

    /// Create with size.
    #[must_use]
    pub fn with_size(mut self, size: WorldSize) -> Self {
        self.size = size;
        self
    }

    /// Create with difficulty.
    #[must_use]
    pub fn with_difficulty(mut self, difficulty: WorldDifficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Create with world type.
    #[must_use]
    pub fn with_world_type(mut self, world_type: WorldType) -> Self {
        self.world_type = world_type;
        self
    }

    /// Validate options.
    #[must_use]
    pub fn validate(&self) -> WorldCreationValidation {
        let mut validation = WorldCreationValidation::new();

        // Validate name
        let name_validation = validate_world_name(&self.name);
        if !name_validation.is_valid() {
            validation.add_error(&name_validation.message());
        }

        // Validate day length
        if !(1.0..=120.0).contains(&self.day_length_minutes) {
            validation.add_warning("Day length should be between 1 and 120 minutes.");
        }

        // Warn about hardcore + keep inventory
        if self.difficulty.is_permadeath() && self.keep_inventory {
            validation.add_warning("Keep inventory has no effect in Hardcore mode.");
        }

        // Warn about cheats in hardcore
        if self.difficulty.is_permadeath() && self.cheats_enabled {
            validation.add_warning("Cheats may undermine the Hardcore experience.");
        }

        validation
    }

    /// Check if options are valid for world creation.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validate().is_valid()
    }

    /// Generate a unique folder name for this world.
    #[must_use]
    pub fn folder_name(&self) -> String {
        let sanitized = sanitize_world_name(&self.name);
        if sanitized.is_empty() {
            format!("world_{}", self.seed.value())
        } else {
            sanitized.replace(' ', "_").to_lowercase()
        }
    }
}

/// World creation validation result.
#[derive(Debug, Clone, Default)]
pub struct WorldCreationValidation {
    /// Error messages (prevent creation).
    pub errors: Vec<String>,
    /// Warning messages (allow creation but warn).
    pub warnings: Vec<String>,
}

impl WorldCreationValidation {
    /// Create empty validation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error.
    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    /// Add a warning.
    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    /// Check if valid (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check for warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_size() {
        let size = WorldSize::Medium;
        let (w, h) = size.chunks();
        assert_eq!(w, 128);
        assert_eq!(h, 128);
        assert_eq!(size.total_chunks(), 128 * 128);
    }

    #[test]
    fn test_world_size_blocks() {
        let size = WorldSize::Small;
        let (bw, bh) = size.blocks();
        assert_eq!(bw, 64 * 16);
        assert_eq!(bh, 64 * 16);
    }

    #[test]
    fn test_world_difficulty_multipliers() {
        assert_eq!(WorldDifficulty::Peaceful.enemy_spawn_rate(), 0.0);
        assert_eq!(WorldDifficulty::Normal.resource_multiplier(), 1.0);
        assert!(WorldDifficulty::Hardcore.is_permadeath());
    }

    #[test]
    fn test_world_seed_from_value() {
        let seed = WorldSeed::from_value(12345);
        assert_eq!(seed.value(), 12345);
        assert!(seed.original().is_none());
    }

    #[test]
    fn test_world_seed_from_string() {
        let seed = WorldSeed::from_string("hello");
        assert!(seed.value() != 0);
        assert_eq!(seed.original(), Some("hello"));
        assert_eq!(seed.display(), "hello");
    }

    #[test]
    fn test_world_seed_from_numeric_string() {
        let seed = WorldSeed::from("12345");
        assert_eq!(seed.value(), 12345);
        assert!(seed.original().is_none());
    }

    #[test]
    fn test_validate_world_name_valid() {
        assert!(validate_world_name("MyWorld").is_valid());
        assert!(validate_world_name("My World").is_valid());
        assert!(validate_world_name("World-1").is_valid());
        assert!(validate_world_name("World_2").is_valid());
    }

    #[test]
    fn test_validate_world_name_empty() {
        assert!(matches!(
            validate_world_name(""),
            WorldNameValidation::Empty
        ));
        assert!(matches!(
            validate_world_name("   "),
            WorldNameValidation::Empty
        ));
    }

    #[test]
    fn test_validate_world_name_too_long() {
        let long_name = "a".repeat(50);
        assert!(matches!(
            validate_world_name(&long_name),
            WorldNameValidation::TooLong { .. }
        ));
    }

    #[test]
    fn test_validate_world_name_invalid_chars() {
        let result = validate_world_name("World<>Name");
        assert!(matches!(
            result,
            WorldNameValidation::InvalidCharacters { .. }
        ));
    }

    #[test]
    fn test_validate_world_name_reserved() {
        assert!(matches!(
            validate_world_name("CON"),
            WorldNameValidation::Reserved
        ));
    }

    #[test]
    fn test_sanitize_world_name() {
        assert_eq!(sanitize_world_name("Hello<>World"), "HelloWorld");
        assert_eq!(sanitize_world_name("  Trimmed  "), "Trimmed");
        assert_eq!(sanitize_world_name("With Spaces"), "With Spaces");
    }

    #[test]
    fn test_world_creation_options_default() {
        let options = WorldCreationOptions::new();
        assert!(options.generate_structures);
        assert!(options.enable_weather);
        assert!(!options.cheats_enabled);
    }

    #[test]
    fn test_world_creation_options_builder() {
        let options = WorldCreationOptions::new()
            .with_name("TestWorld")
            .with_size(WorldSize::Large)
            .with_difficulty(WorldDifficulty::Hard);

        assert_eq!(options.name, "TestWorld");
        assert_eq!(options.size, WorldSize::Large);
        assert_eq!(options.difficulty, WorldDifficulty::Hard);
    }

    #[test]
    fn test_world_creation_validation() {
        let mut options = WorldCreationOptions::new();
        options.name = "ValidName".to_string();
        assert!(options.is_valid());

        options.name = String::new();
        assert!(!options.is_valid());
    }

    #[test]
    fn test_world_creation_warnings() {
        let options = WorldCreationOptions::new()
            .with_name("Test")
            .with_difficulty(WorldDifficulty::Hardcore);
        let mut opts = options;
        opts.cheats_enabled = true;

        let validation = opts.validate();
        assert!(validation.is_valid()); // Still valid
        assert!(validation.has_warnings()); // But has warnings
    }

    #[test]
    fn test_folder_name() {
        let options = WorldCreationOptions::new().with_name("My Test World");

        assert_eq!(options.folder_name(), "my_test_world");

        let empty_name = WorldCreationOptions::new();
        assert!(empty_name.folder_name().starts_with("world_"));
    }

    #[test]
    fn test_starting_items() {
        assert_eq!(StartingItems::Standard.display_name(), "Standard");
        assert_eq!(StartingItems::all().len(), 4);
    }

    #[test]
    fn test_world_type() {
        assert_eq!(WorldType::Standard.display_name(), "Standard");
        assert_eq!(WorldType::all().len(), 8);
    }
}
