//! Chunk-based Minimap UI for displaying explored area.
//!
//! This module provides:
//! - 5x5 chunk grid around player
//! - Player position indicator
//! - Chunk colors based on terrain type
//! - Bottom-right positioning

use egui::{Color32, Context, Id, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Number of chunks visible on each axis.
pub const MINIMAP_CHUNK_VIEW: usize = 5;

/// Default minimap size in pixels.
pub const MINIMAP_SIZE: f32 = 120.0;

/// Terrain types for chunk coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChunkTerrainType {
    /// Unexplored/unknown area.
    #[default]
    Unexplored,
    /// Grass/plains terrain.
    Grass,
    /// Forest terrain.
    Forest,
    /// Desert terrain.
    Desert,
    /// Snow/tundra terrain.
    Snow,
    /// Water/ocean terrain.
    Water,
    /// Mountain terrain.
    Mountain,
    /// Swamp terrain.
    Swamp,
    /// Cave/underground.
    Cave,
    /// Village/settlement.
    Village,
}

impl ChunkTerrainType {
    /// Returns the display color for this terrain type.
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            ChunkTerrainType::Unexplored => Color32::from_gray(40),
            ChunkTerrainType::Grass => Color32::from_rgb(80, 160, 80),
            ChunkTerrainType::Forest => Color32::from_rgb(30, 100, 30),
            ChunkTerrainType::Desert => Color32::from_rgb(210, 180, 100),
            ChunkTerrainType::Snow => Color32::from_rgb(230, 230, 240),
            ChunkTerrainType::Water => Color32::from_rgb(60, 120, 200),
            ChunkTerrainType::Mountain => Color32::from_rgb(120, 110, 100),
            ChunkTerrainType::Swamp => Color32::from_rgb(80, 100, 60),
            ChunkTerrainType::Cave => Color32::from_rgb(50, 50, 60),
            ChunkTerrainType::Village => Color32::from_rgb(180, 140, 100),
        }
    }

    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            ChunkTerrainType::Unexplored => "Unexplored",
            ChunkTerrainType::Grass => "Plains",
            ChunkTerrainType::Forest => "Forest",
            ChunkTerrainType::Desert => "Desert",
            ChunkTerrainType::Snow => "Snow",
            ChunkTerrainType::Water => "Water",
            ChunkTerrainType::Mountain => "Mountain",
            ChunkTerrainType::Swamp => "Swamp",
            ChunkTerrainType::Cave => "Cave",
            ChunkTerrainType::Village => "Village",
        }
    }

    /// Returns all terrain types.
    #[must_use]
    pub fn all() -> &'static [ChunkTerrainType] {
        &[
            ChunkTerrainType::Unexplored,
            ChunkTerrainType::Grass,
            ChunkTerrainType::Forest,
            ChunkTerrainType::Desert,
            ChunkTerrainType::Snow,
            ChunkTerrainType::Water,
            ChunkTerrainType::Mountain,
            ChunkTerrainType::Swamp,
            ChunkTerrainType::Cave,
            ChunkTerrainType::Village,
        ]
    }
}

/// Data for a single chunk in the minimap.
#[derive(Debug, Clone, Default)]
pub struct ChunkData {
    /// Chunk coordinates.
    pub x: i32,
    /// Chunk Y coordinate.
    pub y: i32,
    /// Terrain type.
    pub terrain: ChunkTerrainType,
    /// Whether this chunk has been explored.
    pub explored: bool,
    /// Special marker (quest, danger, etc.).
    pub marker: Option<ChunkMarker>,
}

impl ChunkData {
    /// Creates a new chunk data.
    #[must_use]
    pub fn new(x: i32, y: i32, terrain: ChunkTerrainType) -> Self {
        Self {
            x,
            y,
            terrain,
            explored: true,
            marker: None,
        }
    }

    /// Creates an unexplored chunk.
    #[must_use]
    pub fn unexplored(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            terrain: ChunkTerrainType::Unexplored,
            explored: false,
            marker: None,
        }
    }
}

/// Special markers on chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkMarker {
    /// Quest objective.
    Quest,
    /// Danger/enemy.
    Danger,
    /// Point of interest.
    Interest,
    /// Waypoint.
    Waypoint,
}

impl ChunkMarker {
    /// Returns the marker color.
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            ChunkMarker::Quest => Color32::GOLD,
            ChunkMarker::Danger => Color32::RED,
            ChunkMarker::Interest => Color32::from_rgb(100, 200, 255),
            ChunkMarker::Waypoint => Color32::WHITE,
        }
    }

    /// Returns the marker icon.
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            ChunkMarker::Quest => "!",
            ChunkMarker::Danger => "⚠",
            ChunkMarker::Interest => "?",
            ChunkMarker::Waypoint => "★",
        }
    }
}

/// Minimap chunk data model.
#[derive(Debug, Clone)]
pub struct ChunkMinimapModel {
    /// 5x5 grid of chunks around player.
    pub chunks: [[ChunkData; MINIMAP_CHUNK_VIEW]; MINIMAP_CHUNK_VIEW],
    /// Player position within current chunk (0.0 - 1.0).
    pub player_offset: (f32, f32),
    /// Player's current chunk coordinates.
    pub player_chunk: (i32, i32),
    /// Player facing direction in radians.
    pub player_direction: f32,
}

impl Default for ChunkMinimapModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkMinimapModel {
    /// Creates a new minimap model.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn new() -> Self {
        Self {
            chunks: std::array::from_fn(|row| {
                std::array::from_fn(|col| {
                    let x = col as i32 - (MINIMAP_CHUNK_VIEW as i32 / 2);
                    let y = row as i32 - (MINIMAP_CHUNK_VIEW as i32 / 2);
                    ChunkData::unexplored(x, y)
                })
            }),
            player_offset: (0.5, 0.5),
            player_chunk: (0, 0),
            player_direction: 0.0,
        }
    }

    /// Gets a chunk at the given grid position.
    #[must_use]
    pub fn get_chunk(&self, row: usize, col: usize) -> Option<&ChunkData> {
        self.chunks.get(row).and_then(|r| r.get(col))
    }

    /// Gets a mutable chunk at the given grid position.
    pub fn get_chunk_mut(&mut self, row: usize, col: usize) -> Option<&mut ChunkData> {
        self.chunks.get_mut(row).and_then(|r| r.get_mut(col))
    }

    /// Updates the chunks grid centered on player position.
    #[allow(clippy::cast_possible_wrap)]
    pub fn update_center(&mut self, player_chunk_x: i32, player_chunk_y: i32) {
        self.player_chunk = (player_chunk_x, player_chunk_y);

        for (row, chunk_row) in self.chunks.iter_mut().enumerate() {
            for (col, chunk) in chunk_row.iter_mut().enumerate() {
                chunk.x = player_chunk_x + col as i32 - (MINIMAP_CHUNK_VIEW as i32 / 2);
                chunk.y = player_chunk_y + row as i32 - (MINIMAP_CHUNK_VIEW as i32 / 2);
            }
        }
    }

    /// Sets chunk terrain at grid position.
    pub fn set_terrain(&mut self, row: usize, col: usize, terrain: ChunkTerrainType) {
        if let Some(chunk) = self.get_chunk_mut(row, col) {
            chunk.terrain = terrain;
            chunk.explored = terrain != ChunkTerrainType::Unexplored;
        }
    }

    /// Sets a marker on a chunk.
    pub fn set_marker(&mut self, row: usize, col: usize, marker: Option<ChunkMarker>) {
        if let Some(chunk) = self.get_chunk_mut(row, col) {
            chunk.marker = marker;
        }
    }
}

/// Configuration for the chunk minimap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMinimapConfig {
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
    /// Show chunk coordinates.
    pub show_coordinates: bool,
    /// Show compass.
    pub show_compass: bool,
}

impl Default for ChunkMinimapConfig {
    fn default() -> Self {
        Self {
            size: MINIMAP_SIZE,
            position_offset: (10.0, 80.0), // Above hotbar
            background_color: [20, 20, 20, 220],
            border_color: [80, 80, 80, 255],
            player_color: [255, 255, 255, 255],
            show_coordinates: true,
            show_compass: true,
        }
    }
}

/// Chunk minimap widget.
#[derive(Debug)]
pub struct ChunkMinimap {
    /// Configuration.
    config: ChunkMinimapConfig,
}

impl Default for ChunkMinimap {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkMinimap {
    /// Creates a new chunk minimap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ChunkMinimapConfig::default(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: ChunkMinimapConfig) -> Self {
        Self { config }
    }

    /// Shows the minimap in the bottom-right corner.
    pub fn show(&self, ctx: &Context, model: &ChunkMinimapModel) {
        let screen_rect = ctx.screen_rect();
        let pos = Pos2::new(
            screen_rect.right() - self.config.size - self.config.position_offset.0,
            screen_rect.bottom() - self.config.size - self.config.position_offset.1,
        );

        egui::Area::new(Id::new("chunk_minimap"))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                self.render_minimap(ui, model);
            });
    }

    /// Renders the minimap content.
    fn render_minimap(&self, ui: &mut Ui, model: &ChunkMinimapModel) {
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

        // Calculate chunk cell size
        let cell_size = self.config.size / MINIMAP_CHUNK_VIEW as f32;

        // Render chunks
        for row in 0..MINIMAP_CHUNK_VIEW {
            for col in 0..MINIMAP_CHUNK_VIEW {
                let chunk = &model.chunks[row][col];
                let cell_rect = Rect::from_min_size(
                    rect.min + Vec2::new(col as f32 * cell_size, row as f32 * cell_size),
                    Vec2::splat(cell_size),
                );

                // Chunk terrain color
                let terrain_color = if chunk.explored {
                    chunk.terrain.color()
                } else {
                    ChunkTerrainType::Unexplored.color()
                };
                painter.rect_filled(cell_rect.shrink(0.5), Rounding::ZERO, terrain_color);

                // Chunk marker
                if let Some(marker) = chunk.marker {
                    painter.text(
                        cell_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        marker.icon(),
                        egui::FontId::proportional(cell_size * 0.6),
                        marker.color(),
                    );
                }
            }
        }

        // Grid lines
        let grid_color = Color32::from_gray(60);
        for i in 1..MINIMAP_CHUNK_VIEW {
            let offset = i as f32 * cell_size;
            // Vertical
            painter.line_segment(
                [
                    rect.min + Vec2::new(offset, 0.0),
                    rect.min + Vec2::new(offset, self.config.size),
                ],
                Stroke::new(0.5, grid_color),
            );
            // Horizontal
            painter.line_segment(
                [
                    rect.min + Vec2::new(0.0, offset),
                    rect.min + Vec2::new(self.config.size, offset),
                ],
                Stroke::new(0.5, grid_color),
            );
        }

        // Player marker (white dot in center with offset)
        let center_offset = MINIMAP_CHUNK_VIEW as f32 / 2.0;
        let player_x = rect.min.x + (center_offset + model.player_offset.0 - 0.5) * cell_size;
        let player_y = rect.min.y + (center_offset + model.player_offset.1 - 0.5) * cell_size;
        let player_pos = Pos2::new(player_x, player_y);

        let player_color = Color32::from_rgba_unmultiplied(
            self.config.player_color[0],
            self.config.player_color[1],
            self.config.player_color[2],
            self.config.player_color[3],
        );

        // Player direction indicator (arrow)
        let arrow_len = cell_size * 0.4;
        let arrow_end = Pos2::new(
            player_pos.x + arrow_len * model.player_direction.cos(),
            player_pos.y - arrow_len * model.player_direction.sin(),
        );
        painter.line_segment([player_pos, arrow_end], Stroke::new(2.0, player_color));

        // Player dot
        painter.circle_filled(player_pos, cell_size * 0.2, player_color);

        // Border
        let border_color = Color32::from_rgba_unmultiplied(
            self.config.border_color[0],
            self.config.border_color[1],
            self.config.border_color[2],
            self.config.border_color[3],
        );
        painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(2.0, border_color));

        // Compass (N indicator)
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
    pub fn config(&self) -> &ChunkMinimapConfig {
        &self.config
    }

    /// Sets the configuration.
    pub fn set_config(&mut self, config: ChunkMinimapConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_terrain_type_color() {
        let grass = ChunkTerrainType::Grass.color();
        let water = ChunkTerrainType::Water.color();
        assert_ne!(grass, water);

        let unexplored = ChunkTerrainType::Unexplored.color();
        assert_eq!(unexplored, Color32::from_gray(40));
    }

    #[test]
    fn test_chunk_terrain_type_all() {
        let all = ChunkTerrainType::all();
        assert_eq!(all.len(), 10);
        assert!(all.contains(&ChunkTerrainType::Grass));
        assert!(all.contains(&ChunkTerrainType::Water));
    }

    #[test]
    fn test_chunk_data_new() {
        let chunk = ChunkData::new(5, -3, ChunkTerrainType::Forest);
        assert_eq!(chunk.x, 5);
        assert_eq!(chunk.y, -3);
        assert_eq!(chunk.terrain, ChunkTerrainType::Forest);
        assert!(chunk.explored);
    }

    #[test]
    fn test_chunk_data_unexplored() {
        let chunk = ChunkData::unexplored(0, 0);
        assert!(!chunk.explored);
        assert_eq!(chunk.terrain, ChunkTerrainType::Unexplored);
    }

    #[test]
    fn test_chunk_marker_color() {
        assert_eq!(ChunkMarker::Quest.color(), Color32::GOLD);
        assert_eq!(ChunkMarker::Danger.color(), Color32::RED);
    }

    #[test]
    fn test_chunk_minimap_model_new() {
        let model = ChunkMinimapModel::new();
        assert_eq!(model.player_chunk, (0, 0));
        assert_eq!(model.chunks.len(), MINIMAP_CHUNK_VIEW);
        assert_eq!(model.chunks[0].len(), MINIMAP_CHUNK_VIEW);
    }

    #[test]
    fn test_chunk_minimap_model_get_chunk() {
        let model = ChunkMinimapModel::new();
        let center = model.get_chunk(2, 2);
        assert!(center.is_some());

        let out_of_bounds = model.get_chunk(10, 10);
        assert!(out_of_bounds.is_none());
    }

    #[test]
    fn test_chunk_minimap_model_update_center() {
        let mut model = ChunkMinimapModel::new();
        model.update_center(10, 20);

        assert_eq!(model.player_chunk, (10, 20));

        // Center chunk should be at player position
        let center = model.get_chunk(2, 2).unwrap();
        assert_eq!(center.x, 10);
        assert_eq!(center.y, 20);
    }

    #[test]
    fn test_chunk_minimap_model_set_terrain() {
        let mut model = ChunkMinimapModel::new();
        model.set_terrain(0, 0, ChunkTerrainType::Forest);

        let chunk = model.get_chunk(0, 0).unwrap();
        assert_eq!(chunk.terrain, ChunkTerrainType::Forest);
        assert!(chunk.explored);
    }

    #[test]
    fn test_chunk_minimap_model_set_marker() {
        let mut model = ChunkMinimapModel::new();
        model.set_marker(1, 1, Some(ChunkMarker::Quest));

        let chunk = model.get_chunk(1, 1).unwrap();
        assert_eq!(chunk.marker, Some(ChunkMarker::Quest));
    }

    #[test]
    fn test_chunk_minimap_config_defaults() {
        let config = ChunkMinimapConfig::default();
        assert!((config.size - MINIMAP_SIZE).abs() < f32::EPSILON);
        assert!(config.show_coordinates);
        assert!(config.show_compass);
    }

    #[test]
    fn test_chunk_minimap_new() {
        let minimap = ChunkMinimap::new();
        assert!((minimap.config.size - MINIMAP_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_minimap_constants() {
        assert_eq!(MINIMAP_CHUNK_VIEW, 5);
        assert!((MINIMAP_SIZE - 120.0).abs() < f32::EPSILON);
    }
}
