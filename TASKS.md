# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Complete — All 14 Iterations Done!

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-14

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
| K-56 to K-59 | Menu backdrop, transitions, screenshots, resolution | 🟢 |

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
| G-57 to G-60 | Session management, settings, world creation, pause | 🟢 |

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
| T-56 to T-59 | Main menu, pause menu, options menu, new game wizard | 🟢 |

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
| I-53 to I-56 | Menu state machine, settings persistence, input rebinding, exit | 🟢 |

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

### Milestone 4: Core Loop ✅
- [x] Main menu and options
- [x] Full game session management
- [x] Settings persistence
- [x] Input rebinding

### Milestone 5: Future Work
- [ ] Economy system
- [ ] Vehicle entry/exit
- [ ] Multiplayer prep
- [ ] Steam integration

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
