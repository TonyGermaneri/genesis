//! World interaction system for dig/place operations.
//!
//! This module provides player-world interaction including digging and placing blocks.

use genesis_common::{EntityId, ItemTypeId, WorldCoord};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::input::Vec2;
use crate::inventory::Inventory;
use crate::player::Player;

/// Errors that can occur during world interaction.
#[derive(Debug, Clone, Error)]
pub enum InteractionError {
    /// Target is out of range
    #[error("target out of range: distance {distance:.1} > max {max:.1}")]
    OutOfRange {
        /// Actual distance
        distance: f32,
        /// Maximum allowed distance
        max: f32,
    },

    /// No item in inventory to place
    #[error("no item in inventory: {0:?}")]
    NoItem(ItemTypeId),

    /// Cell is not diggable
    #[error("cell not diggable at ({x}, {y})")]
    NotDiggable {
        /// X coordinate
        x: i64,
        /// Y coordinate
        y: i64,
    },

    /// Cell is not grass (cannot be cut)
    #[error("cell at ({x}, {y}) is not grass")]
    NotGrass {
        /// X coordinate
        x: i64,
        /// Y coordinate
        y: i64,
    },

    /// Cell is not placeable (blocked)
    #[error("cannot place at ({x}, {y}): cell is blocked")]
    CellBlocked {
        /// X coordinate
        x: i64,
        /// Y coordinate
        y: i64,
    },

    /// Inventory is full
    #[error("inventory full, cannot pick up item")]
    InventoryFull,
}

/// Result type for interaction operations.
pub type InteractionResult<T> = Result<T, InteractionError>;

/// Type of cell/block in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CellType {
    /// Empty/air
    #[default]
    Air,
    /// Solid ground (dirt)
    Dirt,
    /// Stone
    Stone,
    /// Sand
    Sand,
    /// Water
    Water,
    /// Grass (tall, cuttable)
    Grass,
    /// Cut grass (recently cut, can regrow)
    CutGrass,
    /// Wood (from trees)
    Wood,
    /// Leaves
    Leaves,
    /// Ore
    Ore,
    /// Custom cell type
    Custom(u32),
}

impl CellType {
    /// Check if this cell type is solid (blocks movement).
    #[must_use]
    pub fn is_solid(self) -> bool {
        !matches!(
            self,
            CellType::Air | CellType::Water | CellType::Grass | CellType::CutGrass
        )
    }

    /// Check if this cell type is diggable.
    #[must_use]
    pub fn is_diggable(self) -> bool {
        matches!(
            self,
            CellType::Dirt
                | CellType::Stone
                | CellType::Sand
                | CellType::Grass
                | CellType::CutGrass
                | CellType::Wood
                | CellType::Leaves
                | CellType::Ore
                | CellType::Custom(_)
        )
    }

    /// Check if this cell type is liquid.
    #[must_use]
    pub fn is_liquid(self) -> bool {
        matches!(self, CellType::Water)
    }

    /// Check if this cell type is grass (cuttable).
    #[must_use]
    pub fn is_grass(self) -> bool {
        matches!(self, CellType::Grass)
    }

    /// Check if this cell type is cut grass (can regrow).
    #[must_use]
    pub fn is_cut_grass(self) -> bool {
        matches!(self, CellType::CutGrass)
    }

    /// Get the item type that this cell drops when mined.
    #[must_use]
    pub fn drop_item(self) -> Option<ItemTypeId> {
        match self {
            CellType::Air | CellType::Water | CellType::CutGrass => None,
            CellType::Dirt => Some(ItemTypeId::new(1)),
            CellType::Stone => Some(ItemTypeId::new(2)),
            CellType::Sand => Some(ItemTypeId::new(3)),
            CellType::Grass => Some(ItemTypeId::new(4)),
            CellType::Wood => Some(ItemTypeId::new(5)),
            CellType::Leaves => Some(ItemTypeId::new(6)),
            CellType::Ore => Some(ItemTypeId::new(7)),
            CellType::Custom(id) => Some(ItemTypeId::new(100 + id)),
        }
    }

    /// Get the cell type that an item places.
    #[must_use]
    pub fn from_item(item: ItemTypeId) -> Option<Self> {
        match item.raw() {
            1 => Some(CellType::Dirt),
            2 => Some(CellType::Stone),
            3 => Some(CellType::Sand),
            4 => Some(CellType::Grass),
            5 => Some(CellType::Wood),
            6 => Some(CellType::Leaves),
            7 => Some(CellType::Ore),
            id if id >= 100 => Some(CellType::Custom(id - 100)),
            _ => None,
        }
    }
}

/// Intent to modify the world (sent to kernel for execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldIntent {
    /// Dig/remove a cell
    Dig {
        /// Entity performing the action
        entity_id: EntityId,
        /// World position of the cell
        position: WorldCoord,
        /// Expected cell type (for validation)
        expected_cell: CellType,
    },
    /// Place a cell
    Place {
        /// Entity performing the action
        entity_id: EntityId,
        /// World position to place at
        position: WorldCoord,
        /// Cell type to place
        cell_type: CellType,
    },
    /// Cut grass (converts Grass to CutGrass)
    CutGrass {
        /// Entity performing the action
        entity_id: EntityId,
        /// World position of the grass
        position: WorldCoord,
    },
}

impl WorldIntent {
    /// Get the entity ID associated with this intent.
    #[must_use]
    pub fn entity_id(&self) -> EntityId {
        match self {
            WorldIntent::Dig { entity_id, .. }
            | WorldIntent::Place { entity_id, .. }
            | WorldIntent::CutGrass { entity_id, .. } => *entity_id,
        }
    }

    /// Get the world position of this intent.
    #[must_use]
    pub fn position(&self) -> WorldCoord {
        match self {
            WorldIntent::Dig { position, .. }
            | WorldIntent::Place { position, .. }
            | WorldIntent::CutGrass { position, .. } => *position,
        }
    }
}

/// Configuration for world interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionConfig {
    /// Maximum distance for interaction
    pub max_range: f32,
    /// Time to dig different cell types (in seconds)
    pub dig_times: DiggingTimes,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            max_range: 64.0,
            dig_times: DiggingTimes::default(),
        }
    }
}

/// Digging times for different cell types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiggingTimes {
    /// Time to dig dirt
    pub dirt: f32,
    /// Time to dig stone
    pub stone: f32,
    /// Time to dig sand
    pub sand: f32,
    /// Time to dig wood
    pub wood: f32,
    /// Time to dig ore
    pub ore: f32,
    /// Time to cut grass
    pub grass: f32,
    /// Default time for other types
    pub default: f32,
}

impl Default for DiggingTimes {
    fn default() -> Self {
        Self {
            dirt: 0.3,
            stone: 1.0,
            sand: 0.2,
            wood: 0.5,
            ore: 1.5,
            grass: 0.1,
            default: 0.5,
        }
    }
}

impl DiggingTimes {
    /// Get the dig time for a cell type.
    #[must_use]
    pub fn get(&self, cell: CellType) -> f32 {
        match cell {
            CellType::Dirt => self.dirt,
            CellType::Grass | CellType::CutGrass => self.grass,
            CellType::Stone => self.stone,
            CellType::Sand => self.sand,
            CellType::Wood | CellType::Leaves => self.wood,
            CellType::Ore => self.ore,
            CellType::Air | CellType::Water => 0.0,
            CellType::Custom(_) => self.default,
        }
    }
}

/// World query interface (placeholder for actual world access).
pub trait WorldQuery {
    /// Get the cell type at a world position.
    fn get_cell(&self, x: i64, y: i64) -> CellType;

    /// Check if a cell is empty (can be placed into).
    fn is_empty(&self, x: i64, y: i64) -> bool {
        self.get_cell(x, y) == CellType::Air
    }

    /// Check if a cell is solid.
    fn is_solid(&self, x: i64, y: i64) -> bool {
        self.get_cell(x, y).is_solid()
    }
}

/// Mock world for testing.
#[derive(Debug, Default)]
pub struct MockWorld {
    /// Override cells (position -> cell type)
    cells: std::collections::HashMap<(i64, i64), CellType>,
    /// Default cell type for positions not in the map
    default_cell: CellType,
}

impl MockWorld {
    /// Create a new mock world.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a cell in the mock world.
    pub fn set_cell(&mut self, x: i64, y: i64, cell: CellType) {
        self.cells.insert((x, y), cell);
    }

    /// Set the default cell type.
    pub fn set_default(&mut self, cell: CellType) {
        self.default_cell = cell;
    }
}

impl WorldQuery for MockWorld {
    fn get_cell(&self, x: i64, y: i64) -> CellType {
        self.cells
            .get(&(x, y))
            .copied()
            .unwrap_or(self.default_cell)
    }
}

/// Current digging state.
#[derive(Debug, Clone, Default)]
pub struct DiggingState {
    /// Position being dug
    pub target: Option<WorldCoord>,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Cell type being dug
    pub cell_type: CellType,
}

impl DiggingState {
    /// Create a new digging state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start digging at a position.
    pub fn start(&mut self, position: WorldCoord, cell_type: CellType) {
        self.target = Some(position);
        self.progress = 0.0;
        self.cell_type = cell_type;
    }

    /// Cancel current digging.
    pub fn cancel(&mut self) {
        self.target = None;
        self.progress = 0.0;
        self.cell_type = CellType::Air;
    }

    /// Update digging progress.
    /// Returns true if digging is complete.
    pub fn update(&mut self, dt: f32, dig_time: f32) -> bool {
        if self.target.is_some() && dig_time > 0.0 {
            self.progress += dt / dig_time;
            if self.progress >= 1.0 {
                self.progress = 1.0;
                return true;
            }
        }
        false
    }

    /// Check if currently digging.
    #[must_use]
    pub fn is_digging(&self) -> bool {
        self.target.is_some()
    }

    /// Get the completion percentage (0-100).
    #[must_use]
    pub fn percentage(&self) -> u8 {
        (self.progress * 100.0).min(100.0) as u8
    }
}

/// World interaction manager.
#[derive(Debug)]
pub struct InteractionManager {
    /// Configuration
    config: InteractionConfig,
    /// Current digging state
    digging: DiggingState,
    /// Pending intents to be processed
    pending_intents: Vec<WorldIntent>,
}

impl Default for InteractionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionManager {
    /// Create a new interaction manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: InteractionConfig::default(),
            digging: DiggingState::new(),
            pending_intents: Vec::new(),
        }
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: InteractionConfig) -> Self {
        Self {
            config,
            digging: DiggingState::new(),
            pending_intents: Vec::new(),
        }
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &InteractionConfig {
        &self.config
    }

    /// Get the current digging state.
    #[must_use]
    pub fn digging_state(&self) -> &DiggingState {
        &self.digging
    }

    /// Take pending intents (clears the list).
    pub fn take_intents(&mut self) -> Vec<WorldIntent> {
        std::mem::take(&mut self.pending_intents)
    }

    /// Convert Vec2 position to WorldCoord.
    fn vec2_to_world_coord(pos: Vec2) -> WorldCoord {
        WorldCoord::new(pos.x.floor() as i64, pos.y.floor() as i64)
    }

    /// Check if a position is in range of the player.
    fn check_range(&self, player: &Player, target: Vec2) -> InteractionResult<()> {
        let distance = player.position().distance(target);
        if distance > self.config.max_range {
            return Err(InteractionError::OutOfRange {
                distance,
                max: self.config.max_range,
            });
        }
        Ok(())
    }

    /// Try to start or continue digging at a position.
    pub fn try_dig<W: WorldQuery>(
        &mut self,
        player: &Player,
        target_pos: Vec2,
        world: &W,
        inventory: &mut Inventory,
        dt: f32,
    ) -> InteractionResult<Option<WorldIntent>> {
        // Check range
        self.check_range(player, target_pos)?;

        let world_coord = Self::vec2_to_world_coord(target_pos);
        let cell = world.get_cell(world_coord.x, world_coord.y);

        // Check if diggable
        if !cell.is_diggable() {
            self.digging.cancel();
            return Err(InteractionError::NotDiggable {
                x: world_coord.x,
                y: world_coord.y,
            });
        }

        // Check if we're digging a different block
        if let Some(current_target) = self.digging.target {
            if current_target != world_coord {
                // Started digging a new block
                self.digging.start(world_coord, cell);
            }
        } else {
            // Not currently digging, start
            self.digging.start(world_coord, cell);
        }

        // Update digging progress
        let dig_time = self.config.dig_times.get(cell);
        if self.digging.update(dt, dig_time) {
            // Digging complete!
            let intent = WorldIntent::Dig {
                entity_id: player.entity_id(),
                position: world_coord,
                expected_cell: cell,
            };

            // Try to add item to inventory
            if let Some(item) = cell.drop_item() {
                if inventory.add(item, 1).is_err() {
                    self.digging.cancel();
                    return Err(InteractionError::InventoryFull);
                }
            }

            self.digging.cancel();
            self.pending_intents.push(intent.clone());
            return Ok(Some(intent));
        }

        Ok(None)
    }

    /// Cancel current digging.
    pub fn cancel_dig(&mut self) {
        self.digging.cancel();
    }

    /// Try to place an item at a position.
    pub fn try_place<W: WorldQuery>(
        &mut self,
        player: &Player,
        target_pos: Vec2,
        item: ItemTypeId,
        world: &W,
        inventory: &mut Inventory,
    ) -> InteractionResult<WorldIntent> {
        // Check range
        self.check_range(player, target_pos)?;

        let world_coord = Self::vec2_to_world_coord(target_pos);

        // Check if cell is empty
        if !world.is_empty(world_coord.x, world_coord.y) {
            return Err(InteractionError::CellBlocked {
                x: world_coord.x,
                y: world_coord.y,
            });
        }

        // Check if item can be placed
        let cell_type = CellType::from_item(item).ok_or(InteractionError::NoItem(item))?;

        // Check inventory has item
        if inventory.count(item) == 0 {
            return Err(InteractionError::NoItem(item));
        }

        // Remove item from inventory
        if inventory.remove(item, 1).is_err() {
            return Err(InteractionError::NoItem(item));
        }

        let intent = WorldIntent::Place {
            entity_id: player.entity_id(),
            position: world_coord,
            cell_type,
        };

        self.pending_intents.push(intent.clone());
        Ok(intent)
    }

    /// Try to cut grass at a position (E key interaction).
    ///
    /// When grass is cut:
    /// - Converts Grass cell to CutGrass
    /// - Awards 1 grass item to the player
    /// - The CutGrass cell can regrow over time (managed externally)
    pub fn try_cut_grass<W: WorldQuery>(
        &mut self,
        player: &Player,
        target_pos: Vec2,
        world: &W,
        inventory: &mut Inventory,
    ) -> InteractionResult<WorldIntent> {
        // Check range
        self.check_range(player, target_pos)?;

        let world_coord = Self::vec2_to_world_coord(target_pos);
        let cell = world.get_cell(world_coord.x, world_coord.y);

        // Check if it's grass
        if !cell.is_grass() {
            return Err(InteractionError::NotGrass {
                x: world_coord.x,
                y: world_coord.y,
            });
        }

        // Award grass item to inventory
        let grass_item = ItemTypeId::new(4); // Grass item ID
        if inventory.add(grass_item, 1).is_err() {
            return Err(InteractionError::InventoryFull);
        }

        let intent = WorldIntent::CutGrass {
            entity_id: player.entity_id(),
            position: world_coord,
        };

        self.pending_intents.push(intent.clone());
        Ok(intent)
    }

    /// Process a full interaction tick.
    /// Call this each frame with the current input state.
    pub fn update<W: WorldQuery>(
        &mut self,
        player: &Player,
        input: &crate::input::Input,
        world: &W,
        inventory: &mut Inventory,
        dt: f32,
        selected_item: Option<ItemTypeId>,
    ) -> Vec<InteractionResult<Option<WorldIntent>>> {
        let mut results = Vec::new();

        // Handle digging (primary action held)
        if input.primary_action {
            let result = self.try_dig(player, input.mouse_world_pos, world, inventory, dt);
            results.push(result);
        } else if self.digging.is_digging() {
            // Released button, cancel dig
            self.cancel_dig();
        }

        // Handle placing (secondary action just pressed)
        if input.secondary_action_just_pressed {
            if let Some(item) = selected_item {
                let result = self.try_place(player, input.mouse_world_pos, item, world, inventory);
                results.push(result.map(Some));
            }
        }

        results
    }
}

// ============================================================
// Interaction Handler (for terrain manipulation integration)
// ============================================================

/// Interaction mode for different player actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InteractionMode {
    /// Normal mode - interact with objects
    #[default]
    Normal,
    /// Dig mode - primary action performs digging
    Dig,
    /// Place mode - primary action places material
    Place,
    /// Inspect mode - show cell info
    Inspect,
}

impl InteractionMode {
    /// Get display name for the mode.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Dig => "Dig",
            Self::Place => "Place",
            Self::Inspect => "Inspect",
        }
    }

    /// Cycle to the next mode.
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::Normal => Self::Dig,
            Self::Dig => Self::Place,
            Self::Place => Self::Inspect,
            Self::Inspect => Self::Normal,
        }
    }

    /// Cycle to the previous mode.
    #[must_use]
    pub fn prev(&self) -> Self {
        match self {
            Self::Normal => Self::Inspect,
            Self::Dig => Self::Normal,
            Self::Place => Self::Dig,
            Self::Inspect => Self::Place,
        }
    }
}

/// Result of an interaction action.
#[derive(Debug, Clone)]
pub enum TerrainInteractionResult {
    /// Terrain was modified at these positions
    TerrainModified(Vec<(i32, i32)>),
    /// Item was picked up
    ItemPickedUp(ItemTypeId),
    /// Object was interacted with
    ObjectInteracted(EntityId),
    /// Grass was cut at this position
    GrassCut {
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
        /// Item awarded to player
        item: ItemTypeId,
    },
    /// Cell inspection result
    CellInspected {
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
        /// Cell type at position
        cell_type: CellType,
    },
    /// Nothing happened
    Nothing,
}

impl TerrainInteractionResult {
    /// Check if this result indicates something happened.
    #[must_use]
    pub fn is_something(&self) -> bool {
        !matches!(self, Self::Nothing)
    }

    /// Get the number of cells modified (if terrain modified).
    #[must_use]
    pub fn modified_count(&self) -> usize {
        match self {
            Self::TerrainModified(cells) => cells.len(),
            _ => 0,
        }
    }
}

/// Player interaction handler for terrain manipulation.
///
/// This struct wraps the interaction and terrain manipulation systems
/// to provide a unified interface for player actions.
#[derive(Debug)]
pub struct InteractionHandler {
    /// Interaction manager for dig/place
    manager: InteractionManager,
    /// Current interaction mode
    interaction_mode: InteractionMode,
    /// Selected material for placing (material ID)
    selected_material: u16,
    /// Brush radius for dig/place
    brush_radius: f32,
    /// Cooldown between actions
    action_cooldown: f32,
    /// Current cooldown timer
    cooldown_timer: f32,
}

impl Default for InteractionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Default brush radius for interactions.
const DEFAULT_HANDLER_BRUSH_RADIUS: f32 = 1.5;
/// Default action cooldown in seconds.
const DEFAULT_HANDLER_COOLDOWN: f32 = 0.1;

impl InteractionHandler {
    /// Create a new interaction handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: InteractionManager::new(),
            interaction_mode: InteractionMode::Normal,
            selected_material: 1, // Dirt
            brush_radius: DEFAULT_HANDLER_BRUSH_RADIUS,
            action_cooldown: DEFAULT_HANDLER_COOLDOWN,
            cooldown_timer: 0.0,
        }
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: InteractionConfig) -> Self {
        Self {
            manager: InteractionManager::with_config(config),
            interaction_mode: InteractionMode::Normal,
            selected_material: 1,
            brush_radius: DEFAULT_HANDLER_BRUSH_RADIUS,
            action_cooldown: DEFAULT_HANDLER_COOLDOWN,
            cooldown_timer: 0.0,
        }
    }

    /// Get current interaction mode.
    #[must_use]
    pub fn mode(&self) -> InteractionMode {
        self.interaction_mode
    }

    /// Set interaction mode.
    pub fn set_mode(&mut self, mode: InteractionMode) {
        self.interaction_mode = mode;
    }

    /// Cycle to next interaction mode.
    pub fn next_mode(&mut self) {
        self.interaction_mode = self.interaction_mode.next();
    }

    /// Cycle to previous interaction mode.
    pub fn prev_mode(&mut self) {
        self.interaction_mode = self.interaction_mode.prev();
    }

    /// Get selected material.
    #[must_use]
    pub fn selected_material(&self) -> u16 {
        self.selected_material
    }

    /// Set selected material.
    pub fn set_selected_material(&mut self, material: u16) {
        self.selected_material = material;
    }

    /// Get brush radius.
    #[must_use]
    pub fn brush_radius(&self) -> f32 {
        self.brush_radius
    }

    /// Set brush radius.
    pub fn set_brush_radius(&mut self, radius: f32) {
        self.brush_radius = radius.clamp(0.5, 10.0);
    }

    /// Check if can perform action (cooldown elapsed).
    #[must_use]
    pub fn can_act(&self) -> bool {
        self.cooldown_timer <= 0.0
    }

    /// Handle primary action (left click).
    pub fn primary_action<W: WorldQuery>(
        &mut self,
        player: &Player,
        world_pos: Vec2,
        world: &W,
        inventory: &mut crate::inventory::Inventory,
        dt: f32,
    ) -> TerrainInteractionResult {
        if !self.can_act() {
            return TerrainInteractionResult::Nothing;
        }

        match self.interaction_mode {
            InteractionMode::Normal => {
                // Normal mode - interact with objects (placeholder)
                TerrainInteractionResult::Nothing
            },
            InteractionMode::Dig => {
                // Dig terrain
                match self
                    .manager
                    .try_dig(player, world_pos, world, inventory, dt)
                {
                    Ok(Some(intent)) => {
                        self.cooldown_timer = self.action_cooldown;
                        let pos = intent.position();
                        TerrainInteractionResult::TerrainModified(vec![(
                            pos.x as i32,
                            pos.y as i32,
                        )])
                    },
                    Ok(None) | Err(_) => TerrainInteractionResult::Nothing, // Still digging or error
                }
            },
            InteractionMode::Place => {
                // Place terrain (using selected material as item ID)
                let item = ItemTypeId::new(u32::from(self.selected_material));
                match self
                    .manager
                    .try_place(player, world_pos, item, world, inventory)
                {
                    Ok(intent) => {
                        self.cooldown_timer = self.action_cooldown;
                        let pos = intent.position();
                        TerrainInteractionResult::TerrainModified(vec![(
                            pos.x as i32,
                            pos.y as i32,
                        )])
                    },
                    Err(_) => TerrainInteractionResult::Nothing,
                }
            },
            InteractionMode::Inspect => {
                // Inspect cell
                let x = world_pos.x.floor() as i64;
                let y = world_pos.y.floor() as i64;
                let cell_type = world.get_cell(x, y);
                TerrainInteractionResult::CellInspected {
                    x: x as i32,
                    y: y as i32,
                    cell_type,
                }
            },
        }
    }

    /// Handle secondary action (right click).
    pub fn secondary_action<W: WorldQuery>(
        &mut self,
        player: &Player,
        world_pos: Vec2,
        world: &W,
        inventory: &mut crate::inventory::Inventory,
    ) -> TerrainInteractionResult {
        if !self.can_act() {
            return TerrainInteractionResult::Nothing;
        }

        match self.interaction_mode {
            InteractionMode::Normal | InteractionMode::Place => {
                // Right click places in normal/place mode
                let item = ItemTypeId::new(u32::from(self.selected_material));
                match self
                    .manager
                    .try_place(player, world_pos, item, world, inventory)
                {
                    Ok(intent) => {
                        self.cooldown_timer = self.action_cooldown;
                        let pos = intent.position();
                        TerrainInteractionResult::TerrainModified(vec![(
                            pos.x as i32,
                            pos.y as i32,
                        )])
                    },
                    Err(_) => TerrainInteractionResult::Nothing,
                }
            },
            InteractionMode::Dig => {
                // Right click in dig mode also places
                let item = ItemTypeId::new(u32::from(self.selected_material));
                match self
                    .manager
                    .try_place(player, world_pos, item, world, inventory)
                {
                    Ok(intent) => {
                        self.cooldown_timer = self.action_cooldown;
                        let pos = intent.position();
                        TerrainInteractionResult::TerrainModified(vec![(
                            pos.x as i32,
                            pos.y as i32,
                        )])
                    },
                    Err(_) => TerrainInteractionResult::Nothing,
                }
            },
            InteractionMode::Inspect => {
                // Right click in inspect mode does nothing special
                TerrainInteractionResult::Nothing
            },
        }
    }

    /// Handle interact action (E key) - used for cutting grass.
    ///
    /// This action attempts to interact with objects near the player,
    /// primarily for cutting grass.
    pub fn interact_action<W: WorldQuery>(
        &mut self,
        player: &Player,
        world_pos: Vec2,
        world: &W,
        inventory: &mut crate::inventory::Inventory,
    ) -> TerrainInteractionResult {
        if !self.can_act() {
            return TerrainInteractionResult::Nothing;
        }

        // Try to cut grass at the position
        match self
            .manager
            .try_cut_grass(player, world_pos, world, inventory)
        {
            Ok(intent) => {
                self.cooldown_timer = self.action_cooldown;
                let pos = intent.position();
                TerrainInteractionResult::GrassCut {
                    x: pos.x as i32,
                    y: pos.y as i32,
                    item: ItemTypeId::new(4), // Grass item ID
                }
            },
            Err(_) => TerrainInteractionResult::Nothing,
        }
    }

    /// Update handler (cooldowns, etc).
    pub fn update(&mut self, dt: f32) {
        self.cooldown_timer = (self.cooldown_timer - dt).max(0.0);
    }

    /// Cancel any ongoing action (e.g., digging).
    pub fn cancel(&mut self) {
        self.manager.cancel_dig();
    }

    /// Get reference to internal interaction manager.
    #[must_use]
    pub fn manager(&self) -> &InteractionManager {
        &self.manager
    }

    /// Get mutable reference to internal interaction manager.
    pub fn manager_mut(&mut self) -> &mut InteractionManager {
        &mut self.manager
    }

    /// Get the digging progress (0.0 to 1.0).
    #[must_use]
    pub fn digging_progress(&self) -> f32 {
        self.manager.digging_state().progress
    }

    /// Check if currently digging.
    #[must_use]
    pub fn is_digging(&self) -> bool {
        self.manager.digging_state().is_digging()
    }
}

// ============================================================
// Grass Regrowth System
// ============================================================

/// Default regrowth time for cut grass in seconds.
pub const DEFAULT_GRASS_REGROWTH_TIME: f32 = 60.0;

/// Configuration for grass regrowth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassRegrowthConfig {
    /// Time in seconds for grass to regrow.
    pub regrowth_time: f32,
    /// Whether regrowth is affected by weather (rain speeds up).
    pub weather_affected: bool,
    /// Multiplier when raining (1.0 = normal, 0.5 = twice as fast).
    pub rain_multiplier: f32,
}

impl Default for GrassRegrowthConfig {
    fn default() -> Self {
        Self {
            regrowth_time: DEFAULT_GRASS_REGROWTH_TIME,
            weather_affected: true,
            rain_multiplier: 0.5,
        }
    }
}

/// Tracks a single cut grass cell's regrowth progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassRegrowthTimer {
    /// World position of the cut grass.
    pub position: WorldCoord,
    /// Time remaining until regrowth (in seconds).
    pub time_remaining: f32,
}

impl GrassRegrowthTimer {
    /// Create a new regrowth timer.
    #[must_use]
    pub fn new(position: WorldCoord, regrowth_time: f32) -> Self {
        Self {
            position,
            time_remaining: regrowth_time,
        }
    }

    /// Update the timer and return true if regrowth is complete.
    pub fn update(&mut self, dt: f32, multiplier: f32) -> bool {
        self.time_remaining -= dt * multiplier;
        self.time_remaining <= 0.0
    }

    /// Get the regrowth progress (0.0 = just cut, 1.0 = ready to regrow).
    #[must_use]
    pub fn progress(&self, total_time: f32) -> f32 {
        1.0 - (self.time_remaining / total_time).clamp(0.0, 1.0)
    }
}

/// Manages grass regrowth across the world.
///
/// When grass is cut (via E key interaction), it converts to CutGrass.
/// This manager tracks all cut grass positions and their timers.
/// When a timer expires, it generates an intent to regrow the grass.
#[derive(Debug, Default)]
pub struct GrassRegrowthManager {
    /// Configuration for regrowth behavior.
    config: GrassRegrowthConfig,
    /// Active regrowth timers.
    timers: Vec<GrassRegrowthTimer>,
    /// Pending regrowth intents (positions that should convert back to grass).
    pending_regrowth: Vec<WorldCoord>,
}

impl GrassRegrowthManager {
    /// Create a new grass regrowth manager with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GrassRegrowthConfig::default(),
            timers: Vec::new(),
            pending_regrowth: Vec::new(),
        }
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: GrassRegrowthConfig) -> Self {
        Self {
            config,
            timers: Vec::new(),
            pending_regrowth: Vec::new(),
        }
    }

    /// Register a new cut grass position for regrowth tracking.
    pub fn register_cut_grass(&mut self, position: WorldCoord) {
        // Don't register duplicates
        if self.timers.iter().any(|t| t.position == position) {
            return;
        }
        self.timers
            .push(GrassRegrowthTimer::new(position, self.config.regrowth_time));
    }

    /// Update all regrowth timers.
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds.
    /// * `is_raining` - Whether it's currently raining (affects regrowth speed).
    pub fn update(&mut self, dt: f32, is_raining: bool) {
        let multiplier = if self.config.weather_affected && is_raining {
            1.0 / self.config.rain_multiplier
        } else {
            1.0
        };

        // Update timers and collect completed ones
        let mut completed_indices = Vec::new();
        for (idx, timer) in self.timers.iter_mut().enumerate() {
            if timer.update(dt, multiplier) {
                completed_indices.push(idx);
                self.pending_regrowth.push(timer.position);
            }
        }

        // Remove completed timers (in reverse to preserve indices)
        for idx in completed_indices.into_iter().rev() {
            self.timers.swap_remove(idx);
        }
    }

    /// Take pending regrowth positions (clears the list).
    pub fn take_pending_regrowth(&mut self) -> Vec<WorldCoord> {
        std::mem::take(&mut self.pending_regrowth)
    }

    /// Get the number of active regrowth timers.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.timers.len()
    }

    /// Check if a position has an active regrowth timer.
    #[must_use]
    pub fn has_timer(&self, position: WorldCoord) -> bool {
        self.timers.iter().any(|t| t.position == position)
    }

    /// Get regrowth progress for a position (0.0 to 1.0, or None if not tracked).
    #[must_use]
    pub fn get_progress(&self, position: WorldCoord) -> Option<f32> {
        self.timers
            .iter()
            .find(|t| t.position == position)
            .map(|t| t.progress(self.config.regrowth_time))
    }

    /// Get reference to configuration.
    #[must_use]
    pub fn config(&self) -> &GrassRegrowthConfig {
        &self.config
    }

    /// Clear all timers.
    pub fn clear(&mut self) {
        self.timers.clear();
        self.pending_regrowth.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_player() -> Player {
        Player::new(Vec2::new(100.0, 100.0))
    }

    fn create_test_inventory() -> Inventory {
        Inventory::new(10)
    }

    #[test]
    fn test_cell_type_properties() {
        assert!(!CellType::Air.is_solid());
        assert!(CellType::Dirt.is_solid());
        assert!(CellType::Stone.is_solid());
        assert!(!CellType::Water.is_solid());

        assert!(!CellType::Air.is_diggable());
        assert!(CellType::Dirt.is_diggable());
        assert!(CellType::Stone.is_diggable());
        assert!(!CellType::Water.is_diggable());

        assert!(!CellType::Air.is_liquid());
        assert!(CellType::Water.is_liquid());
    }

    #[test]
    fn test_cell_item_conversion() {
        let dirt = CellType::Dirt;
        let item = dirt.drop_item().expect("should drop item");
        let back = CellType::from_item(item).expect("should convert back");
        assert_eq!(back, dirt);
    }

    #[test]
    fn test_custom_cell_item_conversion() {
        let custom = CellType::Custom(42);
        let item = custom.drop_item().expect("should drop item");
        assert_eq!(item.raw(), 142);

        let back = CellType::from_item(item).expect("should convert back");
        assert_eq!(back, CellType::Custom(42));
    }

    #[test]
    fn test_digging_state() {
        let mut state = DiggingState::new();
        assert!(!state.is_digging());

        state.start(WorldCoord::new(10, 20), CellType::Dirt);
        assert!(state.is_digging());
        assert_eq!(state.percentage(), 0);

        // Partial progress
        let complete = state.update(0.15, 0.3);
        assert!(!complete);
        assert_eq!(state.percentage(), 50);

        // Complete
        let complete = state.update(0.15, 0.3);
        assert!(complete);
        assert_eq!(state.percentage(), 100);
    }

    #[test]
    fn test_digging_cancel() {
        let mut state = DiggingState::new();
        state.start(WorldCoord::new(10, 20), CellType::Dirt);
        state.update(0.1, 0.3);

        state.cancel();
        assert!(!state.is_digging());
        assert_eq!(state.percentage(), 0);
    }

    #[test]
    fn test_mock_world() {
        let mut world = MockWorld::new();
        world.set_default(CellType::Air);
        world.set_cell(10, 20, CellType::Dirt);

        assert_eq!(world.get_cell(10, 20), CellType::Dirt);
        assert_eq!(world.get_cell(0, 0), CellType::Air);
        assert!(world.is_empty(0, 0));
        assert!(!world.is_empty(10, 20));
    }

    #[test]
    fn test_interaction_manager_creation() {
        let mut manager = InteractionManager::new();
        assert!(!manager.digging_state().is_digging());
        assert!(manager.take_intents().is_empty());
    }

    #[test]
    fn test_dig_out_of_range() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let mut world = MockWorld::new();
        world.set_cell(500, 500, CellType::Dirt);

        let result = manager.try_dig(
            &player,
            Vec2::new(500.0, 500.0),
            &world,
            &mut inventory,
            0.1,
        );
        assert!(matches!(result, Err(InteractionError::OutOfRange { .. })));
    }

    #[test]
    fn test_dig_not_diggable() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let world = MockWorld::new(); // Default is Air

        let result = manager.try_dig(
            &player,
            Vec2::new(100.0, 110.0),
            &world,
            &mut inventory,
            0.1,
        );
        assert!(matches!(result, Err(InteractionError::NotDiggable { .. })));
    }

    #[test]
    fn test_dig_success() {
        let mut manager = InteractionManager::with_config(InteractionConfig {
            max_range: 100.0,
            dig_times: DiggingTimes {
                dirt: 0.1,
                ..Default::default()
            },
        });
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let mut world = MockWorld::new();
        world.set_cell(100, 110, CellType::Dirt);

        // First tick - starts digging
        let result = manager.try_dig(
            &player,
            Vec2::new(100.0, 110.0),
            &world,
            &mut inventory,
            0.05,
        );
        assert!(result.is_ok());
        assert!(result.as_ref().ok().and_then(|r| r.as_ref()).is_none()); // Not complete yet
        assert!(manager.digging_state().is_digging());

        // Second tick - completes
        let result = manager.try_dig(
            &player,
            Vec2::new(100.0, 110.0),
            &world,
            &mut inventory,
            0.1,
        );
        assert!(result.is_ok());
        let intent = result.ok().flatten().expect("should have intent");

        assert!(matches!(intent, WorldIntent::Dig { .. }));
        assert_eq!(inventory.count(ItemTypeId::new(1)), 1); // Got dirt
    }

    #[test]
    fn test_dig_inventory_full() {
        let mut manager = InteractionManager::with_config(InteractionConfig {
            max_range: 100.0,
            dig_times: DiggingTimes {
                dirt: 0.1,
                ..Default::default()
            },
        });
        let player = create_test_player();
        let mut inventory = Inventory::with_stack_limit(1, 1); // Very limited inventory
        let mut world = MockWorld::new();
        world.set_cell(100, 110, CellType::Dirt);

        // Fill inventory
        let _ = inventory.add(ItemTypeId::new(1), 1);

        // Complete dig in one go
        let result = manager.try_dig(
            &player,
            Vec2::new(100.0, 110.0),
            &world,
            &mut inventory,
            1.0,
        );
        assert!(matches!(result, Err(InteractionError::InventoryFull)));
    }

    #[test]
    fn test_place_success() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let _ = inventory.add(ItemTypeId::new(1), 5); // Add dirt
        let world = MockWorld::new();

        let result = manager.try_place(
            &player,
            Vec2::new(100.0, 110.0),
            ItemTypeId::new(1),
            &world,
            &mut inventory,
        );
        assert!(result.is_ok());

        let intent = result.expect("should succeed");
        assert!(matches!(
            intent,
            WorldIntent::Place {
                cell_type: CellType::Dirt,
                ..
            }
        ));
        assert_eq!(inventory.count(ItemTypeId::new(1)), 4); // Used one
    }

    #[test]
    fn test_place_cell_blocked() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let _ = inventory.add(ItemTypeId::new(1), 5);
        let mut world = MockWorld::new();
        world.set_cell(100, 110, CellType::Stone); // Blocked

        let result = manager.try_place(
            &player,
            Vec2::new(100.0, 110.0),
            ItemTypeId::new(1),
            &world,
            &mut inventory,
        );
        assert!(matches!(result, Err(InteractionError::CellBlocked { .. })));
    }

    #[test]
    fn test_place_no_item() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let world = MockWorld::new();

        let result = manager.try_place(
            &player,
            Vec2::new(100.0, 110.0),
            ItemTypeId::new(1),
            &world,
            &mut inventory,
        );
        assert!(matches!(result, Err(InteractionError::NoItem(_))));
    }

    #[test]
    fn test_place_out_of_range() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let _ = inventory.add(ItemTypeId::new(1), 5);
        let world = MockWorld::new();

        let result = manager.try_place(
            &player,
            Vec2::new(500.0, 500.0),
            ItemTypeId::new(1),
            &world,
            &mut inventory,
        );
        assert!(matches!(result, Err(InteractionError::OutOfRange { .. })));
    }

    #[test]
    fn test_world_intent_accessors() {
        let entity_id = EntityId::new();
        let position = WorldCoord::new(10, 20);

        let dig_intent = WorldIntent::Dig {
            entity_id,
            position,
            expected_cell: CellType::Dirt,
        };
        assert_eq!(dig_intent.entity_id(), entity_id);
        assert_eq!(dig_intent.position(), position);

        let place_intent = WorldIntent::Place {
            entity_id,
            position,
            cell_type: CellType::Stone,
        };
        assert_eq!(place_intent.entity_id(), entity_id);
        assert_eq!(place_intent.position(), position);
    }

    #[test]
    fn test_digging_times() {
        let times = DiggingTimes::default();
        assert!(times.get(CellType::Sand) < times.get(CellType::Stone));
        assert_eq!(times.get(CellType::Air), 0.0);
        assert!(times.get(CellType::Custom(99)) > 0.0);
    }

    #[test]
    fn test_pending_intents() {
        let mut manager = InteractionManager::with_config(InteractionConfig {
            max_range: 100.0,
            dig_times: DiggingTimes {
                dirt: 0.01,
                ..Default::default()
            },
        });
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let mut world = MockWorld::new();
        world.set_cell(100, 110, CellType::Dirt);

        // Dig something
        let _ = manager.try_dig(
            &player,
            Vec2::new(100.0, 110.0),
            &world,
            &mut inventory,
            1.0,
        );

        let intents = manager.take_intents();
        assert_eq!(intents.len(), 1);

        // Should be empty after taking
        let intents = manager.take_intents();
        assert!(intents.is_empty());
    }

    #[test]
    fn test_dig_different_block_resets_progress() {
        let mut manager = InteractionManager::with_config(InteractionConfig {
            max_range: 100.0,
            dig_times: DiggingTimes {
                dirt: 1.0,
                ..Default::default()
            },
        });
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let mut world = MockWorld::new();
        world.set_cell(100, 110, CellType::Dirt);
        world.set_cell(100, 120, CellType::Dirt);

        // Start digging first block
        let _ = manager.try_dig(
            &player,
            Vec2::new(100.0, 110.0),
            &world,
            &mut inventory,
            0.5,
        );
        assert!(manager.digging_state().progress > 0.0);

        // Switch to different block - should reset
        let _ = manager.try_dig(
            &player,
            Vec2::new(100.0, 120.0),
            &world,
            &mut inventory,
            0.1,
        );
        assert!(manager.digging_state().progress < 0.5); // Reset to small value
    }

    #[test]
    fn test_grass_cell_properties() {
        assert!(CellType::Grass.is_grass());
        assert!(!CellType::CutGrass.is_grass());
        assert!(CellType::CutGrass.is_cut_grass());
        assert!(!CellType::Grass.is_cut_grass());

        // Grass is not solid (player can walk through)
        assert!(!CellType::Grass.is_solid());
        assert!(!CellType::CutGrass.is_solid());

        // Grass drops an item, cut grass does not
        assert!(CellType::Grass.drop_item().is_some());
        assert!(CellType::CutGrass.drop_item().is_none());
    }

    #[test]
    fn test_cut_grass_success() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let mut world = MockWorld::new();
        world.set_cell(100, 110, CellType::Grass);

        let result =
            manager.try_cut_grass(&player, Vec2::new(100.0, 110.0), &world, &mut inventory);
        assert!(result.is_ok());

        // Player should have received grass item
        assert_eq!(inventory.count(ItemTypeId::new(4)), 1);

        // Should have generated CutGrass intent
        let intents = manager.take_intents();
        assert_eq!(intents.len(), 1);
        assert!(matches!(intents[0], WorldIntent::CutGrass { .. }));
    }

    #[test]
    fn test_cut_grass_not_grass() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let mut world = MockWorld::new();
        world.set_cell(100, 110, CellType::Dirt);

        let result =
            manager.try_cut_grass(&player, Vec2::new(100.0, 110.0), &world, &mut inventory);
        assert!(matches!(result, Err(InteractionError::NotGrass { .. })));
    }

    #[test]
    fn test_cut_grass_out_of_range() {
        let mut manager = InteractionManager::new();
        let player = create_test_player();
        let mut inventory = create_test_inventory();
        let mut world = MockWorld::new();
        world.set_cell(500, 500, CellType::Grass);

        let result =
            manager.try_cut_grass(&player, Vec2::new(500.0, 500.0), &world, &mut inventory);
        assert!(matches!(result, Err(InteractionError::OutOfRange { .. })));
    }

    #[test]
    fn test_grass_regrowth_timer() {
        let mut timer = GrassRegrowthTimer::new(WorldCoord::new(10, 20), 10.0);
        assert!(!timer.update(5.0, 1.0)); // Not complete
        assert!((timer.progress(10.0) - 0.5).abs() < 0.01);

        assert!(timer.update(5.0, 1.0)); // Complete
        assert!((timer.progress(10.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_grass_regrowth_manager() {
        let mut manager = GrassRegrowthManager::with_config(GrassRegrowthConfig {
            regrowth_time: 10.0,
            weather_affected: false,
            rain_multiplier: 1.0,
        });

        let pos = WorldCoord::new(10, 20);
        manager.register_cut_grass(pos);
        assert_eq!(manager.active_count(), 1);
        assert!(manager.has_timer(pos));

        // Update but not complete
        manager.update(5.0, false);
        assert_eq!(manager.active_count(), 1);
        assert!(manager.take_pending_regrowth().is_empty());

        // Update to completion
        manager.update(5.0, false);
        assert_eq!(manager.active_count(), 0);
        let regrown = manager.take_pending_regrowth();
        assert_eq!(regrown.len(), 1);
        assert_eq!(regrown[0], pos);
    }

    #[test]
    fn test_grass_regrowth_rain_speedup() {
        let mut manager = GrassRegrowthManager::with_config(GrassRegrowthConfig {
            regrowth_time: 10.0,
            weather_affected: true,
            rain_multiplier: 0.5, // Rain makes regrowth 2x faster
        });

        let pos = WorldCoord::new(10, 20);
        manager.register_cut_grass(pos);

        // With rain, 5 seconds should complete (effective 10 seconds)
        manager.update(5.0, true);
        assert_eq!(manager.active_count(), 0);
        assert_eq!(manager.take_pending_regrowth().len(), 1);
    }

    #[test]
    fn test_grass_regrowth_no_duplicates() {
        let mut manager = GrassRegrowthManager::new();
        let pos = WorldCoord::new(10, 20);

        manager.register_cut_grass(pos);
        manager.register_cut_grass(pos); // Duplicate
        assert_eq!(manager.active_count(), 1);
    }
}
