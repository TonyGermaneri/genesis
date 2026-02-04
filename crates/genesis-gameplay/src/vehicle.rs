//! Vehicle system for rideable vehicles.

use crate::physics::{CollisionQuery, AABB};
use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for vehicle operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VehicleError {
    /// Vehicle is at capacity
    #[error("Vehicle is full")]
    VehicleFull,
    /// Entity is not in a vehicle
    #[error("Entity is not in a vehicle")]
    NotInVehicle,
    /// Too far away from vehicle
    #[error("Too far away from vehicle (distance: {0:.1})")]
    TooFarAway(u32),
    /// Vehicle is destroyed
    #[error("Vehicle is destroyed")]
    VehicleDestroyed,
    /// Vehicle not found
    #[error("Vehicle not found: {0:?}")]
    VehicleNotFound(EntityId),
    /// Entity already in vehicle
    #[error("Entity is already in a vehicle")]
    AlreadyInVehicle,
    /// Cannot exit here
    #[error("Cannot exit here - blocked")]
    ExitBlocked,
    /// Vehicle has no fuel
    #[error("Vehicle has no fuel")]
    NoFuel,
}

/// Result type for vehicle operations.
pub type VehicleResult<T> = Result<T, VehicleError>;

/// Type of vehicle with specific properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VehicleType {
    /// Ground vehicle pulled by animals or pushed
    Cart {
        /// Maximum speed
        max_speed: f32,
        /// Cargo capacity in slots
        capacity: u32,
    },
    /// Water vehicle
    Boat {
        /// Maximum speed
        max_speed: f32,
        /// Whether the boat can dive underwater
        can_dive: bool,
    },
    /// Rail-based vehicle
    Minecart {
        /// Whether confined to rails only
        rail_only: bool,
    },
    /// Animal or creature mount
    Mount {
        /// Stamina for running
        stamina: f32,
        /// Jump power multiplier
        jump_power: f32,
    },
}

impl VehicleType {
    /// Returns the stats for this vehicle type.
    #[must_use]
    pub fn stats(&self) -> VehicleStats {
        match self {
            VehicleType::Cart {
                max_speed,
                capacity,
            } => VehicleStats {
                max_speed: *max_speed,
                acceleration: 2.0,
                turn_rate: 1.5,
                passenger_slots: 2,
                cargo_slots: *capacity,
            },
            VehicleType::Boat { max_speed, .. } => VehicleStats {
                max_speed: *max_speed,
                acceleration: 1.0,
                turn_rate: 0.8,
                passenger_slots: 4,
                cargo_slots: 8,
            },
            VehicleType::Minecart { .. } => VehicleStats {
                max_speed: 8.0,
                acceleration: 5.0,
                turn_rate: 0.0, // Rails only
                passenger_slots: 1,
                cargo_slots: 4,
            },
            VehicleType::Mount {
                stamina,
                jump_power,
            } => VehicleStats {
                max_speed: 10.0 + *stamina * 0.1,
                acceleration: 4.0,
                turn_rate: 3.0,
                passenger_slots: 1 + u32::from(*jump_power > 1.5),
                cargo_slots: 2,
            },
        }
    }

    /// Returns whether this vehicle can operate in water.
    #[must_use]
    pub const fn is_water_vehicle(&self) -> bool {
        matches!(self, VehicleType::Boat { .. })
    }

    /// Returns whether this vehicle is confined to rails.
    #[must_use]
    pub const fn is_rail_bound(&self) -> bool {
        matches!(self, VehicleType::Minecart { rail_only: true })
    }

    /// Returns whether this vehicle is a living mount.
    #[must_use]
    pub const fn is_mount(&self) -> bool {
        matches!(self, VehicleType::Mount { .. })
    }
}

/// Stats defining vehicle capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VehicleStats {
    /// Maximum movement speed
    pub max_speed: f32,
    /// Acceleration rate
    pub acceleration: f32,
    /// Turn rate in radians per second
    pub turn_rate: f32,
    /// Number of passenger slots (including driver)
    pub passenger_slots: u32,
    /// Number of cargo inventory slots
    pub cargo_slots: u32,
}

impl Default for VehicleStats {
    fn default() -> Self {
        Self {
            max_speed: 5.0,
            acceleration: 2.0,
            turn_rate: 2.0,
            passenger_slots: 1,
            cargo_slots: 4,
        }
    }
}

/// State of an individual vehicle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleState {
    /// Type of vehicle
    pub vehicle_type: VehicleType,
    /// Current driver (None if empty)
    pub driver: Option<EntityId>,
    /// Current passengers (not including driver)
    pub passengers: Vec<EntityId>,
    /// Current fuel level
    pub fuel: f32,
    /// Maximum fuel capacity
    pub max_fuel: f32,
    /// Current health
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
    /// Current velocity (x, y)
    pub velocity: (f32, f32),
    /// Current position (x, y)
    pub position: (f32, f32),
    /// Facing angle in radians
    pub facing: f32,
    /// Whether vehicle is on ground/water
    pub grounded: bool,
    /// Whether vehicle is destroyed
    pub destroyed: bool,
}

impl VehicleState {
    /// Creates a new vehicle state.
    #[must_use]
    pub fn new(vehicle_type: VehicleType, position: (f32, f32)) -> Self {
        let is_mount = vehicle_type.is_mount();
        Self {
            vehicle_type,
            driver: None,
            passengers: Vec::new(),
            fuel: if is_mount { 0.0 } else { 100.0 },
            max_fuel: if is_mount { 0.0 } else { 100.0 },
            health: 100.0,
            max_health: 100.0,
            velocity: (0.0, 0.0),
            position,
            facing: 0.0,
            grounded: true,
            destroyed: false,
        }
    }

    /// Sets the fuel capacity.
    #[must_use]
    pub const fn with_fuel(mut self, current: f32, max: f32) -> Self {
        self.fuel = current;
        self.max_fuel = max;
        self
    }

    /// Sets the health.
    #[must_use]
    pub const fn with_health(mut self, current: f32, max: f32) -> Self {
        self.health = current;
        self.max_health = max;
        self
    }

    /// Returns the vehicle stats.
    #[must_use]
    pub fn stats(&self) -> VehicleStats {
        self.vehicle_type.stats()
    }

    /// Returns whether the vehicle is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.driver.is_none() && self.passengers.is_empty()
    }

    /// Returns whether the vehicle is full.
    #[must_use]
    pub fn is_full(&self) -> bool {
        let stats = self.stats();
        let occupied = u32::from(self.driver.is_some()) + self.passengers.len() as u32;
        occupied >= stats.passenger_slots
    }

    /// Returns the current speed.
    #[must_use]
    pub fn speed(&self) -> f32 {
        let (vx, vy) = self.velocity;
        (vx * vx + vy * vy).sqrt()
    }

    /// Returns the fuel percentage.
    #[must_use]
    pub fn fuel_percent(&self) -> f32 {
        if self.max_fuel <= 0.0 {
            1.0 // Mounts don't use fuel
        } else {
            self.fuel / self.max_fuel
        }
    }

    /// Returns the health percentage.
    #[must_use]
    pub fn health_percent(&self) -> f32 {
        if self.max_health <= 0.0 {
            0.0
        } else {
            self.health / self.max_health
        }
    }

    /// Returns all occupants (driver + passengers).
    #[must_use]
    pub fn occupants(&self) -> Vec<EntityId> {
        let mut result = Vec::with_capacity(1 + self.passengers.len());
        if let Some(driver) = self.driver {
            result.push(driver);
        }
        result.extend(&self.passengers);
        result
    }

    /// Returns distance to a position.
    #[must_use]
    pub fn distance_to(&self, pos: (f32, f32)) -> f32 {
        let dx = pos.0 - self.position.0;
        let dy = pos.1 - self.position.1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns the collision AABB for this vehicle.
    #[must_use]
    pub fn aabb(&self) -> AABB {
        // Vehicles are larger than players
        let half_width = 1.0;
        let half_height = 0.8;
        AABB {
            min_x: self.position.0 - half_width,
            min_y: self.position.1 - half_height,
            max_x: self.position.0 + half_width,
            max_y: self.position.1 + half_height,
        }
    }

    /// Applies damage to the vehicle.
    pub fn damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        if self.health <= 0.0 {
            self.destroyed = true;
        }
    }

    /// Repairs the vehicle.
    pub fn repair(&mut self, amount: f32) {
        if !self.destroyed {
            self.health = (self.health + amount).min(self.max_health);
        }
    }

    /// Refuels the vehicle.
    pub fn refuel(&mut self, amount: f32) {
        self.fuel = (self.fuel + amount).min(self.max_fuel);
    }

    /// Consumes fuel.
    pub fn consume_fuel(&mut self, amount: f32) -> bool {
        if self.fuel >= amount {
            self.fuel -= amount;
            true
        } else {
            false
        }
    }
}

/// Input state for controlling a vehicle.
#[derive(Debug, Clone, Copy, Default)]
pub struct VehicleInput {
    /// Forward/backward input (-1 to 1)
    pub throttle: f32,
    /// Left/right input (-1 to 1)
    pub steering: f32,
    /// Brake input (0 to 1)
    pub brake: f32,
    /// Jump/boost input
    pub boost: bool,
}

impl VehicleInput {
    /// Creates new vehicle input.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            throttle: 0.0,
            steering: 0.0,
            brake: 0.0,
            boost: false,
        }
    }

    /// Sets throttle.
    #[must_use]
    pub const fn with_throttle(mut self, throttle: f32) -> Self {
        self.throttle = throttle;
        self
    }

    /// Sets steering.
    #[must_use]
    pub const fn with_steering(mut self, steering: f32) -> Self {
        self.steering = steering;
        self
    }

    /// Sets brake.
    #[must_use]
    pub const fn with_brake(mut self, brake: f32) -> Self {
        self.brake = brake;
        self
    }
}

/// Vehicle system managing all vehicles.
#[derive(Debug, Default)]
pub struct VehicleSystem {
    /// All registered vehicles
    vehicles: HashMap<EntityId, VehicleState>,
    /// Map of entities to their current vehicle
    entity_to_vehicle: HashMap<EntityId, EntityId>,
    /// Next entity ID for spawning
    next_id: u64,
}

impl VehicleSystem {
    /// Creates a new vehicle system.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of vehicles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.vehicles.len()
    }

    /// Returns whether there are no vehicles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vehicles.is_empty()
    }

    /// Gets a vehicle's state.
    #[must_use]
    pub fn get(&self, vehicle: EntityId) -> Option<&VehicleState> {
        self.vehicles.get(&vehicle)
    }

    /// Gets mutable vehicle state.
    pub fn get_mut(&mut self, vehicle: EntityId) -> Option<&mut VehicleState> {
        self.vehicles.get_mut(&vehicle)
    }

    /// Gets the vehicle an entity is in.
    #[must_use]
    pub fn get_vehicle_for_entity(&self, entity: EntityId) -> Option<EntityId> {
        self.entity_to_vehicle.get(&entity).copied()
    }

    /// Checks if an entity is in a vehicle.
    #[must_use]
    pub fn is_in_vehicle(&self, entity: EntityId) -> bool {
        self.entity_to_vehicle.contains_key(&entity)
    }

    /// Spawns a new vehicle.
    pub fn spawn_vehicle(&mut self, vehicle_type: VehicleType, pos: (f32, f32)) -> EntityId {
        let id = EntityId::from_raw(self.next_id);
        self.next_id += 1;

        let state = VehicleState::new(vehicle_type, pos);
        self.vehicles.insert(id, state);

        id
    }

    /// Registers an existing entity as a vehicle.
    pub fn register_vehicle(
        &mut self,
        vehicle_id: EntityId,
        vehicle_type: VehicleType,
        pos: (f32, f32),
    ) {
        let state = VehicleState::new(vehicle_type, pos);
        self.vehicles.insert(vehicle_id, state);
    }

    /// Removes a vehicle.
    pub fn despawn_vehicle(&mut self, vehicle: EntityId) -> VehicleResult<VehicleState> {
        let state = self
            .vehicles
            .remove(&vehicle)
            .ok_or(VehicleError::VehicleNotFound(vehicle))?;

        // Remove all occupants from tracking
        for occupant in state.occupants() {
            self.entity_to_vehicle.remove(&occupant);
        }

        Ok(state)
    }

    /// Entity enters a vehicle.
    pub fn enter_vehicle(
        &mut self,
        entity: EntityId,
        vehicle: EntityId,
        entity_pos: (f32, f32),
    ) -> VehicleResult<()> {
        // Check if already in a vehicle
        if self.entity_to_vehicle.contains_key(&entity) {
            return Err(VehicleError::AlreadyInVehicle);
        }

        // Get vehicle state
        let state = self
            .vehicles
            .get_mut(&vehicle)
            .ok_or(VehicleError::VehicleNotFound(vehicle))?;

        // Check if destroyed
        if state.destroyed {
            return Err(VehicleError::VehicleDestroyed);
        }

        // Check distance (must be within 3 units)
        let dist = state.distance_to(entity_pos);
        if dist > 3.0 {
            return Err(VehicleError::TooFarAway((dist * 10.0) as u32));
        }

        // Check if full
        if state.is_full() {
            return Err(VehicleError::VehicleFull);
        }

        // Add as driver or passenger
        if state.driver.is_none() {
            state.driver = Some(entity);
        } else {
            state.passengers.push(entity);
        }

        // Track entity -> vehicle
        self.entity_to_vehicle.insert(entity, vehicle);

        Ok(())
    }

    /// Entity exits a vehicle.
    pub fn exit_vehicle<C: CollisionQuery>(
        &mut self,
        entity: EntityId,
        collision: &C,
    ) -> VehicleResult<(f32, f32)> {
        // Get vehicle for entity
        let vehicle_id = self
            .entity_to_vehicle
            .get(&entity)
            .copied()
            .ok_or(VehicleError::NotInVehicle)?;

        let state = self
            .vehicles
            .get_mut(&vehicle_id)
            .ok_or(VehicleError::VehicleNotFound(vehicle_id))?;

        // Find exit position
        let exit_pos = find_exit_position(state, collision)?;

        // Remove from vehicle
        if state.driver == Some(entity) {
            state.driver = None;
            // Promote first passenger to driver
            if !state.passengers.is_empty() {
                state.driver = Some(state.passengers.remove(0));
            }
        } else {
            state.passengers.retain(|&p| p != entity);
        }

        // Remove from tracking
        self.entity_to_vehicle.remove(&entity);

        Ok(exit_pos)
    }

    /// Updates all vehicles.
    pub fn update<C: CollisionQuery>(
        &mut self,
        dt: f32,
        inputs: &HashMap<EntityId, VehicleInput>,
        collision: &C,
    ) {
        let vehicle_ids: Vec<EntityId> = self.vehicles.keys().copied().collect();

        for vehicle_id in vehicle_ids {
            // Get driver's input
            let input = self
                .vehicles
                .get(&vehicle_id)
                .and_then(|v| v.driver)
                .and_then(|driver| inputs.get(&driver))
                .copied()
                .unwrap_or_default();

            if let Some(state) = self.vehicles.get_mut(&vehicle_id) {
                if !state.destroyed {
                    update_vehicle_physics(state, input, dt, collision);
                }
            }
        }
    }

    /// Gets the driver's position for a vehicle.
    #[must_use]
    pub fn get_driver_position(&self, vehicle: EntityId) -> Option<(f32, f32)> {
        self.vehicles.get(&vehicle).map(|v| v.position)
    }

    /// Returns iterator over all vehicles.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &VehicleState)> {
        self.vehicles.iter().map(|(&id, state)| (id, state))
    }

    /// Returns mutable iterator over all vehicles.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut VehicleState)> {
        self.vehicles.iter_mut().map(|(&id, state)| (id, state))
    }

    /// Gets all vehicles of a specific type.
    pub fn get_by_type(&self, matches: impl Fn(&VehicleType) -> bool) -> Vec<EntityId> {
        self.vehicles
            .iter()
            .filter(|(_, state)| matches(&state.vehicle_type))
            .map(|(&id, _)| id)
            .collect()
    }

    /// Gets all vehicles in range of a position.
    pub fn get_in_range(&self, pos: (f32, f32), range: f32) -> Vec<EntityId> {
        self.vehicles
            .iter()
            .filter(|(_, state)| state.distance_to(pos) <= range)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Gets the nearest vehicle to a position.
    #[must_use]
    pub fn get_nearest(&self, pos: (f32, f32)) -> Option<EntityId> {
        self.vehicles
            .iter()
            .filter(|(_, state)| !state.destroyed)
            .min_by(|(_, a), (_, b)| {
                a.distance_to(pos)
                    .partial_cmp(&b.distance_to(pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(&id, _)| id)
    }
}

/// Finds a valid exit position near the vehicle.
fn find_exit_position<C: CollisionQuery>(
    state: &VehicleState,
    collision: &C,
) -> VehicleResult<(f32, f32)> {
    // Try positions around the vehicle
    let offsets = [
        (2.0, 0.0),   // Right
        (-2.0, 0.0),  // Left
        (0.0, 2.0),   // Up
        (0.0, -2.0),  // Down
        (1.5, 1.5),   // Diagonal
        (-1.5, 1.5),  // Diagonal
        (1.5, -1.5),  // Diagonal
        (-1.5, -1.5), // Diagonal
    ];

    for (dx, dy) in offsets {
        let test_pos = (state.position.0 + dx, state.position.1 + dy);
        let test_aabb = AABB {
            min_x: test_pos.0 - 0.4,
            min_y: test_pos.1 - 0.9,
            max_x: test_pos.0 + 0.4,
            max_y: test_pos.1 + 0.9,
        };

        if !collision.check_collision(test_aabb) {
            return Ok(test_pos);
        }
    }

    Err(VehicleError::ExitBlocked)
}

/// Updates vehicle physics for one frame.
fn update_vehicle_physics<C: CollisionQuery>(
    state: &mut VehicleState,
    input: VehicleInput,
    dt: f32,
    collision: &C,
) {
    let stats = state.stats();

    // Check fuel (mounts don't use fuel)
    let has_fuel = state.max_fuel <= 0.0 || state.fuel > 0.0;

    // Apply steering (rotate facing direction)
    if input.steering.abs() > 0.01 && state.speed() > 0.1 {
        state.facing += input.steering * stats.turn_rate * dt;
        // Normalize to 0..2PI
        while state.facing < 0.0 {
            state.facing += std::f32::consts::TAU;
        }
        while state.facing >= std::f32::consts::TAU {
            state.facing -= std::f32::consts::TAU;
        }
    }

    // Calculate direction from facing
    let dir_x = state.facing.cos();
    let dir_y = state.facing.sin();

    // Apply throttle
    if has_fuel && input.throttle.abs() > 0.01 && !state.destroyed {
        let accel = input.throttle * stats.acceleration * dt;
        state.velocity.0 += dir_x * accel;
        state.velocity.1 += dir_y * accel;

        // Consume fuel
        if state.max_fuel > 0.0 {
            state.consume_fuel(dt * 0.5 * input.throttle.abs());
        }
    }

    // Apply braking
    if input.brake > 0.01 {
        let brake_force = input.brake * stats.acceleration * 2.0 * dt;
        let speed = state.speed();
        if speed > 0.01 {
            let slow = (speed - brake_force).max(0.0) / speed;
            state.velocity.0 *= slow;
            state.velocity.1 *= slow;
        }
    }

    // Clamp speed
    let speed = state.speed();
    if speed > stats.max_speed {
        let scale = stats.max_speed / speed;
        state.velocity.0 *= scale;
        state.velocity.1 *= scale;
    }

    // Apply friction/drag
    let friction = 0.98_f32;
    state.velocity.0 *= friction;
    state.velocity.1 *= friction;

    // Apply velocity to position
    let new_pos = (
        state.position.0 + state.velocity.0 * dt,
        state.position.1 + state.velocity.1 * dt,
    );

    // Check collision
    let test_aabb = AABB {
        min_x: new_pos.0 - 1.0,
        min_y: new_pos.1 - 0.8,
        max_x: new_pos.0 + 1.0,
        max_y: new_pos.1 + 0.8,
    };

    if collision.check_collision(test_aabb) {
        // Stop on collision
        state.velocity = (0.0, 0.0);
    } else {
        state.position = new_pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::MockCollision;

    #[test]
    fn test_vehicle_type_cart() {
        let cart = VehicleType::Cart {
            max_speed: 5.0,
            capacity: 8,
        };
        let stats = cart.stats();
        assert_eq!(stats.max_speed, 5.0);
        assert_eq!(stats.cargo_slots, 8);
        assert!(!cart.is_water_vehicle());
        assert!(!cart.is_mount());
    }

    #[test]
    fn test_vehicle_type_boat() {
        let boat = VehicleType::Boat {
            max_speed: 6.0,
            can_dive: true,
        };
        let stats = boat.stats();
        assert_eq!(stats.max_speed, 6.0);
        assert!(boat.is_water_vehicle());
    }

    #[test]
    fn test_vehicle_type_minecart() {
        let cart = VehicleType::Minecart { rail_only: true };
        assert!(cart.is_rail_bound());

        let cart2 = VehicleType::Minecart { rail_only: false };
        assert!(!cart2.is_rail_bound());
    }

    #[test]
    fn test_vehicle_type_mount() {
        let mount = VehicleType::Mount {
            stamina: 50.0,
            jump_power: 2.0,
        };
        assert!(mount.is_mount());
        let stats = mount.stats();
        assert!(stats.max_speed > 10.0);
        assert_eq!(stats.passenger_slots, 2); // Extra slot for high jump_power
    }

    #[test]
    fn test_vehicle_stats_default() {
        let stats = VehicleStats::default();
        assert_eq!(stats.max_speed, 5.0);
        assert_eq!(stats.passenger_slots, 1);
    }

    #[test]
    fn test_vehicle_state_creation() {
        let state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (10.0, 20.0),
        );

        assert_eq!(state.position, (10.0, 20.0));
        assert!(state.is_empty());
        assert!(!state.is_full());
        assert!(!state.destroyed);
    }

    #[test]
    fn test_vehicle_state_mount_no_fuel() {
        let state = VehicleState::new(
            VehicleType::Mount {
                stamina: 50.0,
                jump_power: 1.0,
            },
            (0.0, 0.0),
        );

        // Mounts don't use fuel
        assert_eq!(state.fuel, 0.0);
        assert_eq!(state.max_fuel, 0.0);
        assert_eq!(state.fuel_percent(), 1.0);
    }

    #[test]
    fn test_vehicle_state_fuel() {
        let mut state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        )
        .with_fuel(50.0, 100.0);

        assert_eq!(state.fuel_percent(), 0.5);

        state.refuel(30.0);
        assert_eq!(state.fuel, 80.0);

        state.refuel(50.0); // Overflow
        assert_eq!(state.fuel, 100.0);

        assert!(state.consume_fuel(20.0));
        assert_eq!(state.fuel, 80.0);

        assert!(!state.consume_fuel(100.0)); // Not enough
        assert_eq!(state.fuel, 80.0);
    }

    #[test]
    fn test_vehicle_state_health() {
        let mut state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        )
        .with_health(100.0, 100.0);

        assert_eq!(state.health_percent(), 1.0);

        state.damage(30.0);
        assert_eq!(state.health, 70.0);
        assert!(!state.destroyed);

        state.repair(20.0);
        assert_eq!(state.health, 90.0);

        state.damage(100.0);
        assert_eq!(state.health, 0.0);
        assert!(state.destroyed);

        // Can't repair when destroyed
        state.repair(50.0);
        assert_eq!(state.health, 0.0);
    }

    #[test]
    fn test_vehicle_state_speed() {
        let mut state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        assert_eq!(state.speed(), 0.0);

        state.velocity = (3.0, 4.0);
        assert!((state.speed() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_vehicle_state_occupants() {
        let mut state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        let driver = EntityId::new();
        let passenger = EntityId::new();

        state.driver = Some(driver);
        state.passengers.push(passenger);

        let occupants = state.occupants();
        assert_eq!(occupants.len(), 2);
        assert!(occupants.contains(&driver));
        assert!(occupants.contains(&passenger));
    }

    #[test]
    fn test_vehicle_state_distance() {
        let state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        assert!((state.distance_to((3.0, 4.0)) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_vehicle_state_aabb() {
        let state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (10.0, 20.0),
        );

        let aabb = state.aabb();
        assert!(aabb.min_x < 10.0);
        assert!(aabb.max_x > 10.0);
    }

    #[test]
    fn test_vehicle_input() {
        let input = VehicleInput::new()
            .with_throttle(0.5)
            .with_steering(-0.3)
            .with_brake(0.1);

        assert_eq!(input.throttle, 0.5);
        assert_eq!(input.steering, -0.3);
        assert_eq!(input.brake, 0.1);
    }

    #[test]
    fn test_vehicle_system_creation() {
        let system = VehicleSystem::new();
        assert!(system.is_empty());
        assert_eq!(system.len(), 0);
    }

    #[test]
    fn test_vehicle_spawn() {
        let mut system = VehicleSystem::new();
        let id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (10.0, 20.0),
        );

        assert_eq!(system.len(), 1);
        assert!(system.get(id).is_some());

        let vehicle = system.get(id).expect("Vehicle should exist");
        assert_eq!(vehicle.position, (10.0, 20.0));
    }

    #[test]
    fn test_vehicle_register() {
        let mut system = VehicleSystem::new();
        let id = EntityId::new();

        system.register_vehicle(
            id,
            VehicleType::Boat {
                max_speed: 6.0,
                can_dive: false,
            },
            (5.0, 5.0),
        );

        assert!(system.get(id).is_some());
    }

    #[test]
    fn test_vehicle_despawn() {
        let mut system = VehicleSystem::new();
        let id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        let state = system.despawn_vehicle(id).expect("Despawn should succeed");
        assert!(state.is_empty());
        assert!(system.is_empty());
    }

    #[test]
    fn test_vehicle_despawn_not_found() {
        let mut system = VehicleSystem::new();
        let fake_id = EntityId::new();

        let result = system.despawn_vehicle(fake_id);
        assert!(matches!(result, Err(VehicleError::VehicleNotFound(_))));
    }

    #[test]
    fn test_enter_vehicle() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        let entity_id = EntityId::new();

        system
            .enter_vehicle(entity_id, vehicle_id, (1.0, 1.0))
            .expect("Enter should succeed");

        assert!(system.is_in_vehicle(entity_id));
        assert_eq!(system.get_vehicle_for_entity(entity_id), Some(vehicle_id));

        let vehicle = system.get(vehicle_id).expect("Vehicle should exist");
        assert_eq!(vehicle.driver, Some(entity_id));
    }

    #[test]
    fn test_enter_vehicle_too_far() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        let entity_id = EntityId::new();

        let result = system.enter_vehicle(entity_id, vehicle_id, (100.0, 100.0));
        assert!(matches!(result, Err(VehicleError::TooFarAway(_))));
    }

    #[test]
    fn test_enter_vehicle_full() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Minecart { rail_only: true }, // Only 1 slot
            (0.0, 0.0),
        );

        let entity1 = EntityId::new();
        let entity2 = EntityId::new();

        system
            .enter_vehicle(entity1, vehicle_id, (0.0, 0.0))
            .expect("First enter should succeed");

        let result = system.enter_vehicle(entity2, vehicle_id, (0.0, 0.0));
        assert!(matches!(result, Err(VehicleError::VehicleFull)));
    }

    #[test]
    fn test_enter_vehicle_destroyed() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        // Destroy the vehicle
        if let Some(vehicle) = system.get_mut(vehicle_id) {
            vehicle.destroyed = true;
        }

        let entity_id = EntityId::new();
        let result = system.enter_vehicle(entity_id, vehicle_id, (0.0, 0.0));
        assert!(matches!(result, Err(VehicleError::VehicleDestroyed)));
    }

    #[test]
    fn test_enter_vehicle_already_in() {
        let mut system = VehicleSystem::new();
        let vehicle1 = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        let vehicle2 = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (10.0, 0.0),
        );

        let entity_id = EntityId::new();

        system
            .enter_vehicle(entity_id, vehicle1, (0.0, 0.0))
            .expect("First enter should succeed");

        let result = system.enter_vehicle(entity_id, vehicle2, (10.0, 0.0));
        assert!(matches!(result, Err(VehicleError::AlreadyInVehicle)));
    }

    #[test]
    fn test_exit_vehicle() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        let entity_id = EntityId::new();

        system
            .enter_vehicle(entity_id, vehicle_id, (0.0, 0.0))
            .expect("Enter should succeed");

        let collision = MockCollision::new(); // No obstacles
        let exit_pos = system
            .exit_vehicle(entity_id, &collision)
            .expect("Exit should succeed");

        assert!(!system.is_in_vehicle(entity_id));
        // Exit position should be offset from vehicle
        assert!((exit_pos.0 - 2.0).abs() < 0.1 || (exit_pos.0 + 2.0).abs() < 0.1);
    }

    #[test]
    fn test_exit_vehicle_not_in() {
        let mut system = VehicleSystem::new();
        let entity_id = EntityId::new();
        let collision = MockCollision::new();

        let result = system.exit_vehicle(entity_id, &collision);
        assert!(matches!(result, Err(VehicleError::NotInVehicle)));
    }

    #[test]
    fn test_exit_vehicle_promote_passenger() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        let driver = EntityId::new();
        let passenger = EntityId::new();

        system
            .enter_vehicle(driver, vehicle_id, (0.0, 0.0))
            .expect("Driver enter should succeed");
        system
            .enter_vehicle(passenger, vehicle_id, (0.0, 0.0))
            .expect("Passenger enter should succeed");

        let collision = MockCollision::new();
        system
            .exit_vehicle(driver, &collision)
            .expect("Driver exit should succeed");

        let vehicle = system.get(vehicle_id).expect("Vehicle should exist");
        assert_eq!(vehicle.driver, Some(passenger)); // Passenger promoted
    }

    #[test]
    fn test_update_vehicles() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        let driver = EntityId::new();
        system
            .enter_vehicle(driver, vehicle_id, (0.0, 0.0))
            .expect("Enter should succeed");

        let mut inputs = HashMap::new();
        inputs.insert(driver, VehicleInput::new().with_throttle(1.0));

        let collision = MockCollision::new();

        // Update several frames
        for _ in 0..10 {
            system.update(0.016, &inputs, &collision);
        }

        let vehicle = system.get(vehicle_id).expect("Vehicle should exist");
        assert!(vehicle.position.0 > 0.0 || vehicle.position.1.abs() > 0.0); // Moved
    }

    #[test]
    fn test_get_driver_position() {
        let mut system = VehicleSystem::new();
        let vehicle_id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (10.0, 20.0),
        );

        assert_eq!(system.get_driver_position(vehicle_id), Some((10.0, 20.0)));
    }

    #[test]
    fn test_get_by_type() {
        let mut system = VehicleSystem::new();
        system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        system.spawn_vehicle(
            VehicleType::Boat {
                max_speed: 6.0,
                can_dive: false,
            },
            (10.0, 0.0),
        );
        system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (20.0, 0.0),
        );

        let carts = system.get_by_type(|t| matches!(t, VehicleType::Cart { .. }));
        assert_eq!(carts.len(), 2);

        let boats = system.get_by_type(|t| t.is_water_vehicle());
        assert_eq!(boats.len(), 1);
    }

    #[test]
    fn test_get_in_range() {
        let mut system = VehicleSystem::new();
        system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (5.0, 0.0),
        );
        system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (20.0, 0.0),
        );

        let nearby = system.get_in_range((0.0, 0.0), 10.0);
        assert_eq!(nearby.len(), 2);
    }

    #[test]
    fn test_get_nearest() {
        let mut system = VehicleSystem::new();
        let v1 = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        let _v2 = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (10.0, 0.0),
        );

        let nearest = system.get_nearest((1.0, 1.0));
        assert_eq!(nearest, Some(v1));
    }

    #[test]
    fn test_iter_vehicles() {
        let mut system = VehicleSystem::new();
        let id1 = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        let id2 = system.spawn_vehicle(
            VehicleType::Boat {
                max_speed: 6.0,
                can_dive: false,
            },
            (10.0, 0.0),
        );

        let ids: Vec<EntityId> = system.iter().map(|(id, _)| id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_iter_mut_vehicles() {
        let mut system = VehicleSystem::new();
        let id = system.spawn_vehicle(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );

        for (vehicle_id, state) in system.iter_mut() {
            if vehicle_id == id {
                state.fuel = 50.0;
            }
        }

        let vehicle = system.get(id).expect("Vehicle should exist");
        assert_eq!(vehicle.fuel, 50.0);
    }

    #[test]
    fn test_vehicle_physics_throttle() {
        let mut state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        state.facing = 0.0; // Facing right

        let collision = MockCollision::new();
        let input = VehicleInput::new().with_throttle(1.0);

        update_vehicle_physics(&mut state, input, 0.1, &collision);

        assert!(state.velocity.0 > 0.0); // Moving right
    }

    #[test]
    fn test_vehicle_physics_steering() {
        let mut state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        state.velocity = (1.0, 0.0); // Moving

        let collision = MockCollision::new();
        let input = VehicleInput::new().with_steering(1.0);

        let original_facing = state.facing;
        update_vehicle_physics(&mut state, input, 0.1, &collision);

        assert!(state.facing != original_facing); // Facing changed
    }

    #[test]
    fn test_vehicle_physics_brake() {
        let mut state = VehicleState::new(
            VehicleType::Cart {
                max_speed: 5.0,
                capacity: 4,
            },
            (0.0, 0.0),
        );
        state.velocity = (5.0, 0.0);

        let collision = MockCollision::new();
        let input = VehicleInput::new().with_brake(1.0);

        update_vehicle_physics(&mut state, input, 0.1, &collision);

        assert!(state.speed() < 5.0); // Slowed down
    }

    #[test]
    fn test_vehicle_error_variants() {
        let _full = VehicleError::VehicleFull;
        let _not_in = VehicleError::NotInVehicle;
        let _too_far = VehicleError::TooFarAway(50);
        let _destroyed = VehicleError::VehicleDestroyed;
        let _not_found = VehicleError::VehicleNotFound(EntityId::new());
        let _already_in = VehicleError::AlreadyInVehicle;
        let _blocked = VehicleError::ExitBlocked;
        let _no_fuel = VehicleError::NoFuel;

        // Test error messages
        assert!(_full.to_string().contains("full"));
        assert!(_too_far.to_string().contains("far"));
    }
}
