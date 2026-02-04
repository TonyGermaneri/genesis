//! Inventory UI renderer using egui.
//!
//! This module provides:
//! - Grid layout for inventory slots
//! - Hotbar always visible at bottom
//! - Drag and drop between slots
//! - Right-click context menu
//! - Tooltip on hover
//! - Item count overlay

use egui::{
    Align2, Color32, Context, FontId, Id, Pos2, Response, RichText, Rounding, Sense, Stroke, Ui,
    Vec2, Window,
};
use genesis_common::ItemTypeId;
use serde::{Deserialize, Serialize};

/// Default slot size in pixels.
pub const DEFAULT_SLOT_SIZE: f32 = 48.0;

/// Number of hotbar slots.
pub const HOTBAR_SLOTS: usize = 10;

/// Default inventory rows.
pub const DEFAULT_INVENTORY_ROWS: usize = 4;

/// Default inventory columns.
pub const DEFAULT_INVENTORY_COLS: usize = 10;

/// UI data for a single inventory slot.
#[derive(Debug, Clone, Default)]
pub struct SlotUIData {
    /// Slot index
    pub slot_index: usize,
    /// Item type ID (None if empty)
    pub item_type: Option<ItemTypeId>,
    /// Item name for display
    pub item_name: String,
    /// Item count
    pub count: u32,
    /// Maximum stack size
    pub max_stack: u32,
    /// Icon color (fallback if no texture)
    pub icon_color: [u8; 4],
    /// Whether this slot is selected
    pub is_selected: bool,
    /// Whether this slot is highlighted (e.g., valid drop target)
    pub is_highlighted: bool,
}

impl SlotUIData {
    /// Creates an empty slot.
    #[must_use]
    pub fn empty(slot_index: usize) -> Self {
        Self {
            slot_index,
            item_type: None,
            item_name: String::new(),
            count: 0,
            max_stack: 999,
            icon_color: [128, 128, 128, 255],
            is_selected: false,
            is_highlighted: false,
        }
    }

    /// Creates a slot with an item.
    #[must_use]
    pub fn with_item(
        slot_index: usize,
        item_type: ItemTypeId,
        name: impl Into<String>,
        count: u32,
    ) -> Self {
        Self {
            slot_index,
            item_type: Some(item_type),
            item_name: name.into(),
            count,
            max_stack: 999,
            icon_color: [200, 200, 200, 255],
            is_selected: false,
            is_highlighted: false,
        }
    }

    /// Returns whether the slot is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.item_type.is_none() || self.count == 0
    }
}

/// Tooltip data for an item.
#[derive(Debug, Clone, Default)]
pub struct TooltipData {
    /// Item name
    pub name: String,
    /// Item description
    pub description: String,
    /// Item stats (key-value pairs)
    pub stats: Vec<(String, String)>,
    /// Rarity color
    pub rarity_color: [u8; 4],
}

impl TooltipData {
    /// Creates tooltip data from a slot.
    #[must_use]
    pub fn from_slot(slot: &SlotUIData) -> Self {
        Self {
            name: slot.item_name.clone(),
            description: String::new(),
            stats: vec![(
                "Stack".to_string(),
                format!("{}/{}", slot.count, slot.max_stack),
            )],
            rarity_color: [255, 255, 255, 255],
        }
    }
}

/// Inventory UI model containing all display data.
#[derive(Debug, Clone, Default)]
pub struct InventoryUIModel {
    /// Main inventory slots
    pub slots: Vec<SlotUIData>,
    /// Hotbar slots (indices into main slots or separate)
    pub hotbar: Vec<SlotUIData>,
    /// Currently selected hotbar index
    pub selected_hotbar: usize,
    /// Currently dragged slot index (if any)
    pub dragging: Option<usize>,
    /// Whether inventory is open
    pub is_open: bool,
}

impl InventoryUIModel {
    /// Creates a new empty inventory model.
    #[must_use]
    pub fn new(slot_count: usize, hotbar_count: usize) -> Self {
        Self {
            slots: (0..slot_count).map(SlotUIData::empty).collect(),
            hotbar: (0..hotbar_count).map(SlotUIData::empty).collect(),
            selected_hotbar: 0,
            dragging: None,
            is_open: false,
        }
    }

    /// Creates with default sizes.
    #[must_use]
    pub fn default_sized() -> Self {
        Self::new(
            DEFAULT_INVENTORY_ROWS * DEFAULT_INVENTORY_COLS,
            HOTBAR_SLOTS,
        )
    }
}

/// Actions that can be performed on the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryAction {
    /// Select a slot
    Select(usize),
    /// Move item from slot to slot
    Move {
        /// Source slot index
        from: usize,
        /// Destination slot index
        to: usize,
    },
    /// Use item in slot
    Use(usize),
    /// Drop item from slot
    Drop(usize),
    /// Split stack (take half)
    Split(usize),
    /// Select hotbar slot
    SelectHotbar(usize),
    /// Open inventory
    Open,
    /// Close inventory
    Close,
}

/// Configuration for the inventory UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryUIConfig {
    /// Size of each slot in pixels
    pub slot_size: f32,
    /// Padding between slots
    pub slot_padding: f32,
    /// Hotbar Y position from bottom
    pub hotbar_y_offset: f32,
    /// Background color
    pub background_color: [u8; 4],
    /// Slot color
    pub slot_color: [u8; 4],
    /// Selected slot color
    pub selected_color: [u8; 4],
    /// Highlight color
    pub highlight_color: [u8; 4],
    /// Number of columns
    pub columns: usize,
}

impl Default for InventoryUIConfig {
    fn default() -> Self {
        Self {
            slot_size: DEFAULT_SLOT_SIZE,
            slot_padding: 4.0,
            hotbar_y_offset: 60.0,
            background_color: [40, 40, 40, 220],
            slot_color: [60, 60, 60, 255],
            selected_color: [100, 150, 200, 255],
            highlight_color: [80, 120, 80, 255],
            columns: DEFAULT_INVENTORY_COLS,
        }
    }
}

/// Inventory UI renderer.
#[derive(Debug)]
pub struct InventoryUI {
    /// Whether the inventory window is open
    pub is_open: bool,
    /// Configuration
    pub config: InventoryUIConfig,
    /// Context menu state
    context_menu_slot: Option<usize>,
    /// Drag source slot
    drag_source: Option<usize>,
}

impl Default for InventoryUI {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryUI {
    /// Creates a new inventory UI.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_open: false,
            config: InventoryUIConfig::default(),
            context_menu_slot: None,
            drag_source: None,
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: InventoryUIConfig) -> Self {
        Self {
            is_open: false,
            config,
            context_menu_slot: None,
            drag_source: None,
        }
    }

    /// Shows the inventory UI and returns any actions.
    pub fn show(&mut self, ctx: &Context, model: &mut InventoryUIModel) -> Vec<InventoryAction> {
        let mut actions = Vec::new();

        // Sync open state
        self.is_open = model.is_open;

        if !self.is_open {
            return actions;
        }

        Window::new("Inventory")
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                self.render_inventory_grid(ui, model, &mut actions);
            });

        // Handle context menu
        if let Some(slot_idx) = self.context_menu_slot {
            let menu_id = Id::new("inventory_context_menu");
            egui::Area::new(menu_id)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if let Some(slot) = model.slots.get(slot_idx) {
                            if !slot.is_empty() {
                                if ui.button("Use").clicked() {
                                    actions.push(InventoryAction::Use(slot_idx));
                                    self.context_menu_slot = None;
                                }
                                if ui.button("Drop").clicked() {
                                    actions.push(InventoryAction::Drop(slot_idx));
                                    self.context_menu_slot = None;
                                }
                                if slot.count > 1 && ui.button("Split").clicked() {
                                    actions.push(InventoryAction::Split(slot_idx));
                                    self.context_menu_slot = None;
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.context_menu_slot = None;
                        }
                    });
                });

            // Close menu on click elsewhere
            if ctx.input(|i| i.pointer.any_click()) && self.context_menu_slot.is_some() {
                // Check if click was outside menu - simplified: close on next frame
            }
        }

        actions
    }

    /// Shows the hotbar UI (always visible).
    pub fn show_hotbar(
        &mut self,
        ctx: &Context,
        model: &InventoryUIModel,
    ) -> Option<InventoryAction> {
        let mut action = None;

        let screen_rect = ctx.screen_rect();
        let hotbar_width =
            model.hotbar.len() as f32 * (self.config.slot_size + self.config.slot_padding);
        let hotbar_x = (screen_rect.width() - hotbar_width) / 2.0;
        let hotbar_y = screen_rect.height() - self.config.hotbar_y_offset;

        egui::Area::new(Id::new("hotbar"))
            .fixed_pos(Pos2::new(hotbar_x, hotbar_y))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (idx, slot) in model.hotbar.iter().enumerate() {
                        let is_selected = idx == model.selected_hotbar;
                        let response = self.render_slot_internal(ui, slot, is_selected, false);

                        if response.clicked() {
                            action = Some(InventoryAction::SelectHotbar(idx));
                        }

                        // Show slot number
                        let slot_num = if idx == 9 { 0 } else { idx + 1 };
                        ui.painter().text(
                            response.rect.left_top() + Vec2::new(4.0, 2.0),
                            Align2::LEFT_TOP,
                            slot_num.to_string(),
                            FontId::proportional(10.0),
                            Color32::WHITE,
                        );
                    }
                });
            });

        action
    }

    /// Renders the inventory grid.
    fn render_inventory_grid(
        &mut self,
        ui: &mut Ui,
        model: &mut InventoryUIModel,
        actions: &mut Vec<InventoryAction>,
    ) {
        let cols = self.config.columns;

        egui::Grid::new("inventory_grid")
            .spacing(Vec2::splat(self.config.slot_padding))
            .show(ui, |ui| {
                for (idx, slot) in model.slots.iter().enumerate() {
                    let response = self.render_slot(ui, slot);

                    // Handle interactions
                    if response.clicked() {
                        actions.push(InventoryAction::Select(idx));
                    }

                    if response.secondary_clicked() {
                        self.context_menu_slot = Some(idx);
                    }

                    // Drag and drop
                    if response.drag_started() {
                        self.drag_source = Some(idx);
                        model.dragging = Some(idx);
                    }

                    if response.drag_stopped() {
                        if let Some(_from) = self.drag_source.take() {
                            model.dragging = None;
                            // Find drop target (simplified - would need proper hit testing)
                            // For now, just clear drag state
                        }
                    }

                    // Tooltip on hover
                    if response.hovered() && !slot.is_empty() {
                        let tooltip = TooltipData::from_slot(slot);
                        response.on_hover_ui(|ui| {
                            self.render_tooltip(ui, &tooltip);
                        });
                    }

                    // End row
                    if (idx + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    /// Renders a single inventory slot.
    pub fn render_slot(&self, ui: &mut Ui, slot: &SlotUIData) -> Response {
        self.render_slot_internal(ui, slot, slot.is_selected, slot.is_highlighted)
    }

    fn render_slot_internal(
        &self,
        ui: &mut Ui,
        slot: &SlotUIData,
        is_selected: bool,
        is_highlighted: bool,
    ) -> Response {
        let size = Vec2::splat(self.config.slot_size);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            let bg_color = if is_selected {
                Color32::from_rgba_unmultiplied(
                    self.config.selected_color[0],
                    self.config.selected_color[1],
                    self.config.selected_color[2],
                    self.config.selected_color[3],
                )
            } else if is_highlighted {
                Color32::from_rgba_unmultiplied(
                    self.config.highlight_color[0],
                    self.config.highlight_color[1],
                    self.config.highlight_color[2],
                    self.config.highlight_color[3],
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
            let border_color = if response.hovered() {
                Color32::WHITE
            } else {
                Color32::from_gray(80)
            };
            painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0, border_color));

            // Item icon (fallback: colored square)
            if !slot.is_empty() {
                let icon_rect = rect.shrink(8.0);
                let icon_color = Color32::from_rgba_unmultiplied(
                    slot.icon_color[0],
                    slot.icon_color[1],
                    slot.icon_color[2],
                    slot.icon_color[3],
                );
                painter.rect_filled(icon_rect, Rounding::same(2.0), icon_color);

                // Item count
                if slot.count > 1 {
                    painter.text(
                        rect.right_bottom() - Vec2::new(4.0, 4.0),
                        Align2::RIGHT_BOTTOM,
                        slot.count.to_string(),
                        FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
            }
        }

        response
    }

    /// Renders a tooltip for an item.
    pub fn render_tooltip(&self, ui: &mut Ui, tooltip: &TooltipData) {
        ui.vertical(|ui| {
            // Name with rarity color
            let name_color = Color32::from_rgba_unmultiplied(
                tooltip.rarity_color[0],
                tooltip.rarity_color[1],
                tooltip.rarity_color[2],
                tooltip.rarity_color[3],
            );
            ui.label(RichText::new(&tooltip.name).color(name_color).strong());

            // Description
            if !tooltip.description.is_empty() {
                ui.label(&tooltip.description);
            }

            // Stats
            if !tooltip.stats.is_empty() {
                ui.separator();
                for (key, value) in &tooltip.stats {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(key).weak());
                        ui.label(value);
                    });
                }
            }
        });
    }

    /// Toggles the inventory open/closed.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Opens the inventory.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the inventory.
    pub fn close(&mut self) {
        self.is_open = false;
        self.context_menu_slot = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_ui_data_empty() {
        let slot = SlotUIData::empty(0);
        assert!(slot.is_empty());
        assert_eq!(slot.slot_index, 0);
    }

    #[test]
    fn test_slot_ui_data_with_item() {
        let slot = SlotUIData::with_item(1, ItemTypeId::new(42), "Test Item", 10);
        assert!(!slot.is_empty());
        assert_eq!(slot.slot_index, 1);
        assert_eq!(slot.count, 10);
        assert_eq!(slot.item_name, "Test Item");
    }

    #[test]
    fn test_inventory_ui_model_new() {
        let model = InventoryUIModel::new(40, 10);
        assert_eq!(model.slots.len(), 40);
        assert_eq!(model.hotbar.len(), 10);
        assert_eq!(model.selected_hotbar, 0);
    }

    #[test]
    fn test_inventory_ui_model_default_sized() {
        let model = InventoryUIModel::default_sized();
        assert_eq!(
            model.slots.len(),
            DEFAULT_INVENTORY_ROWS * DEFAULT_INVENTORY_COLS
        );
        assert_eq!(model.hotbar.len(), HOTBAR_SLOTS);
    }

    #[test]
    fn test_inventory_ui_config_defaults() {
        let config = InventoryUIConfig::default();
        assert_eq!(config.slot_size, DEFAULT_SLOT_SIZE);
        assert_eq!(config.columns, DEFAULT_INVENTORY_COLS);
    }

    #[test]
    fn test_inventory_ui_new() {
        let ui = InventoryUI::new();
        assert!(!ui.is_open);
        assert!(ui.context_menu_slot.is_none());
    }

    #[test]
    fn test_inventory_ui_toggle() {
        let mut ui = InventoryUI::new();
        assert!(!ui.is_open);
        ui.toggle();
        assert!(ui.is_open);
        ui.toggle();
        assert!(!ui.is_open);
    }

    #[test]
    fn test_inventory_ui_open_close() {
        let mut ui = InventoryUI::new();
        ui.open();
        assert!(ui.is_open);
        ui.close();
        assert!(!ui.is_open);
    }

    #[test]
    fn test_tooltip_data_from_slot() {
        let slot = SlotUIData::with_item(0, ItemTypeId::new(1), "Sword", 5);
        let tooltip = TooltipData::from_slot(&slot);
        assert_eq!(tooltip.name, "Sword");
        assert!(!tooltip.stats.is_empty());
    }

    #[test]
    fn test_inventory_action_equality() {
        assert_eq!(InventoryAction::Select(5), InventoryAction::Select(5));
        assert_ne!(InventoryAction::Select(5), InventoryAction::Select(6));
        assert_eq!(
            InventoryAction::Move { from: 1, to: 2 },
            InventoryAction::Move { from: 1, to: 2 }
        );
    }
}
