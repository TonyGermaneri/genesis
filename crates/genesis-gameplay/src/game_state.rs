//! Central game state management.
//!
//! This module provides the `GameState` struct which serves as the single source
//! of truth for all game state, coordinating player, entities, and game systems.

use genesis_common::EntityId;
use serde::{Deserialize, Serialize};

use crate::combat::CombatSystem;
use crate::entity::EntityArena;
use crate::input::Input;
use crate::npc::{NPCManager, NPCStorage, NPCType, NPCWorld};
use crate::player::Player;

/// Central game state containing all gameplay data.
///
/// This struct is the single source of truth for game state and is used by the
/// engine for the main game loop.
#[derive(Debug)]
pub struct GameState {
    /// The player entity
    pub player: Player,
    /// All game entities
    pub entities: EntityArena,
    /// NPC manager for AI behaviors
    pub npc_manager: NPCManager,
    /// Combat system for attacks and damage
    pub combat_system: CombatSystem,
    /// Current NPC interaction state
    pub npc_interaction: NPCInteractionState,
    /// Current game time in seconds
    pub game_time: f64,
    /// Whether the game is paused
    pub paused: bool,
    /// World generation seed
    pub world_seed: u64,
    /// Accumulated time for fixed timestep updates
    accumulator: f64,
}

/// State of NPC interaction (dialogue, trade, etc.).
#[derive(Debug, Clone, Default)]
pub struct NPCInteractionState {
    /// Entity ID of the NPC being interacted with (if any)
    pub interacting_with: Option<EntityId>,
    /// Current interaction mode
    pub mode: NPCInteractionMode,
    /// Nearest interactable NPC and distance (for UI prompt)
    pub nearest_interactable: Option<(EntityId, f32)>,
}

/// Mode of player-NPC interaction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NPCInteractionMode {
    /// Not interacting with anyone
    #[default]
    None,
    /// In dialogue with an NPC
    Dialogue,
    /// Trading with a merchant NPC
    Trading,
}

/// Fixed timestep for physics updates (60 updates per second).
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

impl GameState {
    /// Creates a new game state with the given world seed.
    ///
    /// The player is spawned at the origin (0, 0).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            player: Player::new(crate::input::Vec2::ZERO),
            entities: EntityArena::new(),
            npc_manager: NPCManager::new(),
            combat_system: CombatSystem::default(),
            npc_interaction: NPCInteractionState::default(),
            game_time: 0.0,
            paused: false,
            world_seed: seed,
            accumulator: 0.0,
        }
    }

    /// Creates a new game state with a player at the specified position.
    #[must_use]
    pub fn with_player_position(seed: u64, position: (f32, f32)) -> Self {
        Self {
            player: Player::new(crate::input::Vec2::new(position.0, position.1)),
            entities: EntityArena::new(),
            npc_manager: NPCManager::new(),
            combat_system: CombatSystem::default(),
            npc_interaction: NPCInteractionState::default(),
            game_time: 0.0,
            paused: false,
            world_seed: seed,
            accumulator: 0.0,
        }
    }

    /// Updates all game systems for one frame.
    ///
    /// This is the main update loop called by the engine each frame.
    /// Uses a fixed timestep for physics with interpolation.
    ///
    /// # Arguments
    ///
    /// * `dt` - Delta time in seconds since the last frame
    /// * `input` - Current input state
    pub fn update(&mut self, dt: f32, input: &Input) {
        if self.paused {
            return;
        }

        // Accumulate time for fixed timestep
        self.accumulator += f64::from(dt);

        // Run fixed timestep updates
        while self.accumulator >= FIXED_TIMESTEP {
            self.fixed_update(FIXED_TIMESTEP as f32, input);
            self.accumulator -= FIXED_TIMESTEP;
        }

        // Update game time
        self.game_time += f64::from(dt);
    }

    /// Fixed timestep update for deterministic physics.
    fn fixed_update(&mut self, dt: f32, input: &Input) {
        // Update player based on input
        self.player.update(input, dt);

        // Update NPCs with player position for AI targeting
        let player_pos = self.player_position();
        // Use a simple entity ID for the player (ID 0)
        let player_id = EntityId::from_raw(0);

        // Create simple world/storage adapters
        let world = SimpleNPCWorld;
        let mut storage = SimpleNPCStorage;

        self.npc_manager.update(
            dt,
            player_id,
            player_pos,
            &world,
            &mut storage,
            &mut self.combat_system,
        );
    }

    /// Toggles the pause state of the game.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Sets the pause state directly.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Returns whether the game is currently paused.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns the player's current world position as a tuple.
    #[must_use]
    pub fn player_position(&self) -> (f32, f32) {
        let pos = self.player.position();
        (pos.x, pos.y)
    }

    /// Returns the player's current velocity as a tuple.
    #[must_use]
    pub fn player_velocity(&self) -> (f32, f32) {
        let vel = self.player.velocity();
        (vel.x, vel.y)
    }

    /// Returns a reference to the player.
    #[must_use]
    pub const fn player(&self) -> &Player {
        &self.player
    }

    /// Returns a mutable reference to the player.
    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    /// Returns the current game time in seconds.
    #[must_use]
    pub const fn game_time(&self) -> f64 {
        self.game_time
    }

    /// Returns the world seed.
    #[must_use]
    pub const fn world_seed(&self) -> u64 {
        self.world_seed
    }

    /// Returns a reference to the entity arena.
    #[must_use]
    pub const fn entities(&self) -> &EntityArena {
        &self.entities
    }

    /// Returns a mutable reference to the entity arena.
    pub fn entities_mut(&mut self) -> &mut EntityArena {
        &mut self.entities
    }

    /// Returns a reference to the NPC manager.
    #[must_use]
    pub const fn npc_manager(&self) -> &NPCManager {
        &self.npc_manager
    }

    /// Returns a mutable reference to the NPC manager.
    pub fn npc_manager_mut(&mut self) -> &mut NPCManager {
        &mut self.npc_manager
    }

    /// Returns the number of active NPCs.
    #[must_use]
    pub fn npc_count(&self) -> usize {
        self.npc_manager.len()
    }

    /// Returns a reference to the combat system.
    #[must_use]
    pub const fn combat_system(&self) -> &CombatSystem {
        &self.combat_system
    }

    /// Returns a mutable reference to the combat system.
    pub fn combat_system_mut(&mut self) -> &mut CombatSystem {
        &mut self.combat_system
    }

    /// Returns a reference to the NPC interaction state.
    #[must_use]
    pub const fn npc_interaction(&self) -> &NPCInteractionState {
        &self.npc_interaction
    }

    /// Returns whether the player is currently interacting with an NPC.
    #[must_use]
    pub fn is_interacting(&self) -> bool {
        self.npc_interaction.interacting_with.is_some()
    }

    /// Finds the nearest interactable NPC within range of the player.
    ///
    /// Returns the entity ID and distance if found.
    /// Only Merchant NPCs are currently interactable.
    #[must_use]
    pub fn find_nearest_interactable(&self, max_range: f32) -> Option<(EntityId, f32)> {
        let player_pos = self.player_position();
        let mut nearest: Option<(EntityId, f32)> = None;

        // Iterate through all NPCs to find the nearest interactable one
        for id in 0..self.npc_manager.len() as u64 {
            let entity_id = EntityId::from_raw(id + 1); // NPC IDs start at 1
            if let Some(npc) = self.npc_manager.get(entity_id) {
                // Only Merchant NPCs are interactable for now
                if npc.npc_type != NPCType::Merchant {
                    continue;
                }

                let dx = npc.position.0 - player_pos.0;
                let dy = npc.position.1 - player_pos.1;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= max_range {
                    if let Some((_, current_dist)) = nearest {
                        if dist < current_dist {
                            nearest = Some((entity_id, dist));
                        }
                    } else {
                        nearest = Some((entity_id, dist));
                    }
                }
            }
        }

        nearest
    }

    /// Updates the nearest interactable NPC (call each frame).
    pub fn update_nearest_interactable(&mut self) {
        const INTERACTION_RANGE: f32 = 3.0;
        self.npc_interaction.nearest_interactable = self.find_nearest_interactable(INTERACTION_RANGE);
    }

    /// Attempts to start an interaction with the nearest NPC.
    ///
    /// Returns true if an interaction was started.
    pub fn try_interact(&mut self) -> bool {
        // Don't allow interaction if already interacting
        if self.is_interacting() {
            return false;
        }

        // Find nearest interactable NPC
        const INTERACTION_RANGE: f32 = 3.0;
        if let Some((entity_id, _dist)) = self.find_nearest_interactable(INTERACTION_RANGE) {
            if let Some(npc) = self.npc_manager.get(entity_id) {
                // Start appropriate interaction based on NPC type
                let mode = match npc.npc_type {
                    NPCType::Merchant => NPCInteractionMode::Trading,
                    _ => NPCInteractionMode::Dialogue,
                };

                self.npc_interaction.interacting_with = Some(entity_id);
                self.npc_interaction.mode = mode;
                return true;
            }
        }

        false
    }

    /// Ends the current NPC interaction.
    pub fn end_interaction(&mut self) {
        self.npc_interaction.interacting_with = None;
        self.npc_interaction.mode = NPCInteractionMode::None;
    }

    /// Resets the game state with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.player = Player::new(crate::input::Vec2::ZERO);
        self.entities = EntityArena::new();
        self.npc_manager = NPCManager::new();
        self.combat_system = CombatSystem::default();
        self.npc_interaction = NPCInteractionState::default();
        self.game_time = 0.0;
        self.paused = false;
        self.world_seed = seed;
        self.accumulator = 0.0;
    }

    /// Gets the interpolation alpha for rendering between fixed timesteps.
    ///
    /// This can be used to interpolate positions for smooth rendering.
    #[must_use]
    pub fn interpolation_alpha(&self) -> f32 {
        (self.accumulator / FIXED_TIMESTEP) as f32
    }
}

/// Serializable snapshot of game state for saving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    /// Player position
    pub player_position: (f32, f32),
    /// Player velocity
    pub player_velocity: (f32, f32),
    /// Game time
    pub game_time: f64,
    /// World seed
    pub world_seed: u64,
}

impl GameStateSnapshot {
    /// Creates a snapshot from the current game state.
    #[must_use]
    pub fn from_state(state: &GameState) -> Self {
        Self {
            player_position: state.player_position(),
            player_velocity: state.player_velocity(),
            game_time: state.game_time,
            world_seed: state.world_seed,
        }
    }
}

/// Convenience struct for engine integration.
///
/// This struct bundles all gameplay systems together for easy use from the engine.
/// It provides a single entry point for initializing and updating gameplay.
#[derive(Debug)]
pub struct GameplaySystem {
    /// Central game state
    pub state: GameState,
    /// Spawn system for player spawning/respawning
    pub spawn: crate::spawn::SpawnSystem,
    /// Player physics configuration
    pub physics: crate::physics::PlayerPhysics,
}

impl GameplaySystem {
    /// Creates a new gameplay system with the given world seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            state: GameState::new(seed),
            spawn: crate::spawn::SpawnSystem::new(),
            physics: crate::physics::PlayerPhysics::default(),
        }
    }

    /// Initialize with a spawn point from the world.
    ///
    /// Sets the spawn point and spawns the player at that location.
    pub fn initialize(&mut self, spawn_point: (f32, f32)) {
        self.spawn.set_spawn_point(spawn_point.0, spawn_point.1);
        let player = self.spawn.spawn_player();
        self.state.player = player;
    }

    /// Update one frame of gameplay.
    ///
    /// This handles input processing, player updates, and physics.
    pub fn update<C: crate::physics::CollisionQuery>(
        &mut self,
        input: &crate::input::Input,
        collision: &C,
        dt: f32,
    ) {
        if self.state.is_paused() {
            return;
        }

        // Update player based on input
        self.state.player.handle_input(input, dt);

        // Apply physics with collision
        self.state.player.apply_physics(collision, dt);

        // Update game time
        self.state.game_time += f64::from(dt);
    }

    /// Get the player position for camera targeting.
    #[must_use]
    pub fn player_position(&self) -> (f32, f32) {
        self.state.player_position()
    }

    /// Get the player velocity.
    #[must_use]
    pub fn player_velocity(&self) -> (f32, f32) {
        self.state.player_velocity()
    }

    /// Check if game is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.state.is_paused()
    }

    /// Toggle pause state.
    pub fn toggle_pause(&mut self) {
        self.state.toggle_pause();
    }

    /// Get the current game time.
    #[must_use]
    pub fn game_time(&self) -> f64 {
        self.state.game_time()
    }

    /// Respawn the player at the spawn point.
    pub fn respawn_player(&mut self) {
        self.spawn.respawn_player(&mut self.state.player);
    }

    /// Get a reference to the player.
    #[must_use]
    pub fn player(&self) -> &crate::player::Player {
        self.state.player()
    }

    /// Get a mutable reference to the player.
    pub fn player_mut(&mut self) -> &mut crate::player::Player {
        self.state.player_mut()
    }
}

// =============================================================================
// Simple NPC trait implementations for basic world/storage integration
// =============================================================================

/// Simple NPC world implementation for pathfinding and LOS.
///
/// This is a basic implementation that always allows movement and LOS.
/// For more advanced gameplay, this should be replaced with actual world queries.
struct SimpleNPCWorld;

impl NPCWorld for SimpleNPCWorld {
    fn has_line_of_sight(&self, _from: (f32, f32), _to: (f32, f32)) -> bool {
        // Simple implementation: always have LOS
        // In a real game, this would ray-cast through the tile map
        true
    }

    fn get_next_waypoint(&self, from: (f32, f32), to: (f32, f32)) -> Option<(f32, f32)> {
        // Simple implementation: direct path to target
        // In a real game, this would use A* pathfinding
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 0.1 {
            None
        } else {
            // Return a point 1 unit toward the target
            let step = 1.0_f32.min(dist);
            Some((from.0 + dx / dist * step, from.1 + dy / dist * step))
        }
    }

    fn is_walkable(&self, _pos: (f32, f32)) -> bool {
        // Simple implementation: everywhere is walkable
        // In a real game, this would check collision with tiles
        true
    }
}

/// Simple NPC storage for entity data.
///
/// This is a no-op implementation since we store NPC state directly in NPCManager.
/// For more advanced systems, this would integrate with the ECS.
struct SimpleNPCStorage;

impl NPCStorage for SimpleNPCStorage {
    fn get_health_percent(&self, _entity: EntityId) -> Option<f32> {
        // NPCs have full health by default
        Some(1.0)
    }

    fn get_position(&self, _entity: EntityId) -> Option<(f32, f32)> {
        // Position is managed by NPCManager directly
        None
    }

    fn set_position(&mut self, _entity: EntityId, _pos: (f32, f32)) {
        // Position is managed by NPCManager directly
    }

    fn get_facing(&self, _entity: EntityId) -> Option<f32> {
        // Facing is managed by NPCManager directly
        None
    }

    fn set_facing(&mut self, _entity: EntityId, _facing: f32) {
        // Facing is managed by NPCManager directly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_new() {
        let state = GameState::new(12345);
        assert_eq!(state.world_seed, 12345);
        assert!(!state.is_paused());
        assert_eq!(state.game_time(), 0.0);
    }

    #[test]
    fn test_game_state_with_player_position() {
        let state = GameState::with_player_position(42, (100.0, 200.0));
        let pos = state.player_position();
        assert_eq!(pos.0, 100.0);
        assert_eq!(pos.1, 200.0);
    }

    #[test]
    fn test_game_state_toggle_pause() {
        let mut state = GameState::new(0);
        assert!(!state.is_paused());

        state.toggle_pause();
        assert!(state.is_paused());

        state.toggle_pause();
        assert!(!state.is_paused());
    }

    #[test]
    fn test_game_state_set_paused() {
        let mut state = GameState::new(0);
        state.set_paused(true);
        assert!(state.is_paused());

        state.set_paused(false);
        assert!(!state.is_paused());
    }

    #[test]
    fn test_game_state_update_when_paused() {
        let mut state = GameState::new(0);
        state.set_paused(true);

        let input = Input::new();
        state.update(1.0, &input);

        // Game time should not advance when paused
        assert_eq!(state.game_time(), 0.0);
    }

    #[test]
    fn test_game_state_update() {
        let mut state = GameState::new(0);
        let input = Input::new();

        state.update(0.1, &input);
        assert!(state.game_time() > 0.0);
    }

    #[test]
    fn test_game_state_player_velocity() {
        let state = GameState::new(0);
        let vel = state.player_velocity();
        assert_eq!(vel.0, 0.0);
        assert_eq!(vel.1, 0.0);
    }

    #[test]
    fn test_game_state_entities() {
        let state = GameState::new(0);
        assert!(state.entities().is_empty());
    }

    #[test]
    fn test_game_state_entities_mut() {
        use crate::entity::EntityType;

        let mut state = GameState::new(0);
        let id = state.entities_mut().spawn(EntityType::Npc);
        assert!(state.entities().get(id).is_ok());
    }

    #[test]
    fn test_game_state_reset() {
        let mut state = GameState::new(100);
        state.game_time = 500.0;
        state.set_paused(true);

        state.reset(200);

        assert_eq!(state.world_seed(), 200);
        assert_eq!(state.game_time(), 0.0);
        assert!(!state.is_paused());
    }

    #[test]
    fn test_game_state_interpolation_alpha() {
        let state = GameState::new(0);
        let alpha = state.interpolation_alpha();
        assert!(alpha >= 0.0 && alpha <= 1.0);
    }

    #[test]
    fn test_game_state_snapshot() {
        let state = GameState::with_player_position(42, (10.0, 20.0));
        let snapshot = GameStateSnapshot::from_state(&state);

        assert_eq!(snapshot.player_position, (10.0, 20.0));
        assert_eq!(snapshot.world_seed, 42);
        assert_eq!(snapshot.game_time, 0.0);
    }

    #[test]
    fn test_game_state_player_access() {
        let mut state = GameState::new(0);

        // Test immutable access
        let _ = state.player();

        // Test mutable access
        state
            .player_mut()
            .set_position(crate::input::Vec2::new(50.0, 50.0));
        assert_eq!(state.player_position(), (50.0, 50.0));
    }

    #[test]
    fn test_game_state_fixed_timestep_multiple() {
        let mut state = GameState::new(0);
        let input = Input::new();

        // Update with more than one fixed timestep worth of time
        state.update(0.05, &input); // About 3 fixed timesteps
        assert!(state.game_time() > 0.0);
    }

    #[test]
    fn test_gameplay_system_new() {
        let system = GameplaySystem::new(12345);
        assert_eq!(system.state.world_seed(), 12345);
        assert!(!system.is_paused());
    }

    #[test]
    fn test_gameplay_system_initialize() {
        let mut system = GameplaySystem::new(0);
        system.initialize((100.0, 200.0));

        let pos = system.player_position();
        assert_eq!(pos.0, 100.0);
        assert_eq!(pos.1, 200.0);
        assert_eq!(system.spawn.spawn_point(), (100.0, 200.0));
    }

    #[test]
    fn test_gameplay_system_update() {
        use crate::physics::MockCollision;

        let mut system = GameplaySystem::new(0);
        system.initialize((0.0, 0.0));

        let input = Input::new();
        let collision = MockCollision::new();

        system.update(&input, &collision, 0.016);
        assert!(system.game_time() > 0.0);
    }

    #[test]
    fn test_gameplay_system_update_paused() {
        use crate::physics::MockCollision;

        let mut system = GameplaySystem::new(0);
        system.toggle_pause();
        assert!(system.is_paused());

        let input = Input::new();
        let collision = MockCollision::new();

        system.update(&input, &collision, 1.0);
        assert_eq!(system.game_time(), 0.0); // Should not advance
    }

    #[test]
    fn test_gameplay_system_respawn() {
        let mut system = GameplaySystem::new(0);
        system.initialize((100.0, 100.0));

        // Move player away
        system
            .player_mut()
            .set_position(crate::input::Vec2::new(500.0, 500.0));

        // Respawn
        system.respawn_player();

        let pos = system.player_position();
        assert_eq!(pos.0, 100.0);
        assert_eq!(pos.1, 100.0);
    }

    #[test]
    fn test_gameplay_system_player_access() {
        let mut system = GameplaySystem::new(0);

        // Immutable access
        let _ = system.player();

        // Mutable access
        system
            .player_mut()
            .set_position(crate::input::Vec2::new(10.0, 20.0));
        assert_eq!(system.player_position(), (10.0, 20.0));
    }

    #[test]
    fn test_gameplay_system_player_velocity() {
        let system = GameplaySystem::new(0);
        let vel = system.player_velocity();
        assert_eq!(vel.0, 0.0);
        assert_eq!(vel.1, 0.0);
    }
}
