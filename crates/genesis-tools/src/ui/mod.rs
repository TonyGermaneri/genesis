//! UI module for game interface components.
//!
//! This module provides specialized UI widgets and panels:
//! - Inventory panel (6x9 grid)
//! - Player stats HUD (health, hunger, stamina)
//! - Environment HUD (time, weather)
//! - Chunk-based minimap
//! - Biome visualization and debug tools
//! - Audio settings and debug tools
//! - Crafting UI (grid, recipe book, preview, workbench)
//! - Combat UI (health bars, combat HUD, equipment stats, combat debug)
//! - Save/Load UI (save menu, save preview, autosave indicator, save management)
//! - Menu UI (main menu, pause menu, options menu, new game wizard)

pub mod audio_debug;
pub mod audio_settings;
pub mod autosave_indicator;
pub mod biome;
pub mod combat_debug;
pub mod combat_hud;
pub mod crafting_grid;
pub mod crafting_preview;
pub mod environment;
pub mod equipment_stats;
pub mod health_bars;
pub mod inventory;
pub mod main_menu;
pub mod minimap;
pub mod new_game_wizard;
pub mod options_menu;
pub mod pause_menu;
pub mod recipe_book;
pub mod save_management;
pub mod save_menu;
pub mod save_preview;
pub mod sound_test;
pub mod stats;
pub mod workbench_ui;

pub use audio_debug::*;
pub use audio_settings::*;
pub use autosave_indicator::*;
pub use biome::*;
pub use combat_debug::*;
pub use combat_hud::*;
pub use crafting_grid::*;
pub use crafting_preview::*;
pub use environment::*;
pub use equipment_stats::*;
pub use health_bars::*;
pub use inventory::*;
pub use main_menu::*;
pub use minimap::*;
pub use new_game_wizard::*;
pub use options_menu::*;
pub use pause_menu::*;
pub use recipe_book::*;
pub use save_management::*;
pub use save_menu::*;
pub use save_preview::*;
pub use sound_test::*;
pub use stats::*;
pub use workbench_ui::*;
