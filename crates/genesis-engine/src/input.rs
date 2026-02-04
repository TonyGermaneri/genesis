//! Input handling for the engine.
//!
//! Bridges winit window events to the gameplay input system.

use std::collections::HashSet;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};

use genesis_gameplay::input::{Input, InputManager, KeyCode, MouseButton, Vec2};

/// Converts winit KeyCode to gameplay KeyCode
fn convert_key(key: WinitKeyCode) -> Option<KeyCode> {
    Some(match key {
        WinitKeyCode::KeyA => KeyCode::A,
        WinitKeyCode::KeyB => KeyCode::B,
        WinitKeyCode::KeyC => KeyCode::C,
        WinitKeyCode::KeyD => KeyCode::D,
        WinitKeyCode::KeyE => KeyCode::E,
        WinitKeyCode::KeyF => KeyCode::F,
        WinitKeyCode::KeyG => KeyCode::G,
        WinitKeyCode::KeyH => KeyCode::H,
        WinitKeyCode::KeyI => KeyCode::I,
        WinitKeyCode::KeyJ => KeyCode::J,
        WinitKeyCode::KeyK => KeyCode::K,
        WinitKeyCode::KeyL => KeyCode::L,
        WinitKeyCode::KeyM => KeyCode::M,
        WinitKeyCode::KeyN => KeyCode::N,
        WinitKeyCode::KeyO => KeyCode::O,
        WinitKeyCode::KeyP => KeyCode::P,
        WinitKeyCode::KeyQ => KeyCode::Q,
        WinitKeyCode::KeyR => KeyCode::R,
        WinitKeyCode::KeyS => KeyCode::S,
        WinitKeyCode::KeyT => KeyCode::T,
        WinitKeyCode::KeyU => KeyCode::U,
        WinitKeyCode::KeyV => KeyCode::V,
        WinitKeyCode::KeyW => KeyCode::W,
        WinitKeyCode::KeyX => KeyCode::X,
        WinitKeyCode::KeyY => KeyCode::Y,
        WinitKeyCode::KeyZ => KeyCode::Z,
        WinitKeyCode::Digit0 => KeyCode::Num0,
        WinitKeyCode::Digit1 => KeyCode::Num1,
        WinitKeyCode::Digit2 => KeyCode::Num2,
        WinitKeyCode::Digit3 => KeyCode::Num3,
        WinitKeyCode::Digit4 => KeyCode::Num4,
        WinitKeyCode::Digit5 => KeyCode::Num5,
        WinitKeyCode::Digit6 => KeyCode::Num6,
        WinitKeyCode::Digit7 => KeyCode::Num7,
        WinitKeyCode::Digit8 => KeyCode::Num8,
        WinitKeyCode::Digit9 => KeyCode::Num9,
        WinitKeyCode::Space => KeyCode::Space,
        WinitKeyCode::Enter => KeyCode::Enter,
        WinitKeyCode::Escape => KeyCode::Escape,
        WinitKeyCode::ShiftLeft => KeyCode::LShift,
        WinitKeyCode::ShiftRight => KeyCode::RShift,
        WinitKeyCode::ControlLeft => KeyCode::LCtrl,
        WinitKeyCode::ControlRight => KeyCode::RCtrl,
        WinitKeyCode::AltLeft => KeyCode::LAlt,
        WinitKeyCode::AltRight => KeyCode::RAlt,
        WinitKeyCode::Tab => KeyCode::Tab,
        WinitKeyCode::ArrowUp => KeyCode::Up,
        WinitKeyCode::ArrowDown => KeyCode::Down,
        WinitKeyCode::ArrowLeft => KeyCode::Left,
        WinitKeyCode::ArrowRight => KeyCode::Right,
        _ => return None,
    })
}

/// Converts winit MouseButton to gameplay MouseButton
fn convert_mouse_button(button: WinitMouseButton) -> Option<MouseButton> {
    Some(match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        _ => return None,
    })
}

/// Handles input from winit and provides processed input for gameplay.
#[derive(Debug)]
pub struct InputHandler {
    /// The underlying input manager from gameplay crate
    manager: InputManager,
    /// Keys that were just pressed this frame (for edge detection)
    just_pressed_keys: HashSet<KeyCode>,
    /// Keys that were just released this frame
    just_released_keys: HashSet<KeyCode>,
    /// Mouse position in screen coordinates
    mouse_position: (f32, f32),
    /// Whether F3 (debug overlay) was just pressed
    debug_toggle_pressed: bool,
    /// Current hotbar selection from number keys (0-9, where 0 = slot 10)
    hotbar_selection: Option<u8>,
    /// Whether pause was just pressed
    pause_pressed: bool,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl InputHandler {
    /// Create a new input handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: InputManager::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
            mouse_position: (0.0, 0.0),
            debug_toggle_pressed: false,
            hotbar_selection: None,
            pause_pressed: false,
        }
    }

    /// Handle a winit window event. Returns true if the event was handled.
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(winit_key) = event.physical_key {
                    let is_pressed = event.state == ElementState::Pressed;

                    // Handle F3 for debug overlay
                    if winit_key == WinitKeyCode::F3 && is_pressed && !event.repeat {
                        self.debug_toggle_pressed = true;
                    }

                    // Handle number keys for hotbar (only on press, not repeat)
                    if is_pressed && !event.repeat {
                        self.hotbar_selection = match winit_key {
                            WinitKeyCode::Digit1 => Some(0),
                            WinitKeyCode::Digit2 => Some(1),
                            WinitKeyCode::Digit3 => Some(2),
                            WinitKeyCode::Digit4 => Some(3),
                            WinitKeyCode::Digit5 => Some(4),
                            WinitKeyCode::Digit6 => Some(5),
                            WinitKeyCode::Digit7 => Some(6),
                            WinitKeyCode::Digit8 => Some(7),
                            WinitKeyCode::Digit9 => Some(8),
                            WinitKeyCode::Digit0 => Some(9),
                            _ => self.hotbar_selection,
                        };
                    }

                    // Convert to gameplay key code
                    if let Some(key) = convert_key(winit_key) {
                        // Track just pressed/released for edge detection
                        if is_pressed && !event.repeat {
                            self.just_pressed_keys.insert(key);
                        } else if !is_pressed {
                            self.just_released_keys.insert(key);
                        }

                        // Update the input manager
                        self.manager.update_key(key, is_pressed);

                        // Check for pause (escape)
                        if key == KeyCode::Escape && is_pressed && !event.repeat {
                            self.pause_pressed = true;
                        }
                    }
                }
                true
            },
            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = *state == ElementState::Pressed;
                if let Some(btn) = convert_mouse_button(*button) {
                    self.manager.update_mouse_button(btn, is_pressed);
                }
                true
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
                self.manager
                    .update_mouse_position(position.x as f32, position.y as f32);
                true
            },
            _ => false,
        }
    }

    /// Get the processed input state for gameplay.
    #[must_use]
    pub fn get_input(&self) -> Input {
        self.manager.process()
    }

    /// Get the underlying input manager (for direct access to bindings).
    #[must_use]
    pub fn manager(&self) -> &InputManager {
        &self.manager
    }

    /// Get mutable access to the input manager (for rebinding).
    pub fn manager_mut(&mut self) -> &mut InputManager {
        &mut self.manager
    }

    /// Check if a key is currently held.
    #[must_use]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.manager.is_key_pressed(key)
    }

    /// Check if a key was just pressed this frame.
    #[must_use]
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    /// Check if a key was just released this frame.
    #[must_use]
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released_keys.contains(&key)
    }

    /// Get the current mouse position in screen coordinates.
    #[must_use]
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// Check if debug overlay was toggled (F3).
    #[must_use]
    pub fn debug_toggle_pressed(&self) -> bool {
        self.debug_toggle_pressed
    }

    /// Get the hotbar slot that was selected via number keys (if any).
    #[must_use]
    pub fn hotbar_selection(&self) -> Option<u8> {
        self.hotbar_selection
    }

    /// Check if pause was just pressed.
    #[must_use]
    pub fn pause_pressed(&self) -> bool {
        self.pause_pressed
    }

    /// Reset per-frame state. Call at the end of each frame.
    pub fn end_frame(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.manager.end_frame();
        self.debug_toggle_pressed = false;
        self.hotbar_selection = None;
        self.pause_pressed = false;
    }

    /// Update the camera offset for screen-to-world coordinate conversion.
    pub fn set_camera_offset(&mut self, x: f32, y: f32) {
        self.manager.set_camera_offset(Vec2::new(x, y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_handler_creation() {
        let handler = InputHandler::new();
        assert_eq!(handler.mouse_position(), (0.0, 0.0));
        assert!(!handler.debug_toggle_pressed());
        assert!(handler.hotbar_selection().is_none());
    }

    #[test]
    fn test_key_conversion() {
        assert_eq!(convert_key(WinitKeyCode::KeyW), Some(KeyCode::W));
        assert_eq!(convert_key(WinitKeyCode::Space), Some(KeyCode::Space));
        assert_eq!(convert_key(WinitKeyCode::ArrowUp), Some(KeyCode::Up));
        assert_eq!(convert_key(WinitKeyCode::Digit1), Some(KeyCode::Num1));
    }

    #[test]
    fn test_mouse_button_conversion() {
        assert_eq!(
            convert_mouse_button(WinitMouseButton::Left),
            Some(MouseButton::Left)
        );
        assert_eq!(
            convert_mouse_button(WinitMouseButton::Right),
            Some(MouseButton::Right)
        );
        assert_eq!(
            convert_mouse_button(WinitMouseButton::Middle),
            Some(MouseButton::Middle)
        );
    }

    #[test]
    fn test_end_frame_clears_state() {
        let mut handler = InputHandler::new();
        handler.just_pressed_keys.insert(KeyCode::W);
        handler.debug_toggle_pressed = true;
        handler.hotbar_selection = Some(5);
        handler.pause_pressed = true;

        handler.end_frame();

        assert!(handler.just_pressed_keys.is_empty());
        assert!(!handler.debug_toggle_pressed());
        assert!(handler.hotbar_selection().is_none());
        assert!(!handler.pause_pressed());
    }
}
