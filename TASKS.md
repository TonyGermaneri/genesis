# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Iteration 12 — Combat System

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-11

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-27 | Cell simulation, chunks, collision, quadtree | 🟢 |
| K-28 to K-31 | Multi-chunk streaming, chunk activation, env sim, day/night | 🟢 |
| K-32 to K-35 | Biome rendering, transitions, water animation | 🟢 |
| K-36 to K-39 | NPC rendering, collision, batch render, speech bubbles | 🟢 |
| K-40 to K-43 | Audio backend, spatial audio, streaming, mixing | 🟢 |
| K-44 to K-47 | Crafting grid, item stacks, workbench zones, animations | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-28 | Player, physics, terrain manipulation | 🟢 |
| G-29 to G-32 | Grass interaction, weather, time, plant growth | 🟢 |
| G-33 to G-36 | Biome terrain generation, resource distribution | 🟢 |
| G-37 to G-40 | NPC entities, AI behaviors, spawning, dialogue | 🟢 |
| G-41 to G-44 | Sound events, ambient rules, music state, NPC sounds | 🟢 |
| G-45 to G-48 | Recipes, crafting logic, workbench types, progression | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-27 | Egui, HUD, hotbar, debug panels | 🟢 |
| T-28 to T-31 | Inventory, stats, weather HUD, minimap | 🟢 |
| T-32 to T-35 | Biome minimap, debug info, seed display | 🟢 |
| T-36 to T-39 | Dialogue UI, NPC debug, spawn editor | 🟢 |
| T-40 to T-43 | Sound settings, audio debug, sound test | 🟢 |
| T-44 to T-47 | Crafting UI, recipe book, workbench panels | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-23 | CI/CD, input, game loop, egui | 🟢 |
| I-24 to I-27 | ChunkManager wiring, env integration, profiling | 🟢 |
| I-28 to I-31 | Biome generation wiring, seed management | 🟢 |
| I-32 to I-35 | NPC manager, interaction, chunk loading | 🟢 |
| I-36 to I-40 | Audio manager, asset loading, config, profiling | 🟢 |
| I-41 to I-44 | Recipe loading, crafting events, persistence | 🟢 |

---

## Iteration 12 — Combat System

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-48 | Hitbox/hurtbox collision | ⚪ | P0 | Attack collision detection |
| K-49 | Projectile physics | ⚪ | P0 | Arrow, spell projectile trajectories |
| K-50 | Damage number rendering | ⚪ | P1 | Floating damage text sprites |
| K-51 | Combat particle effects | ⚪ | P1 | Hit sparks, blood, impact effects |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-49 | Combat stats system | ⚪ | P0 | HP, attack, defense, crit, dodge |
| G-50 | Melee attack logic | ⚪ | P0 | Swing timing, combos, stamina cost |
| G-51 | Ranged attack logic | ⚪ | P0 | Bow, crossbow, throwing weapons |
| G-52 | Damage calculation | ⚪ | P0 | Formulas, armor, resistances, crits |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-48 | Health/stamina bars | ⚪ | P0 | Player and target health UI |
| T-49 | Combat HUD | ⚪ | P0 | Combo counter, damage taken indicator |
| T-50 | Equipment stats panel | ⚪ | P1 | Show weapon damage, armor values |
| T-51 | Combat debug overlay | ⚪ | P1 | Hitbox visualization, damage log |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-45 | Combat event system | ⚪ | P0 | Wire attacks to damage, sounds, effects |
| I-46 | Weapon data loading | ⚪ | P0 | Load weapon stats from assets |
| I-47 | Combat state persistence | ⚪ | P0 | Save HP, status effects |
| I-48 | Combat profiling | ⚪ | P1 | Measure collision check performance |

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
