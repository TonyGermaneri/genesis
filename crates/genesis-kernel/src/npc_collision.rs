//! NPC collision detection system.
//!
//! This module provides collision detection between:
//! - Player and NPCs
//! - NPCs and other NPCs
//! - NPCs and world geometry
//!
//! Uses circle collision for NPC bodies with separate interaction radius.

/// A 2D vector for collision operations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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

    /// Returns the squared length of the vector (avoids sqrt).
    #[must_use]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns the distance to another point.
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Position and collision data for an NPC.
#[derive(Debug, Clone, Copy)]
pub struct NpcPosition {
    /// Unique NPC identifier
    pub id: u32,
    /// World position
    pub position: Vec2,
    /// Collision radius (body size)
    pub collision_radius: f32,
    /// Interaction radius (larger than collision, for dialogue etc.)
    pub interaction_radius: f32,
    /// Whether this NPC blocks movement
    pub is_solid: bool,
}

impl NpcPosition {
    /// Creates a new NPC position.
    #[must_use]
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Self {
            id,
            position: Vec2::new(x, y),
            collision_radius: 12.0,
            interaction_radius: 32.0,
            is_solid: true,
        }
    }

    /// Sets the collision radius.
    #[must_use]
    pub const fn with_collision_radius(mut self, radius: f32) -> Self {
        self.collision_radius = radius;
        self
    }

    /// Sets the interaction radius.
    #[must_use]
    pub const fn with_interaction_radius(mut self, radius: f32) -> Self {
        self.interaction_radius = radius;
        self
    }

    /// Sets whether the NPC is solid.
    #[must_use]
    pub const fn with_solid(mut self, solid: bool) -> Self {
        self.is_solid = solid;
        self
    }
}

/// Result of a collision check with an NPC.
#[derive(Debug, Clone, Copy)]
pub struct NpcCollision {
    /// ID of the colliding NPC
    pub npc_id: u32,
    /// Penetration depth (negative means inside)
    pub penetration: f32,
    /// Collision normal (direction to push apart)
    pub normal: Vec2,
    /// Whether the entity is within interaction range (but not necessarily colliding)
    pub in_interaction_range: bool,
    /// Distance to NPC center
    pub distance: f32,
}

impl NpcCollision {
    /// Checks if this is an actual physical collision (bodies overlapping).
    #[must_use]
    pub fn is_colliding(&self) -> bool {
        self.penetration > 0.0
    }
}

/// Checks for collisions between a point/circle and NPCs.
///
/// # Arguments
/// * `player_pos` - Position to check from
/// * `player_radius` - Collision radius of the checking entity
/// * `npcs` - Slice of NPC positions to check against
///
/// # Returns
/// Vector of all NPC collisions, sorted by distance (closest first).
#[must_use]
pub fn check_npc_collisions(
    player_pos: Vec2,
    player_radius: f32,
    npcs: &[NpcPosition],
) -> Vec<NpcCollision> {
    let mut collisions = Vec::new();

    for npc in npcs {
        // Calculate distance between centers
        let dx = npc.position.x - player_pos.x;
        let dy = npc.position.y - player_pos.y;
        let distance_sq = dx * dx + dy * dy;
        let distance = distance_sq.sqrt();

        // Check interaction range first (larger radius)
        let interaction_distance = player_radius + npc.interaction_radius;
        let in_interaction_range = distance < interaction_distance;

        // Check collision (body overlap)
        let collision_distance = player_radius + npc.collision_radius;
        let penetration = collision_distance - distance;

        // Only add if within interaction range
        if in_interaction_range {
            // Calculate normal (direction to push player away from NPC)
            let normal = if distance > f32::EPSILON {
                Vec2::new(-dx / distance, -dy / distance)
            } else {
                // Overlapping exactly - push in arbitrary direction
                Vec2::new(1.0, 0.0)
            };

            collisions.push(NpcCollision {
                npc_id: npc.id,
                penetration,
                normal,
                in_interaction_range,
                distance,
            });
        }
    }

    // Sort by distance (closest first)
    collisions.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    collisions
}

/// Checks for collision between two circles.
#[must_use]
pub fn circles_collide(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> Option<(f32, Vec2)> {
    let dx = pos2.x - pos1.x;
    let dy = pos2.y - pos1.y;
    let distance_sq = dx * dx + dy * dy;
    let min_distance = radius1 + radius2;

    if distance_sq < min_distance * min_distance {
        let distance = distance_sq.sqrt();
        let penetration = min_distance - distance;
        let normal = if distance > f32::EPSILON {
            Vec2::new(dx / distance, dy / distance)
        } else {
            Vec2::new(1.0, 0.0)
        };
        Some((penetration, normal))
    } else {
        None
    }
}

/// Resolves collision by calculating push-back vector.
///
/// Returns the displacement vector to apply to move out of collision.
#[must_use]
pub fn resolve_npc_collision(collision: &NpcCollision) -> Vec2 {
    if collision.penetration > 0.0 {
        Vec2::new(
            collision.normal.x * collision.penetration,
            collision.normal.y * collision.penetration,
        )
    } else {
        Vec2::new(0.0, 0.0)
    }
}

/// Finds the closest NPC within interaction range.
#[must_use]
pub fn find_closest_interactable(
    player_pos: Vec2,
    player_radius: f32,
    npcs: &[NpcPosition],
) -> Option<NpcCollision> {
    check_npc_collisions(player_pos, player_radius, npcs)
        .into_iter()
        .find(|c| c.in_interaction_range)
}

/// Checks collisions between all NPCs (NPC-to-NPC).
///
/// Returns pairs of colliding NPC IDs with collision data.
#[must_use]
pub fn check_npc_to_npc_collisions(npcs: &[NpcPosition]) -> Vec<(u32, u32, f32, Vec2)> {
    let mut collisions = Vec::new();

    for i in 0..npcs.len() {
        for j in (i + 1)..npcs.len() {
            let npc1 = &npcs[i];
            let npc2 = &npcs[j];

            if !npc1.is_solid || !npc2.is_solid {
                continue;
            }

            if let Some((penetration, normal)) = circles_collide(
                npc1.position,
                npc1.collision_radius,
                npc2.position,
                npc2.collision_radius,
            ) {
                collisions.push((npc1.id, npc2.id, penetration, normal));
            }
        }
    }

    collisions
}

/// Manager for tracking NPC positions and handling collisions.
pub struct NpcCollisionManager {
    /// Active NPC positions
    npcs: Vec<NpcPosition>,
    /// Default collision radius for new NPCs
    default_collision_radius: f32,
    /// Default interaction radius for new NPCs
    default_interaction_radius: f32,
}

impl Default for NpcCollisionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcCollisionManager {
    /// Creates a new NPC collision manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            npcs: Vec::new(),
            default_collision_radius: 12.0,
            default_interaction_radius: 32.0,
        }
    }

    /// Sets the default collision radius for new NPCs.
    pub fn set_default_collision_radius(&mut self, radius: f32) {
        self.default_collision_radius = radius;
    }

    /// Sets the default interaction radius for new NPCs.
    pub fn set_default_interaction_radius(&mut self, radius: f32) {
        self.default_interaction_radius = radius;
    }

    /// Registers an NPC with the collision system.
    pub fn register_npc(&mut self, id: u32, x: f32, y: f32) {
        // Remove existing NPC with same ID
        self.npcs.retain(|n| n.id != id);

        self.npcs.push(
            NpcPosition::new(id, x, y)
                .with_collision_radius(self.default_collision_radius)
                .with_interaction_radius(self.default_interaction_radius),
        );
    }

    /// Registers an NPC with custom radii.
    pub fn register_npc_with_radii(
        &mut self,
        id: u32,
        x: f32,
        y: f32,
        collision_radius: f32,
        interaction_radius: f32,
    ) {
        self.npcs.retain(|n| n.id != id);
        self.npcs.push(
            NpcPosition::new(id, x, y)
                .with_collision_radius(collision_radius)
                .with_interaction_radius(interaction_radius),
        );
    }

    /// Unregisters an NPC from the collision system.
    pub fn unregister_npc(&mut self, id: u32) {
        self.npcs.retain(|n| n.id != id);
    }

    /// Updates an NPC's position.
    pub fn update_position(&mut self, id: u32, x: f32, y: f32) {
        if let Some(npc) = self.npcs.iter_mut().find(|n| n.id == id) {
            npc.position = Vec2::new(x, y);
        }
    }

    /// Checks collisions with the player.
    #[must_use]
    pub fn check_player_collisions(
        &self,
        player_pos: Vec2,
        player_radius: f32,
    ) -> Vec<NpcCollision> {
        check_npc_collisions(player_pos, player_radius, &self.npcs)
    }

    /// Finds the closest interactable NPC.
    #[must_use]
    pub fn find_closest_interactable(
        &self,
        player_pos: Vec2,
        player_radius: f32,
    ) -> Option<NpcCollision> {
        find_closest_interactable(player_pos, player_radius, &self.npcs)
    }

    /// Checks NPC-to-NPC collisions.
    #[must_use]
    pub fn check_npc_collisions(&self) -> Vec<(u32, u32, f32, Vec2)> {
        check_npc_to_npc_collisions(&self.npcs)
    }

    /// Returns all registered NPCs.
    #[must_use]
    pub fn npcs(&self) -> &[NpcPosition] {
        &self.npcs
    }

    /// Returns the number of registered NPCs.
    #[must_use]
    pub fn npc_count(&self) -> usize {
        self.npcs.len()
    }

    /// Clears all NPCs.
    pub fn clear(&mut self) {
        self.npcs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_position_builder() {
        let npc = NpcPosition::new(1, 100.0, 200.0)
            .with_collision_radius(20.0)
            .with_interaction_radius(50.0)
            .with_solid(false);

        assert_eq!(npc.id, 1);
        assert_eq!(npc.position.x, 100.0);
        assert_eq!(npc.position.y, 200.0);
        assert_eq!(npc.collision_radius, 20.0);
        assert_eq!(npc.interaction_radius, 50.0);
        assert!(!npc.is_solid);
    }

    #[test]
    fn test_no_collision_far_apart() {
        let npcs = vec![NpcPosition::new(1, 100.0, 100.0)];
        let collisions = check_npc_collisions(Vec2::new(0.0, 0.0), 10.0, &npcs);

        assert!(collisions.is_empty());
    }

    #[test]
    fn test_collision_detected() {
        let npcs = vec![NpcPosition::new(1, 20.0, 0.0).with_collision_radius(15.0)];
        let collisions = check_npc_collisions(Vec2::new(0.0, 0.0), 10.0, &npcs);

        assert_eq!(collisions.len(), 1);
        assert!(collisions[0].is_colliding());
        assert_eq!(collisions[0].npc_id, 1);
    }

    #[test]
    fn test_interaction_range() {
        let npcs = vec![NpcPosition::new(1, 40.0, 0.0)
            .with_collision_radius(10.0)
            .with_interaction_radius(35.0)];
        let collisions = check_npc_collisions(Vec2::new(0.0, 0.0), 10.0, &npcs);

        assert_eq!(collisions.len(), 1);
        assert!(collisions[0].in_interaction_range);
        assert!(!collisions[0].is_colliding()); // Not physically colliding
    }

    #[test]
    fn test_collision_normal() {
        let npcs = vec![NpcPosition::new(1, 10.0, 0.0).with_collision_radius(10.0)];
        let collisions = check_npc_collisions(Vec2::new(0.0, 0.0), 10.0, &npcs);

        assert_eq!(collisions.len(), 1);
        // Normal should point away from NPC (towards player)
        assert!(collisions[0].normal.x < 0.0);
        assert!((collisions[0].normal.y).abs() < 0.001);
    }

    #[test]
    fn test_circles_collide() {
        // Overlapping circles
        let result = circles_collide(Vec2::new(0.0, 0.0), 10.0, Vec2::new(15.0, 0.0), 10.0);
        assert!(result.is_some());
        let (penetration, _) = result.unwrap();
        assert!((penetration - 5.0).abs() < 0.001);

        // Non-overlapping circles
        let result = circles_collide(Vec2::new(0.0, 0.0), 10.0, Vec2::new(50.0, 0.0), 10.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_npc_to_npc_collisions() {
        let npcs = vec![
            NpcPosition::new(1, 0.0, 0.0).with_collision_radius(10.0),
            NpcPosition::new(2, 15.0, 0.0).with_collision_radius(10.0),
            NpcPosition::new(3, 100.0, 0.0).with_collision_radius(10.0),
        ];

        let collisions = check_npc_to_npc_collisions(&npcs);

        // Only NPCs 1 and 2 should collide
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].0, 1);
        assert_eq!(collisions[0].1, 2);
    }

    #[test]
    fn test_resolve_collision() {
        let collision = NpcCollision {
            npc_id: 1,
            penetration: 5.0,
            normal: Vec2::new(-1.0, 0.0),
            in_interaction_range: true,
            distance: 15.0,
        };

        let push = resolve_npc_collision(&collision);
        assert_eq!(push.x, -5.0);
        assert_eq!(push.y, 0.0);
    }

    #[test]
    fn test_collision_manager() {
        let mut manager = NpcCollisionManager::new();

        manager.register_npc(1, 20.0, 0.0);
        manager.register_npc(2, 100.0, 100.0);

        assert_eq!(manager.npc_count(), 2);

        let collisions = manager.check_player_collisions(Vec2::new(0.0, 0.0), 15.0);
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].npc_id, 1);

        manager.update_position(1, 200.0, 200.0);
        let collisions = manager.check_player_collisions(Vec2::new(0.0, 0.0), 15.0);
        assert!(collisions.is_empty());
    }

    #[test]
    fn test_find_closest_interactable() {
        let mut manager = NpcCollisionManager::new();

        manager.register_npc_with_radii(1, 50.0, 0.0, 10.0, 60.0);
        manager.register_npc_with_radii(2, 30.0, 0.0, 10.0, 40.0);

        let closest = manager.find_closest_interactable(Vec2::new(0.0, 0.0), 5.0);
        assert!(closest.is_some());
        assert_eq!(closest.unwrap().npc_id, 2); // NPC 2 is closer
    }

    #[test]
    fn test_collision_sorted_by_distance() {
        let npcs = vec![
            NpcPosition::new(1, 50.0, 0.0).with_interaction_radius(100.0),
            NpcPosition::new(2, 20.0, 0.0).with_interaction_radius(100.0),
            NpcPosition::new(3, 30.0, 0.0).with_interaction_radius(100.0),
        ];

        let collisions = check_npc_collisions(Vec2::new(0.0, 0.0), 5.0, &npcs);

        assert_eq!(collisions.len(), 3);
        assert_eq!(collisions[0].npc_id, 2); // Closest
        assert_eq!(collisions[1].npc_id, 3);
        assert_eq!(collisions[2].npc_id, 1); // Farthest
    }
}
