# Project Genesis — Task Board

> Last Updated: 2026-02-03
> Sprint: Iteration 3 — Playable Prototype

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iteration 1 & 2

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 | Cell simulation shader | 🟢 |
| K-2 | Double-buffered cell storage | 🟢 |
| K-3 | Intent buffer upload | 🟢 |
| K-4 | Event buffer readback | 🟢 |
| K-5 | Material property LUT | 🟢 |
| K-6 | GPU validation layer | 🟢 |
| K-7 | Benchmark compute dispatch | 🟢 |
| K-8 | Compute+render integration | 🟢 |
| K-9 | Cell rendering pipeline | 🟢 |
| K-10 | Multi-chunk management | 🟢 |
| K-11 | Edge cell sharing | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 | Entity storage (arena) | 🟢 |
| G-2 | Inventory with stacking | 🟢 |
| G-3 | Crafting recipe execution | 🟢 |
| G-4 | Building placement | 🟢 |
| G-5 | Economy: wallet/prices | 🟢 |
| G-6 | Faction reputation | 🟢 |
| G-7 | Needs system | 🟢 |
| G-10 | Player controller | 🟢 |
| G-11 | Input handling | 🟢 |
| G-12 | World interaction (dig/place) | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 | Replay recording | 🟢 |
| T-2 | Replay playback | 🟢 |
| T-3 | Determinism verification | 🟢 |
| T-4 | Chunk viewer (egui) | 🟢 |
| T-5 | Cell inspector probe | 🟢 |
| T-6 | Performance HUD | 🟢 |
| T-7 | Event log viewer | 🟢 |
| T-8 | Test harness | 🟢 |
| T-9 | Screenshot tests | 🟢 |
| T-10 | Memory profiler | 🟢 |
| T-11 | Hot reload | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 | GitHub Actions workflow | 🟢 |
| I-2 | Clippy + rustfmt in CI | 🟢 |
| I-3 | Test runner in CI | 🟢 |
| I-4 | Nix build in CI | 🟢 |
| I-5 | Release artifact packaging | 🟢 |
| I-6 | Mod package format | 🟢 |
| I-7 | Cross-platform builds | 🟢 |
| I-8 | Performance regression CI | 🟢 |
| I-9 | Documentation site (mdBook) | 🟢 |
| I-10 | Build telemetry | 🟢 |
| I-11 | Docker development image | 🟢 |
| I-12 | Asset pipeline | 🟢 |
| I-13 | Localization system | 🟢 |
| I-14 | Crash reporting | 🟢 |
| I-15 | Telemetry & Analytics | 🟢 |

---

## Iteration 3 — Active

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| K-12 | Chunk streaming system | ⚪ | P0 |
| K-13 | Collision query system | ⚪ | P0 |
| K-14 | Biome material assignment | ⚪ | P1 |
| K-15 | GPU readback optimization | ⚪ | P1 |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| G-13 | Player physics integration | ⚪ | P0 |
| G-14 | Inventory UI model | ⚪ | P0 |
| G-15 | Crafting UI model | ⚪ | P0 |
| G-16 | Save/load game state | ⚪ | P1 |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| T-12 | Inventory UI renderer | ⚪ | P0 |
| T-13 | Crafting UI renderer | ⚪ | P0 |
| T-14 | Minimap renderer | ⚪ | P1 |
| T-15 | Debug console | ⚪ | P1 |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority |
|----|------|--------|----------|
| I-8 | Performance regression CI | 🟢 | P0 |
| I-9 | Documentation site (mdBook) | 🟢 | P1 |
| I-10 | Build telemetry | 🟢 | P1 |
| I-11 | Docker development image | 🟢 | P2 |
| I-12 | Asset pipeline | 🟢 | P1 |
| I-13 | Localization system | 🟢 | P1 |
| I-14 | Crash reporting | 🟢 | P1 |
| I-15 | Telemetry & Analytics | 🟢 | P2 |

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
