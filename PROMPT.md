# Tools Agent — Iteration 8 Prompt

## Context

You are the **Tools Agent** for Project Genesis, a 2D top-down game engine built with Rust and egui.

**Current State:**
- Inventory panel UI (T-28)
- Player stats HUD (T-29)
- Weather/time HUD (T-30)
- Minimap with chunks (T-31)
- Debug panel exists showing FPS, position, chunk info

**Iteration 8 Focus:** Add biome visualization and debug tools.

---

## Assigned Tasks

### T-32: Biome minimap coloring (P0)

**Goal:** Color-code the minimap based on biome type.

**Implementation:**
1. Update minimap rendering to show biome colors:
   - Forest: Dark green (#2d5a1d)
   - Desert: Sandy yellow (#c4a35a)
   - Lake: Blue (#3a7ca5)
   - Plains: Light green (#7cb342)
   - Mountain: Gray (#7a7a7a)
   - Swamp: Dark olive (#4a5a23)
2. Show current player position marker
3. Show chunk boundaries as grid overlay (toggle-able)

**Files to modify:**
- crates/genesis-tools/src/egui_integration.rs (or wherever minimap is)

---

### T-33: Debug biome info panel (P0)

**Goal:** Show current biome information in debug panel.

**Implementation:**
1. Add to debug panel:
   - Current biome name
   - Temperature value
   - Humidity value
   - Elevation value
   - Noise values (raw)
2. Toggle visibility with existing debug key

```rust
// Example debug panel addition
ui.label(format!("Biome: {:?}", current_biome));
ui.label(format!("Temp: {:.2}", temperature));
ui.label(format!("Humidity: {:.2}", humidity));
ui.label(format!("Elevation: {:.2}", elevation));
```

---

### T-34: World seed display/input (P0)

**Goal:** Allow viewing and setting world seed.

**Implementation:**
1. In settings or debug panel:
   - Display current world seed
   - Input field for new seed
   - "Randomize" button for new random seed
   - "Apply" button to regenerate world
2. Seed should be u64, displayed as decimal or hex

---

### T-35: Biome legend overlay (P1)

**Goal:** Show a color key for biome types.

**Implementation:**
1. Toggleable overlay showing:
   - Colored square + biome name for each type
   - Position in corner of screen
2. Toggle with key (e.g., L for Legend)

---

## Constraints

1. Use egui for all UI
2. Consistent with existing UI style
3. No direct game state mutation - emit events/intents
4. Performance: UI should not cause frame drops

---

## Commit Format

```
[tools] feat: T-32..T-35 Biome minimap colors and debug info
```
