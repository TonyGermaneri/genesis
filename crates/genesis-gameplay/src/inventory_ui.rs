//! Inventory UI model for rendering.
//!
//! This module provides data structures and logic for presenting inventory
//! state to the UI layer, handling slot interactions, and generating tooltips.

use genesis_common::ItemTypeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::input::MouseButton;
use crate::inventory::{Inventory, ItemStack};

/// Inventory action to be executed by the game logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryAction {
    /// No action
    None,
    /// Move items from one slot to another
    Move {
        /// Source slot index
        from: usize,
        /// Destination slot index
        to: usize,
    },
    /// Split a stack (take half)
    Split {
        /// Slot to split
        slot: usize,
    },
    /// Drop items from a slot
    Drop {
        /// Slot to drop from
        slot: usize,
        /// Number of items to drop
        count: u32,
    },
    /// Use/activate an item
    Use {
        /// Slot to use
        slot: usize,
    },
    /// Swap items between two slots
    Swap {
        /// First slot
        slot_a: usize,
        /// Second slot
        slot_b: usize,
    },
    /// Quick-move to hotbar or inventory
    QuickMove {
        /// Slot to quick-move
        slot: usize,
    },
}

/// UI data for a single inventory slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotUIData {
    /// Slot index in the inventory
    pub index: usize,
    /// Item in this slot (if any)
    pub item: Option<ItemStack>,
    /// Whether this slot is part of the hotbar
    pub is_hotbar: bool,
    /// Whether this slot is currently selected
    pub is_selected: bool,
    /// Whether this slot is being hovered
    pub is_hovered: bool,
    /// Whether items can be placed here
    pub is_valid_target: bool,
}

impl SlotUIData {
    /// Creates a new slot UI data.
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self {
            index,
            item: None,
            is_hotbar: false,
            is_selected: false,
            is_hovered: false,
            is_valid_target: true,
        }
    }

    /// Creates a slot with an item.
    #[must_use]
    pub fn with_item(mut self, item: ItemStack) -> Self {
        self.item = Some(item);
        self
    }

    /// Sets the hotbar flag.
    #[must_use]
    pub fn hotbar(mut self, is_hotbar: bool) -> Self {
        self.is_hotbar = is_hotbar;
        self
    }

    /// Sets the selected flag.
    #[must_use]
    pub fn selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    /// Check if slot is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }

    /// Get the item type if present.
    #[must_use]
    pub fn item_type(&self) -> Option<ItemTypeId> {
        self.item.map(|s| s.item_type)
    }

    /// Get the quantity if present.
    #[must_use]
    pub fn quantity(&self) -> u32 {
        self.item.map_or(0, |s| s.quantity)
    }
}

/// Tooltip data for displaying item information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TooltipData {
    /// Item display name
    pub item_name: String,
    /// Item description
    pub description: String,
    /// Item statistics (name, value pairs)
    pub stats: Vec<(String, String)>,
    /// Item rarity/quality
    pub rarity: Option<String>,
    /// Stack information
    pub stack_info: Option<String>,
}

impl TooltipData {
    /// Creates a new tooltip.
    #[must_use]
    pub fn new(item_name: impl Into<String>) -> Self {
        Self {
            item_name: item_name.into(),
            description: String::new(),
            stats: Vec::new(),
            rarity: None,
            stack_info: None,
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds a stat.
    #[must_use]
    pub fn with_stat(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.stats.push((name.into(), value.into()));
        self
    }

    /// Sets the rarity.
    #[must_use]
    pub fn with_rarity(mut self, rarity: impl Into<String>) -> Self {
        self.rarity = Some(rarity.into());
        self
    }

    /// Sets stack information.
    #[must_use]
    pub fn with_stack_info(mut self, info: impl Into<String>) -> Self {
        self.stack_info = Some(info.into());
        self
    }
}

/// Item metadata provider trait for generating tooltips.
pub trait ItemMetadata {
    /// Get the display name for an item type.
    fn get_name(&self, item: ItemTypeId) -> String;

    /// Get the description for an item type.
    fn get_description(&self, item: ItemTypeId) -> String;

    /// Get statistics for an item type.
    fn get_stats(&self, item: ItemTypeId) -> Vec<(String, String)>;

    /// Get the rarity for an item type.
    fn get_rarity(&self, item: ItemTypeId) -> Option<String>;
}

/// Simple item metadata implementation using HashMaps.
#[derive(Debug, Default)]
pub struct SimpleItemMetadata {
    names: HashMap<ItemTypeId, String>,
    descriptions: HashMap<ItemTypeId, String>,
    stats: HashMap<ItemTypeId, Vec<(String, String)>>,
    rarities: HashMap<ItemTypeId, String>,
}

impl SimpleItemMetadata {
    /// Creates a new empty metadata store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an item with metadata.
    pub fn register(
        &mut self,
        item: ItemTypeId,
        name: impl Into<String>,
        description: impl Into<String>,
    ) {
        self.names.insert(item, name.into());
        self.descriptions.insert(item, description.into());
    }

    /// Sets stats for an item.
    pub fn set_stats(&mut self, item: ItemTypeId, stats: Vec<(String, String)>) {
        self.stats.insert(item, stats);
    }

    /// Sets rarity for an item.
    pub fn set_rarity(&mut self, item: ItemTypeId, rarity: impl Into<String>) {
        self.rarities.insert(item, rarity.into());
    }
}

impl ItemMetadata for SimpleItemMetadata {
    fn get_name(&self, item: ItemTypeId) -> String {
        self.names
            .get(&item)
            .cloned()
            .unwrap_or_else(|| format!("Item #{}", item.raw()))
    }

    fn get_description(&self, item: ItemTypeId) -> String {
        self.descriptions
            .get(&item)
            .cloned()
            .unwrap_or_else(|| "No description".to_string())
    }

    fn get_stats(&self, item: ItemTypeId) -> Vec<(String, String)> {
        self.stats.get(&item).cloned().unwrap_or_default()
    }

    fn get_rarity(&self, item: ItemTypeId) -> Option<String> {
        self.rarities.get(&item).cloned()
    }
}

/// Inventory UI model for rendering.
#[derive(Debug, Clone)]
pub struct InventoryUIModel {
    /// All inventory slots
    pub slots: Vec<SlotUIData>,
    /// Currently selected slot index
    pub selected_slot: Option<usize>,
    /// Item being dragged
    pub drag_item: Option<ItemStack>,
    /// Slot being dragged from
    pub drag_source: Option<usize>,
    /// Current tooltip data
    pub tooltip: Option<TooltipData>,
    /// Hovered slot index
    pub hovered_slot: Option<usize>,
    /// Number of hotbar slots
    pub hotbar_size: usize,
    /// Total capacity
    pub capacity: usize,
}

impl InventoryUIModel {
    /// Creates a new inventory UI model from an inventory.
    #[must_use]
    pub fn from_inventory(inv: &Inventory, hotbar_size: usize) -> Self {
        let capacity = inv.capacity() as usize;
        let mut slots = Vec::with_capacity(capacity);

        // Create slot UI data for each slot
        for i in 0..capacity {
            let mut slot = SlotUIData::new(i);
            slot.is_hotbar = i < hotbar_size;
            slots.push(slot);
        }

        // Fill in items from inventory
        // Note: Inventory uses HashMap<ItemTypeId, u32>, so we need to convert
        let mut slot_idx = 0;
        for (item_type, quantity) in inv.iter() {
            if slot_idx < capacity {
                slots[slot_idx].item = Some(ItemStack::new(item_type, quantity));
                slot_idx += 1;
            }
        }

        Self {
            slots,
            selected_slot: None,
            drag_item: None,
            drag_source: None,
            tooltip: None,
            hovered_slot: None,
            hotbar_size,
            capacity,
        }
    }

    /// Creates an empty UI model with given capacity.
    #[must_use]
    pub fn new(capacity: usize, hotbar_size: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let mut slot = SlotUIData::new(i);
            slot.is_hotbar = i < hotbar_size;
            slots.push(slot);
        }

        Self {
            slots,
            selected_slot: None,
            drag_item: None,
            drag_source: None,
            tooltip: None,
            hovered_slot: None,
            hotbar_size,
            capacity,
        }
    }

    /// Sets the selected slot.
    pub fn select_slot(&mut self, slot: Option<usize>) {
        // Clear previous selection
        if let Some(prev) = self.selected_slot {
            if prev < self.slots.len() {
                self.slots[prev].is_selected = false;
            }
        }

        // Set new selection
        self.selected_slot = slot;
        if let Some(idx) = slot {
            if idx < self.slots.len() {
                self.slots[idx].is_selected = true;
            }
        }
    }

    /// Sets the hovered slot.
    pub fn set_hovered(&mut self, slot: Option<usize>) {
        // Clear previous hover
        if let Some(prev) = self.hovered_slot {
            if prev < self.slots.len() {
                self.slots[prev].is_hovered = false;
            }
        }

        // Set new hover
        self.hovered_slot = slot;
        if let Some(idx) = slot {
            if idx < self.slots.len() {
                self.slots[idx].is_hovered = true;
            }
        }
    }

    /// Generates tooltip for a slot using metadata provider.
    pub fn generate_tooltip<M: ItemMetadata>(&mut self, slot: usize, metadata: &M) {
        if slot >= self.slots.len() {
            self.tooltip = None;
            return;
        }

        let slot_data = &self.slots[slot];
        if let Some(stack) = &slot_data.item {
            let item = stack.item_type;
            let mut tooltip = TooltipData::new(metadata.get_name(item))
                .with_description(metadata.get_description(item));

            // Add stats
            for (name, value) in metadata.get_stats(item) {
                tooltip = tooltip.with_stat(name, value);
            }

            // Add rarity
            if let Some(rarity) = metadata.get_rarity(item) {
                tooltip = tooltip.with_rarity(rarity);
            }

            // Add stack info
            tooltip = tooltip.with_stack_info(format!("Stack: {}", stack.quantity));

            self.tooltip = Some(tooltip);
        } else {
            self.tooltip = None;
        }
    }

    /// Clears the tooltip.
    pub fn clear_tooltip(&mut self) {
        self.tooltip = None;
    }

    /// Handles a click on a slot.
    #[must_use]
    pub fn handle_click(&mut self, slot: usize, button: MouseButton) -> InventoryAction {
        if slot >= self.slots.len() {
            return InventoryAction::None;
        }

        match button {
            MouseButton::Left => {
                if self.drag_item.is_some() {
                    // We're dragging - try to place item
                    if let Some(source) = self.drag_source {
                        let action = if self.slots[slot].is_empty() {
                            InventoryAction::Move {
                                from: source,
                                to: slot,
                            }
                        } else {
                            InventoryAction::Swap {
                                slot_a: source,
                                slot_b: slot,
                            }
                        };

                        // Clear drag state
                        self.drag_item = None;
                        self.drag_source = None;

                        return action;
                    }
                } else if !self.slots[slot].is_empty() {
                    // Start dragging
                    self.drag_item = self.slots[slot].item;
                    self.drag_source = Some(slot);
                }
                InventoryAction::None
            },
            MouseButton::Right => {
                if self.drag_item.is_some() {
                    // Place single item from drag stack
                    if self.drag_source.is_some() {
                        // For right-click while dragging, we could implement
                        // placing a single item, but for now just cancel
                        self.drag_item = None;
                        self.drag_source = None;
                        return InventoryAction::None;
                    }
                }

                if !self.slots[slot].is_empty() {
                    // Right-click on item - split stack
                    return InventoryAction::Split { slot };
                }
                InventoryAction::None
            },
            MouseButton::Middle => {
                // Middle-click for quick actions (e.g., quick-move)
                if !self.slots[slot].is_empty() {
                    return InventoryAction::QuickMove { slot };
                }
                InventoryAction::None
            },
        }
    }

    /// Handles a drag operation.
    #[must_use]
    pub fn handle_drag(&mut self, from: usize, to: usize) -> InventoryAction {
        if from >= self.slots.len() || to >= self.slots.len() {
            return InventoryAction::None;
        }

        if from == to {
            return InventoryAction::None;
        }

        // Clear drag state
        self.drag_item = None;
        self.drag_source = None;

        if self.slots[to].is_empty() {
            InventoryAction::Move { from, to }
        } else {
            InventoryAction::Swap {
                slot_a: from,
                slot_b: to,
            }
        }
    }

    /// Handles a drop action (dropping items on ground).
    #[must_use]
    pub fn handle_drop(&mut self, slot: usize, drop_all: bool) -> InventoryAction {
        if slot >= self.slots.len() || self.slots[slot].is_empty() {
            return InventoryAction::None;
        }

        let count = if drop_all {
            self.slots[slot].quantity()
        } else {
            1
        };

        InventoryAction::Drop { slot, count }
    }

    /// Handles using an item.
    #[must_use]
    pub fn handle_use(&mut self, slot: usize) -> InventoryAction {
        if slot >= self.slots.len() || self.slots[slot].is_empty() {
            return InventoryAction::None;
        }

        InventoryAction::Use { slot }
    }

    /// Cancels current drag operation.
    pub fn cancel_drag(&mut self) {
        self.drag_item = None;
        self.drag_source = None;
    }

    /// Check if currently dragging.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.drag_item.is_some()
    }

    /// Gets hotbar slots.
    #[must_use]
    pub fn hotbar_slots(&self) -> &[SlotUIData] {
        &self.slots[..self.hotbar_size.min(self.slots.len())]
    }

    /// Gets main inventory slots (non-hotbar).
    #[must_use]
    pub fn main_slots(&self) -> &[SlotUIData] {
        if self.hotbar_size >= self.slots.len() {
            &[]
        } else {
            &self.slots[self.hotbar_size..]
        }
    }

    /// Gets a slot by index.
    #[must_use]
    pub fn get_slot(&self, index: usize) -> Option<&SlotUIData> {
        self.slots.get(index)
    }

    /// Gets the total number of items across all slots.
    #[must_use]
    pub fn total_items(&self) -> u32 {
        self.slots.iter().map(SlotUIData::quantity).sum()
    }

    /// Gets the number of occupied slots.
    #[must_use]
    pub fn occupied_slots(&self) -> usize {
        self.slots.iter().filter(|s| !s.is_empty()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_inventory() -> Inventory {
        let mut inv = Inventory::new(10);
        let _ = inv.add(ItemTypeId::new(1), 5);
        let _ = inv.add(ItemTypeId::new(2), 10);
        inv
    }

    #[test]
    fn test_slot_ui_data_creation() {
        let slot = SlotUIData::new(0);
        assert_eq!(slot.index, 0);
        assert!(slot.is_empty());
        assert!(!slot.is_hotbar);
        assert!(!slot.is_selected);
    }

    #[test]
    fn test_slot_ui_data_with_item() {
        let slot = SlotUIData::new(0)
            .with_item(ItemStack::new(ItemTypeId::new(1), 5))
            .hotbar(true)
            .selected(true);

        assert!(!slot.is_empty());
        assert_eq!(slot.quantity(), 5);
        assert!(slot.is_hotbar);
        assert!(slot.is_selected);
    }

    #[test]
    fn test_tooltip_data_creation() {
        let tooltip = TooltipData::new("Iron Sword")
            .with_description("A sturdy iron sword")
            .with_stat("Damage", "10")
            .with_stat("Durability", "100/100")
            .with_rarity("Common");

        assert_eq!(tooltip.item_name, "Iron Sword");
        assert_eq!(tooltip.description, "A sturdy iron sword");
        assert_eq!(tooltip.stats.len(), 2);
        assert_eq!(tooltip.rarity, Some("Common".to_string()));
    }

    #[test]
    fn test_simple_item_metadata() {
        let mut metadata = SimpleItemMetadata::new();
        let item = ItemTypeId::new(1);

        metadata.register(item, "Stone", "A piece of stone");
        metadata.set_stats(item, vec![("Weight".to_string(), "1kg".to_string())]);
        metadata.set_rarity(item, "Common");

        assert_eq!(metadata.get_name(item), "Stone");
        assert_eq!(metadata.get_description(item), "A piece of stone");
        assert_eq!(metadata.get_stats(item).len(), 1);
        assert_eq!(metadata.get_rarity(item), Some("Common".to_string()));
    }

    #[test]
    fn test_simple_item_metadata_unknown() {
        let metadata = SimpleItemMetadata::new();
        let item = ItemTypeId::new(999);

        assert_eq!(metadata.get_name(item), "Item #999");
        assert_eq!(metadata.get_description(item), "No description");
    }

    #[test]
    fn test_inventory_ui_model_from_inventory() {
        let inv = create_test_inventory();
        let model = InventoryUIModel::from_inventory(&inv, 4);

        assert_eq!(model.capacity, 10);
        assert_eq!(model.hotbar_size, 4);
        assert_eq!(model.occupied_slots(), 2);
    }

    #[test]
    fn test_inventory_ui_model_new() {
        let model = InventoryUIModel::new(20, 8);

        assert_eq!(model.capacity, 20);
        assert_eq!(model.hotbar_size, 8);
        assert_eq!(model.slots.len(), 20);
        assert!(model.slots[0].is_hotbar);
        assert!(model.slots[7].is_hotbar);
        assert!(!model.slots[8].is_hotbar);
    }

    #[test]
    fn test_select_slot() {
        let mut model = InventoryUIModel::new(10, 4);

        model.select_slot(Some(2));
        assert_eq!(model.selected_slot, Some(2));
        assert!(model.slots[2].is_selected);

        model.select_slot(Some(5));
        assert_eq!(model.selected_slot, Some(5));
        assert!(!model.slots[2].is_selected);
        assert!(model.slots[5].is_selected);

        model.select_slot(None);
        assert!(model.selected_slot.is_none());
        assert!(!model.slots[5].is_selected);
    }

    #[test]
    fn test_set_hovered() {
        let mut model = InventoryUIModel::new(10, 4);

        model.set_hovered(Some(3));
        assert_eq!(model.hovered_slot, Some(3));
        assert!(model.slots[3].is_hovered);

        model.set_hovered(None);
        assert!(model.hovered_slot.is_none());
        assert!(!model.slots[3].is_hovered);
    }

    #[test]
    fn test_generate_tooltip() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        let mut metadata = SimpleItemMetadata::new();
        metadata.register(ItemTypeId::new(1), "Stone", "A piece of stone");

        model.generate_tooltip(0, &metadata);

        assert!(model.tooltip.is_some());
        let tooltip = model.tooltip.as_ref().expect("tooltip should exist");
        assert_eq!(tooltip.item_name, "Stone");
    }

    #[test]
    fn test_generate_tooltip_empty_slot() {
        let mut model = InventoryUIModel::new(10, 4);
        let metadata = SimpleItemMetadata::new();

        model.generate_tooltip(0, &metadata);
        assert!(model.tooltip.is_none());
    }

    #[test]
    fn test_handle_click_start_drag() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        let action = model.handle_click(0, MouseButton::Left);

        assert_eq!(action, InventoryAction::None);
        assert!(model.is_dragging());
        assert_eq!(model.drag_source, Some(0));
    }

    #[test]
    fn test_handle_click_drop_drag() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        // Start drag
        model.handle_click(0, MouseButton::Left);

        // Drop on empty slot
        let action = model.handle_click(1, MouseButton::Left);

        assert_eq!(action, InventoryAction::Move { from: 0, to: 1 });
        assert!(!model.is_dragging());
    }

    #[test]
    fn test_handle_click_swap() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));
        model.slots[1].item = Some(ItemStack::new(ItemTypeId::new(2), 10));

        // Start drag
        model.handle_click(0, MouseButton::Left);

        // Drop on occupied slot
        let action = model.handle_click(1, MouseButton::Left);

        assert_eq!(
            action,
            InventoryAction::Swap {
                slot_a: 0,
                slot_b: 1
            }
        );
    }

    #[test]
    fn test_handle_click_right_split() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 10));

        let action = model.handle_click(0, MouseButton::Right);
        assert_eq!(action, InventoryAction::Split { slot: 0 });
    }

    #[test]
    fn test_handle_click_middle_quick_move() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        let action = model.handle_click(0, MouseButton::Middle);
        assert_eq!(action, InventoryAction::QuickMove { slot: 0 });
    }

    #[test]
    fn test_handle_drag() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        let action = model.handle_drag(0, 5);
        assert_eq!(action, InventoryAction::Move { from: 0, to: 5 });
    }

    #[test]
    fn test_handle_drag_same_slot() {
        let mut model = InventoryUIModel::new(10, 4);

        let action = model.handle_drag(0, 0);
        assert_eq!(action, InventoryAction::None);
    }

    #[test]
    fn test_handle_drop() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        let action = model.handle_drop(0, false);
        assert_eq!(action, InventoryAction::Drop { slot: 0, count: 1 });

        let action = model.handle_drop(0, true);
        assert_eq!(action, InventoryAction::Drop { slot: 0, count: 5 });
    }

    #[test]
    fn test_handle_use() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        let action = model.handle_use(0);
        assert_eq!(action, InventoryAction::Use { slot: 0 });

        let action = model.handle_use(1); // Empty slot
        assert_eq!(action, InventoryAction::None);
    }

    #[test]
    fn test_cancel_drag() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));

        model.handle_click(0, MouseButton::Left);
        assert!(model.is_dragging());

        model.cancel_drag();
        assert!(!model.is_dragging());
    }

    #[test]
    fn test_hotbar_and_main_slots() {
        let mut model = InventoryUIModel::new(10, 4);
        for i in 0..10 {
            model.slots[i].item = Some(ItemStack::new(ItemTypeId::new(i as u32 + 1), 1));
        }

        let hotbar = model.hotbar_slots();
        assert_eq!(hotbar.len(), 4);
        assert!(hotbar[0].is_hotbar);

        let main = model.main_slots();
        assert_eq!(main.len(), 6);
        assert!(!main[0].is_hotbar);
    }

    #[test]
    fn test_total_items() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));
        model.slots[1].item = Some(ItemStack::new(ItemTypeId::new(2), 10));

        assert_eq!(model.total_items(), 15);
    }

    #[test]
    fn test_occupied_slots() {
        let mut model = InventoryUIModel::new(10, 4);
        model.slots[0].item = Some(ItemStack::new(ItemTypeId::new(1), 5));
        model.slots[5].item = Some(ItemStack::new(ItemTypeId::new(2), 10));

        assert_eq!(model.occupied_slots(), 2);
    }
}
