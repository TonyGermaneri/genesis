//! Graceful exit handling system.
//!
//! This module provides:
//! - Exit confirmation if unsaved changes
//! - Cleanup tasks: save game, save settings, stop audio
//! - Window close event handling

use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors that can occur during exit handling.
#[derive(Debug, Error)]
pub enum ExitError {
    /// Save failed during exit.
    #[error("Failed to save during exit: {0}")]
    SaveFailed(String),

    /// Cleanup task failed.
    #[error("Cleanup failed: {0}")]
    CleanupFailed(String),

    /// Exit was cancelled.
    #[error("Exit was cancelled")]
    Cancelled,

    /// Timeout during cleanup.
    #[error("Cleanup timed out after {0:?}")]
    Timeout(Duration),
}

/// Result type for exit operations.
pub type ExitResult<T> = Result<T, ExitError>;

/// Reason for exiting the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    /// User requested exit from menu.
    UserRequest,
    /// Window close button clicked.
    WindowClose,
    /// Alt+F4 or system quit.
    SystemQuit,
    /// Critical error.
    CriticalError,
    /// Timeout or watchdog.
    Timeout,
}

impl ExitReason {
    /// Returns display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::UserRequest => "User Request",
            Self::WindowClose => "Window Closed",
            Self::SystemQuit => "System Quit",
            Self::CriticalError => "Critical Error",
            Self::Timeout => "Timeout",
        }
    }

    /// Returns whether to show confirmation dialog.
    #[must_use]
    pub fn should_confirm(self) -> bool {
        matches!(self, Self::UserRequest | Self::WindowClose)
    }
}

/// Status of the exit process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// Not exiting.
    NotExiting,
    /// Waiting for user confirmation.
    PendingConfirmation,
    /// Saving game data.
    SavingGame,
    /// Saving settings.
    SavingSettings,
    /// Stopping audio.
    StoppingAudio,
    /// Running custom cleanup tasks.
    CustomCleanup,
    /// Cleanup complete, ready to exit.
    ReadyToExit,
    /// Exit cancelled by user.
    Cancelled,
    /// Exit failed.
    Failed,
}

impl ExitStatus {
    /// Returns display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::NotExiting => "Not Exiting",
            Self::PendingConfirmation => "Confirm Exit?",
            Self::SavingGame => "Saving Game...",
            Self::SavingSettings => "Saving Settings...",
            Self::StoppingAudio => "Stopping Audio...",
            Self::CustomCleanup => "Cleaning Up...",
            Self::ReadyToExit => "Exiting",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Exit Failed",
        }
    }

    /// Returns whether exit is in progress.
    #[must_use]
    pub fn is_in_progress(self) -> bool {
        matches!(
            self,
            Self::PendingConfirmation
                | Self::SavingGame
                | Self::SavingSettings
                | Self::StoppingAudio
                | Self::CustomCleanup
        )
    }

    /// Returns whether exit is complete.
    #[must_use]
    pub fn is_complete(self) -> bool {
        matches!(self, Self::ReadyToExit | Self::Cancelled | Self::Failed)
    }
}

/// A cleanup task to run during exit.
pub struct CleanupTask {
    /// Task name.
    pub name: String,
    /// Task function.
    pub task: Box<dyn FnOnce() -> ExitResult<()> + Send>,
    /// Whether this task is required (failure aborts exit).
    pub required: bool,
}

impl CleanupTask {
    /// Creates a new cleanup task.
    pub fn new(
        name: impl Into<String>,
        task: impl FnOnce() -> ExitResult<()> + Send + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            task: Box::new(task),
            required: false,
        }
    }

    /// Creates a required cleanup task.
    pub fn required(
        name: impl Into<String>,
        task: impl FnOnce() -> ExitResult<()> + Send + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            task: Box::new(task),
            required: true,
        }
    }
}

/// Callback type for exit events.
pub type ExitCallback = Box<dyn Fn(ExitStatus) + Send + Sync>;

/// Handler for graceful exit.
pub struct ExitHandler {
    /// Current exit status.
    status: ExitStatus,
    /// Exit reason.
    reason: Option<ExitReason>,
    /// Whether there are unsaved changes.
    has_unsaved_changes: bool,
    /// Whether to skip confirmation.
    skip_confirmation: bool,
    /// Cleanup tasks to run.
    cleanup_tasks: Vec<CleanupTask>,
    /// Status change callbacks.
    callbacks: Vec<ExitCallback>,
    /// When exit was initiated.
    exit_started: Option<Instant>,
    /// Timeout for cleanup.
    cleanup_timeout: Duration,
    /// Errors encountered during cleanup.
    cleanup_errors: Vec<String>,
    /// Exit code.
    exit_code: i32,
}

impl Default for ExitHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExitHandler {
    /// Creates a new exit handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            status: ExitStatus::NotExiting,
            reason: None,
            has_unsaved_changes: false,
            skip_confirmation: false,
            cleanup_tasks: Vec::new(),
            callbacks: Vec::new(),
            exit_started: None,
            cleanup_timeout: Duration::from_secs(10),
            cleanup_errors: Vec::new(),
            exit_code: 0,
        }
    }

    /// Returns the current exit status.
    #[must_use]
    pub fn status(&self) -> ExitStatus {
        self.status
    }

    /// Returns the exit reason.
    #[must_use]
    pub fn reason(&self) -> Option<ExitReason> {
        self.reason
    }

    /// Returns whether exit is in progress.
    #[must_use]
    pub fn is_exiting(&self) -> bool {
        self.status.is_in_progress() || self.status == ExitStatus::ReadyToExit
    }

    /// Returns whether ready to actually exit.
    #[must_use]
    pub fn should_exit(&self) -> bool {
        self.status == ExitStatus::ReadyToExit
    }

    /// Returns the exit code.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Returns any cleanup errors.
    #[must_use]
    pub fn cleanup_errors(&self) -> &[String] {
        &self.cleanup_errors
    }

    /// Sets whether there are unsaved changes.
    pub fn set_unsaved_changes(&mut self, unsaved: bool) {
        self.has_unsaved_changes = unsaved;
    }

    /// Returns whether there are unsaved changes.
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    /// Sets whether to skip confirmation.
    pub fn set_skip_confirmation(&mut self, skip: bool) {
        self.skip_confirmation = skip;
    }

    /// Sets the cleanup timeout.
    pub fn set_cleanup_timeout(&mut self, timeout: Duration) {
        self.cleanup_timeout = timeout;
    }

    /// Adds a cleanup task.
    pub fn add_cleanup_task(&mut self, task: CleanupTask) {
        self.cleanup_tasks.push(task);
    }

    /// Registers a callback for status changes.
    pub fn on_status_change(&mut self, callback: ExitCallback) {
        self.callbacks.push(callback);
    }

    /// Sets the status and notifies callbacks.
    fn set_status(&mut self, status: ExitStatus) {
        self.status = status;
        for callback in &self.callbacks {
            callback(status);
        }
    }

    /// Initiates the exit process.
    pub fn request_exit(&mut self, reason: ExitReason) {
        if self.is_exiting() {
            debug!("Exit already in progress");
            return;
        }

        info!("Exit requested: {:?}", reason);
        self.reason = Some(reason);
        self.exit_started = Some(Instant::now());
        self.cleanup_errors.clear();

        // Check if confirmation is needed
        if reason.should_confirm() && self.has_unsaved_changes && !self.skip_confirmation {
            self.set_status(ExitStatus::PendingConfirmation);
        } else {
            self.begin_cleanup();
        }
    }

    /// Confirms exit when pending confirmation.
    pub fn confirm_exit(&mut self) {
        if self.status == ExitStatus::PendingConfirmation {
            info!("Exit confirmed by user");
            self.begin_cleanup();
        }
    }

    /// Cancels exit when pending confirmation.
    pub fn cancel_exit(&mut self) {
        if self.status == ExitStatus::PendingConfirmation {
            info!("Exit cancelled by user");
            self.set_status(ExitStatus::Cancelled);
            self.reason = None;
            self.exit_started = None;
        }
    }

    /// Begins the cleanup process.
    fn begin_cleanup(&mut self) {
        self.set_status(ExitStatus::SavingGame);
    }

    /// Updates the exit handler (call each frame during exit).
    /// Returns true when ready to exit.
    pub fn update(&mut self) -> bool {
        // Check timeout
        if let Some(started) = self.exit_started {
            if started.elapsed() > self.cleanup_timeout {
                warn!("Exit cleanup timed out");
                self.set_status(ExitStatus::ReadyToExit);
                return true;
            }
        }

        match self.status {
            ExitStatus::SavingGame => {
                // In real implementation, this would wait for save to complete
                debug!("Game save step (simulated)");
                self.set_status(ExitStatus::SavingSettings);
                false
            }
            ExitStatus::SavingSettings => {
                debug!("Settings save step (simulated)");
                self.set_status(ExitStatus::StoppingAudio);
                false
            }
            ExitStatus::StoppingAudio => {
                debug!("Audio stop step (simulated)");
                self.set_status(ExitStatus::CustomCleanup);
                false
            }
            ExitStatus::CustomCleanup => {
                self.run_cleanup_tasks();
                // Only transition to ReadyToExit if we didn't fail
                if self.status == ExitStatus::CustomCleanup {
                    self.set_status(ExitStatus::ReadyToExit);
                }
                false
            }
            ExitStatus::ReadyToExit => true,
            ExitStatus::NotExiting | ExitStatus::PendingConfirmation | ExitStatus::Cancelled | ExitStatus::Failed => false,
        }
    }

    /// Runs all cleanup tasks.
    fn run_cleanup_tasks(&mut self) {
        let tasks = std::mem::take(&mut self.cleanup_tasks);

        for task in tasks {
            debug!("Running cleanup task: {}", task.name);
            match (task.task)() {
                Ok(()) => {
                    debug!("Cleanup task completed: {}", task.name);
                }
                Err(e) => {
                    let error_msg = format!("{}: {}", task.name, e);
                    error!("Cleanup task failed: {}", error_msg);
                    self.cleanup_errors.push(error_msg);

                    if task.required {
                        self.set_status(ExitStatus::Failed);
                        return;
                    }
                }
            }
        }
    }

    /// Performs immediate exit without cleanup (emergency).
    pub fn force_exit(&mut self, code: i32) {
        warn!("Force exit requested with code {}", code);
        self.exit_code = code;
        self.set_status(ExitStatus::ReadyToExit);
    }

    /// Handles window close event.
    pub fn on_window_close(&mut self) {
        self.request_exit(ExitReason::WindowClose);
    }

    /// Handles system quit signal.
    pub fn on_system_quit(&mut self) {
        // System quit should not show confirmation
        self.skip_confirmation = true;
        self.request_exit(ExitReason::SystemQuit);
    }

    /// Resets the exit handler (for retry after failure).
    pub fn reset(&mut self) {
        self.status = ExitStatus::NotExiting;
        self.reason = None;
        self.exit_started = None;
        self.cleanup_errors.clear();
        debug!("Exit handler reset");
    }
}

/// Builder for configuring exit handling.
pub struct ExitHandlerBuilder {
    handler: ExitHandler,
}

impl ExitHandlerBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler: ExitHandler::new(),
        }
    }

    /// Sets the cleanup timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.handler.cleanup_timeout = timeout;
        self
    }

    /// Adds a cleanup task.
    #[must_use]
    pub fn with_cleanup_task(mut self, task: CleanupTask) -> Self {
        self.handler.cleanup_tasks.push(task);
        self
    }

    /// Sets skip confirmation.
    #[must_use]
    pub fn skip_confirmation(mut self) -> Self {
        self.handler.skip_confirmation = true;
        self
    }

    /// Builds the exit handler.
    #[must_use]
    pub fn build(self) -> ExitHandler {
        self.handler
    }
}

impl Default for ExitHandlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_reason_display() {
        assert_eq!(ExitReason::UserRequest.display_name(), "User Request");
        assert_eq!(ExitReason::WindowClose.display_name(), "Window Closed");
    }

    #[test]
    fn test_exit_reason_should_confirm() {
        assert!(ExitReason::UserRequest.should_confirm());
        assert!(ExitReason::WindowClose.should_confirm());
        assert!(!ExitReason::SystemQuit.should_confirm());
        assert!(!ExitReason::CriticalError.should_confirm());
    }

    #[test]
    fn test_exit_status_display() {
        assert_eq!(ExitStatus::NotExiting.display_name(), "Not Exiting");
        assert_eq!(ExitStatus::SavingGame.display_name(), "Saving Game...");
    }

    #[test]
    fn test_exit_status_is_in_progress() {
        assert!(!ExitStatus::NotExiting.is_in_progress());
        assert!(ExitStatus::SavingGame.is_in_progress());
        assert!(ExitStatus::PendingConfirmation.is_in_progress());
        assert!(!ExitStatus::ReadyToExit.is_in_progress());
    }

    #[test]
    fn test_exit_handler_new() {
        let handler = ExitHandler::new();
        assert_eq!(handler.status(), ExitStatus::NotExiting);
        assert!(!handler.is_exiting());
        assert!(!handler.should_exit());
    }

    #[test]
    fn test_exit_handler_request_exit_no_unsaved() {
        let mut handler = ExitHandler::new();
        handler.request_exit(ExitReason::UserRequest);

        // Should skip confirmation since no unsaved changes
        assert!(handler.status().is_in_progress());
        assert!(handler.is_exiting());
    }

    #[test]
    fn test_exit_handler_request_exit_with_unsaved() {
        let mut handler = ExitHandler::new();
        handler.set_unsaved_changes(true);
        handler.request_exit(ExitReason::UserRequest);

        assert_eq!(handler.status(), ExitStatus::PendingConfirmation);
    }

    #[test]
    fn test_exit_handler_confirm_exit() {
        let mut handler = ExitHandler::new();
        handler.set_unsaved_changes(true);
        handler.request_exit(ExitReason::UserRequest);

        handler.confirm_exit();

        assert!(handler.status().is_in_progress());
        assert_ne!(handler.status(), ExitStatus::PendingConfirmation);
    }

    #[test]
    fn test_exit_handler_cancel_exit() {
        let mut handler = ExitHandler::new();
        handler.set_unsaved_changes(true);
        handler.request_exit(ExitReason::UserRequest);

        handler.cancel_exit();

        assert_eq!(handler.status(), ExitStatus::Cancelled);
        assert!(!handler.is_exiting());
    }

    #[test]
    fn test_exit_handler_update_cycle() {
        let mut handler = ExitHandler::new();
        handler.request_exit(ExitReason::UserRequest);

        // Should progress through stages
        let mut ready = false;
        for _ in 0..10 {
            ready = handler.update();
            if ready {
                break;
            }
        }

        assert!(ready);
        assert!(handler.should_exit());
    }

    #[test]
    fn test_exit_handler_cleanup_task() {
        let mut handler = ExitHandler::new();
        handler.add_cleanup_task(CleanupTask::new("Test task", || Ok(())));

        handler.request_exit(ExitReason::UserRequest);

        // Run through update cycle
        while !handler.update() {}

        assert!(handler.cleanup_errors().is_empty());
    }

    #[test]
    fn test_exit_handler_cleanup_task_failure() {
        let mut handler = ExitHandler::new();
        handler.add_cleanup_task(CleanupTask::new("Failing task", || {
            Err(ExitError::CleanupFailed("Test failure".into()))
        }));

        handler.request_exit(ExitReason::UserRequest);

        // Run through update cycle
        while !handler.update() {}

        // Non-required task failure doesn't prevent exit
        assert!(!handler.cleanup_errors().is_empty());
        assert!(handler.should_exit());
    }

    #[test]
    fn test_exit_handler_required_task_failure() {
        let mut handler = ExitHandler::new();
        handler.add_cleanup_task(CleanupTask::required("Required failing", || {
            Err(ExitError::CleanupFailed("Critical failure".into()))
        }));

        handler.request_exit(ExitReason::UserRequest);

        // Run through update cycle
        for _ in 0..10 {
            if handler.status() == ExitStatus::Failed {
                break;
            }
            handler.update();
        }

        assert_eq!(handler.status(), ExitStatus::Failed);
    }

    #[test]
    fn test_exit_handler_force_exit() {
        let mut handler = ExitHandler::new();
        handler.force_exit(1);

        assert!(handler.should_exit());
        assert_eq!(handler.exit_code(), 1);
    }

    #[test]
    fn test_exit_handler_on_window_close() {
        let mut handler = ExitHandler::new();
        handler.on_window_close();

        assert_eq!(handler.reason(), Some(ExitReason::WindowClose));
    }

    #[test]
    fn test_exit_handler_on_system_quit() {
        let mut handler = ExitHandler::new();
        handler.set_unsaved_changes(true);
        handler.on_system_quit();

        // System quit should skip confirmation
        assert_ne!(handler.status(), ExitStatus::PendingConfirmation);
    }

    #[test]
    fn test_exit_handler_reset() {
        let mut handler = ExitHandler::new();
        handler.request_exit(ExitReason::UserRequest);
        handler.reset();

        assert_eq!(handler.status(), ExitStatus::NotExiting);
        assert!(handler.reason().is_none());
    }

    #[test]
    fn test_exit_handler_builder() {
        let handler = ExitHandlerBuilder::new()
            .with_timeout(Duration::from_secs(5))
            .skip_confirmation()
            .build();

        assert!(handler.skip_confirmation);
    }

    #[test]
    fn test_exit_error_display() {
        let err = ExitError::SaveFailed("test".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("save"));
    }
}
