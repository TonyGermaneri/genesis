# PROMPT — Tools Agent — Iteration 6

> **Branch**: `tools-agent`
> **Focus**: Egui integration, game HUD, inventory UI, crafting UI

## Your Mission

Build a complete UI system for the RPG. Use egui for all UI rendering. The HUD should show vitals, hotbar, and minimap. Implement inventory and crafting panels.

---

## Tasks

### T-24: Egui Renderer Integration (P0)
**File**: `crates/genesis-tools/src/egui_integration.rs` (use existing, ensure working)

Ensure egui is properly integrated with the wgpu renderer:

```rust
impl EguiIntegration {
    /// Create integration with wgpu
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        window: &winit::window::Window,
        msaa_samples: u32,
    ) -> Self;
    
    /// Handle winit events (returns true if egui consumed event)
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool;
    
    /// Begin frame - call before any egui UI code
    pub fn begin_frame(&mut self, window: &Window);
    
    /// End frame - call after all egui UI code
    pub fn end_frame(&mut self, window: &Window) -> FullOutput;
    
    /// Render egui to screen (call after main game render, before present)
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        screen_descriptor: &ScreenDescriptor,
        output: FullOutput,
    );
    
    /// Get the egui context for drawing UI
    pub fn context(&self) -> &egui::Context;
}
```

**Important**: The UI must render AFTER the game world (on top).

---

### T-25: Main Game HUD (P0)
**File**: `crates/genesis-tools/src/game_hud.rs` (extend existing)

Create a complete in-game HUD:

```rust
/// Main game HUD renderer
pub struct GameHUD {
    show_inventory: bool,
    show_crafting: bool,
    show_debug: bool,
    config: HUDConfig,
}

pub struct HUDConfig {
    pub health_bar_width: f32,
    pub health_bar_height: f32,
    pub hotbar_slot_size: f32,
    pub minimap_size: f32,
    pub padding: f32,
}

impl GameHUD {
    pub fn new() -> Self;
    
    /// Render the complete HUD
    pub fn render(&mut self, ctx: &egui::Context, state: &HUDState);
    
    /// Toggle inventory visibility
    pub fn toggle_inventory(&mut self);
    
    /// Toggle crafting visibility
    pub fn toggle_crafting(&mut self);
}

/// State passed to HUD for rendering
pub struct HUDState<'a> {
    pub player: &'a Player,
    pub health: &'a Health,
    pub inventory: &'a Inventory,
    pub hotbar_selection: u8,
    pub fps: f32,
    pub player_position: (f32, f32),
    pub current_material: u16,
}

/// Individual HUD elements
impl GameHUD {
    /// Health/stamina bars in top-left
    fn render_vitals(&self, ui: &mut egui::Ui, health: &Health);
    
    /// Hotbar at bottom center
    fn render_hotbar(&self, ui: &mut egui::Ui, inventory: &Inventory, selection: u8);
    
    /// Minimap in top-right
    fn render_minimap(&self, ui: &mut egui::Ui, player_pos: (f32, f32));
    
    /// Current tool/material indicator
    fn render_tool_indicator(&self, ui: &mut egui::Ui, material: u16);
    
    /// Debug overlay (F3)
    fn render_debug(&self, ui: &mut egui::Ui, fps: f32, pos: (f32, f32));
}
```

**Layout**:
```
+------------------+--------------------+------------------+
| [HP BAR]         |                    | [MINIMAP]        |
| [STAMINA]        |                    |                  |
+------------------+                    +------------------+
|                                                          |
|                    GAME WORLD                            |
|                                                          |
|                                                          |
+----------------------------------------------------------+
|            [1][2][3][4][5][6][7][8][9][0]                |
|                   ^ selected                             |
+----------------------------------------------------------+
```

---

### T-26: Inventory UI Panel (P0)
**File**: `crates/genesis-tools/src/inventory_ui.rs` (extend existing)

Full inventory management UI:

```rust
/// Inventory panel UI
pub struct InventoryPanel {
    /// Currently dragged item (for drag-drop)
    dragging: Option<(usize, ItemStack)>,
    /// Hovered slot for tooltip
    hovered_slot: Option<usize>,
    /// Search/filter text
    filter_text: String,
}

impl InventoryPanel {
    pub fn new() -> Self;
    
    /// Render the inventory panel (call when visible)
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        inventory: &mut Inventory,
    ) -> InventoryAction;
    
    /// Render a single inventory slot
    fn render_slot(
        &mut self,
        ui: &mut egui::Ui,
        slot_index: usize,
        item: Option<&ItemStack>,
    ) -> SlotInteraction;
}

/// Actions from inventory interaction
pub enum InventoryAction {
    None,
    MoveItem { from: usize, to: usize },
    DropItem { slot: usize },
    UseItem { slot: usize },
    SplitStack { slot: usize, amount: u32 },
}

/// Item stack display
pub struct ItemStack {
    pub item_type: ItemTypeId,
    pub count: u32,
}
```

**Features**:
- Grid of slots (e.g., 6x5 = 30 slots)
- Drag and drop between slots
- Right-click to split stacks
- Hover tooltip shows item details
- Search bar to filter items

---

### T-27: Crafting UI Panel (P1)
**File**: `crates/genesis-tools/src/crafting_ui.rs` (extend existing)

Crafting interface:

```rust
/// Crafting panel UI
pub struct CraftingPanel {
    /// Currently selected recipe
    selected_recipe: Option<RecipeId>,
    /// Category filter
    category_filter: Option<CraftingCategory>,
    /// Search text
    search_text: String,
    /// Craft amount
    craft_amount: u32,
}

impl CraftingPanel {
    pub fn new() -> Self;
    
    /// Render the crafting panel
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        recipes: &RecipeRegistry,
        inventory: &Inventory,
    ) -> CraftingAction;
    
    /// Render recipe list (left side)
    fn render_recipe_list(
        &mut self,
        ui: &mut egui::Ui,
        recipes: &RecipeRegistry,
        inventory: &Inventory,
    );
    
    /// Render selected recipe details (right side)
    fn render_recipe_details(
        &mut self,
        ui: &mut egui::Ui,
        recipe: &Recipe,
        inventory: &Inventory,
    );
}

pub enum CraftingAction {
    None,
    Craft { recipe: RecipeId, amount: u32 },
    SelectRecipe(RecipeId),
}

#[derive(Debug, Clone, Copy)]
pub enum CraftingCategory {
    All,
    Tools,
    Weapons,
    Armor,
    Building,
    Consumables,
}
```

**Layout**:
```
+------------------------+------------------------+
| [Search...        ] 🔍 |  RECIPE NAME           |
+------------------------+                        |
| Category: [All ▼]      |  [ingredient icons]    |
+------------------------+  2x Wood               |
| > Wooden Pickaxe       |  1x Stone              |
|   Stone Pickaxe        |                        |
|   Iron Pickaxe         |  → [result icon]       |
|   ...                  |    Wooden Pickaxe      |
|                        |                        |
|                        |  Amount: [1] [+][-]    |
|                        |  [  CRAFT  ]           |
+------------------------+------------------------+
```

---

## Validation

After each task:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p genesis-tools
```

## Commit Format
```
[tools] feat: T-XX description
```

## Done Criteria
- [ ] Egui renders on top of game world
- [ ] HUD shows health, hotbar, minimap
- [ ] Inventory panel with drag-drop
- [ ] Crafting panel with recipe selection
- [ ] All UI is responsive and styled consistently
