# Project Genesis — Task Board

> Last Updated: 2026-02-03
> Sprint: Day 1 — Bootstrap

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Day 1 Objectives

### Orchestrator Tasks

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| O-1 | Create workspace structure | 🟢 | Orchestrator |
| O-2 | Define contracts and schemas | 🟢 | Orchestrator |
| O-3 | Set up build tooling (just) | 🟢 | Orchestrator |
| O-4 | Create Nix flake | 🟢 | Orchestrator |
| O-5 | Write agent prompts | 🟢 | Orchestrator |
| O-6 | First commit | 🟡 | Orchestrator |

---

## Agent Assignments

### Kernel Agent (Branch: `kernel/main`)

**Scope**: GPU compute pipeline, cell simulation, buffer management

| ID | Task | Status | Priority |
|----|------|--------|----------|
| K-1 | Implement cell simulation shader | ⚪ | P0 |
| K-2 | Create double-buffered cell storage | ⚪ | P0 |
| K-3 | Implement intent buffer upload | ⚪ | P1 |
| K-4 | Implement event buffer readback | ⚪ | P1 |
| K-5 | Add material property LUT | ⚪ | P1 |
| K-6 | GPU validation layer integration | ⚪ | P2 |
| K-7 | Benchmark compute dispatch | ⚪ | P2 |

**Acceptance Criteria**:
- Compute shader compiles and dispatches
- Cell state persists across frames
- Intent → cell modification works
- All tests pass

---

### Gameplay Agent (Branch: `gameplay/main`)

**Scope**: Entity system, inventory, crafting, economy, factions

| ID | Task | Status | Priority |
|----|------|--------|----------|
| G-1 | Entity storage (arena allocator) | ⚪ | P0 |
| G-2 | Inventory system with stacking | ⚪ | P0 |
| G-3 | Crafting recipe execution | ⚪ | P0 |
| G-4 | Building placement system | ⚪ | P0 |
| G-5 | Economy: wallet and prices | ⚪ | P1 |
| G-6 | Faction reputation tracking | ⚪ | P1 |
| G-7 | Needs system (hunger/thirst) | ⚪ | P1 |
| G-8 | Vehicle entity type | ⚪ | P2 |
| G-9 | NPC traffic simulation | ⚪ | P2 |

**Acceptance Criteria**:
- Entity CRUD operations work
- Inventory add/remove/transfer work
- Crafting consumes ingredients, produces output
- Buildings modify world via intents
- All tests pass

---

### Tools Agent (Branch: `tools/main`)

**Scope**: Development tools, replay system, inspectors

| ID | Task | Status | Priority |
|----|------|--------|----------|
| T-1 | Replay recording | ⚪ | P0 |
| T-2 | Replay playback | ⚪ | P0 |
| T-3 | Determinism verification | ⚪ | P1 |
| T-4 | Chunk viewer (egui) | ⚪ | P1 |
| T-5 | Cell inspector probe | ⚪ | P1 |
| T-6 | Performance HUD | ⚪ | P2 |
| T-7 | Event log viewer | ⚪ | P2 |

**Acceptance Criteria**:
- Record 1000 frames, play back identically
- Inspector shows cell properties
- Perf HUD shows FPS, frame time
- All tests pass

---

### Infra Agent (Branch: `infra/main`)

**Scope**: CI/CD, toolchains, mod packaging

| ID | Task | Status | Priority |
|----|------|--------|----------|
| I-1 | GitHub Actions workflow | 🟢 | P0 |
| I-2 | Clippy + rustfmt in CI | 🟢 | P0 |
| I-3 | Test runner in CI | 🟢 | P0 |
| I-4 | Nix build in CI | 🟢 | P1 |
| I-5 | Release artifact packaging | ⚪ | P2 |
| I-6 | Mod package format | ⚪ | P2 |

**Acceptance Criteria**:
- CI runs on every PR
- CI fails on lint/test failures
- CI passes currently
- All tests pass

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

### Milestone 1: Bootstrap (Day 1)
- [x] Repo structure
- [x] Contracts defined
- [x] Build tooling
- [ ] First commit (in progress)

### Milestone 2: Minimal Viable Kernel (Day 2-3)
- [ ] Cell simulation working
- [ ] Chunk load/save working
- [ ] Basic rendering

### Milestone 3: Playable Prototype (Week 1)
- [ ] Player movement
- [ ] Basic crafting
- [ ] Inventory UI
- [ ] One biome generated

### Milestone 4: Core Loop (Week 2)
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
