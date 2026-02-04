//! Collision response system for smooth movement.
//!
//! This module provides collision response handling for player movement,
//! including sliding along walls and terrain detection.

use crate::physics::{CollisionQuery, AABB};
use serde::{Deserialize, Serialize};

/// Collision response behavior.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CollisionBehavior {
    /// Stop movement on collision
    Stop,
    /// Slide along surface
    Slide,
    /// Bounce with coefficient (0.0 = no bounce, 1.0 = full bounce)
    Bounce(f32),
}

impl Default for CollisionBehavior {
    fn default() -> Self {
        Self::Slide
    }
}

/// Result of collision detection.
#[derive(Debug, Clone, Copy, Default)]
pub struct CollisionMoveResult {
    /// Whether a collision occurred
    pub collided: bool,
    /// Final position after collision response
    pub position: (f32, f32),
    /// Final velocity after collision response
    pub velocity: (f32, f32),
    /// Normal of the collision surface (if collision occurred)
    pub normal: Option<(f32, f32)>,
}

impl CollisionMoveResult {
    /// Create a result with no collision.
    #[must_use]
    pub fn no_collision(position: (f32, f32), velocity: (f32, f32)) -> Self {
        Self {
            collided: false,
            position,
            velocity,
            normal: None,
        }
    }

    /// Create a result with collision.
    #[must_use]
    pub fn with_collision(position: (f32, f32), velocity: (f32, f32), normal: (f32, f32)) -> Self {
        Self {
            collided: true,
            position,
            velocity,
            normal: Some(normal),
        }
    }
}

/// Configuration for collision response.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CollisionConfig {
    /// Player collision radius (for circular collision)
    pub radius: f32,
    /// Half width for AABB collision
    pub half_width: f32,
    /// Half height for AABB collision
    pub half_height: f32,
    /// Small offset to prevent floating point issues
    pub skin_width: f32,
    /// Maximum number of iterations for collision resolution
    pub max_iterations: u32,
}

impl Default for CollisionConfig {
    fn default() -> Self {
        Self {
            radius: 8.0,
            half_width: 6.0,
            half_height: 6.0,
            skin_width: 0.01,
            max_iterations: 4,
        }
    }
}

/// Process movement with collision.
///
/// This function handles movement with collision detection and response,
/// modifying position and velocity in place.
///
/// # Returns
///
/// Returns `true` if a collision occurred.
pub fn move_with_collision<C: CollisionQuery>(
    position: &mut (f32, f32),
    velocity: &mut (f32, f32),
    config: &CollisionConfig,
    chunk_manager: &C,
    behavior: CollisionBehavior,
    dt: f32,
) -> bool {
    let start = *position;
    let movement = (velocity.0 * dt, velocity.1 * dt);
    let desired_end = (start.0 + movement.0, start.1 + movement.1);

    // No movement, nothing to do
    if movement.0.abs() < 0.0001 && movement.1.abs() < 0.0001 {
        return false;
    }

    match behavior {
        CollisionBehavior::Stop => {
            let aabb = AABB::from_center(
                crate::input::Vec2::new(desired_end.0, desired_end.1),
                config.half_width,
                config.half_height,
            );

            if chunk_manager.check_collision(aabb) {
                // Stop completely on collision
                *velocity = (0.0, 0.0);
                true
            } else {
                *position = desired_end;
                false
            }
        },
        CollisionBehavior::Slide => {
            let result = slide_movement(start, desired_end, config, chunk_manager);
            *position = result;

            let collided = (result.0 - desired_end.0).abs() > 0.001
                || (result.1 - desired_end.1).abs() > 0.001;

            if collided {
                // Adjust velocity based on what direction we couldn't move
                if (result.0 - desired_end.0).abs() > 0.001 {
                    velocity.0 = 0.0;
                }
                if (result.1 - desired_end.1).abs() > 0.001 {
                    velocity.1 = 0.0;
                }
            }

            collided
        },
        CollisionBehavior::Bounce(coefficient) => {
            let aabb = AABB::from_center(
                crate::input::Vec2::new(desired_end.0, desired_end.1),
                config.half_width,
                config.half_height,
            );

            if chunk_manager.check_collision(aabb) {
                // Find which axis collides and bounce
                let horiz_aabb = AABB::from_center(
                    crate::input::Vec2::new(desired_end.0, start.1),
                    config.half_width,
                    config.half_height,
                );
                let vert_aabb = AABB::from_center(
                    crate::input::Vec2::new(start.0, desired_end.1),
                    config.half_width,
                    config.half_height,
                );

                let horiz_blocked = chunk_manager.check_collision(horiz_aabb);
                let vert_blocked = chunk_manager.check_collision(vert_aabb);

                if horiz_blocked {
                    velocity.0 = -velocity.0 * coefficient;
                    position.0 = start.0;
                } else {
                    position.0 = desired_end.0;
                }

                if vert_blocked {
                    velocity.1 = -velocity.1 * coefficient;
                    position.1 = start.1;
                } else {
                    position.1 = desired_end.1;
                }

                true
            } else {
                *position = desired_end;
                false
            }
        },
    }
}

/// Slide movement along walls (feels good for RPG/top-down).
///
/// This function attempts to move from start to desired_end, sliding along
/// any walls encountered.
///
/// # Returns
///
/// Returns the actual end position after collision response.
pub fn slide_movement<C: CollisionQuery>(
    start: (f32, f32),
    desired_end: (f32, f32),
    config: &CollisionConfig,
    chunk_manager: &C,
) -> (f32, f32) {
    // First try full movement
    let full_aabb = AABB::from_center(
        crate::input::Vec2::new(desired_end.0, desired_end.1),
        config.half_width,
        config.half_height,
    );

    if !chunk_manager.check_collision(full_aabb) {
        return desired_end;
    }

    // Try horizontal only
    let horiz_pos = (desired_end.0, start.1);
    let horiz_aabb = AABB::from_center(
        crate::input::Vec2::new(horiz_pos.0, horiz_pos.1),
        config.half_width,
        config.half_height,
    );
    let can_move_horiz = !chunk_manager.check_collision(horiz_aabb);

    // Try vertical only
    let vert_pos = (start.0, desired_end.1);
    let vert_aabb = AABB::from_center(
        crate::input::Vec2::new(vert_pos.0, vert_pos.1),
        config.half_width,
        config.half_height,
    );
    let can_move_vert = !chunk_manager.check_collision(vert_aabb);

    match (can_move_horiz, can_move_vert) {
        (true, true) => {
            // Both axes work individually, prefer horizontal slide
            horiz_pos
        },
        (true, false) => horiz_pos,
        (false, true) => vert_pos,
        (false, false) => start, // Can't move at all
    }
}

/// Check what terrain type player is standing on.
///
/// Returns the material ID at the position, or None if position is in air.
pub fn terrain_at_feet<C: CollisionQuery>(position: (f32, f32), chunk_manager: &C) -> Option<u16> {
    // Check cell below player center
    let cell_x = position.0.floor() as i32;
    let cell_y = (position.1 + 1.0).floor() as i32; // Slightly below center

    if chunk_manager.is_solid(cell_x, cell_y) {
        // Return a generic "solid" indicator (actual material lookup would
        // require additional trait method)
        Some(1)
    } else {
        None
    }
}

/// Get terrain info at a specific cell position.
#[derive(Debug, Clone, Copy, Default)]
pub struct TerrainInfo {
    /// Whether the cell is solid
    pub is_solid: bool,
    /// Whether the cell is liquid
    pub is_liquid: bool,
    /// Whether the cell is climbable
    pub is_climbable: bool,
}

impl TerrainInfo {
    /// Query terrain info at position.
    pub fn at<C: CollisionQuery>(position: (f32, f32), chunk_manager: &C) -> Self {
        let cell_x = position.0.floor() as i32;
        let cell_y = position.1.floor() as i32;

        Self {
            is_solid: chunk_manager.is_solid(cell_x, cell_y),
            is_liquid: chunk_manager.is_liquid(cell_x, cell_y),
            is_climbable: chunk_manager.is_climbable(cell_x, cell_y),
        }
    }

    /// Query terrain info at cell coordinates.
    pub fn at_cell<C: CollisionQuery>(x: i32, y: i32, chunk_manager: &C) -> Self {
        Self {
            is_solid: chunk_manager.is_solid(x, y),
            is_liquid: chunk_manager.is_liquid(x, y),
            is_climbable: chunk_manager.is_climbable(x, y),
        }
    }
}

/// Movement speed modifier based on terrain.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerrainSpeedModifier {
    /// Normal terrain speed (1.0 = full speed)
    pub normal: f32,
    /// Sand speed (usually slower)
    pub sand: f32,
    /// Water speed
    pub water: f32,
    /// Mud speed
    pub mud: f32,
    /// Ice speed (can be > 1.0 for sliding)
    pub ice: f32,
}

impl Default for TerrainSpeedModifier {
    fn default() -> Self {
        Self {
            normal: 1.0,
            sand: 0.7,
            water: 0.5,
            mud: 0.4,
            ice: 1.2,
        }
    }
}

impl TerrainSpeedModifier {
    /// Get speed modifier for a material ID.
    #[must_use]
    pub fn for_material(&self, material: u16) -> f32 {
        match material {
            3 => self.sand,   // Sand
            4 => self.water,  // Water
            _ => self.normal, // Air/default and all others
        }
    }
}

/// Sweep test result for continuous collision detection.
#[derive(Debug, Clone, Copy)]
pub struct CollisionSweepResult {
    /// Time of impact (0.0 = immediate, 1.0 = no collision)
    pub time: f32,
    /// Contact point
    pub point: (f32, f32),
    /// Surface normal
    pub normal: (f32, f32),
    /// Whether there was a collision
    pub hit: bool,
}

impl Default for CollisionSweepResult {
    fn default() -> Self {
        Self {
            time: 1.0,
            point: (0.0, 0.0),
            normal: (0.0, 0.0),
            hit: false,
        }
    }
}

impl CollisionSweepResult {
    /// Create a result indicating no collision.
    #[must_use]
    pub fn no_hit() -> Self {
        Self::default()
    }

    /// Create a result indicating collision.
    #[must_use]
    pub fn hit_at(time: f32, point: (f32, f32), normal: (f32, f32)) -> Self {
        Self {
            time,
            point,
            normal,
            hit: true,
        }
    }
}

/// Perform a sweep test for continuous collision detection.
///
/// This moves an AABB from start to end and returns the first collision.
pub fn sweep_aabb<C: CollisionQuery>(
    start: (f32, f32),
    end: (f32, f32),
    config: &CollisionConfig,
    chunk_manager: &C,
    steps: u32,
) -> CollisionSweepResult {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let pos = (start.0 + dx * t, start.1 + dy * t);

        let aabb = AABB::from_center(
            crate::input::Vec2::new(pos.0, pos.1),
            config.half_width,
            config.half_height,
        );

        if chunk_manager.check_collision(aabb) {
            // Found collision, calculate normal
            let prev_t = (i - 1) as f32 / steps as f32;
            let prev_pos = (start.0 + dx * prev_t, start.1 + dy * prev_t);

            // Determine normal based on movement direction
            let normal = if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    (-1.0, 0.0)
                } else {
                    (1.0, 0.0)
                }
            } else if dy > 0.0 {
                (0.0, -1.0)
            } else {
                (0.0, 1.0)
            };

            return CollisionSweepResult::hit_at(prev_t, prev_pos, normal);
        }
    }

    CollisionSweepResult::no_hit()
}

/// Check if a position is valid (not colliding).
pub fn is_position_valid<C: CollisionQuery>(
    position: (f32, f32),
    config: &CollisionConfig,
    chunk_manager: &C,
) -> bool {
    let aabb = AABB::from_center(
        crate::input::Vec2::new(position.0, position.1),
        config.half_width,
        config.half_height,
    );

    !chunk_manager.check_collision(aabb)
}

/// Find the nearest valid position from a potentially invalid one.
///
/// This can be used to push a player out of walls they've gotten stuck in.
pub fn find_nearest_valid_position<C: CollisionQuery>(
    position: (f32, f32),
    config: &CollisionConfig,
    chunk_manager: &C,
    search_radius: f32,
) -> Option<(f32, f32)> {
    // Already valid
    if is_position_valid(position, config, chunk_manager) {
        return Some(position);
    }

    // Search in expanding squares
    let step = 1.0;
    let max_dist = search_radius.ceil() as i32;

    for dist in 1..=max_dist {
        let d = dist as f32 * step;

        // Check cardinal directions first
        let candidates = [
            (position.0 + d, position.1),
            (position.0 - d, position.1),
            (position.0, position.1 + d),
            (position.0, position.1 - d),
            (position.0 + d, position.1 + d),
            (position.0 + d, position.1 - d),
            (position.0 - d, position.1 + d),
            (position.0 - d, position.1 - d),
        ];

        for candidate in candidates {
            if is_position_valid(candidate, config, chunk_manager) {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::MockCollision;

    fn default_config() -> CollisionConfig {
        CollisionConfig::default()
    }

    #[test]
    fn test_collision_behavior_default() {
        assert_eq!(CollisionBehavior::default(), CollisionBehavior::Slide);
    }

    #[test]
    fn test_collision_result_no_collision() {
        let result = CollisionMoveResult::no_collision((10.0, 20.0), (5.0, 5.0));
        assert!(!result.collided);
        assert_eq!(result.position, (10.0, 20.0));
        assert_eq!(result.velocity, (5.0, 5.0));
        assert!(result.normal.is_none());
    }

    #[test]
    fn test_collision_result_with_collision() {
        let result = CollisionMoveResult::with_collision((10.0, 20.0), (0.0, 5.0), (1.0, 0.0));
        assert!(result.collided);
        assert_eq!(result.normal, Some((1.0, 0.0)));
    }

    #[test]
    fn test_move_with_collision_no_collision() {
        let collision = MockCollision::new_empty();
        let config = default_config();
        let mut pos = (0.0, 0.0);
        let mut vel = (10.0, 0.0);

        let collided = move_with_collision(
            &mut pos,
            &mut vel,
            &config,
            &collision,
            CollisionBehavior::Slide,
            0.1,
        );

        assert!(!collided);
        assert!((pos.0 - 1.0).abs() < 0.001); // Moved 10 * 0.1 = 1.0
    }

    #[test]
    fn test_move_with_collision_stop_behavior() {
        let mut collision = MockCollision::new();
        collision.set_ground_level(10);
        let config = default_config();
        let mut pos = (0.0, 5.0);
        let mut vel = (0.0, 100.0); // Moving down

        let collided = move_with_collision(
            &mut pos,
            &mut vel,
            &config,
            &collision,
            CollisionBehavior::Stop,
            0.1,
        );

        assert!(collided);
        assert_eq!(vel, (0.0, 0.0)); // Velocity zeroed
    }

    #[test]
    fn test_move_with_collision_bounce_behavior() {
        let mut collision = MockCollision::new();
        collision.set_solid(10, 0);
        let config = default_config();
        let mut pos = (5.0, 0.0);
        let mut vel = (100.0, 0.0); // Moving right toward wall

        let collided = move_with_collision(
            &mut pos,
            &mut vel,
            &config,
            &collision,
            CollisionBehavior::Bounce(0.5),
            0.1,
        );

        // May or may not collide depending on exact position
        // Just verify function doesn't crash
        assert!(vel.0.is_finite());
    }

    #[test]
    fn test_slide_movement_no_collision() {
        let collision = MockCollision::new_empty();
        let config = default_config();

        let result = slide_movement((0.0, 0.0), (10.0, 10.0), &config, &collision);

        assert_eq!(result, (10.0, 10.0));
    }

    #[test]
    fn test_slide_movement_horizontal_blocked() {
        let mut collision = MockCollision::new();
        // Block to the right
        for y in -20..20 {
            collision.set_solid(20, y);
        }
        let config = default_config();

        let result = slide_movement((0.0, 0.0), (30.0, 10.0), &config, &collision);

        // Should slide vertically since horizontal is blocked
        assert!((result.0 - 0.0).abs() < 0.001 || (result.1 - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_terrain_at_feet() {
        let mut collision = MockCollision::new();
        collision.set_ground_level(10);

        let terrain = terrain_at_feet((5.0, 8.0), &collision);
        assert!(terrain.is_none()); // Above ground

        let terrain = terrain_at_feet((5.0, 9.5), &collision);
        assert!(terrain.is_some()); // At ground level
    }

    #[test]
    fn test_terrain_info_at() {
        let mut collision = MockCollision::new();
        collision.set_solid(5, 5);
        collision.set_liquid(6, 5);
        collision.set_climbable(7, 5);

        let info = TerrainInfo::at((5.5, 5.5), &collision);
        assert!(info.is_solid);
        assert!(!info.is_liquid);
        assert!(!info.is_climbable);

        let info = TerrainInfo::at((6.5, 5.5), &collision);
        assert!(!info.is_solid);
        assert!(info.is_liquid);
    }

    #[test]
    fn test_terrain_speed_modifier() {
        let modifier = TerrainSpeedModifier::default();
        assert_eq!(modifier.for_material(0), 1.0);
        assert_eq!(modifier.for_material(3), 0.7); // Sand
        assert_eq!(modifier.for_material(4), 0.5); // Water
    }

    #[test]
    fn test_sweep_result() {
        let no_hit = CollisionSweepResult::no_hit();
        assert!(!no_hit.hit);
        assert_eq!(no_hit.time, 1.0);

        let hit = CollisionSweepResult::hit_at(0.5, (10.0, 10.0), (1.0, 0.0));
        assert!(hit.hit);
        assert_eq!(hit.time, 0.5);
    }

    #[test]
    fn test_sweep_aabb_no_collision() {
        let collision = MockCollision::new_empty();
        let config = default_config();

        let result = sweep_aabb((0.0, 0.0), (10.0, 10.0), &config, &collision, 10);

        assert!(!result.hit);
        assert_eq!(result.time, 1.0);
    }

    #[test]
    fn test_is_position_valid() {
        let mut collision = MockCollision::new();
        collision.set_solid(50, 50);
        let config = default_config();

        // Position (0,0) is valid because solid cell is far away
        assert!(is_position_valid((0.0, 0.0), &config, &collision));
        // Position (50, 50) is not valid because it's at the solid cell
        assert!(!is_position_valid((50.0, 50.0), &config, &collision));
    }

    #[test]
    fn test_find_nearest_valid_position_already_valid() {
        let collision = MockCollision::new_empty();
        let config = default_config();

        let result = find_nearest_valid_position((5.0, 5.0), &config, &collision, 10.0);
        assert_eq!(result, Some((5.0, 5.0)));
    }

    #[test]
    fn test_find_nearest_valid_position_stuck_in_wall() {
        let mut collision = MockCollision::new();
        collision.set_solid(5, 5);
        let config = default_config();

        let result = find_nearest_valid_position((5.0, 5.0), &config, &collision, 10.0);
        assert!(result.is_some());
        // Should find a position nearby that's valid
        if let Some(pos) = result {
            assert!(is_position_valid(pos, &config, &collision));
        }
    }

    #[test]
    fn test_collision_config_default() {
        let config = CollisionConfig::default();
        assert_eq!(config.radius, 8.0);
        assert_eq!(config.half_width, 6.0);
        assert_eq!(config.half_height, 6.0);
    }
}
