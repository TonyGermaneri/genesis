//! Crafting grid UI for drag-drop crafting interface.
//!
//! Provides a 3x3 ingredient grid with:
//! - Drag-drop support for ingredients
//! - Output slot with result preview
//! - Craft button with cooldown visual
//! - Clear grid button

use egui::{Color32, Pos2, Rect, Response, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Grid dimensions for the crafting grid.
pub const CRAFTING_GRID_SIZE: usize = 3;

/// Total number of slots in the crafting grid.
pub const CRAFTING_GRID_SLOTS: usize = CRAFTING_GRID_SIZE * CRAFTING_GRID_SIZE;

/// Unique identifier for an item in the crafting system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CraftingItemId(pub String);

impl CraftingItemId {
    /// Create a new crafting item ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for CraftingItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Rarity level for crafted items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ItemRarity {
    /// Common items.
    #[default]
    Common,
    /// Uncommon items.
    Uncommon,
    /// Rare items.
    Rare,
    /// Epic items.
    Epic,
    /// Legendary items.
    Legendary,
}

impl ItemRarity {
    /// Get the display color for this rarity.
    pub fn color(&self) -> Color32 {
        match self {
            ItemRarity::Common => Color32::from_rgb(200, 200, 200),
            ItemRarity::Uncommon => Color32::from_rgb(100, 200, 100),
            ItemRarity::Rare => Color32::from_rgb(100, 150, 255),
            ItemRarity::Epic => Color32::from_rgb(180, 100, 255),
            ItemRarity::Legendary => Color32::from_rgb(255, 180, 50),
        }
    }

    /// Get display name for this rarity.
    pub fn display_name(&self) -> &'static str {
        match self {
            ItemRarity::Common => "Common",
            ItemRarity::Uncommon => "Uncommon",
            ItemRarity::Rare => "Rare",
            ItemRarity::Epic => "Epic",
            ItemRarity::Legendary => "Legendary",
        }
    }

    /// Get icon for this rarity.
    pub fn icon(&self) -> &'static str {
        match self {
            ItemRarity::Common => "○",
            ItemRarity::Uncommon => "●",
            ItemRarity::Rare => "◆",
            ItemRarity::Epic => "★",
            ItemRarity::Legendary => "✦",
        }
    }
}

/// An item that can be placed in a crafting slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingItem {
    /// Unique item ID.
    pub id: CraftingItemId,
    /// Display name.
    pub name: String,
    /// Item icon identifier.
    pub icon: String,
    /// Stack count.
    pub count: u32,
    /// Maximum stack size.
    pub max_stack: u32,
    /// Item rarity.
    pub rarity: ItemRarity,
}

impl CraftingItem {
    /// Create a new crafting item.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let id_string = id.into();
        Self {
            id: CraftingItemId::new(&id_string),
            name: name.into(),
            icon: id_string,
            count: 1,
            max_stack: 64,
            rarity: ItemRarity::Common,
        }
    }

    /// Set the item count.
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = count.min(self.max_stack);
        self
    }

    /// Set the max stack size.
    pub fn with_max_stack(mut self, max_stack: u32) -> Self {
        self.max_stack = max_stack;
        self.count = self.count.min(max_stack);
        self
    }

    /// Set the item rarity.
    pub fn with_rarity(mut self, rarity: ItemRarity) -> Self {
        self.rarity = rarity;
        self
    }

    /// Set the icon identifier.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Check if we can add more to the stack.
    pub fn can_stack(&self, amount: u32) -> bool {
        self.count + amount <= self.max_stack
    }

    /// Try to add to the stack, returns overflow.
    pub fn add_to_stack(&mut self, amount: u32) -> u32 {
        let space = self.max_stack - self.count;
        let to_add = amount.min(space);
        self.count += to_add;
        amount - to_add
    }

    /// Remove from stack, returns amount actually removed.
    pub fn remove_from_stack(&mut self, amount: u32) -> u32 {
        let to_remove = amount.min(self.count);
        self.count -= to_remove;
        to_remove
    }
}

/// A slot in the crafting grid.
#[derive(Debug, Clone, Default)]
pub struct CraftingSlot {
    /// Item in this slot, if any.
    pub item: Option<CraftingItem>,
    /// Whether slot is highlighted (hover, drag target).
    pub highlighted: bool,
    /// Whether slot is locked.
    pub locked: bool,
}

impl CraftingSlot {
    /// Create a new empty slot.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a slot with an item.
    pub fn with_item(item: CraftingItem) -> Self {
        Self {
            item: Some(item),
            highlighted: false,
            locked: false,
        }
    }

    /// Check if slot is empty.
    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }

    /// Check if slot has an item.
    pub fn has_item(&self) -> bool {
        self.item.is_some()
    }

    /// Take the item from the slot.
    pub fn take_item(&mut self) -> Option<CraftingItem> {
        self.item.take()
    }

    /// Set an item in the slot.
    pub fn set_item(&mut self, item: CraftingItem) {
        self.item = Some(item);
    }

    /// Clear the slot.
    pub fn clear(&mut self) {
        self.item = None;
        self.highlighted = false;
    }
}

/// State of the craft button cooldown.
#[derive(Debug, Clone, Default)]
pub struct CraftCooldown {
    /// Whether crafting is on cooldown.
    pub active: bool,
    /// Current cooldown time remaining in seconds.
    pub remaining: f32,
    /// Total cooldown duration in seconds.
    pub duration: f32,
}

impl CraftCooldown {
    /// Create a new cooldown.
    pub fn new(duration: f32) -> Self {
        Self {
            active: false,
            remaining: 0.0,
            duration,
        }
    }

    /// Start the cooldown.
    pub fn start(&mut self) {
        self.active = true;
        self.remaining = self.duration;
    }

    /// Update the cooldown.
    pub fn update(&mut self, delta_time: f32) {
        if self.active {
            self.remaining -= delta_time;
            if self.remaining <= 0.0 {
                self.remaining = 0.0;
                self.active = false;
            }
        }
    }

    /// Get progress (0.0 = just started, 1.0 = complete).
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 && self.active {
            1.0 - (self.remaining / self.duration)
        } else {
            1.0
        }
    }

    /// Check if ready to craft.
    pub fn is_ready(&self) -> bool {
        !self.active
    }
}

/// Drag state for the crafting grid.
#[derive(Debug, Clone, Default)]
pub struct DragState {
    /// Currently dragged item.
    pub dragging: Option<CraftingItem>,
    /// Source slot index (if from grid).
    pub source_slot: Option<usize>,
    /// Current drag position.
    pub drag_pos: Pos2,
    /// Whether currently dragging.
    pub is_dragging: bool,
}

impl DragState {
    /// Start dragging an item.
    pub fn start_drag(&mut self, item: CraftingItem, source_slot: Option<usize>, pos: Pos2) {
        self.dragging = Some(item);
        self.source_slot = source_slot;
        self.drag_pos = pos;
        self.is_dragging = true;
    }

    /// Update drag position.
    pub fn update_pos(&mut self, pos: Pos2) {
        self.drag_pos = pos;
    }

    /// End dragging, returning the item.
    pub fn end_drag(&mut self) -> Option<CraftingItem> {
        self.is_dragging = false;
        self.source_slot = None;
        self.dragging.take()
    }

    /// Cancel dragging.
    pub fn cancel(&mut self) {
        self.is_dragging = false;
        self.dragging = None;
        self.source_slot = None;
    }
}

/// Actions returned by the crafting grid.
#[derive(Debug, Clone, PartialEq)]
pub enum CraftingGridAction {
    /// Item placed in a slot.
    ItemPlaced {
        /// Slot index where item was placed.
        slot: usize,
        /// ID of the placed item.
        item_id: CraftingItemId,
    },
    /// Item removed from a slot.
    ItemRemoved {
        /// Slot index where item was removed from.
        slot: usize,
        /// ID of the removed item.
        item_id: CraftingItemId,
    },
    /// Craft button clicked.
    CraftClicked,
    /// Clear grid button clicked.
    ClearGrid,
    /// Output item collected.
    OutputCollected {
        /// ID of the collected output item.
        item_id: CraftingItemId,
    },
    /// Slot clicked.
    SlotClicked {
        /// Index of the clicked slot.
        slot: usize,
    },
    /// Drag started.
    DragStarted {
        /// Slot index where drag started.
        slot: usize,
        /// ID of the item being dragged.
        item_id: CraftingItemId,
    },
    /// Drag ended.
    DragEnded {
        /// Target slot where item was dropped (None if dropped outside).
        target_slot: Option<usize>,
    },
}

/// Configuration for the crafting grid UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingGridConfig {
    /// Slot size in pixels.
    pub slot_size: f32,
    /// Spacing between slots.
    pub slot_spacing: f32,
    /// Background color for empty slots.
    pub slot_bg_color: [u8; 4],
    /// Highlight color for hovered slots.
    pub highlight_color: [u8; 4],
    /// Locked slot color.
    pub locked_color: [u8; 4],
    /// Show stack counts.
    pub show_stack_counts: bool,
    /// Show cooldown progress.
    pub show_cooldown: bool,
    /// Cooldown duration in seconds.
    pub craft_cooldown: f32,
}

impl Default for CraftingGridConfig {
    fn default() -> Self {
        Self {
            slot_size: 48.0,
            slot_spacing: 4.0,
            slot_bg_color: [40, 40, 40, 255],
            highlight_color: [80, 120, 180, 255],
            locked_color: [60, 40, 40, 255],
            show_stack_counts: true,
            show_cooldown: true,
            craft_cooldown: 0.5,
        }
    }
}

impl CraftingGridConfig {
    /// Get slot background as Color32.
    pub fn slot_bg(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.slot_bg_color[0],
            self.slot_bg_color[1],
            self.slot_bg_color[2],
            self.slot_bg_color[3],
        )
    }

    /// Get highlight color as Color32.
    pub fn highlight(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.highlight_color[0],
            self.highlight_color[1],
            self.highlight_color[2],
            self.highlight_color[3],
        )
    }

    /// Get locked color as Color32.
    pub fn locked(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.locked_color[0],
            self.locked_color[1],
            self.locked_color[2],
            self.locked_color[3],
        )
    }
}

/// The crafting grid widget.
#[derive(Debug)]
pub struct CraftingGrid {
    /// Grid slots (3x3).
    pub slots: [CraftingSlot; CRAFTING_GRID_SLOTS],
    /// Output slot.
    pub output: CraftingSlot,
    /// Output preview item (shown when recipe matches).
    pub output_preview: Option<CraftingItem>,
    /// Configuration.
    pub config: CraftingGridConfig,
    /// Drag state.
    pub drag_state: DragState,
    /// Craft button cooldown.
    pub cooldown: CraftCooldown,
    /// Whether the grid is open.
    pub open: bool,
    /// Pending actions.
    pending_actions: Vec<CraftingGridAction>,
}

impl Default for CraftingGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl CraftingGrid {
    /// Create a new crafting grid.
    pub fn new() -> Self {
        Self {
            slots: Default::default(),
            output: CraftingSlot::new(),
            output_preview: None,
            config: CraftingGridConfig::default(),
            drag_state: DragState::default(),
            cooldown: CraftCooldown::new(0.5),
            open: false,
            pending_actions: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: CraftingGridConfig) -> Self {
        let cooldown_duration = config.craft_cooldown;
        Self {
            config,
            cooldown: CraftCooldown::new(cooldown_duration),
            ..Self::new()
        }
    }

    /// Open the grid.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the grid.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggle grid visibility.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Get a slot by index.
    pub fn get_slot(&self, index: usize) -> Option<&CraftingSlot> {
        self.slots.get(index)
    }

    /// Get a mutable slot by index.
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut CraftingSlot> {
        self.slots.get_mut(index)
    }

    /// Get slot by grid position.
    pub fn get_slot_at(&self, row: usize, col: usize) -> Option<&CraftingSlot> {
        if row < CRAFTING_GRID_SIZE && col < CRAFTING_GRID_SIZE {
            self.slots.get(row * CRAFTING_GRID_SIZE + col)
        } else {
            None
        }
    }

    /// Set an item in a slot.
    pub fn set_item(&mut self, index: usize, item: CraftingItem) -> bool {
        if let Some(slot) = self.slots.get_mut(index) {
            if !slot.locked {
                let item_id = item.id.clone();
                slot.set_item(item);
                self.pending_actions.push(CraftingGridAction::ItemPlaced {
                    slot: index,
                    item_id,
                });
                return true;
            }
        }
        false
    }

    /// Remove an item from a slot.
    pub fn remove_item(&mut self, index: usize) -> Option<CraftingItem> {
        if let Some(slot) = self.slots.get_mut(index) {
            if let Some(item) = slot.take_item() {
                self.pending_actions.push(CraftingGridAction::ItemRemoved {
                    slot: index,
                    item_id: item.id.clone(),
                });
                return Some(item);
            }
        }
        None
    }

    /// Clear all slots in the grid.
    pub fn clear_grid(&mut self) {
        for slot in &mut self.slots {
            slot.clear();
        }
        self.pending_actions.push(CraftingGridAction::ClearGrid);
    }

    /// Set the output preview item.
    pub fn set_output_preview(&mut self, item: Option<CraftingItem>) {
        self.output_preview = item;
    }

    /// Set the actual output item (after crafting).
    pub fn set_output(&mut self, item: CraftingItem) {
        self.output.set_item(item);
    }

    /// Collect the output item.
    pub fn collect_output(&mut self) -> Option<CraftingItem> {
        if let Some(item) = self.output.take_item() {
            self.pending_actions
                .push(CraftingGridAction::OutputCollected {
                    item_id: item.id.clone(),
                });
            Some(item)
        } else {
            None
        }
    }

    /// Get current grid pattern as item IDs.
    pub fn get_pattern(
        &self,
    ) -> [[Option<CraftingItemId>; CRAFTING_GRID_SIZE]; CRAFTING_GRID_SIZE] {
        let mut pattern = [[None, None, None], [None, None, None], [None, None, None]];
        for (row, row_pattern) in pattern.iter_mut().enumerate().take(CRAFTING_GRID_SIZE) {
            for (col, cell) in row_pattern.iter_mut().enumerate().take(CRAFTING_GRID_SIZE) {
                let index = row * CRAFTING_GRID_SIZE + col;
                *cell = self.slots[index].item.as_ref().map(|i| i.id.clone());
            }
        }
        pattern
    }

    /// Update the cooldown timer.
    pub fn update(&mut self, delta_time: f32) {
        self.cooldown.update(delta_time);
    }

    /// Check if crafting is ready (no cooldown).
    pub fn can_craft(&self) -> bool {
        self.cooldown.is_ready() && self.output_preview.is_some()
    }

    /// Trigger crafting action.
    pub fn craft(&mut self) {
        if self.can_craft() {
            self.cooldown.start();
            self.pending_actions.push(CraftingGridAction::CraftClicked);
        }
    }

    /// Render the crafting grid and return any actions.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<CraftingGridAction> {
        self.pending_actions.clear();

        if !self.open {
            return Vec::new();
        }

        let slot_size = self.config.slot_size;
        let spacing = self.config.slot_spacing;

        ui.vertical(|ui| {
            ui.heading("Crafting");
            ui.separator();

            ui.horizontal(|ui| {
                // 3x3 Grid
                ui.vertical(|ui| {
                    for row in 0..CRAFTING_GRID_SIZE {
                        ui.horizontal(|ui| {
                            for col in 0..CRAFTING_GRID_SIZE {
                                let index = row * CRAFTING_GRID_SIZE + col;
                                self.show_slot(ui, index, slot_size);
                                if col < CRAFTING_GRID_SIZE - 1 {
                                    ui.add_space(spacing);
                                }
                            }
                        });
                        if row < CRAFTING_GRID_SIZE - 1 {
                            ui.add_space(spacing);
                        }
                    }
                });

                ui.add_space(spacing * 4.0);

                // Arrow
                ui.vertical(|ui| {
                    ui.add_space(slot_size);
                    ui.label("→");
                });

                ui.add_space(spacing * 2.0);

                // Output slot
                ui.vertical(|ui| {
                    ui.add_space(slot_size * 0.5);
                    self.show_output_slot(ui, slot_size * 1.2);
                });
            });

            ui.add_space(spacing * 2.0);

            // Buttons
            ui.horizontal(|ui| {
                // Craft button with cooldown
                let can_craft = self.can_craft();
                let button_text = if self.cooldown.active {
                    format!("Crafting... {:.1}s", self.cooldown.remaining)
                } else if self.output_preview.is_some() {
                    "Craft".to_string()
                } else {
                    "No Recipe".to_string()
                };

                if ui
                    .add_enabled(can_craft, egui::Button::new(button_text))
                    .clicked()
                {
                    self.craft();
                }

                // Cooldown progress bar
                if self.config.show_cooldown && self.cooldown.active {
                    let progress = self.cooldown.progress();
                    let bar_width = 100.0;
                    let bar_height = 8.0;
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(bar_width, bar_height),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            rect.min,
                            Vec2::new(rect.width() * progress, bar_height),
                        ),
                        2.0,
                        Color32::from_rgb(100, 180, 100),
                    );
                }

                ui.add_space(spacing * 2.0);

                // Clear button
                if ui.button("Clear").clicked() {
                    self.clear_grid();
                }
            });
        });

        // Draw dragged item if any
        if self.drag_state.is_dragging {
            if let Some(item) = &self.drag_state.dragging {
                let painter = ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Tooltip,
                    egui::Id::new("drag_overlay"),
                ));
                let rect =
                    Rect::from_center_size(self.drag_state.drag_pos, Vec2::splat(slot_size * 0.8));
                painter.rect_filled(rect, 4.0, item.rarity.color().linear_multiply(0.8));
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &item.name[..item.name.len().min(3)],
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
        }

        std::mem::take(&mut self.pending_actions)
    }

    /// Show a single grid slot.
    fn show_slot(&mut self, ui: &mut Ui, index: usize, size: f32) {
        let slot = &self.slots[index];
        let bg_color = if slot.locked {
            self.config.locked()
        } else if slot.highlighted {
            self.config.highlight()
        } else {
            self.config.slot_bg()
        };

        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click_and_drag());

        // Background
        ui.painter().rect_filled(rect, 4.0, bg_color);
        ui.painter()
            .rect_stroke(rect, 4.0, egui::Stroke::new(1.0, Color32::from_gray(60)));

        // Item
        if let Some(item) = &slot.item {
            // Item background based on rarity
            let inner_rect = rect.shrink(4.0);
            ui.painter()
                .rect_filled(inner_rect, 2.0, item.rarity.color().linear_multiply(0.3));

            // Item icon/name
            ui.painter().text(
                inner_rect.center(),
                egui::Align2::CENTER_CENTER,
                &item.icon[..item.icon.len().min(3)],
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );

            // Stack count
            if self.config.show_stack_counts && item.count > 1 {
                ui.painter().text(
                    inner_rect.right_bottom() - Vec2::new(2.0, 2.0),
                    egui::Align2::RIGHT_BOTTOM,
                    format!("{}", item.count),
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }

        // Handle interactions
        self.handle_slot_interaction(index, &response, rect);
    }

    /// Show the output slot.
    fn show_output_slot(&mut self, ui: &mut Ui, size: f32) {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click_and_drag());

        // Background
        let bg = if self.output.has_item() || self.output_preview.is_some() {
            Color32::from_rgb(60, 80, 60)
        } else {
            self.config.slot_bg()
        };
        ui.painter().rect_filled(rect, 4.0, bg);
        ui.painter().rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(2.0, Color32::from_rgb(100, 140, 100)),
        );

        // Output item or preview
        let display_item = self.output.item.as_ref().or(self.output_preview.as_ref());
        if let Some(item) = display_item {
            let inner_rect = rect.shrink(4.0);
            let alpha = if self.output.has_item() { 1.0 } else { 0.5 };
            ui.painter().rect_filled(
                inner_rect,
                2.0,
                item.rarity.color().linear_multiply(0.3 * alpha),
            );

            ui.painter().text(
                inner_rect.center(),
                egui::Align2::CENTER_CENTER,
                &item.icon[..item.icon.len().min(3)],
                egui::FontId::proportional(16.0),
                Color32::WHITE.linear_multiply(alpha),
            );

            // Preview label
            if self.output_preview.is_some() && !self.output.has_item() {
                ui.painter().text(
                    inner_rect.center_bottom(),
                    egui::Align2::CENTER_BOTTOM,
                    "Preview",
                    egui::FontId::proportional(8.0),
                    Color32::from_gray(150),
                );
            }
        }

        // Handle output click
        if response.clicked() && self.output.has_item() {
            if let Some(item) = self.output.take_item() {
                self.pending_actions
                    .push(CraftingGridAction::OutputCollected {
                        item_id: item.id.clone(),
                    });
            }
        }
    }

    /// Handle slot interaction (click, drag).
    fn handle_slot_interaction(&mut self, index: usize, response: &Response, rect: Rect) {
        let slot = &mut self.slots[index];

        // Highlight on hover
        slot.highlighted = response.hovered();

        // Handle drag start
        if response.drag_started() && slot.has_item() && !slot.locked {
            if let Some(item) = slot.take_item() {
                self.pending_actions.push(CraftingGridAction::DragStarted {
                    slot: index,
                    item_id: item.id.clone(),
                });
                self.drag_state.start_drag(item, Some(index), rect.center());
            }
        }

        // Update drag position
        if self.drag_state.is_dragging {
            if let Some(pos) = response.interact_pointer_pos() {
                self.drag_state.update_pos(pos);
            }
        }

        // Handle drag end (drop)
        if response.drag_stopped()
            && self.drag_state.is_dragging
            && response.hovered()
            && !slot.locked
        {
            // Drop on this slot
            if let Some(dragged_item) = self.drag_state.end_drag() {
                // Swap if slot has item
                let existing = slot.take_item();
                slot.set_item(dragged_item.clone());
                self.pending_actions.push(CraftingGridAction::ItemPlaced {
                    slot: index,
                    item_id: dragged_item.id,
                });

                // Return existing item to source slot
                if let (Some(existing_item), Some(source)) = (existing, self.drag_state.source_slot)
                {
                    if let Some(source_slot) = self.slots.get_mut(source) {
                        source_slot.set_item(existing_item);
                    }
                }

                self.pending_actions.push(CraftingGridAction::DragEnded {
                    target_slot: Some(index),
                });
            }
        }

        // Handle click
        if response.clicked() {
            self.pending_actions
                .push(CraftingGridAction::SlotClicked { slot: index });
        }
    }

    /// Drain pending actions.
    pub fn drain_actions(&mut self) -> Vec<CraftingGridAction> {
        std::mem::take(&mut self.pending_actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crafting_item_id() {
        let id = CraftingItemId::new("iron_ingot");
        assert_eq!(id.0, "iron_ingot");
        assert_eq!(format!("{id}"), "iron_ingot");
    }

    #[test]
    fn test_item_rarity() {
        assert_eq!(ItemRarity::Common.display_name(), "Common");
        assert_eq!(ItemRarity::Legendary.display_name(), "Legendary");
        // Colors should be distinct
        assert_ne!(ItemRarity::Common.color(), ItemRarity::Rare.color());
    }

    #[test]
    fn test_crafting_item_new() {
        let item = CraftingItem::new("iron_sword", "Iron Sword");
        assert_eq!(item.id.0, "iron_sword");
        assert_eq!(item.name, "Iron Sword");
        assert_eq!(item.count, 1);
        assert_eq!(item.max_stack, 64);
        assert_eq!(item.rarity, ItemRarity::Common);
    }

    #[test]
    fn test_crafting_item_builders() {
        let item = CraftingItem::new("gold", "Gold Ingot")
            .with_count(10)
            .with_max_stack(32)
            .with_rarity(ItemRarity::Rare)
            .with_icon("gold_icon");

        assert_eq!(item.count, 10);
        assert_eq!(item.max_stack, 32);
        assert_eq!(item.rarity, ItemRarity::Rare);
        assert_eq!(item.icon, "gold_icon");
    }

    #[test]
    fn test_crafting_item_stacking() {
        let mut item = CraftingItem::new("stone", "Stone").with_count(50);

        assert!(item.can_stack(10));
        assert!(!item.can_stack(20));

        let overflow = item.add_to_stack(20);
        assert_eq!(item.count, 64);
        assert_eq!(overflow, 6);

        let removed = item.remove_from_stack(30);
        assert_eq!(removed, 30);
        assert_eq!(item.count, 34);
    }

    #[test]
    fn test_crafting_slot() {
        let mut slot = CraftingSlot::new();
        assert!(slot.is_empty());
        assert!(!slot.has_item());

        let item = CraftingItem::new("test", "Test");
        slot.set_item(item);
        assert!(!slot.is_empty());
        assert!(slot.has_item());

        let taken = slot.take_item();
        assert!(taken.is_some());
        assert!(slot.is_empty());
    }

    #[test]
    fn test_crafting_slot_with_item() {
        let item = CraftingItem::new("test", "Test");
        let slot = CraftingSlot::with_item(item);
        assert!(slot.has_item());
    }

    #[test]
    fn test_craft_cooldown() {
        let mut cooldown = CraftCooldown::new(1.0);
        assert!(cooldown.is_ready());
        assert!((cooldown.progress() - 1.0).abs() < 0.01);

        cooldown.start();
        assert!(!cooldown.is_ready());
        assert!((cooldown.progress() - 0.0).abs() < 0.01);

        cooldown.update(0.5);
        assert!(!cooldown.is_ready());
        assert!((cooldown.progress() - 0.5).abs() < 0.01);

        cooldown.update(0.6);
        assert!(cooldown.is_ready());
    }

    #[test]
    fn test_drag_state() {
        let mut drag = DragState::default();
        assert!(!drag.is_dragging);

        let item = CraftingItem::new("test", "Test");
        drag.start_drag(item, Some(0), Pos2::new(100.0, 100.0));
        assert!(drag.is_dragging);
        assert_eq!(drag.source_slot, Some(0));

        drag.update_pos(Pos2::new(150.0, 150.0));
        assert_eq!(drag.drag_pos, Pos2::new(150.0, 150.0));

        let ended = drag.end_drag();
        assert!(ended.is_some());
        assert!(!drag.is_dragging);
    }

    #[test]
    fn test_drag_state_cancel() {
        let mut drag = DragState::default();
        let item = CraftingItem::new("test", "Test");
        drag.start_drag(item, Some(0), Pos2::ZERO);
        drag.cancel();
        assert!(!drag.is_dragging);
        assert!(drag.dragging.is_none());
    }

    #[test]
    fn test_crafting_grid_new() {
        let grid = CraftingGrid::new();
        assert!(!grid.open);
        assert_eq!(grid.slots.len(), CRAFTING_GRID_SLOTS);
        for slot in &grid.slots {
            assert!(slot.is_empty());
        }
    }

    #[test]
    fn test_crafting_grid_toggle() {
        let mut grid = CraftingGrid::new();
        assert!(!grid.open);
        grid.toggle();
        assert!(grid.open);
        grid.toggle();
        assert!(!grid.open);
    }

    #[test]
    fn test_crafting_grid_set_item() {
        let mut grid = CraftingGrid::new();
        let item = CraftingItem::new("test", "Test");
        assert!(grid.set_item(0, item));
        assert!(grid.get_slot(0).map(|s| s.has_item()).unwrap_or(false));
    }

    #[test]
    fn test_crafting_grid_remove_item() {
        let mut grid = CraftingGrid::new();
        grid.set_item(0, CraftingItem::new("test", "Test"));
        let removed = grid.remove_item(0);
        assert!(removed.is_some());
        assert!(grid.get_slot(0).map(|s| s.is_empty()).unwrap_or(false));
    }

    #[test]
    fn test_crafting_grid_clear() {
        let mut grid = CraftingGrid::new();
        grid.set_item(0, CraftingItem::new("a", "A"));
        grid.set_item(4, CraftingItem::new("b", "B"));
        grid.clear_grid();
        for slot in &grid.slots {
            assert!(slot.is_empty());
        }
    }

    #[test]
    fn test_crafting_grid_get_pattern() {
        let mut grid = CraftingGrid::new();
        grid.set_item(0, CraftingItem::new("a", "A"));
        grid.set_item(4, CraftingItem::new("b", "B"));
        grid.set_item(8, CraftingItem::new("c", "C"));

        let pattern = grid.get_pattern();
        assert_eq!(pattern[0][0], Some(CraftingItemId::new("a")));
        assert_eq!(pattern[1][1], Some(CraftingItemId::new("b")));
        assert_eq!(pattern[2][2], Some(CraftingItemId::new("c")));
        assert_eq!(pattern[0][1], None);
    }

    #[test]
    fn test_crafting_grid_can_craft() {
        let mut grid = CraftingGrid::new();
        assert!(!grid.can_craft()); // No preview

        grid.set_output_preview(Some(CraftingItem::new("result", "Result")));
        assert!(grid.can_craft());

        grid.craft();
        assert!(!grid.can_craft()); // On cooldown
    }

    #[test]
    fn test_crafting_grid_output() {
        let mut grid = CraftingGrid::new();
        grid.set_output(CraftingItem::new("result", "Result"));
        assert!(grid.output.has_item());

        let collected = grid.collect_output();
        assert!(collected.is_some());
        assert!(!grid.output.has_item());
    }

    #[test]
    fn test_crafting_grid_config_defaults() {
        let config = CraftingGridConfig::default();
        assert_eq!(config.slot_size, 48.0);
        assert!(config.show_stack_counts);
        assert!(config.show_cooldown);
    }

    #[test]
    fn test_crafting_grid_config_colors() {
        let config = CraftingGridConfig::default();
        let bg = config.slot_bg();
        let highlight = config.highlight();
        let locked = config.locked();

        assert_ne!(bg, highlight);
        assert_ne!(bg, locked);
    }

    #[test]
    fn test_crafting_grid_get_slot_at() {
        let mut grid = CraftingGrid::new();
        grid.set_item(4, CraftingItem::new("center", "Center")); // Row 1, Col 1

        let slot = grid.get_slot_at(1, 1);
        assert!(slot.is_some());
        assert!(slot.map(|s| s.has_item()).unwrap_or(false));

        assert!(grid.get_slot_at(3, 3).is_none());
    }

    #[test]
    fn test_crafting_grid_action_equality() {
        let action1 = CraftingGridAction::CraftClicked;
        let action2 = CraftingGridAction::CraftClicked;
        assert_eq!(action1, action2);

        let action3 = CraftingGridAction::ClearGrid;
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_crafting_item_serialization() {
        let item = CraftingItem::new("test", "Test Item")
            .with_count(5)
            .with_rarity(ItemRarity::Epic);

        let json = serde_json::to_string(&item).unwrap();
        let loaded: CraftingItem = serde_json::from_str(&json).unwrap();

        assert_eq!(item.id, loaded.id);
        assert_eq!(item.count, loaded.count);
        assert_eq!(item.rarity, loaded.rarity);
    }

    #[test]
    fn test_crafting_grid_config_serialization() {
        let config = CraftingGridConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: CraftingGridConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.slot_size, loaded.slot_size);
        assert_eq!(config.show_stack_counts, loaded.show_stack_counts);
    }

    #[test]
    fn test_item_rarity_serialization() {
        for rarity in [
            ItemRarity::Common,
            ItemRarity::Uncommon,
            ItemRarity::Rare,
            ItemRarity::Epic,
            ItemRarity::Legendary,
        ] {
            let json = serde_json::to_string(&rarity).unwrap();
            let loaded: ItemRarity = serde_json::from_str(&json).unwrap();
            assert_eq!(rarity, loaded);
        }
    }

    #[test]
    fn test_crafting_grid_drain_actions() {
        let mut grid = CraftingGrid::new();
        grid.set_item(0, CraftingItem::new("test", "Test"));

        let actions = grid.drain_actions();
        assert!(!actions.is_empty());

        let actions2 = grid.drain_actions();
        assert!(actions2.is_empty());
    }
}
