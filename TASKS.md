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
| K-1 to K-27 | Cell simulation, chunks, collision, quadtree | 🟢 |
| K-28 to K-31 | Multi-chunk streaming, chunk activation, env sim, day/night | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-28 | Player, physics, terrain manipulation | 🟢 |
| G-29 to G-32 | Grass interaction, weather, time, plant growth | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-27 | Egui, HUD, hotbar, debug panels | 🟢 |
| T-28 to T-31 | Inventory, stats, weather HUD, minimap | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-23 | CI/CD, input, game loop, egui | 🟢 |
| I-24 to I-27 | ChunkManager wiring, env integration, profiling | 🟢 |

---

## Iteration 8 — Procedural Biomes

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-32 | Simplex noise generation | ⚪ | P0 | GPU-friendly noise for terrain height/moisture |
| K-33 | Biome-aware cell rendering | ⚪ | P0 | Different cell colors/textures per biome |
| K-34 | Biome transition blending | ⚪ | P0 | Smooth gradients between adjacent biomes |
| K-35 | Water rendering for lakes | ⚪ | P1 | Animated water shader for lake biomes |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-33 | Biome type definitions | ⚪ | P0 | Forest, desert, lake, plains, mountain enums |
| G-34 | Terrain generation logic | ⚪ | P0 | Noise-based biome assignment per chunk |
| G-35 | Biome resource distribution | ⚪ | P0 | Trees in forest, cacti in desert, fish in lakes |
| G-36 | Biome-specific cell types | ⚪ | P1 | Sand, water, grass variants per biome |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-32 | Biome minimap coloring | ⚪ | P0 | Color-coded biomes on minimap |
| T-33 | Debug biome info panel | ⚪ | P0 | Show current biome, noise values |
| T-34 | World seed display/input | ⚪ | P0 | Show seed, allow seed input for new worlds |
| T-35 | Biome legend overlay | ⚪ | P1 | Color key for biome types |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-28 | Wire biome generation | ⚪ | P0 | Call terrain gen on chunk creation |
| I-29 | World seed management | ⚪ | P0 | Seed storage, deterministic generation |
| I-30 | Chunk biome data flow | ⚪ | P0 | Pass biome info from gameplay to kernel |
| I-31 | Biome generation profiling | ⚪ | P1 | Measure gen time per chunk |

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
