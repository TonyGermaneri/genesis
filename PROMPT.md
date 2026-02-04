# PROMPT — Kernel Agent — Iteration 4

> **Branch**: `kernel-agent`
> **Focus**: World generation, lighting system, particle effects, audio integration

## Your Mission

Complete the following tasks. Work through them sequentially. After each task, run the validation loop. Commit after each task passes.

---

## Tasks

### K-16: Procedural World Generation (P0)
**File**: `crates/genesis-kernel/src/worldgen.rs`

Implement procedural terrain generation:

```rust
use crate::biome::{BiomeManager, BiomeId};
use crate::cell::{Cell, MaterialId};

pub struct WorldGenerator {
    seed: u64,
    noise: FastNoiseLite,
    biome_manager: BiomeManager,
}

pub struct GenerationParams {
    pub sea_level: i32,
    pub terrain_scale: f32,
    pub cave_threshold: f32,
    pub ore_frequency: f32,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self;

    pub fn generate_chunk(&self, chunk_x: i32, chunk_y: i32, params: &GenerationParams) -> Vec<Cell>;

    fn generate_terrain_height(&self, world_x: i32) -> i32;
    fn generate_cave_mask(&self, world_x: i32, world_y: i32) -> bool;
    fn place_ores(&self, cells: &mut [Cell], chunk_x: i32, chunk_y: i32);
    fn place_vegetation(&self, cells: &mut [Cell], chunk_x: i32, chunk_y: i32);
}
```

Requirements:
- Multi-octave noise for terrain height
- Cave systems using 3D noise threshold
- Ore veins with clustering
- Surface vegetation (grass, trees as cell patterns)
- Deterministic from seed

### K-17: Dynamic Lighting System (P0)
**File**: `crates/genesis-kernel/src/lighting.rs`

Implement GPU-accelerated lighting:

```rust
pub struct LightingSystem {
    light_buffer: wgpu::Buffer,
    light_map: wgpu::Texture,
    compute_pipeline: wgpu::ComputePipeline,
}

pub struct Light {
    pub position: (f32, f32),
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub light_type: LightType,
}

pub enum LightType {
    Point,
    Directional,
    Ambient,
}

impl LightingSystem {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self;

    pub fn add_light(&mut self, light: Light) -> LightId;
    pub fn remove_light(&mut self, id: LightId);
    pub fn update_light(&mut self, id: LightId, light: Light);

    pub fn compute_lighting(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        cell_buffer: &wgpu::Buffer,
    );

    pub fn get_light_map(&self) -> &wgpu::Texture;
}
```

Requirements:
- Compute shader for light propagation
- Light blocked by solid cells
- Day/night cycle via ambient light
- Smooth light falloff
- Max 256 dynamic lights

### K-18: Particle System (P1)
**File**: `crates/genesis-kernel/src/particles.rs`

Implement GPU particle effects:

```rust
pub struct ParticleSystem {
    particle_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    max_particles: u32,
}

pub struct ParticleEmitter {
    pub position: (f32, f32),
    pub emission_rate: f32,
    pub particle_lifetime: f32,
    pub velocity_range: ((f32, f32), (f32, f32)),
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub size_range: (f32, f32),
    pub gravity: f32,
}

pub enum ParticleEffect {
    Explosion { position: (f32, f32), intensity: f32 },
    Dust { position: (f32, f32), direction: (f32, f32) },
    Fire { position: (f32, f32), size: f32 },
    Water { position: (f32, f32), velocity: (f32, f32) },
    Sparks { position: (f32, f32), count: u32 },
}

impl ParticleSystem {
    pub fn new(device: &wgpu::Device, max_particles: u32) -> Self;

    pub fn spawn_effect(&mut self, effect: ParticleEffect);
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) -> EmitterId;
    pub fn remove_emitter(&mut self, id: EmitterId);

    pub fn update(&mut self, encoder: &mut wgpu::CommandEncoder, dt: f32);
    pub fn render(&self, render_pass: &mut wgpu::RenderPass);
}
```

Requirements:
- GPU compute for particle physics
- Instanced rendering for particles
- Particle pooling (reuse dead particles)
- Common effect presets
- Collision with cells (optional)

### K-19: Audio Spatial Integration (P1)
**File**: `crates/genesis-kernel/src/audio.rs`

Prepare spatial audio data:

```rust
pub struct AudioSpatialData {
    pub listener_position: (f32, f32),
    pub listener_velocity: (f32, f32),
    pub environment: AudioEnvironment,
}

pub struct AudioSource {
    pub id: AudioSourceId,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub volume: f32,
    pub pitch: f32,
    pub loop_: bool,
    pub attenuation: AttenuationModel,
}

pub enum AttenuationModel {
    Linear { min_dist: f32, max_dist: f32 },
    Inverse { ref_dist: f32, rolloff: f32 },
    Exponential { ref_dist: f32, rolloff: f32 },
}

pub struct AudioEnvironment {
    pub reverb: f32,
    pub dampening: f32,
    pub underground: bool,
}

pub struct SpatialAudioManager {
    sources: HashMap<AudioSourceId, AudioSource>,
    listener: AudioSpatialData,
}

impl SpatialAudioManager {
    pub fn new() -> Self;

    pub fn update_listener(&mut self, position: (f32, f32), velocity: (f32, f32));
    pub fn add_source(&mut self, source: AudioSource) -> AudioSourceId;
    pub fn remove_source(&mut self, id: AudioSourceId);

    pub fn calculate_gains(&self) -> Vec<(AudioSourceId, f32, f32)>; // id, left, right
    pub fn get_environment_at(&self, position: (f32, f32)) -> AudioEnvironment;
}
```

Requirements:
- Position-based stereo panning
- Distance attenuation
- Environment detection (cave reverb)
- Doppler effect preparation
- Data only - actual audio in tools crate

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
[kernel] feat: K-16 procedural world generation
[kernel] feat: K-17 dynamic lighting system
[kernel] feat: K-18 particle system
[kernel] feat: K-19 audio spatial integration
```

---

## Integration Notes

- K-16 uses BiomeManager from previous iteration
- K-17 lighting affects cell rendering
- K-18 particles render on top of cells
- K-19 provides data for tools-agent audio playback
- Export new modules in lib.rs
