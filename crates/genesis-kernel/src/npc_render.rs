//! NPC rendering system for sprite-based characters.
//!
//! This module provides instanced rendering for NPCs with:
//! - 4-directional sprites (N/S/E/W)
//! - Animation frame support
//! - Efficient batch rendering via instancing
//! - Speech bubble overlays

use bytemuck::{Pod, Zeroable};

/// Maximum number of NPCs that can be rendered in a single batch.
pub const MAX_VISIBLE_NPCS: usize = 1000;

/// Maximum number of speech bubbles visible at once.
pub const MAX_SPEECH_BUBBLES: usize = 32;

/// Cardinal direction for NPC facing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FacingDirection {
    /// Facing north (up)
    North = 0,
    /// Facing south (down)
    #[default]
    South = 1,
    /// Facing east (right)
    East = 2,
    /// Facing west (left)
    West = 3,
}

impl FacingDirection {
    /// Creates a FacingDirection from a u8 value.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::North,
            2 => Self::East,
            3 => Self::West,
            _ => Self::South, // Default to South for 1 and invalid values
        }
    }

    /// Returns the UV row offset for this direction (for sprite sheets).
    #[must_use]
    pub fn uv_row(&self) -> f32 {
        match self {
            Self::South => 0.0,
            Self::West => 0.25,
            Self::East => 0.5,
            Self::North => 0.75,
        }
    }

    /// Creates a FacingDirection from a velocity vector.
    #[must_use]
    pub fn from_velocity(vx: f32, vy: f32) -> Self {
        if vx.abs() > vy.abs() {
            if vx > 0.0 {
                Self::East
            } else {
                Self::West
            }
        } else if vy > 0.0 {
            Self::South
        } else {
            Self::North
        }
    }
}

/// NPC render data for a single NPC.
#[derive(Debug, Clone, Copy)]
pub struct NpcRenderData {
    /// World position (x, y)
    pub position: (f32, f32),
    /// Facing direction
    pub direction: FacingDirection,
    /// Current animation frame (0-based)
    pub animation_frame: u32,
    /// NPC type ID (determines which sprite sheet to use)
    pub npc_type: u8,
    /// Render scale multiplier
    pub scale: f32,
    /// Tint color (RGBA, 1.0 = no tint)
    pub tint: [f32; 4],
    /// Whether NPC is visible
    pub visible: bool,
}

impl Default for NpcRenderData {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            direction: FacingDirection::South,
            animation_frame: 0,
            npc_type: 0,
            scale: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            visible: true,
        }
    }
}

impl NpcRenderData {
    /// Creates a new NPC render data at the given position.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            ..Default::default()
        }
    }

    /// Sets the NPC type.
    #[must_use]
    pub const fn with_type(mut self, npc_type: u8) -> Self {
        self.npc_type = npc_type;
        self
    }

    /// Sets the facing direction.
    #[must_use]
    pub const fn with_direction(mut self, direction: FacingDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the animation frame.
    #[must_use]
    pub const fn with_frame(mut self, frame: u32) -> Self {
        self.animation_frame = frame;
        self
    }

    /// Sets the scale.
    #[must_use]
    pub const fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the tint color.
    #[must_use]
    pub const fn with_tint(mut self, tint: [f32; 4]) -> Self {
        self.tint = tint;
        self
    }
}

/// GPU instance data for a single NPC sprite.
///
/// This is the data sent to the GPU for instanced rendering.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct NpcInstance {
    /// World position (x, y) in pixels
    pub position: [f32; 2],
    /// Sprite size (width, height) in pixels
    pub size: [f32; 2],
    /// UV offset in sprite sheet (x, y)
    pub uv_offset: [f32; 2],
    /// UV size in sprite sheet (width, height)
    pub uv_size: [f32; 2],
    /// Tint color (RGBA)
    pub tint: [f32; 4],
}

impl Default for NpcInstance {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [32.0, 32.0],
            uv_offset: [0.0, 0.0],
            uv_size: [0.25, 0.25], // 4x4 sprite sheet default
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl NpcInstance {
    /// Size of the instance data in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Creates an NPC instance from render data.
    #[must_use]
    pub fn from_render_data(data: &NpcRenderData, sprite_config: &SpriteSheetConfig) -> Self {
        let frame_count = sprite_config.frames_per_direction;
        let frame = data.animation_frame % frame_count;

        // Calculate UV coordinates based on direction and frame
        let uv_width = 1.0 / frame_count as f32;
        let uv_height = 0.25; // 4 directions = 4 rows

        Self {
            position: [data.position.0, data.position.1],
            size: [
                sprite_config.sprite_width * data.scale,
                sprite_config.sprite_height * data.scale,
            ],
            uv_offset: [frame as f32 * uv_width, data.direction.uv_row()],
            uv_size: [uv_width, uv_height],
            tint: data.tint,
        }
    }
}

/// Configuration for a sprite sheet.
#[derive(Debug, Clone, Copy)]
pub struct SpriteSheetConfig {
    /// Width of a single sprite in pixels
    pub sprite_width: f32,
    /// Height of a single sprite in pixels
    pub sprite_height: f32,
    /// Number of animation frames per direction
    pub frames_per_direction: u32,
}

impl Default for SpriteSheetConfig {
    fn default() -> Self {
        Self {
            sprite_width: 32.0,
            sprite_height: 32.0,
            frames_per_direction: 4,
        }
    }
}

/// Speech bubble data for rendering text above NPCs.
#[derive(Debug, Clone)]
pub struct SpeechBubble {
    /// NPC ID this bubble belongs to
    pub npc_id: u32,
    /// Position in world coordinates
    pub position: (f32, f32),
    /// Text to display
    pub text: String,
    /// Bubble width in pixels
    pub width: f32,
    /// Bubble height in pixels
    pub height: f32,
    /// Current opacity (0.0 to 1.0 for fade in/out)
    pub opacity: f32,
    /// Time remaining to display (seconds)
    pub time_remaining: f32,
}

impl SpeechBubble {
    /// Creates a new speech bubble.
    #[must_use]
    pub fn new(npc_id: u32, position: (f32, f32), text: String) -> Self {
        // Estimate bubble size based on text length
        let char_width = 8.0;
        let padding = 16.0;
        let width = (text.len() as f32 * char_width + padding * 2.0).min(200.0);
        let height = 32.0;

        Self {
            npc_id,
            position,
            text,
            width,
            height,
            opacity: 0.0,
            time_remaining: 5.0,
        }
    }

    /// Updates the bubble animation.
    ///
    /// Returns true if the bubble should still be displayed.
    pub fn update(&mut self, dt: f32) -> bool {
        self.time_remaining -= dt;

        // Fade in during first 0.3 seconds
        if self.time_remaining > 4.7 {
            self.opacity = (5.0 - self.time_remaining) / 0.3;
        }
        // Fade out during last 0.5 seconds
        else if self.time_remaining < 0.5 {
            self.opacity = self.time_remaining / 0.5;
        } else {
            self.opacity = 1.0;
        }

        self.opacity = self.opacity.clamp(0.0, 1.0);
        self.time_remaining > 0.0
    }
}

/// GPU data for speech bubble background.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SpeechBubbleInstance {
    /// Position (x, y) in screen coordinates
    pub position: [f32; 2],
    /// Size (width, height)
    pub size: [f32; 2],
    /// Background color (RGBA)
    pub color: [f32; 4],
    /// Corner radius
    pub corner_radius: f32,
    /// Border width
    pub border_width: f32,
    /// Padding for alignment
    _padding: [f32; 2],
}

impl Default for SpeechBubbleInstance {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [100.0, 32.0],
            color: [1.0, 1.0, 1.0, 0.9],
            corner_radius: 8.0,
            border_width: 2.0,
            _padding: [0.0; 2],
        }
    }
}

/// Manages NPC rendering state.
pub struct NpcRenderManager {
    /// Current NPC instances for rendering
    instances: Vec<NpcInstance>,
    /// Sprite sheet configurations by NPC type
    sprite_configs: Vec<SpriteSheetConfig>,
    /// Active speech bubbles
    speech_bubbles: Vec<SpeechBubble>,
    /// Whether instance buffer needs updating
    dirty: bool,
}

impl Default for NpcRenderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcRenderManager {
    /// Creates a new NPC render manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(MAX_VISIBLE_NPCS),
            sprite_configs: vec![SpriteSheetConfig::default()],
            speech_bubbles: Vec::with_capacity(MAX_SPEECH_BUBBLES),
            dirty: true,
        }
    }

    /// Registers a sprite sheet configuration for an NPC type.
    pub fn register_sprite_config(&mut self, npc_type: u8, config: SpriteSheetConfig) {
        let idx = npc_type as usize;
        if idx >= self.sprite_configs.len() {
            self.sprite_configs
                .resize(idx + 1, SpriteSheetConfig::default());
        }
        self.sprite_configs[idx] = config;
    }

    /// Gets the sprite config for an NPC type.
    #[must_use]
    pub fn get_sprite_config(&self, npc_type: u8) -> &SpriteSheetConfig {
        self.sprite_configs
            .get(npc_type as usize)
            .unwrap_or(&self.sprite_configs[0])
    }

    /// Updates NPC instances from render data.
    pub fn update_instances(&mut self, npcs: &[NpcRenderData]) {
        self.instances.clear();

        for npc in npcs {
            if !npc.visible {
                continue;
            }

            let config = self.get_sprite_config(npc.npc_type);
            self.instances
                .push(NpcInstance::from_render_data(npc, config));

            if self.instances.len() >= MAX_VISIBLE_NPCS {
                break;
            }
        }

        self.dirty = true;
    }

    /// Returns the current instances for rendering.
    #[must_use]
    pub fn instances(&self) -> &[NpcInstance] {
        &self.instances
    }

    /// Returns the number of visible NPCs.
    #[must_use]
    pub fn visible_count(&self) -> usize {
        self.instances.len()
    }

    /// Checks if the instance buffer needs updating.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the instance buffer as clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Adds a speech bubble for an NPC.
    pub fn add_speech_bubble(&mut self, npc_id: u32, position: (f32, f32), text: String) {
        // Remove existing bubble for this NPC
        self.speech_bubbles.retain(|b| b.npc_id != npc_id);

        if self.speech_bubbles.len() < MAX_SPEECH_BUBBLES {
            self.speech_bubbles
                .push(SpeechBubble::new(npc_id, position, text));
        }
    }

    /// Updates speech bubbles animation.
    pub fn update_speech_bubbles(&mut self, dt: f32) {
        self.speech_bubbles.retain_mut(|bubble| bubble.update(dt));
    }

    /// Returns active speech bubbles.
    #[must_use]
    pub fn speech_bubbles(&self) -> &[SpeechBubble] {
        &self.speech_bubbles
    }

    /// Clears all speech bubbles.
    pub fn clear_speech_bubbles(&mut self) {
        self.speech_bubbles.clear();
    }
}

/// NPC render shader in WGSL.
///
/// This shader renders NPC sprites using instanced rendering.
pub const NPC_RENDER_SHADER: &str = r"
// NPC instance data
struct NpcInstance {
    position: vec2<f32>,
    size: vec2<f32>,
    uv_offset: vec2<f32>,
    uv_size: vec2<f32>,
    tint: vec4<f32>,
}

// Camera/render parameters
struct RenderParams {
    screen_size: vec2<f32>,
    camera_pos: vec2<f32>,
    zoom: f32,
    time: f32,
}

@group(0) @binding(0) var<storage, read> instances: array<NpcInstance>;
@group(0) @binding(1) var<uniform> params: RenderParams;
@group(0) @binding(2) var sprite_texture: texture_2d<f32>;
@group(0) @binding(3) var sprite_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    let instance = instances[instance_idx];

    // Quad vertices (two triangles)
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0)
    );

    let local_pos = positions[vertex_idx];

    // World position
    let world_pos = instance.position + local_pos * instance.size;

    // Convert to screen space
    let screen_pos = (world_pos - params.camera_pos) * params.zoom;
    let ndc = (screen_pos / params.screen_size) * 2.0 - 1.0;

    var out: VertexOutput;
    out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.uv = instance.uv_offset + local_pos * instance.uv_size;
    out.tint = instance.tint;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv);

    // Apply tint and discard transparent pixels
    let final_color = tex_color * in.tint;
    if final_color.a < 0.01 {
        discard;
    }

    return final_color;
}
";

/// Speech bubble render shader in WGSL.
pub const SPEECH_BUBBLE_SHADER: &str = r"
// Speech bubble instance
struct BubbleInstance {
    position: vec2<f32>,
    size: vec2<f32>,
    color: vec4<f32>,
    corner_radius: f32,
    border_width: f32,
    _padding: vec2<f32>,
}

struct RenderParams {
    screen_size: vec2<f32>,
    camera_pos: vec2<f32>,
    zoom: f32,
    time: f32,
}

@group(0) @binding(0) var<storage, read> bubbles: array<BubbleInstance>;
@group(0) @binding(1) var<uniform> params: RenderParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) corner_radius: f32,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    let bubble = bubbles[instance_idx];

    var positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0)
    );

    let local_pos = positions[vertex_idx];
    let world_pos = bubble.position + local_pos * bubble.size;
    let screen_pos = (world_pos - params.camera_pos) * params.zoom;
    let ndc = (screen_pos / params.screen_size) * 2.0 - 1.0;

    var out: VertexOutput;
    out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.local_pos = local_pos * bubble.size;
    out.size = bubble.size;
    out.color = bubble.color;
    out.corner_radius = bubble.corner_radius;

    return out;
}

// Signed distance to rounded rectangle
fn sd_rounded_rect(p: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p - size * 0.5) - size * 0.5 + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = sd_rounded_rect(in.local_pos, in.size, in.corner_radius);

    // Smooth edge
    let alpha = 1.0 - smoothstep(-1.0, 1.0, d);

    if alpha < 0.01 {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_from_u8() {
        assert_eq!(FacingDirection::from_u8(0), FacingDirection::North);
        assert_eq!(FacingDirection::from_u8(1), FacingDirection::South);
        assert_eq!(FacingDirection::from_u8(2), FacingDirection::East);
        assert_eq!(FacingDirection::from_u8(3), FacingDirection::West);
        assert_eq!(FacingDirection::from_u8(255), FacingDirection::South); // Invalid defaults to South
    }

    #[test]
    fn test_direction_from_velocity() {
        assert_eq!(
            FacingDirection::from_velocity(1.0, 0.0),
            FacingDirection::East
        );
        assert_eq!(
            FacingDirection::from_velocity(-1.0, 0.0),
            FacingDirection::West
        );
        assert_eq!(
            FacingDirection::from_velocity(0.0, 1.0),
            FacingDirection::South
        );
        assert_eq!(
            FacingDirection::from_velocity(0.0, -1.0),
            FacingDirection::North
        );
    }

    #[test]
    fn test_direction_uv_row() {
        assert_eq!(FacingDirection::South.uv_row(), 0.0);
        assert_eq!(FacingDirection::West.uv_row(), 0.25);
        assert_eq!(FacingDirection::East.uv_row(), 0.5);
        assert_eq!(FacingDirection::North.uv_row(), 0.75);
    }

    #[test]
    fn test_npc_render_data_builder() {
        let data = NpcRenderData::new(100.0, 200.0)
            .with_type(5)
            .with_direction(FacingDirection::East)
            .with_frame(2)
            .with_scale(2.0);

        assert_eq!(data.position, (100.0, 200.0));
        assert_eq!(data.npc_type, 5);
        assert_eq!(data.direction, FacingDirection::East);
        assert_eq!(data.animation_frame, 2);
        assert_eq!(data.scale, 2.0);
    }

    #[test]
    fn test_npc_instance_from_render_data() {
        let config = SpriteSheetConfig {
            sprite_width: 32.0,
            sprite_height: 32.0,
            frames_per_direction: 4,
        };

        let data = NpcRenderData::new(100.0, 200.0)
            .with_direction(FacingDirection::East)
            .with_frame(2)
            .with_scale(1.5);

        let instance = NpcInstance::from_render_data(&data, &config);

        assert_eq!(instance.position, [100.0, 200.0]);
        assert_eq!(instance.size, [48.0, 48.0]); // 32 * 1.5
        assert_eq!(instance.uv_offset[0], 0.5); // Frame 2 of 4
        assert_eq!(instance.uv_offset[1], 0.5); // East row
    }

    #[test]
    fn test_npc_instance_size() {
        assert_eq!(NpcInstance::SIZE, 48); // 2+2+2+2+4 floats * 4 bytes
    }

    #[test]
    fn test_npc_render_manager() {
        let mut manager = NpcRenderManager::new();

        let npcs = vec![
            NpcRenderData::new(0.0, 0.0),
            NpcRenderData::new(100.0, 100.0),
        ];

        manager.update_instances(&npcs);

        assert_eq!(manager.visible_count(), 2);
        assert!(manager.is_dirty());

        manager.mark_clean();
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_speech_bubble_animation() {
        let mut bubble = SpeechBubble::new(1, (0.0, 0.0), "Hello!".to_string());

        // Initially opacity should be 0
        assert_eq!(bubble.opacity, 0.0);

        // After update, should start fading in
        assert!(bubble.update(0.1));
        assert!(bubble.opacity > 0.0);

        // After full fade in
        bubble.update(0.3);
        assert!((bubble.opacity - 1.0).abs() < 0.1);

        // Fast forward to near end
        bubble.time_remaining = 0.3;
        bubble.update(0.0);
        assert!(bubble.opacity < 1.0);

        // After expiry
        bubble.time_remaining = 0.0;
        assert!(!bubble.update(0.1));
    }

    #[test]
    fn test_sprite_config_registration() {
        let mut manager = NpcRenderManager::new();

        let custom_config = SpriteSheetConfig {
            sprite_width: 64.0,
            sprite_height: 64.0,
            frames_per_direction: 8,
        };

        manager.register_sprite_config(5, custom_config);

        let config = manager.get_sprite_config(5);
        assert_eq!(config.sprite_width, 64.0);
        assert_eq!(config.frames_per_direction, 8);

        // Unknown type should return default
        let default_config = manager.get_sprite_config(100);
        assert_eq!(default_config.sprite_width, 32.0);
    }
}
