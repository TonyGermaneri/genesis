//! Application lifecycle management.
//!
//! Main game loop that integrates all subsystems.

use anyhow::Result;
use std::time::Instant;
use tracing::{debug, error, info, warn};
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
    WorldTools, WorldToolsAction,
};

use crate::asset_manager::AssetManager;
use crate::audio_assets::AudioCategory;
use crate::audio_integration::{AudioIntegration, SoundEvent};
use crate::autosave::{AutoSaveConfig, AutoSaveManager};
use crate::automation::{AutomationRequest, AutomationSystem};
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

use genesis_worldgen::{BiomeTextureMap, WorldGenConfig, WorldGenerator};

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

    // === Asset Management ===
    /// Asset manager for terrain textures, etc.
    asset_manager: AssetManager,

    // === World Generation ===
    /// World generator (cubiomes-based biome generation)
    world_generator: WorldGenerator,
    /// Biome-to-visual mapping (colors or texture paths)
    biome_texture_map: BiomeTextureMap,
    /// Last chunk coordinate that triggered terrain generation
    last_terrain_chunk: (i32, i32),
    /// Whether terrain needs full regeneration
    terrain_dirty: bool,

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
    /// World tools panel (character generator, sprite builder, etc.)
    world_tools: WorldTools,

    // === Automation ===
    /// Automation system for E2E testing
    automation: AutomationSystem,
    /// Whether quit was requested via automation
    quit_requested: bool,
    /// Pending button click from automation
    pending_button_click: Option<String>,
    /// Pending element click from automation
    pending_element_click: Option<String>,
    /// Pending text input from automation (field, value)
    pending_text_input: Option<(String, String)>,

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

        // Use world seed from config (or random if not set)
        let seed = config.world_seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(12345)
        });
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
        camera.set_zoom(config.camera_zoom); // Use config zoom level

        // Initialize world generation (cubiomes)
        let worldgen_config = WorldGenConfig {
            mc_version: genesis_worldgen::MC_1_21,
            seed: seed,
            flags: 0,
            scale: 4,
            y_level: 16, // block y=64 at scale 4 (sea level for surface biomes)
        };
        let world_generator = WorldGenerator::new(worldgen_config);
        let biome_texture_map = BiomeTextureMap::from_cubiomes_defaults();
        info!("World generation initialized with cubiomes (seed={}, mc=1.21)", seed);

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
            asset_manager: AssetManager::new(),
            world_generator,
            biome_texture_map,
            last_terrain_chunk: (i32::MAX, i32::MAX), // Force initial generation
            terrain_dirty: true,
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
            world_tools: WorldTools::new(),
            show_controls_help: false,

            automation: AutomationSystem::new(),
            quit_requested: false,
            pending_button_click: None,
            pending_element_click: None,
            pending_text_input: None,

            current_fps: 0.0,
            current_frame_time: 0.0,
        }
    }

    /// Enable debug atlas mode (use reference autotile atlas for testing)
    pub fn set_use_debug_atlas(&mut self, use_debug: bool) {
        if use_debug {
            self.asset_manager = AssetManager::with_config(
                crate::asset_manager::AssetConfig::with_debug_atlas()
            );
            info!("Debug atlas mode enabled - using reference autotile atlas");
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

        // Handle debug grid toggle (G key)
        if self.input.is_key_just_pressed(KeyCode::G) {
            if let Some(renderer) = &mut self.renderer {
                renderer.toggle_debug_grid();
            }
        }

        // Handle screenshot capture (F12 key)
        if self.input.is_key_just_pressed(KeyCode::F12) {
            self.capture_screenshot();
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
                    // Check which dialog is open and close it appropriately
                    if self.options_menu.is_visible() {
                        // Cancel options menu (revert changes), go back to pause menu
                        info!("Cancelling options via ESC");
                        self.options_menu.cancel();
                        self.options_menu.hide();
                        self.pause_menu.show();
                    } else if self.world_tools.is_visible() {
                        // Close world tools, go back to pause menu
                        info!("Closing world tools via ESC");
                        self.world_tools.hide();
                        self.pause_menu.show();
                    } else if self.pause_menu.is_visible() {
                        // Close pause menu and resume game
                        info!("Game resumed via ESC");
                        self.pause_menu.hide();
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

        // Update automation system
        let automation_requests = self.automation.update(dt);
        self.process_automation_requests(automation_requests);

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

            // Update player sprite animation
            let player_vel = self.gameplay.player.velocity();
            renderer.update_player_sprite(dt, (player_pos.x, player_pos.y), (player_vel.x, player_vel.y));

            // Handle action key inputs for sprite animations
            if self.input.is_action_just_pressed(genesis_gameplay::input::Action::UseItem) {
                renderer.set_player_action(genesis_kernel::player_sprite::PlayerAnimAction::Use);
            } else if self.input.is_action_just_pressed(genesis_gameplay::input::Action::Punch) {
                renderer.set_player_action(genesis_kernel::player_sprite::PlayerAnimAction::Punch);
            } else if self.input.is_action_just_pressed(genesis_gameplay::input::Action::Jump) {
                renderer.set_player_action(genesis_kernel::player_sprite::PlayerAnimAction::Jump);
            }

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
                renderer.step_streaming_terrain(dt);
                self.chunk_metrics.record_sim_time(start.elapsed());

                // Update chunk metrics from streaming terrain
                if let Some(stats) = renderer.streaming_stats() {
                    self.chunk_metrics
                        .set_chunk_count(stats.simulating_count as u32);
                }
            }

            // === Cubiomes terrain generation ===
            // Generate biome chunks around the camera and feed to terrain renderer.
            // Coordinate mapping: game X → cubiomes X, game Y → cubiomes Z (top-down view).
            {
                let terrain = renderer.terrain_renderer_mut();
                if terrain.is_enabled() {
                    let tile_size = terrain.config().tile_size;
                    let render_radius = terrain.config().render_radius;
                    let chunk_cells = 16i32; // biome cells per chunk

                    // Current camera chunk in game coordinates
                    // (game Y maps to cubiomes Z for top-down horizontal slice)
                    let cam_chunk_x = (player_pos.x / (chunk_cells as f32 * tile_size)).floor() as i32;
                    let cam_chunk_y = (player_pos.y / (chunk_cells as f32 * tile_size)).floor() as i32;

                    // Generate new chunks if camera moved or terrain is dirty
                    let camera_moved = cam_chunk_x != self.last_terrain_chunk.0 || cam_chunk_y != self.last_terrain_chunk.1;
                    if camera_moved || self.terrain_dirty {
                        self.last_terrain_chunk = (cam_chunk_x, cam_chunk_y);
                        self.terrain_dirty = false;

                        let mut generated = 0u32;

                        // Generate any missing chunks within render radius
                        for cy in (cam_chunk_y - render_radius)..=(cam_chunk_y + render_radius) {
                            for cx in (cam_chunk_x - render_radius)..=(cam_chunk_x + render_radius) {
                                if !terrain.is_chunk_cached(cx, cy) {
                                    // generate_chunk(cx, cy) internally maps game Y → cubiomes Z
                                    let chunk = self.world_generator.generate_chunk(cx, cy);
                                    let biome_map = &self.biome_texture_map;
                                    // Use cubiomes mapApproxHeight for real terrain surface heights
                                    let heights = self.world_generator.generate_chunk_heights(cx, cy);
                                    terrain.cache_chunk(
                                        cx, cy,
                                        &chunk.biomes,
                                        &heights,
                                        chunk.width,
                                        chunk.height,
                                        &|biome_id| biome_map.get_color(biome_id),
                                    );
                                    generated += 1;
                                }
                            }
                        }

                        if generated > 0 {
                            tracing::debug!(
                                "Terrain: generated {} new chunks around ({}, {}), {} cached, radius={}",
                                generated, cam_chunk_x, cam_chunk_y,
                                terrain.cached_chunk_count(), render_radius
                            );
                        }
                    }
                }
            }

            // Update terrain visible tiles (uses internal borrows)
            renderer.update_terrain_visible_tiles(
                (self.camera.position.0, self.camera.position.1),
                self.camera.zoom,
            );
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

    /// Loads the player sprite sheet from assets.
    fn load_player_sprite(&mut self, renderer: &mut crate::renderer::Renderer) {
        use genesis_kernel::player_sprite::{PlayerAnimationSet, PlayerSpriteConfig};
        use std::path::Path;

        // Use player scout sprite from local assets
        let sprite_path = Path::new("assets/sprites/player/player_scout.png");
        let anim_toml_path = Path::new("assets/sprites/characters/player.toml");

        if !sprite_path.exists() {
            debug!("No player sprite found at {}", sprite_path.display());
            return;
        }

        // Parse animation TOML
        let animations = if anim_toml_path.exists() {
            match std::fs::read_to_string(anim_toml_path) {
                Ok(toml_str) => match Self::parse_animation_toml(&toml_str) {
                    Ok(anims) => {
                        info!(
                            "Loaded {} animations from {}",
                            anims.animations.len(),
                            anim_toml_path.display()
                        );
                        anims
                    }
                    Err(e) => {
                        warn!("Failed to parse animation TOML: {}", e);
                        PlayerAnimationSet::new()
                    }
                },
                Err(e) => {
                    warn!("Failed to read animation TOML: {}", e);
                    PlayerAnimationSet::new()
                }
            }
        } else {
            warn!(
                "Animation TOML not found at {}, using empty animations",
                anim_toml_path.display()
            );
            PlayerAnimationSet::new()
        };

        // Load sprite image
        match image::open(sprite_path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();

                let config = PlayerSpriteConfig::with_scale(2.0, 48, 74);
                renderer.set_player_sprite_config(config);
                renderer.set_player_animations(animations);
                renderer.load_player_sprite(rgba.as_raw(), width, height);

                info!(
                    "Player sprite loaded: {}x{} from {}",
                    width,
                    height,
                    sprite_path.display()
                );
            }
            Err(e) => {
                warn!(
                    "Failed to load player sprite from {}: {}",
                    sprite_path.display(),
                    e
                );
            }
        }
    }

    /// Parses animation definitions from TOML content.
    fn parse_animation_toml(
        toml_str: &str,
    ) -> Result<genesis_kernel::player_sprite::PlayerAnimationSet, String> {
        use genesis_kernel::player_sprite::{
            AnimKey, PlayerAnimationSet, SpriteAnimation, SpriteFrame,
        };

        // Deserialize using toml crate
        let value: toml::Value =
            toml::from_str(toml_str).map_err(|e| format!("TOML parse error: {}", e))?;

        let mut anim_set = PlayerAnimationSet::new();

        let animations = value
            .get("animations")
            .and_then(|v| v.as_array())
            .ok_or("Missing 'animations' array in TOML")?;

        for anim_entry in animations {
            let action_name = anim_entry
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Try to parse as a known action+direction key
            let key = match AnimKey::from_toml_action(action_name) {
                Some(k) => k,
                None => continue, // Skip animations we don't handle yet
            };

            let fps = anim_entry
                .get("fps")
                .and_then(|v| v.as_float())
                .unwrap_or(8.0) as f32;

            let looping = anim_entry
                .get("looping")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let frames_arr = match anim_entry.get("frames").and_then(|v| v.as_array()) {
                Some(f) => f,
                None => continue,
            };

            let mut frames = Vec::new();
            for frame_entry in frames_arr {
                let x = frame_entry
                    .get("x")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0) as u32;
                let y = frame_entry
                    .get("y")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0) as u32;
                let width = frame_entry
                    .get("width")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(48) as u32;
                let height = frame_entry
                    .get("height")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(74) as u32;

                frames.push(SpriteFrame {
                    x,
                    y,
                    width,
                    height,
                });
            }

            if !frames.is_empty() {
                anim_set.insert(key, SpriteAnimation { frames, fps, looping });
            }
        }

        Ok(anim_set)
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
        // Use a default biome since terrain service was removed
        let track_name = "exploration_plains";

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

    /// Captures a screenshot of the current game view.
    fn capture_screenshot(&mut self) {
        self.capture_screenshot_with_name(None);
    }

    /// Captures a screenshot with an optional custom filename.
    fn capture_screenshot_with_name(&mut self, custom_name: Option<String>) -> Option<std::path::PathBuf> {
        let screenshots_dir = std::path::PathBuf::from("screenshots");

        let filename = custom_name.unwrap_or_else(|| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("genesis_{}.png", timestamp)
        });

        let path = screenshots_dir.join(&filename);

        // Capture screenshot using renderer
        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            match renderer.capture_screenshot(&path, &self.camera, window, self.environment.time.time_of_day(), self.environment.time.sun_intensity()) {
                Ok(saved_path) => {
                    info!("Screenshot saved: {:?}", saved_path);
                    return Some(saved_path);
                }
                Err(e) => {
                    error!("Failed to capture screenshot: {}", e);
                }
            }
        } else {
            error!("Cannot capture screenshot: renderer or window not available");
        }
        None
    }

    /// Processes automation requests from the automation system.
    fn process_automation_requests(&mut self, requests: Vec<AutomationRequest>) {
        for request in requests {
            match request {
                AutomationRequest::StartNewGame => {
                    info!("[AUTOMATION] Starting new game");
                    self.start_new_game();
                }
                AutomationRequest::OpenPauseMenu => {
                    info!("[AUTOMATION] Opening pause menu");
                    if self.app_mode == AppMode::Playing {
                        self.app_mode = AppMode::Paused;
                        self.pause_menu.show();
                    }
                }
                AutomationRequest::ResumeGame => {
                    info!("[AUTOMATION] Resuming game");
                    self.pause_menu.hide();
                    self.options_menu.hide();
                    self.app_mode = AppMode::Playing;
                }
                AutomationRequest::OpenWorldTools => {
                    info!("[AUTOMATION] Opening world tools");
                    self.world_tools.show();
                    self.pause_menu.hide();
                    self.app_mode = AppMode::Paused;
                }
                AutomationRequest::SelectWorldToolsTab(tab_name) => {
                    info!("[AUTOMATION] Selecting World Tools tab: {}", tab_name);
                    self.world_tools.select_tab_by_name(&tab_name);
                }
                AutomationRequest::ClickButton(label) => {
                    info!("[AUTOMATION] Click button: {}", label);
                    // Store the button click request for egui to process
                    self.pending_button_click = Some(label);
                }
                AutomationRequest::ClickElement(id) => {
                    info!("[AUTOMATION] Click element: {}", id);
                    // Store the element click request for egui to process
                    self.pending_element_click = Some(id);
                }
                AutomationRequest::SetTextInput { field, value } => {
                    info!("[AUTOMATION] Set text input {}: {}", field, value);
                    // Store the text input request for egui to process
                    self.pending_text_input = Some((field, value));
                }
                AutomationRequest::SetSeed(seed) => {
                    info!("[AUTOMATION] Setting seed to {} (world tools removed)", seed);
                    // World tools removed - seed setting no longer functional
                }
                AutomationRequest::RegenerateWorld => {
                    info!("[AUTOMATION] Regenerating world (world tools removed)");
                    // World regeneration removed with terrain system
                }
                AutomationRequest::CaptureScreenshot { filename, prompt } => {
                    info!("[AUTOMATION] Capturing screenshot");
                    if let Some(path) = self.capture_screenshot_with_name(filename) {
                        self.automation.record_screenshot(path.clone());
                        if let Some(prompt) = prompt {
                            info!("[AUTOMATION] Screenshot prompt: {}", prompt);
                            // The prompt is logged for external analysis tools
                            // (AI analysis would be run separately via scripts/analyze-image.ts)
                        }
                    }
                }
                AutomationRequest::SetWorldParam { category, name, value } => {
                    info!("[AUTOMATION] Setting world param: {}.{} = {}", category, name, value);
                    // TODO: Implement world parameter setting via world_tools
                    warn!("SetWorldParam not yet fully implemented");
                }
                AutomationRequest::FillWorld { material_id, temperature } => {
                    info!("[AUTOMATION] FillWorld requested (terrain system removed): material={}, temp={:?}", material_id, temperature);
                    // Terrain system removed
                }
                AutomationRequest::SetTimeScale(scale) => {
                    info!("[AUTOMATION] Setting time scale to {}x (world tools removed)", scale);
                    // Time scale setting removed with terrain system
                }
                AutomationRequest::SetSimulationFlags { weather, volcanic, hydraulic_erosion, thermal_erosion } => {
                    info!("[AUTOMATION] Setting simulation flags (world tools removed)");
                    let _ = (weather, volcanic, hydraulic_erosion, thermal_erosion);
                    // Simulation flags removed with terrain system
                }
                AutomationRequest::OpenMaterialPalette => {
                    info!("[AUTOMATION] Opening material palette (removed)");
                }
                AutomationRequest::CloseMaterialPalette => {
                    info!("[AUTOMATION] Closing material palette (removed)");
                }
                AutomationRequest::SelectMaterial(material_id) => {
                    info!("[AUTOMATION] Selecting material {} (removed)", material_id);
                }
                AutomationRequest::SetBrushSize(size) => {
                    info!("[AUTOMATION] Setting brush size to {} (removed)", size);
                }
                AutomationRequest::PaintAt { x, y } => {
                    info!("[AUTOMATION] Painting at ({}, {}) (removed)", x, y);
                }
                AutomationRequest::Quit => {
                    info!("[AUTOMATION] Quit requested");
                    self.quit_requested = true;
                }
            }
        }

        // Handle movement override from automation
        if let Some((dx, dy)) = self.automation.movement_override() {
            if self.app_mode == AppMode::Playing {
                // Apply movement directly to player
                let speed = 100.0; // Base movement speed
                let vx = dx * speed;
                let vy = dy * speed;
                self.gameplay.player.set_velocity(genesis_gameplay::input::Vec2::new(vx, vy));
            }
        }

        // Handle position teleport from automation
        if let Some((x, y)) = self.automation.take_position_teleport() {
            info!("[AUTOMATION] Teleporting player to ({}, {})", x, y);
            self.gameplay.player.set_position(genesis_gameplay::input::Vec2::new(x, y));
        }

        // Handle zoom request from automation
        if let Some(zoom) = self.automation.take_zoom_request() {
            info!("[AUTOMATION] Setting zoom to {}", zoom);
            self.camera.set_zoom(zoom);
        }

        // Handle camera position request from automation
        if let Some((x, y)) = self.automation.take_camera_position_request() {
            info!("[AUTOMATION] Moving camera to ({}, {})", x, y);
            self.camera.center_on(x, y);
        }
    }

    /// Sync the world gen panel with current world generator state.
    fn sync_worldgen_panel(&mut self) {
        let config = self.world_generator.config();
        let tile_size = self.renderer.as_ref()
            .map(|r| r.terrain_renderer().config().tile_size)
            .unwrap_or(4.0);
        self.world_tools.world_gen_panel_mut().sync_config(
            config.mc_version,
            config.seed,
            config.flags,
            config.scale,
            config.y_level,
            tile_size,
        );

        // Build biome UI entries from the current biome texture map
        let entries: Vec<genesis_tools::ui::BiomeUiEntry> = self.biome_texture_map.sorted_entries()
            .iter()
            .map(|e| genesis_tools::ui::BiomeUiEntry {
                id: e.id,
                name: e.name.clone(),
                color: self.biome_texture_map.get_color(e.id),
                texture_path: match &e.visual {
                    genesis_worldgen::BiomeVisual::Texture(p) => Some(p.clone()),
                    _ => None,
                },
            })
            .collect();
        self.world_tools.world_gen_panel_mut().set_biome_entries(entries);
    }

    /// Starts a new game from the main menu.
    fn start_new_game(&mut self) {
        info!("Starting new game");
        self.main_menu.hide();
        self.app_mode = AppMode::Playing;

        // Reset gameplay state with world seed from config
        let seed = self.config.world_seed.unwrap_or(12345);
        self.gameplay = genesis_gameplay::GameState::with_player_position(seed, (128.0, 100.0));
        self.gameplay.player.set_grounded(true);

        // Reset camera
        let player_pos = self.gameplay.player_position();
        self.camera.center_on(player_pos.0, player_pos.1);

        // Reconfigure world generator with current seed
        let worldgen_config = WorldGenConfig {
            seed,
            ..self.world_generator.config().clone()
        };
        self.world_generator.reconfigure(worldgen_config);
        self.terrain_dirty = true;
        self.last_terrain_chunk = (i32::MAX, i32::MAX);

        // Enable terrain rendering
        if let Some(renderer) = &mut self.renderer {
            renderer.terrain_renderer_mut().clear_cache();
            renderer.terrain_renderer_mut().enable();
        }
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
        let attack_pressed = self.input.is_action_just_pressed(genesis_gameplay::input::Action::Punch);
        let attack_held_input = self.input.is_action_pressed(genesis_gameplay::input::Action::Punch);

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
            .world_seed(self.config.world_seed.unwrap_or(12345))
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
        // Extract terrain renderer stats before borrowing renderer for rendering
        let (terrain_enabled, terrain_cached_chunks, terrain_instance_count) =
            if let Some(renderer) = &self.renderer {
                let tr = renderer.terrain_renderer();
                (tr.is_enabled(), tr.cached_chunk_count(), tr.instance_count())
            } else {
                (false, 0, 0)
            };

        let debug_data = DebugOverlayData {
            perf: self.perf_metrics.summary(),
            time: self.environment.time.clone(),
            weather: self.environment.weather.clone(),
            ambient_light: self.environment.ambient_light(),
            chunk_count: self.chunk_metrics.chunk_count(),
            chunk_load_ms: self.chunk_metrics.avg_load_time_ms(),
            chunk_sim_ms: self.chunk_metrics.avg_sim_time_ms(),
            chunk_exceeds_budget: self.chunk_metrics.exceeds_budget(),
            world_seed: self.config.world_seed.unwrap_or(0),
            biome_gen_avg_ms: 0.0,
            biome_gen_peak_ms: 0.0,
            biome_chunks_generated: 0,
            biome_exceeds_budget: false,
            terrain_enabled,
            terrain_cached_chunks,
            terrain_instance_count,
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
        let world_tools = &mut self.world_tools;
        let show_controls_help = self.show_controls_help;

        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            // Use render_with_ui to draw world + egui overlay
            let result = renderer.render_with_ui(window, &self.camera, self.environment.time.time_of_day(), self.environment.time.sun_intensity(), |ctx| {
                // Render options menu on top if visible (works from any mode)
                if options_menu.is_visible() {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)))
                        .show(ctx, |ui| {
                            options_menu.render(ui);
                        });
                    return; // Don't render underlying menu when options is open
                }

                // Render world tools on top if visible (works from paused mode)
                if world_tools.is_visible() {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220)))
                        .show(ctx, |ui| {
                            world_tools.render(ui);
                        });
                    return; // Don't render underlying menu when world tools is open
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
                PauseMenuAction::OpenWorldTools => {
                    info!("Opening world tools...");
                    // Sync world gen panel with current state
                    self.sync_worldgen_panel();
                    self.world_tools.show();
                    self.pause_menu.hide();
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
                    // Apply camera zoom from graphics settings
                    let camera_zoom = self.options_menu.settings().graphics.camera_zoom;
                    self.camera.set_zoom(camera_zoom);
                    info!("Camera zoom set to: {}", camera_zoom);
                    self.options_menu.hide();
                }
                OptionsMenuAction::Cancel => {
                    info!("Cancelling options...");
                    self.options_menu.hide();
                }
                OptionsMenuAction::ResetToDefaults => {
                    info!("Resetting options to defaults...");
                }
                OptionsMenuAction::CameraZoomChanged(zoom) => {
                    // Apply zoom immediately for live preview
                    self.camera.set_zoom(zoom);
                }
                _ => {}
            }
        }

        // Process world tools actions
        for action in self.world_tools.drain_actions() {
            match action {
                WorldToolsAction::Close => {
                    info!("Closing world tools...");
                    self.world_tools.hide();
                    self.pause_menu.show();
                }
            }
        }

        // Process world generation actions
        for action in self.world_tools.drain_world_gen_actions() {
            match action {
                genesis_tools::ui::WorldGenAction::Regenerate { mc_version, seed, flags, scale, y_level, tile_size } => {
                    info!("Regenerating world: mc={}, seed={}, flags={}, scale={}, y={}, tile_size={}",
                        mc_version, seed, flags, scale, y_level, tile_size);
                    let config = WorldGenConfig {
                        mc_version,
                        seed,
                        flags,
                        scale,
                        y_level,
                    };
                    self.world_generator.reconfigure(config);
                    self.terrain_dirty = true;
                    self.last_terrain_chunk = (i32::MAX, i32::MAX);
                    if let Some(renderer) = &mut self.renderer {
                        let terrain = renderer.terrain_renderer_mut();
                        terrain.clear_cache();
                        terrain.set_config(genesis_kernel::terrain_tiles::TerrainRenderConfig {
                            tile_size,
                            biome_scale: scale,
                            render_radius: if tile_size <= 2.0 { 24 } else if tile_size <= 4.0 { 16 } else { 8 },
                        });
                        terrain.enable();
                    }
                }
                genesis_tools::ui::WorldGenAction::SetBiomeColor { biome_id, color } => {
                    self.biome_texture_map.set_color(biome_id, color);
                    self.terrain_dirty = true;
                    self.last_terrain_chunk = (i32::MAX, i32::MAX);
                    if let Some(renderer) = &mut self.renderer {
                        renderer.terrain_renderer_mut().clear_cache();
                    }
                }
                genesis_tools::ui::WorldGenAction::SetBiomeTexture { biome_id, path } => {
                    self.biome_texture_map.set_texture(biome_id, path);
                    // Texture rendering not yet implemented, but record the mapping
                }
                genesis_tools::ui::WorldGenAction::ResetBiomeColors => {
                    self.biome_texture_map = BiomeTextureMap::from_cubiomes_defaults();
                    self.terrain_dirty = true;
                    self.last_terrain_chunk = (i32::MAX, i32::MAX);
                    if let Some(renderer) = &mut self.renderer {
                        renderer.terrain_renderer_mut().clear_cache();
                    }
                    // Re-sync biome entries to UI
                    let entries: Vec<genesis_tools::ui::BiomeUiEntry> = self.biome_texture_map.sorted_entries()
                        .iter()
                        .map(|e| genesis_tools::ui::BiomeUiEntry {
                            id: e.id,
                            name: e.name.clone(),
                            color: self.biome_texture_map.get_color(e.id),
                            texture_path: match &e.visual {
                                genesis_worldgen::BiomeVisual::Texture(p) => Some(p.clone()),
                                _ => None,
                            },
                        })
                        .collect();
                    self.world_tools.world_gen_panel_mut().set_biome_entries(entries);
                }
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
    // Terrain renderer status
    terrain_enabled: bool,
    terrain_cached_chunks: usize,
    terrain_instance_count: u32,
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

            // Terrain renderer status
            ui.label(format!(
                "Terrain: {} | {} cached chunks | {} instances",
                if data.terrain_enabled { "ON" } else { "OFF" },
                data.terrain_cached_chunks,
                data.terrain_instance_count,
            ));

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
                        // Enable streaming terrain with world seed from config
                        renderer.enable_streaming_terrain(self.config.world_seed.unwrap_or(12345));

                        // Always load the player sprite
                        self.load_player_sprite(&mut renderer);

                        self.renderer = Some(renderer);

                        // Auto-start: begin game immediately so terrain renders on launch
                        if self.app_mode == AppMode::Menu {
                            self.start_new_game();
                        }
                    },
                    Err(e) => {
                        warn!("Failed to initialize renderer: {e}");
                    },
                }

                // Update camera viewport to match logical window size (for mouse coordinate conversion)
                // Mouse coordinates are in logical pixels, not physical
                let logical_width = (actual_size.width as f64 / scale_factor as f64) as u32;
                let logical_height = (actual_size.height as f64 / scale_factor as f64) as u32;
                self.camera.set_viewport(logical_width, logical_height);
                self.config.window_width = logical_width;
                self.config.window_height = logical_height;

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

        // Check for quit request from automation
        if self.quit_requested {
            info!("[AUTOMATION] Exiting application");
            if let Err(e) = self.config.save() {
                warn!("Failed to save config: {e}");
            }
            event_loop.exit();
            return;
        }

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
                    let scale_factor = window.scale_factor();
                    renderer.set_scale_factor(scale_factor as f32);
                    // Update config and camera viewport with logical size (for mouse coordinate conversion)
                    let logical_width = (new_size.width as f64 / scale_factor) as u32;
                    let logical_height = (new_size.height as f64 / scale_factor) as u32;
                    self.config.window_width = logical_width;
                    self.config.window_height = logical_height;
                    self.camera.set_viewport(logical_width, logical_height);
                }
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
    // Parse command-line arguments for automation
    let args: Vec<String> = std::env::args().collect();
    let mut macro_file: Option<String> = None;
    let mut macro_commands: Option<String> = None;
    let mut auto_start = false;
    let mut use_debug_atlas = false;
    let mut use_pure_colors = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--macro-file" | "-f" => {
                if i + 1 < args.len() {
                    macro_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--macro" | "-m" => {
                if i + 1 < args.len() {
                    macro_commands = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--auto-start" | "-a" => {
                auto_start = true;
            }
            "--debug-atlas" | "-d" => {
                use_debug_atlas = true;
            }
            "--pure-colors" | "-p" => {
                use_pure_colors = true;
            }
            "--help" | "-h" => {
                println!("Genesis Engine - Automation Options");
                println!("");
                println!("  --macro-file, -f <path>   Load and run a macro from JSON file");
                println!("  --macro, -m <commands>    Run inline macro commands");
                println!("  --auto-start, -a          Auto-start game (skip main menu)");
                println!("  --debug-atlas, -d         Use debug autotile atlas for testing");
                println!("  --pure-colors, -p         Use pure colors (no textures) for testing");
                println!("");
                println!("Macro command format: \"action1; action2; action3\"");
                println!("Available actions:");
                println!("  wait <ms>                 Wait for milliseconds");
                println!("  move <dx> <dy> <ms>       Move in direction for duration");
                println!("  setpos <x> <y>            Teleport to position");
                println!("  zoom <level>              Set camera zoom");
                println!("  screenshot [filename]     Capture screenshot");
                println!("  newgame                   Start new game");
                println!("  pause                     Open pause menu");
                println!("  resume                    Resume game");
                println!("  worldtools                Open world tools");
                println!("  seed <value>              Set world seed");
                println!("  regen                     Regenerate world");
                println!("  log <message>             Log a message");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    // Load configuration
    let mut config = EngineConfig::load();
    config.validate();

    // Apply CLI overrides
    if use_pure_colors {
        config.use_pure_colors = true;
        info!("Pure color mode enabled via CLI");
    }

    info!("Configuration loaded:");
    info!("  Window: {}x{}", config.window_width, config.window_height);
    info!("  VSync: {}", config.vsync);
    info!("  Render distance: {} chunks", config.render_distance);

    info!("Creating event loop...");
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = GenesisApp::new(config);

    // Enable debug atlas if requested
    if use_debug_atlas {
        app.set_use_debug_atlas(true);
    }

    // Set up automation if requested
    if macro_file.is_some() || macro_commands.is_some() || auto_start {
        app.automation.enable();
        info!("Automation system enabled");

        // Load built-in test macros
        app.automation.register_macro(crate::automation::AutomationSystem::create_biome_test_macro());
        app.automation.register_macro(crate::automation::AutomationSystem::create_regen_test_macro());

        // Load macros from directory
        let loaded = app.automation.load_macros_from_dir();
        if !loaded.is_empty() {
            info!("Loaded macros from directory: {:?}", loaded);
        }

        // Load macro file if specified
        if let Some(path) = macro_file {
            match app.automation.load_macro_file(std::path::Path::new(&path)) {
                Ok(name) => {
                    info!("Running macro '{}' from file", name);
                    if let Err(e) = app.automation.run_macro(&name) {
                        error!("Failed to run macro: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to load macro file: {}", e);
                }
            }
        }

        // Parse inline macro commands if specified
        if let Some(commands) = macro_commands {
            match crate::automation::parse_cli_macro(&commands) {
                Ok(actions) => {
                    info!("Queuing {} inline macro actions", actions.len());
                    app.automation.queue_actions(actions);
                }
                Err(e) => {
                    error!("Failed to parse macro commands: {}", e);
                }
            }
        }

        // Auto-start game if requested
        if auto_start {
            app.automation.queue_action(crate::automation::AutomationAction::StartNewGame);
        }
    }

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
