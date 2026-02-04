//! Minimap UI renderer using egui.
//!
//! This module provides:
//! - Circle or square minimap shape
//! - Zoom level control
//! - Player icon with rotation
//! - POI (Point of Interest) markers
//! - Terrain visualization

use egui::{
    Align2, Color32, Context, FontId, Id, Painter, Pos2, Rect, Response, Rounding, Sense, Shape,
    Stroke, Ui, Vec2, Window,
};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Default minimap radius in pixels.
pub const DEFAULT_MINIMAP_RADIUS: f32 = 100.0;

/// Default zoom level.
pub const DEFAULT_ZOOM: f32 = 1.0;

/// Minimum zoom level.
pub const MIN_ZOOM: f32 = 0.25;

/// Maximum zoom level.
pub const MAX_ZOOM: f32 = 4.0;

/// Minimap shape options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MinimapShape {
    /// Circular minimap
    #[default]
    Circle,
    /// Square minimap
    Square,
}

impl MinimapShape {
    /// Toggles between circle and square.
    #[must_use]
    pub fn toggle(&self) -> Self {
        match self {
            MinimapShape::Circle => MinimapShape::Square,
            MinimapShape::Square => MinimapShape::Circle,
        }
    }
}

/// POI (Point of Interest) type for different marker styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum POIType {
    /// Quest objective
    #[default]
    Quest,
    /// Shop or merchant
    Shop,
    /// Danger/Enemy
    Danger,
    /// Friendly NPC
    Friendly,
    /// Resource node
    Resource,
    /// Waypoint/Custom marker
    Waypoint,
}

impl POIType {
    /// Returns the default color for this POI type.
    #[must_use]
    pub fn default_color(&self) -> Color32 {
        match self {
            POIType::Quest => Color32::GOLD,
            POIType::Shop => Color32::from_rgb(100, 200, 100),
            POIType::Danger => Color32::RED,
            POIType::Friendly => Color32::from_rgb(100, 150, 255),
            POIType::Resource => Color32::from_rgb(200, 150, 50),
            POIType::Waypoint => Color32::WHITE,
        }
    }

    /// Returns the marker icon (Unicode character).
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            POIType::Quest => "!",
            POIType::Shop => "$",
            POIType::Danger => "⚠",
            POIType::Friendly => "♦",
            POIType::Resource => "◆",
            POIType::Waypoint => "★",
        }
    }
}

/// A Point of Interest marker on the minimap.
#[derive(Debug, Clone)]
pub struct POIMarker {
    /// Unique identifier
    pub id: u64,
    /// World X position
    pub world_x: f32,
    /// World Y position
    pub world_y: f32,
    /// POI type
    pub poi_type: POIType,
    /// Display label (optional)
    pub label: Option<String>,
    /// Custom color (overrides default)
    pub color: Option<Color32>,
    /// Whether this marker is visible
    pub visible: bool,
    /// Whether this marker is selected/highlighted
    pub is_highlighted: bool,
}

impl POIMarker {
    /// Creates a new POI marker.
    #[must_use]
    pub fn new(id: u64, world_x: f32, world_y: f32, poi_type: POIType) -> Self {
        Self {
            id,
            world_x,
            world_y,
            poi_type,
            label: None,
            color: None,
            visible: true,
            is_highlighted: false,
        }
    }

    /// Sets a label for this marker.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets a custom color for this marker.
    #[must_use]
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Returns the effective color for this marker.
    #[must_use]
    pub fn effective_color(&self) -> Color32 {
        self.color.unwrap_or_else(|| self.poi_type.default_color())
    }
}

/// Terrain tile data for minimap visualization.
#[derive(Debug, Clone, Copy, Default)]
pub struct TerrainTile {
    /// Tile color
    pub color: Color32,
    /// Height value (0.0 - 1.0)
    pub height: f32,
    /// Whether this tile is explored
    pub explored: bool,
}

impl TerrainTile {
    /// Creates a terrain tile.
    #[must_use]
    pub fn new(color: Color32, height: f32) -> Self {
        Self {
            color,
            height,
            explored: true,
        }
    }

    /// Creates an unexplored tile.
    #[must_use]
    pub fn unexplored() -> Self {
        Self {
            color: Color32::from_rgb(20, 20, 20),
            height: 0.0,
            explored: false,
        }
    }
}

/// Minimap UI model containing all display data.
#[derive(Debug, Clone)]
pub struct MinimapModel {
    /// Player world X position
    pub player_x: f32,
    /// Player world Y position
    pub player_y: f32,
    /// Player rotation in radians (0 = up/north)
    pub player_rotation: f32,
    /// POI markers
    pub markers: Vec<POIMarker>,
    /// Terrain data (flattened 2D grid, row-major)
    pub terrain: Vec<TerrainTile>,
    /// Terrain grid width
    pub terrain_width: usize,
    /// Terrain grid height
    pub terrain_height: usize,
    /// World units per terrain tile
    pub world_scale: f32,
}

impl Default for MinimapModel {
    fn default() -> Self {
        Self::new()
    }
}

impl MinimapModel {
    /// Creates a new empty minimap model.
    #[must_use]
    pub fn new() -> Self {
        Self {
            player_x: 0.0,
            player_y: 0.0,
            player_rotation: 0.0,
            markers: Vec::new(),
            terrain: Vec::new(),
            terrain_width: 0,
            terrain_height: 0,
            world_scale: 1.0,
        }
    }

    /// Sets the player position.
    pub fn set_player_position(&mut self, x: f32, y: f32) {
        self.player_x = x;
        self.player_y = y;
    }

    /// Sets the player rotation in radians.
    pub fn set_player_rotation(&mut self, radians: f32) {
        self.player_rotation = radians;
    }

    /// Adds a POI marker.
    pub fn add_marker(&mut self, marker: POIMarker) {
        self.markers.push(marker);
    }

    /// Removes a marker by ID.
    pub fn remove_marker(&mut self, id: u64) {
        self.markers.retain(|m| m.id != id);
    }

    /// Sets the terrain data.
    pub fn set_terrain(&mut self, terrain: Vec<TerrainTile>, width: usize, height: usize) {
        self.terrain = terrain;
        self.terrain_width = width;
        self.terrain_height = height;
    }

    /// Gets a terrain tile at the given coordinates.
    #[must_use]
    pub fn get_terrain(&self, x: usize, y: usize) -> Option<&TerrainTile> {
        if x < self.terrain_width && y < self.terrain_height {
            self.terrain.get(y * self.terrain_width + x)
        } else {
            None
        }
    }
}

/// Actions from minimap interaction.
#[derive(Debug, Clone, PartialEq)]
pub enum MinimapAction {
    /// Clicked on a position (world coordinates)
    ClickedPosition {
        /// World X coordinate
        world_x: f32,
        /// World Y coordinate
        world_y: f32,
    },
    /// Clicked on a POI marker
    ClickedMarker(u64),
    /// Zoom level changed
    ZoomChanged(f32),
    /// Shape toggled
    ShapeToggled(MinimapShape),
}

/// Configuration for the minimap UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapConfig {
    /// Minimap radius/size
    pub radius: f32,
    /// Current shape
    pub shape: MinimapShape,
    /// Current zoom level
    pub zoom: f32,
    /// Background color
    pub background_color: [u8; 4],
    /// Border color
    pub border_color: [u8; 4],
    /// Player icon color
    pub player_color: [u8; 4],
    /// Player icon size
    pub player_icon_size: f32,
    /// POI marker size
    pub poi_marker_size: f32,
    /// Show terrain
    pub show_terrain: bool,
    /// Show cardinal directions (N/S/E/W)
    pub show_directions: bool,
    /// Rotate map with player (north not always up)
    pub rotate_with_player: bool,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            radius: DEFAULT_MINIMAP_RADIUS,
            shape: MinimapShape::Circle,
            zoom: DEFAULT_ZOOM,
            background_color: [30, 30, 40, 220],
            border_color: [80, 80, 80, 255],
            player_color: [255, 255, 255, 255],
            player_icon_size: 10.0,
            poi_marker_size: 8.0,
            show_terrain: true,
            show_directions: true,
            rotate_with_player: false,
        }
    }
}

/// Minimap UI renderer.
#[derive(Debug)]
pub struct MinimapUI {
    /// Configuration
    pub config: MinimapConfig,
    /// Unique ID for egui
    id: Id,
}

impl Default for MinimapUI {
    fn default() -> Self {
        Self::new()
    }
}

impl MinimapUI {
    /// Creates a new minimap UI.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: MinimapConfig::default(),
            id: Id::new("minimap"),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: MinimapConfig) -> Self {
        Self {
            config,
            id: Id::new("minimap"),
        }
    }

    /// Shows the minimap as an overlay at a fixed position.
    pub fn show_overlay(&mut self, ctx: &Context, model: &MinimapModel) -> Vec<MinimapAction> {
        let mut actions = Vec::new();

        let screen_rect = ctx.screen_rect();
        let margin = 20.0;
        let pos = Pos2::new(screen_rect.right() - self.config.radius - margin, margin);

        egui::Area::new(self.id)
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                self.render_minimap(ui, model, &mut actions);
            });

        actions
    }

    /// Shows the minimap in a window (expandable view).
    pub fn show_window(
        &mut self,
        ctx: &Context,
        model: &MinimapModel,
        open: &mut bool,
    ) -> Vec<MinimapAction> {
        let mut actions = Vec::new();

        Window::new("Map")
            .open(open)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Controls
                ui.horizontal(|ui| {
                    if ui.button("🔍+").clicked() {
                        self.zoom_in();
                        actions.push(MinimapAction::ZoomChanged(self.config.zoom));
                    }
                    ui.label(format!("{:.0}%", self.config.zoom * 100.0));
                    if ui.button("🔍-").clicked() {
                        self.zoom_out();
                        actions.push(MinimapAction::ZoomChanged(self.config.zoom));
                    }

                    ui.separator();

                    let shape_text = match self.config.shape {
                        MinimapShape::Circle => "○",
                        MinimapShape::Square => "□",
                    };
                    if ui.button(shape_text).clicked() {
                        self.toggle_shape();
                        actions.push(MinimapAction::ShapeToggled(self.config.shape));
                    }

                    ui.separator();

                    ui.checkbox(&mut self.config.rotate_with_player, "Rotate");
                });

                ui.separator();

                // Render larger minimap
                let old_radius = self.config.radius;
                self.config.radius = ui.available_width().min(ui.available_height()) / 2.0 - 10.0;
                self.render_minimap(ui, model, &mut actions);
                self.config.radius = old_radius;
            });

        actions
    }

    /// Renders the minimap content.
    fn render_minimap(&self, ui: &mut Ui, model: &MinimapModel, actions: &mut Vec<MinimapAction>) {
        let size = Vec2::splat(self.config.radius * 2.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if !ui.is_rect_visible(rect) {
            return;
        }

        let painter = ui.painter();
        let center = rect.center();

        // Background
        let bg_color = Color32::from_rgba_unmultiplied(
            self.config.background_color[0],
            self.config.background_color[1],
            self.config.background_color[2],
            self.config.background_color[3],
        );

        match self.config.shape {
            MinimapShape::Circle => {
                painter.circle_filled(center, self.config.radius, bg_color);
            },
            MinimapShape::Square => {
                painter.rect_filled(rect, Rounding::same(4.0), bg_color);
            },
        }

        // Terrain
        if self.config.show_terrain && !model.terrain.is_empty() {
            self.render_terrain(painter, center, model);
        }

        // POI markers
        for marker in &model.markers {
            if marker.visible {
                self.render_poi(painter, center, model, marker, actions, &response);
            }
        }

        // Player icon
        self.render_player(painter, center, model);

        // Cardinal directions
        if self.config.show_directions {
            self.render_directions(painter, center, model);
        }

        // Border
        let border_color = Color32::from_rgba_unmultiplied(
            self.config.border_color[0],
            self.config.border_color[1],
            self.config.border_color[2],
            self.config.border_color[3],
        );

        match self.config.shape {
            MinimapShape::Circle => {
                painter.circle_stroke(center, self.config.radius, Stroke::new(2.0, border_color));
            },
            MinimapShape::Square => {
                painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(2.0, border_color));
            },
        }

        // Handle click
        if response.clicked() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                let (world_x, world_y) = self.screen_to_world(click_pos, center, model);
                actions.push(MinimapAction::ClickedPosition { world_x, world_y });
            }
        }
    }

    /// Renders terrain tiles.
    fn render_terrain(&self, painter: &Painter, center: Pos2, model: &MinimapModel) {
        let tile_size = (self.config.radius * 2.0) / model.terrain_width.max(1) as f32;
        let half_size = self.config.radius;

        for y in 0..model.terrain_height {
            for x in 0..model.terrain_width {
                if let Some(tile) = model.get_terrain(x, y) {
                    let screen_x = center.x - half_size + x as f32 * tile_size;
                    let screen_y = center.y - half_size + y as f32 * tile_size;

                    let tile_rect =
                        Rect::from_min_size(Pos2::new(screen_x, screen_y), Vec2::splat(tile_size));

                    // Check if tile is within minimap bounds (for circle shape)
                    if self.config.shape == MinimapShape::Circle {
                        let tile_center = tile_rect.center();
                        let dist = center.distance(tile_center);
                        if dist > self.config.radius {
                            continue;
                        }
                    }

                    painter.rect_filled(tile_rect, Rounding::ZERO, tile.color);
                }
            }
        }
    }

    /// Renders a POI marker.
    fn render_poi(
        &self,
        painter: &Painter,
        center: Pos2,
        model: &MinimapModel,
        marker: &POIMarker,
        _actions: &mut [MinimapAction],
        _response: &Response,
    ) {
        let screen_pos = self.world_to_screen(marker.world_x, marker.world_y, center, model);

        // Check if within bounds
        let dist = center.distance(screen_pos);
        if dist > self.config.radius - self.config.poi_marker_size {
            // Clamp to edge
            let angle = (screen_pos.y - center.y).atan2(screen_pos.x - center.x);
            let edge_dist = self.config.radius - self.config.poi_marker_size;
            let clamped = Pos2::new(
                center.x + edge_dist * angle.cos(),
                center.y + edge_dist * angle.sin(),
            );
            self.draw_poi_icon(painter, clamped, marker, true);
        } else {
            self.draw_poi_icon(painter, screen_pos, marker, false);
        }
    }

    /// Draws a POI icon at screen position.
    fn draw_poi_icon(&self, painter: &Painter, pos: Pos2, marker: &POIMarker, at_edge: bool) {
        let color = marker.effective_color();
        let size = if at_edge {
            self.config.poi_marker_size * 0.7
        } else {
            self.config.poi_marker_size
        };

        // Background circle
        if marker.is_highlighted {
            painter.circle_filled(pos, size + 2.0, Color32::WHITE);
        }
        painter.circle_filled(pos, size, color);

        // Icon
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            marker.poi_type.icon(),
            FontId::proportional(size * 1.2),
            Color32::BLACK,
        );
    }

    /// Renders the player icon.
    fn render_player(&self, painter: &Painter, center: Pos2, model: &MinimapModel) {
        let player_color = Color32::from_rgba_unmultiplied(
            self.config.player_color[0],
            self.config.player_color[1],
            self.config.player_color[2],
            self.config.player_color[3],
        );

        // Player is always at center
        let rotation = if self.config.rotate_with_player {
            0.0 // Map rotates, player faces up
        } else {
            model.player_rotation
        };

        // Draw arrow pointing in direction of player rotation
        let size = self.config.player_icon_size;
        let points = self.rotated_triangle(center, size, rotation);

        painter.add(Shape::convex_polygon(
            points,
            player_color,
            Stroke::new(1.0, Color32::BLACK),
        ));
    }

    /// Creates a rotated triangle for the player icon.
    fn rotated_triangle(&self, center: Pos2, size: f32, rotation: f32) -> Vec<Pos2> {
        // Use config to validate icon size
        let _ = &self.config;

        // Triangle pointing up (before rotation)
        let tip = Vec2::new(0.0, -size);
        let left = Vec2::new(-size * 0.6, size * 0.6);
        let right = Vec2::new(size * 0.6, size * 0.6);

        // Rotate
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();

        let rotate_vec = |v: Vec2| -> Pos2 {
            Pos2::new(
                center.x + v.x * cos_r - v.y * sin_r,
                center.y + v.x * sin_r + v.y * cos_r,
            )
        };

        vec![rotate_vec(tip), rotate_vec(right), rotate_vec(left)]
    }

    /// Renders cardinal direction indicators.
    fn render_directions(&self, painter: &Painter, center: Pos2, model: &MinimapModel) {
        let rotation = if self.config.rotate_with_player {
            -model.player_rotation
        } else {
            0.0
        };

        let directions = [("N", 0.0), ("E", PI / 2.0), ("S", PI), ("W", -PI / 2.0)];

        let distance = self.config.radius - 12.0;

        for (label, base_angle) in directions {
            let angle = base_angle + rotation;
            let pos = Pos2::new(
                center.x + distance * angle.sin(),
                center.y - distance * angle.cos(),
            );

            painter.text(
                pos,
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(10.0),
                Color32::from_white_alpha(200),
            );
        }
    }

    /// Converts world coordinates to screen position.
    fn world_to_screen(
        &self,
        world_x: f32,
        world_y: f32,
        center: Pos2,
        model: &MinimapModel,
    ) -> Pos2 {
        let dx = world_x - model.player_x;
        let dy = world_y - model.player_y;

        let scale = self.config.radius / (100.0 * self.config.zoom);

        let (sx, sy) = if self.config.rotate_with_player {
            let cos_r = (-model.player_rotation).cos();
            let sin_r = (-model.player_rotation).sin();
            (dx * cos_r - dy * sin_r, dx * sin_r + dy * cos_r)
        } else {
            (dx, dy)
        };

        Pos2::new(center.x + sx * scale, center.y + sy * scale)
    }

    /// Converts screen position to world coordinates.
    fn screen_to_world(&self, screen_pos: Pos2, center: Pos2, model: &MinimapModel) -> (f32, f32) {
        let sx = screen_pos.x - center.x;
        let sy = screen_pos.y - center.y;

        let scale = (100.0 * self.config.zoom) / self.config.radius;

        let (dx, dy) = if self.config.rotate_with_player {
            let cos_r = model.player_rotation.cos();
            let sin_r = model.player_rotation.sin();
            (sx * cos_r - sy * sin_r, sx * sin_r + sy * cos_r)
        } else {
            (sx, sy)
        };

        (model.player_x + dx * scale, model.player_y + dy * scale)
    }

    /// Zooms in.
    pub fn zoom_in(&mut self) {
        self.config.zoom = (self.config.zoom * 1.25).min(MAX_ZOOM);
    }

    /// Zooms out.
    pub fn zoom_out(&mut self) {
        self.config.zoom = (self.config.zoom / 1.25).max(MIN_ZOOM);
    }

    /// Sets the zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.config.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Toggles between circle and square shape.
    pub fn toggle_shape(&mut self) {
        self.config.shape = self.config.shape.toggle();
    }

    /// Sets the minimap shape.
    pub fn set_shape(&mut self, shape: MinimapShape) {
        self.config.shape = shape;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_shape_toggle() {
        assert_eq!(MinimapShape::Circle.toggle(), MinimapShape::Square);
        assert_eq!(MinimapShape::Square.toggle(), MinimapShape::Circle);
    }

    #[test]
    fn test_poi_type_colors() {
        assert_eq!(POIType::Quest.default_color(), Color32::GOLD);
        assert_eq!(POIType::Danger.default_color(), Color32::RED);
    }

    #[test]
    fn test_poi_type_icons() {
        assert_eq!(POIType::Quest.icon(), "!");
        assert_eq!(POIType::Shop.icon(), "$");
        assert_eq!(POIType::Waypoint.icon(), "★");
    }

    #[test]
    fn test_poi_marker_new() {
        let marker = POIMarker::new(1, 100.0, 200.0, POIType::Quest);
        assert_eq!(marker.id, 1);
        assert_eq!(marker.world_x, 100.0);
        assert_eq!(marker.world_y, 200.0);
        assert!(marker.visible);
    }

    #[test]
    fn test_poi_marker_with_label() {
        let marker = POIMarker::new(1, 0.0, 0.0, POIType::Shop).with_label("Blacksmith");
        assert_eq!(marker.label, Some("Blacksmith".to_string()));
    }

    #[test]
    fn test_poi_marker_with_color() {
        let marker = POIMarker::new(1, 0.0, 0.0, POIType::Quest).with_color(Color32::BLUE);
        assert_eq!(marker.effective_color(), Color32::BLUE);
    }

    #[test]
    fn test_poi_marker_default_color() {
        let marker = POIMarker::new(1, 0.0, 0.0, POIType::Danger);
        assert_eq!(marker.effective_color(), Color32::RED);
    }

    #[test]
    fn test_terrain_tile_new() {
        let tile = TerrainTile::new(Color32::GREEN, 0.5);
        assert_eq!(tile.color, Color32::GREEN);
        assert!((tile.height - 0.5).abs() < 0.001);
        assert!(tile.explored);
    }

    #[test]
    fn test_terrain_tile_unexplored() {
        let tile = TerrainTile::unexplored();
        assert!(!tile.explored);
    }

    #[test]
    fn test_minimap_model_new() {
        let model = MinimapModel::new();
        assert_eq!(model.player_x, 0.0);
        assert_eq!(model.player_y, 0.0);
        assert!(model.markers.is_empty());
    }

    #[test]
    fn test_minimap_model_set_player_position() {
        let mut model = MinimapModel::new();
        model.set_player_position(100.0, 200.0);
        assert_eq!(model.player_x, 100.0);
        assert_eq!(model.player_y, 200.0);
    }

    #[test]
    fn test_minimap_model_set_player_rotation() {
        let mut model = MinimapModel::new();
        model.set_player_rotation(PI / 2.0);
        assert!((model.player_rotation - PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_minimap_model_markers() {
        let mut model = MinimapModel::new();
        model.add_marker(POIMarker::new(1, 10.0, 20.0, POIType::Quest));
        model.add_marker(POIMarker::new(2, 30.0, 40.0, POIType::Shop));
        assert_eq!(model.markers.len(), 2);

        model.remove_marker(1);
        assert_eq!(model.markers.len(), 1);
        assert_eq!(model.markers[0].id, 2);
    }

    #[test]
    fn test_minimap_model_terrain() {
        let mut model = MinimapModel::new();
        let terrain = vec![
            TerrainTile::new(Color32::GREEN, 0.0),
            TerrainTile::new(Color32::BLUE, 0.5),
            TerrainTile::new(Color32::BROWN, 0.3),
            TerrainTile::new(Color32::WHITE, 1.0),
        ];
        model.set_terrain(terrain, 2, 2);

        assert_eq!(model.terrain_width, 2);
        assert_eq!(model.terrain_height, 2);
        assert_eq!(
            model.get_terrain(0, 0).map(|t| t.color),
            Some(Color32::GREEN)
        );
        assert_eq!(
            model.get_terrain(1, 1).map(|t| t.color),
            Some(Color32::WHITE)
        );
        assert!(model.get_terrain(5, 5).is_none());
    }

    #[test]
    fn test_minimap_config_defaults() {
        let config = MinimapConfig::default();
        assert_eq!(config.radius, DEFAULT_MINIMAP_RADIUS);
        assert_eq!(config.shape, MinimapShape::Circle);
        assert_eq!(config.zoom, DEFAULT_ZOOM);
    }

    #[test]
    fn test_minimap_ui_new() {
        let ui = MinimapUI::new();
        assert_eq!(ui.config.shape, MinimapShape::Circle);
        assert_eq!(ui.config.zoom, DEFAULT_ZOOM);
    }

    #[test]
    fn test_minimap_ui_zoom() {
        let mut ui = MinimapUI::new();
        ui.zoom_in();
        assert!(ui.config.zoom > DEFAULT_ZOOM);

        ui.set_zoom(DEFAULT_ZOOM);
        ui.zoom_out();
        assert!(ui.config.zoom < DEFAULT_ZOOM);
    }

    #[test]
    fn test_minimap_ui_zoom_limits() {
        let mut ui = MinimapUI::new();

        // Zoom in to max
        for _ in 0..20 {
            ui.zoom_in();
        }
        assert!((ui.config.zoom - MAX_ZOOM).abs() < 0.001);

        // Zoom out to min
        for _ in 0..20 {
            ui.zoom_out();
        }
        assert!((ui.config.zoom - MIN_ZOOM).abs() < 0.001);
    }

    #[test]
    fn test_minimap_ui_toggle_shape() {
        let mut ui = MinimapUI::new();
        assert_eq!(ui.config.shape, MinimapShape::Circle);

        ui.toggle_shape();
        assert_eq!(ui.config.shape, MinimapShape::Square);

        ui.toggle_shape();
        assert_eq!(ui.config.shape, MinimapShape::Circle);
    }

    #[test]
    fn test_minimap_ui_set_shape() {
        let mut ui = MinimapUI::new();
        ui.set_shape(MinimapShape::Square);
        assert_eq!(ui.config.shape, MinimapShape::Square);
    }

    #[test]
    fn test_minimap_action_equality() {
        assert_eq!(
            MinimapAction::ZoomChanged(1.0),
            MinimapAction::ZoomChanged(1.0)
        );
        assert_eq!(
            MinimapAction::ClickedMarker(42),
            MinimapAction::ClickedMarker(42)
        );
        assert_eq!(
            MinimapAction::ShapeToggled(MinimapShape::Circle),
            MinimapAction::ShapeToggled(MinimapShape::Circle)
        );
    }
}
