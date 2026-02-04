//! Save file versioning and migration system.
//!
//! This module provides:
//! - Version tracking for save files
//! - Migration functions between versions
//! - Backward compatibility support
//! - Version mismatch warnings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn};

/// Current save format version.
pub const CURRENT_SAVE_VERSION: u32 = 1;

/// Minimum supported version for loading.
pub const MIN_SUPPORTED_VERSION: u32 = 1;

/// Maximum forward compatibility version.
pub const MAX_FORWARD_VERSION: u32 = 1;

/// Errors related to save versioning.
#[derive(Debug, Error)]
pub enum VersionError {
    /// Version too old and cannot be migrated.
    #[error("Save version {found} is too old (minimum: {minimum})")]
    TooOld {
        /// Found version.
        found: u32,
        /// Minimum supported version.
        minimum: u32,
    },

    /// Version too new (from future game version).
    #[error("Save version {found} is from a newer game version (current: {current})")]
    TooNew {
        /// Found version.
        found: u32,
        /// Current supported version.
        current: u32,
    },

    /// Migration failed.
    #[error("Failed to migrate from version {from} to {to}: {reason}")]
    MigrationFailed {
        /// Source version.
        from: u32,
        /// Target version.
        to: u32,
        /// Reason for failure.
        reason: String,
    },

    /// Unknown version.
    #[error("Unknown save version: {0}")]
    Unknown(u32),
}

/// Result type for version operations.
pub type VersionResult<T> = Result<T, VersionError>;

/// Represents a save file version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SaveVersion {
    /// Major version (breaking changes).
    pub major: u16,
    /// Minor version (compatible additions).
    pub minor: u16,
}

impl SaveVersion {
    /// Creates a new save version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Creates version from a single u32.
    #[must_use]
    pub const fn from_u32(version: u32) -> Self {
        Self {
            major: (version >> 16) as u16,
            minor: (version & 0xFFFF) as u16,
        }
    }

    /// Converts version to u32.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        ((self.major as u32) << 16) | (self.minor as u32)
    }

    /// Returns the current version.
    #[must_use]
    pub const fn current() -> Self {
        Self::from_u32(CURRENT_SAVE_VERSION)
    }

    /// Checks if this version is compatible with another.
    #[must_use]
    pub fn is_compatible_with(self, other: Self) -> bool {
        // Same major version is compatible
        self.major == other.major
    }

    /// Checks if this version can be loaded.
    #[must_use]
    pub fn is_loadable(self) -> bool {
        let version = self.to_u32();
        version >= MIN_SUPPORTED_VERSION && version <= MAX_FORWARD_VERSION
    }

    /// Checks if migration is needed.
    #[must_use]
    pub fn needs_migration(self) -> bool {
        self.to_u32() < CURRENT_SAVE_VERSION
    }
}

impl Default for SaveVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl std::fmt::Display for SaveVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Compatibility level between versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionCompatibility {
    /// Fully compatible, no changes needed.
    Full,
    /// Compatible with migration.
    MigrationRequired,
    /// Forward compatible (newer save, older game).
    ForwardCompatible,
    /// Incompatible, cannot load.
    Incompatible,
}

impl VersionCompatibility {
    /// Checks if the save can be loaded.
    #[must_use]
    pub fn can_load(self) -> bool {
        !matches!(self, Self::Incompatible)
    }

    /// Checks if migration is needed.
    #[must_use]
    pub fn needs_migration(self) -> bool {
        matches!(self, Self::MigrationRequired)
    }

    /// Returns a warning message if any.
    #[must_use]
    pub fn warning_message(self) -> Option<&'static str> {
        match self {
            Self::Full => None,
            Self::MigrationRequired => Some("This save will be migrated to the current format."),
            Self::ForwardCompatible => {
                Some("This save is from a newer game version. Some features may not work.")
            }
            Self::Incompatible => {
                Some("This save is incompatible with the current game version.")
            }
        }
    }
}

/// Checks compatibility between save version and current version.
#[must_use]
pub fn check_compatibility(save_version: u32) -> VersionCompatibility {
    if save_version == CURRENT_SAVE_VERSION {
        VersionCompatibility::Full
    } else if save_version < MIN_SUPPORTED_VERSION {
        VersionCompatibility::Incompatible
    } else if save_version < CURRENT_SAVE_VERSION {
        VersionCompatibility::MigrationRequired
    } else if save_version <= MAX_FORWARD_VERSION {
        VersionCompatibility::ForwardCompatible
    } else {
        VersionCompatibility::Incompatible
    }
}

/// Version information stored in save files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveVersionInfo {
    /// Save format version.
    pub format_version: u32,
    /// Game version string.
    pub game_version: String,
    /// Engine version.
    pub engine_version: String,
    /// Creation timestamp.
    pub created_at: u64,
    /// Last modified timestamp.
    pub modified_at: u64,
    /// Migration history.
    pub migration_history: Vec<MigrationRecord>,
}

impl Default for SaveVersionInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveVersionInfo {
    /// Creates new version info with current version.
    #[must_use]
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            format_version: CURRENT_SAVE_VERSION,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            engine_version: "1.0.0".to_string(),
            created_at: now,
            modified_at: now,
            migration_history: Vec::new(),
        }
    }

    /// Updates the modified timestamp.
    pub fn touch(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};

        self.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(self.modified_at);
    }

    /// Records a migration.
    pub fn record_migration(&mut self, from: u32, to: u32) {
        self.migration_history.push(MigrationRecord {
            from_version: from,
            to_version: to,
            timestamp: self.modified_at,
        });
        self.format_version = to;
        self.touch();
    }

    /// Checks if the save has been migrated.
    #[must_use]
    pub fn was_migrated(&self) -> bool {
        !self.migration_history.is_empty()
    }
}

/// Record of a migration operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Source version.
    pub from_version: u32,
    /// Target version.
    pub to_version: u32,
    /// When migration occurred.
    pub timestamp: u64,
}

/// Trait for migrating save data between versions.
pub trait Migration {
    /// Source version this migration applies to.
    fn source_version(&self) -> u32;

    /// Target version after migration.
    fn target_version(&self) -> u32;

    /// Migrates the data.
    fn migrate(&self, data: &mut serde_json::Value) -> VersionResult<()>;

    /// Description of changes made.
    fn description(&self) -> &str;
}

/// Registry of available migrations.
pub struct MigrationRegistry {
    /// Migrations keyed by source version.
    migrations: HashMap<u32, Box<dyn Migration + Send + Sync>>,
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRegistry {
    /// Creates a new migration registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }

    /// Creates a registry with all built-in migrations.
    #[must_use]
    pub fn with_builtin_migrations() -> Self {
        let registry = Self::new();
        // Register migrations as they're created
        // registry.register(Box::new(MigrationV1ToV2));
        registry
    }

    /// Registers a migration.
    pub fn register(&mut self, migration: Box<dyn Migration + Send + Sync>) {
        let source = migration.source_version();
        self.migrations.insert(source, migration);
    }

    /// Gets a migration for a specific source version.
    #[must_use]
    pub fn get(&self, source_version: u32) -> Option<&(dyn Migration + Send + Sync)> {
        self.migrations.get(&source_version).map(|m| m.as_ref())
    }

    /// Migrates data from source to target version.
    pub fn migrate(
        &self,
        data: &mut serde_json::Value,
        from_version: u32,
        to_version: u32,
    ) -> VersionResult<Vec<MigrationRecord>> {
        if from_version >= to_version {
            return Ok(Vec::new());
        }

        let mut current_version = from_version;
        let mut records = Vec::new();

        while current_version < to_version {
            if let Some(migration) = self.get(current_version) {
                info!(
                    "Applying migration: {} ({} -> {})",
                    migration.description(),
                    current_version,
                    migration.target_version()
                );

                migration.migrate(data)?;

                records.push(MigrationRecord {
                    from_version: current_version,
                    to_version: migration.target_version(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                });

                current_version = migration.target_version();
            } else {
                // No migration available - this could be a gap
                warn!(
                    "No migration found for version {}, skipping to {}",
                    current_version,
                    current_version + 1
                );
                current_version += 1;
            }
        }

        Ok(records)
    }

    /// Lists all available migrations.
    #[must_use]
    pub fn list_migrations(&self) -> Vec<(u32, u32, &str)> {
        let mut result: Vec<_> = self
            .migrations
            .values()
            .map(|m| (m.source_version(), m.target_version(), m.description()))
            .collect();
        result.sort_by_key(|(src, _, _)| *src);
        result
    }
}

/// Validates that a save can be loaded with the current version.
pub fn validate_version(version: u32) -> VersionResult<VersionCompatibility> {
    let compat = check_compatibility(version);

    match compat {
        VersionCompatibility::Incompatible => {
            if version < MIN_SUPPORTED_VERSION {
                Err(VersionError::TooOld {
                    found: version,
                    minimum: MIN_SUPPORTED_VERSION,
                })
            } else {
                Err(VersionError::TooNew {
                    found: version,
                    current: CURRENT_SAVE_VERSION,
                })
            }
        }
        _ => Ok(compat),
    }
}

/// Version change log entry.
#[derive(Debug, Clone)]
pub struct VersionChangeLog {
    /// Version number.
    pub version: u32,
    /// Changes made in this version.
    pub changes: Vec<String>,
    /// Breaking changes flag.
    pub breaking: bool,
}

/// Gets the change log for all versions.
#[must_use]
pub fn get_version_changelog() -> Vec<VersionChangeLog> {
    vec![VersionChangeLog {
        version: 1,
        changes: vec![
            "Initial save format".to_string(),
            "Player position and stats".to_string(),
            "Crafting data".to_string(),
            "Combat data".to_string(),
        ],
        breaking: false,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_version_new() {
        let version = SaveVersion::new(1, 2);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
    }

    #[test]
    fn test_save_version_conversion() {
        let version = SaveVersion::new(1, 5);
        let as_u32 = version.to_u32();
        let back = SaveVersion::from_u32(as_u32);
        assert_eq!(version, back);
    }

    #[test]
    fn test_save_version_current() {
        let current = SaveVersion::current();
        assert_eq!(current.to_u32(), CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_save_version_compatibility() {
        let v1_0 = SaveVersion::new(1, 0);
        let v1_1 = SaveVersion::new(1, 1);
        let v2_0 = SaveVersion::new(2, 0);

        assert!(v1_0.is_compatible_with(v1_1));
        assert!(!v1_0.is_compatible_with(v2_0));
    }

    #[test]
    fn test_save_version_display() {
        let version = SaveVersion::new(1, 5);
        assert_eq!(format!("{version}"), "1.5");
    }

    #[test]
    fn test_check_compatibility_full() {
        let compat = check_compatibility(CURRENT_SAVE_VERSION);
        assert_eq!(compat, VersionCompatibility::Full);
        assert!(compat.can_load());
        assert!(!compat.needs_migration());
        assert!(compat.warning_message().is_none());
    }

    #[test]
    fn test_check_compatibility_too_old() {
        // Version 0 should be incompatible if MIN_SUPPORTED_VERSION > 0
        if MIN_SUPPORTED_VERSION > 0 {
            let compat = check_compatibility(0);
            assert_eq!(compat, VersionCompatibility::Incompatible);
            assert!(!compat.can_load());
        }
    }

    #[test]
    fn test_save_version_info_new() {
        let info = SaveVersionInfo::new();
        assert_eq!(info.format_version, CURRENT_SAVE_VERSION);
        assert!(!info.was_migrated());
    }

    #[test]
    fn test_save_version_info_migration() {
        let mut info = SaveVersionInfo::new();
        info.record_migration(1, 2);

        assert!(info.was_migrated());
        assert_eq!(info.format_version, 2);
        assert_eq!(info.migration_history.len(), 1);
    }

    #[test]
    fn test_migration_registry() {
        let registry = MigrationRegistry::new();
        let migrations = registry.list_migrations();
        assert!(migrations.is_empty());
    }

    #[test]
    fn test_validate_version_current() {
        let result = validate_version(CURRENT_SAVE_VERSION);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), VersionCompatibility::Full);
    }

    #[test]
    fn test_validate_version_too_new() {
        let result = validate_version(CURRENT_SAVE_VERSION + 100);
        assert!(result.is_err());
        assert!(matches!(result, Err(VersionError::TooNew { .. })));
    }

    #[test]
    fn test_version_changelog() {
        let changelog = get_version_changelog();
        assert!(!changelog.is_empty());
        assert_eq!(changelog[0].version, 1);
    }

    #[test]
    fn test_version_error_display() {
        let err = VersionError::TooOld {
            found: 1,
            minimum: 2,
        };
        let msg = format!("{err}");
        assert!(msg.contains("too old"));

        let err = VersionError::TooNew {
            found: 10,
            current: 5,
        };
        let msg = format!("{err}");
        assert!(msg.contains("newer game version"));
    }

    #[test]
    fn test_migration_record() {
        let record = MigrationRecord {
            from_version: 1,
            to_version: 2,
            timestamp: 12345,
        };
        assert_eq!(record.from_version, 1);
        assert_eq!(record.to_version, 2);
    }
}
