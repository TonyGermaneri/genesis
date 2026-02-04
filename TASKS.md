# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Iteration 13 — Save/Load System

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-12

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

---

## Iteration 13 — Save/Load System

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-52 | Chunk serialization | ⚪ | P0 | Binary format for cell data |
| K-53 | World region files | ⚪ | P0 | Region-based chunk storage |
| K-54 | Compression support | ⚪ | P1 | LZ4/zstd for save files |
| K-55 | Incremental saves | ⚪ | P1 | Only save modified chunks |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-53 | Player state serialization | ⚪ | P0 | Position, inventory, stats, quests |
| G-54 | NPC state persistence | ⚪ | P0 | NPC positions, health, AI state |
| G-55 | World time/weather save | ⚪ | P0 | Day cycle, weather state |
| G-56 | Game progress tracking | ⚪ | P1 | Achievements, discovered areas |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-52 | Save/load menu UI | ⚪ | P0 | Save slot selection, new game |
| T-53 | Save slot previews | ⚪ | P0 | Screenshot, playtime, date |
| T-54 | Auto-save indicator | ⚪ | P1 | Show when auto-saving |
| T-55 | Save management UI | ⚪ | P1 | Delete, copy, export saves |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-49 | Save file manager | ⚪ | P0 | Coordinate all save/load operations |
| I-50 | Auto-save system | ⚪ | P0 | Timed auto-saves, configurable |
| I-51 | Save file versioning | ⚪ | P0 | Migration between save formats |
| I-52 | Cloud save prep | ⚪ | P1 | Abstract storage for future cloud |

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
