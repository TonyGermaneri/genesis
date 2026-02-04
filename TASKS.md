# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Iteration 14 — Main Menu & Options

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-13

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-27 | Cell simulation, chunks, collision, quadtree | 🟢 |
| K-28 to K-31 | Multi-chunk streaming, chunk activation, env sim, day/night | 🟢 |
| K-32 to K-35 | Biome rendering, transitions, water animation | 🟢 |
| K-36 to K-39 | NPC rendering, collision, batch render, speech bubbles | 🟢 |
| K-40 to K-43 | Audio backend, spatial audio, streaming, mixing | 🟢 |
| K-44 to K-47 | Crafting grid, item stacks, workbench zones, animations | 🟢 |
| K-48 to K-51 | Combat collision, projectiles, damage render, particles | 🟢 |
| K-52 to K-55 | Chunk serialization, region files, compression, incremental | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-28 | Player, physics, terrain manipulation | 🟢 |
| G-29 to G-32 | Grass interaction, weather, time, plant growth | 🟢 |
| G-33 to G-36 | Biome terrain generation, resource distribution | 🟢 |
| G-37 to G-40 | NPC entities, AI behaviors, spawning, dialogue | 🟢 |
| G-41 to G-44 | Sound events, ambient rules, music state, NPC sounds | 🟢 |
| G-45 to G-48 | Recipes, crafting logic, workbench types, progression | 🟢 |
| G-49 to G-52 | Combat stats, melee/ranged attacks, damage calc | 🟢 |
| G-53 to G-56 | Player save, NPC save, world state, progress tracking | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-27 | Egui, HUD, hotbar, debug panels | 🟢 |
| T-28 to T-31 | Inventory, stats, weather HUD, minimap | 🟢 |
| T-32 to T-35 | Biome minimap, debug info, seed display | 🟢 |
| T-36 to T-39 | Dialogue UI, NPC debug, spawn editor | 🟢 |
| T-40 to T-43 | Sound settings, audio debug, sound test | 🟢 |
| T-44 to T-47 | Crafting UI, recipe book, workbench panels | 🟢 |
| T-48 to T-51 | Health bars, combat HUD, equipment stats, combat debug | 🟢 |
| T-52 to T-55 | Save menu, slot previews, auto-save indicator, management | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-23 | CI/CD, input, game loop, egui | 🟢 |
| I-24 to I-27 | ChunkManager wiring, env integration, profiling | 🟢 |
| I-28 to I-31 | Biome generation wiring, seed management | 🟢 |
| I-32 to I-35 | NPC manager, interaction, chunk loading | 🟢 |
| I-36 to I-40 | Audio manager, asset loading, config, profiling | 🟢 |
| I-41 to I-44 | Recipe loading, crafting events, persistence | 🟢 |
| I-45 to I-48 | Combat events, weapon loading, combat persistence | 🟢 |
| I-49 to I-52 | Save manager, auto-save, versioning, cloud prep | 🟢 |

---

## Iteration 14 — Main Menu & Options

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-56 | Menu background rendering | ⚪ | P0 | Animated/static menu backdrop |
| K-57 | Transition effects | ⚪ | P1 | Fade in/out between screens |
| K-58 | Screenshot capture | ⚪ | P1 | For save slot thumbnails |
| K-59 | Resolution switching | ⚪ | P1 | Apply resolution changes |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-57 | Game session management | ⚪ | P0 | New game, continue, load states |
| G-58 | Settings data model | ⚪ | P0 | Graphics, audio, controls, gameplay |
| G-59 | World creation options | ⚪ | P0 | Seed input, difficulty, world size |
| G-60 | Pause state handling | ⚪ | P1 | Freeze game during menus |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-56 | Main menu UI | ⚪ | P0 | New Game, Continue, Load, Options, Exit |
| T-57 | Pause/ESC menu UI | ⚪ | P0 | Resume, Save, Load, Options, Quit to Menu |
| T-58 | Options menu UI | ⚪ | P0 | Graphics, Audio, Controls, Gameplay tabs |
| T-59 | New game wizard UI | ⚪ | P1 | World name, seed, difficulty selection |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-53 | Menu state machine | ⚪ | P0 | MainMenu → Playing → Paused transitions |
| I-54 | Settings persistence | ⚪ | P0 | Save/load settings.toml |
| I-55 | Input rebinding system | ⚪ | P0 | Configurable key bindings |
| I-56 | Graceful exit handling | ⚪ | P1 | Save on exit, cleanup resources |

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

### Milestone 3: Playable Prototype ✅
- [x] Player movement with physics
- [x] Inventory UI
- [x] Crafting UI
- [x] Biome generation
- [x] Save/Load system
- [x] Combat system
- [x] NPC spawning

### Milestone 4: Core Loop (In Progress)
- [ ] Main menu and options
- [ ] Full game session management
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
