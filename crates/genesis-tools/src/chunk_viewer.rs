//! Chunk viewer using egui for debug visualization.
//!
//! This module provides an interactive chunk viewer for debugging:
//! - Display chunk grid with cells colored by material
//! - Click to inspect individual cell properties
//! - Show chunk coordinates and metadata
//! - Material histogram visualization

use crate::inspector::{calculate_material_histogram, CellInfo, ChunkInfo};
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use genesis_common::{ChunkCoord, LocalCoord, WorldCoord};
use genesis_kernel::Cell;

/// Material color mapping for visualization.
#[derive(Debug, Clone)]
pub struct MaterialColorMap {
    /// Default color for unknown materials
    pub default_color: Color32,
    /// Color mappings (material_id -> color)
    colors: Vec<(u16, Color32)>,
}

impl Default for MaterialColorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialColorMap {
    /// Creates a new color map with default colors.
    #[must_use]
    pub fn new() -> Self {
        let mut map = Self {
            default_color: Color32::DARK_GRAY,
            colors: Vec::new(),
        };

        // Default material colors
        map.set_color(0, Color32::from_rgb(135, 206, 235)); // Air - sky blue
        map.set_color(1, Color32::from_rgb(139, 90, 43)); // Dirt - brown
        map.set_color(2, Color32::from_rgb(128, 128, 128)); // Stone - gray
        map.set_color(3, Color32::from_rgb(34, 139, 34)); // Grass - green
        map.set_color(4, Color32::from_rgb(64, 164, 223)); // Water - blue
        map.set_color(5, Color32::from_rgb(194, 178, 128)); // Sand - tan
        map.set_color(6, Color32::from_rgb(255, 165, 0)); // Lava - orange
        map.set_color(7, Color32::from_rgb(139, 69, 19)); // Wood - dark brown
        map.set_color(8, Color32::from_rgb(50, 50, 50)); // Coal - dark gray
        map.set_color(9, Color32::from_rgb(192, 192, 192)); // Iron - silver
        map.set_color(10, Color32::from_rgb(255, 215, 0)); // Gold - gold

        map
    }

    /// Sets a color for a material.
    pub fn set_color(&mut self, material_id: u16, color: Color32) {
        // Remove existing if present
        self.colors.retain(|(id, _)| *id != material_id);
        self.colors.push((material_id, color));
    }

    /// Gets the color for a material.
    #[must_use]
    pub fn get_color(&self, material_id: u16) -> Color32 {
        self.colors
            .iter()
            .find(|(id, _)| *id == material_id)
            .map_or(self.default_color, |(_, color)| *color)
    }

    /// Gets the color for a cell based on its material and state.
    #[must_use]
    pub fn get_cell_color(&self, cell: &Cell) -> Color32 {
        let base_color = self.get_color(cell.material);

        // Modify color based on temperature (warmer = more red tint)
        if cell.temperature > 100 {
            let heat_factor = (cell.temperature - 100) as f32 / 155.0;
            blend_colors(
                base_color,
                Color32::from_rgb(255, 100, 50),
                heat_factor * 0.5,
            )
        } else if cell.temperature < 10 {
            let cold_factor = (10 - cell.temperature) as f32 / 10.0;
            blend_colors(
                base_color,
                Color32::from_rgb(150, 200, 255),
                cold_factor * 0.3,
            )
        } else {
            base_color
        }
    }
}

/// Blends two colors by a factor (0.0 = a, 1.0 = b).
fn blend_colors(a: Color32, b: Color32, factor: f32) -> Color32 {
    let factor = factor.clamp(0.0, 1.0);
    let inv = 1.0 - factor;

    Color32::from_rgb(
        (a.r() as f32 * inv + b.r() as f32 * factor) as u8,
        (a.g() as f32 * inv + b.g() as f32 * factor) as u8,
        (a.b() as f32 * inv + b.b() as f32 * factor) as u8,
    )
}

/// Configuration for the chunk viewer.
#[derive(Debug, Clone)]
pub struct ChunkViewerConfig {
    /// Cell size in pixels for rendering
    pub cell_size: f32,
    /// Whether to show grid lines
    pub show_grid: bool,
    /// Grid line color
    pub grid_color: Color32,
    /// Whether to highlight selected cell
    pub highlight_selection: bool,
    /// Selection highlight color
    pub selection_color: Color32,
    /// Whether to show cell velocity vectors
    pub show_velocity: bool,
    /// Velocity vector color
    pub velocity_color: Color32,
}

impl Default for ChunkViewerConfig {
    fn default() -> Self {
        Self {
            cell_size: 4.0,
            show_grid: false,
            grid_color: Color32::from_rgba_unmultiplied(100, 100, 100, 100),
            highlight_selection: true,
            selection_color: Color32::YELLOW,
            show_velocity: false,
            velocity_color: Color32::WHITE,
        }
    }
}

/// State for the chunk viewer.
#[derive(Debug, Default)]
pub struct ChunkViewerState {
    /// Currently viewing chunk coordinate
    pub current_chunk: Option<ChunkCoord>,
    /// Currently selected cell (local coordinates)
    pub selected_cell: Option<(u32, u32)>,
    /// Zoom level (1.0 = normal)
    pub zoom: f32,
    /// Pan offset
    pub pan_offset: Vec2,
}

impl ChunkViewerState {
    /// Creates a new viewer state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            ..Default::default()
        }
    }

    /// Resets the view to default.
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan_offset = Vec2::ZERO;
        self.selected_cell = None;
    }

    /// Sets the current chunk to view.
    pub fn set_chunk(&mut self, coord: ChunkCoord) {
        if self.current_chunk != Some(coord) {
            self.current_chunk = Some(coord);
            self.selected_cell = None;
        }
    }
}

/// The chunk viewer widget.
#[derive(Debug)]
pub struct ChunkViewer {
    /// Viewer configuration
    pub config: ChunkViewerConfig,
    /// Material color mapping
    pub color_map: MaterialColorMap,
    /// Viewer state
    pub state: ChunkViewerState,
}

impl Default for ChunkViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkViewer {
    /// Creates a new chunk viewer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ChunkViewerConfig::default(),
            color_map: MaterialColorMap::new(),
            state: ChunkViewerState::new(),
        }
    }

    /// Renders the chunk viewer UI.
    ///
    /// Returns the selected cell info if a cell was clicked.
    pub fn show(&mut self, ui: &mut Ui, cells: &[Cell], chunk_size: u32) -> Option<CellInfo> {
        let mut clicked_cell = None;

        // Control panel
        ui.horizontal(|ui| {
            ui.label("Chunk Viewer");
            if let Some(coord) = self.state.current_chunk {
                ui.label(format!("({}, {})", coord.x, coord.y));
            }
            ui.separator();
            ui.label(format!("Zoom: {:.1}x", self.state.zoom));
            if ui.button("Reset").clicked() {
                self.state.reset_view();
            }
        });

        ui.separator();

        // Settings
        ui.collapsing("Settings", |ui| {
            ui.checkbox(&mut self.config.show_grid, "Show Grid");
            ui.checkbox(&mut self.config.show_velocity, "Show Velocity");
            ui.add(egui::Slider::new(&mut self.config.cell_size, 1.0..=16.0).text("Cell Size"));
        });

        ui.separator();

        // Chunk grid
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, Sense::click_and_drag());

        let rect = response.rect;
        let cell_size = self.config.cell_size * self.state.zoom;

        // Handle zoom with scroll
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = if scroll > 0.0 { 1.1 } else { 0.9 };
                self.state.zoom = (self.state.zoom * zoom_factor).clamp(0.25, 8.0);
            }
        }

        // Handle panning with right-click drag
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.state.pan_offset += response.drag_delta();
        }

        // Calculate visible area
        let grid_size = chunk_size as f32 * cell_size;
        let grid_origin = rect.center() - Vec2::splat(grid_size / 2.0) + self.state.pan_offset;

        // Draw cells
        for (idx, cell) in cells.iter().enumerate() {
            let local = LocalCoord::from_index(idx, chunk_size);
            let x = local.x as f32;
            let y = local.y as f32;

            let cell_rect = Rect::from_min_size(
                egui::pos2(grid_origin.x + x * cell_size, grid_origin.y + y * cell_size),
                Vec2::splat(cell_size),
            );

            // Only draw if visible
            if !rect.intersects(cell_rect) {
                continue;
            }

            let color = self.color_map.get_cell_color(cell);
            painter.rect_filled(cell_rect, 0.0, color);

            // Draw selection highlight
            if self.config.highlight_selection
                && self.state.selected_cell == Some((local.x as u32, local.y as u32))
            {
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    Stroke::new(2.0, self.config.selection_color),
                );
            }

            // Draw velocity vector
            if self.config.show_velocity && (cell.velocity_x != 0 || cell.velocity_y != 0) {
                let center = cell_rect.center();
                let vel_scale = cell_size * 0.3;
                let end = egui::pos2(
                    center.x + cell.velocity_x as f32 * vel_scale / 128.0,
                    center.y + cell.velocity_y as f32 * vel_scale / 128.0,
                );
                painter.line_segment([center, end], Stroke::new(1.0, self.config.velocity_color));
            }
        }

        // Draw grid
        if self.config.show_grid {
            let stroke = Stroke::new(0.5, self.config.grid_color);
            for i in 0..=chunk_size {
                let offset = i as f32 * cell_size;
                // Vertical lines
                painter.line_segment(
                    [
                        egui::pos2(grid_origin.x + offset, grid_origin.y),
                        egui::pos2(grid_origin.x + offset, grid_origin.y + grid_size),
                    ],
                    stroke,
                );
                // Horizontal lines
                painter.line_segment(
                    [
                        egui::pos2(grid_origin.x, grid_origin.y + offset),
                        egui::pos2(grid_origin.x + grid_size, grid_origin.y + offset),
                    ],
                    stroke,
                );
            }
        }

        // Handle click to select cell
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let local_pos = pos - grid_origin;
                let cell_x = (local_pos.x / cell_size).floor();
                let cell_y = (local_pos.y / cell_size).floor();
                let chunk_size_f = chunk_size as f32;

                if cell_x >= 0.0 && cell_x < chunk_size_f && cell_y >= 0.0 && cell_y < chunk_size_f
                {
                    #[allow(clippy::cast_sign_loss)]
                    let local_x = cell_x as u32;
                    #[allow(clippy::cast_sign_loss)]
                    let local_y = cell_y as u32;
                    self.state.selected_cell = Some((local_x, local_y));

                    let idx = (local_y * chunk_size + local_x) as usize;
                    if let Some(cell) = cells.get(idx) {
                        let chunk_coord = self.state.current_chunk.unwrap_or(ChunkCoord::new(0, 0));
                        clicked_cell = Some(CellInfo {
                            world_pos: WorldCoord::new(
                                chunk_coord.x as i64 * chunk_size as i64 + local_x as i64,
                                chunk_coord.y as i64 * chunk_size as i64 + local_y as i64,
                            ),
                            chunk_pos: chunk_coord,
                            local_x,
                            local_y,
                            cell: *cell,
                        });
                    }
                }
            }
        }

        clicked_cell
    }

    /// Shows the cell inspector panel.
    pub fn show_cell_inspector(&self, ui: &mut Ui, cell_info: &CellInfo) {
        ui.heading("Cell Inspector");
        ui.separator();

        egui::Grid::new("cell_inspector_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("World Position:");
                ui.label(format!(
                    "({}, {})",
                    cell_info.world_pos.x, cell_info.world_pos.y
                ));
                ui.end_row();

                ui.label("Chunk:");
                ui.label(format!(
                    "({}, {})",
                    cell_info.chunk_pos.x, cell_info.chunk_pos.y
                ));
                ui.end_row();

                ui.label("Local Position:");
                ui.label(format!("({}, {})", cell_info.local_x, cell_info.local_y));
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                ui.label("Material ID:");
                ui.label(format!("{}", cell_info.cell.material));
                ui.end_row();

                ui.label("Flags:");
                ui.label(format!("{:#04x}", cell_info.cell.flags));
                ui.end_row();

                ui.label("Temperature:");
                ui.label(format!("{}°", cell_info.cell.temperature));
                ui.end_row();

                ui.label("Velocity:");
                ui.label(format!(
                    "({}, {})",
                    cell_info.cell.velocity_x, cell_info.cell.velocity_y
                ));
                ui.end_row();

                ui.label("Data:");
                ui.label(format!("{:#06x}", cell_info.cell.data));
                ui.end_row();
            });
    }

    /// Shows chunk metadata panel.
    pub fn show_chunk_info(&self, ui: &mut Ui, chunk_info: &ChunkInfo) {
        ui.heading("Chunk Info");
        ui.separator();

        egui::Grid::new("chunk_info_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("Coordinate:");
                ui.label(format!("({}, {})", chunk_info.coord.x, chunk_info.coord.y));
                ui.end_row();

                ui.label("Size:");
                ui.label(format!("{}x{}", chunk_info.size, chunk_info.size));
                ui.end_row();

                ui.label("Dirty:");
                ui.label(if chunk_info.is_dirty { "Yes" } else { "No" });
                ui.end_row();
            });

        ui.separator();
        ui.label("Material Distribution:");

        // Show top 5 materials
        for (i, (material_id, count)) in chunk_info.material_counts.iter().take(5).enumerate() {
            let total = chunk_info.size * chunk_info.size;
            let percentage = (*count as f32 / total as f32) * 100.0;
            let color = self.color_map.get_color(*material_id);

            ui.horizontal(|ui| {
                // Color swatch
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);

                ui.label(format!(
                    "#{}: Material {} - {} ({:.1}%)",
                    i + 1,
                    material_id,
                    count,
                    percentage
                ));
            });
        }
    }
}

/// Creates a chunk info from cells.
#[must_use]
pub fn create_chunk_info(
    coord: ChunkCoord,
    size: u32,
    cells: &[Cell],
    is_dirty: bool,
) -> ChunkInfo {
    ChunkInfo {
        coord,
        size,
        is_dirty,
        material_counts: calculate_material_histogram(cells),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_color_map() {
        let map = MaterialColorMap::new();

        // Air should be sky blue
        let air_color = map.get_color(0);
        assert_eq!(air_color.r(), 135);

        // Unknown material should be default
        let unknown = map.get_color(999);
        assert_eq!(unknown, map.default_color);
    }

    #[test]
    fn test_material_color_map_custom() {
        let mut map = MaterialColorMap::new();
        map.set_color(100, Color32::RED);

        assert_eq!(map.get_color(100), Color32::RED);
    }

    #[test]
    fn test_cell_color_temperature() {
        let map = MaterialColorMap::new();

        // Normal temperature
        let normal_cell = Cell {
            material: 1,
            temperature: 50,
            ..Default::default()
        };
        let normal_color = map.get_cell_color(&normal_cell);

        // Hot cell should have red tint
        let hot_cell = Cell {
            material: 1,
            temperature: 200,
            ..Default::default()
        };
        let hot_color = map.get_cell_color(&hot_cell);

        // Hot should have more red than normal
        assert!(hot_color.r() > normal_color.r() || hot_color.r() == 255);
    }

    #[test]
    fn test_blend_colors() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(255, 255, 255);

        let blended = blend_colors(a, b, 0.5);
        assert!(blended.r() > 100 && blended.r() < 200);
    }

    #[test]
    fn test_chunk_viewer_state() {
        let mut state = ChunkViewerState::new();
        assert_eq!(state.zoom, 1.0);
        assert!(state.selected_cell.is_none());

        state.set_chunk(ChunkCoord::new(5, 10));
        assert_eq!(state.current_chunk, Some(ChunkCoord::new(5, 10)));

        state.selected_cell = Some((1, 2));
        state.reset_view();
        assert!(state.selected_cell.is_none());
        assert_eq!(state.zoom, 1.0);
    }

    #[test]
    fn test_create_chunk_info() {
        let cells = vec![Cell::new(1), Cell::new(1), Cell::new(2), Cell::new(1)];

        let info = create_chunk_info(ChunkCoord::new(0, 0), 2, &cells, false);

        assert_eq!(info.coord, ChunkCoord::new(0, 0));
        assert_eq!(info.size, 2);
        assert!(!info.is_dirty);
        // Material 1 should be most common (3 cells)
        assert_eq!(info.material_counts[0].0, 1);
        assert_eq!(info.material_counts[0].1, 3);
    }

    #[test]
    fn test_chunk_viewer_config_defaults() {
        let config = ChunkViewerConfig::default();
        assert_eq!(config.cell_size, 4.0);
        assert!(!config.show_grid);
        assert!(config.highlight_selection);
    }
}
