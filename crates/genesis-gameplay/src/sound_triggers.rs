//! Sound triggers for game events.
//!
//! This module provides:
//! - Automatic sound triggering from game events
//! - Player action sounds
//! - Inventory sounds
//! - NPC and monster sounds
//! - Environment interaction sounds

use crate::ai::{AnimalType, MonsterType};
use crate::biome::BiomeType;
use crate::sound_events::{
    BlockSoundType, HitType, ItemSoundType, MonsterSoundType, QueuedSound, SoundEvent,
    SoundEventQueue, SurfaceType, WeaponSoundType,
};
use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Player action that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerAction {
    /// Started walking.
    Walk,
    /// Started running.
    Run,
    /// Stopped moving.
    Stop,
    /// Jumped.
    Jump,
    /// Landed after jump/fall.
    Land,
    /// Entered water.
    EnterWater,
    /// Exited water.
    ExitWater,
    /// Swimming.
    Swim,
    /// Attacked with weapon.
    Attack,
    /// Got hit.
    TakeDamage,
    /// Died.
    Die,
    /// Blocked with shield.
    Block,
}

/// Inventory action that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventorySoundAction {
    /// Opened inventory.
    Open,
    /// Closed inventory.
    Close,
    /// Picked up item.
    Pickup,
    /// Dropped item.
    Drop,
    /// Equipped item.
    Equip,
    /// Used/consumed item.
    Use,
}

/// NPC action that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NPCSoundAction {
    /// NPC walking.
    Walk,
    /// Started dialogue.
    DialogueStart,
    /// Ended dialogue.
    DialogueEnd,
    /// Spotted player (alert).
    Alert,
}

/// Environment interaction that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvironmentAction {
    /// Opened a door.
    DoorOpen,
    /// Closed a door.
    DoorClose,
    /// Opened a chest.
    ChestOpen,
    /// Closed a chest.
    ChestClose,
    /// Activated lever/switch.
    LeverActivate,
    /// Broke a block.
    BlockBreak,
    /// Placed a block.
    BlockPlace,
}

/// Monster action that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterAction {
    /// Idle growl/sound.
    Idle,
    /// Attacking.
    Attack,
    /// Took damage.
    Hurt,
    /// Died.
    Die,
}

/// Crafting action that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftingAction {
    /// Started crafting.
    Start,
    /// Crafting complete.
    Complete,
    /// Crafting failed.
    Fail,
}

/// Quest action that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestAction {
    /// Accepted quest.
    Accept,
    /// Completed objective.
    Objective,
    /// Completed quest.
    Complete,
    /// Leveled up.
    LevelUp,
}

/// UI action that triggers a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UIAction {
    /// Button click.
    Click,
    /// Menu opened.
    MenuOpen,
    /// Menu closed.
    MenuClose,
    /// Error/invalid action.
    Error,
    /// Success confirmation.
    Success,
    /// Notification appeared.
    Notification,
}

/// Context for player sounds.
#[derive(Debug, Clone, Default)]
pub struct PlayerSoundContext {
    /// Current surface type player is on.
    pub surface: Option<SurfaceType>,
    /// Player position.
    pub position: Option<[f32; 3]>,
    /// Whether player is running.
    pub is_running: bool,
    /// Current equipped weapon type.
    pub weapon_type: Option<WeaponSoundType>,
    /// Fall height (for landing sounds).
    pub fall_height: f32,
}

impl PlayerSoundContext {
    /// Create new context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set surface type.
    #[must_use]
    pub fn with_surface(mut self, surface: SurfaceType) -> Self {
        self.surface = Some(surface);
        self
    }

    /// Set position.
    #[must_use]
    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = Some(pos);
        self
    }

    /// Set running state.
    #[must_use]
    pub fn with_running(mut self, running: bool) -> Self {
        self.is_running = running;
        self
    }

    /// Set weapon type.
    #[must_use]
    pub fn with_weapon(mut self, weapon: WeaponSoundType) -> Self {
        self.weapon_type = Some(weapon);
        self
    }

    /// Set fall height.
    #[must_use]
    pub fn with_fall_height(mut self, height: f32) -> Self {
        self.fall_height = height;
        self
    }
}

/// Context for inventory sounds.
#[derive(Debug, Clone, Default)]
pub struct InventorySoundContext {
    /// Item type for pickup/drop/equip sounds.
    pub item_type: Option<ItemSoundType>,
    /// Position (for 3D pickup sounds).
    pub position: Option<[f32; 3]>,
}

impl InventorySoundContext {
    /// Create new context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set item type.
    #[must_use]
    pub fn with_item_type(mut self, item_type: ItemSoundType) -> Self {
        self.item_type = Some(item_type);
        self
    }

    /// Set position.
    #[must_use]
    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = Some(pos);
        self
    }
}

/// Context for NPC sounds.
#[derive(Debug, Clone, Default)]
pub struct NPCSoundContext {
    /// Surface NPC is on.
    pub surface: Option<SurfaceType>,
    /// NPC position.
    pub position: Option<[f32; 3]>,
}

impl NPCSoundContext {
    /// Create new context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set surface.
    #[must_use]
    pub fn with_surface(mut self, surface: SurfaceType) -> Self {
        self.surface = Some(surface);
        self
    }

    /// Set position.
    #[must_use]
    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = Some(pos);
        self
    }
}

/// Context for environment sounds.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentSoundContext {
    /// Block type for break/place sounds.
    pub block_type: Option<BlockSoundType>,
    /// Position of interaction.
    pub position: Option<[f32; 3]>,
}

impl EnvironmentSoundContext {
    /// Create new context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set block type.
    #[must_use]
    pub fn with_block_type(mut self, block_type: BlockSoundType) -> Self {
        self.block_type = Some(block_type);
        self
    }

    /// Set position.
    #[must_use]
    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = Some(pos);
        self
    }
}

/// Context for monster sounds.
#[derive(Debug, Clone, Default)]
pub struct MonsterSoundContext {
    /// Monster type.
    pub monster_type: Option<MonsterSoundType>,
    /// Monster position.
    pub position: Option<[f32; 3]>,
}

impl MonsterSoundContext {
    /// Create new context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set monster type.
    #[must_use]
    pub fn with_monster_type(mut self, monster_type: MonsterSoundType) -> Self {
        self.monster_type = Some(monster_type);
        self
    }

    /// Set position.
    #[must_use]
    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = Some(pos);
        self
    }
}

/// Convert biome to default surface type.
#[must_use]
pub fn biome_to_surface(biome: BiomeType) -> SurfaceType {
    match biome {
        BiomeType::Desert => SurfaceType::Sand,
        BiomeType::Lake | BiomeType::Swamp => SurfaceType::Water,
        BiomeType::Mountain => SurfaceType::Stone,
        BiomeType::Forest | BiomeType::Plains => SurfaceType::Grass,
    }
}

/// Convert MonsterType from ai module to MonsterSoundType.
#[must_use]
pub fn monster_to_sound_type(monster: MonsterType) -> MonsterSoundType {
    match monster {
        MonsterType::Slime => MonsterSoundType::Slime,
        MonsterType::Skeleton => MonsterSoundType::Skeleton,
        MonsterType::Goblin => MonsterSoundType::Goblin,
        MonsterType::Orc => MonsterSoundType::Orc,
        MonsterType::Scorpion | MonsterType::Spider => MonsterSoundType::Spider,
        MonsterType::Bat => MonsterSoundType::Bat,
    }
}

/// Convert AnimalType to MonsterSoundType (for hostile animals).
#[must_use]
pub fn animal_to_sound_type(animal: AnimalType) -> Option<MonsterSoundType> {
    match animal {
        AnimalType::Wolf => Some(MonsterSoundType::Wolf),
        AnimalType::Bear => Some(MonsterSoundType::Bear),
        _ => None, // Non-hostile animals don't have monster sounds
    }
}

/// Sound trigger system that queues sounds from game events.
#[derive(Debug)]
pub struct SoundTriggerSystem {
    /// Reference to sound event queue.
    /// Note: In actual use, this would be passed to trigger methods.
    /// Current player context.
    player_context: PlayerSoundContext,
    /// Entity positions for 3D audio.
    entity_positions: HashMap<EntityId, [f32; 3]>,
    /// Whether sound triggers are enabled.
    enabled: bool,
}

impl SoundTriggerSystem {
    /// Create a new sound trigger system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            player_context: PlayerSoundContext::new(),
            entity_positions: HashMap::new(),
            enabled: true,
        }
    }

    /// Enable/disable triggers.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Update player context.
    pub fn update_player_context(&mut self, context: PlayerSoundContext) {
        self.player_context = context;
    }

    /// Set player surface.
    pub fn set_player_surface(&mut self, surface: SurfaceType) {
        self.player_context.surface = Some(surface);
    }

    /// Set player position.
    pub fn set_player_position(&mut self, pos: [f32; 3]) {
        self.player_context.position = Some(pos);
    }

    /// Set player weapon.
    pub fn set_player_weapon(&mut self, weapon: WeaponSoundType) {
        self.player_context.weapon_type = Some(weapon);
    }

    /// Update entity position.
    pub fn set_entity_position(&mut self, entity: EntityId, pos: [f32; 3]) {
        self.entity_positions.insert(entity, pos);
    }

    /// Remove entity position.
    pub fn remove_entity(&mut self, entity: EntityId) {
        self.entity_positions.remove(&entity);
    }

    /// Get entity position.
    #[must_use]
    pub fn entity_position(&self, entity: EntityId) -> Option<[f32; 3]> {
        self.entity_positions.get(&entity).copied()
    }

    /// Trigger player action sound.
    pub fn trigger_player(&self, queue: &mut SoundEventQueue, action: PlayerAction) {
        if !self.enabled {
            return;
        }

        let event = match action {
            PlayerAction::Walk | PlayerAction::Run => {
                let surface = self.player_context.surface.unwrap_or(SurfaceType::Grass);
                let running = action == PlayerAction::Run || self.player_context.is_running;
                SoundEvent::Footstep { surface, running }
            },
            PlayerAction::Stop => return, // No sound for stopping
            PlayerAction::Jump => SoundEvent::Jump,
            PlayerAction::Land => SoundEvent::Land {
                height: self.player_context.fall_height,
            },
            PlayerAction::EnterWater => SoundEvent::WaterEnter,
            PlayerAction::ExitWater => SoundEvent::WaterExit,
            PlayerAction::Swim => SoundEvent::Swim,
            PlayerAction::Attack => {
                let weapon_type = self
                    .player_context
                    .weapon_type
                    .unwrap_or(WeaponSoundType::Fist);
                SoundEvent::WeaponSwing { weapon_type }
            },
            PlayerAction::TakeDamage => SoundEvent::PlayerHurt,
            PlayerAction::Die => SoundEvent::PlayerDeath,
            PlayerAction::Block => SoundEvent::ShieldBlock,
        };

        let mut sound = QueuedSound::new(event);
        if let Some(pos) = self.player_context.position {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger player hit sound (attack that landed).
    pub fn trigger_player_hit(
        &self,
        queue: &mut SoundEventQueue,
        hit_type: HitType,
        position: Option<[f32; 3]>,
    ) {
        if !self.enabled {
            return;
        }

        let weapon_type = self
            .player_context
            .weapon_type
            .unwrap_or(WeaponSoundType::Fist);
        let event = SoundEvent::WeaponHit {
            weapon_type,
            hit_type,
        };

        let mut sound = QueuedSound::new(event);
        if let Some(pos) = position.or(self.player_context.position) {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger player miss sound (attack that missed).
    pub fn trigger_player_miss(&self, queue: &mut SoundEventQueue) {
        if !self.enabled {
            return;
        }

        let weapon_type = self
            .player_context
            .weapon_type
            .unwrap_or(WeaponSoundType::Fist);
        let event = SoundEvent::WeaponMiss { weapon_type };

        let mut sound = QueuedSound::new(event);
        if let Some(pos) = self.player_context.position {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger critical hit sound.
    pub fn trigger_critical_hit(&self, queue: &mut SoundEventQueue, position: Option<[f32; 3]>) {
        if !self.enabled {
            return;
        }

        let mut sound = QueuedSound::new(SoundEvent::CriticalHit);
        if let Some(pos) = position {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger inventory action sound.
    pub fn trigger_inventory(
        &self,
        queue: &mut SoundEventQueue,
        action: InventorySoundAction,
        ctx: &InventorySoundContext,
    ) {
        if !self.enabled {
            return;
        }

        let event = match action {
            InventorySoundAction::Open => SoundEvent::InventoryOpen,
            InventorySoundAction::Close => SoundEvent::InventoryClose,
            InventorySoundAction::Pickup => {
                let item_type = ctx.item_type.unwrap_or(ItemSoundType::Material);
                SoundEvent::ItemPickup { item_type }
            },
            InventorySoundAction::Drop => {
                let item_type = ctx.item_type.unwrap_or(ItemSoundType::Material);
                SoundEvent::ItemDrop { item_type }
            },
            InventorySoundAction::Equip => {
                let item_type = ctx.item_type.unwrap_or(ItemSoundType::Material);
                SoundEvent::ItemEquip { item_type }
            },
            InventorySoundAction::Use => {
                let item_type = ctx.item_type.unwrap_or(ItemSoundType::Material);
                SoundEvent::ItemUse { item_type }
            },
        };

        let mut sound = QueuedSound::new(event);
        if let Some(pos) = ctx.position {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger NPC action sound.
    pub fn trigger_npc(
        &self,
        queue: &mut SoundEventQueue,
        action: NPCSoundAction,
        ctx: &NPCSoundContext,
    ) {
        if !self.enabled {
            return;
        }

        let event = match action {
            NPCSoundAction::Walk => {
                let surface = ctx.surface.unwrap_or(SurfaceType::Grass);
                SoundEvent::NPCFootstep { surface }
            },
            NPCSoundAction::DialogueStart => SoundEvent::DialogueStart,
            NPCSoundAction::DialogueEnd => SoundEvent::DialogueEnd,
            NPCSoundAction::Alert => SoundEvent::NPCAlert,
        };

        let mut sound = QueuedSound::new(event);
        if let Some(pos) = ctx.position {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger environment action sound.
    pub fn trigger_environment(
        &self,
        queue: &mut SoundEventQueue,
        action: EnvironmentAction,
        ctx: &EnvironmentSoundContext,
    ) {
        if !self.enabled {
            return;
        }

        let event = match action {
            EnvironmentAction::DoorOpen => SoundEvent::DoorOpen,
            EnvironmentAction::DoorClose => SoundEvent::DoorClose,
            EnvironmentAction::ChestOpen => SoundEvent::ChestOpen,
            EnvironmentAction::ChestClose => SoundEvent::ChestClose,
            EnvironmentAction::LeverActivate => SoundEvent::LeverActivate,
            EnvironmentAction::BlockBreak => {
                let block_type = ctx.block_type.unwrap_or(BlockSoundType::Stone);
                SoundEvent::BlockBreak { block_type }
            },
            EnvironmentAction::BlockPlace => {
                let block_type = ctx.block_type.unwrap_or(BlockSoundType::Stone);
                SoundEvent::BlockPlace { block_type }
            },
        };

        let mut sound = QueuedSound::new(event);
        if let Some(pos) = ctx.position {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger monster action sound.
    pub fn trigger_monster(
        &self,
        queue: &mut SoundEventQueue,
        action: MonsterAction,
        ctx: &MonsterSoundContext,
    ) {
        if !self.enabled {
            return;
        }

        let monster_type = ctx.monster_type.unwrap_or(MonsterSoundType::Slime);

        let event = match action {
            MonsterAction::Idle => SoundEvent::MonsterGrowl { monster_type },
            MonsterAction::Attack => SoundEvent::MonsterAttack { monster_type },
            MonsterAction::Hurt => SoundEvent::MonsterHurt { monster_type },
            MonsterAction::Die => SoundEvent::MonsterDeath { monster_type },
        };

        let mut sound = QueuedSound::new(event);
        if let Some(pos) = ctx.position {
            sound = sound.at_position(pos);
        }
        queue.push_sound(sound);
    }

    /// Trigger crafting action sound.
    pub fn trigger_crafting(&self, queue: &mut SoundEventQueue, action: CraftingAction) {
        if !self.enabled {
            return;
        }

        let event = match action {
            CraftingAction::Start => SoundEvent::CraftStart,
            CraftingAction::Complete => SoundEvent::CraftComplete,
            CraftingAction::Fail => SoundEvent::CraftFail,
        };

        queue.push(event);
    }

    /// Trigger quest action sound.
    pub fn trigger_quest(&self, queue: &mut SoundEventQueue, action: QuestAction) {
        if !self.enabled {
            return;
        }

        let event = match action {
            QuestAction::Accept => SoundEvent::QuestAccept,
            QuestAction::Objective => SoundEvent::QuestObjective,
            QuestAction::Complete => SoundEvent::QuestComplete,
            QuestAction::LevelUp => SoundEvent::LevelUp,
        };

        queue.push(event);
    }

    /// Trigger UI action sound.
    pub fn trigger_ui(&self, queue: &mut SoundEventQueue, action: UIAction) {
        if !self.enabled {
            return;
        }

        let event = match action {
            UIAction::Click => SoundEvent::UIClick,
            UIAction::MenuOpen => SoundEvent::UIMenuOpen,
            UIAction::MenuClose => SoundEvent::UIMenuClose,
            UIAction::Error => SoundEvent::UIError,
            UIAction::Success => SoundEvent::UISuccess,
            UIAction::Notification => SoundEvent::UINotification,
        };

        queue.push(event);
    }
}

impl Default for SoundTriggerSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_sound_context() {
        let ctx = PlayerSoundContext::new()
            .with_surface(SurfaceType::Grass)
            .with_position([1.0, 2.0, 3.0])
            .with_running(true)
            .with_weapon(WeaponSoundType::Sword);

        assert_eq!(ctx.surface, Some(SurfaceType::Grass));
        assert_eq!(ctx.position, Some([1.0, 2.0, 3.0]));
        assert!(ctx.is_running);
        assert_eq!(ctx.weapon_type, Some(WeaponSoundType::Sword));
    }

    #[test]
    fn test_inventory_sound_context() {
        let ctx = InventorySoundContext::new()
            .with_item_type(ItemSoundType::Weapon)
            .with_position([1.0, 2.0, 3.0]);

        assert_eq!(ctx.item_type, Some(ItemSoundType::Weapon));
        assert_eq!(ctx.position, Some([1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_biome_to_surface() {
        assert_eq!(biome_to_surface(BiomeType::Desert), SurfaceType::Sand);
        assert_eq!(biome_to_surface(BiomeType::Lake), SurfaceType::Water);
        assert_eq!(biome_to_surface(BiomeType::Forest), SurfaceType::Grass);
        assert_eq!(biome_to_surface(BiomeType::Mountain), SurfaceType::Stone);
    }

    #[test]
    fn test_monster_to_sound_type() {
        assert_eq!(
            monster_to_sound_type(MonsterType::Slime),
            MonsterSoundType::Slime
        );
        assert_eq!(
            monster_to_sound_type(MonsterType::Skeleton),
            MonsterSoundType::Skeleton
        );
        assert_eq!(
            monster_to_sound_type(MonsterType::Spider),
            MonsterSoundType::Spider
        );
    }

    #[test]
    fn test_animal_to_sound_type() {
        assert_eq!(
            animal_to_sound_type(AnimalType::Wolf),
            Some(MonsterSoundType::Wolf)
        );
        assert_eq!(
            animal_to_sound_type(AnimalType::Bear),
            Some(MonsterSoundType::Bear)
        );
        assert!(animal_to_sound_type(AnimalType::Chicken).is_none());
    }

    #[test]
    fn test_sound_trigger_player_walk() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        trigger.trigger_player(&mut queue, PlayerAction::Walk);
        assert!(!queue.is_empty());
    }

    #[test]
    fn test_sound_trigger_player_jump() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        trigger.trigger_player(&mut queue, PlayerAction::Jump);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(sound.event, SoundEvent::Jump));
    }

    #[test]
    fn test_sound_trigger_player_attack() {
        let mut trigger = SoundTriggerSystem::new();
        trigger.set_player_weapon(WeaponSoundType::Sword);

        let mut queue = SoundEventQueue::new();
        trigger.trigger_player(&mut queue, PlayerAction::Attack);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(
            sound.event,
            SoundEvent::WeaponSwing {
                weapon_type: WeaponSoundType::Sword
            }
        ));
    }

    #[test]
    fn test_sound_trigger_inventory() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        let ctx = InventorySoundContext::new().with_item_type(ItemSoundType::Coin);
        trigger.trigger_inventory(&mut queue, InventorySoundAction::Pickup, &ctx);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(
            sound.event,
            SoundEvent::ItemPickup {
                item_type: ItemSoundType::Coin
            }
        ));
    }

    #[test]
    fn test_sound_trigger_npc() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        let ctx = NPCSoundContext::new().with_surface(SurfaceType::Stone);
        trigger.trigger_npc(&mut queue, NPCSoundAction::Walk, &ctx);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(
            sound.event,
            SoundEvent::NPCFootstep {
                surface: SurfaceType::Stone
            }
        ));
    }

    #[test]
    fn test_sound_trigger_environment() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        let ctx = EnvironmentSoundContext::new().with_block_type(BlockSoundType::Wood);
        trigger.trigger_environment(&mut queue, EnvironmentAction::BlockBreak, &ctx);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(
            sound.event,
            SoundEvent::BlockBreak {
                block_type: BlockSoundType::Wood
            }
        ));
    }

    #[test]
    fn test_sound_trigger_monster() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        let ctx = MonsterSoundContext::new().with_monster_type(MonsterSoundType::Goblin);
        trigger.trigger_monster(&mut queue, MonsterAction::Attack, &ctx);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(
            sound.event,
            SoundEvent::MonsterAttack {
                monster_type: MonsterSoundType::Goblin
            }
        ));
    }

    #[test]
    fn test_sound_trigger_crafting() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        trigger.trigger_crafting(&mut queue, CraftingAction::Complete);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(sound.event, SoundEvent::CraftComplete));
    }

    #[test]
    fn test_sound_trigger_quest() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        trigger.trigger_quest(&mut queue, QuestAction::LevelUp);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(sound.event, SoundEvent::LevelUp));
    }

    #[test]
    fn test_sound_trigger_ui() {
        let trigger = SoundTriggerSystem::new();
        let mut queue = SoundEventQueue::new();

        trigger.trigger_ui(&mut queue, UIAction::Click);

        let sound = queue.pop().expect("should have sound");
        assert!(matches!(sound.event, SoundEvent::UIClick));
    }

    #[test]
    fn test_sound_trigger_disabled() {
        let mut trigger = SoundTriggerSystem::new();
        trigger.set_enabled(false);

        let mut queue = SoundEventQueue::new();
        trigger.trigger_player(&mut queue, PlayerAction::Jump);

        assert!(queue.is_empty());
    }

    #[test]
    fn test_sound_trigger_position_tracking() {
        let mut trigger = SoundTriggerSystem::new();
        let entity = EntityId::new();

        trigger.set_entity_position(entity, [1.0, 2.0, 3.0]);
        assert_eq!(trigger.entity_position(entity), Some([1.0, 2.0, 3.0]));

        trigger.remove_entity(entity);
        assert!(trigger.entity_position(entity).is_none());
    }

    #[test]
    fn test_sound_trigger_player_context_update() {
        let mut trigger = SoundTriggerSystem::new();

        trigger.set_player_surface(SurfaceType::Wood);
        trigger.set_player_position([5.0, 0.0, 5.0]);

        assert_eq!(trigger.player_context.surface, Some(SurfaceType::Wood));
        assert_eq!(trigger.player_context.position, Some([5.0, 0.0, 5.0]));
    }
}
