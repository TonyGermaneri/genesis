//! NPC AI system with behavior trees.

use crate::combat::{AttackIntent, AttackTarget, AttackType, CombatSystem};
use genesis_common::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for NPC operations.
#[derive(Debug, Error)]
pub enum NPCError {
    /// NPC not found
    #[error("NPC not found: {0:?}")]
    NotFound(EntityId),
    /// Behavior tree not found
    #[error("Behavior tree not found for NPC type: {0:?}")]
    BehaviorNotFound(NPCType),
    /// Pathfinding failed
    #[error("Pathfinding failed from ({0}, {1}) to ({2}, {3})")]
    PathfindingFailed(f32, f32, f32, f32),
    /// NPC already registered
    #[error("NPC already registered: {0:?}")]
    AlreadyRegistered(EntityId),
}

/// Result type for NPC operations.
pub type NPCResult<T> = Result<T, NPCError>;

/// Type of NPC determining base behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NPCType {
    /// Flees from player when approached
    Passive,
    /// Only attacks if provoked
    Neutral,
    /// Attacks player on sight
    Hostile,
    /// Offers trading interactions
    Merchant,
    /// Defends a specific area
    Guard,
}

impl NPCType {
    /// Returns default aggro range for this NPC type.
    #[must_use]
    pub const fn default_aggro_range(&self) -> f32 {
        match self {
            NPCType::Passive => 5.0,  // Flee range
            NPCType::Neutral => 3.0,  // Low provoke range
            NPCType::Hostile => 10.0, // Detection range
            NPCType::Merchant => 0.0, // Never aggro
            NPCType::Guard => 15.0,   // Wide patrol view
        }
    }

    /// Returns default wander radius for this NPC type.
    #[must_use]
    pub const fn default_wander_radius(&self) -> f32 {
        match self {
            NPCType::Passive => 8.0,
            NPCType::Neutral => 5.0,
            NPCType::Hostile => 12.0,
            NPCType::Merchant => 2.0,
            NPCType::Guard => 3.0,
        }
    }

    /// Returns whether this NPC type can attack.
    #[must_use]
    pub const fn can_attack(&self) -> bool {
        matches!(self, NPCType::Neutral | NPCType::Hostile | NPCType::Guard)
    }

    /// Returns whether this NPC type flees from threats.
    #[must_use]
    pub const fn flees(&self) -> bool {
        matches!(self, NPCType::Passive | NPCType::Merchant)
    }
}

/// Unique identifier for behaviors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BehaviorId(pub u32);

impl BehaviorId {
    /// Idle behavior.
    pub const IDLE: Self = BehaviorId(0);
    /// Wander behavior.
    pub const WANDER: Self = BehaviorId(1);
    /// Chase behavior.
    pub const CHASE: Self = BehaviorId(2);
    /// Attack behavior.
    pub const ATTACK: Self = BehaviorId(3);
    /// Flee behavior.
    pub const FLEE: Self = BehaviorId(4);
    /// Return home behavior.
    pub const RETURN_HOME: Self = BehaviorId(5);
    /// Patrol behavior.
    pub const PATROL: Self = BehaviorId(6);
    /// Trade behavior.
    pub const TRADE: Self = BehaviorId(7);
}

/// State of an individual NPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCState {
    /// Type of NPC
    pub npc_type: NPCType,
    /// Current behavior being executed
    pub current_behavior: BehaviorId,
    /// Current target (usually player or enemy)
    pub target: Option<EntityId>,
    /// Home position for returning
    pub home_position: (f32, f32),
    /// Range at which NPC detects threats
    pub aggro_range: f32,
    /// Maximum wander distance from home
    pub wander_radius: f32,
    /// Last known position of player/target
    pub last_seen_player: Option<(f32, f32)>,
    /// Current NPC position
    pub position: (f32, f32),
    /// NPC facing direction (radians)
    pub facing: f32,
    /// Movement speed
    pub speed: f32,
    /// Attack cooldown remaining
    pub attack_cooldown: f32,
    /// Whether NPC has been provoked (for neutral)
    pub provoked: bool,
    /// Patrol waypoints
    pub patrol_waypoints: Vec<(f32, f32)>,
    /// Current patrol waypoint index
    pub patrol_index: usize,
    /// Time spent in current behavior
    pub behavior_time: f32,
    /// De-aggro timer
    pub deaggro_timer: f32,
}

impl NPCState {
    /// Creates a new NPC state with default values.
    #[must_use]
    pub fn new(npc_type: NPCType, position: (f32, f32)) -> Self {
        Self {
            npc_type,
            current_behavior: BehaviorId::IDLE,
            target: None,
            home_position: position,
            aggro_range: npc_type.default_aggro_range(),
            wander_radius: npc_type.default_wander_radius(),
            last_seen_player: None,
            position,
            facing: 0.0,
            speed: 3.0,
            attack_cooldown: 0.0,
            provoked: false,
            patrol_waypoints: Vec::new(),
            patrol_index: 0,
            behavior_time: 0.0,
            deaggro_timer: 0.0,
        }
    }

    /// Sets patrol waypoints.
    pub fn with_patrol(mut self, waypoints: Vec<(f32, f32)>) -> Self {
        self.patrol_waypoints = waypoints;
        self
    }

    /// Sets movement speed.
    #[must_use]
    pub const fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Sets aggro range.
    #[must_use]
    pub const fn with_aggro_range(mut self, range: f32) -> Self {
        self.aggro_range = range;
        self
    }

    /// Returns whether NPC is at home.
    #[must_use]
    pub fn is_at_home(&self) -> bool {
        distance(self.position, self.home_position) < 1.0
    }

    /// Returns whether NPC is too far from home.
    #[must_use]
    pub fn is_too_far_from_home(&self) -> bool {
        distance(self.position, self.home_position) > self.wander_radius * 2.0
    }

    /// Returns distance to a position.
    #[must_use]
    pub fn distance_to(&self, pos: (f32, f32)) -> f32 {
        distance(self.position, pos)
    }

    /// Returns direction to a position.
    #[must_use]
    pub fn direction_to(&self, pos: (f32, f32)) -> (f32, f32) {
        let dx = pos.0 - self.position.0;
        let dy = pos.1 - self.position.1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            (0.0, 0.0)
        } else {
            (dx / len, dy / len)
        }
    }
}

/// Actions an NPC can take.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NPCAction {
    /// Stand still
    Idle,
    /// Random wandering near home
    Wander,
    /// Follow patrol route
    Patrol {
        /// Patrol waypoints
        waypoints: Vec<(f32, f32)>,
    },
    /// Chase the current target
    ChaseTarget,
    /// Attack the current target
    AttackTarget,
    /// Run away from threats
    Flee,
    /// Return to home position
    ReturnHome,
    /// Engage in trading
    Trade,
    /// Move to specific position
    MoveTo {
        /// Target position
        destination: (f32, f32),
    },
}

/// Result of evaluating a behavior node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    /// Node succeeded
    Success,
    /// Node failed
    Failure,
    /// Node still running
    Running,
}

/// Condition that can be checked in behavior tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BehaviorCondition {
    /// Target is within specified range
    TargetInRange(f32),
    /// Target is visible (LOS check)
    TargetVisible,
    /// NPC has a target
    HasTarget,
    /// NPC is at home position
    IsAtHome,
    /// NPC is too far from home
    TooFarFromHome,
    /// NPC health is below percentage
    HealthBelow(f32),
    /// NPC has been provoked
    IsProvoked,
    /// NPC is within wander radius
    InWanderRadius,
    /// Attack is off cooldown
    CanAttack,
}

/// A node in the behavior tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorNode {
    /// Runs children in sequence until one fails
    Sequence(Vec<BehaviorNode>),
    /// Runs children until one succeeds (fallback)
    Selector(Vec<BehaviorNode>),
    /// Checks a condition
    Condition(BehaviorCondition),
    /// Inverts child result
    Inverter(Box<BehaviorNode>),
    /// Executes an action
    Action(NPCAction),
    /// Always succeeds
    AlwaysSucceed,
    /// Always fails
    AlwaysFail,
}

impl BehaviorNode {
    /// Creates a sequence node.
    #[must_use]
    pub fn sequence(children: Vec<BehaviorNode>) -> Self {
        BehaviorNode::Sequence(children)
    }

    /// Creates a selector node.
    #[must_use]
    pub fn selector(children: Vec<BehaviorNode>) -> Self {
        BehaviorNode::Selector(children)
    }

    /// Creates a condition node.
    #[must_use]
    pub fn condition(cond: BehaviorCondition) -> Self {
        BehaviorNode::Condition(cond)
    }

    /// Creates an action node.
    #[must_use]
    pub fn action(action: NPCAction) -> Self {
        BehaviorNode::Action(action)
    }

    /// Creates an inverter node.
    #[must_use]
    pub fn inverter(child: BehaviorNode) -> Self {
        BehaviorNode::Inverter(Box::new(child))
    }
}

/// A complete behavior tree for an NPC type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorTree {
    /// Root node of the tree
    root: BehaviorNode,
}

impl BehaviorTree {
    /// Creates a new behavior tree.
    #[must_use]
    pub fn new(root: BehaviorNode) -> Self {
        Self { root }
    }

    /// Returns the root node.
    #[must_use]
    pub fn root(&self) -> &BehaviorNode {
        &self.root
    }

    /// Creates a default hostile behavior tree.
    #[must_use]
    pub fn hostile() -> Self {
        Self::new(BehaviorNode::selector(vec![
            // If target in range and can attack, attack
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::condition(BehaviorCondition::TargetInRange(2.0)),
                BehaviorNode::condition(BehaviorCondition::CanAttack),
                BehaviorNode::action(NPCAction::AttackTarget),
            ]),
            // If has target, chase it
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::condition(BehaviorCondition::TargetInRange(15.0)),
                BehaviorNode::action(NPCAction::ChaseTarget),
            ]),
            // If too far from home, return
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::TooFarFromHome),
                BehaviorNode::action(NPCAction::ReturnHome),
            ]),
            // Otherwise wander
            BehaviorNode::action(NPCAction::Wander),
        ]))
    }

    /// Creates a default passive behavior tree.
    #[must_use]
    pub fn passive() -> Self {
        Self::new(BehaviorNode::selector(vec![
            // If target nearby, flee
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::condition(BehaviorCondition::TargetInRange(5.0)),
                BehaviorNode::action(NPCAction::Flee),
            ]),
            // If too far from home, return
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::TooFarFromHome),
                BehaviorNode::action(NPCAction::ReturnHome),
            ]),
            // Otherwise wander
            BehaviorNode::action(NPCAction::Wander),
        ]))
    }

    /// Creates a default neutral behavior tree.
    #[must_use]
    pub fn neutral() -> Self {
        Self::new(BehaviorNode::selector(vec![
            // If provoked and can attack, attack
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::IsProvoked),
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::condition(BehaviorCondition::TargetInRange(2.0)),
                BehaviorNode::condition(BehaviorCondition::CanAttack),
                BehaviorNode::action(NPCAction::AttackTarget),
            ]),
            // If provoked, chase
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::IsProvoked),
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::action(NPCAction::ChaseTarget),
            ]),
            // If too far from home, return
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::TooFarFromHome),
                BehaviorNode::action(NPCAction::ReturnHome),
            ]),
            // Otherwise wander
            BehaviorNode::action(NPCAction::Wander),
        ]))
    }

    /// Creates a guard behavior tree.
    #[must_use]
    pub fn guard() -> Self {
        Self::new(BehaviorNode::selector(vec![
            // If target in range and can attack, attack
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::condition(BehaviorCondition::TargetInRange(2.0)),
                BehaviorNode::condition(BehaviorCondition::CanAttack),
                BehaviorNode::action(NPCAction::AttackTarget),
            ]),
            // If has target within patrol area, chase
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::condition(BehaviorCondition::InWanderRadius),
                BehaviorNode::action(NPCAction::ChaseTarget),
            ]),
            // Patrol if has waypoints, otherwise idle
            BehaviorNode::action(NPCAction::Patrol {
                waypoints: Vec::new(),
            }),
        ]))
    }

    /// Creates a merchant behavior tree.
    #[must_use]
    pub fn merchant() -> Self {
        Self::new(BehaviorNode::selector(vec![
            // If player nearby, trade
            BehaviorNode::sequence(vec![
                BehaviorNode::condition(BehaviorCondition::HasTarget),
                BehaviorNode::condition(BehaviorCondition::TargetInRange(3.0)),
                BehaviorNode::action(NPCAction::Trade),
            ]),
            // Stay at home
            BehaviorNode::sequence(vec![
                BehaviorNode::inverter(BehaviorNode::condition(BehaviorCondition::IsAtHome)),
                BehaviorNode::action(NPCAction::ReturnHome),
            ]),
            // Idle
            BehaviorNode::action(NPCAction::Idle),
        ]))
    }
}

/// World interface for NPC pathfinding and LOS.
pub trait NPCWorld {
    /// Checks if there's line of sight between two positions.
    fn has_line_of_sight(&self, from: (f32, f32), to: (f32, f32)) -> bool;
    /// Gets the next waypoint on path from start to goal.
    fn get_next_waypoint(&self, from: (f32, f32), to: (f32, f32)) -> Option<(f32, f32)>;
    /// Checks if a position is walkable.
    fn is_walkable(&self, pos: (f32, f32)) -> bool;
}

/// Storage interface for NPC data.
pub trait NPCStorage {
    /// Gets NPC health percentage (0.0-1.0).
    fn get_health_percent(&self, entity: EntityId) -> Option<f32>;
    /// Gets entity position.
    fn get_position(&self, entity: EntityId) -> Option<(f32, f32)>;
    /// Sets entity position.
    fn set_position(&mut self, entity: EntityId, pos: (f32, f32));
    /// Gets entity facing.
    fn get_facing(&self, entity: EntityId) -> Option<f32>;
    /// Sets entity facing.
    fn set_facing(&mut self, entity: EntityId, facing: f32);
}

/// NPC manager handling all NPC behaviors.
#[derive(Debug, Default)]
pub struct NPCManager {
    /// All registered NPCs
    npcs: HashMap<EntityId, NPCState>,
    /// Behavior trees by NPC type
    behavior_trees: HashMap<NPCType, BehaviorTree>,
    /// Next entity ID for spawning
    next_id: u64,
    /// RNG state for wandering
    rng_state: u64,
}

impl NPCManager {
    /// Creates a new NPC manager with default behavior trees.
    #[must_use]
    pub fn new() -> Self {
        let mut behavior_trees = HashMap::new();
        behavior_trees.insert(NPCType::Hostile, BehaviorTree::hostile());
        behavior_trees.insert(NPCType::Passive, BehaviorTree::passive());
        behavior_trees.insert(NPCType::Neutral, BehaviorTree::neutral());
        behavior_trees.insert(NPCType::Guard, BehaviorTree::guard());
        behavior_trees.insert(NPCType::Merchant, BehaviorTree::merchant());

        Self {
            npcs: HashMap::new(),
            behavior_trees,
            next_id: 1,
            rng_state: 12345,
        }
    }

    /// Registers a custom behavior tree for an NPC type.
    pub fn register_behavior_tree(&mut self, npc_type: NPCType, tree: BehaviorTree) {
        self.behavior_trees.insert(npc_type, tree);
    }

    /// Returns the number of NPCs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.npcs.len()
    }

    /// Returns whether there are no NPCs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.npcs.is_empty()
    }

    /// Gets an NPC's state.
    #[must_use]
    pub fn get(&self, entity: EntityId) -> Option<&NPCState> {
        self.npcs.get(&entity)
    }

    /// Gets mutable NPC state.
    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut NPCState> {
        self.npcs.get_mut(&entity)
    }

    /// Spawns a new NPC.
    pub fn spawn_npc(&mut self, npc_type: NPCType, position: (f32, f32)) -> EntityId {
        let id = EntityId::from_raw(self.next_id);
        self.next_id += 1;

        let state = NPCState::new(npc_type, position);
        self.npcs.insert(id, state);

        id
    }

    /// Registers an existing entity as an NPC.
    pub fn register_npc(
        &mut self,
        entity: EntityId,
        npc_type: NPCType,
        position: (f32, f32),
    ) -> NPCResult<()> {
        if self.npcs.contains_key(&entity) {
            return Err(NPCError::AlreadyRegistered(entity));
        }
        let state = NPCState::new(npc_type, position);
        self.npcs.insert(entity, state);
        Ok(())
    }

    /// Removes an NPC.
    pub fn despawn_npc(&mut self, entity: EntityId) -> NPCResult<NPCState> {
        self.npcs.remove(&entity).ok_or(NPCError::NotFound(entity))
    }

    /// Provokes an NPC (for neutral types).
    pub fn provoke(&mut self, entity: EntityId, attacker: EntityId) {
        if let Some(npc) = self.npcs.get_mut(&entity) {
            if npc.npc_type == NPCType::Neutral {
                npc.provoked = true;
                npc.target = Some(attacker);
                npc.deaggro_timer = 10.0; // Reset deaggro timer
            }
        }
    }

    /// Updates all NPCs.
    pub fn update<W: NPCWorld, S: NPCStorage>(
        &mut self,
        dt: f32,
        player_id: EntityId,
        player_pos: (f32, f32),
        world: &W,
        storage: &mut S,
        combat: &mut CombatSystem,
    ) {
        // Collect NPC IDs to process
        let npc_ids: Vec<EntityId> = self.npcs.keys().copied().collect();

        for npc_id in npc_ids {
            // Clone NPC state to avoid borrow issues
            let npc_state = match self.npcs.get(&npc_id) {
                Some(state) => state.clone(),
                None => continue,
            };

            // Evaluate behavior tree
            let action = self.evaluate_behavior(&npc_state, player_id, player_pos, world);

            // Execute action and get updated state
            let updated_state =
                self.execute_action(npc_id, npc_state, action, dt, player_pos, world, combat);

            // Update the stored state
            if let Some(state) = self.npcs.get_mut(&npc_id) {
                *state = updated_state;
            }

            // Sync position to storage
            if let Some(state) = self.npcs.get(&npc_id) {
                storage.set_position(npc_id, state.position);
                storage.set_facing(npc_id, state.facing);
            }
        }
    }

    /// Evaluates behavior tree and returns action.
    fn evaluate_behavior<W: NPCWorld>(
        &self,
        npc: &NPCState,
        _player_id: EntityId,
        player_pos: (f32, f32),
        world: &W,
    ) -> NPCAction {
        let tree = match self.behavior_trees.get(&npc.npc_type) {
            Some(tree) => tree,
            None => return NPCAction::Idle,
        };

        // Create evaluation context
        let ctx = EvalContext {
            npc,
            player_pos,
            world,
        };

        // Evaluate tree
        match Self::evaluate_node(&ctx, tree.root()) {
            (BehaviorStatus::Success | BehaviorStatus::Running, Some(action)) => action,
            _ => NPCAction::Idle,
        }
    }

    /// Evaluates a single behavior node.
    fn evaluate_node<W: NPCWorld>(
        ctx: &EvalContext<'_, W>,
        node: &BehaviorNode,
    ) -> (BehaviorStatus, Option<NPCAction>) {
        match node {
            BehaviorNode::Sequence(children) => {
                let mut last_action = None;
                for child in children {
                    let (status, action) = Self::evaluate_node(ctx, child);
                    if status == BehaviorStatus::Failure {
                        return (BehaviorStatus::Failure, None);
                    }
                    if action.is_some() {
                        last_action = action;
                    }
                }
                (BehaviorStatus::Success, last_action)
            },
            BehaviorNode::Selector(children) => {
                for child in children {
                    let (status, action) = Self::evaluate_node(ctx, child);
                    if status != BehaviorStatus::Failure {
                        return (status, action);
                    }
                }
                (BehaviorStatus::Failure, None)
            },
            BehaviorNode::Condition(cond) => {
                let success = Self::evaluate_condition(ctx, cond);
                if success {
                    (BehaviorStatus::Success, None)
                } else {
                    (BehaviorStatus::Failure, None)
                }
            },
            BehaviorNode::Inverter(child) => {
                let (status, action) = Self::evaluate_node(ctx, child);
                let inverted = match status {
                    BehaviorStatus::Success => BehaviorStatus::Failure,
                    BehaviorStatus::Failure => BehaviorStatus::Success,
                    BehaviorStatus::Running => BehaviorStatus::Running,
                };
                (inverted, action)
            },
            BehaviorNode::Action(action) => (BehaviorStatus::Running, Some(action.clone())),
            BehaviorNode::AlwaysSucceed => (BehaviorStatus::Success, None),
            BehaviorNode::AlwaysFail => (BehaviorStatus::Failure, None),
        }
    }

    /// Evaluates a condition.
    fn evaluate_condition<W: NPCWorld>(ctx: &EvalContext<'_, W>, cond: &BehaviorCondition) -> bool {
        match cond {
            BehaviorCondition::TargetInRange(range) => {
                if ctx.npc.target.is_some() {
                    let dist = ctx.npc.distance_to(ctx.player_pos);
                    dist <= *range
                } else {
                    false
                }
            },
            BehaviorCondition::TargetVisible => {
                if ctx.npc.target.is_some() {
                    ctx.world
                        .has_line_of_sight(ctx.npc.position, ctx.player_pos)
                } else {
                    false
                }
            },
            BehaviorCondition::HasTarget => ctx.npc.target.is_some(),
            BehaviorCondition::IsAtHome => ctx.npc.is_at_home(),
            BehaviorCondition::TooFarFromHome => ctx.npc.is_too_far_from_home(),
            BehaviorCondition::HealthBelow(threshold) => {
                // Would need storage access for health
                // For now assume always false unless very low
                *threshold > 0.9 // Placeholder
            },
            BehaviorCondition::IsProvoked => ctx.npc.provoked,
            BehaviorCondition::InWanderRadius => {
                distance(ctx.npc.position, ctx.npc.home_position) <= ctx.npc.wander_radius
            },
            BehaviorCondition::CanAttack => ctx.npc.attack_cooldown <= 0.0,
        }
    }

    /// Executes an action and returns updated state.
    #[allow(clippy::too_many_arguments)]
    fn execute_action<W: NPCWorld>(
        &mut self,
        npc_id: EntityId,
        mut npc: NPCState,
        action: NPCAction,
        dt: f32,
        player_pos: (f32, f32),
        world: &W,
        combat: &mut CombatSystem,
    ) -> NPCState {
        // Update timers
        npc.attack_cooldown = (npc.attack_cooldown - dt).max(0.0);
        npc.behavior_time += dt;

        // Handle deaggro
        if npc.provoked {
            npc.deaggro_timer -= dt;
            if npc.deaggro_timer <= 0.0 {
                npc.provoked = false;
                npc.target = None;
            }
        }

        // Check for target acquisition
        if npc.target.is_none() && npc.npc_type == NPCType::Hostile {
            let dist = npc.distance_to(player_pos);
            if dist <= npc.aggro_range && world.has_line_of_sight(npc.position, player_pos) {
                npc.target = Some(EntityId::from_raw(0)); // Player ID placeholder
                npc.last_seen_player = Some(player_pos);
            }
        }

        // Execute action
        match action {
            NPCAction::Idle => {
                // Do nothing
            },
            NPCAction::Wander => {
                self.execute_wander(&mut npc, dt, world);
            },
            NPCAction::Patrol { waypoints } => {
                Self::execute_patrol(&mut npc, dt, &waypoints, world);
            },
            NPCAction::ChaseTarget => {
                npc.last_seen_player = Some(player_pos);
                Self::execute_chase(&mut npc, dt, player_pos, world);
            },
            NPCAction::AttackTarget => {
                if let Some(target) = npc.target {
                    Self::execute_attack(npc_id, &mut npc, target, combat);
                }
            },
            NPCAction::Flee => {
                Self::execute_flee(&mut npc, dt, player_pos, world);
            },
            NPCAction::ReturnHome => {
                let home = npc.home_position;
                Self::execute_move_to(&mut npc, dt, home, world);
            },
            NPCAction::Trade => {
                // Face the player
                npc.facing = direction_angle(npc.position, player_pos);
            },
            NPCAction::MoveTo { destination } => {
                Self::execute_move_to(&mut npc, dt, destination, world);
            },
        }

        npc
    }

    /// Executes wander behavior.
    fn execute_wander<W: NPCWorld>(&mut self, npc: &mut NPCState, dt: f32, world: &W) {
        // Pick a new wander target periodically
        if npc.behavior_time > 3.0 {
            npc.behavior_time = 0.0;
            let angle = self.next_random() * std::f32::consts::TAU;
            let dist = self.next_random() * npc.wander_radius;
            let target = (
                npc.home_position.0 + angle.cos() * dist,
                npc.home_position.1 + angle.sin() * dist,
            );

            if world.is_walkable(target) {
                npc.last_seen_player = Some(target); // Reuse for wander target
            }
        }

        // Move toward wander target
        if let Some(target) = npc.last_seen_player {
            let dist = distance(npc.position, target);
            if dist > 0.5 {
                Self::move_toward(npc, target, dt * 0.5, world); // Slower wander
            }
        }
    }

    /// Executes patrol behavior.
    fn execute_patrol<W: NPCWorld>(
        npc: &mut NPCState,
        dt: f32,
        waypoints: &[(f32, f32)],
        world: &W,
    ) {
        // Use NPC's stored waypoints if action waypoints are empty
        let points = if waypoints.is_empty() {
            &npc.patrol_waypoints
        } else {
            waypoints
        };

        if points.is_empty() {
            // No waypoints, idle
            return;
        }

        let target = points[npc.patrol_index % points.len()];
        let dist = distance(npc.position, target);

        if dist < 0.5 {
            // Reached waypoint, move to next
            npc.patrol_index = (npc.patrol_index + 1) % points.len();
        } else {
            Self::move_toward(npc, target, dt, world);
        }
    }

    /// Executes chase behavior.
    fn execute_chase<W: NPCWorld>(npc: &mut NPCState, dt: f32, target_pos: (f32, f32), world: &W) {
        Self::move_toward(npc, target_pos, dt, world);
    }

    /// Executes attack behavior.
    fn execute_attack(
        npc_id: EntityId,
        npc: &mut NPCState,
        target: EntityId,
        combat: &mut CombatSystem,
    ) {
        if npc.attack_cooldown > 0.0 {
            return;
        }

        // Queue attack
        let intent = AttackIntent::new(npc_id, AttackTarget::Entity(target))
            .with_attack_type(AttackType::Melee {
                range: 2.0,
                arc: 90.0_f32.to_radians(),
            })
            .with_damage(10.0);

        combat.queue_attack(intent);
        npc.attack_cooldown = 1.0; // 1 second cooldown
    }

    /// Executes flee behavior.
    fn execute_flee<W: NPCWorld>(npc: &mut NPCState, dt: f32, threat_pos: (f32, f32), world: &W) {
        // Move away from threat
        let dir = npc.direction_to(threat_pos);
        let flee_target = (
            npc.position.0 - dir.0 * npc.speed * 2.0,
            npc.position.1 - dir.1 * npc.speed * 2.0,
        );

        Self::move_toward(npc, flee_target, dt * 1.5, world); // Faster when fleeing
    }

    /// Executes move to behavior.
    fn execute_move_to<W: NPCWorld>(
        npc: &mut NPCState,
        dt: f32,
        destination: (f32, f32),
        world: &W,
    ) {
        Self::move_toward(npc, destination, dt, world);
    }

    /// Moves NPC toward a target position.
    fn move_toward<W: NPCWorld>(npc: &mut NPCState, target: (f32, f32), dt: f32, world: &W) {
        let dist = distance(npc.position, target);
        if dist < 0.1 {
            return;
        }

        // Get next waypoint from pathfinding
        let next = world
            .get_next_waypoint(npc.position, target)
            .unwrap_or(target);

        let dir = npc.direction_to(next);
        let move_dist = (npc.speed * dt).min(dist);

        let new_pos = (
            npc.position.0 + dir.0 * move_dist,
            npc.position.1 + dir.1 * move_dist,
        );

        if world.is_walkable(new_pos) {
            npc.position = new_pos;
            npc.facing = direction_angle(npc.position, next);
        }
    }

    /// Gets a random value 0..1.
    fn next_random(&mut self) -> f32 {
        // Simple LCG
        self.rng_state = self
            .rng_state
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12345);
        ((self.rng_state >> 16) & 0x7fff) as f32 / 32767.0
    }

    /// Returns iterator over all NPCs.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &NPCState)> {
        self.npcs.iter().map(|(&id, state)| (id, state))
    }

    /// Returns mutable iterator over all NPCs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut NPCState)> {
        self.npcs.iter_mut().map(|(&id, state)| (id, state))
    }

    /// Gets all NPCs of a specific type.
    pub fn get_by_type(&self, npc_type: NPCType) -> Vec<EntityId> {
        self.npcs
            .iter()
            .filter(|(_, state)| state.npc_type == npc_type)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Gets all NPCs within range of a position.
    pub fn get_in_range(&self, pos: (f32, f32), range: f32) -> Vec<EntityId> {
        self.npcs
            .iter()
            .filter(|(_, state)| distance(state.position, pos) <= range)
            .map(|(&id, _)| id)
            .collect()
    }
}

/// Evaluation context for behavior tree.
struct EvalContext<'a, W: NPCWorld> {
    npc: &'a NPCState,
    player_pos: (f32, f32),
    world: &'a W,
}

/// Calculates distance between two points.
fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    (dx * dx + dy * dy).sqrt()
}

/// Calculates angle from position a to b.
fn direction_angle(from: (f32, f32), to: (f32, f32)) -> f32 {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    dy.atan2(dx)
}

/// Mock world for testing.
#[derive(Debug, Default)]
pub struct MockNPCWorld {
    /// Blocked positions
    blocked: std::collections::HashSet<(i32, i32)>,
    /// Whether LOS is always true
    los_always_true: bool,
}

impl MockNPCWorld {
    /// Creates a new mock world.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether LOS always returns true.
    #[must_use]
    pub const fn with_los(mut self, los: bool) -> Self {
        self.los_always_true = los;
        self
    }

    /// Blocks a position.
    pub fn block(&mut self, x: i32, y: i32) {
        self.blocked.insert((x, y));
    }
}

impl NPCWorld for MockNPCWorld {
    fn has_line_of_sight(&self, _from: (f32, f32), _to: (f32, f32)) -> bool {
        self.los_always_true
    }

    fn get_next_waypoint(&self, _from: (f32, f32), to: (f32, f32)) -> Option<(f32, f32)> {
        // Simple: just return destination
        Some(to)
    }

    fn is_walkable(&self, pos: (f32, f32)) -> bool {
        !self.blocked.contains(&(pos.0 as i32, pos.1 as i32))
    }
}

/// Mock NPC storage for testing.
#[derive(Debug, Default)]
pub struct MockNPCStorage {
    positions: HashMap<EntityId, (f32, f32)>,
    facings: HashMap<EntityId, f32>,
    health: HashMap<EntityId, f32>,
}

impl MockNPCStorage {
    /// Creates a new mock storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an entity with position and health.
    pub fn add_entity(&mut self, id: EntityId, pos: (f32, f32), health_percent: f32) {
        self.positions.insert(id, pos);
        self.facings.insert(id, 0.0);
        self.health.insert(id, health_percent);
    }
}

impl NPCStorage for MockNPCStorage {
    fn get_health_percent(&self, entity: EntityId) -> Option<f32> {
        self.health.get(&entity).copied()
    }

    fn get_position(&self, entity: EntityId) -> Option<(f32, f32)> {
        self.positions.get(&entity).copied()
    }

    fn set_position(&mut self, entity: EntityId, pos: (f32, f32)) {
        self.positions.insert(entity, pos);
    }

    fn get_facing(&self, entity: EntityId) -> Option<f32> {
        self.facings.get(&entity).copied()
    }

    fn set_facing(&mut self, entity: EntityId, facing: f32) {
        self.facings.insert(entity, facing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_type_defaults() {
        assert_eq!(NPCType::Hostile.default_aggro_range(), 10.0);
        assert_eq!(NPCType::Passive.default_wander_radius(), 8.0);
        assert!(NPCType::Hostile.can_attack());
        assert!(!NPCType::Passive.can_attack());
        assert!(NPCType::Passive.flees());
    }

    #[test]
    fn test_npc_state_creation() {
        let state = NPCState::new(NPCType::Hostile, (10.0, 20.0));
        assert_eq!(state.npc_type, NPCType::Hostile);
        assert_eq!(state.position, (10.0, 20.0));
        assert_eq!(state.home_position, (10.0, 20.0));
        assert!(state.target.is_none());
    }

    #[test]
    fn test_npc_state_builders() {
        let state = NPCState::new(NPCType::Guard, (0.0, 0.0))
            .with_speed(5.0)
            .with_aggro_range(20.0)
            .with_patrol(vec![(0.0, 0.0), (10.0, 0.0)]);

        assert_eq!(state.speed, 5.0);
        assert_eq!(state.aggro_range, 20.0);
        assert_eq!(state.patrol_waypoints.len(), 2);
    }

    #[test]
    fn test_npc_state_distance() {
        let state = NPCState::new(NPCType::Hostile, (0.0, 0.0));
        assert!((state.distance_to((3.0, 4.0)) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_npc_state_direction() {
        let state = NPCState::new(NPCType::Hostile, (0.0, 0.0));
        let dir = state.direction_to((10.0, 0.0));
        assert!((dir.0 - 1.0).abs() < 0.001);
        assert!(dir.1.abs() < 0.001);
    }

    #[test]
    fn test_npc_state_is_at_home() {
        let mut state = NPCState::new(NPCType::Hostile, (0.0, 0.0));
        assert!(state.is_at_home());

        state.position = (10.0, 10.0);
        assert!(!state.is_at_home());
    }

    #[test]
    fn test_npc_manager_creation() {
        let manager = NPCManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_npc_spawn() {
        let mut manager = NPCManager::new();
        let id = manager.spawn_npc(NPCType::Hostile, (5.0, 5.0));

        assert_eq!(manager.len(), 1);
        assert!(manager.get(id).is_some());

        let npc = manager.get(id).expect("NPC should exist");
        assert_eq!(npc.npc_type, NPCType::Hostile);
        assert_eq!(npc.position, (5.0, 5.0));
    }

    #[test]
    fn test_npc_despawn() {
        let mut manager = NPCManager::new();
        let id = manager.spawn_npc(NPCType::Passive, (0.0, 0.0));

        assert_eq!(manager.len(), 1);
        let state = manager.despawn_npc(id).expect("Despawn should succeed");
        assert_eq!(state.npc_type, NPCType::Passive);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_npc_despawn_not_found() {
        let mut manager = NPCManager::new();
        let fake_id = EntityId::new();

        let result = manager.despawn_npc(fake_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_npc_register() {
        let mut manager = NPCManager::new();
        let id = EntityId::new();

        manager
            .register_npc(id, NPCType::Merchant, (1.0, 2.0))
            .expect("Register should succeed");

        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_npc_register_duplicate() {
        let mut manager = NPCManager::new();
        let id = EntityId::new();

        manager
            .register_npc(id, NPCType::Merchant, (1.0, 2.0))
            .expect("First register should succeed");

        let result = manager.register_npc(id, NPCType::Hostile, (3.0, 4.0));
        assert!(matches!(result, Err(NPCError::AlreadyRegistered(_))));
    }

    #[test]
    fn test_npc_provoke() {
        let mut manager = NPCManager::new();
        let npc_id = manager.spawn_npc(NPCType::Neutral, (0.0, 0.0));
        let attacker_id = EntityId::new();

        manager.provoke(npc_id, attacker_id);

        let npc = manager.get(npc_id).expect("NPC should exist");
        assert!(npc.provoked);
        assert_eq!(npc.target, Some(attacker_id));
    }

    #[test]
    fn test_npc_provoke_non_neutral() {
        let mut manager = NPCManager::new();
        let npc_id = manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));
        let attacker_id = EntityId::new();

        manager.provoke(npc_id, attacker_id);

        let npc = manager.get(npc_id).expect("NPC should exist");
        assert!(!npc.provoked); // Hostile doesn't use provoke
    }

    #[test]
    fn test_behavior_node_sequence() {
        let node = BehaviorNode::sequence(vec![
            BehaviorNode::condition(BehaviorCondition::HasTarget),
            BehaviorNode::action(NPCAction::ChaseTarget),
        ]);

        match node {
            BehaviorNode::Sequence(children) => assert_eq!(children.len(), 2),
            _ => panic!("Expected Sequence"),
        }
    }

    #[test]
    fn test_behavior_node_selector() {
        let node = BehaviorNode::selector(vec![
            BehaviorNode::action(NPCAction::AttackTarget),
            BehaviorNode::action(NPCAction::ChaseTarget),
        ]);

        match node {
            BehaviorNode::Selector(children) => assert_eq!(children.len(), 2),
            _ => panic!("Expected Selector"),
        }
    }

    #[test]
    fn test_behavior_tree_hostile() {
        let tree = BehaviorTree::hostile();
        match tree.root() {
            BehaviorNode::Selector(_) => {}, // Expected
            _ => panic!("Hostile tree should have Selector root"),
        }
    }

    #[test]
    fn test_behavior_tree_passive() {
        let tree = BehaviorTree::passive();
        match tree.root() {
            BehaviorNode::Selector(_) => {},
            _ => panic!("Passive tree should have Selector root"),
        }
    }

    #[test]
    fn test_behavior_tree_neutral() {
        let tree = BehaviorTree::neutral();
        match tree.root() {
            BehaviorNode::Selector(_) => {},
            _ => panic!("Neutral tree should have Selector root"),
        }
    }

    #[test]
    fn test_behavior_tree_guard() {
        let tree = BehaviorTree::guard();
        match tree.root() {
            BehaviorNode::Selector(_) => {},
            _ => panic!("Guard tree should have Selector root"),
        }
    }

    #[test]
    fn test_behavior_tree_merchant() {
        let tree = BehaviorTree::merchant();
        match tree.root() {
            BehaviorNode::Selector(_) => {},
            _ => panic!("Merchant tree should have Selector root"),
        }
    }

    #[test]
    fn test_mock_npc_world() {
        let world = MockNPCWorld::new().with_los(true);
        assert!(world.has_line_of_sight((0.0, 0.0), (10.0, 10.0)));
        assert!(world.is_walkable((5.0, 5.0)));
    }

    #[test]
    fn test_mock_npc_world_blocked() {
        let mut world = MockNPCWorld::new();
        world.block(5, 5);

        assert!(!world.is_walkable((5.0, 5.0)));
        assert!(world.is_walkable((6.0, 6.0)));
    }

    #[test]
    fn test_mock_npc_storage() {
        let mut storage = MockNPCStorage::new();
        let id = EntityId::new();

        storage.add_entity(id, (1.0, 2.0), 0.75);

        assert_eq!(storage.get_position(id), Some((1.0, 2.0)));
        assert_eq!(storage.get_health_percent(id), Some(0.75));
        assert_eq!(storage.get_facing(id), Some(0.0));
    }

    #[test]
    fn test_npc_update_basic() {
        let mut manager = NPCManager::new();
        let _npc_id = manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));

        let player_id = EntityId::new();
        let player_pos = (50.0, 50.0); // Far away

        let world = MockNPCWorld::new().with_los(true);
        let mut storage = MockNPCStorage::new();
        let mut combat = CombatSystem::new();

        // Should not crash
        manager.update(
            0.016,
            player_id,
            player_pos,
            &world,
            &mut storage,
            &mut combat,
        );
    }

    #[test]
    fn test_npc_get_by_type() {
        let mut manager = NPCManager::new();
        manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));
        manager.spawn_npc(NPCType::Hostile, (10.0, 0.0));
        manager.spawn_npc(NPCType::Passive, (20.0, 0.0));

        let hostiles = manager.get_by_type(NPCType::Hostile);
        assert_eq!(hostiles.len(), 2);

        let passives = manager.get_by_type(NPCType::Passive);
        assert_eq!(passives.len(), 1);
    }

    #[test]
    fn test_npc_get_in_range() {
        let mut manager = NPCManager::new();
        manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));
        manager.spawn_npc(NPCType::Hostile, (5.0, 0.0));
        manager.spawn_npc(NPCType::Hostile, (20.0, 0.0));

        let nearby = manager.get_in_range((0.0, 0.0), 10.0);
        assert_eq!(nearby.len(), 2);
    }

    #[test]
    fn test_npc_iter() {
        let mut manager = NPCManager::new();
        let id1 = manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));
        let id2 = manager.spawn_npc(NPCType::Passive, (10.0, 0.0));

        let ids: Vec<EntityId> = manager.iter().map(|(id, _)| id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_npc_iter_mut() {
        let mut manager = NPCManager::new();
        let id = manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));

        for (npc_id, state) in manager.iter_mut() {
            if npc_id == id {
                state.speed = 10.0;
            }
        }

        let npc = manager.get(id).expect("NPC should exist");
        assert_eq!(npc.speed, 10.0);
    }

    #[test]
    fn test_behavior_id_constants() {
        assert_eq!(BehaviorId::IDLE.0, 0);
        assert_eq!(BehaviorId::WANDER.0, 1);
        assert_eq!(BehaviorId::CHASE.0, 2);
        assert_eq!(BehaviorId::ATTACK.0, 3);
        assert_eq!(BehaviorId::FLEE.0, 4);
        assert_eq!(BehaviorId::RETURN_HOME.0, 5);
        assert_eq!(BehaviorId::PATROL.0, 6);
        assert_eq!(BehaviorId::TRADE.0, 7);
    }

    #[test]
    fn test_npc_action_variants() {
        let _idle = NPCAction::Idle;
        let _wander = NPCAction::Wander;
        let _patrol = NPCAction::Patrol {
            waypoints: vec![(0.0, 0.0), (10.0, 0.0)],
        };
        let _chase = NPCAction::ChaseTarget;
        let _attack = NPCAction::AttackTarget;
        let _flee = NPCAction::Flee;
        let _return_home = NPCAction::ReturnHome;
        let _trade = NPCAction::Trade;
        let _move = NPCAction::MoveTo {
            destination: (5.0, 5.0),
        };
    }

    #[test]
    fn test_behavior_condition_variants() {
        let _target_range = BehaviorCondition::TargetInRange(5.0);
        let _target_visible = BehaviorCondition::TargetVisible;
        let _has_target = BehaviorCondition::HasTarget;
        let _at_home = BehaviorCondition::IsAtHome;
        let _too_far = BehaviorCondition::TooFarFromHome;
        let _health = BehaviorCondition::HealthBelow(0.5);
        let _provoked = BehaviorCondition::IsProvoked;
        let _in_radius = BehaviorCondition::InWanderRadius;
        let _can_attack = BehaviorCondition::CanAttack;
    }

    #[test]
    fn test_distance_function() {
        assert!((distance((0.0, 0.0), (3.0, 4.0)) - 5.0).abs() < 0.001);
        assert!((distance((0.0, 0.0), (0.0, 0.0))).abs() < 0.001);
    }

    #[test]
    fn test_direction_angle_function() {
        // Angle to right is 0
        let angle = direction_angle((0.0, 0.0), (1.0, 0.0));
        assert!(angle.abs() < 0.001);

        // Angle up is PI/2
        let angle = direction_angle((0.0, 0.0), (0.0, 1.0));
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn test_register_custom_behavior_tree() {
        let mut manager = NPCManager::new();

        let custom_tree = BehaviorTree::new(BehaviorNode::action(NPCAction::Idle));
        manager.register_behavior_tree(NPCType::Hostile, custom_tree);

        // Spawned hostile should now use custom tree
        let _id = manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));
    }

    #[test]
    fn test_npc_deaggro() {
        let mut manager = NPCManager::new();
        let npc_id = manager.spawn_npc(NPCType::Neutral, (0.0, 0.0));
        let attacker_id = EntityId::new();

        manager.provoke(npc_id, attacker_id);

        // Update with enough time to deaggro (>10 seconds)
        let world = MockNPCWorld::new();
        let mut storage = MockNPCStorage::new();
        let mut combat = CombatSystem::new();

        for _ in 0..700 {
            // 700 * 0.016 = ~11 seconds
            manager.update(
                0.016,
                attacker_id,
                (100.0, 100.0),
                &world,
                &mut storage,
                &mut combat,
            );
        }

        let npc = manager.get(npc_id).expect("NPC should exist");
        assert!(!npc.provoked); // Should have de-aggro'd
    }

    #[test]
    fn test_behavior_node_inverter() {
        let node = BehaviorNode::inverter(BehaviorNode::condition(BehaviorCondition::HasTarget));

        match node {
            BehaviorNode::Inverter(child) => {
                match *child {
                    BehaviorNode::Condition(BehaviorCondition::HasTarget) => {}, // Expected
                    _ => panic!("Expected HasTarget condition"),
                }
            },
            _ => panic!("Expected Inverter"),
        }
    }

    #[test]
    fn test_npc_state_too_far_from_home() {
        let mut state = NPCState::new(NPCType::Hostile, (0.0, 0.0));
        assert!(!state.is_too_far_from_home());

        // Move very far
        state.position = (100.0, 100.0);
        assert!(state.is_too_far_from_home());
    }

    #[test]
    fn test_npc_get_mut() {
        let mut manager = NPCManager::new();
        let id = manager.spawn_npc(NPCType::Hostile, (0.0, 0.0));

        if let Some(npc) = manager.get_mut(id) {
            npc.attack_cooldown = 5.0;
        }

        let npc = manager.get(id).expect("NPC should exist");
        assert_eq!(npc.attack_cooldown, 5.0);
    }
}
