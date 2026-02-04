# Iteration 10: Kernel Agent - Audio Backend

## Objective
Implement low-level audio streaming and spatial audio infrastructure using rodio.

## Tasks

### 1. Audio System Core (`crates/genesis-kernel/src/audio.rs`)
Create the core audio backend:
```rust
// Key components:
// - AudioDevice: wraps rodio::OutputStream
// - AudioSink pool for managing multiple simultaneous sounds
// - Streaming support for MP3 files (music/ambient)
// - Sample caching for frequently-used SFX
```

### 2. Spatial Audio (`crates/genesis-kernel/src/audio_spatial.rs`)
Implement 2D spatial audio:
```rust
// - Distance-based volume attenuation
// - Stereo panning based on listener/source positions
// - Configurable falloff curves (linear, exponential)
// - Listener position updates from camera
```

### 3. Audio Resources (`crates/genesis-kernel/src/audio_resource.rs`)
Resource management:
```rust
// - AudioHandle for tracking playing sounds
// - AudioBuffer for cached samples
// - Streaming handles for long-form audio
// - Volume/pan/speed controls per-handle
```

### 4. Update lib.rs
Export new audio modules:
```rust
pub mod audio;
pub mod audio_spatial;
pub mod audio_resource;
```

## Dependencies
Add to `crates/genesis-kernel/Cargo.toml`:
```toml
rodio = { version = "0.19", features = ["mp3"] }
```

## Technical Requirements
- Thread-safe audio playback
- Support for multiple simultaneous streams
- Crossfading capability for music transitions
- Low-latency SFX playback
- Memory-efficient streaming for large files

## File Structure
```
crates/genesis-kernel/src/
├── audio.rs          # Core audio device/sink management
├── audio_spatial.rs  # 2D spatial audio calculations
└── audio_resource.rs # Audio handles and resources
```

## Integration Points
- Gameplay agent will use this for sound events
- Tools agent will use this for volume controls
- Should not block render thread
