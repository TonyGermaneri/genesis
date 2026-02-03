//! Replay and determinism system.
//!
//! This module provides comprehensive replay recording and playback for
//! deterministic game sessions. It captures:
//! - All input events (keyboard, mouse, actions)
//! - Frame-by-frame state hashes for verification
//! - Timing information for accurate playback
//! - World seed for deterministic world generation

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/// Magic bytes for replay file format.
pub const REPLAY_MAGIC: &[u8; 4] = b"GRPL";

/// Current replay format version.
pub const REPLAY_VERSION: u32 = 1;

/// Input action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Input {
    /// Move left
    MoveLeft,
    /// Move right
    MoveRight,
    /// Move up
    MoveUp,
    /// Move down
    MoveDown,
    /// Jump action
    Jump,
    /// Primary action (attack/interact)
    Primary,
    /// Secondary action
    Secondary,
    /// Open inventory
    Inventory,
    /// Open map
    Map,
    /// Pause game
    Pause,
}

/// Mouse button types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button
    Middle,
}

/// Mouse input event.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MouseInput {
    /// X position in screen coordinates
    pub x: f32,
    /// Y position in screen coordinates
    pub y: f32,
    /// Button pressed (if any)
    pub button: Option<MouseButton>,
    /// Whether button was pressed this frame
    pub pressed: bool,
    /// Scroll wheel delta
    pub scroll_delta: f32,
}

impl Default for MouseInput {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            button: None,
            pressed: false,
            scroll_delta: 0.0,
        }
    }
}

/// A single frame of input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFrame {
    /// Frame number
    pub frame: u64,
    /// Inputs active this frame
    pub inputs: Vec<Input>,
    /// Mouse input state
    pub mouse: Option<MouseInput>,
    /// Delta time in microseconds since last frame
    pub delta_time_us: u64,
}

impl InputFrame {
    /// Creates a new input frame.
    #[must_use]
    pub fn new(frame: u64) -> Self {
        Self {
            frame,
            inputs: Vec::new(),
            mouse: None,
            delta_time_us: 0,
        }
    }

    /// Adds an input action to this frame.
    pub fn add_input(&mut self, input: Input) {
        self.inputs.push(input);
    }

    /// Sets the mouse state for this frame.
    pub fn set_mouse(&mut self, mouse: MouseInput) {
        self.mouse = Some(mouse);
    }

    /// Sets the delta time for this frame.
    pub fn set_delta_time(&mut self, delta_us: u64) {
        self.delta_time_us = delta_us;
    }
}

/// A state hash for determinism verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateHash {
    /// Frame this hash was captured at
    pub frame: u64,
    /// Hash of the world state
    pub world_hash: u64,
    /// Hash of entity state
    pub entity_hash: u64,
}

impl StateHash {
    /// Creates a new state hash.
    #[must_use]
    pub const fn new(frame: u64, world_hash: u64, entity_hash: u64) -> Self {
        Self {
            frame,
            world_hash,
            entity_hash,
        }
    }

    /// Computes a combined hash.
    #[must_use]
    pub fn combined(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.frame.hash(&mut hasher);
        self.world_hash.hash(&mut hasher);
        self.entity_hash.hash(&mut hasher);
        hasher.finish()
    }
}

/// Replay metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMetadata {
    /// Version of the replay format
    pub version: u32,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Duration in frames
    pub frame_count: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Recording timestamp (Unix epoch)
    pub recorded_at: u64,
}

impl Default for ReplayMetadata {
    fn default() -> Self {
        Self {
            version: REPLAY_VERSION,
            name: String::new(),
            description: String::new(),
            frame_count: 0,
            duration_ms: 0,
            recorded_at: 0,
        }
    }
}

/// A complete replay recording.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Replay {
    /// Metadata about the replay
    pub metadata: ReplayMetadata,
    /// World seed used
    pub seed: u32,
    /// Starting timestamp (Unix epoch ms)
    pub timestamp: u64,
    /// All recorded frames
    pub frames: Vec<InputFrame>,
    /// State hashes for verification (sparse - captured at intervals)
    pub state_hashes: Vec<StateHash>,
    /// Hash capture interval (every N frames)
    pub hash_interval: u64,
}

impl Replay {
    /// Creates a new empty replay.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a replay with a specific seed.
    #[must_use]
    pub fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Returns the total frame count.
    #[must_use]
    pub fn frame_count(&self) -> u64 {
        self.frames.len() as u64
    }

    /// Returns the total duration in microseconds.
    #[must_use]
    pub fn total_duration_us(&self) -> u64 {
        self.frames.iter().map(|f| f.delta_time_us).sum()
    }

    /// Finds the state hash for a given frame.
    #[must_use]
    pub fn find_hash_at(&self, frame: u64) -> Option<&StateHash> {
        self.state_hashes.iter().find(|h| h.frame == frame)
    }

    /// Finds the nearest state hash at or before a given frame.
    #[must_use]
    pub fn find_hash_before(&self, frame: u64) -> Option<&StateHash> {
        self.state_hashes
            .iter()
            .filter(|h| h.frame <= frame)
            .max_by_key(|h| h.frame)
    }

    /// Serializes the replay to JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<Vec<u8>, ReplayError> {
        serde_json::to_vec(self).map_err(|e| ReplayError::Serialization(e.to_string()))
    }

    /// Deserializes a replay from JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json(data: &[u8]) -> Result<Self, ReplayError> {
        serde_json::from_slice(data).map_err(|e| ReplayError::Deserialization(e.to_string()))
    }

    /// Saves the replay to a writer.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn save<W: Write>(&self, mut writer: W) -> Result<(), ReplayError> {
        // Write magic bytes
        writer
            .write_all(REPLAY_MAGIC)
            .map_err(|e| ReplayError::Io(e.to_string()))?;

        // Write version
        writer
            .write_all(&REPLAY_VERSION.to_le_bytes())
            .map_err(|e| ReplayError::Io(e.to_string()))?;

        // Write JSON payload
        let json = self.to_json()?;
        let len = json.len() as u64;
        writer
            .write_all(&len.to_le_bytes())
            .map_err(|e| ReplayError::Io(e.to_string()))?;
        writer
            .write_all(&json)
            .map_err(|e| ReplayError::Io(e.to_string()))?;

        Ok(())
    }

    /// Loads a replay from a reader.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or parsing fails.
    pub fn load<R: Read>(mut reader: R) -> Result<Self, ReplayError> {
        // Read and verify magic
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|e| ReplayError::Io(e.to_string()))?;
        if &magic != REPLAY_MAGIC {
            return Err(ReplayError::InvalidFormat("Invalid magic bytes".into()));
        }

        // Read version
        let mut version_bytes = [0u8; 4];
        reader
            .read_exact(&mut version_bytes)
            .map_err(|e| ReplayError::Io(e.to_string()))?;
        let version = u32::from_le_bytes(version_bytes);
        if version > REPLAY_VERSION {
            return Err(ReplayError::UnsupportedVersion(version));
        }

        // Read JSON payload
        let mut len_bytes = [0u8; 8];
        reader
            .read_exact(&mut len_bytes)
            .map_err(|e| ReplayError::Io(e.to_string()))?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut json = vec![0u8; len];
        reader
            .read_exact(&mut json)
            .map_err(|e| ReplayError::Io(e.to_string()))?;

        Self::from_json(&json)
    }
}

/// Error types for replay operations.
#[derive(Debug, Clone)]
pub enum ReplayError {
    /// IO error during read/write
    Io(String),
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Invalid file format
    InvalidFormat(String),
    /// Unsupported replay version
    UnsupportedVersion(u32),
    /// Determinism verification failed
    DeterminismFailure {
        /// Frame where divergence occurred
        frame: u64,
        /// Expected hash
        expected: u64,
        /// Actual hash
        actual: u64,
    },
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "IO error: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Deserialization(msg) => write!(f, "Deserialization error: {msg}"),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported version: {v}"),
            Self::DeterminismFailure {
                frame,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Determinism failure at frame {frame}: expected {expected:#x}, got {actual:#x}"
                )
            },
        }
    }
}

impl std::error::Error for ReplayError {}

/// Configuration for replay recording.
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    /// Interval for capturing state hashes (0 = disabled)
    pub hash_interval: u64,
    /// Whether to capture mouse input
    pub capture_mouse: bool,
    /// Whether to capture frame timing
    pub capture_timing: bool,
    /// Maximum frames to record (0 = unlimited)
    pub max_frames: u64,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            hash_interval: 60, // Capture hash every 60 frames (~1 second at 60fps)
            capture_mouse: true,
            capture_timing: true,
            max_frames: 0, // Unlimited
        }
    }
}

/// Records gameplay for replay.
#[derive(Debug)]
pub struct ReplayRecorder {
    /// Current replay being recorded
    replay: Replay,
    /// Recording configuration
    config: RecordingConfig,
    /// Current frame number
    current_frame: u64,
    /// Last frame timestamp (microseconds)
    last_frame_time_us: u64,
    /// Whether recording is active
    is_recording: bool,
    /// Pending inputs for current frame
    pending_inputs: Vec<Input>,
    /// Pending mouse input for current frame
    pending_mouse: Option<MouseInput>,
}

impl Default for ReplayRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayRecorder {
    /// Creates a new recorder with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(RecordingConfig::default())
    }

    /// Creates a new recorder with custom configuration.
    #[must_use]
    pub fn with_config(config: RecordingConfig) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            replay: Replay {
                timestamp,
                hash_interval: config.hash_interval,
                ..Default::default()
            },
            config,
            current_frame: 0,
            last_frame_time_us: 0,
            is_recording: false,
            pending_inputs: Vec::new(),
            pending_mouse: None,
        }
    }

    /// Sets the world seed.
    pub fn set_seed(&mut self, seed: u32) {
        self.replay.seed = seed;
    }

    /// Sets replay metadata.
    pub fn set_metadata(&mut self, name: impl Into<String>, description: impl Into<String>) {
        self.replay.metadata.name = name.into();
        self.replay.metadata.description = description.into();
    }

    /// Starts recording.
    pub fn start(&mut self) {
        self.is_recording = true;
        self.last_frame_time_us = current_time_us();
    }

    /// Stops recording.
    pub fn stop(&mut self) {
        self.is_recording = false;
    }

    /// Returns whether recording is active.
    #[must_use]
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Returns the current frame number.
    #[must_use]
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Records an input action for the current frame.
    pub fn record_input_action(&mut self, input: Input) {
        if self.is_recording {
            self.pending_inputs.push(input);
        }
    }

    /// Records mouse input for the current frame.
    pub fn record_mouse(&mut self, mouse: MouseInput) {
        if self.is_recording && self.config.capture_mouse {
            self.pending_mouse = Some(mouse);
        }
    }

    /// Records an input frame directly (for compatibility).
    pub fn record_input(&mut self, frame: InputFrame) {
        self.replay.frames.push(frame);
    }

    /// Commits the current frame and advances to the next.
    ///
    /// Call this at the end of each game frame.
    pub fn end_frame(&mut self) {
        if !self.is_recording {
            return;
        }

        // Check frame limit
        if self.config.max_frames > 0 && self.current_frame >= self.config.max_frames {
            self.stop();
            return;
        }

        // Calculate delta time
        let now = current_time_us();
        let delta_time_us = if self.config.capture_timing {
            now.saturating_sub(self.last_frame_time_us)
        } else {
            16667 // ~60fps default
        };
        self.last_frame_time_us = now;

        // Create frame
        let frame = InputFrame {
            frame: self.current_frame,
            inputs: std::mem::take(&mut self.pending_inputs),
            mouse: self.pending_mouse.take(),
            delta_time_us,
        };

        self.replay.frames.push(frame);
        self.current_frame += 1;
    }

    /// Records a state hash for determinism verification.
    pub fn record_state_hash(&mut self, world_hash: u64, entity_hash: u64) {
        if !self.is_recording {
            return;
        }

        // Only record at configured intervals
        if self.config.hash_interval > 0
            && (self.current_frame % self.config.hash_interval == 0 || self.current_frame == 0)
        {
            self.replay.state_hashes.push(StateHash::new(
                self.current_frame,
                world_hash,
                entity_hash,
            ));
        }
    }

    /// Forces recording of a state hash regardless of interval.
    pub fn force_record_state_hash(&mut self, world_hash: u64, entity_hash: u64) {
        if self.is_recording {
            self.replay.state_hashes.push(StateHash::new(
                self.current_frame,
                world_hash,
                entity_hash,
            ));
        }
    }

    /// Finishes recording and returns the replay.
    #[must_use]
    pub fn finish(mut self) -> Replay {
        self.stop();

        // Update metadata
        self.replay.metadata.frame_count = self.current_frame;
        self.replay.metadata.duration_ms = self.replay.total_duration_us() / 1000;
        self.replay.metadata.recorded_at = self.replay.timestamp;

        self.replay
    }

    /// Returns a reference to the current replay state.
    #[must_use]
    pub fn replay(&self) -> &Replay {
        &self.replay
    }
}

/// Gets the current time in microseconds.
fn current_time_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Plays back a replay.
#[derive(Debug)]
pub struct ReplayPlayer {
    /// Replay being played
    replay: Replay,
    /// Current frame index
    index: usize,
    /// Playback speed multiplier (1.0 = normal)
    speed: f64,
    /// Whether playback is paused
    paused: bool,
    /// Accumulated time for timing-based playback (microseconds)
    accumulated_time_us: u64,
}

impl ReplayPlayer {
    /// Creates a new player for the given replay.
    #[must_use]
    pub fn new(replay: Replay) -> Self {
        Self {
            replay,
            index: 0,
            speed: 1.0,
            paused: false,
            accumulated_time_us: 0,
        }
    }

    /// Returns the world seed.
    #[must_use]
    pub fn seed(&self) -> u32 {
        self.replay.seed
    }

    /// Returns the replay metadata.
    #[must_use]
    pub fn metadata(&self) -> &ReplayMetadata {
        &self.replay.metadata
    }

    /// Returns the total frame count.
    #[must_use]
    pub fn frame_count(&self) -> u64 {
        self.replay.frames.len() as u64
    }

    /// Returns the current frame index.
    #[must_use]
    pub fn current_index(&self) -> usize {
        self.index
    }

    /// Sets the playback speed.
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.max(0.0);
    }

    /// Pauses playback.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes playback.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggles pause state.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Returns whether playback is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns the next input frame, if any.
    pub fn next_frame(&mut self) -> Option<&InputFrame> {
        if self.paused || self.index >= self.replay.frames.len() {
            return None;
        }

        let frame = &self.replay.frames[self.index];
        self.index += 1;
        Some(frame)
    }

    /// Returns the next frame using real-time timing.
    ///
    /// Pass the elapsed microseconds since last call.
    /// Returns `None` if not enough time has passed or playback is complete.
    pub fn next_frame_timed(&mut self, elapsed_us: u64) -> Option<&InputFrame> {
        if self.paused || self.index >= self.replay.frames.len() {
            return None;
        }

        self.accumulated_time_us += (elapsed_us as f64 * self.speed) as u64;

        let frame = &self.replay.frames[self.index];
        if self.accumulated_time_us >= frame.delta_time_us {
            self.accumulated_time_us = self.accumulated_time_us.saturating_sub(frame.delta_time_us);
            self.index += 1;
            Some(frame)
        } else {
            None
        }
    }

    /// Seeks to a specific frame index.
    pub fn seek(&mut self, frame_index: usize) {
        self.index = frame_index.min(self.replay.frames.len());
        self.accumulated_time_us = 0;
    }

    /// Seeks to the beginning.
    pub fn seek_start(&mut self) {
        self.seek(0);
    }

    /// Seeks to the end.
    pub fn seek_end(&mut self) {
        self.seek(self.replay.frames.len());
    }

    /// Checks if playback is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.index >= self.replay.frames.len()
    }

    /// Resets playback to the beginning.
    pub fn reset(&mut self) {
        self.index = 0;
        self.accumulated_time_us = 0;
        self.paused = false;
    }

    /// Returns the frame at a specific index without advancing.
    #[must_use]
    pub fn peek_frame(&self, index: usize) -> Option<&InputFrame> {
        self.replay.frames.get(index)
    }

    /// Returns the current frame without advancing.
    #[must_use]
    pub fn peek_current(&self) -> Option<&InputFrame> {
        self.replay.frames.get(self.index)
    }

    /// Returns the expected state hash at the current frame.
    #[must_use]
    pub fn expected_hash(&self) -> Option<&StateHash> {
        self.replay.find_hash_at(self.index as u64)
    }

    /// Verifies a state hash against the recorded hash.
    ///
    /// # Errors
    ///
    /// Returns `DeterminismFailure` if the hashes don't match.
    pub fn verify_state(
        &self,
        frame: u64,
        world_hash: u64,
        entity_hash: u64,
    ) -> Result<(), ReplayError> {
        if let Some(expected) = self.replay.find_hash_at(frame) {
            let actual = StateHash::new(frame, world_hash, entity_hash);
            if expected.combined() != actual.combined() {
                return Err(ReplayError::DeterminismFailure {
                    frame,
                    expected: expected.combined(),
                    actual: actual.combined(),
                });
            }
        }
        Ok(())
    }

    /// Returns a reference to the underlying replay.
    #[must_use]
    pub fn replay(&self) -> &Replay {
        &self.replay
    }

    /// Consumes the player and returns the replay.
    #[must_use]
    pub fn into_replay(self) -> Replay {
        self.replay
    }

    /// Steps forward one frame regardless of timing.
    ///
    /// Returns the frame if available, or `None` if at end.
    pub fn step_forward(&mut self) -> Option<&InputFrame> {
        if self.index < self.replay.frames.len() {
            let frame = &self.replay.frames[self.index];
            self.index += 1;
            self.accumulated_time_us = 0;
            Some(frame)
        } else {
            None
        }
    }

    /// Steps backward one frame.
    ///
    /// Returns the frame if available, or `None` if at start.
    pub fn step_backward(&mut self) -> Option<&InputFrame> {
        if self.index > 0 {
            self.index -= 1;
            self.accumulated_time_us = 0;
            self.replay.frames.get(self.index)
        } else {
            None
        }
    }

    /// Returns the progress as a fraction (0.0 to 1.0).
    #[must_use]
    pub fn progress(&self) -> f64 {
        if self.replay.frames.is_empty() {
            return 0.0;
        }
        self.index as f64 / self.replay.frames.len() as f64
    }

    /// Returns the elapsed time in microseconds (based on frame deltas up to current position).
    #[must_use]
    pub fn elapsed_time_us(&self) -> u64 {
        self.replay.frames[..self.index]
            .iter()
            .map(|f| f.delta_time_us)
            .sum()
    }

    /// Returns the remaining time in microseconds.
    #[must_use]
    pub fn remaining_time_us(&self) -> u64 {
        self.replay.frames[self.index..]
            .iter()
            .map(|f| f.delta_time_us)
            .sum()
    }
}

/// Playback session that handles verification during playback.
#[derive(Debug)]
pub struct VerifiedPlaybackSession {
    /// The player
    player: ReplayPlayer,
    /// Verification errors encountered
    verification_errors: Vec<ReplayError>,
    /// Whether to stop on first verification error
    stop_on_error: bool,
}

impl VerifiedPlaybackSession {
    /// Creates a new verified playback session.
    #[must_use]
    pub fn new(replay: Replay) -> Self {
        Self {
            player: ReplayPlayer::new(replay),
            verification_errors: Vec::new(),
            stop_on_error: false,
        }
    }

    /// Sets whether to stop on first verification error.
    pub fn set_stop_on_error(&mut self, stop: bool) {
        self.stop_on_error = stop;
    }

    /// Returns the underlying player.
    #[must_use]
    pub fn player(&self) -> &ReplayPlayer {
        &self.player
    }

    /// Returns a mutable reference to the underlying player.
    pub fn player_mut(&mut self) -> &mut ReplayPlayer {
        &mut self.player
    }

    /// Advances to next frame and verifies state if hash is available.
    ///
    /// Returns `Ok(Some(frame))` if frame was retrieved and verification passed,
    /// `Ok(None)` if playback is complete,
    /// `Err` if verification failed and `stop_on_error` is true.
    pub fn next_frame_verified(
        &mut self,
        world_hash: u64,
        entity_hash: u64,
    ) -> Result<Option<&InputFrame>, &ReplayError> {
        // Get current frame index before advancing
        let current_frame = self.player.index as u64;

        // Verify state at current position if hash exists
        if let Err(e) = self
            .player
            .verify_state(current_frame, world_hash, entity_hash)
        {
            self.verification_errors.push(e);
            if self.stop_on_error {
                return Err(self.verification_errors.last().expect("just pushed"));
            }
        }

        Ok(self.player.next_frame())
    }

    /// Returns all verification errors encountered.
    #[must_use]
    pub fn verification_errors(&self) -> &[ReplayError] {
        &self.verification_errors
    }

    /// Returns whether playback had any verification errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.verification_errors.is_empty()
    }

    /// Consumes the session and returns the results.
    #[must_use]
    pub fn finish(self) -> (Replay, Vec<ReplayError>) {
        (self.player.into_replay(), self.verification_errors)
    }
}

// ============================================================================
// Determinism Verification System (T-3)
// ============================================================================

/// Result of comparing two replays for determinism.
#[derive(Debug, Clone, PartialEq)]
pub enum DeterminismResult {
    /// Both replays are identical
    Identical,
    /// Replays have different seeds
    SeedMismatch {
        /// Expected seed
        expected: u32,
        /// Actual seed
        actual: u32,
    },
    /// Replays have different frame counts
    FrameCountMismatch {
        /// Expected frame count
        expected: u64,
        /// Actual frame count
        actual: u64,
    },
    /// Input diverged at a specific frame
    InputDiverged {
        /// Frame where divergence occurred
        frame: u64,
        /// Expected inputs
        expected_inputs: Vec<Input>,
        /// Actual inputs
        actual_inputs: Vec<Input>,
    },
    /// State hash diverged at a specific frame
    StateDiverged {
        /// Frame where divergence occurred
        frame: u64,
        /// Expected state hash
        expected: StateHash,
        /// Actual state hash
        actual: StateHash,
    },
    /// Mouse input diverged at a specific frame
    MouseDiverged {
        /// Frame where divergence occurred
        frame: u64,
        /// Expected mouse state
        expected: Option<MouseInput>,
        /// Actual mouse state
        actual: Option<MouseInput>,
    },
}

impl std::fmt::Display for DeterminismResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identical => write!(f, "Replays are identical"),
            Self::SeedMismatch { expected, actual } => {
                write!(f, "Seed mismatch: expected {expected}, got {actual}")
            },
            Self::FrameCountMismatch { expected, actual } => {
                write!(f, "Frame count mismatch: expected {expected}, got {actual}")
            },
            Self::InputDiverged {
                frame,
                expected_inputs,
                actual_inputs,
            } => {
                write!(
                    f,
                    "Input divergence at frame {frame}: expected {expected_inputs:?}, got {actual_inputs:?}"
                )
            },
            Self::StateDiverged {
                frame,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "State divergence at frame {frame}: world hash {:#x} vs {:#x}, entity hash {:#x} vs {:#x}",
                    expected.world_hash, actual.world_hash, expected.entity_hash, actual.entity_hash
                )
            },
            Self::MouseDiverged {
                frame,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Mouse divergence at frame {frame}: expected {expected:?}, got {actual:?}"
                )
            },
        }
    }
}

/// Detailed divergence information for debugging.
#[derive(Debug, Clone)]
pub struct DivergenceReport {
    /// The result type
    pub result: DeterminismResult,
    /// Frame number where first divergence occurred
    pub frame: u64,
    /// Additional context messages
    pub context: Vec<String>,
}

impl DivergenceReport {
    /// Creates a new divergence report.
    #[must_use]
    pub fn new(result: DeterminismResult, frame: u64) -> Self {
        Self {
            result,
            frame,
            context: Vec::new(),
        }
    }

    /// Adds context information.
    pub fn add_context(&mut self, msg: impl Into<String>) {
        self.context.push(msg.into());
    }

    /// Creates a report with context.
    #[must_use]
    pub fn with_context(mut self, msg: impl Into<String>) -> Self {
        self.add_context(msg);
        self
    }
}

/// Checker for verifying determinism between replays.
#[derive(Debug, Default)]
pub struct DeterminismChecker {
    /// Whether to check mouse input
    check_mouse: bool,
    /// Whether to check timing
    check_timing: bool,
    /// Tolerance for timing differences in microseconds
    timing_tolerance_us: u64,
}

impl DeterminismChecker {
    /// Creates a new determinism checker with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            check_mouse: true,
            check_timing: false,
            timing_tolerance_us: 1000, // 1ms tolerance
        }
    }

    /// Sets whether to check mouse input.
    pub fn set_check_mouse(&mut self, check: bool) {
        self.check_mouse = check;
    }

    /// Sets whether to check timing.
    pub fn set_check_timing(&mut self, check: bool) {
        self.check_timing = check;
    }

    /// Sets timing tolerance in microseconds.
    pub fn set_timing_tolerance(&mut self, tolerance_us: u64) {
        self.timing_tolerance_us = tolerance_us;
    }

    /// Compares two replays and returns the result.
    ///
    /// Returns `DeterminismResult::Identical` if replays match,
    /// or a specific divergence type if they differ.
    #[must_use]
    pub fn compare_replays(&self, expected: &Replay, actual: &Replay) -> DeterminismResult {
        // Check seeds match
        if expected.seed != actual.seed {
            return DeterminismResult::SeedMismatch {
                expected: expected.seed,
                actual: actual.seed,
            };
        }

        // Check frame counts match
        if expected.frames.len() != actual.frames.len() {
            return DeterminismResult::FrameCountMismatch {
                expected: expected.frames.len() as u64,
                actual: actual.frames.len() as u64,
            };
        }

        // Compare frame by frame
        for (exp_frame, act_frame) in expected.frames.iter().zip(actual.frames.iter()) {
            // Compare inputs
            if exp_frame.inputs != act_frame.inputs {
                return DeterminismResult::InputDiverged {
                    frame: exp_frame.frame,
                    expected_inputs: exp_frame.inputs.clone(),
                    actual_inputs: act_frame.inputs.clone(),
                };
            }

            // Compare mouse if enabled
            if self.check_mouse && !Self::mouse_inputs_match(exp_frame.mouse, act_frame.mouse) {
                return DeterminismResult::MouseDiverged {
                    frame: exp_frame.frame,
                    expected: exp_frame.mouse,
                    actual: act_frame.mouse,
                };
            }
        }

        // Compare state hashes
        if let Some(divergence) = Self::compare_state_hashes(expected, actual) {
            return divergence;
        }

        DeterminismResult::Identical
    }

    /// Compares two replays and returns a detailed report.
    ///
    /// Returns `None` if replays are identical, or a detailed report if they differ.
    #[must_use]
    pub fn compare_replays_detailed(
        &self,
        expected: &Replay,
        actual: &Replay,
    ) -> Option<DivergenceReport> {
        let result = self.compare_replays(expected, actual);

        if result == DeterminismResult::Identical {
            return None;
        }

        let frame = match &result {
            DeterminismResult::Identical | DeterminismResult::SeedMismatch { .. } => 0,
            DeterminismResult::FrameCountMismatch { .. } => {
                expected.frames.len().min(actual.frames.len()) as u64
            },
            DeterminismResult::InputDiverged { frame, .. }
            | DeterminismResult::StateDiverged { frame, .. }
            | DeterminismResult::MouseDiverged { frame, .. } => *frame,
        };

        let mut report = DivergenceReport::new(result, frame);

        // Add context
        report.add_context(format!("Expected replay: {} frames", expected.frames.len()));
        report.add_context(format!("Actual replay: {} frames", actual.frames.len()));
        report.add_context(format!(
            "Expected seed: {}, Actual seed: {}",
            expected.seed, actual.seed
        ));
        report.add_context(format!(
            "State hashes: {} expected, {} actual",
            expected.state_hashes.len(),
            actual.state_hashes.len()
        ));

        Some(report)
    }

    /// Compares mouse inputs with tolerance for floating point.
    fn mouse_inputs_match(a: Option<MouseInput>, b: Option<MouseInput>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
            (Some(ma), Some(mb)) => {
                const EPSILON: f32 = 0.001;
                (ma.x - mb.x).abs() < EPSILON
                    && (ma.y - mb.y).abs() < EPSILON
                    && ma.button == mb.button
                    && ma.pressed == mb.pressed
                    && (ma.scroll_delta - mb.scroll_delta).abs() < EPSILON
            },
        }
    }

    /// Compares state hashes between replays.
    fn compare_state_hashes(expected: &Replay, actual: &Replay) -> Option<DeterminismResult> {
        // Find common frames with hashes
        for exp_hash in &expected.state_hashes {
            if let Some(act_hash) = actual.find_hash_at(exp_hash.frame) {
                if exp_hash.world_hash != act_hash.world_hash
                    || exp_hash.entity_hash != act_hash.entity_hash
                {
                    return Some(DeterminismResult::StateDiverged {
                        frame: exp_hash.frame,
                        expected: *exp_hash,
                        actual: *act_hash,
                    });
                }
            }
        }
        None
    }
}

/// Runs a determinism test by recording two sessions and comparing.
///
/// This is the high-level API for verifying determinism.
#[derive(Debug)]
pub struct DeterminismTest {
    /// The seed to use for both runs
    seed: u32,
    /// First replay (reference)
    reference: Option<Replay>,
    /// Second replay (test)
    test: Option<Replay>,
    /// The checker configuration
    checker: DeterminismChecker,
}

impl DeterminismTest {
    /// Creates a new determinism test with a seed.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            reference: None,
            test: None,
            checker: DeterminismChecker::new(),
        }
    }

    /// Returns the seed for this test.
    #[must_use]
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Sets the reference replay.
    pub fn set_reference(&mut self, replay: Replay) {
        self.reference = Some(replay);
    }

    /// Sets the test replay.
    pub fn set_test(&mut self, replay: Replay) {
        self.test = Some(replay);
    }

    /// Returns the checker for configuration.
    pub fn checker_mut(&mut self) -> &mut DeterminismChecker {
        &mut self.checker
    }

    /// Runs the determinism comparison.
    ///
    /// # Errors
    ///
    /// Returns an error if either replay is missing.
    pub fn run(&self) -> Result<DeterminismResult, &'static str> {
        let reference = self.reference.as_ref().ok_or("Reference replay not set")?;
        let test = self.test.as_ref().ok_or("Test replay not set")?;

        Ok(self.checker.compare_replays(reference, test))
    }

    /// Runs the determinism comparison with detailed report.
    ///
    /// # Errors
    ///
    /// Returns an error if either replay is missing.
    pub fn run_detailed(&self) -> Result<Option<DivergenceReport>, &'static str> {
        let reference = self.reference.as_ref().ok_or("Reference replay not set")?;
        let test = self.test.as_ref().ok_or("Test replay not set")?;

        Ok(self.checker.compare_replays_detailed(reference, test))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_input_frame_creation() {
        let mut frame = InputFrame::new(42);
        assert_eq!(frame.frame, 42);
        assert!(frame.inputs.is_empty());

        frame.add_input(Input::MoveRight);
        frame.add_input(Input::Jump);
        assert_eq!(frame.inputs.len(), 2);
        assert_eq!(frame.inputs[0], Input::MoveRight);
        assert_eq!(frame.inputs[1], Input::Jump);
    }

    #[test]
    fn test_input_frame_mouse() {
        let mut frame = InputFrame::new(0);
        let mouse = MouseInput {
            x: 100.0,
            y: 200.0,
            button: Some(MouseButton::Left),
            pressed: true,
            scroll_delta: 0.0,
        };
        frame.set_mouse(mouse);
        assert!(frame.mouse.is_some());
        assert_eq!(frame.mouse.as_ref().map(|m| m.x), Some(100.0));
    }

    #[test]
    fn test_state_hash_combined() {
        let hash1 = StateHash::new(0, 100, 200);
        let hash2 = StateHash::new(0, 100, 200);
        let hash3 = StateHash::new(0, 100, 201);

        assert_eq!(hash1.combined(), hash2.combined());
        assert_ne!(hash1.combined(), hash3.combined());
    }

    #[test]
    fn test_replay_creation() {
        let replay = Replay::with_seed(12345);
        assert_eq!(replay.seed, 12345);
        assert_eq!(replay.frame_count(), 0);
    }

    #[test]
    fn test_replay_json_roundtrip() {
        let mut replay = Replay::with_seed(42);
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::MoveRight],
            mouse: None,
            delta_time_us: 16667,
        });

        let json = replay.to_json().expect("Failed to serialize");
        let restored = Replay::from_json(&json).expect("Failed to deserialize");

        assert_eq!(restored.seed, 42);
        assert_eq!(restored.frames.len(), 1);
        assert_eq!(restored.frames[0].inputs[0], Input::MoveRight);
    }

    #[test]
    fn test_replay_save_load() {
        let mut replay = Replay::with_seed(99);
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::Jump],
            mouse: None,
            delta_time_us: 16667,
        });
        replay.state_hashes.push(StateHash::new(0, 111, 222));

        let mut buffer = Vec::new();
        replay.save(&mut buffer).expect("Failed to save");

        let cursor = Cursor::new(buffer);
        let loaded = Replay::load(cursor).expect("Failed to load");

        assert_eq!(loaded.seed, 99);
        assert_eq!(loaded.frames.len(), 1);
        assert_eq!(loaded.state_hashes.len(), 1);
        assert_eq!(loaded.state_hashes[0].world_hash, 111);
    }

    #[test]
    fn test_replay_invalid_magic() {
        let bad_data = b"BADM\x01\x00\x00\x00";
        let cursor = Cursor::new(bad_data);
        let result = Replay::load(cursor);
        assert!(matches!(result, Err(ReplayError::InvalidFormat(_))));
    }

    #[test]
    fn test_recorder_basic() {
        let mut recorder = ReplayRecorder::new();
        recorder.set_seed(100);
        recorder.set_metadata("Test Replay", "A test recording");
        recorder.start();

        assert!(recorder.is_recording());
        assert_eq!(recorder.current_frame(), 0);

        recorder.record_input_action(Input::MoveRight);
        recorder.end_frame();

        assert_eq!(recorder.current_frame(), 1);

        recorder.record_input_action(Input::Jump);
        recorder.end_frame();

        let replay = recorder.finish();
        assert_eq!(replay.seed, 100);
        assert_eq!(replay.frames.len(), 2);
        assert_eq!(replay.metadata.name, "Test Replay");
        assert_eq!(replay.metadata.frame_count, 2);
    }

    #[test]
    fn test_recorder_mouse_input() {
        let config = RecordingConfig {
            capture_mouse: true,
            ..Default::default()
        };
        let mut recorder = ReplayRecorder::with_config(config);
        recorder.start();

        let mouse = MouseInput {
            x: 50.0,
            y: 75.0,
            button: Some(MouseButton::Right),
            pressed: true,
            scroll_delta: -1.0,
        };
        recorder.record_mouse(mouse);
        recorder.end_frame();

        let replay = recorder.finish();
        assert!(replay.frames[0].mouse.is_some());
        let recorded_mouse = replay.frames[0].mouse.as_ref();
        assert_eq!(recorded_mouse.map(|m| m.x), Some(50.0));
        assert_eq!(
            recorded_mouse.map(|m| m.button),
            Some(Some(MouseButton::Right))
        );
    }

    #[test]
    fn test_recorder_state_hashes() {
        let config = RecordingConfig {
            hash_interval: 2, // Hash every 2 frames
            ..Default::default()
        };
        let mut recorder = ReplayRecorder::with_config(config);
        recorder.start();

        // Frame 0 - should capture hash
        recorder.record_state_hash(100, 200);
        recorder.end_frame();

        // Frame 1 - should NOT capture hash
        recorder.record_state_hash(101, 201);
        recorder.end_frame();

        // Frame 2 - should capture hash
        recorder.record_state_hash(102, 202);
        recorder.end_frame();

        let replay = recorder.finish();
        assert_eq!(replay.state_hashes.len(), 2);
        assert_eq!(replay.state_hashes[0].frame, 0);
        assert_eq!(replay.state_hashes[1].frame, 2);
    }

    #[test]
    fn test_recorder_max_frames() {
        let config = RecordingConfig {
            max_frames: 3,
            ..Default::default()
        };
        let mut recorder = ReplayRecorder::with_config(config);
        recorder.start();

        for _ in 0..10 {
            recorder.record_input_action(Input::MoveRight);
            recorder.end_frame();
        }

        let replay = recorder.finish();
        assert_eq!(replay.frames.len(), 3);
    }

    #[test]
    fn test_player_basic() {
        let mut replay = Replay::with_seed(42);
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::MoveRight],
            mouse: None,
            delta_time_us: 16667,
        });
        replay.frames.push(InputFrame {
            frame: 1,
            inputs: vec![Input::Jump],
            mouse: None,
            delta_time_us: 16667,
        });

        let mut player = ReplayPlayer::new(replay);
        assert_eq!(player.seed(), 42);
        assert_eq!(player.frame_count(), 2);
        assert!(!player.is_complete());

        let frame0 = player.next_frame();
        assert!(frame0.is_some());
        assert_eq!(frame0.map(|f| f.frame), Some(0));

        let frame1 = player.next_frame();
        assert!(frame1.is_some());
        assert_eq!(frame1.map(|f| f.frame), Some(1));

        assert!(player.is_complete());
        assert!(player.next_frame().is_none());
    }

    #[test]
    fn test_player_seek() {
        let mut replay = Replay::new();
        for i in 0..5 {
            replay.frames.push(InputFrame {
                frame: i,
                inputs: vec![],
                mouse: None,
                delta_time_us: 16667,
            });
        }

        let mut player = ReplayPlayer::new(replay);

        player.seek(3);
        assert_eq!(player.current_index(), 3);
        assert_eq!(player.next_frame().map(|f| f.frame), Some(3));

        player.seek_start();
        assert_eq!(player.current_index(), 0);

        player.seek_end();
        assert!(player.is_complete());
    }

    #[test]
    fn test_player_pause() {
        let mut replay = Replay::new();
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: None,
            delta_time_us: 16667,
        });

        let mut player = ReplayPlayer::new(replay);
        player.pause();
        assert!(player.is_paused());

        // Should return None while paused
        assert!(player.next_frame().is_none());

        player.resume();
        assert!(!player.is_paused());
        assert!(player.next_frame().is_some());
    }

    #[test]
    fn test_player_verify_state() {
        let mut replay = Replay::new();
        replay.state_hashes.push(StateHash::new(0, 100, 200));

        let player = ReplayPlayer::new(replay);

        // Matching hash should succeed
        assert!(player.verify_state(0, 100, 200).is_ok());

        // Mismatching hash should fail
        let result = player.verify_state(0, 100, 201);
        assert!(matches!(
            result,
            Err(ReplayError::DeterminismFailure { .. })
        ));

        // Frame without hash should succeed (nothing to verify)
        assert!(player.verify_state(1, 999, 999).is_ok());
    }

    #[test]
    fn test_replay_find_hash() {
        let mut replay = Replay::new();
        replay.state_hashes.push(StateHash::new(0, 10, 20));
        replay.state_hashes.push(StateHash::new(60, 11, 21));
        replay.state_hashes.push(StateHash::new(120, 12, 22));

        assert!(replay.find_hash_at(0).is_some());
        assert!(replay.find_hash_at(30).is_none());
        assert!(replay.find_hash_at(60).is_some());

        // Find nearest hash before frame 100
        let nearest = replay.find_hash_before(100);
        assert!(nearest.is_some());
        assert_eq!(nearest.map(|h| h.frame), Some(60));
    }

    #[test]
    fn test_error_display() {
        let err = ReplayError::DeterminismFailure {
            frame: 100,
            expected: 0xAABB,
            actual: 0xCCDD,
        };
        let msg = err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("Determinism"));
    }

    #[test]
    fn test_record_1000_frames_playback_identical() {
        // ACCEPTANCE CRITERIA: Record 1000 frames, play back identically
        let config = RecordingConfig {
            hash_interval: 100, // Hash every 100 frames
            capture_mouse: true,
            capture_timing: false, // Use fixed timing for determinism
            max_frames: 0,
        };

        let mut recorder = ReplayRecorder::with_config(config);
        recorder.set_seed(42);
        recorder.set_metadata("1000 Frame Test", "Acceptance criteria test");
        recorder.start();

        // Define a deterministic sequence of inputs
        let input_sequence = [
            Input::MoveRight,
            Input::MoveRight,
            Input::Jump,
            Input::MoveLeft,
            Input::Primary,
            Input::MoveUp,
            Input::MoveDown,
            Input::Secondary,
            Input::Inventory,
            Input::Map,
        ];

        // Record 1000 frames with varying inputs
        for i in 0..1000u64 {
            // Add inputs based on frame number for variety
            let input_idx = (i % input_sequence.len() as u64) as usize;
            recorder.record_input_action(input_sequence[input_idx]);

            // Add mouse movement on some frames
            if i % 3 == 0 {
                recorder.record_mouse(MouseInput {
                    x: (i as f32) % 1920.0,
                    y: ((i * 2) as f32) % 1080.0,
                    button: if i % 7 == 0 {
                        Some(MouseButton::Left)
                    } else {
                        None
                    },
                    pressed: i % 7 == 0,
                    scroll_delta: if i % 11 == 0 { 1.0 } else { 0.0 },
                });
            }

            // Record deterministic state hash
            let world_hash = i.wrapping_mul(0x517cc1b727220a95);
            let entity_hash = i.wrapping_mul(0x2545f4914f6cdd1d);
            recorder.record_state_hash(world_hash, entity_hash);

            recorder.end_frame();
        }

        let replay = recorder.finish();

        // Verify recording
        assert_eq!(replay.frames.len(), 1000);
        assert_eq!(replay.metadata.frame_count, 1000);
        assert_eq!(replay.seed, 42);
        assert_eq!(replay.state_hashes.len(), 10); // 0, 100, 200, ... 900

        // Test save/load roundtrip
        let mut buffer = Vec::new();
        replay.save(&mut buffer).expect("save failed");

        let loaded = Replay::load(Cursor::new(&buffer)).expect("load failed");
        assert_eq!(loaded.frames.len(), 1000);
        assert_eq!(loaded.seed, 42);

        // Play back and verify EVERY frame matches exactly
        let mut player = ReplayPlayer::new(loaded);
        assert_eq!(player.frame_count(), 1000);
        assert_eq!(player.seed(), 42);

        for i in 0..1000u64 {
            let frame = player.next_frame();
            assert!(frame.is_some(), "Frame {i} should exist");
            let frame = frame.expect("frame exists");

            // Verify frame number
            assert_eq!(frame.frame, i, "Frame number mismatch at {i}");

            // Verify input matches
            let input_idx = (i % input_sequence.len() as u64) as usize;
            assert_eq!(
                frame.inputs.len(),
                1,
                "Should have exactly 1 input at frame {i}"
            );
            assert_eq!(
                frame.inputs[0], input_sequence[input_idx],
                "Input mismatch at frame {i}"
            );

            // Verify mouse on frames that had it
            if i % 3 == 0 {
                assert!(frame.mouse.is_some(), "Mouse should exist at frame {i}");
                let mouse = frame.mouse.as_ref().expect("mouse exists");
                assert!(
                    (mouse.x - (i as f32) % 1920.0).abs() < 0.001,
                    "Mouse X mismatch at frame {i}"
                );
            }
        }

        assert!(player.is_complete());
        assert!(player.next_frame().is_none());
    }

    #[test]
    fn test_player_step_forward_backward() {
        let mut replay = Replay::new();
        for i in 0..5 {
            replay.frames.push(InputFrame {
                frame: i,
                inputs: vec![],
                mouse: None,
                delta_time_us: 16667,
            });
        }

        let mut player = ReplayPlayer::new(replay);

        // Step forward
        assert_eq!(player.step_forward().map(|f| f.frame), Some(0));
        assert_eq!(player.step_forward().map(|f| f.frame), Some(1));
        assert_eq!(player.current_index(), 2);

        // Step backward
        assert_eq!(player.step_backward().map(|f| f.frame), Some(1));
        assert_eq!(player.current_index(), 1);
        assert_eq!(player.step_backward().map(|f| f.frame), Some(0));
        assert_eq!(player.current_index(), 0);

        // Can't step backward at start
        assert!(player.step_backward().is_none());
        assert_eq!(player.current_index(), 0);
    }

    #[test]
    fn test_player_progress_and_time() {
        let mut replay = Replay::new();
        for i in 0..4 {
            replay.frames.push(InputFrame {
                frame: i,
                inputs: vec![],
                mouse: None,
                delta_time_us: 10000, // 10ms each
            });
        }

        let mut player = ReplayPlayer::new(replay);
        assert!((player.progress() - 0.0).abs() < 0.001);
        assert_eq!(player.elapsed_time_us(), 0);
        assert_eq!(player.remaining_time_us(), 40000);

        player.seek(2);
        assert!((player.progress() - 0.5).abs() < 0.001);
        assert_eq!(player.elapsed_time_us(), 20000);
        assert_eq!(player.remaining_time_us(), 20000);

        player.seek_end();
        assert!((player.progress() - 1.0).abs() < 0.001);
        assert_eq!(player.elapsed_time_us(), 40000);
        assert_eq!(player.remaining_time_us(), 0);
    }

    #[test]
    fn test_verified_playback_session() {
        let mut replay = Replay::new();
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::Jump],
            mouse: None,
            delta_time_us: 16667,
        });
        replay.frames.push(InputFrame {
            frame: 1,
            inputs: vec![Input::MoveRight],
            mouse: None,
            delta_time_us: 16667,
        });
        replay.state_hashes.push(StateHash::new(0, 100, 200));

        let mut session = VerifiedPlaybackSession::new(replay);

        // First frame with correct hash
        let result = session.next_frame_verified(100, 200);
        assert!(result.is_ok());
        assert!(result.expect("ok").is_some());

        // Second frame (no hash to verify)
        let result = session.next_frame_verified(999, 999);
        assert!(result.is_ok());

        assert!(!session.has_errors());
    }

    #[test]
    fn test_verified_playback_session_error() {
        let mut replay = Replay::new();
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: None,
            delta_time_us: 16667,
        });
        replay.state_hashes.push(StateHash::new(0, 100, 200));

        let mut session = VerifiedPlaybackSession::new(replay);
        session.set_stop_on_error(false);

        // Wrong hash - should record error but continue
        let _ = session.next_frame_verified(999, 999);
        assert!(session.has_errors());
        assert_eq!(session.verification_errors().len(), 1);
    }

    #[test]
    fn test_timed_playback() {
        let mut replay = Replay::new();
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::Jump],
            mouse: None,
            delta_time_us: 10000, // 10ms
        });
        replay.frames.push(InputFrame {
            frame: 1,
            inputs: vec![Input::MoveRight],
            mouse: None,
            delta_time_us: 10000, // 10ms
        });

        let mut player = ReplayPlayer::new(replay);

        // Not enough time elapsed - should return None
        assert!(player.next_frame_timed(5000).is_none());

        // Now enough time - should return frame 0
        let frame = player.next_frame_timed(6000);
        assert!(frame.is_some());
        assert_eq!(frame.map(|f| f.frame), Some(0));

        // Need more time for frame 1
        assert!(player.next_frame_timed(5000).is_none());
        let frame = player.next_frame_timed(10000);
        assert!(frame.is_some());
        assert_eq!(frame.map(|f| f.frame), Some(1));
    }

    #[test]
    fn test_playback_speed() {
        let mut replay = Replay::new();
        replay.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: None,
            delta_time_us: 10000, // 10ms
        });

        let mut player = ReplayPlayer::new(replay);
        player.set_speed(2.0); // 2x speed

        // At 2x speed, 5000us of real time = 10000us of replay time
        let frame = player.next_frame_timed(5000);
        assert!(frame.is_some());
    }

    // ========================================================================
    // T-3: Determinism Verification Tests
    // ========================================================================

    #[test]
    fn test_determinism_identical_replays() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::MoveRight],
            mouse: None,
            delta_time_us: 16667,
        });
        replay1.frames.push(InputFrame {
            frame: 1,
            inputs: vec![Input::Jump],
            mouse: None,
            delta_time_us: 16667,
        });
        replay1.state_hashes.push(StateHash::new(0, 100, 200));

        let replay2 = replay1.clone();

        let checker = DeterminismChecker::new();
        let result = checker.compare_replays(&replay1, &replay2);
        assert_eq!(result, DeterminismResult::Identical);
    }

    #[test]
    fn test_determinism_seed_mismatch() {
        let replay1 = Replay::with_seed(42);
        let replay2 = Replay::with_seed(99);

        let checker = DeterminismChecker::new();
        let result = checker.compare_replays(&replay1, &replay2);

        assert!(matches!(
            result,
            DeterminismResult::SeedMismatch {
                expected: 42,
                actual: 99
            }
        ));
    }

    #[test]
    fn test_determinism_frame_count_mismatch() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: None,
            delta_time_us: 16667,
        });
        replay1.frames.push(InputFrame {
            frame: 1,
            inputs: vec![],
            mouse: None,
            delta_time_us: 16667,
        });

        let mut replay2 = Replay::with_seed(42);
        replay2.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: None,
            delta_time_us: 16667,
        });

        let checker = DeterminismChecker::new();
        let result = checker.compare_replays(&replay1, &replay2);

        assert!(matches!(
            result,
            DeterminismResult::FrameCountMismatch {
                expected: 2,
                actual: 1
            }
        ));
    }

    #[test]
    fn test_determinism_input_divergence() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::MoveRight],
            mouse: None,
            delta_time_us: 16667,
        });

        let mut replay2 = Replay::with_seed(42);
        replay2.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::MoveLeft],
            mouse: None,
            delta_time_us: 16667,
        });

        let checker = DeterminismChecker::new();
        let result = checker.compare_replays(&replay1, &replay2);

        match result {
            DeterminismResult::InputDiverged {
                frame,
                expected_inputs,
                actual_inputs,
            } => {
                assert_eq!(frame, 0);
                assert_eq!(expected_inputs, vec![Input::MoveRight]);
                assert_eq!(actual_inputs, vec![Input::MoveLeft]);
            },
            _ => panic!("Expected InputDiverged"),
        }
    }

    #[test]
    fn test_determinism_state_hash_divergence() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: None,
            delta_time_us: 16667,
        });
        replay1.state_hashes.push(StateHash::new(0, 100, 200));

        let mut replay2 = Replay::with_seed(42);
        replay2.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: None,
            delta_time_us: 16667,
        });
        replay2.state_hashes.push(StateHash::new(0, 100, 201)); // Different entity hash

        let checker = DeterminismChecker::new();
        let result = checker.compare_replays(&replay1, &replay2);

        match result {
            DeterminismResult::StateDiverged {
                frame,
                expected,
                actual,
            } => {
                assert_eq!(frame, 0);
                assert_eq!(expected.entity_hash, 200);
                assert_eq!(actual.entity_hash, 201);
            },
            _ => panic!("Expected StateDiverged"),
        }
    }

    #[test]
    fn test_determinism_mouse_divergence() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: Some(MouseInput {
                x: 100.0,
                y: 200.0,
                button: Some(MouseButton::Left),
                pressed: true,
                scroll_delta: 0.0,
            }),
            delta_time_us: 16667,
        });

        let mut replay2 = Replay::with_seed(42);
        replay2.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: Some(MouseInput {
                x: 150.0, // Different position
                y: 200.0,
                button: Some(MouseButton::Left),
                pressed: true,
                scroll_delta: 0.0,
            }),
            delta_time_us: 16667,
        });

        let checker = DeterminismChecker::new();
        let result = checker.compare_replays(&replay1, &replay2);

        assert!(matches!(
            result,
            DeterminismResult::MouseDiverged { frame: 0, .. }
        ));
    }

    #[test]
    fn test_determinism_detailed_report() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::Jump],
            mouse: None,
            delta_time_us: 16667,
        });

        let mut replay2 = Replay::with_seed(42);
        replay2.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::Primary],
            mouse: None,
            delta_time_us: 16667,
        });

        let checker = DeterminismChecker::new();
        let report = checker.compare_replays_detailed(&replay1, &replay2);

        assert!(report.is_some());
        let report = report.expect("report exists");
        assert_eq!(report.frame, 0);
        assert!(!report.context.is_empty());
    }

    #[test]
    fn test_determinism_test_runner() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![Input::MoveRight],
            mouse: None,
            delta_time_us: 16667,
        });

        let replay2 = replay1.clone();

        let mut test = DeterminismTest::new(42);
        test.set_reference(replay1);
        test.set_test(replay2);

        let result = test.run();
        assert!(result.is_ok());
        assert_eq!(result.expect("ok"), DeterminismResult::Identical);
    }

    #[test]
    fn test_determinism_result_display() {
        let result = DeterminismResult::StateDiverged {
            frame: 100,
            expected: StateHash::new(100, 0xAABB, 0xCCDD),
            actual: StateHash::new(100, 0xAABB, 0xEEFF),
        };

        let display = result.to_string();
        assert!(display.contains("100"));
        assert!(display.contains("State divergence"));
    }

    #[test]
    fn test_determinism_ignore_mouse_when_disabled() {
        let mut replay1 = Replay::with_seed(42);
        replay1.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: Some(MouseInput {
                x: 100.0,
                y: 200.0,
                button: None,
                pressed: false,
                scroll_delta: 0.0,
            }),
            delta_time_us: 16667,
        });

        let mut replay2 = Replay::with_seed(42);
        replay2.frames.push(InputFrame {
            frame: 0,
            inputs: vec![],
            mouse: Some(MouseInput {
                x: 999.0, // Very different
                y: 999.0,
                button: None,
                pressed: false,
                scroll_delta: 0.0,
            }),
            delta_time_us: 16667,
        });

        let mut checker = DeterminismChecker::new();
        checker.set_check_mouse(false);

        let result = checker.compare_replays(&replay1, &replay2);
        assert_eq!(result, DeterminismResult::Identical);
    }
}
