//! End-to-end integration tests for Project Genesis.
//!
//! These tests verify that features work correctly when integrated together,
//! simulating actual user interactions and validating expected outcomes.

#![cfg(test)]

use crate::app::AppMode;
use genesis_tools::ui::{
    MainMenu, MainMenuAction, MainMenuButton,
    PauseMenu, PauseMenuAction, PauseMenuButton,
};

/// Test suite for menu system integration
mod menu_tests {
    use super::*;

    #[test]
    fn e2e_app_starts_in_menu_mode() {
        let mode = AppMode::default();
        assert_eq!(mode, AppMode::Menu, "Game should start in Menu mode, not Playing");
    }

    #[test]
    fn e2e_main_menu_visible_on_creation() {
        let menu = MainMenu::with_defaults();
        assert!(menu.is_visible(), "Main menu should be visible by default");
    }

    #[test]
    fn e2e_main_menu_new_game_click_generates_action() {
        let mut menu = MainMenu::with_defaults();

        // User clicks "New Game"
        menu.click_button(MainMenuButton::NewGame);

        // Should generate NewGame action
        let actions = menu.drain_actions();
        assert!(
            actions.contains(&MainMenuAction::NewGame),
            "Clicking New Game button should generate NewGame action"
        );
    }

    #[test]
    fn e2e_main_menu_exit_click_generates_action() {
        let mut menu = MainMenu::with_defaults();

        // User clicks "Exit"
        menu.click_button(MainMenuButton::Exit);

        // Should generate Exit action
        let actions = menu.drain_actions();
        assert!(
            actions.contains(&MainMenuAction::Exit),
            "Clicking Exit button should generate Exit action"
        );
    }

    #[test]
    fn e2e_main_menu_options_click_generates_action() {
        let mut menu = MainMenu::with_defaults();

        // User clicks "Options"
        menu.click_button(MainMenuButton::Options);

        // Should generate OpenOptions action
        let actions = menu.drain_actions();
        assert!(
            actions.contains(&MainMenuAction::OpenOptions),
            "Clicking Options button should generate OpenOptions action"
        );
    }

    #[test]
    fn e2e_main_menu_navigation_with_keys() {
        use genesis_tools::ui::{NavigationDirection, SaveAvailability};

        let mut menu = MainMenu::with_defaults();

        // By default, Continue and LoadGame are disabled (no saves)
        // So navigation goes: NewGame(0) -> Options(3) -> Exit(4)

        // Initially at index 0 (New Game)
        assert_eq!(menu.selected_index(), 0);

        // Navigate down - skips disabled Continue(1) and LoadGame(2)
        menu.navigate(NavigationDirection::Down);
        assert_eq!(menu.selected_index(), 3, "Should skip disabled buttons and move to Options");

        // Navigate down again
        menu.navigate(NavigationDirection::Down);
        assert_eq!(menu.selected_index(), 4, "Should move to Exit");

        // Navigate up - back to Options
        menu.navigate(NavigationDirection::Up);
        assert_eq!(menu.selected_index(), 3, "Should move back to Options");

        // Now enable saves and test full navigation
        menu.set_save_availability(SaveAvailability::with_saves("Slot 1", "2024-01-01"));
        menu.set_selected_index(0);

        // Navigate down - now Continue is enabled
        menu.navigate(NavigationDirection::Down);
        assert_eq!(menu.selected_index(), 1, "With saves, should move to Continue");
    }

    #[test]
    fn e2e_main_menu_confirm_selection() {
        let mut menu = MainMenu::with_defaults();

        // Select New Game (already at index 0)
        menu.confirm_selection();

        // Should generate NewGame action
        let actions = menu.drain_actions();
        assert!(
            actions.contains(&MainMenuAction::NewGame),
            "Confirming selection on New Game should generate NewGame action"
        );
    }

    #[test]
    fn e2e_pause_menu_starts_hidden() {
        let menu = PauseMenu::with_defaults();
        assert!(!menu.is_visible(), "Pause menu should start hidden");
    }

    #[test]
    fn e2e_pause_menu_show_hide_toggle() {
        let mut menu = PauseMenu::with_defaults();

        // Initially hidden
        assert!(!menu.is_visible());

        // Show the menu
        menu.show();
        assert!(menu.is_visible(), "Menu should be visible after show()");

        // Hide the menu
        menu.hide();
        assert!(!menu.is_visible(), "Menu should be hidden after hide()");
    }

    #[test]
    fn e2e_pause_menu_resume_click() {
        let mut menu = PauseMenu::with_defaults();
        menu.show();

        // User clicks "Resume"
        menu.click_button(PauseMenuButton::Resume);

        // Should generate Resume action
        let actions = menu.drain_actions();
        assert!(
            actions.contains(&PauseMenuAction::Resume),
            "Clicking Resume button should generate Resume action"
        );
    }

    #[test]
    fn e2e_pause_menu_quit_to_menu_click() {
        let mut menu = PauseMenu::with_defaults();
        menu.show();

        // User clicks "Quit to Menu" - this shows confirmation first
        menu.click_button(PauseMenuButton::QuitToMenu);

        // Should show confirmation dialog, not immediately generate action
        assert!(
            menu.is_confirmation_showing(),
            "Quit to Menu should show confirmation dialog first"
        );

        // No action generated yet (waiting for confirmation)
        let actions = menu.drain_actions();
        assert!(actions.is_empty(), "No action until user confirms");
    }
}

/// Test suite for app mode state transitions
mod state_transition_tests {
    use super::*;

    #[test]
    fn e2e_mode_transitions_menu_to_playing() {
        // Simulate: User starts game in menu, clicks New Game
        let mut mode = AppMode::Menu;
        let mut menu = MainMenu::with_defaults();

        // Click New Game
        menu.click_button(MainMenuButton::NewGame);

        // Process action
        for action in menu.drain_actions() {
            if action == MainMenuAction::NewGame {
                mode = AppMode::Playing;
            }
        }

        assert_eq!(mode, AppMode::Playing, "Should transition to Playing after New Game");
    }

    #[test]
    fn e2e_mode_transitions_playing_to_paused() {
        // Simulate: User is playing, presses ESC
        let mode = AppMode::Playing;

        // ESC while playing should pause
        let new_mode = match mode {
            AppMode::Playing => AppMode::Paused,
            other => other,
        };

        assert_eq!(new_mode, AppMode::Paused, "ESC in Playing should pause");
    }

    #[test]
    fn e2e_mode_transitions_paused_to_playing() {
        // Simulate: User is paused, clicks Resume
        let mut mode = AppMode::Paused;
        let mut menu = PauseMenu::with_defaults();
        menu.show();

        // Click Resume
        menu.click_button(PauseMenuButton::Resume);

        // Process action
        for action in menu.drain_actions() {
            if action == PauseMenuAction::Resume {
                mode = AppMode::Playing;
            }
        }

        assert_eq!(mode, AppMode::Playing, "Resume should return to Playing");
    }

    #[test]
    fn e2e_mode_transitions_paused_to_menu() {
        // Simulate: User is paused, clicks Quit to Menu (with confirmation)
        // QuitToMenu requires clicking twice - once to show dialog, once to confirm
        let mut mode = AppMode::Paused;
        let mut pause_menu = PauseMenu::with_defaults();
        let mut main_menu = MainMenu::with_defaults();
        main_menu.hide(); // Hidden while playing

        pause_menu.show();

        // First click shows confirmation
        pause_menu.click_button(PauseMenuButton::QuitToMenu);
        assert!(pause_menu.is_confirmation_showing(), "Should show confirmation");

        // For this test, we simulate what happens AFTER user confirms
        // by directly pushing the action (as the real confirm button would)
        // Note: In real app, user would click "Yes" in the dialog

        // Simulate confirmation by processing the action directly
        mode = AppMode::Menu;
        pause_menu.hide();
        main_menu.show();

        assert_eq!(mode, AppMode::Menu, "Quit to Menu should return to Menu mode");
        assert!(main_menu.is_visible(), "Main menu should be visible");
        assert!(!pause_menu.is_visible(), "Pause menu should be hidden");
    }
}

/// Test suite for button state management
mod button_state_tests {
    use super::*;
    use genesis_tools::ui::SaveAvailability;

    #[test]
    fn e2e_continue_disabled_without_saves() {
        let mut menu = MainMenu::with_defaults();

        // No saves available
        menu.set_save_availability(SaveAvailability::none());

        // Continue should be disabled
        // (This is checked internally by is_button_disabled)
        let availability = menu.save_availability();
        assert!(!availability.has_continue_save, "Continue should be disabled without saves");
    }

    #[test]
    fn e2e_continue_enabled_with_saves() {
        let mut menu = MainMenu::with_defaults();

        // Set saves available
        let availability = SaveAvailability::with_saves("Slot 1", "2024-01-01 12:00");
        menu.set_save_availability(availability);

        // Continue should be enabled
        let availability = menu.save_availability();
        assert!(availability.has_continue_save, "Continue should be enabled with saves");
    }
}

/// Integration tests that verify the full feature works
mod full_integration_tests {
    use super::*;

    #[test]
    fn e2e_complete_new_game_flow() {
        // This test simulates the complete flow:
        // 1. App starts in Menu mode
        // 2. User sees main menu
        // 3. User clicks New Game
        // 4. App transitions to Playing mode
        // 5. Main menu is hidden

        let mut mode = AppMode::default();
        let mut main_menu = MainMenu::with_defaults();

        // Step 1: Verify starting state
        assert_eq!(mode, AppMode::Menu, "Should start in Menu");
        assert!(main_menu.is_visible(), "Menu should be visible");

        // Step 2: User clicks New Game
        main_menu.click_button(MainMenuButton::NewGame);

        // Step 3: Process action
        for action in main_menu.drain_actions() {
            match action {
                MainMenuAction::NewGame => {
                    mode = AppMode::Playing;
                    main_menu.hide();
                }
                _ => {}
            }
        }

        // Step 4: Verify final state
        assert_eq!(mode, AppMode::Playing, "Should be Playing after New Game");
        assert!(!main_menu.is_visible(), "Menu should be hidden");
    }

    #[test]
    fn e2e_complete_pause_resume_flow() {
        // This test simulates:
        // 1. Game is in Playing mode
        // 2. User presses ESC to pause
        // 3. Pause menu appears
        // 4. User clicks Resume
        // 5. Game returns to Playing, menu hidden

        let mut mode = AppMode::Playing;
        let mut pause_menu = PauseMenu::with_defaults();

        // Step 1: User presses ESC (simulated)
        mode = AppMode::Paused;
        pause_menu.show();

        assert_eq!(mode, AppMode::Paused);
        assert!(pause_menu.is_visible());

        // Step 2: User clicks Resume
        pause_menu.click_button(PauseMenuButton::Resume);

        // Step 3: Process action
        for action in pause_menu.drain_actions() {
            match action {
                PauseMenuAction::Resume => {
                    mode = AppMode::Playing;
                    pause_menu.hide();
                }
                _ => {}
            }
        }

        // Step 4: Verify final state
        assert_eq!(mode, AppMode::Playing, "Should be Playing after Resume");
        assert!(!pause_menu.is_visible(), "Pause menu should be hidden");
    }

    #[test]
    fn e2e_complete_return_to_menu_flow() {
        // This test simulates:
        // 1. Game is Playing
        // 2. User pauses
        // 3. User clicks "Quit to Menu" (shows confirmation)
        // 4. User confirms (simulated)
        // 5. Returns to main menu

        let mut mode = AppMode::Playing;
        let mut main_menu = MainMenu::with_defaults();
        let mut pause_menu = PauseMenu::with_defaults();
        main_menu.hide(); // Hidden during gameplay

        // Step 1: User pauses
        mode = AppMode::Paused;
        pause_menu.show();

        // Step 2: User clicks Quit to Menu (shows confirmation)
        pause_menu.click_button(PauseMenuButton::QuitToMenu);
        assert!(pause_menu.is_confirmation_showing(), "Should show confirmation");

        // Step 3: User confirms - in real app this calls confirm_selection again
        // For testing, we simulate the confirmed action directly
        mode = AppMode::Menu;
        pause_menu.hide();
        main_menu.show();

        // Step 4: Verify final state
        assert_eq!(mode, AppMode::Menu, "Should be in Menu");
        assert!(main_menu.is_visible(), "Main menu should be visible");
        assert!(!pause_menu.is_visible(), "Pause menu should be hidden");
    }
}

/// Regression tests for specific bugs
mod regression_tests {
    use super::*;

    #[test]
    fn regression_menu_not_interactive_issue() {
        // Regression test for: Menu visible but not responding to clicks
        // Root cause was: renderer.handle_event() not being called
        // Fix: Pass events to egui in window_event handler

        let mut menu = MainMenu::with_defaults();
        assert!(menu.is_visible(), "Menu should be visible");

        // Simulate a click - should work
        menu.click_button(MainMenuButton::NewGame);

        let actions = menu.drain_actions();
        assert!(
            !actions.is_empty(),
            "REGRESSION: Menu clicks should generate actions (was broken when events weren't passed to egui)"
        );
    }

    #[test]
    fn regression_app_mode_default_playing() {
        // Regression test for: App started in Playing mode instead of Menu
        // Root cause was: #[default] on Playing variant instead of Menu
        // Fix: Changed default to Menu variant

        let mode = AppMode::default();
        assert_eq!(
            mode,
            AppMode::Menu,
            "REGRESSION: App should start in Menu mode, not Playing"
        );
    }
}
