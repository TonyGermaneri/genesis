# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Iteration 9 — NPC System

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-8

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-27 | Cell simulation, chunks, collision, quadtree | 🟢 |
| K-28 to K-31 | Multi-chunk streaming, chunk activation, env sim, day/night | 🟢 |
| K-32 to K-35 | Biome rendering, transitions, water animation | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-28 | Player, physics, terrain manipulation | 🟢 |
| G-29 to G-32 | Grass interaction, weather, time, plant growth | 🟢 |
| G-33 to G-36 | Biome terrain generation, resource distribution | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-27 | Egui, HUD, hotbar, debug panels | 🟢 |
| T-28 to T-31 | Inventory, stats, weather HUD, minimap | 🟢 |
| T-32 to T-35 | Biome minimap, debug info, seed display | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-23 | CI/CD, input, game loop, egui | 🟢 |
| I-24 to I-27 | ChunkManager wiring, env integration, profiling | 🟢 |
| I-28 to I-31 | Biome generation wiring, seed management | 🟢 |

---

## Iteration 9 — NPC System

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-36 | NPC sprite rendering | ⚪ | P0 | Render NPC entities with direction/animation |
| K-37 | NPC collision detection | ⚪ | P0 | Circle collision for NPC bodies |
| K-38 | NPC batch rendering | ⚪ | P1 | Efficient instanced rendering for many NPCs |
| K-39 | Speech bubble rendering | ⚪ | P1 | Render dialogue text above NPCs |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-37 | NPC entity system | ⚪ | P0 | NPC struct with position, state, type |
| G-38 | NPC AI behavior trees | ⚪ | P0 | Patrol, idle, chase, flee behaviors |
| G-39 | NPC spawning system | ⚪ | P0 | Spawn rules per biome, density limits |
| G-40 | Dialogue system | ⚪ | P1 | Dialogue trees, conversation state |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-36 | Dialogue UI panel | ⚪ | P0 | Show NPC dialogue with choices |
| T-37 | NPC debug overlay | ⚪ | P0 | Show NPC state, AI path, targets |
| T-38 | NPC spawn editor | ⚪ | P1 | Debug tool to spawn/remove NPCs |
| T-39 | NPC list panel | ⚪ | P1 | List all NPCs in loaded chunks |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-32 | NPC manager integration | ⚪ | P0 | Add NPC system to game loop |
| I-33 | NPC-player interaction | ⚪ | P0 | Detect interact key near NPCs |
| I-34 | NPC chunk loading | ⚪ | P0 | Load/unload NPCs with chunks |
| I-35 | NPC update profiling | ⚪ | P1 | Measure AI tick performance
| G-36 | Biome-specific cell types | ⚪ | P1 | Sand, water, grass variants per biome |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-32 | Biome minimap coloring | ⚪ | P0 | Color-coded biomes on minimap |
| T-33 | Debug biome info panel | ⚪ | P0 | Show current biome, noise values |
| T-34 | World seed display/input | ⚪ | P0 | Show seed, allow seed input for new worlds |
| T-35 | Biome legend overlay | ⚪ | P1 | Color key for biome types |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-28 | Wire biome generation | ⚪ | P0 | Call terrain gen on chunk creation |
| I-29 | World seed management | ⚪ | P0 | Seed storage, deterministic generation |
| I-30 | Chunk biome data flow | ⚪ | P0 | Pass biome info from gameplay to kernel |
| I-31 | Biome generation profiling | ⚪ | P1 | Measure gen time per chunk |

---

## Integration Checklist

Before merging any agent branch:

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] No merge conflicts with `main`
- [ ] Contracts adhered to (see `spec/CONTRACTS.md`)
- [ ] Documentation updated if API changed

---

## Milestones

### Milestone 1: Bootstrap (Day 1) ✅
- [x] Repo structure
- [x] Contracts defined
- [x] Build tooling
- [x] First commit

### Milestone 2: Minimal Viable Kernel ✅
- [x] Cell simulation working
- [x] Multi-chunk management
- [x] Cell rendering pipeline
- [x] Edge sharing between chunks

### Milestone 3: Playable Prototype (In Progress)
- [ ] Player movement with physics
- [ ] Inventory UI
- [ ] Crafting UI
- [ ] Biome generation
- [ ] Save/Load system

### Milestone 4: Core Loop (Upcoming)
- [ ] Combat system
- [ ] NPC spawning
- [ ] Economy active
- [ ] Vehicle entry/exit

---

## Notes

### Agent Communication
- Agents do NOT communicate directly
- All coordination through orchestrator
- Use event bus for runtime communication

### Worktree Setup
See `docs/WORKTREE_SETUP.md` for git worktree commands.

### Build Commands
```bash
just build       # Build all crates
just test        # Run all tests
just lint        # Format + clippy
just validate    # Full validation loop
just run         # Run engine
```
