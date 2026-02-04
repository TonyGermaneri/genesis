//! Particle system with GPU-accelerated physics.
//!
//! This module provides a compute-based particle system that simulates particle
//! physics on the GPU. Supports multiple emitter types and particle effects.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use tracing::debug;
use wgpu::{util::DeviceExt, Device, Queue};

/// Maximum number of particles supported.
pub const MAX_PARTICLES: usize = 65536;

/// Maximum number of emitters supported.
pub const MAX_EMITTERS: usize = 256;

/// Unique identifier for a particle emitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmitterId(u32);

impl EmitterId {
    /// Creates a new emitter ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Type of particle effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum ParticleEffect {
    /// Generic particles with gravity.
    #[default]
    Generic = 0,
    /// Fire/flame particles (rise upward).
    Fire = 1,
    /// Smoke particles (slow rise, fade).
    Smoke = 2,
    /// Water/rain particles (fall with splash).
    Water = 3,
    /// Spark/electric particles (fast, bright).
    Spark = 4,
    /// Dust particles (drift slowly).
    Dust = 5,
    /// Explosion debris.
    Explosion = 6,
}

impl ParticleEffect {
    /// Converts from raw u32 value.
    #[must_use]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Fire,
            2 => Self::Smoke,
            3 => Self::Water,
            4 => Self::Spark,
            5 => Self::Dust,
            6 => Self::Explosion,
            _ => Self::Generic,
        }
    }

    /// Returns default color for this effect type.
    #[must_use]
    pub const fn default_color(&self) -> [f32; 4] {
        match self {
            Self::Generic => [1.0, 1.0, 1.0, 1.0],
            Self::Fire => [1.0, 0.5, 0.1, 0.9],
            Self::Smoke => [0.3, 0.3, 0.3, 0.5],
            Self::Water => [0.2, 0.4, 0.8, 0.7],
            Self::Spark => [1.0, 1.0, 0.5, 1.0],
            Self::Dust => [0.6, 0.5, 0.4, 0.3],
            Self::Explosion => [1.0, 0.3, 0.0, 1.0],
        }
    }

    /// Returns default lifetime for this effect type.
    #[must_use]
    pub const fn default_lifetime(&self) -> f32 {
        match self {
            Self::Generic => 2.0,
            Self::Fire => 1.5,
            Self::Smoke => 3.0,
            Self::Water => 1.0,
            Self::Spark => 0.5,
            Self::Dust => 4.0,
            Self::Explosion => 0.8,
        }
    }

    /// Returns default gravity modifier for this effect type.
    #[must_use]
    pub const fn default_gravity(&self) -> f32 {
        match self {
            Self::Generic => 1.0,
            Self::Fire => -0.5,  // Rise upward
            Self::Smoke => -0.2, // Slow rise
            Self::Water => 1.5,  // Fall faster
            Self::Spark => 0.3,  // Light
            Self::Dust => 0.1,   // Very light
            Self::Explosion => 0.8,
        }
    }
}

/// A particle emitter configuration.
#[derive(Debug, Clone, Copy)]
pub struct ParticleEmitter {
    /// Position in world coordinates.
    pub position: (f32, f32),
    /// Emission rate (particles per second).
    pub emission_rate: f32,
    /// Particle velocity range (min, max).
    pub velocity_range: ((f32, f32), (f32, f32)),
    /// Particle size range (min, max).
    pub size_range: (f32, f32),
    /// Particle effect type.
    pub effect: ParticleEffect,
    /// Base color (RGBA).
    pub color: [f32; 4],
    /// Particle lifetime in seconds.
    pub lifetime: f32,
    /// Gravity modifier.
    pub gravity: f32,
    /// Whether the emitter is active.
    pub enabled: bool,
    /// Emission angle spread in radians (0 = directional, PI = hemisphere).
    pub spread: f32,
    /// Emission direction (normalized).
    pub direction: (f32, f32),
    /// Accumulated time for emission.
    emission_accumulator: f32,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            emission_rate: 10.0,
            velocity_range: ((-10.0, -50.0), (10.0, -20.0)),
            size_range: (2.0, 5.0),
            effect: ParticleEffect::Generic,
            color: [1.0, 1.0, 1.0, 1.0],
            lifetime: 2.0,
            gravity: 1.0,
            enabled: true,
            spread: std::f32::consts::FRAC_PI_4,
            direction: (0.0, -1.0),
            emission_accumulator: 0.0,
        }
    }
}

impl ParticleEmitter {
    /// Creates a fire emitter.
    #[must_use]
    pub fn fire(position: (f32, f32), rate: f32) -> Self {
        let effect = ParticleEffect::Fire;
        Self {
            position,
            emission_rate: rate,
            velocity_range: ((-5.0, -40.0), (5.0, -20.0)),
            size_range: (3.0, 8.0),
            effect,
            color: effect.default_color(),
            lifetime: effect.default_lifetime(),
            gravity: effect.default_gravity(),
            spread: std::f32::consts::FRAC_PI_6,
            direction: (0.0, -1.0),
            ..Default::default()
        }
    }

    /// Creates a smoke emitter.
    #[must_use]
    pub fn smoke(position: (f32, f32), rate: f32) -> Self {
        let effect = ParticleEffect::Smoke;
        Self {
            position,
            emission_rate: rate,
            velocity_range: ((-3.0, -15.0), (3.0, -5.0)),
            size_range: (5.0, 15.0),
            effect,
            color: effect.default_color(),
            lifetime: effect.default_lifetime(),
            gravity: effect.default_gravity(),
            spread: std::f32::consts::FRAC_PI_4,
            direction: (0.0, -1.0),
            ..Default::default()
        }
    }

    /// Creates a water/rain emitter.
    #[must_use]
    pub fn water(position: (f32, f32), rate: f32) -> Self {
        let effect = ParticleEffect::Water;
        Self {
            position,
            emission_rate: rate,
            velocity_range: ((-2.0, 20.0), (2.0, 40.0)),
            size_range: (1.0, 3.0),
            effect,
            color: effect.default_color(),
            lifetime: effect.default_lifetime(),
            gravity: effect.default_gravity(),
            spread: std::f32::consts::FRAC_PI_8,
            direction: (0.0, 1.0),
            ..Default::default()
        }
    }

    /// Creates a spark emitter.
    #[must_use]
    pub fn spark(position: (f32, f32), rate: f32) -> Self {
        let effect = ParticleEffect::Spark;
        Self {
            position,
            emission_rate: rate,
            velocity_range: ((-80.0, -80.0), (80.0, 80.0)),
            size_range: (1.0, 2.0),
            effect,
            color: effect.default_color(),
            lifetime: effect.default_lifetime(),
            gravity: effect.default_gravity(),
            spread: std::f32::consts::PI,
            direction: (0.0, -1.0),
            ..Default::default()
        }
    }

    /// Creates an explosion emitter.
    #[must_use]
    pub fn explosion(position: (f32, f32), count: u32) -> Self {
        let effect = ParticleEffect::Explosion;
        Self {
            position,
            emission_rate: count as f32 * 100.0, // Burst emission
            velocity_range: ((-100.0, -100.0), (100.0, 100.0)),
            size_range: (2.0, 6.0),
            effect,
            color: effect.default_color(),
            lifetime: effect.default_lifetime(),
            gravity: effect.default_gravity(),
            spread: std::f32::consts::PI,
            direction: (0.0, -1.0),
            enabled: true,
            emission_accumulator: 0.0,
        }
    }

    /// Sets the emitter position.
    #[must_use]
    pub const fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Sets the emission rate.
    #[must_use]
    pub const fn with_rate(mut self, rate: f32) -> Self {
        self.emission_rate = rate;
        self
    }

    /// Enables or disables the emitter.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// GPU-compatible particle data structure.
/// Layout: 32 bytes total, aligned for GPU buffers.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GpuParticle {
    /// Position XY (8 bytes).
    pub position: [f32; 2],
    /// Velocity XY (8 bytes).
    pub velocity: [f32; 2],
    /// Color RGBA (16 bytes).
    pub color: [f32; 4],
    /// Size (4 bytes).
    pub size: f32,
    /// Remaining lifetime (4 bytes).
    pub lifetime: f32,
    /// Max lifetime for alpha calculation (4 bytes).
    pub max_lifetime: f32,
    /// Gravity modifier (4 bytes).
    pub gravity: f32,
}

impl Default for GpuParticle {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            size: 2.0,
            lifetime: 0.0,
            max_lifetime: 1.0,
            gravity: 1.0,
        }
    }
}

/// GPU-compatible particle system parameters.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ParticleParams {
    /// Delta time in seconds.
    pub delta_time: f32,
    /// Global gravity strength.
    pub gravity: f32,
    /// Number of active particles.
    pub particle_count: u32,
    /// Wind X component.
    pub wind_x: f32,
    /// Wind Y component.
    pub wind_y: f32,
    /// Padding for alignment.
    padding: [u32; 3],
}

impl Default for ParticleParams {
    fn default() -> Self {
        Self {
            delta_time: 1.0 / 60.0,
            gravity: 98.0, // Pixels per second squared
            particle_count: 0,
            wind_x: 0.0,
            wind_y: 0.0,
            padding: [0, 0, 0],
        }
    }
}

/// Particle instance data for rendering.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ParticleInstance {
    /// Position XY.
    pub position: [f32; 2],
    /// Size.
    pub size: f32,
    /// Color with alpha.
    pub color: [f32; 4],
    /// Padding for alignment.
    padding: f32,
}

/// Particle physics compute shader in WGSL.
pub const PARTICLE_SHADER: &str = r"
// Particle structure (32 bytes)
struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec4<f32>,
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
    gravity: f32,
}

// Parameters
struct Params {
    delta_time: f32,
    gravity: f32,
    particle_count: u32,
    wind_x: f32,
    wind_y: f32,
    _padding: vec3<u32>,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;

    if idx >= params.particle_count {
        return;
    }

    var p = particles[idx];

    // Skip dead particles
    if p.lifetime <= 0.0 {
        return;
    }

    // Apply gravity
    p.velocity.y += params.gravity * p.gravity * params.delta_time;

    // Apply wind
    p.velocity.x += params.wind_x * params.delta_time;
    p.velocity.y += params.wind_y * params.delta_time;

    // Apply drag (slight air resistance)
    p.velocity *= 0.99;

    // Update position
    p.position += p.velocity * params.delta_time;

    // Update lifetime
    p.lifetime -= params.delta_time;

    // Fade alpha based on lifetime
    let life_ratio = p.lifetime / p.max_lifetime;
    p.color.a = p.color.a * life_ratio;

    particles[idx] = p;
}
";

/// GPU-accelerated particle system.
///
/// Simulates particle physics using a compute shader and provides
/// particle instance data for instanced rendering.
pub struct ParticleSystem {
    /// Particle data buffer.
    particle_buffer: wgpu::Buffer,
    /// Particle parameters buffer.
    params_buffer: wgpu::Buffer,
    /// Compute pipeline.
    compute_pipeline: wgpu::ComputePipeline,
    /// Bind group.
    bind_group: wgpu::BindGroup,
    /// Active emitters.
    emitters: HashMap<EmitterId, ParticleEmitter>,
    /// Next emitter ID.
    next_emitter_id: u32,
    /// Particle data (CPU side for emission).
    particles: Vec<GpuParticle>,
    /// Current particle count.
    particle_count: usize,
    /// Current parameters.
    params: ParticleParams,
    /// Whether particles need upload.
    dirty: bool,
    /// Random number generator seed.
    rng_seed: u32,
}

impl ParticleSystem {
    /// Creates a new particle system.
    pub fn new(device: &Device) -> Self {
        // Create particle buffer
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create params buffer
        let params = ParticleParams::default();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // Create compute pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(PARTICLE_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        debug!(
            "Created particle system with max {} particles",
            MAX_PARTICLES
        );

        Self {
            particle_buffer,
            params_buffer,
            compute_pipeline,
            bind_group,
            emitters: HashMap::new(),
            next_emitter_id: 0,
            particles: vec![GpuParticle::default(); MAX_PARTICLES],
            particle_count: 0,
            params,
            dirty: false,
            rng_seed: 12345,
        }
    }

    /// Adds an emitter to the system.
    ///
    /// Returns a unique ID for the emitter, or `None` if at capacity.
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) -> Option<EmitterId> {
        if self.emitters.len() >= MAX_EMITTERS {
            return None;
        }

        let id = EmitterId::new(self.next_emitter_id);
        self.next_emitter_id += 1;
        self.emitters.insert(id, emitter);

        debug!("Added emitter {:?}", id);
        Some(id)
    }

    /// Removes an emitter from the system.
    pub fn remove_emitter(&mut self, id: EmitterId) -> Option<ParticleEmitter> {
        let removed = self.emitters.remove(&id);
        if removed.is_some() {
            debug!("Removed emitter {:?}", id);
        }
        removed
    }

    /// Gets an emitter by ID.
    #[must_use]
    pub fn get_emitter(&self, id: EmitterId) -> Option<&ParticleEmitter> {
        self.emitters.get(&id)
    }

    /// Gets a mutable emitter by ID.
    pub fn get_emitter_mut(&mut self, id: EmitterId) -> Option<&mut ParticleEmitter> {
        self.emitters.get_mut(&id)
    }

    /// Returns the number of active emitters.
    #[must_use]
    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }

    /// Returns the number of active particles.
    #[must_use]
    pub fn particle_count(&self) -> usize {
        self.particle_count
    }

    /// Sets wind direction and strength.
    pub fn set_wind(&mut self, x: f32, y: f32) {
        self.params.wind_x = x;
        self.params.wind_y = y;
    }

    /// Sets global gravity strength.
    pub fn set_gravity(&mut self, gravity: f32) {
        self.params.gravity = gravity;
    }

    /// Simple pseudo-random number generator (uses local seed).
    fn next_random_with_seed(seed: &mut u32) -> f32 {
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        ((*seed >> 16) & 0x7FFF) as f32 / 32768.0
    }

    /// Generates a random value in a range.
    fn random_range_with_seed(seed: &mut u32, min: f32, max: f32) -> f32 {
        min + Self::next_random_with_seed(seed) * (max - min)
    }

    /// Emits particles from all active emitters.
    pub fn emit(&mut self, delta_time: f32) {
        let mut new_particles = Vec::new();
        let mut seed = self.rng_seed;

        for emitter in self.emitters.values_mut() {
            if !emitter.enabled {
                continue;
            }

            emitter.emission_accumulator += delta_time * emitter.emission_rate;

            while emitter.emission_accumulator >= 1.0 {
                emitter.emission_accumulator -= 1.0;

                if self.particle_count + new_particles.len() >= MAX_PARTICLES {
                    break;
                }

                // Calculate velocity with spread
                let angle_offset =
                    Self::random_range_with_seed(&mut seed, -emitter.spread, emitter.spread);
                let base_angle = emitter.direction.1.atan2(emitter.direction.0);
                let angle = base_angle + angle_offset;

                let speed = Self::random_range_with_seed(
                    &mut seed,
                    (emitter.velocity_range.0 .0.powi(2) + emitter.velocity_range.0 .1.powi(2))
                        .sqrt(),
                    (emitter.velocity_range.1 .0.powi(2) + emitter.velocity_range.1 .1.powi(2))
                        .sqrt(),
                );

                let vx = angle.cos() * speed;
                let vy = angle.sin() * speed;

                // Random position offset within a small radius
                let offset_x = Self::random_range_with_seed(&mut seed, -2.0, 2.0);
                let offset_y = Self::random_range_with_seed(&mut seed, -2.0, 2.0);

                let particle = GpuParticle {
                    position: [emitter.position.0 + offset_x, emitter.position.1 + offset_y],
                    velocity: [vx, vy],
                    color: emitter.color,
                    size: Self::random_range_with_seed(
                        &mut seed,
                        emitter.size_range.0,
                        emitter.size_range.1,
                    ),
                    lifetime: emitter.lifetime,
                    max_lifetime: emitter.lifetime,
                    gravity: emitter.gravity,
                };

                new_particles.push(particle);
            }
        }

        // Store updated seed
        self.rng_seed = seed;

        // Add new particles to the system
        for particle in new_particles {
            if self.particle_count < MAX_PARTICLES {
                self.particles[self.particle_count] = particle;
                self.particle_count += 1;
            }
        }

        self.dirty = true;
    }

    /// Spawns a burst of particles at a position.
    pub fn burst(&mut self, position: (f32, f32), effect: ParticleEffect, count: u32) {
        let mut seed = self.rng_seed;

        for _ in 0..count {
            if self.particle_count >= MAX_PARTICLES {
                break;
            }

            // Random direction
            let angle = Self::random_range_with_seed(&mut seed, 0.0, std::f32::consts::TAU);
            let speed = Self::random_range_with_seed(&mut seed, 20.0, 80.0);

            let particle = GpuParticle {
                position: [position.0, position.1],
                velocity: [angle.cos() * speed, angle.sin() * speed],
                color: effect.default_color(),
                size: Self::random_range_with_seed(&mut seed, 2.0, 5.0),
                lifetime: effect.default_lifetime(),
                max_lifetime: effect.default_lifetime(),
                gravity: effect.default_gravity(),
            };

            self.particles[self.particle_count] = particle;
            self.particle_count += 1;
        }

        self.rng_seed = seed;
        self.dirty = true;
    }

    /// Updates and simulates particles.
    pub fn update(&mut self, queue: &Queue, encoder: &mut wgpu::CommandEncoder, delta_time: f32) {
        // Emit new particles
        self.emit(delta_time);

        // Upload particle data if changed
        if self.dirty && self.particle_count > 0 {
            let data = bytemuck::cast_slice(&self.particles[..self.particle_count]);
            queue.write_buffer(&self.particle_buffer, 0, data);
            self.dirty = false;
        }

        // Update params
        self.params.delta_time = delta_time;
        self.params.particle_count = self.particle_count as u32;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));

        // Run compute shader
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Particle Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        let workgroups = (self.particle_count as u32).div_ceil(256);
        compute_pass.dispatch_workgroups(workgroups, 1, 1);
    }

    /// Compacts the particle array by removing dead particles.
    pub fn compact(&mut self) {
        let mut write_idx = 0;

        for read_idx in 0..self.particle_count {
            if self.particles[read_idx].lifetime > 0.0 {
                if write_idx != read_idx {
                    self.particles[write_idx] = self.particles[read_idx];
                }
                write_idx += 1;
            }
        }

        self.particle_count = write_idx;
        self.dirty = true;
    }

    /// Gets particle instances for rendering.
    #[must_use]
    pub fn get_instances(&self) -> Vec<ParticleInstance> {
        self.particles[..self.particle_count]
            .iter()
            .filter(|p| p.lifetime > 0.0)
            .map(|p| ParticleInstance {
                position: p.position,
                size: p.size,
                color: p.color,
                padding: 0.0,
            })
            .collect()
    }

    /// Gets the particle buffer for direct GPU access.
    #[must_use]
    pub fn particle_buffer(&self) -> &wgpu::Buffer {
        &self.particle_buffer
    }

    /// Clears all particles and emitters.
    pub fn clear(&mut self) {
        self.particles.fill(GpuParticle::default());
        self.particle_count = 0;
        self.emitters.clear();
        self.dirty = true;
    }
}

impl std::fmt::Debug for ParticleSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParticleSystem")
            .field("particle_count", &self.particle_count)
            .field("emitter_count", &self.emitters.len())
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_id() {
        let id = EmitterId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_particle_effect_conversion() {
        assert_eq!(ParticleEffect::from_u32(0), ParticleEffect::Generic);
        assert_eq!(ParticleEffect::from_u32(1), ParticleEffect::Fire);
        assert_eq!(ParticleEffect::from_u32(2), ParticleEffect::Smoke);
        assert_eq!(ParticleEffect::from_u32(3), ParticleEffect::Water);
        assert_eq!(ParticleEffect::from_u32(4), ParticleEffect::Spark);
        assert_eq!(ParticleEffect::from_u32(5), ParticleEffect::Dust);
        assert_eq!(ParticleEffect::from_u32(6), ParticleEffect::Explosion);
        assert_eq!(ParticleEffect::from_u32(99), ParticleEffect::Generic);
    }

    #[test]
    fn test_effect_defaults() {
        let fire = ParticleEffect::Fire;
        let color = fire.default_color();
        assert!(color[0] > 0.9); // Red channel high
        assert!(fire.default_gravity() < 0.0); // Rises

        let water = ParticleEffect::Water;
        assert!(water.default_gravity() > 1.0); // Falls fast
    }

    #[test]
    fn test_emitter_creation() {
        let fire = ParticleEmitter::fire((100.0, 200.0), 50.0);
        assert!((fire.position.0 - 100.0).abs() < f32::EPSILON);
        assert!((fire.position.1 - 200.0).abs() < f32::EPSILON);
        assert_eq!(fire.effect, ParticleEffect::Fire);

        let smoke = ParticleEmitter::smoke((0.0, 0.0), 20.0);
        assert_eq!(smoke.effect, ParticleEffect::Smoke);

        let water = ParticleEmitter::water((50.0, 50.0), 100.0);
        assert_eq!(water.effect, ParticleEffect::Water);

        let spark = ParticleEmitter::spark((10.0, 10.0), 200.0);
        assert_eq!(spark.effect, ParticleEffect::Spark);
    }

    #[test]
    fn test_emitter_builder() {
        let emitter = ParticleEmitter::default()
            .with_position(50.0, 75.0)
            .with_rate(100.0)
            .with_enabled(false);

        assert!((emitter.position.0 - 50.0).abs() < f32::EPSILON);
        assert!((emitter.position.1 - 75.0).abs() < f32::EPSILON);
        assert!((emitter.emission_rate - 100.0).abs() < f32::EPSILON);
        assert!(!emitter.enabled);
    }

    #[test]
    fn test_gpu_particle_size() {
        // Ensure proper alignment for GPU
        assert_eq!(std::mem::size_of::<GpuParticle>(), 48);
    }

    #[test]
    fn test_particle_params_size() {
        // Check struct size for GPU compatibility
        assert_eq!(std::mem::size_of::<ParticleParams>(), 32);
    }

    #[test]
    fn test_particle_instance_size() {
        // Check instance size
        assert_eq!(std::mem::size_of::<ParticleInstance>(), 32);
    }

    #[test]
    fn test_particle_defaults() {
        let particle = GpuParticle::default();
        assert!((particle.size - 2.0).abs() < f32::EPSILON);
        assert!((particle.gravity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_params_defaults() {
        let params = ParticleParams::default();
        assert!((params.gravity - 98.0).abs() < f32::EPSILON);
        assert_eq!(params.particle_count, 0);
    }
}
