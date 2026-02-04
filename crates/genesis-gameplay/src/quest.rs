//! Quest system for tracking missions and objectives.

use crate::npc::NPCType;
use genesis_common::{EntityId, FactionId, ItemTypeId, RecipeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Unique identifier for a quest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestId(pub u32);

impl QuestId {
    /// Creates a new quest ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Unique identifier for unlockables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnlockId(pub u32);

impl UnlockId {
    /// Creates a new unlock ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Error types for quest operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum QuestError {
    /// Quest not found
    #[error("Quest not found: {0:?}")]
    NotFound(QuestId),
    /// Quest already active
    #[error("Quest already active: {0:?}")]
    AlreadyActive(QuestId),
    /// Quest already completed
    #[error("Quest already completed: {0:?}")]
    AlreadyCompleted(QuestId),
    /// Quest not active
    #[error("Quest not active: {0:?}")]
    NotActive(QuestId),
    /// Prerequisites not met
    #[error("Prerequisites not met for quest: {0:?}")]
    PrerequisitesNotMet(QuestId),
    /// Quest not repeatable
    #[error("Quest is not repeatable: {0:?}")]
    NotRepeatable(QuestId),
    /// Quest objectives not complete
    #[error("Quest objectives not complete: {0:?}")]
    ObjectivesIncomplete(QuestId),
}

/// Result type for quest operations.
pub type QuestResult<T> = Result<T, QuestError>;

/// An objective within a quest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestObjective {
    /// Kill a number of enemies of a type
    Kill {
        /// Target NPC type
        target: NPCType,
        /// Required kill count
        count: u32,
    },
    /// Collect items
    Collect {
        /// Item type to collect
        item: ItemTypeId,
        /// Required item count
        count: u32,
    },
    /// Reach a location
    Reach {
        /// Target position (x, y)
        position: (f32, f32),
        /// Radius for completion
        radius: f32,
    },
    /// Talk to a specific NPC
    Talk {
        /// NPC entity ID
        npc_id: EntityId,
    },
    /// Craft items using a recipe
    Craft {
        /// Recipe to craft
        recipe: RecipeId,
        /// Required craft count
        count: u32,
    },
    /// Custom objective with freeform logic
    Custom {
        /// Unique identifier for this objective type
        id: String,
        /// Human-readable description
        description: String,
    },
}

impl QuestObjective {
    /// Creates a kill objective.
    #[must_use]
    pub const fn kill(target: NPCType, count: u32) -> Self {
        Self::Kill { target, count }
    }

    /// Creates a collect objective.
    #[must_use]
    pub const fn collect(item: ItemTypeId, count: u32) -> Self {
        Self::Collect { item, count }
    }

    /// Creates a reach objective.
    #[must_use]
    pub const fn reach(position: (f32, f32), radius: f32) -> Self {
        Self::Reach { position, radius }
    }

    /// Creates a talk objective.
    #[must_use]
    pub const fn talk(npc_id: EntityId) -> Self {
        Self::Talk { npc_id }
    }

    /// Creates a craft objective.
    #[must_use]
    pub const fn craft(recipe: RecipeId, count: u32) -> Self {
        Self::Craft { recipe, count }
    }

    /// Creates a custom objective.
    #[must_use]
    pub fn custom(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self::Custom {
            id: id.into(),
            description: description.into(),
        }
    }

    /// Returns the required count for this objective.
    #[must_use]
    pub const fn required_count(&self) -> u32 {
        match self {
            QuestObjective::Kill { count, .. }
            | QuestObjective::Collect { count, .. }
            | QuestObjective::Craft { count, .. } => *count,
            QuestObjective::Reach { .. }
            | QuestObjective::Talk { .. }
            | QuestObjective::Custom { .. } => 1,
        }
    }

    /// Returns a description of this objective.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            QuestObjective::Kill { target, count } => {
                format!("Kill {count} {target:?}")
            },
            QuestObjective::Collect { item, count } => {
                format!("Collect {count} of item {:?}", item.raw())
            },
            QuestObjective::Reach { position, .. } => {
                format!("Reach location ({:.0}, {:.0})", position.0, position.1)
            },
            QuestObjective::Talk { npc_id } => {
                format!("Talk to NPC {npc_id:?}")
            },
            QuestObjective::Craft { recipe, count } => {
                format!("Craft {count} of recipe {:?}", recipe.raw())
            },
            QuestObjective::Custom { description, .. } => description.clone(),
        }
    }
}

/// A reward given upon quest completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestReward {
    /// Experience points
    Experience(u32),
    /// Item reward
    Item {
        /// Item type ID
        id: ItemTypeId,
        /// Item count
        count: u32,
    },
    /// Currency reward
    Currency(u32),
    /// Reputation with a faction
    Reputation {
        /// Faction ID
        faction: FactionId,
        /// Reputation change (can be negative)
        amount: i32,
    },
    /// Unlock something (recipe, area, etc.)
    Unlock(UnlockId),
}

impl QuestReward {
    /// Creates an experience reward.
    #[must_use]
    pub const fn experience(amount: u32) -> Self {
        Self::Experience(amount)
    }

    /// Creates an item reward.
    #[must_use]
    pub const fn item(id: ItemTypeId, count: u32) -> Self {
        Self::Item { id, count }
    }

    /// Creates a currency reward.
    #[must_use]
    pub const fn currency(amount: u32) -> Self {
        Self::Currency(amount)
    }

    /// Creates a reputation reward.
    #[must_use]
    pub const fn reputation(faction: FactionId, amount: i32) -> Self {
        Self::Reputation { faction, amount }
    }

    /// Creates an unlock reward.
    #[must_use]
    pub const fn unlock(id: UnlockId) -> Self {
        Self::Unlock(id)
    }
}

/// Template for a quest that can be started.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestTemplate {
    /// Unique quest identifier
    pub id: QuestId,
    /// Quest name
    pub name: String,
    /// Quest description
    pub description: String,
    /// Objectives to complete
    pub objectives: Vec<QuestObjective>,
    /// Rewards on completion
    pub rewards: Vec<QuestReward>,
    /// Required quests to be completed first
    pub prerequisites: Vec<QuestId>,
    /// Whether quest can be repeated
    pub repeatable: bool,
}

impl QuestTemplate {
    /// Creates a new quest template.
    #[must_use]
    pub fn new(id: QuestId, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            objectives: Vec::new(),
            rewards: Vec::new(),
            prerequisites: Vec::new(),
            repeatable: false,
        }
    }

    /// Adds an objective.
    #[must_use]
    pub fn with_objective(mut self, objective: QuestObjective) -> Self {
        self.objectives.push(objective);
        self
    }

    /// Adds a reward.
    #[must_use]
    pub fn with_reward(mut self, reward: QuestReward) -> Self {
        self.rewards.push(reward);
        self
    }

    /// Adds a prerequisite quest.
    #[must_use]
    pub fn with_prerequisite(mut self, quest_id: QuestId) -> Self {
        self.prerequisites.push(quest_id);
        self
    }

    /// Sets whether the quest is repeatable.
    #[must_use]
    pub const fn repeatable(mut self, repeatable: bool) -> Self {
        self.repeatable = repeatable;
        self
    }
}

/// Progress tracking for an active quest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    /// Quest ID being tracked
    pub quest_id: QuestId,
    /// Timestamp when quest was started (game time)
    pub started_at: f64,
    /// Progress for each objective (index corresponds to template objectives)
    pub objective_progress: Vec<u32>,
    /// Current stage (for multi-stage quests)
    pub stage: u32,
}

impl QuestProgress {
    /// Creates new quest progress.
    #[must_use]
    pub fn new(quest_id: QuestId, num_objectives: usize, started_at: f64) -> Self {
        Self {
            quest_id,
            started_at,
            objective_progress: vec![0; num_objectives],
            stage: 0,
        }
    }

    /// Returns whether an objective is complete.
    #[must_use]
    pub fn is_objective_complete(&self, index: usize, template: &QuestTemplate) -> bool {
        if let (Some(&progress), Some(objective)) = (
            self.objective_progress.get(index),
            template.objectives.get(index),
        ) {
            progress >= objective.required_count()
        } else {
            false
        }
    }

    /// Returns whether all objectives are complete.
    #[must_use]
    pub fn all_objectives_complete(&self, template: &QuestTemplate) -> bool {
        for (i, objective) in template.objectives.iter().enumerate() {
            if let Some(&progress) = self.objective_progress.get(i) {
                if progress < objective.required_count() {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Returns progress for an objective as (current, required).
    #[must_use]
    pub fn objective_status(&self, index: usize, template: &QuestTemplate) -> Option<(u32, u32)> {
        let progress = self.objective_progress.get(index)?;
        let objective = template.objectives.get(index)?;
        Some((*progress, objective.required_count()))
    }
}

/// Quest manager handling all quest operations.
#[derive(Debug, Default)]
pub struct QuestManager {
    /// Available quest templates
    available_quests: HashMap<QuestId, QuestTemplate>,
    /// Currently active quests
    active_quests: HashMap<QuestId, QuestProgress>,
    /// Completed quests
    completed_quests: HashSet<QuestId>,
    /// Current game time (for tracking quest start time)
    game_time: f64,
}

impl QuestManager {
    /// Creates a new quest manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the current game time.
    pub fn set_game_time(&mut self, time: f64) {
        self.game_time = time;
    }

    /// Returns the current game time.
    #[must_use]
    pub const fn game_time(&self) -> f64 {
        self.game_time
    }

    /// Registers a quest template.
    pub fn register_quest(&mut self, template: QuestTemplate) {
        self.available_quests.insert(template.id, template);
    }

    /// Returns the number of available quests.
    #[must_use]
    pub fn available_count(&self) -> usize {
        self.available_quests.len()
    }

    /// Returns the number of active quests.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active_quests.len()
    }

    /// Returns the number of completed quests.
    #[must_use]
    pub fn completed_count(&self) -> usize {
        self.completed_quests.len()
    }

    /// Gets a quest template.
    #[must_use]
    pub fn get_template(&self, quest_id: QuestId) -> Option<&QuestTemplate> {
        self.available_quests.get(&quest_id)
    }

    /// Gets active quest progress.
    #[must_use]
    pub fn get_progress(&self, quest_id: QuestId) -> Option<&QuestProgress> {
        self.active_quests.get(&quest_id)
    }

    /// Checks if a quest is available to start.
    #[must_use]
    pub fn is_available(&self, quest_id: QuestId) -> bool {
        if let Some(template) = self.available_quests.get(&quest_id) {
            // Check if already active
            if self.active_quests.contains_key(&quest_id) {
                return false;
            }

            // Check if already completed (and not repeatable)
            if self.completed_quests.contains(&quest_id) && !template.repeatable {
                return false;
            }

            // Check prerequisites
            for &prereq in &template.prerequisites {
                if !self.completed_quests.contains(&prereq) {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Checks if a quest is active.
    #[must_use]
    pub fn is_active(&self, quest_id: QuestId) -> bool {
        self.active_quests.contains_key(&quest_id)
    }

    /// Checks if a quest is completed.
    #[must_use]
    pub fn is_completed(&self, quest_id: QuestId) -> bool {
        self.completed_quests.contains(&quest_id)
    }

    /// Starts a quest.
    pub fn start_quest(&mut self, quest_id: QuestId) -> QuestResult<()> {
        let template = self
            .available_quests
            .get(&quest_id)
            .ok_or(QuestError::NotFound(quest_id))?;

        // Check if already active
        if self.active_quests.contains_key(&quest_id) {
            return Err(QuestError::AlreadyActive(quest_id));
        }

        // Check if completed and not repeatable
        if self.completed_quests.contains(&quest_id) && !template.repeatable {
            return Err(QuestError::NotRepeatable(quest_id));
        }

        // Check prerequisites
        for &prereq in &template.prerequisites {
            if !self.completed_quests.contains(&prereq) {
                return Err(QuestError::PrerequisitesNotMet(quest_id));
            }
        }

        // Create progress
        let progress = QuestProgress::new(quest_id, template.objectives.len(), self.game_time);
        self.active_quests.insert(quest_id, progress);

        Ok(())
    }

    /// Abandons a quest.
    pub fn abandon_quest(&mut self, quest_id: QuestId) {
        self.active_quests.remove(&quest_id);
    }

    /// Completes a quest and returns rewards.
    pub fn complete_quest(&mut self, quest_id: QuestId) -> QuestResult<Vec<QuestReward>> {
        let template = self
            .available_quests
            .get(&quest_id)
            .ok_or(QuestError::NotFound(quest_id))?;

        let progress = self
            .active_quests
            .get(&quest_id)
            .ok_or(QuestError::NotActive(quest_id))?;

        // Check if all objectives complete
        if !progress.all_objectives_complete(template) {
            return Err(QuestError::ObjectivesIncomplete(quest_id));
        }

        let rewards = template.rewards.clone();

        // Remove from active, add to completed
        self.active_quests.remove(&quest_id);
        self.completed_quests.insert(quest_id);

        Ok(rewards)
    }

    /// Called when an enemy is killed.
    pub fn on_enemy_killed(&mut self, enemy_type: NPCType) {
        let quest_ids: Vec<QuestId> = self.active_quests.keys().copied().collect();

        for quest_id in quest_ids {
            if let Some(template) = self.available_quests.get(&quest_id) {
                let template_clone = template.clone();
                if let Some(progress) = self.active_quests.get_mut(&quest_id) {
                    for (i, objective) in template_clone.objectives.iter().enumerate() {
                        if let QuestObjective::Kill { target, count } = objective {
                            if *target == enemy_type {
                                if let Some(prog) = progress.objective_progress.get_mut(i) {
                                    *prog = (*prog + 1).min(*count);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Called when an item is collected.
    pub fn on_item_collected(&mut self, item: ItemTypeId, count: u32) {
        let quest_ids: Vec<QuestId> = self.active_quests.keys().copied().collect();

        for quest_id in quest_ids {
            if let Some(template) = self.available_quests.get(&quest_id) {
                let template_clone = template.clone();
                if let Some(progress) = self.active_quests.get_mut(&quest_id) {
                    for (i, objective) in template_clone.objectives.iter().enumerate() {
                        if let QuestObjective::Collect {
                            item: target_item,
                            count: required,
                        } = objective
                        {
                            if *target_item == item {
                                if let Some(prog) = progress.objective_progress.get_mut(i) {
                                    *prog = (*prog + count).min(*required);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Called when a position is reached.
    pub fn on_position_reached(&mut self, position: (f32, f32)) {
        let quest_ids: Vec<QuestId> = self.active_quests.keys().copied().collect();

        for quest_id in quest_ids {
            if let Some(template) = self.available_quests.get(&quest_id) {
                let template_clone = template.clone();
                if let Some(progress) = self.active_quests.get_mut(&quest_id) {
                    for (i, objective) in template_clone.objectives.iter().enumerate() {
                        if let QuestObjective::Reach {
                            position: target,
                            radius,
                        } = objective
                        {
                            let dx = position.0 - target.0;
                            let dy = position.1 - target.1;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist <= *radius {
                                if let Some(prog) = progress.objective_progress.get_mut(i) {
                                    *prog = 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Called when talking to an NPC.
    pub fn on_npc_talked(&mut self, npc_id: EntityId) {
        let quest_ids: Vec<QuestId> = self.active_quests.keys().copied().collect();

        for quest_id in quest_ids {
            if let Some(template) = self.available_quests.get(&quest_id) {
                let template_clone = template.clone();
                if let Some(progress) = self.active_quests.get_mut(&quest_id) {
                    for (i, objective) in template_clone.objectives.iter().enumerate() {
                        if let QuestObjective::Talk { npc_id: target_npc } = objective {
                            if *target_npc == npc_id {
                                if let Some(prog) = progress.objective_progress.get_mut(i) {
                                    *prog = 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Called when crafting an item.
    pub fn on_item_crafted(&mut self, recipe: RecipeId, count: u32) {
        let quest_ids: Vec<QuestId> = self.active_quests.keys().copied().collect();

        for quest_id in quest_ids {
            if let Some(template) = self.available_quests.get(&quest_id) {
                let template_clone = template.clone();
                if let Some(progress) = self.active_quests.get_mut(&quest_id) {
                    for (i, objective) in template_clone.objectives.iter().enumerate() {
                        if let QuestObjective::Craft {
                            recipe: target_recipe,
                            count: required,
                        } = objective
                        {
                            if *target_recipe == recipe {
                                if let Some(prog) = progress.objective_progress.get_mut(i) {
                                    *prog = (*prog + count).min(*required);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Called when a custom objective is completed.
    pub fn on_custom_objective(&mut self, objective_id: &str) {
        let quest_ids: Vec<QuestId> = self.active_quests.keys().copied().collect();

        for quest_id in quest_ids {
            if let Some(template) = self.available_quests.get(&quest_id) {
                let template_clone = template.clone();
                if let Some(progress) = self.active_quests.get_mut(&quest_id) {
                    for (i, objective) in template_clone.objectives.iter().enumerate() {
                        if let QuestObjective::Custom { id, .. } = objective {
                            if id == objective_id {
                                if let Some(prog) = progress.objective_progress.get_mut(i) {
                                    *prog = 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Returns all active quests.
    #[must_use]
    pub fn get_active_quests(&self) -> Vec<&QuestProgress> {
        self.active_quests.values().collect()
    }

    /// Returns all available quest IDs.
    #[must_use]
    pub fn get_available_quest_ids(&self) -> Vec<QuestId> {
        self.available_quests
            .keys()
            .filter(|&&id| self.is_available(id))
            .copied()
            .collect()
    }

    /// Returns whether an objective is complete.
    #[must_use]
    pub fn is_objective_complete(&self, quest_id: QuestId, objective: usize) -> bool {
        if let (Some(progress), Some(template)) = (
            self.active_quests.get(&quest_id),
            self.available_quests.get(&quest_id),
        ) {
            progress.is_objective_complete(objective, template)
        } else {
            false
        }
    }

    /// Gets quest data for UI display.
    #[must_use]
    pub fn get_quest_ui_data(&self, quest_id: QuestId) -> Option<QuestUIData> {
        let template = self.available_quests.get(&quest_id)?;
        let progress = self.active_quests.get(&quest_id);

        let objectives: Vec<ObjectiveUIData> = template
            .objectives
            .iter()
            .enumerate()
            .map(|(i, obj)| {
                let (current, required) = progress
                    .and_then(|p| p.objective_status(i, template))
                    .unwrap_or((0, obj.required_count()));
                ObjectiveUIData {
                    description: obj.description(),
                    current,
                    required,
                    complete: current >= required,
                }
            })
            .collect();

        Some(QuestUIData {
            id: quest_id,
            name: template.name.clone(),
            description: template.description.clone(),
            objectives,
            is_active: self.is_active(quest_id),
            is_complete: progress.is_some_and(|p| p.all_objectives_complete(template)),
        })
    }

    /// Returns iterator over all quest templates.
    pub fn iter_templates(&self) -> impl Iterator<Item = (&QuestId, &QuestTemplate)> {
        self.available_quests.iter()
    }

    /// Returns iterator over all active progress.
    pub fn iter_active(&self) -> impl Iterator<Item = (&QuestId, &QuestProgress)> {
        self.active_quests.iter()
    }
}

/// Quest data formatted for UI display.
#[derive(Debug, Clone)]
pub struct QuestUIData {
    /// Quest ID
    pub id: QuestId,
    /// Quest name
    pub name: String,
    /// Quest description
    pub description: String,
    /// Objective status list
    pub objectives: Vec<ObjectiveUIData>,
    /// Whether quest is currently active
    pub is_active: bool,
    /// Whether all objectives are complete
    pub is_complete: bool,
}

/// Objective data formatted for UI display.
#[derive(Debug, Clone)]
pub struct ObjectiveUIData {
    /// Objective description
    pub description: String,
    /// Current progress
    pub current: u32,
    /// Required for completion
    pub required: u32,
    /// Whether objective is complete
    pub complete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_id() {
        let id = QuestId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_recipe_id() {
        let id = RecipeId::new(10);
        assert_eq!(id.raw(), 10);
    }

    #[test]
    fn test_unlock_id() {
        let id = UnlockId::new(5);
        assert_eq!(id.0, 5);
    }

    #[test]
    fn test_quest_objective_kill() {
        let obj = QuestObjective::kill(NPCType::Hostile, 10);
        assert_eq!(obj.required_count(), 10);
        assert!(obj.description().contains("10"));
    }

    #[test]
    fn test_quest_objective_collect() {
        let obj = QuestObjective::collect(ItemTypeId::new(1), 5);
        assert_eq!(obj.required_count(), 5);
    }

    #[test]
    fn test_quest_objective_reach() {
        let obj = QuestObjective::reach((100.0, 200.0), 10.0);
        assert_eq!(obj.required_count(), 1);
        assert!(obj.description().contains("100"));
    }

    #[test]
    fn test_quest_objective_talk() {
        let npc = EntityId::new();
        let obj = QuestObjective::talk(npc);
        assert_eq!(obj.required_count(), 1);
    }

    #[test]
    fn test_quest_objective_craft() {
        let obj = QuestObjective::craft(RecipeId::new(1), 3);
        assert_eq!(obj.required_count(), 3);
    }

    #[test]
    fn test_quest_objective_custom() {
        let obj = QuestObjective::custom("my_obj", "Do something special");
        assert_eq!(obj.required_count(), 1);
        assert!(obj.description().contains("special"));
    }

    #[test]
    fn test_quest_reward_experience() {
        let reward = QuestReward::experience(100);
        match reward {
            QuestReward::Experience(xp) => assert_eq!(xp, 100),
            _ => panic!("Expected Experience reward"),
        }
    }

    #[test]
    fn test_quest_reward_item() {
        let reward = QuestReward::item(ItemTypeId::new(5), 3);
        match reward {
            QuestReward::Item { id, count } => {
                assert_eq!(id.raw(), 5);
                assert_eq!(count, 3);
            },
            _ => panic!("Expected Item reward"),
        }
    }

    #[test]
    fn test_quest_reward_currency() {
        let reward = QuestReward::currency(500);
        match reward {
            QuestReward::Currency(amount) => assert_eq!(amount, 500),
            _ => panic!("Expected Currency reward"),
        }
    }

    #[test]
    fn test_quest_reward_reputation() {
        let reward = QuestReward::reputation(FactionId::new(1), 50);
        match reward {
            QuestReward::Reputation { faction, amount } => {
                assert_eq!(faction.raw(), 1);
                assert_eq!(amount, 50);
            },
            _ => panic!("Expected Reputation reward"),
        }
    }

    #[test]
    fn test_quest_reward_unlock() {
        let reward = QuestReward::unlock(UnlockId::new(1));
        match reward {
            QuestReward::Unlock(id) => assert_eq!(id.0, 1),
            _ => panic!("Expected Unlock reward"),
        }
    }

    #[test]
    fn test_quest_template_builder() {
        let template = QuestTemplate::new(QuestId::new(1), "Test Quest", "A test quest")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 5))
            .with_objective(QuestObjective::collect(ItemTypeId::new(1), 3))
            .with_reward(QuestReward::experience(100))
            .with_reward(QuestReward::currency(50))
            .with_prerequisite(QuestId::new(0))
            .repeatable(true);

        assert_eq!(template.id.0, 1);
        assert_eq!(template.name, "Test Quest");
        assert_eq!(template.objectives.len(), 2);
        assert_eq!(template.rewards.len(), 2);
        assert_eq!(template.prerequisites.len(), 1);
        assert!(template.repeatable);
    }

    #[test]
    fn test_quest_progress_creation() {
        let progress = QuestProgress::new(QuestId::new(1), 3, 100.0);
        assert_eq!(progress.quest_id.0, 1);
        assert_eq!(progress.started_at, 100.0);
        assert_eq!(progress.objective_progress.len(), 3);
        assert_eq!(progress.stage, 0);
    }

    #[test]
    fn test_quest_progress_objective_complete() {
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 5));

        let mut progress = QuestProgress::new(QuestId::new(1), 1, 0.0);
        assert!(!progress.is_objective_complete(0, &template));

        progress.objective_progress[0] = 5;
        assert!(progress.is_objective_complete(0, &template));
    }

    #[test]
    fn test_quest_progress_all_complete() {
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 5))
            .with_objective(QuestObjective::collect(ItemTypeId::new(1), 3));

        let mut progress = QuestProgress::new(QuestId::new(1), 2, 0.0);
        assert!(!progress.all_objectives_complete(&template));

        progress.objective_progress[0] = 5;
        assert!(!progress.all_objectives_complete(&template));

        progress.objective_progress[1] = 3;
        assert!(progress.all_objectives_complete(&template));
    }

    #[test]
    fn test_quest_progress_objective_status() {
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 10));

        let mut progress = QuestProgress::new(QuestId::new(1), 1, 0.0);
        progress.objective_progress[0] = 7;

        let status = progress.objective_status(0, &template);
        assert_eq!(status, Some((7, 10)));
    }

    #[test]
    fn test_quest_manager_creation() {
        let manager = QuestManager::new();
        assert_eq!(manager.available_count(), 0);
        assert_eq!(manager.active_count(), 0);
        assert_eq!(manager.completed_count(), 0);
    }

    #[test]
    fn test_quest_manager_register() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test");
        manager.register_quest(template);

        assert_eq!(manager.available_count(), 1);
        assert!(manager.get_template(QuestId::new(1)).is_some());
    }

    #[test]
    fn test_quest_manager_start() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 5));
        manager.register_quest(template);

        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");
        assert!(manager.is_active(QuestId::new(1)));
        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_quest_manager_start_not_found() {
        let mut manager = QuestManager::new();
        let result = manager.start_quest(QuestId::new(99));
        assert!(matches!(result, Err(QuestError::NotFound(_))));
    }

    #[test]
    fn test_quest_manager_start_already_active() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test");
        manager.register_quest(template);

        manager
            .start_quest(QuestId::new(1))
            .expect("First start should succeed");
        let result = manager.start_quest(QuestId::new(1));
        assert!(matches!(result, Err(QuestError::AlreadyActive(_))));
    }

    #[test]
    fn test_quest_manager_prerequisites() {
        let mut manager = QuestManager::new();

        let prereq = QuestTemplate::new(QuestId::new(0), "Prereq", "Prereq");
        let quest =
            QuestTemplate::new(QuestId::new(1), "Test", "Test").with_prerequisite(QuestId::new(0));

        manager.register_quest(prereq);
        manager.register_quest(quest);

        // Can't start without prereq
        let result = manager.start_quest(QuestId::new(1));
        assert!(matches!(result, Err(QuestError::PrerequisitesNotMet(_))));

        // Complete prereq
        manager
            .start_quest(QuestId::new(0))
            .expect("Prereq start should succeed");
        manager.active_quests.get_mut(&QuestId::new(0)); // Direct manipulation for test
        manager.active_quests.remove(&QuestId::new(0));
        manager.completed_quests.insert(QuestId::new(0));

        // Now can start
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");
    }

    #[test]
    fn test_quest_manager_abandon() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test");
        manager.register_quest(template);

        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");
        assert!(manager.is_active(QuestId::new(1)));

        manager.abandon_quest(QuestId::new(1));
        assert!(!manager.is_active(QuestId::new(1)));
    }

    #[test]
    fn test_quest_manager_complete() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 2))
            .with_reward(QuestReward::experience(100));
        manager.register_quest(template);

        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        // Not complete yet
        let result = manager.complete_quest(QuestId::new(1));
        assert!(matches!(result, Err(QuestError::ObjectivesIncomplete(_))));

        // Complete objective
        manager.on_enemy_killed(NPCType::Hostile);
        manager.on_enemy_killed(NPCType::Hostile);

        let rewards = manager
            .complete_quest(QuestId::new(1))
            .expect("Complete should succeed");
        assert_eq!(rewards.len(), 1);
        assert!(manager.is_completed(QuestId::new(1)));
        assert!(!manager.is_active(QuestId::new(1)));
    }

    #[test]
    fn test_quest_manager_repeatable() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test").repeatable(true);
        manager.register_quest(template);

        // First time
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");
        manager.active_quests.remove(&QuestId::new(1));
        manager.completed_quests.insert(QuestId::new(1));

        // Can start again
        manager
            .start_quest(QuestId::new(1))
            .expect("Repeat should succeed");
    }

    #[test]
    fn test_quest_manager_not_repeatable() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test").repeatable(false);
        manager.register_quest(template);

        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");
        manager.active_quests.remove(&QuestId::new(1));
        manager.completed_quests.insert(QuestId::new(1));

        let result = manager.start_quest(QuestId::new(1));
        assert!(matches!(result, Err(QuestError::NotRepeatable(_))));
    }

    #[test]
    fn test_quest_manager_on_enemy_killed() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 3));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        manager.on_enemy_killed(NPCType::Hostile);
        manager.on_enemy_killed(NPCType::Passive); // Wrong type
        manager.on_enemy_killed(NPCType::Hostile);

        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 2);
    }

    #[test]
    fn test_quest_manager_on_item_collected() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::collect(ItemTypeId::new(5), 10));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        manager.on_item_collected(ItemTypeId::new(5), 3);
        manager.on_item_collected(ItemTypeId::new(5), 5);

        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 8);
    }

    #[test]
    fn test_quest_manager_on_position_reached() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::reach((100.0, 100.0), 10.0));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        manager.on_position_reached((50.0, 50.0)); // Too far
        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 0);

        manager.on_position_reached((105.0, 100.0)); // Within radius
        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 1);
    }

    #[test]
    fn test_quest_manager_on_npc_talked() {
        let npc = EntityId::new();
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::talk(npc));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        manager.on_npc_talked(EntityId::new()); // Wrong NPC
        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 0);

        manager.on_npc_talked(npc);
        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 1);
    }

    #[test]
    fn test_quest_manager_on_item_crafted() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::craft(RecipeId::new(1), 5));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        manager.on_item_crafted(RecipeId::new(1), 2);
        manager.on_item_crafted(RecipeId::new(2), 3); // Wrong recipe

        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 2);
    }

    #[test]
    fn test_quest_manager_on_custom_objective() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::custom("test_obj", "Do the thing"));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        manager.on_custom_objective("wrong_obj");
        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 0);

        manager.on_custom_objective("test_obj");
        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 1);
    }

    #[test]
    fn test_quest_manager_get_active_quests() {
        let mut manager = QuestManager::new();
        manager.register_quest(QuestTemplate::new(QuestId::new(1), "Test1", "Test1"));
        manager.register_quest(QuestTemplate::new(QuestId::new(2), "Test2", "Test2"));

        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");
        manager
            .start_quest(QuestId::new(2))
            .expect("Start should succeed");

        let active = manager.get_active_quests();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_quest_manager_get_available_quest_ids() {
        let mut manager = QuestManager::new();
        manager.register_quest(QuestTemplate::new(QuestId::new(1), "Test1", "Test1"));
        manager.register_quest(QuestTemplate::new(QuestId::new(2), "Test2", "Test2"));

        let available = manager.get_available_quest_ids();
        assert_eq!(available.len(), 2);

        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        let available = manager.get_available_quest_ids();
        assert_eq!(available.len(), 1);
    }

    #[test]
    fn test_quest_manager_is_objective_complete() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 2));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        assert!(!manager.is_objective_complete(QuestId::new(1), 0));

        manager.on_enemy_killed(NPCType::Hostile);
        manager.on_enemy_killed(NPCType::Hostile);

        assert!(manager.is_objective_complete(QuestId::new(1), 0));
    }

    #[test]
    fn test_quest_manager_game_time() {
        let mut manager = QuestManager::new();
        assert_eq!(manager.game_time(), 0.0);

        manager.set_game_time(100.0);
        assert_eq!(manager.game_time(), 100.0);
    }

    #[test]
    fn test_quest_ui_data() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test Quest", "A test quest")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 5));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        manager.on_enemy_killed(NPCType::Hostile);
        manager.on_enemy_killed(NPCType::Hostile);

        let ui_data = manager
            .get_quest_ui_data(QuestId::new(1))
            .expect("UI data should exist");
        assert_eq!(ui_data.name, "Test Quest");
        assert!(ui_data.is_active);
        assert!(!ui_data.is_complete);
        assert_eq!(ui_data.objectives.len(), 1);
        assert_eq!(ui_data.objectives[0].current, 2);
        assert_eq!(ui_data.objectives[0].required, 5);
    }

    #[test]
    fn test_quest_error_variants() {
        let _not_found = QuestError::NotFound(QuestId::new(1));
        let _already_active = QuestError::AlreadyActive(QuestId::new(1));
        let _already_completed = QuestError::AlreadyCompleted(QuestId::new(1));
        let _not_active = QuestError::NotActive(QuestId::new(1));
        let _prereq = QuestError::PrerequisitesNotMet(QuestId::new(1));
        let _not_repeatable = QuestError::NotRepeatable(QuestId::new(1));
        let _incomplete = QuestError::ObjectivesIncomplete(QuestId::new(1));

        // Test error messages
        assert!(_not_found.to_string().contains("not found"));
    }

    #[test]
    fn test_iter_templates() {
        let mut manager = QuestManager::new();
        manager.register_quest(QuestTemplate::new(QuestId::new(1), "Test1", "Test1"));
        manager.register_quest(QuestTemplate::new(QuestId::new(2), "Test2", "Test2"));

        let templates: Vec<_> = manager.iter_templates().collect();
        assert_eq!(templates.len(), 2);
    }

    #[test]
    fn test_iter_active() {
        let mut manager = QuestManager::new();
        manager.register_quest(QuestTemplate::new(QuestId::new(1), "Test1", "Test1"));
        manager.register_quest(QuestTemplate::new(QuestId::new(2), "Test2", "Test2"));

        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        let active: Vec<_> = manager.iter_active().collect();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_quest_progress_capped() {
        let mut manager = QuestManager::new();
        let template = QuestTemplate::new(QuestId::new(1), "Test", "Test")
            .with_objective(QuestObjective::kill(NPCType::Hostile, 3));
        manager.register_quest(template);
        manager
            .start_quest(QuestId::new(1))
            .expect("Start should succeed");

        // Kill more than required
        for _ in 0..10 {
            manager.on_enemy_killed(NPCType::Hostile);
        }

        let progress = manager
            .get_progress(QuestId::new(1))
            .expect("Progress should exist");
        assert_eq!(progress.objective_progress[0], 3); // Capped at required
    }
}
