//! Edge cell sharing between neighboring chunks.
//!
//! This module provides infrastructure for sharing edge cells between
//! adjacent chunks, enabling physics to work across chunk boundaries.
//!
//! ## Overview
//!
//! Each chunk has a 1-cell "ghost region" on each edge that mirrors
//! the corresponding edge cells from neighboring chunks:
//!
//! ```text
//!  Ghost Region (copied from neighbor)
//!  ↓
//! +---+---+---+---+
//! | G | G | G | G |  ← Top ghost row (from top neighbor)
//! +---+---+---+---+
//! | G | * | * | G |  ← Interior cells
//! +---+---+---+---+
//! | G | * | * | G |  ← Interior cells
//! +---+---+---+---+
//! | G | G | G | G |  ← Bottom ghost row (from bottom neighbor)
//! +---+---+---+---+
//!   ↑           ↑
//!   Left ghost  Right ghost
//! ```
//!
//! The ghost region allows the compute shader to read neighboring cells
//! without special boundary handling.

use crate::{Cell, ChunkId, Direction};

/// Size of the ghost region in cells (1 cell overlap on each edge).
pub const GHOST_SIZE: usize = 1;

/// Edge data extracted from a chunk for sharing with neighbors.
#[derive(Debug, Clone)]
pub struct EdgeData {
    /// The cells on this edge
    pub cells: Vec<Cell>,
    /// Direction this edge faces (e.g., Top edge faces upward)
    pub direction: Direction,
    /// Source chunk ID
    pub source_chunk: ChunkId,
}

impl EdgeData {
    /// Creates new edge data.
    #[must_use]
    pub fn new(cells: Vec<Cell>, direction: Direction, source_chunk: ChunkId) -> Self {
        Self {
            cells,
            direction,
            source_chunk,
        }
    }

    /// Returns the number of cells in this edge.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Returns true if the edge has no cells.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

/// Extracts edge cells from a cell buffer.
///
/// The edge cells are the outermost row/column of the chunk that
/// should be copied to neighboring chunks' ghost regions.
pub struct EdgeExtractor {
    /// Chunk size (width and height in cells)
    chunk_size: usize,
}

impl EdgeExtractor {
    /// Creates a new edge extractor.
    #[must_use]
    pub const fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Extracts edge cells for a given direction.
    ///
    /// # Arguments
    /// * `cells` - Full cell buffer (chunk_size * chunk_size cells)
    /// * `direction` - Which edge to extract
    /// * `chunk_id` - Source chunk identifier
    ///
    /// # Returns
    /// Edge data containing cells from the specified edge.
    #[must_use]
    pub fn extract(&self, cells: &[Cell], direction: Direction, chunk_id: ChunkId) -> EdgeData {
        let edge_cells = match direction {
            Direction::Bottom => self.extract_row(cells, 0),
            Direction::Top => self.extract_row(cells, self.chunk_size - 1),
            Direction::Left => self.extract_column(cells, 0),
            Direction::Right => self.extract_column(cells, self.chunk_size - 1),
        };
        EdgeData::new(edge_cells, direction, chunk_id)
    }

    /// Extracts a row of cells.
    fn extract_row(&self, cells: &[Cell], y: usize) -> Vec<Cell> {
        let start = y * self.chunk_size;
        cells[start..start + self.chunk_size].to_vec()
    }

    /// Extracts a column of cells.
    fn extract_column(&self, cells: &[Cell], x: usize) -> Vec<Cell> {
        (0..self.chunk_size)
            .map(|y| cells[y * self.chunk_size + x])
            .collect()
    }

    /// Extracts all four edges from a cell buffer.
    #[must_use]
    pub fn extract_all(&self, cells: &[Cell], chunk_id: ChunkId) -> [EdgeData; 4] {
        [
            self.extract(cells, Direction::Bottom, chunk_id),
            self.extract(cells, Direction::Top, chunk_id),
            self.extract(cells, Direction::Left, chunk_id),
            self.extract(cells, Direction::Right, chunk_id),
        ]
    }
}

/// Applies edge data to a cell buffer's ghost region.
pub struct EdgeApplicator {
    /// Chunk size (width and height in cells)
    chunk_size: usize,
}

impl EdgeApplicator {
    /// Creates a new edge applicator.
    #[must_use]
    pub const fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Applies edge data from a neighbor to the ghost region.
    ///
    /// The edge data's direction indicates which edge of the SOURCE chunk
    /// it came from. We need to apply it to the OPPOSITE edge of the
    /// destination chunk's ghost region.
    ///
    /// For example:
    /// - Neighbor's TOP edge → our BOTTOM ghost region
    /// - Neighbor's LEFT edge → our RIGHT ghost region
    pub fn apply(&self, cells: &mut [Cell], edge: &EdgeData) {
        // The edge came from the neighbor's side, apply to opposite side of our chunk
        let target_direction = edge.direction.opposite();
        
        match target_direction {
            Direction::Bottom => self.apply_row(cells, 0, &edge.cells),
            Direction::Top => self.apply_row(cells, self.chunk_size - 1, &edge.cells),
            Direction::Left => self.apply_column(cells, 0, &edge.cells),
            Direction::Right => self.apply_column(cells, self.chunk_size - 1, &edge.cells),
        }
    }

    /// Applies cells to a row.
    fn apply_row(&self, cells: &mut [Cell], y: usize, edge_cells: &[Cell]) {
        let start = y * self.chunk_size;
        let count = edge_cells.len().min(self.chunk_size);
        cells[start..start + count].copy_from_slice(&edge_cells[..count]);
    }

    /// Applies cells to a column.
    fn apply_column(&self, cells: &mut [Cell], x: usize, edge_cells: &[Cell]) {
        for (y, &cell) in edge_cells.iter().enumerate().take(self.chunk_size) {
            cells[y * self.chunk_size + x] = cell;
        }
    }
}

/// Manages edge sharing between multiple chunks.
pub struct EdgeSharingManager {
    /// Edge extractor
    extractor: EdgeExtractor,
    /// Edge applicator
    applicator: EdgeApplicator,
    /// Chunk size
    chunk_size: usize,
}

impl EdgeSharingManager {
    /// Creates a new edge sharing manager.
    #[must_use]
    pub fn new(chunk_size: usize) -> Self {
        Self {
            extractor: EdgeExtractor::new(chunk_size),
            applicator: EdgeApplicator::new(chunk_size),
            chunk_size,
        }
    }

    /// Returns the edge extractor.
    #[must_use]
    pub const fn extractor(&self) -> &EdgeExtractor {
        &self.extractor
    }

    /// Returns the edge applicator.
    #[must_use]
    pub const fn applicator(&self) -> &EdgeApplicator {
        &self.applicator
    }

    /// Returns the chunk size.
    #[must_use]
    pub const fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Shares edge data between two adjacent chunks.
    ///
    /// # Arguments
    /// * `chunk_a_cells` - Mutable cells for chunk A
    /// * `chunk_a_id` - Chunk A's identifier
    /// * `chunk_b_cells` - Mutable cells for chunk B
    /// * `chunk_b_id` - Chunk B's identifier
    /// * `direction` - Direction from A to B (e.g., Right means B is to the right of A)
    pub fn share_edges(
        &self,
        chunk_a_cells: &mut [Cell],
        chunk_a_id: ChunkId,
        chunk_b_cells: &mut [Cell],
        chunk_b_id: ChunkId,
        direction: Direction,
    ) {
        // Extract A's edge facing B
        let edge_from_a = self.extractor.extract(chunk_a_cells, direction, chunk_a_id);
        
        // Extract B's edge facing A
        let opposite = direction.opposite();
        let edge_from_b = self.extractor.extract(chunk_b_cells, opposite, chunk_b_id);
        
        // Apply B's edge to A's ghost region
        self.applicator.apply(chunk_a_cells, &edge_from_b);
        
        // Apply A's edge to B's ghost region
        self.applicator.apply(chunk_b_cells, &edge_from_a);
    }
}

/// Helper to determine which edges to share based on chunk positions.
#[must_use]
pub fn get_shared_direction(from: ChunkId, to: ChunkId) -> Option<Direction> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    
    match (dx, dy) {
        (0, -1) => Some(Direction::Bottom),
        (0, 1) => Some(Direction::Top),
        (-1, 0) => Some(Direction::Left),
        (1, 0) => Some(Direction::Right),
        _ => None, // Not adjacent or diagonal (we don't handle diagonals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cells(chunk_size: usize, fill_value: u16) -> Vec<Cell> {
        vec![Cell::new(fill_value); chunk_size * chunk_size]
    }

    fn create_numbered_cells(chunk_size: usize) -> Vec<Cell> {
        (0..chunk_size * chunk_size)
            .map(|i| Cell::new(i as u16))
            .collect()
    }

    #[test]
    fn test_edge_extractor_bottom() {
        let size = 4;
        let cells = create_numbered_cells(size);
        let extractor = EdgeExtractor::new(size);
        let chunk_id = ChunkId::new(0, 0);

        let edge = extractor.extract(&cells, Direction::Bottom, chunk_id);
        
        assert_eq!(edge.cells.len(), 4);
        assert_eq!(edge.cells[0].material, 0);
        assert_eq!(edge.cells[1].material, 1);
        assert_eq!(edge.cells[2].material, 2);
        assert_eq!(edge.cells[3].material, 3);
    }

    #[test]
    fn test_edge_extractor_top() {
        let size = 4;
        let cells = create_numbered_cells(size);
        let extractor = EdgeExtractor::new(size);
        let chunk_id = ChunkId::new(0, 0);

        let edge = extractor.extract(&cells, Direction::Top, chunk_id);
        
        assert_eq!(edge.cells.len(), 4);
        // Top row: indices 12, 13, 14, 15
        assert_eq!(edge.cells[0].material, 12);
        assert_eq!(edge.cells[1].material, 13);
        assert_eq!(edge.cells[2].material, 14);
        assert_eq!(edge.cells[3].material, 15);
    }

    #[test]
    fn test_edge_extractor_left() {
        let size = 4;
        let cells = create_numbered_cells(size);
        let extractor = EdgeExtractor::new(size);
        let chunk_id = ChunkId::new(0, 0);

        let edge = extractor.extract(&cells, Direction::Left, chunk_id);
        
        assert_eq!(edge.cells.len(), 4);
        // Left column: indices 0, 4, 8, 12
        assert_eq!(edge.cells[0].material, 0);
        assert_eq!(edge.cells[1].material, 4);
        assert_eq!(edge.cells[2].material, 8);
        assert_eq!(edge.cells[3].material, 12);
    }

    #[test]
    fn test_edge_extractor_right() {
        let size = 4;
        let cells = create_numbered_cells(size);
        let extractor = EdgeExtractor::new(size);
        let chunk_id = ChunkId::new(0, 0);

        let edge = extractor.extract(&cells, Direction::Right, chunk_id);
        
        assert_eq!(edge.cells.len(), 4);
        // Right column: indices 3, 7, 11, 15
        assert_eq!(edge.cells[0].material, 3);
        assert_eq!(edge.cells[1].material, 7);
        assert_eq!(edge.cells[2].material, 11);
        assert_eq!(edge.cells[3].material, 15);
    }

    #[test]
    fn test_edge_applicator() {
        let size = 4;
        let mut cells = create_test_cells(size, 0);
        let applicator = EdgeApplicator::new(size);
        
        // Create edge data as if from neighbor's TOP edge
        // Should be applied to our BOTTOM row
        let edge_cells: Vec<Cell> = (100..104).map(Cell::new).collect();
        let edge = EdgeData::new(edge_cells, Direction::Top, ChunkId::new(0, 1));
        
        applicator.apply(&mut cells, &edge);
        
        // Check bottom row was updated
        assert_eq!(cells[0].material, 100);
        assert_eq!(cells[1].material, 101);
        assert_eq!(cells[2].material, 102);
        assert_eq!(cells[3].material, 103);
    }

    #[test]
    fn test_edge_sharing_manager() {
        let size = 4;
        let manager = EdgeSharingManager::new(size);
        
        // Create two chunks: A at (0,0), B at (1,0) (B is to the right of A)
        let mut chunk_a = create_test_cells(size, 1);
        let mut chunk_b = create_test_cells(size, 2);
        
        let id_a = ChunkId::new(0, 0);
        let id_b = ChunkId::new(1, 0);
        
        // Share edges (B is to the RIGHT of A)
        manager.share_edges(&mut chunk_a, id_a, &mut chunk_b, id_b, Direction::Right);
        
        // A's right edge should now have B's values (from B's left edge)
        assert_eq!(chunk_a[3].material, 2);  // Right column of A
        
        // B's left edge should now have A's values (from A's right edge)
        assert_eq!(chunk_b[0].material, 1);  // Left column of B
    }

    #[test]
    fn test_get_shared_direction() {
        let center = ChunkId::new(0, 0);
        
        assert_eq!(get_shared_direction(center, ChunkId::new(1, 0)), Some(Direction::Right));
        assert_eq!(get_shared_direction(center, ChunkId::new(-1, 0)), Some(Direction::Left));
        assert_eq!(get_shared_direction(center, ChunkId::new(0, 1)), Some(Direction::Top));
        assert_eq!(get_shared_direction(center, ChunkId::new(0, -1)), Some(Direction::Bottom));
        
        // Non-adjacent
        assert_eq!(get_shared_direction(center, ChunkId::new(2, 0)), None);
        
        // Diagonal (not supported for edge sharing)
        assert_eq!(get_shared_direction(center, ChunkId::new(1, 1)), None);
    }
}
