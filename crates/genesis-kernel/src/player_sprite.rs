//! Player sprite rendering system.
//!
//! Renders the player character with animated sprites using per-frame
//! coordinates from TOML animation definitions. Supports multiple
//! animation actions (idle, walk, run, use, punch, jump) and four
//! directions (down, up, left, right).

use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

// ============================================================================
// Animation Action & Direction
// ============================================================================

/// Player animation action (what the character is doing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlayerAnimAction {
    /// Standing still
    #[default]
    Idle,
    /// Walking (normal speed)
    Walk,
    /// Running (fast speed)
    Run,
    /// Using an item / interacting
    Use,
    /// Punching / melee attack
    Punch,
    /// Jumping
    Jump,
}

impl PlayerAnimAction {
    /// Get the TOML action name prefix for this action.
    pub fn toml_prefix(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Walk => "Walk",
            Self::Run => "Run",
            Self::Use => "Use",
            Self::Punch => "Punch",
            Self::Jump => "Jump",
        }
    }
}

/// Player facing direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlayerDirection {
    /// Facing down (south)
    #[default]
    Down,
    /// Facing up (north)
    Up,
    /// Facing left (west)
    Left,
    /// Facing right (east)
    Right,
}

impl PlayerDirection {
    /// Creates a PlayerDirection from velocity components.
    #[must_use]
    pub fn from_velocity(vx: f32, vy: f32) -> Self {
        let threshold = 0.1;
        if vx.abs() > vy.abs() {
            if vx > threshold {
                Self::Right
            } else if vx < -threshold {
                Self::Left
            } else {
                Self::Down
            }
        } else if vy > threshold {
            Self::Down
        } else if vy < -threshold {
            Self::Up
        } else {
            Self::Down
        }
    }

    /// Get the TOML direction suffix for this direction.
    pub fn toml_suffix(&self) -> &'static str {
        match self {
            Self::Down => "Down",
            Self::Up => "Up",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

// ============================================================================
// Animation Data (loaded from TOML)
// ============================================================================

/// A single frame in a sprite animation, with pixel coordinates in the sheet.
#[derive(Debug, Clone, Copy)]
pub struct SpriteFrame {
    /// X pixel coordinate in the sprite sheet
    pub x: u32,
    /// Y pixel coordinate in the sprite sheet
    pub y: u32,
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
}

/// A complete animation sequence (e.g., "WalkDown").
#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    /// Frames in this animation
    pub frames: Vec<SpriteFrame>,
    /// Frames per second
    pub fps: f32,
    /// Whether the animation loops
    pub looping: bool,
}

/// Animation key combining action + direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimKey {
    /// The action being performed
    pub action: PlayerAnimAction,
    /// The facing direction
    pub direction: PlayerDirection,
}

impl AnimKey {
    /// Create a new animation key.
    pub fn new(action: PlayerAnimAction, direction: PlayerDirection) -> Self {
        Self { action, direction }
    }

    /// Build the TOML action string (e.g. "WalkDown", "IdleLeft").
    pub fn toml_action_name(&self) -> String {
        format!("{}{}", self.action.toml_prefix(), self.direction.toml_suffix())
    }

    /// Parse a TOML action string into an AnimKey, if recognized.
    pub fn from_toml_action(action: &str) -> Option<Self> {
        let actions = [
            ("Idle", PlayerAnimAction::Idle),
            ("Walk", PlayerAnimAction::Walk),
            ("Run", PlayerAnimAction::Run),
            ("Use", PlayerAnimAction::Use),
            ("Punch", PlayerAnimAction::Punch),
            ("Jump", PlayerAnimAction::Jump),
        ];
        let directions = [
            ("Down", PlayerDirection::Down),
            ("Up", PlayerDirection::Up),
            ("Left", PlayerDirection::Left),
            ("Right", PlayerDirection::Right),
        ];
        for (prefix, anim_action) in &actions {
            for (suffix, dir) in &directions {
                let name = format!("{}{}", prefix, suffix);
                if action == name {
                    return Some(AnimKey::new(*anim_action, *dir));
                }
            }
        }
        None
    }
}

/// Collection of all loaded animations, keyed by action+direction.
#[derive(Debug, Clone, Default)]
pub struct PlayerAnimationSet {
    /// Map from (action, direction) to animation data
    pub animations: HashMap<AnimKey, SpriteAnimation>,
}

impl PlayerAnimationSet {
    /// Create an empty animation set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an animation.
    pub fn insert(&mut self, key: AnimKey, anim: SpriteAnimation) {
        self.animations.insert(key, anim);
    }

    /// Get an animation by key, with fallback chain:
    /// requested action → Idle same direction → IdleDown
    pub fn get(&self, key: &AnimKey) -> Option<&SpriteAnimation> {
        self.animations.get(key)
            .or_else(|| self.animations.get(&AnimKey::new(PlayerAnimAction::Idle, key.direction)))
            .or_else(|| self.animations.get(&AnimKey::new(PlayerAnimAction::Idle, PlayerDirection::Down)))
    }

    /// Check if an animation exists.
    pub fn contains(&self, key: &AnimKey) -> bool {
        self.animations.contains_key(key)
    }
}

// ============================================================================
// Sprite Config (simplified)
// ============================================================================

/// Configuration for the player sprite rendering.
#[derive(Debug, Clone, Copy)]
pub struct PlayerSpriteConfig {
    /// Render scale multiplier
    pub scale: f32,
    /// Default frame width (used for display sizing when no animation loaded)
    pub frame_width: u32,
    /// Default frame height (used for display sizing when no animation loaded)
    pub frame_height: u32,
}

impl Default for PlayerSpriteConfig {
    fn default() -> Self {
        Self {
            scale: 2.0,
            frame_width: 48,
            frame_height: 74,
        }
    }
}

// Keep the old fields available for API compatibility but unused internally
#[doc(hidden)]
impl PlayerSpriteConfig {
    /// Create a config with just scale and frame dimensions.
    pub fn with_scale(scale: f32, frame_width: u32, frame_height: u32) -> Self {
        Self {
            scale,
            frame_width,
            frame_height,
        }
    }
}

// ============================================================================
// Player Sprite State
// ============================================================================

/// Backward-compatible alias
pub type PlayerAnimState = PlayerAnimAction;

/// Player sprite state for animation.
#[derive(Debug, Clone)]
pub struct PlayerSpriteState {
    /// Current animation action
    pub action: PlayerAnimAction,
    /// Current facing direction
    pub direction: PlayerDirection,
    /// Current animation frame (0-based)
    pub frame: u32,
    /// Time accumulator for frame timing
    pub frame_time: f32,
    /// World position (x, y)
    pub position: (f32, f32),
    /// Action override from input (e.g., Use, Punch, Jump).
    /// When set, this takes priority over velocity-based action.
    /// Cleared when the animation completes one cycle.
    action_override: Option<PlayerAnimAction>,
    /// Remaining time for the action override animation
    action_override_timer: f32,
}

impl Default for PlayerSpriteState {
    fn default() -> Self {
        Self {
            action: PlayerAnimAction::Idle,
            direction: PlayerDirection::Down,
            frame: 0,
            frame_time: 0.0,
            position: (0.0, 0.0),
            action_override: None,
            action_override_timer: 0.0,
        }
    }
}

impl PlayerSpriteState {
    /// Trigger a one-shot action animation (e.g., Use, Punch, Jump).
    /// The animation plays once and then returns to velocity-based action.
    pub fn set_action_override(&mut self, action: PlayerAnimAction, animations: &PlayerAnimationSet) {
        let key = AnimKey::new(action, self.direction);
        let duration = if let Some(anim) = animations.get(&key) {
            anim.frames.len() as f32 / anim.fps
        } else {
            0.75 // fallback duration
        };
        self.action_override = Some(action);
        self.action_override_timer = duration;
        self.frame = 0;
        self.frame_time = 0.0;
        self.action = action;
    }

    /// Updates the animation state based on velocity and available animations.
    pub fn update(
        &mut self,
        dt: f32,
        velocity: (f32, f32),
        position: (f32, f32),
        animations: &PlayerAnimationSet,
    ) {
        self.position = position;

        // Determine direction from velocity (always update facing)
        let speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();

        let new_direction = if speed > 5.0 {
            PlayerDirection::from_velocity(velocity.0, velocity.1)
        } else {
            self.direction // keep facing same direction when idle
        };

        // Handle action override (one-shot animations from input)
        if let Some(override_action) = self.action_override {
            self.action_override_timer -= dt;
            if self.action_override_timer <= 0.0 {
                // Override expired, return to normal
                self.action_override = None;
                self.frame = 0;
                self.frame_time = 0.0;
            } else {
                // Still playing override — update direction if it changed
                if new_direction != self.direction {
                    self.direction = new_direction;
                    // Don't reset frame, keep playing the action
                }
                self.action = override_action;

                // Advance frame timer
                let key = AnimKey::new(self.action, self.direction);
                let (fps, frame_count) = if let Some(anim) = animations.get(&key) {
                    (anim.fps, anim.frames.len() as u32)
                } else {
                    (8.0, 6)
                };
                if frame_count > 0 {
                    self.frame_time += dt;
                    let frame_duration = 1.0 / fps;
                    if self.frame_time >= frame_duration {
                        self.frame_time -= frame_duration;
                        self.frame = (self.frame + 1) % frame_count;
                    }
                }
                return;
            }
        }

        // Normal velocity-based action selection
        let new_action = if speed > 200.0 {
            // Running threshold
            if animations.contains(&AnimKey::new(PlayerAnimAction::Run, new_direction)) {
                PlayerAnimAction::Run
            } else {
                PlayerAnimAction::Walk
            }
        } else if speed > 5.0 {
            PlayerAnimAction::Walk
        } else {
            PlayerAnimAction::Idle
        };

        // Reset frame if action or direction changed
        if new_action != self.action || new_direction != self.direction {
            self.frame = 0;
            self.frame_time = 0.0;
        }

        self.action = new_action;
        self.direction = new_direction;

        // Get fps from the current animation (or default to 8)
        let key = AnimKey::new(self.action, self.direction);
        let (fps, frame_count) = if let Some(anim) = animations.get(&key) {
            (anim.fps, anim.frames.len() as u32)
        } else {
            (8.0, 6) // fallback
        };

        // Advance frame timer
        if frame_count > 0 {
            self.frame_time += dt;
            let frame_duration = 1.0 / fps;
            if self.frame_time >= frame_duration {
                self.frame_time -= frame_duration;
                self.frame = (self.frame + 1) % frame_count;
            }
        }
    }

    /// Returns the current sprite frame from the animation set, or None.
    pub fn current_frame(&self, animations: &PlayerAnimationSet) -> Option<SpriteFrame> {
        let key = AnimKey::new(self.action, self.direction);
        animations.get(&key).and_then(|anim| {
            let idx = (self.frame as usize).min(anim.frames.len().saturating_sub(1));
            anim.frames.get(idx).copied()
        })
    }
}

/// GPU instance data for player sprite rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PlayerSpriteInstance {
    /// Position in world coordinates (x, y)
    pub position: [f32; 2],
    /// Size in world units (width, height)
    pub size: [f32; 2],
    /// UV offset in sprite sheet (u, v)
    pub uv_offset: [f32; 2],
    /// UV size for one frame
    pub uv_size: [f32; 2],
}

/// Player sprite renderer using instanced quads.
pub struct PlayerSpriteRenderer {
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group for sprite texture
    bind_group: Option<wgpu::BindGroup>,
    /// Vertex buffer for quad
    vertex_buffer: wgpu::Buffer,
    /// Index buffer for quad
    index_buffer: wgpu::Buffer,
    /// Instance buffer
    instance_buffer: wgpu::Buffer,
    /// Camera uniform buffer
    camera_buffer: wgpu::Buffer,
    /// Camera bind group
    camera_bind_group: wgpu::BindGroup,
    /// Sprite texture
    texture: Option<wgpu::Texture>,
    /// Sprite texture view
    texture_view: Option<wgpu::TextureView>,
    /// Sprite sampler
    sampler: wgpu::Sampler,
    /// Sprite sheet dimensions
    sheet_size: (u32, u32),
    /// Sprite configuration
    config: PlayerSpriteConfig,
}

/// Camera uniform for sprite rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    /// Camera position in world coordinates
    camera_pos: [f32; 2],
    /// Screen size in pixels
    screen_size: [f32; 2],
    /// Camera zoom level
    zoom: f32,
    /// Padding to align to 48 bytes (matching WGSL struct layout)
    _padding: [f32; 7],
}

impl PlayerSpriteRenderer {
    /// Creates a new player sprite renderer.
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let config = PlayerSpriteConfig::default();

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Player Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(PLAYER_SPRITE_SHADER.into()),
        });

        // Create camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Player Sprite Camera Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create texture bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Player Sprite Texture Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Player Sprite Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Player Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer (quad vertices)
                    wgpu::VertexBufferLayout {
                        array_stride: 16,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    },
                    // Instance buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<PlayerSpriteInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 3,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 16,
                                shader_location: 4,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 24,
                                shader_location: 5,
                            },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create quad vertices (position, uv)
        // Flip V coordinate (0->1, 1->0) to correct for Y-flip in clip space
        #[rustfmt::skip]
        let vertices: &[[f32; 4]] = &[
            [-0.5, -0.5, 0.0, 0.0], // Bottom-left
            [ 0.5, -0.5, 1.0, 0.0], // Bottom-right
            [ 0.5,  0.5, 1.0, 1.0], // Top-right
            [-0.5,  0.5, 0.0, 1.0], // Top-left
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Sprite Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create instance buffer (single player)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Player Sprite Instance Buffer"),
            size: std::mem::size_of::<PlayerSpriteInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Player Sprite Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Player Sprite Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create sampler (nearest neighbor for pixel art)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Player Sprite Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group: None,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            camera_buffer,
            camera_bind_group,
            texture: None,
            texture_view: None,
            sampler,
            sheet_size: (1, 1),
            config,
        }
    }

    /// Loads a sprite sheet texture from image data.
    pub fn load_sprite_sheet(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) {
        // Create texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Player Sprite Sheet"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload image data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Create texture view
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Player Sprite Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.texture = Some(texture);
        self.texture_view = Some(texture_view);
        self.bind_group = Some(bind_group);
        self.sheet_size = (width, height);
    }

    /// Updates camera parameters for rendering.
    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        camera_pos: (f32, f32),
        screen_size: (u32, u32),
        zoom: f32,
    ) {
        let uniform = CameraUniform {
            camera_pos: [camera_pos.0, camera_pos.1],
            screen_size: [screen_size.0 as f32, screen_size.1 as f32],
            zoom,
            _padding: [0.0; 7],
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Updates the player sprite instance data using per-frame UV from animation set.
    pub fn update_player(
        &self,
        queue: &wgpu::Queue,
        state: &PlayerSpriteState,
        animations: &PlayerAnimationSet,
    ) {
        // Get the current frame from the animation set
        let (uv_x, uv_y, uv_w, uv_h, frame_w, frame_h) =
            if let Some(frame) = state.current_frame(animations) {
                let sheet_w = self.sheet_size.0 as f32;
                let sheet_h = self.sheet_size.1 as f32;
                (
                    frame.x as f32 / sheet_w,
                    frame.y as f32 / sheet_h,
                    frame.width as f32 / sheet_w,
                    frame.height as f32 / sheet_h,
                    frame.width as f32,
                    frame.height as f32,
                )
            } else {
                // Fallback: show first frame area
                let fw = self.config.frame_width as f32;
                let fh = self.config.frame_height as f32;
                (
                    0.0,
                    0.0,
                    fw / self.sheet_size.0 as f32,
                    fh / self.sheet_size.1 as f32,
                    fw,
                    fh,
                )
            };

        let instance = PlayerSpriteInstance {
            position: [state.position.0, state.position.1],
            size: [
                frame_w * self.config.scale,
                frame_h * self.config.scale,
            ],
            uv_offset: [uv_x, uv_y],
            uv_size: [uv_w, uv_h],
        };

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::bytes_of(&instance));
    }

    /// Returns whether the sprite sheet is loaded.
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        self.bind_group.is_some()
    }

    /// Renders the player sprite.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if let Some(bind_group) = &self.bind_group {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    /// Sets the sprite configuration.
    pub fn set_config(&mut self, config: PlayerSpriteConfig) {
        self.config = config;
    }

    /// Returns the current configuration.
    #[must_use]
    pub fn config(&self) -> &PlayerSpriteConfig {
        &self.config
    }
}

/// WGSL shader for player sprite rendering
const PLAYER_SPRITE_SHADER: &str = r#"
struct CameraUniform {
    camera_pos: vec2<f32>,
    screen_size: vec2<f32>,
    zoom: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceInput {
    @location(2) world_pos: vec2<f32>,
    @location(3) size: vec2<f32>,
    @location(4) uv_offset: vec2<f32>,
    @location(5) uv_size: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate world position of this vertex
    let world_pos = instance.world_pos + vertex.position * instance.size;

    // Transform to screen space (camera-relative, then to clip space)
    let screen_pos = (world_pos - camera.camera_pos) * camera.zoom;

    // Convert to normalized device coordinates
    let ndc = screen_pos / (camera.screen_size * 0.5);

    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);

    // Calculate UV within the sprite frame
    out.uv = instance.uv_offset + vertex.uv * instance.uv_size;

    return out;
}

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(sprite_texture, sprite_sampler, in.uv);

    // Discard fully transparent pixels
    if color.a < 0.01 {
        discard;
    }

    return color;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_direction_from_velocity() {
        assert_eq!(PlayerDirection::from_velocity(10.0, 0.0), PlayerDirection::Right);
        assert_eq!(PlayerDirection::from_velocity(-10.0, 0.0), PlayerDirection::Left);
        assert_eq!(PlayerDirection::from_velocity(0.0, 10.0), PlayerDirection::Down);
        assert_eq!(PlayerDirection::from_velocity(0.0, -10.0), PlayerDirection::Up);
        assert_eq!(PlayerDirection::from_velocity(0.0, 0.0), PlayerDirection::Down);
    }

    #[test]
    fn test_anim_key_from_toml() {
        let key = AnimKey::from_toml_action("WalkDown").unwrap();
        assert_eq!(key.action, PlayerAnimAction::Walk);
        assert_eq!(key.direction, PlayerDirection::Down);

        let key = AnimKey::from_toml_action("IdleUp").unwrap();
        assert_eq!(key.action, PlayerAnimAction::Idle);
        assert_eq!(key.direction, PlayerDirection::Up);

        let key = AnimKey::from_toml_action("PunchLeft").unwrap();
        assert_eq!(key.action, PlayerAnimAction::Punch);
        assert_eq!(key.direction, PlayerDirection::Left);

        assert!(AnimKey::from_toml_action("UnknownAction").is_none());
    }

    #[test]
    fn test_player_sprite_state_update() {
        let mut anims = PlayerAnimationSet::new();
        // Insert a walk-right animation with 3 frames
        anims.insert(
            AnimKey::new(PlayerAnimAction::Walk, PlayerDirection::Right),
            SpriteAnimation {
                frames: vec![
                    SpriteFrame { x: 0, y: 0, width: 48, height: 74 },
                    SpriteFrame { x: 48, y: 0, width: 48, height: 74 },
                    SpriteFrame { x: 96, y: 0, width: 48, height: 74 },
                ],
                fps: 8.0,
                looping: true,
            },
        );
        anims.insert(
            AnimKey::new(PlayerAnimAction::Idle, PlayerDirection::Down),
            SpriteAnimation {
                frames: vec![SpriteFrame { x: 0, y: 100, width: 48, height: 74 }],
                fps: 8.0,
                looping: true,
            },
        );

        let mut state = PlayerSpriteState::default();

        // Moving right should update direction and action
        state.update(0.1, (100.0, 0.0), (50.0, 50.0), &anims);
        assert_eq!(state.action, PlayerAnimAction::Walk);
        assert_eq!(state.direction, PlayerDirection::Right);

        // Stopped should be idle
        state.update(0.1, (0.0, 0.0), (50.0, 50.0), &anims);
        assert_eq!(state.action, PlayerAnimAction::Idle);
        // Direction should remain the same when idle
        assert_eq!(state.direction, PlayerDirection::Right);
    }

    #[test]
    fn test_animation_set_fallback() {
        let mut anims = PlayerAnimationSet::new();
        anims.insert(
            AnimKey::new(PlayerAnimAction::Idle, PlayerDirection::Down),
            SpriteAnimation {
                frames: vec![SpriteFrame { x: 0, y: 0, width: 48, height: 74 }],
                fps: 8.0,
                looping: true,
            },
        );

        // Requesting an action that doesn't exist should fall back to Idle
        let key = AnimKey::new(PlayerAnimAction::Punch, PlayerDirection::Down);
        let anim = anims.get(&key);
        assert!(anim.is_some()); // Falls back to IdleDown
    }
}
