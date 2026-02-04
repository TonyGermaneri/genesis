//! Crafting Grid Computation
//!
//! This module provides the low-level crafting grid infrastructure for
//! pattern-based crafting systems. It supports:
//!
//! - Configurable grid sizes (default 3x3)
//! - Shaped recipes (specific pattern required)
//! - Shapeless recipes (any arrangement)
//! - GPU-friendly pattern representation
//!
//! # Architecture
//!
//! ```text
//! ┌───────────────┐     ┌────────────────┐     ┌────────────────┐
//! │ CraftingGrid  │────▶│ RecipePattern  │────▶│ RecipeMatcher  │
//! │ (player input)│     │ (recipe def)   │     │ (find matches) │
//! └───────────────┘     └────────────────┘     └────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use genesis_kernel::crafting_grid::{CraftingGrid, RecipePattern, RecipeMatcher};
//!
//! // Create a 3x3 crafting grid
//! let mut grid = CraftingGrid::new(3, 3);
//!
//! // Place items (item_id = 1 for stick, 2 for plank)
//! grid.set_slot(0, 0, Some(2)); // plank
//! grid.set_slot(1, 0, Some(2)); // plank
//! grid.set_slot(1, 1, Some(1)); // stick
//! grid.set_slot(1, 2, Some(1)); // stick
//!
//! // Define a sword recipe pattern (2x2 - two planks on top, one stick below)
//! let sword_pattern = RecipePattern::shaped(2, 2, &[
//!     Some(2), Some(2), // planks
//!     Some(1), None,    // stick
//! ]);
//!
//! // Check if the grid matches (won't match since pattern differs)
//! let matcher = RecipeMatcher::new();
//! // Pattern matching depends on exact grid layout
//! ```

use std::collections::HashMap;

use tracing::debug;

/// Maximum supported grid dimension.
pub const MAX_GRID_SIZE: usize = 9;

/// Empty slot marker for GPU representation.
pub const EMPTY_SLOT: u32 = 0;

/// Wildcard item that matches any non-empty slot.
pub const WILDCARD_ITEM: u32 = u32::MAX;

/// Item ID type for crafting grids.
pub type ItemId = u32;

/// Slot content: item ID or None for empty.
pub type SlotContent = Option<ItemId>;

/// A single slot in the crafting grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemSlot {
    /// Item ID (0 = empty).
    pub item_id: ItemId,
    /// Item count in this slot.
    pub count: u16,
    /// Metadata/variant for the item.
    pub metadata: u16,
}

impl ItemSlot {
    /// Create an empty slot.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            item_id: EMPTY_SLOT,
            count: 0,
            metadata: 0,
        }
    }

    /// Create a slot with an item.
    #[must_use]
    pub const fn new(item_id: ItemId, count: u16) -> Self {
        Self {
            item_id,
            count,
            metadata: 0,
        }
    }

    /// Create a slot with item and metadata.
    #[must_use]
    pub const fn with_metadata(item_id: ItemId, count: u16, metadata: u16) -> Self {
        Self {
            item_id,
            count,
            metadata,
        }
    }

    /// Check if the slot is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.item_id == EMPTY_SLOT || self.count == 0
    }

    /// Get the item ID if not empty.
    #[must_use]
    pub const fn item(&self) -> Option<ItemId> {
        if self.is_empty() {
            None
        } else {
            Some(self.item_id)
        }
    }
}

/// A configurable crafting grid.
///
/// The grid stores items in row-major order and supports various sizes
/// from 1x1 to 9x9.
#[derive(Debug, Clone)]
pub struct CraftingGrid {
    /// Grid width.
    width: usize,
    /// Grid height.
    height: usize,
    /// Slots in row-major order.
    slots: Vec<ItemSlot>,
}

impl Default for CraftingGrid {
    fn default() -> Self {
        Self::new(3, 3)
    }
}

impl CraftingGrid {
    /// Create a new crafting grid with the given dimensions.
    ///
    /// # Panics
    /// Panics if width or height exceeds `MAX_GRID_SIZE`.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && width <= MAX_GRID_SIZE, "Invalid grid width");
        assert!(height > 0 && height <= MAX_GRID_SIZE, "Invalid grid height");

        Self {
            width,
            height,
            slots: vec![ItemSlot::empty(); width * height],
        }
    }

    /// Create a 2x2 crafting grid (inventory crafting).
    #[must_use]
    pub fn inventory() -> Self {
        Self::new(2, 2)
    }

    /// Create a 3x3 crafting grid (workbench).
    #[must_use]
    pub fn workbench() -> Self {
        Self::new(3, 3)
    }

    /// Get the grid width.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Get the grid height.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Get the total number of slots.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Get the slot at (x, y), or None if out of bounds.
    #[must_use]
    pub fn get_slot(&self, x: usize, y: usize) -> Option<&ItemSlot> {
        if x < self.width && y < self.height {
            Some(&self.slots[y * self.width + x])
        } else {
            None
        }
    }

    /// Get mutable slot at (x, y), or None if out of bounds.
    pub fn get_slot_mut(&mut self, x: usize, y: usize) -> Option<&mut ItemSlot> {
        if x < self.width && y < self.height {
            Some(&mut self.slots[y * self.width + x])
        } else {
            None
        }
    }

    /// Set the item ID at (x, y). Returns false if out of bounds.
    pub fn set_slot(&mut self, x: usize, y: usize, item: SlotContent) -> bool {
        if let Some(slot) = self.get_slot_mut(x, y) {
            match item {
                Some(id) => {
                    slot.item_id = id;
                    slot.count = 1;
                },
                None => {
                    *slot = ItemSlot::empty();
                },
            }
            true
        } else {
            false
        }
    }

    /// Set a full slot at (x, y). Returns false if out of bounds.
    pub fn set_full_slot(&mut self, x: usize, y: usize, slot: ItemSlot) -> bool {
        if x < self.width && y < self.height {
            self.slots[y * self.width + x] = slot;
            true
        } else {
            false
        }
    }

    /// Clear all slots.
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = ItemSlot::empty();
        }
    }

    /// Check if the grid is completely empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(ItemSlot::is_empty)
    }

    /// Get all slots as a slice.
    #[must_use]
    pub fn slots(&self) -> &[ItemSlot] {
        &self.slots
    }

    /// Get the bounding box of non-empty slots.
    /// Returns (min_x, min_y, max_x, max_y) or None if empty.
    #[must_use]
    pub fn bounding_box(&self) -> Option<(usize, usize, usize, usize)> {
        let mut min_x = self.width;
        let mut min_y = self.height;
        let mut max_x = 0;
        let mut max_y = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                if !self.slots[y * self.width + x].is_empty() {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        if min_x <= max_x && min_y <= max_y {
            Some((min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }

    /// Extract the pattern from non-empty slots.
    /// Returns normalized item IDs within the bounding box.
    #[must_use]
    pub fn extract_pattern(&self) -> Option<ExtractedPattern> {
        let (min_x, min_y, max_x, max_y) = self.bounding_box()?;
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;

        let mut items = Vec::with_capacity(width * height);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                items.push(self.slots[y * self.width + x].item());
            }
        }

        Some(ExtractedPattern {
            width,
            height,
            items,
            offset_x: min_x,
            offset_y: min_y,
        })
    }

    /// Convert to GPU-friendly representation (array of u32 item IDs).
    #[must_use]
    pub fn to_gpu_buffer(&self) -> Vec<u32> {
        self.slots.iter().map(|s| s.item_id).collect()
    }

    /// Create from GPU buffer data.
    #[must_use]
    pub fn from_gpu_buffer(width: usize, height: usize, data: &[u32]) -> Option<Self> {
        if data.len() != width * height {
            return None;
        }

        let slots = data
            .iter()
            .map(|&id| {
                if id == EMPTY_SLOT {
                    ItemSlot::empty()
                } else {
                    ItemSlot::new(id, 1)
                }
            })
            .collect();

        Some(Self {
            width,
            height,
            slots,
        })
    }

    /// Count non-empty slots.
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.slots.iter().filter(|s| !s.is_empty()).count()
    }

    /// Get all unique item IDs in the grid.
    #[must_use]
    pub fn unique_items(&self) -> Vec<ItemId> {
        let mut items: Vec<_> = self
            .slots
            .iter()
            .filter_map(|s| if s.is_empty() { None } else { Some(s.item_id) })
            .collect();
        items.sort_unstable();
        items.dedup();
        items
    }
}

/// Pattern extracted from a crafting grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedPattern {
    /// Pattern width.
    pub width: usize,
    /// Pattern height.
    pub height: usize,
    /// Item IDs in row-major order.
    pub items: Vec<SlotContent>,
    /// X offset in original grid.
    pub offset_x: usize,
    /// Y offset in original grid.
    pub offset_y: usize,
}

/// Recipe type: shaped (fixed pattern) or shapeless (any arrangement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecipeType {
    /// Fixed pattern must match exactly.
    #[default]
    Shaped,
    /// Items can be in any arrangement.
    Shapeless,
}

/// A recipe pattern for matching against crafting grids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipePattern {
    /// Recipe type.
    pub recipe_type: RecipeType,
    /// Pattern width (for shaped).
    pub width: usize,
    /// Pattern height (for shaped).
    pub height: usize,
    /// Required item IDs in row-major order.
    /// None = empty slot required, Some(WILDCARD_ITEM) = any item.
    pub items: Vec<SlotContent>,
    /// Result item ID.
    pub result_id: ItemId,
    /// Result count.
    pub result_count: u16,
    /// Whether the pattern can be mirrored horizontally.
    pub allow_mirror: bool,
}

impl RecipePattern {
    /// Create a shaped recipe pattern.
    ///
    /// # Arguments
    /// * `width` - Pattern width
    /// * `height` - Pattern height  
    /// * `items` - Item IDs in row-major order
    #[must_use]
    pub fn shaped(width: usize, height: usize, items: &[SlotContent]) -> Self {
        assert_eq!(
            items.len(),
            width * height,
            "Item count must match dimensions"
        );

        Self {
            recipe_type: RecipeType::Shaped,
            width,
            height,
            items: items.to_vec(),
            result_id: 0,
            result_count: 1,
            allow_mirror: false,
        }
    }

    /// Create a shapeless recipe pattern.
    ///
    /// # Arguments
    /// * `items` - Required item IDs (order doesn't matter)
    #[must_use]
    pub fn shapeless(items: &[ItemId]) -> Self {
        Self {
            recipe_type: RecipeType::Shapeless,
            width: 0,
            height: 0,
            items: items.iter().map(|&id| Some(id)).collect(),
            result_id: 0,
            result_count: 1,
            allow_mirror: false,
        }
    }

    /// Set the result item.
    #[must_use]
    pub const fn with_result(mut self, item_id: ItemId, count: u16) -> Self {
        self.result_id = item_id;
        self.result_count = count;
        self
    }

    /// Allow horizontal mirroring for shaped recipes.
    #[must_use]
    pub const fn with_mirror(mut self, allow: bool) -> Self {
        self.allow_mirror = allow;
        self
    }

    /// Check if this is a shaped recipe.
    #[must_use]
    pub const fn is_shaped(&self) -> bool {
        matches!(self.recipe_type, RecipeType::Shaped)
    }

    /// Check if this is a shapeless recipe.
    #[must_use]
    pub const fn is_shapeless(&self) -> bool {
        matches!(self.recipe_type, RecipeType::Shapeless)
    }

    /// Get the number of required items.
    #[must_use]
    pub fn ingredient_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_some()).count()
    }

    /// Get required items sorted for shapeless comparison.
    #[must_use]
    pub fn sorted_ingredients(&self) -> Vec<ItemId> {
        let mut items: Vec<_> = self.items.iter().filter_map(|&i| i).collect();
        items.sort_unstable();
        items
    }

    /// Create a mirrored version of a shaped pattern.
    #[must_use]
    pub fn mirror_horizontal(&self) -> Self {
        if !self.is_shaped() {
            return self.clone();
        }

        let mut mirrored = Vec::with_capacity(self.items.len());
        for y in 0..self.height {
            for x in (0..self.width).rev() {
                mirrored.push(self.items[y * self.width + x]);
            }
        }

        Self {
            items: mirrored,
            ..self.clone()
        }
    }

    /// Convert to GPU-friendly representation.
    #[must_use]
    pub fn to_gpu_buffer(&self) -> Vec<u32> {
        self.items
            .iter()
            .map(|slot| slot.unwrap_or(EMPTY_SLOT))
            .collect()
    }
}

/// Recipe matcher for finding matching recipes.
#[derive(Debug, Default)]
pub struct RecipeMatcher {
    /// Registered recipes.
    recipes: Vec<RecipePattern>,
    /// Index by ingredient count for quick filtering.
    by_ingredient_count: HashMap<usize, Vec<usize>>,
}

impl RecipeMatcher {
    /// Create a new recipe matcher.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a recipe.
    pub fn register(&mut self, recipe: RecipePattern) {
        let count = recipe.ingredient_count();
        let idx = self.recipes.len();
        self.recipes.push(recipe);
        self.by_ingredient_count.entry(count).or_default().push(idx);
        debug!("Registered recipe with {} ingredients", count);
    }

    /// Get all registered recipes.
    #[must_use]
    pub fn recipes(&self) -> &[RecipePattern] {
        &self.recipes
    }

    /// Find all recipes that match the grid.
    #[must_use]
    pub fn find_matches(&self, grid: &CraftingGrid) -> Vec<&RecipePattern> {
        let grid_count = grid.item_count();

        // Get candidate recipes by ingredient count
        let Some(candidates) = self.by_ingredient_count.get(&grid_count) else {
            return Vec::new();
        };

        candidates
            .iter()
            .filter_map(|&idx| {
                let recipe = &self.recipes[idx];
                if self.matches(grid, recipe) {
                    Some(recipe)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find the first matching recipe.
    #[must_use]
    pub fn find_first_match(&self, grid: &CraftingGrid) -> Option<&RecipePattern> {
        self.find_matches(grid).into_iter().next()
    }

    /// Check if a grid matches a specific recipe.
    #[must_use]
    pub fn matches(&self, grid: &CraftingGrid, recipe: &RecipePattern) -> bool {
        match recipe.recipe_type {
            RecipeType::Shaped => Self::matches_shaped(grid, recipe),
            RecipeType::Shapeless => Self::matches_shapeless(grid, recipe),
        }
    }

    /// Check if grid matches a shaped recipe.
    fn matches_shaped(grid: &CraftingGrid, recipe: &RecipePattern) -> bool {
        let Some(pattern) = grid.extract_pattern() else {
            // Empty grid only matches empty recipe
            return recipe.ingredient_count() == 0;
        };

        // Check dimensions
        if pattern.width != recipe.width || pattern.height != recipe.height {
            return false;
        }

        // Check pattern match
        if Self::patterns_match(&pattern.items, &recipe.items) {
            return true;
        }

        // Try mirrored if allowed
        if recipe.allow_mirror {
            let mirrored = recipe.mirror_horizontal();
            if Self::patterns_match(&pattern.items, &mirrored.items) {
                return true;
            }
        }

        false
    }

    /// Check if two patterns match (including wildcards).
    #[allow(clippy::match_same_arms)]
    fn patterns_match(grid_items: &[SlotContent], recipe_items: &[SlotContent]) -> bool {
        if grid_items.len() != recipe_items.len() {
            return false;
        }

        for (grid_item, recipe_item) in grid_items.iter().zip(recipe_items.iter()) {
            match (grid_item, recipe_item) {
                (None, None) => {},
                (Some(_), Some(WILDCARD_ITEM)) => {}, // Wildcard matches any
                (Some(g), Some(r)) if g == r => {},
                _ => return false,
            }
        }

        true
    }

    /// Check if grid matches a shapeless recipe.
    fn matches_shapeless(grid: &CraftingGrid, recipe: &RecipePattern) -> bool {
        let grid_items = grid.unique_items();
        let recipe_items = recipe.sorted_ingredients();

        // Count occurrences
        let grid_counts = Self::count_items(&grid_items);
        let recipe_counts = Self::count_items(&recipe_items);

        grid_counts == recipe_counts
    }

    /// Count item occurrences.
    fn count_items(items: &[ItemId]) -> HashMap<ItemId, usize> {
        let mut counts = HashMap::new();
        for &item in items {
            *counts.entry(item).or_insert(0) += 1;
        }
        counts
    }

    /// Clear all registered recipes.
    pub fn clear(&mut self) {
        self.recipes.clear();
        self.by_ingredient_count.clear();
    }
}

/// Result of a crafting operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftingResult {
    /// Result item ID.
    pub item_id: ItemId,
    /// Result count.
    pub count: u16,
    /// Slots consumed (indices in the grid).
    pub consumed_slots: Vec<(usize, usize)>,
}

impl CraftingResult {
    /// Create a new crafting result.
    #[must_use]
    pub const fn new(item_id: ItemId, count: u16) -> Self {
        Self {
            item_id,
            count,
            consumed_slots: Vec::new(),
        }
    }

    /// Add a consumed slot.
    pub fn add_consumed(&mut self, x: usize, y: usize) {
        self.consumed_slots.push((x, y));
    }
}

/// Execute a craft operation on a grid.
///
/// Returns the crafting result if successful, consuming one item from each slot.
pub fn execute_craft(grid: &mut CraftingGrid, recipe: &RecipePattern) -> Option<CraftingResult> {
    // Verify the recipe matches
    let matcher = RecipeMatcher::new();
    if !matcher.matches(grid, recipe) {
        return None;
    }

    let mut result = CraftingResult::new(recipe.result_id, recipe.result_count);

    // Consume one item from each non-empty slot
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if let Some(slot) = grid.get_slot_mut(x, y) {
                if !slot.is_empty() {
                    slot.count = slot.count.saturating_sub(1);
                    if slot.count == 0 {
                        *slot = ItemSlot::empty();
                    }
                    result.add_consumed(x, y);
                }
            }
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_slot() {
        let empty = ItemSlot::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.item(), None);

        let slot = ItemSlot::new(42, 5);
        assert!(!slot.is_empty());
        assert_eq!(slot.item(), Some(42));
        assert_eq!(slot.count, 5);
    }

    #[test]
    fn test_crafting_grid_creation() {
        let grid = CraftingGrid::new(3, 3);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.slot_count(), 9);
        assert!(grid.is_empty());
    }

    #[test]
    fn test_crafting_grid_set_get() {
        let mut grid = CraftingGrid::new(3, 3);

        assert!(grid.set_slot(1, 1, Some(42)));
        assert_eq!(grid.get_slot(1, 1).map(|s| s.item_id), Some(42));
        assert!(!grid.is_empty());

        assert!(grid.set_slot(1, 1, None));
        assert!(grid.is_empty());
    }

    #[test]
    fn test_crafting_grid_bounding_box() {
        let mut grid = CraftingGrid::new(3, 3);
        assert_eq!(grid.bounding_box(), None);

        grid.set_slot(1, 0, Some(1));
        grid.set_slot(1, 1, Some(1));
        grid.set_slot(1, 2, Some(1));

        let bbox = grid.bounding_box();
        assert_eq!(bbox, Some((1, 0, 1, 2)));
    }

    #[test]
    fn test_extract_pattern() {
        let mut grid = CraftingGrid::new(3, 3);
        grid.set_slot(0, 0, Some(1));
        grid.set_slot(1, 0, Some(2));
        grid.set_slot(0, 1, Some(3));

        let pattern = grid.extract_pattern().expect("should extract pattern");
        assert_eq!(pattern.width, 2);
        assert_eq!(pattern.height, 2);
        assert_eq!(pattern.items, vec![Some(1), Some(2), Some(3), None]);
    }

    #[test]
    fn test_recipe_pattern_shaped() {
        let pattern =
            RecipePattern::shaped(2, 2, &[Some(1), Some(2), Some(3), None]).with_result(100, 1);

        assert!(pattern.is_shaped());
        assert!(!pattern.is_shapeless());
        assert_eq!(pattern.ingredient_count(), 3);
        assert_eq!(pattern.result_id, 100);
    }

    #[test]
    fn test_recipe_pattern_shapeless() {
        let pattern = RecipePattern::shapeless(&[1, 2, 3]).with_result(100, 1);

        assert!(pattern.is_shapeless());
        assert!(!pattern.is_shaped());
        assert_eq!(pattern.ingredient_count(), 3);
    }

    #[test]
    fn test_recipe_pattern_mirror() {
        let pattern = RecipePattern::shaped(2, 1, &[Some(1), Some(2)]);
        let mirrored = pattern.mirror_horizontal();

        assert_eq!(mirrored.items, vec![Some(2), Some(1)]);
    }

    #[test]
    fn test_recipe_matcher_shaped() {
        let mut matcher = RecipeMatcher::new();

        // Register a stick recipe (2 planks vertical)
        let stick_recipe = RecipePattern::shaped(1, 2, &[Some(1), Some(1)]).with_result(2, 4);
        matcher.register(stick_recipe);

        // Create matching grid
        let mut grid = CraftingGrid::new(3, 3);
        grid.set_slot(1, 0, Some(1));
        grid.set_slot(1, 1, Some(1));

        let matches = matcher.find_matches(&grid);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].result_id, 2);
    }

    #[test]
    fn test_recipe_matcher_shaped_offset() {
        let mut matcher = RecipeMatcher::new();

        let recipe = RecipePattern::shaped(1, 2, &[Some(1), Some(1)]).with_result(2, 4);
        matcher.register(recipe);

        // Pattern in different position
        let mut grid = CraftingGrid::new(3, 3);
        grid.set_slot(2, 1, Some(1));
        grid.set_slot(2, 2, Some(1));

        let matches = matcher.find_matches(&grid);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_recipe_matcher_shapeless() {
        let mut matcher = RecipeMatcher::new();

        let recipe = RecipePattern::shapeless(&[1, 2]).with_result(3, 1);
        matcher.register(recipe);

        // Items in any order
        let mut grid = CraftingGrid::new(3, 3);
        grid.set_slot(0, 0, Some(2));
        grid.set_slot(2, 2, Some(1));

        let matches = matcher.find_matches(&grid);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_recipe_matcher_mirror() {
        let mut matcher = RecipeMatcher::new();

        let recipe = RecipePattern::shaped(2, 1, &[Some(1), Some(2)])
            .with_result(3, 1)
            .with_mirror(true);
        matcher.register(recipe);

        // Original pattern
        let mut grid1 = CraftingGrid::new(3, 3);
        grid1.set_slot(0, 0, Some(1));
        grid1.set_slot(1, 0, Some(2));
        assert_eq!(matcher.find_matches(&grid1).len(), 1);

        // Mirrored pattern
        let mut grid2 = CraftingGrid::new(3, 3);
        grid2.set_slot(0, 0, Some(2));
        grid2.set_slot(1, 0, Some(1));
        assert_eq!(matcher.find_matches(&grid2).len(), 1);
    }

    #[test]
    fn test_execute_craft() {
        let mut grid = CraftingGrid::new(3, 3);
        grid.set_full_slot(0, 0, ItemSlot::new(1, 3));
        grid.set_full_slot(0, 1, ItemSlot::new(1, 2));

        let recipe = RecipePattern::shaped(1, 2, &[Some(1), Some(1)]).with_result(2, 4);

        let result = execute_craft(&mut grid, &recipe).expect("should craft");
        assert_eq!(result.item_id, 2);
        assert_eq!(result.count, 4);
        assert_eq!(result.consumed_slots.len(), 2);

        // Check items were consumed
        assert_eq!(grid.get_slot(0, 0).map(|s| s.count), Some(2));
        assert_eq!(grid.get_slot(0, 1).map(|s| s.count), Some(1));
    }

    #[test]
    fn test_gpu_buffer_roundtrip() {
        let mut grid = CraftingGrid::new(3, 3);
        grid.set_slot(0, 0, Some(1));
        grid.set_slot(1, 1, Some(2));
        grid.set_slot(2, 2, Some(3));

        let buffer = grid.to_gpu_buffer();
        let restored = CraftingGrid::from_gpu_buffer(3, 3, &buffer).expect("should restore");

        assert_eq!(restored.get_slot(0, 0).map(|s| s.item_id), Some(1));
        assert_eq!(restored.get_slot(1, 1).map(|s| s.item_id), Some(2));
        assert_eq!(restored.get_slot(2, 2).map(|s| s.item_id), Some(3));
    }

    #[test]
    fn test_unique_items() {
        let mut grid = CraftingGrid::new(3, 3);
        grid.set_slot(0, 0, Some(1));
        grid.set_slot(1, 0, Some(2));
        grid.set_slot(2, 0, Some(1));
        grid.set_slot(0, 1, Some(3));

        let unique = grid.unique_items();
        assert_eq!(unique, vec![1, 2, 3]);
    }
}
