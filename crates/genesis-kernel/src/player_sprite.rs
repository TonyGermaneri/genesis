//! Player sprite rendering system.
//!
//! Renders the player character with animated sprites for different
//! directions and movement states (idle, walking).

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Player animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerAnimState {
    /// Standing still
    #[default]
    Idle,
    /// Walking
    Walking,
}

/// Player facing direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerDirection {
    /// Facing down (south)
    #[default]
    Down = 0,
    /// Facing left (west)
    Left = 1,
    /// Facing right (east)
    Right = 2,
    /// Facing up (north)
    Up = 3,
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

    /// Returns the row index in the sprite sheet for this direction.
    /// Based on the Modern Exteriors sprite sheet layout.
    #[must_use]
    pub fn sprite_row(&self) -> u32 {
        match self {
            // Idle animations: rows 0-3 (down, left, right, up)
            // Walk animations: rows 4-7 (down, left, right, up)
            Self::Down => 0,
            Self::Left => 1,
            Self::Right => 2,
            Self::Up => 3,
        }
    }
}

/// Configuration for the player sprite sheet.
#[derive(Debug, Clone, Copy)]
pub struct PlayerSpriteConfig {
    /// Width of each sprite frame in pixels
    pub frame_width: u32,
    /// Height of each sprite frame in pixels
    pub frame_height: u32,
    /// Number of frames per idle animation (per direction)
    pub idle_frames: u32,
    /// Number of frames per walk animation (per direction)
    pub walk_frames: u32,
    /// Row index for idle animations (pixel Y start / frame_height)
    pub idle_row: u32,
    /// Row index for walk animations (pixel Y start / frame_height)
    pub walk_row: u32,
    /// Pixel Y offset where sprite content starts
    pub row_y_offset: u32,
    /// Animation speed (frames per second)
    pub anim_fps: f32,
    /// Render scale multiplier
    pub scale: f32,
    /// Whether directions are horizontal (all in one row) or vertical (separate rows)
    pub horizontal_directions: bool,
}

impl Default for PlayerSpriteConfig {
    fn default() -> Self {
        // Configuration for Modern Exteriors Skeleton sprite sheet (simpler layout)
        // Row 1 (y=129): Walk animation with 4 directions x 6 frames = 24 frames horizontal
        // Each direction occupies 6 consecutive frames: Down(0-5), Left(6-11), Right(12-17), Up(18-23)
        Self {
            frame_width: 48,
            frame_height: 63,  // Detected from row height
            idle_frames: 6,    // Use walk frames for idle too
            walk_frames: 6,
            idle_row: 1,       // Use same row for both
            walk_row: 1,       // Row 1 (y=129 / 63 ≈ row index considering offset)
            row_y_offset: 129, // Pixel Y where walk row starts
            anim_fps: 8.0,
            scale: 2.0,        // Scale up for visibility
            horizontal_directions: true, // All 4 directions in one row
        }
    }
}

/// Player sprite state for animation.
#[derive(Debug, Clone)]
pub struct PlayerSpriteState {
    /// Current animation state
    pub anim_state: PlayerAnimState,
    /// Current facing direction
    pub direction: PlayerDirection,
    /// Current animation frame (0-based)
    pub frame: u32,
    /// Time accumulator for frame timing
    pub frame_time: f32,
    /// World position (x, y)
    pub position: (f32, f32),
}

impl Default for PlayerSpriteState {
    fn default() -> Self {
        Self {
            anim_state: PlayerAnimState::Idle,
            direction: PlayerDirection::Down,
            frame: 0,
            frame_time: 0.0,
            position: (0.0, 0.0),
        }
    }
}

impl PlayerSpriteState {
    /// Updates the animation state based on velocity.
    pub fn update(&mut self, dt: f32, velocity: (f32, f32), position: (f32, f32), config: &PlayerSpriteConfig) {
        self.position = position;

        // Determine animation state from velocity
        let speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
        let is_moving = speed > 5.0;

        if is_moving {
            self.anim_state = PlayerAnimState::Walking;
            self.direction = PlayerDirection::from_velocity(velocity.0, velocity.1);
        } else {
            self.anim_state = PlayerAnimState::Idle;
            // Keep current direction when idle
        }

        // Update animation frame
        self.frame_time += dt;
        let frame_duration = 1.0 / config.anim_fps;
        if self.frame_time >= frame_duration {
            self.frame_time -= frame_duration;
            let max_frames = match self.anim_state {
                PlayerAnimState::Idle => config.idle_frames,
                PlayerAnimState::Walking => config.walk_frames,
            };
            self.frame = (self.frame + 1) % max_frames;
        }
    }

    /// Returns the current sprite sheet row for the animation.
    #[must_use]
    pub fn current_row(&self, _config: &PlayerSpriteConfig) -> u32 {
        // For horizontal direction layouts, all directions are in the same row
        // The row is determined by animation type only
        match self.anim_state {
            PlayerAnimState::Idle => 1, // Use walk row for idle (skeleton only has walk)
            PlayerAnimState::Walking => 1,
        }
    }

    /// Returns the current sprite sheet column (frame index).
    /// For horizontal direction layouts, this includes the direction offset.
    #[must_use]
    pub fn current_column(&self, config: &PlayerSpriteConfig) -> u32 {
        if config.horizontal_directions {
            // Directions packed horizontally: Down(0-5), Left(6-11), Right(12-17), Up(18-23)
            let frames_per_dir = match self.anim_state {
                PlayerAnimState::Idle => config.idle_frames,
                PlayerAnimState::Walking => config.walk_frames,
            };
            let dir_offset = self.direction.sprite_row() * frames_per_dir;
            dir_offset + self.frame
        } else {
            // Directions in separate rows
            self.frame
        }
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

    /// Updates the player sprite instance data.
    pub fn update_player(
        &self,
        queue: &wgpu::Queue,
        state: &PlayerSpriteState,
    ) {
        let col = state.current_column(&self.config);

        // Calculate UV coordinates
        let frame_u = self.config.frame_width as f32 / self.sheet_size.0 as f32;
        let frame_v = self.config.frame_height as f32 / self.sheet_size.1 as f32;

        // Use row_y_offset for pixel-accurate Y positioning
        let uv_y = self.config.row_y_offset as f32 / self.sheet_size.1 as f32;

        let instance = PlayerSpriteInstance {
            position: [state.position.0, state.position.1],
            size: [
                self.config.frame_width as f32 * self.config.scale,
                self.config.frame_height as f32 * self.config.scale,
            ],
            uv_offset: [col as f32 * frame_u, uv_y],
            uv_size: [frame_u, frame_v],
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
    fn test_player_sprite_state_update() {
        let config = PlayerSpriteConfig::default();
        let mut state = PlayerSpriteState::default();

        // Moving right should update direction and state
        state.update(0.1, (100.0, 0.0), (50.0, 50.0), &config);
        assert_eq!(state.anim_state, PlayerAnimState::Walking);
        assert_eq!(state.direction, PlayerDirection::Right);

        // Stopped should be idle
        state.update(0.1, (0.0, 0.0), (50.0, 50.0), &config);
        assert_eq!(state.anim_state, PlayerAnimState::Idle);
        // Direction should remain the same when idle
        assert_eq!(state.direction, PlayerDirection::Right);
    }

    #[test]
    fn test_sprite_row_calculation() {
        let config = PlayerSpriteConfig::default();
        let mut state = PlayerSpriteState::default();

        state.direction = PlayerDirection::Down;
        state.anim_state = PlayerAnimState::Idle;
        assert_eq!(state.current_row(&config), 0);

        state.direction = PlayerDirection::Left;
        state.anim_state = PlayerAnimState::Walking;
        assert_eq!(state.current_row(&config), 5); // walk_row_start (4) + Left (1)
    }
}
