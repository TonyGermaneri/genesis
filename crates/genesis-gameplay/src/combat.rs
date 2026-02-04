//! Combat system with melee and ranged attacks.
//!
//! This module provides combat mechanics including:
//! - Attack queueing and processing
//! - Damage calculation with armor and resistances
//! - Hit detection via collision queries
//! - Knockback physics
//! - Death handling

use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

use crate::physics::{CollisionQuery, AABB};

/// Combat system error types.
#[derive(Debug, Clone, Error)]
pub enum CombatError {
    /// Entity not found
    #[error("entity not found: {0:?}")]
    EntityNotFound(EntityId),
    /// Target out of range
    #[error("target out of range: distance {distance}, range {range}")]
    OutOfRange {
        /// Actual distance
        distance: f32,
        /// Required range
        range: f32,
    },
    /// No weapon equipped
    #[error("no weapon equipped")]
    NoWeapon,
    /// Attack on cooldown
    #[error("attack on cooldown: {remaining}s remaining")]
    OnCooldown {
        /// Time remaining in seconds
        remaining: f32,
    },
    /// Invalid target
    #[error("invalid target")]
    InvalidTarget,
}

/// Result type for combat operations.
pub type CombatResult<T> = Result<T, CombatError>;

/// Unique identifier for an item (for weapons).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(u32);

impl ItemId {
    /// Creates a new item ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Type of projectile for ranged attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectileType {
    /// Arrow projectile
    Arrow,
    /// Bolt projectile (crossbow)
    Bolt,
    /// Magic projectile
    Magic,
    /// Thrown weapon
    Thrown,
    /// Bullet
    Bullet,
}

/// Target for an attack.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AttackTarget {
    /// Target a specific entity
    Entity(EntityId),
    /// Target a position in the world
    Position(f32, f32),
    /// Target in a direction (for melee swings)
    Direction(f32, f32),
}

/// Type of attack being performed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AttackType {
    /// Melee attack with range and arc
    Melee {
        /// Maximum range of the attack
        range: f32,
        /// Arc angle in radians (pi = 180 degrees)
        arc: f32,
    },
    /// Ranged attack with projectile
    Ranged {
        /// Type of projectile
        projectile: ProjectileType,
        /// Projectile speed
        speed: f32,
    },
    /// Area of effect attack
    Area {
        /// Radius of the area
        radius: f32,
        /// Whether damage falls off with distance
        falloff: bool,
    },
}

impl Default for AttackType {
    fn default() -> Self {
        Self::Melee {
            range: 1.5,
            arc: std::f32::consts::PI / 2.0,
        }
    }
}

/// An intent to perform an attack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackIntent {
    /// Entity performing the attack
    pub attacker: EntityId,
    /// Target of the attack
    pub target: AttackTarget,
    /// Weapon being used (if any)
    pub weapon: Option<ItemId>,
    /// Type of attack
    pub attack_type: AttackType,
    /// Base damage for this attack
    pub base_damage: f32,
    /// Damage type
    pub damage_type: DamageType,
}

impl AttackIntent {
    /// Creates a new attack intent.
    #[must_use]
    pub fn new(attacker: EntityId, target: AttackTarget) -> Self {
        Self {
            attacker,
            target,
            weapon: None,
            attack_type: AttackType::default(),
            base_damage: 10.0,
            damage_type: DamageType::Physical,
        }
    }

    /// Sets the weapon for this attack.
    #[must_use]
    pub fn with_weapon(mut self, weapon: ItemId) -> Self {
        self.weapon = Some(weapon);
        self
    }

    /// Sets the attack type.
    #[must_use]
    pub fn with_attack_type(mut self, attack_type: AttackType) -> Self {
        self.attack_type = attack_type;
        self
    }

    /// Sets the base damage.
    #[must_use]
    pub fn with_damage(mut self, damage: f32) -> Self {
        self.base_damage = damage;
        self
    }

    /// Sets the damage type.
    #[must_use]
    pub fn with_damage_type(mut self, damage_type: DamageType) -> Self {
        self.damage_type = damage_type;
        self
    }
}

/// Type of damage dealt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    /// Physical damage (reduced by armor)
    Physical,
    /// Fire damage
    Fire,
    /// Ice/cold damage
    Ice,
    /// Electric/lightning damage
    Electric,
    /// Poison damage (may apply DoT)
    Poison,
    /// True damage (ignores armor and resistances)
    True,
}

impl Default for DamageType {
    fn default() -> Self {
        Self::Physical
    }
}

/// An event representing damage dealt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageEvent {
    /// Source entity (None for environmental damage)
    pub source: Option<EntityId>,
    /// Target entity
    pub target: EntityId,
    /// Damage amount (after calculations)
    pub damage: f32,
    /// Type of damage
    pub damage_type: DamageType,
    /// Position where damage occurred
    pub position: (f32, f32),
    /// Knockback vector (if any)
    pub knockback: Option<(f32, f32)>,
    /// Whether this was a critical hit
    pub critical: bool,
    /// Damage blocked by armor
    pub blocked: f32,
    /// Damage resisted
    pub resisted: f32,
}

impl DamageEvent {
    /// Creates a new damage event.
    #[must_use]
    pub fn new(target: EntityId, damage: f32, damage_type: DamageType) -> Self {
        Self {
            source: None,
            target,
            damage,
            damage_type,
            position: (0.0, 0.0),
            knockback: None,
            critical: false,
            blocked: 0.0,
            resisted: 0.0,
        }
    }

    /// Sets the source of the damage.
    #[must_use]
    pub fn with_source(mut self, source: EntityId) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the position of the damage.
    #[must_use]
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Sets knockback for the damage.
    #[must_use]
    pub fn with_knockback(mut self, x: f32, y: f32) -> Self {
        self.knockback = Some((x, y));
        self
    }

    /// Marks this as a critical hit.
    #[must_use]
    pub fn as_critical(mut self) -> Self {
        self.critical = true;
        self
    }

    /// Returns the total raw damage before mitigation.
    #[must_use]
    pub fn raw_damage(&self) -> f32 {
        self.damage + self.blocked + self.resisted
    }
}

/// Combat statistics for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatStats {
    /// Current health
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
    /// Armor value (reduces physical damage)
    pub armor: f32,
    /// Resistances to damage types (0.0 = none, 1.0 = immune)
    pub resistances: HashMap<DamageType, f32>,
    /// Attack speed multiplier
    pub attack_speed: f32,
    /// Damage multiplier
    pub damage_multiplier: f32,
    /// Critical hit chance (0.0 to 1.0)
    pub crit_chance: f32,
    /// Critical hit damage multiplier
    pub crit_multiplier: f32,
    /// Time until next attack allowed
    pub attack_cooldown: f32,
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            armor: 0.0,
            resistances: HashMap::new(),
            attack_speed: 1.0,
            damage_multiplier: 1.0,
            crit_chance: 0.05,
            crit_multiplier: 2.0,
            attack_cooldown: 0.0,
        }
    }
}

impl CombatStats {
    /// Creates new combat stats with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates combat stats with specified health.
    #[must_use]
    pub fn with_health(mut self, health: f32) -> Self {
        self.health = health;
        self.max_health = health;
        self
    }

    /// Sets the armor value.
    #[must_use]
    pub fn with_armor(mut self, armor: f32) -> Self {
        self.armor = armor;
        self
    }

    /// Adds a resistance.
    #[must_use]
    pub fn with_resistance(mut self, damage_type: DamageType, value: f32) -> Self {
        self.resistances.insert(damage_type, value.clamp(0.0, 1.0));
        self
    }

    /// Sets attack speed multiplier.
    #[must_use]
    pub fn with_attack_speed(mut self, speed: f32) -> Self {
        self.attack_speed = speed.max(0.1);
        self
    }

    /// Sets damage multiplier.
    #[must_use]
    pub fn with_damage_multiplier(mut self, multiplier: f32) -> Self {
        self.damage_multiplier = multiplier.max(0.0);
        self
    }

    /// Returns whether this entity is dead.
    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Returns health as a percentage (0.0 to 1.0).
    #[must_use]
    pub fn health_percent(&self) -> f32 {
        if self.max_health <= 0.0 {
            0.0
        } else {
            (self.health / self.max_health).clamp(0.0, 1.0)
        }
    }

    /// Heals the entity.
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Gets resistance for a damage type.
    #[must_use]
    pub fn get_resistance(&self, damage_type: DamageType) -> f32 {
        self.resistances.get(&damage_type).copied().unwrap_or(0.0)
    }

    /// Calculates armor damage reduction.
    /// Uses diminishing returns formula: reduction = armor / (armor + 100)
    #[must_use]
    pub fn armor_reduction(&self) -> f32 {
        if self.armor <= 0.0 {
            0.0
        } else {
            self.armor / (self.armor + 100.0)
        }
    }

    /// Updates cooldowns.
    pub fn tick(&mut self, dt: f32) {
        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
    }

    /// Checks if can attack.
    #[must_use]
    pub fn can_attack(&self) -> bool {
        self.attack_cooldown <= 0.0 && !self.is_dead()
    }

    /// Sets attack on cooldown.
    pub fn set_cooldown(&mut self, base_cooldown: f32) {
        self.attack_cooldown = base_cooldown / self.attack_speed;
    }
}

/// Weapon data for damage calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponStats {
    /// Base damage
    pub damage: f32,
    /// Damage type
    pub damage_type: DamageType,
    /// Attack type
    pub attack_type: AttackType,
    /// Attack cooldown in seconds
    pub cooldown: f32,
    /// Knockback force
    pub knockback: f32,
    /// Critical hit chance bonus
    pub crit_bonus: f32,
}

impl Default for WeaponStats {
    fn default() -> Self {
        Self {
            damage: 10.0,
            damage_type: DamageType::Physical,
            attack_type: AttackType::default(),
            cooldown: 0.5,
            knockback: 5.0,
            crit_bonus: 0.0,
        }
    }
}

impl WeaponStats {
    /// Creates new weapon stats.
    #[must_use]
    pub fn new(damage: f32) -> Self {
        Self {
            damage,
            ..Default::default()
        }
    }

    /// Sets the damage type.
    #[must_use]
    pub fn with_damage_type(mut self, damage_type: DamageType) -> Self {
        self.damage_type = damage_type;
        self
    }

    /// Sets the attack type.
    #[must_use]
    pub fn with_attack_type(mut self, attack_type: AttackType) -> Self {
        self.attack_type = attack_type;
        self
    }

    /// Sets the cooldown.
    #[must_use]
    pub fn with_cooldown(mut self, cooldown: f32) -> Self {
        self.cooldown = cooldown.max(0.1);
        self
    }

    /// Sets the knockback force.
    #[must_use]
    pub fn with_knockback(mut self, knockback: f32) -> Self {
        self.knockback = knockback;
        self
    }
}

/// Position data for an entity in combat.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CombatPosition {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Direction facing (radians)
    pub facing: f32,
}

impl CombatPosition {
    /// Creates a new combat position.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y, facing: 0.0 }
    }

    /// Sets the facing direction.
    #[must_use]
    pub const fn with_facing(mut self, facing: f32) -> Self {
        self.facing = facing;
        self
    }

    /// Calculates distance to another position.
    #[must_use]
    pub fn distance_to(&self, other: &CombatPosition) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculates angle to another position.
    #[must_use]
    pub fn angle_to(&self, other: &CombatPosition) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        dy.atan2(dx)
    }

    /// Returns a normalized direction vector to another position.
    #[must_use]
    pub fn direction_to(&self, other: &CombatPosition) -> (f32, f32) {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 0.0001 {
            (0.0, 0.0)
        } else {
            (dx / dist, dy / dist)
        }
    }

    /// Gets AABB for this position with given half-size.
    #[must_use]
    pub fn to_aabb(&self, half_width: f32, half_height: f32) -> AABB {
        AABB::new(
            self.x - half_width,
            self.y - half_height,
            self.x + half_width,
            self.y + half_height,
        )
    }
}

/// Storage for entity combat data.
pub trait CombatStorage {
    /// Gets an entity's combat stats.
    fn get_stats(&self, entity: EntityId) -> Option<&CombatStats>;
    /// Gets mutable combat stats.
    fn get_stats_mut(&mut self, entity: EntityId) -> Option<&mut CombatStats>;
    /// Gets an entity's position.
    fn get_position(&self, entity: EntityId) -> Option<CombatPosition>;
    /// Gets weapon stats for an item.
    fn get_weapon(&self, item: ItemId) -> Option<&WeaponStats>;
    /// Applies knockback to an entity.
    fn apply_knockback(&mut self, entity: EntityId, knockback: (f32, f32));
    /// Marks an entity as dead.
    fn on_death(&mut self, entity: EntityId);
}

/// Combat system managing attacks and damage.
#[derive(Debug, Default)]
pub struct CombatSystem {
    /// Queue of pending attack intents
    attack_queue: VecDeque<AttackIntent>,
    /// Damage events from last process
    damage_events: Vec<DamageEvent>,
    /// Active projectiles
    projectiles: Vec<Projectile>,
    /// Random seed for crits
    rng_state: u64,
}

/// An active projectile in flight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    /// Source entity
    pub source: EntityId,
    /// Current position
    pub position: (f32, f32),
    /// Velocity
    pub velocity: (f32, f32),
    /// Damage on hit
    pub damage: f32,
    /// Damage type
    pub damage_type: DamageType,
    /// Knockback force
    pub knockback: f32,
    /// Time to live in seconds
    pub ttl: f32,
    /// Projectile type
    pub projectile_type: ProjectileType,
    /// Hitbox radius
    pub radius: f32,
}

impl Projectile {
    /// Creates a new projectile.
    #[must_use]
    pub fn new(
        source: EntityId,
        position: (f32, f32),
        velocity: (f32, f32),
        damage: f32,
        projectile_type: ProjectileType,
    ) -> Self {
        Self {
            source,
            position,
            velocity,
            damage,
            damage_type: DamageType::Physical,
            knockback: 3.0,
            ttl: 5.0,
            projectile_type,
            radius: 0.2,
        }
    }

    /// Gets the AABB for this projectile.
    #[must_use]
    pub fn to_aabb(&self) -> AABB {
        AABB::new(
            self.position.0 - self.radius,
            self.position.1 - self.radius,
            self.position.0 + self.radius,
            self.position.1 + self.radius,
        )
    }

    /// Updates projectile position.
    pub fn update(&mut self, dt: f32) {
        self.position.0 += self.velocity.0 * dt;
        self.position.1 += self.velocity.1 * dt;
        self.ttl -= dt;
    }

    /// Checks if projectile is still active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.ttl > 0.0
    }
}

impl CombatSystem {
    /// Creates a new combat system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            attack_queue: VecDeque::new(),
            damage_events: Vec::new(),
            projectiles: Vec::new(),
            rng_state: 12345,
        }
    }

    /// Returns damage events from the last process call.
    #[must_use]
    pub fn damage_events(&self) -> &[DamageEvent] {
        &self.damage_events
    }

    /// Clears damage events.
    pub fn clear_events(&mut self) {
        self.damage_events.clear();
    }

    /// Returns active projectiles.
    #[must_use]
    pub fn projectiles(&self) -> &[Projectile] {
        &self.projectiles
    }

    /// Returns number of queued attacks.
    #[must_use]
    pub fn queue_len(&self) -> usize {
        self.attack_queue.len()
    }

    /// Queues an attack to be processed.
    pub fn queue_attack(&mut self, intent: AttackIntent) {
        self.attack_queue.push_back(intent);
    }

    /// Generates a pseudo-random value for crits.
    fn next_random(&mut self) -> f32 {
        // Simple xorshift
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32) / (u64::MAX as f32)
    }

    /// Processes all queued attacks and returns damage events.
    pub fn process_attacks<S: CombatStorage, C: CollisionQuery>(
        &mut self,
        storage: &mut S,
        collision: &C,
    ) -> &[DamageEvent] {
        self.damage_events.clear();

        while let Some(intent) = self.attack_queue.pop_front() {
            if let Some(events) = self.process_single_attack(&intent, storage, collision) {
                self.damage_events.extend(events);
            }
        }

        &self.damage_events
    }

    /// Processes a single attack intent.
    fn process_single_attack<S: CombatStorage, C: CollisionQuery>(
        &mut self,
        intent: &AttackIntent,
        storage: &mut S,
        collision: &C,
    ) -> Option<Vec<DamageEvent>> {
        // Get attacker position and stats (clone to avoid borrow issues)
        let attacker_pos = storage.get_position(intent.attacker)?;
        let attacker_stats = storage.get_stats(intent.attacker)?.clone();

        // Check cooldown
        if !attacker_stats.can_attack() {
            return None;
        }

        // Get weapon stats if weapon is specified
        let weapon_stats = intent
            .weapon
            .and_then(|w| storage.get_weapon(w).cloned())
            .unwrap_or_default();

        let mut events = Vec::new();

        match intent.attack_type {
            AttackType::Melee { range, arc } => {
                events.extend(self.process_melee_attack(
                    intent,
                    attacker_pos,
                    &attacker_stats,
                    &weapon_stats,
                    range,
                    arc,
                    storage,
                ));
            },
            AttackType::Ranged { projectile, speed } => {
                self.spawn_projectile(
                    intent,
                    attacker_pos,
                    &attacker_stats,
                    &weapon_stats,
                    projectile,
                    speed,
                );
            },
            AttackType::Area { radius, falloff } => {
                events.extend(self.process_area_attack(
                    intent,
                    attacker_pos,
                    &attacker_stats,
                    &weapon_stats,
                    radius,
                    falloff,
                    storage,
                    collision,
                ));
            },
        }

        // Set cooldown
        if let Some(stats) = storage.get_stats_mut(intent.attacker) {
            stats.set_cooldown(weapon_stats.cooldown);
        }

        Some(events)
    }

    /// Processes a melee attack.
    #[allow(clippy::too_many_arguments)]
    fn process_melee_attack<S: CombatStorage>(
        &mut self,
        intent: &AttackIntent,
        attacker_pos: CombatPosition,
        attacker_stats: &CombatStats,
        weapon_stats: &WeaponStats,
        range: f32,
        arc: f32,
        storage: &mut S,
    ) -> Vec<DamageEvent> {
        let mut events = Vec::new();

        // Determine target position
        let target_pos = match intent.target {
            AttackTarget::Entity(id) => storage.get_position(id),
            AttackTarget::Position(x, y) => Some(CombatPosition::new(x, y)),
            AttackTarget::Direction(dx, dy) => Some(CombatPosition::new(
                attacker_pos.x + dx * range,
                attacker_pos.y + dy * range,
            )),
        };

        let target_pos = match target_pos {
            Some(p) => p,
            None => return events,
        };

        // Check range
        let distance = attacker_pos.distance_to(&target_pos);
        if distance > range {
            return events;
        }

        // Check arc (angle)
        let angle_to_target = attacker_pos.angle_to(&target_pos);
        let angle_diff = (angle_to_target - attacker_pos.facing).abs();
        let normalized_diff = if angle_diff > std::f32::consts::PI {
            2.0 * std::f32::consts::PI - angle_diff
        } else {
            angle_diff
        };

        if normalized_diff > arc / 2.0 {
            return events;
        }

        // If targeting an entity, apply damage
        if let AttackTarget::Entity(target_id) = intent.target {
            if let Some(target_stats) = storage.get_stats(target_id) {
                let event = self.calculate_damage(
                    intent.attacker,
                    target_id,
                    attacker_stats,
                    target_stats,
                    weapon_stats,
                    target_pos,
                );
                events.push(event);
            }
        }

        events
    }

    /// Spawns a projectile for ranged attacks.
    fn spawn_projectile(
        &mut self,
        intent: &AttackIntent,
        attacker_pos: CombatPosition,
        attacker_stats: &CombatStats,
        weapon_stats: &WeaponStats,
        projectile_type: ProjectileType,
        speed: f32,
    ) {
        let direction = match intent.target {
            AttackTarget::Entity(_id) => {
                // For now, just shoot in facing direction
                (attacker_pos.facing.cos(), attacker_pos.facing.sin())
            },
            AttackTarget::Position(x, y) => {
                let dx = x - attacker_pos.x;
                let dy = y - attacker_pos.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 0.0001 {
                    (1.0, 0.0)
                } else {
                    (dx / dist, dy / dist)
                }
            },
            AttackTarget::Direction(dx, dy) => {
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 0.0001 {
                    (1.0, 0.0)
                } else {
                    (dx / dist, dy / dist)
                }
            },
        };

        let velocity = (direction.0 * speed, direction.1 * speed);
        let damage = weapon_stats.damage * attacker_stats.damage_multiplier;

        let mut projectile = Projectile::new(
            intent.attacker,
            (attacker_pos.x, attacker_pos.y),
            velocity,
            damage,
            projectile_type,
        );
        projectile.damage_type = intent.damage_type;
        projectile.knockback = weapon_stats.knockback;

        self.projectiles.push(projectile);
    }

    /// Processes an area attack.
    #[allow(clippy::too_many_arguments)]
    fn process_area_attack<S: CombatStorage, C: CollisionQuery>(
        &mut self,
        intent: &AttackIntent,
        attacker_pos: CombatPosition,
        attacker_stats: &CombatStats,
        weapon_stats: &WeaponStats,
        radius: f32,
        falloff: bool,
        storage: &mut S,
        _collision: &C,
    ) -> Vec<DamageEvent> {
        let mut events = Vec::new();

        let center = match intent.target {
            AttackTarget::Entity(id) => storage.get_position(id).map(|p| (p.x, p.y)),
            AttackTarget::Position(x, y) => Some((x, y)),
            AttackTarget::Direction(dx, dy) => Some((attacker_pos.x + dx, attacker_pos.y + dy)),
        };

        let center = match center {
            Some(c) => c,
            None => return events,
        };

        // Note: In a real implementation, we'd query all entities in range
        // For now, just create the damage event at the position
        // The caller should provide target entities

        if let AttackTarget::Entity(target_id) = intent.target {
            if let Some(target_stats) = storage.get_stats(target_id) {
                let target_pos = CombatPosition::new(center.0, center.1);
                let distance = attacker_pos.distance_to(&target_pos);

                // Apply falloff if enabled
                let damage_mult = if falloff && distance > 0.0 {
                    (1.0 - distance / radius).max(0.0)
                } else {
                    1.0
                };

                if damage_mult > 0.0 {
                    let mut event = self.calculate_damage(
                        intent.attacker,
                        target_id,
                        attacker_stats,
                        target_stats,
                        weapon_stats,
                        target_pos,
                    );
                    event.damage *= damage_mult;
                    events.push(event);
                }
            }
        }

        events
    }

    /// Calculates damage for an attack.
    fn calculate_damage(
        &mut self,
        attacker: EntityId,
        target: EntityId,
        attacker_stats: &CombatStats,
        target_stats: &CombatStats,
        weapon_stats: &WeaponStats,
        target_pos: CombatPosition,
    ) -> DamageEvent {
        // Base damage
        let base_damage = weapon_stats.damage * attacker_stats.damage_multiplier;

        // Check for critical hit
        let crit_chance = attacker_stats.crit_chance + weapon_stats.crit_bonus;
        let is_crit = self.next_random() < crit_chance;
        let crit_mult = if is_crit {
            attacker_stats.crit_multiplier
        } else {
            1.0
        };

        let raw_damage = base_damage * crit_mult;

        // Calculate armor reduction (only for physical damage)
        let blocked = if weapon_stats.damage_type == DamageType::Physical {
            raw_damage * target_stats.armor_reduction()
        } else {
            0.0
        };

        // Calculate resistance reduction
        let resistance = target_stats.get_resistance(weapon_stats.damage_type);
        let after_armor = raw_damage - blocked;
        let resisted = after_armor * resistance;

        let final_damage = (after_armor - resisted).max(0.0);

        // Calculate knockback direction
        let knockback = if weapon_stats.knockback > 0.0 {
            let (dx, dy) = (1.0, 0.0); // Default direction, would use attacker->target direction
            Some((dx * weapon_stats.knockback, dy * weapon_stats.knockback))
        } else {
            None
        };

        let mut event = DamageEvent::new(target, final_damage, weapon_stats.damage_type)
            .with_source(attacker)
            .with_position(target_pos.x, target_pos.y);

        event.blocked = blocked;
        event.resisted = resisted;
        event.critical = is_crit;

        if let Some(kb) = knockback {
            event = event.with_knockback(kb.0, kb.1);
        }

        event
    }

    /// Updates projectiles and checks for hits.
    pub fn update_projectiles<S: CombatStorage, C: CollisionQuery>(
        &mut self,
        dt: f32,
        _storage: &mut S,
        collision: &C,
    ) {
        let mut hits = Vec::new();

        // Update positions and check for hits
        for (i, projectile) in self.projectiles.iter_mut().enumerate() {
            projectile.update(dt);

            let aabb = projectile.to_aabb();

            // Check collision with world
            if collision.check_collision(aabb) {
                hits.push(i);
            }

            // Note: Entity collision would be handled here
            // For now, projectiles just expire
        }

        // Remove hit projectiles (in reverse order)
        for i in hits.into_iter().rev() {
            self.projectiles.swap_remove(i);
        }

        // Remove expired projectiles
        self.projectiles.retain(Projectile::is_active);
    }

    /// Applies damage to an entity and returns whether it died.
    pub fn apply_damage(
        &mut self,
        _target: EntityId,
        damage: &DamageEvent,
        stats: &mut CombatStats,
    ) -> bool {
        stats.health -= damage.damage;

        stats.is_dead()
    }

    /// Applies damage event to storage and handles death.
    pub fn apply_damage_event<S: CombatStorage>(&mut self, event: &DamageEvent, storage: &mut S) {
        // First check if entity exists and apply damage
        let (is_dead, knockback) = {
            if let Some(stats) = storage.get_stats_mut(event.target) {
                stats.health -= event.damage;
                (stats.is_dead(), event.knockback)
            } else {
                return;
            }
        };

        // Apply knockback (separate borrow scope)
        if let Some(kb) = knockback {
            storage.apply_knockback(event.target, kb);
        }

        // Handle death (separate borrow scope)
        if is_dead {
            storage.on_death(event.target);
        }
    }
}

/// Mock combat storage for testing.
#[cfg(test)]
pub struct MockCombatStorage {
    stats: HashMap<EntityId, CombatStats>,
    positions: HashMap<EntityId, CombatPosition>,
    weapons: HashMap<ItemId, WeaponStats>,
    knockbacks: Vec<(EntityId, (f32, f32))>,
    deaths: Vec<EntityId>,
}

#[cfg(test)]
impl MockCombatStorage {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
            positions: HashMap::new(),
            weapons: HashMap::new(),
            knockbacks: Vec::new(),
            deaths: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, id: EntityId, stats: CombatStats, pos: CombatPosition) {
        self.stats.insert(id, stats);
        self.positions.insert(id, pos);
    }

    pub fn add_weapon(&mut self, id: ItemId, stats: WeaponStats) {
        self.weapons.insert(id, stats);
    }
}

#[cfg(test)]
impl CombatStorage for MockCombatStorage {
    fn get_stats(&self, entity: EntityId) -> Option<&CombatStats> {
        self.stats.get(&entity)
    }

    fn get_stats_mut(&mut self, entity: EntityId) -> Option<&mut CombatStats> {
        self.stats.get_mut(&entity)
    }

    fn get_position(&self, entity: EntityId) -> Option<CombatPosition> {
        self.positions.get(&entity).copied()
    }

    fn get_weapon(&self, item: ItemId) -> Option<&WeaponStats> {
        self.weapons.get(&item)
    }

    fn apply_knockback(&mut self, entity: EntityId, knockback: (f32, f32)) {
        self.knockbacks.push((entity, knockback));
    }

    fn on_death(&mut self, entity: EntityId) {
        self.deaths.push(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::MockCollision;

    #[test]
    fn test_combat_stats_default() {
        let stats = CombatStats::new();
        assert_eq!(stats.health, 100.0);
        assert_eq!(stats.max_health, 100.0);
        assert!(!stats.is_dead());
        assert!(stats.can_attack());
    }

    #[test]
    fn test_combat_stats_builder() {
        let stats = CombatStats::new()
            .with_health(200.0)
            .with_armor(50.0)
            .with_resistance(DamageType::Fire, 0.5)
            .with_attack_speed(1.5);

        assert_eq!(stats.health, 200.0);
        assert_eq!(stats.armor, 50.0);
        assert_eq!(stats.get_resistance(DamageType::Fire), 0.5);
        assert_eq!(stats.attack_speed, 1.5);
    }

    #[test]
    fn test_armor_reduction() {
        let stats = CombatStats::new().with_armor(100.0);
        assert!((stats.armor_reduction() - 0.5).abs() < 0.01);

        let stats2 = CombatStats::new().with_armor(0.0);
        assert_eq!(stats2.armor_reduction(), 0.0);
    }

    #[test]
    fn test_health_percent() {
        let mut stats = CombatStats::new().with_health(100.0);
        assert_eq!(stats.health_percent(), 1.0);

        stats.health = 50.0;
        assert_eq!(stats.health_percent(), 0.5);

        stats.health = 0.0;
        assert_eq!(stats.health_percent(), 0.0);
    }

    #[test]
    fn test_combat_stats_tick() {
        let mut stats = CombatStats::new();
        stats.set_cooldown(1.0);
        assert!(!stats.can_attack());

        stats.tick(0.5);
        assert!(!stats.can_attack());

        stats.tick(0.5);
        assert!(stats.can_attack());
    }

    #[test]
    fn test_damage_event_creation() {
        let target = EntityId::new();
        let event = DamageEvent::new(target, 50.0, DamageType::Physical)
            .with_source(EntityId::new())
            .with_position(10.0, 20.0)
            .with_knockback(5.0, 0.0);

        assert_eq!(event.damage, 50.0);
        assert_eq!(event.position, (10.0, 20.0));
        assert!(event.source.is_some());
        assert_eq!(event.knockback, Some((5.0, 0.0)));
    }

    #[test]
    fn test_attack_intent_builder() {
        let attacker = EntityId::new();
        let target = EntityId::new();
        let weapon = ItemId::new(1);

        let intent = AttackIntent::new(attacker, AttackTarget::Entity(target))
            .with_weapon(weapon)
            .with_damage(25.0)
            .with_damage_type(DamageType::Fire);

        assert_eq!(intent.weapon, Some(weapon));
        assert_eq!(intent.base_damage, 25.0);
        assert_eq!(intent.damage_type, DamageType::Fire);
    }

    #[test]
    fn test_combat_system_queue() {
        let mut combat = CombatSystem::new();
        let attacker = EntityId::new();

        combat.queue_attack(AttackIntent::new(
            attacker,
            AttackTarget::Position(0.0, 0.0),
        ));
        combat.queue_attack(AttackIntent::new(
            attacker,
            AttackTarget::Position(1.0, 1.0),
        ));

        assert_eq!(combat.queue_len(), 2);
    }

    #[test]
    fn test_combat_position_distance() {
        let pos1 = CombatPosition::new(0.0, 0.0);
        let pos2 = CombatPosition::new(3.0, 4.0);

        assert!((pos1.distance_to(&pos2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_combat_position_direction() {
        let pos1 = CombatPosition::new(0.0, 0.0);
        let pos2 = CombatPosition::new(1.0, 0.0);

        let dir = pos1.direction_to(&pos2);
        assert!((dir.0 - 1.0).abs() < 0.001);
        assert!(dir.1.abs() < 0.001);
    }

    #[test]
    fn test_weapon_stats_default() {
        let weapon = WeaponStats::default();
        assert_eq!(weapon.damage, 10.0);
        assert_eq!(weapon.damage_type, DamageType::Physical);
    }

    #[test]
    fn test_weapon_stats_builder() {
        let weapon = WeaponStats::new(50.0)
            .with_damage_type(DamageType::Fire)
            .with_cooldown(1.0)
            .with_knockback(10.0);

        assert_eq!(weapon.damage, 50.0);
        assert_eq!(weapon.damage_type, DamageType::Fire);
        assert_eq!(weapon.cooldown, 1.0);
        assert_eq!(weapon.knockback, 10.0);
    }

    #[test]
    fn test_projectile_creation() {
        let source = EntityId::new();
        let projectile =
            Projectile::new(source, (0.0, 0.0), (10.0, 0.0), 25.0, ProjectileType::Arrow);

        assert_eq!(projectile.position, (0.0, 0.0));
        assert_eq!(projectile.velocity, (10.0, 0.0));
        assert_eq!(projectile.damage, 25.0);
        assert!(projectile.is_active());
    }

    #[test]
    fn test_projectile_update() {
        let source = EntityId::new();
        let mut projectile =
            Projectile::new(source, (0.0, 0.0), (10.0, 5.0), 25.0, ProjectileType::Arrow);

        projectile.update(1.0);
        assert!((projectile.position.0 - 10.0).abs() < 0.001);
        assert!((projectile.position.1 - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_projectile_expiry() {
        let source = EntityId::new();
        let mut projectile =
            Projectile::new(source, (0.0, 0.0), (1.0, 0.0), 10.0, ProjectileType::Arrow);

        projectile.ttl = 1.0;
        assert!(projectile.is_active());

        projectile.update(1.5);
        assert!(!projectile.is_active());
    }

    #[test]
    fn test_apply_damage() {
        let mut combat = CombatSystem::new();
        let target = EntityId::new();
        let mut stats = CombatStats::new().with_health(100.0);

        let event = DamageEvent::new(target, 30.0, DamageType::Physical);
        let killed = combat.apply_damage(target, &event, &mut stats);

        assert!(!killed);
        assert_eq!(stats.health, 70.0);
    }

    #[test]
    fn test_apply_lethal_damage() {
        let mut combat = CombatSystem::new();
        let target = EntityId::new();
        let mut stats = CombatStats::new().with_health(50.0);

        let event = DamageEvent::new(target, 100.0, DamageType::Physical);
        let killed = combat.apply_damage(target, &event, &mut stats);

        assert!(killed);
        assert!(stats.is_dead());
    }

    #[test]
    fn test_combat_process_melee_attack() {
        let mut combat = CombatSystem::new();
        let mut storage = MockCombatStorage::new();
        let collision = MockCollision::new();

        let attacker = EntityId::new();
        let target = EntityId::new();

        storage.add_entity(
            attacker,
            CombatStats::new(),
            CombatPosition::new(0.0, 0.0).with_facing(0.0),
        );
        storage.add_entity(target, CombatStats::new(), CombatPosition::new(1.0, 0.0));

        let intent = AttackIntent::new(attacker, AttackTarget::Entity(target)).with_attack_type(
            AttackType::Melee {
                range: 2.0,
                arc: std::f32::consts::PI,
            },
        );

        combat.queue_attack(intent);
        let events = combat.process_attacks(&mut storage, &collision);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, target);
    }

    #[test]
    fn test_combat_out_of_range() {
        let mut combat = CombatSystem::new();
        let mut storage = MockCombatStorage::new();
        let collision = MockCollision::new();

        let attacker = EntityId::new();
        let target = EntityId::new();

        storage.add_entity(attacker, CombatStats::new(), CombatPosition::new(0.0, 0.0));
        storage.add_entity(
            target,
            CombatStats::new(),
            CombatPosition::new(10.0, 0.0), // Far away
        );

        let intent = AttackIntent::new(attacker, AttackTarget::Entity(target)).with_attack_type(
            AttackType::Melee {
                range: 2.0,
                arc: std::f32::consts::PI,
            },
        );

        combat.queue_attack(intent);
        let events = combat.process_attacks(&mut storage, &collision);

        assert!(events.is_empty()); // Out of range
    }

    #[test]
    fn test_combat_spawn_projectile() {
        let mut combat = CombatSystem::new();
        let mut storage = MockCombatStorage::new();
        let collision = MockCollision::new();

        let attacker = EntityId::new();

        storage.add_entity(attacker, CombatStats::new(), CombatPosition::new(0.0, 0.0));

        let intent = AttackIntent::new(attacker, AttackTarget::Direction(1.0, 0.0))
            .with_attack_type(AttackType::Ranged {
                projectile: ProjectileType::Arrow,
                speed: 20.0,
            });

        combat.queue_attack(intent);
        let _ = combat.process_attacks(&mut storage, &collision);

        assert_eq!(combat.projectiles().len(), 1);
        assert_eq!(
            combat.projectiles()[0].projectile_type,
            ProjectileType::Arrow
        );
    }

    #[test]
    fn test_damage_with_armor() {
        let mut combat = CombatSystem::new();
        let mut storage = MockCombatStorage::new();
        let collision = MockCollision::new();

        let attacker = EntityId::new();
        let target = EntityId::new();

        // Disable crits to get predictable damage
        let mut attacker_stats = CombatStats::new();
        attacker_stats.crit_chance = 0.0;

        storage.add_entity(
            attacker,
            attacker_stats,
            CombatPosition::new(0.0, 0.0).with_facing(0.0),
        );
        storage.add_entity(
            target,
            CombatStats::new().with_armor(100.0), // 50% reduction
            CombatPosition::new(1.0, 0.0),
        );

        let intent = AttackIntent::new(attacker, AttackTarget::Entity(target)).with_attack_type(
            AttackType::Melee {
                range: 2.0,
                arc: std::f32::consts::PI,
            },
        );

        combat.queue_attack(intent);
        let events = combat.process_attacks(&mut storage, &collision);

        assert_eq!(events.len(), 1);
        // 10 base damage * 50% armor reduction = 5 damage
        assert!((events[0].damage - 5.0).abs() < 0.1);
        assert!(events[0].blocked > 0.0);
    }

    #[test]
    fn test_damage_with_resistance() {
        let mut combat = CombatSystem::new();
        let mut storage = MockCombatStorage::new();
        let collision = MockCollision::new();

        let attacker = EntityId::new();
        let target = EntityId::new();
        let weapon = ItemId::new(1);

        // Disable crits to get predictable damage
        let mut attacker_stats = CombatStats::new();
        attacker_stats.crit_chance = 0.0;

        storage.add_entity(
            attacker,
            attacker_stats,
            CombatPosition::new(0.0, 0.0).with_facing(0.0),
        );
        storage.add_entity(
            target,
            CombatStats::new().with_resistance(DamageType::Fire, 0.5),
            CombatPosition::new(1.0, 0.0),
        );
        storage.add_weapon(
            weapon,
            WeaponStats::new(20.0).with_damage_type(DamageType::Fire),
        );

        let intent = AttackIntent::new(attacker, AttackTarget::Entity(target))
            .with_weapon(weapon)
            .with_attack_type(AttackType::Melee {
                range: 2.0,
                arc: std::f32::consts::PI,
            });

        combat.queue_attack(intent);
        let events = combat.process_attacks(&mut storage, &collision);

        assert_eq!(events.len(), 1);
        // 20 fire damage * 50% resistance = 10 damage
        assert!((events[0].damage - 10.0).abs() < 0.1);
        assert!(events[0].resisted > 0.0);
    }

    #[test]
    fn test_update_projectiles() {
        let mut combat = CombatSystem::new();
        let mut storage = MockCombatStorage::new();
        let collision = MockCollision::new_empty();

        let attacker = EntityId::new();
        storage.add_entity(attacker, CombatStats::new(), CombatPosition::new(0.0, 0.0));

        // Manually add a projectile
        combat.projectiles.push(Projectile::new(
            attacker,
            (0.0, 0.0),
            (10.0, 0.0),
            20.0,
            ProjectileType::Arrow,
        ));

        combat.update_projectiles(1.0, &mut storage, &collision);

        assert_eq!(combat.projectiles().len(), 1);
        assert!((combat.projectiles()[0].position.0 - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_projectile_wall_collision() {
        let mut combat = CombatSystem::new();
        let mut storage = MockCombatStorage::new();
        let mut collision = MockCollision::new();
        collision.set_ground_level(0); // Ground at y >= 0

        let attacker = EntityId::new();
        storage.add_entity(attacker, CombatStats::new(), CombatPosition::new(0.0, 0.0));

        // Projectile starts at y=0.5, moves down 1.0 to y=-0.5, but we check AFTER move
        // Start at y=1.0 so after update it's at y=0.0 which collides with ground
        combat.projectiles.push(Projectile::new(
            attacker,
            (0.0, 1.0),  // Start above ground
            (0.0, -1.0), // Moving down
            20.0,
            ProjectileType::Arrow,
        ));

        combat.update_projectiles(1.0, &mut storage, &collision);

        // Projectile should be removed on collision (after moving to y=0.0)
        assert!(combat.projectiles().is_empty());
    }

    #[test]
    fn test_damage_type_variants() {
        assert_eq!(DamageType::default(), DamageType::Physical);

        let types = [
            DamageType::Physical,
            DamageType::Fire,
            DamageType::Ice,
            DamageType::Electric,
            DamageType::Poison,
            DamageType::True,
        ];

        for dt in types {
            let _ = format!("{:?}", dt);
        }
    }

    #[test]
    fn test_attack_type_variants() {
        let melee = AttackType::Melee {
            range: 1.0,
            arc: 1.0,
        };
        let ranged = AttackType::Ranged {
            projectile: ProjectileType::Arrow,
            speed: 10.0,
        };
        let area = AttackType::Area {
            radius: 5.0,
            falloff: true,
        };

        assert!(matches!(melee, AttackType::Melee { .. }));
        assert!(matches!(ranged, AttackType::Ranged { .. }));
        assert!(matches!(area, AttackType::Area { .. }));
    }

    #[test]
    fn test_projectile_type_variants() {
        let types = [
            ProjectileType::Arrow,
            ProjectileType::Bolt,
            ProjectileType::Magic,
            ProjectileType::Thrown,
            ProjectileType::Bullet,
        ];

        for pt in types {
            let _ = format!("{:?}", pt);
        }
    }

    #[test]
    fn test_attack_target_variants() {
        let entity_target = AttackTarget::Entity(EntityId::new());
        let pos_target = AttackTarget::Position(1.0, 2.0);
        let dir_target = AttackTarget::Direction(1.0, 0.0);

        assert!(matches!(entity_target, AttackTarget::Entity(_)));
        assert!(matches!(pos_target, AttackTarget::Position(_, _)));
        assert!(matches!(dir_target, AttackTarget::Direction(_, _)));
    }

    #[test]
    fn test_heal() {
        let mut stats = CombatStats::new().with_health(100.0);
        stats.health = 50.0;

        stats.heal(30.0);
        assert_eq!(stats.health, 80.0);

        stats.heal(50.0);
        assert_eq!(stats.health, 100.0); // Capped at max
    }

    #[test]
    fn test_clear_events() {
        let mut combat = CombatSystem::new();
        combat.damage_events.push(DamageEvent::new(
            EntityId::new(),
            10.0,
            DamageType::Physical,
        ));

        assert_eq!(combat.damage_events().len(), 1);
        combat.clear_events();
        assert!(combat.damage_events().is_empty());
    }

    #[test]
    fn test_projectile_aabb() {
        let projectile = Projectile::new(
            EntityId::new(),
            (5.0, 5.0),
            (0.0, 0.0),
            10.0,
            ProjectileType::Arrow,
        );

        let aabb = projectile.to_aabb();
        assert!((aabb.min_x - 4.8).abs() < 0.01);
        assert!((aabb.min_y - 4.8).abs() < 0.01);
        assert!((aabb.max_x - 5.2).abs() < 0.01);
        assert!((aabb.max_y - 5.2).abs() < 0.01);
    }
}
