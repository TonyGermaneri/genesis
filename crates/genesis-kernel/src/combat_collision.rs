//! Combat collision system for hitbox/hurtbox interactions.
//!
//! Provides collision detection for combat systems including:
//! - Hitboxes (attack areas) with shape, offset, and active frames
//! - Hurtboxes (damageable areas) for entities
//! - Layer-based collision filtering
//! - Overlap detection and collision results
//!
//! # Example
//!
//! ```
//! use genesis_kernel::combat_collision::{
//!     Hitbox, Hurtbox, HitboxShape, CollisionLayer, CombatCollider,
//! };
//!
//! let hitbox = Hitbox::new(HitboxShape::Circle { radius: 32.0 })
//!     .with_offset(16.0, 0.0)
//!     .with_active_frames(5, 15)
//!     .with_layer(CollisionLayer::PLAYER_ATTACK);
//!
//! let hurtbox = Hurtbox::new(HitboxShape::Rectangle { width: 24.0, height: 48.0 })
//!     .with_layer(CollisionLayer::ENEMY);
//!
//! let collider = CombatCollider::new();
//! if let Some(hit) = collider.check_collision(&hitbox, (100.0, 100.0), &hurtbox, (120.0, 100.0), 10) {
//!     println!("Hit detected with penetration: {}", hit.penetration);
//! }
//! ```

use std::collections::HashMap;

/// Type alias for entity identifiers.
pub type EntityId = u64;

/// Collision layer flags for filtering interactions.
#[allow(non_snake_case)]
pub mod CollisionLayer {
    /// Layer flag type.
    pub type Flags = u32;

    /// Player character attacks.
    pub const PLAYER_ATTACK: Flags = 1 << 0;
    /// Enemy attacks.
    pub const ENEMY_ATTACK: Flags = 1 << 1;
    /// Player hurtbox (can be damaged by enemies).
    pub const PLAYER: Flags = 1 << 2;
    /// Enemy hurtbox (can be damaged by player).
    pub const ENEMY: Flags = 1 << 3;
    /// Neutral/environmental damage.
    pub const ENVIRONMENTAL: Flags = 1 << 4;
    /// Projectile layer.
    pub const PROJECTILE: Flags = 1 << 5;
    /// Friendly fire enabled.
    pub const FRIENDLY: Flags = 1 << 6;
    /// Boss attacks (may have special properties).
    pub const BOSS_ATTACK: Flags = 1 << 7;
    /// Invincible (no damage taken).
    pub const INVINCIBLE: Flags = 1 << 8;
    /// All layers.
    pub const ALL: Flags = 0xFFFF_FFFF;
    /// No layers.
    pub const NONE: Flags = 0;

    /// Default interaction mask: player attacks hit enemies.
    pub const PLAYER_VS_ENEMY: Flags = PLAYER_ATTACK | ENEMY;
    /// Default interaction mask: enemy attacks hit player.
    pub const ENEMY_VS_PLAYER: Flags = ENEMY_ATTACK | PLAYER;
}

/// Shape of a hitbox or hurtbox.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitboxShape {
    /// Circular hitbox.
    Circle {
        /// Radius in world units.
        radius: f32,
    },
    /// Rectangular hitbox.
    Rectangle {
        /// Width in world units.
        width: f32,
        /// Height in world units.
        height: f32,
    },
    /// Capsule (rounded rectangle) hitbox.
    Capsule {
        /// Width of the capsule.
        width: f32,
        /// Height of the capsule.
        height: f32,
    },
    /// Sector/cone hitbox for sweeping attacks.
    Sector {
        /// Radius of the sector.
        radius: f32,
        /// Angle in radians.
        angle: f32,
        /// Direction in radians (0 = right).
        direction: f32,
    },
}

impl Default for HitboxShape {
    fn default() -> Self {
        Self::Circle { radius: 16.0 }
    }
}

impl HitboxShape {
    /// Create a circular shape.
    #[must_use]
    pub const fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    /// Create a rectangular shape.
    #[must_use]
    pub const fn rectangle(width: f32, height: f32) -> Self {
        Self::Rectangle { width, height }
    }

    /// Create a capsule shape.
    #[must_use]
    pub const fn capsule(width: f32, height: f32) -> Self {
        Self::Capsule { width, height }
    }

    /// Create a sector shape.
    #[must_use]
    pub const fn sector(radius: f32, angle: f32, direction: f32) -> Self {
        Self::Sector {
            radius,
            angle,
            direction,
        }
    }

    /// Get the bounding radius for broad-phase collision.
    #[must_use]
    pub fn bounding_radius(&self) -> f32 {
        match self {
            Self::Circle { radius } | Self::Sector { radius, .. } => *radius,
            Self::Rectangle { width, height } | Self::Capsule { width, height } => {
                (width * width + height * height).sqrt() * 0.5
            },
        }
    }

    /// Get the area of the shape.
    #[must_use]
    pub fn area(&self) -> f32 {
        match self {
            Self::Circle { radius } => std::f32::consts::PI * radius * radius,
            Self::Rectangle { width, height } => width * height,
            Self::Capsule { width, height } => {
                // Rectangle + two semicircles
                let r = width.min(*height) * 0.5;
                (width - 2.0 * r).max(0.0) * height + std::f32::consts::PI * r * r
            },
            Self::Sector { radius, angle, .. } => 0.5 * radius * radius * angle,
        }
    }
}

/// Active frame range for hitboxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameRange {
    /// First active frame (inclusive).
    pub start: u16,
    /// Last active frame (inclusive).
    pub end: u16,
}

impl FrameRange {
    /// Create a new frame range.
    #[must_use]
    pub const fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    /// Check if a frame is within the active range.
    #[must_use]
    pub const fn is_active(&self, frame: u16) -> bool {
        frame >= self.start && frame <= self.end
    }

    /// Get the duration in frames.
    #[must_use]
    pub const fn duration(&self) -> u16 {
        if self.end >= self.start {
            self.end - self.start + 1
        } else {
            0
        }
    }

    /// Always active (0 to max).
    pub const ALWAYS: Self = Self {
        start: 0,
        end: u16::MAX,
    };
}

/// Hitbox represents an attack area.
#[derive(Debug, Clone)]
pub struct Hitbox {
    /// Unique identifier for this hitbox.
    pub id: u32,
    /// Shape of the hitbox.
    pub shape: HitboxShape,
    /// Offset from entity center (x, y).
    pub offset: (f32, f32),
    /// Active frame range.
    pub active_frames: FrameRange,
    /// Collision layer flags.
    pub layer: CollisionLayer::Flags,
    /// Layers this hitbox can hit.
    pub target_layers: CollisionLayer::Flags,
    /// Damage multiplier.
    pub damage_multiplier: f32,
    /// Knockback force.
    pub knockback: f32,
    /// Knockback angle (radians, 0 = right).
    pub knockback_angle: f32,
    /// Whether to use attacker's facing for knockback direction.
    pub knockback_from_center: bool,
    /// Stun duration in frames.
    pub stun_frames: u16,
    /// Whether this hitbox can hit multiple times.
    pub multi_hit: bool,
    /// Frames between multi-hits.
    pub multi_hit_interval: u16,
    /// Priority for trading hits (higher wins).
    pub priority: i8,
    /// Whether this hitbox is currently enabled.
    pub enabled: bool,
}

impl Default for Hitbox {
    fn default() -> Self {
        Self {
            id: 0,
            shape: HitboxShape::default(),
            offset: (0.0, 0.0),
            active_frames: FrameRange::ALWAYS,
            layer: CollisionLayer::PLAYER_ATTACK,
            target_layers: CollisionLayer::ENEMY,
            damage_multiplier: 1.0,
            knockback: 100.0,
            knockback_angle: 0.0,
            knockback_from_center: true,
            stun_frames: 10,
            multi_hit: false,
            multi_hit_interval: 10,
            priority: 0,
            enabled: true,
        }
    }
}

impl Hitbox {
    /// Create a new hitbox with the given shape.
    #[must_use]
    pub fn new(shape: HitboxShape) -> Self {
        Self {
            shape,
            ..Default::default()
        }
    }

    /// Set the hitbox offset.
    #[must_use]
    pub const fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset = (x, y);
        self
    }

    /// Set the active frame range.
    #[must_use]
    pub const fn with_active_frames(mut self, start: u16, end: u16) -> Self {
        self.active_frames = FrameRange::new(start, end);
        self
    }

    /// Set the collision layer.
    #[must_use]
    pub const fn with_layer(mut self, layer: CollisionLayer::Flags) -> Self {
        self.layer = layer;
        self
    }

    /// Set the target layers.
    #[must_use]
    pub const fn with_target_layers(mut self, layers: CollisionLayer::Flags) -> Self {
        self.target_layers = layers;
        self
    }

    /// Set damage multiplier.
    #[must_use]
    pub const fn with_damage_multiplier(mut self, multiplier: f32) -> Self {
        self.damage_multiplier = multiplier;
        self
    }

    /// Set knockback parameters.
    #[must_use]
    pub const fn with_knockback(mut self, force: f32, angle: f32) -> Self {
        self.knockback = force;
        self.knockback_angle = angle;
        self
    }

    /// Set stun duration.
    #[must_use]
    pub const fn with_stun(mut self, frames: u16) -> Self {
        self.stun_frames = frames;
        self
    }

    /// Enable multi-hit.
    #[must_use]
    pub const fn with_multi_hit(mut self, interval: u16) -> Self {
        self.multi_hit = true;
        self.multi_hit_interval = interval;
        self
    }

    /// Set priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: i8) -> Self {
        self.priority = priority;
        self
    }

    /// Set the hitbox ID.
    #[must_use]
    pub const fn with_id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    /// Check if the hitbox is active on a given frame.
    #[must_use]
    pub const fn is_active(&self, frame: u16) -> bool {
        self.enabled && self.active_frames.is_active(frame)
    }

    /// Get world position of hitbox center.
    #[must_use]
    pub const fn world_position(&self, entity_pos: (f32, f32)) -> (f32, f32) {
        (entity_pos.0 + self.offset.0, entity_pos.1 + self.offset.1)
    }

    /// Check if this hitbox can interact with a layer.
    #[must_use]
    pub const fn can_hit_layer(&self, target_layer: CollisionLayer::Flags) -> bool {
        (self.target_layers & target_layer) != 0
    }
}

/// Hurtbox represents a damageable area.
#[derive(Debug, Clone)]
pub struct Hurtbox {
    /// Unique identifier.
    pub id: u32,
    /// Shape of the hurtbox.
    pub shape: HitboxShape,
    /// Offset from entity center.
    pub offset: (f32, f32),
    /// Collision layer flags.
    pub layer: CollisionLayer::Flags,
    /// Damage reduction multiplier (1.0 = full damage).
    pub damage_reduction: f32,
    /// Whether this hurtbox is currently enabled.
    pub enabled: bool,
    /// Invincibility frames remaining.
    pub invincibility_frames: u16,
}

impl Default for Hurtbox {
    fn default() -> Self {
        Self {
            id: 0,
            shape: HitboxShape::default(),
            offset: (0.0, 0.0),
            layer: CollisionLayer::PLAYER,
            damage_reduction: 1.0,
            enabled: true,
            invincibility_frames: 0,
        }
    }
}

impl Hurtbox {
    /// Create a new hurtbox with the given shape.
    #[must_use]
    pub fn new(shape: HitboxShape) -> Self {
        Self {
            shape,
            ..Default::default()
        }
    }

    /// Set the hurtbox offset.
    #[must_use]
    pub const fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset = (x, y);
        self
    }

    /// Set the collision layer.
    #[must_use]
    pub const fn with_layer(mut self, layer: CollisionLayer::Flags) -> Self {
        self.layer = layer;
        self
    }

    /// Set damage reduction.
    #[must_use]
    pub const fn with_damage_reduction(mut self, reduction: f32) -> Self {
        self.damage_reduction = reduction;
        self
    }

    /// Set the hurtbox ID.
    #[must_use]
    pub const fn with_id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    /// Check if the hurtbox can be damaged.
    #[must_use]
    pub const fn is_vulnerable(&self) -> bool {
        self.enabled && self.invincibility_frames == 0
    }

    /// Get world position of hurtbox center.
    #[must_use]
    pub const fn world_position(&self, entity_pos: (f32, f32)) -> (f32, f32) {
        (entity_pos.0 + self.offset.0, entity_pos.1 + self.offset.1)
    }

    /// Apply invincibility frames.
    pub fn apply_invincibility(&mut self, frames: u16) {
        self.invincibility_frames = self.invincibility_frames.max(frames);
    }

    /// Update invincibility (call each frame).
    pub fn tick(&mut self) {
        self.invincibility_frames = self.invincibility_frames.saturating_sub(1);
    }
}

/// Result of a collision check.
#[derive(Debug, Clone)]
pub struct CollisionResult {
    /// Hitbox ID that caused the hit.
    pub hitbox_id: u32,
    /// Hurtbox ID that was hit.
    pub hurtbox_id: u32,
    /// Contact point (world coordinates).
    pub contact_point: (f32, f32),
    /// Collision normal (from hurtbox to hitbox).
    pub normal: (f32, f32),
    /// Penetration depth.
    pub penetration: f32,
    /// Calculated damage multiplier (hitbox * hurtbox reduction).
    pub damage_multiplier: f32,
    /// Knockback vector (x, y).
    pub knockback: (f32, f32),
    /// Stun frames to apply.
    pub stun_frames: u16,
}

impl CollisionResult {
    /// Get knockback magnitude.
    #[must_use]
    pub fn knockback_magnitude(&self) -> f32 {
        (self.knockback.0 * self.knockback.0 + self.knockback.1 * self.knockback.1).sqrt()
    }
}

/// Combat collision detection system.
#[derive(Debug, Default)]
pub struct CombatCollider {
    /// Track which hitbox/hurtbox pairs have already collided.
    hit_pairs: HashMap<(u32, u32), u16>,
}

impl CombatCollider {
    /// Create a new combat collider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear hit tracking (call at start of new attack).
    pub fn clear_hit_tracking(&mut self) {
        self.hit_pairs.clear();
    }

    /// Check if a hitbox has already hit a hurtbox.
    #[must_use]
    pub fn has_already_hit(&self, hitbox_id: u32, hurtbox_id: u32) -> bool {
        self.hit_pairs.contains_key(&(hitbox_id, hurtbox_id))
    }

    /// Record a hit between hitbox and hurtbox.
    pub fn record_hit(&mut self, hitbox_id: u32, hurtbox_id: u32, frame: u16) {
        self.hit_pairs.insert((hitbox_id, hurtbox_id), frame);
    }

    /// Update hit tracking for multi-hit (returns pairs that can hit again).
    pub fn update_multi_hit(&mut self, current_frame: u16, interval: u16) -> Vec<(u32, u32)> {
        let mut can_hit_again = Vec::new();
        for (&pair, &hit_frame) in &self.hit_pairs {
            if current_frame >= hit_frame + interval {
                can_hit_again.push(pair);
            }
        }
        // Remove pairs that can hit again
        for pair in &can_hit_again {
            self.hit_pairs.remove(pair);
        }
        can_hit_again
    }

    /// Check collision between a hitbox and hurtbox.
    #[must_use]
    pub fn check_collision(
        &self,
        hitbox: &Hitbox,
        hitbox_pos: (f32, f32),
        hurtbox: &Hurtbox,
        hurtbox_pos: (f32, f32),
        current_frame: u16,
    ) -> Option<CollisionResult> {
        // Check if hitbox is active
        if !hitbox.is_active(current_frame) {
            return None;
        }

        // Check if hurtbox is vulnerable
        if !hurtbox.is_vulnerable() {
            return None;
        }

        // Check layer compatibility
        if !hitbox.can_hit_layer(hurtbox.layer) {
            return None;
        }

        // Check if already hit (and not multi-hit)
        if !hitbox.multi_hit && self.has_already_hit(hitbox.id, hurtbox.id) {
            return None;
        }

        // Get world positions
        let hitbox_world = hitbox.world_position(hitbox_pos);
        let hurtbox_world = hurtbox.world_position(hurtbox_pos);

        // Perform shape-specific collision detection
        let collision =
            self.check_shapes_overlap(&hitbox.shape, hitbox_world, &hurtbox.shape, hurtbox_world)?;

        // Calculate knockback direction
        let knockback = if hitbox.knockback_from_center {
            let dx = hurtbox_world.0 - hitbox_world.0;
            let dy = hurtbox_world.1 - hitbox_world.1;
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            let nx = dx / len;
            let ny = dy / len;
            (nx * hitbox.knockback, ny * hitbox.knockback)
        } else {
            let cos = hitbox.knockback_angle.cos();
            let sin = hitbox.knockback_angle.sin();
            (cos * hitbox.knockback, sin * hitbox.knockback)
        };

        Some(CollisionResult {
            hitbox_id: hitbox.id,
            hurtbox_id: hurtbox.id,
            contact_point: collision.contact_point,
            normal: collision.normal,
            penetration: collision.penetration,
            damage_multiplier: hitbox.damage_multiplier * hurtbox.damage_reduction,
            knockback,
            stun_frames: hitbox.stun_frames,
        })
    }

    /// Check if two shapes overlap.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn check_shapes_overlap(
        &self,
        shape_a: &HitboxShape,
        pos_a: (f32, f32),
        shape_b: &HitboxShape,
        pos_b: (f32, f32),
    ) -> Option<ShapeOverlap> {
        match (shape_a, shape_b) {
            (HitboxShape::Circle { radius: r1 }, HitboxShape::Circle { radius: r2 }) => {
                self.circle_circle_overlap(pos_a, *r1, pos_b, *r2)
            },
            (HitboxShape::Circle { radius }, HitboxShape::Rectangle { width, height }) => {
                self.circle_rect_overlap(pos_a, *radius, pos_b, *width, *height)
            },
            (HitboxShape::Rectangle { width, height }, HitboxShape::Circle { radius }) => {
                // Swap and invert normal
                self.circle_rect_overlap(pos_b, *radius, pos_a, *width, *height)
                    .map(|mut o| {
                        o.normal = (-o.normal.0, -o.normal.1);
                        o
                    })
            },
            (
                HitboxShape::Rectangle {
                    width: w1,
                    height: h1,
                },
                HitboxShape::Rectangle {
                    width: w2,
                    height: h2,
                },
            ) => self.rect_rect_overlap(pos_a, *w1, *h1, pos_b, *w2, *h2),
            (
                HitboxShape::Sector {
                    radius,
                    angle,
                    direction,
                },
                _,
            ) => self.sector_shape_overlap(pos_a, *radius, *angle, *direction, shape_b, pos_b),
            (
                _,
                HitboxShape::Sector {
                    radius,
                    angle,
                    direction,
                },
            ) => self
                .sector_shape_overlap(pos_b, *radius, *angle, *direction, shape_a, pos_a)
                .map(|mut o| {
                    o.normal = (-o.normal.0, -o.normal.1);
                    o
                }),
            // Capsules treated as rectangles for simplicity
            (
                HitboxShape::Capsule {
                    width: w1,
                    height: h1,
                },
                shape_b,
            ) => self.check_shapes_overlap(
                &HitboxShape::Rectangle {
                    width: *w1,
                    height: *h1,
                },
                pos_a,
                shape_b,
                pos_b,
            ),
            (
                shape_a,
                HitboxShape::Capsule {
                    width: w2,
                    height: h2,
                },
            ) => self.check_shapes_overlap(
                shape_a,
                pos_a,
                &HitboxShape::Rectangle {
                    width: *w2,
                    height: *h2,
                },
                pos_b,
            ),
        }
    }

    /// Circle-circle overlap check.
    #[allow(clippy::unused_self)]
    fn circle_circle_overlap(
        &self,
        pos_a: (f32, f32),
        radius_a: f32,
        pos_b: (f32, f32),
        radius_b: f32,
    ) -> Option<ShapeOverlap> {
        let dx = pos_b.0 - pos_a.0;
        let dy = pos_b.1 - pos_a.1;
        let dist_sq = dx * dx + dy * dy;
        let min_dist = radius_a + radius_b;

        if dist_sq >= min_dist * min_dist {
            return None;
        }

        let dist = dist_sq.sqrt().max(0.001);
        let nx = dx / dist;
        let ny = dy / dist;
        let penetration = min_dist - dist;

        Some(ShapeOverlap {
            contact_point: (pos_a.0 + nx * radius_a, pos_a.1 + ny * radius_a),
            normal: (nx, ny),
            penetration,
        })
    }

    /// Circle-rectangle overlap check.
    #[allow(clippy::unused_self)]
    fn circle_rect_overlap(
        &self,
        circle_pos: (f32, f32),
        radius: f32,
        rect_pos: (f32, f32),
        width: f32,
        height: f32,
    ) -> Option<ShapeOverlap> {
        let half_w = width * 0.5;
        let half_h = height * 0.5;

        // Find closest point on rectangle to circle center
        let closest_x = circle_pos.0.clamp(rect_pos.0 - half_w, rect_pos.0 + half_w);
        let closest_y = circle_pos.1.clamp(rect_pos.1 - half_h, rect_pos.1 + half_h);

        let dx = circle_pos.0 - closest_x;
        let dy = circle_pos.1 - closest_y;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq >= radius * radius {
            return None;
        }

        let dist = dist_sq.sqrt().max(0.001);
        let nx = dx / dist;
        let ny = dy / dist;
        let penetration = radius - dist;

        Some(ShapeOverlap {
            contact_point: (closest_x, closest_y),
            normal: (nx, ny),
            penetration,
        })
    }

    /// Rectangle-rectangle overlap check (AABB).
    #[allow(clippy::unused_self)]
    fn rect_rect_overlap(
        &self,
        pos_a: (f32, f32),
        width_a: f32,
        height_a: f32,
        pos_b: (f32, f32),
        width_b: f32,
        height_b: f32,
    ) -> Option<ShapeOverlap> {
        let half_wa = width_a * 0.5;
        let half_ha = height_a * 0.5;
        let half_wb = width_b * 0.5;
        let half_hb = height_b * 0.5;

        let dx = pos_b.0 - pos_a.0;
        let dy = pos_b.1 - pos_a.1;

        let overlap_x = (half_wa + half_wb) - dx.abs();
        let overlap_y = (half_ha + half_hb) - dy.abs();

        if overlap_x <= 0.0 || overlap_y <= 0.0 {
            return None;
        }

        // Use minimum overlap axis
        let (normal, penetration) = if overlap_x < overlap_y {
            let nx = if dx > 0.0 { 1.0 } else { -1.0 };
            ((nx, 0.0), overlap_x)
        } else {
            let ny = if dy > 0.0 { 1.0 } else { -1.0 };
            ((0.0, ny), overlap_y)
        };

        // Contact point at center of overlap region
        let contact_x = pos_a.0 + dx.signum() * (half_wa - overlap_x * 0.5);
        let contact_y = pos_a.1 + dy.signum() * (half_ha - overlap_y * 0.5);

        Some(ShapeOverlap {
            contact_point: (contact_x, contact_y),
            normal,
            penetration,
        })
    }

    /// Sector collision with another shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn sector_shape_overlap(
        &self,
        sector_pos: (f32, f32),
        radius: f32,
        angle: f32,
        direction: f32,
        other_shape: &HitboxShape,
        other_pos: (f32, f32),
    ) -> Option<ShapeOverlap> {
        // First check circle collision
        let other_radius = other_shape.bounding_radius();
        let circle_result =
            self.circle_circle_overlap(sector_pos, radius, other_pos, other_radius)?;

        // Then check if within sector angle
        let dx = other_pos.0 - sector_pos.0;
        let dy = other_pos.1 - sector_pos.1;
        let to_target = dy.atan2(dx);

        // Normalize angle difference
        let mut angle_diff = to_target - direction;
        while angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }

        if angle_diff.abs() <= angle * 0.5 {
            Some(circle_result)
        } else {
            None
        }
    }
}

/// Intermediate result for shape overlap.
#[derive(Debug, Clone)]
struct ShapeOverlap {
    contact_point: (f32, f32),
    normal: (f32, f32),
    penetration: f32,
}

/// Manager for multiple hitboxes and hurtboxes.
#[derive(Debug, Default)]
pub struct CombatBoxManager {
    hitboxes: Vec<(EntityId, Hitbox, (f32, f32))>,
    hurtboxes: Vec<(EntityId, Hurtbox, (f32, f32))>,
    collider: CombatCollider,
}

impl CombatBoxManager {
    /// Create a new combat box manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all hitboxes and hurtboxes.
    pub fn clear(&mut self) {
        self.hitboxes.clear();
        self.hurtboxes.clear();
        self.collider.clear_hit_tracking();
    }

    /// Add a hitbox for an entity.
    pub fn add_hitbox(&mut self, entity_id: EntityId, hitbox: Hitbox, position: (f32, f32)) {
        self.hitboxes.push((entity_id, hitbox, position));
    }

    /// Add a hurtbox for an entity.
    pub fn add_hurtbox(&mut self, entity_id: EntityId, hurtbox: Hurtbox, position: (f32, f32)) {
        self.hurtboxes.push((entity_id, hurtbox, position));
    }

    /// Update hitbox position.
    pub fn update_hitbox_position(&mut self, hitbox_id: u32, new_pos: (f32, f32)) {
        for (_, hitbox, pos) in &mut self.hitboxes {
            if hitbox.id == hitbox_id {
                *pos = new_pos;
                return;
            }
        }
    }

    /// Update hurtbox position.
    pub fn update_hurtbox_position(&mut self, hurtbox_id: u32, new_pos: (f32, f32)) {
        for (_, hurtbox, pos) in &mut self.hurtboxes {
            if hurtbox.id == hurtbox_id {
                *pos = new_pos;
                return;
            }
        }
    }

    /// Process all collisions for the current frame.
    pub fn process_collisions(
        &mut self,
        current_frame: u16,
    ) -> Vec<(EntityId, EntityId, CollisionResult)> {
        let mut results = Vec::new();

        for (attacker_id, hitbox, hitbox_pos) in &self.hitboxes {
            for (defender_id, hurtbox, hurtbox_pos) in &self.hurtboxes {
                // Skip self-collision
                if attacker_id == defender_id {
                    continue;
                }

                if let Some(result) = self.collider.check_collision(
                    hitbox,
                    *hitbox_pos,
                    hurtbox,
                    *hurtbox_pos,
                    current_frame,
                ) {
                    // Record the hit
                    self.collider
                        .record_hit(hitbox.id, hurtbox.id, current_frame);
                    results.push((*attacker_id, *defender_id, result));
                }
            }
        }

        results
    }

    /// Get number of active hitboxes.
    #[must_use]
    pub fn hitbox_count(&self) -> usize {
        self.hitboxes.len()
    }

    /// Get number of active hurtboxes.
    #[must_use]
    pub fn hurtbox_count(&self) -> usize {
        self.hurtboxes.len()
    }
}

/// Builder for creating hitbox sequences (animation-linked).
#[derive(Debug, Default)]
pub struct HitboxSequenceBuilder {
    hitboxes: Vec<Hitbox>,
    next_id: u32,
}

impl HitboxSequenceBuilder {
    /// Create a new sequence builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a hitbox to the sequence.
    pub fn add(&mut self, shape: HitboxShape, start_frame: u16, end_frame: u16) -> &mut Hitbox {
        let id = self.next_id;
        self.next_id += 1;
        self.hitboxes.push(
            Hitbox::new(shape)
                .with_id(id)
                .with_active_frames(start_frame, end_frame),
        );
        self.hitboxes.last_mut().expect("just pushed")
    }

    /// Build the sequence.
    #[must_use]
    pub fn build(self) -> Vec<Hitbox> {
        self.hitboxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_range() {
        let range = FrameRange::new(5, 15);
        assert!(!range.is_active(4));
        assert!(range.is_active(5));
        assert!(range.is_active(10));
        assert!(range.is_active(15));
        assert!(!range.is_active(16));
        assert_eq!(range.duration(), 11);
    }

    #[test]
    fn test_hitbox_creation() {
        let hitbox = Hitbox::new(HitboxShape::circle(32.0))
            .with_offset(16.0, 0.0)
            .with_active_frames(5, 15)
            .with_layer(CollisionLayer::PLAYER_ATTACK)
            .with_knockback(200.0, 0.0);

        assert_eq!(hitbox.offset, (16.0, 0.0));
        assert!(hitbox.is_active(10));
        assert!(!hitbox.is_active(20));
        assert_eq!(hitbox.knockback, 200.0);
    }

    #[test]
    fn test_hurtbox_creation() {
        let mut hurtbox = Hurtbox::new(HitboxShape::rectangle(24.0, 48.0))
            .with_layer(CollisionLayer::ENEMY)
            .with_damage_reduction(0.8);

        assert!(hurtbox.is_vulnerable());
        hurtbox.apply_invincibility(10);
        assert!(!hurtbox.is_vulnerable());
        for _ in 0..10 {
            hurtbox.tick();
        }
        assert!(hurtbox.is_vulnerable());
    }

    #[test]
    fn test_circle_collision() {
        let collider = CombatCollider::new();
        let hitbox = Hitbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_target_layers(CollisionLayer::ENEMY);
        let hurtbox = Hurtbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_layer(CollisionLayer::ENEMY);

        // Overlapping
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (30.0, 0.0), 0);
        assert!(result.is_some());
        let r = result.expect("should have result");
        assert!(r.penetration > 0.0);

        // Not overlapping
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (50.0, 0.0), 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_rect_collision() {
        let collider = CombatCollider::new();
        let hitbox = Hitbox::new(HitboxShape::rectangle(40.0, 40.0))
            .with_id(1)
            .with_target_layers(CollisionLayer::ENEMY);
        let hurtbox = Hurtbox::new(HitboxShape::rectangle(40.0, 40.0))
            .with_id(1)
            .with_layer(CollisionLayer::ENEMY);

        // Overlapping
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (30.0, 0.0), 0);
        assert!(result.is_some());

        // Not overlapping
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (50.0, 0.0), 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_layer_filtering() {
        let collider = CombatCollider::new();
        let hitbox = Hitbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_target_layers(CollisionLayer::ENEMY);
        let hurtbox = Hurtbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_layer(CollisionLayer::PLAYER); // Player, not enemy

        // Should not collide due to layer mismatch
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (10.0, 0.0), 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_hit_tracking() {
        let mut collider = CombatCollider::new();
        let hitbox = Hitbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_target_layers(CollisionLayer::ENEMY);
        let hurtbox = Hurtbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_layer(CollisionLayer::ENEMY);

        // First hit should succeed
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (10.0, 0.0), 0);
        assert!(result.is_some());
        collider.record_hit(hitbox.id, hurtbox.id, 0);

        // Second hit should fail (already hit)
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (10.0, 0.0), 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_combat_box_manager() {
        let mut manager = CombatBoxManager::new();

        let hitbox = Hitbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_target_layers(CollisionLayer::ENEMY);
        let hurtbox = Hurtbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_layer(CollisionLayer::ENEMY);

        manager.add_hitbox(1, hitbox, (0.0, 0.0));
        manager.add_hurtbox(2, hurtbox, (10.0, 0.0));

        let results = manager.process_collisions(0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1); // Attacker ID
        assert_eq!(results[0].1, 2); // Defender ID
    }

    #[test]
    fn test_hitbox_sequence_builder() {
        let mut builder = HitboxSequenceBuilder::new();
        builder.add(HitboxShape::circle(20.0), 0, 5);
        builder.add(HitboxShape::circle(30.0), 6, 10);
        builder.add(HitboxShape::circle(25.0), 11, 15);

        let sequence = builder.build();
        assert_eq!(sequence.len(), 3);
        assert!(sequence[0].is_active(3));
        assert!(!sequence[0].is_active(6));
        assert!(sequence[1].is_active(8));
    }

    #[test]
    fn test_shape_bounding_radius() {
        let circle = HitboxShape::circle(10.0);
        assert_eq!(circle.bounding_radius(), 10.0);

        let rect = HitboxShape::rectangle(6.0, 8.0);
        assert!((rect.bounding_radius() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_knockback_calculation() {
        let collider = CombatCollider::new();
        let hitbox = Hitbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_knockback(100.0, 0.0)
            .with_target_layers(CollisionLayer::ENEMY);
        let hurtbox = Hurtbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_layer(CollisionLayer::ENEMY);

        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (30.0, 0.0), 0);
        assert!(result.is_some());
        let r = result.expect("should have result");
        assert!(r.knockback.0 > 0.0); // Knockback pushes right
        assert!((r.knockback_magnitude() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_sector_collision() {
        let collider = CombatCollider::new();
        let hitbox = Hitbox::new(HitboxShape::sector(50.0, std::f32::consts::PI / 2.0, 0.0))
            .with_id(1)
            .with_target_layers(CollisionLayer::ENEMY);
        let hurtbox = Hurtbox::new(HitboxShape::circle(10.0))
            .with_id(1)
            .with_layer(CollisionLayer::ENEMY);

        // Target in front (within sector)
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (30.0, 0.0), 0);
        assert!(result.is_some());

        // Target behind (outside sector)
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (-30.0, 0.0), 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_invincibility() {
        let collider = CombatCollider::new();
        let hitbox = Hitbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_target_layers(CollisionLayer::ENEMY);
        let mut hurtbox = Hurtbox::new(HitboxShape::circle(20.0))
            .with_id(1)
            .with_layer(CollisionLayer::ENEMY);

        hurtbox.apply_invincibility(5);

        // Should not hit due to invincibility
        let result = collider.check_collision(&hitbox, (0.0, 0.0), &hurtbox, (10.0, 0.0), 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_collision_layer_flags() {
        assert_eq!(CollisionLayer::PLAYER_ATTACK, 1);
        assert_eq!(CollisionLayer::ENEMY_ATTACK, 2);
        assert_eq!(
            CollisionLayer::PLAYER_VS_ENEMY,
            CollisionLayer::PLAYER_ATTACK | CollisionLayer::ENEMY
        );
    }

    #[test]
    fn test_shape_area() {
        let circle = HitboxShape::circle(10.0);
        let expected = std::f32::consts::PI * 100.0;
        assert!((circle.area() - expected).abs() < 0.01);

        let rect = HitboxShape::rectangle(10.0, 20.0);
        assert_eq!(rect.area(), 200.0);
    }
}
