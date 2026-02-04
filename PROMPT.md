# PROMPT — Tools Agent — Iteration 4

> **Branch**: `tools-agent`
> **Focus**: Audio playback, quest UI, NPC dialogue, combat HUD

## Your Mission

Complete the following tasks. Work through them sequentially. After each task, run the validation loop. Commit after each task passes.

---

## Tasks

### T-16: Audio Engine Integration (P0)
**File**: `crates/genesis-tools/src/audio.rs`

Implement audio playback with rodio:

```rust
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use genesis_kernel::audio::{SpatialAudioManager, AudioSourceId};

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<AudioSourceId, Sink>,
    music_sink: Option<Sink>,
    master_volume: f32,
    sfx_volume: f32,
    music_volume: f32,
}

pub struct SoundEffect {
    pub id: SoundId,
    pub data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioEngine {
    pub fn new() -> Result<Self, AudioError>;

    pub fn play_sound(&mut self, sound: &SoundEffect, position: Option<(f32, f32)>) -> AudioSourceId;
    pub fn play_music(&mut self, music: &SoundEffect, fade_in: f32);
    pub fn stop_music(&mut self, fade_out: f32);

    pub fn update_spatial(&mut self, spatial: &SpatialAudioManager);

    pub fn set_master_volume(&mut self, volume: f32);
    pub fn set_sfx_volume(&mut self, volume: f32);
    pub fn set_music_volume(&mut self, volume: f32);

    pub fn stop_sound(&mut self, id: AudioSourceId);
    pub fn stop_all(&mut self);
}

pub enum AudioError {
    NoOutputDevice,
    DecodingError(String),
    PlaybackError(String),
}
```

Requirements:
- rodio for cross-platform audio
- Spatial positioning from kernel data
- Separate volume controls
- Music crossfading
- Sound pooling for frequent effects

### T-17: Quest UI (P0)
**File**: `crates/genesis-tools/src/quest_ui.rs`

Render quest tracker and log:

```rust
use egui::{Context, Window};
use genesis_gameplay::quest::{QuestManager, QuestProgress, QuestObjective};

pub struct QuestUI {
    pub tracker_visible: bool,
    pub log_open: bool,
    selected_quest: Option<QuestId>,
}

pub struct QuestUIData {
    pub active_quests: Vec<QuestDisplayData>,
    pub available_quests: Vec<QuestDisplayData>,
    pub completed_count: u32,
}

pub struct QuestDisplayData {
    pub id: QuestId,
    pub name: String,
    pub description: String,
    pub objectives: Vec<ObjectiveDisplayData>,
    pub rewards: Vec<String>,
    pub tracked: bool,
}

pub struct ObjectiveDisplayData {
    pub description: String,
    pub progress: u32,
    pub required: u32,
    pub complete: bool,
}

impl QuestUI {
    pub fn new() -> Self;

    // Compact tracker (corner of screen)
    pub fn show_tracker(&mut self, ctx: &Context, data: &QuestUIData);

    // Full quest log window
    pub fn show_log(&mut self, ctx: &Context, data: &QuestUIData) -> Option<QuestAction>;

    fn render_objective(&self, ui: &mut egui::Ui, obj: &ObjectiveDisplayData);
}

pub enum QuestAction {
    Track(QuestId),
    Untrack(QuestId),
    Abandon(QuestId),
    SelectQuest(QuestId),
}
```

Requirements:
- Compact tracker (tracked quests only)
- Full quest log with categories
- Objective progress bars
- Track/untrack toggle
- Quest details panel

### T-18: Dialogue System UI (P0)
**File**: `crates/genesis-tools/src/dialogue_ui.rs`

Implement NPC dialogue interface:

```rust
use egui::{Context, Window, RichText};

pub struct DialogueUI {
    pub is_active: bool,
    current_node: Option<DialogueNodeId>,
    history: Vec<DialogueLine>,
    typewriter_progress: f32,
}

pub struct DialogueTree {
    pub nodes: HashMap<DialogueNodeId, DialogueNode>,
    pub start_node: DialogueNodeId,
}

pub struct DialogueNode {
    pub speaker: String,
    pub portrait: Option<String>,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
    pub on_enter: Vec<DialogueEffect>,
}

pub struct DialogueChoice {
    pub text: String,
    pub next_node: Option<DialogueNodeId>,
    pub condition: Option<DialogueCondition>,
    pub effects: Vec<DialogueEffect>,
}

pub enum DialogueCondition {
    HasItem(ItemId, u32),
    QuestComplete(QuestId),
    QuestActive(QuestId),
    ReputationAbove(FactionId, i32),
    Custom(String),
}

pub enum DialogueEffect {
    GiveItem(ItemId, u32),
    TakeItem(ItemId, u32),
    StartQuest(QuestId),
    CompleteQuest(QuestId),
    AddReputation(FactionId, i32),
    OpenShop(ShopId),
}

impl DialogueUI {
    pub fn new() -> Self;

    pub fn start_dialogue(&mut self, tree: &DialogueTree, npc_name: &str);
    pub fn show(&mut self, ctx: &Context, tree: &DialogueTree) -> Vec<DialogueEffect>;
    pub fn end_dialogue(&mut self);

    fn render_speaker(&self, ui: &mut egui::Ui, node: &DialogueNode);
    fn render_choices(&self, ui: &mut egui::Ui, choices: &[DialogueChoice]) -> Option<usize>;
}
```

Requirements:
- Typewriter text effect
- Speaker portrait display
- Choice buttons with conditions
- Dialogue history scroll
- Effects returned for gameplay

### T-19: Combat HUD (P1)
**File**: `crates/genesis-tools/src/combat_hud.rs`

Render combat-related UI:

```rust
use egui::{Context, Painter, Pos2, Color32};
use genesis_gameplay::combat::{CombatStats, DamageEvent};

pub struct CombatHUD {
    damage_numbers: Vec<DamageNumber>,
    health_bars: HashMap<EntityId, HealthBarState>,
    crosshair_style: CrosshairStyle,
}

pub struct DamageNumber {
    pub value: f32,
    pub position: (f32, f32),
    pub color: Color32,
    pub lifetime: f32,
    pub velocity: (f32, f32),
}

pub struct HealthBarState {
    pub current: f32,
    pub max: f32,
    pub damage_preview: f32,
    pub heal_preview: f32,
}

pub enum CrosshairStyle {
    None,
    Dot,
    Cross,
    Circle,
    Custom(String),
}

impl CombatHUD {
    pub fn new() -> Self;

    // Player health/mana bars
    pub fn show_player_vitals(&self, ctx: &Context, stats: &CombatStats);

    // Enemy health bars (world-space)
    pub fn show_enemy_health_bars(
        &self,
        painter: &Painter,
        enemies: &[(EntityId, (f32, f32), &CombatStats)],
        camera: &Camera,
    );

    // Floating damage numbers
    pub fn spawn_damage_number(&mut self, event: &DamageEvent);
    pub fn update_damage_numbers(&mut self, dt: f32);
    pub fn render_damage_numbers(&self, painter: &Painter, camera: &Camera);

    // Crosshair
    pub fn show_crosshair(&self, ctx: &Context);

    // Cooldown indicators
    pub fn show_ability_cooldowns(&self, ctx: &Context, cooldowns: &[f32]);
}
```

Requirements:
- Animated health bars
- Floating damage numbers
- Critical hit emphasis
- Low health warning effect
- Target lock indicator

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
[tools] feat: T-16 audio engine integration
[tools] feat: T-17 quest UI
[tools] feat: T-18 dialogue system UI
[tools] feat: T-19 combat HUD
```

---

## Dependencies

Add to `crates/genesis-tools/Cargo.toml`:
```toml
rodio = "0.19"
```

---

## Integration Notes

- T-16 uses SpatialAudioManager data from genesis-kernel
- T-17 displays QuestManager data from genesis-gameplay
- T-18 controls NPC dialogue flow
- T-19 visualizes combat events
- Export new modules in lib.rs
