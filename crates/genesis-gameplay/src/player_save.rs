//! Player state serialization.
//!
//! This module provides comprehensive player state saving:
//! - Position, rotation, velocity
//! - Stats (HP, stamina, experience, level)
//! - Inventory and equipment
//! - Quest progress and learned recipes
//! - Skills and abilities

use genesis_common::{EntityId, FactionId, ItemTypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// G-53: Player Stats Save
// ============================================================================

/// Player vital statistics for saving.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerStatsSave {
    /// Current health points.
    pub hp: f32,
    /// Maximum health points.
    pub max_hp: f32,
    /// Current stamina.
    pub stamina: f32,
    /// Maximum stamina.
    pub max_stamina: f32,
    /// Current mana/magic points.
    pub mana: f32,
    /// Maximum mana.
    pub max_mana: f32,
    /// Total experience points.
    pub experience: u64,
    /// Current level.
    pub level: u32,
    /// Experience required for next level.
    pub exp_to_next_level: u64,
    /// Base attack stat.
    pub attack: f32,
    /// Base defense stat.
    pub defense: f32,
    /// Movement speed modifier.
    pub speed: f32,
}

impl Default for PlayerStatsSave {
    fn default() -> Self {
        Self {
            hp: 100.0,
            max_hp: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            mana: 50.0,
            max_mana: 50.0,
            experience: 0,
            level: 1,
            exp_to_next_level: 100,
            attack: 10.0,
            defense: 5.0,
            speed: 1.0,
        }
    }
}

impl PlayerStatsSave {
    /// Create new player stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set health values.
    #[must_use]
    pub fn with_health(mut self, hp: f32, max_hp: f32) -> Self {
        self.hp = hp;
        self.max_hp = max_hp;
        self
    }

    /// Set stamina values.
    #[must_use]
    pub fn with_stamina(mut self, stamina: f32, max_stamina: f32) -> Self {
        self.stamina = stamina;
        self.max_stamina = max_stamina;
        self
    }

    /// Set mana values.
    #[must_use]
    pub fn with_mana(mut self, mana: f32, max_mana: f32) -> Self {
        self.mana = mana;
        self.max_mana = max_mana;
        self
    }

    /// Set experience and level.
    #[must_use]
    pub fn with_experience(mut self, exp: u64, level: u32) -> Self {
        self.experience = exp;
        self.level = level;
        self.exp_to_next_level = Self::calculate_exp_for_level(level + 1);
        self
    }

    /// Calculate experience needed for a level.
    #[must_use]
    pub fn calculate_exp_for_level(level: u32) -> u64 {
        // Standard RPG formula: 100 * level^1.5
        (100.0 * (level as f64).powf(1.5)) as u64
    }

    /// Check if player can level up.
    #[must_use]
    pub fn can_level_up(&self) -> bool {
        self.experience >= self.exp_to_next_level
    }

    /// Get health percentage.
    #[must_use]
    pub fn hp_percent(&self) -> f32 {
        if self.max_hp > 0.0 {
            (self.hp / self.max_hp * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    /// Get stamina percentage.
    #[must_use]
    pub fn stamina_percent(&self) -> f32 {
        if self.max_stamina > 0.0 {
            (self.stamina / self.max_stamina * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }
}

// ============================================================================
// G-53: Item Save Data
// ============================================================================

/// Saved item data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSave {
    /// Item type ID.
    pub item_type: u32,
    /// Stack quantity.
    pub quantity: u32,
    /// Item durability (if applicable).
    pub durability: Option<f32>,
    /// Maximum durability.
    pub max_durability: Option<f32>,
    /// Custom item data (enchantments, etc.).
    pub custom_data: Option<Vec<u8>>,
    /// Unique item ID (for tracking).
    pub unique_id: Option<u64>,
}

impl ItemSave {
    /// Create new item save data.
    #[must_use]
    pub fn new(item_type: ItemTypeId, quantity: u32) -> Self {
        Self {
            item_type: item_type.raw(),
            quantity,
            durability: None,
            max_durability: None,
            custom_data: None,
            unique_id: None,
        }
    }

    /// Set durability.
    #[must_use]
    pub fn with_durability(mut self, current: f32, max: f32) -> Self {
        self.durability = Some(current);
        self.max_durability = Some(max);
        self
    }

    /// Set custom data.
    #[must_use]
    pub fn with_custom_data(mut self, data: Vec<u8>) -> Self {
        self.custom_data = Some(data);
        self
    }

    /// Set unique ID.
    #[must_use]
    pub fn with_unique_id(mut self, id: u64) -> Self {
        self.unique_id = Some(id);
        self
    }

    /// Get item type ID.
    #[must_use]
    pub fn item_type(&self) -> ItemTypeId {
        ItemTypeId::new(self.item_type)
    }

    /// Get durability percentage.
    #[must_use]
    pub fn durability_percent(&self) -> Option<f32> {
        match (self.durability, self.max_durability) {
            (Some(current), Some(max)) if max > 0.0 => Some(current / max * 100.0),
            _ => None,
        }
    }
}

// ============================================================================
// G-53: Inventory Save
// ============================================================================

/// Inventory slot save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventorySlotSave {
    /// Slot index.
    pub slot: usize,
    /// Item in slot (if any).
    pub item: Option<ItemSave>,
}

/// Complete inventory save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InventorySave {
    /// Main inventory slots.
    pub slots: Vec<InventorySlotSave>,
    /// Total slot count.
    pub capacity: usize,
    /// Currently selected hotbar slot.
    pub selected_slot: usize,
}

impl InventorySave {
    /// Create new inventory save.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: Vec::new(),
            capacity,
            selected_slot: 0,
        }
    }

    /// Add item to a slot.
    pub fn set_slot(&mut self, slot: usize, item: ItemSave) {
        self.slots.push(InventorySlotSave {
            slot,
            item: Some(item),
        });
    }

    /// Get item at slot.
    #[must_use]
    pub fn get_slot(&self, slot: usize) -> Option<&ItemSave> {
        self.slots
            .iter()
            .find(|s| s.slot == slot)
            .and_then(|s| s.item.as_ref())
    }

    /// Count total items.
    #[must_use]
    pub fn total_items(&self) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.item.as_ref())
            .map(|i| i.quantity)
            .sum()
    }

    /// Count occupied slots.
    #[must_use]
    pub fn occupied_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.item.is_some()).count()
    }
}

// ============================================================================
// G-53: Equipment Save
// ============================================================================

/// Equipment slot type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    /// Head armor.
    Head,
    /// Chest armor.
    Chest,
    /// Leg armor.
    Legs,
    /// Foot armor.
    Feet,
    /// Hand armor/gloves.
    Hands,
    /// Main hand weapon.
    MainHand,
    /// Off hand (shield/secondary).
    OffHand,
    /// Accessory slot 1.
    Accessory1,
    /// Accessory slot 2.
    Accessory2,
    /// Back slot (cape/backpack).
    Back,
}

impl EquipmentSlot {
    /// Get all equipment slots.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Head,
            Self::Chest,
            Self::Legs,
            Self::Feet,
            Self::Hands,
            Self::MainHand,
            Self::OffHand,
            Self::Accessory1,
            Self::Accessory2,
            Self::Back,
        ]
    }

    /// Check if slot is armor.
    #[must_use]
    pub fn is_armor(&self) -> bool {
        matches!(
            self,
            Self::Head | Self::Chest | Self::Legs | Self::Feet | Self::Hands
        )
    }

    /// Check if slot is weapon.
    #[must_use]
    pub fn is_weapon(&self) -> bool {
        matches!(self, Self::MainHand | Self::OffHand)
    }
}

/// Equipment save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EquipmentSave {
    /// Equipped items by slot.
    pub equipped: HashMap<EquipmentSlot, ItemSave>,
}

impl EquipmentSave {
    /// Create new equipment save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Equip item to slot.
    pub fn equip(&mut self, slot: EquipmentSlot, item: ItemSave) {
        self.equipped.insert(slot, item);
    }

    /// Get equipped item.
    #[must_use]
    pub fn get(&self, slot: EquipmentSlot) -> Option<&ItemSave> {
        self.equipped.get(&slot)
    }

    /// Check if slot is occupied.
    #[must_use]
    pub fn is_equipped(&self, slot: EquipmentSlot) -> bool {
        self.equipped.contains_key(&slot)
    }

    /// Count equipped items.
    #[must_use]
    pub fn equipped_count(&self) -> usize {
        self.equipped.len()
    }
}

// ============================================================================
// G-53: Hotbar Save
// ============================================================================

/// Hotbar slot save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotbarSlotSave {
    /// Item reference (inventory slot or item ID).
    pub item_ref: Option<u32>,
    /// Quick action bound to slot.
    pub action: Option<String>,
}

/// Hotbar save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HotbarSave {
    /// Hotbar slots.
    pub slots: Vec<HotbarSlotSave>,
    /// Currently selected slot.
    pub selected: usize,
}

impl HotbarSave {
    /// Create new hotbar save with slot count.
    #[must_use]
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: (0..slot_count)
                .map(|_| HotbarSlotSave {
                    item_ref: None,
                    action: None,
                })
                .collect(),
            selected: 0,
        }
    }

    /// Set item in slot.
    pub fn set_item(&mut self, slot: usize, item_ref: u32) {
        if slot < self.slots.len() {
            self.slots[slot].item_ref = Some(item_ref);
        }
    }

    /// Set action in slot.
    pub fn set_action(&mut self, slot: usize, action: String) {
        if slot < self.slots.len() {
            self.slots[slot].action = Some(action);
        }
    }

    /// Clear slot.
    pub fn clear_slot(&mut self, slot: usize) {
        if slot < self.slots.len() {
            self.slots[slot] = HotbarSlotSave {
                item_ref: None,
                action: None,
            };
        }
    }
}

// ============================================================================
// G-53: Quest Progress Save
// ============================================================================

/// Quest objective progress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestObjectiveSave {
    /// Objective ID.
    pub objective_id: String,
    /// Current progress.
    pub progress: u32,
    /// Required for completion.
    pub required: u32,
    /// Whether objective is complete.
    pub complete: bool,
}

/// Quest save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestSave {
    /// Quest ID.
    pub quest_id: String,
    /// Current quest stage.
    pub stage: u32,
    /// Objective progress.
    pub objectives: Vec<QuestObjectiveSave>,
    /// Time quest was started (game time).
    pub started_at: f64,
    /// Whether quest is complete.
    pub complete: bool,
    /// Whether quest failed.
    pub failed: bool,
}

impl QuestSave {
    /// Create new quest save.
    #[must_use]
    pub fn new(quest_id: impl Into<String>, started_at: f64) -> Self {
        Self {
            quest_id: quest_id.into(),
            stage: 0,
            objectives: Vec::new(),
            started_at,
            complete: false,
            failed: false,
        }
    }

    /// Add objective progress.
    pub fn add_objective(&mut self, id: impl Into<String>, progress: u32, required: u32) {
        self.objectives.push(QuestObjectiveSave {
            objective_id: id.into(),
            progress,
            required,
            complete: progress >= required,
        });
    }

    /// Check if all objectives are complete.
    #[must_use]
    pub fn all_objectives_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.complete)
    }
}

/// Quest progress save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct QuestProgressSave {
    /// Active quests.
    pub active: Vec<QuestSave>,
    /// Completed quest IDs.
    pub completed: HashSet<String>,
    /// Failed quest IDs.
    pub failed: HashSet<String>,
    /// Quest-related flags.
    pub flags: HashMap<String, bool>,
}

impl QuestProgressSave {
    /// Create new quest progress save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add active quest.
    pub fn add_active(&mut self, quest: QuestSave) {
        self.active.push(quest);
    }

    /// Mark quest complete.
    pub fn complete_quest(&mut self, quest_id: &str) {
        self.active.retain(|q| q.quest_id != quest_id);
        self.completed.insert(quest_id.to_string());
    }

    /// Mark quest failed.
    pub fn fail_quest(&mut self, quest_id: &str) {
        self.active.retain(|q| q.quest_id != quest_id);
        self.failed.insert(quest_id.to_string());
    }

    /// Set quest flag.
    pub fn set_flag(&mut self, flag: impl Into<String>, value: bool) {
        self.flags.insert(flag.into(), value);
    }

    /// Get quest flag.
    #[must_use]
    pub fn get_flag(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    /// Count active quests.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Count completed quests.
    #[must_use]
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }
}

// ============================================================================
// G-53: Learned Recipes Save
// ============================================================================

/// Learned recipe save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LearnedRecipesSave {
    /// Learned recipe IDs.
    pub recipes: HashSet<u32>,
    /// Recipe mastery levels (recipe_id -> mastery).
    pub mastery: HashMap<u32, u32>,
    /// Times each recipe has been crafted.
    pub craft_counts: HashMap<u32, u32>,
    /// Favorite recipes.
    pub favorites: HashSet<u32>,
}

impl LearnedRecipesSave {
    /// Create new learned recipes save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Learn a recipe.
    pub fn learn(&mut self, recipe_id: u32) {
        self.recipes.insert(recipe_id);
    }

    /// Check if recipe is learned.
    #[must_use]
    pub fn is_learned(&self, recipe_id: u32) -> bool {
        self.recipes.contains(&recipe_id)
    }

    /// Record a craft.
    pub fn record_craft(&mut self, recipe_id: u32) {
        *self.craft_counts.entry(recipe_id).or_insert(0) += 1;

        // Update mastery based on craft count
        let count = self.craft_counts[&recipe_id];
        let mastery_level = match count {
            0..=4 => 0,
            5..=19 => 1,
            20..=49 => 2,
            50..=99 => 3,
            _ => 4,
        };
        self.mastery.insert(recipe_id, mastery_level);
    }

    /// Get mastery level.
    #[must_use]
    pub fn get_mastery(&self, recipe_id: u32) -> u32 {
        self.mastery.get(&recipe_id).copied().unwrap_or(0)
    }

    /// Toggle favorite.
    pub fn toggle_favorite(&mut self, recipe_id: u32) {
        if self.favorites.contains(&recipe_id) {
            self.favorites.remove(&recipe_id);
        } else {
            self.favorites.insert(recipe_id);
        }
    }

    /// Count learned recipes.
    #[must_use]
    pub fn learned_count(&self) -> usize {
        self.recipes.len()
    }
}

// ============================================================================
// G-53: Skills Save
// ============================================================================

/// Skill save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillSave {
    /// Skill ID.
    pub skill_id: String,
    /// Current level.
    pub level: u32,
    /// Current experience.
    pub experience: u64,
    /// Experience to next level.
    pub exp_to_next: u64,
}

impl SkillSave {
    /// Create new skill save.
    #[must_use]
    pub fn new(skill_id: impl Into<String>) -> Self {
        Self {
            skill_id: skill_id.into(),
            level: 1,
            experience: 0,
            exp_to_next: 100,
        }
    }

    /// Set level and experience.
    #[must_use]
    pub fn with_progress(mut self, level: u32, exp: u64) -> Self {
        self.level = level;
        self.experience = exp;
        self.exp_to_next = 100 * (level as u64 + 1);
        self
    }
}

/// Skills save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SkillsSave {
    /// Skills by ID.
    pub skills: HashMap<String, SkillSave>,
    /// Total skill points spent.
    pub points_spent: u32,
    /// Available skill points.
    pub points_available: u32,
}

impl SkillsSave {
    /// Create new skills save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update skill.
    pub fn set_skill(&mut self, skill: SkillSave) {
        self.skills.insert(skill.skill_id.clone(), skill);
    }

    /// Get skill by ID.
    #[must_use]
    pub fn get_skill(&self, skill_id: &str) -> Option<&SkillSave> {
        self.skills.get(skill_id)
    }

    /// Get total skill levels.
    #[must_use]
    pub fn total_levels(&self) -> u32 {
        self.skills.values().map(|s| s.level).sum()
    }
}

// ============================================================================
// G-53: Faction Reputation Save
// ============================================================================

/// Faction reputation save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactionRepSave {
    /// Faction ID.
    pub faction_id: u16,
    /// Current reputation value.
    pub reputation: i32,
    /// Highest reputation ever achieved.
    pub highest: i32,
    /// Whether player is a member.
    pub is_member: bool,
    /// Faction rank (if member).
    pub rank: Option<u32>,
}

impl FactionRepSave {
    /// Create new faction reputation save.
    #[must_use]
    pub fn new(faction_id: FactionId, reputation: i32) -> Self {
        Self {
            faction_id: faction_id.raw(),
            reputation,
            highest: reputation,
            is_member: false,
            rank: None,
        }
    }

    /// Get faction ID.
    #[must_use]
    pub fn faction_id(&self) -> FactionId {
        FactionId::new(self.faction_id)
    }

    /// Set membership status.
    #[must_use]
    pub fn with_membership(mut self, is_member: bool, rank: Option<u32>) -> Self {
        self.is_member = is_member;
        self.rank = rank;
        self
    }
}

/// Factions save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionsSave {
    /// Reputation with each faction.
    pub factions: HashMap<u16, FactionRepSave>,
}

impl FactionsSave {
    /// Create new factions save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set faction reputation.
    pub fn set_reputation(&mut self, save: FactionRepSave) {
        self.factions.insert(save.faction_id, save);
    }

    /// Get reputation with faction.
    #[must_use]
    pub fn get_reputation(&self, faction_id: FactionId) -> Option<&FactionRepSave> {
        self.factions.get(&faction_id.raw())
    }
}

// ============================================================================
// G-53: Complete Player Save
// ============================================================================

/// Complete player state save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerFullSave {
    /// Save format version.
    pub version: u32,
    /// Player unique ID.
    pub player_id: u64,
    /// Player name.
    pub name: String,
    /// Position (x, y).
    pub position: (f32, f32),
    /// Rotation/facing angle.
    pub rotation: f32,
    /// Velocity (for physics continuity).
    pub velocity: (f32, f32),
    /// Player stats.
    pub stats: PlayerStatsSave,
    /// Inventory.
    pub inventory: InventorySave,
    /// Equipment.
    pub equipment: EquipmentSave,
    /// Hotbar.
    pub hotbar: HotbarSave,
    /// Quest progress.
    pub quests: QuestProgressSave,
    /// Learned recipes.
    pub recipes: LearnedRecipesSave,
    /// Skills.
    pub skills: SkillsSave,
    /// Faction reputations.
    pub factions: FactionsSave,
    /// Spawn point.
    pub spawn_point: (f32, f32),
    /// Last death position (if any).
    pub last_death_position: Option<(f32, f32)>,
    /// Total playtime in seconds.
    pub playtime_seconds: f64,
    /// Custom player flags.
    pub flags: HashMap<String, String>,
}

impl Default for PlayerFullSave {
    fn default() -> Self {
        Self {
            version: 1,
            player_id: 0,
            name: String::from("Player"),
            position: (0.0, 0.0),
            rotation: 0.0,
            velocity: (0.0, 0.0),
            stats: PlayerStatsSave::default(),
            inventory: InventorySave::new(40),
            equipment: EquipmentSave::new(),
            hotbar: HotbarSave::new(10),
            quests: QuestProgressSave::new(),
            recipes: LearnedRecipesSave::new(),
            skills: SkillsSave::new(),
            factions: FactionsSave::new(),
            spawn_point: (0.0, 0.0),
            last_death_position: None,
            playtime_seconds: 0.0,
            flags: HashMap::new(),
        }
    }
}

impl PlayerFullSave {
    /// Create new player save.
    #[must_use]
    pub fn new(player_id: EntityId, name: impl Into<String>) -> Self {
        Self {
            player_id: player_id.raw(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set position and rotation.
    #[must_use]
    pub fn with_transform(mut self, x: f32, y: f32, rotation: f32) -> Self {
        self.position = (x, y);
        self.rotation = rotation;
        self
    }

    /// Set velocity.
    #[must_use]
    pub fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.velocity = (vx, vy);
        self
    }

    /// Set stats.
    #[must_use]
    pub fn with_stats(mut self, stats: PlayerStatsSave) -> Self {
        self.stats = stats;
        self
    }

    /// Set inventory.
    #[must_use]
    pub fn with_inventory(mut self, inventory: InventorySave) -> Self {
        self.inventory = inventory;
        self
    }

    /// Set equipment.
    #[must_use]
    pub fn with_equipment(mut self, equipment: EquipmentSave) -> Self {
        self.equipment = equipment;
        self
    }

    /// Set spawn point.
    #[must_use]
    pub fn with_spawn_point(mut self, x: f32, y: f32) -> Self {
        self.spawn_point = (x, y);
        self
    }

    /// Set playtime.
    #[must_use]
    pub fn with_playtime(mut self, seconds: f64) -> Self {
        self.playtime_seconds = seconds;
        self
    }

    /// Add playtime.
    pub fn add_playtime(&mut self, seconds: f64) {
        self.playtime_seconds += seconds;
    }

    /// Set custom flag.
    pub fn set_flag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.flags.insert(key.into(), value.into());
    }

    /// Get custom flag.
    #[must_use]
    pub fn get_flag(&self, key: &str) -> Option<&str> {
        self.flags.get(key).map(String::as_str)
    }

    /// Record death.
    pub fn record_death(&mut self) {
        self.last_death_position = Some(self.position);
    }

    /// Get player ID.
    #[must_use]
    pub fn player_id(&self) -> EntityId {
        EntityId::from_raw(self.player_id)
    }

    /// Check if player is alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.stats.hp > 0.0
    }

    /// Get formatted playtime.
    #[must_use]
    pub fn formatted_playtime(&self) -> String {
        let total_seconds = self.playtime_seconds as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_stats_default() {
        let stats = PlayerStatsSave::new();
        assert_eq!(stats.hp, 100.0);
        assert_eq!(stats.level, 1);
        assert_eq!(stats.experience, 0);
    }

    #[test]
    fn test_player_stats_with_health() {
        let stats = PlayerStatsSave::new().with_health(50.0, 100.0);
        assert_eq!(stats.hp, 50.0);
        assert_eq!(stats.hp_percent(), 50.0);
    }

    #[test]
    fn test_player_stats_experience() {
        let stats = PlayerStatsSave::new().with_experience(500, 5);
        assert_eq!(stats.level, 5);
        assert_eq!(stats.experience, 500);
    }

    #[test]
    fn test_exp_for_level() {
        assert_eq!(PlayerStatsSave::calculate_exp_for_level(1), 100);
        assert!(PlayerStatsSave::calculate_exp_for_level(10) > 100);
    }

    #[test]
    fn test_item_save() {
        let item = ItemSave::new(ItemTypeId::new(1), 5);
        assert_eq!(item.item_type(), ItemTypeId::new(1));
        assert_eq!(item.quantity, 5);
    }

    #[test]
    fn test_item_durability() {
        let item = ItemSave::new(ItemTypeId::new(1), 1).with_durability(50.0, 100.0);
        assert_eq!(item.durability_percent(), Some(50.0));
    }

    #[test]
    fn test_inventory_save() {
        let mut inv = InventorySave::new(40);
        inv.set_slot(0, ItemSave::new(ItemTypeId::new(1), 10));
        inv.set_slot(5, ItemSave::new(ItemTypeId::new(2), 5));

        assert!(inv.get_slot(0).is_some());
        assert!(inv.get_slot(1).is_none());
        assert_eq!(inv.total_items(), 15);
        assert_eq!(inv.occupied_slots(), 2);
    }

    #[test]
    fn test_equipment_slots() {
        assert!(EquipmentSlot::Head.is_armor());
        assert!(EquipmentSlot::MainHand.is_weapon());
        assert!(!EquipmentSlot::Accessory1.is_armor());
    }

    #[test]
    fn test_equipment_save() {
        let mut equip = EquipmentSave::new();
        equip.equip(
            EquipmentSlot::MainHand,
            ItemSave::new(ItemTypeId::new(10), 1),
        );

        assert!(equip.is_equipped(EquipmentSlot::MainHand));
        assert!(!equip.is_equipped(EquipmentSlot::OffHand));
        assert_eq!(equip.equipped_count(), 1);
    }

    #[test]
    fn test_hotbar_save() {
        let mut hotbar = HotbarSave::new(10);
        hotbar.set_item(0, 5);
        hotbar.set_action(1, "attack".to_string());

        assert_eq!(hotbar.slots[0].item_ref, Some(5));
        assert_eq!(hotbar.slots[1].action, Some("attack".to_string()));
    }

    #[test]
    fn test_quest_save() {
        let mut quest = QuestSave::new("main_quest_1", 100.0);
        quest.add_objective("kill_rats", 5, 10);

        assert!(!quest.all_objectives_complete());
        assert_eq!(quest.objectives[0].progress, 5);
    }

    #[test]
    fn test_quest_progress_save() {
        let mut progress = QuestProgressSave::new();
        progress.add_active(QuestSave::new("quest1", 0.0));
        progress.complete_quest("quest1");

        assert_eq!(progress.active_count(), 0);
        assert_eq!(progress.completed_count(), 1);
    }

    #[test]
    fn test_learned_recipes() {
        let mut recipes = LearnedRecipesSave::new();
        recipes.learn(1);
        recipes.learn(2);
        recipes.record_craft(1);
        recipes.record_craft(1);
        recipes.record_craft(1);
        recipes.record_craft(1);
        recipes.record_craft(1);

        assert!(recipes.is_learned(1));
        assert_eq!(recipes.get_mastery(1), 1); // 5 crafts = mastery 1
        assert_eq!(recipes.learned_count(), 2);
    }

    #[test]
    fn test_skills_save() {
        let mut skills = SkillsSave::new();
        skills.set_skill(SkillSave::new("mining").with_progress(5, 250));

        assert_eq!(skills.get_skill("mining").unwrap().level, 5);
        assert_eq!(skills.total_levels(), 5);
    }

    #[test]
    fn test_faction_rep_save() {
        let rep = FactionRepSave::new(FactionId::new(1), 50).with_membership(true, Some(2));

        assert_eq!(rep.faction_id(), FactionId::new(1));
        assert!(rep.is_member);
        assert_eq!(rep.rank, Some(2));
    }

    #[test]
    fn test_player_full_save() {
        let save = PlayerFullSave::new(EntityId::from_raw(1), "TestPlayer")
            .with_transform(100.0, 200.0, 1.5)
            .with_spawn_point(0.0, 0.0)
            .with_playtime(3661.0);

        assert_eq!(save.name, "TestPlayer");
        assert_eq!(save.position, (100.0, 200.0));
        assert_eq!(save.formatted_playtime(), "01:01:01");
    }

    #[test]
    fn test_player_death_recording() {
        let mut save =
            PlayerFullSave::new(EntityId::from_raw(1), "Player").with_transform(50.0, 50.0, 0.0);

        save.record_death();
        assert_eq!(save.last_death_position, Some((50.0, 50.0)));
    }

    #[test]
    fn test_player_flags() {
        let mut save = PlayerFullSave::default();
        save.set_flag("tutorial_complete", "true");

        assert_eq!(save.get_flag("tutorial_complete"), Some("true"));
        assert_eq!(save.get_flag("nonexistent"), None);
    }
}
