# Gameplay Agent — Iteration 9 Prompt

## Context

You are the **Gameplay Agent** for Project Genesis, a 2D top-down game engine built with Rust.

**Current State:**
- Biome terrain generation complete (G-33 to G-36)
- Player movement and interaction systems
- Weather and time systems
- Plant growth system

**Iteration 9 Focus:** NPC entity system, AI behaviors, spawning, and dialogue.

---

## Assigned Tasks

### G-37: NPC entity system (P0)

**Goal:** Define NPC data structures and management.

**Implementation:**
1. Create `crates/genesis-gameplay/src/npc.rs`
2. Define NPC types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcType {
    Villager,
    Merchant,
    Guard,
    Animal(AnimalType),
    Monster(MonsterType),
}

#[derive(Debug, Clone, Copy)]
pub enum AnimalType {
    Chicken, Cow, Pig, Sheep, Wolf, Bear,
}

#[derive(Debug, Clone, Copy)]
pub enum MonsterType {
    Slime, Skeleton, Goblin, Orc,
}

pub struct Npc {
    pub id: u32,
    pub npc_type: NpcType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub facing: Direction,
    pub state: NpcState,
    pub health: f32,
    pub max_health: f32,
    pub name: Option<String>,
    pub dialogue_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcState {
    Idle,
    Walking,
    Running,
    Attacking,
    Fleeing,
    Talking,
    Dead,
}
```

3. NpcManager to track all NPCs:

```rust
pub struct NpcManager {
    npcs: HashMap<u32, Npc>,
    next_id: u32,
    spatial_index: QuadTree<u32>, // For efficient lookup
}
```

---

### G-38: NPC AI behavior trees (P0)

**Goal:** Implement simple behavior trees for NPC AI.

**Implementation:**
1. Create `crates/genesis-gameplay/src/ai.rs`
2. Behavior types:

```rust
pub enum Behavior {
    Idle { duration: f32 },
    Patrol { waypoints: Vec<Vec2>, current: usize },
    Wander { radius: f32, center: Vec2 },
    Follow { target_id: u32, distance: f32 },
    Flee { from: Vec2, speed: f32 },
    Attack { target_id: u32 },
}

pub struct BehaviorTree {
    root: BehaviorNode,
}

pub enum BehaviorNode {
    Selector(Vec<BehaviorNode>),  // Try children until one succeeds
    Sequence(Vec<BehaviorNode>), // Run children in order
    Action(Behavior),
    Condition(Box<dyn Fn(&Npc, &World) -> bool>),
}
```

3. Default behaviors per NPC type:
   - Villager: Wander during day, go home at night
   - Merchant: Stay at shop location
   - Guard: Patrol, chase hostiles
   - Animal: Wander, flee from player if wild
   - Monster: Wander, chase player if in range

---

### G-39: NPC spawning system (P0)

**Goal:** Spawn NPCs based on biome and rules.

**Implementation:**
1. Spawn rules per biome:

```rust
pub struct SpawnRule {
    pub npc_type: NpcType,
    pub biomes: Vec<BiomeType>,
    pub min_density: f32,  // Per chunk
    pub max_density: f32,
    pub group_size: (u32, u32), // Min, max
    pub time_of_day: Option<(f32, f32)>, // Active hours
}
```

2. Default spawn rules:
   - Forest: Deer, Wolf, Villager
   - Desert: Scorpion, Merchant (rare)
   - Plains: Cow, Sheep, Chicken, Villager
   - Mountain: Goat, Bear
   - Swamp: Slime, Frog

3. Spawn on chunk load, despawn on chunk unload
4. Respect max NPC count per chunk (e.g., 20)

---

### G-40: Dialogue system (P1)

**Goal:** Support NPC dialogue with branching conversations.

**Implementation:**
1. Create `crates/genesis-gameplay/src/dialogue.rs`

```rust
pub struct DialogueNode {
    pub id: u32,
    pub speaker: String,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
}

pub struct DialogueChoice {
    pub text: String,
    pub next_node: Option<u32>,
    pub condition: Option<DialogueCondition>,
    pub effect: Option<DialogueEffect>,
}

pub enum DialogueCondition {
    HasItem(ItemId, u32),
    QuestComplete(QuestId),
    ReputationAbove(i32),
}

pub enum DialogueEffect {
    GiveItem(ItemId, u32),
    TakeItem(ItemId, u32),
    StartQuest(QuestId),
    AddReputation(i32),
}

pub struct DialogueManager {
    dialogues: HashMap<u32, DialogueNode>,
    active_dialogue: Option<ActiveDialogue>,
}
```

2. Load dialogues from data files (JSON or RON)
3. Support variables in text: "Hello, {player_name}!"

---

## Constraints

1. **No rendering:** Only game logic, data structures
2. **Deterministic:** Same seed = same NPC spawns
3. **Performance:** AI updates < 1ms for 100 NPCs
4. **Modular:** Easy to add new NPC types

---

## Commit Format

```
[gameplay] feat: G-37..G-40 NPC entity system, AI behaviors, spawning, dialogue
```
