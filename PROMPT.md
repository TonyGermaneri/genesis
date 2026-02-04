# PROMPT — Gameplay Agent — Iteration 3

> **Branch**: `gameplay-agent`
> **Focus**: Player physics, inventory UI data, crafting UI, save/load

## Your Mission

Complete the following tasks. Work through them sequentially. After each task, run the validation loop. Commit after each task passes.

---

## Tasks

### G-13: Player Physics Integration (P0)
**File**: `crates/genesis-gameplay/src/physics.rs`

Integrate player movement with kernel collision:

```rust
use genesis_kernel::collision::CollisionQuery;

pub struct PlayerPhysics {
    pub gravity: f32,
    pub move_speed: f32,
    pub jump_velocity: f32,
    pub friction: f32,
}

impl PlayerPhysics {
    pub fn new() -> Self;
    pub fn update(
        &self,
        player: &mut Player,
        input: &InputState,
        collision: &CollisionQuery,
        dt: f32,
    );
    pub fn is_grounded(&self, player: &Player, collision: &CollisionQuery) -> bool;
}
```

Requirements:
- AABB collision with cell grid
- Gravity when not grounded
- Wall sliding (don't stick to walls)
- Jump only when grounded
- Velocity clamping

### G-14: Inventory UI Model (P0)
**File**: `crates/genesis-gameplay/src/inventory_ui.rs`

Prepare inventory data for UI rendering:

```rust
pub struct InventoryUIModel {
    pub slots: Vec<SlotUIData>,
    pub selected_slot: Option<usize>,
    pub drag_item: Option<ItemStack>,
    pub tooltip: Option<TooltipData>,
}

pub struct SlotUIData {
    pub index: usize,
    pub item: Option<ItemStack>,
    pub is_hotbar: bool,
    pub is_selected: bool,
}

pub struct TooltipData {
    pub item_name: String,
    pub description: String,
    pub stats: Vec<(String, String)>,
}

impl InventoryUIModel {
    pub fn from_inventory(inv: &Inventory, hotbar_size: usize) -> Self;
    pub fn handle_click(&mut self, slot: usize, button: MouseButton) -> InventoryAction;
    pub fn handle_drag(&mut self, from: usize, to: usize) -> InventoryAction;
}

pub enum InventoryAction {
    None,
    Move { from: usize, to: usize },
    Split { slot: usize },
    Drop { slot: usize, count: u32 },
    Use { slot: usize },
}
```

Requirements:
- Transform Inventory → display model
- Handle slot click/drag actions
- Return actions for Inventory to execute
- Tooltip generation from item metadata

### G-15: Crafting UI Model (P0)
**File**: `crates/genesis-gameplay/src/crafting_ui.rs`

Prepare crafting data for UI rendering:

```rust
pub struct CraftingUIModel {
    pub available_recipes: Vec<RecipeUIData>,
    pub selected_recipe: Option<usize>,
    pub crafting_queue: Vec<CraftingQueueItem>,
    pub filter: RecipeFilter,
}

pub struct RecipeUIData {
    pub recipe_id: RecipeId,
    pub name: String,
    pub icon: String,
    pub can_craft: bool,
    pub missing_ingredients: Vec<String>,
    pub inputs: Vec<IngredientUIData>,
    pub outputs: Vec<IngredientUIData>,
}

pub struct IngredientUIData {
    pub item_name: String,
    pub required: u32,
    pub available: u32,
}

pub struct CraftingQueueItem {
    pub recipe_id: RecipeId,
    pub progress: f32,
    pub time_remaining: f32,
}

pub enum RecipeFilter {
    All,
    Craftable,
    Category(String),
    Search(String),
}

impl CraftingUIModel {
    pub fn from_state(
        recipes: &[CraftingRecipe],
        inventory: &Inventory,
        queue: &CraftingQueue,
        filter: RecipeFilter,
    ) -> Self;
    pub fn select_recipe(&mut self, index: usize);
    pub fn queue_craft(&self) -> Option<RecipeId>;
}
```

Requirements:
- Filter recipes by craftability, category, search
- Show ingredient availability
- Display crafting queue progress
- Sort by name/category/craftable

### G-16: Save/Load Game State (P1)
**File**: `crates/genesis-gameplay/src/save.rs`

Implement game state serialization:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SaveGame {
    pub version: u32,
    pub timestamp: u64,
    pub player: PlayerSaveData,
    pub entities: Vec<EntitySaveData>,
    pub world_seed: u64,
    pub game_time: f64,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerSaveData {
    pub position: (f32, f32),
    pub health: f32,
    pub inventory: Vec<ItemStackSaveData>,
    pub equipped: Vec<Option<ItemStackSaveData>>,
}

pub struct SaveManager {
    save_dir: PathBuf,
}

impl SaveManager {
    pub fn new(save_dir: PathBuf) -> Self;
    pub fn save(&self, name: &str, data: &SaveGame) -> Result<(), SaveError>;
    pub fn load(&self, name: &str) -> Result<SaveGame, SaveError>;
    pub fn list_saves(&self) -> Vec<SaveMetadata>;
    pub fn delete(&self, name: &str) -> Result<(), SaveError>;
}

pub struct SaveMetadata {
    pub name: String,
    pub timestamp: u64,
    pub playtime: f64,
}
```

Requirements:
- Binary format (bincode) for compactness
- Version field for migrations
- Save metadata without loading full save
- Atomic writes (write temp, rename)

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
[gameplay] feat: G-13 player physics integration
[gameplay] feat: G-14 inventory UI model
[gameplay] feat: G-15 crafting UI model
[gameplay] feat: G-16 save/load game state
```

---

## Integration Notes

- G-13 uses CollisionQuery from genesis-kernel (add dependency)
- G-14/G-15 provide data models, actual UI in genesis-tools
- G-16 coordinates with chunk save in genesis-world
- Export new modules in lib.rs
