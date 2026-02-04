# Iteration 7: Tools Agent Tasks

## Context
You are the Tools Agent responsible for UI, debug tools, and developer experience.
The game now has basic egui integration with debug panel and hotbar.
This iteration expands UI with inventory, player stats, and environmental HUD.

## Tasks

### T-28: Inventory Panel UI (P0)
**Goal:** Full inventory panel with grid display and item management.

**Location:** `crates/genesis-tools/src/ui/inventory.rs` (new file)

**Requirements:**
1. InventoryPanel egui widget: 6x9 grid (54 slots)
2. Slot shows item icon (colored square) + count
3. Toggle with Tab key
4. Panel centered on screen with close button

### T-29: Player Stats HUD (P0)
**Goal:** Display player health, hunger, and status effects.

**Location:** `crates/genesis-tools/src/ui/stats.rs` (new file)

**Requirements:**
1. StatsHud positioned top-left (below debug):
   - Health bar (red)
   - Hunger bar (orange)
   - Stamina bar (green)
2. Bars animate smoothly
3. Low values flash as warning

### T-30: Weather/Time HUD (P0)
**Goal:** Display current time and weather conditions.

**Location:** `crates/genesis-tools/src/ui/environment.rs` (new file)

**Requirements:**
1. EnvironmentHud top-right:
   - Clock (HH:MM), Day counter
   - Weather icon (sun/cloud/rain/storm)
2. Background tint matches time of day

### T-31: Minimap with Chunk Visibility (P1)
**Goal:** Small minimap showing explored area and player position.

**Location:** `crates/genesis-tools/src/ui/minimap.rs` (new file)

**Requirements:**
1. Minimap bottom-right (above hotbar)
2. Shows 5x5 chunk grid around player
3. Player position as white dot
4. Chunk colors based on terrain type

## UI Layout
```
[Debug]              [Time/Weather]
FPS: 60              12:30 Day 1
[Stats]              [Minimap]
Health ████          ▪▪▪▪▪
Hunger ████          ▪▪●▪▪
Stamina ████         ▪▪▪▪▪

        [Game World]

    [1][2][3][4][5][6][7][8][9][0]
              Hotbar
```

## Files to Create/Modify
- crates/genesis-tools/src/ui/inventory.rs (new)
- crates/genesis-tools/src/ui/stats.rs (new)
- crates/genesis-tools/src/ui/environment.rs (new)
- crates/genesis-tools/src/ui/minimap.rs (new)
- crates/genesis-tools/src/ui/mod.rs (new)
- crates/genesis-tools/src/lib.rs

## Commit Format: [tools] feat: T-XX description
