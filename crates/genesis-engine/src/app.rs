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
use genesis_tools::ui::{
    MainMenu, MainMenuAction,
    OptionsMenu, OptionsMenuAction,
    PauseMenu, PauseMenuAction,
};

use crate::audio_assets::AudioCategory;
use crate::audio_integration::{AudioIntegration, SoundEvent};
use crate::autosave::{AutoSaveConfig, AutoSaveManager};
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
use crate::save_manager::{SaveFileBuilder, SaveManager};
use crate::timing::{ChunkMetrics, FpsCounter, FrameTiming, NpcMetrics};
use crate::weapon_loader::WeaponLoader;
use crate::world::TerrainGenerationService;

/// Application mode (menu/playing/paused).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum AppMode {
    /// Normal gameplay
    Playing,
    /// Game is paused
    Paused,
    /// In main menu
    #[default]
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

    // === Combat ===
    /// Weapon loader for loading weapon definitions
    weapon_loader: WeaponLoader,
    /// Combat event handler
    combat_events: CombatEventHandler,
    /// Combat persistence (health, stats, equipment)
    combat_persistence: CombatPersistence,
    /// Combat profiler for performance tracking
    combat_profiler: CombatProfiler,
    /// Whether attack input is held (for charge attacks)
    attack_held: bool,
    /// Time attack has been held
    attack_hold_time: f32,

    // === Save System ===
    /// Save file manager
    save_manager: SaveManager,
    /// Auto-save manager
    autosave_manager: AutoSaveManager,
    /// Current save slot name
    current_save_slot: Option<String>,

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
    /// Whether map is open
    show_map: bool,
    /// Currently selected hotbar slot
    hotbar_slot: u8,

    // === Menu State ===
    /// Main menu UI
    main_menu: MainMenu,
    /// Pause menu UI
    pause_menu: PauseMenu,
    /// Options menu UI
    options_menu: OptionsMenu,
    /// Whether showing controls help overlay
    show_controls_help: bool,

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

        // Initialize combat system
        let mut weapon_loader = WeaponLoader::with_default_path();
        if let Err(e) = weapon_loader.load_all() {
            warn!("Failed to load weapons: {}", e);
        } else {
            info!("Loaded {} weapons", weapon_loader.registry().len());
        }
        let combat_events = CombatEventHandler::new();
        let combat_persistence = CombatPersistence::new();
        let combat_profiler = CombatProfiler::new();

        // Initialize save system
        let save_manager = SaveManager::new("saves");
        let autosave_config = AutoSaveConfig::default();
        let autosave_manager = AutoSaveManager::new(autosave_config);
        info!("Save system initialized");

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

            weapon_loader,
            combat_events,
            combat_persistence,
            combat_profiler,
            attack_held: false,
            attack_hold_time: 0.0,

            save_manager,
            autosave_manager,
            current_save_slot: None,

            gameplay,
            camera,
            app_mode: AppMode::default(),
            show_inventory: false,
            show_map: false,
            hotbar_slot: 0,

            main_menu: MainMenu::with_defaults(),
            pause_menu: PauseMenu::with_defaults(),
            options_menu: OptionsMenu::with_defaults(),
            show_controls_help: false,

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

        // Handle inventory toggle (Tab or I)
        if self.input.is_key_just_pressed(KeyCode::Tab) || self.input.is_key_just_pressed(KeyCode::I) {
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
                if self.show_crafting {
                    "OPEN"
                } else {
                    "CLOSED"
                }
            );
        }

        // Handle map toggle (M key)
        if self.input.is_key_just_pressed(KeyCode::M) {
            self.show_map = !self.show_map;
            info!(
                "Map: {}",
                if self.show_map {
                    "OPEN"
                } else {
                    "CLOSED"
                }
            );
        }

        // Handle controls help toggle (F1 key)
        if self.input.is_key_just_pressed(KeyCode::F1) {
            self.show_controls_help = !self.show_controls_help;
            info!(
                "Controls help: {}",
                if self.show_controls_help {
                    "SHOWN"
                } else {
                    "HIDDEN"
                }
            );
        }

        // Handle pause/menu toggle (Escape)
        if self.input.pause_pressed() {
            match self.app_mode {
                AppMode::Playing => {
                    info!("Game paused");
                    self.app_mode = AppMode::Paused;
                    self.pause_menu.show();
                },
                AppMode::Paused => {
                    // Toggle pause menu or close to resume
                    self.pause_menu.toggle();
                    if !self.pause_menu.is_visible() {
                        info!("Game resumed");
                        self.app_mode = AppMode::Playing;
                    }
                },
                AppMode::Menu => {
                    // ESC in main menu - do nothing (or could quit confirmation)
                },
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

        // Update combat system (process events, update cooldowns)
        self.update_combat(dt);

        // Update save system (auto-save timer, check triggers)
        self.update_save_system(dt);

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

            // Update streaming terrain with player position (player-centered streaming)
            if renderer.is_streaming_terrain_enabled() {
                let start = Instant::now();
                renderer.update_player_position_streaming(player_pos.x, player_pos.y);
                self.chunk_metrics.record_load_time(start.elapsed());

                let start = Instant::now();
                renderer.step_streaming_terrain();
                self.chunk_metrics.record_sim_time(start.elapsed());

                // Update chunk metrics from streaming terrain
                if let Some(stats) = renderer.streaming_stats() {
                    self.chunk_metrics
                        .set_chunk_count(stats.simulating_count as u32);
                }
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

        // Day/night ambient layers (use actual filenames without extension)
        if is_night {
            // Use crickets for night, owls would also work
            self.audio
                .fade_in_ambient("night", "crickets", 0.5, 2.0);
            self.audio.fade_out_ambient("day", 2.0);
        } else if is_dawn_dusk {
            // Dawn/dusk transition - both layers at reduced volume
            self.audio.fade_in_ambient("day", "birds", 0.3, 2.0);
            self.audio
                .fade_in_ambient("night", "crickets", 0.2, 2.0);
        } else {
            self.audio.fade_in_ambient("day", "birds", 0.5, 2.0);
            self.audio.fade_out_ambient("night", 2.0);
        }

        // Weather-based ambient
        if self.environment.weather.is_raining() {
            let rain_volume = self.environment.weather.rain_intensity();
            self.audio
                .fade_in_ambient("rain", "rain", rain_volume, 1.0);
        } else {
            self.audio.fade_out_ambient("rain", 2.0);
        }

        if self.environment.weather.is_stormy() {
            self.audio
                .fade_in_ambient("thunder", "thunder", 0.7, 0.5);
        } else {
            self.audio.fade_out_ambient("thunder", 1.0);
        }

        // Wind based on weather intensity
        let wind_volume = self.environment.weather.wind_strength() * 0.4;
        if wind_volume > 0.1 {
            self.audio
                .fade_in_ambient("wind", "wind_light", wind_volume, 1.5);
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
                self.crafting_profiler.record_craft(recipe_id.raw(), &recipe.category);
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

    /// Updates combat system for the frame.
    fn update_combat(&mut self, dt: f32) {
        // Check for weapon hot-reload in debug mode
        if self.weapon_loader.check_hot_reload().unwrap_or(false) {
            info!("Weapons hot-reloaded");
        }

        // Handle attack input
        let attack_pressed = self.input.is_key_just_pressed(genesis_gameplay::input::KeyCode::Space);
        let attack_held_input = self.input.is_key_pressed(genesis_gameplay::input::KeyCode::Space);

        // Track attack hold time for charge attacks
        if attack_held_input {
            if !self.attack_held {
                // Just started holding
                self.attack_held = true;
                self.attack_hold_time = 0.0;
            } else {
                self.attack_hold_time += dt;
            }
        } else if self.attack_held {
            // Just released - could trigger charged attack based on hold_time
            self.attack_held = false;
            // Reset hold time
            self.attack_hold_time = 0.0;
        }

        // Process attack if button was just pressed
        if attack_pressed {
            // Get player position and direction for attack
            let player_pos = self.gameplay.player.position();
            let player_vel = self.gameplay.player.velocity();

            // Determine attack direction from movement or facing
            let direction = if player_vel.x.abs() > 0.1 || player_vel.y.abs() > 0.1 {
                let len = (player_vel.x * player_vel.x + player_vel.y * player_vel.y).sqrt();
                (player_vel.x / len, player_vel.y / len)
            } else {
                (1.0, 0.0) // Default facing right
            };

            // Check if player can attack (has stamina, no cooldown)
            let stamina_cost = if let Some(weapon_id) = self.combat_persistence.player().equipped_weapon {
                self.weapon_loader.registry().get(weapon_id)
                    .map(|w| w.stamina_cost)
                    .unwrap_or(10.0)
            } else {
                5.0 // Unarmed attack cost
            };

            if self.combat_persistence.player().can_attack(stamina_cost) {
                use crate::combat_events::{AttackCategory, AttackTarget, CombatEventHandler};

                // Determine attack type based on equipped weapon
                let attack_type = if let Some(weapon_id) = self.combat_persistence.player().equipped_weapon {
                    self.weapon_loader.registry().get(weapon_id)
                        .map(|w| match w.category {
                            crate::weapon_loader::WeaponCategory::Sword |
                            crate::weapon_loader::WeaponCategory::Greatsword |
                            crate::weapon_loader::WeaponCategory::Axe |
                            crate::weapon_loader::WeaponCategory::Greataxe => AttackCategory::MeleeSwing,
                            crate::weapon_loader::WeaponCategory::Dagger |
                            crate::weapon_loader::WeaponCategory::Spear => AttackCategory::MeleeThrust,
                            crate::weapon_loader::WeaponCategory::Bow |
                            crate::weapon_loader::WeaponCategory::Crossbow => AttackCategory::RangedBow,
                            crate::weapon_loader::WeaponCategory::Gun => AttackCategory::RangedGun,
                            crate::weapon_loader::WeaponCategory::Staff => AttackCategory::MagicSpell,
                            _ => AttackCategory::MeleeSwing,
                        })
                        .unwrap_or(AttackCategory::Unarmed)
                } else {
                    AttackCategory::Unarmed
                };

                // Create attack event
                let event = CombatEventHandler::make_attack_event(
                    genesis_common::EntityId::from_raw(1), // Player entity ID
                    AttackTarget::Direction(direction.0, direction.1),
                    attack_type,
                    (player_pos.x, player_pos.y),
                    direction,
                );
                self.combat_events.queue_event(event);

                // Consume stamina
                let current_stamina = self.combat_persistence.player().stamina;
                self.combat_persistence.player_mut().set_stamina(current_stamina - stamina_cost);

                // Set attack cooldown based on weapon
                let cooldown = if let Some(weapon_id) = self.combat_persistence.player().equipped_weapon {
                    self.weapon_loader.registry().get(weapon_id)
                        .map(|w| 1.0 / w.attack_speed)
                        .unwrap_or(0.5)
                } else {
                    0.3 // Unarmed attack cooldown
                };
                self.combat_persistence.player_mut().attack_cooldown = cooldown;

                debug!("Player attacked with {:?}", attack_type);
            }
        }

        // Start profiling event processing
        self.combat_profiler.start_event_processing();

        // Process pending combat events
        let result = self.combat_events.process_events(Some(&mut self.audio));

        // End profiling
        self.combat_profiler.end_event_processing(
            result.attacks.len() + result.hits.len() + result.deaths.len()
        );

        // Handle deaths
        for death in &result.deaths {
            if death.entity.raw() == 1 {
                // Player died
                self.combat_persistence.record_death(5.0); // 5 second respawn
                info!("Player died!");
            } else {
                // Enemy died - record kill
                self.combat_persistence.record_kill("enemy", 0, death.experience.into());
            }
        }

        // Update combat persistence (cooldowns, status effects)
        self.combat_persistence.update(dt);

        // Regenerate stamina when not attacking
        if !self.attack_held && self.combat_persistence.player().attack_cooldown <= 0.0 {
            let current_stamina = self.combat_persistence.player().stamina;
            let max_stamina = self.combat_persistence.player().max_stamina;
            let regen_rate = 20.0; // Stamina per second
            let new_stamina = (current_stamina + regen_rate * dt).min(max_stamina);
            self.combat_persistence.player_mut().stamina = new_stamina;
        }

        // Update combat memory usage for profiling
        self.combat_profiler.update_memory(
            1, // Player entity
            0, // Active projectiles (would come from a projectile system)
            self.combat_persistence.player().status_effects.len(),
            self.weapon_loader.registry().len(),
        );

        // End combat profiler frame
        self.combat_profiler.end_frame();
    }

    /// Updates save system for the frame.
    fn update_save_system(&mut self, dt: f32) {
        // Update auto-save timer
        self.autosave_manager.update(dt as f64);

        // Check for quicksave input (Ctrl+S - using S key here, real implementation would check modifiers)
        // For now use Num5 as quicksave key
        if self.input.is_key_just_pressed(genesis_gameplay::input::KeyCode::Num5) {
            self.quicksave();
        }

        // Check for quickload input (Num9 as quickload key)
        if self.input.is_key_just_pressed(genesis_gameplay::input::KeyCode::Num9) {
            self.quickload();
        }

        // Process auto-save if conditions are met
        if self.app_mode == AppMode::Playing {
            let save_data = self.build_save_data("autosave");
            if self.autosave_manager.check_and_save(&mut self.save_manager, &save_data) {
                debug!("Auto-save completed");
            }
        }

        // Check for auto-save pause conditions
        // (Combat pause is handled by combat_events integration)
    }

    /// Builds save file data from current game state.
    fn build_save_data(&self, slot_name: &str) -> crate::save_manager::SaveFileData {
        let player_pos = self.gameplay.player.position();

        SaveFileBuilder::new(slot_name)
            .display_name(format!("Slot {}", slot_name))
            .player_position(player_pos.x, player_pos.y)
            .world_seed(self.terrain_service.seed())
            .game_time(self.gameplay.game_time() as f64)
            .playtime(self.gameplay.game_time() as f64)
            .player_level(self.combat_persistence.data().combat_level)
            .location("Unknown".to_string())
            .crafting(self.crafting_persistence.save_data().clone())
            .combat(self.combat_persistence.save_data())
            .build()
    }

    /// Performs a quicksave operation.
    fn quicksave(&mut self) {
        info!("Quicksave requested");
        let save_data = self.build_save_data("quicksave");

        match self.save_manager.quicksave(&save_data) {
            Ok(()) => {
                self.current_save_slot = Some("quicksave".to_string());
                info!("Quicksave successful");
            }
            Err(e) => {
                warn!("Quicksave failed: {}", e);
            }
        }
    }

    /// Performs a quickload operation.
    fn quickload(&mut self) {
        info!("Quickload requested");

        match self.save_manager.quickload() {
            Ok(save_data) => {
                self.apply_save_data(&save_data);
                self.current_save_slot = Some("quicksave".to_string());
                info!("Quickload successful");
            }
            Err(e) => {
                warn!("Quickload failed: {}", e);
            }
        }
    }

    /// Saves the game to a specific slot.
    #[allow(dead_code)]
    fn save_game(&mut self, slot_name: &str) -> Result<()> {
        info!("Saving game to slot: {}", slot_name);
        let save_data = self.build_save_data(slot_name);

        self.save_manager.save(slot_name, &save_data)
            .map_err(|e| anyhow::anyhow!("Save failed: {}", e))?;

        self.current_save_slot = Some(slot_name.to_string());
        info!("Game saved to slot: {}", slot_name);
        Ok(())
    }

    /// Loads the game from a specific slot.
    #[allow(dead_code)]
    fn load_game(&mut self, slot_name: &str) -> Result<()> {
        info!("Loading game from slot: {}", slot_name);

        let save_data = self.save_manager.load(slot_name)
            .map_err(|e| anyhow::anyhow!("Load failed: {}", e))?;

        self.apply_save_data(&save_data);
        self.current_save_slot = Some(slot_name.to_string());
        info!("Game loaded from slot: {}", slot_name);
        Ok(())
    }

    /// Applies loaded save data to the game state.
    fn apply_save_data(&mut self, save_data: &crate::save_manager::SaveFileData) {
        // Restore player position using Vec2
        let pos = genesis_gameplay::Vec2::new(
            save_data.player_position.0,
            save_data.player_position.1,
        );
        self.gameplay.player.set_position(pos);

        // Restore crafting state
        self.crafting_persistence.load_data(&save_data.crafting);

        // Restore combat state
        self.combat_persistence.load_data(&save_data.combat);

        // Update camera to follow restored player position
        self.camera.center_on(save_data.player_position.0, save_data.player_position.1);

        // Reset auto-save timer after load
        self.autosave_manager.reset_timer();

        debug!("Save data applied: position=({}, {}), playtime={}",
            save_data.player_position.0,
            save_data.player_position.1,
            save_data.metadata.playtime_seconds
        );
    }

    /// Called when combat starts (pauses auto-save).
    #[allow(dead_code)]
    fn on_combat_start(&mut self) {
        self.autosave_manager.on_combat_start();
    }

    /// Called when combat ends (resumes auto-save).
    #[allow(dead_code)]
    fn on_combat_end(&mut self) {
        self.autosave_manager.on_combat_end();
    }

    /// Called when entering a new area (triggers auto-save if configured).
    #[allow(dead_code)]
    fn on_area_transition(&mut self) {
        self.autosave_manager.on_area_transition();
    }

    /// Render the frame.
    fn render(&mut self) {
        // Extract all data needed for UI before borrowing renderer
        let show_debug = self.show_debug;
        let show_inventory = self.show_inventory;
        let show_crafting = self.show_crafting;
        let show_map = self.show_map;
        let hotbar_slot = self.hotbar_slot;
        let app_mode = self.app_mode;
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

        // Get mutable refs to menus for the closure
        let main_menu = &mut self.main_menu;
        let pause_menu = &mut self.pause_menu;
        let options_menu = &mut self.options_menu;
        let show_controls_help = self.show_controls_help;

        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            // Use render_with_ui to draw world + egui overlay
            let result = renderer.render_with_ui(window, &self.camera, |ctx| {
                // Render options menu on top if visible (works from any mode)
                if options_menu.is_visible() {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)))
                        .show(ctx, |ui| {
                            options_menu.render(ui);
                        });
                    return; // Don't render underlying menu when options is open
                }

                // Render controls help overlay if visible
                if show_controls_help {
                    render_controls_help(ctx);
                }

                match app_mode {
                    AppMode::Menu => {
                        // Render main menu (full screen)
                        egui::CentralPanel::default()
                            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 30)))
                            .show(ctx, |ui| {
                                main_menu.render(ui);
                            });
                    }
                    AppMode::Paused => {
                        // Render game world behind with overlay
                        // Show HUD elements
                        render_hud(ctx, hotbar_slot, &environment_time, &environment_weather);

                        // Render pause menu overlay
                        egui::CentralPanel::default()
                            .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
                            .show(ctx, |ui| {
                                pause_menu.render(ui);
                            });

                        if show_debug {
                            render_debug_overlay(ctx, &debug_data);
                        }
                    }
                    AppMode::Playing => {
                        // Normal gameplay UI
                        if show_debug {
                            render_debug_overlay(ctx, &debug_data);
                        }

                        if show_inventory {
                            render_inventory(ctx, hotbar_slot);
                        }

                        if show_crafting {
                            render_crafting(ctx);
                        }

                        if show_map {
                            render_map(ctx);
                        }

                        // Always show HUD elements (hotbar, vitals, minimap)
                        render_hud(ctx, hotbar_slot, &environment_time, &environment_weather);

                        // Show interaction prompt if near an NPC
                        render_interaction_prompt(ctx, &interaction_data);
                    }
                }
            });

            if let Err(e) = result {
                warn!("Render error: {e}");
            }
        }

        // Process menu actions after rendering
        self.process_menu_actions();
    }

    /// Process any pending menu actions
    fn process_menu_actions(&mut self) {
        // Process main menu actions
        let actions = self.main_menu.drain_actions();
        if !actions.is_empty() {
            debug!("Processing {} main menu actions", actions.len());
        }
        for action in actions {
            info!("Main menu action: {:?}", action);
            match action {
                MainMenuAction::NewGame => {
                    info!("Starting new game...");
                    self.app_mode = AppMode::Playing;
                    self.main_menu.hide();
                }
                MainMenuAction::Continue => {
                    info!("Continuing game...");
                    self.quickload();
                    self.app_mode = AppMode::Playing;
                    self.main_menu.hide();
                }
                MainMenuAction::OpenLoadMenu => {
                    info!("Opening load menu...");
                    // TODO: Show load game menu
                }
                MainMenuAction::OpenOptions => {
                    info!("Opening options menu...");
                    self.options_menu.show();
                }
                MainMenuAction::Exit => {
                    info!("Exit requested from menu");
                    // Exit is handled by the window close event
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        // Process pause menu actions
        for action in self.pause_menu.drain_actions() {
            match action {
                PauseMenuAction::Resume => {
                    info!("Resuming game...");
                    self.app_mode = AppMode::Playing;
                    self.pause_menu.hide();
                }
                PauseMenuAction::OpenSaveMenu => {
                    info!("Opening save menu...");
                    // TODO: Show save menu
                }
                PauseMenuAction::OpenLoadMenu => {
                    info!("Opening load menu...");
                    // TODO: Show load menu
                }
                PauseMenuAction::OpenOptions => {
                    info!("Opening options menu...");
                    self.options_menu.show();
                }
                PauseMenuAction::QuitToMenu => {
                    info!("Quitting to main menu...");
                    self.app_mode = AppMode::Menu;
                    self.pause_menu.hide();
                    self.main_menu.show();
                }
                PauseMenuAction::QuitToDesktop => {
                    info!("Quitting to desktop...");
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        // Process options menu actions
        for action in self.options_menu.drain_actions() {
            match action {
                OptionsMenuAction::Apply => {
                    info!("Applying options...");
                    // TODO: Apply settings to engine config
                    self.options_menu.hide();
                }
                OptionsMenuAction::Cancel => {
                    info!("Cancelling options...");
                    self.options_menu.hide();
                }
                OptionsMenuAction::ResetToDefaults => {
                    info!("Resetting options to defaults...");
                }
                _ => {}
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
            ui.label("Inventory (Tab/I to close)");
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

/// Renders the crafting panel.
fn render_crafting(ctx: &egui::Context) {
    egui::Window::new("Crafting")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .min_width(400.0)
        .show(ctx, |ui| {
            ui.label("Crafting (C to close)");
            ui.separator();

            ui.horizontal(|ui| {
                // Left side: Recipe categories
                ui.vertical(|ui| {
                    ui.set_min_width(120.0);
                    ui.heading("Categories");
                    ui.separator();
                    for category in &["All", "Tools", "Weapons", "Armor", "Materials", "Food"] {
                        if ui.selectable_label(false, *category).clicked() {
                            // Category selection would be handled here
                        }
                    }
                });

                ui.separator();

                // Right side: Recipe list
                ui.vertical(|ui| {
                    ui.set_min_width(250.0);
                    ui.heading("Recipes");
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            ui.label("No recipes available yet.");
                            ui.label("");
                            ui.label("Gather materials and discover");
                            ui.label("new crafting recipes!");
                        });
                });
            });
        });
}

/// Renders the map panel.
fn render_map(ctx: &egui::Context) {
    egui::Window::new("World Map")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(true)
        .collapsible(false)
        .default_size([500.0, 400.0])
        .show(ctx, |ui| {
            ui.label("Map (M to close)");
            ui.separator();

            // Map display area
            let available = ui.available_size();
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(available.x.min(480.0), available.y.min(360.0)),
                egui::Sense::drag(),
            );

            // Draw map background
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(4.0),
                egui::Color32::from_rgb(30, 40, 30),
            );

            // Draw grid lines
            let grid_spacing = 40.0;
            for i in 0..=(rect.width() / grid_spacing) as i32 {
                let x = rect.left() + i as f32 * grid_spacing;
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 50)),
                );
            }
            for i in 0..=(rect.height() / grid_spacing) as i32 {
                let y = rect.top() + i as f32 * grid_spacing;
                ui.painter().line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 50)),
                );
            }

            // Draw player marker at center
            let center = rect.center();
            ui.painter().circle_filled(center, 6.0, egui::Color32::from_rgb(100, 200, 100));
            ui.painter().circle_stroke(center, 6.0, egui::Stroke::new(2.0, egui::Color32::WHITE));

            // Legend
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("🟢 You");
                ui.separator();
                ui.label("Drag to pan • Scroll to zoom");
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

                // Get actual window size and scale factor
                let actual_size = window.inner_size();
                let scale_factor = window.scale_factor() as f32;
                info!("Window actual size: {}x{}, scale factor: {}", actual_size.width, actual_size.height, scale_factor);

                // Initialize renderer
                match pollster::block_on(Renderer::new(&window)) {
                    Ok(mut renderer) => {
                        info!("Renderer initialized");
                        // Trigger initial resize to ensure surface is properly configured
                        renderer.resize(actual_size);
                        // Ensure egui scale factor matches window
                        renderer.set_scale_factor(scale_factor);
                        // Enable streaming terrain with world seed
                        renderer.enable_streaming_terrain(self.terrain_service.seed());
                        self.renderer = Some(renderer);
                    },
                    Err(e) => {
                        warn!("Failed to initialize renderer: {e}");
                    },
                }

                // Update camera viewport to match actual window size
                self.camera.set_viewport(actual_size.width, actual_size.height);
                self.config.window_width = actual_size.width;
                self.config.window_height = actual_size.height;

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
        // Let egui process the event first for UI interaction
        let egui_consumed = if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            renderer.handle_event(window, &event)
        } else {
            false
        };

        // Let input handler process the event (for game controls)
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
                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    renderer.resize(new_size);
                    // Update egui scale factor to match window
                    renderer.set_scale_factor(window.scale_factor() as f32);
                }
                // Update config and camera viewport
                self.config.window_width = new_size.width;
                self.config.window_height = new_size.height;
                self.camera.set_viewport(new_size.width, new_size.height);
            },
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.set_scale_factor(scale_factor as f32);
                }
            },
            WindowEvent::RedrawRequested => {
                self.update_and_render();

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            _ => {
                if !handled && !egui_consumed {
                    // Event wasn't handled by input, egui, or above
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

/// Renders the controls help overlay.
fn render_controls_help(ctx: &egui::Context) {
    egui::Window::new("Controls")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 240)),
        )
        .show(ctx, |ui| {
            ui.heading("Keyboard Controls");
            ui.add_space(10.0);

            egui::Grid::new("controls_grid")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    // Movement
                    ui.strong("Movement");
                    ui.label("");
                    ui.end_row();

                    ui.label("W / ↑");
                    ui.label("Move Up");
                    ui.end_row();

                    ui.label("S / ↓");
                    ui.label("Move Down");
                    ui.end_row();

                    ui.label("A / ←");
                    ui.label("Move Left");
                    ui.end_row();

                    ui.label("D / →");
                    ui.label("Move Right");
                    ui.end_row();

                    ui.label("Shift");
                    ui.label("Sprint");
                    ui.end_row();

                    ui.label("");
                    ui.label("");
                    ui.end_row();

                    // Interaction
                    ui.strong("Interaction");
                    ui.label("");
                    ui.end_row();

                    ui.label("E");
                    ui.label("Interact / Use");
                    ui.end_row();

                    ui.label("Left Click");
                    ui.label("Primary Action / Attack");
                    ui.end_row();

                    ui.label("Right Click");
                    ui.label("Secondary Action");
                    ui.end_row();

                    ui.label("");
                    ui.label("");
                    ui.end_row();

                    // UI Panels
                    ui.strong("UI Panels");
                    ui.label("");
                    ui.end_row();

                    ui.label("Tab / I");
                    ui.label("Toggle Inventory");
                    ui.end_row();

                    ui.label("C");
                    ui.label("Toggle Crafting");
                    ui.end_row();

                    ui.label("M");
                    ui.label("Toggle Map");
                    ui.end_row();

                    ui.label("1-0");
                    ui.label("Select Hotbar Slot");
                    ui.end_row();

                    ui.label("");
                    ui.label("");
                    ui.end_row();

                    // System
                    ui.strong("System");
                    ui.label("");
                    ui.end_row();

                    ui.label("Escape");
                    ui.label("Pause Menu");
                    ui.end_row();

                    ui.label("F1");
                    ui.label("Toggle This Help");
                    ui.end_row();

                    ui.label("F3");
                    ui.label("Toggle Debug Overlay");
                    ui.end_row();

                    ui.label("F5");
                    ui.label("Quick Save");
                    ui.end_row();

                    ui.label("F9");
                    ui.label("Quick Load");
                    ui.end_row();
                });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(5.0);
            ui.centered_and_justified(|ui| {
                ui.label("Press F1 to close");
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_mode_default_is_menu() {
        let mode = AppMode::default();
        assert_eq!(mode, AppMode::Menu, "App should start in Menu mode");
    }

    #[test]
    fn test_app_mode_variants() {
        assert_ne!(AppMode::Menu, AppMode::Playing);
        assert_ne!(AppMode::Menu, AppMode::Paused);
        assert_ne!(AppMode::Playing, AppMode::Paused);
    }

    #[test]
    fn test_format_cells_with_commas() {
        assert_eq!(format_cells(0), "0");
        assert_eq!(format_cells(123), "123");
        assert_eq!(format_cells(1234), "1,234");
        assert_eq!(format_cells(1_000_000), "1,000,000");
        assert_eq!(format_cells(1_234_567_890), "1,234,567,890");
    }

    #[test]
    fn test_main_menu_new_game_action() {
        use genesis_tools::ui::{MainMenu, MainMenuAction, MainMenuButton};

        let mut menu = MainMenu::with_defaults();
        assert!(menu.is_visible(), "Menu should start visible");

        // Simulate clicking new game
        menu.click_button(MainMenuButton::NewGame);

        let actions = menu.drain_actions();
        assert!(!actions.is_empty(), "Should have pending action");
        assert!(actions.contains(&MainMenuAction::NewGame), "Should have NewGame action");
    }

    #[test]
    fn test_main_menu_exit_action() {
        use genesis_tools::ui::{MainMenu, MainMenuAction, MainMenuButton};

        let mut menu = MainMenu::with_defaults();

        // Simulate clicking exit
        menu.click_button(MainMenuButton::Exit);

        let actions = menu.drain_actions();
        assert!(!actions.is_empty(), "Should have pending action");
        assert!(actions.contains(&MainMenuAction::Exit), "Should have Exit action");
    }

    #[test]
    fn test_main_menu_visibility() {
        use genesis_tools::ui::MainMenu;

        let mut menu = MainMenu::with_defaults();
        assert!(menu.is_visible(), "Menu should start visible");

        menu.hide();
        assert!(!menu.is_visible(), "Menu should be hidden");

        menu.show();
        assert!(menu.is_visible(), "Menu should be visible again");
    }

    #[test]
    fn test_pause_menu_visibility() {
        use genesis_tools::ui::PauseMenu;

        let mut menu = PauseMenu::with_defaults();
        assert!(!menu.is_visible(), "Pause menu should start hidden");

        menu.show();
        assert!(menu.is_visible(), "Pause menu should be visible after show()");

        menu.hide();
        assert!(!menu.is_visible(), "Pause menu should be hidden after hide()");
    }

    #[test]
    fn test_pause_menu_toggle() {
        use genesis_tools::ui::PauseMenu;

        let mut menu = PauseMenu::with_defaults();
        assert!(!menu.is_visible(), "Should start hidden");

        menu.toggle();
        assert!(menu.is_visible(), "Should be visible after first toggle");

        menu.toggle();
        // After toggle while visible, it may close or stay (depends on implementation)
        // Just ensure toggle works without panic
    }
}
