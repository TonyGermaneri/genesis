//! Version types for schema compatibility.

use serde::{Deserialize, Serialize};

/// Schema version using semantic versioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Major version (breaking changes)
    pub major: u16,
    /// Minor version (backwards-compatible additions)
    pub minor: u16,
    /// Patch version (bug fixes)
    pub patch: u16,
}

impl SchemaVersion {
    /// Creates a new schema version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Current cell format version.
    pub const CELL_FORMAT: Self = Self::new(1, 0, 0);

    /// Current chunk header version.
    pub const CHUNK_HEADER: Self = Self::new(1, 0, 0);

    /// Current event bus protocol version.
    pub const EVENT_BUS: Self = Self::new(1, 0, 0);

    /// Current crafting recipe version.
    pub const CRAFTING_RECIPE: Self = Self::new(1, 0, 0);

    /// Current building definition version.
    pub const BUILDING_DEF: Self = Self::new(1, 0, 0);

    /// Checks if this version is compatible with another version.
    /// Compatible means same major version and this minor >= other minor.
    #[must_use]
    pub const fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }

    /// Checks if this version can read data from another version.
    #[must_use]
    pub const fn can_read(&self, data_version: &Self) -> bool {
        self.major == data_version.major
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Magic bytes for file format identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicBytes(pub [u8; 4]);

impl MagicBytes {
    /// Genesis chunk file magic bytes.
    pub const CHUNK: Self = Self(*b"GNCH");

    /// Genesis save file magic bytes.
    pub const SAVE: Self = Self(*b"GNSV");

    /// Genesis mod package magic bytes.
    pub const MOD: Self = Self(*b"GNMD");
}
