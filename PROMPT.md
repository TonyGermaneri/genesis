# PROMPT — Kernel Agent — Iteration 3

> **Branch**: `kernel-agent`
> **Focus**: Chunk streaming, collision queries, biome material generation

## Your Mission

Complete the following tasks. Work through them sequentially. After each task, run the validation loop. Commit after each task passes.

---

## Tasks

### K-12: Chunk Streaming System (P0)
**File**: `crates/genesis-kernel/src/streaming.rs`

Implement chunk streaming based on camera/player position:

```rust
pub struct ChunkStreamer {
    load_radius: u32,      // chunks to keep loaded
    unload_radius: u32,    // chunks to unload when outside
    pending_loads: VecDeque<ChunkId>,
    pending_unloads: VecDeque<ChunkId>,
}

impl ChunkStreamer {
    pub fn new(load_radius: u32, unload_radius: u32) -> Self;
    pub fn update(&mut self, center: WorldCoord, manager: &mut ChunkManager);
    pub fn get_pending_loads(&self) -> &[ChunkId];
    pub fn get_pending_unloads(&self) -> &[ChunkId];
}
```

Requirements:
- Stream chunks in spiral pattern from center
- Prioritize chunks in view frustum
- Track loading/unloading state
- Budget: max 2 chunk loads per frame

### K-13: Collision Query System (P0)
**File**: `crates/genesis-kernel/src/collision.rs`

Implement collision queries for gameplay use:

```rust
pub struct CollisionQuery {
    buffer: Arc<CellBuffer>,
}

impl CollisionQuery {
    pub fn is_solid(&self, coord: WorldCoord) -> bool;
    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_dist: f32) -> Option<RayHit>;
    pub fn box_query(&self, min: WorldCoord, max: WorldCoord) -> Vec<WorldCoord>;
    pub fn find_ground(&self, x: i32, start_y: i32) -> Option<i32>;
}

pub struct RayHit {
    pub coord: WorldCoord,
    pub distance: f32,
    pub normal: Vec2,
}
```

Requirements:
- Read from cell buffer (GPU readback or CPU shadow)
- Bresenham line algorithm for raycast
- Solid = material flag check
- Used by gameplay for player physics

### K-14: Biome Material Assignment (P1)
**File**: `crates/genesis-kernel/src/biome.rs`

Implement biome-based material generation:

```rust
pub struct BiomeConfig {
    pub id: BiomeId,
    pub name: String,
    pub surface_material: MaterialId,
    pub subsurface_material: MaterialId,
    pub deep_material: MaterialId,
    pub surface_depth: u32,
    pub subsurface_depth: u32,
}

pub struct BiomeManager {
    biomes: HashMap<BiomeId, BiomeConfig>,
    noise: SimplexNoise,
}

impl BiomeManager {
    pub fn register_biome(&mut self, config: BiomeConfig);
    pub fn get_biome_at(&self, coord: WorldCoord) -> BiomeId;
    pub fn get_material_at(&self, coord: WorldCoord, depth: u32) -> MaterialId;
}
```

Requirements:
- At least 3 biomes: Forest, Desert, Cave
- Simplex noise for biome boundaries
- Smooth transitions at edges
- Depth-based material layers

### K-15: GPU Readback Optimization (P1)
**File**: `crates/genesis-kernel/src/readback.rs`

Optimize GPU→CPU data transfer:

```rust
pub struct ReadbackManager {
    staging_buffer: wgpu::Buffer,
    pending_reads: Vec<PendingRead>,
}

pub struct PendingRead {
    pub chunk_id: ChunkId,
    pub frame_submitted: u64,
}

impl ReadbackManager {
    pub fn request_readback(&mut self, chunk_id: ChunkId);
    pub fn poll_readbacks(&mut self, device: &wgpu::Device) -> Vec<(ChunkId, Vec<Cell>)>;
    pub fn is_pending(&self, chunk_id: ChunkId) -> bool;
}
```

Requirements:
- Double-buffered staging for async
- Track which chunks have pending reads
- Coalesce nearby chunk reads
- Timeout handling for stalled reads

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
[kernel] feat: K-12 chunk streaming system
[kernel] feat: K-13 collision query system
[kernel] feat: K-14 biome material assignment
[kernel] feat: K-15 GPU readback optimization
```

---

## Integration Notes

- K-13 collision will be used by gameplay-agent's player controller
- K-14 biome will be used by world generation
- Coordinate with genesis-common types
- Export new modules in lib.rs
