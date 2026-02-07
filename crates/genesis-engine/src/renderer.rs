//! GPU renderer using wgpu.
//!
//! Provides basic wgpu rendering with egui UI overlay, terrain tiles,
//! and player sprite.

#![allow(unsafe_code)]
#![allow(dead_code)]

use anyhow::{Context, Result};
use genesis_kernel::{
    Camera,
    player_sprite::{
        PlayerAnimationSet, PlayerSpriteConfig, PlayerSpriteRenderer, PlayerSpriteState,
    },
    terrain_tiles::TerrainTileRenderer,
};
use genesis_tools::EguiIntegration;
use tracing::info;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

/// Default chunk size for simulation
pub const DEFAULT_CHUNK_SIZE: u32 = 256;

/// Main renderer that manages GPU resources and rendering.
pub struct Renderer {
    /// wgpu surface for presenting to the window
    surface: wgpu::Surface<'static>,
    /// wgpu device for GPU operations
    device: wgpu::Device,
    /// wgpu queue for submitting commands
    queue: wgpu::Queue,
    /// Surface configuration
    config: wgpu::SurfaceConfiguration,
    /// Current surface size
    size: PhysicalSize<u32>,
    /// Player sprite renderer
    player_sprite_renderer: PlayerSpriteRenderer,
    /// Player sprite animation state
    player_sprite_state: PlayerSpriteState,
    /// Player animation set (loaded from TOML)
    player_animations: PlayerAnimationSet,
    /// Terrain tile renderer (biome-based world)
    terrain_renderer: TerrainTileRenderer,
    /// Egui integration for UI overlay
    egui: EguiIntegration,
    /// Frame counter
    frame_count: u64,
    /// Skip player sprite rendering (for testing)
    skip_player_sprite: bool,
    /// Whether streaming terrain is "enabled" (placeholder for compatibility)
    streaming_enabled: bool,
    /// Whether to show debug grid
    show_debug_grid: bool,
    /// Chunk size for debug grid (in world units/pixels)
    chunk_size: u32,
}

impl Renderer {
    /// Creates a new renderer for the given window.
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // Create surface
        // SAFETY: The window handle is valid for the lifetime of the surface
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)?)?
        };

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find a suitable GPU adapter")?;

        info!("Using GPU adapter: {:?}", adapter.get_info().name);

        // Request device
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Genesis Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .context("Failed to create GPU device")?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Initialize egui integration
        info!("Initializing egui integration...");
        let egui = EguiIntegration::new(&device, surface_format, window, 1);

        // Initialize player sprite renderer
        info!("Initializing player sprite renderer...");
        let player_sprite_renderer = PlayerSpriteRenderer::new(&device, surface_format);
        let player_sprite_state = PlayerSpriteState::default();
        let player_animations = PlayerAnimationSet::new();

        // Initialize terrain tile renderer
        info!("Initializing terrain tile renderer...");
        let terrain_renderer = TerrainTileRenderer::new(&device, surface_format);

        info!("Renderer initialized successfully");

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            player_sprite_renderer,
            player_sprite_state,
            player_animations,
            terrain_renderer,
            egui,
            frame_count: 0,
            skip_player_sprite: false,
            streaming_enabled: false,
            show_debug_grid: true,  // Enable by default
            chunk_size: DEFAULT_CHUNK_SIZE,
        })
    }

    /// Enables streaming terrain mode (placeholder - terrain removed).
    pub fn enable_streaming_terrain(&mut self, _seed: u64) {
        info!("Streaming terrain requested (terrain system removed)");
        self.streaming_enabled = true;
    }

    /// Returns whether streaming terrain is enabled.
    #[must_use]
    pub const fn is_streaming_terrain_enabled(&self) -> bool {
        self.streaming_enabled
    }

    /// Updates streaming terrain based on player position (placeholder).
    pub fn update_player_position_streaming(&mut self, _player_x: f32, _player_y: f32) {
        // Terrain streaming removed
    }

    /// Steps the streaming terrain simulation (placeholder).
    pub fn step_streaming_terrain(&mut self, _delta_time: f32) {
        // Terrain simulation removed
    }

    /// Regenerates the terrain (placeholder).
    pub fn regenerate_terrain(&mut self, _new_seed: u64) {
        info!("Terrain regeneration requested (terrain system removed)");
    }

    /// Set debug visualization flags (placeholder).
    pub fn set_debug_flags(&mut self, _flags: u32) {
        // Debug flags removed with terrain
    }

    /// Returns streaming terrain statistics (placeholder).
    #[must_use]
    pub fn streaming_stats(&self) -> Option<StreamingStats> {
        None
    }

    /// Enable autotile terrain (placeholder).
    pub fn enable_autotile_terrain<T>(
        &mut self,
        _atlas: std::sync::Arc<parking_lot::RwLock<T>>,
    ) {
        info!("Autotile terrain requested (terrain system removed)");
    }

    /// Enable textured terrain (placeholder).
    pub fn enable_textured_terrain<T>(
        &mut self,
        _atlas: std::sync::Arc<parking_lot::RwLock<T>>,
    ) {
        info!("Textured terrain requested (terrain system removed)");
    }

    /// Loads the player sprite sheet from image data.
    pub fn load_player_sprite(&mut self, image_data: &[u8], width: u32, height: u32) {
        self.player_sprite_renderer.load_sprite_sheet(
            &self.device,
            &self.queue,
            image_data,
            width,
            height,
        );
        info!("Loaded player sprite sheet: {}x{}", width, height);
    }

    /// Sets the player sprite configuration.
    pub fn set_player_sprite_config(&mut self, config: PlayerSpriteConfig) {
        self.player_sprite_renderer.set_config(config);
    }

    /// Updates the player sprite state with explicit delta time.
    pub fn update_player_sprite(&mut self, dt: f32, position: (f32, f32), velocity: (f32, f32)) {
        self.player_sprite_state
            .update(dt, velocity, position, &self.player_animations);
    }

    /// Triggers a one-shot animation action (e.g., Use, Punch, Jump).
    pub fn set_player_action(&mut self, action: genesis_kernel::player_sprite::PlayerAnimAction) {
        self.player_sprite_state
            .set_action_override(action, &self.player_animations);
    }

    /// Sets the player animation set (parsed from TOML).
    pub fn set_player_animations(&mut self, animations: PlayerAnimationSet) {
        self.player_animations = animations;
    }

    /// Sets whether to skip player sprite rendering (for testing).
    pub fn set_skip_player_sprite(&mut self, skip: bool) {
        self.skip_player_sprite = skip;
        if skip {
            info!("Player sprite rendering disabled");
        }
    }

    /// Returns a mutable reference to the terrain tile renderer.
    pub fn terrain_renderer_mut(&mut self) -> &mut TerrainTileRenderer {
        &mut self.terrain_renderer
    }

    /// Returns a reference to the terrain tile renderer.
    pub fn terrain_renderer(&self) -> &TerrainTileRenderer {
        &self.terrain_renderer
    }

    /// Update terrain visible tiles for the current camera position.
    /// This method handles borrowing internally to avoid borrow-checker issues.
    pub fn update_terrain_visible_tiles(&mut self, camera_pos: (f32, f32), zoom: f32) {
        if self.terrain_renderer.is_enabled() {
            let size = (self.size.width, self.size.height);
            self.terrain_renderer.update_visible_tiles(&self.queue, camera_pos, zoom, size);
        }
    }

    /// Returns whether multi-chunk rendering is enabled (always false now).
    #[must_use]
    pub const fn is_multi_chunk_enabled(&self) -> bool {
        false
    }

    /// Resizes the renderer to match the new window size.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Updates the scale factor for egui rendering.
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.egui.set_pixels_per_point(scale_factor);
    }

    /// Updates the chunk manager camera position (placeholder).
    pub fn update_camera_position(&mut self, _camera: &Camera) {
        // Chunk manager removed
    }

    /// Prepares chunks for simulation (placeholder).
    pub fn prepare_multi_chunk_simulation(&mut self) {
        // Multi-chunk removed
    }

    /// Steps simulation for all active chunks (placeholder).
    pub fn step_multi_chunk_simulation(&mut self) {
        // Multi-chunk removed
    }

    /// Handle a window event for egui.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui.handle_event(window, event)
    }

    /// Returns the number of visible/active chunks (always 0 now).
    #[must_use]
    pub fn visible_chunk_count(&self) -> usize {
        0
    }

    /// Returns the total number of cells being simulated (always 0 now).
    #[must_use]
    pub fn total_cell_count(&self) -> u64 {
        0
    }

    /// Returns a reference to the GPU device.
    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns a reference to the GPU queue.
    #[must_use]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Returns the current surface size.
    #[must_use]
    pub const fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    /// Returns the surface format.
    #[must_use]
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Returns the current frame count.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Toggles the debug grid visibility.
    pub fn toggle_debug_grid(&mut self) {
        self.show_debug_grid = !self.show_debug_grid;
        info!("Debug grid: {}", if self.show_debug_grid { "ON" } else { "OFF" });
    }

    /// Sets the debug grid visibility.
    pub fn set_debug_grid(&mut self, enabled: bool) {
        self.show_debug_grid = enabled;
    }

    /// Returns whether the debug grid is visible.
    #[must_use]
    pub const fn is_debug_grid_visible(&self) -> bool {
        self.show_debug_grid
    }

    /// Draws the debug chunk grid using egui.
    fn draw_debug_grid(&self, camera: &Camera, scale_factor: f32) {
        let ctx = self.egui.context();
        let painter = ctx.layer_painter(egui::LayerId::background());

        // Screen dimensions in logical pixels
        let screen_width = self.size.width as f32 / scale_factor;
        let screen_height = self.size.height as f32 / scale_factor;

        // Camera position and zoom
        let (cam_x, cam_y) = camera.position;
        let zoom = camera.zoom;
        let chunk_size = self.chunk_size as f32;

        // World bounds visible on screen
        let half_width = screen_width / (2.0 * zoom);
        let half_height = screen_height / (2.0 * zoom);
        let world_left = cam_x - half_width;
        let world_right = cam_x + half_width;
        let world_top = cam_y - half_height;
        let world_bottom = cam_y + half_height;

        // Find chunk boundaries to draw
        let chunk_x_start = (world_left / chunk_size).floor() as i32;
        let chunk_x_end = (world_right / chunk_size).ceil() as i32;
        let chunk_y_start = (world_top / chunk_size).floor() as i32;
        let chunk_y_end = (world_bottom / chunk_size).ceil() as i32;

        // Grid line style
        let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 128));
        let chunk_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(255, 200, 0, 200));

        // Helper to convert world coords to screen coords
        let world_to_screen = |wx: f32, wy: f32| -> egui::Pos2 {
            let sx = (wx - cam_x) * zoom + screen_width / 2.0;
            let sy = (wy - cam_y) * zoom + screen_height / 2.0;
            egui::pos2(sx, sy)
        };

        // Draw vertical chunk lines
        for chunk_x in chunk_x_start..=chunk_x_end {
            let world_x = chunk_x as f32 * chunk_size;
            let top = world_to_screen(world_x, world_top);
            let bottom = world_to_screen(world_x, world_bottom);
            painter.line_segment([top, bottom], chunk_stroke);

            // Draw chunk coordinate labels for each cell in this column
            for chunk_y in chunk_y_start..=chunk_y_end {
                let label_pos = world_to_screen(world_x + chunk_size / 2.0, chunk_y as f32 * chunk_size + 20.0);
                let label = format!("({}, {})", chunk_x, chunk_y);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_TOP,
                    label,
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgba_unmultiplied(255, 200, 0, 180),
                );
            }
        }

        // Draw horizontal chunk lines
        for chunk_y in chunk_y_start..=chunk_y_end {
            let world_y = chunk_y as f32 * chunk_size;
            let left = world_to_screen(world_left, world_y);
            let right = world_to_screen(world_right, world_y);
            painter.line_segment([left, right], chunk_stroke);
        }

        // Draw finer grid lines within chunks (every 64 pixels)
        let sub_grid_size = 64.0;
        let sub_x_start = (world_left / sub_grid_size).floor() as i32;
        let sub_x_end = (world_right / sub_grid_size).ceil() as i32;
        let sub_y_start = (world_top / sub_grid_size).floor() as i32;
        let sub_y_end = (world_bottom / sub_grid_size).ceil() as i32;

        for sx in sub_x_start..=sub_x_end {
            let world_x = sx as f32 * sub_grid_size;
            // Skip chunk boundaries (already drawn thicker)
            if (world_x % chunk_size).abs() < 0.1 {
                continue;
            }
            let top = world_to_screen(world_x, world_top);
            let bottom = world_to_screen(world_x, world_bottom);
            painter.line_segment([top, bottom], grid_stroke);
        }

        for sy in sub_y_start..=sub_y_end {
            let world_y = sy as f32 * sub_grid_size;
            // Skip chunk boundaries (already drawn thicker)
            if (world_y % chunk_size).abs() < 0.1 {
                continue;
            }
            let left = world_to_screen(world_left, world_y);
            let right = world_to_screen(world_right, world_y);
            painter.line_segment([left, right], grid_stroke);
        }

        // Draw origin crosshair
        let origin = world_to_screen(0.0, 0.0);
        let origin_stroke = egui::Stroke::new(2.0, egui::Color32::RED);
        painter.line_segment([egui::pos2(origin.x - 20.0, origin.y), egui::pos2(origin.x + 20.0, origin.y)], origin_stroke);
        painter.line_segment([egui::pos2(origin.x, origin.y - 20.0), egui::pos2(origin.x, origin.y + 20.0)], origin_stroke);
    }

    /// Renders a frame with game state and egui UI overlay.
    pub fn render_with_ui<F>(&mut self, window: &Window, camera: &Camera, time_of_day: f32, sun_intensity: f32, ui_fn: F) -> Result<()>
    where
        F: FnOnce(&egui::Context),
    {
        // Begin egui frame
        self.egui.begin_frame(window);

        // Draw debug grid if enabled (using egui painter for simplicity)
        if self.show_debug_grid {
            self.draw_debug_grid(camera, window.scale_factor() as f32);
        }

        // Build UI via callback
        ui_fn(self.egui.context());

        // End egui frame
        let egui_output = self.egui.end_frame(window);

        // Get surface texture
        let output = self
            .surface
            .get_current_texture()
            .context("Failed to get surface texture")?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Prepare egui rendering
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        let paint_jobs = self.egui.prepare(
            &self.device,
            &self.queue,
            &mut encoder,
            &screen_descriptor,
            egui_output,
        );

        // Clear the screen with a sky blue color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.7,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render terrain tiles (background layer, before player)
        if self.terrain_renderer.is_enabled() && self.terrain_renderer.instance_count() > 0 {
            self.terrain_renderer.update_camera(
                &self.queue,
                camera.position,
                (self.size.width, self.size.height),
                camera.zoom,
                time_of_day,
                sun_intensity,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Terrain Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve clear color
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.terrain_renderer.render(&mut render_pass);
        }

        // Render player sprite (between terrain and UI) - skip if disabled
        if self.player_sprite_renderer.is_loaded() && !self.skip_player_sprite {
            // Update camera and player state for the sprite renderer
            self.player_sprite_renderer.update_camera(
                &self.queue,
                camera.position,
                (self.size.width, self.size.height),
                camera.zoom,
            );
            self.player_sprite_renderer.update_player(
                &self.queue,
                &self.player_sprite_state,
                &self.player_animations,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Player Sprite Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve clear color
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.player_sprite_renderer.render(&mut render_pass);
        }

        // Render egui UI overlay (on top of world)
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve world rendering
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // SAFETY: We drop the render pass before using the encoder again
            self.egui.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.frame_count += 1;

        Ok(())
    }

    /// Captures a screenshot and saves it to the specified path.
    pub fn capture_screenshot<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        camera: &Camera,
        _window: &Window,
        time_of_day: f32,
        sun_intensity: f32,
    ) -> Result<std::path::PathBuf> {
        use std::io::Write;

        info!("Capturing screenshot to {:?}", path.as_ref());

        let width = self.size.width;
        let height = self.size.height;

        // Create a texture to render to
        let capture_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Screenshot Capture Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let capture_view = capture_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Calculate buffer size with proper alignment
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;
        let buffer_size = (padded_bytes_per_row * height) as u64;

        // Create buffer for reading back
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screenshot Encoder"),
        });

        // Clear with sky color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screenshot Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &capture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.7,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render terrain tiles for screenshot
        if self.terrain_renderer.is_enabled() && self.terrain_renderer.instance_count() > 0 {
            self.terrain_renderer.update_camera(
                &self.queue,
                camera.position,
                (self.size.width, self.size.height),
                camera.zoom,
                time_of_day,
                sun_intensity,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screenshot Terrain Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &capture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.terrain_renderer.render(&mut render_pass);
        }

        // Render player sprite if loaded
        if self.player_sprite_renderer.is_loaded() && !self.skip_player_sprite {
            self.player_sprite_renderer.update_camera(
                &self.queue,
                camera.position,
                (self.size.width, self.size.height),
                camera.zoom,
            );
            self.player_sprite_renderer.update_player(
                &self.queue,
                &self.player_sprite_state,
                &self.player_animations,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screenshot Player Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &capture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.player_sprite_renderer.render(&mut render_pass);
        }

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &capture_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        self.device.poll(wgpu::Maintain::Wait);

        rx.recv()
            .context("Failed to receive map result")?
            .context("Failed to map buffer")?;

        let data = buffer_slice.get_mapped_range();

        // Remove padding and collect actual image data
        let is_bgra = matches!(
            self.config.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );

        let mut image_data = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..height {
            let row_start = (row * padded_bytes_per_row) as usize;
            let row_end = row_start + (width * bytes_per_pixel) as usize;
            let row_data = &data[row_start..row_end];

            if is_bgra {
                // Convert BGRA to RGBA
                for pixel in row_data.chunks(4) {
                    image_data.push(pixel[2]); // R (was B)
                    image_data.push(pixel[1]); // G
                    image_data.push(pixel[0]); // B (was R)
                    image_data.push(pixel[3]); // A
                }
            } else {
                image_data.extend_from_slice(row_data);
            }
        }

        drop(data);
        output_buffer.unmap();

        // Encode as PNG
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().context("Failed to write PNG header")?;
            writer.write_image_data(&image_data).context("Failed to write PNG data")?;
        }

        // Save to file
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create screenshot directory")?;
        }
        let mut file = std::fs::File::create(path).context("Failed to create screenshot file")?;
        file.write_all(&png_data).context("Failed to write screenshot data")?;

        info!("Screenshot saved: {:?} ({}x{})", path, width, height);

        Ok(path.to_path_buf())
    }
}

/// Placeholder streaming stats (terrain removed).
pub struct StreamingStats {
    /// Number of simulating chunks
    pub simulating_count: usize,
}
