//! Event-driven sound system.
//!
//! This module provides:
//! - Sound event types for all game actions
//! - Event queue with priority batching
//! - Cooldown system to prevent sound spam
//! - Volume categories for mixing

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Sound event categories for volume control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoundCategory {
    /// Sound effects (footsteps, combat, etc.)
    SFX,
    /// Ambient/environmental sounds
    Ambient,
    /// Music tracks
    Music,
    /// UI sounds
    UI,
    /// Voice/dialogue
    Voice,
}

impl SoundCategory {
    /// Get default volume (0.0-1.0).
    #[must_use]
    pub fn default_volume(self) -> f32 {
        match self {
            Self::SFX => 0.8,
            Self::Ambient => 0.6,
            Self::Music => 0.5,
            Self::UI => 0.7,
            Self::Voice => 1.0,
        }
    }

    /// Get priority (higher = more important).
    #[must_use]
    pub fn priority(self) -> u8 {
        match self {
            Self::Voice => 5,
            Self::SFX => 4,
            Self::UI => 3,
            Self::Ambient => 2,
            Self::Music => 1,
        }
    }

    /// Get all categories.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [Self::SFX, Self::Ambient, Self::Music, Self::UI, Self::Voice]
    }
}

/// Sound events that can be triggered in the game.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SoundEvent {
    // === Player Movement ===
    /// Footstep on terrain.
    Footstep {
        /// Surface type being walked on.
        surface: SurfaceType,
        /// Whether player is running.
        running: bool,
    },
    /// Jump sound.
    Jump,
    /// Landing after jump/fall.
    Land {
        /// Fall height in meters.
        height: f32,
    },
    /// Entering water.
    WaterEnter,
    /// Exiting water.
    WaterExit,
    /// Swimming.
    Swim,

    // === Combat ===
    /// Weapon swing.
    WeaponSwing {
        /// Type of weapon being swung.
        weapon_type: WeaponSoundType,
    },
    /// Weapon hit target.
    WeaponHit {
        /// Type of weapon that hit.
        weapon_type: WeaponSoundType,
        /// Type of hit (flesh, armor, etc.).
        hit_type: HitType,
    },
    /// Weapon miss.
    WeaponMiss {
        /// Type of weapon that missed.
        weapon_type: WeaponSoundType,
    },
    /// Shield block.
    ShieldBlock,
    /// Player takes damage.
    PlayerHurt,
    /// Player death.
    PlayerDeath,
    /// Critical hit.
    CriticalHit,

    // === Inventory ===
    /// Inventory open.
    InventoryOpen,
    /// Inventory close.
    InventoryClose,
    /// Item pickup.
    ItemPickup {
        /// Type of item picked up.
        item_type: ItemSoundType,
    },
    /// Item drop.
    ItemDrop {
        /// Type of item dropped.
        item_type: ItemSoundType,
    },
    /// Item equip.
    ItemEquip {
        /// Type of item equipped.
        item_type: ItemSoundType,
    },
    /// Item use (potion, food).
    ItemUse {
        /// Type of item used.
        item_type: ItemSoundType,
    },

    // === Environment ===
    /// Door open.
    DoorOpen,
    /// Door close.
    DoorClose,
    /// Chest open.
    ChestOpen,
    /// Chest close.
    ChestClose,
    /// Lever/switch activate.
    LeverActivate,
    /// Block break.
    BlockBreak {
        /// Type of block being broken.
        block_type: BlockSoundType,
    },
    /// Block place.
    BlockPlace {
        /// Type of block being placed.
        block_type: BlockSoundType,
    },

    // === NPC ===
    /// NPC footstep.
    NPCFootstep {
        /// Surface type NPC is walking on.
        surface: SurfaceType,
    },
    /// Dialogue start.
    DialogueStart,
    /// Dialogue end.
    DialogueEnd,
    /// NPC alert (spotted player).
    NPCAlert,

    // === Monsters ===
    /// Monster growl/idle sound.
    MonsterGrowl {
        /// Type of monster.
        monster_type: MonsterSoundType,
    },
    /// Monster attack.
    MonsterAttack {
        /// Type of monster.
        monster_type: MonsterSoundType,
    },
    /// Monster hurt.
    MonsterHurt {
        /// Type of monster.
        monster_type: MonsterSoundType,
    },
    /// Monster death.
    MonsterDeath {
        /// Type of monster.
        monster_type: MonsterSoundType,
    },

    // === UI ===
    /// Button click.
    UIClick,
    /// Menu open.
    UIMenuOpen,
    /// Menu close.
    UIMenuClose,
    /// Error/invalid action
    UIError,
    /// Success/confirm
    UISuccess,
    /// Notification
    UINotification,

    // === Crafting ===
    /// Crafting start
    CraftStart,
    /// Crafting complete
    CraftComplete,
    /// Crafting fail
    CraftFail,

    // === Quest ===
    /// Quest accepted
    QuestAccept,
    /// Quest objective complete
    QuestObjective,
    /// Quest complete
    QuestComplete,
    /// Level up
    LevelUp,
}

impl SoundEvent {
    /// Get the category for this sound.
    #[must_use]
    pub fn category(&self) -> SoundCategory {
        match self {
            Self::Footstep { .. }
            | Self::Jump
            | Self::Land { .. }
            | Self::WaterEnter
            | Self::WaterExit
            | Self::Swim
            | Self::WeaponSwing { .. }
            | Self::WeaponHit { .. }
            | Self::WeaponMiss { .. }
            | Self::ShieldBlock
            | Self::PlayerHurt
            | Self::PlayerDeath
            | Self::CriticalHit
            | Self::ItemPickup { .. }
            | Self::ItemDrop { .. }
            | Self::ItemEquip { .. }
            | Self::ItemUse { .. }
            | Self::DoorOpen
            | Self::DoorClose
            | Self::ChestOpen
            | Self::ChestClose
            | Self::LeverActivate
            | Self::BlockBreak { .. }
            | Self::BlockPlace { .. }
            | Self::NPCFootstep { .. }
            | Self::NPCAlert
            | Self::MonsterGrowl { .. }
            | Self::MonsterAttack { .. }
            | Self::MonsterHurt { .. }
            | Self::MonsterDeath { .. }
            | Self::CraftStart
            | Self::CraftComplete
            | Self::CraftFail => SoundCategory::SFX,

            Self::DialogueStart | Self::DialogueEnd => SoundCategory::Voice,

            Self::UIClick
            | Self::UIMenuOpen
            | Self::UIMenuClose
            | Self::UIError
            | Self::UISuccess
            | Self::UINotification
            | Self::InventoryOpen
            | Self::InventoryClose
            | Self::QuestAccept
            | Self::QuestObjective
            | Self::QuestComplete
            | Self::LevelUp => SoundCategory::UI,
        }
    }

    /// Get default cooldown in seconds (0 = no cooldown).
    #[must_use]
    pub fn default_cooldown(&self) -> f32 {
        match self {
            Self::Footstep { running, .. } => {
                if *running {
                    0.2
                } else {
                    0.35
                }
            },
            Self::Swim => 0.5,
            Self::NPCFootstep { .. } => 0.4,
            Self::MonsterGrowl { .. } => 2.0,
            _ => 0.0,
        }
    }

    /// Get a unique key for cooldown tracking.
    #[must_use]
    pub fn cooldown_key(&self) -> String {
        match self {
            Self::Footstep { surface, running } => {
                format!("footstep_{surface:?}_{running}")
            },
            Self::NPCFootstep { surface } => format!("npc_footstep_{surface:?}"),
            Self::MonsterGrowl { monster_type } => format!("monster_growl_{monster_type:?}"),
            _ => format!("{self:?}"),
        }
    }

    /// Get the asset path for this sound.
    #[must_use]
    pub fn asset_path(&self) -> &'static str {
        match self {
            // Movement
            Self::Footstep { surface, running } => match (surface, running) {
                (SurfaceType::Grass, false) => "sounds/sfx/footstep_grass.ogg",
                (SurfaceType::Grass, true) => "sounds/sfx/footstep_grass_run.ogg",
                (SurfaceType::Stone, false) => "sounds/sfx/footstep_stone.ogg",
                (SurfaceType::Stone, true) => "sounds/sfx/footstep_stone_run.ogg",
                (SurfaceType::Sand, false) => "sounds/sfx/footstep_sand.ogg",
                (SurfaceType::Sand, true) => "sounds/sfx/footstep_sand_run.ogg",
                (SurfaceType::Wood, false) => "sounds/sfx/footstep_wood.ogg",
                (SurfaceType::Wood, true) => "sounds/sfx/footstep_wood_run.ogg",
                (SurfaceType::Water, _) => "sounds/sfx/footstep_water.ogg",
                (SurfaceType::Snow, false) => "sounds/sfx/footstep_snow.ogg",
                (SurfaceType::Snow, true) => "sounds/sfx/footstep_snow_run.ogg",
                (SurfaceType::Metal, _) => "sounds/sfx/footstep_metal.ogg",
            },
            Self::Jump => "sounds/sfx/jump.ogg",
            Self::Land { .. } => "sounds/sfx/land.ogg",
            Self::WaterEnter => "sounds/sfx/water_enter.ogg",
            Self::WaterExit => "sounds/sfx/water_exit.ogg",
            Self::Swim => "sounds/sfx/swim.ogg",

            // Combat
            Self::WeaponSwing { weapon_type } => match weapon_type {
                WeaponSoundType::Sword => "sounds/sfx/sword_swing.ogg",
                WeaponSoundType::Axe => "sounds/sfx/axe_swing.ogg",
                WeaponSoundType::Hammer => "sounds/sfx/hammer_swing.ogg",
                WeaponSoundType::Bow => "sounds/sfx/bow_draw.ogg",
                WeaponSoundType::Staff => "sounds/sfx/staff_swing.ogg",
                WeaponSoundType::Fist => "sounds/sfx/punch_swing.ogg",
            },
            Self::WeaponHit {
                weapon_type,
                hit_type,
            } => match (weapon_type, hit_type) {
                (_, HitType::Flesh) => "sounds/sfx/hit_flesh.ogg",
                (_, HitType::Armor) => "sounds/sfx/hit_armor.ogg",
                (WeaponSoundType::Bow, _) => "sounds/sfx/arrow_hit.ogg",
                _ => "sounds/sfx/hit_generic.ogg",
            },
            Self::WeaponMiss { .. } => "sounds/sfx/whoosh.ogg",
            Self::ShieldBlock => "sounds/sfx/shield_block.ogg",
            Self::PlayerHurt => "sounds/sfx/player_hurt.ogg",
            Self::PlayerDeath => "sounds/sfx/player_death.ogg",
            Self::CriticalHit => "sounds/sfx/critical_hit.ogg",

            // Inventory
            Self::InventoryOpen => "sounds/ui/inventory_open.ogg",
            Self::InventoryClose => "sounds/ui/inventory_close.ogg",
            Self::ItemPickup { item_type } => match item_type {
                ItemSoundType::Coin => "sounds/sfx/coin_pickup.ogg",
                ItemSoundType::Potion => "sounds/sfx/potion_pickup.ogg",
                ItemSoundType::Weapon => "sounds/sfx/weapon_pickup.ogg",
                ItemSoundType::Armor => "sounds/sfx/armor_pickup.ogg",
                ItemSoundType::Food => "sounds/sfx/food_pickup.ogg",
                ItemSoundType::Material => "sounds/sfx/material_pickup.ogg",
                ItemSoundType::Quest => "sounds/sfx/quest_item_pickup.ogg",
            },
            Self::ItemDrop { .. } => "sounds/sfx/item_drop.ogg",
            Self::ItemEquip { item_type } => match item_type {
                ItemSoundType::Weapon => "sounds/sfx/weapon_equip.ogg",
                ItemSoundType::Armor => "sounds/sfx/armor_equip.ogg",
                _ => "sounds/sfx/item_equip.ogg",
            },
            Self::ItemUse { item_type } => match item_type {
                ItemSoundType::Potion => "sounds/sfx/potion_drink.ogg",
                ItemSoundType::Food => "sounds/sfx/eat.ogg",
                _ => "sounds/sfx/item_use.ogg",
            },

            // Environment
            Self::DoorOpen => "sounds/sfx/door_open.ogg",
            Self::DoorClose => "sounds/sfx/door_close.ogg",
            Self::ChestOpen => "sounds/sfx/chest_open.ogg",
            Self::ChestClose => "sounds/sfx/chest_close.ogg",
            Self::LeverActivate => "sounds/sfx/lever.ogg",
            Self::BlockBreak { block_type } => match block_type {
                BlockSoundType::Stone => "sounds/sfx/break_stone.ogg",
                BlockSoundType::Wood => "sounds/sfx/break_wood.ogg",
                BlockSoundType::Dirt => "sounds/sfx/break_dirt.ogg",
                BlockSoundType::Glass => "sounds/sfx/break_glass.ogg",
                BlockSoundType::Metal => "sounds/sfx/break_metal.ogg",
            },
            Self::BlockPlace { block_type } => match block_type {
                BlockSoundType::Stone => "sounds/sfx/place_stone.ogg",
                BlockSoundType::Wood => "sounds/sfx/place_wood.ogg",
                BlockSoundType::Dirt => "sounds/sfx/place_dirt.ogg",
                BlockSoundType::Glass => "sounds/sfx/place_glass.ogg",
                BlockSoundType::Metal => "sounds/sfx/place_metal.ogg",
            },

            // NPC
            Self::NPCFootstep { surface } => match surface {
                SurfaceType::Grass => "sounds/sfx/footstep_grass.ogg",
                SurfaceType::Stone => "sounds/sfx/footstep_stone.ogg",
                SurfaceType::Sand => "sounds/sfx/footstep_sand.ogg",
                SurfaceType::Wood => "sounds/sfx/footstep_wood.ogg",
                SurfaceType::Water => "sounds/sfx/footstep_water.ogg",
                SurfaceType::Snow => "sounds/sfx/footstep_snow.ogg",
                SurfaceType::Metal => "sounds/sfx/footstep_metal.ogg",
            },
            Self::DialogueStart => "sounds/ui/dialogue_start.ogg",
            Self::DialogueEnd => "sounds/ui/dialogue_end.ogg",
            Self::NPCAlert => "sounds/sfx/npc_alert.ogg",

            // Monsters
            Self::MonsterGrowl { monster_type } => match monster_type {
                MonsterSoundType::Slime => "sounds/monsters/slime_idle.ogg",
                MonsterSoundType::Skeleton => "sounds/monsters/skeleton_idle.ogg",
                MonsterSoundType::Goblin => "sounds/monsters/goblin_idle.ogg",
                MonsterSoundType::Orc => "sounds/monsters/orc_idle.ogg",
                MonsterSoundType::Wolf => "sounds/monsters/wolf_idle.ogg",
                MonsterSoundType::Bear => "sounds/monsters/bear_idle.ogg",
                MonsterSoundType::Spider => "sounds/monsters/spider_idle.ogg",
                MonsterSoundType::Bat => "sounds/monsters/bat_idle.ogg",
            },
            Self::MonsterAttack { monster_type } => match monster_type {
                MonsterSoundType::Slime => "sounds/monsters/slime_attack.ogg",
                MonsterSoundType::Skeleton => "sounds/monsters/skeleton_attack.ogg",
                MonsterSoundType::Goblin => "sounds/monsters/goblin_attack.ogg",
                MonsterSoundType::Orc => "sounds/monsters/orc_attack.ogg",
                MonsterSoundType::Wolf => "sounds/monsters/wolf_attack.ogg",
                MonsterSoundType::Bear => "sounds/monsters/bear_attack.ogg",
                MonsterSoundType::Spider => "sounds/monsters/spider_attack.ogg",
                MonsterSoundType::Bat => "sounds/monsters/bat_attack.ogg",
            },
            Self::MonsterHurt { monster_type } => match monster_type {
                MonsterSoundType::Slime => "sounds/monsters/slime_hurt.ogg",
                MonsterSoundType::Skeleton => "sounds/monsters/skeleton_hurt.ogg",
                MonsterSoundType::Goblin => "sounds/monsters/goblin_hurt.ogg",
                MonsterSoundType::Orc => "sounds/monsters/orc_hurt.ogg",
                MonsterSoundType::Wolf => "sounds/monsters/wolf_hurt.ogg",
                MonsterSoundType::Bear => "sounds/monsters/bear_hurt.ogg",
                MonsterSoundType::Spider => "sounds/monsters/spider_hurt.ogg",
                MonsterSoundType::Bat => "sounds/monsters/bat_hurt.ogg",
            },
            Self::MonsterDeath { monster_type } => match monster_type {
                MonsterSoundType::Slime => "sounds/monsters/slime_death.ogg",
                MonsterSoundType::Skeleton => "sounds/monsters/skeleton_death.ogg",
                MonsterSoundType::Goblin => "sounds/monsters/goblin_death.ogg",
                MonsterSoundType::Orc => "sounds/monsters/orc_death.ogg",
                MonsterSoundType::Wolf => "sounds/monsters/wolf_death.ogg",
                MonsterSoundType::Bear => "sounds/monsters/bear_death.ogg",
                MonsterSoundType::Spider => "sounds/monsters/spider_death.ogg",
                MonsterSoundType::Bat => "sounds/monsters/bat_death.ogg",
            },

            // UI
            Self::UIClick => "sounds/ui/click.ogg",
            Self::UIMenuOpen => "sounds/ui/menu_open.ogg",
            Self::UIMenuClose => "sounds/ui/menu_close.ogg",
            Self::UIError => "sounds/ui/error.ogg",
            Self::UISuccess => "sounds/ui/success.ogg",
            Self::UINotification => "sounds/ui/notification.ogg",

            // Crafting
            Self::CraftStart => "sounds/sfx/craft_start.ogg",
            Self::CraftComplete => "sounds/sfx/craft_complete.ogg",
            Self::CraftFail => "sounds/sfx/craft_fail.ogg",

            // Quest
            Self::QuestAccept => "sounds/ui/quest_accept.ogg",
            Self::QuestObjective => "sounds/ui/quest_objective.ogg",
            Self::QuestComplete => "sounds/ui/quest_complete.ogg",
            Self::LevelUp => "sounds/ui/level_up.ogg",
        }
    }
}

/// Surface types for footstep sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurfaceType {
    /// Grass/dirt
    Grass,
    /// Stone/rock
    Stone,
    /// Sand
    Sand,
    /// Wood/planks
    Wood,
    /// Shallow water
    Water,
    /// Snow/ice
    Snow,
    /// Metal
    Metal,
}

/// Weapon types for combat sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponSoundType {
    /// Sword/blade
    Sword,
    /// Axe
    Axe,
    /// Hammer/mace
    Hammer,
    /// Bow/crossbow
    Bow,
    /// Staff/wand
    Staff,
    /// Unarmed
    Fist,
}

/// Hit types for impact sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HitType {
    /// Hit flesh/unarmored
    Flesh,
    /// Hit armor
    Armor,
    /// Hit shield
    Shield,
}

/// Item types for inventory sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemSoundType {
    /// Coins/currency
    Coin,
    /// Potion/vial
    Potion,
    /// Weapon
    Weapon,
    /// Armor/clothing
    Armor,
    /// Food items
    Food,
    /// Raw materials
    Material,
    /// Quest items
    Quest,
}

/// Block types for terrain sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockSoundType {
    /// Stone blocks
    Stone,
    /// Wood blocks
    Wood,
    /// Dirt/sand
    Dirt,
    /// Glass
    Glass,
    /// Metal
    Metal,
}

/// Monster types for creature sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterSoundType {
    /// Slime
    Slime,
    /// Skeleton
    Skeleton,
    /// Goblin
    Goblin,
    /// Orc
    Orc,
    /// Wolf
    Wolf,
    /// Bear
    Bear,
    /// Spider
    Spider,
    /// Bat
    Bat,
}

/// A queued sound event with metadata.
#[derive(Debug, Clone)]
pub struct QueuedSound {
    /// The sound event.
    pub event: SoundEvent,
    /// Position in world space (for 3D audio).
    pub position: Option<[f32; 3]>,
    /// Volume multiplier (0.0-1.0).
    pub volume: f32,
    /// Pitch multiplier (0.5-2.0).
    pub pitch: f32,
    /// Time when queued.
    pub queued_at: f64,
}

impl QueuedSound {
    /// Create a new queued sound.
    #[must_use]
    pub fn new(event: SoundEvent) -> Self {
        Self {
            event,
            position: None,
            volume: 1.0,
            pitch: 1.0,
            queued_at: 0.0,
        }
    }

    /// Set position for 3D audio.
    #[must_use]
    pub fn at_position(mut self, pos: [f32; 3]) -> Self {
        self.position = Some(pos);
        self
    }

    /// Set volume multiplier.
    #[must_use]
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set pitch multiplier.
    #[must_use]
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.5, 2.0);
        self
    }

    /// Set queued time.
    #[must_use]
    pub fn at_time(mut self, time: f64) -> Self {
        self.queued_at = time;
        self
    }
}

/// Volume settings for each category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSettings {
    /// Master volume.
    pub master: f32,
    /// Per-category volumes.
    pub categories: HashMap<SoundCategory, f32>,
    /// Muted categories.
    pub muted: HashMap<SoundCategory, bool>,
}

impl VolumeSettings {
    /// Create default volume settings.
    #[must_use]
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        let mut muted = HashMap::new();

        for cat in SoundCategory::all() {
            categories.insert(cat, cat.default_volume());
            muted.insert(cat, false);
        }

        Self {
            master: 1.0,
            categories,
            muted,
        }
    }

    /// Get effective volume for a category.
    #[must_use]
    pub fn effective_volume(&self, category: SoundCategory) -> f32 {
        if self.muted.get(&category).copied().unwrap_or(false) {
            return 0.0;
        }
        self.master * self.categories.get(&category).copied().unwrap_or(1.0)
    }

    /// Set volume for a category.
    pub fn set_volume(&mut self, category: SoundCategory, volume: f32) {
        self.categories.insert(category, volume.clamp(0.0, 1.0));
    }

    /// Toggle mute for a category.
    pub fn toggle_mute(&mut self, category: SoundCategory) {
        let current = self.muted.get(&category).copied().unwrap_or(false);
        self.muted.insert(category, !current);
    }

    /// Set mute state for a category.
    pub fn set_muted(&mut self, category: SoundCategory, muted: bool) {
        self.muted.insert(category, muted);
    }

    /// Check if category is muted.
    #[must_use]
    pub fn is_muted(&self, category: SoundCategory) -> bool {
        self.muted.get(&category).copied().unwrap_or(false)
    }
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Cooldown tracker for sounds.
#[derive(Debug, Default)]
pub struct SoundCooldowns {
    /// Last play time for each cooldown key.
    cooldowns: HashMap<String, f64>,
}

impl SoundCooldowns {
    /// Create new cooldown tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a sound can play (not on cooldown).
    #[must_use]
    pub fn can_play(&self, event: &SoundEvent, current_time: f64) -> bool {
        let cooldown = event.default_cooldown();
        if cooldown <= 0.0 {
            return true;
        }

        let key = event.cooldown_key();
        match self.cooldowns.get(&key) {
            Some(last_time) => current_time - last_time >= cooldown as f64,
            None => true,
        }
    }

    /// Mark a sound as played.
    pub fn mark_played(&mut self, event: &SoundEvent, current_time: f64) {
        let cooldown = event.default_cooldown();
        if cooldown > 0.0 {
            let key = event.cooldown_key();
            self.cooldowns.insert(key, current_time);
        }
    }

    /// Clear expired cooldowns.
    pub fn cleanup(&mut self, current_time: f64, max_age: f64) {
        self.cooldowns
            .retain(|_, last_time| current_time - *last_time < max_age);
    }
}

/// Sound event queue with priority ordering.
#[derive(Debug)]
pub struct SoundEventQueue {
    /// Pending sound events.
    queue: VecDeque<QueuedSound>,
    /// Volume settings.
    volumes: VolumeSettings,
    /// Cooldown tracker.
    cooldowns: SoundCooldowns,
    /// Maximum queue size.
    max_queue_size: usize,
    /// Current game time.
    current_time: f64,
}

impl SoundEventQueue {
    /// Create a new sound event queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(64),
            volumes: VolumeSettings::new(),
            cooldowns: SoundCooldowns::new(),
            max_queue_size: 64,
            current_time: 0.0,
        }
    }

    /// Create with custom max queue size.
    #[must_use]
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size),
            volumes: VolumeSettings::new(),
            cooldowns: SoundCooldowns::new(),
            max_queue_size: max_size,
            current_time: 0.0,
        }
    }

    /// Update current time.
    pub fn set_time(&mut self, time: f64) {
        self.current_time = time;
    }

    /// Get volume settings.
    #[must_use]
    pub fn volumes(&self) -> &VolumeSettings {
        &self.volumes
    }

    /// Get mutable volume settings.
    pub fn volumes_mut(&mut self) -> &mut VolumeSettings {
        &mut self.volumes
    }

    /// Queue a sound event.
    pub fn push(&mut self, event: SoundEvent) {
        self.push_sound(QueuedSound::new(event).at_time(self.current_time));
    }

    /// Queue a sound at a position.
    pub fn push_at(&mut self, event: SoundEvent, position: [f32; 3]) {
        self.push_sound(
            QueuedSound::new(event)
                .at_position(position)
                .at_time(self.current_time),
        );
    }

    /// Queue a sound with full options.
    pub fn push_sound(&mut self, sound: QueuedSound) {
        // Check cooldown
        if !self.cooldowns.can_play(&sound.event, self.current_time) {
            return;
        }

        // Check if category is muted
        if self.volumes.is_muted(sound.event.category()) {
            return;
        }

        // Mark cooldown
        self.cooldowns.mark_played(&sound.event, self.current_time);

        // Enforce max size by removing lowest priority
        while self.queue.len() >= self.max_queue_size {
            self.remove_lowest_priority();
        }

        self.queue.push_back(sound);
    }

    /// Remove lowest priority sound from queue.
    fn remove_lowest_priority(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        let mut lowest_idx = 0;
        let mut lowest_priority = u8::MAX;

        for (i, sound) in self.queue.iter().enumerate() {
            let priority = sound.event.category().priority();
            if priority < lowest_priority {
                lowest_priority = priority;
                lowest_idx = i;
            }
        }

        self.queue.remove(lowest_idx);
    }

    /// Pop next sound to play (sorted by priority).
    pub fn pop(&mut self) -> Option<QueuedSound> {
        if self.queue.is_empty() {
            return None;
        }

        // Find highest priority
        let mut best_idx = 0;
        let mut best_priority = 0;

        for (i, sound) in self.queue.iter().enumerate() {
            let priority = sound.event.category().priority();
            if priority > best_priority {
                best_priority = priority;
                best_idx = i;
            }
        }

        self.queue.remove(best_idx)
    }

    /// Drain all sounds in priority order.
    pub fn drain(&mut self) -> Vec<QueuedSound> {
        let mut result = Vec::with_capacity(self.queue.len());
        while let Some(sound) = self.pop() {
            result.push(sound);
        }
        result
    }

    /// Get queue length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clear the queue.
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Cleanup old cooldowns.
    pub fn cleanup_cooldowns(&mut self) {
        self.cooldowns.cleanup(self.current_time, 10.0);
    }
}

impl Default for SoundEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_category_defaults() {
        assert!(SoundCategory::SFX.default_volume() > 0.0);
        assert!(SoundCategory::Music.default_volume() > 0.0);
    }

    #[test]
    fn test_sound_category_priority() {
        assert!(SoundCategory::Voice.priority() > SoundCategory::Music.priority());
        assert!(SoundCategory::SFX.priority() > SoundCategory::Ambient.priority());
    }

    #[test]
    fn test_sound_event_category() {
        assert_eq!(
            SoundEvent::Footstep {
                surface: SurfaceType::Grass,
                running: false
            }
            .category(),
            SoundCategory::SFX
        );
        assert_eq!(SoundEvent::UIClick.category(), SoundCategory::UI);
        assert_eq!(SoundEvent::DialogueStart.category(), SoundCategory::Voice);
    }

    #[test]
    fn test_sound_event_asset_path() {
        let event = SoundEvent::Footstep {
            surface: SurfaceType::Grass,
            running: false,
        };
        assert!(event.asset_path().contains("footstep"));
    }

    #[test]
    fn test_queued_sound_builder() {
        let sound = QueuedSound::new(SoundEvent::Jump)
            .at_position([1.0, 2.0, 3.0])
            .with_volume(0.5)
            .with_pitch(1.2);

        assert_eq!(sound.position, Some([1.0, 2.0, 3.0]));
        assert!((sound.volume - 0.5).abs() < 0.001);
        assert!((sound.pitch - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_volume_settings() {
        let mut settings = VolumeSettings::new();
        settings.set_volume(SoundCategory::Music, 0.3);
        settings.master = 0.5;

        let effective = settings.effective_volume(SoundCategory::Music);
        assert!((effective - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_volume_mute() {
        let mut settings = VolumeSettings::new();
        assert!(!settings.is_muted(SoundCategory::SFX));

        settings.toggle_mute(SoundCategory::SFX);
        assert!(settings.is_muted(SoundCategory::SFX));
        assert!((settings.effective_volume(SoundCategory::SFX) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_sound_cooldowns() {
        let mut cooldowns = SoundCooldowns::new();
        let event = SoundEvent::Footstep {
            surface: SurfaceType::Grass,
            running: false,
        };

        assert!(cooldowns.can_play(&event, 0.0));
        cooldowns.mark_played(&event, 0.0);
        assert!(!cooldowns.can_play(&event, 0.1));
        assert!(cooldowns.can_play(&event, 1.0));
    }

    #[test]
    fn test_sound_queue_push_pop() {
        let mut queue = SoundEventQueue::new();
        queue.push(SoundEvent::Jump);
        queue.push(SoundEvent::UIClick);

        assert_eq!(queue.len(), 2);

        let sound = queue.pop().expect("should have sound");
        // UI has higher priority than SFX... wait, let me check
        // Voice > SFX > UI > Ambient > Music
        // So Jump (SFX) should come first
        assert!(matches!(sound.event, SoundEvent::Jump));
    }

    #[test]
    fn test_sound_queue_priority() {
        let mut queue = SoundEventQueue::new();
        queue.push(SoundEvent::UIClick); // UI priority 3
        queue.push(SoundEvent::DialogueStart); // Voice priority 5

        let first = queue.pop().expect("should have sound");
        assert!(matches!(first.event, SoundEvent::DialogueStart));
    }

    #[test]
    fn test_sound_queue_cooldown() {
        let mut queue = SoundEventQueue::new();
        queue.set_time(0.0);

        let event = SoundEvent::Footstep {
            surface: SurfaceType::Grass,
            running: false,
        };

        queue.push(event.clone());
        queue.push(event.clone()); // Should be blocked by cooldown

        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_sound_queue_muted() {
        let mut queue = SoundEventQueue::new();
        queue.volumes_mut().set_muted(SoundCategory::UI, true);
        queue.push(SoundEvent::UIClick);

        assert!(queue.is_empty());
    }

    #[test]
    fn test_sound_queue_drain() {
        let mut queue = SoundEventQueue::new();
        queue.push(SoundEvent::Jump);
        queue.push(SoundEvent::UIClick);

        let sounds = queue.drain();
        assert_eq!(sounds.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_sound_queue_max_size() {
        let mut queue = SoundEventQueue::with_max_size(2);
        queue.push(SoundEvent::Jump);
        queue.push(SoundEvent::UIClick);
        queue.push(SoundEvent::DialogueStart); // Should evict lowest priority

        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_surface_types() {
        let surfaces = [
            SurfaceType::Grass,
            SurfaceType::Stone,
            SurfaceType::Sand,
            SurfaceType::Wood,
            SurfaceType::Water,
            SurfaceType::Snow,
            SurfaceType::Metal,
        ];

        for surface in surfaces {
            let event = SoundEvent::Footstep {
                surface,
                running: false,
            };
            assert!(!event.asset_path().is_empty());
        }
    }

    #[test]
    fn test_weapon_sound_types() {
        let weapons = [
            WeaponSoundType::Sword,
            WeaponSoundType::Axe,
            WeaponSoundType::Hammer,
            WeaponSoundType::Bow,
            WeaponSoundType::Staff,
            WeaponSoundType::Fist,
        ];

        for weapon in weapons {
            let event = SoundEvent::WeaponSwing {
                weapon_type: weapon,
            };
            assert!(!event.asset_path().is_empty());
        }
    }

    #[test]
    fn test_monster_sound_types() {
        let monsters = [
            MonsterSoundType::Slime,
            MonsterSoundType::Skeleton,
            MonsterSoundType::Goblin,
            MonsterSoundType::Orc,
            MonsterSoundType::Wolf,
            MonsterSoundType::Bear,
            MonsterSoundType::Spider,
            MonsterSoundType::Bat,
        ];

        for monster in monsters {
            let growl = SoundEvent::MonsterGrowl {
                monster_type: monster,
            };
            let attack = SoundEvent::MonsterAttack {
                monster_type: monster,
            };
            let hurt = SoundEvent::MonsterHurt {
                monster_type: monster,
            };
            let death = SoundEvent::MonsterDeath {
                monster_type: monster,
            };

            assert!(!growl.asset_path().is_empty());
            assert!(!attack.asset_path().is_empty());
            assert!(!hurt.asset_path().is_empty());
            assert!(!death.asset_path().is_empty());
        }
    }
}
