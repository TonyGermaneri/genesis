//! Character Generator - Comprehensive character creation and editing system.
//!
//! This module provides a character generation system that allows:
//! - Creating characters with layered sprite customization
//! - Setting character stats with random ranges
//! - Configuring inventory with probability-based items
//! - Setting dialog options
//! - Organizing characters in a folder hierarchy
//! - Save/load/undo/redo functionality

use egui::{Color32, ColorImage, RichText, ScrollArea, TextureHandle, TextureOptions, Ui, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::sprite_builder::{AnimationAction, CharacterSpriteDef, SpriteFrame};

// ============================================================================
// Utility Functions
// ============================================================================

/// Truncate a string to a maximum length, adding "..." if truncated.
#[allow(dead_code)]
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s.chars().take(max_len).collect()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}

// ============================================================================
// Character Sprite Layers
// ============================================================================

/// Available sprite layer categories that can be customized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpriteLayerCategory {
    /// Base body sprite
    Body,
    /// Eye sprites
    Eyes,
    /// Hair style sprites
    Hairstyle,
    /// Outfit/clothing sprites
    Outfit,
    /// Accessory sprites (hats, glasses, etc.)
    Accessory,
    /// Book sprites (for reading animations)
    Book,
    /// Smartphone sprites
    Smartphone,
}

impl SpriteLayerCategory {
    /// Get all layer categories in rendering order (bottom to top).
    pub fn all() -> &'static [Self] {
        &[
            Self::Body,
            Self::Eyes,
            Self::Outfit,
            Self::Hairstyle,
            Self::Accessory,
            Self::Book,
            Self::Smartphone,
        ]
    }

    /// Get display name for the category.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Body => "Body",
            Self::Eyes => "Eyes",
            Self::Hairstyle => "Hairstyle",
            Self::Outfit => "Outfit",
            Self::Accessory => "Accessory",
            Self::Book => "Book",
            Self::Smartphone => "Smartphone",
        }
    }

    /// Get the subdirectory name for this category.
    pub fn subdir_name(&self) -> &'static str {
        match self {
            Self::Body => "Bodies",
            Self::Eyes => "Eyes",
            Self::Hairstyle => "Hairstyles",
            Self::Outfit => "Outfits",
            Self::Accessory => "Accessories",
            Self::Book => "Books",
            Self::Smartphone => "Smartphones",
        }
    }

    /// Get icon for the category.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Body => "👤",
            Self::Eyes => "👁",
            Self::Hairstyle => "💇",
            Self::Outfit => "👕",
            Self::Accessory => "🎩",
            Self::Book => "📖",
            Self::Smartphone => "📱",
        }
    }
}

/// A sprite option available for a layer category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteOption {
    /// Unique identifier (filename without extension).
    pub id: String,
    /// Display name extracted from filename.
    pub display_name: String,
    /// Full path to the sprite file.
    pub path: PathBuf,
    /// Group name (e.g., "Hairstyle_01" groups all color variants).
    pub group: String,
    /// Variant within the group (e.g., color variant "01", "02").
    pub variant: String,
}

/// Available sprite options for all categories.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AvailableSpriteAssets {
    /// Options by category.
    pub options: HashMap<SpriteLayerCategory, Vec<SpriteOption>>,
    /// Grouped options by category (group name -> variants).
    pub grouped: HashMap<SpriteLayerCategory, HashMap<String, Vec<SpriteOption>>>,
    /// Last scan time.
    pub last_scan: Option<std::time::SystemTime>,
}

impl AvailableSpriteAssets {
    /// Scan the assets directory to find all available sprite options.
    pub fn scan(base_path: &Path) -> Self {
        let mut assets = Self::default();
        assets.last_scan = Some(std::time::SystemTime::now());

        for category in SpriteLayerCategory::all() {
            let category_path = base_path
                .join(category.subdir_name())
                .join("48x48");

            if let Ok(entries) = std::fs::read_dir(&category_path) {
                let mut options = Vec::new();

                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "png") {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let (group, variant) = Self::parse_sprite_name(file_stem);
                            let display_name = Self::format_display_name(file_stem);

                            options.push(SpriteOption {
                                id: file_stem.to_string(),
                                display_name,
                                path: path.clone(),
                                group,
                                variant,
                            });
                        }
                    }
                }

                // Sort options by id
                options.sort_by(|a, b| a.id.cmp(&b.id));

                // Build grouped map
                let mut grouped: HashMap<String, Vec<SpriteOption>> = HashMap::new();
                for opt in &options {
                    grouped
                        .entry(opt.group.clone())
                        .or_default()
                        .push(opt.clone());
                }

                assets.options.insert(*category, options);
                assets.grouped.insert(*category, grouped);
            }
        }

        assets
    }

    /// Parse sprite name to extract group and variant.
    /// E.g., "Hairstyle_01_48x48_03" -> ("Hairstyle_01", "03")
    fn parse_sprite_name(name: &str) -> (String, String) {
        // Remove the "_48x48" suffix and split
        let parts: Vec<&str> = name.split('_').collect();

        if parts.len() >= 3 {
            // Find the 48x48 part
            let mut group_parts = Vec::new();
            let mut variant = String::new();
            let mut found_size = false;

            for (i, part) in parts.iter().enumerate() {
                if *part == "48x48" {
                    found_size = true;
                    // Everything after 48x48 is the variant
                    if i + 1 < parts.len() {
                        variant = parts[i + 1..].join("_");
                    }
                    break;
                }
                group_parts.push(*part);
            }

            if found_size {
                return (group_parts.join("_"), variant);
            }
        }

        // Fallback: use the whole name as group
        (name.to_string(), String::new())
    }

    /// Format a display name from a sprite filename.
    fn format_display_name(name: &str) -> String {
        // Remove 48x48 and convert underscores to spaces
        name.replace("_48x48", "")
            .replace('_', " ")
            .trim()
            .to_string()
    }

    /// Get options for a category.
    pub fn get_options(&self, category: SpriteLayerCategory) -> &[SpriteOption] {
        self.options.get(&category).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get grouped options for a category.
    pub fn get_groups(&self, category: SpriteLayerCategory) -> Option<&HashMap<String, Vec<SpriteOption>>> {
        self.grouped.get(&category)
    }
}

// ============================================================================
// Character Appearance
// ============================================================================

/// Selected sprite layer for a character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedLayer {
    /// Whether this layer is enabled.
    pub enabled: bool,
    /// Selected sprite option ID (filename without extension).
    pub sprite_id: Option<String>,
}

impl Default for SelectedLayer {
    fn default() -> Self {
        Self {
            enabled: true,
            sprite_id: None,
        }
    }
}

/// Character appearance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAppearance {
    /// Base animation template TOML file path.
    pub animation_template: PathBuf,
    /// Selected layers by category.
    pub layers: HashMap<SpriteLayerCategory, SelectedLayer>,
}

impl Default for CharacterAppearance {
    fn default() -> Self {
        let mut layers = HashMap::new();
        for category in SpriteLayerCategory::all() {
            layers.insert(*category, SelectedLayer::default());
        }

        Self {
            animation_template: PathBuf::from("assets/sprites/characters/character_0.toml"),
            layers,
        }
    }
}

// ============================================================================
// Character Stats
// ============================================================================

/// A stat value with random range for spawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatRange {
    /// Minimum value when spawned.
    pub min: f32,
    /// Maximum value when spawned.
    pub max: f32,
    /// Base/default value for display.
    pub base: f32,
}

impl StatRange {
    /// Create a new stat range.
    pub fn new(base: f32, variance: f32) -> Self {
        Self {
            min: (base - variance).max(0.0),
            max: base + variance,
            base,
        }
    }

    /// Create a fixed stat (no variance).
    pub fn fixed(value: f32) -> Self {
        Self {
            min: value,
            max: value,
            base: value,
        }
    }

    /// Roll a random value within the range.
    pub fn roll(&self) -> f32 {
        if (self.max - self.min).abs() < 0.001 {
            self.base
        } else {
            // Would use actual random in game
            self.base
        }
    }
}

impl Default for StatRange {
    fn default() -> Self {
        Self::fixed(10.0)
    }
}

/// Character stats with random ranges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    // Core stats
    /// Maximum health.
    pub max_health: StatRange,
    /// Maximum stamina.
    pub max_stamina: StatRange,
    /// Maximum hunger.
    pub max_hunger: StatRange,

    // Combat stats
    /// Base attack damage.
    pub attack: StatRange,
    /// Defense/damage reduction.
    pub defense: StatRange,
    /// Attack speed modifier.
    pub speed: StatRange,
    /// Critical hit chance (0-100).
    pub crit_chance: StatRange,

    // Skill levels
    /// Mining skill level.
    pub mining: StatRange,
    /// Crafting skill level.
    pub crafting: StatRange,
    /// Combat skill level.
    pub combat: StatRange,
    /// Farming skill level.
    pub farming: StatRange,

    // AI/Meta
    /// AI difficulty level (0-100).
    pub ai_difficulty: StatRange,
    /// XP reward when defeated.
    pub xp_reward: StatRange,
    /// Starting money range.
    pub money: StatRange,
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            max_health: StatRange::new(100.0, 20.0),
            max_stamina: StatRange::new(100.0, 20.0),
            max_hunger: StatRange::new(100.0, 10.0),
            attack: StatRange::new(10.0, 5.0),
            defense: StatRange::new(5.0, 3.0),
            speed: StatRange::new(1.0, 0.2),
            crit_chance: StatRange::new(5.0, 3.0),
            mining: StatRange::new(1.0, 0.5),
            crafting: StatRange::new(1.0, 0.5),
            combat: StatRange::new(1.0, 0.5),
            farming: StatRange::new(1.0, 0.5),
            ai_difficulty: StatRange::new(50.0, 25.0),
            xp_reward: StatRange::new(10.0, 5.0),
            money: StatRange::new(50.0, 30.0),
        }
    }
}

// ============================================================================
// Character Inventory
// ============================================================================

/// An inventory item with spawn probability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemChance {
    /// Item ID.
    pub item_id: String,
    /// Display name.
    pub item_name: String,
    /// Probability of appearing (0.0 - 1.0).
    pub probability: f32,
    /// Minimum quantity of items.
    pub min_quantity: u32,
    /// Maximum quantity of items.
    pub max_quantity: u32,
}

impl InventoryItemChance {
    /// Create a new item chance entry.
    pub fn new(item_id: impl Into<String>, name: impl Into<String>, probability: f32) -> Self {
        Self {
            item_id: item_id.into(),
            item_name: name.into(),
            probability,
            min_quantity: 1,
            max_quantity: 1,
        }
    }
}

/// Character starting inventory configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterInventory {
    /// Items that may appear in starting inventory.
    pub items: Vec<InventoryItemChance>,
    /// Equipment slots (slot name -> item id).
    pub equipped: HashMap<String, String>,
}

// ============================================================================
// Character Dialog
// ============================================================================

/// Supported dialog variables that get replaced at runtime:
/// - `{name}` - The character's display name
/// - `{player_name}` - The player character's name
/// - `{location}` - Current location/area name
/// - `{faction}` - The character's faction name
/// - `{time}` - Current time of day (morning/afternoon/evening/night)
/// - `{weather}` - Current weather condition
/// - `{day}` - Current day number
/// - `{season}` - Current season name
/// - `{item}` - Context-dependent item name (for trade dialogs)
/// - `{price}` - Context-dependent price (for trade dialogs)
pub const DIALOG_VARIABLES: &[(&str, &str)] = &[
    ("{name}", "Character's name"),
    ("{player_name}", "Player's name"),
    ("{location}", "Current location"),
    ("{faction}", "Character's faction"),
    ("{time}", "Time of day"),
    ("{weather}", "Weather condition"),
    ("{day}", "Day number"),
    ("{season}", "Season name"),
    ("{item}", "Item name (trade context)"),
    ("{price}", "Price (trade context)"),
];

/// A dialog line with conditions.
///
/// Dialog text supports variable substitution using `{variable}` syntax.
/// See [`DIALOG_VARIABLES`] for available variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogLine {
    /// Unique identifier for this line.
    pub id: String,
    /// The dialog text.
    pub text: String,
    /// Condition for when this line can be shown (empty = always).
    pub condition: String,
    /// Priority (higher = more likely to be chosen).
    pub priority: i32,
}

impl DialogLine {
    /// Create a new dialog line.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            condition: String::new(),
            priority: 0,
        }
    }
}


/// Character dialog configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterDialog {
    /// Dialog lines by category name (ad-hoc string categories).
    pub categories: HashMap<String, Vec<DialogLine>>,
    /// Voice/personality type.
    pub voice_type: String,
    /// Whether this character can be talked to.
    pub can_talk: bool,
}

impl CharacterDialog {
    /// Default dialog category names.
    pub const DEFAULT_CATEGORIES: &'static [&'static str] = &[
        "Greeting",
        "Idle",
        "About Me",
        "About This Location",
        "Trade",
        "Farewell",
    ];

    /// Create default dialog settings with standard categories and template lines.
    pub fn new() -> Self {
        let mut categories = HashMap::new();

        // Greeting category with default lines
        categories.insert("Greeting".to_string(), vec![
            DialogLine::new("greeting_1", "Hello there, traveler! Welcome to {location}."),
            DialogLine::new("greeting_2", "Ah, {player_name}! Good to see you."),
            DialogLine::new("greeting_3", "Greetings! What brings you here this {time}?"),
        ]);

        // Idle category
        categories.insert("Idle".to_string(), vec![
            DialogLine::new("idle_1", "The {weather} today reminds me of my youth..."),
            DialogLine::new("idle_2", "It's day {day} of {season}. Time flies."),
            DialogLine::new("idle_3", "*hums quietly*"),
        ]);

        // About Me category
        categories.insert("About Me".to_string(), vec![
            DialogLine::new("about_1", "I'm {name}, a proud member of {faction}."),
            DialogLine::new("about_2", "I've been living in {location} for many years now."),
        ]);

        // About This Location category
        categories.insert("About This Location".to_string(), vec![
            DialogLine::new("location_1", "{location} has been my home for as long as I can remember."),
            DialogLine::new("location_2", "This area is known for its natural beauty."),
        ]);

        // Trade category
        categories.insert("Trade".to_string(), vec![
            DialogLine::new("trade_1", "Looking to buy something? I have good prices."),
            DialogLine::new("trade_2", "The {item} will cost you {price} coins."),
            DialogLine::new("trade_3", "A pleasure doing business with you, {player_name}."),
        ]);

        // Farewell category
        categories.insert("Farewell".to_string(), vec![
            DialogLine::new("farewell_1", "Safe travels, {player_name}!"),
            DialogLine::new("farewell_2", "Until we meet again."),
            DialogLine::new("farewell_3", "May the {weather} be kind to you."),
        ]);

        Self {
            categories,
            voice_type: String::from("default"),
            can_talk: true,
        }
    }

    /// Create dialog settings for player (no dialog).
    pub fn player() -> Self {
        Self {
            categories: HashMap::new(),
            voice_type: String::from("default"),
            can_talk: false,
        }
    }
}

// ============================================================================
// Character Definition
// ============================================================================

/// Full character definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDefinition {
    /// Unique character ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Character description.
    pub description: String,
    /// Folder path in the hierarchy (e.g., "factions/bandits").
    pub folder: String,
    /// Faction ID this character belongs to.
    pub faction_id: Option<u16>,
    /// Whether this is the player character.
    pub is_player: bool,
    /// Character appearance configuration.
    pub appearance: CharacterAppearance,
    /// Character stats.
    pub stats: CharacterStats,
    /// Starting inventory.
    pub inventory: CharacterInventory,
    /// Dialog configuration.
    pub dialog: CharacterDialog,
    /// Tags for categorization and AI behavior.
    pub tags: Vec<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last modified timestamp.
    pub modified_at: String,
}

impl Default for CharacterDefinition {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::from("New Character"),
            description: String::new(),
            folder: String::from("characters"),
            faction_id: None,
            is_player: false,
            appearance: CharacterAppearance::default(),
            stats: CharacterStats::default(),
            inventory: CharacterInventory::default(),
            dialog: CharacterDialog::new(),
            tags: Vec::new(),
            created_at: now.clone(),
            modified_at: now,
        }
    }
}

impl CharacterDefinition {
    /// Create a new character with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        let mut char = Self::default();
        char.name = name.into();
        char
    }

    /// Create the player character template.
    pub fn player() -> Self {
        let mut char = Self::new("Player");
        char.is_player = true;
        char.folder = String::from("player");
        char.description = String::from("The player character");
        char.faction_id = Some(0); // Player faction
        char.dialog.can_talk = false; // Player doesn't have NPC dialog
        char
    }

    /// Get the TOML save path for this character.
    pub fn save_path(&self) -> PathBuf {
        PathBuf::from("assets/characters")
            .join(&self.folder)
            .join(format!("{}.toml", self.id))
    }
}

// ============================================================================
// Character Folder Hierarchy
// ============================================================================

/// A folder in the character hierarchy.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterFolder {
    /// Folder name.
    pub name: String,
    /// Full path.
    pub path: String,
    /// Child folders.
    pub children: Vec<CharacterFolder>,
    /// Characters in this folder.
    pub characters: Vec<String>,
    /// Whether the folder is expanded in the UI.
    #[serde(skip)]
    pub expanded: bool,
}

impl CharacterFolder {
    /// Create a new folder.
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            children: Vec::new(),
            characters: Vec::new(),
            expanded: true,
        }
    }
}

// ============================================================================
// Undo/Redo History
// ============================================================================

/// An entry in the undo/redo history.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct HistoryEntry {
    /// Description of the action.
    description: String,
    /// Snapshot of character state.
    character: CharacterDefinition,
}

/// Undo/redo history manager.
#[derive(Debug, Clone, Default)]
struct UndoHistory {
    /// Past states (for undo).
    undo_stack: Vec<HistoryEntry>,
    /// Future states (for redo).
    redo_stack: Vec<HistoryEntry>,
    /// Maximum history size.
    max_size: usize,
}

impl UndoHistory {
    /// Create a new history manager.
    fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Record a new state.
    fn record(&mut self, description: impl Into<String>, character: &CharacterDefinition) {
        self.undo_stack.push(HistoryEntry {
            description: description.into(),
            character: character.clone(),
        });

        // Clear redo stack on new action
        self.redo_stack.clear();

        // Limit history size
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last action, returning the previous state.
    fn undo(&mut self, current: &CharacterDefinition) -> Option<CharacterDefinition> {
        if let Some(entry) = self.undo_stack.pop() {
            self.redo_stack.push(HistoryEntry {
                description: String::from("Undo"),
                character: current.clone(),
            });
            Some(entry.character)
        } else {
            None
        }
    }

    /// Redo the last undone action.
    fn redo(&mut self, current: &CharacterDefinition) -> Option<CharacterDefinition> {
        if let Some(entry) = self.redo_stack.pop() {
            self.undo_stack.push(HistoryEntry {
                description: String::from("Redo"),
                character: current.clone(),
            });
            Some(entry.character)
        } else {
            None
        }
    }

    /// Check if undo is available.
    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history.
    fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

// ============================================================================
// Character Generator UI Tabs
// ============================================================================

/// Active tab in the character generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterTab {
    /// Basic info and appearance.
    #[default]
    Appearance,
    /// Character stats.
    Stats,
    /// Starting inventory.
    Inventory,
    /// Dialog configuration.
    Dialog,
}

impl CharacterTab {
    /// Get all tabs.
    pub fn all() -> &'static [Self] {
        &[Self::Appearance, Self::Stats, Self::Inventory, Self::Dialog]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Stats => "Stats",
            Self::Inventory => "Inventory",
            Self::Dialog => "Dialog",
        }
    }

    /// Get icon.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Appearance => "🎨",
            Self::Stats => "📊",
            Self::Inventory => "🎒",
            Self::Dialog => "💬",
        }
    }
}

// ============================================================================
// Character Generator State
// ============================================================================

/// The character generator panel state.
pub struct CharacterGenerator {
    /// Currently edited character.
    pub current_character: CharacterDefinition,
    /// All loaded characters.
    pub characters: HashMap<String, CharacterDefinition>,
    /// Folder hierarchy.
    pub folders: CharacterFolder,
    /// Available sprite assets.
    pub available_assets: AvailableSpriteAssets,
    /// Active tab.
    active_tab: CharacterTab,
    /// Selected folder path.
    selected_folder: String,
    /// Selected character ID.
    selected_character_id: Option<String>,
    /// Undo/redo history.
    history: UndoHistory,
    /// Whether assets have been scanned.
    assets_scanned: bool,
    /// Search filter text.
    search_filter: String,
    /// Selected sprite category for appearance tab.
    selected_layer_category: SpriteLayerCategory,
    /// Selected dialog category name.
    selected_dialog_category: String,
    /// New category name input.
    new_category_name: String,
    /// Whether there are unsaved changes.
    has_unsaved_changes: bool,
    /// Status message.
    status_message: Option<(String, std::time::Instant)>,
    /// Texture cache for loaded sprite sheets (sprite_id -> texture).
    texture_cache: HashMap<String, TextureHandle>,
    /// Loaded sprite map from character_0.toml.
    sprite_map: Option<CharacterSpriteDef>,
    /// Preview animation state.
    preview_animation: PreviewAnimationState,
}

/// Animation state for the sprite preview.
#[derive(Debug, Clone)]
struct PreviewAnimationState {
    /// Current animation action being previewed.
    current_action: AnimationAction,
    /// Current frame index.
    current_frame: usize,
    /// Last frame update time.
    last_update: Instant,
    /// Whether preview is playing.
    playing: bool,
}

impl Default for PreviewAnimationState {
    fn default() -> Self {
        Self {
            current_action: AnimationAction::IdleDown,
            current_frame: 0,
            last_update: Instant::now(),
            playing: true,
        }
    }
}

impl Default for CharacterGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterGenerator {
    /// Create a new character generator.
    pub fn new() -> Self {
        let mut folders = CharacterFolder::new("Characters", "");
        folders.children.push(CharacterFolder::new("Player", "player"));
        folders.children.push(CharacterFolder::new("NPCs", "npcs"));
        folders.children.push(CharacterFolder::new("Enemies", "enemies"));
        folders.children.push(CharacterFolder::new("Factions", "factions"));

        Self {
            current_character: CharacterDefinition::default(),
            characters: HashMap::new(),
            folders,
            available_assets: AvailableSpriteAssets::default(),
            active_tab: CharacterTab::Appearance,
            selected_folder: String::new(),
            selected_character_id: None,
            history: UndoHistory::new(50),
            assets_scanned: false,
            search_filter: String::new(),
            selected_layer_category: SpriteLayerCategory::Body,
            selected_dialog_category: String::from("Greeting"),
            new_category_name: String::new(),
            has_unsaved_changes: false,
            status_message: None,
            texture_cache: HashMap::new(),
            sprite_map: None,
            preview_animation: PreviewAnimationState::default(),
        }
    }

    /// Load the sprite map from character_0.toml.
    fn load_sprite_map(&mut self) {
        let path = PathBuf::from("assets/sprites/characters/character_0.toml");
        match CharacterSpriteDef::load_from_file(&path) {
            Ok(def) => {
                self.sprite_map = Some(def);
            }
            Err(e) => {
                tracing::warn!("Failed to load sprite map: {}", e);
            }
        }
    }

    /// Get the sprite sheet path for a layer.
    fn get_layer_sprite_path(&self, category: SpriteLayerCategory, sprite_id: &str) -> Option<PathBuf> {
        let base = PathBuf::from("assets/sprites/characters");
        let subdir = category.subdir_name();
        let path = base.join(subdir).join("48x48").join(format!("{}.png", sprite_id));
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Load a sprite texture into the cache.
    fn load_sprite_texture(&mut self, ctx: &egui::Context, sprite_id: &str, path: &Path) -> Option<TextureHandle> {
        // Check cache first
        if let Some(texture) = self.texture_cache.get(sprite_id) {
            return Some(texture.clone());
        }

        // Try to load the image
        if !path.exists() {
            return None;
        }

        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                let texture = ctx.load_texture(
                    sprite_id,
                    color_image,
                    TextureOptions::NEAREST,
                );
                self.texture_cache.insert(sprite_id.to_string(), texture.clone());
                Some(texture)
            }
            Err(e) => {
                tracing::warn!("Failed to load sprite texture {}: {}", sprite_id, e);
                None
            }
        }
    }

    /// Update the preview animation frame based on elapsed time.
    fn update_preview_animation(&mut self) {
        if !self.preview_animation.playing {
            return;
        }

        // Get FPS from sprite map
        let fps = self.sprite_map
            .as_ref()
            .and_then(|sm| sm.get_animation(self.preview_animation.current_action))
            .map(|anim| anim.fps)
            .unwrap_or(8.0);

        let frame_duration = std::time::Duration::from_secs_f32(1.0 / fps);

        if self.preview_animation.last_update.elapsed() >= frame_duration {
            // Get frame count
            let frame_count = self.sprite_map
                .as_ref()
                .and_then(|sm| sm.get_animation(self.preview_animation.current_action))
                .map(|anim| anim.frames.len())
                .unwrap_or(1);

            if frame_count > 0 {
                self.preview_animation.current_frame =
                    (self.preview_animation.current_frame + 1) % frame_count;
            }
            self.preview_animation.last_update = Instant::now();
        }
    }

    /// Get the current frame info from the sprite map.
    fn get_current_frame_info(&self) -> Option<SpriteFrame> {
        self.sprite_map
            .as_ref()
            .and_then(|sm| sm.get_animation(self.preview_animation.current_action))
            .and_then(|anim| anim.frames.get(self.preview_animation.current_frame))
            .cloned()
    }

    /// Scan for available sprite assets.
    pub fn scan_assets(&mut self) {
        let base_path = PathBuf::from("assets/sprites/characters");
        self.available_assets = AvailableSpriteAssets::scan(&base_path);
        self.load_sprite_map();
        self.assets_scanned = true;
        self.set_status("Sprite assets scanned");
    }

    /// Load all characters from the assets directory.
    pub fn load_all_characters(&mut self) {
        let base_path = PathBuf::from("assets/characters");
        if !base_path.exists() {
            // Create the directory structure
            let _ = std::fs::create_dir_all(&base_path);
            let _ = std::fs::create_dir_all(base_path.join("player"));
            let _ = std::fs::create_dir_all(base_path.join("npcs"));
            let _ = std::fs::create_dir_all(base_path.join("enemies"));
            let _ = std::fs::create_dir_all(base_path.join("factions"));
        }

        self.characters.clear();
        self.load_characters_recursive(&base_path);
        self.rebuild_folder_hierarchy();
        self.set_status(format!("Loaded {} characters", self.characters.len()));
    }

    /// Recursively load characters from a directory.
    fn load_characters_recursive(&mut self, path: &Path) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    self.load_characters_recursive(&entry_path);
                } else if entry_path.extension().map_or(false, |e| e == "toml") {
                    if let Ok(content) = std::fs::read_to_string(&entry_path) {
                        if let Ok(character) = toml::from_str::<CharacterDefinition>(&content) {
                            self.characters.insert(character.id.clone(), character);
                        }
                    }
                }
            }
        }
    }

    /// Rebuild the folder hierarchy from loaded characters.
    fn rebuild_folder_hierarchy(&mut self) {
        // Clear existing character lists in folders
        Self::clear_folder_characters_recursive(&mut self.folders);

        // Collect folder assignments first
        let assignments: Vec<(String, String)> = self.characters
            .values()
            .map(|c| (c.folder.clone(), c.id.clone()))
            .collect();

        // Add characters to their folders
        for (folder_path, character_id) in assignments {
            self.add_character_to_folder(&folder_path, &character_id);
        }
    }

    /// Clear character lists in folders recursively.
    fn clear_folder_characters_recursive(folder: &mut CharacterFolder) {
        folder.characters.clear();
        for child in &mut folder.children {
            Self::clear_folder_characters_recursive(child);
        }
    }

    /// Add a character to the appropriate folder.
    fn add_character_to_folder(&mut self, folder_path: &str, character_id: &str) {
        if let Some(folder) = self.find_or_create_folder(folder_path) {
            if !folder.characters.contains(&character_id.to_string()) {
                folder.characters.push(character_id.to_string());
            }
        }
    }

    /// Find or create a folder by path.
    fn find_or_create_folder(&mut self, path: &str) -> Option<&mut CharacterFolder> {
        if path.is_empty() {
            return Some(&mut self.folders);
        }

        let parts: Vec<&str> = path.split('/').collect();
        let mut current = &mut self.folders;

        for part in parts {
            let found_idx = current.children.iter().position(|c| c.name == part);

            if let Some(idx) = found_idx {
                current = &mut current.children[idx];
            } else {
                // Create new folder
                let new_path = if current.path.is_empty() {
                    part.to_string()
                } else {
                    format!("{}/{}", current.path, part)
                };
                current.children.push(CharacterFolder::new(part, new_path));
                let idx = current.children.len() - 1;
                current = &mut current.children[idx];
            }
        }

        Some(current)
    }

    /// Save the current character to a TOML file.
    pub fn save_current(&mut self) -> Result<(), String> {
        let path = self.current_character.save_path();

        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        // Update modified timestamp
        self.current_character.modified_at = chrono::Utc::now().to_rfc3339();

        // Serialize to TOML
        let content = toml::to_string_pretty(&self.current_character)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        // Write file
        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // Update in-memory state
        self.characters
            .insert(self.current_character.id.clone(), self.current_character.clone());
        self.has_unsaved_changes = false;
        self.set_status(format!("Saved: {}", self.current_character.name));

        Ok(())
    }

    /// Create a new character.
    pub fn new_character(&mut self) {
        self.record_history("New character");
        self.current_character = CharacterDefinition::default();
        self.current_character.folder = self.selected_folder.clone();
        self.selected_character_id = None;
        self.has_unsaved_changes = true;
    }

    /// Select and load a character.
    pub fn select_character(&mut self, id: &str) {
        if let Some(character) = self.characters.get(id) {
            self.current_character = character.clone();
            self.selected_character_id = Some(id.to_string());
            self.has_unsaved_changes = false;
            self.history.clear();
        }
    }

    /// Duplicate the current character.
    pub fn duplicate_current(&mut self) {
        let mut new_char = self.current_character.clone();
        new_char.id = uuid::Uuid::new_v4().to_string();
        new_char.name = format!("{} (Copy)", new_char.name);
        new_char.created_at = chrono::Utc::now().to_rfc3339();
        new_char.modified_at = new_char.created_at.clone();

        self.current_character = new_char;
        self.selected_character_id = None;
        self.has_unsaved_changes = true;
        self.set_status("Character duplicated");
    }

    /// Randomize the current character's appearance.
    pub fn randomize_appearance(&mut self) {
        self.record_history("Randomize appearance");

        for category in SpriteLayerCategory::all() {
            if let Some(options) = self.available_assets.options.get(category) {
                if !options.is_empty() {
                    // Pick a random option (using simple modulo for now)
                    let idx = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_micros())
                        .unwrap_or(0) as usize
                        + *category as usize * 1337)
                        % options.len();

                    if let Some(layer) = self.current_character.appearance.layers.get_mut(category) {
                        layer.sprite_id = Some(options[idx].id.clone());
                        layer.enabled = true;
                    }
                }
            }
        }

        self.has_unsaved_changes = true;
        self.set_status("Appearance randomized");
    }

    /// Randomize all character properties.
    pub fn randomize_all(&mut self) {
        self.record_history("Randomize all");
        self.randomize_appearance();
        // Stats are already randomized by their ranges when spawned
        self.set_status("Character fully randomized");
    }

    /// Record current state for undo.
    fn record_history(&mut self, description: &str) {
        self.history.record(description, &self.current_character);
    }

    /// Undo the last change.
    pub fn undo(&mut self) {
        if let Some(prev) = self.history.undo(&self.current_character) {
            self.current_character = prev;
            self.has_unsaved_changes = true;
            self.set_status("Undone");
        }
    }

    /// Redo the last undone change.
    pub fn redo(&mut self) {
        if let Some(next) = self.history.redo(&self.current_character) {
            self.current_character = next;
            self.has_unsaved_changes = true;
            self.set_status("Redone");
        }
    }

    /// Set a status message.
    fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some((message.into(), std::time::Instant::now()));
    }

    /// Render the character generator UI.
    pub fn render(&mut self, ui: &mut Ui) {
        // Scan assets on first render
        if !self.assets_scanned {
            self.scan_assets();
            self.load_all_characters();
        }

        // Set minimum height (window height - 100px)
        let available_height = ui.available_height();
        let min_height = (available_height - 100.0).max(300.0);
        ui.set_min_height(min_height);

        ui.horizontal(|ui| {
            // Left panel: folder hierarchy
            ui.vertical(|ui| {
                ui.set_width(200.0);
                ui.set_min_height(min_height - 20.0);
                self.render_hierarchy_panel(ui);
            });

            ui.separator();

            // Right panel: character editor
            ui.vertical(|ui| {
                ui.set_min_height(min_height - 20.0);
                self.render_editor_panel(ui, min_height - 150.0);
            });
        });
    }

    /// Render the folder hierarchy panel.
    fn render_hierarchy_panel(&mut self, ui: &mut Ui) {
        ui.heading("📁 Characters");

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("➕ New").clicked() {
                self.new_character();
            }
            if ui.button("🔄").on_hover_text("Reload").clicked() {
                self.load_all_characters();
            }
        });

        // Search
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_filter);
        });

        ui.separator();

        // Folder tree
        ScrollArea::vertical()
            .id_salt("char_hierarchy")
            .max_height(400.0)
            .show(ui, |ui| {
                self.render_folder_tree(ui, &self.folders.clone());
            });
    }

    /// Render a folder in the tree.
    fn render_folder_tree(&mut self, ui: &mut Ui, folder: &CharacterFolder) {
        let is_selected = self.selected_folder == folder.path;

        let header = ui.collapsing(
            RichText::new(format!("📁 {}", folder.name))
                .color(if is_selected { Color32::YELLOW } else { Color32::WHITE }),
            |ui| {
                // Child folders
                for child in &folder.children {
                    self.render_folder_tree(ui, child);
                }

                // Characters in this folder
                for char_id in &folder.characters {
                    if let Some(character) = self.characters.get(char_id) {
                        // Apply search filter
                        if !self.search_filter.is_empty()
                            && !character.name.to_lowercase().contains(&self.search_filter.to_lowercase())
                        {
                            continue;
                        }

                        let is_char_selected = self.selected_character_id.as_ref() == Some(char_id);
                        let icon = if character.is_player { "👤" } else { "🧑" };

                        if ui
                            .selectable_label(
                                is_char_selected,
                                format!("{} {}", icon, character.name),
                            )
                            .clicked()
                        {
                            self.select_character(char_id);
                        }
                    }
                }
            },
        );

        if header.header_response.clicked() {
            self.selected_folder = folder.path.clone();
        }
    }

    /// Render the character editor panel.
    fn render_editor_panel(&mut self, ui: &mut Ui, content_height: f32) {
        // Header with character name
        ui.horizontal(|ui| {
            ui.heading(format!(
                "{}{}",
                if self.current_character.is_player { "👤 " } else { "" },
                &self.current_character.name
            ));

            if self.has_unsaved_changes {
                ui.label(RichText::new("*").color(Color32::YELLOW));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾 Save").clicked() {
                    if let Err(e) = self.save_current() {
                        self.set_status(format!("Error: {}", e));
                    }
                }

                if ui.add_enabled(self.history.can_redo(), egui::Button::new("↪ Redo")).clicked() {
                    self.redo();
                }

                if ui.add_enabled(self.history.can_undo(), egui::Button::new("↩ Undo")).clicked() {
                    self.undo();
                }

                if ui.button("📋 Duplicate").clicked() {
                    self.duplicate_current();
                }

                if ui.button("🎲 Random").clicked() {
                    self.randomize_all();
                }
            });
        });

        // Status message
        if let Some((msg, time)) = &self.status_message {
            if time.elapsed().as_secs() < 3 {
                ui.label(RichText::new(msg).color(Color32::LIGHT_GREEN));
            }
        }

        ui.separator();

        // Basic info
        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut self.current_character.name).changed() {
                self.has_unsaved_changes = true;
            }

            ui.label("Folder:");
            if ui.text_edit_singleline(&mut self.current_character.folder).changed() {
                self.has_unsaved_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Description:");
            if ui.text_edit_singleline(&mut self.current_character.description).changed() {
                self.has_unsaved_changes = true;
            }
        });

        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.current_character.is_player, "Is Player").changed() {
                self.has_unsaved_changes = true;
            }

            ui.label("Faction ID:");
            let mut faction_str = self.current_character.faction_id
                .map(|f| f.to_string())
                .unwrap_or_default();
            if ui.text_edit_singleline(&mut faction_str).changed() {
                self.current_character.faction_id = faction_str.parse().ok();
                self.has_unsaved_changes = true;
            }
        });

        ui.separator();

        // Tab bar
        ui.horizontal(|ui| {
            for tab in CharacterTab::all() {
                if ui
                    .selectable_label(
                        self.active_tab == *tab,
                        format!("{} {}", tab.icon(), tab.display_name()),
                    )
                    .clicked()
                {
                    self.active_tab = *tab;
                }
            }
        });

        ui.separator();

        // Tab content with proper height
        ScrollArea::vertical()
            .id_salt("char_editor_content")
            .min_scrolled_height(content_height.max(400.0))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_height(content_height.max(400.0));
                match self.active_tab {
                    CharacterTab::Appearance => self.render_appearance_tab(ui, content_height),
                    CharacterTab::Stats => self.render_stats_tab(ui),
                    CharacterTab::Inventory => self.render_inventory_tab(ui),
                    CharacterTab::Dialog => self.render_dialog_tab(ui),
                }
            });
    }

    /// Render the appearance tab.
    fn render_appearance_tab(&mut self, ui: &mut Ui, content_height: f32) {
        let ctx = ui.ctx().clone();

        // Update animation frame
        self.update_preview_animation();

        // Use horizontal layout: left side for preview, right side for options
        ui.horizontal(|ui| {
            // Left panel: Character preview (compact)
            ui.vertical(|ui| {
                ui.set_width(160.0); // 48*3 + padding
                ui.set_min_height(content_height.max(300.0));

                ui.group(|ui| {
                    ui.heading("Preview");
                    ui.separator();

                    // Animation controls
                    ui.horizontal(|ui| {
                        if ui.button(if self.preview_animation.playing { "⏸" } else { "▶" }).clicked() {
                            self.preview_animation.playing = !self.preview_animation.playing;
                        }

                        // Animation selector - show all standard actions
                        egui::ComboBox::from_id_salt("preview_anim")
                            .selected_text(self.preview_animation.current_action.display_name())
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.set_min_width(150.0);
                                for action in AnimationAction::all_standard() {
                                    if ui.selectable_label(
                                        self.preview_animation.current_action == *action,
                                        action.display_name()
                                    ).clicked() {
                                        self.preview_animation.current_action = *action;
                                        self.preview_animation.current_frame = 0;
                                    }
                                }
                            });
                    });

                    // Show composite character preview
                    let scale = 3.0;
                    let preview_size = Vec2::new(48.0 * scale, 74.0 * scale); // Using 74 height from sprite map
                    let (rect, _response) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

                    // Draw background
                    ui.painter().rect_filled(rect, 4.0, Color32::from_gray(40));

                    // Get current animation frame info from sprite map
                    let frame_info = self.get_current_frame_info();

                    // Render each enabled layer in order
                    let render_order = [
                        SpriteLayerCategory::Body,
                        SpriteLayerCategory::Eyes,
                        SpriteLayerCategory::Outfit,
                        SpriteLayerCategory::Hairstyle,
                        SpriteLayerCategory::Accessory,
                        SpriteLayerCategory::Book,
                        SpriteLayerCategory::Smartphone,
                    ];

                    // First, collect layer info to avoid borrow issues
                    let layers_to_render: Vec<(SpriteLayerCategory, String, PathBuf)> = render_order
                        .iter()
                        .filter_map(|category| {
                            self.current_character.appearance.layers.get(category)
                                .filter(|layer| layer.enabled)
                                .and_then(|layer| layer.sprite_id.as_ref())
                                .and_then(|sprite_id| {
                                    self.get_layer_sprite_path(*category, sprite_id)
                                        .map(|path| (*category, sprite_id.clone(), path))
                                })
                        })
                        .collect();

                    let mut rendered_any = false;
                    for (_category, sprite_id, path) in &layers_to_render {
                        if let Some(texture) = self.load_sprite_texture(&ctx, sprite_id, path) {
                            if let Some(frame) = &frame_info {
                                // Calculate UV coordinates for this frame
                                let tex_size = texture.size_vec2();
                                let uv_min = egui::pos2(
                                    frame.x as f32 / tex_size.x,
                                    frame.y as f32 / tex_size.y,
                                );
                                let uv_max = egui::pos2(
                                    (frame.x + frame.width) as f32 / tex_size.x,
                                    (frame.y + frame.height) as f32 / tex_size.y,
                                );

                                // Draw the sprite frame
                                ui.painter().image(
                                    texture.id(),
                                    rect,
                                    egui::Rect::from_min_max(uv_min, uv_max),
                                    Color32::WHITE,
                                );
                                rendered_any = true;
                            }
                        }
                    }

                    if !rendered_any {
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "No layers\\nenabled",
                            egui::FontId::proportional(14.0),
                            Color32::GRAY,
                        );
                    }
                });

                ui.add_space(8.0);

                // Show frame info
                if let Some(frame) = self.get_current_frame_info() {
                    ui.label(format!("Frame {}: {}x{}",
                        self.preview_animation.current_frame,
                        frame.width, frame.height
                    ));
                }

                if ui.button("🎲 Randomize Appearance").clicked() {
                    self.randomize_appearance();
                }
            });

            ui.add_space(16.0);

            // Right panel: Layer selection
            ui.vertical(|ui| {
                ui.set_min_height(content_height.max(300.0));

        ui.separator();

        // Layer category selector
        ui.horizontal(|ui| {
            ui.label("Layer:");
            for category in SpriteLayerCategory::all() {
                if ui
                    .selectable_label(
                        self.selected_layer_category == *category,
                        format!("{} {}", category.icon(), category.display_name()),
                    )
                    .clicked()
                {
                    self.selected_layer_category = *category;
                }
            }
        });

        ui.separator();

        let category = self.selected_layer_category;

        // Get current layer enabled state and sprite_id
        let layer_enabled = self.current_character.appearance.layers
            .get(&category)
            .map(|l| l.enabled)
            .unwrap_or(true);
        let current_sprite_id = self.current_character.appearance.layers
            .get(&category)
            .and_then(|l| l.sprite_id.clone());

        // Enabled checkbox
        let mut enabled = layer_enabled;
        ui.horizontal(|ui| {
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                self.current_character.appearance.layers
                    .entry(category)
                    .or_insert_with(SelectedLayer::default)
                    .enabled = enabled;
                self.has_unsaved_changes = true;
            }
        });

        // Collect sprite options to show - show ALL variants, not just first per group
        let mut sprite_options: Vec<(String, String)> = Vec::new(); // (id, display_name)
        if let Some(options) = self.available_assets.options.get(&category) {
            for opt in options {
                sprite_options.push((opt.id.clone(), opt.display_name.clone()));
            }
        }

        if sprite_options.is_empty() {
            ui.label("No sprites found for this category");
            return;
        }

        ui.label(format!("{} options available:", sprite_options.len()));

        // Track which sprite to select
        let mut selected_sprite: Option<String> = None;

        // Grid of options
        let columns = 4;
        egui::Grid::new("sprite_options_grid")
            .num_columns(columns)
            .spacing(Vec2::new(8.0, 8.0))
            .show(ui, |ui| {
                let mut col = 0;
                for (id, display_name) in &sprite_options {
                    let is_selected = current_sprite_id.as_ref() == Some(id);

                    let btn = ui.selectable_label(
                        is_selected,
                        RichText::new(display_name).size(11.0),
                    );

                    if btn.clicked() {
                        selected_sprite = Some(id.clone());
                    }

                    col += 1;
                    if col >= columns {
                        ui.end_row();
                        col = 0;
                    }
                }
            });

        // Apply selection after the loop
        if let Some(new_sprite_id) = selected_sprite {
            self.record_history("Change sprite");
            self.current_character.appearance.layers
                .entry(category)
                .or_insert_with(SelectedLayer::default)
                .sprite_id = Some(new_sprite_id);
            self.has_unsaved_changes = true;
        }
            });
        });
    }

    /// Render the stats tab.
    fn render_stats_tab(&mut self, ui: &mut Ui) {
        ui.label("Stats are defined as ranges. When a character spawns, values are rolled within these ranges.");

        let mut changed = false;

        ui.separator();
        ui.heading("Core Stats");

        changed |= Self::render_stat_range_ui(ui, "Max Health", &mut self.current_character.stats.max_health);
        changed |= Self::render_stat_range_ui(ui, "Max Stamina", &mut self.current_character.stats.max_stamina);
        changed |= Self::render_stat_range_ui(ui, "Max Hunger", &mut self.current_character.stats.max_hunger);

        ui.separator();
        ui.heading("Combat Stats");

        changed |= Self::render_stat_range_ui(ui, "Attack", &mut self.current_character.stats.attack);
        changed |= Self::render_stat_range_ui(ui, "Defense", &mut self.current_character.stats.defense);
        changed |= Self::render_stat_range_ui(ui, "Speed", &mut self.current_character.stats.speed);
        changed |= Self::render_stat_range_ui(ui, "Crit Chance", &mut self.current_character.stats.crit_chance);

        ui.separator();
        ui.heading("Skills");

        changed |= Self::render_stat_range_ui(ui, "Mining", &mut self.current_character.stats.mining);
        changed |= Self::render_stat_range_ui(ui, "Crafting", &mut self.current_character.stats.crafting);
        changed |= Self::render_stat_range_ui(ui, "Combat", &mut self.current_character.stats.combat);
        changed |= Self::render_stat_range_ui(ui, "Farming", &mut self.current_character.stats.farming);

        ui.separator();
        ui.heading("AI / Rewards");

        changed |= Self::render_stat_range_ui(ui, "AI Difficulty", &mut self.current_character.stats.ai_difficulty);
        changed |= Self::render_stat_range_ui(ui, "XP Reward", &mut self.current_character.stats.xp_reward);
        changed |= Self::render_stat_range_ui(ui, "Money", &mut self.current_character.stats.money);

        if changed {
            self.has_unsaved_changes = true;
        }
    }

    /// Render a stat range editor. Returns true if changed.
    fn render_stat_range_ui(ui: &mut Ui, label: &str, stat: &mut StatRange) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(format!("{:12}", label));
            ui.label("Min:");
            if ui.add(egui::DragValue::new(&mut stat.min).speed(0.5)).changed() {
                changed = true;
            }
            ui.label("Max:");
            if ui.add(egui::DragValue::new(&mut stat.max).speed(0.5)).changed() {
                changed = true;
            }
            ui.label("Base:");
            if ui.add(egui::DragValue::new(&mut stat.base).speed(0.5)).changed() {
                changed = true;
            }
        });
        changed
    }

    /// Render the inventory tab.
    fn render_inventory_tab(&mut self, ui: &mut Ui) {
        ui.label("Configure starting inventory items with spawn probabilities.");
        ui.label(RichText::new("Note: No items are currently defined in the game.").color(Color32::GRAY));

        ui.separator();

        if ui.button("➕ Add Item").clicked() {
            self.current_character.inventory.items.push(InventoryItemChance::new(
                format!("item_{}", self.current_character.inventory.items.len()),
                "New Item",
                1.0,
            ));
            self.has_unsaved_changes = true;
        }

        ui.separator();

        // Item list
        let mut to_remove = None;
        for (i, item) in self.current_character.inventory.items.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label("ID:");
                if ui.text_edit_singleline(&mut item.item_id).changed() {
                    self.has_unsaved_changes = true;
                }

                ui.label("Name:");
                if ui.text_edit_singleline(&mut item.item_name).changed() {
                    self.has_unsaved_changes = true;
                }

                ui.label("Prob:");
                if ui.add(egui::Slider::new(&mut item.probability, 0.0..=1.0)).changed() {
                    self.has_unsaved_changes = true;
                }

                ui.label("Qty:");
                if ui.add(egui::DragValue::new(&mut item.min_quantity).range(1..=99)).changed() {
                    self.has_unsaved_changes = true;
                }
                ui.label("-");
                if ui.add(egui::DragValue::new(&mut item.max_quantity).range(1..=99)).changed() {
                    self.has_unsaved_changes = true;
                }

                if ui.button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });
        }

        if let Some(idx) = to_remove {
            self.current_character.inventory.items.remove(idx);
            self.has_unsaved_changes = true;
        }
    }

    /// Render the dialog tab.
    fn render_dialog_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.current_character.dialog.can_talk, "Can Talk").changed() {
                self.has_unsaved_changes = true;
            }

            ui.label("Voice Type:");
            if ui.text_edit_singleline(&mut self.current_character.dialog.voice_type).changed() {
                self.has_unsaved_changes = true;
            }
        });

        ui.separator();

        // Category selector with ability to add new categories
        ui.horizontal(|ui| {
            ui.label("Category:");

            // Collect category names to avoid borrow issues
            let category_names: Vec<String> = self.current_character.dialog.categories.keys().cloned().collect();

            for cat in &category_names {
                if ui.selectable_label(&self.selected_dialog_category == cat, cat.as_str()).clicked() {
                    self.selected_dialog_category = cat.clone();
                }
            }
        });

        // Add new category
        ui.horizontal(|ui| {
            ui.label("New Category:");
            ui.text_edit_singleline(&mut self.new_category_name);
            if ui.button("➕ Add Category").clicked() && !self.new_category_name.is_empty() {
                let name = std::mem::take(&mut self.new_category_name);
                self.current_character.dialog.categories.entry(name.clone()).or_insert_with(Vec::new);
                self.selected_dialog_category = name;
                self.has_unsaved_changes = true;
            }
        });

        ui.separator();

        let category = self.selected_dialog_category.clone();

        ui.horizontal(|ui| {
            if ui.button("➕ Add Line").clicked() {
                let lines = self.current_character.dialog.categories
                    .entry(category.clone())
                    .or_insert_with(Vec::new);
                lines.push(DialogLine::new(
                    format!("line_{}", lines.len()),
                    "New dialog line",
                ));
                self.has_unsaved_changes = true;
            }

            // Delete category button (only if not a default)
            if !CharacterDialog::DEFAULT_CATEGORIES.contains(&category.as_str()) {
                if ui.button("🗑 Delete Category").clicked() {
                    self.current_character.dialog.categories.remove(&category);
                    self.selected_dialog_category = String::from("Greeting");
                    self.has_unsaved_changes = true;
                }
            }
        });

        // Dialog lines for selected category
        let mut to_remove = None;
        if let Some(lines) = self.current_character.dialog.categories.get_mut(&category) {
            for (i, line) in lines.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        if ui.text_edit_singleline(&mut line.id).changed() {
                            self.has_unsaved_changes = true;
                        }

                        ui.label("Priority:");
                        if ui.add(egui::DragValue::new(&mut line.priority)).changed() {
                            self.has_unsaved_changes = true;
                        }

                        if ui.button("🗑").clicked() {
                            to_remove = Some(i);
                        }
                    });

                    ui.label("Text:");
                    if ui.text_edit_multiline(&mut line.text).changed() {
                        self.has_unsaved_changes = true;
                    }

                    ui.horizontal(|ui| {
                        ui.label("Condition:");
                        if ui.text_edit_singleline(&mut line.condition).changed() {
                            self.has_unsaved_changes = true;
                        }
                    });
                });
            }
        }

        if let Some(idx) = to_remove {
            if let Some(lines) = self.current_character.dialog.categories.get_mut(&category) {
                lines.remove(idx);
                self.has_unsaved_changes = true;
            }
        }
    }

    /// Get characters assigned to a faction.
    pub fn get_faction_characters(&self, faction_id: u16) -> Vec<&CharacterDefinition> {
        self.characters
            .values()
            .filter(|c| c.faction_id == Some(faction_id))
            .collect()
    }
}
