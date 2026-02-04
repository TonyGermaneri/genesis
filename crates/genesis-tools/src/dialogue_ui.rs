//! Dialogue system UI for NPC conversations.
//!
//! This module provides:
//! - Dialogue tree rendering with typewriter effect
//! - Choice selection and branching
//! - Speaker portraits and names
//! - Dialogue effects (mood, reputation, items)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for dialogue nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct DialogueNodeId(u64);

impl DialogueNodeId {
    /// Creates a new node ID.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Unique identifier for dialogue trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct DialogueTreeId(u64);

impl DialogueTreeId {
    /// Creates a new tree ID.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[must_use]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Speaker emotion/mood for portraits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpeakerMood {
    /// Default neutral mood
    #[default]
    Neutral,
    /// Happy/pleased
    Happy,
    /// Sad/disappointed
    Sad,
    /// Angry/hostile
    Angry,
    /// Surprised/shocked
    Surprised,
    /// Thoughtful/pondering
    Thoughtful,
    /// Worried/concerned
    Worried,
}

impl SpeakerMood {
    /// Returns a display string for this mood.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Neutral => "Neutral",
            Self::Happy => "Happy",
            Self::Sad => "Sad",
            Self::Angry => "Angry",
            Self::Surprised => "Surprised",
            Self::Thoughtful => "Thoughtful",
            Self::Worried => "Worried",
        }
    }

    /// Returns a portrait suffix for this mood.
    #[must_use]
    pub fn portrait_suffix(&self) -> &'static str {
        match self {
            Self::Neutral => "_neutral",
            Self::Happy => "_happy",
            Self::Sad => "_sad",
            Self::Angry => "_angry",
            Self::Surprised => "_surprised",
            Self::Thoughtful => "_thoughtful",
            Self::Worried => "_worried",
        }
    }
}

/// Effect that occurs during dialogue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogueEffect {
    /// Change reputation with a faction
    Reputation {
        /// Faction name
        faction: String,
        /// Amount to change (positive or negative)
        amount: i32,
    },
    /// Give item to player
    GiveItem {
        /// Item ID
        item_id: String,
        /// Quantity
        quantity: u32,
    },
    /// Take item from player
    TakeItem {
        /// Item ID
        item_id: String,
        /// Quantity
        quantity: u32,
    },
    /// Start a quest
    StartQuest {
        /// Quest ID
        quest_id: u64,
    },
    /// Complete a quest
    CompleteQuest {
        /// Quest ID
        quest_id: u64,
    },
    /// Set a dialogue flag
    SetFlag {
        /// Flag name
        name: String,
        /// Flag value
        value: bool,
    },
    /// Trigger a custom event
    CustomEvent {
        /// Event name
        name: String,
        /// Event data
        data: String,
    },
}

impl DialogueEffect {
    /// Creates a reputation change effect.
    #[must_use]
    pub fn reputation(faction: impl Into<String>, amount: i32) -> Self {
        Self::Reputation {
            faction: faction.into(),
            amount,
        }
    }

    /// Creates a give item effect.
    #[must_use]
    pub fn give_item(item_id: impl Into<String>, quantity: u32) -> Self {
        Self::GiveItem {
            item_id: item_id.into(),
            quantity,
        }
    }

    /// Creates a set flag effect.
    #[must_use]
    pub fn set_flag(name: impl Into<String>, value: bool) -> Self {
        Self::SetFlag {
            name: name.into(),
            value,
        }
    }
}

/// Condition for showing a dialogue choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogueCondition {
    /// Check if a flag is set
    HasFlag {
        /// Flag name
        name: String,
        /// Expected value
        value: bool,
    },
    /// Check if player has an item
    HasItem {
        /// Item ID
        item_id: String,
        /// Minimum quantity
        quantity: u32,
    },
    /// Check reputation level
    ReputationAtLeast {
        /// Faction name
        faction: String,
        /// Minimum reputation
        amount: i32,
    },
    /// Check if a quest is active
    QuestActive {
        /// Quest ID
        quest_id: u64,
    },
    /// Check if a quest is complete
    QuestComplete {
        /// Quest ID
        quest_id: u64,
    },
    /// Custom condition
    Custom {
        /// Condition name
        name: String,
    },
}

impl DialogueCondition {
    /// Creates a flag check condition.
    #[must_use]
    pub fn has_flag(name: impl Into<String>, value: bool) -> Self {
        Self::HasFlag {
            name: name.into(),
            value,
        }
    }

    /// Creates an item check condition.
    #[must_use]
    pub fn has_item(item_id: impl Into<String>, quantity: u32) -> Self {
        Self::HasItem {
            item_id: item_id.into(),
            quantity,
        }
    }

    /// Creates a reputation check condition.
    #[must_use]
    pub fn reputation_at_least(faction: impl Into<String>, amount: i32) -> Self {
        Self::ReputationAtLeast {
            faction: faction.into(),
            amount,
        }
    }
}

/// A single dialogue choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    /// Unique ID within the node
    pub id: u32,
    /// Text displayed to player
    pub text: String,
    /// Node to go to when selected
    pub next_node: Option<DialogueNodeId>,
    /// Conditions required to show this choice
    pub conditions: Vec<DialogueCondition>,
    /// Effects triggered when selected
    pub effects: Vec<DialogueEffect>,
    /// Whether this choice ends the dialogue
    pub ends_dialogue: bool,
}

impl DialogueChoice {
    /// Creates a new dialogue choice.
    #[must_use]
    pub fn new(id: u32, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
            next_node: None,
            conditions: Vec::new(),
            effects: Vec::new(),
            ends_dialogue: false,
        }
    }

    /// Sets the next node.
    #[must_use]
    pub fn with_next(mut self, node: DialogueNodeId) -> Self {
        self.next_node = Some(node);
        self
    }

    /// Adds a condition.
    #[must_use]
    pub fn with_condition(mut self, condition: DialogueCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Adds an effect.
    #[must_use]
    pub fn with_effect(mut self, effect: DialogueEffect) -> Self {
        self.effects.push(effect);
        self
    }

    /// Sets whether this choice ends dialogue.
    #[must_use]
    pub fn with_ends_dialogue(mut self, ends: bool) -> Self {
        self.ends_dialogue = ends;
        self
    }
}

/// A single node in a dialogue tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// Node ID
    pub id: DialogueNodeId,
    /// Speaker name
    pub speaker: String,
    /// Speaker mood/portrait
    pub mood: SpeakerMood,
    /// Dialogue text
    pub text: String,
    /// Available choices
    pub choices: Vec<DialogueChoice>,
    /// Effects triggered when entering this node
    pub on_enter: Vec<DialogueEffect>,
    /// Auto-advance delay (None = wait for input)
    pub auto_advance: Option<f32>,
}

impl DialogueNode {
    /// Creates a new dialogue node.
    #[must_use]
    pub fn new(id: DialogueNodeId, speaker: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id,
            speaker: speaker.into(),
            mood: SpeakerMood::Neutral,
            text: text.into(),
            choices: Vec::new(),
            on_enter: Vec::new(),
            auto_advance: None,
        }
    }

    /// Sets the speaker mood.
    #[must_use]
    pub fn with_mood(mut self, mood: SpeakerMood) -> Self {
        self.mood = mood;
        self
    }

    /// Adds a choice.
    #[must_use]
    pub fn with_choice(mut self, choice: DialogueChoice) -> Self {
        self.choices.push(choice);
        self
    }

    /// Adds an on-enter effect.
    #[must_use]
    pub fn with_on_enter(mut self, effect: DialogueEffect) -> Self {
        self.on_enter.push(effect);
        self
    }

    /// Sets auto-advance delay.
    #[must_use]
    pub fn with_auto_advance(mut self, delay: f32) -> Self {
        self.auto_advance = Some(delay);
        self
    }

    /// Returns whether this node has choices.
    #[must_use]
    pub fn has_choices(&self) -> bool {
        !self.choices.is_empty()
    }

    /// Returns whether this is an end node.
    #[must_use]
    pub fn is_end_node(&self) -> bool {
        self.choices.is_empty() && self.auto_advance.is_none()
    }
}

/// A complete dialogue tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    /// Tree ID
    pub id: DialogueTreeId,
    /// Tree name/title
    pub name: String,
    /// Starting node ID
    pub start_node: DialogueNodeId,
    /// All nodes in the tree
    pub nodes: HashMap<DialogueNodeId, DialogueNode>,
}

impl DialogueTree {
    /// Creates a new dialogue tree.
    #[must_use]
    pub fn new(id: DialogueTreeId, name: impl Into<String>, start_node: DialogueNodeId) -> Self {
        Self {
            id,
            name: name.into(),
            start_node,
            nodes: HashMap::new(),
        }
    }

    /// Adds a node to the tree.
    pub fn add_node(&mut self, node: DialogueNode) {
        self.nodes.insert(node.id, node);
    }

    /// Gets a node by ID.
    #[must_use]
    pub fn get_node(&self, id: DialogueNodeId) -> Option<&DialogueNode> {
        self.nodes.get(&id)
    }

    /// Gets the starting node.
    #[must_use]
    pub fn get_start_node(&self) -> Option<&DialogueNode> {
        self.nodes.get(&self.start_node)
    }
}

/// Typewriter effect state.
#[derive(Debug, Clone, Default)]
pub struct TypewriterState {
    /// Full text to display
    full_text: String,
    /// Current character index
    char_index: usize,
    /// Time since last character
    char_timer: f32,
    /// Whether typing is complete
    complete: bool,
}

impl TypewriterState {
    /// Creates a new typewriter state.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let full_text = text.into();
        let complete = full_text.is_empty();
        Self {
            full_text,
            char_index: 0,
            char_timer: 0.0,
            complete,
        }
    }

    /// Resets with new text.
    pub fn reset(&mut self, text: impl Into<String>) {
        self.full_text = text.into();
        self.char_index = 0;
        self.char_timer = 0.0;
        self.complete = self.full_text.is_empty();
    }

    /// Updates the typewriter effect.
    pub fn update(&mut self, dt: f32, chars_per_second: f32) {
        if self.complete {
            return;
        }

        self.char_timer += dt;
        let char_delay = 1.0 / chars_per_second;

        while self.char_timer >= char_delay && !self.complete {
            self.char_timer -= char_delay;
            self.char_index += 1;

            if self.char_index >= self.full_text.chars().count() {
                self.complete = true;
                self.char_index = self.full_text.chars().count();
            }
        }
    }

    /// Skips to the end of the text.
    pub fn skip(&mut self) {
        self.char_index = self.full_text.chars().count();
        self.complete = true;
    }

    /// Returns the currently visible text.
    #[must_use]
    pub fn visible_text(&self) -> String {
        self.full_text.chars().take(self.char_index).collect()
    }

    /// Returns whether typing is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Returns the progress (0.0 - 1.0).
    #[must_use]
    pub fn progress(&self) -> f32 {
        let total = self.full_text.chars().count();
        if total == 0 {
            return 1.0;
        }
        self.char_index as f32 / total as f32
    }
}

/// Active dialogue session state.
#[derive(Debug, Clone)]
pub struct DialogueSession {
    /// Current dialogue tree
    tree_id: DialogueTreeId,
    /// Current node
    current_node: DialogueNodeId,
    /// Typewriter state
    typewriter: TypewriterState,
    /// Available choices (filtered by conditions)
    available_choices: Vec<DialogueChoice>,
    /// Triggered effects to process
    pending_effects: Vec<DialogueEffect>,
    /// Auto-advance timer
    auto_timer: Option<f32>,
    /// Whether choices are visible
    choices_visible: bool,
}

impl DialogueSession {
    /// Creates a new session.
    #[must_use]
    pub fn new(tree_id: DialogueTreeId, start_node: DialogueNodeId, text: String) -> Self {
        Self {
            tree_id,
            current_node: start_node,
            typewriter: TypewriterState::new(text),
            available_choices: Vec::new(),
            pending_effects: Vec::new(),
            auto_timer: None,
            choices_visible: false,
        }
    }

    /// Returns the current node ID.
    #[must_use]
    pub fn current_node(&self) -> DialogueNodeId {
        self.current_node
    }

    /// Returns the tree ID.
    #[must_use]
    pub fn tree_id(&self) -> DialogueTreeId {
        self.tree_id
    }

    /// Returns the typewriter state.
    #[must_use]
    pub fn typewriter(&self) -> &TypewriterState {
        &self.typewriter
    }

    /// Returns the visible text.
    #[must_use]
    pub fn visible_text(&self) -> String {
        self.typewriter.visible_text()
    }

    /// Returns whether typing is complete.
    #[must_use]
    pub fn is_typing_complete(&self) -> bool {
        self.typewriter.is_complete()
    }

    /// Returns the available choices.
    #[must_use]
    pub fn available_choices(&self) -> &[DialogueChoice] {
        &self.available_choices
    }

    /// Returns whether choices are visible.
    #[must_use]
    pub fn choices_visible(&self) -> bool {
        self.choices_visible
    }

    /// Drains pending effects.
    pub fn drain_effects(&mut self) -> Vec<DialogueEffect> {
        std::mem::take(&mut self.pending_effects)
    }

    /// Skips the typewriter effect.
    pub fn skip_typewriter(&mut self) {
        self.typewriter.skip();
    }

    /// Shows choices (after typing complete).
    pub fn show_choices(&mut self) {
        if self.typewriter.is_complete() {
            self.choices_visible = true;
        }
    }

    /// Sets available choices.
    pub fn set_choices(&mut self, choices: Vec<DialogueChoice>) {
        self.available_choices = choices;
    }

    /// Advances to a new node.
    pub fn advance(&mut self, node_id: DialogueNodeId, text: String, effects: Vec<DialogueEffect>) {
        self.current_node = node_id;
        self.typewriter.reset(text);
        self.available_choices.clear();
        self.choices_visible = false;
        self.pending_effects.extend(effects);
        self.auto_timer = None;
    }

    /// Sets auto-advance timer.
    pub fn set_auto_advance(&mut self, delay: f32) {
        self.auto_timer = Some(delay);
    }

    /// Updates the session.
    pub fn update(&mut self, dt: f32, chars_per_second: f32) -> bool {
        self.typewriter.update(dt, chars_per_second);

        // Handle auto-advance
        if let Some(timer) = &mut self.auto_timer {
            if self.typewriter.is_complete() {
                *timer -= dt;
                if *timer <= 0.0 {
                    return true; // Signal auto-advance
                }
            }
        }

        false
    }
}

/// Dialogue UI configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueUIConfig {
    /// Characters per second for typewriter
    pub chars_per_second: f32,
    /// Panel width
    pub panel_width: f32,
    /// Panel height
    pub panel_height: f32,
    /// Portrait size
    pub portrait_size: f32,
    /// Text font size
    pub text_size: f32,
    /// Choice font size
    pub choice_size: f32,
    /// Panel opacity
    pub opacity: f32,
    /// Show speaker name
    pub show_speaker_name: bool,
    /// Show continue indicator
    pub show_continue_indicator: bool,
}

impl Default for DialogueUIConfig {
    fn default() -> Self {
        Self {
            chars_per_second: 30.0,
            panel_width: 800.0,
            panel_height: 200.0,
            portrait_size: 128.0,
            text_size: 16.0,
            choice_size: 14.0,
            opacity: 0.95,
            show_speaker_name: true,
            show_continue_indicator: true,
        }
    }
}

/// UI action triggered by dialogue.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogueAction {
    /// Continue to next node (no choices)
    Continue,
    /// Skip typewriter effect
    Skip,
    /// Select a choice
    SelectChoice(u32),
    /// Close dialogue
    Close,
}

/// Dialogue condition checker trait.
pub trait ConditionChecker {
    /// Checks if a condition is met.
    fn check(&self, condition: &DialogueCondition) -> bool;
}

/// Default condition checker that always returns true.
#[derive(Debug, Clone, Default)]
pub struct AlwaysTrueChecker;

impl ConditionChecker for AlwaysTrueChecker {
    fn check(&self, _condition: &DialogueCondition) -> bool {
        true
    }
}

/// Flag-based condition checker.
#[derive(Debug, Clone, Default)]
pub struct FlagChecker {
    flags: HashMap<String, bool>,
}

impl FlagChecker {
    /// Creates a new flag checker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a flag.
    pub fn set_flag(&mut self, name: impl Into<String>, value: bool) {
        self.flags.insert(name.into(), value);
    }

    /// Gets a flag.
    #[must_use]
    pub fn get_flag(&self, name: &str) -> Option<bool> {
        self.flags.get(name).copied()
    }
}

impl ConditionChecker for FlagChecker {
    fn check(&self, condition: &DialogueCondition) -> bool {
        match condition {
            DialogueCondition::HasFlag { name, value } => {
                self.flags.get(name).copied().unwrap_or(false) == *value
            },
            _ => true, // Other conditions not handled by this checker
        }
    }
}

/// Dialogue UI model (state).
#[derive(Debug, Clone)]
pub struct DialogueUIModel {
    /// Loaded dialogue trees
    trees: HashMap<DialogueTreeId, DialogueTree>,
    /// Active session
    session: Option<DialogueSession>,
    /// Configuration
    config: DialogueUIConfig,
    /// Selected choice index
    selected_choice: usize,
    /// Dialogue history
    history: Vec<(String, String)>, // (speaker, text)
}

impl Default for DialogueUIModel {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueUIModel {
    /// Creates a new dialogue UI model.
    #[must_use]
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            session: None,
            config: DialogueUIConfig::default(),
            selected_choice: 0,
            history: Vec::new(),
        }
    }

    /// Registers a dialogue tree.
    pub fn register_tree(&mut self, tree: DialogueTree) {
        self.trees.insert(tree.id, tree);
    }

    /// Unregisters a dialogue tree.
    pub fn unregister_tree(&mut self, id: DialogueTreeId) {
        self.trees.remove(&id);
    }

    /// Gets a tree by ID.
    #[must_use]
    pub fn get_tree(&self, id: DialogueTreeId) -> Option<&DialogueTree> {
        self.trees.get(&id)
    }

    /// Returns whether a dialogue is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.session.is_some()
    }

    /// Returns the active session.
    #[must_use]
    pub fn session(&self) -> Option<&DialogueSession> {
        self.session.as_ref()
    }

    /// Returns a mutable reference to the active session.
    pub fn session_mut(&mut self) -> Option<&mut DialogueSession> {
        self.session.as_mut()
    }

    /// Starts a dialogue.
    pub fn start_dialogue<C: ConditionChecker>(&mut self, tree_id: DialogueTreeId, checker: &C) {
        if let Some(tree) = self.trees.get(&tree_id) {
            if let Some(node) = tree.get_start_node() {
                let mut session = DialogueSession::new(tree_id, node.id, node.text.clone());

                // Add on-enter effects
                session.pending_effects.extend(node.on_enter.clone());

                // Filter choices by conditions
                let choices: Vec<_> = node
                    .choices
                    .iter()
                    .filter(|c| c.conditions.iter().all(|cond| checker.check(cond)))
                    .cloned()
                    .collect();
                session.set_choices(choices);

                // Set auto-advance if present
                if let Some(delay) = node.auto_advance {
                    session.set_auto_advance(delay);
                }

                // Record history
                self.history.push((node.speaker.clone(), node.text.clone()));

                self.session = Some(session);
                self.selected_choice = 0;
            }
        }
    }

    /// Ends the current dialogue.
    pub fn end_dialogue(&mut self) {
        self.session = None;
        self.selected_choice = 0;
    }

    /// Advances to a specific node.
    pub fn advance_to_node<C: ConditionChecker>(&mut self, node_id: DialogueNodeId, checker: &C) {
        let session = match &mut self.session {
            Some(s) => s,
            None => return,
        };

        let tree_id = session.tree_id();
        let tree = match self.trees.get(&tree_id) {
            Some(t) => t,
            None => return,
        };

        let node = match tree.get_node(node_id) {
            Some(n) => n,
            None => return,
        };

        // Record history
        self.history.push((node.speaker.clone(), node.text.clone()));

        // Advance session
        session.advance(node_id, node.text.clone(), node.on_enter.clone());

        // Filter choices
        let choices: Vec<_> = node
            .choices
            .iter()
            .filter(|c| c.conditions.iter().all(|cond| checker.check(cond)))
            .cloned()
            .collect();
        session.set_choices(choices);

        // Set auto-advance
        if let Some(delay) = node.auto_advance {
            session.set_auto_advance(delay);
        }

        self.selected_choice = 0;
    }

    /// Selects a choice by index.
    pub fn select_choice<C: ConditionChecker>(&mut self, index: usize, checker: &C) {
        // Extract needed data first to avoid borrow conflicts
        let (_effects, ends_dialogue, next_node) = {
            let session = match &mut self.session {
                Some(s) => s,
                None => return,
            };

            let choices = session.available_choices().to_vec();
            if index >= choices.len() {
                return;
            }

            let choice = &choices[index];

            // Add effects to session
            session.pending_effects.extend(choice.effects.clone());

            (
                choice.effects.clone(),
                choice.ends_dialogue,
                choice.next_node,
            )
        };

        // Now we can call methods that borrow self mutably
        if ends_dialogue {
            self.end_dialogue();
        } else if let Some(next_node) = next_node {
            self.advance_to_node(next_node, checker);
        }
    }

    /// Selects the next choice.
    pub fn select_next_choice(&mut self) {
        if let Some(session) = &self.session {
            let count = session.available_choices().len();
            if count > 0 {
                self.selected_choice = (self.selected_choice + 1) % count;
            }
        }
    }

    /// Selects the previous choice.
    pub fn select_prev_choice(&mut self) {
        if let Some(session) = &self.session {
            let count = session.available_choices().len();
            if count > 0 {
                self.selected_choice = if self.selected_choice == 0 {
                    count - 1
                } else {
                    self.selected_choice - 1
                };
            }
        }
    }

    /// Returns the selected choice index.
    #[must_use]
    pub fn selected_choice(&self) -> usize {
        self.selected_choice
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &DialogueUIConfig {
        &self.config
    }

    /// Returns mutable configuration.
    pub fn config_mut(&mut self) -> &mut DialogueUIConfig {
        &mut self.config
    }

    /// Returns the dialogue history.
    #[must_use]
    pub fn history(&self) -> &[(String, String)] {
        &self.history
    }

    /// Clears the dialogue history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Updates the model.
    pub fn update<C: ConditionChecker>(&mut self, dt: f32, checker: &C) {
        let chars_per_second = self.config.chars_per_second;

        // Extract auto-advance info first
        let next_node_for_auto_advance = {
            let session = match &mut self.session {
                Some(s) => s,
                None => return,
            };

            let should_auto_advance = session.update(dt, chars_per_second);

            // Show choices when typing complete
            if session.is_typing_complete() && !session.choices_visible() {
                session.show_choices();
            }

            if should_auto_advance {
                let choices = session.available_choices();
                if choices.len() == 1 {
                    choices[0].next_node
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Now handle auto-advance outside of borrow
        if let Some(next) = next_node_for_auto_advance {
            self.advance_to_node(next, checker);
        }
    }

    /// Gets the current speaker name.
    #[must_use]
    pub fn current_speaker(&self) -> Option<String> {
        let session = self.session.as_ref()?;
        let tree = self.trees.get(&session.tree_id())?;
        let node = tree.get_node(session.current_node())?;
        Some(node.speaker.clone())
    }

    /// Gets the current speaker mood.
    #[must_use]
    pub fn current_mood(&self) -> Option<SpeakerMood> {
        let session = self.session.as_ref()?;
        let tree = self.trees.get(&session.tree_id())?;
        let node = tree.get_node(session.current_node())?;
        Some(node.mood)
    }
}

/// Dialogue UI widget.
#[derive(Debug)]
pub struct DialogueUI {
    /// Pending actions
    actions: Vec<DialogueAction>,
}

impl Default for DialogueUI {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueUI {
    /// Creates a new dialogue UI.
    #[must_use]
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Drains pending actions.
    pub fn drain_actions(&mut self) -> Vec<DialogueAction> {
        std::mem::take(&mut self.actions)
    }

    /// Handles an input action.
    pub fn handle_input(&mut self, model: &mut DialogueUIModel, action: DialogueAction) {
        match &action {
            DialogueAction::Skip => {
                if let Some(session) = model.session_mut() {
                    session.skip_typewriter();
                }
            },
            DialogueAction::Continue => {
                if let Some(session) = model.session() {
                    if session.is_typing_complete() {
                        let choices = session.available_choices();
                        if choices.is_empty() {
                            self.actions.push(DialogueAction::Close);
                        }
                    } else {
                        self.actions.push(DialogueAction::Skip);
                    }
                }
            },
            DialogueAction::SelectChoice(index) => {
                model.select_choice(*index as usize, &AlwaysTrueChecker);
            },
            DialogueAction::Close => {
                model.end_dialogue();
            },
        }
        self.actions.push(action);
    }

    /// Renders the dialogue UI.
    pub fn render(&mut self, ctx: &egui::Context, model: &DialogueUIModel) {
        let session = match model.session() {
            Some(s) => s,
            None => return,
        };

        let config = model.config();

        egui::Area::new(egui::Id::new("dialogue_panel"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -50.0))
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(
                        20,
                        20,
                        30,
                        (config.opacity * 255.0) as u8,
                    ))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(16.0))
                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(60, 60, 80)))
                    .show(ui, |ui| {
                        ui.set_width(config.panel_width);
                        ui.set_min_height(config.panel_height);

                        // Speaker name
                        if config.show_speaker_name {
                            if let Some(speaker) = model.current_speaker() {
                                ui.label(
                                    egui::RichText::new(&speaker)
                                        .size(config.text_size + 2.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(200, 180, 100)),
                                );
                                ui.add_space(8.0);
                            }
                        }

                        // Dialogue text with typewriter effect
                        let visible_text = session.visible_text();
                        ui.label(
                            egui::RichText::new(&visible_text)
                                .size(config.text_size)
                                .color(egui::Color32::WHITE),
                        );

                        // Continue indicator or choices
                        if session.is_typing_complete() {
                            ui.add_space(16.0);

                            let choices = session.available_choices();
                            if choices.is_empty() {
                                // Show continue indicator
                                if config.show_continue_indicator {
                                    ui.horizontal(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add(egui::Label::new(
                                                        egui::RichText::new("▼ Continue")
                                                            .size(config.choice_size)
                                                            .color(egui::Color32::LIGHT_GRAY),
                                                    ))
                                                    .clicked()
                                                {
                                                    self.actions.push(DialogueAction::Continue);
                                                }
                                            },
                                        );
                                    });
                                }
                            } else if session.choices_visible() {
                                // Show choices
                                ui.separator();
                                for (i, choice) in choices.iter().enumerate() {
                                    let is_selected = i == model.selected_choice();
                                    let color = if is_selected {
                                        egui::Color32::from_rgb(255, 220, 100)
                                    } else {
                                        egui::Color32::LIGHT_GRAY
                                    };

                                    let prefix = if is_selected { "▶ " } else { "  " };
                                    let text = format!("{prefix}{}", choice.text);

                                    if ui
                                        .add(
                                            egui::Label::new(
                                                egui::RichText::new(&text)
                                                    .size(config.choice_size)
                                                    .color(color),
                                            )
                                            .sense(egui::Sense::click()),
                                        )
                                        .clicked()
                                    {
                                        self.actions.push(DialogueAction::SelectChoice(choice.id));
                                    }
                                }
                            }
                        } else {
                            // Show skip hint
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new("[Click to skip]")
                                                .size(10.0)
                                                .color(egui::Color32::DARK_GRAY),
                                        );
                                    },
                                );
                            });
                        }
                    });
            });

        // Handle click on panel for skip/continue
        let response = ctx.input(|i| i.pointer.any_click());
        if response {
            if let Some(session) = model.session() {
                if !session.is_typing_complete() {
                    self.actions.push(DialogueAction::Skip);
                } else if session.available_choices().is_empty() {
                    self.actions.push(DialogueAction::Continue);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_node_id() {
        let id = DialogueNodeId::new(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_dialogue_tree_id() {
        let id = DialogueTreeId::new(123);
        assert_eq!(id.raw(), 123);
    }

    #[test]
    fn test_speaker_mood() {
        assert_eq!(SpeakerMood::Neutral.display_name(), "Neutral");
        assert_eq!(SpeakerMood::Happy.portrait_suffix(), "_happy");
        assert_eq!(SpeakerMood::Angry.portrait_suffix(), "_angry");
    }

    #[test]
    fn test_dialogue_effect_factories() {
        let rep = DialogueEffect::reputation("TestFaction", 10);
        if let DialogueEffect::Reputation { faction, amount } = rep {
            assert_eq!(faction, "TestFaction");
            assert_eq!(amount, 10);
        } else {
            panic!("Wrong effect type");
        }

        let item = DialogueEffect::give_item("sword", 1);
        if let DialogueEffect::GiveItem { item_id, quantity } = item {
            assert_eq!(item_id, "sword");
            assert_eq!(quantity, 1);
        } else {
            panic!("Wrong effect type");
        }
    }

    #[test]
    fn test_dialogue_condition_factories() {
        let flag = DialogueCondition::has_flag("talked", true);
        if let DialogueCondition::HasFlag { name, value } = flag {
            assert_eq!(name, "talked");
            assert!(value);
        } else {
            panic!("Wrong condition type");
        }

        let item = DialogueCondition::has_item("key", 1);
        if let DialogueCondition::HasItem { item_id, quantity } = item {
            assert_eq!(item_id, "key");
            assert_eq!(quantity, 1);
        } else {
            panic!("Wrong condition type");
        }
    }

    #[test]
    fn test_dialogue_choice_new() {
        let choice = DialogueChoice::new(1, "Hello there!")
            .with_next(DialogueNodeId::new(2))
            .with_ends_dialogue(false);

        assert_eq!(choice.id, 1);
        assert_eq!(choice.text, "Hello there!");
        assert_eq!(choice.next_node, Some(DialogueNodeId::new(2)));
        assert!(!choice.ends_dialogue);
    }

    #[test]
    fn test_dialogue_choice_with_condition() {
        let choice =
            DialogueChoice::new(1, "Test").with_condition(DialogueCondition::has_flag("met", true));

        assert_eq!(choice.conditions.len(), 1);
    }

    #[test]
    fn test_dialogue_choice_with_effect() {
        let choice =
            DialogueChoice::new(1, "Test").with_effect(DialogueEffect::reputation("Guild", 5));

        assert_eq!(choice.effects.len(), 1);
    }

    #[test]
    fn test_dialogue_node_new() {
        let node = DialogueNode::new(DialogueNodeId::new(1), "NPC", "Hello traveler!")
            .with_mood(SpeakerMood::Happy);

        assert_eq!(node.speaker, "NPC");
        assert_eq!(node.text, "Hello traveler!");
        assert_eq!(node.mood, SpeakerMood::Happy);
        assert!(!node.has_choices());
        assert!(node.is_end_node());
    }

    #[test]
    fn test_dialogue_node_with_choices() {
        let node = DialogueNode::new(DialogueNodeId::new(1), "NPC", "Hello!")
            .with_choice(DialogueChoice::new(1, "Hi!"));

        assert!(node.has_choices());
        assert!(!node.is_end_node());
    }

    #[test]
    fn test_dialogue_node_with_auto_advance() {
        let node = DialogueNode::new(DialogueNodeId::new(1), "NPC", "...").with_auto_advance(2.0);

        assert!(!node.is_end_node());
        assert_eq!(node.auto_advance, Some(2.0));
    }

    #[test]
    fn test_dialogue_tree_new() {
        let mut tree = DialogueTree::new(
            DialogueTreeId::new(1),
            "Test Dialogue",
            DialogueNodeId::new(1),
        );

        let node = DialogueNode::new(DialogueNodeId::new(1), "NPC", "Hello!");
        tree.add_node(node);

        assert_eq!(tree.name, "Test Dialogue");
        assert!(tree.get_node(DialogueNodeId::new(1)).is_some());
        assert!(tree.get_start_node().is_some());
    }

    #[test]
    fn test_typewriter_state_new() {
        let tw = TypewriterState::new("Hello");
        assert!(!tw.is_complete());
        assert_eq!(tw.visible_text(), "");
        assert!((tw.progress() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_typewriter_state_update() {
        let mut tw = TypewriterState::new("Hi");
        tw.update(0.1, 10.0); // 10 chars/sec, 0.1s = 1 char
        assert_eq!(tw.visible_text(), "H");

        tw.update(0.1, 10.0);
        assert_eq!(tw.visible_text(), "Hi");
        assert!(tw.is_complete());
    }

    #[test]
    fn test_typewriter_state_skip() {
        let mut tw = TypewriterState::new("Hello World");
        assert!(!tw.is_complete());

        tw.skip();
        assert!(tw.is_complete());
        assert_eq!(tw.visible_text(), "Hello World");
    }

    #[test]
    fn test_typewriter_state_reset() {
        let mut tw = TypewriterState::new("First");
        tw.skip();
        assert!(tw.is_complete());

        tw.reset("Second");
        assert!(!tw.is_complete());
        assert_eq!(tw.visible_text(), "");
    }

    #[test]
    fn test_typewriter_empty_text() {
        let tw = TypewriterState::new("");
        assert!(tw.is_complete());
        assert!((tw.progress() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_dialogue_session_new() {
        let session = DialogueSession::new(
            DialogueTreeId::new(1),
            DialogueNodeId::new(1),
            "Hello".to_string(),
        );

        assert_eq!(session.tree_id(), DialogueTreeId::new(1));
        assert_eq!(session.current_node(), DialogueNodeId::new(1));
        assert!(!session.is_typing_complete());
    }

    #[test]
    fn test_dialogue_session_advance() {
        let mut session = DialogueSession::new(
            DialogueTreeId::new(1),
            DialogueNodeId::new(1),
            "First".to_string(),
        );

        session.advance(
            DialogueNodeId::new(2),
            "Second".to_string(),
            vec![DialogueEffect::set_flag("advanced", true)],
        );

        assert_eq!(session.current_node(), DialogueNodeId::new(2));
        assert_eq!(session.drain_effects().len(), 1);
    }

    #[test]
    fn test_dialogue_ui_config_defaults() {
        let config = DialogueUIConfig::default();
        assert!((config.chars_per_second - 30.0).abs() < 0.001);
        assert!(config.show_speaker_name);
        assert!(config.show_continue_indicator);
    }

    #[test]
    fn test_dialogue_action_equality() {
        let a1 = DialogueAction::SelectChoice(1);
        let a2 = DialogueAction::SelectChoice(1);
        assert_eq!(a1, a2);

        let a3 = DialogueAction::SelectChoice(2);
        assert_ne!(a1, a3);
    }

    #[test]
    fn test_flag_checker() {
        let mut checker = FlagChecker::new();
        checker.set_flag("test", true);

        assert!(checker.check(&DialogueCondition::has_flag("test", true)));
        assert!(!checker.check(&DialogueCondition::has_flag("test", false)));
        assert!(!checker.check(&DialogueCondition::has_flag("unknown", true)));
    }

    #[test]
    fn test_always_true_checker() {
        let checker = AlwaysTrueChecker;
        assert!(checker.check(&DialogueCondition::has_flag("any", true)));
        assert!(checker.check(&DialogueCondition::has_item("any", 999)));
    }

    #[test]
    fn test_dialogue_ui_model_new() {
        let model = DialogueUIModel::new();
        assert!(!model.is_active());
        assert!(model.session().is_none());
        assert!(model.history().is_empty());
    }

    #[test]
    fn test_dialogue_ui_model_register_tree() {
        let mut model = DialogueUIModel::new();
        let tree = DialogueTree::new(DialogueTreeId::new(1), "Test", DialogueNodeId::new(1));

        model.register_tree(tree);
        assert!(model.get_tree(DialogueTreeId::new(1)).is_some());

        model.unregister_tree(DialogueTreeId::new(1));
        assert!(model.get_tree(DialogueTreeId::new(1)).is_none());
    }

    #[test]
    fn test_dialogue_ui_model_start_dialogue() {
        let mut model = DialogueUIModel::new();

        let mut tree = DialogueTree::new(DialogueTreeId::new(1), "Test", DialogueNodeId::new(1));
        tree.add_node(DialogueNode::new(DialogueNodeId::new(1), "NPC", "Hello!"));
        model.register_tree(tree);

        model.start_dialogue(DialogueTreeId::new(1), &AlwaysTrueChecker);
        assert!(model.is_active());
        assert_eq!(model.history().len(), 1);
    }

    #[test]
    fn test_dialogue_ui_model_end_dialogue() {
        let mut model = DialogueUIModel::new();

        let mut tree = DialogueTree::new(DialogueTreeId::new(1), "Test", DialogueNodeId::new(1));
        tree.add_node(DialogueNode::new(DialogueNodeId::new(1), "NPC", "Hello!"));
        model.register_tree(tree);

        model.start_dialogue(DialogueTreeId::new(1), &AlwaysTrueChecker);
        assert!(model.is_active());

        model.end_dialogue();
        assert!(!model.is_active());
    }

    #[test]
    fn test_dialogue_ui_model_choice_navigation() {
        let mut model = DialogueUIModel::new();

        let mut tree = DialogueTree::new(DialogueTreeId::new(1), "Test", DialogueNodeId::new(1));
        tree.add_node(
            DialogueNode::new(DialogueNodeId::new(1), "NPC", "Choose:")
                .with_choice(DialogueChoice::new(1, "A"))
                .with_choice(DialogueChoice::new(2, "B"))
                .with_choice(DialogueChoice::new(3, "C")),
        );
        model.register_tree(tree);

        model.start_dialogue(DialogueTreeId::new(1), &AlwaysTrueChecker);
        assert_eq!(model.selected_choice(), 0);

        model.select_next_choice();
        assert_eq!(model.selected_choice(), 1);

        model.select_next_choice();
        assert_eq!(model.selected_choice(), 2);

        model.select_next_choice();
        assert_eq!(model.selected_choice(), 0); // Wraps around

        model.select_prev_choice();
        assert_eq!(model.selected_choice(), 2); // Wraps back
    }

    #[test]
    fn test_dialogue_ui_model_current_speaker() {
        let mut model = DialogueUIModel::new();

        let mut tree = DialogueTree::new(DialogueTreeId::new(1), "Test", DialogueNodeId::new(1));
        tree.add_node(
            DialogueNode::new(DialogueNodeId::new(1), "Elder", "Greetings!")
                .with_mood(SpeakerMood::Happy),
        );
        model.register_tree(tree);

        assert!(model.current_speaker().is_none());
        assert!(model.current_mood().is_none());

        model.start_dialogue(DialogueTreeId::new(1), &AlwaysTrueChecker);
        assert_eq!(model.current_speaker(), Some("Elder".to_string()));
        assert_eq!(model.current_mood(), Some(SpeakerMood::Happy));
    }

    #[test]
    fn test_dialogue_ui_model_clear_history() {
        let mut model = DialogueUIModel::new();

        let mut tree = DialogueTree::new(DialogueTreeId::new(1), "Test", DialogueNodeId::new(1));
        tree.add_node(DialogueNode::new(DialogueNodeId::new(1), "NPC", "Hello!"));
        model.register_tree(tree);

        model.start_dialogue(DialogueTreeId::new(1), &AlwaysTrueChecker);
        assert_eq!(model.history().len(), 1);

        model.clear_history();
        assert!(model.history().is_empty());
    }

    #[test]
    fn test_dialogue_ui_new() {
        let ui = DialogueUI::new();
        assert!(ui.actions.is_empty());
    }

    #[test]
    fn test_dialogue_ui_drain_actions() {
        let mut ui = DialogueUI::new();
        let mut model = DialogueUIModel::new();

        ui.handle_input(&mut model, DialogueAction::Skip);
        ui.handle_input(&mut model, DialogueAction::Continue);

        let actions = ui.drain_actions();
        assert_eq!(actions.len(), 2);

        let actions2 = ui.drain_actions();
        assert!(actions2.is_empty());
    }

    #[test]
    fn test_dialogue_session_choices() {
        let mut session = DialogueSession::new(
            DialogueTreeId::new(1),
            DialogueNodeId::new(1),
            "Test".to_string(),
        );

        assert!(!session.choices_visible());
        assert!(session.available_choices().is_empty());

        session.set_choices(vec![
            DialogueChoice::new(1, "A"),
            DialogueChoice::new(2, "B"),
        ]);
        assert_eq!(session.available_choices().len(), 2);

        session.skip_typewriter();
        session.show_choices();
        assert!(session.choices_visible());
    }
}
