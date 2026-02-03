//! Cell inspector probe for interactive debugging.
//!
//! This module provides an advanced cell inspector with:
//! - Click-to-select cell inspection
//! - Display of all cell properties
//! - Material property lookup
//! - Neighboring cell information
//! - Real-time updates during simulation

use crate::chunk_viewer::MaterialColorMap;
use crate::inspector::CellInfo;
use egui::{Color32, Grid, RichText, Ui, Vec2};
use genesis_common::{ChunkCoord, WorldCoord};
use genesis_kernel::{Cell, MaterialProperties};

/// Default material names for common materials.
const MATERIAL_NAMES: &[(u16, &str)] = &[
    (0, "Air"),
    (1, "Dirt"),
    (2, "Stone"),
    (3, "Grass"),
    (4, "Water"),
    (5, "Sand"),
    (6, "Lava"),
    (7, "Wood"),
    (8, "Coal"),
    (9, "Iron"),
    (10, "Gold"),
];

/// Gets the name for a material ID.
#[must_use]
pub fn get_material_name(material_id: u16) -> &'static str {
    MATERIAL_NAMES
        .iter()
        .find(|(id, _)| *id == material_id)
        .map_or("Unknown", |(_, name)| name)
}

/// Decodes cell flags into readable strings.
#[must_use]
pub fn decode_flags(flags: u8) -> Vec<&'static str> {
    let mut result = Vec::new();
    if flags & 0x01 != 0 {
        result.push("SOLID");
    }
    if flags & 0x02 != 0 {
        result.push("LIQUID");
    }
    if flags & 0x04 != 0 {
        result.push("BURNING");
    }
    if flags & 0x08 != 0 {
        result.push("ELECTRIC");
    }
    if flags & 0x10 != 0 {
        result.push("UPDATED");
    }
    if result.is_empty() {
        result.push("NONE");
    }
    result
}

/// Direction for neighboring cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborDirection {
    /// North (up)
    North,
    /// South (down)
    South,
    /// East (right)
    East,
    /// West (left)
    West,
    /// North-East
    NorthEast,
    /// North-West
    NorthWest,
    /// South-East
    SouthEast,
    /// South-West
    SouthWest,
}

impl NeighborDirection {
    /// Returns all cardinal and diagonal directions.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::North,
            Self::South,
            Self::East,
            Self::West,
            Self::NorthEast,
            Self::NorthWest,
            Self::SouthEast,
            Self::SouthWest,
        ]
    }

    /// Returns only cardinal directions.
    #[must_use]
    pub const fn cardinals() -> &'static [Self] {
        &[Self::North, Self::South, Self::East, Self::West]
    }

    /// Returns the offset for this direction.
    #[must_use]
    pub const fn offset(&self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::East => (1, 0),
            Self::West => (-1, 0),
            Self::NorthEast => (1, -1),
            Self::NorthWest => (-1, -1),
            Self::SouthEast => (1, 1),
            Self::SouthWest => (-1, 1),
        }
    }

    /// Returns the display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::North => "N",
            Self::South => "S",
            Self::East => "E",
            Self::West => "W",
            Self::NorthEast => "NE",
            Self::NorthWest => "NW",
            Self::SouthEast => "SE",
            Self::SouthWest => "SW",
        }
    }
}

/// Information about a neighboring cell.
#[derive(Debug, Clone)]
pub struct NeighborInfo {
    /// Direction from selected cell
    pub direction: NeighborDirection,
    /// The neighbor cell (if in bounds)
    pub cell: Option<Cell>,
    /// Local position of neighbor
    pub local_pos: Option<(u32, u32)>,
}

/// Material properties lookup table.
#[derive(Debug, Clone, Default)]
pub struct MaterialLUT {
    /// Properties for each material ID
    properties: Vec<(u16, MaterialProperties, String)>,
}

/// Helper to create `MaterialProperties` with the required fields.
/// The reserved field is automatically set to 0.
fn make_material_props(
    density: u16,
    friction: u8,
    flammability: u8,
    conductivity: u8,
    hardness: u8,
    flags: u8,
) -> MaterialProperties {
    // Use bytemuck to construct since reserved is private
    let mut props = MaterialProperties::default();
    // Access through raw bytes since reserved is private
    let bytes = bytemuck::bytes_of_mut(&mut props);
    bytes[0..2].copy_from_slice(&density.to_ne_bytes());
    bytes[2] = friction;
    bytes[3] = flammability;
    bytes[4] = conductivity;
    bytes[5] = hardness;
    bytes[6] = flags;
    bytes[7] = 0; // reserved
    props
}

impl MaterialLUT {
    /// Creates a new LUT with default materials.
    #[must_use]
    pub fn new() -> Self {
        let mut lut = Self::default();

        // Default material properties
        lut.add(0, make_material_props(0, 0, 0, 255, 0, 0), "Air");
        lut.add(1, make_material_props(1500, 80, 0, 30, 20, 0x01), "Dirt");
        lut.add(2, make_material_props(2500, 90, 0, 50, 80, 0x01), "Stone");
        lut.add(3, make_material_props(1200, 70, 50, 20, 15, 0x01), "Grass");
        lut.add(4, make_material_props(1000, 10, 0, 100, 0, 0x02), "Water");
        lut.add(5, make_material_props(1600, 60, 0, 20, 10, 0), "Sand");
        lut.add(6, make_material_props(3000, 5, 0, 200, 0, 0x06), "Lava");

        lut
    }

    /// Adds a material to the LUT.
    pub fn add(&mut self, id: u16, properties: MaterialProperties, name: impl Into<String>) {
        self.properties.retain(|(mat_id, _, _)| *mat_id != id);
        self.properties.push((id, properties, name.into()));
    }

    /// Gets properties for a material.
    #[must_use]
    pub fn get(&self, id: u16) -> Option<&MaterialProperties> {
        self.properties
            .iter()
            .find(|(mat_id, _, _)| *mat_id == id)
            .map(|(_, props, _)| props)
    }

    /// Gets the name for a material.
    #[must_use]
    pub fn get_name(&self, id: u16) -> &str {
        self.properties
            .iter()
            .find(|(mat_id, _, _)| *mat_id == id)
            .map_or("Unknown", |(_, _, name)| name.as_str())
    }
}

/// Configuration for the cell inspector.
#[derive(Debug, Clone)]
pub struct CellInspectorConfig {
    /// Whether to show neighbor information
    pub show_neighbors: bool,
    /// Whether to show only cardinal neighbors (N/S/E/W) or all 8
    pub cardinal_only: bool,
    /// Whether to show material properties
    pub show_material_props: bool,
    /// Whether to show raw hex values
    pub show_hex: bool,
    /// Whether to auto-refresh during simulation
    pub auto_refresh: bool,
}

impl Default for CellInspectorConfig {
    fn default() -> Self {
        Self {
            show_neighbors: true,
            cardinal_only: true,
            show_material_props: true,
            show_hex: false,
            auto_refresh: true,
        }
    }
}

/// Interactive cell inspector probe.
#[derive(Debug)]
pub struct CellInspector {
    /// Currently selected position (local x, y)
    selected_pos: Option<(u32, u32)>,
    /// Current chunk being inspected
    current_chunk: Option<ChunkCoord>,
    /// Configuration
    pub config: CellInspectorConfig,
    /// Material lookup table
    pub material_lut: MaterialLUT,
    /// Material color map (for visual indicators)
    pub color_map: MaterialColorMap,
    /// Cached cell info
    cached_cell: Option<CellInfo>,
    /// Cached neighbor info
    cached_neighbors: Vec<NeighborInfo>,
    /// Frame counter for refresh
    frame_counter: u64,
}

impl Default for CellInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl CellInspector {
    /// Creates a new cell inspector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            selected_pos: None,
            current_chunk: None,
            config: CellInspectorConfig::default(),
            material_lut: MaterialLUT::new(),
            color_map: MaterialColorMap::new(),
            cached_cell: None,
            cached_neighbors: Vec::new(),
            frame_counter: 0,
        }
    }

    /// Selects a cell for inspection.
    pub fn select(&mut self, x: u32, y: u32, chunk: ChunkCoord) {
        self.selected_pos = Some((x, y));
        self.current_chunk = Some(chunk);
        // Clear cache to force refresh
        self.cached_cell = None;
        self.cached_neighbors.clear();
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selected_pos = None;
        self.current_chunk = None;
        self.cached_cell = None;
        self.cached_neighbors.clear();
    }

    /// Returns whether a cell is selected.
    #[must_use]
    pub fn has_selection(&self) -> bool {
        self.selected_pos.is_some()
    }

    /// Returns the selected position.
    #[must_use]
    pub fn selected_pos(&self) -> Option<(u32, u32)> {
        self.selected_pos
    }

    /// Updates cached data from cells array.
    pub fn update(&mut self, cells: &[Cell], chunk_size: u32) {
        self.frame_counter = self.frame_counter.wrapping_add(1);

        let Some((x, y)) = self.selected_pos else {
            return;
        };

        // Get selected cell
        let idx = (y * chunk_size + x) as usize;
        if let Some(cell) = cells.get(idx) {
            let chunk_coord = self.current_chunk.unwrap_or(ChunkCoord::new(0, 0));
            self.cached_cell = Some(CellInfo {
                world_pos: WorldCoord::new(
                    chunk_coord.x as i64 * chunk_size as i64 + x as i64,
                    chunk_coord.y as i64 * chunk_size as i64 + y as i64,
                ),
                chunk_pos: chunk_coord,
                local_x: x,
                local_y: y,
                cell: *cell,
            });
        }

        // Get neighbor cells
        self.cached_neighbors.clear();
        let directions = if self.config.cardinal_only {
            NeighborDirection::cardinals()
        } else {
            NeighborDirection::all()
        };

        // Use i64 to avoid cast_possible_wrap
        let x_i64 = i64::from(x);
        let y_i64 = i64::from(y);
        let size_i64 = i64::from(chunk_size);

        for &dir in directions {
            let (dx, dy) = dir.offset();
            let nx = x_i64 + i64::from(dx);
            let ny = y_i64 + i64::from(dy);

            let neighbor = if nx >= 0 && nx < size_i64 && ny >= 0 && ny < size_i64 {
                #[allow(clippy::cast_sign_loss)]
                let nidx = (ny as u64 * chunk_size as u64 + nx as u64) as usize;
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let pos = (nx as u32, ny as u32);
                cells.get(nidx).map(|c| (*c, pos))
            } else {
                None
            };

            self.cached_neighbors.push(NeighborInfo {
                direction: dir,
                cell: neighbor.map(|(c, _)| c),
                local_pos: neighbor.map(|(_, pos)| pos),
            });
        }
    }

    /// Renders the inspector UI.
    pub fn render_ui(&self, ui: &mut Ui) {
        ui.heading("🔍 Cell Inspector");
        ui.separator();

        if let Some(ref cell_info) = self.cached_cell {
            self.render_cell_info(ui, cell_info);

            if self.config.show_material_props {
                ui.separator();
                self.render_material_props(ui, cell_info.cell.material);
            }

            if self.config.show_neighbors {
                ui.separator();
                self.render_neighbors(ui);
            }
        } else {
            ui.label("No cell selected. Click on a cell in the chunk viewer.");
        }

        // Settings collapsible
        ui.separator();
        ui.collapsing("⚙ Inspector Settings", |ui| {
            self.render_settings(ui);
        });
    }

    /// Renders main cell information.
    fn render_cell_info(&self, ui: &mut Ui, info: &CellInfo) {
        let cell = &info.cell;

        // Position section
        ui.label(RichText::new("📍 Position").strong());
        Grid::new("cell_position_grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .show(ui, |ui| {
                ui.label("World:");
                ui.label(format!("({}, {})", info.world_pos.x, info.world_pos.y));
                ui.end_row();

                ui.label("Chunk:");
                ui.label(format!("({}, {})", info.chunk_pos.x, info.chunk_pos.y));
                ui.end_row();

                ui.label("Local:");
                ui.label(format!("({}, {})", info.local_x, info.local_y));
                ui.end_row();
            });

        ui.add_space(8.0);

        // Cell data section
        ui.label(RichText::new("📦 Cell Data").strong());

        // Material with color swatch
        ui.horizontal(|ui| {
            let color = self.color_map.get_color(cell.material);
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, color);
            ui.label(format!(
                "Material: {} (ID: {})",
                self.material_lut.get_name(cell.material),
                cell.material
            ));
        });

        Grid::new("cell_data_grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .show(ui, |ui| {
                ui.label("Flags:");
                let flags_str = decode_flags(cell.flags).join(" | ");
                if self.config.show_hex {
                    ui.label(format!("{flags_str} ({:#04x})", cell.flags));
                } else {
                    ui.label(flags_str);
                }
                ui.end_row();

                ui.label("Temperature:");
                let temp_color = if cell.temperature > 100 {
                    Color32::from_rgb(255, 100, 50)
                } else if cell.temperature < 10 {
                    Color32::from_rgb(100, 150, 255)
                } else {
                    Color32::WHITE
                };
                ui.label(RichText::new(format!("{}°", cell.temperature)).color(temp_color));
                ui.end_row();

                ui.label("Velocity:");
                ui.label(format!("({}, {})", cell.velocity_x, cell.velocity_y));
                ui.end_row();

                ui.label("Data:");
                if self.config.show_hex {
                    ui.label(format!("{:#06x}", cell.data));
                } else {
                    ui.label(format!("{}", cell.data));
                }
                ui.end_row();
            });
    }

    /// Renders material properties from LUT.
    fn render_material_props(&self, ui: &mut Ui, material_id: u16) {
        ui.label(RichText::new("🧪 Material Properties").strong());

        if let Some(props) = self.material_lut.get(material_id) {
            Grid::new("material_props_grid")
                .num_columns(2)
                .spacing([20.0, 2.0])
                .show(ui, |ui| {
                    ui.label("Density:");
                    ui.label(format!("{} kg/m³", props.density));
                    ui.end_row();

                    ui.label("Friction:");
                    ui.label(format!("{}%", props.friction));
                    ui.end_row();

                    ui.label("Flammability:");
                    let flam_color = if props.flammability > 50 {
                        Color32::from_rgb(255, 100, 50)
                    } else {
                        Color32::WHITE
                    };
                    ui.label(RichText::new(format!("{}%", props.flammability)).color(flam_color));
                    ui.end_row();

                    ui.label("Conductivity:");
                    ui.label(format!("{}", props.conductivity));
                    ui.end_row();

                    ui.label("Hardness:");
                    ui.label(format!("{}", props.hardness));
                    ui.end_row();
                });
        } else {
            ui.label("Material properties not found in LUT");
        }
    }

    /// Renders neighbor cell information.
    fn render_neighbors(&self, ui: &mut Ui) {
        ui.label(RichText::new("🔗 Neighbors").strong());

        // Render as a mini-grid layout
        Grid::new("neighbor_grid")
            .num_columns(3)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                // Top row: NW N NE
                self.render_neighbor_cell(ui, NeighborDirection::NorthWest);
                self.render_neighbor_cell(ui, NeighborDirection::North);
                self.render_neighbor_cell(ui, NeighborDirection::NorthEast);
                ui.end_row();

                // Middle row: W [X] E
                self.render_neighbor_cell(ui, NeighborDirection::West);
                // Center cell (selected)
                ui.label(RichText::new("X").strong().color(Color32::YELLOW));
                self.render_neighbor_cell(ui, NeighborDirection::East);
                ui.end_row();

                // Bottom row: SW S SE
                self.render_neighbor_cell(ui, NeighborDirection::SouthWest);
                self.render_neighbor_cell(ui, NeighborDirection::South);
                self.render_neighbor_cell(ui, NeighborDirection::SouthEast);
                ui.end_row();
            });

        // Detailed neighbor list
        if !self.cached_neighbors.is_empty() {
            ui.add_space(4.0);
            for neighbor in &self.cached_neighbors {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", neighbor.direction.name()));
                    if let Some(cell) = &neighbor.cell {
                        let color = self.color_map.get_color(cell.material);
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color);
                        ui.label(format!(
                            "{} (T:{}°)",
                            self.material_lut.get_name(cell.material),
                            cell.temperature
                        ));
                    } else {
                        ui.label("(out of bounds)");
                    }
                });
            }
        }
    }

    /// Renders a single neighbor cell in the grid.
    fn render_neighbor_cell(&self, ui: &mut Ui, direction: NeighborDirection) {
        let neighbor = self
            .cached_neighbors
            .iter()
            .find(|n| n.direction == direction);

        if let Some(info) = neighbor {
            if let Some(cell) = &info.cell {
                let color = self.color_map.get_color(cell.material);
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
            } else if self.config.cardinal_only
                && !matches!(
                    direction,
                    NeighborDirection::North
                        | NeighborDirection::South
                        | NeighborDirection::East
                        | NeighborDirection::West
                )
            {
                // Skip diagonal in cardinal-only mode
                ui.label("");
            } else {
                ui.label("·");
            }
        } else {
            ui.label("");
        }
    }

    /// Renders settings panel.
    fn render_settings(&self, ui: &mut Ui) {
        // Note: These are display-only since config isn't mutable here
        // In practice, you'd pass &mut self and make these checkboxes
        ui.label(format!(
            "Show neighbors: {}",
            if self.config.show_neighbors {
                "Yes"
            } else {
                "No"
            }
        ));
        ui.label(format!(
            "Cardinal only: {}",
            if self.config.cardinal_only {
                "Yes"
            } else {
                "No"
            }
        ));
        ui.label(format!(
            "Show material props: {}",
            if self.config.show_material_props {
                "Yes"
            } else {
                "No"
            }
        ));
        ui.label(format!(
            "Show hex values: {}",
            if self.config.show_hex { "Yes" } else { "No" }
        ));
    }

    /// Renders the settings panel with mutable access.
    pub fn render_settings_mut(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.config.show_neighbors, "Show neighbors");
        ui.checkbox(&mut self.config.cardinal_only, "Cardinal directions only");
        ui.checkbox(
            &mut self.config.show_material_props,
            "Show material properties",
        );
        ui.checkbox(&mut self.config.show_hex, "Show hex values");
        ui.checkbox(&mut self.config.auto_refresh, "Auto-refresh");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_material_name() {
        assert_eq!(get_material_name(0), "Air");
        assert_eq!(get_material_name(4), "Water");
        assert_eq!(get_material_name(999), "Unknown");
    }

    #[test]
    fn test_decode_flags() {
        assert_eq!(decode_flags(0), vec!["NONE"]);
        assert_eq!(decode_flags(0x01), vec!["SOLID"]);
        assert_eq!(decode_flags(0x03), vec!["SOLID", "LIQUID"]);
        assert_eq!(decode_flags(0x07), vec!["SOLID", "LIQUID", "BURNING"]);
    }

    #[test]
    fn test_neighbor_directions() {
        assert_eq!(NeighborDirection::North.offset(), (0, -1));
        assert_eq!(NeighborDirection::SouthEast.offset(), (1, 1));
        assert_eq!(NeighborDirection::cardinals().len(), 4);
        assert_eq!(NeighborDirection::all().len(), 8);
    }

    #[test]
    fn test_material_lut() {
        let lut = MaterialLUT::new();

        let air = lut.get(0);
        assert!(air.is_some());
        assert_eq!(air.map(|p| p.density), Some(0));

        let stone = lut.get(2);
        assert!(stone.is_some());
        assert_eq!(stone.map(|p| p.hardness), Some(80));

        assert_eq!(lut.get_name(4), "Water");
        assert_eq!(lut.get_name(999), "Unknown");
    }

    #[test]
    fn test_cell_inspector_selection() {
        let mut inspector = CellInspector::new();
        assert!(!inspector.has_selection());

        inspector.select(5, 10, ChunkCoord::new(1, 2));
        assert!(inspector.has_selection());
        assert_eq!(inspector.selected_pos(), Some((5, 10)));

        inspector.clear_selection();
        assert!(!inspector.has_selection());
    }

    #[test]
    fn test_cell_inspector_update() {
        let mut inspector = CellInspector::new();
        inspector.select(1, 1, ChunkCoord::new(0, 0));

        // Create a 3x3 grid of cells
        let cells = vec![
            Cell::new(1),
            Cell::new(2),
            Cell::new(3),
            Cell::new(4),
            Cell::new(5),
            Cell::new(6),
            Cell::new(7),
            Cell::new(8),
            Cell::new(9),
        ];

        inspector.update(&cells, 3);

        // Selected cell should be at (1,1) = index 4 = material 5
        assert!(inspector.cached_cell.is_some());
        assert_eq!(
            inspector.cached_cell.as_ref().map(|c| c.cell.material),
            Some(5)
        );

        // Should have 4 cardinal neighbors
        assert_eq!(inspector.cached_neighbors.len(), 4);
    }

    #[test]
    fn test_cell_inspector_config_defaults() {
        let config = CellInspectorConfig::default();
        assert!(config.show_neighbors);
        assert!(config.cardinal_only);
        assert!(config.show_material_props);
        assert!(!config.show_hex);
        assert!(config.auto_refresh);
    }
}
