//! # Genesis Common
//!
//! Common types, utilities, and shared abstractions for Project Genesis.
//!
//! This crate provides foundational types used across all Genesis subsystems:
//! - Coordinate types (world, chunk, local)
//! - ID types (EntityId, ChunkId, etc.)
//! - Version information for schemas
//! - Common error types
//! - Prelude for convenient imports

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod coords;
pub mod error;
pub mod ids;
pub mod version;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::coords::*;
    pub use crate::error::*;
    pub use crate::ids::*;
    pub use crate::version::*;
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coords_conversion() {
        let world = WorldCoord::new(100, 200);
        let chunk = world.to_chunk_coord(32);
        let local = world.to_local_coord(32);

        assert_eq!(chunk, ChunkCoord::new(3, 6));
        assert_eq!(local, LocalCoord::new(4, 8));
    }

    #[test]
    fn test_entity_id_generation() {
        let id1 = EntityId::new();
        let id2 = EntityId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 1, 0);
        let v3 = SchemaVersion::new(2, 0, 0);

        // v2 can read v1 data (newer version reading older data)
        assert!(v2.is_compatible_with(&v1));
        // Different major versions are incompatible
        assert!(!v1.is_compatible_with(&v3));
    }
}
