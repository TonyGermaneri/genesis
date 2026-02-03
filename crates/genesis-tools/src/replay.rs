//! Replay and determinism system.

use serde::{Deserialize, Serialize};

/// Input action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// A single frame of input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFrame {
    /// Frame number
    pub frame: u64,
    /// Inputs active this frame
    pub inputs: Vec<Input>,
}

/// A complete replay recording.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Replay {
    /// World seed used
    pub seed: u32,
    /// Starting timestamp
    pub timestamp: u64,
    /// All recorded frames
    pub frames: Vec<InputFrame>,
}

/// Records gameplay for replay.
#[derive(Debug, Default)]
pub struct ReplayRecorder {
    /// Current replay being recorded
    replay: Replay,
}

impl ReplayRecorder {
    /// Creates a new recorder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the world seed.
    pub fn set_seed(&mut self, seed: u32) {
        self.replay.seed = seed;
    }

    /// Records an input frame.
    pub fn record_input(&mut self, frame: InputFrame) {
        self.replay.frames.push(frame);
    }

    /// Finishes recording and returns the replay.
    #[must_use]
    pub fn finish(self) -> Replay {
        self.replay
    }
}

/// Plays back a replay.
#[derive(Debug)]
pub struct ReplayPlayer {
    /// Replay being played
    replay: Replay,
    /// Current frame index
    index: usize,
}

impl ReplayPlayer {
    /// Creates a new player for the given replay.
    #[must_use]
    pub fn new(replay: Replay) -> Self {
        Self { replay, index: 0 }
    }

    /// Returns the world seed.
    #[must_use]
    pub fn seed(&self) -> u32 {
        self.replay.seed
    }

    /// Returns the next input frame, if any.
    pub fn next_frame(&mut self) -> Option<&InputFrame> {
        if self.index < self.replay.frames.len() {
            let frame = &self.replay.frames[self.index];
            self.index += 1;
            Some(frame)
        } else {
            None
        }
    }

    /// Checks if playback is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.index >= self.replay.frames.len()
    }

    /// Resets playback to the beginning.
    pub fn reset(&mut self) {
        self.index = 0;
    }
}
