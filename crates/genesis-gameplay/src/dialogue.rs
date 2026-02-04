//! Dialogue system for NPC conversations.
//!
//! This module provides:
//! - Dialogue nodes with branching choices
//! - Conditions based on items, quests, reputation
//! - Effects like giving/taking items, starting quests
//! - Variable substitution in text

use crate::quest::QuestId;
use genesis_common::{FactionId, ItemTypeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for dialogue operations.
#[derive(Debug, Error)]
pub enum DialogueError {
    /// Dialogue not found
    #[error("Dialogue not found: {0}")]
    NotFound(u32),
    /// Node not found in dialogue
    #[error("Node not found: {0}")]
    NodeNotFound(u32),
    /// No active dialogue
    #[error("No active dialogue")]
    NoActiveDialogue,
    /// Condition not met
    #[error("Condition not met for choice")]
    ConditionNotMet,
    /// Invalid choice index
    #[error("Invalid choice index: {0}")]
    InvalidChoice(usize),
}

/// Result type for dialogue operations.
pub type DialogueResult<T> = Result<T, DialogueError>;

/// Unique identifier for a dialogue tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DialogueId(pub u32);

impl DialogueId {
    /// Create a new dialogue ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A single node in a dialogue tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// Unique ID of this node.
    pub id: u32,
    /// Speaker name (NPC name, "Player", etc.).
    pub speaker: String,
    /// The text to display.
    pub text: String,
    /// Available choices/responses.
    pub choices: Vec<DialogueChoice>,
}

impl DialogueNode {
    /// Create a new dialogue node.
    #[must_use]
    pub fn new(id: u32, speaker: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id,
            speaker: speaker.into(),
            text: text.into(),
            choices: Vec::new(),
        }
    }

    /// Add a choice to this node.
    #[must_use]
    pub fn with_choice(mut self, choice: DialogueChoice) -> Self {
        self.choices.push(choice);
        self
    }

    /// Add multiple choices.
    #[must_use]
    pub fn with_choices(mut self, choices: Vec<DialogueChoice>) -> Self {
        self.choices = choices;
        self
    }

    /// Check if this is an end node (no choices).
    #[must_use]
    pub fn is_end(&self) -> bool {
        self.choices.is_empty()
    }
}

/// A choice within a dialogue node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    /// Text displayed for this choice.
    pub text: String,
    /// Next node ID, or None if this ends the dialogue.
    pub next_node: Option<u32>,
    /// Condition required to show this choice.
    pub condition: Option<DialogueCondition>,
    /// Effect triggered when choosing this.
    pub effect: Option<DialogueEffect>,
}

impl DialogueChoice {
    /// Create a simple choice that leads to another node.
    #[must_use]
    pub fn new(text: impl Into<String>, next_node: u32) -> Self {
        Self {
            text: text.into(),
            next_node: Some(next_node),
            condition: None,
            effect: None,
        }
    }

    /// Create a choice that ends the dialogue.
    #[must_use]
    pub fn end(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            next_node: None,
            condition: None,
            effect: None,
        }
    }

    /// Add a condition to this choice.
    #[must_use]
    pub fn with_condition(mut self, condition: DialogueCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Add an effect to this choice.
    #[must_use]
    pub fn with_effect(mut self, effect: DialogueEffect) -> Self {
        self.effect = Some(effect);
        self
    }
}

/// Conditions that must be met to show a dialogue choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueCondition {
    /// Player has at least N of an item.
    HasItem(ItemTypeId, u32),
    /// Player doesn't have an item.
    MissingItem(ItemTypeId),
    /// Quest is complete.
    QuestComplete(QuestId),
    /// Quest is active (started but not complete).
    QuestActive(QuestId),
    /// Quest is not started.
    QuestNotStarted(QuestId),
    /// Reputation with faction is above threshold.
    ReputationAbove(FactionId, i32),
    /// Reputation with faction is below threshold.
    ReputationBelow(FactionId, i32),
    /// Player level is at least N.
    LevelAtLeast(u32),
    /// Custom flag is set.
    FlagSet(String),
    /// Custom flag is not set.
    FlagNotSet(String),
    /// All conditions must be true.
    All(Vec<DialogueCondition>),
    /// At least one condition must be true.
    Any(Vec<DialogueCondition>),
    /// Condition must be false.
    Not(Box<DialogueCondition>),
}

impl DialogueCondition {
    /// Create an item requirement.
    #[must_use]
    pub fn has_item(item: ItemTypeId, count: u32) -> Self {
        Self::HasItem(item, count)
    }

    /// Create a quest complete requirement.
    #[must_use]
    pub fn quest_complete(quest: QuestId) -> Self {
        Self::QuestComplete(quest)
    }

    /// Create a reputation requirement.
    #[must_use]
    pub fn reputation_above(faction: FactionId, threshold: i32) -> Self {
        Self::ReputationAbove(faction, threshold)
    }

    /// Combine conditions with AND.
    #[must_use]
    pub fn and(conditions: Vec<DialogueCondition>) -> Self {
        Self::All(conditions)
    }

    /// Combine conditions with OR.
    #[must_use]
    pub fn or(conditions: Vec<DialogueCondition>) -> Self {
        Self::Any(conditions)
    }

    /// Negate a condition.
    #[must_use]
    pub fn negate(condition: DialogueCondition) -> Self {
        Self::Not(Box::new(condition))
    }
}

/// Effects triggered when selecting a dialogue choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueEffect {
    /// Give item(s) to player.
    GiveItem(ItemTypeId, u32),
    /// Take item(s) from player.
    TakeItem(ItemTypeId, u32),
    /// Start a quest.
    StartQuest(QuestId),
    /// Complete a quest objective.
    CompleteObjective(QuestId, u32),
    /// Add reputation with faction.
    AddReputation(FactionId, i32),
    /// Set a custom flag.
    SetFlag(String),
    /// Clear a custom flag.
    ClearFlag(String),
    /// Give currency.
    GiveCurrency(u64),
    /// Take currency.
    TakeCurrency(u64),
    /// Trigger multiple effects.
    Multiple(Vec<DialogueEffect>),
    /// Custom effect (for extension).
    Custom(String),
}

impl DialogueEffect {
    /// Create an item gift.
    #[must_use]
    pub fn give_item(item: ItemTypeId, count: u32) -> Self {
        Self::GiveItem(item, count)
    }

    /// Create an item requirement.
    #[must_use]
    pub fn take_item(item: ItemTypeId, count: u32) -> Self {
        Self::TakeItem(item, count)
    }

    /// Create a quest start effect.
    #[must_use]
    pub fn start_quest(quest: QuestId) -> Self {
        Self::StartQuest(quest)
    }

    /// Create a reputation effect.
    #[must_use]
    pub fn add_reputation(faction: FactionId, amount: i32) -> Self {
        Self::AddReputation(faction, amount)
    }

    /// Combine multiple effects.
    #[must_use]
    pub fn multiple(effects: Vec<DialogueEffect>) -> Self {
        Self::Multiple(effects)
    }
}

/// A complete dialogue tree with nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    /// Unique ID of this dialogue.
    pub id: DialogueId,
    /// Title/name of this dialogue.
    pub title: String,
    /// Starting node ID.
    pub start_node: u32,
    /// All nodes in this dialogue.
    nodes: HashMap<u32, DialogueNode>,
}

impl Dialogue {
    /// Create a new dialogue.
    #[must_use]
    pub fn new(id: DialogueId, title: impl Into<String>, start_node: u32) -> Self {
        Self {
            id,
            title: title.into(),
            start_node,
            nodes: HashMap::new(),
        }
    }

    /// Add a node to this dialogue.
    pub fn add_node(&mut self, node: DialogueNode) {
        self.nodes.insert(node.id, node);
    }

    /// Add a node (builder pattern).
    #[must_use]
    pub fn with_node(mut self, node: DialogueNode) -> Self {
        self.add_node(node);
        self
    }

    /// Get a node by ID.
    #[must_use]
    pub fn get_node(&self, id: u32) -> Option<&DialogueNode> {
        self.nodes.get(&id)
    }

    /// Get the start node.
    #[must_use]
    pub fn get_start_node(&self) -> Option<&DialogueNode> {
        self.get_node(self.start_node)
    }

    /// Get all node IDs.
    pub fn node_ids(&self) -> impl Iterator<Item = &u32> {
        self.nodes.keys()
    }

    /// Get node count.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Variables that can be substituted in dialogue text.
#[derive(Debug, Clone, Default)]
pub struct DialogueVariables {
    variables: HashMap<String, String>,
}

impl DialogueVariables {
    /// Create empty variables.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Get a variable value.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(String::as_str)
    }

    /// Check if a variable is set.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Substitute variables in text.
    /// Variables are enclosed in braces: {variable_name}
    #[must_use]
    pub fn substitute(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (name, value) in &self.variables {
            let pattern = format!("{{{name}}}");
            result = result.replace(&pattern, value);
        }
        result
    }

    /// Add common player variables.
    pub fn set_player_info(&mut self, name: &str, level: u32) {
        self.set("player_name", name);
        self.set("player_level", level.to_string());
    }
}

/// Context for evaluating conditions.
pub trait DialogueContext {
    /// Check if player has an item.
    fn has_item(&self, item: ItemTypeId, count: u32) -> bool;

    /// Check if quest is complete.
    fn quest_complete(&self, quest: QuestId) -> bool;

    /// Check if quest is active.
    fn quest_active(&self, quest: QuestId) -> bool;

    /// Get reputation with faction.
    fn reputation(&self, faction: FactionId) -> i32;

    /// Get player level.
    fn player_level(&self) -> u32;

    /// Check if flag is set.
    fn flag_set(&self, flag: &str) -> bool;

    /// Get player currency.
    fn currency(&self) -> u64;
}

/// Evaluates dialogue conditions.
pub fn evaluate_condition<C: DialogueContext>(condition: &DialogueCondition, context: &C) -> bool {
    match condition {
        DialogueCondition::HasItem(item, count) => context.has_item(*item, *count),
        DialogueCondition::MissingItem(item) => !context.has_item(*item, 1),
        DialogueCondition::QuestComplete(quest) => context.quest_complete(*quest),
        DialogueCondition::QuestActive(quest) => context.quest_active(*quest),
        DialogueCondition::QuestNotStarted(quest) => {
            !context.quest_complete(*quest) && !context.quest_active(*quest)
        },
        DialogueCondition::ReputationAbove(faction, threshold) => {
            context.reputation(*faction) > *threshold
        },
        DialogueCondition::ReputationBelow(faction, threshold) => {
            context.reputation(*faction) < *threshold
        },
        DialogueCondition::LevelAtLeast(level) => context.player_level() >= *level,
        DialogueCondition::FlagSet(flag) => context.flag_set(flag),
        DialogueCondition::FlagNotSet(flag) => !context.flag_set(flag),
        DialogueCondition::All(conditions) => {
            conditions.iter().all(|c| evaluate_condition(c, context))
        },
        DialogueCondition::Any(conditions) => {
            conditions.iter().any(|c| evaluate_condition(c, context))
        },
        DialogueCondition::Not(condition) => !evaluate_condition(condition, context),
    }
}

/// Currently active dialogue state.
#[derive(Debug, Clone)]
pub struct ActiveDialogue {
    /// The dialogue being played.
    pub dialogue_id: DialogueId,
    /// Current node ID.
    pub current_node: u32,
    /// NPC entity we're talking to.
    pub npc_entity: Option<u64>,
}

impl ActiveDialogue {
    /// Create a new active dialogue.
    #[must_use]
    pub fn new(dialogue_id: DialogueId, start_node: u32) -> Self {
        Self {
            dialogue_id,
            current_node: start_node,
            npc_entity: None,
        }
    }

    /// Set the NPC entity.
    #[must_use]
    pub fn with_npc(mut self, entity: u64) -> Self {
        self.npc_entity = Some(entity);
        self
    }
}

/// Pending effects from dialogue choices.
#[derive(Debug, Clone, Default)]
pub struct PendingEffects {
    effects: Vec<DialogueEffect>,
}

impl PendingEffects {
    /// Create empty pending effects.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an effect.
    pub fn add(&mut self, effect: DialogueEffect) {
        self.effects.push(effect);
    }

    /// Take all pending effects.
    pub fn take(&mut self) -> Vec<DialogueEffect> {
        std::mem::take(&mut self.effects)
    }

    /// Check if there are pending effects.
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.effects.is_empty()
    }
}

/// Manages all dialogues and active conversation state.
#[derive(Debug)]
pub struct DialogueManager {
    /// All registered dialogues.
    dialogues: HashMap<DialogueId, Dialogue>,
    /// Currently active dialogue.
    active: Option<ActiveDialogue>,
    /// Variables for text substitution.
    variables: DialogueVariables,
    /// Pending effects from choices.
    pending_effects: PendingEffects,
    /// Custom flags.
    flags: HashMap<String, bool>,
}

impl DialogueManager {
    /// Create a new dialogue manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialogues: HashMap::new(),
            active: None,
            variables: DialogueVariables::new(),
            pending_effects: PendingEffects::new(),
            flags: HashMap::new(),
        }
    }

    /// Register a dialogue.
    pub fn register(&mut self, dialogue: Dialogue) {
        self.dialogues.insert(dialogue.id, dialogue);
    }

    /// Get a dialogue by ID.
    #[must_use]
    pub fn get_dialogue(&self, id: DialogueId) -> Option<&Dialogue> {
        self.dialogues.get(&id)
    }

    /// Get variables for modification.
    pub fn variables_mut(&mut self) -> &mut DialogueVariables {
        &mut self.variables
    }

    /// Get variables.
    #[must_use]
    pub fn variables(&self) -> &DialogueVariables {
        &self.variables
    }

    /// Set a flag.
    pub fn set_flag(&mut self, flag: impl Into<String>) {
        self.flags.insert(flag.into(), true);
    }

    /// Clear a flag.
    pub fn clear_flag(&mut self, flag: &str) {
        self.flags.remove(flag);
    }

    /// Check if flag is set.
    #[must_use]
    pub fn is_flag_set(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    /// Start a dialogue.
    pub fn start(&mut self, dialogue_id: DialogueId) -> DialogueResult<&DialogueNode> {
        let dialogue = self
            .dialogues
            .get(&dialogue_id)
            .ok_or(DialogueError::NotFound(dialogue_id.0))?;

        let start_node = dialogue.start_node;
        self.active = Some(ActiveDialogue::new(dialogue_id, start_node));

        dialogue
            .get_node(start_node)
            .ok_or(DialogueError::NodeNotFound(start_node))
    }

    /// Start dialogue with an NPC.
    pub fn start_with_npc(
        &mut self,
        dialogue_id: DialogueId,
        npc_entity: u64,
    ) -> DialogueResult<&DialogueNode> {
        let dialogue = self
            .dialogues
            .get(&dialogue_id)
            .ok_or(DialogueError::NotFound(dialogue_id.0))?;

        let start_node = dialogue.start_node;
        self.active = Some(ActiveDialogue::new(dialogue_id, start_node).with_npc(npc_entity));

        dialogue
            .get_node(start_node)
            .ok_or(DialogueError::NodeNotFound(start_node))
    }

    /// Get current dialogue node.
    #[must_use]
    pub fn current_node(&self) -> Option<&DialogueNode> {
        let active = self.active.as_ref()?;
        let dialogue = self.dialogues.get(&active.dialogue_id)?;
        dialogue.get_node(active.current_node)
    }

    /// Get current node text with variables substituted.
    #[must_use]
    pub fn current_text(&self) -> Option<String> {
        let node = self.current_node()?;
        Some(self.variables.substitute(&node.text))
    }

    /// Get available choices for current node.
    pub fn available_choices<C: DialogueContext>(
        &self,
        context: &C,
    ) -> Vec<(usize, &DialogueChoice)> {
        let node = match self.current_node() {
            Some(n) => n,
            None => return Vec::new(),
        };

        node.choices
            .iter()
            .enumerate()
            .filter(|(_, choice)| {
                choice
                    .condition
                    .as_ref()
                    .map_or(true, |c| evaluate_condition(c, context))
            })
            .collect()
    }

    /// Select a choice and advance the dialogue.
    pub fn select_choice<C: DialogueContext>(
        &mut self,
        choice_index: usize,
        context: &C,
    ) -> DialogueResult<Option<&DialogueNode>> {
        let active = self
            .active
            .as_ref()
            .ok_or(DialogueError::NoActiveDialogue)?;
        let dialogue = self
            .dialogues
            .get(&active.dialogue_id)
            .ok_or(DialogueError::NotFound(active.dialogue_id.0))?;
        let node = dialogue
            .get_node(active.current_node)
            .ok_or(DialogueError::NodeNotFound(active.current_node))?;

        let choice = node
            .choices
            .get(choice_index)
            .ok_or(DialogueError::InvalidChoice(choice_index))?;

        // Check condition
        if let Some(ref condition) = choice.condition {
            if !evaluate_condition(condition, context) {
                return Err(DialogueError::ConditionNotMet);
            }
        }

        // Queue effect
        if let Some(ref effect) = choice.effect {
            self.pending_effects.add(effect.clone());
        }

        // Advance to next node or end
        if let Some(next_id) = choice.next_node {
            self.active.as_mut().expect("checked above").current_node = next_id;
            Ok(dialogue.get_node(next_id))
        } else {
            self.active = None;
            Ok(None)
        }
    }

    /// End the current dialogue.
    pub fn end(&mut self) {
        self.active = None;
    }

    /// Check if dialogue is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Get active dialogue info.
    #[must_use]
    pub fn active_dialogue(&self) -> Option<&ActiveDialogue> {
        self.active.as_ref()
    }

    /// Take pending effects.
    pub fn take_effects(&mut self) -> Vec<DialogueEffect> {
        self.pending_effects.take()
    }

    /// Check if there are pending effects.
    #[must_use]
    pub fn has_pending_effects(&self) -> bool {
        self.pending_effects.has_pending()
    }
}

impl Default for DialogueManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple dialogue context for testing.
#[derive(Debug, Default)]
pub struct SimpleDialogueContext {
    items: HashMap<ItemTypeId, u32>,
    quests_complete: Vec<QuestId>,
    quests_active: Vec<QuestId>,
    reputation: HashMap<FactionId, i32>,
    level: u32,
    flags: HashMap<String, bool>,
    currency: u64,
}

impl SimpleDialogueContext {
    /// Create a new context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            level: 1,
            ..Default::default()
        }
    }

    /// Add items.
    pub fn with_item(mut self, item: ItemTypeId, count: u32) -> Self {
        self.items.insert(item, count);
        self
    }

    /// Set level.
    #[must_use]
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    /// Add complete quest.
    pub fn with_quest_complete(mut self, quest: QuestId) -> Self {
        self.quests_complete.push(quest);
        self
    }

    /// Add active quest.
    pub fn with_quest_active(mut self, quest: QuestId) -> Self {
        self.quests_active.push(quest);
        self
    }

    /// Set reputation.
    pub fn with_reputation(mut self, faction: FactionId, rep: i32) -> Self {
        self.reputation.insert(faction, rep);
        self
    }

    /// Set flag.
    pub fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.flags.insert(flag.into(), true);
        self
    }

    /// Set currency.
    #[must_use]
    pub fn with_currency(mut self, amount: u64) -> Self {
        self.currency = amount;
        self
    }
}

impl DialogueContext for SimpleDialogueContext {
    fn has_item(&self, item: ItemTypeId, count: u32) -> bool {
        self.items.get(&item).copied().unwrap_or(0) >= count
    }

    fn quest_complete(&self, quest: QuestId) -> bool {
        self.quests_complete.contains(&quest)
    }

    fn quest_active(&self, quest: QuestId) -> bool {
        self.quests_active.contains(&quest)
    }

    fn reputation(&self, faction: FactionId) -> i32 {
        self.reputation.get(&faction).copied().unwrap_or(0)
    }

    fn player_level(&self) -> u32 {
        self.level
    }

    fn flag_set(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    fn currency(&self) -> u64 {
        self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item() -> ItemTypeId {
        ItemTypeId::new(1)
    }

    fn test_quest() -> QuestId {
        QuestId::new(1)
    }

    fn test_faction() -> FactionId {
        FactionId::new(1)
    }

    // Dialogue node tests
    #[test]
    fn test_dialogue_node_creation() {
        let node = DialogueNode::new(1, "NPC", "Hello, traveler!");
        assert_eq!(node.id, 1);
        assert_eq!(node.speaker, "NPC");
        assert_eq!(node.text, "Hello, traveler!");
        assert!(node.is_end());
    }

    #[test]
    fn test_dialogue_node_with_choices() {
        let node = DialogueNode::new(1, "NPC", "What do you need?")
            .with_choice(DialogueChoice::new("Buy items", 2))
            .with_choice(DialogueChoice::new("Sell items", 3))
            .with_choice(DialogueChoice::end("Goodbye"));

        assert!(!node.is_end());
        assert_eq!(node.choices.len(), 3);
    }

    // Dialogue choice tests
    #[test]
    fn test_dialogue_choice_simple() {
        let choice = DialogueChoice::new("Yes", 2);
        assert_eq!(choice.text, "Yes");
        assert_eq!(choice.next_node, Some(2));
        assert!(choice.condition.is_none());
        assert!(choice.effect.is_none());
    }

    #[test]
    fn test_dialogue_choice_end() {
        let choice = DialogueChoice::end("Goodbye");
        assert!(choice.next_node.is_none());
    }

    #[test]
    fn test_dialogue_choice_with_condition() {
        let choice = DialogueChoice::new("Give me the artifact", 5)
            .with_condition(DialogueCondition::has_item(test_item(), 1));

        assert!(choice.condition.is_some());
    }

    #[test]
    fn test_dialogue_choice_with_effect() {
        let choice = DialogueChoice::new("Accept quest", 10)
            .with_effect(DialogueEffect::start_quest(test_quest()));

        assert!(choice.effect.is_some());
    }

    // Dialogue condition tests
    #[test]
    fn test_condition_has_item() {
        let ctx = SimpleDialogueContext::new().with_item(test_item(), 5);
        let cond = DialogueCondition::has_item(test_item(), 3);

        assert!(evaluate_condition(&cond, &ctx));
    }

    #[test]
    fn test_condition_has_item_fail() {
        let ctx = SimpleDialogueContext::new().with_item(test_item(), 2);
        let cond = DialogueCondition::has_item(test_item(), 3);

        assert!(!evaluate_condition(&cond, &ctx));
    }

    #[test]
    fn test_condition_quest_complete() {
        let ctx = SimpleDialogueContext::new().with_quest_complete(test_quest());
        let cond = DialogueCondition::quest_complete(test_quest());

        assert!(evaluate_condition(&cond, &ctx));
    }

    #[test]
    fn test_condition_reputation() {
        let ctx = SimpleDialogueContext::new().with_reputation(test_faction(), 50);
        let above = DialogueCondition::reputation_above(test_faction(), 25);
        let below = DialogueCondition::ReputationBelow(test_faction(), 75);

        assert!(evaluate_condition(&above, &ctx));
        assert!(evaluate_condition(&below, &ctx));
    }

    #[test]
    fn test_condition_level() {
        let ctx = SimpleDialogueContext::new().with_level(10);
        let cond = DialogueCondition::LevelAtLeast(5);

        assert!(evaluate_condition(&cond, &ctx));
    }

    #[test]
    fn test_condition_flag() {
        let ctx = SimpleDialogueContext::new().with_flag("talked_to_king");

        assert!(evaluate_condition(
            &DialogueCondition::FlagSet("talked_to_king".to_string()),
            &ctx
        ));
        assert!(evaluate_condition(
            &DialogueCondition::FlagNotSet("other_flag".to_string()),
            &ctx
        ));
    }

    #[test]
    fn test_condition_all() {
        let ctx = SimpleDialogueContext::new()
            .with_item(test_item(), 5)
            .with_level(10);

        let cond = DialogueCondition::All(vec![
            DialogueCondition::has_item(test_item(), 3),
            DialogueCondition::LevelAtLeast(5),
        ]);

        assert!(evaluate_condition(&cond, &ctx));
    }

    #[test]
    fn test_condition_any() {
        let ctx = SimpleDialogueContext::new().with_level(10);

        let cond = DialogueCondition::Any(vec![
            DialogueCondition::has_item(test_item(), 3), // False
            DialogueCondition::LevelAtLeast(5),          // True
        ]);

        assert!(evaluate_condition(&cond, &ctx));
    }

    #[test]
    fn test_condition_not() {
        let ctx = SimpleDialogueContext::new();
        let cond = DialogueCondition::Not(Box::new(DialogueCondition::has_item(test_item(), 1)));

        assert!(evaluate_condition(&cond, &ctx));
    }

    // Dialogue tests
    #[test]
    fn test_dialogue_creation() {
        let dialogue = Dialogue::new(DialogueId::new(1), "Test Dialogue", 0)
            .with_node(DialogueNode::new(0, "NPC", "Hello!"));

        assert_eq!(dialogue.id, DialogueId::new(1));
        assert_eq!(dialogue.node_count(), 1);
        assert!(dialogue.get_start_node().is_some());
    }

    #[test]
    fn test_dialogue_multiple_nodes() {
        let dialogue = Dialogue::new(DialogueId::new(1), "Test", 0)
            .with_node(
                DialogueNode::new(0, "NPC", "Hello!").with_choice(DialogueChoice::new("Hi!", 1)),
            )
            .with_node(DialogueNode::new(1, "NPC", "Goodbye!"));

        assert_eq!(dialogue.node_count(), 2);
    }

    // Variable substitution tests
    #[test]
    fn test_variables_substitute() {
        let mut vars = DialogueVariables::new();
        vars.set("player_name", "Hero");
        vars.set("item", "Sword");

        let text = "Hello, {player_name}! Here is your {item}.";
        let result = vars.substitute(text);

        assert_eq!(result, "Hello, Hero! Here is your Sword.");
    }

    #[test]
    fn test_variables_no_match() {
        let vars = DialogueVariables::new();
        let text = "No variables here.";
        let result = vars.substitute(text);

        assert_eq!(result, text);
    }

    #[test]
    fn test_variables_player_info() {
        let mut vars = DialogueVariables::new();
        vars.set_player_info("Hero", 15);

        assert_eq!(vars.get("player_name"), Some("Hero"));
        assert_eq!(vars.get("player_level"), Some("15"));
    }

    // Dialogue manager tests
    #[test]
    fn test_dialogue_manager_register() {
        let mut manager = DialogueManager::new();
        let dialogue = Dialogue::new(DialogueId::new(1), "Test", 0)
            .with_node(DialogueNode::new(0, "NPC", "Hello!"));

        manager.register(dialogue);

        assert!(manager.get_dialogue(DialogueId::new(1)).is_some());
    }

    #[test]
    fn test_dialogue_manager_start() {
        let mut manager = DialogueManager::new();
        manager.register(
            Dialogue::new(DialogueId::new(1), "Test", 0)
                .with_node(DialogueNode::new(0, "NPC", "Hello!")),
        );

        let node = manager.start(DialogueId::new(1)).unwrap();
        assert_eq!(node.text, "Hello!");
        assert!(manager.is_active());
    }

    #[test]
    fn test_dialogue_manager_current_text() {
        let mut manager = DialogueManager::new();
        manager.register(
            Dialogue::new(DialogueId::new(1), "Test", 0).with_node(DialogueNode::new(
                0,
                "NPC",
                "Hello, {player_name}!",
            )),
        );
        manager.variables_mut().set("player_name", "Hero");

        manager.start(DialogueId::new(1)).unwrap();
        let text = manager.current_text().unwrap();

        assert_eq!(text, "Hello, Hero!");
    }

    #[test]
    fn test_dialogue_manager_select_choice() {
        let mut manager = DialogueManager::new();
        manager.register(
            Dialogue::new(DialogueId::new(1), "Test", 0)
                .with_node(
                    DialogueNode::new(0, "NPC", "Hello!")
                        .with_choice(DialogueChoice::new("Hi!", 1)),
                )
                .with_node(DialogueNode::new(1, "NPC", "Goodbye!")),
        );

        let ctx = SimpleDialogueContext::new();
        manager.start(DialogueId::new(1)).unwrap();

        let next = manager.select_choice(0, &ctx).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().text, "Goodbye!");
    }

    #[test]
    fn test_dialogue_manager_end_choice() {
        let mut manager = DialogueManager::new();
        manager.register(Dialogue::new(DialogueId::new(1), "Test", 0).with_node(
            DialogueNode::new(0, "NPC", "Hello!").with_choice(DialogueChoice::end("Bye!")),
        ));

        let ctx = SimpleDialogueContext::new();
        manager.start(DialogueId::new(1)).unwrap();

        let next = manager.select_choice(0, &ctx).unwrap();
        assert!(next.is_none());
        assert!(!manager.is_active());
    }

    #[test]
    fn test_dialogue_manager_available_choices() {
        let mut manager = DialogueManager::new();
        manager.register(
            Dialogue::new(DialogueId::new(1), "Test", 0).with_node(
                DialogueNode::new(0, "NPC", "What do you want?")
                    .with_choice(DialogueChoice::new("Talk", 1))
                    .with_choice(
                        DialogueChoice::new("Secret option", 2)
                            .with_condition(DialogueCondition::has_item(test_item(), 1)),
                    ),
            ),
        );

        let ctx = SimpleDialogueContext::new();
        manager.start(DialogueId::new(1)).unwrap();

        let choices = manager.available_choices(&ctx);
        assert_eq!(choices.len(), 1); // Only "Talk" is available

        let ctx_with_item = SimpleDialogueContext::new().with_item(test_item(), 1);
        let choices = manager.available_choices(&ctx_with_item);
        assert_eq!(choices.len(), 2); // Both available
    }

    #[test]
    fn test_dialogue_manager_effects() {
        let mut manager = DialogueManager::new();
        manager.register(
            Dialogue::new(DialogueId::new(1), "Test", 0).with_node(
                DialogueNode::new(0, "NPC", "Take this!").with_choice(
                    DialogueChoice::end("Thanks!")
                        .with_effect(DialogueEffect::give_item(test_item(), 5)),
                ),
            ),
        );

        let ctx = SimpleDialogueContext::new();
        manager.start(DialogueId::new(1)).unwrap();
        manager.select_choice(0, &ctx).unwrap();

        assert!(manager.has_pending_effects());
        let effects = manager.take_effects();
        assert_eq!(effects.len(), 1);
    }

    #[test]
    fn test_dialogue_manager_flags() {
        let mut manager = DialogueManager::new();

        assert!(!manager.is_flag_set("test_flag"));
        manager.set_flag("test_flag");
        assert!(manager.is_flag_set("test_flag"));
        manager.clear_flag("test_flag");
        assert!(!manager.is_flag_set("test_flag"));
    }

    #[test]
    fn test_dialogue_not_found() {
        let mut manager = DialogueManager::new();
        let result = manager.start(DialogueId::new(999));
        assert!(matches!(result, Err(DialogueError::NotFound(999))));
    }

    #[test]
    fn test_invalid_choice() {
        let mut manager = DialogueManager::new();
        manager.register(
            Dialogue::new(DialogueId::new(1), "Test", 0)
                .with_node(DialogueNode::new(0, "NPC", "Hello!")),
        );

        let ctx = SimpleDialogueContext::new();
        manager.start(DialogueId::new(1)).unwrap();

        let result = manager.select_choice(99, &ctx);
        assert!(matches!(result, Err(DialogueError::InvalidChoice(99))));
    }

    // DialogueEffect tests
    #[test]
    fn test_effect_multiple() {
        let effect = DialogueEffect::multiple(vec![
            DialogueEffect::give_item(test_item(), 5),
            DialogueEffect::start_quest(test_quest()),
        ]);

        if let DialogueEffect::Multiple(effects) = effect {
            assert_eq!(effects.len(), 2);
        } else {
            panic!("Expected Multiple effect");
        }
    }
}
