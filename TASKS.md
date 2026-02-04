# Project Genesis — Task Board

> Last Updated: 2026-02-04
> Sprint: Iteration 10 — Sound System

## Legend

- 🟢 Complete
- 🟡 In Progress
- 🔴 Blocked
- ⚪ Not Started

---

## Completed — Iterations 1-9

### Kernel Agent
| ID | Task | Status |
|----|------|--------|
| K-1 to K-27 | Cell simulation, chunks, collision, quadtree | 🟢 |
| K-28 to K-31 | Multi-chunk streaming, chunk activation, env sim, day/night | 🟢 |
| K-32 to K-35 | Biome rendering, transitions, water animation | 🟢 |
| K-36 to K-39 | NPC rendering, collision, batch render, speech bubbles | 🟢 |

### Gameplay Agent
| ID | Task | Status |
|----|------|--------|
| G-1 to G-28 | Player, physics, terrain manipulation | 🟢 |
| G-29 to G-32 | Grass interaction, weather, time, plant growth | 🟢 |
| G-33 to G-36 | Biome terrain generation, resource distribution | 🟢 |
| G-37 to G-40 | NPC entities, AI behaviors, spawning, dialogue | 🟢 |

### Tools Agent
| ID | Task | Status |
|----|------|--------|
| T-1 to T-27 | Egui, HUD, hotbar, debug panels | 🟢 |
| T-28 to T-31 | Inventory, stats, weather HUD, minimap | 🟢 |
| T-32 to T-35 | Biome minimap, debug info, seed display | 🟢 |
| T-36 to T-39 | Dialogue UI, NPC debug, spawn editor | 🟢 |

### Infra Agent
| ID | Task | Status |
|----|------|--------|
| I-1 to I-23 | CI/CD, input, game loop, egui | 🟢 |
| I-24 to I-27 | ChunkManager wiring, env integration, profiling | 🟢 |
| I-28 to I-31 | Biome generation wiring, seed management | 🟢 |
| I-32 to I-35 | NPC manager, interaction, chunk loading | 🟢 |

---

## Iteration 10 — Sound System

### Kernel Agent (Branch: `kernel-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| K-40 | Audio backend initialization | ⚪ | P0 | Initialize rodio/kira audio device |
| K-41 | Spatial audio positioning | ⚪ | P0 | 2D positional audio based on distance |
| K-42 | Audio streaming for music | ⚪ | P1 | Stream large MP3 files for music |
| K-43 | Audio mixing/channels | ⚪ | P1 | Separate channels for music/sfx/ambient |

### Gameplay Agent (Branch: `gameplay-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| G-41 | Sound event system | ⚪ | P0 | Events for footsteps, attacks, pickups |
| G-42 | Ambient sound rules | ⚪ | P0 | Biome-specific ambient sounds |
| G-43 | Music state machine | ⚪ | P0 | Combat/explore/menu music transitions |
| G-44 | NPC sound triggers | ⚪ | P1 | NPC voices, attack sounds |

### Tools Agent (Branch: `tools-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| T-40 | Sound settings UI | ⚪ | P0 | Volume sliders for music/sfx/ambient |
| T-41 | Audio debug panel | ⚪ | P0 | Show playing sounds, channels |
| T-42 | Sound test panel | ⚪ | P1 | Preview sounds in debug mode |
| T-43 | Jukebox/music player | ⚪ | P1 | Manual music selection (debug) |

### Infra Agent (Branch: `infra-agent`)

| ID | Task | Status | Priority | Description |
|----|------|--------|----------|-------------|
| I-36 | Audio manager integration | ⚪ | P0 | Add audio to game loop |
| I-37 | Sound asset loading | ⚪ | P0 | Load MP3/WAV from assets folder |
| I-38 | Audio config persistence | ⚪ | P0 | Save/load volume settings |
| I-39 | Sound performance profiling | ⚪ | P1 | Track audio CPU usage
| I-33 | NPC-player interaction | ⚪ | P0 | Detect interact key near NPCs |
| I-34 | NPC chunk loading | ⚪ | P0 | Load/unload NPCs with chunks |
| I-35 | NPC update profiling | ⚪ | P1 | Measure AI tick performance
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
