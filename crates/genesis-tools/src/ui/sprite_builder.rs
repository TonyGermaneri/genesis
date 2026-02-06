//! Character Sprite Builder - Interactive tool for defining sprite sheet animations.
//!
//! This module provides an in-game tool for manually defining sprite frame boundaries,
//! supporting:
//! - Multiple characters with unique configurations
//! - Per-action frame definitions (idle, walk_up, walk_down, etc.)
//! - Auto-generation of frame sequences from a starting point
//! - Visual sprite sheet preview with selection box
//! - Animated sprite preview
//! - Export to TOML configuration files

use egui::{Color32, ColorImage, RichText, TextureHandle, TextureOptions, Ui, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Default sprite sheet path for new characters.
pub const DEFAULT_SPRITE_SHEET: &str = "assets/sprites/player/player_scout.png";

// ============================================================================
// Sprite Frame Definition
// ============================================================================

/// A single frame in a sprite animation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpriteFrame {
    /// X position in the sprite sheet (pixels)
    pub x: u32,
    /// Y position in the sprite sheet (pixels)
    pub y: u32,
    /// Width of the frame (pixels)
    pub width: u32,
    /// Height of the frame (pixels)
    pub height: u32,
}

impl Default for SpriteFrame {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 48,
            height: 48,
        }
    }
}

impl SpriteFrame {
    /// Create a new frame at the given position with the given size.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Create an offset copy of this frame.
    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: (self.x as i32 + dx).max(0) as u32,
            y: (self.y as i32 + dy).max(0) as u32,
            width: self.width,
            height: self.height,
        }
    }
}

// ============================================================================
// Animation Action Types
// ============================================================================

/// Standard animation actions supported by the sprite system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationAction {
    IdleDown,
    IdleUp,
    IdleLeft,
    IdleRight,
    WalkDown,
    WalkUp,
    WalkLeft,
    WalkRight,
    RunDown,
    RunUp,
    RunLeft,
    RunRight,
    AttackDown,
    AttackUp,
    AttackLeft,
    AttackRight,
    UseDown,
    UseUp,
    UseLeft,
    UseRight,
    JumpDown,
    JumpUp,
    JumpLeft,
    JumpRight,
    SitDown,
    SitUp,
    SitLeft,
    SitRight,
    CraftDown,
    CraftUp,
    CraftLeft,
    CraftRight,
    ThrowDown,
    ThrowUp,
    ThrowLeft,
    ThrowRight,
    ShootDown,
    ShootUp,
    ShootLeft,
    ShootRight,
    HitDown,
    HitUp,
    HitLeft,
    HitRight,
    PunchDown,
    PunchUp,
    PunchLeft,
    PunchRight,
    StabDown,
    StabUp,
    StabLeft,
    StabRight,
    LiftDown,
    LiftUp,
    LiftLeft,
    LiftRight,
    PickUpDown,
    PickUpUp,
    PickUpLeft,
    PickUpRight,
    CartPushDown,
    CartPushUp,
    CartPushLeft,
    CartPushRight,
    GiftDown,
    GiftUp,
    GiftLeft,
    GiftRight,
    GrabGunDown,
    GrabGunUp,
    GrabGunLeft,
    GrabGunRight,
    GunIdleDown,
    GunIdleUp,
    GunIdleLeft,
    GunIdleRight,
    GunShootDown,
    GunShootUp,
    GunShootLeft,
    GunShootRight,
    HurtDown,
    HurtUp,
    HurtLeft,
    HurtRight,
    ReadDown,
    ReadUp,
    ReadLeft,
    ReadRight,
    SleepDown,
    SleepUp,
    SleepLeft,
    SleepRight,
    PhoneDown,
    PhoneUp,
    PhoneLeft,
    PhoneRight,
    Custom(u8),
}

impl AnimationAction {
    /// Get all standard actions (excludes Custom).
    pub fn all_standard() -> &'static [Self] {
        &[
            Self::IdleDown,
            Self::IdleUp,
            Self::IdleLeft,
            Self::IdleRight,
            Self::WalkDown,
            Self::WalkUp,
            Self::WalkLeft,
            Self::WalkRight,
            Self::RunDown,
            Self::RunUp,
            Self::RunLeft,
            Self::RunRight,
            Self::AttackDown,
            Self::AttackUp,
            Self::AttackLeft,
            Self::AttackRight,
            Self::UseDown,
            Self::UseUp,
            Self::UseLeft,
            Self::UseRight,
            Self::JumpDown,
            Self::JumpUp,
            Self::JumpLeft,
            Self::JumpRight,
            Self::SitDown,
            Self::SitUp,
            Self::SitLeft,
            Self::SitRight,
            Self::CraftDown,
            Self::CraftUp,
            Self::CraftLeft,
            Self::CraftRight,
            Self::ThrowDown,
            Self::ThrowUp,
            Self::ThrowLeft,
            Self::ThrowRight,
            Self::ShootDown,
            Self::ShootUp,
            Self::ShootLeft,
            Self::ShootRight,
            Self::HitDown,
            Self::HitUp,
            Self::HitLeft,
            Self::HitRight,
            Self::PunchDown,
            Self::PunchUp,
            Self::PunchLeft,
            Self::PunchRight,
            Self::StabDown,
            Self::StabUp,
            Self::StabLeft,
            Self::StabRight,
            Self::LiftDown,
            Self::LiftUp,
            Self::LiftLeft,
            Self::LiftRight,
            Self::PickUpDown,
            Self::PickUpUp,
            Self::PickUpLeft,
            Self::PickUpRight,
            Self::CartPushDown,
            Self::CartPushUp,
            Self::CartPushLeft,
            Self::CartPushRight,
            Self::GiftDown,
            Self::GiftUp,
            Self::GiftLeft,
            Self::GiftRight,
            Self::GrabGunDown,
            Self::GrabGunUp,
            Self::GrabGunLeft,
            Self::GrabGunRight,
            Self::GunIdleDown,
            Self::GunIdleUp,
            Self::GunIdleLeft,
            Self::GunIdleRight,
            Self::GunShootDown,
            Self::GunShootUp,
            Self::GunShootLeft,
            Self::GunShootRight,
            Self::HurtDown,
            Self::HurtUp,
            Self::HurtLeft,
            Self::HurtRight,
            Self::ReadDown,
            Self::ReadUp,
            Self::ReadLeft,
            Self::ReadRight,
            Self::SleepDown,
            Self::SleepUp,
            Self::SleepLeft,
            Self::SleepRight,
            Self::PhoneDown,
            Self::PhoneUp,
            Self::PhoneLeft,
            Self::PhoneRight,
        ]
    }

    /// Get the display name for this action.
    pub fn display_name(&self) -> String {
        match self {
            Self::IdleDown => "Idle Down".to_string(),
            Self::IdleUp => "Idle Up".to_string(),
            Self::IdleLeft => "Idle Left".to_string(),
            Self::IdleRight => "Idle Right".to_string(),
            Self::WalkDown => "Walk Down".to_string(),
            Self::WalkUp => "Walk Up".to_string(),
            Self::WalkLeft => "Walk Left".to_string(),
            Self::WalkRight => "Walk Right".to_string(),
            Self::RunDown => "Run Down".to_string(),
            Self::RunUp => "Run Up".to_string(),
            Self::RunLeft => "Run Left".to_string(),
            Self::RunRight => "Run Right".to_string(),
            Self::AttackDown => "Attack Down".to_string(),
            Self::AttackUp => "Attack Up".to_string(),
            Self::AttackLeft => "Attack Left".to_string(),
            Self::AttackRight => "Attack Right".to_string(),
            Self::UseDown => "Use Down".to_string(),
            Self::UseUp => "Use Up".to_string(),
            Self::UseLeft => "Use Left".to_string(),
            Self::UseRight => "Use Right".to_string(),
            Self::JumpDown => "Jump Down".to_string(),
            Self::JumpUp => "Jump Up".to_string(),
            Self::JumpLeft => "Jump Left".to_string(),
            Self::JumpRight => "Jump Right".to_string(),
            Self::SitDown => "Sit Down".to_string(),
            Self::SitUp => "Sit Up".to_string(),
            Self::SitLeft => "Sit Left".to_string(),
            Self::SitRight => "Sit Right".to_string(),
            Self::CraftDown => "Craft Down".to_string(),
            Self::CraftUp => "Craft Up".to_string(),
            Self::CraftLeft => "Craft Left".to_string(),
            Self::CraftRight => "Craft Right".to_string(),
            Self::ThrowDown => "Throw Down".to_string(),
            Self::ThrowUp => "Throw Up".to_string(),
            Self::ThrowLeft => "Throw Left".to_string(),
            Self::ThrowRight => "Throw Right".to_string(),
            Self::ShootDown => "Shoot Down".to_string(),
            Self::ShootUp => "Shoot Up".to_string(),
            Self::ShootLeft => "Shoot Left".to_string(),
            Self::ShootRight => "Shoot Right".to_string(),
            Self::HitDown => "Hit Down".to_string(),
            Self::HitUp => "Hit Up".to_string(),
            Self::HitLeft => "Hit Left".to_string(),
            Self::HitRight => "Hit Right".to_string(),
            Self::PunchDown => "Punch Down".to_string(),
            Self::PunchUp => "Punch Up".to_string(),
            Self::PunchLeft => "Punch Left".to_string(),
            Self::PunchRight => "Punch Right".to_string(),
            Self::StabDown => "Stab Down".to_string(),
            Self::StabUp => "Stab Up".to_string(),
            Self::StabLeft => "Stab Left".to_string(),
            Self::StabRight => "Stab Right".to_string(),
            Self::LiftDown => "Lift Down".to_string(),
            Self::LiftUp => "Lift Up".to_string(),
            Self::LiftLeft => "Lift Left".to_string(),
            Self::LiftRight => "Lift Right".to_string(),
            Self::PickUpDown => "Pick Up Down".to_string(),
            Self::PickUpUp => "Pick Up Up".to_string(),
            Self::PickUpLeft => "Pick Up Left".to_string(),
            Self::PickUpRight => "Pick Up Right".to_string(),
            Self::CartPushDown => "Cart Push Down".to_string(),
            Self::CartPushUp => "Cart Push Up".to_string(),
            Self::CartPushLeft => "Cart Push Left".to_string(),
            Self::CartPushRight => "Cart Push Right".to_string(),
            Self::GiftDown => "Gift Down".to_string(),
            Self::GiftUp => "Gift Up".to_string(),
            Self::GiftLeft => "Gift Left".to_string(),
            Self::GiftRight => "Gift Right".to_string(),
            Self::GrabGunDown => "Grab Gun Down".to_string(),
            Self::GrabGunUp => "Grab Gun Up".to_string(),
            Self::GrabGunLeft => "Grab Gun Left".to_string(),
            Self::GrabGunRight => "Grab Gun Right".to_string(),
            Self::GunIdleDown => "Gun Idle Down".to_string(),
            Self::GunIdleUp => "Gun Idle Up".to_string(),
            Self::GunIdleLeft => "Gun Idle Left".to_string(),
            Self::GunIdleRight => "Gun Idle Right".to_string(),
            Self::GunShootDown => "Gun Shoot Down".to_string(),
            Self::GunShootUp => "Gun Shoot Up".to_string(),
            Self::GunShootLeft => "Gun Shoot Left".to_string(),
            Self::GunShootRight => "Gun Shoot Right".to_string(),
            Self::HurtDown => "Hurt Down".to_string(),
            Self::HurtUp => "Hurt Up".to_string(),
            Self::HurtLeft => "Hurt Left".to_string(),
            Self::HurtRight => "Hurt Right".to_string(),
            Self::ReadDown => "Read Down".to_string(),
            Self::ReadUp => "Read Up".to_string(),
            Self::ReadLeft => "Read Left".to_string(),
            Self::ReadRight => "Read Right".to_string(),
            Self::SleepDown => "Sleep Down".to_string(),
            Self::SleepUp => "Sleep Up".to_string(),
            Self::SleepLeft => "Sleep Left".to_string(),
            Self::SleepRight => "Sleep Right".to_string(),
            Self::PhoneDown => "Phone Down".to_string(),
            Self::PhoneUp => "Phone Up".to_string(),
            Self::PhoneLeft => "Phone Left".to_string(),
            Self::PhoneRight => "Phone Right".to_string(),
            Self::Custom(id) => format!("Custom {}", id),
        }
    }

    /// Get the icon for this action.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::IdleDown | Self::IdleUp | Self::IdleLeft | Self::IdleRight => "🧍",
            Self::WalkDown | Self::WalkUp | Self::WalkLeft | Self::WalkRight => "🚶",
            Self::RunDown | Self::RunUp | Self::RunLeft | Self::RunRight => "🏃",
            Self::AttackDown | Self::AttackUp | Self::AttackLeft | Self::AttackRight => "⚔",
            Self::UseDown | Self::UseUp | Self::UseLeft | Self::UseRight => "🤲",
            Self::JumpDown | Self::JumpUp | Self::JumpLeft | Self::JumpRight => "⬆",
            Self::SitDown | Self::SitUp | Self::SitLeft | Self::SitRight => "🪑",
            Self::CraftDown | Self::CraftUp | Self::CraftLeft | Self::CraftRight => "🔨",
            Self::ThrowDown | Self::ThrowUp | Self::ThrowLeft | Self::ThrowRight => "🎯",
            Self::ShootDown | Self::ShootUp | Self::ShootLeft | Self::ShootRight => "🏹",
            Self::HitDown | Self::HitUp | Self::HitLeft | Self::HitRight => "💥",
            Self::PunchDown | Self::PunchUp | Self::PunchLeft | Self::PunchRight => "🥊",
            Self::StabDown | Self::StabUp | Self::StabLeft | Self::StabRight => "🗡",
            Self::LiftDown | Self::LiftUp | Self::LiftLeft | Self::LiftRight => "🏋",
            Self::PickUpDown | Self::PickUpUp | Self::PickUpLeft | Self::PickUpRight => "⬇",
            Self::CartPushDown | Self::CartPushUp | Self::CartPushLeft | Self::CartPushRight => "🛒",
            Self::GiftDown | Self::GiftUp | Self::GiftLeft | Self::GiftRight => "🎁",
            Self::GrabGunDown | Self::GrabGunUp | Self::GrabGunLeft | Self::GrabGunRight => "🔫",
            Self::GunIdleDown | Self::GunIdleUp | Self::GunIdleLeft | Self::GunIdleRight => "🔫",
            Self::GunShootDown | Self::GunShootUp | Self::GunShootLeft | Self::GunShootRight => "💨",
            Self::HurtDown | Self::HurtUp | Self::HurtLeft | Self::HurtRight => "🤕",
            Self::ReadDown | Self::ReadUp | Self::ReadLeft | Self::ReadRight => "📖",
            Self::SleepDown | Self::SleepUp | Self::SleepLeft | Self::SleepRight => "😴",
            Self::PhoneDown | Self::PhoneUp | Self::PhoneLeft | Self::PhoneRight => "📱",
            Self::Custom(_) => "📦",
        }
    }
}

// ============================================================================
// Animation Definition
// ============================================================================

/// Playback mode for animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlaybackMode {
    /// Play frames in order (0, 1, 2, 3, 0, 1, 2, 3, ...)
    #[default]
    Normal,
    /// Play frames in reverse order (3, 2, 1, 0, 3, 2, 1, 0, ...)
    Reverse,
    /// Play forward then backward (0, 1, 2, 3, 2, 1, 0, 1, 2, 3, ...)
    PingPong,
    /// Play frames in random order (for organic effects like fire, sparks)
    Random,
}

impl PlaybackMode {
    /// Get all playback modes.
    pub fn all() -> &'static [Self] {
        &[Self::Normal, Self::Reverse, Self::PingPong, Self::Random]
    }

    /// Get the display name for this mode.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Reverse => "Reverse",
            Self::PingPong => "Ping-Pong",
            Self::Random => "Random",
        }
    }
}

/// Definition for a single animation (one action like "walk_down").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDef {
    /// The action this animation represents.
    pub action: AnimationAction,
    /// Whether this animation is enabled/defined.
    pub enabled: bool,
    /// Frames in this animation.
    pub frames: Vec<SpriteFrame>,
    /// Frames per second for this animation.
    pub fps: f32,
    /// Speed multiplier (1.0 = normal, 0.5 = half speed, 2.0 = double speed).
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Whether to loop this animation.
    pub looping: bool,
    /// Playback mode (normal, reverse, ping-pong).
    #[serde(default)]
    pub playback_mode: PlaybackMode,
}

/// Default speed value for serde deserialization.
fn default_speed() -> f32 {
    1.0
}

impl AnimationDef {
    /// Create a new empty animation for the given action.
    pub fn new(action: AnimationAction) -> Self {
        Self {
            action,
            enabled: false,
            frames: Vec::new(),
            fps: 8.0,
            speed: 1.0,
            looping: true,
            playback_mode: PlaybackMode::Normal,
        }
    }

    /// Create an animation with a single starting frame and auto-generate more.
    pub fn with_auto_frames(
        action: AnimationAction,
        start_frame: SpriteFrame,
        frame_count: u32,
        x_offset: i32,
        y_offset: i32,
    ) -> Self {
        let mut frames = Vec::with_capacity(frame_count as usize);
        for i in 0..frame_count {
            frames.push(SpriteFrame {
                x: (start_frame.x as i32 + x_offset * i as i32).max(0) as u32,
                y: (start_frame.y as i32 + y_offset * i as i32).max(0) as u32,
                width: start_frame.width,
                height: start_frame.height,
            });
        }
        Self {
            action,
            enabled: true,
            frames,
            fps: 8.0,
            speed: 1.0,
            looping: true,
            playback_mode: PlaybackMode::Normal,
        }
    }
}

// ============================================================================
// Character Sprite Definition
// ============================================================================

/// Complete sprite definition for a character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSpriteDef {
    /// Unique identifier for this character sprite.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Path to the sprite sheet image (relative to assets/).
    pub sprite_sheet_path: String,
    /// Default frame width (for auto-generation).
    pub default_frame_width: u32,
    /// Default frame height (for auto-generation).
    pub default_frame_height: u32,
    /// Render scale multiplier.
    pub scale: f32,
    /// Animation definitions for each action.
    pub animations: Vec<AnimationDef>,
}

impl Default for CharacterSpriteDef {
    fn default() -> Self {
        Self {
            id: String::from("new_character"),
            name: String::from("New Character"),
            sprite_sheet_path: DEFAULT_SPRITE_SHEET.to_string(),
            default_frame_width: 48,
            default_frame_height: 48,
            scale: 2.0,
            animations: AnimationAction::all_standard()
                .iter()
                .map(|&action| AnimationDef::new(action))
                .collect(),
        }
    }
}

impl CharacterSpriteDef {
    /// Create a new character sprite definition with the given ID.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Get animation for a specific action.
    pub fn get_animation(&self, action: AnimationAction) -> Option<&AnimationDef> {
        self.animations.iter().find(|a| a.action == action)
    }

    /// Get mutable animation for a specific action.
    pub fn get_animation_mut(&mut self, action: AnimationAction) -> Option<&mut AnimationDef> {
        self.animations.iter_mut().find(|a| a.action == action)
    }

    /// Save to TOML file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), String> {
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        std::fs::write(path, toml_str)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    /// Load from TOML file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))
    }
}

// ============================================================================
// Sprite Builder State
// ============================================================================

/// State for the sprite builder UI.
pub struct SpriteBuilder {
    /// All character definitions.
    pub characters: Vec<CharacterSpriteDef>,
    /// Currently selected character index.
    pub selected_character: usize,
    /// Currently selected animation index within the character.
    pub selected_animation: usize,
    /// Currently selected frame index within the animation.
    pub selected_frame: usize,
    /// Whether the builder has unsaved changes.
    pub modified: bool,
    /// Status message to display.
    pub status_message: Option<(String, Color32)>,
    /// Auto-generation settings.
    pub auto_gen: AutoGenSettings,
    /// Texture cache for loaded sprite sheets.
    texture_cache: HashMap<String, TextureHandle>,
    /// Current sprite sheet path being displayed.
    current_sprite_path: String,
    /// Sprite sheet dimensions (width, height).
    sprite_sheet_size: Option<(u32, u32)>,
    /// Animation preview state.
    preview_state: AnimationPreviewState,
    /// Sprite sheet scroll position.
    sheet_scroll: Vec2,
    /// Sprite sheet zoom level.
    sheet_zoom: f32,
    /// Available width from parent container (for responsive layout).
    available_width: f32,
    /// Current mouse position on sprite sheet (in sprite sheet coordinates).
    mouse_pos_on_sheet: Option<(u32, u32)>,
    /// Active drag operation on sprite sheet.
    drag_state: Option<DragState>,
    /// Undo history stack (previous states).
    undo_stack: Vec<Vec<CharacterSpriteDef>>,
    /// Redo history stack (undone states).
    redo_stack: Vec<Vec<CharacterSpriteDef>>,
    /// Maximum undo history size.
    max_history_size: usize,
}

impl std::fmt::Debug for SpriteBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpriteBuilder")
            .field("characters", &self.characters)
            .field("selected_character", &self.selected_character)
            .field("selected_animation", &self.selected_animation)
            .field("modified", &self.modified)
            .field("texture_cache_len", &self.texture_cache.len())
            .field("available_width", &self.available_width)
            .field("mouse_pos_on_sheet", &self.mouse_pos_on_sheet)
            .field("drag_state", &self.drag_state)
            .field("undo_stack_size", &self.undo_stack.len())
            .field("redo_stack_size", &self.redo_stack.len())
            .finish()
    }
}

impl Clone for SpriteBuilder {
    fn clone(&self) -> Self {
        Self {
            characters: self.characters.clone(),
            selected_character: self.selected_character,
            selected_animation: self.selected_animation,
            selected_frame: self.selected_frame,
            modified: self.modified,
            status_message: self.status_message.clone(),
            auto_gen: self.auto_gen.clone(),
            // Texture cache cannot be cloned - start fresh
            texture_cache: HashMap::new(),
            current_sprite_path: self.current_sprite_path.clone(),
            sprite_sheet_size: self.sprite_sheet_size,
            preview_state: self.preview_state.clone(),
            sheet_scroll: self.sheet_scroll,
            sheet_zoom: self.sheet_zoom,
            available_width: self.available_width,
            mouse_pos_on_sheet: self.mouse_pos_on_sheet,
            drag_state: self.drag_state.clone(),
            undo_stack: self.undo_stack.clone(),
            redo_stack: self.redo_stack.clone(),
            max_history_size: self.max_history_size,
        }
    }
}

/// State for the animated sprite preview.
#[derive(Debug, Clone)]
pub struct AnimationPreviewState {
    /// Whether animation is playing.
    pub playing: bool,
    /// Current frame index in the animation.
    pub current_frame: usize,
    /// Time of last frame advance.
    pub last_frame_time: Instant,
    /// Direction for ping-pong playback (true = forward, false = backward).
    pub ping_pong_forward: bool,
}

impl Default for AnimationPreviewState {
    fn default() -> Self {
        Self {
            playing: true,
            current_frame: 0,
            last_frame_time: Instant::now(),
            ping_pong_forward: true,
        }
    }
}

/// Type of drag operation being performed on the sprite sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragHandle {
    /// Dragging the entire frame (moving it).
    Move,
    /// Dragging top-left corner.
    TopLeft,
    /// Dragging top-right corner.
    TopRight,
    /// Dragging bottom-left corner.
    BottomLeft,
    /// Dragging bottom-right corner.
    BottomRight,
    /// Dragging top edge.
    Top,
    /// Dragging bottom edge.
    Bottom,
    /// Dragging left edge.
    Left,
    /// Dragging right edge.
    Right,
}

/// State for an active drag operation.
#[derive(Debug, Clone)]
pub struct DragState {
    /// Which handle/edge is being dragged.
    pub handle: DragHandle,
    /// Starting position of the drag (in sprite sheet coords).
    pub start_pos: (u32, u32),
    /// Original frame before drag started.
    pub original_frame: SpriteFrame,
    /// Index of the frame being dragged (animation_idx, frame_idx).
    pub frame_index: (usize, usize),
}

/// Settings for auto-generating animation frames.
#[derive(Debug, Clone)]
pub struct AutoGenSettings {
    /// Starting X position.
    pub start_x: u32,
    /// Starting Y position.
    pub start_y: u32,
    /// Frame width.
    pub frame_width: u32,
    /// Frame height.
    pub frame_height: u32,
    /// Number of frames to generate.
    pub frame_count: u32,
    /// X offset between frames (default: frame_width, 0 for vertical strips).
    pub x_offset: i32,
    /// Y offset between frames (default: 0, or frame_height for vertical strips).
    pub y_offset: i32,
}

impl Default for AutoGenSettings {
    fn default() -> Self {
        Self {
            start_x: 0,
            start_y: 0,
            frame_width: 48,
            frame_height: 48,
            frame_count: 4,
            x_offset: 48, // Default: horizontal strip
            y_offset: 0,
        }
    }
}

impl Default for SpriteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteBuilder {
    /// Create a new sprite builder.
    pub fn new() -> Self {
        Self {
            characters: Vec::new(),
            selected_character: 0,
            selected_animation: 0,
            selected_frame: 0,
            modified: false,
            status_message: None,
            auto_gen: AutoGenSettings::default(),
            texture_cache: HashMap::new(),
            current_sprite_path: String::new(),
            sprite_sheet_size: None,
            preview_state: AnimationPreviewState::default(),
            sheet_scroll: Vec2::ZERO,
            sheet_zoom: 1.0,
            available_width: 1000.0, // Default reasonable width
            mouse_pos_on_sheet: None,
            drag_state: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_size: 50,
        }
    }

    /// Push current state to undo stack (call before making changes).
    pub fn push_undo_state(&mut self) {
        self.undo_stack.push(self.characters.clone());
        // Limit history size
        if self.undo_stack.len() > self.max_history_size {
            self.undo_stack.remove(0);
        }
        // Clear redo stack when new changes are made
        self.redo_stack.clear();
    }

    /// Undo the last change.
    pub fn undo(&mut self) -> bool {
        if let Some(previous_state) = self.undo_stack.pop() {
            // Save current state to redo stack
            self.redo_stack.push(self.characters.clone());
            // Restore previous state
            self.characters = previous_state;
            self.modified = true;
            // Clamp selection indices
            self.selected_character = self.selected_character.min(self.characters.len().saturating_sub(1));
            if let Some(character) = self.characters.get(self.selected_character) {
                self.selected_animation = self.selected_animation.min(character.animations.len().saturating_sub(1));
                if let Some(anim) = character.animations.get(self.selected_animation) {
                    self.selected_frame = self.selected_frame.min(anim.frames.len().saturating_sub(1));
                }
            }
            self.status_message = Some(("Undo".to_string(), Color32::LIGHT_BLUE));
            true
        } else {
            self.status_message = Some(("Nothing to undo".to_string(), Color32::GRAY));
            false
        }
    }

    /// Redo the last undone change.
    pub fn redo(&mut self) -> bool {
        if let Some(next_state) = self.redo_stack.pop() {
            // Save current state to undo stack
            self.undo_stack.push(self.characters.clone());
            // Restore next state
            self.characters = next_state;
            self.modified = true;
            // Clamp selection indices
            self.selected_character = self.selected_character.min(self.characters.len().saturating_sub(1));
            if let Some(character) = self.characters.get(self.selected_character) {
                self.selected_animation = self.selected_animation.min(character.animations.len().saturating_sub(1));
                if let Some(anim) = character.animations.get(self.selected_animation) {
                    self.selected_frame = self.selected_frame.min(anim.frames.len().saturating_sub(1));
                }
            }
            self.status_message = Some(("Redo".to_string(), Color32::LIGHT_BLUE));
            true
        } else {
            self.status_message = Some(("Nothing to redo".to_string(), Color32::GRAY));
            false
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Set the available width for responsive layout.
    pub fn set_available_width(&mut self, width: f32) {
        self.available_width = width;
    }

    /// Load a sprite sheet texture into the cache.
    fn load_sprite_sheet(&mut self, ctx: &egui::Context, path: &str) -> Option<TextureHandle> {
        // Check cache first
        if let Some(texture) = self.texture_cache.get(path) {
            return Some(texture.clone());
        }

        // Try to load the image
        let img_path = Path::new(path);
        if !img_path.exists() {
            return None;
        }

        match image::open(img_path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                self.sprite_sheet_size = Some((rgba.width(), rgba.height()));

                let color_image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                let texture = ctx.load_texture(
                    path,
                    color_image,
                    TextureOptions::NEAREST,
                );

                self.texture_cache.insert(path.to_string(), texture.clone());
                self.current_sprite_path = path.to_string();
                Some(texture)
            }
            Err(e) => {
                tracing::warn!("Failed to load sprite sheet {}: {}", path, e);
                None
            }
        }
    }

    /// Load all character definitions from the assets/sprites/characters/ directory.
    pub fn load_all(&mut self) {
        let dir = std::path::Path::new("assets/sprites/characters");
        if !dir.exists() {
            if let Err(e) = std::fs::create_dir_all(dir) {
                self.status_message = Some((
                    format!("Failed to create directory: {}", e),
                    Color32::RED,
                ));
                return;
            }
        }

        self.characters.clear();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "toml").unwrap_or(false) {
                    match CharacterSpriteDef::load_from_file(&path) {
                        Ok(def) => self.characters.push(def),
                        Err(e) => {
                            self.status_message = Some((
                                format!("Failed to load {}: {}", path.display(), e),
                                Color32::YELLOW,
                            ));
                        }
                    }
                }
            }
        }

        // Ensure we have at least one character
        if self.characters.is_empty() {
            self.characters.push(CharacterSpriteDef::default());
        }

        self.selected_character = 0;
        self.selected_animation = 0;
        self.selected_frame = 0;
        self.modified = false;
        self.status_message = Some((
            format!("Loaded {} character(s)", self.characters.len()),
            Color32::GREEN,
        ));
    }

    /// Save all character definitions.
    pub fn save_all(&mut self) {
        let dir = std::path::Path::new("assets/sprites/characters");
        if let Err(e) = std::fs::create_dir_all(dir) {
            self.status_message = Some((
                format!("Failed to create directory: {}", e),
                Color32::RED,
            ));
            return;
        }

        let mut saved_count = 0;
        for character in &self.characters {
            let path = dir.join(format!("{}.toml", character.id));
            match character.save_to_file(&path) {
                Ok(()) => saved_count += 1,
                Err(e) => {
                    self.status_message = Some((
                        format!("Failed to save {}: {}", character.id, e),
                        Color32::RED,
                    ));
                    return;
                }
            }
        }

        self.modified = false;
        self.status_message = Some((
            format!("Saved {} character(s)", saved_count),
            Color32::GREEN,
        ));
    }

    /// Get the currently selected character.
    pub fn current_character(&self) -> Option<&CharacterSpriteDef> {
        self.characters.get(self.selected_character)
    }

    /// Get the currently selected character mutably.
    pub fn current_character_mut(&mut self) -> Option<&mut CharacterSpriteDef> {
        self.characters.get_mut(self.selected_character)
    }

    /// Get the currently selected animation.
    pub fn current_animation(&self) -> Option<&AnimationDef> {
        self.current_character()
            .and_then(|c| c.animations.get(self.selected_animation))
    }

    /// Get the currently selected animation mutably.
    pub fn current_animation_mut(&mut self) -> Option<&mut AnimationDef> {
        let idx = self.selected_animation;
        self.current_character_mut()
            .and_then(|c| c.animations.get_mut(idx))
    }

    /// Add a new character.
    pub fn add_character(&mut self) {
        let id = format!("character_{}", self.characters.len());
        let name = format!("Character {}", self.characters.len());
        self.characters.push(CharacterSpriteDef::new(id, name));
        self.selected_character = self.characters.len() - 1;
        self.selected_animation = 0;
        self.selected_frame = 0;
        self.modified = true;
    }

    /// Remove the currently selected character.
    pub fn remove_current_character(&mut self) {
        if self.characters.len() > 1 {
            self.characters.remove(self.selected_character);
            if self.selected_character >= self.characters.len() {
                self.selected_character = self.characters.len() - 1;
            }
            self.modified = true;
        }
    }

    /// Apply auto-generation to the current animation.
    pub fn apply_auto_gen(&mut self) {
        let settings = self.auto_gen.clone();
        if let Some(anim) = self.current_animation_mut() {
            anim.frames.clear();
            for i in 0..settings.frame_count {
                anim.frames.push(SpriteFrame {
                    x: (settings.start_x as i32 + settings.x_offset * i as i32).max(0) as u32,
                    y: (settings.start_y as i32 + settings.y_offset * i as i32).max(0) as u32,
                    width: settings.frame_width,
                    height: settings.frame_height,
                });
            }
            anim.enabled = true;
            self.modified = true;
            self.status_message = Some((
                format!("Generated {} frames", settings.frame_count),
                Color32::GREEN,
            ));
        }
    }

    /// Render the sprite builder UI.
    pub fn render(&mut self, ui: &mut Ui) {
        // Get ctx for texture loading
        let ctx = ui.ctx().clone();

        // Handle keyboard shortcuts for undo/redo
        let modifiers = ui.input(|i| i.modifiers);
        let z_pressed = ui.input(|i| i.key_pressed(egui::Key::Z));

        if z_pressed && modifiers.command {
            if modifiers.shift {
                // Cmd+Shift+Z = Redo
                self.redo();
            } else {
                // Cmd+Z = Undo
                self.undo();
            }
        }

        // Handle arrow keys to move selection box by 1 pixel
        let arrow_left = ui.input(|i| i.key_pressed(egui::Key::ArrowLeft));
        let arrow_right = ui.input(|i| i.key_pressed(egui::Key::ArrowRight));
        let arrow_up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let arrow_down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));

        if arrow_left || arrow_right || arrow_up || arrow_down {
            // Calculate movement delta
            let dx: i32 = if arrow_left { -1 } else if arrow_right { 1 } else { 0 };
            let dy: i32 = if arrow_up { -1 } else if arrow_down { 1 } else { 0 };

            // Check if we have a frame to move first
            let has_frame = self.characters
                .get(self.selected_character)
                .and_then(|c| c.animations.get(self.selected_animation))
                .and_then(|a| a.frames.get(self.selected_frame))
                .is_some();

            if has_frame {
                // Push undo state before modifying
                self.push_undo_state();

                // Now move the selected frame
                if let Some(character) = self.characters.get_mut(self.selected_character) {
                    if let Some(anim) = character.animations.get_mut(self.selected_animation) {
                        if let Some(frame) = anim.frames.get_mut(self.selected_frame) {
                            frame.x = (frame.x as i32 + dx).max(0) as u32;
                            frame.y = (frame.y as i32 + dy).max(0) as u32;
                            // Sync auto_gen with new position
                            self.auto_gen.start_x = frame.x;
                            self.auto_gen.start_y = frame.y;
                            self.modified = true;
                        }
                    }
                }
            }
        }

        ui.heading("🎨 Character Sprite Builder");
        ui.label("Define sprite sheet frame boundaries for character animations.");
        ui.add_space(4.0);

        // Status message
        let mut clear_status = false;
        if let Some((msg, color)) = &self.status_message {
            let msg = msg.clone();
            let color = *color;
            ui.horizontal(|ui| {
                ui.colored_label(color, &msg);
                if ui.small_button("✕").clicked() {
                    clear_status = true;
                }
            });
            ui.add_space(4.0);
        }
        if clear_status {
            self.status_message = None;
        }

        // Top toolbar
        ui.horizontal(|ui| {
            if ui.button("📂 Load All").clicked() {
                self.load_all();
            }
            if ui.button("💾 Save All").clicked() {
                self.save_all();
            }
            if ui.button("➕ New Character").clicked() {
                self.push_undo_state();
                self.add_character();
            }

            ui.separator();

            // Undo/Redo buttons
            ui.add_enabled_ui(self.can_undo(), |ui| {
                if ui.button("↩ Undo").on_hover_text("Undo (⌘Z)").clicked() {
                    self.undo();
                }
            });
            ui.add_enabled_ui(self.can_redo(), |ui| {
                if ui.button("↪ Redo").on_hover_text("Redo (⌘⇧Z)").clicked() {
                    self.redo();
                }
            });

            if self.modified {
                ui.label(RichText::new("● Unsaved Changes").color(Color32::YELLOW));
            }
        });

        ui.separator();

        // Main layout: 3 columns
        // Column 1: Character/Animation list
        // Column 2: Frame editor
        // Column 3: Sprite sheet preview + Animation preview (stacked vertically)

        // Get available space from parent
        let available = ui.available_width();

        // Fixed column widths
        let char_list_width = 160.0;
        let frame_editor_width = 280.0;
        let preview_width = 700.0;

        // Scale down if needed to fit
        let total_needed = char_list_width + frame_editor_width + preview_width + 24.0;
        let scale = if total_needed > available { available / total_needed } else { 1.0 };

        let char_list_width = char_list_width * scale;
        let frame_editor_width = frame_editor_width * scale;
        let preview_width = preview_width * scale;

        ui.horizontal(|ui| {
            // Column 1: Character and animation selection
            ui.vertical(|ui| {
                ui.set_width(char_list_width);
                ui.set_height(700.0);
                self.render_character_list(ui);
                ui.separator();
                self.render_animation_list(ui);
            });

            ui.separator();

            // Column 2: Frame editor
            ui.vertical(|ui| {
                ui.set_width(frame_editor_width);
                self.render_frame_editor(ui);
            });

            ui.separator();

            // Column 3: Sprite sheet preview + Animation preview (stacked)
            ui.vertical(|ui| {
                ui.set_width(preview_width);

                // Sprite sheet preview (main area)
                self.render_sprite_sheet_preview(ui, &ctx);

                ui.separator();

                // Animation preview (below sprite sheet)
                self.render_animation_preview(ui, &ctx);
            });
        });
    }

    /// Render the sprite sheet preview with interactive selection boxes.
    fn render_sprite_sheet_preview(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.label(RichText::new("Sprite Sheet Preview").strong());
        ui.set_max_width(1200.0);
        // Zoom controls
        ui.horizontal(|ui| {
            ui.label("Zoom:");
            if ui.button("−").clicked() {
                self.sheet_zoom = (self.sheet_zoom - 0.25).max(0.25);
            }
            ui.label(format!("{:.0}%", self.sheet_zoom * 100.0));
            if ui.button("+").clicked() {
                self.sheet_zoom = (self.sheet_zoom + 0.25).min(4.0);
            }
            if ui.button("Reset").clicked() {
                self.sheet_zoom = 1.0;
                self.sheet_scroll = Vec2::ZERO;
            }
        });

        // Get current sprite sheet path
        let sprite_path = self.characters
            .get(self.selected_character)
            .map(|c| c.sprite_sheet_path.clone())
            .unwrap_or_default();

        if sprite_path.is_empty() {
            ui.label(RichText::new("No sprite sheet specified").color(Color32::GRAY).italics());
            self.mouse_pos_on_sheet = None;
            return;
        }

        // Load texture if needed
        let texture = self.load_sprite_sheet(ctx, &sprite_path);

        let Some(texture) = texture else {
            ui.label(RichText::new(format!("Failed to load: {}", sprite_path)).color(Color32::RED));
            self.mouse_pos_on_sheet = None;
            return;
        };

        // Get all frames in current animation for drawing
        let animation_frames: Vec<(usize, SpriteFrame)> = self.characters
            .get(self.selected_character)
            .and_then(|c| c.animations.get(self.selected_animation))
            .map(|a| a.frames.iter().cloned().enumerate().collect())
            .unwrap_or_default();

        let selected_frame_idx = self.selected_frame;
        let selected_anim_idx = self.selected_animation;
        let zoom = self.sheet_zoom;

        // Draw sprite sheet in scroll area - use fixed max heights to prevent overflow
        let scroll_width = 1200.0;
        let scroll_height = 300.0; // Fixed height for sprite sheet preview

        let mut new_selected_frame: Option<usize> = None;
        let mut frame_modified = false;
        let mut new_frame_values: Option<(u32, u32, u32, u32)> = None;

        egui::ScrollArea::both()
            .id_salt("sprite_sheet_scroll")
            .max_width(scroll_width)
            .max_height(scroll_height)
            .auto_shrink(false) // Prevent scroll area from triggering layout changes
            .show(ui, |ui| {
                let texture_size = texture.size_vec2() * zoom;

                // Allocate space for the image with drag sensing
                let (rect, response) = ui.allocate_exact_size(texture_size, egui::Sense::click_and_drag());

                // Draw the sprite sheet
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Track mouse position on sheet
                if let Some(hover_pos) = response.hover_pos() {
                    let relative_pos = hover_pos - rect.min;
                    let x = (relative_pos.x / zoom).max(0.0) as u32;
                    let y = (relative_pos.y / zoom).max(0.0) as u32;
                    self.mouse_pos_on_sheet = Some((x, y));
                } else {
                    self.mouse_pos_on_sheet = None;
                }

                // Draw all frames in current animation
                let handle_size = 8.0;
                for (i, frame) in &animation_frames {
                    let is_selected = *i == selected_frame_idx;
                    let frame_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(
                            frame.x as f32 * zoom,
                            frame.y as f32 * zoom,
                        ),
                        egui::vec2(
                            frame.width as f32 * zoom,
                            frame.height as f32 * zoom,
                        ),
                    );

                    // Draw frame rectangle
                    let (stroke_width, stroke_color) = if is_selected {
                        (2.5, Color32::YELLOW)
                    } else {
                        (1.5, Color32::from_rgba_unmultiplied(100, 200, 255, 200))
                    };

                    ui.painter().rect_stroke(
                        frame_rect,
                        0.0,
                        egui::Stroke::new(stroke_width, stroke_color),
                    );

                    // Draw frame number label
                    let label_pos = frame_rect.min + egui::vec2(2.0, 2.0);
                    ui.painter().text(
                        label_pos,
                        egui::Align2::LEFT_TOP,
                        format!("{}", i),
                        egui::FontId::proportional(10.0),
                        if is_selected { Color32::YELLOW } else { Color32::WHITE },
                    );

                    // Draw resize handles for selected frame
                    if is_selected {
                        // Corner handles
                        let corners = [
                            (frame_rect.min, DragHandle::TopLeft),
                            (egui::pos2(frame_rect.max.x, frame_rect.min.y), DragHandle::TopRight),
                            (egui::pos2(frame_rect.min.x, frame_rect.max.y), DragHandle::BottomLeft),
                            (frame_rect.max, DragHandle::BottomRight),
                        ];

                        for (corner, _handle) in &corners {
                            ui.painter().rect_filled(
                                egui::Rect::from_center_size(*corner, egui::vec2(handle_size, handle_size)),
                                2.0,
                                Color32::YELLOW,
                            );
                        }

                        // Edge handles (midpoints)
                        let edges = [
                            (egui::pos2((frame_rect.min.x + frame_rect.max.x) / 2.0, frame_rect.min.y), DragHandle::Top),
                            (egui::pos2((frame_rect.min.x + frame_rect.max.x) / 2.0, frame_rect.max.y), DragHandle::Bottom),
                            (egui::pos2(frame_rect.min.x, (frame_rect.min.y + frame_rect.max.y) / 2.0), DragHandle::Left),
                            (egui::pos2(frame_rect.max.x, (frame_rect.min.y + frame_rect.max.y) / 2.0), DragHandle::Right),
                        ];

                        for (edge, _handle) in &edges {
                            ui.painter().circle_filled(*edge, handle_size / 2.0, Color32::from_rgb(255, 200, 0));
                        }
                    }
                }

                // Handle mouse interactions
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let relative_pos = pos - rect.min;
                        let click_x = (relative_pos.x / zoom) as u32;
                        let click_y = (relative_pos.y / zoom) as u32;

                        // Check if clicking on an existing frame
                        let mut clicked_frame: Option<usize> = None;
                        for (i, frame) in animation_frames.iter().rev() {
                            if click_x >= frame.x && click_x < frame.x + frame.width &&
                               click_y >= frame.y && click_y < frame.y + frame.height {
                                clicked_frame = Some(*i);
                                break;
                            }
                        }

                        if let Some(idx) = clicked_frame {
                            // Select the clicked frame
                            new_selected_frame = Some(idx);
                        } else {
                            // Move current frame to clicked position
                            new_frame_values = Some((click_x, click_y, 0, 0)); // 0,0 means keep size
                            frame_modified = true;
                        }
                    }
                }

                // Handle dragging for resize/move
                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let relative_pos = pos - rect.min;
                        let drag_x = (relative_pos.x / zoom).max(0.0) as u32;
                        let drag_y = (relative_pos.y / zoom).max(0.0) as u32;

                        // Get current frame and check if we're dragging a handle
                        if let Some(frame) = animation_frames.get(selected_frame_idx).map(|(_, f)| f) {
                            let frame_right = frame.x + frame.width;
                            let frame_bottom = frame.y + frame.height;

                            // Determine which handle is being dragged based on initial drag position
                            if self.drag_state.is_none() {
                                // Start new drag - save undo state first
                                self.push_undo_state();

                                // Determine what we're dragging
                                let handle_margin = (8.0 / zoom) as u32;

                                let near_left = drag_x <= frame.x + handle_margin && drag_x + handle_margin >= frame.x;
                                let near_right = drag_x + handle_margin >= frame_right && drag_x <= frame_right + handle_margin;
                                let near_top = drag_y <= frame.y + handle_margin && drag_y + handle_margin >= frame.y;
                                let near_bottom = drag_y + handle_margin >= frame_bottom && drag_y <= frame_bottom + handle_margin;

                                let handle = if near_left && near_top {
                                    DragHandle::TopLeft
                                } else if near_right && near_top {
                                    DragHandle::TopRight
                                } else if near_left && near_bottom {
                                    DragHandle::BottomLeft
                                } else if near_right && near_bottom {
                                    DragHandle::BottomRight
                                } else if near_top {
                                    DragHandle::Top
                                } else if near_bottom {
                                    DragHandle::Bottom
                                } else if near_left {
                                    DragHandle::Left
                                } else if near_right {
                                    DragHandle::Right
                                } else {
                                    DragHandle::Move
                                };

                                self.drag_state = Some(DragState {
                                    handle,
                                    start_pos: (drag_x, drag_y),
                                    original_frame: frame.clone(),
                                    frame_index: (selected_anim_idx, selected_frame_idx),
                                });
                            }

                            // Apply drag
                            if let Some(ref drag) = self.drag_state {
                                let orig = &drag.original_frame;
                                let dx = drag_x as i32 - drag.start_pos.0 as i32;
                                let dy = drag_y as i32 - drag.start_pos.1 as i32;

                                let (new_x, new_y, new_w, new_h) = match drag.handle {
                                    DragHandle::Move => {
                                        ((orig.x as i32 + dx).max(0) as u32,
                                         (orig.y as i32 + dy).max(0) as u32,
                                         orig.width,
                                         orig.height)
                                    }
                                    DragHandle::TopLeft => {
                                        let new_x = (orig.x as i32 + dx).max(0) as u32;
                                        let new_y = (orig.y as i32 + dy).max(0) as u32;
                                        let new_w = ((orig.width as i32 - dx).max(1)) as u32;
                                        let new_h = ((orig.height as i32 - dy).max(1)) as u32;
                                        (new_x, new_y, new_w, new_h)
                                    }
                                    DragHandle::TopRight => {
                                        let new_w = ((orig.width as i32 + dx).max(1)) as u32;
                                        let new_y = (orig.y as i32 + dy).max(0) as u32;
                                        let new_h = ((orig.height as i32 - dy).max(1)) as u32;
                                        (orig.x, new_y, new_w, new_h)
                                    }
                                    DragHandle::BottomLeft => {
                                        let new_x = (orig.x as i32 + dx).max(0) as u32;
                                        let new_w = ((orig.width as i32 - dx).max(1)) as u32;
                                        let new_h = ((orig.height as i32 + dy).max(1)) as u32;
                                        (new_x, orig.y, new_w, new_h)
                                    }
                                    DragHandle::BottomRight => {
                                        let new_w = ((orig.width as i32 + dx).max(1)) as u32;
                                        let new_h = ((orig.height as i32 + dy).max(1)) as u32;
                                        (orig.x, orig.y, new_w, new_h)
                                    }
                                    DragHandle::Top => {
                                        let new_y = (orig.y as i32 + dy).max(0) as u32;
                                        let new_h = ((orig.height as i32 - dy).max(1)) as u32;
                                        (orig.x, new_y, orig.width, new_h)
                                    }
                                    DragHandle::Bottom => {
                                        let new_h = ((orig.height as i32 + dy).max(1)) as u32;
                                        (orig.x, orig.y, orig.width, new_h)
                                    }
                                    DragHandle::Left => {
                                        let new_x = (orig.x as i32 + dx).max(0) as u32;
                                        let new_w = ((orig.width as i32 - dx).max(1)) as u32;
                                        (new_x, orig.y, new_w, orig.height)
                                    }
                                    DragHandle::Right => {
                                        let new_w = ((orig.width as i32 + dx).max(1)) as u32;
                                        (orig.x, orig.y, new_w, orig.height)
                                    }
                                };

                                new_frame_values = Some((new_x, new_y, new_w, new_h));
                                frame_modified = true;
                            }
                        }
                    }
                }

                // Clear drag state when released
                if response.drag_stopped() {
                    self.drag_state = None;
                }
            });

        // Apply frame modifications
        if let Some((new_x, new_y, new_w, new_h)) = new_frame_values {
            if let Some(character) = self.characters.get_mut(self.selected_character) {
                if let Some(anim) = character.animations.get_mut(self.selected_animation) {
                    if let Some(frame) = anim.frames.get_mut(self.selected_frame) {
                        frame.x = new_x;
                        frame.y = new_y;
                        if new_w > 0 {
                            frame.width = new_w;
                        }
                        if new_h > 0 {
                            frame.height = new_h;
                        }
                        self.modified = true;
                        // Sync auto-gen settings with modified frame (position and size)
                        self.auto_gen.start_x = new_x;
                        self.auto_gen.start_y = new_y;
                        self.auto_gen.frame_width = frame.width;
                        self.auto_gen.frame_height = frame.height;
                    }
                }
            }
        }

        // Apply frame selection
        if let Some(idx) = new_selected_frame {
            self.selected_frame = idx;
            self.preview_state.current_frame = idx;
        }

        // Bottom info bar with sprite sheet size and mouse coordinates
        ui.horizontal(|ui| {
            // Sprite sheet size
            if let Some((w, h)) = self.sprite_sheet_size {
                ui.label(format!("Sheet: {}×{}", w, h));
                ui.separator();
            }

            // Mouse position on sprite sheet
            if let Some((x, y)) = self.mouse_pos_on_sheet {
                ui.label(format!("X: {} Y: {}", x, y));
            } else {
                ui.label(RichText::new("X: - Y: -").color(Color32::GRAY));
            }

            // Current frame info
            if let Some((_, frame)) = animation_frames.get(selected_frame_idx) {
                ui.separator();
                ui.label(format!("Frame: {}×{} @ ({},{})",
                    frame.width, frame.height, frame.x, frame.y));
            }
        });
    }

    /// Render the animated sprite preview.
    fn render_animation_preview(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.label(RichText::new("Animation Preview").strong());

        // Get current sprite sheet path
        let sprite_path = self.characters
            .get(self.selected_character)
            .map(|c| c.sprite_sheet_path.clone())
            .unwrap_or_default();

        // Load texture
        let texture = if !sprite_path.is_empty() {
            self.load_sprite_sheet(ctx, &sprite_path)
        } else {
            None
        };

        // Get current animation info
        let (frames, fps, speed, playback_mode, scale) = self.characters
            .get(self.selected_character)
            .and_then(|c| {
                let scale = c.scale;
                c.animations.get(self.selected_animation).map(|a| (a.frames.clone(), a.fps, a.speed, a.playback_mode, scale))
            })
            .unwrap_or_else(|| (Vec::new(), 8.0, 1.0, PlaybackMode::Normal, 2.0));

        // Animation controls
        ui.horizontal(|ui| {
            if ui.button(if self.preview_state.playing { "⏸" } else { "▶" }).clicked() {
                self.preview_state.playing = !self.preview_state.playing;
            }
            if ui.button("⏮").clicked() {
                self.preview_state.current_frame = 0;
                self.preview_state.ping_pong_forward = true;
            }
            if ui.button("⏭").clicked() {
                if !frames.is_empty() {
                    self.preview_state.current_frame = frames.len() - 1;
                }
            }
            // Show current playback mode
            ui.label(RichText::new(format!("({})", playback_mode.display_name())).small().color(Color32::GRAY));
        });

        // Advance animation frame if playing (respecting playback mode and speed)
        if self.preview_state.playing && !frames.is_empty() {
            let effective_fps = fps * speed;
            let frame_duration = std::time::Duration::from_secs_f32(1.0 / effective_fps);
            if self.preview_state.last_frame_time.elapsed() >= frame_duration {
                match playback_mode {
                    PlaybackMode::Normal => {
                        self.preview_state.current_frame = (self.preview_state.current_frame + 1) % frames.len();
                    }
                    PlaybackMode::Reverse => {
                        if self.preview_state.current_frame == 0 {
                            self.preview_state.current_frame = frames.len() - 1;
                        } else {
                            self.preview_state.current_frame -= 1;
                        }
                    }
                    PlaybackMode::PingPong => {
                        if frames.len() > 1 {
                            if self.preview_state.ping_pong_forward {
                                if self.preview_state.current_frame >= frames.len() - 1 {
                                    self.preview_state.ping_pong_forward = false;
                                    self.preview_state.current_frame = frames.len().saturating_sub(2);
                                } else {
                                    self.preview_state.current_frame += 1;
                                }
                            } else {
                                if self.preview_state.current_frame == 0 {
                                    self.preview_state.ping_pong_forward = true;
                                    self.preview_state.current_frame = 1.min(frames.len() - 1);
                                } else {
                                    self.preview_state.current_frame -= 1;
                                }
                            }
                        }
                    }
                    PlaybackMode::Random => {
                        use std::time::SystemTime;
                        let seed = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_nanos() as usize)
                            .unwrap_or(0);
                        self.preview_state.current_frame = seed % frames.len();
                    }
                }
                self.preview_state.last_frame_time = Instant::now();
                ctx.request_repaint(); // Keep animating
            } else {
                ctx.request_repaint_after(frame_duration - self.preview_state.last_frame_time.elapsed());
            }
        }

        // Frame indicator
        if !frames.is_empty() {
            ui.label(format!("Frame: {} / {}", self.preview_state.current_frame + 1, frames.len()));
        } else {
            ui.label(RichText::new("No frames").color(Color32::GRAY).italics());
        }

        ui.add_space(8.0);

        // Draw the current frame in a fixed-size container to prevent layout shifts
        // Use a fixed preview area height so resizing frames doesn't change the window size
        let preview_area_height = 150.0;

        if let (Some(texture), Some(frame)) = (&texture, frames.get(self.preview_state.current_frame)) {
            let texture_size = texture.size_vec2();

            // Calculate frame size, capping to fit within preview area
            let desired_size = egui::vec2(frame.width as f32 * scale, frame.height as f32 * scale);
            let max_preview_height = preview_area_height - 30.0; // Leave room for label
            let display_scale = if desired_size.y > max_preview_height {
                max_preview_height / desired_size.y
            } else {
                1.0
            };
            let frame_size = egui::vec2(
                desired_size.x * display_scale,
                desired_size.y * display_scale,
            );

            // Calculate UV coordinates for the frame
            let uv_min = egui::pos2(
                frame.x as f32 / texture_size.x,
                frame.y as f32 / texture_size.y,
            );
            let uv_max = egui::pos2(
                (frame.x + frame.width) as f32 / texture_size.x,
                (frame.y + frame.height) as f32 / texture_size.y,
            );

            // Allocate the fixed-height preview area
            let available_width = ui.available_width();
            let (preview_rect, _) = ui.allocate_exact_size(
                egui::vec2(available_width, preview_area_height),
                egui::Sense::hover()
            );

            // Draw background for the entire preview area
            ui.painter().rect_filled(preview_rect, 4.0, Color32::from_rgb(30, 30, 40));

            // Center the sprite within the preview area
            let sprite_rect = egui::Rect::from_center_size(
                egui::pos2(preview_rect.center().x, preview_rect.center().y - 10.0),
                frame_size,
            );

            // Draw sprite background
            ui.painter().rect_filled(sprite_rect, 2.0, Color32::from_rgb(40, 40, 50));

            // Draw the sprite frame
            ui.painter().image(
                texture.id(),
                sprite_rect,
                egui::Rect::from_min_max(uv_min, uv_max),
                Color32::WHITE,
            );

            // Draw border
            ui.painter().rect_stroke(sprite_rect, 2.0, egui::Stroke::new(1.0, Color32::GRAY));

            // Draw label at bottom of preview area
            let label_pos = egui::pos2(preview_rect.center().x, preview_rect.bottom() - 12.0);
            ui.painter().text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                format!("{}×{} @ {:.0} FPS", frame.width, frame.height, fps),
                egui::FontId::default(),
                Color32::LIGHT_GRAY,
            );
        } else {
            // Allocate the fixed-height preview area even when no texture
            let available_width = ui.available_width();
            let (preview_rect, _) = ui.allocate_exact_size(
                egui::vec2(available_width, preview_area_height),
                egui::Sense::hover()
            );
            ui.painter().rect_filled(preview_rect, 4.0, Color32::from_rgb(30, 30, 40));

            let msg = if texture.is_none() && !sprite_path.is_empty() {
                "Texture not loaded"
            } else {
                "No frame selected"
            };
            ui.painter().text(
                preview_rect.center(),
                egui::Align2::CENTER_CENTER,
                msg,
                egui::FontId::default(),
                Color32::GRAY,
            );
        }

        // Click on a frame in the list to preview it
        ui.add_space(8.0);
        ui.label(RichText::new("Click frame to preview:").small());
        ui.horizontal_wrapped(|ui| {
            for (i, _) in frames.iter().enumerate() {
                let selected = self.preview_state.current_frame == i;
                let text = format!("{}", i);
                if ui.selectable_label(selected, text).clicked() {
                    self.preview_state.current_frame = i;
                    self.preview_state.playing = false;
                }
            }
        });
    }

    fn render_character_list(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Characters").strong());

        let mut to_select = None;
        for (i, character) in self.characters.iter().enumerate() {
            let selected = self.selected_character == i;
            let text = format!("📁 {}", character.name);
            if ui.selectable_label(selected, text).clicked() {
                to_select = Some(i);
            }
        }
        if let Some(i) = to_select {
            self.selected_character = i;
            self.selected_animation = 0;
            self.selected_frame = 0;
            // Reset animation preview
            self.preview_state.current_frame = 0;
            self.preview_state.last_frame_time = Instant::now();
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.small_button("➕").on_hover_text("Add Character").clicked() {
                self.push_undo_state();
                self.add_character();
            }
            if ui.small_button("🗑").on_hover_text("Remove Character").clicked() {
                self.push_undo_state();
                self.remove_current_character();
            }
        });
    }

    fn render_animation_list(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Animations").strong());

        if let Some(character) = self.characters.get(self.selected_character) {
            // Fixed height for animation list
            egui::ScrollArea::vertical()
                .id_salt("animation_list_scroll")
                .max_height(600.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut to_select = None;
                    for (i, anim) in character.animations.iter().enumerate() {
                        let selected = self.selected_animation == i;
                        let status = if anim.enabled && !anim.frames.is_empty() {
                            format!(" ({} frames)", anim.frames.len())
                        } else {
                            String::new()
                        };
                        let color = if anim.enabled && !anim.frames.is_empty() {
                            Color32::WHITE
                        } else {
                            Color32::GRAY
                        };
                        let text = format!("{} {}{}", anim.action.icon(), anim.action.display_name(), status);
                        if ui.selectable_label(selected, RichText::new(text).color(color)).clicked() {
                            to_select = Some(i);
                        }
                    }
                    if let Some(i) = to_select {
                        self.selected_animation = i;
                        self.selected_frame = 0;
                        // Reset animation preview
                        self.preview_state.current_frame = 0;
                        self.preview_state.last_frame_time = Instant::now();
                    }
                });
        }
    }

    fn render_frame_editor(&mut self, ui: &mut Ui) {
        let mut was_modified = false;

        // Character settings
        ui.label(RichText::new("Character Settings").strong());

        let selected_char = self.selected_character;
        if let Some(character) = self.characters.get_mut(selected_char) {
            ui.horizontal(|ui| {
                ui.label("ID:");
                if ui.text_edit_singleline(&mut character.id).changed() {
                    was_modified = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut character.name).changed() {
                    was_modified = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Sprite Sheet:");
                if ui.text_edit_singleline(&mut character.sprite_sheet_path).changed() {
                    was_modified = true;
                }
            });

            let mut w = character.default_frame_width as i32;
            let mut h = character.default_frame_height as i32;
            let mut scale = character.scale;

            ui.horizontal(|ui| {
                ui.label("Default Frame Size:");
                ui.label("W:");
                if ui.add(egui::DragValue::new(&mut w).range(1..=512)).changed() {
                    was_modified = true;
                }
                ui.label("H:");
                if ui.add(egui::DragValue::new(&mut h).range(1..=512)).changed() {
                    was_modified = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Scale:");
                if ui.add(egui::DragValue::new(&mut scale).speed(0.1).range(0.1..=10.0)).changed() {
                    was_modified = true;
                }
            });

            // Apply changes
            character.default_frame_width = w as u32;
            character.default_frame_height = h as u32;
            character.scale = scale;
        }

        // Update auto-gen if character settings changed
        if was_modified {
            if let Some(character) = self.characters.get(selected_char) {
                self.auto_gen.frame_width = character.default_frame_width;
                self.auto_gen.frame_height = character.default_frame_height;
                self.auto_gen.x_offset = character.default_frame_width as i32;
            }
        }

        ui.add_space(8.0);
        ui.separator();

        // Animation settings
        ui.label(RichText::new("Animation Settings").strong());

        let selected_animation = self.selected_animation;
        let selected_char = self.selected_character;
        if let Some(character) = self.characters.get_mut(selected_char) {
            if let Some(anim) = character.animations.get_mut(selected_animation) {
                let mut enabled = anim.enabled;
                let mut fps = anim.fps;
                let mut speed = anim.speed;
                let mut looping = anim.looping;
                let mut playback_mode = anim.playback_mode;

                ui.horizontal(|ui| {
                    ui.label("Enabled:");
                    if ui.checkbox(&mut enabled, "").changed() {
                        was_modified = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    if ui.add(egui::DragValue::new(&mut fps).speed(0.5).range(1.0..=60.0)).changed() {
                        was_modified = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    if ui.add(egui::DragValue::new(&mut speed).speed(0.1).range(0.1..=5.0).suffix("x")).changed() {
                        was_modified = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Looping:");
                    if ui.checkbox(&mut looping, "").changed() {
                        was_modified = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Playback:");
                    egui::ComboBox::from_id_salt("playback_mode")
                        .selected_text(playback_mode.display_name())
                        .show_ui(ui, |ui| {
                            for mode in PlaybackMode::all() {
                                if ui.selectable_value(&mut playback_mode, *mode, mode.display_name()).changed() {
                                    was_modified = true;
                                }
                            }
                        });
                });

                anim.enabled = enabled;
                anim.fps = fps;
                anim.speed = speed;
                anim.looping = looping;
                anim.playback_mode = playback_mode;
            }
        }

        // Copy frames to another animation
        ui.add_space(4.0);
        let mut copy_target: Option<usize> = None;
        if let Some(character) = self.characters.get(selected_char) {
            if let Some(anim) = character.animations.get(selected_animation) {
                if !anim.frames.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Copy frames to:");
                        egui::ComboBox::from_id_salt("copy_to_animation")
                            .selected_text("Select...")
                            .show_ui(ui, |ui| {
                                for (i, other_anim) in character.animations.iter().enumerate() {
                                    if i != selected_animation {
                                        let label = format!("{} {}", other_anim.action.icon(), other_anim.action.display_name());
                                        if ui.selectable_label(false, label).clicked() {
                                            copy_target = Some(i);
                                        }
                                    }
                                }
                            });
                    });
                }
            }
        }

        // Perform the copy operation
        if let Some(target_idx) = copy_target {
            self.push_undo_state();
            if let Some(character) = self.characters.get_mut(selected_char) {
                // Clone frames from source
                let frames_to_copy = character.animations
                    .get(selected_animation)
                    .map(|a| a.frames.clone())
                    .unwrap_or_default();
                // Apply to target
                if let Some(target_anim) = character.animations.get_mut(target_idx) {
                    target_anim.frames = frames_to_copy;
                    target_anim.enabled = true;
                    was_modified = true;
                    self.status_message = Some((
                        format!("Copied frames to {}", target_anim.action.display_name()),
                        Color32::GREEN,
                    ));
                }
            }
        }

        ui.add_space(8.0);
        ui.separator();

        // Auto-generation
        ui.label(RichText::new("Auto-Generate Frames").strong());
        ui.label("Define the first frame, then auto-generate the rest:");

        ui.horizontal(|ui| {
            ui.label("Start X:");
            ui.add(egui::DragValue::new(&mut self.auto_gen.start_x).range(0..=4096));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut self.auto_gen.start_y).range(0..=4096));
        });
        ui.horizontal(|ui| {
            ui.label("Frame W:");
            ui.add(egui::DragValue::new(&mut self.auto_gen.frame_width).range(1..=512));
            ui.label("H:");
            ui.add(egui::DragValue::new(&mut self.auto_gen.frame_height).range(1..=512));
        });
        ui.horizontal(|ui| {
            ui.label("# Frames:");
            ui.add(egui::DragValue::new(&mut self.auto_gen.frame_count).range(1..=64));
        });
        ui.horizontal(|ui| {
            ui.label("X Offset:");
            ui.add(egui::DragValue::new(&mut self.auto_gen.x_offset).range(-512..=512));
            ui.label("Y Offset:");
            ui.add(egui::DragValue::new(&mut self.auto_gen.y_offset).range(-512..=512));
        });

        let mut apply_auto_gen = false;
        ui.horizontal(|ui| {
            if ui.button("🔧 Generate Frames").clicked() {
                apply_auto_gen = true;
            }
            if ui.button("↔ Horizontal Strip").clicked() {
                self.auto_gen.x_offset = self.auto_gen.frame_width as i32;
                self.auto_gen.y_offset = 0;
            }
            if ui.button("↕ Vertical Strip").clicked() {
                self.auto_gen.x_offset = 0;
                self.auto_gen.y_offset = self.auto_gen.frame_height as i32;
            }
        });

        if apply_auto_gen {
            self.push_undo_state();
            self.apply_auto_gen();
            was_modified = true;
        }

        ui.add_space(8.0);
        ui.separator();

        // Manual frame list
        ui.label(RichText::new("Frames").strong());

        let mut frame_to_remove: Option<usize> = None;
        let mut frame_to_add = false;

        // Collect frame data for display
        let selected_char = self.selected_character;
        let selected_animation = self.selected_animation;

        if let Some(character) = self.characters.get_mut(selected_char) {
            if let Some(anim) = character.animations.get_mut(selected_animation) {
                if anim.frames.is_empty() {
                    ui.label(RichText::new("No frames defined").color(Color32::GRAY).italics());
                } else {
                    for i in 0..anim.frames.len() {
                        let mut x = anim.frames[i].x as i32;
                        let mut y = anim.frames[i].y as i32;
                        let mut w = anim.frames[i].width as i32;
                        let mut h = anim.frames[i].height as i32;
                        let mut remove_this = false;

                        let is_selected_frame = self.selected_frame == i;
                        ui.horizontal(|ui| {
                            // Clickable frame label to select it
                            if ui.selectable_label(is_selected_frame, format!("Frame {}:", i)).clicked() {
                                self.selected_frame = i;
                                self.preview_state.current_frame = i;
                            }
                            ui.label("X:");
                            if ui.add(egui::DragValue::new(&mut x).range(0..=4096)).changed() {
                                was_modified = true;
                            }
                            ui.label("Y:");
                            if ui.add(egui::DragValue::new(&mut y).range(0..=4096)).changed() {
                                was_modified = true;
                            }
                            ui.label("W:");
                            if ui.add(egui::DragValue::new(&mut w).range(1..=512)).changed() {
                                was_modified = true;
                            }
                            ui.label("H:");
                            if ui.add(egui::DragValue::new(&mut h).range(1..=512)).changed() {
                                was_modified = true;
                            }
                            if ui.small_button("🗑").clicked() {
                                remove_this = true;
                            }
                        });

                        // Apply changes
                        anim.frames[i].x = x as u32;
                        anim.frames[i].y = y as u32;
                        anim.frames[i].width = w as u32;
                        anim.frames[i].height = h as u32;

                        // Update auto_gen and selection when selected frame changes via input
                        if is_selected_frame && was_modified {
                            self.auto_gen.start_x = x as u32;
                            self.auto_gen.start_y = y as u32;
                            self.auto_gen.frame_width = w as u32;
                            self.auto_gen.frame_height = h as u32;
                        }

                        if remove_this {
                            frame_to_remove = Some(i);
                        }
                    }
                }

                ui.add_space(4.0);
                if ui.button("➕ Add Frame").clicked() {
                    frame_to_add = true;
                }
            }
        }

        // Apply frame modifications after the borrow ends
        if let Some(idx) = frame_to_remove {
            self.push_undo_state();
            if let Some(character) = self.characters.get_mut(selected_char) {
                if let Some(anim) = character.animations.get_mut(selected_animation) {
                    anim.frames.remove(idx);
                    was_modified = true;
                }
            }
        }

        if frame_to_add {
            self.push_undo_state();
            if let Some(character) = self.characters.get_mut(selected_char) {
                if let Some(anim) = character.animations.get_mut(selected_animation) {
                    let last_frame = anim.frames.last().cloned().unwrap_or(SpriteFrame::default());
                    anim.frames.push(SpriteFrame {
                        x: last_frame.x + last_frame.width,
                        y: last_frame.y,
                        width: last_frame.width,
                        height: last_frame.height,
                    });
                    was_modified = true;
                }
            }
        }

        if was_modified {
            self.modified = true;
        }
    }
}
