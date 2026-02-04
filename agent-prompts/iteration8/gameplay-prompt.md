# Gameplay Agent — Iteration 8 Prompt

## Context

You are the **Gameplay Agent** for Project Genesis, a 2D top-down game engine built with Rust.

**Current State:**
- Grass interaction system (G-29)
- Weather state system with Clear/Cloudy/Rain/Storm (G-30)
- Time/day cycle system (G-31)
- Plant growth system (G-32)
- BiomeManager exists in genesis-kernel with Forest, Desert, Ocean, Cave biomes

**Iteration 8 Focus:** Expand biome system with proper terrain generation and resource distribution.

---

## Assigned Tasks

### G-33: Biome type definitions (P0)

**Goal:** Define comprehensive biome types with properties.

**Implementation:**
1. Create `crates/genesis-gameplay/src/biome.rs` with:
   - BiomeType enum: Forest, Desert, Lake, Plains, Mountain, Swamp
   - BiomeProperties struct: temperature, humidity, vegetation_density
   - Resource spawn rates per biome
   - Terrain features per biome (trees, rocks, water)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Forest,
    Desert,
    Lake,
    Plains,
    Mountain,
    Swamp,
}

pub struct BiomeProperties {
    pub temperature: f32,      // -1.0 (cold) to 1.0 (hot)
    pub humidity: f32,         // 0.0 (dry) to 1.0 (wet)
    pub vegetation_density: f32, // 0.0 to 1.0
    pub elevation_min: f32,
    pub elevation_max: f32,
}
```

---

### G-34: Terrain generation logic (P0)

**Goal:** Procedural biome assignment using noise.

**Implementation:**
1. Create `TerrainGenerator` struct in biome.rs
2. Use 2D noise for temperature and humidity
3. Map temperature/humidity to biome type:
   - Low temp + high humidity = Swamp
   - Low temp + low humidity = Mountain (elevation-based)
   - High temp + low humidity = Desert
   - High temp + high humidity = Forest
   - Mid values = Plains
   - Very low elevation = Lake
4. Support world seed for deterministic generation

```rust
pub struct TerrainGenerator {
    seed: u64,
    temperature_noise: SimplexNoise,
    humidity_noise: SimplexNoise,
    elevation_noise: SimplexNoise,
}

impl TerrainGenerator {
    pub fn get_biome_at(&self, world_x: i32, world_y: i32) -> BiomeType;
    pub fn get_elevation_at(&self, world_x: i32, world_y: i32) -> f32;
}
```

---

### G-35: Biome resource distribution (P0)

**Goal:** Spawn biome-appropriate resources and features.

**Implementation:**
1. Define resource types: Tree, Cactus, Rock, Bush, Reed, Fish
2. Spawn rules per biome:
   - Forest: Trees (high), Bushes (medium), Rocks (low)
   - Desert: Cacti (medium), Rocks (high), no trees
   - Lake: Fish (medium), Reeds (low on edges)
   - Plains: Grass (high), occasional trees
   - Mountain: Rocks (high), Snow (above elevation)
   - Swamp: Reeds (high), dead trees (medium)
3. Use noise for natural clustering

---

### G-36: Biome-specific cell types (P1)

**Goal:** Define cell material variants per biome.

**Implementation:**
1. Extend Cell material_id to include biome variants:
   - Forest grass vs Plains grass (different green shades)
   - Desert sand vs Beach sand
   - Mountain stone vs Cave stone
2. Update WorldGenerator to use biome-specific materials

---

## Constraints

1. No direct GPU access - use genesis-kernel APIs
2. Deterministic generation from seed
3. Smooth transitions at biome borders
4. Performance: Generation must complete in < 10ms per chunk

---

## Commit Format

```
[gameplay] feat: G-33..G-36 Biome terrain generation and resource distribution
```
