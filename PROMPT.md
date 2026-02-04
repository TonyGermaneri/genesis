# Iteration 10: Infra Agent - Audio Integration & Asset Loading

## Objective
Integrate all audio components into the engine and set up asset loading pipeline.

## Tasks

### 1. Audio Asset Loading (`crates/genesis-engine/src/audio_assets.rs`)
Create audio asset management:
```rust
// Key components:
// - AudioAssetLoader: loads MP3/WAV from assets/sounds/
// - Caching strategy (SFX cached, music streamed)
// - Asset validation on load
// - Fallback/placeholder for missing files
// - Hot-reload support for development
```

### 2. Audio System Integration (`crates/genesis-engine/src/audio_integration.rs`)
Wire up all audio components:
```rust
// - Initialize audio device on startup
// - Connect gameplay events to sound system
// - Apply volume settings from UI
// - Update spatial audio listener position
// - Handle audio device changes
```

### 3. Game Loop Integration
Update main loop in `crates/genesis-engine/src/lib.rs`:
```rust
// In game loop:
// 1. Process sound event queue
// 2. Update music based on game state
// 3. Update ambient based on player position/biome
// 4. Update spatial audio listener
// 5. Clean up finished sounds
```

### 4. Audio State Management (`crates/genesis-engine/src/audio_state.rs`)
Manage audio system state:
```rust
// - AudioState: current volumes, mute states
// - MusicState: current track, crossfade progress
// - AmbientState: active layers, transition progress
// - Save/restore audio state
```

### 5. Update Engine Exports
Add to `crates/genesis-engine/src/lib.rs`:
```rust
pub mod audio_assets;
pub mod audio_integration;
pub mod audio_state;
```

## Asset Structure
```
assets/sounds/
├── SOUND_ASSETS.md      # Asset manifest (reference)
├── music/               # Streaming MP3s (user provides)
│   ├── *.mp3.stub       # Placeholder stubs
├── ambient/             # Streaming ambient MP3s
│   ├── *.mp3.stub       # Placeholder stubs
└── sfx/                 # Short SFX (to be added)
    ├── player/
    ├── inventory/
    ├── environment/
    ├── npcs/
    ├── monsters/
    └── ui/
```

## Dependencies
Ensure `Cargo.toml` includes:
```toml
# In genesis-engine
rodio = { version = "0.19", features = ["mp3", "wav"] }
```

## Technical Requirements
- Graceful handling of missing assets
- Log warnings for stub files
- Memory management for large audio
- Clean shutdown of audio threads

## Integration Checklist
- [ ] Audio device initializes cleanly
- [ ] Music plays based on biome
- [ ] SFX play on game events
- [ ] Ambient layers work
- [ ] Volume controls function
- [ ] No audio artifacts on transitions
- [ ] Clean shutdown

## Error Handling
```rust
// Handle missing/corrupt audio gracefully:
// - Log error but don't crash
// - Show "missing audio" indicator in debug UI
// - Continue game without that sound
```
