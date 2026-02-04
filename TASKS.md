# Project Genesis — Task Board

> Last Updated: 2026-02-03
> Sprint: Iteration 6 — Interactive World

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-5

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-19 | Cell simulation, buffers, rendering, chunks, streaming, collision, biome, world gen, lighting, particles, audio | 🟢 |
| K-20 to K-23 | Camera system, terrain rendering, world init | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-20 | Entity, inventory, crafting, economy, factions, player, physics, save/load, combat, NPC AI, vehicles, quests | 🟢 |
| G-21 to G-24 | Game state manager, spawn system, movement controller | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-19 | Replay, inspectors, HUD, test harness, UI renderers, console, audio, quest UI, dialogue, combat HUD | 🟢 |
| T-20 to T-23 | Egui integration, game HUD, hotbar, debug overlay | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-15 | CI/CD, releases, mod format, Docker, docs, assets, localization, crash reports, analytics | 🟢 |
| I-16 to I-19 | Input system, game loop, config, frame timing | 🟢 |

---

## Iteration 6 — Interactive World (Active)

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-24 | Quadtree spatial partitioning | ⚪ | P0 | O(log n) spatial queries for simulation |
| K-25 | Multi-chunk visible area rendering | ⚪ | P0 | Load/unload chunks based on camera |
| K-26 | Player-terrain collision detection | ⚪ | P0 | Circle-vs-cells collision |
| K-27 | Top-down physics model | ⚪ | P1 | Friction-based movement, no gravity |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-25 | Terrain manipulation system | ⚪ | P0 | Dig/place cells with brush |
| G-26 | Top-down player controller | ⚪ | P0 | 8-direction movement with friction |
| G-27 | Player-world collision response | ⚪ | P0 | Smooth wall sliding |
| G-28 | Interaction system | ⚪ | P1 | Wire dig/place to input |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-24 | Egui renderer integration | ⚪ | P0 | Render egui on top of world |
| T-25 | Main game HUD | ⚪ | P0 | Health, hotbar, minimap |
| T-26 | Inventory UI panel | ⚪ | P0 | Drag-drop inventory grid |
| T-27 | Crafting UI panel | ⚪ | P1 | Recipe list and crafting |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-20 | Egui in main render loop | ⚪ | P0 | Wire egui to renderer |
| I-21 | Multi-chunk terrain integration | ⚪ | P0 | Use ChunkManager in renderer |
| I-22 | Player z-index fix | ⚪ | P0 | Player renders above terrain |
| I-23 | Performance profiling | ⚪ | P1 | Metrics in debug overlay |

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
