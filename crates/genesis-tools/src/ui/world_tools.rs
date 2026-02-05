//! World Tools Panel - Comprehensive biome and world generation controls.
//!
//! This module provides an in-game tool panel accessible from the ESC menu
//! for parameterizing and generating world features including:
//! - Biome configuration (materials, depths, thresholds)
//! - Noise function parameters (seed, scale, octaves, persistence)
//! - Weather system controls
//! - Faction management
//! - Material definitions
//!
//! The panel supports meta-recursion of noise layers for complex terrain generation.

use egui::{Color32, Context, Id, RichText, Ui};
use serde::{Deserialize, Serialize};

// ============================================================================
// Noise Layer Configuration
// ============================================================================

/// A configurable noise layer for world generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseLayer {
    /// Layer name/identifier.
    pub name: String,
    /// Whether this layer is enabled.
    pub enabled: bool,
    /// Random seed for noise generation.
    pub seed: u64,
    /// Scale factor (smaller = larger features).
    pub scale: f64,
    /// Number of octaves for fractal noise.
    pub octaves: u32,
    /// Persistence (amplitude falloff per octave).
    pub persistence: f64,
    /// Lacunarity (frequency multiplier per octave).
    pub lacunarity: f64,
    /// Layer weight when blending.
    pub weight: f64,
    /// Operation mode when combining with other layers.
    pub blend_mode: NoiseBlendMode,
}

impl Default for NoiseLayer {
    fn default() -> Self {
        Self {
            name: String::from("Primary"),
            enabled: true,
            seed: 42,
            scale: 0.005,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
            weight: 1.0,
            blend_mode: NoiseBlendMode::Add,
        }
    }
}

impl NoiseLayer {
    /// Create a new noise layer with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create a detail/variation layer.
    pub fn detail() -> Self {
        Self {
            name: String::from("Detail"),
            enabled: true,
            seed: 43,
            scale: 0.02,
            octaves: 2,
            persistence: 0.6,
            lacunarity: 2.0,
            weight: 0.2,
            blend_mode: NoiseBlendMode::Add,
        }
    }

    /// Create an elevation layer for mountains.
    pub fn elevation() -> Self {
        Self {
            name: String::from("Elevation"),
            enabled: true,
            seed: 44,
            scale: 0.01,
            octaves: 4,
            persistence: 0.45,
            lacunarity: 2.2,
            weight: 0.8,
            blend_mode: NoiseBlendMode::Multiply,
        }
    }

    /// Create a moisture layer for biome determination.
    pub fn moisture() -> Self {
        Self {
            name: String::from("Moisture"),
            enabled: true,
            seed: 45,
            scale: 0.008,
            octaves: 3,
            persistence: 0.55,
            lacunarity: 2.0,
            weight: 0.6,
            blend_mode: NoiseBlendMode::Add,
        }
    }

    /// Create a temperature layer.
    pub fn temperature() -> Self {
        Self {
            name: String::from("Temperature"),
            enabled: true,
            seed: 46,
            scale: 0.003,
            octaves: 2,
            persistence: 0.5,
            lacunarity: 2.0,
            weight: 0.5,
            blend_mode: NoiseBlendMode::Add,
        }
    }
}

/// Blend mode for combining noise layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NoiseBlendMode {
    /// Add layer values together.
    #[default]
    Add,
    /// Multiply layer values.
    Multiply,
    /// Take maximum of layer values.
    Max,
    /// Take minimum of layer values.
    Min,
    /// Subtract this layer from the result.
    Subtract,
    /// Average this layer with the result.
    Average,
    /// Use this layer as a mask (0-1 range).
    Mask,
}

impl NoiseBlendMode {
    /// Get all blend modes.
    pub fn all() -> &'static [Self] {
        &[
            Self::Add,
            Self::Multiply,
            Self::Max,
            Self::Min,
            Self::Subtract,
            Self::Average,
            Self::Mask,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Add => "Add",
            Self::Multiply => "Multiply",
            Self::Max => "Maximum",
            Self::Min => "Minimum",
            Self::Subtract => "Subtract",
            Self::Average => "Average",
            Self::Mask => "Mask",
        }
    }
}

// ============================================================================
// Biome Configuration
// ============================================================================

/// Configurable biome parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeParams {
    /// Biome ID.
    pub id: u8,
    /// Display name.
    pub name: String,
    /// Whether biome is enabled.
    pub enabled: bool,
    /// Surface material ID.
    pub surface_material: u16,
    /// Subsurface material ID.
    pub subsurface_material: u16,
    /// Deep layer material ID.
    pub deep_material: u16,
    /// Surface layer depth in cells.
    pub surface_depth: u32,
    /// Subsurface layer depth in cells.
    pub subsurface_depth: u32,
    /// Minimum noise threshold for this biome.
    pub min_threshold: f64,
    /// Maximum noise threshold for this biome.
    pub max_threshold: f64,
    /// Minimum elevation for this biome (-1 to 1).
    pub min_elevation: f64,
    /// Maximum elevation for this biome (-1 to 1).
    pub max_elevation: f64,
    /// Display color for minimap/preview.
    pub color: [u8; 3],
}

impl BiomeParams {
    /// Create forest biome params.
    pub fn forest() -> Self {
        Self {
            id: 0,
            name: String::from("Forest"),
            enabled: true,
            surface_material: 3,  // GRASS
            subsurface_material: 1, // DIRT
            deep_material: 2,     // STONE
            surface_depth: 1,
            subsurface_depth: 8,
            min_threshold: -0.15,
            max_threshold: 0.15,
            min_elevation: -0.5,
            max_elevation: 0.5,
            color: [45, 90, 29], // #2d5a1d
        }
    }

    /// Create desert biome params.
    pub fn desert() -> Self {
        Self {
            id: 1,
            name: String::from("Desert"),
            enabled: true,
            surface_material: 5,  // SAND
            subsurface_material: 7, // SANDSTONE
            deep_material: 2,     // STONE
            surface_depth: 4,
            subsurface_depth: 16,
            min_threshold: 0.15,
            max_threshold: 0.4,
            min_elevation: -0.3,
            max_elevation: 0.3,
            color: [196, 163, 90], // #c4a35a
        }
    }

    /// Create ocean biome params.
    pub fn ocean() -> Self {
        Self {
            id: 3,
            name: String::from("Ocean"),
            enabled: true,
            surface_material: 5,  // SAND
            subsurface_material: 8, // CLAY
            deep_material: 2,     // STONE
            surface_depth: 2,
            subsurface_depth: 10,
            min_threshold: -1.0,
            max_threshold: -0.4,
            min_elevation: -1.0,
            max_elevation: -0.2,
            color: [58, 124, 165], // #3a7ca5
        }
    }

    /// Create plains biome params.
    pub fn plains() -> Self {
        Self {
            id: 4,
            name: String::from("Plains"),
            enabled: true,
            surface_material: 3,  // GRASS
            subsurface_material: 1, // DIRT
            deep_material: 2,     // STONE
            surface_depth: 2,
            subsurface_depth: 12,
            min_threshold: -0.4,
            max_threshold: -0.15,
            min_elevation: -0.3,
            max_elevation: 0.3,
            color: [124, 179, 66], // #7cb342
        }
    }

    /// Create mountain biome params.
    pub fn mountain() -> Self {
        Self {
            id: 5,
            name: String::from("Mountain"),
            enabled: true,
            surface_material: 2,  // STONE
            subsurface_material: 2, // STONE
            deep_material: 2,     // STONE
            surface_depth: 0,
            subsurface_depth: 0,
            min_threshold: 0.3,
            max_threshold: 1.0,
            min_elevation: 0.3,
            max_elevation: 1.0,
            color: [122, 122, 122], // #7a7a7a
        }
    }

    /// Create cave biome params.
    pub fn cave() -> Self {
        Self {
            id: 2,
            name: String::from("Cave"),
            enabled: true,
            surface_material: 2,  // STONE
            subsurface_material: 2, // STONE
            deep_material: 2,     // STONE
            surface_depth: 0,
            subsurface_depth: 0,
            min_threshold: 0.6,
            max_threshold: 1.0,
            min_elevation: -1.0,
            max_elevation: -0.5,
            color: [60, 60, 60],
        }
    }

    /// Get all default biomes.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::forest(),
            Self::desert(),
            Self::ocean(),
            Self::plains(),
            Self::mountain(),
            Self::cave(),
        ]
    }
}

// ============================================================================
// Weather Configuration
// ============================================================================

/// Configurable weather parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherParams {
    /// Whether weather system is enabled.
    pub enabled: bool,
    /// Current weather state index.
    pub current_state: usize,
    /// Minimum weather duration in seconds.
    pub min_duration: f32,
    /// Maximum weather duration in seconds.
    pub max_duration: f32,
    /// Weather transition weights (4x4 matrix).
    pub transition_weights: [[f32; 4]; 4],
    /// Light level modifiers per weather state.
    pub light_modifiers: [f32; 4],
    /// Movement speed modifiers per weather state.
    pub movement_modifiers: [f32; 4],
    /// Plant growth modifiers per weather state.
    pub growth_modifiers: [f32; 4],
}

impl Default for WeatherParams {
    fn default() -> Self {
        Self {
            enabled: true,
            current_state: 0, // Clear
            min_duration: 120.0,
            max_duration: 600.0,
            transition_weights: [
                [0.5, 0.4, 0.08, 0.02], // From Clear
                [0.3, 0.3, 0.3, 0.1],   // From Cloudy
                [0.1, 0.3, 0.4, 0.2],   // From Raining
                [0.05, 0.15, 0.5, 0.3], // From Storm
            ],
            light_modifiers: [1.0, 0.8, 0.6, 0.4],
            movement_modifiers: [1.0, 1.0, 0.9, 0.75],
            growth_modifiers: [1.0, 1.1, 1.5, 1.3],
        }
    }
}

impl WeatherParams {
    /// Weather state names.
    pub const STATES: [&'static str; 4] = ["Clear", "Cloudy", "Raining", "Storm"];
}

// ============================================================================
// Faction Configuration
// ============================================================================

/// Configurable faction parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionParams {
    /// Faction ID.
    pub id: u16,
    /// Faction name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Whether faction is joinable by player.
    pub joinable: bool,
    /// Reputation required to join.
    pub join_requirement: i32,
    /// Starting reputation with this faction.
    pub starting_reputation: i32,
    /// Display color.
    pub color: [u8; 3],
}

impl FactionParams {
    /// Create a new faction.
    pub fn new(id: u16, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: String::new(),
            joinable: true,
            join_requirement: 0,
            starting_reputation: 0,
            color: [128, 128, 128],
        }
    }

    /// Get default factions.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: 1,
                name: String::from("Town Guard"),
                description: String::from("Protectors of the settlements"),
                joinable: true,
                join_requirement: 10,
                starting_reputation: 0,
                color: [70, 130, 180],
            },
            Self {
                id: 2,
                name: String::from("Merchants Guild"),
                description: String::from("Trade and commerce association"),
                joinable: true,
                join_requirement: 0,
                starting_reputation: 5,
                color: [218, 165, 32],
            },
            Self {
                id: 3,
                name: String::from("Forest Keepers"),
                description: String::from("Guardians of nature and wildlife"),
                joinable: true,
                join_requirement: 15,
                starting_reputation: 0,
                color: [34, 139, 34],
            },
            Self {
                id: 4,
                name: String::from("Bandits"),
                description: String::from("Outlaws and raiders"),
                joinable: false,
                join_requirement: -50,
                starting_reputation: -20,
                color: [139, 69, 19],
            },
        ]
    }
}

// ============================================================================
// Material Configuration
// ============================================================================

/// Configurable material parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialParams {
    /// Material ID.
    pub id: u16,
    /// Material name.
    pub name: String,
    /// Whether material is solid (blocks movement).
    pub solid: bool,
    /// Whether material is liquid.
    pub liquid: bool,
    /// Hardness (affects mining time).
    pub hardness: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Display color.
    pub color: [u8; 3],
    /// Autotile terrain index.
    pub terrain_index: u32,
}

impl MaterialParams {
    /// Get default materials.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: 0,
                name: String::from("Air"),
                solid: false,
                liquid: false,
                hardness: 0.0,
                friction: 0.0,
                color: [0, 0, 0],
                terrain_index: 0,
            },
            Self {
                id: 1,
                name: String::from("Dirt"),
                solid: true,
                liquid: false,
                hardness: 1.0,
                friction: 0.8,
                color: [139, 90, 43],
                terrain_index: 1,
            },
            Self {
                id: 2,
                name: String::from("Stone"),
                solid: true,
                liquid: false,
                hardness: 5.0,
                friction: 0.9,
                color: [128, 128, 128],
                terrain_index: 2,
            },
            Self {
                id: 3,
                name: String::from("Grass"),
                solid: true,
                liquid: false,
                hardness: 0.8,
                friction: 0.7,
                color: [34, 139, 34],
                terrain_index: 3,
            },
            Self {
                id: 4,
                name: String::from("Water"),
                solid: false,
                liquid: true,
                hardness: 0.0,
                friction: 0.3,
                color: [64, 164, 223],
                terrain_index: 4,
            },
            Self {
                id: 5,
                name: String::from("Sand"),
                solid: true,
                liquid: false,
                hardness: 0.5,
                friction: 0.6,
                color: [238, 214, 175],
                terrain_index: 5,
            },
            Self {
                id: 6,
                name: String::from("Lava"),
                solid: false,
                liquid: true,
                hardness: 0.0,
                friction: 0.2,
                color: [255, 80, 20],
                terrain_index: 6,
            },
            Self {
                id: 7,
                name: String::from("Sandstone"),
                solid: true,
                liquid: false,
                hardness: 3.0,
                friction: 0.85,
                color: [210, 180, 140],
                terrain_index: 7,
            },
            Self {
                id: 8,
                name: String::from("Clay"),
                solid: true,
                liquid: false,
                hardness: 1.5,
                friction: 0.75,
                color: [178, 147, 121],
                terrain_index: 8,
            },
            Self {
                id: 9,
                name: String::from("Gravel"),
                solid: true,
                liquid: false,
                hardness: 1.2,
                friction: 0.65,
                color: [169, 169, 169],
                terrain_index: 9,
            },
            Self {
                id: 10,
                name: String::from("Snow"),
                solid: true,
                liquid: false,
                hardness: 0.3,
                friction: 0.4,
                color: [255, 250, 250],
                terrain_index: 10,
            },
        ]
    }
}

// ============================================================================
// World Generation Configuration
// ============================================================================

/// Complete world generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldGenConfig {
    /// World seed.
    pub seed: u64,
    /// Noise layers for terrain generation.
    pub noise_layers: Vec<NoiseLayer>,
    /// Biome configurations.
    pub biomes: Vec<BiomeParams>,
    /// Weather configuration.
    pub weather: WeatherParams,
    /// Faction configurations.
    pub factions: Vec<FactionParams>,
    /// Material configurations.
    pub materials: Vec<MaterialParams>,
    /// World size in chunks.
    pub world_size_chunks: (u32, u32),
    /// Chunk size in cells.
    pub chunk_size: u32,
    /// Sea level (0.0 = middle, negative = lower).
    pub sea_level: f64,
    /// Enable cave generation.
    pub caves_enabled: bool,
    /// Cave density (0.0 - 1.0).
    pub cave_density: f64,
    /// Enable ore generation.
    pub ores_enabled: bool,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            noise_layers: vec![
                NoiseLayer::default(),
                NoiseLayer::detail(),
                NoiseLayer::elevation(),
                NoiseLayer::moisture(),
                NoiseLayer::temperature(),
            ],
            biomes: BiomeParams::defaults(),
            weather: WeatherParams::default(),
            factions: FactionParams::defaults(),
            materials: MaterialParams::defaults(),
            world_size_chunks: (64, 64),
            chunk_size: 32,
            sea_level: -0.2,
            caves_enabled: true,
            cave_density: 0.1,
            ores_enabled: true,
        }
    }
}

// ============================================================================
// World Tools Panel
// ============================================================================

/// Active tab in the world tools panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorldToolsTab {
    /// Biome configuration tab.
    #[default]
    Biomes,
    /// Noise layers configuration tab.
    Noise,
    /// Weather configuration tab.
    Weather,
    /// Faction configuration tab.
    Factions,
    /// Material configuration tab.
    Materials,
    /// World generation settings tab.
    WorldGen,
}

impl WorldToolsTab {
    /// Get all tabs.
    pub fn all() -> &'static [Self] {
        &[
            Self::Biomes,
            Self::Noise,
            Self::Weather,
            Self::Factions,
            Self::Materials,
            Self::WorldGen,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Biomes => "Biomes",
            Self::Noise => "Noise",
            Self::Weather => "Weather",
            Self::Factions => "Factions",
            Self::Materials => "Materials",
            Self::WorldGen => "World Gen",
        }
    }

    /// Get icon.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Biomes => "🌲",
            Self::Noise => "〰",
            Self::Weather => "🌤",
            Self::Factions => "⚔",
            Self::Materials => "🧱",
            Self::WorldGen => "🌍",
        }
    }
}

/// World Tools panel state and actions.
#[derive(Debug, Clone)]
pub struct WorldTools {
    /// Whether the panel is visible.
    visible: bool,
    /// Active tab.
    active_tab: WorldToolsTab,
    /// World generation configuration.
    config: WorldGenConfig,
    /// Selected biome index.
    selected_biome: usize,
    /// Selected noise layer index.
    selected_noise_layer: usize,
    /// Selected faction index.
    selected_faction: usize,
    /// Selected material index.
    selected_material: usize,
    /// Whether config has been modified.
    modified: bool,
    /// Pending actions.
    actions: Vec<WorldToolsAction>,
}

/// Actions that can be performed from the world tools panel.
#[derive(Debug, Clone, PartialEq)]
pub enum WorldToolsAction {
    /// Close the panel.
    Close,
    /// Apply changes without regenerating.
    ApplyChanges,
    /// Regenerate the world with current settings.
    RegenerateWorld,
    /// Reset to defaults.
    ResetDefaults,
    /// Export configuration to file.
    ExportConfig,
    /// Import configuration from file.
    ImportConfig,
    /// Randomize seed.
    RandomizeSeed,
}

impl Default for WorldTools {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldTools {
    /// Create a new world tools panel.
    pub fn new() -> Self {
        Self {
            visible: false,
            active_tab: WorldToolsTab::Biomes,
            config: WorldGenConfig::default(),
            selected_biome: 0,
            selected_noise_layer: 0,
            selected_faction: 0,
            selected_material: 0,
            modified: false,
            actions: Vec::new(),
        }
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the panel.
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the panel.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Get the configuration.
    pub fn config(&self) -> &WorldGenConfig {
        &self.config
    }

    /// Get mutable configuration.
    pub fn config_mut(&mut self) -> &mut WorldGenConfig {
        self.modified = true;
        &mut self.config
    }

    /// Set the configuration.
    pub fn set_config(&mut self, config: WorldGenConfig) {
        self.config = config;
        self.modified = true;
    }

    /// Check if modified.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Clear modified flag.
    pub fn clear_modified(&mut self) {
        self.modified = false;
    }

    /// Drain pending actions.
    pub fn drain_actions(&mut self) -> Vec<WorldToolsAction> {
        std::mem::take(&mut self.actions)
    }

    /// Check if has pending actions.
    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    /// Render the world tools panel.
    pub fn render(&mut self, ctx: &Context) {
        if !self.visible {
            return;
        }

        // Semi-transparent overlay
        egui::Area::new(Id::new("world_tools_overlay"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(ctx, |ui| {
                let screen = ctx.screen_rect();
                ui.painter().rect_filled(
                    screen,
                    0.0,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 160),
                );
            });

        // Main panel
        egui::Window::new("🌍 World Tools")
            .id(Id::new("world_tools_panel"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(true)
            .default_width(700.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                // Header with tabs
                ui.horizontal(|ui| {
                    for tab in WorldToolsTab::all() {
                        let selected = self.active_tab == *tab;
                        let text = format!("{} {}", tab.icon(), tab.display_name());
                        if ui.selectable_label(selected, text).clicked() {
                            self.active_tab = *tab;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕ Close").clicked() {
                            self.actions.push(WorldToolsAction::Close);
                            self.visible = false;
                        }
                    });
                });

                ui.separator();

                // Content area
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.active_tab {
                        WorldToolsTab::Biomes => self.render_biomes_tab(ui),
                        WorldToolsTab::Noise => self.render_noise_tab(ui),
                        WorldToolsTab::Weather => self.render_weather_tab(ui),
                        WorldToolsTab::Factions => self.render_factions_tab(ui),
                        WorldToolsTab::Materials => self.render_materials_tab(ui),
                        WorldToolsTab::WorldGen => self.render_worldgen_tab(ui),
                    }
                });

                ui.separator();

                // Footer with actions
                ui.horizontal(|ui| {
                    if self.modified {
                        ui.label(RichText::new("● Modified").color(Color32::YELLOW));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄 Regenerate World").clicked() {
                            self.actions.push(WorldToolsAction::RegenerateWorld);
                        }
                        if ui.button("✓ Apply").clicked() {
                            self.actions.push(WorldToolsAction::ApplyChanges);
                            self.modified = false;
                        }
                        if ui.button("↺ Reset").clicked() {
                            self.config = WorldGenConfig::default();
                            self.modified = false;
                            self.actions.push(WorldToolsAction::ResetDefaults);
                        }
                    });
                });
            });
    }

    fn render_biomes_tab(&mut self, ui: &mut Ui) {
        ui.heading("Biome Configuration");
        ui.label("Configure biome materials, depths, and distribution thresholds.");
        ui.add_space(8.0);

        // Biome list on left, details on right
        ui.horizontal(|ui| {
            // Biome list
            ui.vertical(|ui| {
                ui.set_min_width(150.0);
                ui.label(RichText::new("Biomes").strong());

                for (i, biome) in self.config.biomes.iter().enumerate() {
                    let selected = self.selected_biome == i;
                    let color = Color32::from_rgb(biome.color[0], biome.color[1], biome.color[2]);
                    ui.horizontal(|ui| {
                        ui.colored_label(color, "●");
                        if ui.selectable_label(selected, &biome.name).clicked() {
                            self.selected_biome = i;
                        }
                    });
                }

                ui.add_space(8.0);
                if ui.button("+ Add Biome").clicked() {
                    let new_id = self.config.biomes.len() as u8;
                    self.config.biomes.push(BiomeParams {
                        id: new_id,
                        name: format!("Biome {}", new_id),
                        ..BiomeParams::forest()
                    });
                    self.selected_biome = self.config.biomes.len() - 1;
                    self.modified = true;
                }
            });

            ui.separator();

            // Biome details
            ui.vertical(|ui| {
                if let Some(biome) = self.config.biomes.get_mut(self.selected_biome) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(&mut biome.name).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        if ui.checkbox(&mut biome.enabled, "").changed() {
                            self.modified = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("Materials").strong());

                    ui.horizontal(|ui| {
                        ui.label("Surface:");
                        let mut surface = biome.surface_material as i32;
                        if ui.add(egui::DragValue::new(&mut surface).range(0..=255)).changed() {
                            biome.surface_material = surface as u16;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Subsurface:");
                        let mut sub = biome.subsurface_material as i32;
                        if ui.add(egui::DragValue::new(&mut sub).range(0..=255)).changed() {
                            biome.subsurface_material = sub as u16;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Deep:");
                        let mut deep = biome.deep_material as i32;
                        if ui.add(egui::DragValue::new(&mut deep).range(0..=255)).changed() {
                            biome.deep_material = deep as u16;
                            self.modified = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("Layer Depths").strong());

                    ui.horizontal(|ui| {
                        ui.label("Surface Depth:");
                        let mut d = biome.surface_depth as i32;
                        if ui.add(egui::DragValue::new(&mut d).range(0..=100)).changed() {
                            biome.surface_depth = d as u32;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Subsurface Depth:");
                        let mut d = biome.subsurface_depth as i32;
                        if ui.add(egui::DragValue::new(&mut d).range(0..=100)).changed() {
                            biome.subsurface_depth = d as u32;
                            self.modified = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("Distribution Thresholds").strong());

                    ui.horizontal(|ui| {
                        ui.label("Min Threshold:");
                        let mut t = biome.min_threshold;
                        if ui.add(egui::DragValue::new(&mut t).speed(0.01).range(-1.0..=1.0)).changed() {
                            biome.min_threshold = t;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max Threshold:");
                        let mut t = biome.max_threshold;
                        if ui.add(egui::DragValue::new(&mut t).speed(0.01).range(-1.0..=1.0)).changed() {
                            biome.max_threshold = t;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Min Elevation:");
                        let mut e = biome.min_elevation;
                        if ui.add(egui::DragValue::new(&mut e).speed(0.01).range(-1.0..=1.0)).changed() {
                            biome.min_elevation = e;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max Elevation:");
                        let mut e = biome.max_elevation;
                        if ui.add(egui::DragValue::new(&mut e).speed(0.01).range(-1.0..=1.0)).changed() {
                            biome.max_elevation = e;
                            self.modified = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = egui::Color32::from_rgb(biome.color[0], biome.color[1], biome.color[2]);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            biome.color = [color.r(), color.g(), color.b()];
                            self.modified = true;
                        }
                    });
                }
            });
        });
    }

    fn render_noise_tab(&mut self, ui: &mut Ui) {
        ui.heading("Noise Layer Configuration");
        ui.label("Configure multiple noise layers for meta-recursive world generation.");
        ui.add_space(8.0);

        // Noise layer list on left, details on right
        ui.horizontal(|ui| {
            // Layer list
            ui.vertical(|ui| {
                ui.set_min_width(150.0);
                ui.label(RichText::new("Layers").strong());

                for (i, layer) in self.config.noise_layers.iter().enumerate() {
                    let selected = self.selected_noise_layer == i;
                    let label = if layer.enabled {
                        format!("● {}", layer.name)
                    } else {
                        format!("○ {}", layer.name)
                    };
                    if ui.selectable_label(selected, label).clicked() {
                        self.selected_noise_layer = i;
                    }
                }

                ui.add_space(8.0);
                if ui.button("+ Add Layer").clicked() {
                    let new_name = format!("Layer {}", self.config.noise_layers.len());
                    self.config.noise_layers.push(NoiseLayer::new(new_name));
                    self.selected_noise_layer = self.config.noise_layers.len() - 1;
                    self.modified = true;
                }

                if self.config.noise_layers.len() > 1 && ui.button("- Remove").clicked() {
                    self.config.noise_layers.remove(self.selected_noise_layer);
                    self.selected_noise_layer = self.selected_noise_layer.saturating_sub(1);
                    self.modified = true;
                }
            });

            ui.separator();

            // Layer details
            ui.vertical(|ui| {
                if let Some(layer) = self.config.noise_layers.get_mut(self.selected_noise_layer) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(&mut layer.name).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        if ui.checkbox(&mut layer.enabled, "").changed() {
                            self.modified = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("Noise Parameters").strong());

                    ui.horizontal(|ui| {
                        ui.label("Seed:");
                        let mut seed = layer.seed as i64;
                        if ui.add(egui::DragValue::new(&mut seed).range(0..=i64::MAX)).changed() {
                            layer.seed = seed as u64;
                            self.modified = true;
                        }
                        if ui.button("🎲").on_hover_text("Randomize").clicked() {
                            layer.seed = rand_seed();
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        if ui.add(egui::DragValue::new(&mut layer.scale).speed(0.0001).range(0.0001..=1.0)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Octaves:");
                        let mut oct = layer.octaves as i32;
                        if ui.add(egui::DragValue::new(&mut oct).range(1..=8)).changed() {
                            layer.octaves = oct as u32;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Persistence:");
                        if ui.add(egui::DragValue::new(&mut layer.persistence).speed(0.01).range(0.0..=1.0)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Lacunarity:");
                        if ui.add(egui::DragValue::new(&mut layer.lacunarity).speed(0.1).range(1.0..=4.0)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("Blending").strong());

                    ui.horizontal(|ui| {
                        ui.label("Weight:");
                        if ui.add(egui::DragValue::new(&mut layer.weight).speed(0.01).range(0.0..=2.0)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Blend Mode:");
                        egui::ComboBox::from_id_salt("blend_mode")
                            .selected_text(layer.blend_mode.display_name())
                            .show_ui(ui, |ui| {
                                for mode in NoiseBlendMode::all() {
                                    if ui.selectable_value(&mut layer.blend_mode, *mode, mode.display_name()).changed() {
                                        self.modified = true;
                                    }
                                }
                            });
                    });

                    ui.add_space(16.0);
                    ui.label(RichText::new("Presets").strong());
                    ui.horizontal(|ui| {
                        if ui.button("Primary").clicked() {
                            *layer = NoiseLayer::default();
                            layer.name = String::from("Primary");
                            self.modified = true;
                        }
                        if ui.button("Detail").clicked() {
                            let name = layer.name.clone();
                            *layer = NoiseLayer::detail();
                            layer.name = name;
                            self.modified = true;
                        }
                        if ui.button("Elevation").clicked() {
                            let name = layer.name.clone();
                            *layer = NoiseLayer::elevation();
                            layer.name = name;
                            self.modified = true;
                        }
                        if ui.button("Moisture").clicked() {
                            let name = layer.name.clone();
                            *layer = NoiseLayer::moisture();
                            layer.name = name;
                            self.modified = true;
                        }
                    });
                }
            });
        });
    }

    fn render_weather_tab(&mut self, ui: &mut Ui) {
        ui.heading("Weather Configuration");
        ui.label("Configure weather states, transitions, and effects on gameplay.");
        ui.add_space(8.0);

        let weather = &mut self.config.weather;

        ui.horizontal(|ui| {
            ui.label("Weather System:");
            if ui.checkbox(&mut weather.enabled, "Enabled").changed() {
                self.modified = true;
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Current State").strong());
        ui.horizontal(|ui| {
            for (i, state) in WeatherParams::STATES.iter().enumerate() {
                if ui.selectable_label(weather.current_state == i, *state).clicked() {
                    weather.current_state = i;
                    self.modified = true;
                }
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Duration Range (seconds)").strong());
        ui.horizontal(|ui| {
            ui.label("Min:");
            if ui.add(egui::DragValue::new(&mut weather.min_duration).range(10.0..=600.0)).changed() {
                self.modified = true;
            }
            ui.label("Max:");
            if ui.add(egui::DragValue::new(&mut weather.max_duration).range(60.0..=1800.0)).changed() {
                self.modified = true;
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Weather Modifiers").strong());
        ui.label("Light / Movement / Growth modifiers per state:");

        egui::Grid::new("weather_modifiers").striped(true).show(ui, |ui| {
            ui.label("State");
            ui.label("Light");
            ui.label("Movement");
            ui.label("Growth");
            ui.end_row();

            for (i, state) in WeatherParams::STATES.iter().enumerate() {
                ui.label(*state);
                if ui.add(egui::DragValue::new(&mut weather.light_modifiers[i]).speed(0.01).range(0.0..=2.0)).changed() {
                    self.modified = true;
                }
                if ui.add(egui::DragValue::new(&mut weather.movement_modifiers[i]).speed(0.01).range(0.0..=2.0)).changed() {
                    self.modified = true;
                }
                if ui.add(egui::DragValue::new(&mut weather.growth_modifiers[i]).speed(0.01).range(0.0..=3.0)).changed() {
                    self.modified = true;
                }
                ui.end_row();
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Transition Probabilities").strong());
        ui.label("Probability of transitioning from row to column:");

        egui::Grid::new("weather_transitions").striped(true).show(ui, |ui| {
            ui.label("From \\ To");
            for state in WeatherParams::STATES.iter() {
                ui.label(*state);
            }
            ui.end_row();

            for (i, from_state) in WeatherParams::STATES.iter().enumerate() {
                ui.label(*from_state);
                for j in 0..4 {
                    if ui.add(egui::DragValue::new(&mut weather.transition_weights[i][j]).speed(0.01).range(0.0..=1.0)).changed() {
                        self.modified = true;
                    }
                }
                ui.end_row();
            }
        });
    }

    fn render_factions_tab(&mut self, ui: &mut Ui) {
        ui.heading("Faction Configuration");
        ui.label("Configure factions, relationships, and reputation requirements.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            // Faction list
            ui.vertical(|ui| {
                ui.set_min_width(150.0);
                ui.label(RichText::new("Factions").strong());

                for (i, faction) in self.config.factions.iter().enumerate() {
                    let selected = self.selected_faction == i;
                    let color = Color32::from_rgb(faction.color[0], faction.color[1], faction.color[2]);
                    ui.horizontal(|ui| {
                        ui.colored_label(color, "●");
                        if ui.selectable_label(selected, &faction.name).clicked() {
                            self.selected_faction = i;
                        }
                    });
                }

                ui.add_space(8.0);
                if ui.button("+ Add Faction").clicked() {
                    let new_id = self.config.factions.len() as u16 + 100;
                    self.config.factions.push(FactionParams::new(new_id, format!("Faction {}", new_id)));
                    self.selected_faction = self.config.factions.len() - 1;
                    self.modified = true;
                }
            });

            ui.separator();

            // Faction details
            ui.vertical(|ui| {
                if let Some(faction) = self.config.factions.get_mut(self.selected_faction) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(&mut faction.name).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        if ui.text_edit_multiline(&mut faction.description).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Joinable:");
                        if ui.checkbox(&mut faction.joinable, "").changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Join Requirement:");
                        if ui.add(egui::DragValue::new(&mut faction.join_requirement).range(-100..=100)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Starting Reputation:");
                        if ui.add(egui::DragValue::new(&mut faction.starting_reputation).range(-100..=100)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = egui::Color32::from_rgb(faction.color[0], faction.color[1], faction.color[2]);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            faction.color = [color.r(), color.g(), color.b()];
                            self.modified = true;
                        }
                    });
                }
            });
        });
    }

    fn render_materials_tab(&mut self, ui: &mut Ui) {
        ui.heading("Material Configuration");
        ui.label("Configure material properties for terrain and items.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            // Material list
            ui.vertical(|ui| {
                ui.set_min_width(150.0);
                ui.label(RichText::new("Materials").strong());

                for (i, mat) in self.config.materials.iter().enumerate() {
                    let selected = self.selected_material == i;
                    let color = Color32::from_rgb(mat.color[0], mat.color[1], mat.color[2]);
                    ui.horizontal(|ui| {
                        ui.colored_label(color, "■");
                        if ui.selectable_label(selected, &mat.name).clicked() {
                            self.selected_material = i;
                        }
                    });
                }

                ui.add_space(8.0);
                if ui.button("+ Add Material").clicked() {
                    let new_id = self.config.materials.len() as u16;
                    self.config.materials.push(MaterialParams {
                        id: new_id,
                        name: format!("Material {}", new_id),
                        solid: true,
                        liquid: false,
                        hardness: 1.0,
                        friction: 0.8,
                        color: [128, 128, 128],
                        terrain_index: new_id as u32,
                    });
                    self.selected_material = self.config.materials.len() - 1;
                    self.modified = true;
                }
            });

            ui.separator();

            // Material details
            ui.vertical(|ui| {
                if let Some(mat) = self.config.materials.get_mut(self.selected_material) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(&mut mat.name).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Solid:");
                        if ui.checkbox(&mut mat.solid, "").changed() {
                            self.modified = true;
                        }
                        ui.label("Liquid:");
                        if ui.checkbox(&mut mat.liquid, "").changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Hardness:");
                        if ui.add(egui::DragValue::new(&mut mat.hardness).speed(0.1).range(0.0..=100.0)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Friction:");
                        if ui.add(egui::DragValue::new(&mut mat.friction).speed(0.01).range(0.0..=1.0)).changed() {
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Terrain Index:");
                        let mut idx = mat.terrain_index as i32;
                        if ui.add(egui::DragValue::new(&mut idx).range(0..=255)).changed() {
                            mat.terrain_index = idx as u32;
                            self.modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = egui::Color32::from_rgb(mat.color[0], mat.color[1], mat.color[2]);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            mat.color = [color.r(), color.g(), color.b()];
                            self.modified = true;
                        }
                    });
                }
            });
        });
    }

    fn render_worldgen_tab(&mut self, ui: &mut Ui) {
        ui.heading("World Generation Settings");
        ui.label("Configure global world generation parameters.");
        ui.add_space(8.0);

        ui.label(RichText::new("World Seed").strong());
        ui.horizontal(|ui| {
            ui.label("Seed:");
            let mut seed = self.config.seed as i64;
            if ui.add(egui::DragValue::new(&mut seed).range(0..=i64::MAX)).changed() {
                self.config.seed = seed as u64;
                self.modified = true;
            }
            if ui.button("🎲 Randomize").clicked() {
                self.config.seed = rand_seed();
                self.modified = true;
                self.actions.push(WorldToolsAction::RandomizeSeed);
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("World Size").strong());
        ui.horizontal(|ui| {
            ui.label("Chunks (W x H):");
            let mut w = self.config.world_size_chunks.0 as i32;
            let mut h = self.config.world_size_chunks.1 as i32;
            if ui.add(egui::DragValue::new(&mut w).range(1..=1024)).changed() {
                self.config.world_size_chunks.0 = w as u32;
                self.modified = true;
            }
            ui.label("x");
            if ui.add(egui::DragValue::new(&mut h).range(1..=1024)).changed() {
                self.config.world_size_chunks.1 = h as u32;
                self.modified = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Chunk Size:");
            let mut cs = self.config.chunk_size as i32;
            if ui.add(egui::DragValue::new(&mut cs).range(8..=128)).changed() {
                self.config.chunk_size = cs as u32;
                self.modified = true;
            }
            ui.label("cells");
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Terrain").strong());
        ui.horizontal(|ui| {
            ui.label("Sea Level:");
            if ui.add(egui::DragValue::new(&mut self.config.sea_level).speed(0.01).range(-1.0..=1.0)).changed() {
                self.modified = true;
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Cave Generation").strong());
        ui.horizontal(|ui| {
            ui.label("Caves:");
            if ui.checkbox(&mut self.config.caves_enabled, "Enabled").changed() {
                self.modified = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Cave Density:");
            if ui.add(egui::DragValue::new(&mut self.config.cave_density).speed(0.01).range(0.0..=1.0)).changed() {
                self.modified = true;
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Resource Generation").strong());
        ui.horizontal(|ui| {
            ui.label("Ores:");
            if ui.checkbox(&mut self.config.ores_enabled, "Enabled").changed() {
                self.modified = true;
            }
        });

        ui.add_space(16.0);
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("📥 Export Config").clicked() {
                self.actions.push(WorldToolsAction::ExportConfig);
            }
            if ui.button("📤 Import Config").clicked() {
                self.actions.push(WorldToolsAction::ImportConfig);
            }
        });
    }
}

/// Generate a pseudo-random seed based on current time.
fn rand_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_layer_default() {
        let layer = NoiseLayer::default();
        assert!(layer.enabled);
        assert_eq!(layer.octaves, 3);
    }

    #[test]
    fn test_biome_params_defaults() {
        let biomes = BiomeParams::defaults();
        assert_eq!(biomes.len(), 6);
    }

    #[test]
    fn test_world_tools_visibility() {
        let mut tools = WorldTools::new();
        assert!(!tools.is_visible());
        tools.show();
        assert!(tools.is_visible());
        tools.hide();
        assert!(!tools.is_visible());
    }

    #[test]
    fn test_worldgen_config_default() {
        let config = WorldGenConfig::default();
        assert_eq!(config.seed, 42);
        assert!(!config.noise_layers.is_empty());
        assert!(!config.biomes.is_empty());
    }
}
