//! Audio System
//!
//! This module provides the audio infrastructure for the Genesis kernel.
//! It re-exports types from the submodules for convenience.
//!
//! # Modules
//!
//! - [`audio_backend`](crate::audio_backend): Core rodio-based audio playback
//! - [`audio_spatial`](crate::audio_spatial): 2D spatial audio calculations
//! - [`audio_resource`](crate::audio_resource): Audio resource management
//!
//! # Quick Start
//!
//! ```ignore
//! use genesis_kernel::audio::*;
//!
//! // Create the audio engine
//! let engine = AudioEngine::new_default()?;
//!
//! // Load a sound effect
//! let sfx_id = engine.load_sound("assets/sounds/explosion.wav")?;
//!
//! // Play the sound
//! let handle = engine.play_sound(sfx_id)?;
//!
//! // Play spatial sound at a position
//! let handle2 = engine.play_sound_at(sfx_id, 100.0, 200.0)?;
//!
//! // Play music (streaming)
//! let music = engine.play_music("assets/music/theme.mp3")?;
//!
//! // Set listener position (usually camera position)
//! engine.set_listener_position(player_x, player_y);
//!
//! // Update each frame
//! engine.update();
//! ```
//!
//! # Categories and Volume
//!
//! Audio is organized into categories for mixing:
//!
//! - `Master`: Global volume multiplier
//! - `Music`: Background music
//! - `Sfx`: Sound effects
//! - `Ambient`: Environmental sounds
//! - `Ui`: UI/menu sounds
//! - `Voice`: Dialogue/voice
//!
//! ```ignore
//! use genesis_kernel::audio::{AudioEngine, AudioCategory};
//!
//! let engine = AudioEngine::new_default()?;
//!
//! // Set category volumes
//! engine.set_category_volume(AudioCategory::Music, 0.7);
//! engine.set_category_volume(AudioCategory::Sfx, 1.0);
//! engine.set_master_volume(0.8);
//! ```
//!
//! # Spatial Audio
//!
//! Spatial audio calculates volume and panning based on positions:
//!
//! ```ignore
//! use genesis_kernel::audio::{AudioEngine, Environment};
//!
//! let engine = AudioEngine::new_default()?;
//!
//! // Set listener (camera/player) position
//! engine.set_listener_position(0.0, 0.0);
//! engine.set_listener_direction(1.0, 0.0); // Facing right
//!
//! // Set environment for reverb
//! engine.set_environment(Environment::Cave);
//!
//! // Sounds are automatically panned and attenuated
//! let handle = engine.play_sound_at(sfx_id, 100.0, 50.0)?;
//! ```

// Re-export the backend module's public API
pub use crate::audio_backend::{
    AudioConfig, AudioDevice, AudioEngine, AudioError, AudioResult, AudioSinkPool, CachedBuffer,
    DistanceModel, Environment, Listener, SoundBufferId, SoundCategory, SoundHandle,
    DEFAULT_CHANNELS, DEFAULT_SAMPLE_RATE, MAX_CACHED_BUFFERS, MAX_SINKS,
};

// Re-export spatial audio types
pub use crate::audio_spatial::{
    AttenuationModel, AudioEnvironment, EnvironmentParams, ListenerData, SoundSourceData,
    SpatialAudioProcessor, SpatialParams, DEFAULT_MAX_DISTANCE, DEFAULT_REFERENCE_DISTANCE,
    MIN_AUDIBLE_VOLUME, SPEED_OF_SOUND,
};

// Re-export resource types
pub use crate::audio_resource::{
    AudioBuffer, AudioBufferCache, AudioCategory, AudioControls, AudioHandle, AudioSource,
    AudioSourceType, BufferId, HandleGenerator, PlaybackState, PlayingSound, VolumeSettings,
    DEFAULT_CROSSFADE_DURATION, MAX_CACHEABLE_SIZE,
};

// Legacy compatibility: re-export the old module for existing code
pub use crate::audio_legacy::{
    AudioListener,
    SoundSource,
    SoundSourceId,
    SpatialAudioManager,
    // Note: AttenuationModel and AudioEnvironment are already exported from audio_spatial
};
