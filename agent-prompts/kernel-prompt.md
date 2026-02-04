# Iteration 7: Kernel Agent Tasks

## Context
You are the Kernel Agent responsible for GPU compute, rendering, and low-level systems.
The game now has working top-down movement with static terrain and egui UI.
This iteration adds multi-chunk streaming and environment simulation.

## Tasks

### K-28: Multi-Chunk Streaming Render (P0)
**Goal:** Render multiple chunks around the player, streaming new chunks as player moves.

**Location:** `crates/genesis-kernel/src/render.rs` + `chunk_manager.rs`

**Requirements:**
1. Modify render shader to support multiple chunk buffers
2. Create a `ChunkRenderManager` that:
   - Tracks which chunks are visible based on camera viewport
   - Uploads dirty chunks to GPU
   - Renders all visible chunks in correct world positions
3. Shader needs to accept chunk offset uniforms for world positioning
4. Support render distance of at least 3x3 chunks (9 total)

### K-29: Quadtree Chunk Activation (P0)
**Goal:** Use quadtree to efficiently determine which chunks need simulation.

**Location:** `crates/genesis-kernel/src/quadtree.rs` + `chunk_manager.rs`

**Requirements:**
1. Create `ChunkActivationTree` using existing Quadtree
2. Only chunks in player's "active radius" run GPU simulation
3. Chunks outside active radius are frozen (no compute dispatch)
4. Active radius configurable (default: 2 chunks from player)
5. Track chunk state: Dormant, Active, Simulating

### K-30: Environment Simulation Shader (P1)
**Goal:** Add grass growth, weather effects to compute shader.

**Location:** `crates/genesis-kernel/src/compute.rs`

**Requirements:**
1. Grass lifecycle: growth stage 0-255, spreads to dirt, dies without light/water
2. Rain effect: when rain_active, water cells spawn, hydrates nearby
3. Add EnvParams uniform with time_of_day, rain_active, growth_rate

### K-31: Day/Night Cycle Rendering (P1)
**Goal:** Visual day/night cycle with lighting changes.

**Location:** `crates/genesis-kernel/src/render.rs`

**Requirements:**
1. Add time_of_day to render params (0.0-1.0)
2. Modulate ambient light: Dawn orange, Day bright, Dusk purple, Night blue
3. Grass color varies by growth stage
4. Water reflects sky color

## Files to Modify
- crates/genesis-kernel/src/render.rs
- crates/genesis-kernel/src/compute.rs
- crates/genesis-kernel/src/chunk_manager.rs
- crates/genesis-kernel/src/lib.rs

## Commit Format: [kernel] feat: K-XX description
