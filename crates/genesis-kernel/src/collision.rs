//! Collision query system for gameplay physics.
//!
//! This module provides collision queries against the cell buffer,
//! enabling gameplay systems to interact with the simulated world.
//!
//! ## Overview
//!
//! The collision system supports:
//! - Point queries (is this cell solid?)
//! - Raycasting (shoot a ray, find first solid hit)
//! - Box queries (find all solid cells in a region)
//! - Ground finding (find the first solid cell below a point)

use genesis_common::WorldCoord;

use crate::cell::{Cell, CellFlags};

/// Result of a raycast operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// World coordinate of the hit cell
    pub coord: WorldCoord,
    /// Distance from ray origin to hit
    pub distance: f32,
    /// Surface normal at hit point (approximate)
    pub normal: (f32, f32),
}

/// A 2D vector for ray operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
}

impl Vec2 {
    /// Creates a new Vec2.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the length of the vector.
    #[must_use]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns a normalized version of the vector.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > f32::EPSILON {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::new(0.0, 0.0)
        }
    }
}

/// Provides collision queries against cell data.
///
/// This struct holds a reference to cell data (either from GPU readback
/// or a CPU-side shadow buffer) and provides various query operations.
pub struct CollisionQuery<'a> {
    /// Cell data to query against
    cells: &'a [Cell],
    /// Chunk size (width and height in cells)
    chunk_size: u32,
    /// World offset of the chunk's origin
    chunk_origin: WorldCoord,
}

impl<'a> CollisionQuery<'a> {
    /// Creates a new collision query for a chunk.
    #[must_use]
    pub const fn new(cells: &'a [Cell], chunk_size: u32, chunk_origin: WorldCoord) -> Self {
        Self {
            cells,
            chunk_size,
            chunk_origin,
        }
    }

    /// Checks if a world coordinate is solid.
    #[must_use]
    pub fn is_solid(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_cell(coord) {
            cell.flags & CellFlags::SOLID != 0
        } else {
            false
        }
    }

    /// Checks if a world coordinate is liquid.
    #[must_use]
    pub fn is_liquid(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_cell(coord) {
            cell.flags & CellFlags::LIQUID != 0
        } else {
            false
        }
    }

    /// Checks if a world coordinate is empty (air).
    #[must_use]
    pub fn is_empty(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_cell(coord) {
            cell.material == 0 && cell.flags == 0
        } else {
            true
        }
    }

    /// Gets the cell at a world coordinate.
    #[must_use]
    pub fn get_cell(&self, coord: WorldCoord) -> Option<&Cell> {
        let local_x = coord.x - self.chunk_origin.x;
        let local_y = coord.y - self.chunk_origin.y;

        // Bounds check
        let size = self.chunk_size as i64;
        if local_x < 0 || local_y < 0 || local_x >= size || local_y >= size {
            return None;
        }

        let index = (local_y as usize) * (self.chunk_size as usize) + (local_x as usize);
        self.cells.get(index)
    }

    /// Performs a raycast from origin in direction, up to max_dist.
    ///
    /// Uses Bresenham's line algorithm for pixel-perfect traversal.
    ///
    /// # Arguments
    /// * `origin` - Ray starting point in world coordinates
    /// * `direction` - Normalized direction vector
    /// * `max_dist` - Maximum distance to check
    ///
    /// # Returns
    /// The first solid cell hit, or None if no hit within max_dist.
    #[must_use]
    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_dist: f32) -> Option<RayHit> {
        let dir = direction.normalize();
        if dir.length() < f32::EPSILON {
            return None;
        }

        // Use DDA (Digital Differential Analyzer) for ray marching
        let x = origin.x;
        let y = origin.y;
        let mut dist = 0.0;

        // Step size along each axis
        let step_x = if dir.x.abs() > f32::EPSILON {
            (1.0 / dir.x).abs()
        } else {
            f32::MAX
        };
        let step_y = if dir.y.abs() > f32::EPSILON {
            (1.0 / dir.y).abs()
        } else {
            f32::MAX
        };

        // Direction of stepping
        let sign_x = if dir.x >= 0.0 { 1i64 } else { -1i64 };
        let sign_y = if dir.y >= 0.0 { 1i64 } else { -1i64 };

        // Distance to next cell boundary
        let mut t_max_x = if dir.x.abs() > f32::EPSILON {
            let next_x = if dir.x >= 0.0 {
                (x.floor() + 1.0 - x) / dir.x
            } else {
                (x - x.floor()) / (-dir.x)
            };
            next_x.max(0.0)
        } else {
            f32::MAX
        };

        let mut t_max_y = if dir.y.abs() > f32::EPSILON {
            let next_y = if dir.y >= 0.0 {
                (y.floor() + 1.0 - y) / dir.y
            } else {
                (y - y.floor()) / (-dir.y)
            };
            next_y.max(0.0)
        } else {
            f32::MAX
        };

        // Current cell
        #[allow(clippy::cast_possible_truncation)]
        let mut cell_x = x.floor() as i64;
        #[allow(clippy::cast_possible_truncation)]
        let mut cell_y = y.floor() as i64;

        let mut prev_x = cell_x;

        while dist < max_dist {
            // Check current cell
            let coord = WorldCoord::new(cell_x, cell_y);
            if self.is_solid(coord) {
                // Calculate hit distance more accurately
                let hit_dist = dist;

                // Calculate normal based on entry direction
                let normal = if prev_x == cell_x {
                    (0.0, -sign_y as f32)
                } else {
                    (-sign_x as f32, 0.0)
                };

                return Some(RayHit {
                    coord,
                    distance: hit_dist,
                    normal,
                });
            }

            prev_x = cell_x;

            // Step to next cell
            if t_max_x < t_max_y {
                dist = t_max_x;
                t_max_x += step_x;
                cell_x += sign_x;
            } else {
                dist = t_max_y;
                t_max_y += step_y;
                cell_y += sign_y;
            }
        }

        None
    }

    /// Finds all solid cells within a box region.
    ///
    /// # Arguments
    /// * `min` - Minimum corner (inclusive)
    /// * `max` - Maximum corner (inclusive)
    ///
    /// # Returns
    /// Vector of world coordinates containing solid cells.
    #[must_use]
    pub fn box_query(&self, min: WorldCoord, max: WorldCoord) -> Vec<WorldCoord> {
        let mut result = Vec::new();

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let coord = WorldCoord::new(x, y);
                if self.is_solid(coord) {
                    result.push(coord);
                }
            }
        }

        result
    }

    /// Finds the ground level below a given position.
    ///
    /// Searches downward from start_y until finding a solid cell.
    ///
    /// # Arguments
    /// * `x` - X coordinate to search at
    /// * `start_y` - Y coordinate to start searching from (searches downward)
    ///
    /// # Returns
    /// The Y coordinate of the first solid cell found, or None.
    #[must_use]
    pub fn find_ground(&self, x: i64, start_y: i64) -> Option<i64> {
        // Search downward
        let min_y = self.chunk_origin.y;

        for y in (min_y..=start_y).rev() {
            let coord = WorldCoord::new(x, y);
            if self.is_solid(coord) {
                return Some(y);
            }
        }

        None
    }

    /// Checks if an entity can stand at a position.
    ///
    /// An entity can stand if there's a solid cell below and air above.
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate (feet position)
    /// * `height` - Entity height in cells
    #[must_use]
    pub fn can_stand_at(&self, x: i64, y: i64, height: u32) -> bool {
        // Must have ground below
        let below = WorldCoord::new(x, y - 1);
        if !self.is_solid(below) {
            return false;
        }

        // Must have air above for full height
        for dy in 0..height as i64 {
            let coord = WorldCoord::new(x, y + dy);
            if self.is_solid(coord) {
                return false;
            }
        }

        true
    }

    /// Returns the chunk size.
    #[must_use]
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Returns the chunk origin.
    #[must_use]
    pub const fn chunk_origin(&self) -> WorldCoord {
        self.chunk_origin
    }
}

/// Multi-chunk collision query that can span multiple chunks.
pub struct WorldCollisionQuery<'a> {
    /// Queries for individual chunks
    chunk_queries: Vec<CollisionQuery<'a>>,
}

impl<'a> WorldCollisionQuery<'a> {
    /// Creates a new world collision query from multiple chunk queries.
    #[must_use]
    pub fn new(chunk_queries: Vec<CollisionQuery<'a>>) -> Self {
        Self { chunk_queries }
    }

    /// Checks if a world coordinate is solid across all loaded chunks.
    #[must_use]
    pub fn is_solid(&self, coord: WorldCoord) -> bool {
        for query in &self.chunk_queries {
            if query.get_cell(coord).is_some() {
                return query.is_solid(coord);
            }
        }
        false
    }

    /// Finds the appropriate chunk query for a coordinate.
    #[must_use]
    pub fn find_chunk(&self, coord: WorldCoord) -> Option<&CollisionQuery<'a>> {
        self.chunk_queries
            .iter()
            .find(|q| q.get_cell(coord).is_some())
    }

    /// Performs a raycast across multiple chunks.
    #[must_use]
    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_dist: f32) -> Option<RayHit> {
        // For now, use the simple approach of checking the first chunk
        // In a real implementation, we'd need to handle chunk boundaries
        if let Some(query) = self.chunk_queries.first() {
            query.raycast(origin, direction, max_dist)
        } else {
            None
        }
    }
}

// =============================================================================
// Terrain Collision Detection (K-26)
// =============================================================================

use crate::chunk_manager::VisibleChunkManager;

/// Collision result from terrain check.
#[derive(Debug, Clone, Default)]
pub struct TerrainCollision {
    /// Whether a collision occurred.
    pub collided: bool,
    /// Push direction (normalized normal).
    pub normal: (f32, f32),
    /// How far into the solid cell.
    pub penetration: f32,
    /// Material type that was hit (if any).
    pub cell_type: Option<u16>,
}

impl TerrainCollision {
    /// Creates a no-collision result.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            collided: false,
            normal: (0.0, 0.0),
            penetration: 0.0,
            cell_type: None,
        }
    }

    /// Creates a collision result.
    #[must_use]
    pub fn hit(normal: (f32, f32), penetration: f32, cell_type: u16) -> Self {
        Self {
            collided: true,
            normal,
            penetration,
            cell_type: Some(cell_type),
        }
    }
}

/// Check collision between a circle and terrain.
///
/// Checks all cells within the circle's bounding box for solid cells,
/// then calculates the push-out direction and penetration.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn check_terrain_collision(
    chunk_manager: &VisibleChunkManager,
    position: (f32, f32),
    radius: f32,
) -> TerrainCollision {
    let mut total_normal = (0.0f32, 0.0f32);
    let mut max_penetration = 0.0f32;
    let mut hit_material: Option<u16> = None;

    // Check all cells that could be touching the circle
    let min_x = (position.0 - radius).floor() as i32;
    let max_x = (position.0 + radius).ceil() as i32;
    let min_y = (position.1 - radius).floor() as i32;
    let max_y = (position.1 + radius).ceil() as i32;

    for cell_y in min_y..=max_y {
        for cell_x in min_x..=max_x {
            if let Some(cell) = chunk_manager.get_cell(cell_x, cell_y) {
                // Check if cell is solid
                if cell.flags & CellFlags::SOLID != 0 {
                    // Cell center
                    let cell_cx = cell_x as f32 + 0.5;
                    let cell_cy = cell_y as f32 + 0.5;

                    // Find closest point on cell to circle center
                    let closest_x = position.0.clamp(cell_x as f32, cell_x as f32 + 1.0);
                    let closest_y = position.1.clamp(cell_y as f32, cell_y as f32 + 1.0);

                    // Distance from circle center to closest point
                    let dx = position.0 - closest_x;
                    let dy = position.1 - closest_y;
                    let dist_sq = dx * dx + dy * dy;
                    let dist = dist_sq.sqrt();

                    // Check if circle overlaps this cell
                    if dist < radius {
                        let penetration = radius - dist;

                        // Calculate push direction (from cell to circle)
                        let (nx, ny) = if dist > f32::EPSILON {
                            (dx / dist, dy / dist)
                        } else {
                            // Circle center inside cell - push towards cell center's opposite
                            let to_center_x = position.0 - cell_cx;
                            let to_center_y = position.1 - cell_cy;
                            let len =
                                (to_center_x * to_center_x + to_center_y * to_center_y).sqrt();
                            if len > f32::EPSILON {
                                (to_center_x / len, to_center_y / len)
                            } else {
                                (0.0, -1.0) // Default: push up
                            }
                        };

                        // Accumulate normals weighted by penetration
                        total_normal.0 += nx * penetration;
                        total_normal.1 += ny * penetration;

                        if penetration > max_penetration {
                            max_penetration = penetration;
                            hit_material = Some(cell.material);
                        }
                    }
                }
            }
        }
    }

    if max_penetration > 0.0 {
        // Normalize the accumulated normal
        let len = (total_normal.0 * total_normal.0 + total_normal.1 * total_normal.1).sqrt();
        let normal = if len > f32::EPSILON {
            (total_normal.0 / len, total_normal.1 / len)
        } else {
            (0.0, -1.0)
        };

        TerrainCollision::hit(normal, max_penetration, hit_material.unwrap_or(0))
    } else {
        TerrainCollision::none()
    }
}

/// Check collision for a moving object (sweep test).
///
/// Returns the collision info and the safe position (last non-colliding position).
#[must_use]
pub fn sweep_terrain_collision(
    chunk_manager: &VisibleChunkManager,
    start: (f32, f32),
    end: (f32, f32),
    radius: f32,
) -> (TerrainCollision, (f32, f32)) {
    // Number of steps to check (based on distance)
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < f32::EPSILON {
        // Not moving, just check current position
        let collision = check_terrain_collision(chunk_manager, start, radius);
        return (collision, start);
    }

    // Step size should be smaller than radius for accurate detection
    let step_count = ((dist / (radius * 0.5)).ceil() as usize).max(2);
    let step_x = dx / step_count as f32;
    let step_y = dy / step_count as f32;

    let mut last_safe = start;
    let mut pos = start;

    for _ in 0..=step_count {
        let collision = check_terrain_collision(chunk_manager, pos, radius);
        if collision.collided {
            return (collision, last_safe);
        }

        last_safe = pos;
        pos.0 += step_x;
        pos.1 += step_y;
    }

    // No collision along path
    (TerrainCollision::none(), end)
}

/// Resolve collision by pushing entity out of terrain.
#[must_use]
pub fn resolve_collision(position: (f32, f32), collision: &TerrainCollision) -> (f32, f32) {
    if !collision.collided {
        return position;
    }

    // Push out by penetration amount along normal
    (
        position.0 + collision.normal.0 * collision.penetration,
        position.1 + collision.normal.1 * collision.penetration,
    )
}

/// Checks if a point is inside solid terrain.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn is_point_in_terrain(chunk_manager: &VisibleChunkManager, x: f32, y: f32) -> bool {
    let cell_x = x.floor() as i32;
    let cell_y = y.floor() as i32;

    chunk_manager
        .get_cell(cell_x, cell_y)
        .is_some_and(|cell| cell.flags & CellFlags::SOLID != 0)
}

/// Gets the terrain material at a world position.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn get_terrain_material(chunk_manager: &VisibleChunkManager, x: f32, y: f32) -> Option<u16> {
    let cell_x = x.floor() as i32;
    let cell_y = y.floor() as i32;

    chunk_manager
        .get_cell(cell_x, cell_y)
        .map(|cell| cell.material)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cells(chunk_size: usize) -> Vec<Cell> {
        vec![Cell::default(); chunk_size * chunk_size]
    }

    #[test]
    fn test_vec2_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 0.001);
        assert!((n.x - 0.6).abs() < 0.001);
        assert!((n.y - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_is_solid() {
        let chunk_size = 16usize;
        let mut cells = create_test_cells(chunk_size);

        // Make cell at (5, 5) solid
        cells[5 * chunk_size + 5].flags = CellFlags::SOLID;

        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(0, 0));

        assert!(query.is_solid(WorldCoord::new(5, 5)));
        assert!(!query.is_solid(WorldCoord::new(0, 0)));
        assert!(!query.is_solid(WorldCoord::new(100, 100))); // Out of bounds
    }

    #[test]
    fn test_get_cell() {
        let chunk_size = 16usize;
        let mut cells = create_test_cells(chunk_size);
        cells[0].material = 42;

        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(0, 0));

        let cell = query.get_cell(WorldCoord::new(0, 0));
        assert!(cell.is_some());
        assert_eq!(cell.map(|c| c.material), Some(42));

        // Out of bounds
        assert!(query.get_cell(WorldCoord::new(-1, 0)).is_none());
        assert!(query.get_cell(WorldCoord::new(16, 0)).is_none());
    }

    #[test]
    fn test_box_query() {
        let chunk_size = 16usize;
        let mut cells = create_test_cells(chunk_size);

        // Create a small solid region
        for y in 2..=4 {
            for x in 2..=4 {
                cells[y * chunk_size + x].flags = CellFlags::SOLID;
            }
        }

        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(0, 0));
        let solids = query.box_query(WorldCoord::new(0, 0), WorldCoord::new(5, 5));

        // Should find 9 solid cells (3x3)
        assert_eq!(solids.len(), 9);
    }

    #[test]
    fn test_find_ground() {
        let chunk_size = 16usize;
        let mut cells = create_test_cells(chunk_size);

        // Create ground at y=5
        for x in 0..chunk_size {
            cells[5 * chunk_size + x].flags = CellFlags::SOLID;
        }

        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(0, 0));

        // Search from y=10, should find ground at y=5
        let ground = query.find_ground(8, 10);
        assert_eq!(ground, Some(5));

        // Search from y=3, should find nothing (below ground)
        let ground = query.find_ground(8, 3);
        assert_eq!(ground, None);
    }

    #[test]
    fn test_can_stand_at() {
        let chunk_size = 16usize;
        let mut cells = create_test_cells(chunk_size);

        // Create ground at y=5
        for x in 0..chunk_size {
            cells[5 * chunk_size + x].flags = CellFlags::SOLID;
        }

        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(0, 0));

        // Can stand on ground at y=6 with height 2
        assert!(query.can_stand_at(8, 6, 2));

        // Cannot stand in air (no ground below)
        assert!(!query.can_stand_at(8, 10, 2));

        // Cannot stand inside ground
        assert!(!query.can_stand_at(8, 5, 2));
    }

    #[test]
    fn test_raycast_hit() {
        let chunk_size = 16usize;
        let mut cells = create_test_cells(chunk_size);

        // Create a wall at x=10
        for y in 0..chunk_size {
            cells[y * chunk_size + 10].flags = CellFlags::SOLID;
        }

        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(0, 0));

        // Cast ray from (5, 5) towards right
        let hit = query.raycast(Vec2::new(5.0, 5.5), Vec2::new(1.0, 0.0), 20.0);

        assert!(hit.is_some());
        let hit = hit.expect("should have hit");
        assert_eq!(hit.coord.x, 10);
        assert_eq!(hit.coord.y, 5);
        assert!(hit.distance > 4.0 && hit.distance < 6.0);
    }

    #[test]
    fn test_raycast_miss() {
        let chunk_size = 16usize;
        let cells = create_test_cells(chunk_size); // No solid cells

        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(0, 0));

        // Cast ray from (5, 5) towards right - should miss
        let hit = query.raycast(Vec2::new(5.0, 5.5), Vec2::new(1.0, 0.0), 10.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_chunk_with_offset() {
        let chunk_size = 16usize;
        let mut cells = create_test_cells(chunk_size);

        // Make cell at local (5, 5) solid
        cells[5 * chunk_size + 5].flags = CellFlags::SOLID;

        // Chunk origin at (100, 200)
        let query = CollisionQuery::new(&cells, chunk_size as u32, WorldCoord::new(100, 200));

        // Cell should be solid at world (105, 205)
        assert!(query.is_solid(WorldCoord::new(105, 205)));
        assert!(!query.is_solid(WorldCoord::new(5, 5))); // Wrong - that's outside this chunk
    }

    #[test]
    fn test_terrain_collision_no_hit() {
        let collision = TerrainCollision::none();
        assert!(!collision.collided);
        assert!(collision.cell_type.is_none());
    }

    #[test]
    fn test_terrain_collision_hit() {
        let collision = TerrainCollision::hit((0.0, -1.0), 0.5, 5);
        assert!(collision.collided);
        assert_eq!(collision.normal, (0.0, -1.0));
        assert!((collision.penetration - 0.5).abs() < f32::EPSILON);
        assert_eq!(collision.cell_type, Some(5));
    }

    #[test]
    fn test_resolve_collision() {
        let position = (10.0, 10.0);

        // No collision - position unchanged
        let no_hit = TerrainCollision::none();
        let resolved = resolve_collision(position, &no_hit);
        assert_eq!(resolved, position);

        // With collision - pushed out
        let hit = TerrainCollision::hit((0.0, -1.0), 0.5, 5);
        let resolved = resolve_collision(position, &hit);
        assert!((resolved.0 - 10.0).abs() < f32::EPSILON);
        assert!((resolved.1 - 9.5).abs() < f32::EPSILON);
    }
}
