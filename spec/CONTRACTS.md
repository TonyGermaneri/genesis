# Project Genesis - Contracts & Interfaces

> Version: 1.0.0  
> Last Updated: 2026-02-03

## Overview

This document defines the contracts between all subsystems in Project Genesis.
All agents MUST adhere to these contracts. Breaking changes require orchestrator approval
and version bumps.

---

## 1. GPU ↔ CPU Contract

### 1.1 Cell Buffer Layout

The GPU kernel operates on cell buffers. The CPU entity layer submits intents via
a command buffer, and the GPU executes them during simulation.

```
Cell (8 bytes):
┌─────────────────┬──────────┬─────────────┬────────────┬────────────┬──────────┐
│ material (u16)  │ flags(u8)│ temp (u8)   │ vel_x (i8) │ vel_y (i8) │ data(u16)│
└─────────────────┴──────────┴─────────────┴────────────┴────────────┴──────────┘
```

### 1.2 Intent Buffer (CPU → GPU)

```rust
struct CellIntent {
    coord: WorldCoord,      // Target cell
    intent_type: u8,        // 0=set_material, 1=apply_force, 2=ignite, etc.
    payload: [u8; 7],       // Intent-specific data
}
```

### 1.3 Event Buffer (GPU → CPU)

```rust
struct CellEvent {
    coord: WorldCoord,      // Source cell
    event_type: u8,         // 0=destroyed, 1=state_change, etc.
    payload: [u8; 7],
}
```

---

## 2. Chunk Contract

### 2.1 Chunk Ownership

- **GPU Kernel**: Physical authority (cell simulation)
- **CPU World**: Persistence authority (save/load)
- **CPU Gameplay**: Entity authority (NPCs, vehicles)

### 2.2 Chunk Lifecycle

```
[Unloaded] → generate/load → [Loaded] → simulate → [Dirty] → save → [Clean] → unload → [Unloaded]
```

### 2.3 Chunk Handoff Protocol

1. CPU requests chunk load
2. World crate loads/generates chunk data
3. Chunk data uploaded to GPU buffer
4. GPU simulates chunk
5. On save: GPU readback → CPU compression → disk

---

## 3. Event Bus Contract

### 3.1 Event Categories

| Category | Publisher | Subscribers |
|----------|-----------|-------------|
| Entity   | Gameplay  | UI, Audio, World |
| Economy  | Gameplay  | UI, NPCs |
| World    | World/Kernel | Gameplay, UI |
| Input    | Engine    | Gameplay, UI |

### 3.2 Event Ordering

- Events within a frame are unordered
- Cross-frame ordering is guaranteed (frame N before frame N+1)
- Subscribers MUST NOT rely on intra-frame order

### 3.3 Event Backpressure

- Event bus has bounded capacity (1024 by default)
- On overflow: oldest events dropped, warning logged
- Critical events use separate high-priority channel

---

## 4. Crafting Contract

### 4.1 Recipe Format

See `spec/schemas/crafting_recipe.ron`

### 4.2 Crafting Flow

1. Player initiates craft (via UI intent)
2. Gameplay validates: ingredients, tools, skill
3. If valid: consume ingredients, queue crafting
4. On completion: add output to inventory, emit event

### 4.3 Building Placement

1. Player selects building + location
2. Gameplay validates: space, components, terrain
3. If valid: consume components, submit world intent
4. GPU/World updates cell data
5. Building entity created

---

## 5. Module/Mod Contract

### 5.1 Mod Loading Order

1. Core modules (engine-provided)
2. Official modules (bundled)
3. User modules (mod folder, alphabetical)

### 5.2 Mod Capabilities

| Capability | Description |
|------------|-------------|
| `recipes`  | Add/modify crafting recipes |
| `items`    | Add item definitions |
| `entities` | Add entity templates |
| `materials`| Add cell materials |
| `events`   | Subscribe to events |

### 5.3 Mod Isolation

- Mods cannot access filesystem directly
- Mods cannot access network
- Mods communicate via event bus only

---

## 6. Agent Integration Contract

### 6.1 Branch Naming

```
<agent>/<feature>
kernel/cell-simulation
gameplay/crafting-system
tools/replay-harness
```

### 6.2 Commit Convention

```
[<agent>] <type>: <description>

Types: feat, fix, refactor, test, docs, chore
```

### 6.3 Integration Requirements

- `cargo fmt --check` passes
- `cargo clippy -- -D warnings` passes
- `cargo test --workspace` passes
- No new `unsafe` without approval

---

## 7. Version Compatibility

### 7.1 Schema Versions

| Schema | Current | Min Compatible |
|--------|---------|----------------|
| Cell   | 1.0.0   | 1.0.0 |
| Chunk  | 1.0.0   | 1.0.0 |
| Event  | 1.0.0   | 1.0.0 |
| Recipe | 1.0.0   | 1.0.0 |
| Building | 1.0.0 | 1.0.0 |

### 7.2 Version Bump Rules

- **Patch**: Bug fixes, no schema change
- **Minor**: New optional fields, backwards compatible
- **Major**: Breaking changes, migration required
