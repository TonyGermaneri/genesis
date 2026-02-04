# PROMPT — Gameplay Agent — Iteration 6

> **Branch**: `gameplay-agent`
> **Focus**: Terrain manipulation, top-down player controller, collision response, interaction system

## Your Mission

Make the world interactive! Players need to dig, place materials, and collide with terrain. Convert the platformer physics to smooth top-down movement.

---

## Tasks

### G-25: Terrain Manipulation System (P0)
**File**: `crates/genesis-gameplay/src/terrain_manipulation.rs`

Implement dig/place system for terrain modification:

```rust
/// Actions the player can perform on terrain
#[derive(Debug, Clone, Copy)]
pub enum TerrainAction {
    Dig { radius: f32 },
    Place { material: u16, radius: f32 },
    Fill { material: u16, radius: f32 },  // Only fills air
}

/// Terrain manipulation system
pub struct TerrainManipulator {
    /// Current selected material for placing
    pub selected_material: u16,
    /// Dig/place radius
    pub brush_radius: f32,
    /// Cooldown between actions
    pub action_cooldown: f32,
    /// Time until next action allowed
    cooldown_timer: f32,
}

impl TerrainManipulator {
    pub fn new() -> Self;
    
    /// Attempt to perform terrain action at world position
    /// Returns the cells that were modified
    pub fn perform_action(
        &mut self,
        action: TerrainAction,
        world_pos: (f32, f32),
        chunk_manager: &mut ChunkManager,
    ) -> Vec<(i32, i32, Cell)>;
    
    /// Update cooldown timer
    pub fn update(&mut self, dt: f32);
    
    /// Check if action is ready
    pub fn can_act(&self) -> bool;
    
    /// Set brush radius (clamped)
    pub fn set_radius(&mut self, radius: f32);
    
    /// Cycle to next material
    pub fn next_material(&mut self);
    
    /// Cycle to previous material
    pub fn prev_material(&mut self);
}

/// Generate intent for GPU kernel to apply terrain change
pub fn create_terrain_intent(
    action: TerrainAction,
    center: (i32, i32),
) -> Vec<Intent>;
```

---

### G-26: Top-Down Player Controller (P0)
**File**: `crates/genesis-gameplay/src/player.rs` (modify existing)

Convert platformer controller to top-down:

```rust
/// Top-down player configuration (replace PlayerConfig)
#[derive(Debug, Clone)]
pub struct TopDownPlayerConfig {
    pub walk_speed: f32,        // Normal movement speed
    pub run_speed: f32,         // Sprint speed (shift held)
    pub acceleration: f32,      // How fast to reach target speed
    pub friction: f32,          // How fast to stop (0-1, lower = more slide)
    pub interaction_range: f32, // How far player can interact
    pub dig_radius: f32,        // Default dig radius
    pub place_radius: f32,      // Default place radius
}

impl Player {
    /// Update for top-down movement (replace update method)
    pub fn update_topdown(&mut self, input: &Input, terrain: &ChunkManager, dt: f32) {
        // 1. Get input direction
        // 2. Apply acceleration towards target velocity
        // 3. Apply friction when no input
        // 4. Check collision with terrain
        // 5. Resolve collision (slide along walls)
        // 6. Update position
    }
    
    /// Get position player is aiming at (for dig/place)
    pub fn aim_position(&self, mouse_world: (f32, f32)) -> (f32, f32);
    
    /// Check if player can interact with position
    pub fn can_interact_at(&self, world_pos: (f32, f32)) -> bool;
}
```

Key changes from platformer:
- No gravity
- Full 8-direction movement
- Friction-based slowdown
- Collision slides along walls instead of stopping

---

### G-27: Player-World Collision Response (P0)
**File**: `crates/genesis-gameplay/src/collision_response.rs`

Handle collision response for smooth movement:

```rust
/// Collision response behavior
#[derive(Debug, Clone, Copy)]
pub enum CollisionBehavior {
    Stop,           // Stop movement on collision
    Slide,          // Slide along surface
    Bounce(f32),    // Bounce with coefficient
}

/// Process movement with collision
pub fn move_with_collision(
    position: &mut (f32, f32),
    velocity: &mut (f32, f32),
    radius: f32,
    chunk_manager: &ChunkManager,
    behavior: CollisionBehavior,
    dt: f32,
) -> bool;  // Returns true if collision occurred

/// Slide movement along walls (feels good for RPG)
pub fn slide_movement(
    start: (f32, f32),
    desired_end: (f32, f32),
    radius: f32,
    chunk_manager: &ChunkManager,
) -> (f32, f32);  // Returns actual end position

/// Check what terrain type player is standing on
pub fn terrain_at_feet(
    position: (f32, f32),
    chunk_manager: &ChunkManager,
) -> Option<u16>;
```

---

### G-28: Interaction System (P1)
**File**: `crates/genesis-gameplay/src/interaction.rs` (extend)

Wire up terrain manipulation to player input:

```rust
/// Player interaction handler
pub struct InteractionHandler {
    manipulator: TerrainManipulator,
    interaction_mode: InteractionMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionMode {
    Normal,     // Interact with objects
    Dig,        // Primary action = dig
    Place,      // Primary action = place
    Inspect,    // Show cell info
}

impl InteractionHandler {
    pub fn new() -> Self;
    
    /// Handle primary action (left click)
    pub fn primary_action(
        &mut self,
        player: &Player,
        world_pos: (f32, f32),
        chunk_manager: &mut ChunkManager,
    ) -> Option<InteractionResult>;
    
    /// Handle secondary action (right click)
    pub fn secondary_action(
        &mut self,
        player: &Player,
        world_pos: (f32, f32),
        chunk_manager: &mut ChunkManager,
    ) -> Option<InteractionResult>;
    
    /// Toggle interaction mode
    pub fn set_mode(&mut self, mode: InteractionMode);
    
    /// Update (cooldowns, etc)
    pub fn update(&mut self, dt: f32);
}

pub enum InteractionResult {
    TerrainModified(Vec<(i32, i32)>),
    ItemPickedUp(ItemTypeId),
    ObjectInteracted(EntityId),
    Nothing,
}
```

---

## Validation

After each task:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p genesis-gameplay
```

## Commit Format
```
[gameplay] feat: G-XX description
```

## Done Criteria
- [ ] Left-click digs terrain, right-click places
- [ ] Player slides smoothly along walls
- [ ] Movement feels responsive (no ice skating, no instant stop)
- [ ] Different terrain affects movement speed
