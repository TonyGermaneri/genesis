//! # Genesis World
//!
//! World management for Project Genesis.
//!
//! This crate handles:
//! - Chunk loading/unloading
//! - Persistent world streaming
//! - Procedural generation
//! - World-to-disk serialization

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod chunk;
pub mod generation;
pub mod streaming;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::chunk::*;
    pub use crate::generation::*;
    pub use crate::streaming::*;
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_common::ChunkCoord;

    #[test]
    fn test_chunk_creation() {
        let coord = ChunkCoord::new(0, 0);
        let chunk = Chunk::new(coord, 256);
        assert_eq!(chunk.coord(), coord);
        assert!(!chunk.is_dirty());
    }

    #[test]
    fn test_chunk_serialization() {
        let coord = ChunkCoord::new(1, 2);
        let chunk = Chunk::new(coord, 64);
        let bytes = chunk.serialize().expect("serialize failed");
        let loaded = Chunk::deserialize(&bytes).expect("deserialize failed");
        assert_eq!(loaded.coord(), coord);
    }
}
