# Tools Agent — Iteration 9 Prompt

## Context

You are the **Tools Agent** for Project Genesis, a 2D top-down game engine built with Rust and egui.

**Current State:**
- Biome minimap and debug info (T-32 to T-35)
- Inventory UI, stats HUD complete
- Weather/time HUD working
- Debug panels for various systems

**Iteration 9 Focus:** NPC-related UI: dialogue, debug overlays, spawn editor.

---

## Assigned Tasks

### T-36: Dialogue UI panel (P0)

**Goal:** Display NPC dialogue with player choices.

**Implementation:**
1. Create dialogue window that appears when talking to NPC
2. Layout:
   - NPC portrait/name at top
   - Dialogue text in main area
   - Choice buttons at bottom
3. Features:
   - Text typewriter effect (optional)
   - Choice highlighting on hover
   - Keyboard navigation (1-4 for choices)
   - Close button / ESC to exit

```rust
pub struct DialoguePanel {
    visible: bool,
    npc_name: String,
    npc_portrait: Option<TextureId>,
    current_text: String,
    choices: Vec<String>,
    selected_choice: usize,
}

impl DialoguePanel {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<usize> {
        // Returns selected choice index when player chooses
    }
}
```

**Files to modify:**
- Create `crates/genesis-tools/src/dialogue_ui.rs`
- Update `lib.rs` to export

---

### T-37: NPC debug overlay (P0)

**Goal:** Debug visualization for NPC AI state.

**Implementation:**
1. Overlay showing for each NPC (toggle with key):
   - NPC ID and type
   - Current state (Idle/Walking/etc)
   - AI behavior (Patrol/Wander/etc)
   - Target position (line to target)
   - Collision radius (circle)
   - Interaction radius (larger circle)
   - Health bar above NPC

2. In debug panel:
   - Total NPC count
   - NPCs per chunk
   - AI update time
   - Selected NPC details

---

### T-38: NPC spawn editor (P1)

**Goal:** Debug tool to manually spawn/remove NPCs.

**Implementation:**
1. Spawn panel:
   - Dropdown to select NPC type
   - Click on world to spawn at location
   - Or spawn at player position
2. Remove:
   - Click on NPC to select
   - Delete button or key to remove
3. List of spawned debug NPCs (separate from natural spawns)

---

### T-39: NPC list panel (P1)

**Goal:** List all NPCs in loaded chunks.

**Implementation:**
1. Scrollable list showing:
   - NPC ID
   - Type
   - Position
   - State
   - Distance from player
2. Click to select/highlight NPC
3. "Teleport to" button for debugging
4. Filter by type or state

---

## Constraints

1. Use egui for all UI
2. Consistent with existing UI style
3. No direct NPC mutation - emit events/intents
4. Toggle-able overlays (don't clutter screen)

---

## Commit Format

```
[tools] feat: T-36..T-39 Dialogue UI and NPC debug tools
```
