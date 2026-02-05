//! Engine configuration.
//!
//! Provides configurable parameters for window, graphics, world, and debug settings.
//! Configuration can be loaded from and saved to a file.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Configuration file name.
const CONFIG_FILE: &str = "genesis.toml";

/// Engine configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    // === Window Settings ===
    /// Window width in pixels
    pub window_width: u32,
    /// Window height in pixels
    pub window_height: u32,
    /// Start in fullscreen mode
    pub fullscreen: bool,
    /// Enable VSync
    pub vsync: bool,
    /// Target frames per second (when VSync is off)
    pub target_fps: u32,

    // === World Settings ===
    /// World seed (None = random)
    pub world_seed: Option<u64>,
    /// Render distance in chunks
    pub render_distance: u32,
    /// Chunk size in cells
    pub chunk_size: u32,

    // === Graphics Settings ===
    /// Cell render scale (pixels per cell)
    pub cell_scale: f32,
    /// Camera zoom level (1.0-20.0)
    pub camera_zoom: f32,
    /// Enable particle effects
    pub enable_particles: bool,
    /// Enable dynamic lighting
    pub enable_lighting: bool,
    /// Enable ambient occlusion
    pub enable_ao: bool,

    // === Audio Settings ===
    /// Master volume (0.0 - 1.0)
    pub master_volume: f32,
    /// Music volume (0.0 - 1.0)
    pub music_volume: f32,
    /// Sound effects volume (0.0 - 1.0)
    pub sfx_volume: f32,

    // === Debug Settings ===
    /// Show FPS counter
    pub show_fps: bool,
    /// Show debug overlay (F3)
    pub show_debug_overlay: bool,
    /// Enable GPU validation layers
    pub gpu_validation: bool,
    /// Enable performance profiling
    pub enable_profiling: bool,

    // === Gameplay Settings ===
    /// Mouse sensitivity
    pub mouse_sensitivity: f32,
    /// Invert Y axis
    pub invert_y: bool,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u32,

    // === Accessibility ===
    /// UI scale multiplier
    pub ui_scale: f32,
    /// Enable high contrast mode
    pub high_contrast: bool,
    /// Enable screen shake
    pub screen_shake: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            // Window
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            vsync: true,
            target_fps: 60,

            // World
            world_seed: None,
            render_distance: 4,
            chunk_size: 256,

            // Graphics
            cell_scale: 4.0,
            camera_zoom: 10.0,
            enable_particles: true,
            enable_lighting: true,
            enable_ao: true,

            // Audio
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,

            // Debug
            show_fps: true,
            show_debug_overlay: false,
            gpu_validation: cfg!(debug_assertions),
            enable_profiling: false,

            // Gameplay
            mouse_sensitivity: 1.0,
            invert_y: false,
            auto_save_interval: 300, // 5 minutes

            // Accessibility
            ui_scale: 1.0,
            high_contrast: false,
            screen_shake: true,
        }
    }
}

impl EngineConfig {
    /// Load configuration from the default file location.
    /// Returns default config if file doesn't exist.
    pub fn load() -> Self {
        Self::load_from(Self::config_path())
    }

    /// Load configuration from a specific path.
    /// Returns default config if file doesn't exist or is invalid.
    pub fn load_from<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        if !path.exists() {
            info!("Config file not found, using defaults");
            return Self::default();
        }

        match fs::File::open(path) {
            Ok(mut file) => {
                let mut contents = String::new();
                if let Err(e) = file.read_to_string(&mut contents) {
                    warn!("Failed to read config file: {e}");
                    return Self::default();
                }

                match toml::from_str(&contents) {
                    Ok(config) => {
                        info!("Loaded config from {}", path.display());
                        config
                    },
                    Err(e) => {
                        warn!("Failed to parse config file: {e}");
                        Self::default()
                    },
                }
            },
            Err(e) => {
                warn!("Failed to open config file: {e}");
                Self::default()
            },
        }
    }

    /// Save configuration to the default file location.
    pub fn save(&self) -> io::Result<()> {
        self.save_to(Self::config_path())
    }

    /// Save configuration to a specific path.
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file = fs::File::create(path)?;
        file.write_all(contents.as_bytes())?;

        info!("Saved config to {}", path.display());
        Ok(())
    }

    /// Get the default configuration file path.
    fn config_path() -> PathBuf {
        // Try to use standard config directory
        if let Some(config_dir) = dirs_config_path() {
            config_dir.join("genesis").join(CONFIG_FILE)
        } else {
            // Fall back to current directory
            PathBuf::from(CONFIG_FILE)
        }
    }

    /// Validate and clamp configuration values to sensible ranges.
    pub fn validate(&mut self) {
        // Window size
        self.window_width = self.window_width.clamp(640, 7680);
        self.window_height = self.window_height.clamp(480, 4320);
        self.target_fps = self.target_fps.clamp(30, 240);

        // World
        self.render_distance = self.render_distance.clamp(1, 16);
        self.chunk_size = self.chunk_size.clamp(64, 512);

        // Graphics
        self.cell_scale = self.cell_scale.clamp(1.0, 16.0);

        // Audio
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.music_volume = self.music_volume.clamp(0.0, 1.0);
        self.sfx_volume = self.sfx_volume.clamp(0.0, 1.0);

        // Gameplay
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(0.1, 5.0);

        // Accessibility
        self.ui_scale = self.ui_scale.clamp(0.5, 3.0);
    }

    /// Check if this is a debug build configuration.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_debug(&self) -> bool {
        self.gpu_validation || self.enable_profiling || self.show_debug_overlay
    }
}

/// Get platform-specific config directory.
fn dirs_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join("Library/Application Support"))
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.window_width, 1280);
        assert_eq!(config.window_height, 720);
        assert!(config.vsync);
        assert_eq!(config.render_distance, 4);
    }

    #[test]
    fn test_config_validation() {
        let mut config = EngineConfig::default();

        // Set invalid values
        config.window_width = 100;
        config.master_volume = 2.0;
        config.mouse_sensitivity = 0.0;

        config.validate();

        // Should be clamped
        assert_eq!(config.window_width, 640);
        assert_eq!(config.master_volume, 1.0);
        assert!((config.mouse_sensitivity - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.toml");

        // Create and save config
        let mut config = EngineConfig::default();
        config.window_width = 1920;
        config.vsync = false;
        config.world_seed = Some(12345);

        config.save_to(&config_path).expect("Failed to save config");

        // Load and verify
        let loaded = EngineConfig::load_from(&config_path);
        assert_eq!(loaded.window_width, 1920);
        assert!(!loaded.vsync);
        assert_eq!(loaded.world_seed, Some(12345));
    }

    #[test]
    fn test_config_load_missing_file() {
        let config = EngineConfig::load_from("/nonexistent/path/config.toml");
        // Should return defaults
        assert_eq!(config.window_width, 1280);
    }

    #[test]
    fn test_config_is_debug() {
        let mut config = EngineConfig::default();
        config.gpu_validation = false;
        config.enable_profiling = false;
        config.show_debug_overlay = false;

        assert!(!config.is_debug());

        config.show_debug_overlay = true;
        assert!(config.is_debug());
    }

    #[test]
    fn test_config_toml_serialization() {
        let config = EngineConfig::default();
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");

        assert!(toml_str.contains("window_width"));
        assert!(toml_str.contains("vsync"));
    }
}
