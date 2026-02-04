//! Application lifecycle management.
//!
//! Main game loop that integrates all subsystems.

use anyhow::Result;
use std::time::Instant;
use tracing::{debug, info, warn};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use genesis_gameplay::GameState as GameplayState;
use genesis_kernel::Camera;

use crate::config::EngineConfig;
use crate::input::InputHandler;
use crate::renderer::Renderer;
use crate::timing::{FpsCounter, FrameTiming};

// Re-export egui for UI building
use egui;

/// Application mode (menu/playing/paused).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum AppMode {
    /// Normal gameplay
    #[default]
    Playing,
    /// Game is paused
    Paused,
    /// In main menu
    Menu,
}

/// Application state machine.
struct GenesisApp {
    /// Engine configuration
    config: EngineConfig,
    /// Window handle (created after resume)
    window: Option<Window>,
    /// Renderer (initialized after window creation)
    renderer: Option<Renderer>,

    // === Game Systems ===
    /// Input handler
    input: InputHandler,
    /// Frame timing
    timing: FrameTiming,
    /// FPS counter for display
    fps_counter: FpsCounter,
    /// Last update time
    last_update: Instant,

    // === Gameplay State ===
    /// Gameplay state (player, entities, etc.)
    gameplay: GameplayState,
    /// Camera for viewing the world
    camera: Camera,
    /// Application mode
    app_mode: AppMode,
    /// Whether debug overlay is visible
    show_debug: bool,
    /// Currently selected hotbar slot
    hotbar_slot: u8,

    // === Debug Info ===
    /// Current FPS
    current_fps: f32,
    /// Current frame time in ms
    current_frame_time: f32,
}

impl GenesisApp {
    /// Creates a new application instance.
    fn new(config: EngineConfig) -> Self {
        let timing = FrameTiming::new(config.target_fps).with_vsync(config.vsync);

        // Create gameplay state with a seed
        let seed = 42; // TODO: Make configurable or random
                       // Spawn player at center of chunk (128, 128) for 256x256 chunk
        let mut gameplay = GameplayState::with_player_position(seed, (128.0, 100.0));
        // Set player as grounded for top-down movement
        gameplay.player.set_grounded(true);

        // Create camera with default viewport and higher zoom for visibility
        let mut camera = Camera::new(config.window_width, config.window_height);
        camera.set_zoom(4.0); // 4x zoom for bigger pixels

        Self {
            show_debug: config.show_debug_overlay,
            config,
            window: None,
            renderer: None,

            input: InputHandler::new(),
            timing,
            fps_counter: FpsCounter::new(),
            last_update: Instant::now(),

            gameplay,
            camera,
            app_mode: AppMode::default(),
            hotbar_slot: 0,

            current_fps: 0.0,
            current_frame_time: 0.0,
        }
    }

    /// Main update and render loop.
    fn update_and_render(&mut self) {
        // Calculate delta time
        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32().min(0.25); // Clamp to prevent spiral
        self.last_update = now;

        // Update FPS counter
        let (fps, frame_time) = self.fps_counter.tick();
        self.current_fps = fps;
        self.current_frame_time = frame_time;

        // Handle debug toggle (F3)
        if self.input.debug_toggle_pressed() {
            self.show_debug = !self.show_debug;
            self.config.show_debug_overlay = self.show_debug;
            info!(
                "Debug overlay: {}",
                if self.show_debug { "ON" } else { "OFF" }
            );
        }

        // Handle pause toggle (Escape)
        if self.input.pause_pressed() {
            self.app_mode = match self.app_mode {
                AppMode::Playing => {
                    info!("Game paused");
                    AppMode::Paused
                },
                AppMode::Paused => {
                    info!("Game resumed");
                    AppMode::Playing
                },
                AppMode::Menu => AppMode::Menu,
            };
        }

        // Handle hotbar selection
        if let Some(slot) = self.input.hotbar_selection() {
            self.hotbar_slot = slot;
            debug!("Hotbar slot selected: {}", slot + 1);
        }

        // Update game logic (only when playing)
        if self.app_mode == AppMode::Playing {
            self.update_gameplay(dt);
        }

        // Render
        self.render();

        // End frame input processing
        self.input.end_frame();

        // Frame rate limiting (if not using VSync)
        self.timing.sleep_remainder();
    }

    /// Update gameplay systems.
    fn update_gameplay(&mut self, dt: f32) {
        // Get processed input from the engine's input handler
        // This already returns the gameplay Input struct
        let input = self.input.get_input();

        // Update gameplay state (player, entities, etc.)
        self.gameplay.update(dt, &input);

        // Update camera to follow player
        let player_pos = self.gameplay.player.position();
        self.camera.center_on(player_pos.x, player_pos.y);

        // Update frame timing
        let _ = self.timing.accumulate(dt);
    }

    /// Render the frame with egui UI overlay.
    fn render(&mut self) {
        let (renderer, window) = match (&mut self.renderer, &self.window) {
            (Some(r), Some(w)) => (r, w),
            _ => return,
        };

        // Capture state for UI closure
        let show_debug = self.show_debug;
        let fps = self.current_fps;
        let frame_time = self.current_frame_time;
        let player_pos = self.gameplay.player.position();
        let player_vel = self.gameplay.player.velocity();
        let camera_pos = self.camera.position;
        let camera_zoom = self.camera.zoom;
        let hotbar_slot = self.hotbar_slot;
        let app_mode = self.app_mode;

        // Render with egui UI
        let result = renderer.render_with_ui(window, &self.camera, |ctx| {
            // Debug overlay (top-left)
            if show_debug {
                egui::Window::new("Debug")
                    .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
                    .resizable(false)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label(format!("FPS: {:.0}", fps));
                        ui.label(format!("Frame: {:.2}ms", frame_time));
                        ui.separator();
                        ui.label(format!("Player: ({:.0}, {:.0})", player_pos.x, player_pos.y));
                        ui.label(format!("Velocity: ({:.1}, {:.1})", player_vel.x, player_vel.y));
                        ui.separator();
                        ui.label(format!("Camera: ({:.0}, {:.0})", camera_pos.0, camera_pos.1));
                        ui.label(format!("Zoom: {:.1}x", camera_zoom));
                    });
            }

            // Hotbar (bottom-center)
            egui::TopBottomPanel::bottom("hotbar_panel")
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 400.0) / 2.0);
                        for i in 0..10 {
                            let selected = hotbar_slot == i;
                            let text = format!("{}", (i + 1) % 10);
                            let btn = egui::Button::new(text)
                                .min_size(egui::vec2(36.0, 36.0))
                                .fill(if selected {
                                    egui::Color32::from_rgb(80, 120, 180)
                                } else {
                                    egui::Color32::from_rgb(40, 40, 50)
                                });
                            ui.add(btn);
                        }
                    });
                    ui.add_space(10.0);
                });

            // Mode indicator
            if app_mode == AppMode::Paused {
                egui::Window::new("Paused")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .resizable(false)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("PAUSED");
                        ui.label("Press ESC to resume");
                    });
            }
        });

        if let Err(e) = result {
            warn!("Render error: {e}");
        }
    }
}

impl ApplicationHandler for GenesisApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Application resumed, creating window...");

        let window_attrs = Window::default_attributes()
            .with_title("Project Genesis")
            .with_inner_size(PhysicalSize::new(
                self.config.window_width,
                self.config.window_height,
            ));

        match event_loop.create_window(window_attrs) {
            Ok(window) => {
                info!("Window created successfully");

                // Initialize renderer
                match pollster::block_on(Renderer::new(&window)) {
                    Ok(renderer) => {
                        info!("Renderer initialized");
                        self.renderer = Some(renderer);
                    },
                    Err(e) => {
                        warn!("Failed to initialize renderer: {e}");
                    },
                }

                self.window = Some(window);

                // Reset timing after window creation
                self.timing.reset();
                self.last_update = Instant::now();

                info!(
                    "Genesis Engine ready - {}x{} @ {} FPS target",
                    self.config.window_width, self.config.window_height, self.config.target_fps
                );
            },
            Err(e) => {
                warn!("Failed to create window: {e}");
                event_loop.exit();
            },
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let input handler process the event first
        let handled = self.input.handle_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, shutting down...");
                // Save config on exit
                if let Err(e) = self.config.save() {
                    warn!("Failed to save config: {e}");
                }
                event_loop.exit();
            },
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
                // Update config and camera viewport
                self.config.window_width = new_size.width;
                self.config.window_height = new_size.height;
                self.camera.set_viewport(new_size.width, new_size.height);
            },
            WindowEvent::RedrawRequested => {
                self.update_and_render();

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            _ => {
                if !handled {
                    // Event wasn't handled by input or above
                }
            },
        }
    }
}

/// Runs the main application loop.
pub fn run() -> Result<()> {
    // Load configuration
    let mut config = EngineConfig::load();
    config.validate();

    info!("Configuration loaded:");
    info!("  Window: {}x{}", config.window_width, config.window_height);
    info!("  VSync: {}", config.vsync);
    info!("  Render distance: {} chunks", config.render_distance);

    info!("Creating event loop...");
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = GenesisApp::new(config);

    info!("Starting event loop...");
    event_loop.run_app(&mut app)?;

    Ok(())
}
