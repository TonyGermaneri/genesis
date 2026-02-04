# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Iteration 11 — Crafting System

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-10

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-27 | Cell simulation, chunks, collision, quadtree | 🟢 |
| K-28 to K-31 | Multi-chunk streaming, chunk activation, env sim, day/night | 🟢 |
| K-32 to K-35 | Biome rendering, transitions, water animation | 🟢 |
| K-36 to K-39 | NPC rendering, collision, batch render, speech bubbles | 🟢 |
| K-40 to K-43 | Audio backend, spatial audio, streaming, mixing | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-28 | Player, physics, terrain manipulation | 🟢 |
| G-29 to G-32 | Grass interaction, weather, time, plant growth | 🟢 |
| G-33 to G-36 | Biome terrain generation, resource distribution | 🟢 |
| G-37 to G-40 | NPC entities, AI behaviors, spawning, dialogue | 🟢 |
| G-41 to G-44 | Sound events, ambient rules, music state, NPC sounds | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-27 | Egui, HUD, hotbar, debug panels | 🟢 |
| T-28 to T-31 | Inventory, stats, weather HUD, minimap | 🟢 |
| T-32 to T-35 | Biome minimap, debug info, seed display | 🟢 |
| T-36 to T-39 | Dialogue UI, NPC debug, spawn editor | 🟢 |
| T-40 to T-43 | Sound settings, audio debug, sound test | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-23 | CI/CD, input, game loop, egui | 🟢 |
| I-24 to I-27 | ChunkManager wiring, env integration, profiling | 🟢 |
| I-28 to I-31 | Biome generation wiring, seed management | 🟢 |
| I-32 to I-35 | NPC manager, interaction, chunk loading | 🟢 |
| I-36 to I-40 | Audio manager, asset loading, config, profiling | 🟢 |

---

## Iteration 11 — Crafting System

---

## Iteration 11 — Crafting System

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-44 | Crafting grid compute shader | ⚪ | P0 | GPU-accelerated recipe matching |
| K-45 | Item stack management | ⚪ | P0 | Efficient item combining/splitting |
| K-46 | Workbench interaction zones | ⚪ | P1 | Spatial detection for crafting stations |
| K-47 | Crafting animation support | ⚪ | P1 | Progress bar, particle effects data |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-45 | Recipe data structure | ⚪ | P0 | Define recipes, ingredients, outputs |
| G-46 | Crafting logic | ⚪ | P0 | Validate recipes, consume items, produce output |
| G-47 | Workbench types | ⚪ | P0 | Forge, anvil, alchemy table, etc. |
| G-48 | Crafting progression | ⚪ | P1 | Unlock recipes via skills/discovery |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-44 | Crafting UI grid | ⚪ | P0 | Drag-drop crafting interface |
| T-45 | Recipe book UI | ⚪ | P0 | Browse known recipes by category |
| T-46 | Crafting result preview | ⚪ | P0 | Show output item before crafting |
| T-47 | Workbench interaction UI | ⚪ | P1 | Station-specific crafting panels |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-41 | Recipe loading from assets | ⚪ | P0 | Load recipes from JSON/TOML files |
| I-42 | Crafting event integration | ⚪ | P0 | Wire crafting to inventory/sound/stats |
| I-43 | Crafting persistence | ⚪ | P0 | Save learned recipes, queue state |
| I-44 | Crafting profiling | ⚪ | P1 | Measure recipe search performance |

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
