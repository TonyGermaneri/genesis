//! Player spawn and respawn system.
//!
//! This module handles player spawning at designated spawn points
//! and respawning after death.

use crate::input::Vec2;
use crate::inventory::Inventory;
use crate::player::Player;
use serde::{Deserialize, Serialize};

/// Default spawn point coordinates.
const DEFAULT_SPAWN_X: f32 = 0.0;
const DEFAULT_SPAWN_Y: f32 = 0.0;

/// Default respawn delay in seconds.
const DEFAULT_RESPAWN_DELAY: f32 = 3.0;

/// Default inventory size for new players.
const DEFAULT_INVENTORY_SIZE: usize = 20;

/// Configuration for the spawn system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    /// Default spawn point
    pub spawn_point: (f32, f32),
    /// Respawn delay in seconds
    pub respawn_delay: f32,
    /// Default inventory size
    pub inventory_size: usize,
    /// Whether to keep inventory on death
    pub keep_inventory_on_death: bool,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            spawn_point: (DEFAULT_SPAWN_X, DEFAULT_SPAWN_Y),
            respawn_delay: DEFAULT_RESPAWN_DELAY,
            inventory_size: DEFAULT_INVENTORY_SIZE,
            keep_inventory_on_death: false,
        }
    }
}

/// Handles player spawning and respawning.
#[derive(Debug)]
pub struct SpawnSystem {
    /// Current spawn point coordinates
    spawn_point: (f32, f32),
    /// Delay before respawning (in seconds)
    respawn_delay: f32,
    /// Default inventory size for new players
    inventory_size: usize,
    /// Whether to keep inventory on death
    keep_inventory_on_death: bool,
    /// Active respawn timers (entity_id -> remaining_time)
    respawn_timers: std::collections::HashMap<genesis_common::EntityId, f32>,
}

impl Default for SpawnSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SpawnSystem {
    /// Creates a new spawn system with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            spawn_point: (DEFAULT_SPAWN_X, DEFAULT_SPAWN_Y),
            respawn_delay: DEFAULT_RESPAWN_DELAY,
            inventory_size: DEFAULT_INVENTORY_SIZE,
            keep_inventory_on_death: false,
            respawn_timers: std::collections::HashMap::new(),
        }
    }

    /// Creates a new spawn system with custom configuration.
    #[must_use]
    pub fn with_config(config: &SpawnConfig) -> Self {
        Self {
            spawn_point: config.spawn_point,
            respawn_delay: config.respawn_delay,
            inventory_size: config.inventory_size,
            keep_inventory_on_death: config.keep_inventory_on_death,
            respawn_timers: std::collections::HashMap::new(),
        }
    }

    /// Sets the world spawn point.
    pub fn set_spawn_point(&mut self, x: f32, y: f32) {
        self.spawn_point = (x, y);
    }

    /// Returns the current spawn point.
    #[must_use]
    pub const fn spawn_point(&self) -> (f32, f32) {
        self.spawn_point
    }

    /// Sets the respawn delay.
    pub fn set_respawn_delay(&mut self, delay: f32) {
        self.respawn_delay = delay.max(0.0);
    }

    /// Returns the respawn delay.
    #[must_use]
    pub const fn respawn_delay(&self) -> f32 {
        self.respawn_delay
    }

    /// Sets whether to keep inventory on death.
    pub fn set_keep_inventory(&mut self, keep: bool) {
        self.keep_inventory_on_death = keep;
    }

    /// Returns whether inventory is kept on death.
    #[must_use]
    pub const fn keep_inventory_on_death(&self) -> bool {
        self.keep_inventory_on_death
    }

    /// Spawns a new player at the spawn point with default inventory.
    #[must_use]
    pub fn spawn_player(&self) -> Player {
        Player::new(Vec2::new(self.spawn_point.0, self.spawn_point.1))
    }

    /// Spawns a new player at a specific position.
    #[must_use]
    pub fn spawn_player_at(&self, x: f32, y: f32) -> Player {
        Player::new(Vec2::new(x, y))
    }

    /// Creates a default inventory for a new player.
    #[must_use]
    pub fn create_default_inventory(&self) -> Inventory {
        #[allow(clippy::cast_possible_truncation)]
        Inventory::new(self.inventory_size as u32)
    }

    /// Respawns the player after death.
    ///
    /// Resets the player's position to the spawn point and resets velocity.
    /// Optionally clears inventory based on configuration.
    pub fn respawn_player(&self, player: &mut Player) {
        // Reset position to spawn point
        player.set_position(Vec2::new(self.spawn_point.0, self.spawn_point.1));

        // Reset player state (grounded, velocity handled by Player)
        player.set_grounded(true);
    }

    /// Starts a respawn timer for a player.
    pub fn start_respawn_timer(&mut self, entity_id: genesis_common::EntityId) {
        self.respawn_timers.insert(entity_id, self.respawn_delay);
    }

    /// Updates respawn timers and returns IDs of players ready to respawn.
    pub fn update_timers(&mut self, dt: f32) -> Vec<genesis_common::EntityId> {
        let mut ready = Vec::new();

        self.respawn_timers.retain(|&id, time| {
            *time -= dt;
            if *time <= 0.0 {
                ready.push(id);
                false // Remove from map
            } else {
                true // Keep in map
            }
        });

        ready
    }

    /// Gets the remaining respawn time for a player.
    #[must_use]
    pub fn respawn_time_remaining(&self, entity_id: genesis_common::EntityId) -> Option<f32> {
        self.respawn_timers.get(&entity_id).copied()
    }

    /// Checks if a player is waiting to respawn.
    #[must_use]
    pub fn is_awaiting_respawn(&self, entity_id: genesis_common::EntityId) -> bool {
        self.respawn_timers.contains_key(&entity_id)
    }

    /// Cancels a pending respawn.
    pub fn cancel_respawn(&mut self, entity_id: genesis_common::EntityId) {
        self.respawn_timers.remove(&entity_id);
    }

    /// Returns the number of pending respawns.
    #[must_use]
    pub fn pending_respawn_count(&self) -> usize {
        self.respawn_timers.len()
    }
}

/// Data about a spawn event for logging/events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnEvent {
    /// Entity that spawned
    pub entity_id: genesis_common::EntityId,
    /// Position where they spawned
    pub position: (f32, f32),
    /// Whether this was a respawn
    pub is_respawn: bool,
    /// Game time when spawn occurred
    pub game_time: f64,
}

impl SpawnEvent {
    /// Creates a new spawn event.
    #[must_use]
    pub const fn new(
        entity_id: genesis_common::EntityId,
        position: (f32, f32),
        is_respawn: bool,
        game_time: f64,
    ) -> Self {
        Self {
            entity_id,
            position,
            is_respawn,
            game_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_common::EntityId;

    #[test]
    fn test_spawn_system_new() {
        let spawn = SpawnSystem::new();
        assert_eq!(spawn.spawn_point(), (0.0, 0.0));
        assert_eq!(spawn.respawn_delay(), DEFAULT_RESPAWN_DELAY);
    }

    #[test]
    fn test_spawn_system_with_config() {
        let config = SpawnConfig {
            spawn_point: (100.0, 200.0),
            respawn_delay: 5.0,
            inventory_size: 30,
            keep_inventory_on_death: true,
        };
        let spawn = SpawnSystem::with_config(&config);

        assert_eq!(spawn.spawn_point(), (100.0, 200.0));
        assert_eq!(spawn.respawn_delay(), 5.0);
        assert!(spawn.keep_inventory_on_death());
    }

    #[test]
    fn test_spawn_system_set_spawn_point() {
        let mut spawn = SpawnSystem::new();
        spawn.set_spawn_point(50.0, 75.0);
        assert_eq!(spawn.spawn_point(), (50.0, 75.0));
    }

    #[test]
    fn test_spawn_system_set_respawn_delay() {
        let mut spawn = SpawnSystem::new();
        spawn.set_respawn_delay(10.0);
        assert_eq!(spawn.respawn_delay(), 10.0);

        // Negative values clamped to 0
        spawn.set_respawn_delay(-5.0);
        assert_eq!(spawn.respawn_delay(), 0.0);
    }

    #[test]
    fn test_spawn_system_set_keep_inventory() {
        let mut spawn = SpawnSystem::new();
        assert!(!spawn.keep_inventory_on_death());

        spawn.set_keep_inventory(true);
        assert!(spawn.keep_inventory_on_death());
    }

    #[test]
    fn test_spawn_player() {
        let mut spawn = SpawnSystem::new();
        spawn.set_spawn_point(100.0, 200.0);

        let player = spawn.spawn_player();
        let pos = player.position();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
    }

    #[test]
    fn test_spawn_player_at() {
        let spawn = SpawnSystem::new();
        let player = spawn.spawn_player_at(50.0, 60.0);
        let pos = player.position();
        assert_eq!(pos.x, 50.0);
        assert_eq!(pos.y, 60.0);
    }

    #[test]
    fn test_create_default_inventory() {
        let spawn = SpawnSystem::new();
        let inv = spawn.create_default_inventory();
        assert_eq!(inv.capacity() as usize, DEFAULT_INVENTORY_SIZE);
    }

    #[test]
    fn test_respawn_player() {
        let mut spawn = SpawnSystem::new();
        spawn.set_spawn_point(100.0, 100.0);

        let mut player = Player::new(Vec2::new(500.0, 500.0));
        spawn.respawn_player(&mut player);

        let pos = player.position();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 100.0);
    }

    #[test]
    fn test_respawn_timer() {
        let mut spawn = SpawnSystem::new();
        spawn.set_respawn_delay(2.0);

        let id = EntityId::new();
        spawn.start_respawn_timer(id);

        assert!(spawn.is_awaiting_respawn(id));
        assert_eq!(spawn.respawn_time_remaining(id), Some(2.0));
        assert_eq!(spawn.pending_respawn_count(), 1);
    }

    #[test]
    fn test_update_timers() {
        let mut spawn = SpawnSystem::new();
        spawn.set_respawn_delay(1.0);

        let id = EntityId::new();
        spawn.start_respawn_timer(id);

        // Update but not enough time
        let ready = spawn.update_timers(0.5);
        assert!(ready.is_empty());
        assert!(spawn.is_awaiting_respawn(id));

        // Update past the delay
        let ready = spawn.update_timers(0.6);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], id);
        assert!(!spawn.is_awaiting_respawn(id));
    }

    #[test]
    fn test_cancel_respawn() {
        let mut spawn = SpawnSystem::new();
        let id = EntityId::new();

        spawn.start_respawn_timer(id);
        assert!(spawn.is_awaiting_respawn(id));

        spawn.cancel_respawn(id);
        assert!(!spawn.is_awaiting_respawn(id));
    }

    #[test]
    fn test_spawn_event() {
        let id = EntityId::new();
        let event = SpawnEvent::new(id, (10.0, 20.0), false, 100.0);

        assert_eq!(event.entity_id, id);
        assert_eq!(event.position, (10.0, 20.0));
        assert!(!event.is_respawn);
        assert_eq!(event.game_time, 100.0);
    }

    #[test]
    fn test_spawn_config_default() {
        let config = SpawnConfig::default();
        assert_eq!(config.spawn_point, (0.0, 0.0));
        assert_eq!(config.respawn_delay, DEFAULT_RESPAWN_DELAY);
        assert_eq!(config.inventory_size, DEFAULT_INVENTORY_SIZE);
        assert!(!config.keep_inventory_on_death);
    }

    #[test]
    fn test_multiple_respawn_timers() {
        let mut spawn = SpawnSystem::new();
        spawn.set_respawn_delay(1.0);

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        spawn.start_respawn_timer(id1);
        spawn.start_respawn_timer(id2);

        assert_eq!(spawn.pending_respawn_count(), 2);

        let ready = spawn.update_timers(1.5);
        assert_eq!(ready.len(), 2);
        assert_eq!(spawn.pending_respawn_count(), 0);
    }
}
