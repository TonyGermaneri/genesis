# PROMPT — Tools Agent — Iteration 5

> **Branch**: `tools-agent`
> **Focus**: HUD integration, inventory hotbar, debug overlay, egui setup

## Your Mission

Wire up the UI subsystems for visible gameplay. Complete these tasks sequentially, validating after each.

---

## Tasks

### T-20: Egui Integration Layer (P0)
**File**: `crates/genesis-tools/src/egui_integration.rs`

Set up egui for in-game UI:

```rust
use egui::{Context, FullOutput};
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiWinit;
use winit::window::Window;

pub struct EguiIntegration {
    context: Context,
    state: EguiWinit,
    renderer: EguiRenderer,
}

impl EguiIntegration {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self;

    /// Handle winit window event, returns true if egui consumed it
    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool;

    /// Begin frame - call before any UI code
    pub fn begin_frame(&mut self, window: &Window);

    /// End frame and get paint jobs
    pub fn end_frame(&mut self, window: &Window) -> FullOutput;

    /// Render egui output
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        output: FullOutput,
    );

    /// Get egui context for UI code
    pub fn context(&self) -> &Context;
}
```

Requirements:
- Clean egui-wgpu-winit setup
- Event handling passthrough
- Proper frame lifecycle
- Screen scaling support

### T-21: Game HUD Renderer (P0)
**File**: `crates/genesis-tools/src/game_hud.rs`

Render the main gameplay HUD:

```rust
use egui::Context;
use genesis_gameplay::{Player, Inventory};

pub struct GameHUD {
    show_debug: bool,
    show_inventory: bool,
}

impl GameHUD {
    pub fn new() -> Self;

    /// Render the full HUD
    pub fn render(
        &mut self,
        ctx: &Context,
        player: &Player,
        inventory: &Inventory,
        fps: f32,
        frame_time: f32,
    );

    /// Render health/stamina bars (top-left)
    fn render_vitals(&self, ctx: &Context, player: &Player);

    /// Render hotbar (bottom-center)
    fn render_hotbar(&self, ctx: &Context, inventory: &Inventory);

    /// Render debug info (top-right, toggleable)
    fn render_debug(&self, ctx: &Context, fps: f32, frame_time: f32, player: &Player);

    /// Toggle debug overlay
    pub fn toggle_debug(&mut self);

    /// Toggle inventory panel
    pub fn toggle_inventory(&mut self);
}
```

Requirements:
- Health bar with color gradient
- Hotbar with 10 slots (1-9, 0)
- Selected slot highlight
- Debug info: FPS, position, velocity, chunk
- Minimap placeholder

### T-22: Hotbar Widget (P0)
**File**: `crates/genesis-tools/src/hotbar.rs`

Focused hotbar implementation:

```rust
use egui::{Context, Ui, Response};
use genesis_gameplay::inventory::{Inventory, ItemStack};

pub struct Hotbar {
    selected_slot: usize,
    slot_size: f32,
}

impl Hotbar {
    pub fn new() -> Self;

    /// Render hotbar, returns selected slot if changed
    pub fn render(&mut self, ctx: &Context, inventory: &Inventory) -> Option<usize>;

    /// Select slot by number (0-9)
    pub fn select_slot(&mut self, slot: usize);

    /// Select next/previous slot (mouse wheel)
    pub fn cycle_slot(&mut self, delta: i32);

    /// Get currently selected slot
    pub fn selected(&self) -> usize;

    /// Render single slot with item
    fn render_slot(&self, ui: &mut Ui, slot: usize, item: Option<&ItemStack>, selected: bool) -> Response;
}
```

Requirements:
- 10 slots horizontally centered
- Number key hints (1-9, 0)
- Item icon and count
- Selection highlight
- Mouse wheel cycling

### T-23: Debug Overlay (P1)
**File**: `crates/genesis-tools/src/debug_overlay.rs`

Comprehensive debug information:

```rust
use egui::Context;
use genesis_gameplay::Player;
use genesis_kernel::Camera;

pub struct DebugOverlay {
    visible: bool,
    show_fps: bool,
    show_position: bool,
    show_chunk_info: bool,
    show_memory: bool,
}

impl DebugOverlay {
    pub fn new() -> Self;

    pub fn render(
        &mut self,
        ctx: &Context,
        fps: f32,
        frame_time_ms: f32,
        player: &Player,
        camera: &Camera,
        memory_usage: usize,
    );

    pub fn toggle(&mut self);
    pub fn is_visible(&self) -> bool;
}

/// FPS tracking helper
pub struct FpsCounter {
    frames: VecDeque<f64>,
    last_time: std::time::Instant,
}

impl FpsCounter {
    pub fn new() -> Self;
    pub fn tick(&mut self) -> (f32, f32);  // (fps, frame_time_ms)
}
```

Requirements:
- F3 to toggle (like Minecraft)
- Smooth FPS counter (rolling average)
- Player position and velocity
- Current chunk coordinates
- Memory usage estimate

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
[tools] feat: T-20 egui integration layer
[tools] feat: T-21 game HUD renderer
[tools] feat: T-22 hotbar widget
[tools] feat: T-23 debug overlay
```

---

## Dependencies

Ensure these are in `crates/genesis-tools/Cargo.toml`:
```toml
egui = "0.30"
egui-wgpu = "0.30"
egui-winit = "0.30"
```

---

## Integration Notes

- T-20 provides egui setup used by app.rs
- T-21 HUD rendered each frame after game render
- T-22 hotbar integrated with inventory
- T-23 debug toggled by F3 key
- Export new modules in lib.rs
