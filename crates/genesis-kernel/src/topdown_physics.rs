//! Top-down friction-based physics model.
//!
//! This module provides physics for top-down movement without gravity,
//! using friction to create smooth acceleration and deceleration.

use crate::biome::material_ids;

/// Physics configuration for top-down movement.
#[derive(Debug, Clone)]
pub struct TopDownPhysicsConfig {
    /// Ground friction coefficient (0.9 = slippery ice, 0.5 = grippy).
    pub friction: f32,
    /// Friction multiplier when in water.
    pub water_friction: f32,
    /// Friction multiplier when on sand.
    pub sand_friction: f32,
    /// Friction multiplier when on grass.
    pub grass_friction: f32,
    /// Friction multiplier when on stone.
    pub stone_friction: f32,
    /// How fast to reach target velocity (units/s²).
    pub acceleration: f32,
    /// Maximum movement speed (units/s).
    pub max_speed: f32,
}

impl Default for TopDownPhysicsConfig {
    fn default() -> Self {
        Self {
            friction: 0.85,
            water_friction: 0.95,
            sand_friction: 0.70,
            grass_friction: 0.80,
            stone_friction: 0.85,
            acceleration: 800.0,
            max_speed: 200.0,
        }
    }
}

impl TopDownPhysicsConfig {
    /// Creates a slippery ice-like physics config.
    #[must_use]
    pub fn ice() -> Self {
        Self {
            friction: 0.98,
            water_friction: 0.95,
            sand_friction: 0.90,
            grass_friction: 0.95,
            stone_friction: 0.98,
            acceleration: 200.0,
            max_speed: 250.0,
        }
    }

    /// Creates a heavy/sluggish physics config.
    #[must_use]
    pub fn heavy() -> Self {
        Self {
            friction: 0.70,
            water_friction: 0.85,
            sand_friction: 0.50,
            grass_friction: 0.65,
            stone_friction: 0.70,
            acceleration: 400.0,
            max_speed: 120.0,
        }
    }

    /// Creates a nimble/responsive physics config.
    #[must_use]
    pub fn nimble() -> Self {
        Self {
            friction: 0.75,
            water_friction: 0.88,
            sand_friction: 0.60,
            grass_friction: 0.72,
            stone_friction: 0.75,
            acceleration: 1200.0,
            max_speed: 180.0,
        }
    }
}

/// Get friction multiplier for a terrain material type.
#[must_use]
pub fn terrain_friction(material: u16, config: &TopDownPhysicsConfig) -> f32 {
    match material {
        m if m == material_ids::WATER => config.water_friction,
        m if m == material_ids::SAND => config.sand_friction,
        m if m == material_ids::GRASS => config.grass_friction,
        m if m == material_ids::STONE => config.stone_friction,
        m if m == material_ids::DIRT => config.grass_friction, // Similar to grass
        m if m == material_ids::LAVA => config.water_friction, // Slow in lava too
        _ => config.friction,                                  // Default friction
    }
}

/// Apply top-down physics to velocity.
///
/// This creates smooth, responsive movement:
/// - Accelerate towards input direction when pressing movement
/// - Gradually slow down when releasing (friction-based)
/// - Different terrain affects movement speed
///
/// # Arguments
/// * `velocity` - Current velocity (modified in place)
/// * `input_direction` - Input direction (-1 to 1 for each axis)
/// * `terrain_type` - Material ID of terrain below player
/// * `config` - Physics configuration
/// * `dt` - Delta time in seconds
pub fn apply_topdown_physics(
    velocity: &mut (f32, f32),
    input_direction: (f32, f32),
    terrain_type: u16,
    config: &TopDownPhysicsConfig,
    dt: f32,
) {
    // Normalize input if needed
    let input_len =
        (input_direction.0 * input_direction.0 + input_direction.1 * input_direction.1).sqrt();
    let (input_x, input_y) = if input_len > 1.0 {
        (input_direction.0 / input_len, input_direction.1 / input_len)
    } else {
        input_direction
    };

    // Get terrain friction
    let friction = terrain_friction(terrain_type, config);

    // Target velocity based on input
    let target_vx = input_x * config.max_speed;
    let target_vy = input_y * config.max_speed;

    // Check if player is providing input
    let has_input = input_len > 0.01;

    if has_input {
        // Accelerate towards target velocity
        let accel = config.acceleration * dt;

        // Move velocity towards target
        let diff_x = target_vx - velocity.0;
        let diff_y = target_vy - velocity.1;

        if diff_x.abs() < accel {
            velocity.0 = target_vx;
        } else {
            velocity.0 += accel * diff_x.signum();
        }

        if diff_y.abs() < accel {
            velocity.1 = target_vy;
        } else {
            velocity.1 += accel * diff_y.signum();
        }
    } else {
        // No input - apply friction to slow down
        velocity.0 *= friction.powf(dt * 60.0); // Normalize to 60fps equivalent
        velocity.1 *= friction.powf(dt * 60.0);

        // Stop completely when very slow
        if velocity.0.abs() < 0.1 {
            velocity.0 = 0.0;
        }
        if velocity.1.abs() < 0.1 {
            velocity.1 = 0.0;
        }
    }

    // Apply terrain-based speed modifier
    let terrain_speed_mult = terrain_speed_multiplier(terrain_type);
    let current_speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
    let max_terrain_speed = config.max_speed * terrain_speed_mult;

    // Clamp to terrain-adjusted max speed
    if current_speed > max_terrain_speed {
        let scale = max_terrain_speed / current_speed;
        velocity.0 *= scale;
        velocity.1 *= scale;
    }
}

/// Get speed multiplier for terrain type.
///
/// Returns a value 0.0-1.0 that scales maximum speed.
#[must_use]
pub fn terrain_speed_multiplier(material: u16) -> f32 {
    match material {
        m if m == material_ids::WATER => 0.5,  // Half speed in water
        m if m == material_ids::SAND => 0.7,   // Slower on sand
        m if m == material_ids::GRASS => 0.95, // Slightly slower on grass
        m if m == material_ids::STONE => 1.0,  // Full speed on stone
        m if m == material_ids::DIRT => 0.9,   // Slightly slower on dirt
        m if m == material_ids::LAVA => 0.3,   // Very slow in lava
        m if m == material_ids::AIR => 1.0,    // Normal speed (flying?)
        _ => 1.0,
    }
}

/// Calculate movement delta for this frame.
///
/// Convenience function that returns how much to move this frame.
#[must_use]
pub fn calculate_movement(velocity: (f32, f32), dt: f32) -> (f32, f32) {
    (velocity.0 * dt, velocity.1 * dt)
}

/// Apply velocity to position.
#[must_use]
pub fn apply_velocity(position: (f32, f32), velocity: (f32, f32), dt: f32) -> (f32, f32) {
    (position.0 + velocity.0 * dt, position.1 + velocity.1 * dt)
}

/// Simple state struct for tracking an entity's physics.
#[derive(Debug, Clone, Default)]
pub struct TopDownPhysicsState {
    /// Current position.
    pub position: (f32, f32),
    /// Current velocity.
    pub velocity: (f32, f32),
    /// Input direction (set each frame by player input).
    pub input: (f32, f32),
}

impl TopDownPhysicsState {
    /// Creates a new physics state at the given position.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            velocity: (0.0, 0.0),
            input: (0.0, 0.0),
        }
    }

    /// Updates physics for one frame.
    pub fn update(&mut self, terrain_type: u16, config: &TopDownPhysicsConfig, dt: f32) {
        apply_topdown_physics(&mut self.velocity, self.input, terrain_type, config, dt);
        self.position = apply_velocity(self.position, self.velocity, dt);
    }

    /// Sets input direction from WASD-style input.
    pub fn set_input(&mut self, left: bool, right: bool, up: bool, down: bool) {
        self.input.0 = if right { 1.0 } else { 0.0 } + if left { -1.0 } else { 0.0 };
        self.input.1 = if down { 1.0 } else { 0.0 } + if up { -1.0 } else { 0.0 };
    }

    /// Returns the current speed.
    #[must_use]
    pub fn speed(&self) -> f32 {
        (self.velocity.0 * self.velocity.0 + self.velocity.1 * self.velocity.1).sqrt()
    }

    /// Returns true if moving.
    #[must_use]
    pub fn is_moving(&self) -> bool {
        self.speed() > 0.1
    }

    /// Returns the facing direction in radians.
    #[must_use]
    pub fn facing_angle(&self) -> f32 {
        self.velocity.1.atan2(self.velocity.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TopDownPhysicsConfig::default();
        assert!((config.friction - 0.85).abs() < f32::EPSILON);
        assert!((config.max_speed - 200.0).abs() < f32::EPSILON);
        assert!((config.acceleration - 800.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ice_config() {
        let config = TopDownPhysicsConfig::ice();
        assert!(config.friction > 0.95); // Very slippery
        assert!(config.acceleration < 300.0); // Slow to accelerate
    }

    #[test]
    fn test_terrain_friction() {
        let config = TopDownPhysicsConfig::default();

        let water = terrain_friction(material_ids::WATER, &config);
        let sand = terrain_friction(material_ids::SAND, &config);
        let stone = terrain_friction(material_ids::STONE, &config);

        // Water should be most slippery (highest friction = slowest stop)
        assert!(water > sand);
        assert!(water > stone);

        // Sand should be grippier than stone
        assert!(sand < stone);
    }

    #[test]
    fn test_terrain_speed_multiplier() {
        let water_speed = terrain_speed_multiplier(material_ids::WATER);
        let stone_speed = terrain_speed_multiplier(material_ids::STONE);
        let lava_speed = terrain_speed_multiplier(material_ids::LAVA);

        assert!(water_speed < stone_speed);
        assert!(lava_speed < water_speed); // Lava is slowest
        assert!((stone_speed - 1.0).abs() < f32::EPSILON); // Full speed on stone
    }

    #[test]
    fn test_apply_physics_acceleration() {
        let config = TopDownPhysicsConfig::default();
        let mut velocity = (0.0, 0.0);
        let input = (1.0, 0.0); // Moving right

        // Apply physics for a few frames
        for _ in 0..10 {
            apply_topdown_physics(
                &mut velocity,
                input,
                material_ids::STONE,
                &config,
                1.0 / 60.0,
            );
        }

        // Should have accelerated
        assert!(velocity.0 > 0.0);
        assert!(velocity.0 < config.max_speed); // Not yet at max
    }

    #[test]
    fn test_apply_physics_friction() {
        let config = TopDownPhysicsConfig::default();
        let mut velocity = (100.0, 0.0); // Already moving
        let input = (0.0, 0.0); // No input

        // Apply physics for a few frames
        for _ in 0..60 {
            apply_topdown_physics(
                &mut velocity,
                input,
                material_ids::STONE,
                &config,
                1.0 / 60.0,
            );
        }

        // Should have slowed down significantly
        assert!(velocity.0 < 50.0);
    }

    #[test]
    fn test_apply_physics_max_speed() {
        let config = TopDownPhysicsConfig::default();
        let mut velocity = (0.0, 0.0);
        let input = (1.0, 0.0);

        // Apply physics for many frames
        for _ in 0..1000 {
            apply_topdown_physics(
                &mut velocity,
                input,
                material_ids::STONE,
                &config,
                1.0 / 60.0,
            );
        }

        // Should be clamped at max speed
        let speed = velocity.0;
        assert!((speed - config.max_speed).abs() < 1.0);
    }

    #[test]
    fn test_apply_physics_terrain_slows() {
        let config = TopDownPhysicsConfig::default();
        let mut velocity_stone = (0.0, 0.0);
        let mut velocity_water = (0.0, 0.0);
        let input = (1.0, 0.0);

        // Apply physics for same duration on different terrain
        for _ in 0..100 {
            apply_topdown_physics(
                &mut velocity_stone,
                input,
                material_ids::STONE,
                &config,
                1.0 / 60.0,
            );
            apply_topdown_physics(
                &mut velocity_water,
                input,
                material_ids::WATER,
                &config,
                1.0 / 60.0,
            );
        }

        // Water should be slower
        assert!(velocity_water.0 < velocity_stone.0);
    }

    #[test]
    fn test_physics_state() {
        let mut state = TopDownPhysicsState::new(100.0, 100.0);
        assert_eq!(state.position, (100.0, 100.0));
        assert_eq!(state.velocity, (0.0, 0.0));
        assert!(!state.is_moving());

        // Set input and update
        state.set_input(false, true, false, false); // Moving right
        assert_eq!(state.input, (1.0, 0.0));

        let config = TopDownPhysicsConfig::default();
        state.update(material_ids::STONE, &config, 1.0 / 60.0);

        // Should have started moving
        assert!(state.velocity.0 > 0.0);
        assert!(state.position.0 > 100.0);
    }

    #[test]
    fn test_calculate_movement() {
        let velocity = (100.0, 50.0);
        let dt = 0.016; // ~60fps

        let movement = calculate_movement(velocity, dt);
        assert!((movement.0 - 1.6).abs() < 0.01);
        assert!((movement.1 - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_diagonal_normalized() {
        let config = TopDownPhysicsConfig::default();
        let mut velocity = (0.0, 0.0);
        let input = (1.0, 1.0); // Diagonal

        // Apply physics
        for _ in 0..1000 {
            apply_topdown_physics(
                &mut velocity,
                input,
                material_ids::STONE,
                &config,
                1.0 / 60.0,
            );
        }

        // Speed should not exceed max_speed even diagonally
        let speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
        assert!(speed <= config.max_speed + 1.0); // Small tolerance
    }

    #[test]
    fn test_facing_angle() {
        let mut state = TopDownPhysicsState::new(0.0, 0.0);

        // Moving right
        state.velocity = (1.0, 0.0);
        let angle = state.facing_angle();
        assert!(angle.abs() < 0.01); // Should be ~0 (facing right)

        // Moving up (negative Y in screen coords)
        state.velocity = (0.0, -1.0);
        let angle = state.facing_angle();
        assert!((angle + std::f32::consts::FRAC_PI_2).abs() < 0.01); // Should be -π/2
    }
}
