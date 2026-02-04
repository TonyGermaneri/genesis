//! Input rebinding system.
//!
//! This module provides:
//! - GameAction enum for all bindable actions
//! - KeyBinding with primary/secondary keys
//! - Listen for key press during rebind
//! - Conflict detection
//! - Reset to defaults

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info};

/// Errors that can occur during input rebinding.
#[derive(Debug, Error)]
pub enum RebindError {
    /// Key is already bound to another action.
    #[error("Key {key} is already bound to {action:?}")]
    Conflict {
        /// The conflicting key.
        key: String,
        /// The action it's bound to.
        action: GameAction,
    },

    /// Action not found.
    #[error("Action not found: {0:?}")]
    ActionNotFound(GameAction),

    /// Invalid key.
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Cannot unbind required action.
    #[error("Cannot unbind required action: {0:?}")]
    RequiredAction(GameAction),
}

/// Result type for rebind operations.
pub type RebindResult<T> = Result<T, RebindError>;

/// All bindable game actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameAction {
    // === Movement ===
    /// Move up/forward.
    MoveUp,
    /// Move down/backward.
    MoveDown,
    /// Move left.
    MoveLeft,
    /// Move right.
    MoveRight,
    /// Sprint/run modifier.
    Sprint,
    /// Dodge/roll.
    Dodge,
    /// Jump.
    Jump,
    /// Crouch/sneak.
    Crouch,

    // === Combat ===
    /// Primary attack.
    Attack,
    /// Secondary attack/heavy attack.
    AttackSecondary,
    /// Block/defend.
    Block,
    /// Lock on to target.
    LockOn,
    /// Switch target (when locked on).
    SwitchTarget,
    /// Use equipped item.
    UseItem,

    // === Interaction ===
    /// Interact with object/NPC.
    Interact,
    /// Pick up item.
    PickUp,
    /// Drop item.
    Drop,

    // === UI ===
    /// Open/close inventory.
    Inventory,
    /// Open/close map.
    Map,
    /// Open/close journal/quest log.
    Journal,
    /// Open/close character screen.
    Character,
    /// Open/close crafting menu.
    Crafting,
    /// Pause game.
    Pause,
    /// Quick save.
    QuickSave,
    /// Quick load.
    QuickLoad,

    // === Hotbar ===
    /// Hotbar slot 1.
    Hotbar1,
    /// Hotbar slot 2.
    Hotbar2,
    /// Hotbar slot 3.
    Hotbar3,
    /// Hotbar slot 4.
    Hotbar4,
    /// Hotbar slot 5.
    Hotbar5,
    /// Hotbar slot 6.
    Hotbar6,
    /// Hotbar slot 7.
    Hotbar7,
    /// Hotbar slot 8.
    Hotbar8,
    /// Hotbar slot 9.
    Hotbar9,
    /// Hotbar slot 10.
    Hotbar10,

    // === Camera ===
    /// Zoom in.
    ZoomIn,
    /// Zoom out.
    ZoomOut,
    /// Rotate camera left.
    CameraLeft,
    /// Rotate camera right.
    CameraRight,
    /// Reset camera.
    CameraReset,

    // === Debug ===
    /// Toggle debug overlay.
    DebugOverlay,
    /// Toggle debug console.
    DebugConsole,
}

impl GameAction {
    /// Returns display name for the action.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            // Movement
            Self::MoveUp => "Move Up",
            Self::MoveDown => "Move Down",
            Self::MoveLeft => "Move Left",
            Self::MoveRight => "Move Right",
            Self::Sprint => "Sprint",
            Self::Dodge => "Dodge",
            Self::Jump => "Jump",
            Self::Crouch => "Crouch",
            // Combat
            Self::Attack => "Attack",
            Self::AttackSecondary => "Heavy Attack",
            Self::Block => "Block",
            Self::LockOn => "Lock On",
            Self::SwitchTarget => "Switch Target",
            Self::UseItem => "Use Item",
            // Interaction
            Self::Interact => "Interact",
            Self::PickUp => "Pick Up",
            Self::Drop => "Drop",
            // UI
            Self::Inventory => "Inventory",
            Self::Map => "Map",
            Self::Journal => "Journal",
            Self::Character => "Character",
            Self::Crafting => "Crafting",
            Self::Pause => "Pause",
            Self::QuickSave => "Quick Save",
            Self::QuickLoad => "Quick Load",
            // Hotbar
            Self::Hotbar1 => "Hotbar 1",
            Self::Hotbar2 => "Hotbar 2",
            Self::Hotbar3 => "Hotbar 3",
            Self::Hotbar4 => "Hotbar 4",
            Self::Hotbar5 => "Hotbar 5",
            Self::Hotbar6 => "Hotbar 6",
            Self::Hotbar7 => "Hotbar 7",
            Self::Hotbar8 => "Hotbar 8",
            Self::Hotbar9 => "Hotbar 9",
            Self::Hotbar10 => "Hotbar 10",
            // Camera
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
            Self::CameraLeft => "Camera Left",
            Self::CameraRight => "Camera Right",
            Self::CameraReset => "Reset Camera",
            // Debug
            Self::DebugOverlay => "Debug Overlay",
            Self::DebugConsole => "Debug Console",
        }
    }

    /// Returns the category for this action.
    #[must_use]
    pub fn category(self) -> ActionCategory {
        match self {
            Self::MoveUp
            | Self::MoveDown
            | Self::MoveLeft
            | Self::MoveRight
            | Self::Sprint
            | Self::Dodge
            | Self::Jump
            | Self::Crouch => ActionCategory::Movement,

            Self::Attack
            | Self::AttackSecondary
            | Self::Block
            | Self::LockOn
            | Self::SwitchTarget
            | Self::UseItem => ActionCategory::Combat,

            Self::Interact | Self::PickUp | Self::Drop => ActionCategory::Interaction,

            Self::Inventory
            | Self::Map
            | Self::Journal
            | Self::Character
            | Self::Crafting
            | Self::Pause
            | Self::QuickSave
            | Self::QuickLoad => ActionCategory::UI,

            Self::Hotbar1
            | Self::Hotbar2
            | Self::Hotbar3
            | Self::Hotbar4
            | Self::Hotbar5
            | Self::Hotbar6
            | Self::Hotbar7
            | Self::Hotbar8
            | Self::Hotbar9
            | Self::Hotbar10 => ActionCategory::Hotbar,

            Self::ZoomIn
            | Self::ZoomOut
            | Self::CameraLeft
            | Self::CameraRight
            | Self::CameraReset => ActionCategory::Camera,

            Self::DebugOverlay | Self::DebugConsole => ActionCategory::Debug,
        }
    }

    /// Returns whether this action is required (cannot be unbound).
    #[must_use]
    pub fn is_required(self) -> bool {
        matches!(self, Self::Pause | Self::MoveUp | Self::MoveDown | Self::MoveLeft | Self::MoveRight)
    }

    /// Returns all actions in a category.
    #[must_use]
    pub fn actions_in_category(category: ActionCategory) -> Vec<Self> {
        ALL_ACTIONS
            .iter()
            .copied()
            .filter(|a| a.category() == category)
            .collect()
    }
}

/// Categories for grouping actions in UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionCategory {
    /// Movement actions.
    Movement,
    /// Combat actions.
    Combat,
    /// Interaction actions.
    Interaction,
    /// UI/menu actions.
    UI,
    /// Hotbar slots.
    Hotbar,
    /// Camera controls.
    Camera,
    /// Debug actions.
    Debug,
}

impl ActionCategory {
    /// Returns display name.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Movement => "Movement",
            Self::Combat => "Combat",
            Self::Interaction => "Interaction",
            Self::UI => "Interface",
            Self::Hotbar => "Hotbar",
            Self::Camera => "Camera",
            Self::Debug => "Debug",
        }
    }
}

/// All game actions for iteration.
pub const ALL_ACTIONS: &[GameAction] = &[
    GameAction::MoveUp,
    GameAction::MoveDown,
    GameAction::MoveLeft,
    GameAction::MoveRight,
    GameAction::Sprint,
    GameAction::Dodge,
    GameAction::Jump,
    GameAction::Crouch,
    GameAction::Attack,
    GameAction::AttackSecondary,
    GameAction::Block,
    GameAction::LockOn,
    GameAction::SwitchTarget,
    GameAction::UseItem,
    GameAction::Interact,
    GameAction::PickUp,
    GameAction::Drop,
    GameAction::Inventory,
    GameAction::Map,
    GameAction::Journal,
    GameAction::Character,
    GameAction::Crafting,
    GameAction::Pause,
    GameAction::QuickSave,
    GameAction::QuickLoad,
    GameAction::Hotbar1,
    GameAction::Hotbar2,
    GameAction::Hotbar3,
    GameAction::Hotbar4,
    GameAction::Hotbar5,
    GameAction::Hotbar6,
    GameAction::Hotbar7,
    GameAction::Hotbar8,
    GameAction::Hotbar9,
    GameAction::Hotbar10,
    GameAction::ZoomIn,
    GameAction::ZoomOut,
    GameAction::CameraLeft,
    GameAction::CameraRight,
    GameAction::CameraReset,
    GameAction::DebugOverlay,
    GameAction::DebugConsole,
];

/// All action categories.
pub const ALL_CATEGORIES: &[ActionCategory] = &[
    ActionCategory::Movement,
    ActionCategory::Combat,
    ActionCategory::Interaction,
    ActionCategory::UI,
    ActionCategory::Hotbar,
    ActionCategory::Camera,
    ActionCategory::Debug,
];

/// A key binding with primary and optional secondary key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    /// The action this binding is for.
    pub action: GameAction,
    /// Primary key (required).
    pub primary: String,
    /// Secondary/alternate key (optional).
    pub secondary: Option<String>,
}

impl KeyBinding {
    /// Creates a new key binding.
    #[must_use]
    pub fn new(action: GameAction, primary: impl Into<String>) -> Self {
        Self {
            action,
            primary: primary.into(),
            secondary: None,
        }
    }

    /// Creates a key binding with secondary key.
    #[must_use]
    pub fn with_secondary(mut self, secondary: impl Into<String>) -> Self {
        self.secondary = Some(secondary.into());
        self
    }

    /// Returns whether the given key matches this binding.
    #[must_use]
    pub fn matches(&self, key: &str) -> bool {
        self.primary == key || self.secondary.as_deref() == Some(key)
    }

    /// Returns all bound keys.
    #[must_use]
    pub fn all_keys(&self) -> Vec<&str> {
        let mut keys = vec![self.primary.as_str()];
        if let Some(ref sec) = self.secondary {
            keys.push(sec.as_str());
        }
        keys
    }
}

/// Rebind mode state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebindMode {
    /// Not in rebind mode.
    Inactive,
    /// Waiting for primary key input.
    WaitingForPrimary(GameAction),
    /// Waiting for secondary key input.
    WaitingForSecondary(GameAction),
}

/// Manager for input rebinding.
pub struct InputRebindManager {
    /// All key bindings.
    bindings: HashMap<GameAction, KeyBinding>,
    /// Reverse mapping: key -> action.
    key_to_action: HashMap<String, GameAction>,
    /// Current rebind mode.
    rebind_mode: RebindMode,
    /// Whether bindings have changed.
    dirty: bool,
}

impl Default for InputRebindManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputRebindManager {
    /// Creates a new rebind manager with default bindings.
    #[must_use]
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            key_to_action: HashMap::new(),
            rebind_mode: RebindMode::Inactive,
            dirty: false,
        };
        manager.reset_to_defaults();
        manager
    }

    /// Returns default key bindings.
    #[must_use]
    pub fn default_bindings() -> Vec<KeyBinding> {
        vec![
            // Movement (WASD + arrows)
            KeyBinding::new(GameAction::MoveUp, "W").with_secondary("Up"),
            KeyBinding::new(GameAction::MoveDown, "S").with_secondary("Down"),
            KeyBinding::new(GameAction::MoveLeft, "A").with_secondary("Left"),
            KeyBinding::new(GameAction::MoveRight, "D").with_secondary("Right"),
            KeyBinding::new(GameAction::Sprint, "LShift"),
            KeyBinding::new(GameAction::Dodge, "Space"),
            KeyBinding::new(GameAction::Jump, "Space"),
            KeyBinding::new(GameAction::Crouch, "LCtrl"),
            // Combat
            KeyBinding::new(GameAction::Attack, "MouseLeft"),
            KeyBinding::new(GameAction::AttackSecondary, "MouseRight"),
            KeyBinding::new(GameAction::Block, "Q"),
            KeyBinding::new(GameAction::LockOn, "Tab"),
            KeyBinding::new(GameAction::SwitchTarget, "MouseMiddle"),
            KeyBinding::new(GameAction::UseItem, "R"),
            // Interaction
            KeyBinding::new(GameAction::Interact, "E"),
            KeyBinding::new(GameAction::PickUp, "F"),
            KeyBinding::new(GameAction::Drop, "G"),
            // UI
            KeyBinding::new(GameAction::Inventory, "I").with_secondary("Tab"),
            KeyBinding::new(GameAction::Map, "M"),
            KeyBinding::new(GameAction::Journal, "J"),
            KeyBinding::new(GameAction::Character, "C"),
            KeyBinding::new(GameAction::Crafting, "K"),
            KeyBinding::new(GameAction::Pause, "Escape"),
            KeyBinding::new(GameAction::QuickSave, "F5"),
            KeyBinding::new(GameAction::QuickLoad, "F9"),
            // Hotbar (1-0 keys)
            KeyBinding::new(GameAction::Hotbar1, "1"),
            KeyBinding::new(GameAction::Hotbar2, "2"),
            KeyBinding::new(GameAction::Hotbar3, "3"),
            KeyBinding::new(GameAction::Hotbar4, "4"),
            KeyBinding::new(GameAction::Hotbar5, "5"),
            KeyBinding::new(GameAction::Hotbar6, "6"),
            KeyBinding::new(GameAction::Hotbar7, "7"),
            KeyBinding::new(GameAction::Hotbar8, "8"),
            KeyBinding::new(GameAction::Hotbar9, "9"),
            KeyBinding::new(GameAction::Hotbar10, "0"),
            // Camera
            KeyBinding::new(GameAction::ZoomIn, "ScrollUp"),
            KeyBinding::new(GameAction::ZoomOut, "ScrollDown"),
            KeyBinding::new(GameAction::CameraLeft, "["),
            KeyBinding::new(GameAction::CameraRight, "]"),
            KeyBinding::new(GameAction::CameraReset, "Home"),
            // Debug
            KeyBinding::new(GameAction::DebugOverlay, "F3"),
            KeyBinding::new(GameAction::DebugConsole, "`"),
        ]
    }

    /// Resets all bindings to defaults.
    pub fn reset_to_defaults(&mut self) {
        self.bindings.clear();
        self.key_to_action.clear();

        for binding in Self::default_bindings() {
            self.add_binding_internal(binding);
        }

        self.dirty = true;
        info!("Key bindings reset to defaults");
    }

    /// Adds a binding without conflict checking (internal use).
    fn add_binding_internal(&mut self, binding: KeyBinding) {
        // Update reverse mapping
        self.key_to_action
            .insert(binding.primary.clone(), binding.action);
        if let Some(ref sec) = binding.secondary {
            self.key_to_action.insert(sec.clone(), binding.action);
        }

        self.bindings.insert(binding.action, binding);
    }

    /// Returns whether bindings have changed.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clears the dirty flag.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Returns all bindings.
    #[must_use]
    pub fn bindings(&self) -> &HashMap<GameAction, KeyBinding> {
        &self.bindings
    }

    /// Gets the binding for an action.
    #[must_use]
    pub fn get_binding(&self, action: GameAction) -> Option<&KeyBinding> {
        self.bindings.get(&action)
    }

    /// Gets the action bound to a key.
    #[must_use]
    pub fn get_action_for_key(&self, key: &str) -> Option<GameAction> {
        self.key_to_action.get(key).copied()
    }

    /// Returns the primary key for an action.
    #[must_use]
    pub fn get_primary_key(&self, action: GameAction) -> Option<&str> {
        self.bindings.get(&action).map(|b| b.primary.as_str())
    }

    /// Returns all actions triggered by a key.
    pub fn actions_for_key(&self, key: &str) -> Vec<GameAction> {
        self.bindings
            .values()
            .filter(|b| b.matches(key))
            .map(|b| b.action)
            .collect()
    }

    /// Returns current rebind mode.
    #[must_use]
    pub fn rebind_mode(&self) -> RebindMode {
        self.rebind_mode
    }

    /// Returns whether in rebind mode.
    #[must_use]
    pub fn is_rebinding(&self) -> bool {
        !matches!(self.rebind_mode, RebindMode::Inactive)
    }

    /// Starts rebinding the primary key for an action.
    pub fn start_rebind_primary(&mut self, action: GameAction) {
        self.rebind_mode = RebindMode::WaitingForPrimary(action);
        info!("Started rebinding primary key for {:?}", action);
    }

    /// Starts rebinding the secondary key for an action.
    pub fn start_rebind_secondary(&mut self, action: GameAction) {
        self.rebind_mode = RebindMode::WaitingForSecondary(action);
        info!("Started rebinding secondary key for {:?}", action);
    }

    /// Cancels the current rebind operation.
    pub fn cancel_rebind(&mut self) {
        self.rebind_mode = RebindMode::Inactive;
        debug!("Rebind cancelled");
    }

    /// Handles key input during rebind mode.
    /// Returns Ok(true) if rebind completed, Ok(false) if not in rebind mode.
    pub fn handle_rebind_input(&mut self, key: &str) -> RebindResult<bool> {
        match self.rebind_mode {
            RebindMode::Inactive => Ok(false),
            RebindMode::WaitingForPrimary(action) => {
                self.rebind_primary(action, key)?;
                self.rebind_mode = RebindMode::Inactive;
                Ok(true)
            }
            RebindMode::WaitingForSecondary(action) => {
                self.rebind_secondary(action, key)?;
                self.rebind_mode = RebindMode::Inactive;
                Ok(true)
            }
        }
    }

    /// Checks for key conflicts.
    fn check_conflict(&self, key: &str, exclude_action: GameAction) -> Option<GameAction> {
        self.key_to_action.get(key).copied().filter(|&a| a != exclude_action)
    }

    /// Rebinds the primary key for an action.
    pub fn rebind_primary(&mut self, action: GameAction, key: &str) -> RebindResult<()> {
        // Check for conflicts
        if let Some(conflicting) = self.check_conflict(key, action) {
            return Err(RebindError::Conflict {
                key: key.to_string(),
                action: conflicting,
            });
        }

        // Get or create binding
        let binding = self.bindings.entry(action).or_insert_with(|| KeyBinding::new(action, key));

        // Remove old key from reverse mapping
        self.key_to_action.remove(&binding.primary);

        // Update binding
        binding.primary = key.to_string();

        // Update reverse mapping
        self.key_to_action.insert(key.to_string(), action);

        self.dirty = true;
        info!("Rebound {:?} primary to {}", action, key);
        Ok(())
    }

    /// Rebinds the secondary key for an action.
    pub fn rebind_secondary(&mut self, action: GameAction, key: &str) -> RebindResult<()> {
        // Check for conflicts
        if let Some(conflicting) = self.check_conflict(key, action) {
            return Err(RebindError::Conflict {
                key: key.to_string(),
                action: conflicting,
            });
        }

        let binding = self
            .bindings
            .get_mut(&action)
            .ok_or(RebindError::ActionNotFound(action))?;

        // Remove old secondary key from reverse mapping
        if let Some(ref old_sec) = binding.secondary {
            self.key_to_action.remove(old_sec);
        }

        // Update binding
        binding.secondary = Some(key.to_string());

        // Update reverse mapping
        self.key_to_action.insert(key.to_string(), action);

        self.dirty = true;
        info!("Rebound {:?} secondary to {}", action, key);
        Ok(())
    }

    /// Clears the secondary key for an action.
    pub fn clear_secondary(&mut self, action: GameAction) -> RebindResult<()> {
        let binding = self
            .bindings
            .get_mut(&action)
            .ok_or(RebindError::ActionNotFound(action))?;

        if let Some(ref sec) = binding.secondary {
            self.key_to_action.remove(sec);
        }

        binding.secondary = None;
        self.dirty = true;
        info!("Cleared {:?} secondary binding", action);
        Ok(())
    }

    /// Unbinds an action completely.
    pub fn unbind(&mut self, action: GameAction) -> RebindResult<()> {
        if action.is_required() {
            return Err(RebindError::RequiredAction(action));
        }

        if let Some(binding) = self.bindings.remove(&action) {
            self.key_to_action.remove(&binding.primary);
            if let Some(ref sec) = binding.secondary {
                self.key_to_action.remove(sec);
            }
            self.dirty = true;
            info!("Unbound {:?}", action);
        }

        Ok(())
    }

    /// Swaps bindings between two keys.
    pub fn swap_binding(&mut self, key: &str, with_key: &str) {
        let action1 = self.key_to_action.get(key).copied();
        let action2 = self.key_to_action.get(with_key).copied();

        if let Some(a1) = action1 {
            self.key_to_action.insert(with_key.to_string(), a1);
            if let Some(binding) = self.bindings.get_mut(&a1) {
                if binding.primary == key {
                    binding.primary = with_key.to_string();
                } else if binding.secondary.as_deref() == Some(key) {
                    binding.secondary = Some(with_key.to_string());
                }
            }
        }

        if let Some(a2) = action2 {
            self.key_to_action.insert(key.to_string(), a2);
            if let Some(binding) = self.bindings.get_mut(&a2) {
                if binding.primary == with_key {
                    binding.primary = key.to_string();
                } else if binding.secondary.as_deref() == Some(with_key) {
                    binding.secondary = Some(key.to_string());
                }
            }
        }

        self.dirty = true;
    }

    /// Exports bindings to a map for settings persistence.
    #[must_use]
    pub fn export_to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (action, binding) in &self.bindings {
            let key = format!("{action:?}");
            let value = if let Some(ref sec) = binding.secondary {
                format!("{},{}", binding.primary, sec)
            } else {
                binding.primary.clone()
            };
            map.insert(key, value);
        }
        map
    }

    /// Imports bindings from a settings map.
    pub fn import_from_map(&mut self, map: &HashMap<String, String>) {
        for (action_str, keys_str) in map {
            // Parse action from string (simplified - would use proper parsing)
            let action = match action_str.as_str() {
                "MoveUp" => Some(GameAction::MoveUp),
                "MoveDown" => Some(GameAction::MoveDown),
                "MoveLeft" => Some(GameAction::MoveLeft),
                "MoveRight" => Some(GameAction::MoveRight),
                "Sprint" => Some(GameAction::Sprint),
                "Attack" => Some(GameAction::Attack),
                "Interact" => Some(GameAction::Interact),
                "Inventory" => Some(GameAction::Inventory),
                "Pause" => Some(GameAction::Pause),
                _ => None,
            };

            if let Some(action) = action {
                let parts: Vec<&str> = keys_str.split(',').collect();
                if !parts.is_empty() {
                    let binding = if parts.len() > 1 {
                        KeyBinding::new(action, parts[0]).with_secondary(parts[1])
                    } else {
                        KeyBinding::new(action, parts[0])
                    };
                    self.add_binding_internal(binding);
                }
            }
        }
        self.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_action_display_name() {
        assert_eq!(GameAction::MoveUp.display_name(), "Move Up");
        assert_eq!(GameAction::Attack.display_name(), "Attack");
    }

    #[test]
    fn test_game_action_category() {
        assert_eq!(GameAction::MoveUp.category(), ActionCategory::Movement);
        assert_eq!(GameAction::Attack.category(), ActionCategory::Combat);
        assert_eq!(GameAction::Inventory.category(), ActionCategory::UI);
    }

    #[test]
    fn test_game_action_is_required() {
        assert!(GameAction::Pause.is_required());
        assert!(GameAction::MoveUp.is_required());
        assert!(!GameAction::Attack.is_required());
    }

    #[test]
    fn test_key_binding_new() {
        let binding = KeyBinding::new(GameAction::Attack, "MouseLeft");
        assert_eq!(binding.action, GameAction::Attack);
        assert_eq!(binding.primary, "MouseLeft");
        assert!(binding.secondary.is_none());
    }

    #[test]
    fn test_key_binding_with_secondary() {
        let binding = KeyBinding::new(GameAction::MoveUp, "W").with_secondary("Up");
        assert_eq!(binding.secondary, Some("Up".to_string()));
    }

    #[test]
    fn test_key_binding_matches() {
        let binding = KeyBinding::new(GameAction::MoveUp, "W").with_secondary("Up");
        assert!(binding.matches("W"));
        assert!(binding.matches("Up"));
        assert!(!binding.matches("S"));
    }

    #[test]
    fn test_rebind_manager_new() {
        let manager = InputRebindManager::new();
        assert!(!manager.bindings().is_empty());
        assert!(manager.get_binding(GameAction::MoveUp).is_some());
    }

    #[test]
    fn test_rebind_manager_get_action_for_key() {
        let manager = InputRebindManager::new();
        assert_eq!(manager.get_action_for_key("W"), Some(GameAction::MoveUp));
    }

    #[test]
    fn test_rebind_manager_rebind_primary() {
        let mut manager = InputRebindManager::new();

        // Find an unbound key
        assert!(manager.rebind_primary(GameAction::Sprint, "X").is_ok());
        assert_eq!(manager.get_primary_key(GameAction::Sprint), Some("X"));
    }

    #[test]
    fn test_rebind_manager_conflict_detection() {
        let mut manager = InputRebindManager::new();

        // W is bound to MoveUp by default
        let result = manager.rebind_primary(GameAction::Sprint, "W");
        assert!(matches!(result, Err(RebindError::Conflict { .. })));
    }

    #[test]
    fn test_rebind_manager_unbind_required() {
        let mut manager = InputRebindManager::new();

        let result = manager.unbind(GameAction::Pause);
        assert!(matches!(result, Err(RebindError::RequiredAction(_))));
    }

    #[test]
    fn test_rebind_manager_rebind_mode() {
        let mut manager = InputRebindManager::new();

        assert!(!manager.is_rebinding());

        manager.start_rebind_primary(GameAction::Attack);
        assert!(manager.is_rebinding());
        assert!(matches!(
            manager.rebind_mode(),
            RebindMode::WaitingForPrimary(GameAction::Attack)
        ));

        manager.cancel_rebind();
        assert!(!manager.is_rebinding());
    }

    #[test]
    fn test_rebind_manager_handle_rebind_input() {
        let mut manager = InputRebindManager::new();

        // Not in rebind mode
        assert!(manager.handle_rebind_input("X").is_ok());
        assert!(!manager.handle_rebind_input("X").expect("failed"));

        // Start rebind
        manager.start_rebind_primary(GameAction::UseItem);
        assert!(manager.handle_rebind_input("X").expect("failed"));
        assert!(!manager.is_rebinding());
    }

    #[test]
    fn test_rebind_manager_reset() {
        let mut manager = InputRebindManager::new();
        let _ = manager.rebind_primary(GameAction::Sprint, "X");

        manager.reset_to_defaults();

        assert_eq!(
            manager.get_primary_key(GameAction::Sprint),
            Some("LShift")
        );
    }

    #[test]
    fn test_rebind_manager_export_import() {
        let manager = InputRebindManager::new();
        let exported = manager.export_to_map();

        let mut manager2 = InputRebindManager::new();
        manager2.bindings.clear();
        manager2.key_to_action.clear();
        manager2.import_from_map(&exported);

        // Check some bindings were imported
        assert!(manager2.get_binding(GameAction::MoveUp).is_some());
    }

    #[test]
    fn test_action_category_display() {
        assert_eq!(ActionCategory::Movement.display_name(), "Movement");
        assert_eq!(ActionCategory::Combat.display_name(), "Combat");
    }

    #[test]
    fn test_actions_in_category() {
        let movement_actions = GameAction::actions_in_category(ActionCategory::Movement);
        assert!(movement_actions.contains(&GameAction::MoveUp));
        assert!(!movement_actions.contains(&GameAction::Attack));
    }

    #[test]
    fn test_rebind_error_display() {
        let err = RebindError::Conflict {
            key: "W".to_string(),
            action: GameAction::MoveUp,
        };
        let msg = format!("{err}");
        assert!(msg.contains("already bound"));
    }
}
