# Project Genesis — Task Board

> Last Updated: 2026-02-03
> Sprint: Iteration 5 — Playable Integration

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-4

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-15 | Cell simulation, buffers, rendering, chunks, streaming, collision, biome | 🟢 |
| K-16 | Procedural world generation | 🟢 |
| K-17 | Dynamic lighting system | 🟢 |
| K-18 | Particle system | 🟢 |
| K-19 | Audio spatial integration | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-16 | Entity, inventory, crafting, economy, factions, player, physics, save/load | 🟢 |
| G-17 | Combat system | 🟢 |
| G-18 | NPC AI system | 🟢 |
| G-19 | Vehicle system | 🟢 |
| G-20 | Quest system | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-15 | Replay, inspectors, HUD, test harness, UI renderers, console | 🟢 |
| T-16 | Audio engine integration | 🟢 |
| T-17 | Quest UI | 🟢 |
| T-18 | Dialogue system UI | 🟢 |
| T-19 | Combat HUD | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-11 | CI/CD, releases, mod format, Docker, docs | 🟢 |
| I-12 | Asset pipeline | 🟢 |
| I-13 | Localization system | 🟢 |
| I-14 | Crash reporting | 🟢 |
| I-15 | Telemetry & analytics | 🟢 |

---

## Iteration 5 — Playable Integration (Active)

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| K-20 | Camera system | ⚪ | P0 |
| K-21 | World terrain rendering | ⚪ | P0 |
| K-22 | Cell rendering with camera | ⚪ | P0 |
| K-23 | Initial world and biome display | ⚪ | P1 |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| G-21 | Game state manager | ⚪ | P0 |
| G-22 | Player spawn system | ⚪ | P0 |
| G-23 | Player movement controller | ⚪ | P0 |
| G-24 | Engine integration exports | ⚪ | P0 |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| T-20 | Egui integration layer | ⚪ | P0 |
| T-21 | Game HUD renderer | ⚪ | P0 |
| T-22 | Hotbar widget | ⚪ | P0 |
| T-23 | Debug overlay | ⚪ | P1 |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| I-16 | Input system integration | ⚪ | P0 |
| I-17 | Main game loop integration | ⚪ | P0 |
| I-18 | Engine configuration | ⚪ | P0 |
| I-19 | Frame timing | ⚪ | P1 |

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
