//! Inventory Panel UI for displaying a 6x9 grid of item slots.
//!
//! This module provides:
//! - 6x9 grid layout (54 slots)
//! - Item icon display with count overlay
//! - Tab key toggle
//! - Centered panel with close button

use egui::{
    Align2, Color32, Context, FontId, Id, Key, Response, RichText, Rounding, Sense, Stroke, Ui,
    Vec2, Window,
};
use serde::{Deserialize, Serialize};

/// Number of inventory columns.
pub const INVENTORY_COLS: usize = 9;

/// Number of inventory rows.
pub const INVENTORY_ROWS: usize = 6;

/// Total inventory slots.
pub const INVENTORY_SLOTS: usize = INVENTORY_COLS * INVENTORY_ROWS;

/// Default slot size in pixels.
pub const INVENTORY_SLOT_SIZE: f32 = 48.0;

/// An item in the inventory.
#[derive(Debug, Clone, Default)]
pub struct InventoryItem {
    /// Item ID (0 = empty).
    pub id: u32,
    /// Item display name.
    pub name: String,
    /// Item count.
    pub count: u32,
    /// Item icon color (fallback).
    pub color: [u8; 4],
    /// Item rarity (0-4).
    pub rarity: u8,
}

impl InventoryItem {
    /// Creates an empty slot.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates an item with the given properties.
    #[must_use]
    pub fn new(id: u32, name: impl Into<String>, count: u32, color: [u8; 4]) -> Self {
        Self {
            id,
            name: name.into(),
            count,
            color,
            rarity: 0,
        }
    }

    /// Returns whether the slot is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.id == 0 || self.count == 0
    }

    /// Returns the rarity color.
    #[must_use]
    pub fn rarity_color(&self) -> Color32 {
        match self.rarity {
            1 => Color32::GREEN,                  // Uncommon
            2 => Color32::from_rgb(30, 144, 255), // Rare (blue)
            3 => Color32::from_rgb(163, 53, 238), // Epic (purple)
            4 => Color32::GOLD,                   // Legendary
            _ => Color32::WHITE,                  // Common or unknown
        }
    }
}

/// Inventory panel data model.
#[derive(Debug, Clone)]
pub struct InventoryPanelModel {
    /// Inventory slots (54 total).
    pub slots: Vec<InventoryItem>,
    /// Currently selected slot index.
    pub selected_slot: Option<usize>,
}

impl Default for InventoryPanelModel {
    fn default() -> Self {
        Self {
            slots: vec![InventoryItem::empty(); INVENTORY_SLOTS],
            selected_slot: None,
        }
    }
}

impl InventoryPanelModel {
    /// Creates a new inventory model.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an item in a slot.
    pub fn set_item(&mut self, index: usize, item: InventoryItem) {
        if index < self.slots.len() {
            self.slots[index] = item;
        }
    }

    /// Gets an item from a slot.
    #[must_use]
    pub fn get_item(&self, index: usize) -> Option<&InventoryItem> {
        self.slots.get(index)
    }

    /// Clears a slot.
    pub fn clear_slot(&mut self, index: usize) {
        if index < self.slots.len() {
            self.slots[index] = InventoryItem::empty();
        }
    }
}

/// Actions from inventory panel interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryPanelAction {
    /// Close the panel.
    Close,
    /// Select a slot.
    SelectSlot(usize),
    /// Use item in slot.
    UseItem(usize),
    /// Drop item from slot.
    DropItem(usize),
}

/// Configuration for the inventory panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryPanelConfig {
    /// Slot size in pixels.
    pub slot_size: f32,
    /// Padding between slots.
    pub slot_padding: f32,
    /// Background color.
    pub background_color: [u8; 4],
    /// Slot color.
    pub slot_color: [u8; 4],
    /// Selected slot color.
    pub selected_color: [u8; 4],
    /// Hover slot color.
    pub hover_color: [u8; 4],
}

impl Default for InventoryPanelConfig {
    fn default() -> Self {
        Self {
            slot_size: INVENTORY_SLOT_SIZE,
            slot_padding: 4.0,
            background_color: [30, 30, 30, 240],
            slot_color: [50, 50, 50, 255],
            selected_color: [80, 120, 180, 255],
            hover_color: [70, 70, 70, 255],
        }
    }
}

/// Inventory panel widget.
#[derive(Debug)]
pub struct InventoryPanel {
    /// Whether the panel is visible.
    is_open: bool,
    /// Configuration.
    config: InventoryPanelConfig,
}

impl Default for InventoryPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryPanel {
    /// Creates a new inventory panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_open: false,
            config: InventoryPanelConfig::default(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: InventoryPanelConfig) -> Self {
        Self {
            is_open: false,
            config,
        }
    }

    /// Returns whether the panel is open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Opens the panel.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the panel.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Toggles panel visibility.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Shows the inventory panel and returns actions.
    pub fn show(
        &mut self,
        ctx: &Context,
        model: &mut InventoryPanelModel,
    ) -> Vec<InventoryPanelAction> {
        let mut actions = Vec::new();

        // Handle Tab key toggle
        if ctx.input(|i| i.key_pressed(Key::Tab)) {
            self.toggle();
        }

        if !self.is_open {
            return actions;
        }

        // Calculate panel size
        let panel_width = INVENTORY_COLS as f32
            * (self.config.slot_size + self.config.slot_padding)
            + self.config.slot_padding;
        let panel_height = INVENTORY_ROWS as f32
            * (self.config.slot_size + self.config.slot_padding)
            + self.config.slot_padding
            + 30.0; // Title bar

        Window::new("Inventory")
            .id(Id::new("inventory_panel"))
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(panel_width, panel_height))
            .show(ctx, |ui| {
                // Close button
                ui.horizontal(|ui| {
                    ui.heading("Inventory");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            actions.push(InventoryPanelAction::Close);
                            self.is_open = false;
                        }
                    });
                });

                ui.separator();

                // Render grid
                self.render_grid(ui, model, &mut actions);
            });

        actions
    }

    /// Renders the inventory grid.
    fn render_grid(
        &self,
        ui: &mut Ui,
        model: &mut InventoryPanelModel,
        actions: &mut Vec<InventoryPanelAction>,
    ) {
        egui::Grid::new("inventory_grid")
            .spacing(Vec2::splat(self.config.slot_padding))
            .show(ui, |ui| {
                for row in 0..INVENTORY_ROWS {
                    for col in 0..INVENTORY_COLS {
                        let index = row * INVENTORY_COLS + col;
                        let item = &model.slots[index];
                        let is_selected = model.selected_slot == Some(index);

                        let response = self.render_slot(ui, item, is_selected);

                        if response.clicked() {
                            model.selected_slot = Some(index);
                            actions.push(InventoryPanelAction::SelectSlot(index));
                        }

                        if response.double_clicked() && !item.is_empty() {
                            actions.push(InventoryPanelAction::UseItem(index));
                        }

                        if response.secondary_clicked() && !item.is_empty() {
                            actions.push(InventoryPanelAction::DropItem(index));
                        }

                        // Tooltip
                        if response.hovered() && !item.is_empty() {
                            response.on_hover_ui(|ui| {
                                ui.label(
                                    RichText::new(&item.name)
                                        .color(item.rarity_color())
                                        .strong(),
                                );
                                ui.label(format!("Count: {}", item.count));
                            });
                        }
                    }
                    ui.end_row();
                }
            });
    }

    /// Renders a single slot.
    fn render_slot(&self, ui: &mut Ui, item: &InventoryItem, is_selected: bool) -> Response {
        let size = Vec2::splat(self.config.slot_size);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background color
            let bg_color = if is_selected {
                Color32::from_rgba_unmultiplied(
                    self.config.selected_color[0],
                    self.config.selected_color[1],
                    self.config.selected_color[2],
                    self.config.selected_color[3],
                )
            } else if response.hovered() {
                Color32::from_rgba_unmultiplied(
                    self.config.hover_color[0],
                    self.config.hover_color[1],
                    self.config.hover_color[2],
                    self.config.hover_color[3],
                )
            } else {
                Color32::from_rgba_unmultiplied(
                    self.config.slot_color[0],
                    self.config.slot_color[1],
                    self.config.slot_color[2],
                    self.config.slot_color[3],
                )
            };

            painter.rect_filled(rect, Rounding::same(4.0), bg_color);

            // Border
            let border_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::from_gray(80)
            };
            painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0, border_color));

            // Item icon (colored square)
            if !item.is_empty() {
                let icon_rect = rect.shrink(8.0);
                let icon_color = Color32::from_rgba_unmultiplied(
                    item.color[0],
                    item.color[1],
                    item.color[2],
                    item.color[3],
                );
                painter.rect_filled(icon_rect, Rounding::same(2.0), icon_color);

                // Rarity border
                if item.rarity > 0 {
                    painter.rect_stroke(
                        icon_rect,
                        Rounding::same(2.0),
                        Stroke::new(2.0, item.rarity_color()),
                    );
                }

                // Item count
                if item.count > 1 {
                    painter.text(
                        rect.right_bottom() - Vec2::new(4.0, 4.0),
                        Align2::RIGHT_BOTTOM,
                        item.count.to_string(),
                        FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_item_empty() {
        let item = InventoryItem::empty();
        assert!(item.is_empty());
        assert_eq!(item.id, 0);
    }

    #[test]
    fn test_inventory_item_new() {
        let item = InventoryItem::new(1, "Sword", 5, [200, 100, 50, 255]);
        assert!(!item.is_empty());
        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Sword");
        assert_eq!(item.count, 5);
    }

    #[test]
    fn test_inventory_item_rarity_color() {
        let mut item = InventoryItem::new(1, "Item", 1, [255; 4]);
        assert_eq!(item.rarity_color(), Color32::WHITE);

        item.rarity = 1;
        assert_eq!(item.rarity_color(), Color32::GREEN);

        item.rarity = 4;
        assert_eq!(item.rarity_color(), Color32::GOLD);
    }

    #[test]
    fn test_inventory_panel_model_default() {
        let model = InventoryPanelModel::default();
        assert_eq!(model.slots.len(), INVENTORY_SLOTS);
        assert!(model.selected_slot.is_none());
    }

    #[test]
    fn test_inventory_panel_model_set_get() {
        let mut model = InventoryPanelModel::new();
        let item = InventoryItem::new(42, "Test", 10, [255; 4]);
        model.set_item(5, item.clone());

        let retrieved = model.get_item(5).expect("Item should exist");
        assert_eq!(retrieved.id, 42);
        assert_eq!(retrieved.count, 10);
    }

    #[test]
    fn test_inventory_panel_model_clear_slot() {
        let mut model = InventoryPanelModel::new();
        model.set_item(0, InventoryItem::new(1, "Item", 1, [255; 4]));
        assert!(!model.slots[0].is_empty());

        model.clear_slot(0);
        assert!(model.slots[0].is_empty());
    }

    #[test]
    fn test_inventory_panel_toggle() {
        let mut panel = InventoryPanel::new();
        assert!(!panel.is_open());

        panel.toggle();
        assert!(panel.is_open());

        panel.toggle();
        assert!(!panel.is_open());
    }

    #[test]
    fn test_inventory_panel_open_close() {
        let mut panel = InventoryPanel::new();
        panel.open();
        assert!(panel.is_open());

        panel.close();
        assert!(!panel.is_open());
    }

    #[test]
    fn test_inventory_panel_config_defaults() {
        let config = InventoryPanelConfig::default();
        assert_eq!(config.slot_size, INVENTORY_SLOT_SIZE);
        assert_eq!(config.slot_padding, 4.0);
    }

    #[test]
    fn test_inventory_panel_action_equality() {
        assert_eq!(
            InventoryPanelAction::SelectSlot(5),
            InventoryPanelAction::SelectSlot(5)
        );
        assert_ne!(
            InventoryPanelAction::SelectSlot(5),
            InventoryPanelAction::SelectSlot(6)
        );
        assert_eq!(InventoryPanelAction::Close, InventoryPanelAction::Close);
    }

    #[test]
    fn test_inventory_constants() {
        assert_eq!(INVENTORY_COLS, 9);
        assert_eq!(INVENTORY_ROWS, 6);
        assert_eq!(INVENTORY_SLOTS, 54);
    }
}
