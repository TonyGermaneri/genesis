# Infra Agent — Iteration 9 Prompt

## Context

You are the **Infra Agent** for Project Genesis, a 2D top-down game engine built with Rust.

**Current State:**
- Biome generation wiring complete (I-28 to I-31)
- ChunkManager handles chunk loading/unloading
- Game loop processes input, updates, renders
- Player interaction system working

**Iteration 9 Focus:** Wire NPC system into game loop and chunk management.

---

## Assigned Tasks

### I-32: NPC manager integration (P0)

**Goal:** Add NPC system to the main game loop.

**Implementation:**
1. Create/use NpcManager from gameplay crate
2. In App struct, add:
   ```rust
   npc_manager: NpcManager,
   ```
3. In game loop:
   - Update AI behaviors each tick
   - Update NPC positions
   - Check for state transitions
4. Pass NPC data to renderer

```rust
// In update loop
self.npc_manager.update(dt, &self.world, &self.player);

// Before render
let npc_render_data = self.npc_manager.get_render_data();
renderer.render_npcs(&npc_render_data);
```

---

### I-33: NPC-player interaction (P0)

**Goal:** Detect when player wants to interact with NPC.

**Implementation:**
1. Check for interact key (E) press
2. Find nearest NPC within interaction range
3. If found, start dialogue or interaction
4. Handle interaction state:
   - Lock player movement during dialogue
   - Unlock when dialogue ends

```rust
pub fn handle_npc_interaction(&mut self, input: &Input) {
    if input.interact_just_pressed {
        if let Some(npc_id) = self.find_nearest_interactable_npc() {
            self.start_interaction(npc_id);
        }
    }
    
    if self.in_dialogue {
        // Handle dialogue input
        if let Some(choice) = self.dialogue_ui.get_choice() {
            self.dialogue_manager.select_choice(choice);
        }
    }
}
```

---

### I-34: NPC chunk loading (P0)

**Goal:** Spawn and despawn NPCs with chunk loading.

**Implementation:**
1. On chunk load:
   - Generate spawn positions using seed + chunk coords
   - Create NPCs according to spawn rules
   - Add to NpcManager
2. On chunk unload:
   - Save NPC state if modified
   - Remove NPCs from NpcManager
3. Persistent NPCs (named/quest NPCs) handled separately

```rust
// In ChunkManager
pub fn on_chunk_loaded(&mut self, chunk_pos: (i32, i32), npc_manager: &mut NpcManager) {
    let spawn_data = self.terrain_gen.get_npc_spawns(chunk_pos);
    for spawn in spawn_data {
        npc_manager.spawn(spawn.npc_type, spawn.position);
    }
}

pub fn on_chunk_unloaded(&mut self, chunk_pos: (i32, i32), npc_manager: &mut NpcManager) {
    npc_manager.despawn_in_chunk(chunk_pos);
}
```

---

### I-35: NPC update profiling (P1)

**Goal:** Measure and report NPC system performance.

**Implementation:**
1. Time NPC AI update
2. Time NPC collision checks
3. Time NPC rendering
4. Report in debug panel:
   - AI update: X.XXms
   - Collision: X.XXms
   - Render: X.XXms
   - Active NPCs: N
   - NPCs in view: N

---

## Constraints

1. **Minimal coupling:** Use traits/interfaces between crates
2. **Thread-safe:** NPC updates could be parallelized later
3. **Deterministic:** Same seed = same NPC spawns
4. **Memory efficient:** Unload NPCs when chunks unload

---

## Commit Format

```
[infra] feat: I-32..I-35 NPC manager integration and chunk loading
```
