# PROMPT — Tools Agent — Iteration 3

> **Branch**: `tools-agent`
> **Focus**: Inventory UI rendering, crafting UI rendering, minimap, debug console

## Your Mission

Complete the following tasks. Work through them sequentially. After each task, run the validation loop. Commit after each task passes.

---

## Tasks

### T-12: Inventory UI Renderer (P0)
**File**: `crates/genesis-tools/src/inventory_ui.rs`

Render inventory using egui:

```rust
use egui::{Context, Window, Grid, Image, Response};
use genesis_gameplay::inventory_ui::{InventoryUIModel, InventoryAction, SlotUIData};

pub struct InventoryUI {
    pub is_open: bool,
    slot_size: f32,
    hotbar_y: f32,
}

impl InventoryUI {
    pub fn new() -> Self;

    pub fn show(&mut self, ctx: &Context, model: &mut InventoryUIModel) -> Vec<InventoryAction>;

    pub fn show_hotbar(&mut self, ctx: &Context, model: &InventoryUIModel) -> Option<InventoryAction>;

    fn render_slot(&self, ui: &mut egui::Ui, slot: &SlotUIData) -> Response;

    fn render_tooltip(&self, ui: &mut egui::Ui, tooltip: &TooltipData);
}
```

Requirements:
- Grid layout for inventory slots
- Hotbar always visible at bottom
- Drag and drop between slots
- Right-click context menu (use, drop, split)
- Tooltip on hover
- Item count overlay on slot

### T-13: Crafting UI Renderer (P0)
**File**: `crates/genesis-tools/src/crafting_ui.rs`

Render crafting interface using egui:

```rust
use egui::{Context, Window, ScrollArea};
use genesis_gameplay::crafting_ui::{CraftingUIModel, RecipeUIData};

pub struct CraftingUI {
    pub is_open: bool,
    search_text: String,
}

impl CraftingUI {
    pub fn new() -> Self;

    pub fn show(&mut self, ctx: &Context, model: &mut CraftingUIModel) -> Option<CraftRequest>;

    fn render_recipe_list(&mut self, ui: &mut egui::Ui, model: &CraftingUIModel) -> Option<usize>;

    fn render_recipe_detail(&self, ui: &mut egui::Ui, recipe: &RecipeUIData);

    fn render_crafting_queue(&self, ui: &mut egui::Ui, model: &CraftingUIModel);
}

pub struct CraftRequest {
    pub recipe_id: RecipeId,
    pub count: u32,
}
```

Requirements:
- Recipe list with filter/search
- Recipe detail panel (ingredients, outputs)
- "Craft" button (disabled if can't craft)
- Crafting queue with progress bars
- Visual feedback for missing ingredients

### T-14: Minimap Renderer (P1)
**File**: `crates/genesis-tools/src/minimap.rs`

Implement minimap display:

```rust
use egui::{Context, Painter, Rect, Color32};

pub struct Minimap {
    pub is_visible: bool,
    pub size: f32,
    pub zoom: f32,
    texture: Option<egui::TextureHandle>,
}

pub struct MinimapData {
    pub player_pos: (f32, f32),
    pub player_rotation: f32,
    pub entities: Vec<MinimapEntity>,
    pub terrain_colors: Vec<u8>,  // RGBA for terrain
    pub width: u32,
    pub height: u32,
}

pub struct MinimapEntity {
    pub pos: (f32, f32),
    pub entity_type: MinimapEntityType,
}

pub enum MinimapEntityType {
    Player,
    NPC,
    Enemy,
    Item,
    Building,
}

impl Minimap {
    pub fn new(size: f32) -> Self;

    pub fn show(&mut self, ctx: &Context, data: &MinimapData);

    pub fn update_terrain(&mut self, ctx: &Context, colors: &[u8], width: u32, height: u32);

    fn world_to_minimap(&self, world_pos: (f32, f32), center: (f32, f32)) -> Option<(f32, f32)>;
}
```

Requirements:
- Corner overlay (top-right by default)
- Terrain texture from chunk data
- Entity markers with icons/colors
- Player arrow showing direction
- Zoom in/out controls
- Click to set waypoint (optional)

### T-15: Debug Console (P1)
**File**: `crates/genesis-tools/src/console.rs`

Implement in-game debug console:

```rust
use egui::{Context, Window, TextEdit, ScrollArea};

pub struct DebugConsole {
    pub is_open: bool,
    input_buffer: String,
    history: Vec<ConsoleEntry>,
    command_history: Vec<String>,
    history_index: Option<usize>,
}

pub struct ConsoleEntry {
    pub timestamp: f64,
    pub level: ConsoleLevel,
    pub message: String,
}

pub enum ConsoleLevel {
    Info,
    Warning,
    Error,
    Command,
    Result,
}

pub trait ConsoleCommand {
    fn name(&self) -> &str;
    fn help(&self) -> &str;
    fn execute(&self, args: &[&str]) -> String;
}

impl DebugConsole {
    pub fn new() -> Self;

    pub fn show(&mut self, ctx: &Context, commands: &[Box<dyn ConsoleCommand>]) -> Option<String>;

    pub fn log(&mut self, level: ConsoleLevel, message: String);

    pub fn execute(&mut self, input: &str, commands: &[Box<dyn ConsoleCommand>]);
}

// Built-in commands
pub struct HelpCommand;
pub struct ClearCommand;
pub struct TeleportCommand;
pub struct SpawnCommand;
pub struct GiveCommand;
pub struct SetTimeCommand;
```

Requirements:
- Toggle with backtick/tilde key
- Command history (up/down arrows)
- Tab completion for commands
- Color-coded output levels
- Scrollable history
- Built-in debug commands

---

## Validation Loop

After each task:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --workspace
```

If ANY step fails, FIX IT before committing.

---

## Commit Convention

```
[tools] feat: T-12 inventory UI renderer
[tools] feat: T-13 crafting UI renderer
[tools] feat: T-14 minimap renderer
[tools] feat: T-15 debug console
```

---

## Integration Notes

- T-12/T-13 consume data models from genesis-gameplay
- Add genesis-gameplay dependency if not present
- Use egui 0.30 (already in workspace)
- Export new modules in lib.rs
- Test with mock data if gameplay types not available
