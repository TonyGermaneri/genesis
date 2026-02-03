//! Engine configuration.

/// Engine configuration parameters.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EngineConfig {
    /// Window width in pixels
    pub window_width: u32,
    /// Window height in pixels
    pub window_height: u32,
    /// Target frames per second
    pub target_fps: u32,
    /// Chunk size in pixels
    pub chunk_size: u32,
    /// View distance in chunks
    pub view_distance: u32,
    /// Enable GPU validation layers
    pub gpu_validation: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            target_fps: 60,
            chunk_size: 256,
            view_distance: 3,
            gpu_validation: cfg!(debug_assertions),
        }
    }
}
