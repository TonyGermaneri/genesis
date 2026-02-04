# PROMPT — Kernel Agent — Iteration 6

> **Branch**: `kernel-agent`
> **Focus**: Quadtree spatial partitioning, multi-chunk visible area rendering, collision detection, top-down physics

## Your Mission

Implement high-performance spatial systems for the pixel simulation. The current system only renders a single 256x256 chunk - we need to render the visible area efficiently using quadtree optimization.

---

## Tasks

### K-24: Quadtree Spatial Partitioning (P0)
**File**: `crates/genesis-kernel/src/quadtree.rs`

Implement a quadtree for efficient spatial queries and simulation optimization:

```rust
/// Quadtree node for spatial partitioning
pub struct QuadTree<T> {
    bounds: Rect,
    max_objects: usize,
    max_levels: usize,
    level: usize,
    objects: Vec<(Rect, T)>,
    children: Option<Box<[QuadTree<T>; 4]>>,
}

impl<T> QuadTree<T> {
    pub fn new(bounds: Rect, max_objects: usize, max_levels: usize) -> Self;
    
    /// Insert an object with its bounding rect
    pub fn insert(&mut self, bounds: Rect, object: T) -> bool;
    
    /// Query all objects that intersect with the given rect
    pub fn query(&self, range: Rect) -> Vec<&T>;
    
    /// Query objects with their bounds
    pub fn query_with_bounds(&self, range: Rect) -> Vec<(&Rect, &T)>;
    
    /// Clear all objects
    pub fn clear(&mut self);
    
    /// Get statistics (node count, object count, depth)
    pub fn stats(&self) -> QuadTreeStats;
}

/// Simple rectangle for bounds
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn contains_point(&self, x: f32, y: f32) -> bool;
    pub fn intersects(&self, other: &Rect) -> bool;
    pub fn contains(&self, other: &Rect) -> bool;
}
```

**Tests**:
- Insert 10,000 objects, query visible area < 1ms
- Proper subdivision when max_objects exceeded

---

### K-25: Multi-Chunk Visible Area Rendering (P0)
**File**: `crates/genesis-kernel/src/chunk_manager.rs`

Implement a chunk manager that loads/unloads chunks based on camera position:

```rust
/// Manages multiple chunks for visible area rendering
pub struct ChunkManager {
    chunks: HashMap<(i32, i32), Chunk>,
    chunk_size: u32,
    render_distance: u32,  // How many chunks to keep loaded around camera
    center_chunk: (i32, i32),
}

impl ChunkManager {
    pub fn new(chunk_size: u32, render_distance: u32) -> Self;
    
    /// Update which chunks are loaded based on camera position
    pub fn update_visible(&mut self, camera: &Camera, world_gen: &dyn WorldGenerator);
    
    /// Get a cell at world coordinates (across chunk boundaries)
    pub fn get_cell(&self, world_x: i32, world_y: i32) -> Option<&Cell>;
    
    /// Set a cell at world coordinates
    pub fn set_cell(&mut self, world_x: i32, world_y: i32, cell: Cell);
    
    /// Get all visible chunks for rendering
    pub fn visible_chunks(&self) -> impl Iterator<Item = &Chunk>;
    
    /// Convert world coords to chunk coords
    pub fn world_to_chunk(x: i32, y: i32, chunk_size: u32) -> (i32, i32);
}

/// A single chunk of the world
pub struct Chunk {
    pub position: (i32, i32),  // Chunk coordinates
    pub cells: Vec<Cell>,
    pub dirty: bool,  // Needs GPU upload
}
```

**Integration**: Update `render.rs` to render all visible chunks, not just one.

---

### K-26: Player-Terrain Collision Detection (P0)
**File**: `crates/genesis-kernel/src/collision.rs` (extend existing)

Add player collision detection against terrain cells:

```rust
/// Collision result from terrain check
#[derive(Debug, Clone)]
pub struct TerrainCollision {
    pub collided: bool,
    pub normal: (f32, f32),      // Push direction
    pub penetration: f32,        // How far into solid
    pub cell_type: Option<u16>,  // What material was hit
}

/// Check collision between a point/circle and terrain
pub fn check_terrain_collision(
    chunk_manager: &ChunkManager,
    position: (f32, f32),
    radius: f32,
) -> TerrainCollision;

/// Check collision for a moving object (sweep test)
pub fn sweep_terrain_collision(
    chunk_manager: &ChunkManager,
    start: (f32, f32),
    end: (f32, f32),
    radius: f32,
) -> (TerrainCollision, (f32, f32));  // Returns collision and safe position

/// Resolve collision by pushing entity out of terrain
pub fn resolve_collision(
    position: (f32, f32),
    collision: &TerrainCollision,
) -> (f32, f32);
```

---

### K-27: Top-Down Physics Model (P1)
**File**: `crates/genesis-kernel/src/topdown_physics.rs`

Replace platformer gravity with top-down friction-based physics:

```rust
/// Physics configuration for top-down movement
#[derive(Debug, Clone)]
pub struct TopDownPhysicsConfig {
    pub friction: f32,           // Ground friction (0.9 = slippery, 0.5 = grippy)
    pub water_friction: f32,     // Higher friction in water
    pub sand_friction: f32,      // Sand slows you down
    pub acceleration: f32,       // How fast to reach target velocity
    pub max_speed: f32,          // Maximum movement speed
}

impl Default for TopDownPhysicsConfig {
    fn default() -> Self {
        Self {
            friction: 0.85,
            water_friction: 0.95,
            sand_friction: 0.7,
            acceleration: 800.0,
            max_speed: 200.0,
        }
    }
}

/// Apply top-down physics to velocity
pub fn apply_topdown_physics(
    velocity: &mut (f32, f32),
    input_direction: (f32, f32),
    terrain_type: u16,
    config: &TopDownPhysicsConfig,
    dt: f32,
);

/// Get friction multiplier for terrain type
pub fn terrain_friction(material: u16, config: &TopDownPhysicsConfig) -> f32;
```

The feel should be:
- Smooth acceleration when pressing movement
- Gradual slowdown when releasing (not instant stop)
- Different terrain affects speed (water = slow, sand = slower)
- No gravity pulling down - this is top-down view

---

## Validation

After each task:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p genesis-kernel
```

## Commit Format
```
[kernel] feat: K-XX description
```

## Done Criteria
- [ ] Quadtree with O(log n) spatial queries
- [ ] Multi-chunk rendering with seamless boundaries
- [ ] Player collision stops at solid terrain
- [ ] Top-down physics feels smooth and responsive
