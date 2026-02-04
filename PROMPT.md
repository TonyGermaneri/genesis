# Iteration 13: Gameplay Agent - Game State Serialization

## Objective
Implement player state, NPC persistence, world time, and progress tracking.

## Tasks

### 1. Player State Serialization (player_save.rs)
- PlayerSaveData: position, rotation, inventory
- Stats: HP, stamina, experience, level
- Equipment: equipped items, hotbar
- Quest progress, learned recipes

### 2. NPC State Persistence (npc_save.rs)
- NPC positions and AI states
- Health, inventory for each NPC
- Dialogue progress flags
- Respawn timers for defeated NPCs

### 3. World Time/Weather Save (world_state_save.rs)
- Current day/time
- Weather state and forecast
- Moon phase, season
- Environment event timers

### 4. Game Progress Tracking (progress_save.rs)
- Discovered map regions
- Achievements unlocked
- Statistics (enemies killed, items crafted)
- Playtime tracking

### 5. Update lib.rs
Export: player_save, npc_save, world_state_save, progress_save
