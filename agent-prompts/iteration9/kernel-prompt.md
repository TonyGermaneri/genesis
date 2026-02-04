# Kernel Agent — Iteration 9 Prompt

## Context

You are the **Kernel Agent** for Project Genesis, a 2D top-down game engine built with Rust/wgpu.

**Current State:**
- Biome rendering with transitions working (K-32 to K-35)
- Water animation shader complete
- Cell-based world with multi-chunk streaming
- Day/night cycle and weather effects

**Iteration 9 Focus:** NPC rendering and collision detection.

---

## Assigned Tasks

### K-36: NPC sprite rendering (P0)

**Goal:** Render NPC entities as sprites with direction and animation frames.

**Implementation:**
1. Create `NpcRenderData` struct:
   - position: Vec2
   - direction: Direction (N/S/E/W)
   - animation_frame: u32
   - npc_type: u8 (for sprite selection)
   - scale: f32
2. Add NPC render pass after cell rendering
3. Use instanced rendering for efficiency
4. Support 4-direction sprites (facing direction)

**Files to modify:**
- `crates/genesis-kernel/src/render.rs` — Add NPC rendering
- Create `crates/genesis-kernel/src/npc_render.rs` if needed

```rust
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NpcInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub uv_offset: [f32; 2],
    pub uv_size: [f32; 2],
    pub tint: [f32; 4],
}
```

---

### K-37: NPC collision detection (P0)

**Goal:** Detect collisions between player and NPCs, and between NPCs.

**Implementation:**
1. Use circle collision for NPC bodies
2. Add to existing collision system or create `npc_collision.rs`
3. Return collision info: which NPC, penetration depth, normal
4. Support interaction radius (larger than collision radius)

```rust
pub struct NpcCollision {
    pub npc_id: u32,
    pub penetration: f32,
    pub normal: Vec2,
    pub in_interaction_range: bool,
}

pub fn check_npc_collisions(
    player_pos: Vec2,
    player_radius: f32,
    npcs: &[NpcPosition],
) -> Vec<NpcCollision>;
```

---

### K-38: NPC batch rendering (P1)

**Goal:** Efficiently render many NPCs using instancing.

**Implementation:**
1. Create instance buffer for NPC data
2. Single draw call for all NPCs of same type
3. Update instance buffer only when NPCs move
4. Support up to 1000 NPCs visible at once

---

### K-39: Speech bubble rendering (P1)

**Goal:** Render dialogue text above NPCs.

**Implementation:**
1. Simple rounded rectangle background
2. Text rendered using egui or custom text rendering
3. Position above NPC sprite
4. Fade in/out animation

---

## Constraints

1. **Performance:** NPC rendering must add < 2ms to frame time
2. **GPU-friendly:** Use instanced rendering, minimize draw calls
3. **No AI logic:** Only rendering and collision detection
4. **Coordinate system:** Use same world coordinates as cells

---

## Commit Format

```
[kernel] feat: K-36..K-39 NPC rendering and collision detection
```
