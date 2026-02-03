//! # Genesis Tools
//!
//! Development tools for Project Genesis.
//!
//! This crate provides:
//! - Replay/determinism harness
//! - Chunk viewer
//! - Cell inspector
//! - Performance HUD

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod inspector;
pub mod perf;
pub mod replay;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::inspector::*;
    pub use crate::perf::*;
    pub use crate::replay::*;
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_record_playback() {
        let mut recorder = ReplayRecorder::new();
        recorder.record_input(InputFrame {
            frame: 0,
            inputs: vec![Input::MoveRight],
        });
        recorder.record_input(InputFrame {
            frame: 1,
            inputs: vec![Input::Jump],
        });

        let replay = recorder.finish();
        assert_eq!(replay.frames.len(), 2);

        let mut player = ReplayPlayer::new(replay);
        let frame0 = player.next_frame();
        assert!(frame0.is_some());
        assert_eq!(frame0.as_ref().map(|f| f.frame), Some(0));
    }

    #[test]
    fn test_perf_tracker() {
        let mut tracker = PerfTracker::new(60);
        for _ in 0..100 {
            tracker.frame_start();
            tracker.frame_end();
        }
        assert!(tracker.average_fps() > 0.0);
    }
}
