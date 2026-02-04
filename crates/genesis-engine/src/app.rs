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

use genesis_gameplay::input::KeyCode;
use genesis_gameplay::GameState as GameplayState;
use genesis_kernel::Camera;

use crate::config::EngineConfig;
use crate::environment::EnvironmentState;
use crate::input::InputHandler;
use crate::perf::PerfMetrics;
use crate::renderer::Renderer;
use crate::timing::{ChunkMetrics, FpsCounter, FrameTiming};

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

    // === Environment ===
    /// Environment state (time, weather)
    environment: EnvironmentState,

    // === Performance Tracking ===
    /// Performance metrics collector
    perf_metrics: PerfMetrics,
    /// Chunk-specific metrics
    chunk_metrics: ChunkMetrics,

    // === Gameplay State ===
    /// Gameplay state (player, entities, etc.)
    gameplay: GameplayState,
    /// Camera for viewing the world
    camera: Camera,
    /// Application mode
    app_mode: AppMode,
    /// Whether debug overlay is visible
    show_debug: bool,
    /// Whether inventory is open
    show_inventory: bool,
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

            environment: EnvironmentState::new(),
            perf_metrics: PerfMetrics::new(120),
            chunk_metrics: ChunkMetrics::new(),

            gameplay,
            camera,
            app_mode: AppMode::default(),
            show_inventory: false,
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

        // Handle inventory toggle (Tab)
        if self.input.is_key_just_pressed(KeyCode::Tab) {
            self.show_inventory = !self.show_inventory;
            info!(
                "Inventory: {}",
                if self.show_inventory {
                    "OPEN"
                } else {
                    "CLOSED"
                }
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

        // Update environment (time and weather)
        self.environment.update(dt);

        // Update game logic (only when playing)
        if self.app_mode == AppMode::Playing {
            self.update_gameplay(dt);
        }

        // Record performance metrics
        self.perf_metrics.record_frame(dt, dt * 0.3, dt * 0.5); // Approximate update/render split
        if let Some(renderer) = &self.renderer {
            self.perf_metrics.set_world_stats(
                renderer.visible_chunk_count() as u32,
                renderer.total_cell_count(),
            );
        }
        self.perf_metrics.set_camera(
            (self.camera.position.0, self.camera.position.1),
            self.camera.zoom,
        );
        let player_pos = self.gameplay.player.position();
        let player_vel = self.gameplay.player.velocity();
        self.perf_metrics
            .set_player((player_pos.x, player_pos.y), (player_vel.x, player_vel.y));

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

        // Update chunk manager camera position for multi-chunk streaming
        if let Some(renderer) = &mut self.renderer {
            renderer.update_camera_position(&self.camera);

            // Prepare and step multi-chunk simulation if enabled
            if renderer.is_multi_chunk_enabled() {
                let start = Instant::now();
                renderer.prepare_multi_chunk_simulation();
                self.chunk_metrics.record_load_time(start.elapsed());

                let start = Instant::now();
                renderer.step_multi_chunk_simulation();
                self.chunk_metrics.record_sim_time(start.elapsed());

                // Update chunk metrics
                self.chunk_metrics
                    .set_chunk_count(renderer.visible_chunk_count() as u32);
            }
        }

        // Update frame timing
        let _ = self.timing.accumulate(dt);
    }

    /// Render the frame.
    fn render(&mut self) {
        // Extract all data needed for UI before borrowing renderer
        let show_debug = self.show_debug;
        let show_inventory = self.show_inventory;
        let hotbar_slot = self.hotbar_slot;
        let debug_data = DebugOverlayData {
            perf: self.perf_metrics.summary(),
            time: self.environment.time.clone(),
            weather: self.environment.weather.clone(),
            ambient_light: self.environment.ambient_light(),
            chunk_count: self.chunk_metrics.chunk_count(),
            chunk_load_ms: self.chunk_metrics.avg_load_time_ms(),
            chunk_sim_ms: self.chunk_metrics.avg_sim_time_ms(),
            chunk_exceeds_budget: self.chunk_metrics.exceeds_budget(),
        };
        let environment_time = debug_data.time.clone();
        let environment_weather = debug_data.weather.clone();

        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            // Use render_with_ui to draw world + egui overlay
            let result = renderer.render_with_ui(window, &self.camera, |ctx| {
                // Only show UI elements when needed
                if show_debug {
                    render_debug_overlay(ctx, &debug_data);
                }

                if show_inventory {
                    render_inventory(ctx, hotbar_slot);
                }

                // Always show HUD elements (hotbar, vitals, minimap)
                render_hud(ctx, hotbar_slot, &environment_time, &environment_weather);
            });

            if let Err(e) = result {
                warn!("Render error: {e}");
            }
        }
    }
}

/// Data needed for the debug overlay (to avoid borrow conflicts).
struct DebugOverlayData {
    perf: crate::perf::PerfSummary,
    time: crate::environment::GameTime,
    weather: crate::environment::WeatherSystem,
    ambient_light: f32,
    chunk_count: u32,
    chunk_load_ms: f64,
    chunk_sim_ms: f64,
    chunk_exceeds_budget: bool,
}

/// Renders the debug overlay.
#[allow(clippy::too_many_lines)]
fn render_debug_overlay(ctx: &egui::Context, data: &DebugOverlayData) {
    egui::Window::new("Debug")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .resizable(false)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label(format!(
                "FPS: {:.0} (1% low: {:.0})",
                data.perf.fps, data.perf.fps_1_percent_low
            ));
            ui.label(format!(
                "Frame: {:.1}ms (Update: {:.1}ms, Render: {:.1}ms)",
                data.perf.frame_time_ms, data.perf.update_time_ms, data.perf.render_time_ms
            ));
            ui.separator();

            ui.label(format!(
                "Chunks: {} ({} cells)",
                data.perf.chunks_loaded,
                format_cells(data.perf.cells_simulated)
            ));
            ui.label(format!(
                "Camera: ({:.1}, {:.1}) Zoom: {:.1}x",
                data.perf.camera_position.0, data.perf.camera_position.1, data.perf.zoom
            ));
            ui.label(format!(
                "Player: ({:.1}, {:.1})",
                data.perf.player_position.0, data.perf.player_position.1
            ));
            ui.separator();

            // Environment info
            ui.label(format!(
                "Time: {} (Day {})",
                data.time.formatted_time(),
                data.time.day_count()
            ));
            ui.label(format!(
                "Weather: {} ({})",
                data.weather.current_weather().display_name(),
                if data.weather.is_raining() {
                    "Raining"
                } else {
                    "Dry"
                }
            ));
            ui.label(format!("Ambient: {:.0}%", data.ambient_light * 100.0));

            // Chunk metrics
            if data.chunk_count > 0 {
                ui.separator();
                let chunk_load_ms = data.chunk_load_ms;
                let chunk_sim_ms = data.chunk_sim_ms;
                ui.label(format!("Chunk Load: {chunk_load_ms:.2}ms"));
                ui.label(format!("Chunk Sim: {chunk_sim_ms:.2}ms"));
                if data.chunk_exceeds_budget {
                    ui.colored_label(egui::Color32::RED, "⚠ Frame budget exceeded!");
                }
            }
        });
}

/// Renders the inventory panel.
fn render_inventory(ctx: &egui::Context, hotbar_slot: u8) {
    egui::Window::new("Inventory")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label("Inventory (Tab to close)");
            ui.separator();

            // Display inventory slots in a grid
            egui::Grid::new("inventory_grid")
                .num_columns(10)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    for row in 0..4 {
                        for col in 0..10 {
                            let slot = row * 10 + col;
                            let is_selected = slot == hotbar_slot as usize && row == 0;
                            let (rect, _response) = ui
                                .allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::click());
                            let color = if is_selected {
                                egui::Color32::from_rgb(100, 150, 200)
                            } else {
                                egui::Color32::from_rgb(60, 60, 60)
                            };
                            ui.painter()
                                .rect_filled(rect, egui::Rounding::same(4.0), color);
                            ui.painter().rect_stroke(
                                rect,
                                egui::Rounding::same(4.0),
                                egui::Stroke::new(1.0, egui::Color32::GRAY),
                            );
                        }
                        ui.end_row();
                    }
                });
        });
}

/// Renders the main HUD (hotbar, vitals, minimap).
fn render_hud(
    ctx: &egui::Context,
    hotbar_slot: u8,
    time: &crate::environment::GameTime,
    weather: &crate::environment::WeatherSystem,
) {
    // Hotbar at bottom center
    egui::TopBottomPanel::bottom("hotbar")
        .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space((ui.available_width() - 10.0 * 48.0 - 9.0 * 4.0) / 2.0);
                for i in 0..10 {
                    let is_selected = i == hotbar_slot;
                    let (rect, _response) =
                        ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
                    let color = if is_selected {
                        egui::Color32::from_rgb(100, 150, 200)
                    } else {
                        egui::Color32::from_rgb(40, 40, 40)
                    };
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::same(4.0), color);
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(4.0),
                        egui::Stroke::new(
                            if is_selected { 2.0 } else { 1.0 },
                            if is_selected {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::GRAY
                            },
                        ),
                    );
                    // Draw slot number
                    ui.painter().text(
                        rect.left_top() + egui::vec2(4.0, 2.0),
                        egui::Align2::LEFT_TOP,
                        format!("{}", (i + 1) % 10),
                        egui::FontId::proportional(10.0),
                        egui::Color32::GRAY,
                    );
                }
            });
        });

    // Environment info (time/weather) in top right
    egui::Window::new("Environment")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
        .title_bar(false)
        .resizable(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Time icon and value
                ui.label(if time.is_daytime() { "☀" } else { "☾" });
                ui.label(time.formatted_time());
                ui.separator();
                // Weather
                ui.label(weather.current_weather().display_name());
            });
        });
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

/// Formats a cell count with commas for readability.
fn format_cells(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
