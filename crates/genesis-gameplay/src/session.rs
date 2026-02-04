//! Game session management.
//!
//! This module handles overall game session state:
//! - New game creation
//! - Continue from last save
//! - Load specific save slot
//! - Return to main menu

use serde::{Deserialize, Serialize};
use std::time::Instant;

// ============================================================================
// G-57: Session State
// ============================================================================

/// Current session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionState {
    /// At the main menu.
    MainMenu,
    /// Creating a new game.
    NewGame,
    /// Loading a saved game.
    Loading,
    /// Game is actively being played.
    Playing,
    /// Game is paused.
    Paused,
    /// Saving the game.
    Saving,
    /// Returning to main menu.
    ExitingToMenu,
    /// Quitting the game entirely.
    Quitting,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::MainMenu
    }
}

impl SessionState {
    /// Check if game world is active (playing or paused).
    #[must_use]
    pub fn is_world_active(&self) -> bool {
        matches!(self, Self::Playing | Self::Paused | Self::Saving)
    }

    /// Check if game is in a menu state.
    #[must_use]
    pub fn is_in_menu(&self) -> bool {
        matches!(self, Self::MainMenu | Self::NewGame | Self::Loading)
    }

    /// Check if game should process world updates.
    #[must_use]
    pub fn should_update_world(&self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Check if game should render world.
    #[must_use]
    pub fn should_render_world(&self) -> bool {
        matches!(self, Self::Playing | Self::Paused | Self::Saving)
    }

    /// Check if transitioning between states.
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
        matches!(
            self,
            Self::Loading | Self::Saving | Self::ExitingToMenu | Self::Quitting
        )
    }
}

// ============================================================================
// G-57: Save Slot
// ============================================================================

/// Save slot identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SaveSlot(u32);

impl SaveSlot {
    /// Create a new save slot.
    #[must_use]
    pub const fn new(slot: u32) -> Self {
        Self(slot)
    }

    /// Get the slot number.
    #[must_use]
    pub const fn number(&self) -> u32 {
        self.0
    }

    /// Quick save slot.
    pub const QUICKSAVE: Self = Self(0);

    /// Auto save slot.
    pub const AUTOSAVE: Self = Self(1);

    /// First manual save slot.
    pub const MANUAL_START: Self = Self(2);

    /// Maximum number of manual save slots.
    pub const MAX_MANUAL_SLOTS: u32 = 10;

    /// Check if this is the quicksave slot.
    #[must_use]
    pub const fn is_quicksave(&self) -> bool {
        self.0 == 0
    }

    /// Check if this is the autosave slot.
    #[must_use]
    pub const fn is_autosave(&self) -> bool {
        self.0 == 1
    }

    /// Check if this is a manual save slot.
    #[must_use]
    pub const fn is_manual(&self) -> bool {
        self.0 >= 2
    }

    /// Get save filename.
    #[must_use]
    pub fn filename(&self) -> String {
        match self.0 {
            0 => "quicksave.sav".to_string(),
            1 => "autosave.sav".to_string(),
            n => format!("save_{}.sav", n - 1),
        }
    }
}

impl Default for SaveSlot {
    fn default() -> Self {
        Self::MANUAL_START
    }
}

// ============================================================================
// G-57: Session Action
// ============================================================================

/// Actions that can be performed on a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionAction {
    /// Start a new game.
    NewGame {
        /// World name.
        world_name: String,
        /// World seed.
        seed: Option<u64>,
    },
    /// Continue from the most recent save.
    Continue,
    /// Load a specific save slot.
    LoadSave(SaveSlot),
    /// Save to a specific slot.
    SaveGame(SaveSlot),
    /// Quick save.
    QuickSave,
    /// Quick load.
    QuickLoad,
    /// Pause the game.
    Pause,
    /// Resume the game.
    Resume,
    /// Return to main menu.
    ReturnToMenu,
    /// Quit the game.
    Quit,
}

impl SessionAction {
    /// Check if action requires confirmation.
    #[must_use]
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::ReturnToMenu | Self::Quit | Self::NewGame { .. })
    }

    /// Get confirmation message.
    #[must_use]
    pub fn confirmation_message(&self) -> Option<&'static str> {
        match self {
            Self::ReturnToMenu => Some("Return to main menu? Unsaved progress will be lost."),
            Self::Quit => Some("Quit game? Unsaved progress will be lost."),
            Self::NewGame { .. } => Some("Start a new game? Current progress will be lost."),
            _ => None,
        }
    }
}

// ============================================================================
// G-57: Session Event
// ============================================================================

/// Events emitted by the session system.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    /// Session state changed.
    StateChanged {
        /// Previous state.
        from: SessionState,
        /// New state.
        to: SessionState,
    },
    /// Game was saved.
    GameSaved {
        /// Save slot used.
        slot: SaveSlot,
        /// Whether save was successful.
        success: bool,
    },
    /// Game was loaded.
    GameLoaded {
        /// Save slot loaded.
        slot: SaveSlot,
        /// Whether load was successful.
        success: bool,
    },
    /// New game started.
    NewGameStarted {
        /// World name.
        world_name: String,
        /// World seed.
        seed: u64,
    },
    /// Error occurred.
    Error {
        /// Error message.
        message: String,
    },
}

// ============================================================================
// G-57: Session Result
// ============================================================================

/// Result type for session operations.
pub type SessionResult<T> = Result<T, SessionError>;

/// Session operation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    /// Invalid state transition.
    InvalidTransition {
        /// Current state.
        from: SessionState,
        /// Attempted state.
        to: SessionState,
    },
    /// Save slot not found.
    SaveNotFound(SaveSlot),
    /// Save operation failed.
    SaveFailed(String),
    /// Load operation failed.
    LoadFailed(String),
    /// No saves available to continue.
    NoSavesAvailable,
    /// Operation cancelled.
    Cancelled,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(f, "Invalid transition from {from:?} to {to:?}")
            },
            Self::SaveNotFound(slot) => write!(f, "Save not found: {}", slot.filename()),
            Self::SaveFailed(msg) => write!(f, "Save failed: {msg}"),
            Self::LoadFailed(msg) => write!(f, "Load failed: {msg}"),
            Self::NoSavesAvailable => write!(f, "No saves available"),
            Self::Cancelled => write!(f, "Operation cancelled"),
        }
    }
}

impl std::error::Error for SessionError {}

// ============================================================================
// G-57: Session Manager
// ============================================================================

/// Manages game session state and transitions.
#[derive(Debug)]
pub struct SessionManager {
    /// Current session state.
    state: SessionState,
    /// Previous state (for resume).
    previous_state: Option<SessionState>,
    /// Current save slot (if loaded from save).
    current_slot: Option<SaveSlot>,
    /// Most recent save slot.
    last_save_slot: Option<SaveSlot>,
    /// Session start time.
    session_start: Option<Instant>,
    /// Total play time in seconds.
    total_play_time: f64,
    /// Current world name.
    world_name: Option<String>,
    /// Current world seed.
    world_seed: Option<u64>,
    /// Pending events.
    events: Vec<SessionEvent>,
    /// Auto-save enabled.
    auto_save_enabled: bool,
    /// Auto-save interval in seconds.
    auto_save_interval: f64,
    /// Time since last auto-save.
    time_since_auto_save: f64,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Create a new session manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: SessionState::MainMenu,
            previous_state: None,
            current_slot: None,
            last_save_slot: None,
            session_start: None,
            total_play_time: 0.0,
            world_name: None,
            world_seed: None,
            events: Vec::new(),
            auto_save_enabled: true,
            auto_save_interval: 300.0, // 5 minutes
            time_since_auto_save: 0.0,
        }
    }

    /// Get current session state.
    #[must_use]
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Get current world name.
    #[must_use]
    pub fn world_name(&self) -> Option<&str> {
        self.world_name.as_deref()
    }

    /// Get current world seed.
    #[must_use]
    pub fn world_seed(&self) -> Option<u64> {
        self.world_seed
    }

    /// Get current save slot.
    #[must_use]
    pub fn current_slot(&self) -> Option<SaveSlot> {
        self.current_slot
    }

    /// Get total play time.
    #[must_use]
    pub fn total_play_time(&self) -> f64 {
        self.total_play_time
    }

    /// Check if auto-save is enabled.
    #[must_use]
    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save_enabled
    }

    /// Set auto-save enabled.
    pub fn set_auto_save_enabled(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }

    /// Set auto-save interval.
    pub fn set_auto_save_interval(&mut self, seconds: f64) {
        self.auto_save_interval = seconds.max(30.0); // Minimum 30 seconds
    }

    /// Take pending events.
    pub fn take_events(&mut self) -> Vec<SessionEvent> {
        std::mem::take(&mut self.events)
    }

    /// Start a new game.
    pub fn start_new_game(&mut self, world_name: String, seed: u64) -> SessionResult<()> {
        self.transition_to(SessionState::NewGame)?;

        self.world_name = Some(world_name.clone());
        self.world_seed = Some(seed);
        self.current_slot = None;
        self.session_start = Some(Instant::now());
        self.time_since_auto_save = 0.0;

        self.events
            .push(SessionEvent::NewGameStarted { world_name, seed });

        // Immediately transition to playing
        self.transition_to(SessionState::Playing)?;

        Ok(())
    }

    /// Load a save slot.
    pub fn load_save(&mut self, slot: SaveSlot) -> SessionResult<()> {
        self.transition_to(SessionState::Loading)?;
        self.current_slot = Some(slot);
        self.last_save_slot = Some(slot);
        self.session_start = Some(Instant::now());
        self.time_since_auto_save = 0.0;
        Ok(())
    }

    /// Mark load as complete.
    pub fn load_complete(&mut self, success: bool) -> SessionResult<()> {
        let slot = self.current_slot.unwrap_or(SaveSlot::MANUAL_START);

        self.events.push(SessionEvent::GameLoaded { slot, success });

        if success {
            self.transition_to(SessionState::Playing)?;
        } else {
            self.transition_to(SessionState::MainMenu)?;
            self.current_slot = None;
        }

        Ok(())
    }

    /// Continue from last save.
    pub fn continue_game(&mut self) -> SessionResult<()> {
        if let Some(slot) = self.last_save_slot {
            self.load_save(slot)
        } else {
            Err(SessionError::NoSavesAvailable)
        }
    }

    /// Save to a slot.
    pub fn save_game(&mut self, slot: SaveSlot) -> SessionResult<()> {
        if !self.state.is_world_active() {
            return Err(SessionError::InvalidTransition {
                from: self.state,
                to: SessionState::Saving,
            });
        }

        self.previous_state = Some(self.state);
        self.state = SessionState::Saving;
        self.current_slot = Some(slot);
        self.last_save_slot = Some(slot);

        Ok(())
    }

    /// Mark save as complete.
    pub fn save_complete(&mut self, success: bool) -> SessionResult<()> {
        let slot = self.current_slot.unwrap_or(SaveSlot::MANUAL_START);

        self.events.push(SessionEvent::GameSaved { slot, success });

        if success {
            self.time_since_auto_save = 0.0;
        }

        // Return to previous state
        if let Some(prev) = self.previous_state.take() {
            self.state = prev;
        } else {
            self.state = SessionState::Playing;
        }

        Ok(())
    }

    /// Quick save.
    pub fn quick_save(&mut self) -> SessionResult<()> {
        self.save_game(SaveSlot::QUICKSAVE)
    }

    /// Quick load.
    pub fn quick_load(&mut self) -> SessionResult<()> {
        self.load_save(SaveSlot::QUICKSAVE)
    }

    /// Pause the game.
    pub fn pause(&mut self) -> SessionResult<()> {
        if self.state == SessionState::Playing {
            self.transition_to(SessionState::Paused)
        } else {
            Err(SessionError::InvalidTransition {
                from: self.state,
                to: SessionState::Paused,
            })
        }
    }

    /// Resume the game.
    pub fn resume(&mut self) -> SessionResult<()> {
        if self.state == SessionState::Paused {
            self.transition_to(SessionState::Playing)
        } else {
            Err(SessionError::InvalidTransition {
                from: self.state,
                to: SessionState::Playing,
            })
        }
    }

    /// Toggle pause.
    pub fn toggle_pause(&mut self) -> SessionResult<()> {
        match self.state {
            SessionState::Playing => self.pause(),
            SessionState::Paused => self.resume(),
            _ => Err(SessionError::InvalidTransition {
                from: self.state,
                to: SessionState::Paused,
            }),
        }
    }

    /// Return to main menu.
    pub fn return_to_menu(&mut self) -> SessionResult<()> {
        self.transition_to(SessionState::ExitingToMenu)?;

        // Clean up session
        self.world_name = None;
        self.world_seed = None;
        self.current_slot = None;

        if let Some(start) = self.session_start.take() {
            self.total_play_time += start.elapsed().as_secs_f64();
        }

        self.transition_to(SessionState::MainMenu)?;

        Ok(())
    }

    /// Quit the game.
    pub fn quit(&mut self) -> SessionResult<()> {
        self.transition_to(SessionState::Quitting)
    }

    /// Update session (call each frame).
    pub fn update(&mut self, delta_time: f64) {
        // Track auto-save timing
        if self.state == SessionState::Playing && self.auto_save_enabled {
            self.time_since_auto_save += delta_time;
        }
    }

    /// Check if auto-save is needed.
    #[must_use]
    pub fn should_auto_save(&self) -> bool {
        self.state == SessionState::Playing
            && self.auto_save_enabled
            && self.time_since_auto_save >= self.auto_save_interval
    }

    /// Trigger auto-save.
    pub fn auto_save(&mut self) -> SessionResult<()> {
        self.save_game(SaveSlot::AUTOSAVE)
    }

    /// Transition to a new state.
    fn transition_to(&mut self, new_state: SessionState) -> SessionResult<()> {
        let old_state = self.state;

        // Validate transition
        if !Self::is_valid_transition(old_state, new_state) {
            return Err(SessionError::InvalidTransition {
                from: old_state,
                to: new_state,
            });
        }

        self.state = new_state;

        self.events.push(SessionEvent::StateChanged {
            from: old_state,
            to: new_state,
        });

        Ok(())
    }

    /// Check if a state transition is valid.
    #[must_use]
    #[allow(clippy::unnested_or_patterns)]
    fn is_valid_transition(from: SessionState, to: SessionState) -> bool {
        if from == to {
            return true; // Same state is always valid
        }
        matches!(
            (from, to),
            // From MainMenu
            (
                SessionState::MainMenu,
                SessionState::NewGame | SessionState::Loading | SessionState::Quitting
            ) |
            // From NewGame
            (
                SessionState::NewGame,
                SessionState::Playing | SessionState::MainMenu
            ) |
            // From Loading
            (
                SessionState::Loading,
                SessionState::Playing | SessionState::MainMenu
            ) |
            // From Playing
            (
                SessionState::Playing,
                SessionState::Paused
                    | SessionState::Saving
                    | SessionState::ExitingToMenu
                    | SessionState::Quitting
            ) |
            // From Paused
            (
                SessionState::Paused,
                SessionState::Playing
                    | SessionState::Saving
                    | SessionState::ExitingToMenu
                    | SessionState::Quitting
            ) |
            // From Saving
            (
                SessionState::Saving,
                SessionState::Playing | SessionState::Paused
            ) |
            // From ExitingToMenu
            (SessionState::ExitingToMenu, SessionState::MainMenu)
        )
    }

    /// Set the most recent save slot (for continue functionality).
    pub fn set_last_save_slot(&mut self, slot: SaveSlot) {
        self.last_save_slot = Some(slot);
    }

    /// Check if continue is available.
    #[must_use]
    pub fn can_continue(&self) -> bool {
        self.last_save_slot.is_some()
    }

    /// Get session duration in seconds.
    #[must_use]
    pub fn session_duration(&self) -> f64 {
        self.session_start
            .map_or(0.0, |s| s.elapsed().as_secs_f64())
    }

    /// Set world info (when loading).
    pub fn set_world_info(&mut self, name: String, seed: u64) {
        self.world_name = Some(name);
        self.world_seed = Some(seed);
    }

    /// Set total play time (when loading).
    pub fn set_total_play_time(&mut self, seconds: f64) {
        self.total_play_time = seconds;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_default() {
        let state = SessionState::default();
        assert_eq!(state, SessionState::MainMenu);
    }

    #[test]
    fn test_session_state_checks() {
        assert!(SessionState::Playing.is_world_active());
        assert!(SessionState::Paused.is_world_active());
        assert!(!SessionState::MainMenu.is_world_active());

        assert!(SessionState::MainMenu.is_in_menu());
        assert!(!SessionState::Playing.is_in_menu());

        assert!(SessionState::Playing.should_update_world());
        assert!(!SessionState::Paused.should_update_world());

        assert!(SessionState::Playing.should_render_world());
        assert!(SessionState::Paused.should_render_world());
    }

    #[test]
    fn test_save_slot() {
        let slot = SaveSlot::new(5);
        assert_eq!(slot.number(), 5);
        assert!(!slot.is_quicksave());
        assert!(!slot.is_autosave());
        assert!(slot.is_manual());

        assert!(SaveSlot::QUICKSAVE.is_quicksave());
        assert!(SaveSlot::AUTOSAVE.is_autosave());
    }

    #[test]
    fn test_save_slot_filename() {
        assert_eq!(SaveSlot::QUICKSAVE.filename(), "quicksave.sav");
        assert_eq!(SaveSlot::AUTOSAVE.filename(), "autosave.sav");
        assert_eq!(SaveSlot::new(2).filename(), "save_1.sav");
        assert_eq!(SaveSlot::new(3).filename(), "save_2.sav");
    }

    #[test]
    fn test_session_action_confirmation() {
        assert!(SessionAction::Quit.requires_confirmation());
        assert!(SessionAction::ReturnToMenu.requires_confirmation());
        assert!(!SessionAction::Pause.requires_confirmation());
    }

    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert_eq!(manager.state(), SessionState::MainMenu);
        assert!(manager.world_name().is_none());
        assert!(!manager.can_continue());
    }

    #[test]
    fn test_session_new_game() {
        let mut manager = SessionManager::new();
        manager
            .start_new_game("TestWorld".to_string(), 12345)
            .unwrap();

        assert_eq!(manager.state(), SessionState::Playing);
        assert_eq!(manager.world_name(), Some("TestWorld"));
        assert_eq!(manager.world_seed(), Some(12345));
    }

    #[test]
    fn test_session_pause_resume() {
        let mut manager = SessionManager::new();
        manager.start_new_game("Test".to_string(), 1).unwrap();

        assert!(manager.pause().is_ok());
        assert_eq!(manager.state(), SessionState::Paused);

        assert!(manager.resume().is_ok());
        assert_eq!(manager.state(), SessionState::Playing);
    }

    #[test]
    fn test_session_toggle_pause() {
        let mut manager = SessionManager::new();
        manager.start_new_game("Test".to_string(), 1).unwrap();

        manager.toggle_pause().unwrap();
        assert_eq!(manager.state(), SessionState::Paused);

        manager.toggle_pause().unwrap();
        assert_eq!(manager.state(), SessionState::Playing);
    }

    #[test]
    fn test_session_save() {
        let mut manager = SessionManager::new();
        manager.start_new_game("Test".to_string(), 1).unwrap();

        manager.save_game(SaveSlot::new(2)).unwrap();
        assert_eq!(manager.state(), SessionState::Saving);
        assert_eq!(manager.current_slot(), Some(SaveSlot::new(2)));

        manager.save_complete(true).unwrap();
        assert_eq!(manager.state(), SessionState::Playing);
    }

    #[test]
    fn test_session_return_to_menu() {
        let mut manager = SessionManager::new();
        manager.start_new_game("Test".to_string(), 1).unwrap();

        manager.return_to_menu().unwrap();
        assert_eq!(manager.state(), SessionState::MainMenu);
        assert!(manager.world_name().is_none());
    }

    #[test]
    fn test_invalid_transition() {
        let mut manager = SessionManager::new();

        // Can't pause from main menu
        let result = manager.pause();
        assert!(result.is_err());
    }

    #[test]
    fn test_continue_game() {
        let mut manager = SessionManager::new();

        // Can't continue without a save
        assert!(manager.continue_game().is_err());

        // Set last save and try again
        manager.set_last_save_slot(SaveSlot::AUTOSAVE);
        assert!(manager.can_continue());
        assert!(manager.continue_game().is_ok());
    }

    #[test]
    fn test_auto_save() {
        let mut manager = SessionManager::new();
        manager.start_new_game("Test".to_string(), 1).unwrap();
        manager.set_auto_save_interval(60.0);

        // Not enough time
        manager.update(30.0);
        assert!(!manager.should_auto_save());

        // Enough time
        manager.update(35.0);
        assert!(manager.should_auto_save());
    }

    #[test]
    fn test_session_events() {
        let mut manager = SessionManager::new();
        manager.start_new_game("Test".to_string(), 123).unwrap();

        let events = manager.take_events();
        assert!(!events.is_empty());

        // Should find NewGameStarted event
        let has_new_game = events.iter().any(|e| {
            matches!(e, SessionEvent::NewGameStarted { world_name, seed }
                if world_name == "Test" && *seed == 123)
        });
        assert!(has_new_game);
    }
}
