//! Crafting event integration.
//!
//! This module provides:
//! - Wiring CraftItem events to inventory system
//! - Triggering crafting sounds
//! - Updating player stats and skills
//! - Achievement and quest triggers

use genesis_common::{EntityId, ItemTypeId, RecipeId};
use std::collections::VecDeque;
use tracing::{debug, info};

use crate::audio_assets::AudioCategory;
use crate::audio_integration::{AudioIntegration, SoundEvent};
use crate::recipe_loader::RecipeDefinition;

/// Event types for crafting actions.
#[derive(Debug, Clone)]
pub enum CraftingEvent {
    /// Player started crafting.
    CraftStarted {
        /// Entity doing the crafting.
        crafter: EntityId,
        /// Recipe being crafted.
        recipe_id: RecipeId,
        /// Recipe name (for display/sound).
        recipe_name: String,
        /// Category of the recipe.
        category: String,
    },
    /// Crafting progress update.
    CraftProgress {
        /// Entity doing the crafting.
        crafter: EntityId,
        /// Recipe being crafted.
        recipe_id: RecipeId,
        /// Progress (0.0 - 1.0).
        progress: f32,
    },
    /// Crafting completed successfully.
    CraftCompleted {
        /// Entity that crafted.
        crafter: EntityId,
        /// Recipe that was crafted.
        recipe_id: RecipeId,
        /// Output item.
        output_item: ItemTypeId,
        /// Output quantity.
        output_quantity: u32,
        /// Skill gained.
        skill_gain: u32,
    },
    /// Crafting failed.
    CraftFailed {
        /// Entity that attempted.
        crafter: EntityId,
        /// Recipe that failed.
        recipe_id: RecipeId,
        /// Failure reason.
        reason: CraftFailReason,
    },
    /// Recipe learned/unlocked.
    RecipeLearned {
        /// Entity that learned.
        learner: EntityId,
        /// Recipe that was learned.
        recipe_id: RecipeId,
    },
}

/// Reasons for craft failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftFailReason {
    /// Missing required ingredients.
    MissingIngredients,
    /// Missing required tools.
    MissingTools,
    /// Missing required workstation.
    MissingWorkstation,
    /// Skill level too low.
    SkillTooLow,
    /// Inventory full, can't add output.
    InventoryFull,
    /// Recipe not known.
    RecipeNotKnown,
    /// Crafting interrupted.
    Interrupted,
}

/// Statistics tracked for crafting.
#[derive(Debug, Clone, Default)]
pub struct CraftingStats {
    /// Total items crafted.
    pub items_crafted: u64,
    /// Total crafts completed.
    pub crafts_completed: u64,
    /// Total crafts failed.
    pub crafts_failed: u64,
    /// Total skill gained from crafting.
    pub skill_gained: u64,
    /// Crafts per recipe ID.
    pub crafts_by_recipe: std::collections::HashMap<u32, u32>,
}

impl CraftingStats {
    /// Creates new empty stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a completed craft.
    pub fn record_craft(&mut self, recipe_id: RecipeId, quantity: u32, skill_gain: u32) {
        self.items_crafted += u64::from(quantity);
        self.crafts_completed += 1;
        self.skill_gained += u64::from(skill_gain);
        *self.crafts_by_recipe.entry(recipe_id.raw()).or_insert(0) += 1;
    }

    /// Records a failed craft.
    pub fn record_failure(&mut self) {
        self.crafts_failed += 1;
    }

    /// Returns the most crafted recipe ID.
    #[must_use]
    pub fn most_crafted_recipe(&self) -> Option<(u32, u32)> {
        self.crafts_by_recipe
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(id, count)| (*id, *count))
    }
}

/// Achievement triggers from crafting.
#[derive(Debug, Clone)]
pub enum CraftingAchievement {
    /// First item crafted.
    FirstCraft,
    /// Crafted N items total.
    TotalCrafts(u64),
    /// Crafted N of a specific recipe.
    RecipeMastery {
        /// Recipe ID.
        recipe_id: u32,
        /// Count required.
        count: u32,
    },
    /// Learned N recipes.
    RecipeCollector(u32),
    /// Reached skill level.
    SkillMilestone {
        /// Skill name.
        skill: String,
        /// Level reached.
        level: u32,
    },
}

/// Quest progress triggers from crafting.
#[derive(Debug, Clone)]
pub struct CraftingQuestTrigger {
    /// Quest ID.
    pub quest_id: u32,
    /// Item type crafted.
    pub item_type: Option<ItemTypeId>,
    /// Recipe used.
    pub recipe_id: Option<RecipeId>,
    /// Count towards quest.
    pub count: u32,
}

/// Handler for crafting events that integrates with other systems.
pub struct CraftingEventHandler {
    /// Pending events to process.
    event_queue: VecDeque<CraftingEvent>,
    /// Crafting statistics.
    stats: CraftingStats,
    /// Achievement thresholds.
    achievement_thresholds: Vec<u64>,
    /// Pending achievements to grant.
    pending_achievements: Vec<CraftingAchievement>,
    /// Pending quest updates.
    pending_quest_triggers: Vec<CraftingQuestTrigger>,
}

impl Default for CraftingEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CraftingEventHandler {
    /// Creates a new crafting event handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            stats: CraftingStats::new(),
            achievement_thresholds: vec![1, 10, 50, 100, 500, 1000],
            pending_achievements: Vec::new(),
            pending_quest_triggers: Vec::new(),
        }
    }

    /// Queues a crafting event for processing.
    pub fn queue_event(&mut self, event: CraftingEvent) {
        self.event_queue.push_back(event);
    }

    /// Processes all pending events and returns triggered actions.
    pub fn process_events(&mut self, mut audio: Option<&mut AudioIntegration>) -> ProcessedCraftingEvents {
        let mut result = ProcessedCraftingEvents::default();

        while let Some(event) = self.event_queue.pop_front() {
            match event {
                CraftingEvent::CraftStarted {
                    recipe_name,
                    category,
                    ..
                } => {
                    debug!("Craft started: {} ({})", recipe_name, category);
                    
                    // Play craft start sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound_name = craft_start_sound(&category);
                        let sound = SoundEvent::new(AudioCategory::Sfx, sound_name);
                        audio.queue_sound(sound);
                    }
                }
                CraftingEvent::CraftProgress { progress, .. } => {
                    result.in_progress = true;
                    result.progress = progress;
                }
                CraftingEvent::CraftCompleted {
                    crafter,
                    recipe_id,
                    output_item,
                    output_quantity,
                    skill_gain,
                } => {
                    info!(
                        "Craft completed: {:?} made {} x {:?}",
                        crafter, output_quantity, output_item
                    );

                    // Update stats
                    let prev_total = self.stats.crafts_completed;
                    self.stats.record_craft(recipe_id, output_quantity, skill_gain);

                    // Check for achievements
                    self.check_craft_achievements(prev_total, recipe_id);

                    // Play completion sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound = SoundEvent::new(AudioCategory::Sfx, "craft_complete");
                        audio.queue_sound(sound);
                    }

                    result.completed_crafts.push((recipe_id, output_item, output_quantity));
                    result.skill_gained += skill_gain;
                }
                CraftingEvent::CraftFailed { reason, .. } => {
                    debug!("Craft failed: {:?}", reason);
                    self.stats.record_failure();

                    // Play failure sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound = SoundEvent::new(AudioCategory::Ui, "craft_failed");
                        audio.queue_sound(sound);
                    }

                    result.failed = true;
                    result.fail_reason = Some(reason);
                }
                CraftingEvent::RecipeLearned { learner, recipe_id } => {
                    info!("Recipe learned: {:?} learned {:?}", learner, recipe_id);

                    // Play recipe learned sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound = SoundEvent::new(AudioCategory::Ui, "recipe_learned");
                        audio.queue_sound(sound);
                    }

                    result.recipes_learned.push(recipe_id);
                }
            }
        }

        // Collect pending achievements
        result.achievements = std::mem::take(&mut self.pending_achievements);
        result.quest_triggers = std::mem::take(&mut self.pending_quest_triggers);

        result
    }

    /// Checks and queues achievements after a craft.
    fn check_craft_achievements(&mut self, prev_total: u64, recipe_id: RecipeId) {
        let current_total = self.stats.crafts_completed;

        // First craft achievement
        if prev_total == 0 && current_total == 1 {
            self.pending_achievements.push(CraftingAchievement::FirstCraft);
        }

        // Total craft milestones
        for &threshold in &self.achievement_thresholds {
            if prev_total < threshold && current_total >= threshold {
                self.pending_achievements
                    .push(CraftingAchievement::TotalCrafts(threshold));
            }
        }

        // Recipe mastery milestones
        let recipe_count = self.stats.crafts_by_recipe.get(&recipe_id.raw()).copied().unwrap_or(0);
        for milestone in [10, 50, 100] {
            if recipe_count == milestone {
                self.pending_achievements
                    .push(CraftingAchievement::RecipeMastery {
                        recipe_id: recipe_id.raw(),
                        count: milestone,
                    });
            }
        }
    }

    /// Returns crafting statistics.
    #[must_use]
    pub fn stats(&self) -> &CraftingStats {
        &self.stats
    }

    /// Returns mutable statistics (for loading saves).
    pub fn stats_mut(&mut self) -> &mut CraftingStats {
        &mut self.stats
    }

    /// Creates a craft started event from a recipe definition.
    #[must_use]
    pub fn make_start_event(crafter: EntityId, recipe: &RecipeDefinition) -> CraftingEvent {
        CraftingEvent::CraftStarted {
            crafter,
            recipe_id: RecipeId::new(recipe.id),
            recipe_name: recipe.name.clone(),
            category: recipe.category.clone(),
        }
    }

    /// Creates a craft completed event from a recipe definition.
    #[must_use]
    pub fn make_complete_event(crafter: EntityId, recipe: &RecipeDefinition) -> CraftingEvent {
        CraftingEvent::CraftCompleted {
            crafter,
            recipe_id: RecipeId::new(recipe.id),
            output_item: ItemTypeId::new(recipe.output.item_id),
            output_quantity: recipe.output.quantity,
            skill_gain: recipe.skill_gain,
        }
    }

    /// Registers a quest trigger for crafting.
    pub fn register_quest_trigger(
        &mut self,
        quest_id: u32,
        item_type: Option<ItemTypeId>,
        recipe_id: Option<RecipeId>,
    ) {
        self.pending_quest_triggers.push(CraftingQuestTrigger {
            quest_id,
            item_type,
            recipe_id,
            count: 1,
        });
    }
}

/// Result of processing crafting events.
#[derive(Debug, Default)]
pub struct ProcessedCraftingEvents {
    /// Whether crafting is in progress.
    pub in_progress: bool,
    /// Current progress (0.0 - 1.0).
    pub progress: f32,
    /// Completed crafts (recipe_id, output_item, quantity).
    pub completed_crafts: Vec<(RecipeId, ItemTypeId, u32)>,
    /// Total skill gained.
    pub skill_gained: u32,
    /// Whether a craft failed.
    pub failed: bool,
    /// Failure reason (if failed).
    pub fail_reason: Option<CraftFailReason>,
    /// Recipes learned this frame.
    pub recipes_learned: Vec<RecipeId>,
    /// Achievements earned.
    pub achievements: Vec<CraftingAchievement>,
    /// Quest triggers activated.
    pub quest_triggers: Vec<CraftingQuestTrigger>,
}

/// Returns the appropriate crafting start sound for a recipe category.
#[allow(clippy::match_same_arms)]
fn craft_start_sound(category: &str) -> &'static str {
    match category {
        "weapons" => "craft_metal_start",
        "armor" => "craft_metal_start",
        "tools" => "craft_tool_start",
        "consumables" => "craft_potion_start",
        "materials" => "craft_material_start",
        "building_components" => "craft_construction_start",
        "electronics" => "craft_tech_start",
        "chemistry" => "craft_chemistry_start",
        _ => "craft_generic_start",
    }
}

/// Workstation types for crafting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkstationType {
    /// Basic crafting table.
    CraftingTable,
    /// Forge/anvil for metalwork.
    Forge,
    /// Chemistry station.
    ChemistryBench,
    /// Electronics workbench.
    ElectronicsBench,
    /// Cooking station.
    CookingStation,
    /// Sewing/textile station.
    SewingStation,
}

impl WorkstationType {
    /// Returns the building ID for this workstation type.
    #[must_use]
    pub const fn building_id(&self) -> u32 {
        match self {
            Self::CraftingTable => 100,
            Self::Forge => 101,
            Self::ChemistryBench => 102,
            Self::ElectronicsBench => 103,
            Self::CookingStation => 104,
            Self::SewingStation => 105,
        }
    }

    /// Creates a workstation type from a building ID.
    #[must_use]
    pub fn from_building_id(id: u32) -> Option<Self> {
        match id {
            100 => Some(Self::CraftingTable),
            101 => Some(Self::Forge),
            102 => Some(Self::ChemistryBench),
            103 => Some(Self::ElectronicsBench),
            104 => Some(Self::CookingStation),
            105 => Some(Self::SewingStation),
            _ => None,
        }
    }

    /// Returns the name of this workstation type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::CraftingTable => "Crafting Table",
            Self::Forge => "Forge",
            Self::ChemistryBench => "Chemistry Bench",
            Self::ElectronicsBench => "Electronics Bench",
            Self::CookingStation => "Cooking Station",
            Self::SewingStation => "Sewing Station",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crafting_stats_record() {
        let mut stats = CraftingStats::new();

        stats.record_craft(RecipeId::new(1), 5, 10);
        assert_eq!(stats.items_crafted, 5);
        assert_eq!(stats.crafts_completed, 1);
        assert_eq!(stats.skill_gained, 10);

        stats.record_craft(RecipeId::new(1), 3, 5);
        assert_eq!(stats.items_crafted, 8);
        assert_eq!(stats.crafts_completed, 2);
        assert_eq!(stats.skill_gained, 15);
        assert_eq!(stats.crafts_by_recipe.get(&1), Some(&2));
    }

    #[test]
    fn test_most_crafted_recipe() {
        let mut stats = CraftingStats::new();

        stats.record_craft(RecipeId::new(1), 1, 0);
        stats.record_craft(RecipeId::new(2), 1, 0);
        stats.record_craft(RecipeId::new(2), 1, 0);
        stats.record_craft(RecipeId::new(3), 1, 0);

        let most_crafted = stats.most_crafted_recipe();
        assert_eq!(most_crafted, Some((2, 2)));
    }

    #[test]
    fn test_event_handler_queue_and_process() {
        let mut handler = CraftingEventHandler::new();

        handler.queue_event(CraftingEvent::CraftCompleted {
            crafter: EntityId::from_raw(1),
            recipe_id: RecipeId::new(1),
            output_item: ItemTypeId::new(100),
            output_quantity: 1,
            skill_gain: 5,
        });

        let result = handler.process_events(None);
        assert_eq!(result.completed_crafts.len(), 1);
        assert_eq!(result.skill_gained, 5);
        assert_eq!(handler.stats().crafts_completed, 1);
    }

    #[test]
    fn test_first_craft_achievement() {
        let mut handler = CraftingEventHandler::new();

        handler.queue_event(CraftingEvent::CraftCompleted {
            crafter: EntityId::from_raw(1),
            recipe_id: RecipeId::new(1),
            output_item: ItemTypeId::new(100),
            output_quantity: 1,
            skill_gain: 0,
        });

        let result = handler.process_events(None);
        assert!(result
            .achievements
            .iter()
            .any(|a| matches!(a, CraftingAchievement::FirstCraft)));
    }

    #[test]
    fn test_craft_start_sounds() {
        assert_eq!(craft_start_sound("weapons"), "craft_metal_start");
        assert_eq!(craft_start_sound("consumables"), "craft_potion_start");
        assert_eq!(craft_start_sound("unknown"), "craft_generic_start");
    }

    #[test]
    fn test_workstation_type_roundtrip() {
        for ws in [
            WorkstationType::CraftingTable,
            WorkstationType::Forge,
            WorkstationType::ChemistryBench,
        ] {
            let id = ws.building_id();
            let recovered = WorkstationType::from_building_id(id);
            assert_eq!(recovered, Some(ws));
        }
    }
}
