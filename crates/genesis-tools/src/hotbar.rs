//! Hotbar widget for quick item access.
//!
//! A 10-slot hotbar displayed at the bottom of the screen with mouse wheel cycling.

use egui::{Color32, Context, Pos2, Rect, Response, Stroke, Ui, Vec2};
use genesis_gameplay::inventory::{Inventory, ItemStack};

/// The hotbar widget for quick item access.
#[derive(Debug, Clone)]
pub struct Hotbar {
    /// Currently selected slot (0-9).
    selected_slot: usize,
    /// Size of each slot in pixels.
    slot_size: f32,
    /// Spacing between slots.
    slot_spacing: f32,
    /// Items stored in the hotbar slots (separate from main inventory).
    slots: [Option<HotbarSlot>; Self::SLOT_COUNT],
}

/// Data for an item in a hotbar slot.
#[derive(Debug, Clone, Copy)]
pub struct HotbarSlot {
    /// The item stack in this slot.
    pub item: ItemStack,
}

/// Result of hotbar interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotbarAction {
    /// No action taken.
    None,
    /// Slot was selected.
    SlotSelected(usize),
    /// Item in slot was used.
    ItemUsed(usize),
}

impl Default for Hotbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Hotbar {
    /// Number of slots in the hotbar.
    pub const SLOT_COUNT: usize = 10;

    /// Default slot size in pixels.
    pub const DEFAULT_SLOT_SIZE: f32 = 48.0;

    /// Default spacing between slots.
    pub const DEFAULT_SPACING: f32 = 4.0;

    /// Create a new hotbar with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            selected_slot: 0,
            slot_size: Self::DEFAULT_SLOT_SIZE,
            slot_spacing: Self::DEFAULT_SPACING,
            slots: [None; Self::SLOT_COUNT],
        }
    }

    /// Create a new hotbar with custom slot size.
    #[must_use]
    pub fn with_slot_size(slot_size: f32, spacing: f32) -> Self {
        Self {
            selected_slot: 0,
            slot_size,
            slot_spacing: spacing,
            slots: [None; Self::SLOT_COUNT],
        }
    }

    /// Render the hotbar, returns action if slot selection changed.
    pub fn render(&mut self, ctx: &Context, inventory: &Inventory) -> HotbarAction {
        let screen_rect = ctx.screen_rect();
        let hotbar_width =
            Self::SLOT_COUNT as f32 * (self.slot_size + self.slot_spacing) - self.slot_spacing;
        let padding = 20.0;

        let pos = Pos2::new(
            (screen_rect.width() - hotbar_width) / 2.0,
            screen_rect.height() - padding - self.slot_size,
        );

        let mut action = HotbarAction::None;

        egui::Area::new(egui::Id::new("hotbar_widget"))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for slot_index in 0..Self::SLOT_COUNT {
                        let is_selected = slot_index == self.selected_slot;
                        let item = self.get_slot_item(slot_index, inventory);

                        let response = self.render_slot(ui, slot_index, item.as_ref(), is_selected);

                        if response.clicked() {
                            if is_selected {
                                action = HotbarAction::ItemUsed(slot_index);
                            } else {
                                self.selected_slot = slot_index;
                                action = HotbarAction::SlotSelected(slot_index);
                            }
                        }
                    }
                });
            });

        action
    }

    /// Get item for a slot, checking hotbar slots first, then inventory.
    fn get_slot_item(&self, slot: usize, _inventory: &Inventory) -> Option<ItemStack> {
        self.slots.get(slot).and_then(|s| s.map(|hs| hs.item))
    }

    /// Select slot by number (0-9).
    ///
    /// Returns true if the selection changed.
    pub fn select_slot(&mut self, slot: usize) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        let changed = self.selected_slot != slot;
        self.selected_slot = slot;
        changed
    }

    /// Select slot by key number (1-9 selects 0-8, 0 selects 9).
    ///
    /// Returns true if the selection changed.
    pub fn select_by_key(&mut self, key: u8) -> bool {
        let slot = if key == 0 { 9 } else { (key - 1) as usize };
        self.select_slot(slot)
    }

    /// Select next/previous slot (mouse wheel).
    ///
    /// Positive delta selects next slot, negative selects previous.
    #[allow(clippy::cast_possible_wrap)]
    pub fn cycle_slot(&mut self, delta: i32) {
        // Safe: SLOT_COUNT is 10 and selected_slot is always < 10
        let current = self.selected_slot as i32;
        let count = Self::SLOT_COUNT as i32;
        let new_slot = (current - delta).rem_euclid(count) as usize;
        self.selected_slot = new_slot;
    }

    /// Get currently selected slot index.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected_slot
    }

    /// Get the item in the selected slot.
    #[must_use]
    pub fn selected_item(&self, inventory: &Inventory) -> Option<ItemStack> {
        self.get_slot_item(self.selected_slot, inventory)
    }

    /// Set an item in a hotbar slot.
    pub fn set_slot(&mut self, slot: usize, item: Option<ItemStack>) {
        if slot < Self::SLOT_COUNT {
            self.slots[slot] = item.map(|i| HotbarSlot { item: i });
        }
    }

    /// Clear a hotbar slot.
    pub fn clear_slot(&mut self, slot: usize) {
        if slot < Self::SLOT_COUNT {
            self.slots[slot] = None;
        }
    }

    /// Clear all hotbar slots.
    pub fn clear_all(&mut self) {
        self.slots = [None; Self::SLOT_COUNT];
    }

    /// Get slot size.
    #[must_use]
    pub const fn slot_size(&self) -> f32 {
        self.slot_size
    }

    /// Set slot size.
    pub fn set_slot_size(&mut self, size: f32) {
        self.slot_size = size;
    }

    /// Render single slot with item.
    fn render_slot(
        &self,
        ui: &mut Ui,
        slot: usize,
        item: Option<&ItemStack>,
        selected: bool,
    ) -> Response {
        let size = Vec2::splat(self.slot_size);
        let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
        let rect = response.rect;

        // Background color based on selection
        let bg_color = if selected {
            Color32::from_rgb(80, 80, 120)
        } else if response.hovered() {
            Color32::from_rgb(60, 60, 70)
        } else {
            Color32::from_rgb(40, 40, 50)
        };

        // Draw background
        painter.rect_filled(rect, 4.0, bg_color);

        // Draw border
        let border_color = if selected {
            Color32::from_rgb(200, 200, 255)
        } else {
            Color32::from_rgb(80, 80, 80)
        };
        let border_width = if selected { 2.0 } else { 1.0 };
        painter.rect_stroke(rect, 4.0, Stroke::new(border_width, border_color));

        // Draw key number hint (1-9, 0)
        let key_text = key_label(slot);
        painter.text(
            rect.min + Vec2::new(4.0, 2.0),
            egui::Align2::LEFT_TOP,
            key_text,
            egui::FontId::proportional(10.0),
            Color32::from_rgb(150, 150, 150),
        );

        // Draw item if present
        if let Some(stack) = item {
            // Item icon placeholder (colored square based on item type)
            let icon_rect =
                Rect::from_center_size(rect.center(), Vec2::splat(self.slot_size - 16.0));
            let item_color = item_color(stack.item_type.raw());
            painter.rect_filled(icon_rect, 2.0, item_color);

            // Item count (if more than 1)
            if stack.quantity > 1 {
                let count_text = format!("{}", stack.quantity);
                painter.text(
                    rect.max - Vec2::new(4.0, 4.0),
                    egui::Align2::RIGHT_BOTTOM,
                    count_text,
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
        }

        response
    }
}

/// Get the key label for a slot index.
fn key_label(slot: usize) -> &'static str {
    match slot {
        0 => "1",
        1 => "2",
        2 => "3",
        3 => "4",
        4 => "5",
        5 => "6",
        6 => "7",
        7 => "8",
        8 => "9",
        9 => "0",
        _ => "?",
    }
}

/// Generate a color for an item based on its type ID.
fn item_color(item_id: u32) -> Color32 {
    // Simple hash-based color for variety
    let r = ((item_id * 123) % 200 + 55) as u8;
    let g = ((item_id * 456) % 200 + 55) as u8;
    let b = ((item_id * 789) % 200 + 55) as u8;
    Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_common::ItemTypeId;

    #[test]
    fn test_hotbar_new() {
        let hotbar = Hotbar::new();
        assert_eq!(hotbar.selected(), 0);
        assert_eq!(hotbar.slot_size(), Hotbar::DEFAULT_SLOT_SIZE);
    }

    #[test]
    fn test_hotbar_select_slot() {
        let mut hotbar = Hotbar::new();
        assert!(hotbar.select_slot(5));
        assert_eq!(hotbar.selected(), 5);

        // Selecting same slot returns false
        assert!(!hotbar.select_slot(5));

        // Invalid slot doesn't change selection
        assert!(!hotbar.select_slot(15));
        assert_eq!(hotbar.selected(), 5);
    }

    #[test]
    fn test_hotbar_select_by_key() {
        let mut hotbar = Hotbar::new();

        // Key 1 selects slot 0
        hotbar.select_by_key(1);
        assert_eq!(hotbar.selected(), 0);

        // Key 5 selects slot 4
        hotbar.select_by_key(5);
        assert_eq!(hotbar.selected(), 4);

        // Key 0 selects slot 9
        hotbar.select_by_key(0);
        assert_eq!(hotbar.selected(), 9);
    }

    #[test]
    fn test_hotbar_cycle_slot() {
        let mut hotbar = Hotbar::new();
        hotbar.select_slot(0);

        // Cycle forward
        hotbar.cycle_slot(1);
        assert_eq!(hotbar.selected(), 9); // Wraps from 0 to 9

        // Cycle backward
        hotbar.cycle_slot(-1);
        assert_eq!(hotbar.selected(), 0); // Wraps from 9 to 0

        // Multiple cycle
        hotbar.select_slot(5);
        hotbar.cycle_slot(3);
        assert_eq!(hotbar.selected(), 2);
    }

    #[test]
    fn test_hotbar_set_slot() {
        let mut hotbar = Hotbar::new();
        let item = ItemStack::new(ItemTypeId::new(1), 10);

        hotbar.set_slot(0, Some(item));

        let inventory = Inventory::new(10);
        let retrieved = hotbar.get_slot_item(0, &inventory);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.map(|i| i.quantity), Some(10));
    }

    #[test]
    fn test_hotbar_clear_slot() {
        let mut hotbar = Hotbar::new();
        let item = ItemStack::new(ItemTypeId::new(1), 10);

        hotbar.set_slot(0, Some(item));
        hotbar.clear_slot(0);

        let inventory = Inventory::new(10);
        assert!(hotbar.get_slot_item(0, &inventory).is_none());
    }

    #[test]
    fn test_hotbar_clear_all() {
        let mut hotbar = Hotbar::new();
        let item1 = ItemStack::new(ItemTypeId::new(1), 5);
        let item2 = ItemStack::new(ItemTypeId::new(2), 10);

        hotbar.set_slot(0, Some(item1));
        hotbar.set_slot(5, Some(item2));
        hotbar.clear_all();

        let inventory = Inventory::new(10);
        assert!(hotbar.get_slot_item(0, &inventory).is_none());
        assert!(hotbar.get_slot_item(5, &inventory).is_none());
    }

    #[test]
    fn test_hotbar_selected_item() {
        let mut hotbar = Hotbar::new();
        let item = ItemStack::new(ItemTypeId::new(42), 5);
        let inventory = Inventory::new(10);

        hotbar.set_slot(3, Some(item));
        hotbar.select_slot(3);

        let selected = hotbar.selected_item(&inventory);
        assert!(selected.is_some());
        assert_eq!(selected.map(|i| i.item_type.raw()), Some(42));
    }

    #[test]
    fn test_hotbar_with_slot_size() {
        let hotbar = Hotbar::with_slot_size(64.0, 8.0);
        assert_eq!(hotbar.slot_size(), 64.0);
    }

    #[test]
    fn test_hotbar_set_slot_size() {
        let mut hotbar = Hotbar::new();
        hotbar.set_slot_size(72.0);
        assert_eq!(hotbar.slot_size(), 72.0);
    }

    #[test]
    fn test_hotbar_action_equality() {
        assert_eq!(HotbarAction::None, HotbarAction::None);
        assert_eq!(HotbarAction::SlotSelected(5), HotbarAction::SlotSelected(5));
        assert_ne!(HotbarAction::SlotSelected(5), HotbarAction::SlotSelected(3));
        assert_ne!(HotbarAction::SlotSelected(5), HotbarAction::ItemUsed(5));
    }

    #[test]
    fn test_key_label() {
        assert_eq!(key_label(0), "1");
        assert_eq!(key_label(8), "9");
        assert_eq!(key_label(9), "0");
        assert_eq!(key_label(10), "?");
    }

    #[test]
    fn test_item_color_deterministic() {
        let color1 = item_color(42);
        let color2 = item_color(42);
        assert_eq!(color1, color2);

        // Different IDs should produce different colors
        let color3 = item_color(100);
        assert_ne!(color1, color3);
    }
}
