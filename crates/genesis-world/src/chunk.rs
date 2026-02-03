//! Chunk data structure and serialization.

use genesis_common::{ChunkCoord, MagicBytes, SchemaVersion};
use genesis_kernel::Cell;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Chunk errors.
#[derive(Debug, Error)]
pub enum ChunkError {
    /// Serialization failed
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    /// Deserialization failed
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    /// Invalid magic bytes
    #[error("Invalid chunk format")]
    InvalidFormat,
    /// Version mismatch
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch {
        /// Expected version
        expected: String,
        /// Actual version
        actual: String,
    },
    /// Compression failed
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
}

/// Result type for chunk operations.
pub type ChunkResult<T> = Result<T, ChunkError>;

/// Chunk header for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHeader {
    /// Magic bytes for format identification
    pub magic: [u8; 4],
    /// Schema version
    pub version: SchemaVersion,
    /// Chunk X coordinate
    pub x: i32,
    /// Chunk Y coordinate
    pub y: i32,
    /// Chunk size in pixels
    pub size: u32,
    /// Compression type (0 = none, 1 = lz4)
    pub compression: u8,
    /// Layer (0 = overworld)
    pub layer: u8,
}

impl ChunkHeader {
    /// Creates a new header.
    #[must_use]
    pub fn new(coord: ChunkCoord, size: u32) -> Self {
        Self {
            magic: MagicBytes::CHUNK.0,
            version: SchemaVersion::CHUNK_HEADER,
            x: coord.x,
            y: coord.y,
            size,
            compression: 1, // LZ4 by default
            layer: 0,
        }
    }

    /// Validates the header.
    pub fn validate(&self) -> ChunkResult<()> {
        if self.magic != MagicBytes::CHUNK.0 {
            return Err(ChunkError::InvalidFormat);
        }
        if !SchemaVersion::CHUNK_HEADER.can_read(&self.version) {
            return Err(ChunkError::VersionMismatch {
                expected: SchemaVersion::CHUNK_HEADER.to_string(),
                actual: self.version.to_string(),
            });
        }
        Ok(())
    }
}

/// A chunk of the world containing cells.
#[derive(Debug)]
pub struct Chunk {
    /// Chunk coordinate
    coord: ChunkCoord,
    /// Chunk size (width and height in pixels)
    size: u32,
    /// Cell data (size × size cells)
    cells: Vec<Cell>,
    /// Whether chunk has been modified since last save
    dirty: bool,
}

impl Chunk {
    /// Creates a new empty chunk.
    #[must_use]
    pub fn new(coord: ChunkCoord, size: u32) -> Self {
        let cell_count = (size * size) as usize;
        Self {
            coord,
            size,
            cells: vec![Cell::default(); cell_count],
            dirty: false,
        }
    }

    /// Returns the chunk coordinate.
    #[must_use]
    pub const fn coord(&self) -> ChunkCoord {
        self.coord
    }

    /// Returns the chunk size.
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }

    /// Returns whether the chunk is dirty.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the chunk as dirty.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Marks the chunk as clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Gets a cell at local coordinates.
    #[must_use]
    pub fn get_cell(&self, x: u32, y: u32) -> Option<&Cell> {
        if x >= self.size || y >= self.size {
            return None;
        }
        let index = (y * self.size + x) as usize;
        self.cells.get(index)
    }

    /// Sets a cell at local coordinates.
    pub fn set_cell(&mut self, x: u32, y: u32, cell: Cell) -> bool {
        if x >= self.size || y >= self.size {
            return false;
        }
        let index = (y * self.size + x) as usize;
        if let Some(slot) = self.cells.get_mut(index) {
            *slot = cell;
            self.dirty = true;
            return true;
        }
        false
    }

    /// Returns a slice of all cells.
    #[must_use]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Returns a mutable slice of all cells.
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        self.dirty = true;
        &mut self.cells
    }

    /// Serializes the chunk to bytes.
    pub fn serialize(&self) -> ChunkResult<Vec<u8>> {
        let header = ChunkHeader::new(self.coord, self.size);

        // Serialize header
        let header_bytes = bincode::serialize(&header)
            .map_err(|e| ChunkError::SerializationFailed(e.to_string()))?;

        // Serialize cells
        let cell_bytes: Vec<u8> = self
            .cells
            .iter()
            .flat_map(|c| bytemuck::bytes_of(c).to_vec())
            .collect();

        // Compress cells
        let compressed = lz4_flex::compress_prepend_size(&cell_bytes);

        // Combine header + compressed cells
        let mut result = Vec::with_capacity(header_bytes.len() + compressed.len() + 4);
        result.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&header_bytes);
        result.extend_from_slice(&compressed);

        Ok(result)
    }

    /// Deserializes a chunk from bytes.
    pub fn deserialize(bytes: &[u8]) -> ChunkResult<Self> {
        if bytes.len() < 8 {
            return Err(ChunkError::DeserializationFailed("data too short".into()));
        }

        // Read header length
        let header_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        if bytes.len() < 4 + header_len {
            return Err(ChunkError::DeserializationFailed(
                "header length mismatch".into(),
            ));
        }

        // Deserialize header
        let header: ChunkHeader = bincode::deserialize(&bytes[4..4 + header_len])
            .map_err(|e| ChunkError::DeserializationFailed(e.to_string()))?;
        header.validate()?;

        // Decompress cells
        let compressed = &bytes[4 + header_len..];
        let cell_bytes = lz4_flex::decompress_size_prepended(compressed)
            .map_err(|e| ChunkError::CompressionFailed(e.to_string()))?;

        // Parse cells
        let cell_size = std::mem::size_of::<Cell>();
        let cell_count = (header.size * header.size) as usize;
        if cell_bytes.len() != cell_count * cell_size {
            return Err(ChunkError::DeserializationFailed(
                "cell data size mismatch".into(),
            ));
        }

        let cells: Vec<Cell> = cell_bytes
            .chunks_exact(cell_size)
            .map(|chunk| *bytemuck::from_bytes(chunk))
            .collect();

        Ok(Self {
            coord: ChunkCoord::new(header.x, header.y),
            size: header.size,
            cells,
            dirty: false,
        })
    }
}
