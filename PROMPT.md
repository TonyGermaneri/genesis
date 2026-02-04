# PROMPT — Infra Agent — Iteration 5

> **Branch**: `infra-agent`
> **Focus**: App.rs integration, main loop, input handling, config

## Your Mission

Wire up the engine main loop in `app.rs`. This is the CRITICAL integration task. Complete these tasks sequentially, validating after each.

---

## Tasks

### I-16: Input System Integration (P0)
**File**: `crates/genesis-engine/src/input.rs`

Handle all input and convert to game actions:

```rust
use winit::event::{WindowEvent, KeyEvent, ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use genesis_gameplay::input::InputState;

pub struct InputHandler {
    current_state: InputState,
    keys_pressed: HashSet<KeyCode>,
    mouse_position: (f32, f32),
    mouse_buttons: HashSet<MouseButton>,
}

impl InputHandler {
    pub fn new() -> Self;

    /// Handle a winit window event, returns true if handled
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool;

    /// Get current input state for gameplay
    pub fn state(&self) -> &InputState;

    /// Check if a key is currently held
    pub fn is_key_pressed(&self, key: KeyCode) -> bool;

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool;

    /// Get mouse position in screen coordinates
    pub fn mouse_position(&self) -> (f32, f32);

    /// Reset per-frame state (call at end of frame)
    pub fn end_frame(&mut self);
}

/// Map physical keys to game input state
impl InputState {
    pub fn from_handler(handler: &InputHandler) -> Self {
        Self {
            move_left: handler.is_key_pressed(KeyCode::KeyA) || handler.is_key_pressed(KeyCode::ArrowLeft),
            move_right: handler.is_key_pressed(KeyCode::KeyD) || handler.is_key_pressed(KeyCode::ArrowRight),
            move_up: handler.is_key_pressed(KeyCode::KeyW) || handler.is_key_pressed(KeyCode::ArrowUp),
            move_down: handler.is_key_pressed(KeyCode::KeyS) || handler.is_key_pressed(KeyCode::ArrowDown),
            jump: handler.is_key_pressed(KeyCode::Space),
            action_primary: handler.mouse_buttons.contains(&MouseButton::Left),
            action_secondary: handler.mouse_buttons.contains(&MouseButton::Right),
            // ... etc
        }
    }
}
```

Requirements:
- WASD + Arrow keys for movement
- Space for jump
- Mouse buttons for actions
- Number keys 1-9,0 for hotbar
- F3 for debug overlay
- Escape for pause/menu
- Track just-pressed vs held

### I-17: Main Game Loop Integration (P0)
**File**: Update `crates/genesis-engine/src/app.rs`

Integrate all subsystems into the main loop:

```rust
use genesis_gameplay::GameplaySystem;
use genesis_kernel::{Camera, TerrainRenderer, WorldInitializer};
use genesis_tools::{EguiIntegration, GameHUD, FpsCounter};
use crate::input::InputHandler;

struct GenesisApp {
    // Existing fields...
    config: EngineConfig,
    window: Option<Window>,
    renderer: Option<Renderer>,

    // NEW: Game systems
    gameplay: Option<GameplaySystem>,
    terrain: Option<TerrainRenderer>,
    camera: Option<Camera>,
    egui: Option<EguiIntegration>,
    hud: Option<GameHUD>,
    input: InputHandler,
    fps_counter: FpsCounter,
    last_update: std::time::Instant,
}

impl ApplicationHandler for GenesisApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ... existing window/renderer creation ...

        // Initialize game systems
        let seed = self.config.world_seed.unwrap_or_else(|| rand::random());

        // Initialize terrain renderer
        let mut terrain = TerrainRenderer::new(seed, self.renderer.device());
        let world_init = WorldInitializer::new(seed);
        world_init.generate_starting_area(&mut terrain, self.renderer.device());

        // Initialize gameplay
        let spawn_point = world_init.find_spawn_position(&terrain);
        let mut gameplay = GameplaySystem::new(seed);
        gameplay.initialize(spawn_point);

        // Initialize camera centered on player
        let mut camera = Camera::new(self.config.window_width, self.config.window_height);
        camera.center_on(spawn_point.0, spawn_point.1);

        // Initialize egui
        let egui = EguiIntegration::new(
            self.renderer.device(),
            self.renderer.surface_format(),
            &window,
        );

        // Store systems
        self.terrain = Some(terrain);
        self.gameplay = Some(gameplay);
        self.camera = Some(camera);
        self.egui = Some(egui);
        self.hud = Some(GameHUD::new());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        // Let egui handle event first
        if let Some(egui) = &mut self.egui {
            if egui.handle_event(&self.window.as_ref().unwrap(), &event) {
                return; // egui consumed the event
            }
        }

        // Handle input
        self.input.handle_event(&event);

        match event {
            WindowEvent::RedrawRequested => {
                self.update_and_render();
            },
            // ... other events
        }
    }
}

impl GenesisApp {
    fn update_and_render(&mut self) {
        let now = std::time::Instant::now();
        let dt = (now - self.last_update).as_secs_f32();
        self.last_update = now;

        let (fps, frame_time) = self.fps_counter.tick();

        // Update gameplay
        if let (Some(gameplay), Some(terrain), Some(camera)) =
            (&mut self.gameplay, &mut self.terrain, &mut self.camera)
        {
            let input_state = InputState::from_handler(&self.input);
            gameplay.update(&input_state, terrain.collision_query(), dt);

            // Update camera to follow player
            let player_pos = gameplay.player_position();
            camera.center_on(player_pos.0, player_pos.1);

            // Update visible terrain
            terrain.update_visible(camera);
            terrain.generate_pending(self.renderer.device(), 2);
        }

        // Render
        if let Some(renderer) = &mut self.renderer {
            // ... render terrain with camera
            // ... render egui HUD
        }

        self.input.end_frame();
    }
}
```

Requirements:
- Proper delta time calculation
- Input → gameplay → camera → render pipeline
- Terrain streaming based on camera
- HUD rendering after game world
- Pause when escape pressed

### I-18: Engine Configuration (P0)
**File**: Update `crates/genesis-engine/src/config.rs`

Expand engine configuration:

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    // Window settings
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,

    // World settings
    pub world_seed: Option<u64>,
    pub render_distance: u32,  // in chunks

    // Graphics settings
    pub cell_scale: f32,
    pub enable_particles: bool,
    pub enable_lighting: bool,

    // Debug settings
    pub show_fps: bool,
    pub show_debug_overlay: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            vsync: true,
            world_seed: None,
            render_distance: 4,
            cell_scale: 4.0,
            enable_particles: true,
            enable_lighting: true,
            show_fps: true,
            show_debug_overlay: false,
        }
    }
}

impl EngineConfig {
    /// Load from file or return default
    pub fn load() -> Self;

    /// Save to file
    pub fn save(&self) -> Result<(), std::io::Error>;
}
```

Requirements:
- Sensible defaults
- Optional config file loading
- All tunable parameters exposed

### I-19: Frame Timing & Performance (P1)
**File**: `crates/genesis-engine/src/timing.rs`

Proper frame timing:

```rust
pub struct FrameTiming {
    target_fps: u32,
    frame_budget: std::time::Duration,
    last_frame: std::time::Instant,
    accumulator: f32,
    fixed_dt: f32,
}

impl FrameTiming {
    pub fn new(target_fps: u32) -> Self;

    /// Get time since last frame
    pub fn delta_time(&mut self) -> f32;

    /// For fixed timestep physics (call in loop)
    pub fn should_update_fixed(&mut self) -> bool;

    /// Sleep to maintain target framerate (if vsync off)
    pub fn sleep_remainder(&self);

    /// Get current FPS
    pub fn current_fps(&self) -> f32;
}
```

Requirements:
- Smooth delta time
- Fixed timestep option for physics
- FPS limiting when vsync off
- Prevent spiral of death

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
[infra] feat: I-16 input system integration
[infra] feat: I-17 main game loop integration
[infra] feat: I-18 engine configuration
[infra] feat: I-19 frame timing
```

---

## Integration Notes

- I-16 input bridges winit → gameplay input state
- I-17 is the MAIN integration task - wire everything together
- I-18 config used throughout engine
- I-19 timing ensures smooth gameplay

**CRITICAL**: Task I-17 must make the game actually playable:
- Arrow keys/WASD move player
- Camera follows player
- Terrain generates as you explore
- HUD shows health/hotbar
- F3 shows debug info
