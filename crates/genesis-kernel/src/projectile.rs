//! Projectile physics system for combat.
//!
//! Provides physics simulation for various projectile types:
//! - Arc trajectories (arrows, thrown objects)
//! - Straight trajectories (spells, bullets)
//! - Homing projectiles
//! - Terrain and entity collision
//!
//! # Example
//!
//! ```
//! use genesis_kernel::projectile::{Projectile, ProjectileType, ProjectileManager};
//!
//! let mut manager = ProjectileManager::new();
//!
//! // Create an arrow with arc trajectory
//! let arrow = Projectile::arrow((0.0, 0.0), (1.0, -0.5), 500.0);
//! let id = manager.spawn(arrow);
//!
//! // Update physics
//! manager.update(1.0 / 60.0);
//! ```

use std::collections::HashMap;

/// Type alias for projectile identifiers.
pub type ProjectileId = u64;

/// Type alias for entity identifiers.
pub type EntityId = u64;

/// Default gravity constant.
pub const DEFAULT_GRAVITY: f32 = 980.0;

/// Default air resistance.
pub const DEFAULT_DRAG: f32 = 0.02;

/// Type of projectile trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ProjectileType {
    /// Affected by gravity (arrows, thrown rocks).
    #[default]
    Arc = 0,
    /// Straight line (spells, bullets).
    Straight = 1,
    /// Homes in on target.
    Homing = 2,
    /// Falls straight down (dropped items, debris).
    Falling = 3,
    /// Bounces off surfaces.
    Bouncing = 4,
    /// Follows a bezier curve.
    Curved = 5,
}

impl ProjectileType {
    /// Convert from u8.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Arc),
            1 => Some(Self::Straight),
            2 => Some(Self::Homing),
            3 => Some(Self::Falling),
            4 => Some(Self::Bouncing),
            5 => Some(Self::Curved),
            _ => None,
        }
    }

    /// Check if affected by gravity.
    #[must_use]
    pub const fn has_gravity(&self) -> bool {
        matches!(self, Self::Arc | Self::Falling | Self::Bouncing)
    }

    /// Check if affected by drag.
    #[must_use]
    pub const fn has_drag(&self) -> bool {
        matches!(self, Self::Arc | Self::Bouncing)
    }
}

/// Collision type for projectiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CollisionType {
    /// No collision occurred.
    None = 0,
    /// Hit terrain.
    Terrain = 1,
    /// Hit an entity.
    Entity = 2,
    /// Hit water surface.
    Water = 3,
    /// Hit shield or barrier.
    Shield = 4,
}

/// Result of a projectile collision.
#[derive(Debug, Clone)]
pub struct ProjectileCollision {
    /// Type of collision.
    pub collision_type: CollisionType,
    /// Projectile ID.
    pub projectile_id: ProjectileId,
    /// Hit position.
    pub position: (f32, f32),
    /// Impact velocity.
    pub velocity: (f32, f32),
    /// Entity hit (if any).
    pub entity_id: Option<EntityId>,
    /// Damage to apply.
    pub damage: f32,
    /// Whether the projectile should be destroyed.
    pub destroyed: bool,
}

/// Projectile state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ProjectileState {
    /// Active and flying.
    #[default]
    Active = 0,
    /// Hit something and embedded.
    Embedded = 1,
    /// Destroyed/expired.
    Destroyed = 2,
    /// Waiting to be spawned.
    Pending = 3,
}

/// A projectile in flight.
#[derive(Debug, Clone)]
pub struct Projectile {
    /// Unique identifier.
    pub id: ProjectileId,
    /// Projectile type.
    pub projectile_type: ProjectileType,
    /// Current state.
    pub state: ProjectileState,
    /// Position (x, y).
    pub position: (f32, f32),
    /// Velocity (x, y).
    pub velocity: (f32, f32),
    /// Rotation angle (radians).
    pub rotation: f32,
    /// Whether rotation follows velocity.
    pub rotate_to_velocity: bool,
    /// Gravity multiplier (0 = no gravity).
    pub gravity_scale: f32,
    /// Drag coefficient.
    pub drag: f32,
    /// Base damage.
    pub damage: f32,
    /// Damage falloff per second.
    pub damage_falloff: f32,
    /// Collision radius.
    pub radius: f32,
    /// Maximum lifetime (seconds).
    pub max_lifetime: f32,
    /// Current lifetime.
    pub lifetime: f32,
    /// Maximum range (-1 = infinite).
    pub max_range: f32,
    /// Distance traveled.
    pub distance_traveled: f32,
    /// Owner entity (won't collide with).
    pub owner: Option<EntityId>,
    /// Target entity for homing.
    pub target: Option<EntityId>,
    /// Homing strength (turn rate per second).
    pub homing_strength: f32,
    /// Number of bounces remaining.
    pub bounces_remaining: u8,
    /// Bounce velocity retention (0-1).
    pub bounce_elasticity: f32,
    /// Whether to pierce through entities.
    pub piercing: bool,
    /// Number of entities pierced.
    pub pierce_count: u8,
    /// Maximum pierce count.
    pub max_pierce: u8,
    /// Trail effect ID.
    pub trail_effect: u16,
    /// Impact effect ID.
    pub impact_effect: u16,
    /// Spawn position (for range calculation).
    #[allow(dead_code)]
    spawn_position: (f32, f32),
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            id: 0,
            projectile_type: ProjectileType::Arc,
            state: ProjectileState::Active,
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            rotation: 0.0,
            rotate_to_velocity: true,
            gravity_scale: 1.0,
            drag: DEFAULT_DRAG,
            damage: 10.0,
            damage_falloff: 0.0,
            radius: 4.0,
            max_lifetime: 10.0,
            lifetime: 0.0,
            max_range: -1.0,
            distance_traveled: 0.0,
            owner: None,
            target: None,
            homing_strength: 0.0,
            bounces_remaining: 0,
            bounce_elasticity: 0.5,
            piercing: false,
            pierce_count: 0,
            max_pierce: 0,
            trail_effect: 0,
            impact_effect: 0,
            spawn_position: (0.0, 0.0),
        }
    }
}

impl Projectile {
    /// Create a new projectile.
    #[must_use]
    pub fn new(position: (f32, f32), velocity: (f32, f32)) -> Self {
        let rotation = velocity.1.atan2(velocity.0);
        Self {
            position,
            velocity,
            rotation,
            spawn_position: position,
            ..Default::default()
        }
    }

    /// Create an arrow projectile.
    #[must_use]
    pub fn arrow(position: (f32, f32), direction: (f32, f32), speed: f32) -> Self {
        let len = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
        let normalized = if len > 0.0 {
            (direction.0 / len, direction.1 / len)
        } else {
            (1.0, 0.0)
        };
        let velocity = (normalized.0 * speed, normalized.1 * speed);

        Self::new(position, velocity)
            .with_type(ProjectileType::Arc)
            .with_damage(25.0)
            .with_radius(3.0)
            .with_lifetime(5.0)
    }

    /// Create a spell projectile.
    #[must_use]
    pub fn spell(position: (f32, f32), direction: (f32, f32), speed: f32) -> Self {
        let len = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
        let normalized = if len > 0.0 {
            (direction.0 / len, direction.1 / len)
        } else {
            (1.0, 0.0)
        };
        let velocity = (normalized.0 * speed, normalized.1 * speed);

        Self::new(position, velocity)
            .with_type(ProjectileType::Straight)
            .with_gravity(0.0)
            .with_drag(0.0)
            .with_damage(30.0)
            .with_radius(8.0)
            .with_lifetime(3.0)
    }

    /// Create a thrown rock projectile.
    #[must_use]
    pub fn thrown(position: (f32, f32), velocity: (f32, f32)) -> Self {
        Self::new(position, velocity)
            .with_type(ProjectileType::Arc)
            .with_damage(15.0)
            .with_radius(6.0)
            .with_lifetime(4.0)
    }

    /// Create a homing projectile.
    #[must_use]
    pub fn homing(position: (f32, f32), target: EntityId, speed: f32) -> Self {
        Self::new(position, (speed, 0.0))
            .with_type(ProjectileType::Homing)
            .with_target(target)
            .with_homing_strength(5.0)
            .with_gravity(0.0)
            .with_damage(20.0)
            .with_lifetime(5.0)
    }

    /// Set projectile type.
    #[must_use]
    pub const fn with_type(mut self, projectile_type: ProjectileType) -> Self {
        self.projectile_type = projectile_type;
        self
    }

    /// Set gravity scale.
    #[must_use]
    pub const fn with_gravity(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Set drag coefficient.
    #[must_use]
    pub const fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag;
        self
    }

    /// Set damage.
    #[must_use]
    pub const fn with_damage(mut self, damage: f32) -> Self {
        self.damage = damage;
        self
    }

    /// Set collision radius.
    #[must_use]
    pub const fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set maximum lifetime.
    #[must_use]
    pub const fn with_lifetime(mut self, lifetime: f32) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    /// Set owner entity.
    #[must_use]
    pub const fn with_owner(mut self, owner: EntityId) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Set target entity.
    #[must_use]
    pub const fn with_target(mut self, target: EntityId) -> Self {
        self.target = Some(target);
        self
    }

    /// Set homing strength.
    #[must_use]
    pub const fn with_homing_strength(mut self, strength: f32) -> Self {
        self.homing_strength = strength;
        self
    }

    /// Set bouncing.
    #[must_use]
    pub const fn with_bounces(mut self, count: u8, elasticity: f32) -> Self {
        self.bounces_remaining = count;
        self.bounce_elasticity = elasticity;
        self.projectile_type = ProjectileType::Bouncing;
        self
    }

    /// Set piercing.
    #[must_use]
    pub const fn with_piercing(mut self, max_pierce: u8) -> Self {
        self.piercing = true;
        self.max_pierce = max_pierce;
        self
    }

    /// Set trail effect.
    #[must_use]
    pub const fn with_trail(mut self, effect_id: u16) -> Self {
        self.trail_effect = effect_id;
        self
    }

    /// Set impact effect.
    #[must_use]
    pub const fn with_impact(mut self, effect_id: u16) -> Self {
        self.impact_effect = effect_id;
        self
    }

    /// Set projectile ID.
    #[must_use]
    pub const fn with_id(mut self, id: ProjectileId) -> Self {
        self.id = id;
        self
    }

    /// Get current speed.
    #[must_use]
    pub fn speed(&self) -> f32 {
        (self.velocity.0 * self.velocity.0 + self.velocity.1 * self.velocity.1).sqrt()
    }

    /// Get current direction (normalized).
    #[must_use]
    pub fn direction(&self) -> (f32, f32) {
        let speed = self.speed();
        if speed > 0.001 {
            (self.velocity.0 / speed, self.velocity.1 / speed)
        } else {
            (self.rotation.cos(), self.rotation.sin())
        }
    }

    /// Check if projectile is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, ProjectileState::Active)
    }

    /// Check if projectile should be removed.
    #[must_use]
    pub const fn should_remove(&self) -> bool {
        matches!(self.state, ProjectileState::Destroyed)
    }

    /// Get effective damage (with falloff).
    #[must_use]
    pub fn effective_damage(&self) -> f32 {
        (self.damage - self.damage_falloff * self.lifetime).max(0.0)
    }

    /// Update projectile physics.
    pub fn update(&mut self, dt: f32, gravity: f32) {
        if !self.is_active() {
            return;
        }

        // Apply gravity
        if self.projectile_type.has_gravity() {
            self.velocity.1 += gravity * self.gravity_scale * dt;
        }

        // Apply drag
        if self.projectile_type.has_drag() && self.drag > 0.0 {
            let speed = self.speed();
            if speed > 0.0 {
                let drag_force = self.drag * speed * speed;
                let drag_x = -self.velocity.0 / speed * drag_force * dt;
                let drag_y = -self.velocity.1 / speed * drag_force * dt;
                self.velocity.0 += drag_x;
                self.velocity.1 += drag_y;
            }
        }

        // Update position
        let old_pos = self.position;
        self.position.0 += self.velocity.0 * dt;
        self.position.1 += self.velocity.1 * dt;

        // Update rotation
        if self.rotate_to_velocity {
            let speed = self.speed();
            if speed > 1.0 {
                self.rotation = self.velocity.1.atan2(self.velocity.0);
            }
        }

        // Update distance and lifetime
        let dx = self.position.0 - old_pos.0;
        let dy = self.position.1 - old_pos.1;
        self.distance_traveled += (dx * dx + dy * dy).sqrt();
        self.lifetime += dt;

        // Check lifetime expiration
        if self.lifetime >= self.max_lifetime {
            self.state = ProjectileState::Destroyed;
        }

        // Check range expiration
        if self.max_range > 0.0 && self.distance_traveled >= self.max_range {
            self.state = ProjectileState::Destroyed;
        }
    }

    /// Apply homing behavior.
    pub fn apply_homing(&mut self, target_pos: (f32, f32), dt: f32) {
        if self.homing_strength <= 0.0 {
            return;
        }

        let to_target = (
            target_pos.0 - self.position.0,
            target_pos.1 - self.position.1,
        );
        let dist = (to_target.0 * to_target.0 + to_target.1 * to_target.1).sqrt();
        if dist < 0.001 {
            return;
        }

        let desired_dir = (to_target.0 / dist, to_target.1 / dist);
        let current_dir = self.direction();

        // Smoothly rotate towards target
        let turn_amount = self.homing_strength * dt;
        let new_dir_x = current_dir.0 + (desired_dir.0 - current_dir.0) * turn_amount;
        let new_dir_y = current_dir.1 + (desired_dir.1 - current_dir.1) * turn_amount;

        // Normalize and apply speed
        let len = (new_dir_x * new_dir_x + new_dir_y * new_dir_y).sqrt();
        if len > 0.001 {
            let speed = self.speed();
            self.velocity.0 = new_dir_x / len * speed;
            self.velocity.1 = new_dir_y / len * speed;
        }
    }

    /// Handle bounce off a surface.
    pub fn bounce(&mut self, normal: (f32, f32)) {
        if self.bounces_remaining == 0 {
            self.state = ProjectileState::Destroyed;
            return;
        }

        self.bounces_remaining -= 1;

        // Reflect velocity
        let dot = self.velocity.0 * normal.0 + self.velocity.1 * normal.1;
        self.velocity.0 -= 2.0 * dot * normal.0;
        self.velocity.1 -= 2.0 * dot * normal.1;

        // Apply elasticity
        self.velocity.0 *= self.bounce_elasticity;
        self.velocity.1 *= self.bounce_elasticity;
    }

    /// Handle entity pierce.
    pub fn pierce(&mut self) -> bool {
        if !self.piercing || self.pierce_count >= self.max_pierce {
            self.state = ProjectileState::Destroyed;
            return false;
        }
        self.pierce_count += 1;
        true
    }
}

/// GPU-friendly projectile instance for rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ProjectileInstance {
    /// Position (x, y).
    pub position: [f32; 2],
    /// Rotation angle.
    pub rotation: f32,
    /// Scale.
    pub scale: f32,
    /// UV offset in sprite atlas.
    pub uv_offset: [f32; 2],
    /// UV size in sprite atlas.
    pub uv_size: [f32; 2],
    /// Tint color.
    pub tint: [f32; 4],
}

impl ProjectileInstance {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create from a projectile.
    #[must_use]
    pub fn from_projectile(projectile: &Projectile, sprite_data: (f32, f32, f32, f32)) -> Self {
        Self {
            position: [projectile.position.0, projectile.position.1],
            rotation: projectile.rotation,
            scale: projectile.radius * 2.0,
            uv_offset: [sprite_data.0, sprite_data.1],
            uv_size: [sprite_data.2, sprite_data.3],
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Projectile manager.
pub struct ProjectileManager {
    projectiles: HashMap<ProjectileId, Projectile>,
    next_id: ProjectileId,
    gravity: f32,
    /// Terrain collision callback signature: (x, y) -> is_solid
    terrain_collider: Option<Box<dyn Fn(f32, f32) -> bool + Send + Sync>>,
}

impl Default for ProjectileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ProjectileManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectileManager")
            .field("projectiles", &self.projectiles)
            .field("next_id", &self.next_id)
            .field("gravity", &self.gravity)
            .field("terrain_collider", &self.terrain_collider.is_some())
            .finish()
    }
}

impl ProjectileManager {
    /// Create a new projectile manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            projectiles: HashMap::new(),
            next_id: 1,
            gravity: DEFAULT_GRAVITY,
            terrain_collider: None,
        }
    }

    /// Set gravity.
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    /// Set terrain collision function.
    pub fn set_terrain_collider<F>(&mut self, collider: F)
    where
        F: Fn(f32, f32) -> bool + Send + Sync + 'static,
    {
        self.terrain_collider = Some(Box::new(collider));
    }

    /// Spawn a projectile.
    pub fn spawn(&mut self, mut projectile: Projectile) -> ProjectileId {
        let id = self.next_id;
        self.next_id += 1;
        projectile.id = id;
        self.projectiles.insert(id, projectile);
        id
    }

    /// Get a projectile by ID.
    #[must_use]
    pub fn get(&self, id: ProjectileId) -> Option<&Projectile> {
        self.projectiles.get(&id)
    }

    /// Get a mutable projectile by ID.
    pub fn get_mut(&mut self, id: ProjectileId) -> Option<&mut Projectile> {
        self.projectiles.get_mut(&id)
    }

    /// Remove a projectile.
    pub fn remove(&mut self, id: ProjectileId) -> Option<Projectile> {
        self.projectiles.remove(&id)
    }

    /// Update all projectiles.
    pub fn update(&mut self, dt: f32) -> Vec<ProjectileCollision> {
        let mut collisions = Vec::new();

        for projectile in self.projectiles.values_mut() {
            if !projectile.is_active() {
                continue;
            }

            let old_pos = projectile.position;
            projectile.update(dt, self.gravity);

            // Check terrain collision
            if let Some(ref collider) = self.terrain_collider {
                if collider(projectile.position.0, projectile.position.1) {
                    let collision = ProjectileCollision {
                        collision_type: CollisionType::Terrain,
                        projectile_id: projectile.id,
                        position: projectile.position,
                        velocity: projectile.velocity,
                        entity_id: None,
                        damage: projectile.effective_damage(),
                        destroyed: true,
                    };

                    // Handle bouncing
                    if projectile.projectile_type == ProjectileType::Bouncing
                        && projectile.bounces_remaining > 0
                    {
                        // Simple normal estimation (up if hitting from above)
                        let dy = projectile.position.1 - old_pos.1;
                        let normal = if dy > 0.0 { (0.0, -1.0) } else { (0.0, 1.0) };
                        projectile.position = old_pos;
                        projectile.bounce(normal);
                    } else {
                        projectile.state = ProjectileState::Embedded;
                        collisions.push(collision);
                    }
                }
            }
        }

        // Remove destroyed projectiles
        self.projectiles.retain(|_, p| !p.should_remove());

        collisions
    }

    /// Check collision with entities.
    pub fn check_entity_collisions<F>(
        &mut self,
        entity_positions: &[(EntityId, (f32, f32), f32)],
        mut on_hit: F,
    ) -> Vec<ProjectileCollision>
    where
        F: FnMut(&Projectile, EntityId) -> bool,
    {
        let mut collisions = Vec::new();

        for projectile in self.projectiles.values_mut() {
            if !projectile.is_active() {
                continue;
            }

            for &(entity_id, pos, radius) in entity_positions {
                // Skip owner
                if projectile.owner == Some(entity_id) {
                    continue;
                }

                // Circle-circle collision
                let dx = projectile.position.0 - pos.0;
                let dy = projectile.position.1 - pos.1;
                let dist_sq = dx * dx + dy * dy;
                let min_dist = projectile.radius + radius;

                if dist_sq < min_dist * min_dist {
                    let collision = ProjectileCollision {
                        collision_type: CollisionType::Entity,
                        projectile_id: projectile.id,
                        position: projectile.position,
                        velocity: projectile.velocity,
                        entity_id: Some(entity_id),
                        damage: projectile.effective_damage(),
                        destroyed: !projectile.piercing
                            || projectile.pierce_count >= projectile.max_pierce,
                    };

                    // Callback to handle hit
                    if on_hit(projectile, entity_id) {
                        collisions.push(collision);

                        if projectile.piercing {
                            if !projectile.pierce() {
                                break;
                            }
                        } else {
                            projectile.state = ProjectileState::Destroyed;
                            break;
                        }
                    }
                }
            }
        }

        collisions
    }

    /// Apply homing to all homing projectiles.
    pub fn apply_homing(&mut self, target_positions: &HashMap<EntityId, (f32, f32)>, dt: f32) {
        for projectile in self.projectiles.values_mut() {
            if projectile.projectile_type != ProjectileType::Homing {
                continue;
            }
            if let Some(target_id) = projectile.target {
                if let Some(&target_pos) = target_positions.get(&target_id) {
                    projectile.apply_homing(target_pos, dt);
                }
            }
        }
    }

    /// Get all active projectiles for rendering.
    #[must_use]
    pub fn get_render_instances(
        &self,
        sprite_lookup: impl Fn(&Projectile) -> (f32, f32, f32, f32),
    ) -> Vec<ProjectileInstance> {
        self.projectiles
            .values()
            .filter(|p| p.is_active())
            .map(|p| ProjectileInstance::from_projectile(p, sprite_lookup(p)))
            .collect()
    }

    /// Get count of active projectiles.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.projectiles.values().filter(|p| p.is_active()).count()
    }

    /// Get total projectile count.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.projectiles.len()
    }

    /// Clear all projectiles.
    pub fn clear(&mut self) {
        self.projectiles.clear();
    }
}

/// Trajectory prediction utilities.
pub struct TrajectoryPredictor;

impl TrajectoryPredictor {
    /// Predict landing position for an arc projectile.
    #[must_use]
    pub fn predict_landing(
        start: (f32, f32),
        velocity: (f32, f32),
        gravity: f32,
        ground_y: f32,
    ) -> Option<(f32, f32)> {
        // Quadratic formula for time to reach ground_y
        // y = start.1 + vy*t + 0.5*g*t^2
        // 0.5*g*t^2 + vy*t + (start.1 - ground_y) = 0
        let a = 0.5 * gravity;
        let b = velocity.1;
        let c = start.1 - ground_y;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let t1 = (-b + sqrt_d) / (2.0 * a);
        let t2 = (-b - sqrt_d) / (2.0 * a);

        // Use the larger positive time (projectile goes up first if thrown upward)
        let t = t1.max(t2);

        if t > 0.0 {
            Some((start.0 + velocity.0 * t, ground_y))
        } else {
            None
        }
    }

    /// Calculate launch velocity for a target position.
    #[must_use]
    pub fn calculate_launch_velocity(
        start: (f32, f32),
        target: (f32, f32),
        speed: f32,
        gravity: f32,
        prefer_high_arc: bool,
    ) -> Option<(f32, f32)> {
        let dx = target.0 - start.0;
        let dy = target.1 - start.1;
        let dist = dx.abs();

        if dist < 0.001 {
            return Some((0.0, -speed)); // Directly above/below
        }

        // Solve for launch angle
        let v2 = speed * speed;
        let v4 = v2 * v2;
        let g = gravity;

        let discriminant = v4 - g * (g * dist * dist + 2.0 * dy * v2);
        if discriminant < 0.0 {
            return None; // Target unreachable
        }

        let sqrt_d = discriminant.sqrt();
        let angle1 = ((v2 + sqrt_d) / (g * dist)).atan();
        let angle2 = ((v2 - sqrt_d) / (g * dist)).atan();

        let angle = if prefer_high_arc {
            angle1.max(angle2)
        } else {
            angle1.min(angle2)
        };

        let sign = dx.signum();
        Some((angle.cos() * speed * sign, -angle.sin() * speed))
    }

    /// Get trajectory points for visualization.
    #[must_use]
    pub fn get_trajectory_points(
        start: (f32, f32),
        velocity: (f32, f32),
        gravity: f32,
        time_step: f32,
        max_points: usize,
    ) -> Vec<(f32, f32)> {
        let mut points = Vec::with_capacity(max_points);
        let mut pos = start;
        let mut vel = velocity;

        for _ in 0..max_points {
            points.push(pos);
            vel.1 += gravity * time_step;
            pos.0 += vel.0 * time_step;
            pos.1 += vel.1 * time_step;
        }

        points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projectile_creation() {
        let proj = Projectile::new((0.0, 0.0), (100.0, -50.0));
        assert_eq!(proj.position, (0.0, 0.0));
        assert_eq!(proj.velocity, (100.0, -50.0));
        assert!(proj.is_active());
    }

    #[test]
    fn test_arrow_factory() {
        let arrow = Projectile::arrow((10.0, 20.0), (1.0, 0.0), 500.0);
        assert_eq!(arrow.projectile_type, ProjectileType::Arc);
        assert_eq!(arrow.damage, 25.0);
        assert!((arrow.speed() - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_spell_factory() {
        let spell = Projectile::spell((0.0, 0.0), (0.0, 1.0), 300.0);
        assert_eq!(spell.projectile_type, ProjectileType::Straight);
        assert_eq!(spell.gravity_scale, 0.0);
        assert_eq!(spell.drag, 0.0);
    }

    #[test]
    fn test_projectile_update_gravity() {
        let mut proj = Projectile::new((0.0, 0.0), (100.0, 0.0))
            .with_type(ProjectileType::Arc)
            .with_gravity(1.0)
            .with_drag(0.0);

        proj.update(1.0, 100.0);
        assert!(proj.velocity.1 > 0.0); // Gravity pulls down (positive Y)
        assert!(proj.position.1 > 0.0);
    }

    #[test]
    fn test_projectile_update_straight() {
        let mut proj = Projectile::spell((0.0, 0.0), (1.0, 0.0), 100.0);
        proj.update(1.0, 100.0);
        assert!((proj.velocity.1).abs() < 0.001); // No gravity effect
        assert!((proj.position.0 - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_projectile_lifetime() {
        let mut proj = Projectile::new((0.0, 0.0), (0.0, 0.0)).with_lifetime(1.0);
        proj.update(0.5, 0.0);
        assert!(proj.is_active());
        proj.update(0.6, 0.0);
        assert!(!proj.is_active());
    }

    #[test]
    fn test_projectile_bounce() {
        let mut proj = Projectile::new((0.0, 0.0), (100.0, 100.0)).with_bounces(2, 0.8);

        proj.bounce((0.0, -1.0));
        assert_eq!(proj.bounces_remaining, 1);
        assert!(proj.velocity.1 < 0.0); // Reflected upward
    }

    #[test]
    fn test_projectile_pierce() {
        let mut proj = Projectile::new((0.0, 0.0), (100.0, 0.0)).with_piercing(2);

        assert!(proj.pierce());
        assert_eq!(proj.pierce_count, 1);
        assert!(proj.pierce());
        assert_eq!(proj.pierce_count, 2);
        assert!(!proj.pierce()); // Max reached
    }

    #[test]
    fn test_projectile_manager() {
        let mut manager = ProjectileManager::new();
        let id = manager.spawn(Projectile::arrow((0.0, 0.0), (1.0, 0.0), 100.0));

        assert_eq!(manager.active_count(), 1);
        assert!(manager.get(id).is_some());

        manager.update(0.1);
        let proj = manager.get(id).expect("projectile should exist");
        assert!(proj.position.0 > 0.0);
    }

    #[test]
    fn test_projectile_instance_size() {
        assert_eq!(ProjectileInstance::SIZE, 48);
    }

    #[test]
    fn test_trajectory_predictor() {
        // Start at height 100, shoot horizontally with negative gravity pulling down
        let landing = TrajectoryPredictor::predict_landing(
            (0.0, 100.0),
            (100.0, 0.0),
            -980.0, // Gravity should be negative (pulling down)
            0.0,
        );
        assert!(landing.is_some());
        let (x, y) = landing.expect("should have landing");
        assert!(x > 0.0);
        assert!((y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_trajectory_points() {
        let points =
            TrajectoryPredictor::get_trajectory_points((0.0, 0.0), (100.0, -50.0), 980.0, 0.1, 10);
        assert_eq!(points.len(), 10);
        assert_eq!(points[0], (0.0, 0.0));
    }

    #[test]
    fn test_homing_projectile() {
        let mut proj = Projectile::homing((0.0, 0.0), 1, 100.0);
        assert_eq!(proj.projectile_type, ProjectileType::Homing);

        proj.apply_homing((100.0, 0.0), 0.1);
        let dir = proj.direction();
        assert!(dir.0 > 0.0); // Should turn toward target
    }

    #[test]
    fn test_effective_damage() {
        let proj = Projectile::new((0.0, 0.0), (0.0, 0.0)).with_damage(100.0);
        assert_eq!(proj.effective_damage(), 100.0);
    }

    #[test]
    fn test_projectile_type_from_u8() {
        assert_eq!(ProjectileType::from_u8(0), Some(ProjectileType::Arc));
        assert_eq!(ProjectileType::from_u8(1), Some(ProjectileType::Straight));
        assert_eq!(ProjectileType::from_u8(99), None);
    }

    #[test]
    fn test_projectile_type_properties() {
        assert!(ProjectileType::Arc.has_gravity());
        assert!(!ProjectileType::Straight.has_gravity());
        assert!(ProjectileType::Arc.has_drag());
        assert!(!ProjectileType::Straight.has_drag());
    }
}
