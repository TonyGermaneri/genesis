# Iteration 10: Gameplay Agent - Sound Events & Music System

## Objective
Implement the gameplay layer for sound events, music management, and ambient soundscapes.

## Tasks

### 1. Sound Events (`crates/genesis-gameplay/src/sound_events.rs`)
Create event-driven sound system:
```rust
// Key components:
// - SoundEvent enum (footsteps, combat, pickup, etc.)
// - SoundEventQueue for batching events
// - Priority system (SFX > ambient > music)
// - Cooldown system to prevent sound spam
```

### 2. Music Manager (`crates/genesis-gameplay/src/music.rs`)
Implement music system:
```rust
// - MusicTrack enum matching assets/sounds/music/
// - Biome-to-track mapping
// - Crossfade transitions (configurable duration)
// - Combat music triggers and fadeout
// - Day/night music variants
```

### 3. Ambient Soundscape (`crates/genesis-gameplay/src/ambient.rs`)
Create ambient sound layers:
```rust
// - AmbientLayer struct (biome, time of day)
// - Multi-layer mixing (base + weather + extras)
// - Smooth transitions between biomes
// - Weather sound integration (rain, storm)
```

### 4. Sound Triggers (`crates/genesis-gameplay/src/sound_triggers.rs`)
Game event to sound mappings:
```rust
// - Player action sounds (walk, run, jump, attack)
// - Inventory sounds (open, close, pickup, drop)
// - NPC sounds (footsteps, dialogue start/end)
// - Environment sounds (door, chest, water splash)
// - Monster sounds (growl, attack, death)
```

### 5. Update lib.rs
Export new modules:
```rust
pub mod sound_events;
pub mod music;
pub mod ambient;
pub mod sound_triggers;
```

## Asset Mappings
Reference `assets/sounds/SOUND_ASSETS.md` for all available sound files.

Music tracks map to biomes:
- Forest/default → forest.mp3
- Desert → desert.mp3
- Combat → combat.mp3
- Boss fights → boss.mp3
- Night time → night.mp3
- Villages → village.mp3
- Menu → menu_theme.mp3
- Exploration/travel → exploration.mp3

## Technical Requirements
- Non-blocking sound triggers
- Configurable volumes per category
- Sound pooling for rapid SFX
- Graceful fallback if assets missing

## Integration Points
- Kernel agent provides audio backend
- Tools agent provides volume UI
- Responds to game state changes
