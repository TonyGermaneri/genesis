# Project Genesis — Architecture

> Version: 1.0.0  
> Last Updated: 2026-02-03

## Overview

Project Genesis is an action RPG with GPU-compute pixel-cell simulation. This document
describes the high-level architecture and design decisions.

## System Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                   ENGINE                                         │
│  ┌───────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │   Input   │───►│   Gameplay   │───►│    World     │───►│   Renderer   │     │
│  │  (winit)  │    │   (CPU)      │    │   (CPU)      │    │   (wgpu)     │     │
│  └───────────┘    └──────┬───────┘    └──────┬───────┘    └──────────────┘     │
│                          │                    │                                  │
│                          │    Event Bus       │                                  │
│                    ┌─────▼────────────────────▼─────┐                           │
│                    │         GPU KERNEL             │                           │
│                    │   (wgpu compute pipeline)      │                           │
│                    │                                │                           │
│                    │  ┌─────────────────────────┐  │                           │
│                    │  │   Cell Simulation       │  │                           │
│                    │  │   (physical authority)  │  │                           │
│                    │  └─────────────────────────┘  │                           │
│                    └────────────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Core Principles

### 1. GPU-First Design

The GPU kernel is the **physical authority** for the world. Every pixel is a simulated
cell with material properties, temperature, and velocity. The CPU entity layer submits
**intents** that the GPU executes during simulation.

```
CPU Intent → GPU Buffer → Compute Shader → Cell State Change → GPU Event → CPU Handler
```

### 2. Dual Authority Model

| System | Authority | Location |
|--------|-----------|----------|
| Cell Simulation | Physical (materials, physics) | GPU Kernel |
| Entity Logic | RPG (health, inventory, AI) | CPU Gameplay |
| Persistence | Storage (save/load) | CPU World |
| Presentation | Rendering | GPU Renderer |

### 3. Chunked Infinite World

The world is divided into chunks (default 256×256 pixels). Chunks are:
- Generated procedurally on demand
- Streamed from disk when revisited
- Unloaded when far from player
- Saved when modified (dirty flag)

### 4. Event-Driven Communication

Systems communicate via an event bus. This enables:
- Loose coupling between systems
- Mod hooks at event points
- Replay/determinism (record/playback events)
- Debugging (event logs)

## Crate Structure

```
genesis/
├── crates/
│   ├── genesis-engine    # Main binary, window, app loop
│   ├── genesis-kernel    # GPU compute, cell simulation
│   ├── genesis-gameplay  # Entities, inventory, economy
│   ├── genesis-world     # Chunks, streaming, generation
│   ├── genesis-common    # Shared types, IDs, coords
│   └── genesis-tools     # Dev tools, replay, inspector
├── spec/                 # Contracts and schemas
├── docs/                 # Documentation
└── tools/                # Build scripts, asset tools
```

## Data Flow

### Frame Loop

```
1. Input Collection (winit events)
         │
         ▼
2. Gameplay Tick (entity updates, AI, physics intents)
         │
         ▼
3. GPU Simulation (cell compute, physics resolution)
         │
         ▼
4. GPU → CPU Events (destruction, state changes)
         │
         ▼
5. World Update (chunk management, persistence)
         │
         ▼
6. Render (present GPU buffers to screen)
         │
         ▼
7. Event Dispatch (notify subscribers)
```

### Chunk Lifecycle

```
                    ┌─────────────┐
                    │  Unloaded   │
                    └──────┬──────┘
                           │ load/generate
                           ▼
                    ┌─────────────┐
              ┌────►│   Loaded    │◄────┐
              │     └──────┬──────┘     │
              │            │ modify     │ save
              │            ▼            │
              │     ┌─────────────┐     │
              │     │    Dirty    │─────┘
              │     └──────┬──────┘
              │            │ unload (far)
              │            ▼
              │     ┌─────────────┐
              └─────│   Saved     │
                    └─────────────┘
```

## Gameplay Systems

### Entity Component Model

Entities have components attached:
- **Position**: World coordinates
- **Health**: Current/max HP
- **Inventory**: Item storage
- **Wallet**: Currency balance
- **Needs**: Hunger, thirst
- **Faction**: Allegiance, reputation
- **Vehicle**: If entity is a vehicle

### Crafting System

Two types of crafting:

1. **Item Crafting**: Combine ingredients → produce items
   - Recipes define ingredients, tools, skill requirements
   - Output quality based on skill
   - XP gained on success

2. **Building Crafting**: Place structures in world
   - Components consumed from inventory
   - Building modifies cell data (blocking, etc.)
   - Buildings can have production recipes

### Economy

- **Prices**: Dynamic based on supply/demand
- **Degradation**: Items wear out over time
- **Repair**: Restore item condition with materials
- **Trade**: Buy/sell with NPCs, other players

### Factions

- Multiple factions with relations to each other
- Player reputation per faction
- Reputation affects: prices, quest access, NPC behavior
- Actions affect reputation (combat, trade, quests)

## Dual Gameplay Modes

### Top-Down Overworld

- NES Jackal-style vehicle combat
- Large-scale exploration
- Vehicle entry/exit
- City navigation

### Platform Interiors

- NES River City Ransom-style combat
- Building exploration
- Melee combat
- NPC interaction

Modes are linked: entering a building door transitions to interior mode.

## Mod System

Mods can:
- Add recipes, items, buildings, materials
- Subscribe to events
- Define new factions
- Extend world generation

Mods cannot:
- Access filesystem directly
- Access network
- Modify core engine code
- Break other mods

## Performance Targets

| Metric | Target |
|--------|--------|
| Frame rate | 60 FPS |
| Chunk load time | < 16ms |
| Entity count | 10,000+ |
| Cell updates/frame | 1M+ |
| Memory (chunks) | < 2GB |

## Future Considerations

- Multiplayer (networking layer)
- Lua/Rhai scripting for mods
- Advanced AI (behavior trees)
- Procedural quests
- Audio system integration
