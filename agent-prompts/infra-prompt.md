# Iteration 7: Infra Agent Tasks

## Context
You are the Infra Agent responsible for engine integration, build systems, and cross-cutting concerns.
The game now has egui UI rendering. This iteration wires up multi-chunk rendering and environment systems.

## Tasks

### I-24: Wire ChunkManager into Render Loop (P0)
**Goal:** Enable multi-chunk rendering in the main engine loop.

**Location:** `crates/genesis-engine/src/renderer.rs` + `app.rs`

**Requirements:**
1. Enable multi_chunk mode in Renderer
2. Create chunks around player spawn position
3. Update camera position to ChunkManager each frame
4. Load/unload chunks as player moves
5. Render all visible chunks

### I-25: Connect Weather/Time to Kernel (P0)
**Goal:** Pass environment state from gameplay to kernel shaders.

**Location:** `crates/genesis-engine/src/app.rs` + `renderer.rs`

**Requirements:**
1. Add WeatherSystem and GameTime to GenesisApp
2. Update them each frame
3. Pass time_of_day and rain_active to render params
4. Kernel uses these for lighting and simulation

### I-26: Wire UI Systems to App (P0)
**Goal:** Connect new UI components to app rendering.

**Location:** `crates/genesis-engine/src/app.rs`

**Requirements:**
1. Add InventoryPanel, StatsHud, EnvironmentHud, Minimap
2. Handle Tab key for inventory toggle
3. Pass player stats to StatsHud
4. Pass time/weather to EnvironmentHud
5. Collect chunk info for Minimap

### I-27: Performance Profiling for Multi-Chunk (P1)
**Goal:** Add performance metrics for chunk system.

**Location:** `crates/genesis-engine/src/timing.rs`

**Requirements:**
1. Track chunk load/unload times
2. Track GPU simulation time per chunk
3. Display in debug overlay (F3)
4. Warn if frame time exceeds budget

## Integration Checklist
- [ ] ChunkManager creates chunks around player
- [ ] Chunks load/unload as player moves
- [ ] Weather affects kernel simulation
- [ ] Time of day changes lighting
- [ ] All UI elements visible
- [ ] Debug overlay shows chunk metrics

## Files to Modify
- crates/genesis-engine/src/app.rs
- crates/genesis-engine/src/renderer.rs
- crates/genesis-engine/src/timing.rs

## Commit Format: [infra] feat: I-XX description
