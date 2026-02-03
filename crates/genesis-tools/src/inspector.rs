//! World and cell inspection tools.

use genesis_common::{ChunkCoord, WorldCoord};
use genesis_kernel::Cell;

/// Cell inspection data.
#[derive(Debug, Clone)]
pub struct CellInfo {
    /// World coordinate
    pub world_pos: WorldCoord,
    /// Chunk coordinate
    pub chunk_pos: ChunkCoord,
    /// Local position in chunk
    pub local_x: u32,
    /// Local Y in chunk
    pub local_y: u32,
    /// Cell data
    pub cell: Cell,
}

/// Chunk inspection data.
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    /// Chunk coordinate
    pub coord: ChunkCoord,
    /// Chunk size
    pub size: u32,
    /// Whether dirty
    pub is_dirty: bool,
    /// Material histogram (material_id -> count)
    pub material_counts: Vec<(u16, u32)>,
}

/// Inspector for examining world state.
#[derive(Debug, Default)]
pub struct WorldInspector {
    /// Currently selected cell (if any)
    selected_cell: Option<CellInfo>,
    /// Currently selected chunk (if any)
    selected_chunk: Option<ChunkInfo>,
}

impl WorldInspector {
    /// Creates a new inspector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects a cell for inspection.
    pub fn select_cell(&mut self, info: CellInfo) {
        self.selected_cell = Some(info);
    }

    /// Clears cell selection.
    pub fn clear_cell_selection(&mut self) {
        self.selected_cell = None;
    }

    /// Returns the selected cell info.
    #[must_use]
    pub fn selected_cell(&self) -> Option<&CellInfo> {
        self.selected_cell.as_ref()
    }

    /// Selects a chunk for inspection.
    pub fn select_chunk(&mut self, info: ChunkInfo) {
        self.selected_chunk = Some(info);
    }

    /// Returns the selected chunk info.
    #[must_use]
    pub fn selected_chunk(&self) -> Option<&ChunkInfo> {
        self.selected_chunk.as_ref()
    }
}

/// Generates a material histogram for a chunk.
#[must_use]
pub fn calculate_material_histogram(cells: &[Cell]) -> Vec<(u16, u32)> {
    use std::collections::HashMap;
    let mut counts: HashMap<u16, u32> = HashMap::new();

    for cell in cells {
        *counts.entry(cell.material).or_insert(0) += 1;
    }

    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    result
}
