//! # Genesis Tools
//!
//! Development tools for Project Genesis.
//!
//! This crate provides:
//! - Replay/determinism harness
//! - Chunk viewer (egui)
//! - Cell inspector probe
//! - Performance HUD

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod cell_inspector;
pub mod chunk_viewer;
pub mod inspector;
pub mod perf;
pub mod perf_hud;
pub mod replay;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::cell_inspector::*;
    pub use crate::chunk_viewer::*;
    pub use crate::inspector::*;
    pub use crate::perf::*;
    pub use crate::perf_hud::*;
    pub use crate::replay::*;
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_record_playback() {
        let mut recorder = ReplayRecorder::new();
        recorder.set_seed(42);
        recorder.start();

        recorder.record_input_action(Input::MoveRight);
        recorder.end_frame();

        recorder.record_input_action(Input::Jump);
        recorder.end_frame();

        let replay = recorder.finish();
        assert_eq!(replay.frames.len(), 2);
        assert_eq!(replay.seed, 42);

        let mut player = ReplayPlayer::new(replay);
        let frame0 = player.next_frame();
        assert!(frame0.is_some());
        assert_eq!(frame0.map(|f| f.frame), Some(0));
        assert_eq!(
            frame0.map(|f| f.inputs.first().copied()),
            Some(Some(Input::MoveRight))
        );
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
