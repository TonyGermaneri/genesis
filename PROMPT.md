# Kernel Agent — Iteration 8 Prompt

## Context

You are the **Kernel Agent** for Project Genesis, a 2D top-down game engine built with Rust/wgpu.

**Current State:**
- Multi-chunk streaming render is working (K-28)
- Quadtree chunk activation for simulation (K-29)
- Environment simulation shader for grass/rain (K-30)
- Day/night cycle rendering (K-31)
- Biome system exists in biome.rs with SimplexNoise, BiomeManager, BiomeConfig
- WorldGenerator uses biomes for material selection

**Iteration 8 Focus:** Enhance biome rendering with visual distinction and smooth transitions.

---

## Assigned Tasks

### K-32: Biome-aware cell coloring (P0)

**Goal:** Modify the render shader to use biome-specific color palettes.

**Implementation:**
1. Add biome_id field to RenderParams or compute from noise in shader
2. Create color palettes for each biome:
   - Forest: Lush greens (grass #4a7c23, dirt #8b6914)
   - Desert: Warm yellows/oranges (sand #c2a655, sandstone #b8956e)
   - Lake/Ocean: Blues (water #3a7ca5, deep #1e4d6b)
   - Plains: Light greens/yellows (grass #7cb342, dirt #a08060)
   - Mountain: Grays/whites (stone #7a7a7a, snow #e8e8e8)
3. Use material_id AND biome_id to determine final color

**Files to modify:**
- crates/genesis-kernel/src/render.rs
- Inline WGSL shader code

---

### K-33: Biome transition blending (P0)

**Goal:** Smooth visual transitions between adjacent biomes.

**Implementation:**
1. Sample biome at neighboring cells in shader
2. Apply gradient blending using noise-based weights
3. Blend over 3-5 cells for natural transition
4. Use dithering or noise for organic boundary appearance

---

### K-34: Lake/water rendering (P0)

**Goal:** Add animated water shader for lake biomes.

**Implementation:**
1. Detect water material cells in render shader (material_id == 4)
2. Add wave animation using time uniform and sine functions
3. Add subtle color variation based on depth
4. Water should have slight transparency (alpha < 1.0)

---

### K-35: Mountain/elevation rendering (P1)

**Goal:** Add elevation-based rendering for mountain biomes.

**Implementation:**
1. Use noise to generate elevation values
2. Higher elevations get snow-capped appearance
3. Add shadow/highlight based on light direction

---

## Constraints

1. Performance: Biome calculations must not exceed 1ms per chunk
2. GPU-friendly: Use uniforms, not per-cell CPU computation
3. No gameplay logic: Only rendering
4. Existing APIs: Use existing SimplexNoise and BiomeManager
5. Backward compatible: Existing cell rendering must still work

---

## Commit Format

```
[kernel] feat: K-32..K-35 Biome rendering with transitions and water animation
```
