//! Menu state machine for game state transitions.
//!
//! This module provides:
//! - GameState enum with all valid states
//! - State machine with valid transitions
//! - Query methods for gameplay state

use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur during state transitions.
#[derive(Debug, Error)]
pub enum StateError {
    /// Invalid state transition.
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        /// Source state.
        from: GameState,
        /// Target state.
        to: GameState,
    },

    /// State machine is locked (e.g., during transition).
    #[error("State machine is locked during transition")]
    Locked,

    /// Required condition not met.
    #[error("Condition not met: {0}")]
    ConditionNotMet(String),
}

/// Result type for state operations.
pub type StateResult<T> = Result<T, StateError>;

/// All possible game states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GameState {
    /// Game is initializing (loading assets, etc.).
    #[default]
    Initializing,
    /// Main menu screen.
    MainMenu,
    /// New game character/world setup wizard.
    NewGameWizard,
    /// Loading a saved game.
    Loading,
    /// Active gameplay.
    Playing,
    /// Game is paused (in-game pause menu).
    Paused,
    /// Options/settings menu.
    Options,
    /// Game is exiting (cleanup in progress).
    Exiting,
}

impl GameState {
    /// Returns display name for the state.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Initializing => "Initializing",
            Self::MainMenu => "Main Menu",
            Self::NewGameWizard => "New Game",
            Self::Loading => "Loading",
            Self::Playing => "Playing",
            Self::Paused => "Paused",
            Self::Options => "Options",
            Self::Exiting => "Exiting",
        }
    }

    /// Returns whether this state allows world updates.
    #[must_use]
    pub fn should_update_world(self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Returns whether this state is a menu state.
    #[must_use]
    pub fn is_menu(self) -> bool {
        matches!(
            self,
            Self::MainMenu | Self::NewGameWizard | Self::Options | Self::Paused
        )
    }

    /// Returns whether gameplay is active (playing or paused).
    #[must_use]
    pub fn is_in_game(self) -> bool {
        matches!(self, Self::Playing | Self::Paused | Self::Options)
    }

    /// Returns whether input should be captured for gameplay.
    #[must_use]
    pub fn should_capture_gameplay_input(self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Returns valid transitions from this state.
    #[must_use]
    pub fn valid_transitions(self) -> &'static [GameState] {
        match self {
            Self::Initializing => &[Self::MainMenu, Self::Exiting],
            Self::MainMenu => &[
                Self::NewGameWizard,
                Self::Loading,
                Self::Options,
                Self::Exiting,
            ],
            Self::NewGameWizard | Self::Loading => &[Self::MainMenu, Self::Playing],
            Self::Playing => &[Self::Paused, Self::MainMenu, Self::Exiting],
            Self::Paused => &[Self::Playing, Self::Options, Self::MainMenu, Self::Exiting],
            Self::Options => &[Self::MainMenu, Self::Paused],
            Self::Exiting => &[], // Terminal state
        }
    }

    /// Checks if transition to target state is valid.
    #[must_use]
    pub fn can_transition_to(self, target: Self) -> bool {
        self.valid_transitions().contains(&target)
    }
}

/// Record of a state transition.
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Previous state.
    pub from: GameState,
    /// New state.
    pub to: GameState,
    /// When transition occurred.
    pub timestamp: Instant,
    /// Reason for transition.
    pub reason: Option<String>,
}

/// Callback type for state change notifications.
pub type StateChangeCallback = Box<dyn Fn(&StateTransition) + Send + Sync>;

/// State machine for managing game state.
pub struct MenuStateMachine {
    /// Current game state.
    current_state: GameState,
    /// Previous state (for back navigation).
    previous_state: Option<GameState>,
    /// Whether state machine is locked during transition.
    locked: bool,
    /// State change callbacks.
    callbacks: Vec<StateChangeCallback>,
    /// Transition history.
    history: Vec<StateTransition>,
    /// Maximum history entries.
    max_history: usize,
    /// Time entered current state.
    state_entered_at: Instant,
    /// Whether there are unsaved changes (affects exit behavior).
    has_unsaved_changes: bool,
    /// State to return to after options.
    return_state: Option<GameState>,
}

impl Default for MenuStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuStateMachine {
    /// Creates a new state machine starting in Initializing state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_state: GameState::Initializing,
            previous_state: None,
            locked: false,
            callbacks: Vec::new(),
            history: Vec::new(),
            max_history: 50,
            state_entered_at: Instant::now(),
            has_unsaved_changes: false,
            return_state: None,
        }
    }

    /// Returns the current state.
    #[must_use]
    pub fn current_state(&self) -> GameState {
        self.current_state
    }

    /// Returns the previous state.
    #[must_use]
    pub fn previous_state(&self) -> Option<GameState> {
        self.previous_state
    }

    /// Returns whether the game is currently playing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.current_state == GameState::Playing
    }

    /// Returns whether the game is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.current_state == GameState::Paused
    }

    /// Returns whether the world should be updated.
    #[must_use]
    pub fn should_update_world(&self) -> bool {
        self.current_state.should_update_world()
    }

    /// Returns whether in a menu state.
    #[must_use]
    pub fn is_in_menu(&self) -> bool {
        self.current_state.is_menu()
    }

    /// Returns whether in-game (playing or paused).
    #[must_use]
    pub fn is_in_game(&self) -> bool {
        self.current_state.is_in_game()
    }

    /// Returns whether exiting.
    #[must_use]
    pub fn is_exiting(&self) -> bool {
        self.current_state == GameState::Exiting
    }

    /// Returns time spent in current state.
    #[must_use]
    pub fn time_in_current_state(&self) -> f64 {
        self.state_entered_at.elapsed().as_secs_f64()
    }

    /// Returns whether there are unsaved changes.
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    /// Sets whether there are unsaved changes.
    pub fn set_unsaved_changes(&mut self, unsaved: bool) {
        self.has_unsaved_changes = unsaved;
    }

    /// Clears the unsaved changes flag.
    pub fn mark_saved(&mut self) {
        self.has_unsaved_changes = false;
    }

    /// Attempts to transition to a new state.
    pub fn transition_to(&mut self, target: GameState) -> StateResult<()> {
        self.transition_to_with_reason(target, None)
    }

    /// Attempts to transition to a new state with a reason.
    pub fn transition_to_with_reason(
        &mut self,
        target: GameState,
        reason: Option<String>,
    ) -> StateResult<()> {
        if self.locked {
            return Err(StateError::Locked);
        }

        if !self.current_state.can_transition_to(target) {
            return Err(StateError::InvalidTransition {
                from: self.current_state,
                to: target,
            });
        }

        self.perform_transition(target, reason);
        Ok(())
    }

    /// Performs the actual state transition.
    fn perform_transition(&mut self, target: GameState, reason: Option<String>) {
        let from = self.current_state;

        // Lock during transition
        self.locked = true;

        // Create transition record
        let transition = StateTransition {
            from,
            to: target,
            timestamp: Instant::now(),
            reason: reason.clone(),
        };

        // Update state
        self.previous_state = Some(from);
        self.current_state = target;
        self.state_entered_at = Instant::now();

        // Add to history
        self.history.push(transition.clone());
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Notify callbacks
        for callback in &self.callbacks {
            callback(&transition);
        }

        info!(
            "State transition: {:?} -> {:?}{}",
            from,
            target,
            reason.map(|r| format!(" ({r})")).unwrap_or_default()
        );

        // Unlock
        self.locked = false;
    }

    /// Registers a callback for state changes.
    pub fn on_state_change(&mut self, callback: StateChangeCallback) {
        self.callbacks.push(callback);
    }

    /// Returns the transition history.
    #[must_use]
    pub fn history(&self) -> &[StateTransition] {
        &self.history
    }

    // === Convenience transition methods ===

    /// Transitions from Initializing to MainMenu.
    pub fn finish_initialization(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::MainMenu, Some("Initialization complete".into()))
    }

    /// Starts a new game (from NewGameWizard or direct).
    pub fn start_new_game(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::Playing, Some("New game started".into()))
    }

    /// Loads a saved game.
    pub fn load_game(&mut self) -> StateResult<()> {
        if self.current_state == GameState::MainMenu {
            self.transition_to(GameState::Loading)?;
        }
        // Loading state will transition to Playing when load completes
        Ok(())
    }

    /// Finishes loading and starts playing.
    pub fn finish_loading(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::Playing, Some("Load complete".into()))
    }

    /// Pauses the game.
    pub fn pause(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::Paused, Some("Game paused".into()))
    }

    /// Resumes the game from pause.
    pub fn resume(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::Playing, Some("Game resumed".into()))
    }

    /// Toggles between playing and paused.
    pub fn toggle_pause(&mut self) -> StateResult<()> {
        match self.current_state {
            GameState::Playing => self.pause(),
            GameState::Paused => self.resume(),
            _ => Err(StateError::InvalidTransition {
                from: self.current_state,
                to: GameState::Paused,
            }),
        }
    }

    /// Opens the options menu.
    pub fn open_options(&mut self) -> StateResult<()> {
        // Remember where to return
        self.return_state = Some(self.current_state);
        self.transition_to_with_reason(GameState::Options, Some("Options opened".into()))
    }

    /// Closes options and returns to previous state.
    pub fn close_options(&mut self) -> StateResult<()> {
        let return_to = self.return_state.unwrap_or(GameState::MainMenu);
        self.return_state = None;
        self.transition_to_with_reason(return_to, Some("Options closed".into()))
    }

    /// Returns to main menu from gameplay.
    pub fn return_to_menu(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::MainMenu, Some("Returned to menu".into()))
    }

    /// Begins exit process.
    pub fn begin_exit(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::Exiting, Some("Exit requested".into()))
    }

    /// Opens the new game wizard.
    pub fn open_new_game_wizard(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::NewGameWizard, Some("New game wizard opened".into()))
    }

    /// Cancels new game wizard and returns to menu.
    pub fn cancel_new_game(&mut self) -> StateResult<()> {
        self.transition_to_with_reason(GameState::MainMenu, Some("New game cancelled".into()))
    }

    /// Force transitions to a state (bypasses validation - use with caution).
    pub fn force_state(&mut self, state: GameState) {
        warn!("Force transitioning to {:?}", state);
        self.perform_transition(state, Some("Forced transition".into()));
    }

    /// Resets state machine to initial state.
    pub fn reset(&mut self) {
        self.current_state = GameState::Initializing;
        self.previous_state = None;
        self.locked = false;
        self.history.clear();
        self.state_entered_at = Instant::now();
        self.has_unsaved_changes = false;
        self.return_state = None;
        debug!("State machine reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_display_name() {
        assert_eq!(GameState::Playing.display_name(), "Playing");
        assert_eq!(GameState::Paused.display_name(), "Paused");
        assert_eq!(GameState::MainMenu.display_name(), "Main Menu");
    }

    #[test]
    fn test_game_state_should_update_world() {
        assert!(GameState::Playing.should_update_world());
        assert!(!GameState::Paused.should_update_world());
        assert!(!GameState::MainMenu.should_update_world());
    }

    #[test]
    fn test_game_state_is_menu() {
        assert!(GameState::MainMenu.is_menu());
        assert!(GameState::Paused.is_menu());
        assert!(GameState::Options.is_menu());
        assert!(!GameState::Playing.is_menu());
    }

    #[test]
    fn test_game_state_is_in_game() {
        assert!(GameState::Playing.is_in_game());
        assert!(GameState::Paused.is_in_game());
        assert!(!GameState::MainMenu.is_in_game());
    }

    #[test]
    fn test_game_state_valid_transitions() {
        // From Initializing
        assert!(GameState::Initializing.can_transition_to(GameState::MainMenu));
        assert!(!GameState::Initializing.can_transition_to(GameState::Playing));

        // From MainMenu
        assert!(GameState::MainMenu.can_transition_to(GameState::NewGameWizard));
        assert!(GameState::MainMenu.can_transition_to(GameState::Loading));
        assert!(GameState::MainMenu.can_transition_to(GameState::Options));
        assert!(GameState::MainMenu.can_transition_to(GameState::Exiting));
        assert!(!GameState::MainMenu.can_transition_to(GameState::Playing));

        // From Playing
        assert!(GameState::Playing.can_transition_to(GameState::Paused));
        assert!(GameState::Playing.can_transition_to(GameState::MainMenu));
        assert!(!GameState::Playing.can_transition_to(GameState::Options));

        // From Paused
        assert!(GameState::Paused.can_transition_to(GameState::Playing));
        assert!(GameState::Paused.can_transition_to(GameState::Options));
        assert!(GameState::Paused.can_transition_to(GameState::MainMenu));
    }

    #[test]
    fn test_state_machine_new() {
        let sm = MenuStateMachine::new();
        assert_eq!(sm.current_state(), GameState::Initializing);
        assert!(!sm.is_playing());
        assert!(!sm.is_paused());
    }

    #[test]
    fn test_state_machine_finish_initialization() {
        let mut sm = MenuStateMachine::new();
        assert!(sm.finish_initialization().is_ok());
        assert_eq!(sm.current_state(), GameState::MainMenu);
    }

    #[test]
    fn test_state_machine_start_new_game() {
        let mut sm = MenuStateMachine::new();
        sm.finish_initialization().expect("init failed");
        sm.open_new_game_wizard().expect("wizard failed");
        assert!(sm.start_new_game().is_ok());
        assert!(sm.is_playing());
    }

    #[test]
    fn test_state_machine_pause_resume() {
        let mut sm = MenuStateMachine::new();
        sm.finish_initialization().expect("init failed");
        sm.open_new_game_wizard().expect("wizard failed");
        sm.start_new_game().expect("start failed");

        assert!(sm.pause().is_ok());
        assert!(sm.is_paused());
        assert!(!sm.should_update_world());

        assert!(sm.resume().is_ok());
        assert!(sm.is_playing());
        assert!(sm.should_update_world());
    }

    #[test]
    fn test_state_machine_toggle_pause() {
        let mut sm = MenuStateMachine::new();
        sm.finish_initialization().expect("init failed");
        sm.open_new_game_wizard().expect("wizard failed");
        sm.start_new_game().expect("start failed");

        assert!(sm.toggle_pause().is_ok());
        assert!(sm.is_paused());

        assert!(sm.toggle_pause().is_ok());
        assert!(sm.is_playing());
    }

    #[test]
    fn test_state_machine_invalid_transition() {
        let mut sm = MenuStateMachine::new();
        // Can't go directly from Initializing to Playing
        let result = sm.transition_to(GameState::Playing);
        assert!(matches!(result, Err(StateError::InvalidTransition { .. })));
    }

    #[test]
    fn test_state_machine_options_return() {
        let mut sm = MenuStateMachine::new();
        sm.finish_initialization().expect("init failed");
        sm.open_new_game_wizard().expect("wizard failed");
        sm.start_new_game().expect("start failed");
        sm.pause().expect("pause failed");

        // Open options from paused state
        assert!(sm.open_options().is_ok());
        assert_eq!(sm.current_state(), GameState::Options);

        // Close options should return to paused
        assert!(sm.close_options().is_ok());
        assert_eq!(sm.current_state(), GameState::Paused);
    }

    #[test]
    fn test_state_machine_unsaved_changes() {
        let mut sm = MenuStateMachine::new();
        assert!(!sm.has_unsaved_changes());

        sm.set_unsaved_changes(true);
        assert!(sm.has_unsaved_changes());

        sm.mark_saved();
        assert!(!sm.has_unsaved_changes());
    }

    #[test]
    fn test_state_machine_history() {
        let mut sm = MenuStateMachine::new();
        sm.finish_initialization().expect("init failed");
        sm.open_new_game_wizard().expect("wizard failed");
        sm.start_new_game().expect("start failed");

        let history = sm.history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].from, GameState::Initializing);
        assert_eq!(history[0].to, GameState::MainMenu);
    }

    #[test]
    fn test_state_machine_previous_state() {
        let mut sm = MenuStateMachine::new();
        sm.finish_initialization().expect("init failed");

        assert_eq!(sm.previous_state(), Some(GameState::Initializing));
    }

    #[test]
    fn test_state_machine_force_state() {
        let mut sm = MenuStateMachine::new();
        sm.force_state(GameState::Playing);
        assert!(sm.is_playing());
    }

    #[test]
    fn test_state_machine_reset() {
        let mut sm = MenuStateMachine::new();
        sm.finish_initialization().expect("init failed");
        sm.set_unsaved_changes(true);

        sm.reset();

        assert_eq!(sm.current_state(), GameState::Initializing);
        assert!(sm.previous_state().is_none());
        assert!(!sm.has_unsaved_changes());
        assert!(sm.history().is_empty());
    }

    #[test]
    fn test_state_error_display() {
        let err = StateError::InvalidTransition {
            from: GameState::MainMenu,
            to: GameState::Playing,
        };
        let msg = format!("{err}");
        assert!(msg.contains("Invalid"));
    }
}
