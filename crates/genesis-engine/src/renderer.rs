//! GPU renderer using wgpu.
//!
//! Integrates the cell simulation compute pipeline with visual rendering.
//! Supports egui UI overlay for debug HUD and game menus.
//! Supports both single-chunk and multi-chunk rendering modes.

#![allow(unsafe_code)]
#![allow(dead_code)]

use anyhow::{Context, Result};
use genesis_gameplay::GameState;
use genesis_kernel::{
    Camera, CellBuffer, CellComputePipeline, CellRenderPipeline, ChunkManager,
    ChunkRenderManager, StreamingConfig, StreamingTerrain, TexturedChunkRenderer,
    autotile_atlas::AutotileAtlas, autotile_render::AutotileChunkRenderer,
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
    /// Cell compute pipeline
    compute_pipeline: CellComputePipeline,
    /// Cell render pipeline
    render_pipeline: CellRenderPipeline,
    /// Cell buffer (double-buffered) - for single-chunk mode
    cell_buffer: CellBuffer,
    /// Chunk manager for multi-chunk mode (optional)
    chunk_manager: Option<ChunkManager>,
    /// Streaming terrain system (player-centered chunk loading)
    streaming_terrain: Option<StreamingTerrain>,
    /// Chunk render manager for multi-chunk rendering
    chunk_render_manager: Option<ChunkRenderManager>,
    /// Textured chunk renderer (uses terrain atlas textures - singles)
    textured_renderer: Option<TexturedChunkRenderer>,
    /// Autotile chunk renderer (uses autotile atlas)
    autotile_renderer: Option<AutotileChunkRenderer>,
    /// Render bind group for current buffer
    render_bind_group: wgpu::BindGroup,
    /// Egui integration for UI overlay
    egui: EguiIntegration,
    /// Whether simulation is running
    simulation_running: bool,
    /// Frame counter
    frame_count: u64,
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

        // Request device with compute features
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

        // Create compute pipeline
        info!("Initializing compute pipeline...");
        let compute_pipeline = CellComputePipeline::new(&device);

        // Create render pipeline
        info!("Initializing render pipeline...");
        let render_pipeline = CellRenderPipeline::new(&device, surface_format);

        // Create cell buffer with static terrain (no gravity simulation)
        let chunk_size = DEFAULT_CHUNK_SIZE;
        let initial_cells = create_static_terrain(chunk_size as usize);
        let cell_buffer =
            CellBuffer::with_cells(&device, &compute_pipeline, chunk_size, &initial_cells);

        // Create render bind group
        let render_bind_group =
            render_pipeline.create_bind_group(&device, cell_buffer.current_buffer());

        // Initialize egui integration
        info!("Initializing egui integration...");
        let egui = EguiIntegration::new(&device, surface_format, window, 1);

        info!("Renderer initialized successfully");

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            compute_pipeline,
            render_pipeline,
            cell_buffer,
            chunk_manager: None, // Single-chunk mode by default
            streaming_terrain: None, // Streaming terrain disabled by default
            chunk_render_manager: None, // Created when streaming terrain is enabled
            textured_renderer: None, // Created when terrain textures are loaded (singles)
            autotile_renderer: None, // Created when autotile atlas is loaded
            render_bind_group,
            egui,
            simulation_running: false, // Disable simulation (no cell gravity)
            frame_count: 0,
        })
    }

    /// Enables streaming terrain mode with player-centered chunk loading.
    ///
    /// When enabled, chunks are loaded/unloaded based on player position,
    /// and only nearby chunks are simulated on the GPU.
    pub fn enable_streaming_terrain(&mut self, seed: u64) {
        if self.streaming_terrain.is_none() {
            info!("Enabling streaming terrain mode with seed {}", seed);
            self.streaming_terrain = Some(StreamingTerrain::with_seed(seed));
            // Create chunk render manager for multi-chunk rendering
            self.chunk_render_manager = Some(ChunkRenderManager::new(
                &self.device,
                self.config.format,
                DEFAULT_CHUNK_SIZE,
            ));
        }
    }

    /// Enables streaming terrain with custom configuration.
    pub fn enable_streaming_terrain_with_config(&mut self, seed: u64, config: StreamingConfig) {
        if self.streaming_terrain.is_none() {
            info!("Enabling streaming terrain mode with custom config");
            let chunk_size = config.chunk_size;
            self.streaming_terrain = Some(StreamingTerrain::new(seed, config));
            // Create chunk render manager for multi-chunk rendering
            self.chunk_render_manager = Some(ChunkRenderManager::new(
                &self.device,
                self.config.format,
                chunk_size,
            ));
        }
    }

    /// Disables streaming terrain mode.
    pub fn disable_streaming_terrain(&mut self) {
        if self.streaming_terrain.is_some() {
            info!("Disabling streaming terrain mode");
            self.streaming_terrain = None;
            self.chunk_render_manager = None;
        }
    }

    /// Regenerates the terrain with a new seed.
    ///
    /// This clears all loaded chunks and recreates the streaming terrain
    /// with the new seed, forcing full regeneration.
    pub fn regenerate_terrain(&mut self, new_seed: u64) {
        info!("Regenerating terrain with new seed: {}", new_seed);
        
        // Get the current config if streaming is enabled
        let config = self.streaming_terrain
            .as_ref()
            .map(|t| t.config().clone())
            .unwrap_or_default();
        
        let chunk_size = config.chunk_size;
        
        // Clear autotile renderer if present
        if let Some(autotile) = &mut self.autotile_renderer {
            autotile.clear();
        }
        
        // Clear chunk render manager by recreating it
        if self.chunk_render_manager.is_some() {
            self.chunk_render_manager = Some(ChunkRenderManager::new(
                &self.device,
                self.config.format,
                chunk_size,
            ));
        }
        
        // Recreate streaming terrain with new seed
        if self.streaming_terrain.is_some() {
            self.streaming_terrain = Some(StreamingTerrain::new(new_seed, config));
            info!("Streaming terrain recreated with seed {}", new_seed);
        }
    }

    /// Returns whether streaming terrain is enabled.
    #[must_use]
    pub const fn is_streaming_terrain_enabled(&self) -> bool {
        self.streaming_terrain.is_some()
    }

    /// Updates streaming terrain based on player position.
    ///
    /// Call this each frame with the player's world position.
    pub fn update_player_position_streaming(&mut self, player_x: f32, player_y: f32) {
        // First update terrain position (this may load/unload chunks)
        if let Some(terrain) = &mut self.streaming_terrain {
            terrain.update_player_position(player_x, player_y, &self.device);
        }

        // Then sync to render manager (separate borrow)
        self.sync_streaming_chunks_to_render();
    }

    /// Syncs streaming terrain chunks to the render manager.
    fn sync_streaming_chunks_to_render(&mut self) {
        use genesis_kernel::VisibleChunk;

        let Some(terrain) = &self.streaming_terrain else { return };

        // Check if we have any renderers to update
        let has_chunk_mgr = self.chunk_render_manager.is_some();
        let has_textured = self.textured_renderer.is_some();
        let has_autotile = self.autotile_renderer.is_some();

        if !has_chunk_mgr && !has_textured && !has_autotile {
            return;
        }

        // Upload loaded chunks to GPU render managers
        for chunk in terrain.loaded_chunks() {
            // Create a VisibleChunk for the render manager
            let visible_chunk = VisibleChunk {
                position: (chunk.id.x, chunk.id.y),
                cells: chunk.cells.clone(),
                dirty: true, // Always mark dirty to ensure upload
            };

            // Upload to autotile renderer if available (preferred)
            if let Some(autotile_mgr) = &mut self.autotile_renderer {
                autotile_mgr.update_chunk(
                    &self.device,
                    &self.queue,
                    chunk.id.x,
                    chunk.id.y,
                    bytemuck::cast_slice(&visible_chunk.cells),
                );
            }

            // Upload to textured renderer if available
            if let Some(textured_mgr) = &mut self.textured_renderer {
                textured_mgr.upload_chunk(&self.device, &self.queue, &visible_chunk);
            }

            // Also upload to standard chunk render manager
            if let Some(render_mgr) = &mut self.chunk_render_manager {
                render_mgr.upload_chunk(&self.device, &self.queue, &visible_chunk);
            }
        }
    }

    /// Steps the streaming terrain simulation.
    pub fn step_streaming_terrain(&mut self) {
        if let Some(terrain) = &mut self.streaming_terrain {
            terrain.step_simulation(&self.device, &self.queue, &self.compute_pipeline);
        }
    }

    /// Returns a reference to the streaming terrain.
    #[must_use]
    pub fn streaming_terrain(&self) -> Option<&StreamingTerrain> {
        self.streaming_terrain.as_ref()
    }

    /// Returns streaming terrain statistics.
    #[must_use]
    pub fn streaming_stats(&self) -> Option<genesis_kernel::StreamingStats> {
        self.streaming_terrain.as_ref().map(|t| t.stats().clone())
    }

    /// Enables textured terrain rendering with the given atlas.
    ///
    /// When enabled, terrain will be rendered using textures from the atlas
    /// instead of procedural colors.
    pub fn enable_textured_terrain(
        &mut self,
        atlas: std::sync::Arc<parking_lot::RwLock<genesis_kernel::TerrainTextureAtlas>>,
    ) {
        info!("Enabling textured terrain rendering (singles)");

        // Create textured renderer if not exists
        if self.textured_renderer.is_none() {
            self.textured_renderer = Some(TexturedChunkRenderer::new(
                &self.device,
                self.config.format,
                DEFAULT_CHUNK_SIZE,
            ));
        }

        // Bind the atlas to the renderer
        if let Some(renderer) = &mut self.textured_renderer {
            renderer.set_terrain_atlas(&self.queue, atlas);
        }
    }

    /// Enables autotile-based terrain rendering with the given atlas.
    ///
    /// When enabled, terrain will be rendered using the autotile atlas
    /// with proper biome-to-terrain mapping.
    pub fn enable_autotile_terrain(
        &mut self,
        atlas: std::sync::Arc<parking_lot::RwLock<AutotileAtlas>>,
    ) {
        info!("Enabling autotile terrain rendering");

        // Create autotile renderer if not exists
        if self.autotile_renderer.is_none() {
            self.autotile_renderer = Some(AutotileChunkRenderer::new(
                &self.device,
                DEFAULT_CHUNK_SIZE,
            ));
        }

        // Bind the atlas to the renderer
        if let Some(renderer) = &mut self.autotile_renderer {
            renderer.bind_atlas(atlas, &self.queue);
        }
    }

    /// Disables textured terrain rendering.
    pub fn disable_textured_terrain(&mut self) {
        info!("Disabling textured terrain rendering");
        self.textured_renderer = None;
        self.autotile_renderer = None;
    }

    /// Returns whether textured terrain rendering is enabled.
    #[must_use]
    pub fn is_textured_terrain_enabled(&self) -> bool {
        self.textured_renderer.as_ref().map_or(false, |r| r.has_terrain_atlas())
            || self.autotile_renderer.as_ref().map_or(false, |r| r.is_atlas_bound())
    }

    /// Returns whether autotile terrain rendering is enabled.
    #[must_use]
    pub fn is_autotile_terrain_enabled(&self) -> bool {
        self.autotile_renderer.as_ref().map_or(false, |r| r.is_atlas_bound())
    }

    /// Enables multi-chunk rendering mode.
    ///
    /// When enabled, the renderer will use the ChunkManager for multi-chunk
    /// streaming and rendering instead of the single CellBuffer.
    pub fn enable_multi_chunk(&mut self) {
        if self.chunk_manager.is_none() {
            info!("Enabling multi-chunk rendering mode");
            self.chunk_manager = Some(ChunkManager::new(DEFAULT_CHUNK_SIZE));
        }
    }

    /// Disables multi-chunk rendering mode.
    pub fn disable_multi_chunk(&mut self) {
        if self.chunk_manager.is_some() {
            info!("Disabling multi-chunk rendering mode");
            self.chunk_manager = None;
        }
    }

    /// Returns whether multi-chunk rendering is enabled.
    #[must_use]
    pub const fn is_multi_chunk_enabled(&self) -> bool {
        self.chunk_manager.is_some()
    }

    /// Returns the chunk manager if multi-chunk mode is enabled.
    #[must_use]
    pub fn chunk_manager(&self) -> Option<&ChunkManager> {
        self.chunk_manager.as_ref()
    }

    /// Returns a mutable reference to the chunk manager.
    pub fn chunk_manager_mut(&mut self) -> Option<&mut ChunkManager> {
        self.chunk_manager.as_mut()
    }

    /// Resizes the renderer to match the new window size.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update render params for new size
            self.render_pipeline
                .set_screen_size(&self.queue, new_size.width, new_size.height);
        }
    }

    /// Updates the scale factor for egui rendering.
    /// Call this when the window's scale factor changes.
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.egui.set_pixels_per_point(scale_factor);
    }

    /// Renders a frame, running simulation if enabled.
    pub fn render(&mut self) -> Result<()> {
        // Run simulation step if enabled
        if self.simulation_running {
            self.step_simulation();
        }

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

        // Render cells
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cell Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.render_pipeline
                .render(&mut render_pass, &self.render_bind_group);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.frame_count += 1;

        Ok(())
    }

    /// Renders a frame with game state, camera, and optional debug overlay.
    ///
    /// This is the main render entry point that integrates:
    /// - Cell simulation
    /// - Camera-relative view
    /// - Player visualization
    /// - Debug overlay (when enabled)
    pub fn render_with_state(
        &mut self,
        camera: &Camera,
        _gameplay: &GameState,
        show_debug: bool,
        fps: f32,
        frame_time: f32,
        _hotbar_slot: u8,
    ) -> Result<()> {
        // Run simulation step if enabled
        if self.simulation_running {
            self.step_simulation();
        }

        // Update render pipeline with camera position
        self.render_pipeline.set_camera(
            &self.queue,
            camera.position.0 as i32,
            camera.position.1 as i32,
        );
        self.render_pipeline.set_zoom(&self.queue, camera.zoom);

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

        // Render cells with camera offset
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cell Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.15,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.render_pipeline
                .render(&mut render_pass, &self.render_bind_group);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // Log debug info (simplified - real HUD would use egui)
        if show_debug && self.frame_count % 60 == 0 {
            info!(
                "FPS: {:.1}, Frame: {:.2}ms, Camera: ({:.0}, {:.0}), Zoom: {:.1}x",
                fps, frame_time, camera.position.0, camera.position.1, camera.zoom
            );
        }

        output.present();

        self.frame_count += 1;

        Ok(())
    }

    /// Renders a frame with game state and egui UI overlay.
    ///
    /// This method combines world rendering with egui UI. The UI is rendered
    /// on top of the game world. Use the `ui_fn` callback to build your UI.
    ///
    /// # Arguments
    /// * `window` - The winit window (for egui input handling)
    /// * `camera` - Camera for world-to-screen transform
    /// * `ui_fn` - Callback that builds the egui UI
    pub fn render_with_ui<F>(&mut self, window: &Window, camera: &Camera, ui_fn: F) -> Result<()>
    where
        F: FnOnce(&egui::Context),
    {
        // Run simulation step if enabled
        if self.simulation_running {
            self.step_simulation();
        }

        // Update render pipeline with camera position
        self.render_pipeline.set_camera(
            &self.queue,
            camera.position.0 as i32,
            camera.position.1 as i32,
        );
        self.render_pipeline.set_zoom(&self.queue, camera.zoom);

        // Also update chunk render manager camera if using streaming terrain
        if let Some(render_mgr) = &mut self.chunk_render_manager {
            render_mgr.update_camera(&self.queue, camera);
        }

        // Update textured renderer camera if enabled
        if let Some(textured_mgr) = &mut self.textured_renderer {
            textured_mgr.update_camera(&self.queue, camera);
        }

        // Update autotile renderer camera if enabled
        if let Some(autotile_mgr) = &mut self.autotile_renderer {
            autotile_mgr.update_params(&self.queue, camera, 0.5); // TODO: pass actual time of day
        }

        // Begin egui frame
        self.egui.begin_frame(window);

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

        // Render world (cells/terrain)
        {
            // Collect visible chunks for autotile renderer
            let visible_chunks: Vec<(i32, i32)> = if let Some(terrain) = &self.streaming_terrain {
                terrain.loaded_chunks().map(|c| (c.id.x, c.id.y)).collect()
            } else {
                vec![]
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cell Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.15,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Priority: autotile renderer > textured renderer > chunk render manager > single-chunk mode
            if let Some(autotile_mgr) = &self.autotile_renderer {
                if autotile_mgr.is_atlas_bound() && !visible_chunks.is_empty() {
                    autotile_mgr.render(&mut render_pass, &visible_chunks);
                } else if let Some(render_mgr) = &self.chunk_render_manager {
                    render_mgr.render(&mut render_pass);
                } else {
                    self.render_pipeline
                        .render(&mut render_pass, &self.render_bind_group);
                }
            } else if let Some(textured_mgr) = &self.textured_renderer {
                if textured_mgr.has_terrain_atlas() {
                    textured_mgr.render(&mut render_pass);
                } else if let Some(render_mgr) = &self.chunk_render_manager {
                    render_mgr.render(&mut render_pass);
                } else {
                    self.render_pipeline
                        .render(&mut render_pass, &self.render_bind_group);
                }
            } else if let Some(render_mgr) = &self.chunk_render_manager {
                render_mgr.render(&mut render_pass);
            } else {
                self.render_pipeline
                    .render(&mut render_pass, &self.render_bind_group);
            }
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

    /// Handle a window event for egui.
    ///
    /// Call this before processing events in the game loop.
    /// Returns `true` if egui consumed the event (game should ignore it).
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui.handle_event(window, event)
    }

    /// Runs one simulation step on the GPU.
    fn step_simulation(&mut self) {
        // CellBuffer::step() handles params update, dispatch, and buffer swap internally
        self.cell_buffer
            .step(&self.device, &self.queue, &self.compute_pipeline);

        // Update render bind group to point to new current buffer
        self.render_bind_group = self
            .render_pipeline
            .create_bind_group(&self.device, self.cell_buffer.current_buffer());
    }

    /// Toggles simulation on/off.
    pub fn toggle_simulation(&mut self) {
        self.simulation_running = !self.simulation_running;
        info!(
            "Simulation {}",
            if self.simulation_running {
                "running"
            } else {
                "paused"
            }
        );
    }

    /// Returns whether simulation is running.
    #[must_use]
    pub const fn is_simulation_running(&self) -> bool {
        self.simulation_running
    }

    /// Returns the current frame count.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
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

    /// Returns a reference to the cell buffer.
    #[must_use]
    pub fn cell_buffer(&self) -> &CellBuffer {
        &self.cell_buffer
    }

    /// Returns a mutable reference to the cell buffer.
    pub fn cell_buffer_mut(&mut self) -> &mut CellBuffer {
        &mut self.cell_buffer
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

    /// Returns the compute pipeline.
    #[must_use]
    pub fn compute_pipeline(&self) -> &CellComputePipeline {
        &self.compute_pipeline
    }

    /// Returns the number of visible/active chunks.
    ///
    /// In single-chunk mode, this always returns 1.
    /// In multi-chunk mode, this returns the number of active chunks from the ChunkManager.
    #[must_use]
    pub fn visible_chunk_count(&self) -> usize {
        match &self.chunk_manager {
            Some(cm) => cm.active_chunk_count(),
            None => 1, // Single chunk mode
        }
    }

    /// Returns the total number of cells being simulated.
    #[must_use]
    pub fn total_cell_count(&self) -> u64 {
        let cells_per_chunk = DEFAULT_CHUNK_SIZE as u64 * DEFAULT_CHUNK_SIZE as u64;
        match &self.chunk_manager {
            Some(cm) => cm.active_chunk_count() as u64 * cells_per_chunk,
            None => cells_per_chunk, // Single chunk
        }
    }

    /// Updates the chunk manager camera position for multi-chunk mode.
    ///
    /// Call this when the camera moves to ensure correct chunks are loaded.
    pub fn update_camera_position(&mut self, camera: &Camera) {
        if let Some(cm) = &mut self.chunk_manager {
            cm.update_camera(camera.position.0 as i32, camera.position.1 as i32);
        }
    }

    /// Prepares chunks for simulation in multi-chunk mode.
    ///
    /// This loads/unloads chunks based on camera position.
    pub fn prepare_multi_chunk_simulation(&mut self) {
        if let Some(cm) = &mut self.chunk_manager {
            cm.prepare_simulation(&self.device, &self.compute_pipeline);
        }
    }

    /// Steps simulation for all active chunks in multi-chunk mode.
    pub fn step_multi_chunk_simulation(&mut self) {
        if let Some(cm) = &mut self.chunk_manager {
            cm.step_simulation(&self.device, &self.queue, &self.compute_pipeline);
        }
    }
}

/// Creates static terrain for top-down RPG view (no gravity/falling).
/// This terrain is designed for exploration, not cell physics simulation.
#[allow(clippy::cast_possible_wrap)]
fn create_static_terrain(chunk_size: usize) -> Vec<genesis_kernel::Cell> {
    use genesis_kernel::{Cell, CellFlags};

    let total_cells = chunk_size * chunk_size;
    let mut cells = vec![Cell::default(); total_cells];

    // Create a top-down RPG terrain with:
    // - Grass as base
    // - Paths of dirt
    // - Some water bodies
    // - Stone formations
    // - Trees (represented by darker grass)

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = y * chunk_size + x;

            // Base grass everywhere
            cells[idx] = Cell::new(3).with_flag(CellFlags::SOLID); // Grass

            // Create some variation with noise-like patterns
            let noise1 = ((x as f32 * 0.1).sin() * (y as f32 * 0.1).cos() * 10.0) as i32;
            let noise2 = ((x as f32 * 0.05 + 2.0).sin() * (y as f32 * 0.07).sin() * 15.0) as i32;
            let combined = noise1 + noise2;

            // Dirt paths (crossing the map)
            let center = chunk_size / 2;
            let dx = (x as i32 - center as i32).abs();
            let dy = (y as i32 - center as i32).abs();

            // Horizontal path
            if dy < 8 && (x > 20 && x < chunk_size - 20) {
                cells[idx] = Cell::new(4).with_flag(CellFlags::SOLID); // Dirt
            }
            // Vertical path
            if dx < 8 && (y > 20 && y < chunk_size - 20) {
                cells[idx] = Cell::new(4).with_flag(CellFlags::SOLID); // Dirt
            }

            // Water lake in corner
            let lake_x = chunk_size * 3 / 4;
            let lake_y = chunk_size * 3 / 4;
            let lake_dx = (x as i32 - lake_x as i32) as f32;
            let lake_dy = (y as i32 - lake_y as i32) as f32;
            let lake_dist = (lake_dx * lake_dx + lake_dy * lake_dy).sqrt();
            if lake_dist < 30.0 {
                cells[idx] = Cell::new(1).with_flag(CellFlags::LIQUID); // Water
            }
            // Sandy beach around water
            if (30.0..35.0).contains(&lake_dist) {
                cells[idx] = Cell::new(2).with_flag(CellFlags::SOLID); // Sand
            }

            // Stone formations (scattered)
            if combined > 12 && lake_dist > 40.0 {
                cells[idx] = Cell::new(5).with_flag(CellFlags::SOLID); // Stone
            }

            // Some darker grass patches (forest areas)
            let forest1_x = chunk_size / 4;
            let forest1_y = chunk_size / 4;
            let forest_dx = (x as i32 - forest1_x as i32) as f32;
            let forest_dy = (y as i32 - forest1_y as i32) as f32;
            let forest_dist = (forest_dx * forest_dx + forest_dy * forest_dy).sqrt();
            if forest_dist < 40.0 && combined > 0 && cells[idx].material == 3 {
                // Make it look like denser vegetation (darker grass - use material 3 with higher temp)
                cells[idx] = Cell::new(3).with_flag(CellFlags::SOLID);
            }
        }
    }

    cells
}

/// Creates terrain for initial visualization - a 2D side-view world.
/// (Legacy function for platformer-style games)
#[allow(clippy::cast_possible_wrap)]
#[allow(dead_code)]
fn create_test_pattern(chunk_size: usize) -> Vec<genesis_kernel::Cell> {
    use genesis_kernel::{Cell, CellFlags};

    let total_cells = chunk_size * chunk_size;
    let mut cells = vec![Cell::default(); total_cells];

    // Create a 2D terrain with:
    // - Sky (air) at top
    // - Rolling hills with grass
    // - Dirt layer below grass
    // - Stone at the bottom
    // - A cave system
    // - A pond of water

    let ground_base = chunk_size / 2; // Ground level at middle

    for x in 0..chunk_size {
        // Create rolling hills using sine waves
        let hill1 = ((x as f32 * 0.05).sin() * 15.0) as i32;
        let hill2 = ((x as f32 * 0.02 + 1.0).sin() * 25.0) as i32;
        let ground_y = (ground_base as i32 + hill1 + hill2) as usize;

        for y in 0..chunk_size {
            let idx = y * chunk_size + x;

            // Below ground
            if y > ground_y {
                let depth = y - ground_y;

                // Grass layer (top 2 cells)
                if depth <= 2 {
                    cells[idx] = Cell::new(3).with_flag(CellFlags::SOLID); // Grass
                }
                // Dirt layer (next 15 cells)
                else if depth <= 17 {
                    cells[idx] = Cell::new(4).with_flag(CellFlags::SOLID); // Dirt
                }
                // Stone below
                else {
                    cells[idx] = Cell::new(5).with_flag(CellFlags::SOLID); // Stone
                }

                // Create a cave in the stone layer
                let cave_center_x = chunk_size / 3;
                let cave_center_y = ground_y + 30;
                let dx = (x as i32 - cave_center_x as i32).abs() as f32;
                let dy = (y as i32 - cave_center_y as i32).abs() as f32;
                let cave_dist = (dx * dx + dy * dy * 0.5).sqrt();
                if cave_dist < 20.0 && depth > 17 {
                    cells[idx] = Cell::default(); // Air (cave)
                }
            }
        }

        // Create a water pond in a depression
        let pond_start = chunk_size * 2 / 3;
        let pond_end = pond_start + 40;
        if x >= pond_start && x < pond_end {
            let pond_depth = 8 - ((x as i32 - (pond_start + 20) as i32).abs() / 3) as usize;
            let pond_ground = (ground_base as i32 + hill1 + hill2) as usize;
            for y in pond_ground.saturating_sub(pond_depth)..=pond_ground {
                if y < chunk_size {
                    let idx = y * chunk_size + x;
                    cells[idx] = Cell::new(1).with_flag(CellFlags::LIQUID); // Water
                }
            }
        }

        // Add some sand near the water
        if x >= pond_start.saturating_sub(5) && x < pond_end + 5 {
            let sand_ground = (ground_base as i32 + hill1 + hill2) as usize;
            for y in sand_ground..sand_ground.saturating_add(3).min(chunk_size) {
                let idx = y * chunk_size + x;
                if cells[idx].material != 1 {
                    // Don't overwrite water
                    cells[idx] = Cell::new(2).with_flag(CellFlags::SOLID); // Sand
                }
            }
        }
    }

    cells
}
