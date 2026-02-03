//! Error types for Project Genesis.

use thiserror::Error;

/// Top-level error type for Genesis operations.
#[derive(Debug, Error)]
pub enum GenesisError {
    /// GPU-related errors
    #[error("GPU error: {0}")]
    Gpu(#[from] GpuError),

    /// World/chunk errors
    #[error("World error: {0}")]
    World(#[from] WorldError),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Schema version mismatch
    #[error("Schema version mismatch: expected {expected}, got {actual}")]
    VersionMismatch {
        /// Expected version
        expected: String,
        /// Actual version found
        actual: String,
    },
}

/// GPU-specific errors.
#[derive(Debug, Error)]
pub enum GpuError {
    /// Failed to initialize GPU
    #[error("GPU initialization failed: {0}")]
    InitFailed(String),

    /// Shader compilation error
    #[error("Shader compilation failed: {0}")]
    ShaderError(String),

    /// Buffer allocation failed
    #[error("Buffer allocation failed: {0}")]
    BufferAlloc(String),

    /// Compute dispatch error
    #[error("Compute dispatch error: {0}")]
    DispatchError(String),

    /// GPU validation error
    #[error("GPU validation error: {0}")]
    ValidationError(String),
}

/// World and chunk errors.
#[derive(Debug, Error)]
pub enum WorldError {
    /// Chunk not found
    #[error("Chunk not found at ({x}, {y})")]
    ChunkNotFound {
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
    },

    /// Chunk load failed
    #[error("Failed to load chunk: {0}")]
    LoadFailed(String),

    /// Chunk save failed
    #[error("Failed to save chunk: {0}")]
    SaveFailed(String),

    /// Invalid chunk data
    #[error("Invalid chunk data: {0}")]
    InvalidData(String),
}

/// Result type alias for Genesis operations.
pub type GenesisResult<T> = Result<T, GenesisError>;
