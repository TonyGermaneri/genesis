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

use crate::audio_assets::AudioCategory;
use crate::audio_integration::{AudioIntegration, SoundEvent};
use crate::combat_events::CombatEventHandler;
use crate::combat_profile::CombatProfiler;
use crate::combat_save::CombatPersistence;
use crate::config::EngineConfig;
use crate::crafting_events::CraftingEventHandler;
use crate::crafting_profile::CraftingProfiler;
use crate::crafting_save::CraftingPersistence;
use crate::environment::EnvironmentState;
use crate::input::InputHandler;
use crate::perf::PerfMetrics;
use crate::recipe_loader::RecipeLoader;
use crate::renderer::Renderer;
use crate::timing::{ChunkMetrics, FpsCounter, FrameTiming, NpcMetrics};
use crate::weapon_loader::WeaponLoader;
use crate::world::TerrainGenerationService;

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
    /// NPC-specific metrics
    npc_metrics: NpcMetrics,

    // === World Generation ===
    /// Terrain generation service with biome management
    terrain_service: TerrainGenerationService,

    // === NPC Spawning ===
    /// NPC chunk spawner for loading/unloading NPCs with chunks
    npc_spawner: genesis_gameplay::NPCChunkSpawner,
    /// Last player chunk position (for detecting chunk changes)
    last_player_chunk: (i32, i32),

    // === Audio ===
    /// Audio integration system
    audio: AudioIntegration,

    // === Crafting ===
    /// Recipe loader for loading recipes from assets
    recipe_loader: RecipeLoader,
    /// Crafting event handler
    crafting_events: CraftingEventHandler,
    /// Crafting persistence (learned recipes, workbenches)
    crafting_persistence: CraftingPersistence,
    /// Crafting profiler for performance tracking
    crafting_profiler: CraftingProfiler,
    /// Whether crafting UI is open
    show_crafting: bool,

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

        // Create terrain generation service from config
        let terrain_service = TerrainGenerationService::from_engine_config(&config);
        let seed = terrain_service.seed();
        info!("World seed: {}", seed);

        // Create gameplay state with the world seed
        // Spawn player at center of chunk (128, 128) for 256x256 chunk
        let mut gameplay = GameplayState::with_player_position(seed, (128.0, 100.0));
        // Set player as grounded for top-down movement
        gameplay.player.set_grounded(true);

        // Create NPC chunk spawner with the world seed
        let npc_spawn_config = genesis_gameplay::NPCSpawnConfig::with_seed(seed);
        let npc_spawner = genesis_gameplay::NPCChunkSpawner::new(npc_spawn_config);

        // Initialize audio system
        let mut audio = AudioIntegration::with_default_assets();
        if audio.is_available() {
            info!("Audio system initialized");
            // Preload SFX for immediate playback
            audio.preload_sfx();
        } else {
            warn!("Audio system not available - continuing without audio");
        }

        // Initialize crafting system
        let mut recipe_loader = RecipeLoader::with_default_path();
        if let Err(e) = recipe_loader.load_all() {
            warn!("Failed to load recipes: {}", e);
        } else {
            info!("Loaded {} recipes", recipe_loader.registry().len());
        }
        let crafting_events = CraftingEventHandler::new();
        // Starter recipes that all players know (basic tools)
        let crafting_persistence = CraftingPersistence::with_starter_recipes([1, 2, 3, 4, 5]);
        let crafting_profiler = CraftingProfiler::new();

        // Create camera with default viewport and higher zoom for visibility
        let mut camera = Camera::new(config.window_width, config.window_height);
        camera.set_zoom(4.0); // 4x zoom for bigger pixels

        // Calculate initial player chunk
        let player_pos = gameplay.player_position();
        let chunk_size = 256; // Default chunk size
        let initial_chunk = (
            (player_pos.0 / chunk_size as f32).floor() as i32,
            (player_pos.1 / chunk_size as f32).floor() as i32,
        );

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
            npc_metrics: NpcMetrics::new(),
            terrain_service,
            npc_spawner,
            last_player_chunk: initial_chunk,
            audio,
            recipe_loader,
            crafting_events,
            crafting_persistence,
            crafting_profiler,
            show_crafting: false,

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

        // Handle crafting toggle (C key)
        if self.input.is_key_just_pressed(KeyCode::C) {
            self.show_crafting = !self.show_crafting;
            info!(
                "Crafting: {}",
                if self.show_crafting { "OPEN" } else { "CLOSED" }
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

        // Handle NPC interaction (E key)
        if self.input.is_key_just_pressed(KeyCode::E) {
            if self.gameplay.is_interacting() {
                // End current interaction
                self.gameplay.end_interaction();
                info!("Ended NPC interaction");
            } else if self.gameplay.try_interact() {
                // Started new interaction
                if let Some(entity_id) = self.gameplay.npc_interaction().interacting_with {
                    info!("Started NPC interaction with {:?}", entity_id);
                }
            }
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

        // Time NPC updates (NPCs are updated inside gameplay.update via fixed_update)
        let npc_start = Instant::now();

        // Update gameplay state (player, entities, etc.)
        self.gameplay.update(dt, &input);

        // Record NPC update timing
        let npc_elapsed = npc_start.elapsed();
        self.npc_metrics.record_ai_time(npc_elapsed);
        self.npc_metrics.set_npc_count(self.gameplay.npc_count());

        // Update nearest interactable NPC for UI prompt
        self.gameplay.update_nearest_interactable();

        // Update camera to follow player
        let player_pos = self.gameplay.player.position();
        self.camera.center_on(player_pos.x, player_pos.y);

        // Update audio system
        self.update_audio(dt, player_pos.x, player_pos.y);

        // Update crafting system (check hot-reload, process events)
        self.update_crafting(dt);

        // Check for chunk changes and spawn/despawn NPCs
        self.update_npc_chunks();

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

    /// Updates NPC spawning/despawning based on player chunk position.
    fn update_npc_chunks(&mut self) {
        let player_pos = self.gameplay.player_position();
        let chunk_size = self.npc_spawner.config().chunk_size as f32;

        let current_chunk = (
            (player_pos.0 / chunk_size).floor() as i32,
            (player_pos.1 / chunk_size).floor() as i32,
        );

        // Only update if player moved to a different chunk
        if current_chunk == self.last_player_chunk {
            return;
        }

        let old_chunk = self.last_player_chunk;
        self.last_player_chunk = current_chunk;

        // Calculate which chunks should be loaded (3x3 grid around player)
        let render_distance = 1; // Load chunks within 1 chunk of player
        let mut chunks_to_load = Vec::new();
        let mut chunks_to_unload = Vec::new();

        // Determine chunks that should be loaded around new position
        for dx in -render_distance..=render_distance {
            for dy in -render_distance..=render_distance {
                let chunk = (current_chunk.0 + dx, current_chunk.1 + dy);
                if self.npc_spawner.get_chunk_npcs(chunk).is_none() {
                    chunks_to_load.push(chunk);
                }
            }
        }

        // Determine chunks that should be unloaded (were in range of old pos but not new)
        for dx in -render_distance..=render_distance {
            for dy in -render_distance..=render_distance {
                let old_visible_chunk = (old_chunk.0 + dx, old_chunk.1 + dy);
                // Check if this chunk is still visible from new position
                let still_visible = (old_visible_chunk.0 - current_chunk.0).abs()
                    <= render_distance
                    && (old_visible_chunk.1 - current_chunk.1).abs() <= render_distance;

                if !still_visible && self.npc_spawner.get_chunk_npcs(old_visible_chunk).is_some() {
                    chunks_to_unload.push(old_visible_chunk);
                }
            }
        }

        // Load new chunks
        for chunk_pos in chunks_to_load {
            let count = self
                .npc_spawner
                .on_chunk_loaded(chunk_pos, self.gameplay.npc_manager_mut());
            if count > 0 {
                debug!("Spawned {} NPCs in chunk {:?}", count, chunk_pos);
            }
        }

        // Unload old chunks
        for chunk_pos in chunks_to_unload {
            let count = self
                .npc_spawner
                .on_chunk_unloaded(chunk_pos, self.gameplay.npc_manager_mut());
            if count > 0 {
                debug!("Despawned {} NPCs from chunk {:?}", count, chunk_pos);
            }
        }
    }

    /// Spawns initial NPCs around the player's starting position.
    fn spawn_initial_npcs(&mut self) {
        let chunk_size = self.npc_spawner.config().chunk_size as f32;
        let player_pos = self.gameplay.player_position();

        let current_chunk = (
            (player_pos.0 / chunk_size).floor() as i32,
            (player_pos.1 / chunk_size).floor() as i32,
        );

        // Load NPCs in 3x3 grid around player
        let render_distance = 1;
        let mut total_spawned = 0;

        for dx in -render_distance..=render_distance {
            for dy in -render_distance..=render_distance {
                let chunk_pos = (current_chunk.0 + dx, current_chunk.1 + dy);
                let count = self
                    .npc_spawner
                    .on_chunk_loaded(chunk_pos, self.gameplay.npc_manager_mut());
                total_spawned += count;
            }
        }

        if total_spawned > 0 {
            info!("Spawned {} initial NPCs around player", total_spawned);
        }
    }

    /// Updates audio system for the frame.
    fn update_audio(&mut self, dt: f32, player_x: f32, player_y: f32) {
        // Update listener position to player
        self.audio.set_listener_position(player_x, player_y);

        // Update music based on biome
        self.update_biome_music();

        // Update ambient based on environment
        self.update_ambient_audio();

        // Process queued sounds and update fades
        self.audio.update(dt);
    }

    /// Updates music track based on current biome.
    fn update_biome_music(&mut self) {
        // Get current biome from player position
        let player_pos = self.gameplay.player_position();
        let biome = self
            .terrain_service
            .get_biome_at(player_pos.0, player_pos.1);

        // Map biome to music track (if different from current)
        #[allow(clippy::match_same_arms)]
        let track_name = match biome.as_str() {
            "plains" | "grassland" => "exploration_plains",
            "forest" | "woodland" => "exploration_forest",
            "desert" | "wasteland" => "exploration_desert",
            "snow" | "tundra" | "arctic" => "exploration_snow",
            "swamp" | "marsh" => "exploration_swamp",
            "mountain" | "highland" => "exploration_mountain",
            "cave" | "underground" => "exploration_cave",
            _ => "exploration_plains", // Default
        };

        // Only change if different from current (to avoid resetting)
        if self.audio.state().music.current_track.as_deref() != Some(track_name) {
            // Check if we have this track, otherwise skip
            if self.audio.state().music.is_playing() {
                self.audio.crossfade_music(track_name, 3.0);
            } else {
                self.audio.play_music(track_name, Some(2.0));
            }
        }
    }

    /// Updates ambient audio based on environment state.
    fn update_ambient_audio(&mut self) {
        let hour = self.environment.time.hour();
        let is_night = !(6..20).contains(&hour);
        let is_dawn_dusk = (5..7).contains(&hour) || (18..20).contains(&hour);

        // Day/night ambient layers
        if is_night {
            self.audio
                .fade_in_ambient("night", "ambient/night_crickets", 0.5, 2.0);
            self.audio.fade_out_ambient("day", 2.0);
        } else if is_dawn_dusk {
            // Dawn/dusk transition - both layers at reduced volume
            self.audio.fade_in_ambient("day", "ambient/birds", 0.3, 2.0);
            self.audio
                .fade_in_ambient("night", "ambient/night_crickets", 0.2, 2.0);
        } else {
            self.audio.fade_in_ambient("day", "ambient/birds", 0.5, 2.0);
            self.audio.fade_out_ambient("night", 2.0);
        }

        // Weather-based ambient
        if self.environment.weather.is_raining() {
            let rain_volume = self.environment.weather.rain_intensity();
            self.audio
                .fade_in_ambient("rain", "ambient/rain", rain_volume, 1.0);
        } else {
            self.audio.fade_out_ambient("rain", 2.0);
        }

        if self.environment.weather.is_stormy() {
            self.audio
                .fade_in_ambient("thunder", "ambient/thunder", 0.7, 0.5);
        } else {
            self.audio.fade_out_ambient("thunder", 1.0);
        }

        // Wind based on weather intensity
        let wind_volume = self.environment.weather.wind_strength() * 0.4;
        if wind_volume > 0.1 {
            self.audio
                .fade_in_ambient("wind", "ambient/wind", wind_volume, 1.5);
        } else {
            self.audio.fade_out_ambient("wind", 2.0);
        }
    }

    /// Plays a sound effect for a gameplay event.
    #[allow(dead_code)]
    pub fn play_sfx(&mut self, name: &str, position: Option<(f32, f32)>) {
        let mut event = SoundEvent::new(AudioCategory::Sfx, name);
        if let Some((x, y)) = position {
            event = event.at_position(x, y);
        }
        self.audio.queue_sound(event);
    }

    /// Plays a UI sound effect.
    #[allow(dead_code)]
    pub fn play_ui_sound(&mut self, name: &str) {
        let event = SoundEvent::new(AudioCategory::Ui, name);
        self.audio.queue_sound(event);
    }

    /// Updates crafting system for the frame.
    fn update_crafting(&mut self, _dt: f32) {
        // Check for recipe hot-reload in debug mode
        if self.recipe_loader.check_hot_reload() {
            info!("Recipes hot-reloaded");
        }

        // Process pending crafting events
        let result = self.crafting_events.process_events(Some(&mut self.audio));

        // Handle completed crafts
        for (recipe_id, _output_item, _quantity) in &result.completed_crafts {
            // Add to recent recipes
            self.crafting_persistence.add_recent(*recipe_id);

            // Record for profiling
            if let Some(recipe) = self.recipe_loader.get_recipe(recipe_id.raw()) {
                self.crafting_profiler
                    .record_craft(recipe_id.raw(), &recipe.category);
            }
        }

        // Handle learned recipes
        for recipe_id in &result.recipes_learned {
            self.crafting_persistence.learn_recipe(*recipe_id);
        }

        // Update playtime for frequency stats
        let playtime = self.gameplay.game_time();
        self.crafting_profiler.update_playtime(playtime);
    }

    /// Render the frame.
    fn render(&mut self) {
        // Extract all data needed for UI before borrowing renderer
        let show_debug = self.show_debug;
        let show_inventory = self.show_inventory;
        let hotbar_slot = self.hotbar_slot;
        let biome_metrics = self.terrain_service.metrics();
        let debug_data = DebugOverlayData {
            perf: self.perf_metrics.summary(),
            time: self.environment.time.clone(),
            weather: self.environment.weather.clone(),
            ambient_light: self.environment.ambient_light(),
            chunk_count: self.chunk_metrics.chunk_count(),
            chunk_load_ms: self.chunk_metrics.avg_load_time_ms(),
            chunk_sim_ms: self.chunk_metrics.avg_sim_time_ms(),
            chunk_exceeds_budget: self.chunk_metrics.exceeds_budget(),
            world_seed: self.terrain_service.seed(),
            biome_gen_avg_ms: biome_metrics.avg_generation_time_ms(),
            biome_gen_peak_ms: biome_metrics.peak_generation_time_ms(),
            biome_chunks_generated: biome_metrics.total_chunks_generated(),
            biome_exceeds_budget: biome_metrics.exceeds_budget(),
            npc_count: self.gameplay.npc_count(),
            npc_update_avg_ms: self.npc_metrics.avg_ai_time_ms(),
            npc_update_peak_ms: self.npc_metrics.peak_ai_time_ms(),
            npc_exceeds_budget: self.npc_metrics.exceeds_budget(),
        };
        let environment_time = debug_data.time.clone();
        let environment_weather = debug_data.weather.clone();

        // Collect interaction data for UI
        let npc_interaction = self.gameplay.npc_interaction();
        let interaction_data = InteractionData {
            can_interact: npc_interaction.nearest_interactable.is_some(),
            is_interacting: npc_interaction.interacting_with.is_some(),
            mode: npc_interaction.mode,
        };

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

                // Show interaction prompt if near an NPC
                render_interaction_prompt(ctx, &interaction_data);
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
    // Biome generation metrics
    world_seed: u64,
    biome_gen_avg_ms: f64,
    biome_gen_peak_ms: f64,
    biome_chunks_generated: u64,
    biome_exceeds_budget: bool,
    // NPC metrics
    npc_count: usize,
    npc_update_avg_ms: f64,
    npc_update_peak_ms: f64,
    npc_exceeds_budget: bool,
}

/// Data needed for NPC interaction UI.
struct InteractionData {
    /// Whether player can interact with nearby NPC
    can_interact: bool,
    /// Whether currently in an interaction
    is_interacting: bool,
    /// Interaction mode (if interacting)
    mode: genesis_gameplay::NPCInteractionMode,
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

            // Biome generation metrics
            ui.separator();
            ui.label(format!("World Seed: {}", data.world_seed));
            if data.biome_chunks_generated > 0 {
                let biome_gen_avg_ms = data.biome_gen_avg_ms;
                let biome_gen_peak_ms = data.biome_gen_peak_ms;
                ui.label(format!(
                    "Biome Gen: {biome_gen_avg_ms:.2}ms avg, {biome_gen_peak_ms:.2}ms peak"
                ));
                ui.label(format!("Chunks Generated: {}", data.biome_chunks_generated));
                if data.biome_exceeds_budget {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Biome gen > 16ms!");
                }
            }

            // NPC metrics
            ui.separator();
            ui.label(format!("NPCs: {}", data.npc_count));
            if data.npc_count > 0 {
                ui.label(format!(
                    "NPC Update: {:.2}ms avg, {:.2}ms peak",
                    data.npc_update_avg_ms, data.npc_update_peak_ms
                ));
                if data.npc_exceeds_budget {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ NPC update > 2ms!");
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

/// Renders the NPC interaction prompt or dialogue window.
fn render_interaction_prompt(ctx: &egui::Context, data: &InteractionData) {
    use genesis_gameplay::NPCInteractionMode;

    if data.is_interacting {
        // Show interaction window based on mode
        let title = match data.mode {
            NPCInteractionMode::Trading => "Trade",
            NPCInteractionMode::Dialogue => "Dialogue",
            NPCInteractionMode::None => return,
        };

        egui::Window::new(title)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| match data.mode {
                NPCInteractionMode::Trading => {
                    ui.label("Trading with Merchant");
                    ui.separator();
                    ui.label("(Trade UI coming soon...)");
                    ui.separator();
                    ui.label("Press [E] to close");
                },
                NPCInteractionMode::Dialogue => {
                    ui.label("NPC says: Hello, traveler!");
                    ui.separator();
                    ui.label("Press [E] to close");
                },
                NPCInteractionMode::None => {},
            });
    } else if data.can_interact {
        // Show interaction prompt
        egui::Window::new("Interaction")
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -80.0))
            .title_bar(false)
            .resizable(false)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)),
            )
            .show(ctx, |ui| {
                ui.label("Press [E] to interact");
            });
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

                // Spawn initial NPCs around player
                self.spawn_initial_npcs();

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
