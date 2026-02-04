//! Combat particle effects system.
//!
//! Provides particle effects for combat:
//! - Hit sparks for weapon impacts
//! - Blood splatter for creature hits
//! - Impact dust for terrain hits
//! - Particle pooling for performance
//!
//! # Example
//!
//! ```
//! use genesis_kernel::combat_particles::{
//!     CombatParticleManager, HitSparkEffect, CombatEffectType,
//! };
//!
//! let mut manager = CombatParticleManager::new();
//!
//! // Spawn a hit spark
//! manager.spawn_hit_spark((100.0, 100.0), (1.0, 0.0), 1.0);
//!
//! // Update particles
//! manager.update(1.0 / 60.0);
//! ```

use std::collections::VecDeque;

/// Type of combat effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum CombatEffectType {
    /// Sparks from weapon impacts.
    #[default]
    HitSpark = 0,
    /// Blood from creature damage.
    BloodSplatter = 1,
    /// Dust from terrain impacts.
    ImpactDust = 2,
    /// Slash trail effect.
    SlashTrail = 3,
    /// Magic impact.
    MagicImpact = 4,
    /// Fire burst.
    FireBurst = 5,
    /// Ice shatter.
    IceShatter = 6,
    /// Electric arc.
    ElectricArc = 7,
    /// Poison cloud.
    PoisonCloud = 8,
    /// Shield block.
    ShieldBlock = 9,
}

impl CombatEffectType {
    /// Convert from u8.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::HitSpark),
            1 => Some(Self::BloodSplatter),
            2 => Some(Self::ImpactDust),
            3 => Some(Self::SlashTrail),
            4 => Some(Self::MagicImpact),
            5 => Some(Self::FireBurst),
            6 => Some(Self::IceShatter),
            7 => Some(Self::ElectricArc),
            8 => Some(Self::PoisonCloud),
            9 => Some(Self::ShieldBlock),
            _ => None,
        }
    }

    /// Get default particle count for this effect.
    #[must_use]
    pub const fn default_particle_count(&self) -> usize {
        match self {
            Self::HitSpark => 12,
            Self::BloodSplatter | Self::ElectricArc | Self::ShieldBlock => 8,
            Self::ImpactDust => 6,
            Self::SlashTrail => 20,
            Self::MagicImpact => 16,
            Self::FireBurst => 24,
            Self::IceShatter => 10,
            Self::PoisonCloud => 15,
        }
    }

    /// Get default lifetime for particles of this effect.
    #[must_use]
    pub const fn default_lifetime(&self) -> f32 {
        match self {
            Self::HitSpark => 0.3,
            Self::BloodSplatter | Self::MagicImpact => 0.5,
            Self::ImpactDust | Self::FireBurst => 0.4,
            Self::SlashTrail => 0.2,
            Self::IceShatter => 0.6,
            Self::ElectricArc => 0.15,
            Self::PoisonCloud => 1.5,
            Self::ShieldBlock => 0.25,
        }
    }

    /// Get base color for this effect.
    #[must_use]
    pub const fn base_color(&self) -> [f32; 4] {
        match self {
            Self::HitSpark => [1.0, 0.9, 0.5, 1.0],      // Yellow-white
            Self::BloodSplatter => [0.8, 0.1, 0.1, 1.0], // Red
            Self::ImpactDust => [0.6, 0.5, 0.4, 0.8],    // Brown
            Self::SlashTrail => [1.0, 1.0, 1.0, 0.7],    // White
            Self::MagicImpact => [0.5, 0.3, 1.0, 1.0],   // Purple
            Self::FireBurst => [1.0, 0.5, 0.1, 1.0],     // Orange
            Self::IceShatter => [0.6, 0.9, 1.0, 1.0],    // Cyan
            Self::ElectricArc => [0.4, 0.7, 1.0, 1.0],   // Blue
            Self::PoisonCloud => [0.3, 0.8, 0.2, 0.6],   // Green
            Self::ShieldBlock => [0.8, 0.8, 1.0, 1.0],   // Light blue
        }
    }
}

/// A single combat particle.
#[derive(Debug, Clone, Copy)]
pub struct CombatParticle {
    /// Position (x, y).
    pub position: (f32, f32),
    /// Velocity (x, y).
    pub velocity: (f32, f32),
    /// Current lifetime.
    pub lifetime: f32,
    /// Maximum lifetime.
    pub max_lifetime: f32,
    /// Size.
    pub size: f32,
    /// Initial size.
    pub initial_size: f32,
    /// Color with alpha.
    pub color: [f32; 4],
    /// Rotation.
    pub rotation: f32,
    /// Rotation velocity.
    pub rotation_velocity: f32,
    /// Gravity scale.
    pub gravity: f32,
    /// Drag coefficient.
    pub drag: f32,
    /// Whether particle is active.
    pub active: bool,
    /// Effect type.
    pub effect_type: CombatEffectType,
}

impl Default for CombatParticle {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            lifetime: 0.0,
            max_lifetime: 1.0,
            size: 4.0,
            initial_size: 4.0,
            color: [1.0, 1.0, 1.0, 1.0],
            rotation: 0.0,
            rotation_velocity: 0.0,
            gravity: 0.0,
            drag: 0.0,
            active: true,
            effect_type: CombatEffectType::HitSpark,
        }
    }
}

impl CombatParticle {
    /// Check if particle is alive.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active && self.lifetime < self.max_lifetime
    }

    /// Get progress (0-1).
    #[must_use]
    pub fn progress(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }

    /// Get current alpha.
    #[must_use]
    pub fn alpha(&self) -> f32 {
        let progress = self.progress();
        // Fade out in last 30%
        if progress > 0.7 {
            self.color[3] * (1.0 - (progress - 0.7) / 0.3)
        } else {
            self.color[3]
        }
    }

    /// Update particle.
    pub fn update(&mut self, dt: f32, gravity: f32) {
        if !self.is_active() {
            return;
        }

        self.lifetime += dt;

        if self.lifetime >= self.max_lifetime {
            self.active = false;
            return;
        }

        // Apply gravity
        self.velocity.1 += gravity * self.gravity * dt;

        // Apply drag
        if self.drag > 0.0 {
            let speed =
                (self.velocity.0 * self.velocity.0 + self.velocity.1 * self.velocity.1).sqrt();
            if speed > 0.01 {
                let drag_force = self.drag * dt;
                self.velocity.0 *= 1.0 - drag_force;
                self.velocity.1 *= 1.0 - drag_force;
            }
        }

        // Update position
        self.position.0 += self.velocity.0 * dt;
        self.position.1 += self.velocity.1 * dt;

        // Update rotation
        self.rotation += self.rotation_velocity * dt;

        // Update size based on effect type
        let progress = self.progress();
        match self.effect_type {
            CombatEffectType::HitSpark => {
                // Shrink over time
                self.size = self.initial_size * (1.0 - progress);
            },
            CombatEffectType::BloodSplatter => {
                // Grow slightly then shrink
                if progress < 0.2 {
                    self.size = self.initial_size * (1.0 + progress);
                } else {
                    self.size = self.initial_size * 1.2 * (1.0 - (progress - 0.2) / 0.8);
                }
            },
            CombatEffectType::ImpactDust => {
                // Grow and fade
                self.size = self.initial_size * (1.0 + progress * 2.0);
            },
            CombatEffectType::PoisonCloud => {
                // Grow slowly
                self.size = self.initial_size * (1.0 + progress * 0.5);
            },
            _ => {
                // Default: shrink
                self.size = self.initial_size * (1.0 - progress * 0.5);
            },
        }
    }
}

/// GPU-friendly particle instance.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CombatParticleInstance {
    /// Position (x, y).
    pub position: [f32; 2],
    /// Size.
    pub size: f32,
    /// Rotation.
    pub rotation: f32,
    /// Color with alpha.
    pub color: [f32; 4],
    /// UV offset in atlas.
    pub uv_offset: [f32; 2],
    /// UV size.
    pub uv_size: [f32; 2],
}

impl CombatParticleInstance {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create from a combat particle.
    #[must_use]
    pub fn from_particle(particle: &CombatParticle, uv: (f32, f32, f32, f32)) -> Self {
        let mut color = particle.color;
        color[3] = particle.alpha();
        Self {
            position: [particle.position.0, particle.position.1],
            size: particle.size,
            rotation: particle.rotation,
            color,
            uv_offset: [uv.0, uv.1],
            uv_size: [uv.2, uv.3],
        }
    }
}

/// Hit spark effect configuration.
#[derive(Debug, Clone)]
pub struct HitSparkEffect {
    /// Base color.
    pub color: [f32; 4],
    /// Particle count.
    pub count: usize,
    /// Base size.
    pub size: f32,
    /// Speed range.
    pub speed_min: f32,
    /// Speed range.
    pub speed_max: f32,
    /// Spread angle (radians).
    pub spread: f32,
    /// Lifetime.
    pub lifetime: f32,
}

impl Default for HitSparkEffect {
    fn default() -> Self {
        Self {
            color: CombatEffectType::HitSpark.base_color(),
            count: CombatEffectType::HitSpark.default_particle_count(),
            size: 4.0,
            speed_min: 100.0,
            speed_max: 250.0,
            spread: std::f32::consts::PI * 0.5,
            lifetime: CombatEffectType::HitSpark.default_lifetime(),
        }
    }
}

/// Blood splatter effect configuration.
#[derive(Debug, Clone)]
pub struct BloodSplatterEffect {
    /// Base color.
    pub color: [f32; 4],
    /// Particle count.
    pub count: usize,
    /// Base size.
    pub size: f32,
    /// Speed range.
    pub speed_min: f32,
    /// Speed range.
    pub speed_max: f32,
    /// Gravity.
    pub gravity: f32,
    /// Lifetime.
    pub lifetime: f32,
}

impl Default for BloodSplatterEffect {
    fn default() -> Self {
        Self {
            color: CombatEffectType::BloodSplatter.base_color(),
            count: CombatEffectType::BloodSplatter.default_particle_count(),
            size: 3.0,
            speed_min: 80.0,
            speed_max: 180.0,
            gravity: 1.0,
            lifetime: CombatEffectType::BloodSplatter.default_lifetime(),
        }
    }
}

/// Impact dust effect configuration.
#[derive(Debug, Clone)]
pub struct ImpactDustEffect {
    /// Base color.
    pub color: [f32; 4],
    /// Particle count.
    pub count: usize,
    /// Base size.
    pub size: f32,
    /// Speed range.
    pub speed_max: f32,
    /// Lifetime.
    pub lifetime: f32,
}

impl Default for ImpactDustEffect {
    fn default() -> Self {
        Self {
            color: CombatEffectType::ImpactDust.base_color(),
            count: CombatEffectType::ImpactDust.default_particle_count(),
            size: 8.0,
            speed_max: 50.0,
            lifetime: CombatEffectType::ImpactDust.default_lifetime(),
        }
    }
}

/// Particle pool for performance.
#[derive(Debug, Default)]
pub struct ParticlePool {
    particles: Vec<CombatParticle>,
    free_indices: VecDeque<usize>,
    active_count: usize,
}

impl ParticlePool {
    /// Create a new pool with capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            particles: Vec::with_capacity(capacity),
            free_indices: VecDeque::with_capacity(capacity),
            active_count: 0,
        }
    }

    /// Allocate a particle from the pool.
    pub fn allocate(&mut self) -> Option<&mut CombatParticle> {
        if let Some(index) = self.free_indices.pop_front() {
            self.particles[index].active = true;
            self.active_count += 1;
            Some(&mut self.particles[index])
        } else if self.particles.len() < self.particles.capacity() {
            self.particles.push(CombatParticle::default());
            self.active_count += 1;
            self.particles.last_mut()
        } else {
            // Pool exhausted, try to recycle oldest inactive
            for (i, p) in self.particles.iter_mut().enumerate() {
                if !p.is_active() {
                    p.active = true;
                    p.lifetime = 0.0;
                    self.active_count += 1;
                    return Some(&mut self.particles[i]);
                }
            }
            None
        }
    }

    /// Update all particles and return inactive to pool.
    pub fn update(&mut self, dt: f32, gravity: f32) {
        self.active_count = 0;
        for (i, particle) in self.particles.iter_mut().enumerate() {
            if particle.active {
                particle.update(dt, gravity);
                if particle.is_active() {
                    self.active_count += 1;
                } else {
                    particle.active = false;
                    self.free_indices.push_back(i);
                }
            }
        }
    }

    /// Get active particles.
    pub fn active_particles(&self) -> impl Iterator<Item = &CombatParticle> {
        self.particles.iter().filter(|p| p.is_active())
    }

    /// Get active count.
    #[must_use]
    pub const fn active_count(&self) -> usize {
        self.active_count
    }

    /// Get total capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.particles.capacity()
    }

    /// Clear all particles.
    pub fn clear(&mut self) {
        for p in &mut self.particles {
            p.active = false;
        }
        self.free_indices.clear();
        for i in 0..self.particles.len() {
            self.free_indices.push_back(i);
        }
        self.active_count = 0;
    }
}

/// Combat particle manager.
#[derive(Debug, Default)]
pub struct CombatParticleManager {
    pool: ParticlePool,
    gravity: f32,
    seed: u32,
    hit_spark_config: HitSparkEffect,
    blood_config: BloodSplatterEffect,
    dust_config: ImpactDustEffect,
}

impl CombatParticleManager {
    /// Create a new combat particle manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: ParticlePool::with_capacity(512),
            gravity: 400.0,
            seed: 12345,
            hit_spark_config: HitSparkEffect::default(),
            blood_config: BloodSplatterEffect::default(),
            dust_config: ImpactDustEffect::default(),
        }
    }

    /// Create with custom capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: ParticlePool::with_capacity(capacity),
            ..Self::new()
        }
    }

    /// Set gravity.
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    /// Configure hit spark effect.
    pub fn configure_hit_spark(&mut self, config: HitSparkEffect) {
        self.hit_spark_config = config;
    }

    /// Configure blood splatter effect.
    pub fn configure_blood(&mut self, config: BloodSplatterEffect) {
        self.blood_config = config;
    }

    /// Configure impact dust effect.
    pub fn configure_dust(&mut self, config: ImpactDustEffect) {
        self.dust_config = config;
    }

    /// Simple random number generator.
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        self.seed as f32 / u32::MAX as f32
    }

    /// Random in range.
    fn random_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.random() * (max - min)
    }

    /// Spawn hit spark effect.
    pub fn spawn_hit_spark(&mut self, position: (f32, f32), direction: (f32, f32), intensity: f32) {
        let count = ((self.hit_spark_config.count as f32) * intensity) as usize;
        let base_angle = direction.1.atan2(direction.0);

        for _ in 0..count {
            // Pre-compute random values before borrowing pool
            let spread = self.hit_spark_config.spread;
            let speed_min = self.hit_spark_config.speed_min;
            let speed_max = self.hit_spark_config.speed_max;
            let lifetime = self.hit_spark_config.lifetime;
            let size = self.hit_spark_config.size;
            let color = self.hit_spark_config.color;

            let angle = base_angle + self.random_range(-spread * 0.5, spread * 0.5);
            let speed = self.random_range(speed_min, speed_max) * intensity;
            let max_lifetime = lifetime * self.random_range(0.8, 1.2);
            let particle_size = size * self.random_range(0.5, 1.5);
            let rotation = self.random() * std::f32::consts::TAU;
            let rotation_velocity = self.random_range(-10.0, 10.0);

            if let Some(particle) = self.pool.allocate() {
                particle.position = position;
                particle.velocity = (angle.cos() * speed, angle.sin() * speed);
                particle.lifetime = 0.0;
                particle.max_lifetime = max_lifetime;
                particle.size = particle_size;
                particle.initial_size = particle_size;
                particle.color = color;
                particle.rotation = rotation;
                particle.rotation_velocity = rotation_velocity;
                particle.gravity = 0.0;
                particle.drag = 0.1;
                particle.effect_type = CombatEffectType::HitSpark;
            }
        }
    }

    /// Spawn blood splatter effect.
    pub fn spawn_blood_splatter(
        &mut self,
        position: (f32, f32),
        direction: (f32, f32),
        intensity: f32,
    ) {
        let count = ((self.blood_config.count as f32) * intensity) as usize;
        let base_angle = direction.1.atan2(direction.0);

        for _ in 0..count {
            // Pre-compute random values and config before borrowing pool
            let speed_min = self.blood_config.speed_min;
            let speed_max = self.blood_config.speed_max;
            let lifetime = self.blood_config.lifetime;
            let size = self.blood_config.size;
            let color = self.blood_config.color;
            let gravity = self.blood_config.gravity;

            let angle = base_angle + self.random_range(-0.5, 0.5);
            let speed = self.random_range(speed_min, speed_max) * intensity;
            let max_lifetime = lifetime * self.random_range(0.8, 1.2);
            let particle_size = size * self.random_range(0.5, 1.5);
            let color_variation = self.random_range(-0.1, 0.1);

            if let Some(particle) = self.pool.allocate() {
                particle.position = position;
                particle.velocity = (angle.cos() * speed, angle.sin() * speed);
                particle.lifetime = 0.0;
                particle.max_lifetime = max_lifetime;
                particle.size = particle_size;
                particle.initial_size = particle_size;
                particle.color = color;
                // Slight color variation
                particle.color[0] += color_variation;
                particle.rotation = 0.0;
                particle.rotation_velocity = 0.0;
                particle.gravity = gravity;
                particle.drag = 0.05;
                particle.effect_type = CombatEffectType::BloodSplatter;
            }
        }
    }

    /// Spawn impact dust effect.
    pub fn spawn_impact_dust(&mut self, position: (f32, f32), intensity: f32) {
        let count = ((self.dust_config.count as f32) * intensity) as usize;

        for _ in 0..count {
            // Pre-compute random values and config before borrowing pool
            let speed_max = self.dust_config.speed_max;
            let lifetime = self.dust_config.lifetime;
            let size = self.dust_config.size;
            let color = self.dust_config.color;

            let angle = self.random() * std::f32::consts::TAU;
            let speed = self.random_range(0.0, speed_max) * intensity;
            let max_lifetime = lifetime * self.random_range(0.8, 1.2);
            let particle_size = size * self.random_range(0.5, 1.5);
            let rotation = self.random() * std::f32::consts::TAU;
            let rotation_velocity = self.random_range(-2.0, 2.0);

            if let Some(particle) = self.pool.allocate() {
                particle.position = position;
                particle.velocity = (angle.cos() * speed, angle.sin() * speed - 20.0);
                particle.lifetime = 0.0;
                particle.max_lifetime = max_lifetime;
                particle.size = particle_size;
                particle.initial_size = particle_size;
                particle.color = color;
                particle.rotation = rotation;
                particle.rotation_velocity = rotation_velocity;
                particle.gravity = -0.2; // Rise slightly
                particle.drag = 0.2;
                particle.effect_type = CombatEffectType::ImpactDust;
            }
        }
    }

    /// Spawn slash trail effect.
    pub fn spawn_slash_trail(&mut self, start: (f32, f32), end: (f32, f32), color: [f32; 4]) {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let len = (dx * dx + dy * dy).sqrt();
        let count = (len / 10.0) as usize;

        for i in 0..count {
            // Pre-compute random values before borrowing pool
            let offset_x = self.random_range(-2.0, 2.0);
            let offset_y = self.random_range(-2.0, 2.0);
            let vel_x = self.random_range(-20.0, 20.0);
            let vel_y = self.random_range(-20.0, 20.0);

            if let Some(particle) = self.pool.allocate() {
                let t = i as f32 / count.max(1) as f32;
                particle.position = (start.0 + dx * t + offset_x, start.1 + dy * t + offset_y);
                particle.velocity = (vel_x, vel_y);
                particle.lifetime = 0.0;
                particle.max_lifetime = CombatEffectType::SlashTrail.default_lifetime();
                particle.size = 6.0 - t * 4.0; // Larger at start
                particle.initial_size = particle.size;
                particle.color = color;
                particle.rotation = dy.atan2(dx);
                particle.rotation_velocity = 0.0;
                particle.gravity = 0.0;
                particle.drag = 0.0;
                particle.effect_type = CombatEffectType::SlashTrail;
            }
        }
    }

    /// Spawn magic impact effect.
    pub fn spawn_magic_impact(&mut self, position: (f32, f32), color: [f32; 4], intensity: f32) {
        let count =
            (CombatEffectType::MagicImpact.default_particle_count() as f32 * intensity) as usize;

        for _ in 0..count {
            // Pre-compute random values before borrowing pool
            let angle = self.random() * std::f32::consts::TAU;
            let speed = self.random_range(50.0, 150.0) * intensity;
            let particle_size = 5.0 * self.random_range(0.5, 1.5);
            let rotation = self.random() * std::f32::consts::TAU;
            let rotation_velocity = self.random_range(-5.0, 5.0);

            if let Some(particle) = self.pool.allocate() {
                particle.position = position;
                particle.velocity = (angle.cos() * speed, angle.sin() * speed);
                particle.lifetime = 0.0;
                particle.max_lifetime = CombatEffectType::MagicImpact.default_lifetime();
                particle.size = particle_size;
                particle.initial_size = particle_size;
                particle.color = color;
                particle.rotation = rotation;
                particle.rotation_velocity = rotation_velocity;
                particle.gravity = 0.0;
                particle.drag = 0.15;
                particle.effect_type = CombatEffectType::MagicImpact;
            }
        }
    }

    /// Spawn shield block effect.
    pub fn spawn_shield_block(&mut self, position: (f32, f32), direction: (f32, f32)) {
        let count = CombatEffectType::ShieldBlock.default_particle_count();
        let base_angle = direction.1.atan2(direction.0);

        for _ in 0..count {
            // Pre-compute random values before borrowing pool
            let angle = base_angle + self.random_range(-0.8, 0.8);
            let speed = self.random_range(80.0, 160.0);
            let particle_size = 4.0 * self.random_range(0.5, 1.5);

            if let Some(particle) = self.pool.allocate() {
                particle.position = position;
                particle.velocity = (angle.cos() * speed, angle.sin() * speed);
                particle.lifetime = 0.0;
                particle.max_lifetime = CombatEffectType::ShieldBlock.default_lifetime();
                particle.size = particle_size;
                particle.initial_size = particle_size;
                particle.color = CombatEffectType::ShieldBlock.base_color();
                particle.rotation = 0.0;
                particle.rotation_velocity = 0.0;
                particle.gravity = 0.0;
                particle.drag = 0.1;
                particle.effect_type = CombatEffectType::ShieldBlock;
            }
        }
    }

    /// Spawn generic effect.
    pub fn spawn_effect(
        &mut self,
        effect_type: CombatEffectType,
        position: (f32, f32),
        direction: (f32, f32),
        intensity: f32,
    ) {
        match effect_type {
            CombatEffectType::HitSpark => self.spawn_hit_spark(position, direction, intensity),
            CombatEffectType::BloodSplatter => {
                self.spawn_blood_splatter(position, direction, intensity);
            },
            CombatEffectType::ImpactDust => self.spawn_impact_dust(position, intensity),
            CombatEffectType::ShieldBlock => self.spawn_shield_block(position, direction),
            CombatEffectType::MagicImpact => {
                self.spawn_magic_impact(position, effect_type.base_color(), intensity);
            },
            _ => {
                // Generic radial burst
                let count = (effect_type.default_particle_count() as f32 * intensity) as usize;
                let color = effect_type.base_color();
                for _ in 0..count {
                    // Pre-compute random values before borrowing pool
                    let angle = self.random() * std::f32::consts::TAU;
                    let speed = self.random_range(50.0, 150.0) * intensity;

                    if let Some(particle) = self.pool.allocate() {
                        particle.position = position;
                        particle.velocity = (angle.cos() * speed, angle.sin() * speed);
                        particle.lifetime = 0.0;
                        particle.max_lifetime = effect_type.default_lifetime();
                        particle.size = 4.0;
                        particle.initial_size = 4.0;
                        particle.color = color;
                        particle.effect_type = effect_type;
                    }
                }
            },
        }
    }

    /// Update all particles.
    pub fn update(&mut self, dt: f32) {
        self.pool.update(dt, self.gravity);
    }

    /// Get render instances.
    #[must_use]
    pub fn get_render_instances(
        &self,
        uv_lookup: impl Fn(CombatEffectType) -> (f32, f32, f32, f32),
    ) -> Vec<CombatParticleInstance> {
        self.pool
            .active_particles()
            .map(|p| CombatParticleInstance::from_particle(p, uv_lookup(p.effect_type)))
            .collect()
    }

    /// Get active particle count.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.pool.active_count()
    }

    /// Clear all particles.
    pub fn clear(&mut self) {
        self.pool.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_type_from_u8() {
        assert_eq!(
            CombatEffectType::from_u8(0),
            Some(CombatEffectType::HitSpark)
        );
        assert_eq!(
            CombatEffectType::from_u8(1),
            Some(CombatEffectType::BloodSplatter)
        );
        assert_eq!(CombatEffectType::from_u8(99), None);
    }

    #[test]
    fn test_effect_type_properties() {
        let spark = CombatEffectType::HitSpark;
        assert_eq!(spark.default_particle_count(), 12);
        assert!((spark.default_lifetime() - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_particle_creation() {
        let particle = CombatParticle::default();
        assert!(particle.is_active());
        assert_eq!(particle.progress(), 0.0);
    }

    #[test]
    fn test_particle_update() {
        let mut particle = CombatParticle {
            position: (0.0, 0.0),
            velocity: (100.0, 0.0),
            max_lifetime: 1.0,
            active: true,
            ..Default::default()
        };

        particle.update(0.5, 0.0);
        assert!(particle.position.0 > 0.0);
        assert!(particle.is_active());

        particle.update(0.6, 0.0);
        assert!(!particle.is_active());
    }

    #[test]
    fn test_particle_pool() {
        let mut pool = ParticlePool::with_capacity(10);

        let p1 = pool.allocate();
        assert!(p1.is_some());
        assert_eq!(pool.active_count(), 1);

        pool.update(2.0, 0.0); // All particles should die
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_combat_particle_manager() {
        let mut manager = CombatParticleManager::new();

        manager.spawn_hit_spark((0.0, 0.0), (1.0, 0.0), 1.0);
        assert!(manager.active_count() > 0);

        manager.update(0.1);
        assert!(manager.active_count() > 0);

        manager.update(1.0);
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_blood_splatter() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_blood_splatter((0.0, 0.0), (1.0, 0.0), 1.0);
        assert!(manager.active_count() > 0);
    }

    #[test]
    fn test_impact_dust() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_impact_dust((0.0, 0.0), 1.0);
        assert!(manager.active_count() > 0);
    }

    #[test]
    fn test_slash_trail() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_slash_trail((0.0, 0.0), (100.0, 0.0), [1.0, 1.0, 1.0, 1.0]);
        assert!(manager.active_count() > 0);
    }

    #[test]
    fn test_magic_impact() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_magic_impact((0.0, 0.0), [0.5, 0.3, 1.0, 1.0], 1.0);
        assert!(manager.active_count() > 0);
    }

    #[test]
    fn test_shield_block() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_shield_block((0.0, 0.0), (1.0, 0.0));
        assert!(manager.active_count() > 0);
    }

    #[test]
    fn test_particle_instance_size() {
        assert_eq!(CombatParticleInstance::SIZE, 48);
    }

    #[test]
    fn test_render_instances() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_hit_spark((0.0, 0.0), (1.0, 0.0), 1.0);

        let instances = manager.get_render_instances(|_| (0.0, 0.0, 0.1, 0.1));
        assert!(!instances.is_empty());
    }

    #[test]
    fn test_particle_alpha_fade() {
        let mut particle = CombatParticle {
            max_lifetime: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            active: true,
            ..Default::default()
        };

        particle.lifetime = 0.5;
        assert!((particle.alpha() - 1.0).abs() < 0.01);

        particle.lifetime = 0.9;
        assert!(particle.alpha() < 1.0);
    }

    #[test]
    fn test_clear() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_hit_spark((0.0, 0.0), (1.0, 0.0), 1.0);
        assert!(manager.active_count() > 0);

        manager.clear();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_spawn_generic_effect() {
        let mut manager = CombatParticleManager::new();
        manager.spawn_effect(CombatEffectType::FireBurst, (0.0, 0.0), (1.0, 0.0), 1.0);
        assert!(manager.active_count() > 0);
    }

    #[test]
    fn test_effect_colors() {
        let spark_color = CombatEffectType::HitSpark.base_color();
        assert!(spark_color[0] > 0.9);

        let blood_color = CombatEffectType::BloodSplatter.base_color();
        assert!(blood_color[0] > 0.7);
        assert!(blood_color[1] < 0.2);
    }
}
