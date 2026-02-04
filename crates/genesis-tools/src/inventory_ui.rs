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

/// Item stack for drag operations.
#[derive(Debug, Clone)]
pub struct ItemStack {
    /// Item type ID
    pub item_type: ItemTypeId,
    /// Item name
    pub name: String,
    /// Item count
    pub count: u32,
    /// Icon color
    pub icon_color: [u8; 4],
}

impl ItemStack {
    /// Creates a new item stack from a slot.
    #[must_use]
    pub fn from_slot(slot: &SlotUIData) -> Option<Self> {
        slot.item_type.map(|item_type| Self {
            item_type,
            name: slot.item_name.clone(),
            count: slot.count,
            icon_color: slot.icon_color,
        })
    }
}

/// Sort mode for inventory items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InventorySortMode {
    /// No sorting
    #[default]
    None,
    /// Sort by item name
    ByName,
    /// Sort by item count (descending)
    ByCount,
    /// Sort by item type ID
    ByType,
}

/// Inventory panel with drag-drop, search, and filtering.
#[derive(Debug)]
pub struct InventoryPanel {
    /// Currently dragging item (slot index, item stack)
    pub dragging: Option<(usize, ItemStack)>,
    /// Currently hovered slot index
    pub hovered_slot: Option<usize>,
    /// Search/filter text
    pub filter_text: String,
    /// Current sort mode
    pub sort_mode: InventorySortMode,
    /// Configuration
    pub config: InventoryUIConfig,
    /// Whether the panel is open
    pub is_open: bool,
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
            dragging: None,
            hovered_slot: None,
            filter_text: String::new(),
            sort_mode: InventorySortMode::None,
            config: InventoryUIConfig::default(),
            is_open: false,
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: InventoryUIConfig) -> Self {
        Self {
            dragging: None,
            hovered_slot: None,
            filter_text: String::new(),
            sort_mode: InventorySortMode::None,
            config,
            is_open: false,
        }
    }

    /// Renders the inventory panel and returns any actions.
    pub fn render(&mut self, ctx: &Context, model: &mut InventoryUIModel) -> Vec<InventoryAction> {
        let mut actions = Vec::new();

        if !self.is_open {
            return actions;
        }

        Window::new("Inventory")
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                // Search bar
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.filter_text);
                    if ui.button("✕").clicked() {
                        self.filter_text.clear();
                    }
                });

                // Sort controls
                ui.horizontal(|ui| {
                    ui.label("Sort:");
                    if ui
                        .selectable_label(self.sort_mode == InventorySortMode::None, "None")
                        .clicked()
                    {
                        self.sort_mode = InventorySortMode::None;
                    }
                    if ui
                        .selectable_label(self.sort_mode == InventorySortMode::ByName, "Name")
                        .clicked()
                    {
                        self.sort_mode = InventorySortMode::ByName;
                    }
                    if ui
                        .selectable_label(self.sort_mode == InventorySortMode::ByCount, "Count")
                        .clicked()
                    {
                        self.sort_mode = InventorySortMode::ByCount;
                    }
                    if ui
                        .selectable_label(self.sort_mode == InventorySortMode::ByType, "Type")
                        .clicked()
                    {
                        self.sort_mode = InventorySortMode::ByType;
                    }
                });

                ui.separator();

                // Inventory grid
                self.render_grid(ctx, ui, model, &mut actions);
            });

        // Render drag preview
        if let Some((_, ref stack)) = self.dragging {
            self.render_drag_preview(ctx, stack);
        }

        actions
    }

    /// Renders the inventory grid with filtering.
    fn render_grid(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        model: &mut InventoryUIModel,
        actions: &mut Vec<InventoryAction>,
    ) {
        let cols = self.config.columns;
        let filter_lower = self.filter_text.to_lowercase();

        egui::Grid::new("inventory_panel_grid")
            .spacing(Vec2::splat(self.config.slot_padding))
            .show(ui, |ui| {
                for (idx, slot) in model.slots.iter().enumerate() {
                    // Filter slots
                    let matches_filter = filter_lower.is_empty()
                        || slot.item_name.to_lowercase().contains(&filter_lower);

                    let is_valid_drop_target = self.is_valid_drop_target(idx, slot);
                    let response = self.render_slot(ui, slot, is_valid_drop_target, matches_filter);

                    // Track hover
                    if response.hovered() {
                        self.hovered_slot = Some(idx);
                    }

                    // Handle interactions
                    if response.clicked() && matches_filter {
                        actions.push(InventoryAction::Select(idx));
                    }

                    // Drag start
                    if response.drag_started() && !slot.is_empty() && matches_filter {
                        if let Some(stack) = ItemStack::from_slot(slot) {
                            self.dragging = Some((idx, stack));
                            model.dragging = Some(idx);
                        }
                    }

                    // Drop handling
                    if response.hovered() && ctx.input(|i| i.pointer.any_released()) {
                        if let Some((from_idx, _)) = self.dragging.take() {
                            if from_idx != idx {
                                actions.push(InventoryAction::Move {
                                    from: from_idx,
                                    to: idx,
                                });
                            }
                            model.dragging = None;
                        }
                    }

                    // Tooltip
                    if response.hovered() && !slot.is_empty() && matches_filter {
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

    /// Checks if a slot is a valid drop target.
    fn is_valid_drop_target(&self, slot_idx: usize, slot: &SlotUIData) -> bool {
        if let Some((from_idx, ref stack)) = self.dragging {
            if from_idx == slot_idx {
                return false;
            }
            // Valid if slot is empty or has same item type
            slot.is_empty() || slot.item_type == Some(stack.item_type)
        } else {
            false
        }
    }

    /// Renders a single slot with drag-drop visual feedback.
    fn render_slot(
        &self,
        ui: &mut Ui,
        slot: &SlotUIData,
        is_drop_target: bool,
        matches_filter: bool,
    ) -> Response {
        let size = Vec2::splat(self.config.slot_size);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Determine background color
            let bg_color = if !matches_filter {
                // Dimmed for filtered out slots
                Color32::from_rgba_unmultiplied(30, 30, 30, 150)
            } else if is_drop_target && response.hovered() {
                // Green highlight for valid drop target
                Color32::from_rgba_unmultiplied(60, 150, 60, 255)
            } else if slot.is_selected {
                Color32::from_rgba_unmultiplied(
                    self.config.selected_color[0],
                    self.config.selected_color[1],
                    self.config.selected_color[2],
                    self.config.selected_color[3],
                )
            } else if slot.is_highlighted {
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

            // Border - thicker for drop targets
            let (border_width, border_color) = if is_drop_target && response.hovered() {
                (2.0, Color32::GREEN)
            } else if response.hovered() && matches_filter {
                (1.5, Color32::WHITE)
            } else {
                (1.0, Color32::from_gray(80))
            };
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(border_width, border_color),
            );

            // Item icon
            if !slot.is_empty() {
                let icon_rect = rect.shrink(8.0);
                let alpha = if matches_filter { 255 } else { 100 };
                let icon_color = Color32::from_rgba_unmultiplied(
                    slot.icon_color[0],
                    slot.icon_color[1],
                    slot.icon_color[2],
                    alpha,
                );
                painter.rect_filled(icon_rect, Rounding::same(2.0), icon_color);

                // Item count
                if slot.count > 1 {
                    painter.text(
                        rect.right_bottom() - Vec2::new(4.0, 4.0),
                        Align2::RIGHT_BOTTOM,
                        slot.count.to_string(),
                        FontId::proportional(12.0),
                        if matches_filter {
                            Color32::WHITE
                        } else {
                            Color32::GRAY
                        },
                    );
                }
            }
        }

        response
    }

    /// Renders the drag preview following the cursor.
    fn render_drag_preview(&self, ctx: &Context, stack: &ItemStack) {
        if let Some(pos) = ctx.pointer_hover_pos() {
            egui::Area::new(Id::new("drag_preview"))
                .fixed_pos(pos + Vec2::new(8.0, 8.0))
                .order(egui::Order::Tooltip)
                .show(ctx, |ui| {
                    let size = Vec2::splat(self.config.slot_size * 0.75);
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

                    let painter = ui.painter();
                    let icon_color = Color32::from_rgba_unmultiplied(
                        stack.icon_color[0],
                        stack.icon_color[1],
                        stack.icon_color[2],
                        200,
                    );
                    painter.rect_filled(rect, Rounding::same(4.0), icon_color);

                    if stack.count > 1 {
                        painter.text(
                            rect.right_bottom() - Vec2::new(2.0, 2.0),
                            Align2::RIGHT_BOTTOM,
                            stack.count.to_string(),
                            FontId::proportional(10.0),
                            Color32::WHITE,
                        );
                    }
                });
        }
    }

    /// Renders item tooltip.
    #[allow(clippy::unused_self)]
    fn render_tooltip(&self, ui: &mut Ui, tooltip: &TooltipData) {
        ui.vertical(|ui| {
            let name_color = Color32::from_rgba_unmultiplied(
                tooltip.rarity_color[0],
                tooltip.rarity_color[1],
                tooltip.rarity_color[2],
                tooltip.rarity_color[3],
            );
            ui.label(RichText::new(&tooltip.name).color(name_color).strong());

            if !tooltip.description.is_empty() {
                ui.label(&tooltip.description);
            }

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

    /// Sorts the inventory according to the current sort mode.
    pub fn apply_sort(&self, model: &mut InventoryUIModel) {
        match self.sort_mode {
            InventorySortMode::None => {},
            InventorySortMode::ByName => {
                model
                    .slots
                    .sort_by(|a, b| match (a.is_empty(), b.is_empty()) {
                        (true, true) => std::cmp::Ordering::Equal,
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, true) => std::cmp::Ordering::Less,
                        (false, false) => a.item_name.cmp(&b.item_name),
                    });
            },
            InventorySortMode::ByCount => {
                model.slots.sort_by(|a, b| {
                    match (a.is_empty(), b.is_empty()) {
                        (true, true) => std::cmp::Ordering::Equal,
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, true) => std::cmp::Ordering::Less,
                        (false, false) => b.count.cmp(&a.count), // Descending
                    }
                });
            },
            InventorySortMode::ByType => {
                model
                    .slots
                    .sort_by(|a, b| match (&a.item_type, &b.item_type) {
                        (None, None) => std::cmp::Ordering::Equal,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (Some(a_type), Some(b_type)) => a_type.raw().cmp(&b_type.raw()),
                    });
            },
        }
        // Re-index slots after sorting
        for (idx, slot) in model.slots.iter_mut().enumerate() {
            slot.slot_index = idx;
        }
    }

    /// Filters slots matching the current filter text.
    #[must_use]
    pub fn filter_slots<'a>(&self, slots: &'a [SlotUIData]) -> Vec<&'a SlotUIData> {
        if self.filter_text.is_empty() {
            return slots.iter().collect();
        }
        let filter_lower = self.filter_text.to_lowercase();
        slots
            .iter()
            .filter(|s| s.item_name.to_lowercase().contains(&filter_lower))
            .collect()
    }

    /// Toggles the panel open/closed.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if !self.is_open {
            self.dragging = None;
            self.hovered_slot = None;
        }
    }

    /// Opens the panel.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the panel.
    pub fn close(&mut self) {
        self.is_open = false;
        self.dragging = None;
        self.hovered_slot = None;
    }

    /// Clears the filter.
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
    }

    /// Sets the filter text.
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter_text = filter.into();
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

    #[test]
    fn test_inventory_panel_new() {
        let panel = InventoryPanel::new();
        assert!(!panel.is_open);
        assert!(panel.dragging.is_none());
        assert!(panel.hovered_slot.is_none());
        assert!(panel.filter_text.is_empty());
        assert_eq!(panel.sort_mode, InventorySortMode::None);
    }

    #[test]
    fn test_inventory_panel_toggle() {
        let mut panel = InventoryPanel::new();
        assert!(!panel.is_open);
        panel.toggle();
        assert!(panel.is_open);
        panel.toggle();
        assert!(!panel.is_open);
    }

    #[test]
    fn test_inventory_panel_filter() {
        let mut panel = InventoryPanel::new();
        panel.set_filter("sword");
        assert_eq!(panel.filter_text, "sword");
        panel.clear_filter();
        assert!(panel.filter_text.is_empty());
    }

    #[test]
    fn test_inventory_panel_filter_slots() {
        let panel = InventoryPanel {
            filter_text: "sw".to_string(),
            ..Default::default()
        };
        let slots = vec![
            SlotUIData::with_item(0, ItemTypeId::new(1), "Sword", 1),
            SlotUIData::with_item(1, ItemTypeId::new(2), "Shield", 1),
            SlotUIData::with_item(2, ItemTypeId::new(3), "Swamp Boots", 1),
        ];
        let filtered = panel.filter_slots(&slots);
        assert_eq!(filtered.len(), 2); // Sword and Swamp Boots
    }

    #[test]
    fn test_inventory_panel_sort_by_name() {
        let mut panel = InventoryPanel::new();
        panel.sort_mode = InventorySortMode::ByName;
        let mut model = InventoryUIModel::new(3, 0);
        model.slots[0] = SlotUIData::with_item(0, ItemTypeId::new(1), "Zelda", 1);
        model.slots[1] = SlotUIData::with_item(1, ItemTypeId::new(2), "Apple", 1);
        model.slots[2] = SlotUIData::with_item(2, ItemTypeId::new(3), "Banana", 1);

        panel.apply_sort(&mut model);

        assert_eq!(model.slots[0].item_name, "Apple");
        assert_eq!(model.slots[1].item_name, "Banana");
        assert_eq!(model.slots[2].item_name, "Zelda");
    }

    #[test]
    fn test_inventory_panel_sort_by_count() {
        let mut panel = InventoryPanel::new();
        panel.sort_mode = InventorySortMode::ByCount;
        let mut model = InventoryUIModel::new(3, 0);
        model.slots[0] = SlotUIData::with_item(0, ItemTypeId::new(1), "A", 5);
        model.slots[1] = SlotUIData::with_item(1, ItemTypeId::new(2), "B", 20);
        model.slots[2] = SlotUIData::with_item(2, ItemTypeId::new(3), "C", 10);

        panel.apply_sort(&mut model);

        assert_eq!(model.slots[0].count, 20); // Descending order
        assert_eq!(model.slots[1].count, 10);
        assert_eq!(model.slots[2].count, 5);
    }

    #[test]
    fn test_item_stack_from_slot() {
        let slot = SlotUIData::with_item(0, ItemTypeId::new(42), "Test", 5);
        let stack = ItemStack::from_slot(&slot);
        assert!(stack.is_some());
        let stack = stack.unwrap();
        assert_eq!(stack.item_type, ItemTypeId::new(42));
        assert_eq!(stack.name, "Test");
        assert_eq!(stack.count, 5);
    }

    #[test]
    fn test_item_stack_from_empty_slot() {
        let slot = SlotUIData::empty(0);
        let stack = ItemStack::from_slot(&slot);
        assert!(stack.is_none());
    }

    #[test]
    fn test_inventory_sort_mode_default() {
        assert_eq!(InventorySortMode::default(), InventorySortMode::None);
    }
}
