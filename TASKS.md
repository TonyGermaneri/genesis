# Project Genesis — Task Board

> Last Updated: 2026-02-03
> Sprint: Iteration 4 — Core Loop

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-3

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-11 | Cell simulation, buffers, rendering, chunks | 🟢 |
| K-12 | Chunk streaming system | 🟢 |
| K-13 | Collision query system | 🟢 |
| K-14 | Biome material assignment | 🟢 |
| K-15 | GPU readback optimization | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-12 | Entity, inventory, crafting, economy, factions, player | 🟢 |
| G-13 | Player physics integration | 🟢 |
| G-14 | Inventory UI model | 🟢 |
| G-15 | Crafting UI model | 🟢 |
| G-16 | Save/load game state | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-11 | Replay, inspectors, HUD, test harness, hot reload | 🟢 |
| T-12 | Inventory UI renderer | 🟢 |
| T-13 | Crafting UI renderer | 🟢 |
| T-14 | Minimap renderer | 🟢 |
| T-15 | Debug console | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-7 | CI/CD, releases, mod format | 🟢 |
| I-8 | Performance regression CI | 🟢 |
| I-9 | Documentation site | 🟢 |
| I-10 | Build telemetry | 🟢 |
| I-11 | Docker development image | 🟢 |

---

## Iteration 4 — Active

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| K-16 | Procedural world generation | ⚪ | P0 |
| K-17 | Dynamic lighting system | ⚪ | P0 |
| K-18 | Particle system | ⚪ | P1 |
| K-19 | Audio spatial integration | ⚪ | P1 |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| G-17 | Combat system | ⚪ | P0 |
| G-18 | NPC AI system | ⚪ | P0 |
| G-19 | Vehicle system | ⚪ | P0 |
| G-20 | Quest system | ⚪ | P1 |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| T-16 | Audio engine integration | ⚪ | P0 |
| T-17 | Quest UI | ⚪ | P0 |
| T-18 | Dialogue system UI | ⚪ | P0 |
| T-19 | Combat HUD | ⚪ | P1 |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| I-12 | Asset pipeline | ⚪ | P0 |
| I-13 | Localization system | ⚪ | P0 |
| I-14 | Crash reporting | ⚪ | P1 |
| I-15 | Telemetry & analytics | ⚪ | P2 |

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
