//! Coordinate types for world, chunk, and local positions.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// World coordinate in pixels (global position).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct WorldCoord {
    /// X coordinate in world space
    pub x: i64,
    /// Y coordinate in world space
    pub y: i64,
}

impl WorldCoord {
    /// Creates a new world coordinate.
    #[must_use]
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    /// Converts to chunk coordinate given chunk size.
    #[must_use]
    pub const fn to_chunk_coord(self, chunk_size: u32) -> ChunkCoord {
        let size = chunk_size as i64;
        ChunkCoord {
            x: self.x.div_euclid(size) as i32,
            y: self.y.div_euclid(size) as i32,
        }
    }

    /// Converts to local coordinate within a chunk.
    #[must_use]
    pub const fn to_local_coord(self, chunk_size: u32) -> LocalCoord {
        let size = chunk_size as i64;
        LocalCoord {
            x: self.x.rem_euclid(size) as u16,
            y: self.y.rem_euclid(size) as u16,
        }
    }
}

/// Chunk coordinate (identifies a chunk in the world grid).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct ChunkCoord {
    /// X coordinate in chunk space
    pub x: i32,
    /// Y coordinate in chunk space
    pub y: i32,
}

impl ChunkCoord {
    /// Creates a new chunk coordinate.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Converts to world coordinate (top-left corner of chunk).
    #[must_use]
    pub const fn to_world_coord(self, chunk_size: u32) -> WorldCoord {
        WorldCoord {
            x: (self.x as i64) * (chunk_size as i64),
            y: (self.y as i64) * (chunk_size as i64),
        }
    }
}

/// Local coordinate within a chunk (0 to chunk_size-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct LocalCoord {
    /// X coordinate within chunk
    pub x: u16,
    /// Y coordinate within chunk
    pub y: u16,
}

impl LocalCoord {
    /// Creates a new local coordinate.
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Converts to linear index for array access.
    #[must_use]
    pub const fn to_index(self, chunk_size: u32) -> usize {
        (self.y as usize) * (chunk_size as usize) + (self.x as usize)
    }

    /// Creates from linear index.
    #[must_use]
    pub const fn from_index(index: usize, chunk_size: u32) -> Self {
        let size = chunk_size as usize;
        Self {
            x: (index % size) as u16,
            y: (index / size) as u16,
        }
    }
}
