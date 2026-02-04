//! Crafting progression system.
//!
//! This module provides:
//! - Learned recipe tracking per player
//! - Recipe discovery through experimentation
//! - Skill-gated recipe unlocks
//! - Quest and NPC recipe unlocks

use genesis_common::{EntityId, RecipeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// G-48: Recipe Discovery Sources
// ============================================================================

/// How a recipe was discovered/unlocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoverySource {
    /// Available from the start.
    Default,
    /// Discovered through experimentation.
    Experimentation,
    /// Unlocked by reaching a skill level.
    SkillLevel {
        /// Skill type.
        skill: String,
        /// Required level.
        level: u32,
    },
    /// Learned from an NPC.
    NPC {
        /// NPC identifier.
        npc_id: EntityId,
        /// NPC name for display.
        npc_name: String,
    },
    /// Unlocked by completing a quest.
    Quest {
        /// Quest identifier.
        quest_id: u32,
        /// Quest name for display.
        quest_name: String,
    },
    /// Purchased from a shop.
    Purchased {
        /// Cost in currency.
        cost: u32,
    },
    /// Found in the world (loot, treasure).
    WorldDrop {
        /// Location description.
        location: String,
    },
    /// Achievement reward.
    Achievement {
        /// Achievement identifier.
        achievement_id: u32,
    },
    /// Admin/debug unlock.
    Admin,
}

impl DiscoverySource {
    /// Get display text for the source.
    #[must_use]
    pub fn display_text(&self) -> String {
        match self {
            Self::Default => "Known by default".to_string(),
            Self::Experimentation => "Discovered through experimentation".to_string(),
            Self::SkillLevel { skill, level } => format!("Unlocked at {skill} level {level}"),
            Self::NPC { npc_name, .. } => format!("Learned from {npc_name}"),
            Self::Quest { quest_name, .. } => format!("Quest reward: {quest_name}"),
            Self::Purchased { cost } => format!("Purchased for {cost} gold"),
            Self::WorldDrop { location } => format!("Found in {location}"),
            Self::Achievement { achievement_id } => format!("Achievement #{achievement_id}"),
            Self::Admin => "Debug unlock".to_string(),
        }
    }
}

// ============================================================================
// G-48: Learned Recipe Entry
// ============================================================================

/// Information about a learned recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedRecipe {
    /// Recipe identifier.
    pub recipe_id: RecipeId,
    /// When the recipe was learned (game time).
    pub learned_at: u64,
    /// How it was discovered.
    pub source: DiscoverySource,
    /// Number of times crafted.
    pub times_crafted: u32,
    /// Whether marked as favorite.
    pub is_favorite: bool,
    /// Custom notes from player.
    pub notes: String,
}

impl LearnedRecipe {
    /// Create a new learned recipe entry.
    #[must_use]
    pub fn new(recipe_id: RecipeId, source: DiscoverySource, game_time: u64) -> Self {
        Self {
            recipe_id,
            learned_at: game_time,
            source,
            times_crafted: 0,
            is_favorite: false,
            notes: String::new(),
        }
    }

    /// Increment craft counter.
    pub fn record_craft(&mut self) {
        self.times_crafted += 1;
    }

    /// Toggle favorite status.
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
    }

    /// Set notes.
    pub fn set_notes(&mut self, notes: impl Into<String>) {
        self.notes = notes.into();
    }
}

// ============================================================================
// G-48: Learned Recipes Collection
// ============================================================================

/// Collection of all recipes a player has learned.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearnedRecipes {
    /// All learned recipes.
    recipes: HashMap<RecipeId, LearnedRecipe>,
    /// Recipes pending discovery (partially matched).
    pending_discovery: HashSet<RecipeId>,
    /// Discovery hints (show partial info).
    discovery_hints: HashMap<RecipeId, DiscoveryHint>,
}

impl LearnedRecipes {
    /// Create new empty collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Learn a recipe.
    pub fn learn(&mut self, recipe_id: RecipeId, source: DiscoverySource, game_time: u64) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.recipes.entry(recipe_id) {
            e.insert(LearnedRecipe::new(recipe_id, source, game_time));
            self.pending_discovery.remove(&recipe_id);
            self.discovery_hints.remove(&recipe_id);
        }
    }

    /// Check if a recipe is learned.
    #[must_use]
    pub fn is_learned(&self, recipe_id: RecipeId) -> bool {
        self.recipes.contains_key(&recipe_id)
    }

    /// Get learned recipe info.
    #[must_use]
    pub fn get(&self, recipe_id: RecipeId) -> Option<&LearnedRecipe> {
        self.recipes.get(&recipe_id)
    }

    /// Get mutable learned recipe info.
    pub fn get_mut(&mut self, recipe_id: RecipeId) -> Option<&mut LearnedRecipe> {
        self.recipes.get_mut(&recipe_id)
    }

    /// Record a craft (updates times_crafted).
    pub fn record_craft(&mut self, recipe_id: RecipeId) {
        if let Some(learned) = self.recipes.get_mut(&recipe_id) {
            learned.record_craft();
        }
    }

    /// Get all learned recipe IDs.
    #[must_use]
    pub fn learned_ids(&self) -> Vec<RecipeId> {
        self.recipes.keys().copied().collect()
    }

    /// Get all learned recipes.
    pub fn all(&self) -> impl Iterator<Item = &LearnedRecipe> {
        self.recipes.values()
    }

    /// Get learned recipe count.
    #[must_use]
    pub fn count(&self) -> usize {
        self.recipes.len()
    }

    /// Get favorite recipes.
    #[must_use]
    pub fn favorites(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().filter(|r| r.is_favorite).collect()
    }

    /// Add a discovery hint for a recipe.
    pub fn add_hint(&mut self, recipe_id: RecipeId, hint: DiscoveryHint) {
        if !self.is_learned(recipe_id) {
            self.discovery_hints.insert(recipe_id, hint);
        }
    }

    /// Get discovery hint for a recipe.
    #[must_use]
    pub fn get_hint(&self, recipe_id: RecipeId) -> Option<&DiscoveryHint> {
        self.discovery_hints.get(&recipe_id)
    }

    /// Mark recipe as pending discovery (close to discovering).
    pub fn mark_pending(&mut self, recipe_id: RecipeId) {
        if !self.is_learned(recipe_id) {
            self.pending_discovery.insert(recipe_id);
        }
    }

    /// Check if recipe is pending discovery.
    #[must_use]
    pub fn is_pending(&self, recipe_id: RecipeId) -> bool {
        self.pending_discovery.contains(&recipe_id)
    }

    /// Get pending discoveries.
    #[must_use]
    pub fn pending(&self) -> &HashSet<RecipeId> {
        &self.pending_discovery
    }

    /// Unlearn a recipe (for admin/debug).
    pub fn unlearn(&mut self, recipe_id: RecipeId) -> bool {
        self.recipes.remove(&recipe_id).is_some()
    }
}

/// Hint about an undiscovered recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryHint {
    /// Recipe identifier.
    pub recipe_id: RecipeId,
    /// Hint text.
    pub hint_text: String,
    /// Number of known ingredients.
    pub known_ingredients: u32,
    /// Total ingredients.
    pub total_ingredients: u32,
    /// Whether output is known.
    pub output_known: bool,
}

impl DiscoveryHint {
    /// Create new hint.
    #[must_use]
    pub fn new(recipe_id: RecipeId, hint: impl Into<String>) -> Self {
        Self {
            recipe_id,
            hint_text: hint.into(),
            known_ingredients: 0,
            total_ingredients: 0,
            output_known: false,
        }
    }

    /// Set ingredient progress.
    #[must_use]
    pub fn with_ingredient_progress(mut self, known: u32, total: u32) -> Self {
        self.known_ingredients = known;
        self.total_ingredients = total;
        self
    }

    /// Set whether output is known.
    #[must_use]
    pub fn with_output_known(mut self, known: bool) -> Self {
        self.output_known = known;
        self
    }

    /// Get discovery progress percentage.
    #[must_use]
    pub fn progress_percent(&self) -> f32 {
        if self.total_ingredients == 0 {
            return 0.0;
        }
        (self.known_ingredients as f32 / self.total_ingredients as f32) * 100.0
    }
}

// ============================================================================
// G-48: Skill Requirements
// ============================================================================

/// Skill requirement for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRequirement {
    /// Skill type name.
    pub skill_type: String,
    /// Minimum level required.
    pub min_level: u32,
}

impl SkillRequirement {
    /// Create new requirement.
    #[must_use]
    pub fn new(skill_type: impl Into<String>, min_level: u32) -> Self {
        Self {
            skill_type: skill_type.into(),
            min_level,
        }
    }

    /// Check if requirement is met.
    #[must_use]
    pub fn is_met(&self, player_skills: &PlayerSkills) -> bool {
        player_skills.get_level(&self.skill_type) >= self.min_level
    }
}

/// Player skill levels for crafting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerSkills {
    /// Skill levels by type.
    skills: HashMap<String, u32>,
    /// Skill experience by type.
    experience: HashMap<String, u32>,
}

impl PlayerSkills {
    /// Create new skill tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get skill level.
    #[must_use]
    pub fn get_level(&self, skill: &str) -> u32 {
        self.skills.get(skill).copied().unwrap_or(0)
    }

    /// Set skill level.
    pub fn set_level(&mut self, skill: impl Into<String>, level: u32) {
        self.skills.insert(skill.into(), level);
    }

    /// Add experience and potentially level up.
    /// Returns new level if leveled up.
    pub fn add_experience(&mut self, skill: impl Into<String>, xp: u32) -> Option<u32> {
        let skill = skill.into();
        let current_xp = self.experience.entry(skill.clone()).or_insert(0);
        *current_xp += xp;

        // Simple leveling: 100 XP per level
        let new_level = *current_xp / 100;
        let current_level = self.skills.get(&skill).copied().unwrap_or(0);

        if new_level > current_level {
            self.skills.insert(skill, new_level);
            Some(new_level)
        } else {
            None
        }
    }

    /// Get experience for a skill.
    #[must_use]
    pub fn get_experience(&self, skill: &str) -> u32 {
        self.experience.get(skill).copied().unwrap_or(0)
    }

    /// Get experience progress to next level (0.0-1.0).
    #[must_use]
    pub fn level_progress(&self, skill: &str) -> f32 {
        let xp = self.get_experience(skill);
        (xp % 100) as f32 / 100.0
    }
}

// ============================================================================
// G-48: Recipe Unlock Conditions
// ============================================================================

/// Conditions for unlocking a recipe.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnlockConditions {
    /// Required skill levels.
    pub skill_requirements: Vec<SkillRequirement>,
    /// Required completed quests.
    pub required_quests: HashSet<u32>,
    /// Required learned recipes.
    pub required_recipes: HashSet<RecipeId>,
    /// Required achievements.
    pub required_achievements: HashSet<u32>,
    /// Minimum player level.
    pub min_player_level: u32,
}

impl UnlockConditions {
    /// Create empty conditions (always unlockable).
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Add skill requirement.
    #[must_use]
    pub fn with_skill(mut self, skill: impl Into<String>, level: u32) -> Self {
        self.skill_requirements
            .push(SkillRequirement::new(skill, level));
        self
    }

    /// Add required quest.
    #[must_use]
    pub fn with_quest(mut self, quest_id: u32) -> Self {
        self.required_quests.insert(quest_id);
        self
    }

    /// Add required recipe.
    #[must_use]
    pub fn with_recipe(mut self, recipe_id: RecipeId) -> Self {
        self.required_recipes.insert(recipe_id);
        self
    }

    /// Set min player level.
    #[must_use]
    pub fn with_min_level(mut self, level: u32) -> Self {
        self.min_player_level = level;
        self
    }

    /// Check if all conditions are met.
    #[must_use]
    pub fn are_met(&self, context: &UnlockContext<'_>) -> bool {
        // Check player level
        if context.player_level < self.min_player_level {
            return false;
        }

        // Check skills
        for req in &self.skill_requirements {
            if !req.is_met(context.skills) {
                return false;
            }
        }

        // Check quests
        if !self.required_quests.is_subset(context.completed_quests) {
            return false;
        }

        // Check recipes
        if !self
            .required_recipes
            .iter()
            .all(|r| context.learned_recipes.is_learned(*r))
        {
            return false;
        }

        // Check achievements
        if !self.required_achievements.is_subset(context.achievements) {
            return false;
        }

        true
    }

    /// Get missing requirements as text.
    #[must_use]
    pub fn missing_requirements(&self, context: &UnlockContext<'_>) -> Vec<String> {
        let mut missing = Vec::new();

        if context.player_level < self.min_player_level {
            missing.push(format!(
                "Requires player level {} (current: {})",
                self.min_player_level, context.player_level
            ));
        }

        for req in &self.skill_requirements {
            let current = context.skills.get_level(&req.skill_type);
            if current < req.min_level {
                missing.push(format!(
                    "Requires {} level {} (current: {})",
                    req.skill_type, req.min_level, current
                ));
            }
        }

        for quest_id in &self.required_quests {
            if !context.completed_quests.contains(quest_id) {
                missing.push(format!("Requires quest #{quest_id} completion"));
            }
        }

        for recipe_id in &self.required_recipes {
            if !context.learned_recipes.is_learned(*recipe_id) {
                missing.push(format!("Requires recipe {recipe_id:?} to be learned"));
            }
        }

        missing
    }
}

/// Context for checking unlock conditions.
#[derive(Debug)]
pub struct UnlockContext<'a> {
    /// Player level.
    pub player_level: u32,
    /// Player skills.
    pub skills: &'a PlayerSkills,
    /// Completed quest IDs.
    pub completed_quests: &'a HashSet<u32>,
    /// Learned recipes.
    pub learned_recipes: &'a LearnedRecipes,
    /// Unlocked achievements.
    pub achievements: &'a HashSet<u32>,
}

// ============================================================================
// G-48: Crafting Progression System
// ============================================================================

/// Manages crafting progression for a player.
#[derive(Debug, Default)]
pub struct CraftingProgression {
    /// Learned recipes.
    pub learned: LearnedRecipes,
    /// Player skills.
    pub skills: PlayerSkills,
    /// Completed quests.
    pub completed_quests: HashSet<u32>,
    /// Unlocked achievements.
    pub achievements: HashSet<u32>,
    /// Player level.
    pub player_level: u32,
    /// Current game time.
    pub game_time: u64,
    /// Recipe unlock conditions.
    unlock_conditions: HashMap<RecipeId, UnlockConditions>,
}

impl CraftingProgression {
    /// Create new progression tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get unlock context for checking conditions.
    #[must_use]
    pub fn unlock_context(&self) -> UnlockContext<'_> {
        UnlockContext {
            player_level: self.player_level,
            skills: &self.skills,
            completed_quests: &self.completed_quests,
            learned_recipes: &self.learned,
            achievements: &self.achievements,
        }
    }

    /// Set unlock conditions for a recipe.
    pub fn set_unlock_conditions(&mut self, recipe_id: RecipeId, conditions: UnlockConditions) {
        self.unlock_conditions.insert(recipe_id, conditions);
    }

    /// Check if a recipe can be unlocked.
    #[must_use]
    pub fn can_unlock(&self, recipe_id: RecipeId) -> bool {
        if self.learned.is_learned(recipe_id) {
            return false;
        }

        if let Some(conditions) = self.unlock_conditions.get(&recipe_id) {
            conditions.are_met(&self.unlock_context())
        } else {
            true // No conditions = always unlockable
        }
    }

    /// Try to unlock a recipe.
    pub fn try_unlock(&mut self, recipe_id: RecipeId, source: DiscoverySource) -> bool {
        if !self.can_unlock(recipe_id) {
            return false;
        }

        self.learned.learn(recipe_id, source, self.game_time);
        true
    }

    /// Learn a recipe (ignores conditions - for admin/NPC teaching).
    pub fn force_learn(&mut self, recipe_id: RecipeId, source: DiscoverySource) {
        self.learned.learn(recipe_id, source, self.game_time);
    }

    /// Record crafting a recipe (grants XP).
    pub fn record_craft(&mut self, recipe_id: RecipeId, skill_type: &str, xp: u32) {
        self.learned.record_craft(recipe_id);
        self.skills.add_experience(skill_type, xp);
    }

    /// Complete a quest (may unlock recipes).
    pub fn complete_quest(&mut self, quest_id: u32) -> Vec<RecipeId> {
        self.completed_quests.insert(quest_id);
        self.check_newly_unlockable()
    }

    /// Unlock an achievement (may unlock recipes).
    pub fn unlock_achievement(&mut self, achievement_id: u32) -> Vec<RecipeId> {
        self.achievements.insert(achievement_id);
        self.check_newly_unlockable()
    }

    /// Set player level.
    pub fn set_player_level(&mut self, level: u32) -> Vec<RecipeId> {
        self.player_level = level;
        self.check_newly_unlockable()
    }

    /// Check for newly unlockable recipes.
    fn check_newly_unlockable(&self) -> Vec<RecipeId> {
        self.unlock_conditions
            .iter()
            .filter(|(id, conditions)| {
                !self.learned.is_learned(**id) && conditions.are_met(&self.unlock_context())
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all recipes that could be unlocked if conditions are met.
    #[must_use]
    pub fn potential_unlocks(&self) -> Vec<(RecipeId, Vec<String>)> {
        self.unlock_conditions
            .iter()
            .filter(|(id, _)| !self.learned.is_learned(**id))
            .map(|(id, conditions)| (*id, conditions.missing_requirements(&self.unlock_context())))
            .collect()
    }

    /// Update game time.
    pub fn set_game_time(&mut self, time: u64) {
        self.game_time = time;
    }

    /// Discover recipe through experimentation.
    pub fn discover_through_experimentation(&mut self, recipe_id: RecipeId) -> bool {
        self.try_unlock(recipe_id, DiscoverySource::Experimentation)
    }

    /// Learn recipe from NPC.
    pub fn learn_from_npc(&mut self, recipe_id: RecipeId, npc_id: EntityId, npc_name: &str) {
        self.force_learn(
            recipe_id,
            DiscoverySource::NPC {
                npc_id,
                npc_name: npc_name.to_string(),
            },
        );
    }

    /// Learn recipe as quest reward.
    pub fn learn_from_quest(&mut self, recipe_id: RecipeId, quest_id: u32, quest_name: &str) {
        self.force_learn(
            recipe_id,
            DiscoverySource::Quest {
                quest_id,
                quest_name: quest_name.to_string(),
            },
        );
    }
}

// ============================================================================
// G-48: Experimentation System
// ============================================================================

/// Tracks experimentation progress for recipe discovery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentationTracker {
    /// Attempted combinations (hash -> attempt count).
    attempted_combinations: HashMap<u64, u32>,
    /// Close matches (combinations that almost worked).
    close_matches: HashMap<u64, CloseMatch>,
    /// Total experiments performed.
    pub total_experiments: u32,
    /// Successful discoveries.
    pub discoveries: u32,
}

impl ExperimentationTracker {
    /// Create new tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an experiment attempt.
    pub fn record_attempt(&mut self, combination_hash: u64) {
        *self
            .attempted_combinations
            .entry(combination_hash)
            .or_insert(0) += 1;
        self.total_experiments += 1;
    }

    /// Record a close match.
    pub fn record_close_match(
        &mut self,
        combination_hash: u64,
        recipe_id: RecipeId,
        similarity: f32,
    ) {
        self.close_matches.insert(
            combination_hash,
            CloseMatch {
                recipe_id,
                similarity,
            },
        );
    }

    /// Record a successful discovery.
    pub fn record_discovery(&mut self) {
        self.discoveries += 1;
    }

    /// Get attempt count for a combination.
    #[must_use]
    pub fn attempt_count(&self, combination_hash: u64) -> u32 {
        self.attempted_combinations
            .get(&combination_hash)
            .copied()
            .unwrap_or(0)
    }

    /// Get close match for a combination.
    #[must_use]
    pub fn get_close_match(&self, combination_hash: u64) -> Option<&CloseMatch> {
        self.close_matches.get(&combination_hash)
    }

    /// Get discovery rate.
    #[must_use]
    pub fn discovery_rate(&self) -> f32 {
        if self.total_experiments == 0 {
            return 0.0;
        }
        self.discoveries as f32 / self.total_experiments as f32
    }
}

/// A close match from experimentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseMatch {
    /// Recipe that was almost matched.
    pub recipe_id: RecipeId,
    /// Similarity score (0.0-1.0).
    pub similarity: f32,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_recipe_id(id: u32) -> RecipeId {
        RecipeId::new(id)
    }

    fn test_entity_id(id: u64) -> EntityId {
        EntityId::from_raw(id)
    }

    #[test]
    fn test_discovery_source_display() {
        let source = DiscoverySource::Default;
        assert_eq!(source.display_text(), "Known by default");

        let source = DiscoverySource::SkillLevel {
            skill: "smithing".to_string(),
            level: 10,
        };
        assert_eq!(source.display_text(), "Unlocked at smithing level 10");
    }

    #[test]
    fn test_learned_recipe() {
        let mut recipe = LearnedRecipe::new(test_recipe_id(1), DiscoverySource::Default, 1000);
        assert_eq!(recipe.times_crafted, 0);

        recipe.record_craft();
        assert_eq!(recipe.times_crafted, 1);

        recipe.toggle_favorite();
        assert!(recipe.is_favorite);

        recipe.set_notes("My favorite recipe");
        assert_eq!(recipe.notes, "My favorite recipe");
    }

    #[test]
    fn test_learned_recipes_collection() {
        let mut learned = LearnedRecipes::new();
        assert_eq!(learned.count(), 0);

        learned.learn(test_recipe_id(1), DiscoverySource::Default, 0);
        assert!(learned.is_learned(test_recipe_id(1)));
        assert!(!learned.is_learned(test_recipe_id(2)));
        assert_eq!(learned.count(), 1);
    }

    #[test]
    fn test_learned_recipes_record_craft() {
        let mut learned = LearnedRecipes::new();
        learned.learn(test_recipe_id(1), DiscoverySource::Default, 0);

        learned.record_craft(test_recipe_id(1));

        let recipe = learned.get(test_recipe_id(1)).unwrap();
        assert_eq!(recipe.times_crafted, 1);
    }

    #[test]
    fn test_learned_recipes_favorites() {
        let mut learned = LearnedRecipes::new();
        learned.learn(test_recipe_id(1), DiscoverySource::Default, 0);
        learned.learn(test_recipe_id(2), DiscoverySource::Default, 0);

        learned
            .get_mut(test_recipe_id(1))
            .unwrap()
            .toggle_favorite();

        let favorites = learned.favorites();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].recipe_id, test_recipe_id(1));
    }

    #[test]
    fn test_discovery_hint() {
        let hint = DiscoveryHint::new(test_recipe_id(1), "Try combining wood and stone")
            .with_ingredient_progress(2, 4)
            .with_output_known(true);

        assert_eq!(hint.progress_percent(), 50.0);
        assert!(hint.output_known);
    }

    #[test]
    fn test_skill_requirement() {
        let req = SkillRequirement::new("smithing", 5);

        let mut skills = PlayerSkills::new();
        assert!(!req.is_met(&skills));

        skills.set_level("smithing", 5);
        assert!(req.is_met(&skills));
    }

    #[test]
    fn test_player_skills_experience() {
        let mut skills = PlayerSkills::new();

        // Add 150 XP (should level up to 1)
        let level_up = skills.add_experience("crafting", 150);
        assert_eq!(level_up, Some(1));
        assert_eq!(skills.get_level("crafting"), 1);

        // Progress is 50% to level 2
        assert_eq!(skills.level_progress("crafting"), 0.5);
    }

    #[test]
    fn test_unlock_conditions() {
        let mut learned = LearnedRecipes::new();
        let skills = PlayerSkills::new();
        let completed_quests = HashSet::new();
        let achievements = HashSet::new();

        let conditions = UnlockConditions::none()
            .with_skill("smithing", 5)
            .with_min_level(10);

        let context = UnlockContext {
            player_level: 5,
            skills: &skills,
            completed_quests: &completed_quests,
            learned_recipes: &learned,
            achievements: &achievements,
        };

        assert!(!conditions.are_met(&context));

        let missing = conditions.missing_requirements(&context);
        assert!(missing.iter().any(|m| m.contains("player level")));
        assert!(missing.iter().any(|m| m.contains("smithing")));
    }

    #[test]
    fn test_crafting_progression() {
        let mut progression = CraftingProgression::new();

        // Set up conditions
        progression.set_unlock_conditions(
            test_recipe_id(1),
            UnlockConditions::none().with_min_level(5),
        );

        // Can't unlock at level 0
        assert!(!progression.can_unlock(test_recipe_id(1)));

        // Level up and check again
        progression.set_player_level(5);
        assert!(progression.can_unlock(test_recipe_id(1)));

        // Try to unlock
        assert!(progression.try_unlock(test_recipe_id(1), DiscoverySource::Default));
        assert!(progression.learned.is_learned(test_recipe_id(1)));
    }

    #[test]
    fn test_force_learn() {
        let mut progression = CraftingProgression::new();

        // Set up strict conditions
        progression.set_unlock_conditions(
            test_recipe_id(1),
            UnlockConditions::none().with_min_level(100),
        );

        // Force learn bypasses conditions
        progression.force_learn(test_recipe_id(1), DiscoverySource::Admin);
        assert!(progression.learned.is_learned(test_recipe_id(1)));
    }

    #[test]
    fn test_learn_from_npc() {
        let mut progression = CraftingProgression::new();

        progression.learn_from_npc(test_recipe_id(1), test_entity_id(100), "Master Smith");

        let recipe = progression.learned.get(test_recipe_id(1)).unwrap();
        assert!(matches!(recipe.source, DiscoverySource::NPC { .. }));
    }

    #[test]
    fn test_record_craft_grants_xp() {
        let mut progression = CraftingProgression::new();
        progression.force_learn(test_recipe_id(1), DiscoverySource::Default);

        progression.record_craft(test_recipe_id(1), "smithing", 50);

        let recipe = progression.learned.get(test_recipe_id(1)).unwrap();
        assert_eq!(recipe.times_crafted, 1);
        assert_eq!(progression.skills.get_experience("smithing"), 50);
    }

    #[test]
    fn test_experimentation_tracker() {
        let mut tracker = ExperimentationTracker::new();

        tracker.record_attempt(12345);
        tracker.record_attempt(12345);
        tracker.record_attempt(67890);

        assert_eq!(tracker.attempt_count(12345), 2);
        assert_eq!(tracker.attempt_count(67890), 1);
        assert_eq!(tracker.total_experiments, 3);
    }

    #[test]
    fn test_experimentation_discovery_rate() {
        let mut tracker = ExperimentationTracker::new();

        for _ in 0..10 {
            tracker.record_attempt(0);
        }
        tracker.record_discovery();
        tracker.record_discovery();

        assert_eq!(tracker.discovery_rate(), 0.2);
    }

    #[test]
    fn test_potential_unlocks() {
        let mut progression = CraftingProgression::new();

        progression.set_unlock_conditions(
            test_recipe_id(1),
            UnlockConditions::none().with_min_level(5),
        );
        progression.set_unlock_conditions(
            test_recipe_id(2),
            UnlockConditions::none().with_min_level(10),
        );

        let potential = progression.potential_unlocks();
        assert_eq!(potential.len(), 2);

        // All should have missing requirements
        for (_, missing) in &potential {
            assert!(!missing.is_empty());
        }
    }
}
