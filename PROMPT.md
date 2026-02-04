# Iteration 7: Gameplay Agent Tasks

## Context
You are the Gameplay Agent responsible for player mechanics, entities, and game systems.
The game now has working top-down movement. This iteration adds environment interaction
and weather/time systems.

## Tasks

### G-29: Grass Interaction System (P0)
**Goal:** Player can cut grass by interacting with it.

**Location:** `crates/genesis-gameplay/src/interaction.rs`

**Requirements:**
1. Add CutGrass action to interaction system
2. When player presses E near grass:
   - Grass cell converts to cut grass material
   - Player receives grass item
   - Cut grass regrows over time
3. Add grass-specific interaction radius

### G-30: Weather State System (P0)
**Goal:** Track weather state that affects environment simulation.

**Location:** `crates/genesis-gameplay/src/weather.rs` (new file)

**Requirements:**
1. Create WeatherSystem with states: Clear, Cloudy, Raining, Storm
2. Weather transitions over time randomly
3. Weather affects grass growth rate
4. Rain fills water bodies, hydrates soil
5. Expose weather state for kernel simulation

### G-31: Time/Day Cycle System (P0)
**Goal:** Game time system with day/night cycle.

**Location:** `crates/genesis-gameplay/src/time.rs` (new file)

**Requirements:**
1. Create GameTime with time_of_day (0.0-1.0), day_count
2. Time scale: 1 game minute = 1 real second
3. Time affects lighting (passed to kernel)
4. Helper methods: is_day(), is_night(), hour(), minute()

### G-32: Plant Growth System (P1)
**Goal:** Track plant entities and their growth lifecycle.

**Location:** `crates/genesis-gameplay/src/plants.rs` (new file)

**Requirements:**
1. PlantRegistry for plant types with growth stages
2. Growth affected by light, water, weather
3. Mature plants can be harvested
4. Integrate with grass cutting

## Files to Create/Modify
- crates/genesis-gameplay/src/weather.rs (new)
- crates/genesis-gameplay/src/time.rs (new)
- crates/genesis-gameplay/src/plants.rs (new)
- crates/genesis-gameplay/src/interaction.rs
- crates/genesis-gameplay/src/lib.rs
- crates/genesis-gameplay/src/game_state.rs

## Commit Format: [gameplay] feat: G-XX description
