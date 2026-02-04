# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Iteration 8 — Procedural World & Polish

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-7

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-31 | Cell simulation, chunks, collision, quadtree, multi-chunk render, env shaders, day/night | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-32 | Player, physics, terrain manipulation, weather, time, plant growth | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-31 | Egui, HUD, hotbar, debug, inventory, stats, weather HUD, minimap | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-27 | CI/CD, input, game loop, egui, ChunkManager, env wiring, profiling | 🟢 |

---

## Iteration 8 — Procedural World & Polish (Pending)

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-24 | Quadtree spatial partitioning | ⚪ | P0 | O(log n) spatial queries for simulation |
| K-25 | Multi-chunk visible area rendering | ⚪ | P0 | Load/unload chunks based on camera |
| K-26 | Player-terrain collision detection | ⚪ | P0 | Circle-vs-cells collision |
| K-27 | Top-down physics model | ⚪ | P1 | Friction-based movement, no gravity |

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-28 | Multi-chunk streaming render | ⚪ | P0 | Render multiple chunks around player |
| K-29 | Quadtree chunk activation | ⚪ | P0 | Only simulate active chunks |
| K-30 | Environment simulation shader | ⚪ | P1 | Grass growth, rain effects |
| K-31 | Day/night cycle rendering | ⚪ | P1 | Time-based lighting |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-29 | Grass interaction system | ⚪ | P0 | Cut grass, get items |
| G-30 | Weather state system | ⚪ | P0 | Clear/cloudy/rain/storm |
| G-31 | Time/day cycle system | ⚪ | P0 | Game time with day/night |
| G-32 | Plant growth system | ⚪ | P1 | Growth stages, harvesting |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-28 | Inventory panel UI | ⚪ | P0 | 6x9 grid inventory |
| T-29 | Player stats HUD | ⚪ | P0 | Health, hunger, stamina bars |
| T-30 | Weather/time HUD | ⚪ | P0 | Clock, weather icon |
| T-31 | Minimap with chunks | ⚪ | P1 | 5x5 chunk minimap |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-24 | ChunkManager in render loop | ⚪ | P0 | Enable multi-chunk mode |
| I-25 | Weather/time to kernel | ⚪ | P0 | Pass env state to shaders |
| I-26 | Wire UI systems to app | ⚪ | P0 | Connect all HUD elements |
| I-27 | Multi-chunk perf profiling | ⚪ | P1 | Chunk metrics in debug |

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
