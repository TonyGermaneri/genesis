//! GPU renderer using wgpu.
//!
//! Integrates the cell simulation compute pipeline with visual rendering.

#![allow(unsafe_code)]
#![allow(dead_code)]

use anyhow::{Context, Result};
use genesis_gameplay::GameState;
use genesis_kernel::{Camera, CellBuffer, CellComputePipeline, CellRenderPipeline};
use tracing::info;
use winit::{dpi::PhysicalSize, window::Window};

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
    /// Cell buffer (double-buffered)
    cell_buffer: CellBuffer,
    /// Render bind group for current buffer
    render_bind_group: wgpu::BindGroup,
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

        // Create cell buffer with initial test data
        let chunk_size = DEFAULT_CHUNK_SIZE;
        let initial_cells = create_test_pattern(chunk_size as usize);
        let cell_buffer =
            CellBuffer::with_cells(&device, &compute_pipeline, chunk_size, &initial_cells);

        // Create render bind group
        let render_bind_group =
            render_pipeline.create_bind_group(&device, cell_buffer.current_buffer());

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
            render_bind_group,
            simulation_running: true,
            frame_count: 0,
        })
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
}

/// Creates terrain for initial visualization - a 2D side-view world.
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
                if cells[idx].material != 1 { // Don't overwrite water
                    cells[idx] = Cell::new(2).with_flag(CellFlags::SOLID); // Sand
                }
            }
        }
    }

    cells
}
