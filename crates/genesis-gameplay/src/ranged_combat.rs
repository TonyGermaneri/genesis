//! Ranged attack logic.
//!
//! This module provides:
//! - Bow draw time and power scaling
//! - Arrow velocity based on draw
//! - Throwing weapons (instant)
//! - Ammo consumption

use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// G-51: Ranged Weapon Types
// ============================================================================

/// Type of ranged weapon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RangedWeaponType {
    /// Standard bow.
    Bow,
    /// Crossbow (no draw, instant fire).
    Crossbow,
    /// Longbow (slower draw, more damage).
    Longbow,
    /// Shortbow (faster draw, less damage).
    Shortbow,
    /// Sling.
    Sling,
    /// Throwing knife.
    ThrowingKnife,
    /// Throwing axe.
    ThrowingAxe,
    /// Javelin.
    Javelin,
    /// Magic staff (no ammo).
    Staff,
}

impl RangedWeaponType {
    /// Check if weapon requires drawing.
    #[must_use]
    pub fn requires_draw(&self) -> bool {
        matches!(self, Self::Bow | Self::Longbow | Self::Shortbow)
    }

    /// Check if weapon uses ammo.
    #[must_use]
    pub fn uses_ammo(&self) -> bool {
        matches!(
            self,
            Self::Bow | Self::Crossbow | Self::Longbow | Self::Shortbow | Self::Sling
        )
    }

    /// Check if weapon is thrown (consumed on use).
    #[must_use]
    pub fn is_thrown(&self) -> bool {
        matches!(
            self,
            Self::ThrowingKnife | Self::ThrowingAxe | Self::Javelin
        )
    }

    /// Get default draw time for this weapon.
    #[must_use]
    pub fn default_draw_time(&self) -> f32 {
        match self {
            Self::Bow => 1.0,
            Self::Crossbow => 0.0,
            Self::Longbow => 1.5,
            Self::Shortbow => 0.6,
            Self::Sling | Self::Javelin => 0.4,
            Self::ThrowingKnife => 0.2,
            Self::ThrowingAxe => 0.3,
            Self::Staff => 0.5,
        }
    }

    /// Get default projectile speed.
    #[must_use]
    pub fn default_projectile_speed(&self) -> f32 {
        match self {
            Self::Bow => 40.0,
            Self::Crossbow => 60.0,
            Self::Longbow => 50.0,
            Self::Shortbow | Self::Staff => 35.0,
            Self::Sling => 30.0,
            Self::ThrowingKnife | Self::Javelin => 25.0,
            Self::ThrowingAxe => 20.0,
        }
    }

    /// Get default range.
    #[must_use]
    pub fn default_range(&self) -> f32 {
        match self {
            Self::Bow => 30.0,
            Self::Crossbow => 40.0,
            Self::Longbow => 50.0,
            Self::Staff => 25.0,
            Self::Shortbow => 20.0,
            Self::Javelin => 18.0,
            Self::Sling => 15.0,
            Self::ThrowingKnife => 12.0,
            Self::ThrowingAxe => 10.0,
        }
    }

    /// Get default stamina cost.
    #[must_use]
    pub fn default_stamina_cost(&self) -> f32 {
        match self {
            Self::Bow => 15.0,
            Self::Crossbow | Self::Shortbow => 10.0,
            Self::Longbow => 25.0,
            Self::Sling => 8.0,
            Self::ThrowingKnife => 5.0,
            Self::ThrowingAxe => 12.0,
            Self::Javelin => 18.0,
            Self::Staff => 20.0,
        }
    }
}

// ============================================================================
// G-51: Ammo Types
// ============================================================================

/// Type of ammunition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmmoType {
    /// Standard arrow.
    Arrow,
    /// Fire arrow.
    FireArrow,
    /// Ice arrow.
    IceArrow,
    /// Poison arrow.
    PoisonArrow,
    /// Explosive arrow.
    ExplosiveArrow,
    /// Crossbow bolt.
    Bolt,
    /// Sling stone.
    Stone,
    /// Magic projectile (no physical ammo).
    Magic,
}

impl AmmoType {
    /// Get damage type for this ammo.
    #[must_use]
    pub fn damage_type(&self) -> RangedDamageType {
        match self {
            Self::Arrow | Self::Bolt | Self::Stone => RangedDamageType::Physical,
            Self::FireArrow | Self::ExplosiveArrow => RangedDamageType::Fire,
            Self::IceArrow => RangedDamageType::Ice,
            Self::PoisonArrow => RangedDamageType::Poison,
            Self::Magic => RangedDamageType::Magic,
        }
    }

    /// Get damage multiplier for this ammo.
    #[must_use]
    pub fn damage_multiplier(&self) -> f32 {
        match self {
            Self::Arrow | Self::Magic => 1.0,
            Self::FireArrow => 1.2,
            Self::IceArrow => 1.1,
            Self::PoisonArrow => 0.9,
            Self::ExplosiveArrow => 1.5,
            Self::Bolt => 1.3,
            Self::Stone => 0.8,
        }
    }

    /// Check if ammo has area effect.
    #[must_use]
    pub fn has_aoe(&self) -> bool {
        matches!(self, Self::ExplosiveArrow)
    }

    /// Get AOE radius (if any).
    #[must_use]
    pub fn aoe_radius(&self) -> f32 {
        match self {
            Self::ExplosiveArrow => 3.0,
            _ => 0.0,
        }
    }
}

/// Damage type for ranged attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RangedDamageType {
    /// Physical (piercing).
    Physical,
    /// Fire damage.
    Fire,
    /// Ice damage.
    Ice,
    /// Poison damage.
    Poison,
    /// Magic damage.
    Magic,
}

// ============================================================================
// G-51: Ranged Weapon Stats
// ============================================================================

/// Statistics for a ranged weapon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangedWeaponStats {
    /// Weapon type.
    pub weapon_type: RangedWeaponType,
    /// Base damage.
    pub damage: f32,
    /// Draw time (for bows).
    pub draw_time: f32,
    /// Minimum draw for shot (0.0-1.0).
    pub min_draw: f32,
    /// Projectile speed at full draw.
    pub projectile_speed: f32,
    /// Maximum range.
    pub range: f32,
    /// Stamina cost.
    pub stamina_cost: f32,
    /// Reload time (for crossbows).
    pub reload_time: f32,
    /// Required ammo type (None for thrown/magic).
    pub ammo_type: Option<AmmoType>,
    /// Critical chance bonus.
    pub crit_bonus: f32,
    /// Headshot multiplier.
    pub headshot_mult: f32,
}

impl Default for RangedWeaponStats {
    fn default() -> Self {
        Self {
            weapon_type: RangedWeaponType::Bow,
            damage: 15.0,
            draw_time: RangedWeaponType::Bow.default_draw_time(),
            min_draw: 0.3,
            projectile_speed: RangedWeaponType::Bow.default_projectile_speed(),
            range: RangedWeaponType::Bow.default_range(),
            stamina_cost: RangedWeaponType::Bow.default_stamina_cost(),
            reload_time: 0.0,
            ammo_type: Some(AmmoType::Arrow),
            crit_bonus: 0.1,
            headshot_mult: 2.0,
        }
    }
}

impl RangedWeaponStats {
    /// Create weapon stats for a weapon type.
    #[must_use]
    pub fn for_type(weapon_type: RangedWeaponType) -> Self {
        let ammo_type = if weapon_type.uses_ammo() {
            match weapon_type {
                RangedWeaponType::Crossbow => Some(AmmoType::Bolt),
                RangedWeaponType::Sling => Some(AmmoType::Stone),
                _ => Some(AmmoType::Arrow),
            }
        } else if weapon_type == RangedWeaponType::Staff {
            Some(AmmoType::Magic)
        } else {
            None
        };

        Self {
            weapon_type,
            draw_time: weapon_type.default_draw_time(),
            projectile_speed: weapon_type.default_projectile_speed(),
            range: weapon_type.default_range(),
            stamina_cost: weapon_type.default_stamina_cost(),
            reload_time: if weapon_type == RangedWeaponType::Crossbow {
                1.5
            } else {
                0.0
            },
            ammo_type,
            ..Default::default()
        }
    }

    /// Set damage.
    #[must_use]
    pub fn with_damage(mut self, damage: f32) -> Self {
        self.damage = damage;
        self
    }

    /// Set draw time.
    #[must_use]
    pub fn with_draw_time(mut self, time: f32) -> Self {
        self.draw_time = time;
        self
    }

    /// Set projectile speed.
    #[must_use]
    pub fn with_projectile_speed(mut self, speed: f32) -> Self {
        self.projectile_speed = speed;
        self
    }

    /// Set range.
    #[must_use]
    pub fn with_range(mut self, range: f32) -> Self {
        self.range = range;
        self
    }

    /// Set stamina cost.
    #[must_use]
    pub fn with_stamina_cost(mut self, cost: f32) -> Self {
        self.stamina_cost = cost;
        self
    }

    /// Get damage at specific draw level.
    #[must_use]
    pub fn damage_at_draw(&self, draw: f32) -> f32 {
        // Damage scales quadratically with draw
        self.damage * draw * draw
    }

    /// Get projectile speed at specific draw level.
    #[must_use]
    pub fn speed_at_draw(&self, draw: f32) -> f32 {
        // Speed scales linearly with draw
        self.projectile_speed * draw.max(0.3)
    }
}

// ============================================================================
// G-51: Draw State
// ============================================================================

/// State of drawing a ranged weapon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrawState {
    /// Not drawing.
    Idle,
    /// Currently drawing.
    Drawing,
    /// Fully drawn, holding.
    Held,
    /// Releasing shot.
    Releasing,
    /// Reloading (crossbow).
    Reloading,
}

impl DrawState {
    /// Check if can fire.
    #[must_use]
    pub fn can_fire(&self) -> bool {
        matches!(self, Self::Drawing | Self::Held)
    }

    /// Check if drawing.
    #[must_use]
    pub fn is_drawing(&self) -> bool {
        matches!(self, Self::Drawing | Self::Held)
    }
}

// ============================================================================
// G-51: Active Ranged Attack
// ============================================================================

/// An active ranged attack (drawing/shooting).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveRangedAttack {
    /// Attacking entity.
    pub attacker: EntityId,
    /// Aim direction (radians).
    pub direction: f32,
    /// Current draw state.
    pub state: DrawState,
    /// Current draw amount (0.0-1.0).
    pub draw_amount: f32,
    /// Weapon stats.
    pub weapon: RangedWeaponStats,
    /// Time in current state.
    pub state_time: f32,
    /// Arm fatigue (reduces accuracy over time).
    pub fatigue: f32,
}

impl ActiveRangedAttack {
    /// Create a new ranged attack.
    #[must_use]
    pub fn new(attacker: EntityId, weapon: RangedWeaponStats) -> Self {
        let initial_state = if weapon.weapon_type == RangedWeaponType::Crossbow {
            DrawState::Idle
        } else if weapon.weapon_type.requires_draw() {
            DrawState::Drawing
        } else {
            DrawState::Idle
        };

        Self {
            attacker,
            direction: 0.0,
            state: initial_state,
            draw_amount: 0.0,
            weapon,
            state_time: 0.0,
            fatigue: 0.0,
        }
    }

    /// Start drawing.
    pub fn start_draw(&mut self) {
        if self.state == DrawState::Idle {
            self.state = DrawState::Drawing;
            self.draw_amount = 0.0;
            self.state_time = 0.0;
            self.fatigue = 0.0;
        }
    }

    /// Update draw state.
    pub fn tick(&mut self, dt: f32) {
        self.state_time += dt;

        match self.state {
            DrawState::Drawing => {
                if self.weapon.draw_time > 0.0 {
                    self.draw_amount += dt / self.weapon.draw_time;
                    if self.draw_amount >= 1.0 {
                        self.draw_amount = 1.0;
                        self.state = DrawState::Held;
                        self.state_time = 0.0;
                    }
                } else {
                    self.draw_amount = 1.0;
                    self.state = DrawState::Held;
                }
            },
            DrawState::Held => {
                // Accumulate fatigue while holding
                self.fatigue += dt * 0.1;
            },
            DrawState::Reloading => {
                if self.state_time >= self.weapon.reload_time {
                    self.state = DrawState::Idle;
                }
            },
            _ => {},
        }
    }

    /// Set aim direction.
    pub fn aim(&mut self, direction: f32) {
        self.direction = direction;
    }

    /// Check if can fire.
    #[must_use]
    pub fn can_fire(&self) -> bool {
        self.state.can_fire() && self.draw_amount >= self.weapon.min_draw
    }

    /// Release shot, returns projectile data if successful.
    pub fn release(&mut self) -> Option<ProjectileData> {
        if !self.can_fire() {
            return None;
        }

        let draw = self.draw_amount;
        let damage = self.weapon.damage_at_draw(draw);
        let speed = self.weapon.speed_at_draw(draw);

        // Calculate accuracy reduction from fatigue
        let accuracy = (1.0 - self.fatigue.min(0.5)).max(0.5);

        self.state = DrawState::Releasing;
        self.draw_amount = 0.0;

        Some(ProjectileData {
            damage,
            speed,
            direction: self.direction,
            range: self.weapon.range,
            accuracy,
            ammo_type: self.weapon.ammo_type,
            crit_bonus: self.weapon.crit_bonus,
            headshot_mult: self.weapon.headshot_mult,
        })
    }

    /// Cancel draw.
    pub fn cancel(&mut self) {
        self.state = DrawState::Idle;
        self.draw_amount = 0.0;
        self.fatigue = 0.0;
    }

    /// Start reload (crossbow).
    pub fn start_reload(&mut self) {
        if self.weapon.weapon_type == RangedWeaponType::Crossbow
            && matches!(self.state, DrawState::Idle | DrawState::Releasing)
        {
            self.state = DrawState::Reloading;
            self.state_time = 0.0;
        }
    }

    /// Get draw percentage for UI.
    #[must_use]
    pub fn draw_percent(&self) -> f32 {
        self.draw_amount * 100.0
    }

    /// Get accuracy percentage.
    #[must_use]
    pub fn accuracy_percent(&self) -> f32 {
        (1.0 - self.fatigue.min(0.5)) * 100.0
    }
}

/// Data for spawning a projectile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectileData {
    /// Damage on hit.
    pub damage: f32,
    /// Projectile speed.
    pub speed: f32,
    /// Direction (radians).
    pub direction: f32,
    /// Maximum range.
    pub range: f32,
    /// Accuracy (0.0-1.0).
    pub accuracy: f32,
    /// Ammo type.
    pub ammo_type: Option<AmmoType>,
    /// Critical chance bonus.
    pub crit_bonus: f32,
    /// Headshot damage multiplier.
    pub headshot_mult: f32,
}

// ============================================================================
// G-51: Active Projectile
// ============================================================================

/// An active projectile in flight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangedProjectile {
    /// Owner entity.
    pub owner: EntityId,
    /// Current position.
    pub position: (f32, f32),
    /// Velocity.
    pub velocity: (f32, f32),
    /// Damage on hit.
    pub damage: f32,
    /// Damage type.
    pub damage_type: RangedDamageType,
    /// Distance traveled.
    pub distance: f32,
    /// Maximum range.
    pub max_range: f32,
    /// Gravity effect (for arc).
    pub gravity: f32,
    /// Ammo type.
    pub ammo_type: Option<AmmoType>,
    /// Critical chance bonus.
    pub crit_bonus: f32,
    /// Headshot multiplier.
    pub headshot_mult: f32,
    /// Whether projectile is still active.
    pub active: bool,
}

impl RangedProjectile {
    /// Create a new projectile.
    #[must_use]
    pub fn new(owner: EntityId, position: (f32, f32), data: &ProjectileData) -> Self {
        let vel_x = data.direction.cos() * data.speed;
        let vel_y = data.direction.sin() * data.speed;

        let damage_type = data
            .ammo_type
            .map_or(RangedDamageType::Physical, |a| a.damage_type());

        Self {
            owner,
            position,
            velocity: (vel_x, vel_y),
            damage: data.damage,
            damage_type,
            distance: 0.0,
            max_range: data.range,
            gravity: 5.0, // Default gravity
            ammo_type: data.ammo_type,
            crit_bonus: data.crit_bonus,
            headshot_mult: data.headshot_mult,
            active: true,
        }
    }

    /// Set gravity (0 = no arc).
    #[must_use]
    pub fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity;
        self
    }

    /// Update projectile position.
    pub fn tick(&mut self, dt: f32) {
        if !self.active {
            return;
        }

        // Apply gravity to Y velocity
        self.velocity.1 -= self.gravity * dt;

        // Update position
        let dx = self.velocity.0 * dt;
        let dy = self.velocity.1 * dt;
        self.position.0 += dx;
        self.position.1 += dy;

        // Track distance
        self.distance += (dx * dx + dy * dy).sqrt();

        // Check range
        if self.distance >= self.max_range {
            self.active = false;
        }
    }

    /// Check collision with point (simplified).
    #[must_use]
    pub fn check_collision(&self, point: (f32, f32), radius: f32) -> bool {
        if !self.active {
            return false;
        }
        let dx = self.position.0 - point.0;
        let dy = self.position.1 - point.1;
        (dx * dx + dy * dy).sqrt() <= radius
    }

    /// Deactivate on hit.
    pub fn on_hit(&mut self) {
        self.active = false;
    }

    /// Check if has AOE.
    #[must_use]
    pub fn has_aoe(&self) -> bool {
        self.ammo_type.is_some_and(|a| a.has_aoe())
    }

    /// Get AOE radius.
    #[must_use]
    pub fn aoe_radius(&self) -> f32 {
        self.ammo_type.map_or(0.0, |a| a.aoe_radius())
    }
}

// ============================================================================
// G-51: Ammo Inventory
// ============================================================================

/// Manages ammo for an entity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AmmoInventory {
    /// Ammo counts by type.
    ammo: HashMap<AmmoType, u32>,
    /// Currently selected ammo type.
    pub selected: Option<AmmoType>,
}

impl AmmoInventory {
    /// Create new ammo inventory.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add ammo.
    pub fn add(&mut self, ammo_type: AmmoType, count: u32) {
        *self.ammo.entry(ammo_type).or_insert(0) += count;
    }

    /// Remove ammo, returns true if successful.
    pub fn consume(&mut self, ammo_type: AmmoType, count: u32) -> bool {
        if let Some(current) = self.ammo.get_mut(&ammo_type) {
            if *current >= count {
                *current -= count;
                return true;
            }
        }
        false
    }

    /// Get ammo count.
    #[must_use]
    pub fn count(&self, ammo_type: AmmoType) -> u32 {
        self.ammo.get(&ammo_type).copied().unwrap_or(0)
    }

    /// Check if has ammo.
    #[must_use]
    pub fn has(&self, ammo_type: AmmoType) -> bool {
        self.count(ammo_type) > 0
    }

    /// Select ammo type.
    pub fn select(&mut self, ammo_type: AmmoType) {
        if self.has(ammo_type) {
            self.selected = Some(ammo_type);
        }
    }

    /// Get selected ammo if available.
    #[must_use]
    pub fn get_selected(&self) -> Option<AmmoType> {
        self.selected.filter(|&t| self.has(t))
    }

    /// Consume selected ammo.
    pub fn consume_selected(&mut self) -> Option<AmmoType> {
        let selected = self.selected?;
        if self.consume(selected, 1) {
            Some(selected)
        } else {
            None
        }
    }

    /// Get all ammo types with counts.
    pub fn all(&self) -> impl Iterator<Item = (&AmmoType, &u32)> {
        self.ammo.iter()
    }
}

// ============================================================================
// G-51: Ranged Combat System
// ============================================================================

/// Result of a ranged attack attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum RangedAttackResult {
    /// Started drawing.
    Drawing,
    /// Shot fired, returns projectile data.
    Fired(ProjectileData),
    /// Weapon thrown.
    Thrown(ProjectileData),
    /// Not enough stamina.
    InsufficientStamina {
        /// Stamina required for the attack.
        required: f32,
        /// Stamina currently available.
        available: f32,
    },
    /// No ammo.
    NoAmmo,
    /// Weapon needs reload.
    NeedsReload,
    /// Cannot attack (stunned, etc.).
    CannotAttack,
    /// Still drawing.
    StillDrawing {
        /// Current draw progress (0.0-1.0).
        progress: f32,
    },
}

/// A hit from a ranged attack.
#[derive(Debug, Clone, PartialEq)]
pub struct RangedHit {
    /// Target entity.
    pub target: EntityId,
    /// Damage dealt.
    pub damage: f32,
    /// Damage type.
    pub damage_type: RangedDamageType,
    /// Hit position.
    pub position: (f32, f32),
    /// Whether this was a critical hit.
    pub critical: bool,
    /// Whether this was a headshot.
    pub headshot: bool,
}

/// Manages ranged combat for entities.
#[derive(Debug, Default)]
pub struct RangedCombatSystem {
    /// Active ranged attacks by entity.
    active_attacks: HashMap<EntityId, ActiveRangedAttack>,
    /// Active projectiles.
    projectiles: Vec<RangedProjectile>,
    /// Ammo inventories by entity.
    ammo_inventories: HashMap<EntityId, AmmoInventory>,
    /// Attack cooldowns by entity.
    cooldowns: HashMap<EntityId, f32>,
}

impl RangedCombatSystem {
    /// Create new ranged combat system.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start drawing a ranged weapon.
    pub fn start_draw(
        &mut self,
        entity: EntityId,
        weapon: RangedWeaponStats,
        can_attack: bool,
    ) -> RangedAttackResult {
        if !can_attack {
            return RangedAttackResult::CannotAttack;
        }

        // Check if already drawing
        if let Some(attack) = self.active_attacks.get(&entity) {
            if attack.state.is_drawing() {
                return RangedAttackResult::StillDrawing {
                    progress: attack.draw_amount,
                };
            }
        }

        // Check ammo
        if weapon.weapon_type.uses_ammo() {
            if let Some(ammo) = self.ammo_inventories.get(&entity) {
                if let Some(required) = weapon.ammo_type {
                    if !ammo.has(required) {
                        return RangedAttackResult::NoAmmo;
                    }
                }
            } else {
                return RangedAttackResult::NoAmmo;
            }
        }

        // Check reload for crossbow
        if weapon.weapon_type == RangedWeaponType::Crossbow {
            if let Some(attack) = self.active_attacks.get(&entity) {
                if attack.state == DrawState::Reloading {
                    return RangedAttackResult::NeedsReload;
                }
            }
        }

        let mut attack = ActiveRangedAttack::new(entity, weapon);
        attack.start_draw();
        self.active_attacks.insert(entity, attack);

        RangedAttackResult::Drawing
    }

    /// Update aim direction.
    pub fn aim(&mut self, entity: EntityId, direction: f32) {
        if let Some(attack) = self.active_attacks.get_mut(&entity) {
            attack.aim(direction);
        }
    }

    /// Release shot.
    pub fn release(
        &mut self,
        entity: EntityId,
        position: (f32, f32),
        stamina: f32,
    ) -> RangedAttackResult {
        let attack = match self.active_attacks.get_mut(&entity) {
            Some(a) => a,
            None => return RangedAttackResult::CannotAttack,
        };

        if !attack.can_fire() {
            if attack.state == DrawState::Drawing {
                return RangedAttackResult::StillDrawing {
                    progress: attack.draw_amount,
                };
            }
            return RangedAttackResult::CannotAttack;
        }

        // Check stamina
        if stamina < attack.weapon.stamina_cost {
            return RangedAttackResult::InsufficientStamina {
                required: attack.weapon.stamina_cost,
                available: stamina,
            };
        }

        // Consume ammo
        if attack.weapon.weapon_type.uses_ammo() {
            if let Some(ammo) = self.ammo_inventories.get_mut(&entity) {
                if ammo.consume_selected().is_none() {
                    return RangedAttackResult::NoAmmo;
                }
            } else {
                return RangedAttackResult::NoAmmo;
            }
        }

        // Release
        if let Some(data) = attack.release() {
            // Spawn projectile
            let projectile = RangedProjectile::new(entity, position, &data);
            self.projectiles.push(projectile);

            // Handle crossbow reload
            if attack.weapon.weapon_type == RangedWeaponType::Crossbow {
                attack.start_reload();
            }

            RangedAttackResult::Fired(data)
        } else {
            RangedAttackResult::CannotAttack
        }
    }

    /// Throw a weapon (instant, no draw).
    pub fn throw_weapon(
        &mut self,
        entity: EntityId,
        weapon: &RangedWeaponStats,
        position: (f32, f32),
        direction: f32,
        stamina: f32,
        can_attack: bool,
    ) -> RangedAttackResult {
        if !can_attack {
            return RangedAttackResult::CannotAttack;
        }

        if !weapon.weapon_type.is_thrown() {
            return RangedAttackResult::CannotAttack;
        }

        if stamina < weapon.stamina_cost {
            return RangedAttackResult::InsufficientStamina {
                required: weapon.stamina_cost,
                available: stamina,
            };
        }

        let data = ProjectileData {
            damage: weapon.damage,
            speed: weapon.projectile_speed,
            direction,
            range: weapon.range,
            accuracy: 1.0,
            ammo_type: None,
            crit_bonus: weapon.crit_bonus,
            headshot_mult: weapon.headshot_mult,
        };

        let projectile = RangedProjectile::new(entity, position, &data);
        self.projectiles.push(projectile);

        RangedAttackResult::Thrown(data)
    }

    /// Cancel current draw.
    pub fn cancel(&mut self, entity: EntityId) {
        if let Some(attack) = self.active_attacks.get_mut(&entity) {
            attack.cancel();
        }
    }

    /// Update all attacks and projectiles.
    pub fn tick(&mut self, dt: f32) {
        // Update cooldowns
        for cooldown in self.cooldowns.values_mut() {
            *cooldown = (*cooldown - dt).max(0.0);
        }

        // Update active attacks
        for attack in self.active_attacks.values_mut() {
            attack.tick(dt);
        }

        // Update projectiles
        for projectile in &mut self.projectiles {
            projectile.tick(dt);
        }

        // Remove inactive projectiles
        self.projectiles.retain(|p| p.active);
    }

    /// Get active attack for entity.
    #[must_use]
    pub fn get_attack(&self, entity: EntityId) -> Option<&ActiveRangedAttack> {
        self.active_attacks.get(&entity)
    }

    /// Get all active projectiles.
    pub fn projectiles(&self) -> &[RangedProjectile] {
        &self.projectiles
    }

    /// Get mutable projectiles.
    pub fn projectiles_mut(&mut self) -> &mut Vec<RangedProjectile> {
        &mut self.projectiles
    }

    /// Check if entity is drawing.
    #[must_use]
    pub fn is_drawing(&self, entity: EntityId) -> bool {
        self.active_attacks
            .get(&entity)
            .is_some_and(|a| a.state.is_drawing())
    }

    /// Get ammo inventory for entity.
    #[must_use]
    pub fn get_ammo(&self, entity: EntityId) -> Option<&AmmoInventory> {
        self.ammo_inventories.get(&entity)
    }

    /// Get mutable ammo inventory.
    pub fn get_ammo_mut(&mut self, entity: EntityId) -> &mut AmmoInventory {
        self.ammo_inventories.entry(entity).or_default()
    }

    /// Set ammo inventory for entity.
    pub fn set_ammo(&mut self, entity: EntityId, ammo: AmmoInventory) {
        self.ammo_inventories.insert(entity, ammo);
    }

    /// Process hit for a projectile.
    pub fn process_hit(
        &mut self,
        projectile_idx: usize,
        target: EntityId,
        hit_pos: (f32, f32),
        critical: bool,
        headshot: bool,
    ) -> Option<RangedHit> {
        let projectile = self.projectiles.get_mut(projectile_idx)?;

        if !projectile.active {
            return None;
        }

        let mut damage = projectile.damage;
        if critical {
            damage *= 1.5;
        }
        if headshot {
            damage *= projectile.headshot_mult;
        }

        // Apply ammo type multiplier
        if let Some(ammo) = projectile.ammo_type {
            damage *= ammo.damage_multiplier();
        }

        projectile.on_hit();

        Some(RangedHit {
            target,
            damage,
            damage_type: projectile.damage_type,
            position: hit_pos,
            critical,
            headshot,
        })
    }

    /// Remove entity from system.
    pub fn remove_entity(&mut self, entity: EntityId) {
        self.active_attacks.remove(&entity);
        self.ammo_inventories.remove(&entity);
        self.cooldowns.remove(&entity);
        self.projectiles.retain(|p| p.owner != entity);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entity() -> EntityId {
        EntityId::from_raw(1)
    }

    fn test_target() -> EntityId {
        EntityId::from_raw(2)
    }

    #[test]
    fn test_weapon_type_properties() {
        assert!(RangedWeaponType::Bow.requires_draw());
        assert!(!RangedWeaponType::Crossbow.requires_draw());
        assert!(RangedWeaponType::ThrowingKnife.is_thrown());
        assert!(RangedWeaponType::Bow.uses_ammo());
        assert!(!RangedWeaponType::ThrowingAxe.uses_ammo());
    }

    #[test]
    fn test_ammo_type_damage() {
        assert_eq!(AmmoType::Arrow.damage_type(), RangedDamageType::Physical);
        assert_eq!(AmmoType::FireArrow.damage_type(), RangedDamageType::Fire);
        assert!(AmmoType::ExplosiveArrow.has_aoe());
    }

    #[test]
    fn test_weapon_damage_scaling() {
        let weapon = RangedWeaponStats::default().with_damage(100.0);

        // Full draw = full damage
        assert_eq!(weapon.damage_at_draw(1.0), 100.0);

        // Half draw = quarter damage (quadratic)
        assert_eq!(weapon.damage_at_draw(0.5), 25.0);
    }

    #[test]
    fn test_active_attack_draw() {
        let weapon = RangedWeaponStats::default();
        let mut attack = ActiveRangedAttack::new(test_entity(), weapon);

        assert_eq!(attack.state, DrawState::Drawing);
        assert_eq!(attack.draw_amount, 0.0);

        attack.tick(0.5);
        assert_eq!(attack.draw_amount, 0.5);

        attack.tick(0.5);
        assert_eq!(attack.state, DrawState::Held);
        assert_eq!(attack.draw_amount, 1.0);
    }

    #[test]
    fn test_attack_release() {
        let weapon = RangedWeaponStats::default().with_damage(100.0);
        let mut attack = ActiveRangedAttack::new(test_entity(), weapon);

        // Can't fire at start
        assert!(!attack.can_fire());

        // Draw past minimum
        attack.tick(0.5);
        assert!(attack.can_fire());

        let data = attack.release().unwrap();
        assert!(data.damage > 0.0);
        assert_eq!(attack.state, DrawState::Releasing);
    }

    #[test]
    fn test_projectile_movement() {
        let data = ProjectileData {
            damage: 10.0,
            speed: 40.0,
            direction: 0.0, // Right
            range: 100.0,
            accuracy: 1.0,
            ammo_type: Some(AmmoType::Arrow),
            crit_bonus: 0.0,
            headshot_mult: 2.0,
        };

        let mut projectile = RangedProjectile::new(test_entity(), (0.0, 0.0), &data);

        projectile.tick(1.0);

        // Should have moved right
        assert!(projectile.position.0 > 0.0);
        assert!(projectile.active);
    }

    #[test]
    fn test_projectile_range_limit() {
        let data = ProjectileData {
            damage: 10.0,
            speed: 100.0,
            direction: 0.0,
            range: 10.0, // Short range
            accuracy: 1.0,
            ammo_type: None,
            crit_bonus: 0.0,
            headshot_mult: 2.0,
        };

        let mut projectile =
            RangedProjectile::new(test_entity(), (0.0, 0.0), &data).with_gravity(0.0);

        projectile.tick(1.0); // Should exceed range
        assert!(!projectile.active);
    }

    #[test]
    fn test_ammo_inventory() {
        let mut ammo = AmmoInventory::new();

        ammo.add(AmmoType::Arrow, 50);
        assert_eq!(ammo.count(AmmoType::Arrow), 50);

        assert!(ammo.consume(AmmoType::Arrow, 10));
        assert_eq!(ammo.count(AmmoType::Arrow), 40);

        assert!(!ammo.consume(AmmoType::Arrow, 100));
        assert_eq!(ammo.count(AmmoType::Arrow), 40);
    }

    #[test]
    fn test_ammo_selection() {
        let mut ammo = AmmoInventory::new();
        ammo.add(AmmoType::Arrow, 10);
        ammo.add(AmmoType::FireArrow, 5);

        ammo.select(AmmoType::FireArrow);
        assert_eq!(ammo.get_selected(), Some(AmmoType::FireArrow));

        let consumed = ammo.consume_selected();
        assert_eq!(consumed, Some(AmmoType::FireArrow));
        assert_eq!(ammo.count(AmmoType::FireArrow), 4);
    }

    #[test]
    fn test_ranged_system_draw() {
        let mut system = RangedCombatSystem::new();
        let weapon = RangedWeaponStats::default();

        // Add ammo
        system.get_ammo_mut(test_entity()).add(AmmoType::Arrow, 10);
        system.get_ammo_mut(test_entity()).select(AmmoType::Arrow);

        let result = system.start_draw(test_entity(), weapon, true);
        assert!(matches!(result, RangedAttackResult::Drawing));

        assert!(system.is_drawing(test_entity()));
    }

    #[test]
    fn test_ranged_system_no_ammo() {
        let mut system = RangedCombatSystem::new();
        let weapon = RangedWeaponStats::default();

        let result = system.start_draw(test_entity(), weapon, true);
        assert!(matches!(result, RangedAttackResult::NoAmmo));
    }

    #[test]
    fn test_thrown_weapon() {
        let mut system = RangedCombatSystem::new();
        let weapon = RangedWeaponStats::for_type(RangedWeaponType::ThrowingKnife);

        let result = system.throw_weapon(test_entity(), &weapon, (0.0, 0.0), 0.0, 100.0, true);

        assert!(matches!(result, RangedAttackResult::Thrown(_)));
        assert_eq!(system.projectiles().len(), 1);
    }

    #[test]
    fn test_crossbow_reload() {
        let weapon = RangedWeaponStats::for_type(RangedWeaponType::Crossbow);
        let mut attack = ActiveRangedAttack::new(test_entity(), weapon);

        // Crossbow doesn't draw, starts idle
        assert_eq!(attack.state, DrawState::Idle);

        attack.start_draw();
        attack.tick(0.1); // Should be at held quickly
        assert_eq!(attack.state, DrawState::Held);

        attack.release();
        attack.start_reload();
        assert_eq!(attack.state, DrawState::Reloading);
    }

    #[test]
    fn test_draw_fatigue() {
        let weapon = RangedWeaponStats::default();
        let mut attack = ActiveRangedAttack::new(test_entity(), weapon);

        attack.tick(1.0); // Full draw
        assert_eq!(attack.state, DrawState::Held);

        attack.tick(5.0); // Hold for 5 seconds
        assert!(attack.fatigue > 0.0);
        assert!(attack.accuracy_percent() < 100.0);
    }
}
