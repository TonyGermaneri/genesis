# Infra Agent — Iteration 8 Prompt

## Context

You are the **Infra Agent** for Project Genesis, a 2D top-down game engine built with Rust.

**Current State:**
- ChunkManager in render loop (I-24)
- Weather/time passed to kernel (I-25)
- UI systems wired to app (I-26)
- Multi-chunk performance profiling (I-27)
- App struct in genesis-engine handles game loop

**Iteration 8 Focus:** Wire biome generation into chunk creation and manage world seed.

---

## Assigned Tasks

### I-28: Wire biome generation (P0)

**Goal:** Call terrain generator when chunks are created.

**Implementation:**
1. In ChunkManager or wherever chunks are created:
   - Get TerrainGenerator from gameplay
   - On new chunk, call terrain_gen.generate_chunk(chunk_x, chunk_y)
   - Store biome data with chunk
2. Ensure generation happens before render

```rust
// In chunk creation
let biome_data = terrain_generator.generate_chunk(chunk_x, chunk_y, chunk_size);
chunk.set_biome_data(biome_data);
```

---

### I-29: World seed management (P0)

**Goal:** Centralized seed storage and propagation.

**Implementation:**
1. Add `world_seed: u64` to App or Config
2. On startup:
   - Load seed from config file if exists
   - Generate random seed if not
3. Pass seed to:
   - TerrainGenerator
   - BiomeManager
   - WorldGenerator
4. Support seed change at runtime (triggers world regeneration)

```rust
pub struct WorldConfig {
    pub seed: u64,
    pub chunk_size: u32,
    pub render_distance: u32,
}
```

---

### I-30: Chunk biome data flow (P0)

**Goal:** Pass biome information from gameplay to kernel for rendering.

**Implementation:**
1. Add biome_id to Cell struct or separate biome buffer
2. Flow: TerrainGenerator -> ChunkManager -> RenderPipeline
3. Update render params to include biome data pointer
4. Ensure biome data updates when player moves to new area

---

### I-31: Biome generation profiling (P1)

**Goal:** Measure and report biome generation performance.

**Implementation:**
1. Add timers around terrain generation
2. Report in debug panel:
   - Avg generation time per chunk
   - Total chunks generated this session
   - Peak generation time
3. Warn if generation exceeds 16ms

---

## Constraints

1. Minimal coupling between crates
2. Thread-safe if generation is async
3. Deterministic: same seed = same world
4. No memory leaks on chunk unload

---

## Commit Format

```
[infra] feat: I-28..I-31 Biome generation wiring and seed management
```
