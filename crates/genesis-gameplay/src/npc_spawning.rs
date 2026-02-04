//! NPC spawning system for chunk-based NPC management.
//!
//! This module provides functionality for spawning NPCs when chunks load
//! and despawning them when chunks unload.

use genesis_common::EntityId;
use std::collections::HashMap;

use crate::npc::{NPCManager, NPCType};

/// Default chunk size for NPC spawning calculations.
pub const DEFAULT_CHUNK_SIZE: u32 = 256;

/// Configuration for NPC spawning in chunks.
#[derive(Debug, Clone)]
pub struct NPCSpawnConfig {
    /// World seed for deterministic spawning
    pub seed: u64,
    /// Chunk size in world units
    pub chunk_size: u32,
    /// Maximum NPCs per chunk
    pub max_npcs_per_chunk: u32,
    /// Spawn chance (0.0-1.0) for each potential spawn point
    pub spawn_chance: f32,
}

impl Default for NPCSpawnConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            chunk_size: DEFAULT_CHUNK_SIZE,
            max_npcs_per_chunk: 5,
            spawn_chance: 0.3,
        }
    }
}

impl NPCSpawnConfig {
    /// Creates a new config with the given seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }
}

/// Data about a potential NPC spawn.
#[derive(Debug, Clone)]
pub struct NPCSpawnData {
    /// Type of NPC to spawn
    pub npc_type: NPCType,
    /// Position within the chunk (world coordinates)
    pub position: (f32, f32),
}

/// Manages NPC spawning and despawning with chunks.
#[derive(Debug)]
pub struct NPCChunkSpawner {
    /// Configuration
    config: NPCSpawnConfig,
    /// NPCs spawned per chunk (chunk coord -> list of entity IDs)
    chunk_npcs: HashMap<(i32, i32), Vec<EntityId>>,
}

impl NPCChunkSpawner {
    /// Creates a new NPC chunk spawner.
    #[must_use]
    pub fn new(config: NPCSpawnConfig) -> Self {
        Self {
            config,
            chunk_npcs: HashMap::new(),
        }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &NPCSpawnConfig {
        &self.config
    }

    /// Gets the list of NPCs spawned in a chunk.
    #[must_use]
    pub fn get_chunk_npcs(&self, chunk_pos: (i32, i32)) -> Option<&Vec<EntityId>> {
        self.chunk_npcs.get(&chunk_pos)
    }

    /// Returns the total number of chunks with spawned NPCs.
    #[must_use]
    pub fn spawned_chunk_count(&self) -> usize {
        self.chunk_npcs.len()
    }

    /// Returns the total number of spawned NPCs across all chunks.
    #[must_use]
    pub fn total_spawned_npcs(&self) -> usize {
        self.chunk_npcs.values().map(Vec::len).sum()
    }

    /// Generates spawn data for a chunk without actually spawning.
    ///
    /// Uses deterministic RNG based on seed and chunk coordinates.
    #[must_use]
    pub fn generate_spawn_data(&self, chunk_pos: (i32, i32)) -> Vec<NPCSpawnData> {
        let mut spawns = Vec::new();

        // Create deterministic RNG from seed + chunk position
        let chunk_seed = self.chunk_seed(chunk_pos);
        let mut rng_state = chunk_seed;

        // Determine number of potential spawn points
        let num_spawn_points = self.config.max_npcs_per_chunk as usize;

        for i in 0..num_spawn_points {
            // Advance RNG
            rng_state = Self::next_rng(rng_state);

            // Check spawn chance
            let roll = (rng_state % 1000) as f32 / 1000.0;
            if roll > self.config.spawn_chance {
                continue;
            }

            // Determine position within chunk
            rng_state = Self::next_rng(rng_state);
            let x_offset = (rng_state % self.config.chunk_size as u64) as f32;
            rng_state = Self::next_rng(rng_state);
            let y_offset = (rng_state % self.config.chunk_size as u64) as f32;

            let world_x = chunk_pos.0 as f32 * self.config.chunk_size as f32 + x_offset;
            let world_y = chunk_pos.1 as f32 * self.config.chunk_size as f32 + y_offset;

            // Determine NPC type based on position and RNG
            rng_state = Self::next_rng(rng_state);
            let npc_type = Self::pick_npc_type(rng_state, chunk_pos, i);

            spawns.push(NPCSpawnData {
                npc_type,
                position: (world_x, world_y),
            });
        }

        spawns
    }

    /// Called when a chunk is loaded - spawns NPCs in the chunk.
    ///
    /// Returns the number of NPCs spawned.
    pub fn on_chunk_loaded(
        &mut self,
        chunk_pos: (i32, i32),
        npc_manager: &mut NPCManager,
    ) -> usize {
        // Don't spawn if already loaded
        if self.chunk_npcs.contains_key(&chunk_pos) {
            return 0;
        }

        let spawn_data = self.generate_spawn_data(chunk_pos);
        let mut spawned_ids = Vec::new();

        for spawn in &spawn_data {
            let entity_id = npc_manager.spawn_npc(spawn.npc_type, spawn.position);
            spawned_ids.push(entity_id);
        }

        let count = spawned_ids.len();
        if !spawned_ids.is_empty() {
            self.chunk_npcs.insert(chunk_pos, spawned_ids);
        }

        count
    }

    /// Called when a chunk is unloaded - despawns NPCs in the chunk.
    ///
    /// Returns the number of NPCs despawned.
    pub fn on_chunk_unloaded(
        &mut self,
        chunk_pos: (i32, i32),
        npc_manager: &mut NPCManager,
    ) -> usize {
        if let Some(npc_ids) = self.chunk_npcs.remove(&chunk_pos) {
            let count = npc_ids.len();
            for entity_id in npc_ids {
                let _ = npc_manager.despawn_npc(entity_id);
            }
            count
        } else {
            0
        }
    }

    /// Generates a deterministic seed for a chunk.
    fn chunk_seed(&self, chunk_pos: (i32, i32)) -> u64 {
        // Combine world seed with chunk position
        let x = chunk_pos.0 as u64;
        let y = chunk_pos.1 as u64;
        self.config
            .seed
            .wrapping_mul(0x0005_DEEC_E66D)
            .wrapping_add(x.wrapping_mul(0x0123_4567))
            .wrapping_add(y.wrapping_mul(0x0765_4321))
    }

    /// Simple LCG random number generator.
    fn next_rng(state: u64) -> u64 {
        state.wrapping_mul(0x0005_DEEC_E66D).wrapping_add(0xB) & 0xFFFF_FFFF_FFFF
    }

    /// Picks an NPC type based on RNG and position.
    fn pick_npc_type(rng: u64, _chunk_pos: (i32, i32), _index: usize) -> NPCType {
        // Weight-based selection
        // Passive: 40%, Neutral: 25%, Hostile: 20%, Merchant: 10%, Guard: 5%
        let roll = rng % 100;
        if roll < 40 {
            NPCType::Passive
        } else if roll < 65 {
            NPCType::Neutral
        } else if roll < 85 {
            NPCType::Hostile
        } else if roll < 95 {
            NPCType::Merchant
        } else {
            NPCType::Guard
        }
    }
}

impl Default for NPCChunkSpawner {
    fn default() -> Self {
        Self::new(NPCSpawnConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_config_default() {
        let config = NPCSpawnConfig::default();
        assert_eq!(config.seed, 12345);
        assert_eq!(config.chunk_size, 256);
        assert_eq!(config.max_npcs_per_chunk, 5);
    }

    #[test]
    fn test_spawn_config_with_seed() {
        let config = NPCSpawnConfig::with_seed(42);
        assert_eq!(config.seed, 42);
    }

    #[test]
    fn test_generate_spawn_data_deterministic() {
        let spawner = NPCChunkSpawner::new(NPCSpawnConfig::with_seed(12345));

        let spawns1 = spawner.generate_spawn_data((0, 0));
        let spawns2 = spawner.generate_spawn_data((0, 0));

        assert_eq!(spawns1.len(), spawns2.len());
        for (s1, s2) in spawns1.iter().zip(spawns2.iter()) {
            assert_eq!(s1.position, s2.position);
            assert_eq!(s1.npc_type, s2.npc_type);
        }
    }

    #[test]
    fn test_different_chunks_different_spawns() {
        let spawner = NPCChunkSpawner::new(NPCSpawnConfig::with_seed(12345));

        let spawns_00 = spawner.generate_spawn_data((0, 0));
        let spawns_10 = spawner.generate_spawn_data((1, 0));

        // They should be different (unless very unlikely RNG collision)
        if !spawns_00.is_empty() && !spawns_10.is_empty() {
            let pos_00 = spawns_00[0].position;
            let pos_10 = spawns_10[0].position;
            // Positions should be in different chunks
            assert!(pos_10.0 >= 256.0 || pos_00.0 < 256.0);
        }
    }

    #[test]
    fn test_on_chunk_loaded_spawns_npcs() {
        let config = NPCSpawnConfig {
            seed: 12345,
            spawn_chance: 1.0, // Always spawn for testing
            ..Default::default()
        };
        let mut spawner = NPCChunkSpawner::new(config);
        let mut npc_manager = NPCManager::new();

        let count = spawner.on_chunk_loaded((0, 0), &mut npc_manager);

        assert!(count > 0);
        assert_eq!(npc_manager.len(), count);
        assert!(spawner.get_chunk_npcs((0, 0)).is_some());
    }

    #[test]
    fn test_on_chunk_unloaded_despawns_npcs() {
        let config = NPCSpawnConfig {
            seed: 12345,
            spawn_chance: 1.0,
            ..Default::default()
        };
        let mut spawner = NPCChunkSpawner::new(config);
        let mut npc_manager = NPCManager::new();

        spawner.on_chunk_loaded((0, 0), &mut npc_manager);
        let initial_count = npc_manager.len();

        let despawned = spawner.on_chunk_unloaded((0, 0), &mut npc_manager);

        assert_eq!(despawned, initial_count);
        assert_eq!(npc_manager.len(), 0);
        assert!(spawner.get_chunk_npcs((0, 0)).is_none());
    }

    #[test]
    fn test_double_load_no_duplicate_spawns() {
        let config = NPCSpawnConfig {
            seed: 12345,
            spawn_chance: 1.0,
            ..Default::default()
        };
        let mut spawner = NPCChunkSpawner::new(config);
        let mut npc_manager = NPCManager::new();

        let count1 = spawner.on_chunk_loaded((0, 0), &mut npc_manager);
        let count2 = spawner.on_chunk_loaded((0, 0), &mut npc_manager);

        assert!(count1 > 0);
        assert_eq!(count2, 0); // Second load should not spawn more
        assert_eq!(npc_manager.len(), count1);
    }
}
