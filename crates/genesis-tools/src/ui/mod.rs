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
//! - World Tools (biome, noise, weather, faction, material configuration)
//! - Sprite Builder (character sprite sheet frame definitions)

// ============================================================================
// Constrained Window Helpers
// ============================================================================

/// Screen constraints for windows with a configurable margin.
///
/// All windows should use these constraints to ensure they never overflow
/// the screen boundaries.
#[derive(Debug, Clone, Copy)]
pub struct ScreenConstraints {
    /// Maximum width the window can be
    pub max_width: f32,
    /// Maximum height the window can be
    pub max_height: f32,
    /// Available width (screen width minus margins)
    pub available_width: f32,
    /// Available height (screen height minus margins)
    pub available_height: f32,
}

impl ScreenConstraints {
    /// Default margin in pixels from screen edges.
    pub const DEFAULT_MARGIN: f32 = 10.0;

    /// Create screen constraints from an egui context with the default margin.
    pub fn from_context(ctx: &egui::Context) -> Self {
        Self::from_context_with_margin(ctx, Self::DEFAULT_MARGIN)
    }

    /// Create screen constraints from an egui context with a custom margin.
    pub fn from_context_with_margin(ctx: &egui::Context, margin: f32) -> Self {
        let screen_rect = ctx.screen_rect();
        let available_width = (screen_rect.width() - margin * 2.0).max(100.0);
        let available_height = (screen_rect.height() - margin * 2.0).max(100.0);

        Self {
            max_width: available_width,
            max_height: available_height,
            available_width,
            available_height,
        }
    }

    /// Get a constrained default width (returns the smaller of requested or available).
    pub fn constrained_width(&self, requested: f32) -> f32 {
        requested.min(self.max_width)
    }

    /// Get a constrained default height (returns the smaller of requested or available).
    pub fn constrained_height(&self, requested: f32) -> f32 {
        requested.min(self.max_height)
    }
}

/// Extension trait to apply screen constraints to egui windows.
pub trait ConstrainedWindow {
    /// Apply screen constraints (max_width and max_height) to the window.
    ///
    /// This should be called on any egui::Window to ensure it never overflows
    /// the screen boundaries.
    fn with_screen_constraints(self, constraints: &ScreenConstraints) -> Self;

    /// Apply screen constraints and also constrain the default size.
    fn with_constrained_defaults(
        self,
        constraints: &ScreenConstraints,
        default_width: f32,
        default_height: f32,
    ) -> Self;
}

impl<'a> ConstrainedWindow for egui::Window<'a> {
    fn with_screen_constraints(self, constraints: &ScreenConstraints) -> Self {
        self.max_width(constraints.max_width)
            .max_height(constraints.max_height)
    }

    fn with_constrained_defaults(
        self,
        constraints: &ScreenConstraints,
        default_width: f32,
        default_height: f32,
    ) -> Self {
        self.default_width(constraints.constrained_width(default_width))
            .default_height(constraints.constrained_height(default_height))
            .max_width(constraints.max_width)
            .max_height(constraints.max_height)
    }
}

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
pub mod sprite_builder;
pub mod stats;
pub mod workbench_ui;
pub mod world_tools;

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
pub use sprite_builder::*;
pub use stats::*;
pub use workbench_ui::*;
pub use world_tools::*;
