//! Terrain manipulation system for dig/place operations.
//!
//! This module provides terrain modification capabilities including digging,
//! placing, and filling cells with different materials.

use serde::{Deserialize, Serialize};

/// Actions the player can perform on terrain.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TerrainAction {
    /// Dig out terrain in a radius
    Dig {
        /// Radius of dig area
        radius: f32,
    },
    /// Place material in a radius
    Place {
        /// Material ID to place
        material: u16,
        /// Radius of place area
        radius: f32,
    },
    /// Fill only air cells with material
    Fill {
        /// Material ID to fill with
        material: u16,
        /// Radius of fill area
        radius: f32,
    },
}

impl TerrainAction {
    /// Create a new dig action with the given radius.
    #[must_use]
    pub fn dig(radius: f32) -> Self {
        Self::Dig { radius }
    }

    /// Create a new place action with the given material and radius.
    #[must_use]
    pub fn place(material: u16, radius: f32) -> Self {
        Self::Place { material, radius }
    }

    /// Create a new fill action with the given material and radius.
    #[must_use]
    pub fn fill(material: u16, radius: f32) -> Self {
        Self::Fill { material, radius }
    }

    /// Get the radius of this action.
    #[must_use]
    pub fn radius(&self) -> f32 {
        match *self {
            Self::Dig { radius } | Self::Place { radius, .. } | Self::Fill { radius, .. } => radius,
        }
    }

    /// Get the material ID if this is a place or fill action.
    #[must_use]
    pub fn material(&self) -> Option<u16> {
        match *self {
            Self::Place { material, .. } | Self::Fill { material, .. } => Some(material),
            Self::Dig { .. } => None,
        }
    }
}

/// Simple cell representation for terrain operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Cell {
    /// Material ID (0 = air)
    pub material: u16,
    /// Additional flags
    pub flags: u8,
}

impl Cell {
    /// Air cell constant.
    pub const AIR: Cell = Cell {
        material: 0,
        flags: 0,
    };

    /// Create a new cell with the given material.
    #[must_use]
    pub const fn new(material: u16) -> Self {
        Self { material, flags: 0 }
    }

    /// Create a cell with material and flags.
    #[must_use]
    pub const fn with_flags(material: u16, flags: u8) -> Self {
        Self { material, flags }
    }

    /// Check if this cell is air (empty).
    #[must_use]
    pub fn is_air(&self) -> bool {
        self.material == 0
    }

    /// Check if this cell is solid (non-air).
    #[must_use]
    pub fn is_solid(&self) -> bool {
        self.material != 0
    }
}

/// Common material IDs.
pub mod materials {
    /// Air (empty)
    pub const AIR: u16 = 0;
    /// Dirt
    pub const DIRT: u16 = 1;
    /// Stone
    pub const STONE: u16 = 2;
    /// Sand
    pub const SAND: u16 = 3;
    /// Water
    pub const WATER: u16 = 4;
    /// Grass
    pub const GRASS: u16 = 5;
    /// Wood
    pub const WOOD: u16 = 6;
    /// Leaves
    pub const LEAVES: u16 = 7;
    /// Ore
    pub const ORE: u16 = 8;
}

/// Chunk manager trait for terrain access.
///
/// This trait abstracts the chunk system to allow terrain manipulation
/// without depending on the specific chunk implementation.
pub trait ChunkManager {
    /// Get the cell at the given world position.
    fn get_cell(&self, x: i32, y: i32) -> Cell;

    /// Set the cell at the given world position.
    fn set_cell(&mut self, x: i32, y: i32, cell: Cell);

    /// Check if a position is loaded (valid for modification).
    fn is_loaded(&self, x: i32, y: i32) -> bool;
}

/// Mock chunk manager for testing.
#[derive(Debug, Default)]
pub struct MockChunkManager {
    cells: std::collections::HashMap<(i32, i32), Cell>,
    default_cell: Cell,
}

impl MockChunkManager {
    /// Create a new mock chunk manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a default solid terrain.
    #[must_use]
    pub fn with_solid_terrain(material: u16) -> Self {
        Self {
            cells: std::collections::HashMap::new(),
            default_cell: Cell::new(material),
        }
    }

    /// Set the default cell type.
    pub fn set_default(&mut self, cell: Cell) {
        self.default_cell = cell;
    }
}

impl ChunkManager for MockChunkManager {
    fn get_cell(&self, x: i32, y: i32) -> Cell {
        self.cells
            .get(&(x, y))
            .copied()
            .unwrap_or(self.default_cell)
    }

    fn set_cell(&mut self, x: i32, y: i32, cell: Cell) {
        self.cells.insert((x, y), cell);
    }

    fn is_loaded(&self, _x: i32, _y: i32) -> bool {
        true // Mock always loaded
    }
}

/// Modified cell tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifiedCell {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Previous cell state
    pub old_cell: Cell,
    /// New cell state
    pub new_cell: Cell,
}

impl ModifiedCell {
    /// Create a new modified cell record.
    #[must_use]
    pub fn new(x: i32, y: i32, old_cell: Cell, new_cell: Cell) -> Self {
        Self {
            x,
            y,
            old_cell,
            new_cell,
        }
    }
}

/// Terrain manipulation system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainManipulator {
    /// Current selected material for placing
    pub selected_material: u16,
    /// Dig/place radius
    pub brush_radius: f32,
    /// Cooldown between actions in seconds
    pub action_cooldown: f32,
    /// Time until next action allowed
    cooldown_timer: f32,
}

/// Minimum brush radius.
const MIN_BRUSH_RADIUS: f32 = 0.5;
/// Maximum brush radius.
const MAX_BRUSH_RADIUS: f32 = 10.0;
/// Default brush radius.
const DEFAULT_BRUSH_RADIUS: f32 = 1.5;
/// Default action cooldown.
const DEFAULT_COOLDOWN: f32 = 0.1;

/// List of available materials for cycling.
const PLACEABLE_MATERIALS: [u16; 8] = [
    materials::DIRT,
    materials::STONE,
    materials::SAND,
    materials::WATER,
    materials::GRASS,
    materials::WOOD,
    materials::LEAVES,
    materials::ORE,
];

impl Default for TerrainManipulator {
    fn default() -> Self {
        Self::new()
    }
}

impl TerrainManipulator {
    /// Create a new terrain manipulator with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            selected_material: materials::DIRT,
            brush_radius: DEFAULT_BRUSH_RADIUS,
            action_cooldown: DEFAULT_COOLDOWN,
            cooldown_timer: 0.0,
        }
    }

    /// Create a terrain manipulator with custom settings.
    #[must_use]
    pub fn with_config(selected_material: u16, brush_radius: f32, action_cooldown: f32) -> Self {
        Self {
            selected_material,
            brush_radius: brush_radius.clamp(MIN_BRUSH_RADIUS, MAX_BRUSH_RADIUS),
            action_cooldown,
            cooldown_timer: 0.0,
        }
    }

    /// Attempt to perform terrain action at world position.
    /// Returns the cells that were modified.
    pub fn perform_action<C: ChunkManager>(
        &mut self,
        action: TerrainAction,
        world_pos: (f32, f32),
        chunk_manager: &mut C,
    ) -> Vec<ModifiedCell> {
        if !self.can_act() {
            return Vec::new();
        }

        let center_x = world_pos.0.floor() as i32;
        let center_y = world_pos.1.floor() as i32;
        let radius = action.radius();

        let modified = match action {
            TerrainAction::Dig { .. } => Self::apply_dig(center_x, center_y, radius, chunk_manager),
            TerrainAction::Place { material, .. } => {
                Self::apply_place(center_x, center_y, radius, material, chunk_manager)
            },
            TerrainAction::Fill { material, .. } => {
                Self::apply_fill(center_x, center_y, radius, material, chunk_manager)
            },
        };

        if !modified.is_empty() {
            self.cooldown_timer = self.action_cooldown;
        }

        modified
    }

    /// Apply dig action to terrain.
    fn apply_dig<C: ChunkManager>(
        center_x: i32,
        center_y: i32,
        radius: f32,
        chunk_manager: &mut C,
    ) -> Vec<ModifiedCell> {
        let mut modified = Vec::new();
        let int_radius = radius.ceil() as i32;

        for dy in -int_radius..=int_radius {
            for dx in -int_radius..=int_radius {
                let x = center_x + dx;
                let y = center_y + dy;

                // Check if within circular radius
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > radius * radius {
                    continue;
                }

                if !chunk_manager.is_loaded(x, y) {
                    continue;
                }

                let old_cell = chunk_manager.get_cell(x, y);
                if old_cell.is_solid() {
                    let new_cell = Cell::AIR;
                    chunk_manager.set_cell(x, y, new_cell);
                    modified.push(ModifiedCell::new(x, y, old_cell, new_cell));
                }
            }
        }

        modified
    }

    /// Apply place action to terrain.
    fn apply_place<C: ChunkManager>(
        center_x: i32,
        center_y: i32,
        radius: f32,
        material: u16,
        chunk_manager: &mut C,
    ) -> Vec<ModifiedCell> {
        let mut modified = Vec::new();
        let int_radius = radius.ceil() as i32;

        for dy in -int_radius..=int_radius {
            for dx in -int_radius..=int_radius {
                let x = center_x + dx;
                let y = center_y + dy;

                // Check if within circular radius
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > radius * radius {
                    continue;
                }

                if !chunk_manager.is_loaded(x, y) {
                    continue;
                }

                let old_cell = chunk_manager.get_cell(x, y);
                let new_cell = Cell::new(material);
                chunk_manager.set_cell(x, y, new_cell);
                modified.push(ModifiedCell::new(x, y, old_cell, new_cell));
            }
        }

        modified
    }

    /// Apply fill action to terrain (only fills air cells).
    fn apply_fill<C: ChunkManager>(
        center_x: i32,
        center_y: i32,
        radius: f32,
        material: u16,
        chunk_manager: &mut C,
    ) -> Vec<ModifiedCell> {
        let mut modified = Vec::new();
        let int_radius = radius.ceil() as i32;

        for dy in -int_radius..=int_radius {
            for dx in -int_radius..=int_radius {
                let x = center_x + dx;
                let y = center_y + dy;

                // Check if within circular radius
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > radius * radius {
                    continue;
                }

                if !chunk_manager.is_loaded(x, y) {
                    continue;
                }

                let old_cell = chunk_manager.get_cell(x, y);
                // Only fill air cells
                if old_cell.is_air() {
                    let new_cell = Cell::new(material);
                    chunk_manager.set_cell(x, y, new_cell);
                    modified.push(ModifiedCell::new(x, y, old_cell, new_cell));
                }
            }
        }

        modified
    }

    /// Update cooldown timer.
    pub fn update(&mut self, dt: f32) {
        self.cooldown_timer = (self.cooldown_timer - dt).max(0.0);
    }

    /// Check if action is ready.
    #[must_use]
    pub fn can_act(&self) -> bool {
        self.cooldown_timer <= 0.0
    }

    /// Get remaining cooldown time.
    #[must_use]
    pub fn cooldown_remaining(&self) -> f32 {
        self.cooldown_timer
    }

    /// Set brush radius (clamped to valid range).
    pub fn set_radius(&mut self, radius: f32) {
        self.brush_radius = radius.clamp(MIN_BRUSH_RADIUS, MAX_BRUSH_RADIUS);
    }

    /// Increase brush radius by amount.
    pub fn increase_radius(&mut self, amount: f32) {
        self.set_radius(self.brush_radius + amount);
    }

    /// Decrease brush radius by amount.
    pub fn decrease_radius(&mut self, amount: f32) {
        self.set_radius(self.brush_radius - amount);
    }

    /// Cycle to next material.
    pub fn next_material(&mut self) {
        if let Some(idx) = PLACEABLE_MATERIALS
            .iter()
            .position(|&m| m == self.selected_material)
        {
            let next_idx = (idx + 1) % PLACEABLE_MATERIALS.len();
            self.selected_material = PLACEABLE_MATERIALS[next_idx];
        } else {
            self.selected_material = PLACEABLE_MATERIALS[0];
        }
    }

    /// Cycle to previous material.
    pub fn prev_material(&mut self) {
        if let Some(idx) = PLACEABLE_MATERIALS
            .iter()
            .position(|&m| m == self.selected_material)
        {
            let prev_idx = if idx == 0 {
                PLACEABLE_MATERIALS.len() - 1
            } else {
                idx - 1
            };
            self.selected_material = PLACEABLE_MATERIALS[prev_idx];
        } else {
            self.selected_material = PLACEABLE_MATERIALS[0];
        }
    }

    /// Set selected material directly.
    pub fn set_material(&mut self, material: u16) {
        self.selected_material = material;
    }

    /// Get selected material.
    #[must_use]
    pub fn material(&self) -> u16 {
        self.selected_material
    }

    /// Get brush radius.
    #[must_use]
    pub fn radius(&self) -> f32 {
        self.brush_radius
    }
}

/// Intent for GPU kernel to apply terrain change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerrainIntent {
    /// Action type
    pub action: IntentAction,
    /// Center X coordinate
    pub x: i32,
    /// Center Y coordinate
    pub y: i32,
    /// Material ID (for place/fill)
    pub material: u16,
}

/// Intent action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentAction {
    /// Set cell to air
    Dig,
    /// Set cell to material
    Place,
}

impl TerrainIntent {
    /// Create a dig intent.
    #[must_use]
    pub fn dig(x: i32, y: i32) -> Self {
        Self {
            action: IntentAction::Dig,
            x,
            y,
            material: materials::AIR,
        }
    }

    /// Create a place intent.
    #[must_use]
    pub fn place(x: i32, y: i32, material: u16) -> Self {
        Self {
            action: IntentAction::Place,
            x,
            y,
            material,
        }
    }
}

/// Generate intents for GPU kernel to apply terrain change.
///
/// This creates a list of intents for each cell in the action radius.
pub fn create_terrain_intents(action: TerrainAction, center: (i32, i32)) -> Vec<TerrainIntent> {
    let mut intents = Vec::new();
    let radius = action.radius();
    let int_radius = radius.ceil() as i32;

    for dy in -int_radius..=int_radius {
        for dx in -int_radius..=int_radius {
            // Check if within circular radius
            let dist_sq = (dx * dx + dy * dy) as f32;
            if dist_sq > radius * radius {
                continue;
            }

            let x = center.0 + dx;
            let y = center.1 + dy;

            let intent = match action {
                TerrainAction::Dig { .. } => TerrainIntent::dig(x, y),
                TerrainAction::Place { material, .. } | TerrainAction::Fill { material, .. } => {
                    TerrainIntent::place(x, y, material)
                },
            };

            intents.push(intent);
        }
    }

    intents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_action_constructors() {
        let dig = TerrainAction::dig(2.0);
        assert_eq!(dig.radius(), 2.0);
        assert_eq!(dig.material(), None);

        let place = TerrainAction::place(materials::STONE, 1.5);
        assert_eq!(place.radius(), 1.5);
        assert_eq!(place.material(), Some(materials::STONE));

        let fill = TerrainAction::fill(materials::WATER, 3.0);
        assert_eq!(fill.radius(), 3.0);
        assert_eq!(fill.material(), Some(materials::WATER));
    }

    #[test]
    fn test_cell_properties() {
        let air = Cell::AIR;
        assert!(air.is_air());
        assert!(!air.is_solid());

        let dirt = Cell::new(materials::DIRT);
        assert!(!dirt.is_air());
        assert!(dirt.is_solid());
    }

    #[test]
    fn test_terrain_manipulator_new() {
        let manip = TerrainManipulator::new();
        assert_eq!(manip.selected_material, materials::DIRT);
        assert_eq!(manip.brush_radius, DEFAULT_BRUSH_RADIUS);
        assert!(manip.can_act());
    }

    #[test]
    fn test_terrain_manipulator_cooldown() {
        let mut manip = TerrainManipulator::new();
        let mut chunks = MockChunkManager::with_solid_terrain(materials::STONE);

        // First action should work
        let modified = manip.perform_action(TerrainAction::dig(1.0), (0.0, 0.0), &mut chunks);
        assert!(!modified.is_empty());
        assert!(!manip.can_act());

        // Second action should be blocked
        let modified = manip.perform_action(TerrainAction::dig(1.0), (5.0, 5.0), &mut chunks);
        assert!(modified.is_empty());

        // After cooldown, should work again
        manip.update(manip.action_cooldown + 0.01);
        assert!(manip.can_act());
    }

    #[test]
    fn test_terrain_manipulator_dig() {
        let mut manip = TerrainManipulator::new();
        let mut chunks = MockChunkManager::with_solid_terrain(materials::STONE);

        let modified = manip.perform_action(TerrainAction::dig(1.0), (0.0, 0.0), &mut chunks);

        assert!(!modified.is_empty());
        // Center cell should now be air
        assert!(chunks.get_cell(0, 0).is_air());
    }

    #[test]
    fn test_terrain_manipulator_place() {
        let mut manip = TerrainManipulator::new();
        let mut chunks = MockChunkManager::new(); // All air by default

        let modified = manip.perform_action(
            TerrainAction::place(materials::DIRT, 1.0),
            (0.0, 0.0),
            &mut chunks,
        );

        assert!(!modified.is_empty());
        // Center cell should now be dirt
        assert_eq!(chunks.get_cell(0, 0).material, materials::DIRT);
    }

    #[test]
    fn test_terrain_manipulator_fill() {
        let mut manip = TerrainManipulator::new();
        let mut chunks = MockChunkManager::new(); // All air by default

        // Set one cell as solid
        chunks.set_cell(0, 0, Cell::new(materials::STONE));

        let modified = manip.perform_action(
            TerrainAction::fill(materials::WATER, 1.0),
            (0.0, 0.0),
            &mut chunks,
        );

        // Stone cell should remain stone (fill only affects air)
        assert_eq!(chunks.get_cell(0, 0).material, materials::STONE);

        // Adjacent air cells should be water
        assert_eq!(chunks.get_cell(1, 0).material, materials::WATER);
        assert!(!modified
            .iter()
            .any(|m| m.x == 0 && m.y == 0 && m.new_cell.material == materials::WATER));
    }

    #[test]
    fn test_terrain_manipulator_radius() {
        let mut manip = TerrainManipulator::new();

        manip.set_radius(5.0);
        assert_eq!(manip.brush_radius, 5.0);

        // Test clamping
        manip.set_radius(100.0);
        assert_eq!(manip.brush_radius, MAX_BRUSH_RADIUS);

        manip.set_radius(0.0);
        assert_eq!(manip.brush_radius, MIN_BRUSH_RADIUS);
    }

    #[test]
    fn test_terrain_manipulator_material_cycling() {
        let mut manip = TerrainManipulator::new();
        let initial = manip.selected_material;

        manip.next_material();
        assert_ne!(manip.selected_material, initial);

        // Cycle back
        manip.prev_material();
        assert_eq!(manip.selected_material, initial);
    }

    #[test]
    fn test_create_terrain_intents() {
        let intents = create_terrain_intents(TerrainAction::dig(1.0), (5, 5));

        assert!(!intents.is_empty());
        // Should include center
        assert!(intents.iter().any(|i| i.x == 5 && i.y == 5));
        // All should be dig intents
        assert!(intents.iter().all(|i| i.action == IntentAction::Dig));
    }

    #[test]
    fn test_terrain_intent_constructors() {
        let dig = TerrainIntent::dig(10, 20);
        assert_eq!(dig.action, IntentAction::Dig);
        assert_eq!(dig.x, 10);
        assert_eq!(dig.y, 20);
        assert_eq!(dig.material, materials::AIR);

        let place = TerrainIntent::place(15, 25, materials::STONE);
        assert_eq!(place.action, IntentAction::Place);
        assert_eq!(place.x, 15);
        assert_eq!(place.y, 25);
        assert_eq!(place.material, materials::STONE);
    }

    #[test]
    fn test_modified_cell() {
        let old_cell = Cell::new(materials::STONE);
        let new_cell = Cell::AIR;
        let modified = ModifiedCell::new(10, 20, old_cell, new_cell);

        assert_eq!(modified.x, 10);
        assert_eq!(modified.y, 20);
        assert_eq!(modified.old_cell.material, materials::STONE);
        assert!(modified.new_cell.is_air());
    }

    #[test]
    fn test_mock_chunk_manager() {
        let mut chunks = MockChunkManager::new();
        assert!(chunks.is_loaded(0, 0));
        assert!(chunks.get_cell(0, 0).is_air());

        chunks.set_cell(5, 5, Cell::new(materials::DIRT));
        assert_eq!(chunks.get_cell(5, 5).material, materials::DIRT);
    }

    #[test]
    fn test_increase_decrease_radius() {
        let mut manip = TerrainManipulator::new();
        let initial = manip.brush_radius;

        manip.increase_radius(1.0);
        assert_eq!(manip.brush_radius, initial + 1.0);

        manip.decrease_radius(1.0);
        assert_eq!(manip.brush_radius, initial);
    }

    #[test]
    fn test_cell_with_flags() {
        let cell = Cell::with_flags(materials::WATER, 0b0000_0001);
        assert_eq!(cell.material, materials::WATER);
        assert_eq!(cell.flags, 1);
    }
}
