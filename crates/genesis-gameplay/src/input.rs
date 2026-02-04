//! Input handling system for player controls.
//!
//! This module provides input abstraction with keyboard/mouse state tracking,
//! action mapping, and rebindable controls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur in the input system.
#[derive(Debug, Clone, Error)]
pub enum InputError {
    /// Action not found in bindings
    #[error("action not bound: {0}")]
    ActionNotBound(String),

    /// Key already bound to another action
    #[error("key {key:?} already bound to action: {action}")]
    KeyAlreadyBound {
        /// The key that's already bound
        key: KeyCode,
        /// The action it's bound to
        action: String,
    },
}

/// 2D vector for positions and directions.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
}

impl Vec2 {
    /// Zero vector.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Unit vector pointing up.
    pub const UP: Self = Self { x: 0.0, y: -1.0 };

    /// Unit vector pointing down.
    pub const DOWN: Self = Self { x: 0.0, y: 1.0 };

    /// Unit vector pointing left.
    pub const LEFT: Self = Self { x: -1.0, y: 0.0 };

    /// Unit vector pointing right.
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0 };

    /// Creates a new Vec2.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the length (magnitude) of the vector.
    #[must_use]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns a normalized (unit length) version of the vector.
    /// Returns zero vector if the vector has zero length.
    #[must_use]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }

    /// Dot product of two vectors.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Scale the vector by a scalar.
    #[must_use]
    pub fn scale(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    /// Add two vectors.
    #[must_use]
    pub fn plus(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Subtract two vectors.
    #[must_use]
    pub fn minus(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// Distance between two points.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        self.minus(other).length()
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.plus(rhs)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.minus(rhs)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.scale(rhs)
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

/// Key codes for keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Letter keys A-Z
    A,
    /// B key
    B,
    /// C key
    C,
    /// D key
    D,
    /// E key
    E,
    /// F key
    F,
    /// G key
    G,
    /// H key
    H,
    /// I key
    I,
    /// J key
    J,
    /// K key
    K,
    /// L key
    L,
    /// M key
    M,
    /// N key
    N,
    /// O key
    O,
    /// P key
    P,
    /// Q key
    Q,
    /// R key
    R,
    /// S key
    S,
    /// T key
    T,
    /// U key
    U,
    /// V key
    V,
    /// W key
    W,
    /// X key
    X,
    /// Y key
    Y,
    /// Z key
    Z,
    /// Number keys 0-9
    Num0,
    /// Num1 key
    Num1,
    /// Num2 key
    Num2,
    /// Num3 key
    Num3,
    /// Num4 key
    Num4,
    /// Num5 key
    Num5,
    /// Num6 key
    Num6,
    /// Num7 key
    Num7,
    /// Num8 key
    Num8,
    /// Num9 key
    Num9,
    /// Space bar
    Space,
    /// Enter/Return
    Enter,
    /// Escape
    Escape,
    /// Left Shift
    LShift,
    /// Right Shift
    RShift,
    /// Left Control
    LCtrl,
    /// Right Control
    RCtrl,
    /// Left Alt
    LAlt,
    /// Right Alt
    RAlt,
    /// Tab
    Tab,
    /// Arrow keys
    Up,
    /// Down arrow
    Down,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
}

/// Mouse button codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button (scroll wheel click)
    Middle,
}

/// State of a button (pressed, just pressed, released).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ButtonState {
    /// Whether the button is currently held down
    pub pressed: bool,
    /// Whether the button was just pressed this frame
    pub just_pressed: bool,
    /// Whether the button was just released this frame
    pub just_released: bool,
}

impl ButtonState {
    /// Create a new button state (not pressed).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pressed: false,
            just_pressed: false,
            just_released: false,
        }
    }

    /// Update the button state based on whether it's currently pressed.
    pub fn update(&mut self, is_pressed: bool) {
        self.just_pressed = is_pressed && !self.pressed;
        self.just_released = !is_pressed && self.pressed;
        self.pressed = is_pressed;
    }

    /// Clear the frame-specific state (just_pressed, just_released).
    pub fn clear_frame(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
    }
}

/// Game actions that can be bound to keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Move up (W key by default)
    MoveUp,
    /// Move down (S key by default)
    MoveDown,
    /// Move left (A key by default)
    MoveLeft,
    /// Move right (D key by default)
    MoveRight,
    /// Jump (Space by default)
    Jump,
    /// Run/sprint (Shift by default)
    Run,
    /// Interact with world (E by default)
    Interact,
    /// Open inventory (I by default)
    Inventory,
    /// Pause/menu (Escape by default)
    Pause,
}

/// Processed input state for gameplay use.
#[derive(Debug, Clone, Default)]
pub struct Input {
    /// Movement direction (-1 to 1 on each axis)
    pub movement: Vec2,
    /// Whether jump is pressed
    pub jump: bool,
    /// Whether jump was just pressed this frame
    pub jump_just_pressed: bool,
    /// Whether interact is pressed
    pub interact: bool,
    /// Whether interact was just pressed this frame
    pub interact_just_pressed: bool,
    /// Whether running (sprint) is pressed
    pub running: bool,
    /// Primary action (left click / dig)
    pub primary_action: bool,
    /// Primary action was just pressed
    pub primary_action_just_pressed: bool,
    /// Secondary action (right click / place)
    pub secondary_action: bool,
    /// Secondary action was just pressed
    pub secondary_action_just_pressed: bool,
    /// Mouse position in screen coordinates
    pub mouse_screen_pos: Vec2,
    /// Mouse position in world coordinates
    pub mouse_world_pos: Vec2,
}

impl Input {
    /// Create a new input state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any movement input is active.
    #[must_use]
    pub fn has_movement(&self) -> bool {
        self.movement.x != 0.0 || self.movement.y != 0.0
    }

    /// Returns the movement direction as a tuple (-1 to 1 for x, y).
    ///
    /// This is the primary method for getting player movement input.
    #[must_use]
    pub fn move_direction(&self) -> (f32, f32) {
        (self.movement.x, self.movement.y)
    }

    /// Check if jump was just pressed this frame.
    #[must_use]
    pub fn jump_pressed(&self) -> bool {
        self.jump_just_pressed
    }

    /// Check if the primary action key is held.
    #[must_use]
    pub fn action_held(&self) -> bool {
        self.primary_action
    }

    /// Check if the secondary action key is held.
    #[must_use]
    pub fn secondary_action_held(&self) -> bool {
        self.secondary_action
    }

    /// Check if interact was just pressed.
    #[must_use]
    pub fn interact_pressed(&self) -> bool {
        self.interact_just_pressed
    }
}

/// Type alias for Input - used for API compatibility.
///
/// `InputState` is the name used by the engine integration layer.
pub type InputState = Input;

/// Key binding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Primary key for this action
    pub primary: KeyCode,
    /// Optional secondary key
    pub secondary: Option<KeyCode>,
}

impl KeyBinding {
    /// Create a new key binding with only a primary key.
    #[must_use]
    pub const fn new(primary: KeyCode) -> Self {
        Self {
            primary,
            secondary: None,
        }
    }

    /// Create a new key binding with primary and secondary keys.
    #[must_use]
    pub const fn with_secondary(primary: KeyCode, secondary: KeyCode) -> Self {
        Self {
            primary,
            secondary: Some(secondary),
        }
    }

    /// Check if a key matches this binding.
    #[must_use]
    pub fn matches(&self, key: KeyCode) -> bool {
        self.primary == key || self.secondary == Some(key)
    }
}

/// Input manager that handles raw input and converts to game actions.
#[derive(Debug)]
pub struct InputManager {
    /// Current key states
    key_states: HashMap<KeyCode, ButtonState>,
    /// Current mouse button states
    mouse_states: HashMap<MouseButton, ButtonState>,
    /// Action to key bindings
    bindings: HashMap<Action, KeyBinding>,
    /// Current mouse position (screen space)
    mouse_screen_pos: Vec2,
    /// Current mouse position (world space)
    mouse_world_pos: Vec2,
    /// Camera offset for screen-to-world conversion
    camera_offset: Vec2,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    /// Create a new input manager with default bindings.
    #[must_use]
    pub fn new() -> Self {
        let mut manager = Self {
            key_states: HashMap::new(),
            mouse_states: HashMap::new(),
            bindings: HashMap::new(),
            mouse_screen_pos: Vec2::ZERO,
            mouse_world_pos: Vec2::ZERO,
            camera_offset: Vec2::ZERO,
        };
        manager.set_default_bindings();
        manager
    }

    /// Set default key bindings.
    pub fn set_default_bindings(&mut self) {
        self.bindings.clear();
        self.bindings.insert(
            Action::MoveUp,
            KeyBinding::with_secondary(KeyCode::W, KeyCode::Up),
        );
        self.bindings.insert(
            Action::MoveDown,
            KeyBinding::with_secondary(KeyCode::S, KeyCode::Down),
        );
        self.bindings.insert(
            Action::MoveLeft,
            KeyBinding::with_secondary(KeyCode::A, KeyCode::Left),
        );
        self.bindings.insert(
            Action::MoveRight,
            KeyBinding::with_secondary(KeyCode::D, KeyCode::Right),
        );
        self.bindings
            .insert(Action::Jump, KeyBinding::new(KeyCode::Space));
        self.bindings
            .insert(Action::Run, KeyBinding::new(KeyCode::LShift));
        self.bindings
            .insert(Action::Interact, KeyBinding::new(KeyCode::E));
        self.bindings
            .insert(Action::Inventory, KeyBinding::new(KeyCode::I));
        self.bindings
            .insert(Action::Pause, KeyBinding::new(KeyCode::Escape));
    }

    /// Rebind an action to a new key.
    pub fn rebind(&mut self, action: Action, binding: KeyBinding) {
        self.bindings.insert(action, binding);
    }

    /// Get the current binding for an action.
    #[must_use]
    pub fn get_binding(&self, action: Action) -> Option<&KeyBinding> {
        self.bindings.get(&action)
    }

    /// Update a key state.
    pub fn update_key(&mut self, key: KeyCode, is_pressed: bool) {
        self.key_states.entry(key).or_default().update(is_pressed);
    }

    /// Update a mouse button state.
    pub fn update_mouse_button(&mut self, button: MouseButton, is_pressed: bool) {
        self.mouse_states
            .entry(button)
            .or_default()
            .update(is_pressed);
    }

    /// Update mouse position.
    pub fn update_mouse_position(&mut self, screen_x: f32, screen_y: f32) {
        self.mouse_screen_pos = Vec2::new(screen_x, screen_y);
        // Convert to world space using camera offset
        self.mouse_world_pos = self.mouse_screen_pos + self.camera_offset;
    }

    /// Set camera offset for screen-to-world conversion.
    pub fn set_camera_offset(&mut self, offset: Vec2) {
        self.camera_offset = offset;
        // Recalculate world position
        self.mouse_world_pos = self.mouse_screen_pos + self.camera_offset;
    }

    /// Clear frame-specific state. Call at the end of each frame.
    pub fn end_frame(&mut self) {
        for state in self.key_states.values_mut() {
            state.clear_frame();
        }
        for state in self.mouse_states.values_mut() {
            state.clear_frame();
        }
    }

    /// Check if a key is currently pressed.
    #[must_use]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.key_states.get(&key).is_some_and(|state| state.pressed)
    }

    /// Check if a key was just pressed this frame.
    #[must_use]
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.key_states
            .get(&key)
            .is_some_and(|state| state.just_pressed)
    }

    /// Check if a key was just released this frame.
    #[must_use]
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.key_states
            .get(&key)
            .is_some_and(|state| state.just_released)
    }

    /// Check if a mouse button is currently pressed.
    #[must_use]
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_states
            .get(&button)
            .is_some_and(|state| state.pressed)
    }

    /// Check if a mouse button was just pressed this frame.
    #[must_use]
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_states
            .get(&button)
            .is_some_and(|state| state.just_pressed)
    }

    /// Check if an action is currently active.
    #[must_use]
    pub fn is_action_pressed(&self, action: Action) -> bool {
        self.bindings.get(&action).is_some_and(|binding| {
            self.is_key_pressed(binding.primary)
                || binding
                    .secondary
                    .is_some_and(|key| self.is_key_pressed(key))
        })
    }

    /// Check if an action was just pressed this frame.
    #[must_use]
    pub fn is_action_just_pressed(&self, action: Action) -> bool {
        self.bindings.get(&action).is_some_and(|binding| {
            self.is_key_just_pressed(binding.primary)
                || binding
                    .secondary
                    .is_some_and(|key| self.is_key_just_pressed(key))
        })
    }

    /// Process raw input into game-ready Input struct.
    #[must_use]
    pub fn process(&self) -> Input {
        let mut movement = Vec2::ZERO;

        if self.is_action_pressed(Action::MoveUp) {
            movement.y -= 1.0;
        }
        if self.is_action_pressed(Action::MoveDown) {
            movement.y += 1.0;
        }
        if self.is_action_pressed(Action::MoveLeft) {
            movement.x -= 1.0;
        }
        if self.is_action_pressed(Action::MoveRight) {
            movement.x += 1.0;
        }

        // Normalize diagonal movement
        if movement.length() > 1.0 {
            movement = movement.normalized();
        }

        Input {
            movement,
            jump: self.is_action_pressed(Action::Jump),
            jump_just_pressed: self.is_action_just_pressed(Action::Jump),
            interact: self.is_action_pressed(Action::Interact),
            interact_just_pressed: self.is_action_just_pressed(Action::Interact),
            running: self.is_action_pressed(Action::Run),
            primary_action: self.is_mouse_pressed(MouseButton::Left),
            primary_action_just_pressed: self.is_mouse_just_pressed(MouseButton::Left),
            secondary_action: self.is_mouse_pressed(MouseButton::Right),
            secondary_action_just_pressed: self.is_mouse_just_pressed(MouseButton::Right),
            mouse_screen_pos: self.mouse_screen_pos,
            mouse_world_pos: self.mouse_world_pos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(3.0, 4.0);
        let b = Vec2::new(1.0, 2.0);

        assert_eq!(a.plus(b), Vec2::new(4.0, 6.0));
        assert_eq!(a.minus(b), Vec2::new(2.0, 2.0));
        assert_eq!(a.scale(2.0), Vec2::new(6.0, 8.0));
        assert!((a.length() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_vec2_normalized() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalized();
        assert!((n.length() - 1.0).abs() < 0.001);

        let zero = Vec2::ZERO;
        assert_eq!(zero.normalized(), Vec2::ZERO);
    }

    #[test]
    fn test_button_state() {
        let mut state = ButtonState::new();
        assert!(!state.pressed);
        assert!(!state.just_pressed);

        // Press the button
        state.update(true);
        assert!(state.pressed);
        assert!(state.just_pressed);
        assert!(!state.just_released);

        // Hold the button
        state.clear_frame();
        state.update(true);
        assert!(state.pressed);
        assert!(!state.just_pressed);

        // Release the button
        state.clear_frame();
        state.update(false);
        assert!(!state.pressed);
        assert!(!state.just_pressed);
        assert!(state.just_released);
    }

    #[test]
    fn test_input_manager_default_bindings() {
        let manager = InputManager::new();

        assert!(manager.get_binding(Action::MoveUp).is_some());
        assert!(manager.get_binding(Action::Jump).is_some());
        assert_eq!(
            manager.get_binding(Action::MoveUp).map(|b| b.primary),
            Some(KeyCode::W)
        );
    }

    #[test]
    fn test_input_manager_key_state() {
        let mut manager = InputManager::new();

        manager.update_key(KeyCode::W, true);
        assert!(manager.is_key_pressed(KeyCode::W));
        assert!(manager.is_key_just_pressed(KeyCode::W));

        manager.end_frame();
        manager.update_key(KeyCode::W, true);
        assert!(manager.is_key_pressed(KeyCode::W));
        assert!(!manager.is_key_just_pressed(KeyCode::W));

        manager.end_frame();
        manager.update_key(KeyCode::W, false);
        assert!(!manager.is_key_pressed(KeyCode::W));
        assert!(manager.is_key_just_released(KeyCode::W));
    }

    #[test]
    fn test_input_manager_mouse_state() {
        let mut manager = InputManager::new();

        manager.update_mouse_button(MouseButton::Left, true);
        assert!(manager.is_mouse_pressed(MouseButton::Left));
        assert!(manager.is_mouse_just_pressed(MouseButton::Left));
    }

    #[test]
    fn test_input_manager_action_state() {
        let mut manager = InputManager::new();

        // Test with primary key
        manager.update_key(KeyCode::W, true);
        assert!(manager.is_action_pressed(Action::MoveUp));

        manager.end_frame();
        manager.update_key(KeyCode::W, false);

        // Test with secondary key
        manager.update_key(KeyCode::Up, true);
        assert!(manager.is_action_pressed(Action::MoveUp));
    }

    #[test]
    fn test_input_manager_rebind() {
        let mut manager = InputManager::new();

        manager.rebind(Action::Jump, KeyBinding::new(KeyCode::W));
        manager.update_key(KeyCode::W, true);
        assert!(manager.is_action_pressed(Action::Jump));

        // Space no longer works for jump
        manager.end_frame();
        manager.update_key(KeyCode::W, false);
        manager.update_key(KeyCode::Space, true);
        assert!(!manager.is_action_pressed(Action::Jump));
    }

    #[test]
    fn test_input_manager_process() {
        let mut manager = InputManager::new();

        manager.update_key(KeyCode::W, true);
        manager.update_key(KeyCode::D, true);
        manager.update_key(KeyCode::LShift, true);
        manager.update_mouse_button(MouseButton::Left, true);

        let input = manager.process();

        // Diagonal movement should be normalized
        assert!(input.movement.length() <= 1.01);
        assert!(input.movement.y < 0.0); // Moving up
        assert!(input.movement.x > 0.0); // Moving right
        assert!(input.running);
        assert!(input.primary_action);
        assert!(input.primary_action_just_pressed);
    }

    #[test]
    fn test_input_manager_mouse_position() {
        let mut manager = InputManager::new();

        manager.update_mouse_position(100.0, 200.0);
        manager.set_camera_offset(Vec2::new(50.0, 50.0));

        let input = manager.process();
        assert_eq!(input.mouse_screen_pos.x, 100.0);
        assert_eq!(input.mouse_screen_pos.y, 200.0);
        assert_eq!(input.mouse_world_pos.x, 150.0);
        assert_eq!(input.mouse_world_pos.y, 250.0);
    }

    #[test]
    fn test_input_has_movement() {
        let mut input = Input::new();
        assert!(!input.has_movement());

        input.movement = Vec2::new(1.0, 0.0);
        assert!(input.has_movement());
    }

    #[test]
    fn test_vec2_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.distance(b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_vec2_dot() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert_eq!(a.dot(b), 0.0);

        let c = Vec2::new(1.0, 2.0);
        let d = Vec2::new(3.0, 4.0);
        assert_eq!(c.dot(d), 11.0);
    }

    #[test]
    fn test_vec2_operators() {
        let mut v = Vec2::new(1.0, 2.0);

        v += Vec2::new(1.0, 1.0);
        assert_eq!(v, Vec2::new(2.0, 3.0));

        v -= Vec2::new(1.0, 1.0);
        assert_eq!(v, Vec2::new(1.0, 2.0));

        v *= 2.0;
        assert_eq!(v, Vec2::new(2.0, 4.0));

        let result = v + Vec2::new(1.0, 1.0);
        assert_eq!(result, Vec2::new(3.0, 5.0));

        let result = v - Vec2::new(1.0, 1.0);
        assert_eq!(result, Vec2::new(1.0, 3.0));

        let result = v * 0.5;
        assert_eq!(result, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn test_key_binding_matches() {
        let binding = KeyBinding::with_secondary(KeyCode::W, KeyCode::Up);
        assert!(binding.matches(KeyCode::W));
        assert!(binding.matches(KeyCode::Up));
        assert!(!binding.matches(KeyCode::S));
    }
}
