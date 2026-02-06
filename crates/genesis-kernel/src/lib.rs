//! # Genesis Kernel
//!
//! GPU compute kernel for pixel-cell simulation.
//!
//! This crate provides the core GPU-accelerated simulation:
//! - Pixel-cell simulation (each pixel is a simulated cell)
//! - Buffer layouts for GPU compute
//! - Double-buffered cell storage for efficient GPU simulation
//! - Intent buffer for CPU → GPU communication
//! - Event buffer for GPU → CPU communication
//! - GPU validation harness
//! - Performance benchmarking utilities
//!
//! ## Architecture
//!
//! The kernel runs on the GPU and is the physical authority for the world.
//! The CPU entity layer (gameplay crate) submits intents which the kernel
//! executes as part of the simulation step.
//!
//! ## Double Buffering
//!
//! The simulation uses double-buffering to ensure correct GPU execution:
//! - Two cell buffers alternate between input (read) and output (write)
//! - Each simulation step reads from one buffer and writes to the other
//! - Buffers are swapped after each step
//!
//! ## Intent/Event System
//!
//! Communication between CPU and GPU uses bounded queues:
//! - **Intents** (CPU → GPU): Actions to apply to cells (place material, ignite, etc.)
//! - **Events** (GPU → CPU): Notifications about cell state changes
//!
//! ## Validation
//!
//! In debug builds, wgpu validation is enabled to catch GPU errors early.
//! Use `create_validated_instance()` to create a wgpu instance with appropriate settings.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod audio;
pub mod audio_backend;
pub mod audio_legacy;
pub mod audio_resource;
pub mod audio_spatial;
pub mod autotile_atlas;
pub mod autotile_render;
pub mod benchmark;
pub mod biome;
pub mod buffer;
pub mod camera;
pub mod cell;
pub mod chunk;
pub mod chunk_manager;
pub mod collision;
pub mod combat_collision;
pub mod combat_particles;
pub mod compute;
pub mod crafting_anim;
pub mod crafting_grid;
pub mod damage_render;
pub mod edge;
pub mod event;
pub mod intent;
pub mod item_stack;
pub mod lighting;
pub mod npc_collision;
pub mod npc_render;
pub mod particles;
pub mod player_sprite;
pub mod projectile;
pub mod quadtree;
pub mod readback;
pub mod render;
pub mod streaming;
pub mod streaming_terrain;
pub mod terrain_assets;
pub mod terrain_atlas;
pub mod terrain_render;
pub mod texture_loader;
pub mod textured_render;
pub mod textured_terrain;
pub mod topdown_physics;
pub mod validation;
pub mod workbench;
pub mod world_init;
pub mod worldgen;

// Save/Load infrastructure
pub mod chunk_serialize;
pub mod incremental_save;
pub mod save_compression;
pub mod world_region;

// Main Menu & Options infrastructure
pub mod menu_backdrop;
pub mod resolution;
pub mod screenshot;
pub mod transitions;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::audio::*;
    pub use crate::benchmark::*;
    pub use crate::biome::*;
    pub use crate::buffer::*;
    pub use crate::camera::*;
    pub use crate::cell::*;
    pub use crate::chunk::*;
    pub use crate::chunk_manager::*;
    pub use crate::collision::*;
    // Combat modules use explicit imports to avoid conflicts
    pub use crate::combat_collision::{
        CollisionResult, CombatBoxManager, CombatCollider, FrameRange, Hitbox,
        HitboxSequenceBuilder, HitboxShape, Hurtbox,
    };
    pub use crate::combat_particles::{
        BloodSplatterEffect, CombatEffectType, CombatParticle, CombatParticleInstance,
        CombatParticleManager, HitSparkEffect, ImpactDustEffect,
    };
    pub use crate::compute::*;
    // Crafting modules use explicit imports to avoid conflicts
    pub use crate::crafting_anim::{
        AnimationPreset, AnimationState, CraftingProgress, SoundTrigger,
    };
    pub use crate::crafting_grid::{
        CraftingGrid, CraftingResult, ExtractedPattern, ItemSlot, RecipeMatcher, RecipePattern,
        RecipeType,
    };
    pub use crate::damage_render::{
        AnimationStyle, DamageNumber, DamageNumberBatch, DamageNumberInstance, DamageNumberManager,
        DamageType,
    };
    pub use crate::edge::*;
    pub use crate::event::*;
    pub use crate::intent::*;
    pub use crate::item_stack::{
        CompactStack, ItemMetadata, ItemStack, ItemStackBuilder, StackResult,
    };
    pub use crate::lighting::*;
    pub use crate::npc_collision::*;
    pub use crate::npc_render::*;
    pub use crate::particles::*;
    pub use crate::player_sprite::{
        PlayerAnimState, PlayerDirection, PlayerSpriteConfig, PlayerSpriteInstance,
        PlayerSpriteRenderer, PlayerSpriteState,
    };
    pub use crate::projectile::{
        Projectile, ProjectileCollision, ProjectileInstance, ProjectileManager, ProjectileState,
        ProjectileType, TrajectoryPredictor,
    };
    pub use crate::quadtree::*;
    pub use crate::readback::*;
    pub use crate::render::*;
    pub use crate::streaming::*;
    pub use crate::streaming_terrain::{
        StreamingChunk, StreamingConfig, StreamingStats, StreamingTerrain,
        DEFAULT_LOAD_RADIUS, DEFAULT_SIMULATION_RADIUS, DEFAULT_UNLOAD_RADIUS,
    };
    pub use crate::terrain_assets::{
        BiomeTerrainMapping, PixelRGBA, TerrainAssetManifest, TerrainCategory, TerrainTile,
        TilePosition, PIXELS_PER_TILE, TILE_SIZE,
    };
    pub use crate::terrain_atlas::{
        NeighborMask, PixelEffectState, TerrainAtlasParams, TerrainTextureAtlas, TileMetadata,
        ATLAS_SIZE, ATLAS_TILES_PER_ROW, MAX_ATLAS_TILES,
    };
    pub use crate::terrain_render::*;
    pub use crate::texture_loader::{
        create_texture_atlas, get_tile_dominant_color, load_manifest_textures, load_texture_rgba,
        TextureLoadError, TextureLoaderConfig,
    };
    pub use crate::textured_render::TexturedChunkRenderer;
    pub use crate::textured_terrain::{
        CellTextureRef, ChunkTextureBuffer, ChunkTextureLayer, TextureColorLookup,
        TextureRenderConfig,
    };
    pub use crate::topdown_physics::*;
    pub use crate::validation::*;
    pub use crate::workbench::{
        CraftingStation, CraftingStationBuilder, StationRegistry, WorkbenchType, WorkbenchZone,
    };
    pub use crate::world_init::*;
    pub use crate::worldgen::*;
    // Save/Load modules
    pub use crate::chunk_serialize::{
        ChunkEncoding, ChunkHeader, ChunkSerializer, Crc32, SerializeError, SerializeStats,
        SerializedChunk,
    };
    pub use crate::incremental_save::{
        ChunkDelta, DeltaOp, IncrementalSaver, SaveConfig, SavePriority, SaveRequest, SaveResponse,
        SaveStats,
    };
    pub use crate::save_compression::{
        CompressionConfig, CompressionError, CompressionLevel, CompressionStats, CompressionType,
        Compressor,
    };
    pub use crate::world_region::{
        ChunkLocation, RegionCoord, RegionError, RegionFile, RegionHeader, RegionManager,
        RegionStats,
    };
    // Main Menu & Options modules
    pub use crate::menu_backdrop::{
        AmbientParticle, BackdropMode, BackdropState, BackdropUniforms, CloudParticle,
        DayNightCycle, ParallaxLayer, StaticBackdrop, TimeOfDay,
    };
    pub use crate::resolution::{
        DisplayMode, OrthoProjection, Resolution, ResolutionChangeRequest, ResolutionManager,
        ResolutionUniforms, ScalingMode, VSyncMode, Viewport,
    };
    pub use crate::screenshot::{
        CaptureConfig, CaptureRequest, CaptureStatus, ScreenshotData, ScreenshotFormat,
        ScreenshotManager, ScreenshotQuality,
    };
    pub use crate::transitions::{
        TransitionConfig, TransitionEasing, TransitionManager, TransitionState, TransitionType,
        TransitionUniforms,
    };
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_size() {
        // Ensure cell fits in expected GPU buffer alignment
        assert_eq!(std::mem::size_of::<Cell>(), 8);
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.material, 0);
        assert_eq!(cell.flags, 0);
    }
}
