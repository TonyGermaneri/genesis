# Iteration 10: Tools Agent - Audio UI & Settings

## Objective
Create UI components for audio settings, volume controls, and debug visualization.

## Tasks

### 1. Audio Settings Panel (`crates/genesis-tools/src/ui/audio_settings.rs`)
Create volume control UI:
```rust
// Key components:
// - Master volume slider (0-100%)
// - Music volume slider
// - SFX volume slider
// - Ambient volume slider
// - Mute toggles per category
// - Apply/Reset buttons
```

### 2. Audio Debug UI (`crates/genesis-tools/src/ui/audio_debug.rs`)
Debug visualization:
```rust
// - Currently playing sounds list
// - Active music track display
// - Ambient layers status
// - Spatial audio sources on minimap
// - Audio channel usage meters
// - Peak level indicators
```

### 3. Sound Test Panel (`crates/genesis-tools/src/ui/sound_test.rs`)
Testing interface:
```rust
// - Sound browser by category
// - Play/stop controls
// - Position controls for spatial testing
// - Volume/pan preview
// - Loop toggle
```

### 4. Settings Persistence
Save/load audio preferences:
```rust
// - AudioSettings struct (all volumes, mute states)
// - Serialize to config file
// - Load on startup
// - Apply settings to audio system
```

### 5. Update UI module
Export new components in `crates/genesis-tools/src/ui/mod.rs`:
```rust
pub mod audio_settings;
pub mod audio_debug;
pub mod sound_test;
```

## UI Layout
```
┌─────────────────────────────────────┐
│ Audio Settings                   [X]│
├─────────────────────────────────────┤
│ Master Volume    [████████░░] 80%   │
│ Music Volume     [██████░░░░] 60%   │
│ SFX Volume       [████████░░] 80%   │
│ Ambient Volume   [██████████] 100%  │
├─────────────────────────────────────┤
│ [✓] Enable Spatial Audio            │
│ [ ] Mono Audio                      │
├─────────────────────────────────────┤
│      [Apply]  [Reset]  [Defaults]   │
└─────────────────────────────────────┘
```

## Technical Requirements
- Real-time volume updates (no restart)
- Slider granularity: 1%
- Visual feedback on change
- Keyboard accessibility

## Integration Points
- Uses kernel audio system for playback
- Gameplay agent respects volume settings
- Settings saved to config system (if exists)
