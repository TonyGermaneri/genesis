//! Biome visualization and debug tools.
//!
//! This module provides:
//! - Biome-colored minimap (T-32)
//! - Biome info debug panel (T-33)
//! - World seed display/input (T-34)
//! - Biome legend overlay (T-35)

use egui::{Color32, Context, Id, Key, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

// ============================================================================
// T-32: Biome Types and Coloring
// ============================================================================

/// Biome types for minimap coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BiomeType {
    /// Unknown/unexplored biome.
    #[default]
    Unknown,
    /// Forest biome - dark green.
    Forest,
    /// Desert biome - sandy yellow.
    Desert,
    /// Lake/water biome - blue.
    Lake,
    /// Plains/grassland biome - light green.
    Plains,
    /// Mountain biome - gray.
    Mountain,
    /// Swamp biome - dark olive.
    Swamp,
}

impl BiomeType {
    /// Returns the display color for this biome type (per spec).
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            BiomeType::Unknown => Color32::from_gray(40),
            BiomeType::Forest => Color32::from_rgb(0x2d, 0x5a, 0x1d), // #2d5a1d
            BiomeType::Desert => Color32::from_rgb(0xc4, 0xa3, 0x5a), // #c4a35a
            BiomeType::Lake => Color32::from_rgb(0x3a, 0x7c, 0xa5),   // #3a7ca5
            BiomeType::Plains => Color32::from_rgb(0x7c, 0xb3, 0x42), // #7cb342
            BiomeType::Mountain => Color32::from_rgb(0x7a, 0x7a, 0x7a), // #7a7a7a
            BiomeType::Swamp => Color32::from_rgb(0x4a, 0x5a, 0x23),  // #4a5a23
        }
    }

    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            BiomeType::Unknown => "Unknown",
            BiomeType::Forest => "Forest",
            BiomeType::Desert => "Desert",
            BiomeType::Lake => "Lake",
            BiomeType::Plains => "Plains",
            BiomeType::Mountain => "Mountain",
            BiomeType::Swamp => "Swamp",
        }
    }

    /// Returns all biome types (excluding Unknown).
    #[must_use]
    pub fn all() -> &'static [BiomeType] {
        &[
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Lake,
            BiomeType::Plains,
            BiomeType::Mountain,
            BiomeType::Swamp,
        ]
    }

    /// Returns all biome types including Unknown.
    #[must_use]
    pub fn all_with_unknown() -> &'static [BiomeType] {
        &[
            BiomeType::Unknown,
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Lake,
            BiomeType::Plains,
            BiomeType::Mountain,
            BiomeType::Swamp,
        ]
    }
}

// ============================================================================
// T-32: Biome Minimap
// ============================================================================

/// Data for a single cell in the biome minimap.
#[derive(Debug, Clone, Default)]
pub struct BiomeCell {
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Y coordinate.
    pub chunk_y: i32,
    /// Biome type.
    pub biome: BiomeType,
    /// Whether this cell has been explored.
    pub explored: bool,
}

impl BiomeCell {
    /// Creates a new biome cell.
    #[must_use]
    pub fn new(chunk_x: i32, chunk_y: i32, biome: BiomeType) -> Self {
        Self {
            chunk_x,
            chunk_y,
            biome,
            explored: true,
        }
    }

    /// Creates an unexplored cell.
    #[must_use]
    pub fn unexplored(chunk_x: i32, chunk_y: i32) -> Self {
        Self {
            chunk_x,
            chunk_y,
            biome: BiomeType::Unknown,
            explored: false,
        }
    }
}

/// Number of chunks visible on each axis.
pub const BIOME_MINIMAP_VIEW: usize = 7;

/// Default biome minimap size.
pub const BIOME_MINIMAP_SIZE: f32 = 140.0;

/// Biome minimap data model.
#[derive(Debug, Clone)]
pub struct BiomeMinimapModel {
    /// Grid of biome cells.
    pub cells: [[BiomeCell; BIOME_MINIMAP_VIEW]; BIOME_MINIMAP_VIEW],
    /// Player chunk position.
    pub player_chunk: (i32, i32),
    /// Player position within chunk (0.0-1.0).
    pub player_offset: (f32, f32),
    /// Player facing direction in radians.
    pub player_direction: f32,
}

impl Default for BiomeMinimapModel {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeMinimapModel {
    /// Creates a new biome minimap model.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn new() -> Self {
        Self {
            cells: std::array::from_fn(|row| {
                std::array::from_fn(|col| {
                    let x = col as i32 - (BIOME_MINIMAP_VIEW as i32 / 2);
                    let y = row as i32 - (BIOME_MINIMAP_VIEW as i32 / 2);
                    BiomeCell::unexplored(x, y)
                })
            }),
            player_chunk: (0, 0),
            player_offset: (0.5, 0.5),
            player_direction: 0.0,
        }
    }

    /// Updates the grid centered on player position.
    #[allow(clippy::cast_possible_wrap)]
    pub fn update_center(&mut self, chunk_x: i32, chunk_y: i32) {
        self.player_chunk = (chunk_x, chunk_y);

        for (row, cell_row) in self.cells.iter_mut().enumerate() {
            for (col, cell) in cell_row.iter_mut().enumerate() {
                cell.chunk_x = chunk_x + col as i32 - (BIOME_MINIMAP_VIEW as i32 / 2);
                cell.chunk_y = chunk_y + row as i32 - (BIOME_MINIMAP_VIEW as i32 / 2);
            }
        }
    }

    /// Sets biome at grid position.
    pub fn set_biome(&mut self, row: usize, col: usize, biome: BiomeType) {
        if let Some(cell) = self.cells.get_mut(row).and_then(|r| r.get_mut(col)) {
            cell.biome = biome;
            cell.explored = biome != BiomeType::Unknown;
        }
    }

    /// Gets cell at grid position.
    #[must_use]
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&BiomeCell> {
        self.cells.get(row).and_then(|r| r.get(col))
    }
}

/// Configuration for the biome minimap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMinimapConfig {
    /// Minimap size in pixels.
    pub size: f32,
    /// Position offset from bottom-right corner.
    pub position_offset: (f32, f32),
    /// Background color.
    pub background_color: [u8; 4],
    /// Border color.
    pub border_color: [u8; 4],
    /// Player marker color.
    pub player_color: [u8; 4],
    /// Show chunk grid overlay.
    pub show_grid: bool,
    /// Grid line color.
    pub grid_color: [u8; 4],
    /// Show coordinates.
    pub show_coordinates: bool,
    /// Show compass.
    pub show_compass: bool,
}

impl Default for BiomeMinimapConfig {
    fn default() -> Self {
        Self {
            size: BIOME_MINIMAP_SIZE,
            position_offset: (10.0, 80.0),
            background_color: [20, 20, 20, 220],
            border_color: [80, 80, 80, 255],
            player_color: [255, 255, 255, 255],
            show_grid: true,
            grid_color: [100, 100, 100, 150],
            show_coordinates: true,
            show_compass: true,
        }
    }
}

/// Biome minimap widget.
#[derive(Debug)]
pub struct BiomeMinimap {
    /// Configuration.
    config: BiomeMinimapConfig,
}

impl Default for BiomeMinimap {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeMinimap {
    /// Creates a new biome minimap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BiomeMinimapConfig::default(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: BiomeMinimapConfig) -> Self {
        Self { config }
    }

    /// Toggles grid visibility.
    pub fn toggle_grid(&mut self) {
        self.config.show_grid = !self.config.show_grid;
    }

    /// Returns whether grid is shown.
    #[must_use]
    pub fn is_grid_visible(&self) -> bool {
        self.config.show_grid
    }

    /// Sets grid visibility.
    pub fn set_grid_visible(&mut self, visible: bool) {
        self.config.show_grid = visible;
    }

    /// Shows the biome minimap in the bottom-right corner.
    pub fn show(&self, ctx: &Context, model: &BiomeMinimapModel) {
        let screen_rect = ctx.screen_rect();
        let pos = Pos2::new(
            screen_rect.right() - self.config.size - self.config.position_offset.0,
            screen_rect.bottom() - self.config.size - self.config.position_offset.1,
        );

        egui::Area::new(Id::new("biome_minimap"))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                self.render_minimap(ui, model);
            });
    }

    /// Renders the minimap content.
    fn render_minimap(&self, ui: &mut Ui, model: &BiomeMinimapModel) {
        let size = Vec2::splat(self.config.size);
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return;
        }

        let painter = ui.painter();

        // Background
        let bg_color = Color32::from_rgba_unmultiplied(
            self.config.background_color[0],
            self.config.background_color[1],
            self.config.background_color[2],
            self.config.background_color[3],
        );
        painter.rect_filled(rect, Rounding::same(4.0), bg_color);

        // Calculate cell size
        let cell_size = self.config.size / BIOME_MINIMAP_VIEW as f32;

        // Render biome cells
        for row in 0..BIOME_MINIMAP_VIEW {
            for col in 0..BIOME_MINIMAP_VIEW {
                let cell = &model.cells[row][col];
                let cell_rect = Rect::from_min_size(
                    rect.min + Vec2::new(col as f32 * cell_size, row as f32 * cell_size),
                    Vec2::splat(cell_size),
                );

                // Biome color
                let biome_color = if cell.explored {
                    cell.biome.color()
                } else {
                    BiomeType::Unknown.color()
                };
                painter.rect_filled(cell_rect.shrink(0.5), Rounding::ZERO, biome_color);
            }
        }

        // Grid overlay (toggleable)
        if self.config.show_grid {
            let grid_color = Color32::from_rgba_unmultiplied(
                self.config.grid_color[0],
                self.config.grid_color[1],
                self.config.grid_color[2],
                self.config.grid_color[3],
            );
            for i in 1..BIOME_MINIMAP_VIEW {
                let offset = i as f32 * cell_size;
                // Vertical
                painter.line_segment(
                    [
                        rect.min + Vec2::new(offset, 0.0),
                        rect.min + Vec2::new(offset, self.config.size),
                    ],
                    Stroke::new(1.0, grid_color),
                );
                // Horizontal
                painter.line_segment(
                    [
                        rect.min + Vec2::new(0.0, offset),
                        rect.min + Vec2::new(self.config.size, offset),
                    ],
                    Stroke::new(1.0, grid_color),
                );
            }
        }

        // Player marker
        let center_offset = BIOME_MINIMAP_VIEW as f32 / 2.0;
        let player_x = rect.min.x + (center_offset + model.player_offset.0 - 0.5) * cell_size;
        let player_y = rect.min.y + (center_offset + model.player_offset.1 - 0.5) * cell_size;
        let player_pos = Pos2::new(player_x, player_y);

        let player_color = Color32::from_rgba_unmultiplied(
            self.config.player_color[0],
            self.config.player_color[1],
            self.config.player_color[2],
            self.config.player_color[3],
        );

        // Player direction arrow
        let arrow_len = cell_size * 0.5;
        let arrow_end = Pos2::new(
            player_pos.x + arrow_len * model.player_direction.cos(),
            player_pos.y - arrow_len * model.player_direction.sin(),
        );
        painter.line_segment([player_pos, arrow_end], Stroke::new(2.0, player_color));

        // Player dot
        painter.circle_filled(player_pos, cell_size * 0.25, player_color);

        // Border
        let border_color = Color32::from_rgba_unmultiplied(
            self.config.border_color[0],
            self.config.border_color[1],
            self.config.border_color[2],
            self.config.border_color[3],
        );
        painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(2.0, border_color));

        // Compass
        if self.config.show_compass {
            painter.text(
                rect.center_top() + Vec2::new(0.0, 8.0),
                egui::Align2::CENTER_CENTER,
                "N",
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
        }

        // Coordinates
        if self.config.show_coordinates {
            painter.text(
                rect.left_bottom() + Vec2::new(4.0, -4.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{},{}", model.player_chunk.0, model.player_chunk.1),
                egui::FontId::proportional(9.0),
                Color32::LIGHT_GRAY,
            );
        }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &BiomeMinimapConfig {
        &self.config
    }

    /// Sets the configuration.
    pub fn set_config(&mut self, config: BiomeMinimapConfig) {
        self.config = config;
    }
}

// ============================================================================
// T-33: Biome Debug Info
// ============================================================================

/// Biome information for debug display.
#[derive(Debug, Clone, Default)]
pub struct BiomeInfo {
    /// Current biome type.
    pub biome: BiomeType,
    /// Temperature value (0.0-1.0).
    pub temperature: f32,
    /// Humidity value (0.0-1.0).
    pub humidity: f32,
    /// Elevation value (0.0-1.0).
    pub elevation: f32,
    /// Raw noise values for debugging.
    pub noise_values: BiomeNoiseValues,
}

/// Raw noise values for biome debug display.
#[derive(Debug, Clone, Default)]
pub struct BiomeNoiseValues {
    /// Continental noise value.
    pub continentalness: f32,
    /// Erosion noise value.
    pub erosion: f32,
    /// Peaks and valleys noise value.
    pub peaks_valleys: f32,
    /// Temperature noise value.
    pub temperature: f32,
    /// Humidity noise value.
    pub humidity: f32,
}

/// Configuration for biome debug panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDebugConfig {
    /// Font size for panel text.
    pub font_size: f32,
    /// Show raw noise values.
    pub show_noise: bool,
}

impl Default for BiomeDebugConfig {
    fn default() -> Self {
        Self {
            font_size: 13.0,
            show_noise: true,
        }
    }
}

/// Biome debug panel widget.
#[derive(Debug)]
pub struct BiomeDebugPanel {
    /// Configuration.
    config: BiomeDebugConfig,
}

impl Default for BiomeDebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeDebugPanel {
    /// Creates a new biome debug panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BiomeDebugConfig::default(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: BiomeDebugConfig) -> Self {
        Self { config }
    }

    /// Renders the biome debug section into an existing UI.
    /// Call this from within an existing debug panel.
    pub fn render(&self, ui: &mut Ui, info: &BiomeInfo) {
        ui.label(
            RichText::new("Biome Info")
                .color(Color32::LIGHT_GREEN)
                .size(self.config.font_size),
        );

        // Current biome
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Biome:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(info.biome.display_name())
                    .color(info.biome.color())
                    .size(self.config.font_size),
            );
        });

        // Temperature
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Temp:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("{:.2}", info.temperature))
                    .color(temperature_color(info.temperature))
                    .size(self.config.font_size),
            );
        });

        // Humidity
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Humidity:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("{:.2}", info.humidity))
                    .color(humidity_color(info.humidity))
                    .size(self.config.font_size),
            );
        });

        // Elevation
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Elevation:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(format!("{:.2}", info.elevation))
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });

        // Raw noise values
        if self.config.show_noise {
            ui.add_space(4.0);
            ui.label(
                RichText::new("Noise Values")
                    .color(Color32::LIGHT_BLUE)
                    .size(self.config.font_size - 1.0),
            );

            let noise = &info.noise_values;
            ui.label(
                RichText::new(format!(
                    "C:{:.2} E:{:.2} PV:{:.2}",
                    noise.continentalness, noise.erosion, noise.peaks_valleys
                ))
                .color(Color32::GRAY)
                .size(self.config.font_size - 2.0),
            );
            ui.label(
                RichText::new(format!(
                    "T:{:.2} H:{:.2}",
                    noise.temperature, noise.humidity
                ))
                .color(Color32::GRAY)
                .size(self.config.font_size - 2.0),
            );
        }

        ui.add_space(4.0);
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &BiomeDebugConfig {
        &self.config
    }

    /// Sets whether to show noise values.
    pub fn set_show_noise(&mut self, show: bool) {
        self.config.show_noise = show;
    }
}

/// Returns color based on temperature value.
fn temperature_color(temp: f32) -> Color32 {
    if temp > 0.7 {
        Color32::from_rgb(255, 100, 100) // Hot - red
    } else if temp > 0.3 {
        Color32::from_rgb(255, 200, 100) // Warm - orange
    } else {
        Color32::from_rgb(100, 180, 255) // Cold - blue
    }
}

/// Returns color based on humidity value.
fn humidity_color(humidity: f32) -> Color32 {
    if humidity > 0.7 {
        Color32::from_rgb(100, 150, 255) // Wet - blue
    } else if humidity > 0.3 {
        Color32::from_rgb(150, 200, 150) // Normal - green
    } else {
        Color32::from_rgb(200, 180, 140) // Dry - tan
    }
}

// ============================================================================
// T-34: World Seed Display/Input
// ============================================================================

/// Actions emitted by the seed panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedPanelAction {
    /// Apply a new seed and regenerate world.
    ApplySeed(u64),
    /// Randomize the seed.
    RandomizeSeed,
}

/// Configuration for the seed panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedPanelConfig {
    /// Font size for panel text.
    pub font_size: f32,
    /// Show hex format alongside decimal.
    pub show_hex: bool,
}

impl Default for SeedPanelConfig {
    fn default() -> Self {
        Self {
            font_size: 13.0,
            show_hex: true,
        }
    }
}

/// World seed display and input panel.
#[derive(Debug)]
pub struct SeedPanel {
    /// Configuration.
    config: SeedPanelConfig,
    /// Input buffer for seed text.
    seed_input: String,
    /// Pending actions.
    actions: Vec<SeedPanelAction>,
}

impl Default for SeedPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SeedPanel {
    /// Creates a new seed panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SeedPanelConfig::default(),
            seed_input: String::new(),
            actions: Vec::new(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: SeedPanelConfig) -> Self {
        Self {
            config,
            seed_input: String::new(),
            actions: Vec::new(),
        }
    }

    /// Renders the seed panel section into an existing UI.
    pub fn render(&mut self, ui: &mut Ui, current_seed: u64) {
        ui.label(
            RichText::new("World Seed")
                .color(Color32::LIGHT_YELLOW)
                .size(self.config.font_size),
        );

        // Display current seed
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Current:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.label(
                RichText::new(current_seed.to_string())
                    .color(Color32::WHITE)
                    .size(self.config.font_size),
            );
        });

        // Show hex if enabled
        if self.config.show_hex {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Hex:")
                        .color(Color32::GRAY)
                        .size(self.config.font_size - 1.0),
                );
                ui.label(
                    RichText::new(format!("0x{current_seed:016X}"))
                        .color(Color32::LIGHT_GRAY)
                        .size(self.config.font_size - 1.0),
                );
            });
        }

        ui.add_space(4.0);

        // Input field
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("New Seed:")
                    .color(Color32::GRAY)
                    .size(self.config.font_size),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.seed_input)
                    .desired_width(120.0)
                    .hint_text("Enter seed..."),
            );
        });

        // Buttons
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(seed) = self.parse_seed() {
                    self.actions.push(SeedPanelAction::ApplySeed(seed));
                    self.seed_input.clear();
                }
            }
            if ui.button("Randomize").clicked() {
                self.actions.push(SeedPanelAction::RandomizeSeed);
            }
            if ui.button("Copy").clicked() {
                ui.output_mut(|o| o.copied_text = current_seed.to_string());
            }
        });

        ui.add_space(4.0);
    }

    /// Parses the seed input, supporting decimal and hex formats.
    fn parse_seed(&self) -> Option<u64> {
        let input = self.seed_input.trim();
        if input.is_empty() {
            return None;
        }

        // Try hex format
        if let Some(hex) = input
            .strip_prefix("0x")
            .or_else(|| input.strip_prefix("0X"))
        {
            return u64::from_str_radix(hex, 16).ok();
        }

        // Try decimal
        input.parse().ok()
    }

    /// Drains and returns pending actions.
    pub fn drain_actions(&mut self) -> Vec<SeedPanelAction> {
        std::mem::take(&mut self.actions)
    }

    /// Returns pending actions without draining.
    #[must_use]
    pub fn actions(&self) -> &[SeedPanelAction] {
        &self.actions
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &SeedPanelConfig {
        &self.config
    }

    /// Sets the input text (useful for pre-filling).
    pub fn set_input(&mut self, text: impl Into<String>) {
        self.seed_input = text.into();
    }
}

// ============================================================================
// T-35: Biome Legend Overlay
// ============================================================================

/// Configuration for the biome legend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeLegendConfig {
    /// Font size for legend text.
    pub font_size: f32,
    /// Size of color squares.
    pub color_square_size: f32,
    /// Padding from screen edge.
    pub padding: f32,
    /// Background opacity.
    pub background_opacity: f32,
    /// Toggle key name (e.g., "L").
    #[serde(default = "default_toggle_key")]
    pub toggle_key_name: String,
}

fn default_toggle_key() -> String {
    "L".to_string()
}

impl Default for BiomeLegendConfig {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            color_square_size: 14.0,
            padding: 10.0,
            background_opacity: 0.8,
            toggle_key_name: default_toggle_key(),
        }
    }
}

impl BiomeLegendConfig {
    /// Returns the toggle key.
    #[must_use]
    pub fn toggle_key(&self) -> Option<Key> {
        match self.toggle_key_name.to_uppercase().as_str() {
            "L" => Some(Key::L),
            "K" => Some(Key::K),
            "B" => Some(Key::B),
            "M" => Some(Key::M),
            _ => None,
        }
    }
}

/// Biome legend overlay widget.
#[derive(Debug)]
pub struct BiomeLegend {
    /// Configuration.
    config: BiomeLegendConfig,
    /// Whether the legend is visible.
    visible: bool,
}

impl Default for BiomeLegend {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeLegend {
    /// Creates a new biome legend.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BiomeLegendConfig::default(),
            visible: false,
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: BiomeLegendConfig) -> Self {
        Self {
            config,
            visible: false,
        }
    }

    /// Toggles visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Returns whether the legend is visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Shows the biome legend overlay.
    /// Handles key input for toggle.
    pub fn show(&mut self, ctx: &Context) {
        // Handle toggle key
        if let Some(key) = self.config.toggle_key() {
            if ctx.input(|i| i.key_pressed(key)) {
                self.toggle();
            }
        }

        if !self.visible {
            return;
        }

        let screen_rect = ctx.screen_rect();

        egui::Area::new(Id::new("biome_legend"))
            .fixed_pos(Pos2::new(
                self.config.padding,
                screen_rect.bottom() - 200.0, // Position in bottom-left
            ))
            .show(ctx, |ui| {
                let bg_alpha = (self.config.background_opacity * 255.0) as u8;
                egui::Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, bg_alpha))
                    .inner_margin(8.0)
                    .rounding(Rounding::same(4.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "Biome Legend ({})",
                                self.config.toggle_key_name
                            ))
                            .color(Color32::WHITE)
                            .size(self.config.font_size + 2.0),
                        );
                        ui.separator();

                        for biome in BiomeType::all() {
                            ui.horizontal(|ui| {
                                // Color square
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::splat(self.config.color_square_size),
                                    egui::Sense::hover(),
                                );
                                ui.painter()
                                    .rect_filled(rect, Rounding::same(2.0), biome.color());

                                // Biome name
                                ui.label(
                                    RichText::new(biome.display_name())
                                        .color(Color32::LIGHT_GRAY)
                                        .size(self.config.font_size),
                                );
                            });
                        }
                    });
            });
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &BiomeLegendConfig {
        &self.config
    }

    /// Sets the configuration.
    pub fn set_config(&mut self, config: BiomeLegendConfig) {
        self.config = config;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // T-32 Tests
    #[test]
    fn test_biome_type_colors() {
        assert_eq!(
            BiomeType::Forest.color(),
            Color32::from_rgb(0x2d, 0x5a, 0x1d)
        );
        assert_eq!(
            BiomeType::Desert.color(),
            Color32::from_rgb(0xc4, 0xa3, 0x5a)
        );
        assert_eq!(BiomeType::Lake.color(), Color32::from_rgb(0x3a, 0x7c, 0xa5));
        assert_eq!(
            BiomeType::Plains.color(),
            Color32::from_rgb(0x7c, 0xb3, 0x42)
        );
        assert_eq!(
            BiomeType::Mountain.color(),
            Color32::from_rgb(0x7a, 0x7a, 0x7a)
        );
        assert_eq!(
            BiomeType::Swamp.color(),
            Color32::from_rgb(0x4a, 0x5a, 0x23)
        );
    }

    #[test]
    fn test_biome_type_all() {
        let all = BiomeType::all();
        assert_eq!(all.len(), 6);
        assert!(!all.contains(&BiomeType::Unknown));
    }

    #[test]
    fn test_biome_type_all_with_unknown() {
        let all = BiomeType::all_with_unknown();
        assert_eq!(all.len(), 7);
        assert!(all.contains(&BiomeType::Unknown));
    }

    #[test]
    fn test_biome_cell_new() {
        let cell = BiomeCell::new(5, -3, BiomeType::Forest);
        assert_eq!(cell.chunk_x, 5);
        assert_eq!(cell.chunk_y, -3);
        assert_eq!(cell.biome, BiomeType::Forest);
        assert!(cell.explored);
    }

    #[test]
    fn test_biome_cell_unexplored() {
        let cell = BiomeCell::unexplored(0, 0);
        assert!(!cell.explored);
        assert_eq!(cell.biome, BiomeType::Unknown);
    }

    #[test]
    fn test_biome_minimap_model_new() {
        let model = BiomeMinimapModel::new();
        assert_eq!(model.player_chunk, (0, 0));
        assert_eq!(model.cells.len(), BIOME_MINIMAP_VIEW);
    }

    #[test]
    fn test_biome_minimap_model_update_center() {
        let mut model = BiomeMinimapModel::new();
        model.update_center(10, 20);
        assert_eq!(model.player_chunk, (10, 20));

        let center = model.get_cell(3, 3).unwrap();
        assert_eq!(center.chunk_x, 10);
        assert_eq!(center.chunk_y, 20);
    }

    #[test]
    fn test_biome_minimap_model_set_biome() {
        let mut model = BiomeMinimapModel::new();
        model.set_biome(0, 0, BiomeType::Forest);

        let cell = model.get_cell(0, 0).unwrap();
        assert_eq!(cell.biome, BiomeType::Forest);
        assert!(cell.explored);
    }

    #[test]
    fn test_biome_minimap_config_defaults() {
        let config = BiomeMinimapConfig::default();
        assert!((config.size - BIOME_MINIMAP_SIZE).abs() < f32::EPSILON);
        assert!(config.show_grid);
        assert!(config.show_coordinates);
    }

    #[test]
    fn test_biome_minimap_toggle_grid() {
        let mut minimap = BiomeMinimap::new();
        assert!(minimap.is_grid_visible());

        minimap.toggle_grid();
        assert!(!minimap.is_grid_visible());

        minimap.toggle_grid();
        assert!(minimap.is_grid_visible());
    }

    // T-33 Tests
    #[test]
    fn test_biome_info_default() {
        let info = BiomeInfo::default();
        assert_eq!(info.biome, BiomeType::Unknown);
        assert_eq!(info.temperature, 0.0);
        assert_eq!(info.humidity, 0.0);
    }

    #[test]
    fn test_biome_noise_values_default() {
        let noise = BiomeNoiseValues::default();
        assert_eq!(noise.continentalness, 0.0);
        assert_eq!(noise.erosion, 0.0);
    }

    #[test]
    fn test_biome_debug_config_defaults() {
        let config = BiomeDebugConfig::default();
        assert_eq!(config.font_size, 13.0);
        assert!(config.show_noise);
    }

    #[test]
    fn test_temperature_color() {
        let hot = temperature_color(0.9);
        let warm = temperature_color(0.5);
        let cold = temperature_color(0.1);
        assert_ne!(hot, warm);
        assert_ne!(warm, cold);
    }

    #[test]
    fn test_humidity_color() {
        let wet = humidity_color(0.9);
        let normal = humidity_color(0.5);
        let dry = humidity_color(0.1);
        assert_ne!(wet, normal);
        assert_ne!(normal, dry);
    }

    // T-34 Tests
    #[test]
    fn test_seed_panel_new() {
        let panel = SeedPanel::new();
        assert!(panel.actions.is_empty());
        assert!(panel.seed_input.is_empty());
    }

    #[test]
    fn test_seed_panel_parse_decimal() {
        let mut panel = SeedPanel::new();
        panel.seed_input = "12345".to_string();
        assert_eq!(panel.parse_seed(), Some(12345));
    }

    #[test]
    fn test_seed_panel_parse_hex() {
        let mut panel = SeedPanel::new();
        panel.seed_input = "0xABCD".to_string();
        assert_eq!(panel.parse_seed(), Some(0xABCD));

        panel.seed_input = "0X1234".to_string();
        assert_eq!(panel.parse_seed(), Some(0x1234));
    }

    #[test]
    fn test_seed_panel_parse_empty() {
        let panel = SeedPanel::new();
        assert_eq!(panel.parse_seed(), None);
    }

    #[test]
    fn test_seed_panel_config_defaults() {
        let config = SeedPanelConfig::default();
        assert_eq!(config.font_size, 13.0);
        assert!(config.show_hex);
    }

    #[test]
    fn test_seed_panel_action_equality() {
        let a1 = SeedPanelAction::ApplySeed(123);
        let a2 = SeedPanelAction::ApplySeed(123);
        let a3 = SeedPanelAction::ApplySeed(456);
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);

        assert_eq!(
            SeedPanelAction::RandomizeSeed,
            SeedPanelAction::RandomizeSeed
        );
    }

    #[test]
    fn test_seed_panel_drain_actions() {
        let mut panel = SeedPanel::new();
        panel.actions.push(SeedPanelAction::RandomizeSeed);
        panel.actions.push(SeedPanelAction::ApplySeed(100));

        let drained = panel.drain_actions();
        assert_eq!(drained.len(), 2);
        assert!(panel.actions.is_empty());
    }

    // T-35 Tests
    #[test]
    fn test_biome_legend_new() {
        let legend = BiomeLegend::new();
        assert!(!legend.is_visible());
    }

    #[test]
    fn test_biome_legend_toggle() {
        let mut legend = BiomeLegend::new();
        assert!(!legend.is_visible());

        legend.toggle();
        assert!(legend.is_visible());

        legend.toggle();
        assert!(!legend.is_visible());
    }

    #[test]
    fn test_biome_legend_set_visible() {
        let mut legend = BiomeLegend::new();
        legend.set_visible(true);
        assert!(legend.is_visible());

        legend.set_visible(false);
        assert!(!legend.is_visible());
    }

    #[test]
    fn test_biome_legend_config_defaults() {
        let config = BiomeLegendConfig::default();
        assert_eq!(config.font_size, 12.0);
        assert_eq!(config.toggle_key_name, "L");
        assert_eq!(config.toggle_key(), Some(Key::L));
    }
}
