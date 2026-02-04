# PROMPT — Gameplay Agent — Iteration 4

> **Branch**: `gameplay-agent`
> **Focus**: Combat system, NPC AI, vehicle system, quest framework

## Your Mission

Complete the following tasks. Work through them sequentially. After each task, run the validation loop. Commit after each task passes.

---

## Tasks

### G-17: Combat System (P0)
**File**: `crates/genesis-gameplay/src/combat.rs`

Implement melee and ranged combat:

```rust
pub struct CombatSystem {
    attack_queue: VecDeque<AttackIntent>,
    damage_events: Vec<DamageEvent>,
}

pub struct AttackIntent {
    pub attacker: EntityId,
    pub target: AttackTarget,
    pub weapon: Option<ItemId>,
    pub attack_type: AttackType,
}

pub enum AttackTarget {
    Entity(EntityId),
    Position(f32, f32),
    Direction(f32, f32),
}

pub enum AttackType {
    Melee { range: f32, arc: f32 },
    Ranged { projectile: ProjectileType, speed: f32 },
    Area { radius: f32, falloff: bool },
}

pub struct DamageEvent {
    pub source: Option<EntityId>,
    pub target: EntityId,
    pub damage: f32,
    pub damage_type: DamageType,
    pub position: (f32, f32),
    pub knockback: Option<(f32, f32)>,
}

pub enum DamageType {
    Physical,
    Fire,
    Ice,
    Electric,
    Poison,
}

pub struct CombatStats {
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub resistances: HashMap<DamageType, f32>,
    pub attack_speed: f32,
    pub damage_multiplier: f32,
}

impl CombatSystem {
    pub fn new() -> Self;

    pub fn queue_attack(&mut self, intent: AttackIntent);
    pub fn process_attacks(
        &mut self,
        entities: &mut EntityStorage,
        collision: &CollisionQuery,
    ) -> Vec<DamageEvent>;

    pub fn apply_damage(
        &mut self,
        target: EntityId,
        damage: &DamageEvent,
        stats: &mut CombatStats,
    ) -> bool; // returns true if killed
}
```

Requirements:
- Weapon damage calculation with stats
- Armor and resistance reduction
- Hit detection via collision queries
- Knockback physics
- Death handling

### G-18: NPC AI System (P0)
**File**: `crates/genesis-gameplay/src/npc.rs`

Implement NPC behavior:

```rust
pub struct NPCManager {
    npcs: HashMap<EntityId, NPCState>,
    behavior_trees: HashMap<NPCType, BehaviorTree>,
}

pub struct NPCState {
    pub npc_type: NPCType,
    pub current_behavior: BehaviorId,
    pub target: Option<EntityId>,
    pub home_position: (f32, f32),
    pub aggro_range: f32,
    pub wander_radius: f32,
    pub last_seen_player: Option<(f32, f32)>,
}

pub enum NPCType {
    Passive,      // Flees from player
    Neutral,      // Attacks if provoked
    Hostile,      // Attacks on sight
    Merchant,     // Trading AI
    Guard,        // Defends area
}

pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Condition(Box<dyn Fn(&NPCState, &World) -> bool>),
    Action(NPCAction),
}

pub enum NPCAction {
    Idle,
    Wander,
    Patrol { waypoints: Vec<(f32, f32)> },
    ChaseTarget,
    AttackTarget,
    Flee,
    ReturnHome,
    Trade,
}

impl NPCManager {
    pub fn new() -> Self;

    pub fn spawn_npc(&mut self, npc_type: NPCType, position: (f32, f32)) -> EntityId;
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: (f32, f32),
        entities: &mut EntityStorage,
        combat: &mut CombatSystem,
    );

    fn evaluate_behavior(&self, npc: &NPCState, world: &World) -> NPCAction;
    fn execute_action(&mut self, npc_id: EntityId, action: NPCAction);
}
```

Requirements:
- Behavior tree execution
- Line-of-sight checks
- Aggro/de-aggro logic
- Pathfinding (A* or simple steering)
- NPC spawning/despawning

### G-19: Vehicle System (P0)
**File**: `crates/genesis-gameplay/src/vehicle.rs`

Implement rideable vehicles:

```rust
pub struct VehicleSystem {
    vehicles: HashMap<EntityId, VehicleState>,
}

pub struct VehicleState {
    pub vehicle_type: VehicleType,
    pub driver: Option<EntityId>,
    pub passengers: Vec<EntityId>,
    pub fuel: f32,
    pub max_fuel: f32,
    pub health: f32,
    pub velocity: (f32, f32),
}

pub enum VehicleType {
    Cart { max_speed: f32, capacity: u32 },
    Boat { max_speed: f32, can_dive: bool },
    Minecart { rail_only: bool },
    Mount { stamina: f32, jump_power: f32 },
}

pub struct VehicleStats {
    pub max_speed: f32,
    pub acceleration: f32,
    pub turn_rate: f32,
    pub passenger_slots: u32,
    pub cargo_slots: u32,
}

impl VehicleSystem {
    pub fn new() -> Self;

    pub fn spawn_vehicle(&mut self, vehicle_type: VehicleType, pos: (f32, f32)) -> EntityId;

    pub fn enter_vehicle(&mut self, entity: EntityId, vehicle: EntityId) -> Result<(), VehicleError>;
    pub fn exit_vehicle(&mut self, entity: EntityId) -> Result<(f32, f32), VehicleError>;

    pub fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        collision: &CollisionQuery,
    );

    pub fn get_driver_position(&self, vehicle: EntityId) -> Option<(f32, f32)>;
}

pub enum VehicleError {
    VehicleFull,
    NotInVehicle,
    TooFarAway,
    VehicleDestroyed,
}
```

Requirements:
- Entry/exit with position validation
- Vehicle physics (different from player)
- Fuel consumption
- Passenger positions
- Collision as larger hitbox

### G-20: Quest System (P1)
**File**: `crates/genesis-gameplay/src/quest.rs`

Implement quest tracking:

```rust
pub struct QuestManager {
    available_quests: HashMap<QuestId, QuestTemplate>,
    active_quests: HashMap<QuestId, QuestProgress>,
    completed_quests: HashSet<QuestId>,
}

pub struct QuestTemplate {
    pub id: QuestId,
    pub name: String,
    pub description: String,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<QuestReward>,
    pub prerequisites: Vec<QuestId>,
    pub repeatable: bool,
}

pub enum QuestObjective {
    Kill { target: NPCType, count: u32 },
    Collect { item: ItemId, count: u32 },
    Reach { position: (f32, f32), radius: f32 },
    Talk { npc_id: EntityId },
    Craft { recipe: RecipeId, count: u32 },
    Custom { id: String, description: String },
}

pub struct QuestProgress {
    pub quest_id: QuestId,
    pub started_at: f64,
    pub objective_progress: Vec<u32>,
    pub stage: u32,
}

pub enum QuestReward {
    Experience(u32),
    Item { id: ItemId, count: u32 },
    Currency(u32),
    Reputation { faction: FactionId, amount: i32 },
    Unlock(UnlockId),
}

impl QuestManager {
    pub fn new() -> Self;

    pub fn start_quest(&mut self, quest_id: QuestId) -> Result<(), QuestError>;
    pub fn abandon_quest(&mut self, quest_id: QuestId);
    pub fn complete_quest(&mut self, quest_id: QuestId) -> Vec<QuestReward>;

    pub fn on_enemy_killed(&mut self, enemy_type: NPCType);
    pub fn on_item_collected(&mut self, item: ItemId, count: u32);
    pub fn on_position_reached(&mut self, position: (f32, f32));

    pub fn get_active_quests(&self) -> Vec<&QuestProgress>;
    pub fn is_objective_complete(&self, quest: QuestId, objective: usize) -> bool;
}
```

Requirements:
- Multi-objective quests
- Progress tracking
- Prerequisite checking
- Reward distribution
- Quest log data for UI

---

## Validation Loop

After each task:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --workspace
```

If ANY step fails, FIX IT before committing.

---

## Commit Convention

```
[gameplay] feat: G-17 combat system
[gameplay] feat: G-18 NPC AI system
[gameplay] feat: G-19 vehicle system
[gameplay] feat: G-20 quest system
```

---

## Integration Notes

- G-17 combat uses CollisionQuery from genesis-kernel
- G-18 NPCs use combat system for attacks
- G-19 vehicles override player physics when mounted
- G-20 quest hooks into combat, inventory, position
- Export new modules in lib.rs
