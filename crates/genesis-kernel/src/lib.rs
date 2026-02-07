//! # Genesis Kernel
//!
//! Core rendering and gameplay kernel.
//!
//! This crate provides:
//! - Camera system for world navigation
//! - Player sprite rendering
//! - NPC rendering and collision
//! - Combat systems (particles, collision, damage)
//! - Crafting grid and animations
//! - Audio systems
//! - UI transitions and effects
//! - Screenshot capture

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

// Audio systems
pub mod audio;
pub mod audio_backend;
pub mod audio_legacy;
pub mod audio_resource;
pub mod audio_spatial;

// Camera
pub mod camera;

// Combat systems
pub mod combat_collision;
pub mod combat_particles;
pub mod damage_render;
pub mod npc_collision;
pub mod projectile;

// Crafting systems
pub mod crafting_anim;
pub mod crafting_grid;
pub mod workbench;

// Item stacks
pub mod item_stack;

// NPC rendering
pub mod npc_render;

// Particles
pub mod particles;

// Player sprite
pub mod player_sprite;

// Terrain tile rendering
pub mod terrain_tiles;

// Spatial indexing
pub mod quadtree;

// Resolution management
pub mod resolution;

// Screenshot capture
pub mod screenshot;

// Transitions
pub mod transitions;

// Menu backdrop
pub mod menu_backdrop;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::audio::*;
    pub use crate::camera::*;
    // Combat modules use explicit imports to avoid conflicts
    pub use crate::combat_collision::{
        CollisionResult, CombatBoxManager, CombatCollider, FrameRange, Hitbox,
        HitboxSequenceBuilder, HitboxShape, Hurtbox,
    };
    pub use crate::combat_particles::{
        BloodSplatterEffect, CombatEffectType, CombatParticle, CombatParticleInstance,
        CombatParticleManager, HitSparkEffect, ImpactDustEffect,
    };
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
    pub use crate::item_stack::{
        CompactStack, ItemMetadata, ItemStack, ItemStackBuilder, StackResult,
    };
    pub use crate::npc_collision::*;
    pub use crate::npc_render::*;
    pub use crate::particles::*;
    pub use crate::player_sprite::{
        AnimKey, PlayerAnimAction, PlayerAnimState, PlayerAnimationSet, PlayerDirection,
        PlayerSpriteConfig, PlayerSpriteInstance, PlayerSpriteRenderer, PlayerSpriteState,
        SpriteAnimation, SpriteFrame,
    };
    pub use crate::projectile::{
        Projectile, ProjectileCollision, ProjectileInstance, ProjectileManager, ProjectileState,
        ProjectileType, TrajectoryPredictor,
    };
    pub use crate::quadtree::*;
    pub use crate::workbench::{
        CraftingStation, CraftingStationBuilder, StationRegistry, WorkbenchType, WorkbenchZone,
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
