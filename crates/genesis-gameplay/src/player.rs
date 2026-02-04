//! Player controller for movement and state management.
//!
//! This module provides the player entity with movement, physics, and state handling.

use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::input::{Input, Vec2};

/// Errors that can occur in the player system.
#[derive(Debug, Clone, Error)]
pub enum PlayerError {
    /// Player cannot move in the current state
    #[error("invalid movement in state: {0:?}")]
    InvalidMovementState(PlayerState),

    /// Player cannot perform action
    #[error("cannot perform action: {reason}")]
    CannotPerformAction {
        /// Reason the action cannot be performed
        reason: String,
    },
}

/// Direction the player is facing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Direction {
    /// Facing up
    Up,
    /// Facing down (default)
    #[default]
    Down,
    /// Facing left
    Left,
    /// Facing right
    Right,
}

impl Direction {
    /// Convert direction to a unit vector.
    #[must_use]
    pub fn to_vec2(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::UP,
            Direction::Down => Vec2::DOWN,
            Direction::Left => Vec2::LEFT,
            Direction::Right => Vec2::RIGHT,
        }
    }

    /// Create direction from a movement vector.
    #[must_use]
    pub fn from_vec2(v: Vec2) -> Option<Self> {
        if v.x == 0.0 && v.y == 0.0 {
            return None;
        }

        // Determine primary direction based on largest component
        if v.x.abs() > v.y.abs() {
            if v.x > 0.0 {
                Some(Direction::Right)
            } else {
                Some(Direction::Left)
            }
        } else if v.y > 0.0 {
            Some(Direction::Down)
        } else {
            Some(Direction::Up)
        }
    }
}

/// State of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PlayerState {
    /// Standing still
    #[default]
    Idle,
    /// Walking at normal speed
    Walking,
    /// Running at faster speed
    Running,
    /// In the air going up
    Jumping,
    /// In the air going down
    Falling,
    /// In water
    Swimming,
    /// On a climbable surface
    Climbing,
}

impl PlayerState {
    /// Check if the player is grounded (can jump).
    #[must_use]
    pub fn is_grounded(self) -> bool {
        matches!(
            self,
            PlayerState::Idle | PlayerState::Walking | PlayerState::Running
        )
    }

    /// Check if the player is in the air.
    #[must_use]
    pub fn is_airborne(self) -> bool {
        matches!(self, PlayerState::Jumping | PlayerState::Falling)
    }

    /// Check if the player can move horizontally.
    #[must_use]
    pub fn can_move(self) -> bool {
        // Player can move in all states (with different speeds)
        true
    }

    /// Check if the player is in water.
    #[must_use]
    pub fn is_swimming(self) -> bool {
        matches!(self, PlayerState::Swimming)
    }

    /// Check if the player is climbing.
    #[must_use]
    pub fn is_climbing(self) -> bool {
        matches!(self, PlayerState::Climbing)
    }
}

/// Animation state for rendering.
///
/// This enum is used to communicate the current animation to the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PlayerAnimationState {
    /// Standing still
    #[default]
    Idle,
    /// Walking at normal speed
    Walking,
    /// Running at faster speed
    Running,
    /// Jumping (going up)
    Jumping,
    /// Falling (going down)
    Falling,
    /// Digging/mining
    Digging,
    /// Swimming
    Swimming,
    /// Climbing
    Climbing,
}

impl PlayerAnimationState {
    /// Convert from PlayerState.
    #[must_use]
    pub fn from_player_state(state: PlayerState, is_digging: bool) -> Self {
        if is_digging {
            return Self::Digging;
        }
        match state {
            PlayerState::Idle => Self::Idle,
            PlayerState::Walking => Self::Walking,
            PlayerState::Running => Self::Running,
            PlayerState::Jumping => Self::Jumping,
            PlayerState::Falling => Self::Falling,
            PlayerState::Swimming => Self::Swimming,
            PlayerState::Climbing => Self::Climbing,
        }
    }
}

/// Player movement configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// Walk speed in units per second
    pub walk_speed: f32,
    /// Run speed in units per second
    pub run_speed: f32,
    /// Swim speed in units per second
    pub swim_speed: f32,
    /// Climb speed in units per second
    pub climb_speed: f32,
    /// Jump velocity (initial upward velocity)
    pub jump_velocity: f32,
    /// Gravity acceleration (positive = downward)
    pub gravity: f32,
    /// Terminal velocity (max fall speed)
    pub terminal_velocity: f32,
    /// Air control multiplier (0-1)
    pub air_control: f32,
    /// Maximum interaction distance
    pub interaction_range: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            walk_speed: 100.0,
            run_speed: 200.0,
            swim_speed: 75.0,
            climb_speed: 50.0,
            jump_velocity: 300.0,
            gravity: 800.0,
            terminal_velocity: 500.0,
            air_control: 0.3,
            interaction_range: 64.0,
        }
    }
}

/// The player entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique entity ID
    entity_id: EntityId,
    /// Position in world space
    position: Vec2,
    /// Current velocity
    velocity: Vec2,
    /// Direction the player is facing
    facing: Direction,
    /// Current state
    state: PlayerState,
    /// Configuration
    config: PlayerConfig,
    /// Whether the player is on the ground (collision result)
    grounded: bool,
    /// Whether the player is in water (collision result)
    in_water: bool,
    /// Whether the player is on a climbable surface
    on_climbable: bool,
    /// Jump buffer timer (allows pressing jump slightly before landing)
    jump_buffer: f32,
    /// Coyote time (allows jumping shortly after leaving ground)
    coyote_time: f32,
}

impl Player {
    /// Jump buffer duration in seconds.
    const JUMP_BUFFER_TIME: f32 = 0.1;
    /// Coyote time duration in seconds.
    const COYOTE_TIME: f32 = 0.08;

    /// Create a new player at the given position.
    #[must_use]
    pub fn new(position: Vec2) -> Self {
        Self {
            entity_id: EntityId::new(),
            position,
            velocity: Vec2::ZERO,
            facing: Direction::Down,
            state: PlayerState::Idle,
            config: PlayerConfig::default(),
            grounded: true,
            in_water: false,
            on_climbable: false,
            jump_buffer: 0.0,
            coyote_time: 0.0,
        }
    }

    /// Create a new player with custom configuration.
    #[must_use]
    pub fn with_config(position: Vec2, config: PlayerConfig) -> Self {
        Self {
            entity_id: EntityId::new(),
            position,
            velocity: Vec2::ZERO,
            facing: Direction::Down,
            state: PlayerState::Idle,
            config,
            grounded: true,
            in_water: false,
            on_climbable: false,
            jump_buffer: 0.0,
            coyote_time: 0.0,
        }
    }

    /// Get the player's entity ID.
    #[must_use]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Get the player's current position.
    #[must_use]
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Set the player's position directly.
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Get the player's current velocity.
    #[must_use]
    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Get the direction the player is facing.
    #[must_use]
    pub fn facing(&self) -> Direction {
        self.facing
    }

    /// Get the player's current state.
    #[must_use]
    pub fn state(&self) -> PlayerState {
        self.state
    }

    /// Get the player's configuration.
    #[must_use]
    pub fn config(&self) -> &PlayerConfig {
        &self.config
    }

    /// Get mutable reference to the player's configuration.
    pub fn config_mut(&mut self) -> &mut PlayerConfig {
        &mut self.config
    }

    /// Check if the player is grounded.
    #[must_use]
    pub fn is_grounded(&self) -> bool {
        self.grounded
    }

    /// Set grounded state (called by collision system).
    pub fn set_grounded(&mut self, grounded: bool) {
        let was_grounded = self.grounded;
        self.grounded = grounded;

        // Start coyote time when leaving ground
        if was_grounded && !grounded && !self.state.is_airborne() {
            self.coyote_time = Self::COYOTE_TIME;
        }
    }

    /// Set water state (called by collision system).
    pub fn set_in_water(&mut self, in_water: bool) {
        self.in_water = in_water;
    }

    /// Set climbable state (called by collision system).
    pub fn set_on_climbable(&mut self, on_climbable: bool) {
        self.on_climbable = on_climbable;
    }

    /// Check if a position is within interaction range.
    #[must_use]
    pub fn in_range(&self, target: Vec2) -> bool {
        self.position.distance(target) <= self.config.interaction_range
    }

    /// Update the player based on input.
    /// Returns the new position (collision should be checked externally).
    pub fn update(&mut self, input: &Input, dt: f32) {
        // Update timers
        self.jump_buffer = (self.jump_buffer - dt).max(0.0);
        self.coyote_time = (self.coyote_time - dt).max(0.0);

        // Handle jump input buffering
        if input.jump_just_pressed {
            self.jump_buffer = Self::JUMP_BUFFER_TIME;
        }

        // Update state based on environment
        self.update_state(input);

        // Update movement
        self.update_movement(input, dt);

        // Update facing direction
        if let Some(dir) = Direction::from_vec2(input.movement) {
            self.facing = dir;
        }
    }

    /// Update player state based on input and environment.
    fn update_state(&mut self, input: &Input) {
        // Check for swimming
        if self.in_water {
            self.state = PlayerState::Swimming;
            return;
        }

        // Check for climbing
        if self.on_climbable && (input.movement.y != 0.0 || self.state.is_climbing()) {
            self.state = PlayerState::Climbing;
            return;
        }

        // Check for jumping
        let can_jump = (self.grounded || self.coyote_time > 0.0) && self.jump_buffer > 0.0;

        if can_jump {
            self.velocity.y = -self.config.jump_velocity;
            self.state = PlayerState::Jumping;
            self.jump_buffer = 0.0;
            self.coyote_time = 0.0;
            self.grounded = false;
            return;
        }

        // Airborne states
        if !self.grounded {
            if self.velocity.y < 0.0 {
                self.state = PlayerState::Jumping;
            } else {
                self.state = PlayerState::Falling;
            }
            return;
        }

        // Grounded states
        if input.has_movement() {
            if input.running {
                self.state = PlayerState::Running;
            } else {
                self.state = PlayerState::Walking;
            }
        } else {
            self.state = PlayerState::Idle;
        }
    }

    /// Update movement based on state and input.
    fn update_movement(&mut self, input: &Input, dt: f32) {
        match self.state {
            PlayerState::Idle | PlayerState::Walking | PlayerState::Running => {
                self.update_ground_movement(input, dt);
            },
            PlayerState::Jumping | PlayerState::Falling => {
                self.update_air_movement(input, dt);
            },
            PlayerState::Swimming => {
                self.update_swim_movement(input, dt);
            },
            PlayerState::Climbing => {
                self.update_climb_movement(input, dt);
            },
        }
    }

    /// Ground movement update.
    fn update_ground_movement(&mut self, input: &Input, dt: f32) {
        let speed = if input.running {
            self.config.run_speed
        } else {
            self.config.walk_speed
        };

        // Direct control on ground
        let target_velocity = input.movement.scale(speed);
        self.velocity.x = target_velocity.x;

        // Reset vertical velocity when grounded
        if self.grounded {
            self.velocity.y = 0.0;
        }

        // Apply movement
        self.position += self.velocity.scale(dt);
    }

    /// Air movement update.
    fn update_air_movement(&mut self, input: &Input, dt: f32) {
        // Limited air control
        let target_x = input.movement.x * self.config.walk_speed;
        let control = self.config.air_control;
        self.velocity.x += (target_x - self.velocity.x) * control * dt * 10.0;

        // Apply gravity
        self.velocity.y += self.config.gravity * dt;

        // Clamp to terminal velocity
        if self.velocity.y > self.config.terminal_velocity {
            self.velocity.y = self.config.terminal_velocity;
        }

        // Apply movement
        self.position += self.velocity.scale(dt);
    }

    /// Swimming movement update.
    fn update_swim_movement(&mut self, input: &Input, dt: f32) {
        // Full directional control in water
        let target_velocity = input.movement.scale(self.config.swim_speed);
        self.velocity = target_velocity;

        // Apply movement
        self.position += self.velocity.scale(dt);
    }

    /// Climbing movement update.
    fn update_climb_movement(&mut self, input: &Input, dt: f32) {
        // Vertical movement when climbing
        let target_velocity = input.movement.scale(self.config.climb_speed);
        self.velocity = target_velocity;

        // Apply movement
        self.position += self.velocity.scale(dt);
    }

    /// Apply collision response (called after collision detection).
    pub fn apply_collision(&mut self, push_out: Vec2, hit_ground: bool, hit_ceiling: bool) {
        self.position += push_out;

        if hit_ground {
            self.velocity.y = 0.0;
            self.grounded = true;
        }

        if hit_ceiling && self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }
    }

    /// Teleport the player to a new position.
    pub fn teleport(&mut self, position: Vec2) {
        self.position = position;
        self.velocity = Vec2::ZERO;
        self.state = PlayerState::Falling;
        self.grounded = false;
    }

    /// Stop all movement.
    pub fn stop(&mut self) {
        self.velocity = Vec2::ZERO;
        if self.grounded {
            self.state = PlayerState::Idle;
        }
    }

    /// Process input and update player state.
    ///
    /// This is an alias for `update` that matches the engine integration API.
    pub fn handle_input(&mut self, input: &Input, dt: f32) {
        self.update(input, dt);
    }

    /// Apply physics (gravity, collision response) using a collision query.
    ///
    /// This method integrates with the physics system for world collision.
    pub fn apply_physics<C: crate::physics::CollisionQuery>(&mut self, collision: &C, dt: f32) {
        use crate::physics::AABB;

        // Get player AABB at current position
        let half_width = 6.0; // Half of player width in cells
        let half_height = 12.0; // Half of player height in cells
        let player_aabb = AABB::from_center(self.position, half_width, half_height);

        // Check ground collision
        let feet_aabb = AABB::new(
            player_aabb.min_x + 0.1,
            player_aabb.max_y,
            player_aabb.max_x - 0.1,
            player_aabb.max_y + 1.0,
        );
        let was_grounded = self.grounded;
        self.grounded = collision.check_collision(feet_aabb);

        // Apply gravity if not grounded
        if !self.grounded {
            self.velocity.y += self.config.gravity * dt;
            if self.velocity.y > self.config.terminal_velocity {
                self.velocity.y = self.config.terminal_velocity;
            }
        }

        // Check water
        let _center_aabb = AABB::from_center(self.position, half_width * 0.5, half_height * 0.5);
        let center_x = (self.position.x / 1.0).floor() as i32;
        let center_y = (self.position.y / 1.0).floor() as i32;
        self.in_water = collision.is_liquid(center_x, center_y);

        // Check climbable
        self.on_climbable = collision.is_climbable(center_x, center_y);

        // Start coyote time when leaving ground
        if was_grounded && !self.grounded && !self.state.is_airborne() {
            self.coyote_time = Self::COYOTE_TIME;
        }

        // Resolve collisions
        let future_pos = self.position + self.velocity.scale(dt);
        let future_aabb = AABB::from_center(future_pos, half_width, half_height);

        if collision.check_collision(future_aabb) {
            // Try horizontal only
            let horiz_pos = Vec2::new(future_pos.x, self.position.y);
            let horiz_aabb = AABB::from_center(horiz_pos, half_width, half_height);
            if !collision.check_collision(horiz_aabb) {
                self.position.x = future_pos.x;
                self.velocity.y = 0.0;
            }

            // Try vertical only
            let vert_pos = Vec2::new(self.position.x, future_pos.y);
            let vert_aabb = AABB::from_center(vert_pos, half_width, half_height);
            if collision.check_collision(vert_aabb) {
                // Hit something vertically
                if self.velocity.y > 0.0 {
                    // Hit ground
                    self.grounded = true;
                }
                self.velocity.y = 0.0;
            } else {
                self.position.y = future_pos.y;
            }
        } else {
            self.position = future_pos;
        }
    }

    /// Get current animation state for rendering.
    ///
    /// Returns the appropriate animation state based on current player state.
    #[must_use]
    pub fn animation_state(&self) -> PlayerAnimationState {
        // For now, assume not digging - digging state would come from interaction system
        PlayerAnimationState::from_player_state(self.state, false)
    }

    /// Get current animation state with digging flag.
    #[must_use]
    pub fn animation_state_with_digging(&self, is_digging: bool) -> PlayerAnimationState {
        PlayerAnimationState::from_player_state(self.state, is_digging)
    }
}

/// Collision check result (placeholder for world collision).
#[derive(Debug, Clone, Copy, Default)]
pub struct CollisionResult {
    /// Push-out vector to resolve collision
    pub push_out: Vec2,
    /// Whether the player hit the ground
    pub hit_ground: bool,
    /// Whether the player hit the ceiling
    pub hit_ceiling: bool,
    /// Whether the player is in water
    pub in_water: bool,
    /// Whether the player is on a climbable surface
    pub on_climbable: bool,
}

/// Placeholder function for collision detection.
/// In a real implementation, this would check against the world.
#[must_use]
pub fn check_collision(_player: &Player, _position: Vec2, _velocity: Vec2) -> CollisionResult {
    // Placeholder - always return no collision
    CollisionResult {
        push_out: Vec2::ZERO,
        hit_ground: true, // Assume ground for testing
        hit_ceiling: false,
        in_water: false,
        on_climbable: false,
    }
}

/// Cell solidity check (placeholder).
/// Returns true if the cell at the given world position is solid.
#[must_use]
pub fn is_cell_solid(_world_x: i64, _world_y: i64) -> bool {
    // Placeholder - in real implementation, this would check the world
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new(Vec2::new(100.0, 200.0));
        assert_eq!(player.position().x, 100.0);
        assert_eq!(player.position().y, 200.0);
        assert_eq!(player.state(), PlayerState::Idle);
        assert_eq!(player.facing(), Direction::Down);
        assert!(player.entity_id().is_valid());
    }

    #[test]
    fn test_player_with_config() {
        let config = PlayerConfig {
            walk_speed: 150.0,
            ..Default::default()
        };
        let player = Player::with_config(Vec2::ZERO, config);
        assert_eq!(player.config().walk_speed, 150.0);
    }

    #[test]
    fn test_direction_from_vec2() {
        assert_eq!(
            Direction::from_vec2(Vec2::new(1.0, 0.0)),
            Some(Direction::Right)
        );
        assert_eq!(
            Direction::from_vec2(Vec2::new(-1.0, 0.0)),
            Some(Direction::Left)
        );
        assert_eq!(
            Direction::from_vec2(Vec2::new(0.0, 1.0)),
            Some(Direction::Down)
        );
        assert_eq!(
            Direction::from_vec2(Vec2::new(0.0, -1.0)),
            Some(Direction::Up)
        );
        assert_eq!(Direction::from_vec2(Vec2::ZERO), None);

        // Diagonal - should favor larger axis
        assert_eq!(
            Direction::from_vec2(Vec2::new(1.0, 0.5)),
            Some(Direction::Right)
        );
    }

    #[test]
    fn test_direction_to_vec2() {
        assert_eq!(Direction::Up.to_vec2(), Vec2::UP);
        assert_eq!(Direction::Down.to_vec2(), Vec2::DOWN);
        assert_eq!(Direction::Left.to_vec2(), Vec2::LEFT);
        assert_eq!(Direction::Right.to_vec2(), Vec2::RIGHT);
    }

    #[test]
    fn test_player_state_checks() {
        assert!(PlayerState::Idle.is_grounded());
        assert!(PlayerState::Walking.is_grounded());
        assert!(PlayerState::Running.is_grounded());
        assert!(!PlayerState::Jumping.is_grounded());
        assert!(!PlayerState::Falling.is_grounded());

        assert!(PlayerState::Jumping.is_airborne());
        assert!(PlayerState::Falling.is_airborne());
        assert!(!PlayerState::Idle.is_airborne());

        assert!(PlayerState::Swimming.is_swimming());
        assert!(PlayerState::Climbing.is_climbing());
    }

    #[test]
    fn test_player_walking() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(true);

        let mut input = Input::new();
        input.movement = Vec2::new(1.0, 0.0);

        player.update(&input, 0.1);

        assert!(player.position().x > 0.0);
        assert_eq!(player.state(), PlayerState::Walking);
        assert_eq!(player.facing(), Direction::Right);
    }

    #[test]
    fn test_player_running() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(true);

        let mut input = Input::new();
        input.movement = Vec2::new(1.0, 0.0);
        input.running = true;

        player.update(&input, 0.1);

        assert!(player.position().x > 0.0);
        assert_eq!(player.state(), PlayerState::Running);
    }

    #[test]
    fn test_player_jump() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(true);

        let mut input = Input::new();
        input.jump_just_pressed = true;

        player.update(&input, 0.016);

        assert_eq!(player.state(), PlayerState::Jumping);
        assert!(player.velocity().y < 0.0); // Moving up (negative Y)
        assert!(!player.is_grounded());
    }

    #[test]
    fn test_player_falling() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(false);
        player.velocity = Vec2::new(0.0, 10.0); // Already falling

        let input = Input::new();
        player.update(&input, 0.1);

        assert_eq!(player.state(), PlayerState::Falling);
    }

    #[test]
    fn test_player_gravity() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(false);
        let initial_velocity_y = player.velocity().y;

        let input = Input::new();
        player.update(&input, 0.1);

        // Gravity should increase downward velocity
        assert!(player.velocity().y > initial_velocity_y);
    }

    #[test]
    fn test_player_swimming() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_in_water(true);

        let input = Input::new();
        player.update(&input, 0.1);

        assert_eq!(player.state(), PlayerState::Swimming);
    }

    #[test]
    fn test_player_climbing() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_on_climbable(true);

        let mut input = Input::new();
        input.movement = Vec2::new(0.0, -1.0); // Climbing up

        player.update(&input, 0.1);

        assert_eq!(player.state(), PlayerState::Climbing);
    }

    #[test]
    fn test_player_teleport() {
        let mut player = Player::new(Vec2::ZERO);
        player.teleport(Vec2::new(500.0, 500.0));

        assert_eq!(player.position(), Vec2::new(500.0, 500.0));
        assert_eq!(player.velocity(), Vec2::ZERO);
        assert_eq!(player.state(), PlayerState::Falling);
    }

    #[test]
    fn test_player_stop() {
        let mut player = Player::new(Vec2::ZERO);
        player.velocity = Vec2::new(100.0, 50.0);
        player.set_grounded(true);

        player.stop();

        assert_eq!(player.velocity(), Vec2::ZERO);
        assert_eq!(player.state(), PlayerState::Idle);
    }

    #[test]
    fn test_player_in_range() {
        let player = Player::new(Vec2::ZERO);

        assert!(player.in_range(Vec2::new(30.0, 30.0)));
        assert!(!player.in_range(Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn test_player_collision_response() {
        let mut player = Player::new(Vec2::new(0.0, 100.0));
        player.velocity = Vec2::new(0.0, 50.0);
        player.set_grounded(false);

        player.apply_collision(Vec2::new(0.0, -10.0), true, false);

        assert_eq!(player.position().y, 90.0);
        assert_eq!(player.velocity().y, 0.0);
        assert!(player.is_grounded());
    }

    #[test]
    fn test_player_ceiling_collision() {
        let mut player = Player::new(Vec2::new(0.0, 100.0));
        player.velocity = Vec2::new(0.0, -100.0); // Moving up
        player.set_grounded(false);

        player.apply_collision(Vec2::new(0.0, 10.0), false, true);

        assert_eq!(player.velocity().y, 0.0); // Stopped by ceiling
    }

    #[test]
    fn test_coyote_time() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(true);

        // Leave the ground
        player.set_grounded(false);

        // Within coyote time, should still be able to jump
        let mut input = Input::new();
        input.jump_just_pressed = true;
        player.update(&input, 0.01); // Small dt, within coyote time

        // Should have jumped
        assert_eq!(player.state(), PlayerState::Jumping);
    }

    #[test]
    fn test_jump_buffer() {
        // Create player and manually set up state to be airborne without triggering coyote time
        let mut player = Player::new(Vec2::ZERO);
        player.state = PlayerState::Falling;
        player.grounded = false;
        player.coyote_time = 0.0; // No coyote time

        // Press jump while in air
        let mut input = Input::new();
        input.jump_just_pressed = true;
        player.update(&input, 0.01);

        // Should still be falling, not jumping (not grounded, no coyote time)
        assert!(player.state().is_airborne());
        assert!(!player.is_grounded());

        // Land within buffer time (use small dt to stay within buffer)
        input.jump_just_pressed = false;
        player.set_grounded(true);
        player.update(&input, 0.01);

        // Should have jumped from the buffer
        assert_eq!(player.state(), PlayerState::Jumping);
    }

    #[test]
    fn test_air_control() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(false);
        player.velocity = Vec2::new(0.0, 10.0);

        let mut input = Input::new();
        input.movement = Vec2::new(1.0, 0.0);

        // Air control should give some horizontal influence
        player.update(&input, 0.1);

        assert!(player.velocity().x > 0.0);
        assert!(player.velocity().x < player.config().walk_speed); // Limited by air control
    }

    #[test]
    fn test_terminal_velocity() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_grounded(false);
        player.velocity = Vec2::new(0.0, player.config().terminal_velocity + 100.0);

        let input = Input::new();
        player.update(&input, 0.1);

        // Should be clamped to terminal velocity
        assert!(player.velocity().y <= player.config().terminal_velocity);
    }

    #[test]
    fn test_set_position() {
        let mut player = Player::new(Vec2::ZERO);
        player.set_position(Vec2::new(42.0, 84.0));
        assert_eq!(player.position(), Vec2::new(42.0, 84.0));
    }

    #[test]
    fn test_config_mut() {
        let mut player = Player::new(Vec2::ZERO);
        player.config_mut().walk_speed = 250.0;
        assert_eq!(player.config().walk_speed, 250.0);
    }
}
