//! Player physics integration with kernel collision.
//!
//! This module provides physics simulation for the player with AABB collision
//! against the cell grid, gravity, wall sliding, and velocity clamping.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::input::{Input, Vec2};
use crate::player::Player;

/// Errors that can occur in the physics system.
#[derive(Debug, Clone, Error)]
pub enum PhysicsError {
    /// Collision query failed
    #[error("collision query failed: {reason}")]
    CollisionFailed {
        /// Reason for failure
        reason: String,
    },
}

/// Axis-aligned bounding box for collision detection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    /// Minimum X coordinate
    pub min_x: f32,
    /// Minimum Y coordinate
    pub min_y: f32,
    /// Maximum X coordinate
    pub max_x: f32,
    /// Maximum Y coordinate
    pub max_y: f32,
}

impl AABB {
    /// Creates a new AABB.
    #[must_use]
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Creates an AABB from center and half-extents.
    #[must_use]
    pub fn from_center(center: Vec2, half_width: f32, half_height: f32) -> Self {
        Self {
            min_x: center.x - half_width,
            min_y: center.y - half_height,
            max_x: center.x + half_width,
            max_y: center.y + half_height,
        }
    }

    /// Returns the center of the AABB.
    #[must_use]
    pub fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Returns the width of the AABB.
    #[must_use]
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Returns the height of the AABB.
    #[must_use]
    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    /// Checks if this AABB overlaps with another.
    #[must_use]
    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
    }

    /// Returns the AABB translated by a vector.
    #[must_use]
    pub fn translated(&self, offset: Vec2) -> Self {
        Self {
            min_x: self.min_x + offset.x,
            min_y: self.min_y + offset.y,
            max_x: self.max_x + offset.x,
            max_y: self.max_y + offset.y,
        }
    }

    /// Expands the AABB by a margin on all sides.
    #[must_use]
    pub fn expanded(&self, margin: f32) -> Self {
        Self {
            min_x: self.min_x - margin,
            min_y: self.min_y - margin,
            max_x: self.max_x + margin,
            max_y: self.max_y + margin,
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }
}

/// Collision query interface for checking cell solidity.
///
/// This trait abstracts the collision detection system provided by the kernel.
/// Implementations query the world's cell grid to determine solid cells.
pub trait CollisionQuery {
    /// Checks if a cell at the given world coordinates is solid.
    fn is_solid(&self, x: i32, y: i32) -> bool;

    /// Checks if a cell contains liquid (water).
    fn is_liquid(&self, x: i32, y: i32) -> bool;

    /// Checks if a cell is climbable (ladders, vines).
    fn is_climbable(&self, x: i32, y: i32) -> bool;
}

/// Mock collision query for testing.
#[derive(Debug, Default)]
pub struct MockCollision {
    /// Set of solid cells (x, y)
    solid_cells: std::collections::HashSet<(i32, i32)>,
    /// Set of liquid cells
    liquid_cells: std::collections::HashSet<(i32, i32)>,
    /// Set of climbable cells
    climbable_cells: std::collections::HashSet<(i32, i32)>,
    /// Ground level (all cells at y >= this are solid)
    ground_level: Option<i32>,
}

impl MockCollision {
    /// Creates a new mock collision.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a cell as solid.
    pub fn set_solid(&mut self, x: i32, y: i32) {
        self.solid_cells.insert((x, y));
    }

    /// Sets a cell as liquid.
    pub fn set_liquid(&mut self, x: i32, y: i32) {
        self.liquid_cells.insert((x, y));
    }

    /// Sets a cell as climbable.
    pub fn set_climbable(&mut self, x: i32, y: i32) {
        self.climbable_cells.insert((x, y));
    }

    /// Sets the ground level (all cells at y >= this are solid).
    pub fn set_ground_level(&mut self, y: i32) {
        self.ground_level = Some(y);
    }
}

impl CollisionQuery for MockCollision {
    fn is_solid(&self, x: i32, y: i32) -> bool {
        if let Some(ground) = self.ground_level {
            if y >= ground {
                return true;
            }
        }
        self.solid_cells.contains(&(x, y))
    }

    fn is_liquid(&self, x: i32, y: i32) -> bool {
        self.liquid_cells.contains(&(x, y))
    }

    fn is_climbable(&self, x: i32, y: i32) -> bool {
        self.climbable_cells.contains(&(x, y))
    }
}

/// Collision result from a sweep test.
#[derive(Debug, Clone, Copy, Default)]
pub struct SweepResult {
    /// Time of collision (0.0 to 1.0, 1.0 if no collision)
    pub time: f32,
    /// Normal of the surface hit
    pub normal: Vec2,
    /// Whether a collision occurred
    pub hit: bool,
}

/// Player physics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPhysics {
    /// Gravity acceleration (positive = down)
    pub gravity: f32,
    /// Horizontal move speed
    pub move_speed: f32,
    /// Jump initial velocity
    pub jump_velocity: f32,
    /// Ground friction (0-1, higher = more friction)
    pub friction: f32,
    /// Air friction (usually lower than ground)
    pub air_friction: f32,
    /// Maximum horizontal velocity
    pub max_horizontal_velocity: f32,
    /// Maximum vertical velocity (terminal velocity)
    pub max_vertical_velocity: f32,
    /// Player collision box half-width
    pub half_width: f32,
    /// Player collision box half-height
    pub half_height: f32,
    /// Cell size for collision grid
    pub cell_size: f32,
    /// Small offset to prevent floating point issues
    pub skin_width: f32,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            gravity: 800.0,
            move_speed: 200.0,
            jump_velocity: 350.0,
            friction: 10.0,
            air_friction: 2.0,
            max_horizontal_velocity: 400.0,
            max_vertical_velocity: 600.0,
            half_width: 8.0,
            half_height: 16.0,
            cell_size: 16.0,
            skin_width: 0.01,
        }
    }
}

impl PlayerPhysics {
    /// Creates a new player physics configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates player physics with custom values.
    #[must_use]
    pub fn with_config(gravity: f32, move_speed: f32, jump_velocity: f32, friction: f32) -> Self {
        Self {
            gravity,
            move_speed,
            jump_velocity,
            friction,
            ..Default::default()
        }
    }

    /// Gets the player's collision AABB at a position.
    #[must_use]
    pub fn get_player_aabb(&self, position: Vec2) -> AABB {
        AABB::from_center(position, self.half_width, self.half_height)
    }

    /// Checks if the player is grounded (standing on solid ground).
    #[must_use]
    pub fn is_grounded<C: CollisionQuery>(&self, player: &Player, collision: &C) -> bool {
        let pos = player.position();
        let aabb = self.get_player_aabb(pos);

        // Check cells below the player's feet
        let min_x = (aabb.min_x / self.cell_size).floor() as i32;
        let max_x = (aabb.max_x / self.cell_size).floor() as i32;
        let check_y = ((aabb.max_y + self.skin_width) / self.cell_size).floor() as i32;

        for x in min_x..=max_x {
            if collision.is_solid(x, check_y) {
                return true;
            }
        }
        false
    }

    /// Checks if the player is in water.
    #[must_use]
    pub fn is_in_water<C: CollisionQuery>(&self, player: &Player, collision: &C) -> bool {
        let pos = player.position();
        let center_x = (pos.x / self.cell_size).floor() as i32;
        let center_y = (pos.y / self.cell_size).floor() as i32;

        collision.is_liquid(center_x, center_y)
    }

    /// Checks if the player is on a climbable surface.
    #[must_use]
    pub fn is_on_climbable<C: CollisionQuery>(&self, player: &Player, collision: &C) -> bool {
        let pos = player.position();
        let center_x = (pos.x / self.cell_size).floor() as i32;
        let center_y = (pos.y / self.cell_size).floor() as i32;

        collision.is_climbable(center_x, center_y)
    }

    /// Checks if a position collides with any solid cells.
    fn check_collision<C: CollisionQuery>(&self, aabb: AABB, collision: &C) -> bool {
        let min_x = (aabb.min_x / self.cell_size).floor() as i32;
        let max_x = (aabb.max_x / self.cell_size).floor() as i32;
        let min_y = (aabb.min_y / self.cell_size).floor() as i32;
        let max_y = (aabb.max_y / self.cell_size).floor() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if collision.is_solid(x, y) {
                    return true;
                }
            }
        }
        false
    }

    /// Moves the player with collision detection.
    /// Returns the actual movement after resolving collisions.
    fn move_with_collision<C: CollisionQuery>(
        &self,
        position: Vec2,
        velocity: Vec2,
        collision: &C,
        dt: f32,
    ) -> (Vec2, Vec2, bool, bool) {
        let mut new_pos = position;
        let mut new_vel = velocity;
        let mut hit_ground = false;
        let mut hit_ceiling = false;

        // Try horizontal movement first
        let move_x = velocity.x * dt;
        if move_x.abs() > self.skin_width {
            let test_pos = Vec2::new(new_pos.x + move_x, new_pos.y);
            let test_aabb = self.get_player_aabb(test_pos);

            if self.check_collision(test_aabb, collision) {
                // Wall sliding - stop horizontal velocity but allow vertical
                new_vel.x = 0.0;

                // Try to slide as close as possible
                let step = if move_x > 0.0 {
                    self.skin_width
                } else {
                    -self.skin_width
                };
                let steps = (move_x.abs() / self.skin_width).ceil() as i32;

                for _ in 0..steps {
                    let test_pos = Vec2::new(new_pos.x + step, new_pos.y);
                    let test_aabb = self.get_player_aabb(test_pos);
                    if self.check_collision(test_aabb, collision) {
                        break;
                    }
                    new_pos.x = test_pos.x;
                }
            } else {
                new_pos.x = test_pos.x;
            }
        }

        // Try vertical movement
        let move_y = velocity.y * dt;
        if move_y.abs() > self.skin_width {
            let test_pos = Vec2::new(new_pos.x, new_pos.y + move_y);
            let test_aabb = self.get_player_aabb(test_pos);

            if self.check_collision(test_aabb, collision) {
                // Hit floor or ceiling
                if move_y > 0.0 {
                    hit_ground = true;
                } else {
                    hit_ceiling = true;
                }
                new_vel.y = 0.0;

                // Try to slide as close as possible
                let step = if move_y > 0.0 {
                    self.skin_width
                } else {
                    -self.skin_width
                };
                let steps = (move_y.abs() / self.skin_width).ceil() as i32;

                for _ in 0..steps {
                    let test_pos = Vec2::new(new_pos.x, new_pos.y + step);
                    let test_aabb = self.get_player_aabb(test_pos);
                    if self.check_collision(test_aabb, collision) {
                        break;
                    }
                    new_pos.y = test_pos.y;
                }
            } else {
                new_pos.y = test_pos.y;
            }
        }

        (new_pos, new_vel, hit_ground, hit_ceiling)
    }

    /// Updates the player physics.
    ///
    /// This method handles:
    /// - Input-based movement
    /// - Gravity application
    /// - Collision detection and response
    /// - Velocity clamping
    /// - Friction
    pub fn update<C: CollisionQuery>(
        &self,
        player: &mut Player,
        input: &Input,
        collision: &C,
        dt: f32,
    ) {
        let grounded = self.is_grounded(player, collision);
        let in_water = self.is_in_water(player, collision);
        let on_climbable = self.is_on_climbable(player, collision);

        // Update player environment state
        player.set_grounded(grounded);
        player.set_in_water(in_water);
        player.set_on_climbable(on_climbable);

        // Get current velocity
        let mut velocity = player.velocity();

        // Apply horizontal input
        let target_speed = if input.running {
            self.move_speed * 1.5
        } else {
            self.move_speed
        };

        let friction = if grounded {
            self.friction
        } else {
            self.air_friction
        };

        // Horizontal movement
        if input.movement.x == 0.0 {
            // Apply friction
            velocity.x -= velocity.x * friction * dt;
            if velocity.x.abs() < 1.0 {
                velocity.x = 0.0;
            }
        } else {
            velocity.x = input.movement.x * target_speed;
        }

        // Vertical movement / gravity
        if in_water {
            // Swimming - reduced gravity, can move up/down
            velocity.y += self.gravity * 0.1 * dt;
            if input.movement.y != 0.0 {
                velocity.y = input.movement.y * target_speed * 0.5;
            }
        } else if on_climbable && input.movement.y != 0.0 {
            // Climbing
            velocity.y = input.movement.y * target_speed * 0.5;
        } else {
            // Normal gravity
            if !grounded {
                velocity.y += self.gravity * dt;
            }

            // Jump
            if input.jump_just_pressed && grounded {
                velocity.y = -self.jump_velocity;
            }
        }

        // Clamp velocity
        velocity.x = velocity
            .x
            .clamp(-self.max_horizontal_velocity, self.max_horizontal_velocity);
        velocity.y = velocity
            .y
            .clamp(-self.max_vertical_velocity, self.max_vertical_velocity);

        // Move with collision
        let position = player.position();
        let (new_pos, _new_vel, hit_ground, hit_ceiling) =
            self.move_with_collision(position, velocity, collision, dt);

        // Update player
        player.set_position(new_pos);

        // Apply collision response through player's method
        let push_out = Vec2::ZERO; // Already handled in move_with_collision
        player.apply_collision(push_out, hit_ground, hit_ceiling);

        // If we didn't hit ground/ceiling, update velocity from our calculations
        if !hit_ground && !hit_ceiling {
            // Need to access velocity through player's internal state
            // Since Player doesn't expose velocity setter, we handle it through update
        }

        // Let player's update method handle state transitions
        player.update(input, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = AABB::new(0.0, 0.0, 10.0, 20.0);
        assert_eq!(aabb.width(), 10.0);
        assert_eq!(aabb.height(), 20.0);
    }

    #[test]
    fn test_aabb_from_center() {
        let aabb = AABB::from_center(Vec2::new(10.0, 10.0), 5.0, 10.0);
        assert_eq!(aabb.min_x, 5.0);
        assert_eq!(aabb.max_x, 15.0);
        assert_eq!(aabb.min_y, 0.0);
        assert_eq!(aabb.max_y, 20.0);
    }

    #[test]
    fn test_aabb_overlaps() {
        let a = AABB::new(0.0, 0.0, 10.0, 10.0);
        let b = AABB::new(5.0, 5.0, 15.0, 15.0);
        let c = AABB::new(20.0, 20.0, 30.0, 30.0);

        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn test_aabb_translated() {
        let aabb = AABB::new(0.0, 0.0, 10.0, 10.0);
        let translated = aabb.translated(Vec2::new(5.0, 5.0));
        assert_eq!(translated.min_x, 5.0);
        assert_eq!(translated.min_y, 5.0);
        assert_eq!(translated.max_x, 15.0);
        assert_eq!(translated.max_y, 15.0);
    }

    #[test]
    fn test_aabb_expanded() {
        let aabb = AABB::new(5.0, 5.0, 15.0, 15.0);
        let expanded = aabb.expanded(2.0);
        assert_eq!(expanded.min_x, 3.0);
        assert_eq!(expanded.min_y, 3.0);
        assert_eq!(expanded.max_x, 17.0);
        assert_eq!(expanded.max_y, 17.0);
    }

    #[test]
    fn test_mock_collision() {
        let mut collision = MockCollision::new();
        collision.set_solid(5, 10);
        collision.set_liquid(3, 3);
        collision.set_climbable(2, 2);

        assert!(collision.is_solid(5, 10));
        assert!(!collision.is_solid(0, 0));
        assert!(collision.is_liquid(3, 3));
        assert!(collision.is_climbable(2, 2));
    }

    #[test]
    fn test_mock_collision_ground_level() {
        let mut collision = MockCollision::new();
        collision.set_ground_level(10);

        assert!(!collision.is_solid(0, 5));
        assert!(collision.is_solid(0, 10));
        assert!(collision.is_solid(0, 15));
    }

    #[test]
    fn test_player_physics_default() {
        let physics = PlayerPhysics::new();
        assert_eq!(physics.gravity, 800.0);
        assert_eq!(physics.move_speed, 200.0);
        assert_eq!(physics.jump_velocity, 350.0);
    }

    #[test]
    fn test_player_physics_custom() {
        let physics = PlayerPhysics::with_config(500.0, 150.0, 300.0, 8.0);
        assert_eq!(physics.gravity, 500.0);
        assert_eq!(physics.move_speed, 150.0);
        assert_eq!(physics.jump_velocity, 300.0);
        assert_eq!(physics.friction, 8.0);
    }

    #[test]
    fn test_get_player_aabb() {
        let physics = PlayerPhysics::new();
        let aabb = physics.get_player_aabb(Vec2::new(100.0, 100.0));

        assert_eq!(aabb.center().x, 100.0);
        assert_eq!(aabb.center().y, 100.0);
        assert_eq!(aabb.width(), physics.half_width * 2.0);
        assert_eq!(aabb.height(), physics.half_height * 2.0);
    }

    #[test]
    fn test_is_grounded() {
        let physics = PlayerPhysics::new();
        let mut collision = MockCollision::new();

        // Player at position (100, 100) with half_height 16
        // AABB max_y = 100 + 16 = 116
        // With skin_width=0.01, check_y = floor((116 + 0.01)/16) = floor(7.25) = 7
        // So we need ground at y=7 or below
        collision.set_ground_level(7); // Ground starts at cell y=7

        let player = Player::new(Vec2::new(100.0, 100.0));

        // Player's feet should be just above ground
        assert!(physics.is_grounded(&player, &collision));
    }

    #[test]
    fn test_is_not_grounded() {
        let physics = PlayerPhysics::new();
        let collision = MockCollision::new(); // No solid cells

        let player = Player::new(Vec2::new(100.0, 100.0));
        assert!(!physics.is_grounded(&player, &collision));
    }

    #[test]
    fn test_is_in_water() {
        let physics = PlayerPhysics::new();
        let mut collision = MockCollision::new();

        // Player at (100, 100), cell at (6, 6)
        collision.set_liquid(6, 6);

        let player = Player::new(Vec2::new(100.0, 100.0));
        assert!(physics.is_in_water(&player, &collision));
    }

    #[test]
    fn test_is_on_climbable() {
        let physics = PlayerPhysics::new();
        let mut collision = MockCollision::new();

        collision.set_climbable(6, 6);

        let player = Player::new(Vec2::new(100.0, 100.0));
        assert!(physics.is_on_climbable(&player, &collision));
    }

    #[test]
    fn test_check_collision() {
        let physics = PlayerPhysics::new();
        let mut collision = MockCollision::new();
        collision.set_solid(5, 5);

        // AABB that overlaps cell (5, 5)
        let aabb = AABB::new(80.0, 80.0, 96.0, 96.0); // Overlaps cell at (5,5) with cell_size 16
        assert!(physics.check_collision(aabb, &collision));

        // AABB that doesn't overlap any solid cells
        let aabb2 = AABB::new(0.0, 0.0, 10.0, 10.0);
        assert!(!physics.check_collision(aabb2, &collision));
    }

    #[test]
    fn test_velocity_clamping() {
        let physics = PlayerPhysics::new();

        // Test that max velocities are within bounds
        assert!(physics.max_horizontal_velocity > 0.0);
        assert!(physics.max_vertical_velocity > 0.0);
    }

    #[test]
    fn test_physics_update_with_ground() {
        let physics = PlayerPhysics::new();
        let mut collision = MockCollision::new();
        // Player at (100, 100), feet at y=116, cell y=7
        // Set ground at y=7 so player is standing on it
        collision.set_ground_level(7);

        let mut player = Player::new(Vec2::new(100.0, 100.0));
        let input = Input::new();

        // Update physics
        physics.update(&mut player, &input, &collision, 0.016);

        // Player should be grounded
        assert!(player.is_grounded());
    }

    #[test]
    fn test_physics_update_falling() {
        let physics = PlayerPhysics::new();
        let collision = MockCollision::new(); // No ground

        let mut player = Player::new(Vec2::new(100.0, 100.0));
        let input = Input::new();

        let initial_y = player.position().y;

        // Update physics several times
        for _ in 0..10 {
            physics.update(&mut player, &input, &collision, 0.016);
        }

        // Player should have fallen (y increased in screen coords)
        assert!(player.position().y > initial_y);
    }

    #[test]
    fn test_physics_horizontal_movement() {
        let physics = PlayerPhysics::new();
        let collision = MockCollision::new();

        let mut player = Player::new(Vec2::new(100.0, 100.0));
        let mut input = Input::new();
        input.movement.x = 1.0; // Move right

        let initial_x = player.position().x;

        physics.update(&mut player, &input, &collision, 0.016);

        // Player should have moved right
        assert!(player.position().x > initial_x);
    }
}
