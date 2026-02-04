# Architecture Overview

Genesis is built as a modular, multi-crate Rust project optimized for performance and extensibility.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    genesis-engine                        │
│  ┌───────────┐  ┌───────────┐  ┌───────────────────┐   │
│  │  Renderer │  │  Config   │  │    App Loop       │   │
│  │  (wgpu)   │  │           │  │  (winit + egui)   │   │
│  └─────┬─────┘  └───────────┘  └─────────┬─────────┘   │
└────────┼────────────────────────────────────┼───────────┘
         │                                    │
┌────────┴────────────────────────────────────┴───────────┐
│                    Event Bus                             │
└────────┬────────────────────────────────────┬───────────┘
         │                                    │
┌────────┴────────┐              ┌────────────┴───────────┐
│  genesis-kernel │              │   genesis-gameplay     │
│  ┌────────────┐ │              │  ┌─────────────────┐   │
│  │   Cell     │ │              │  │    Entity       │   │
│  │ Simulation │ │              │  │    System       │   │
│  │   (GPU)    │ │              │  ├─────────────────┤   │
│  ├────────────┤ │              │  │   Inventory     │   │
│  │  Compute   │ │              │  ├─────────────────┤   │
│  │  Pipeline  │ │              │  │   Crafting      │   │
│  └────────────┘ │              │  ├─────────────────┤   │
└─────────────────┘              │  │   Economy       │   │
                                 │  └─────────────────┘   │
                                 └────────────────────────┘
         │
┌────────┴────────┐              ┌────────────────────────┐
│  genesis-world  │              │    genesis-tools       │
│  ┌────────────┐ │              │  ┌─────────────────┐   │
│  │   Chunk    │ │              │  │    Replay       │   │
│  │  Storage   │ │              │  │    System       │   │
│  ├────────────┤ │              │  ├─────────────────┤   │
│  │ Generation │ │              │  │   Inspector     │   │
│  ├────────────┤ │              │  ├─────────────────┤   │
│  │ Streaming  │ │              │  │   Perf HUD      │   │
│  └────────────┘ │              │  └─────────────────┘   │
└─────────────────┘              └────────────────────────┘
         │
┌────────┴────────┐
│ genesis-common  │
│  IDs, Coords,   │
│  Errors, Utils  │
└─────────────────┘
```

## Crate Responsibilities

### genesis-common
Shared types and utilities used across all crates:
- Entity IDs and chunk coordinates
- Error types and result wrappers
- Version information

### genesis-kernel
GPU compute pipeline for cell simulation:
- Cell state representation
- Compute shader dispatch
- Double-buffered storage
- Intent processing

### genesis-world
World data management:
- Chunk storage and serialization
- Procedural terrain generation
- Chunk streaming (load/unload)

### genesis-gameplay
Game mechanics:
- Entity system (players, NPCs, items)
- Inventory with stacking
- Crafting recipes
- Economy and trading
- Faction reputation

### genesis-tools
Development and debugging:
- Replay recording/playback
- Cell inspection
- Performance monitoring

### genesis-engine
Main application:
- Window management (winit)
- Rendering (wgpu)
- UI (egui)
- Main game loop

## Data Flow

```
    Input Events                    GPU Commands
         │                               │
         ▼                               ▼
┌─────────────────┐             ┌─────────────────┐
│   Gameplay      │   Intents   │     Kernel      │
│   Systems       │ ──────────► │   (GPU Compute) │
│   (CPU)         │             │                 │
└────────┬────────┘             └────────┬────────┘
         │                               │
         │ Queries        Cell Events    │
         ▼                               ▼
┌─────────────────────────────────────────────────┐
│                  Event Bus                       │
└─────────────────────────────────────────────────┘
                       │
                       ▼
              ┌─────────────────┐
              │    Renderer     │
              │    (wgpu)       │
              └─────────────────┘
```

## Key Patterns

### Entity-Component Pattern
Entities are stored in arena allocators with components in parallel arrays for cache efficiency.

### Intent System
Gameplay doesn't modify cells directly. Instead, it sends "intents" to the kernel:
```rust
Intent::ModifyCell { pos, new_material }
Intent::SpawnEntity { entity_type, pos }
```

### Event Bus
Cross-system communication via typed events:
```rust
event_bus.publish(CellChanged { pos, old, new });
event_bus.subscribe::<CellChanged>(|event| { ... });
```

### Chunk Streaming
World is divided into chunks that load/unload based on player position:
```rust
ChunkCoord { x: i32, y: i32 }  // 32x32 cell chunks
```

## Learn More

- [Kernel Deep Dive](kernel.md) - GPU compute details
- [World System](world.md) - Chunk and terrain generation
- [Gameplay Systems](gameplay.md) - Entities, inventory, crafting
- [Event Bus](events.md) - Inter-system communication
