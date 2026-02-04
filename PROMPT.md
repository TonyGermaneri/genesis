# Gameplay Agent — Iteration 14: Main Menu & Options

## Branch: `gameplay-agent`

You are implementing game session management, settings data models, and pause state handling.

---

## Your Tasks

| ID | Task | Priority | Description |
|----|------|----------|-------------|
| G-57 | Game session management | P0 | New game, continue, load states |
| G-58 | Settings data model | P0 | Graphics, audio, controls, gameplay settings |
| G-59 | World creation options | P0 | Seed input, difficulty, world size |
| G-60 | Pause state handling | P1 | Freeze game during menus |

---

## Detailed Requirements

### G-57: Game Session Management
**File:** `crates/genesis-gameplay/src/session.rs`

Manage the overall game session state:
- New game creation
- Continue from last save
- Load specific save slot
- Return to main menu

### G-58: Settings Data Model
**File:** `crates/genesis-gameplay/src/settings.rs`

Define all game settings:
- GraphicsSettings: resolution, fullscreen, vsync, render distance, quality
- AudioSettings: master, music, sfx, ambient volumes
- ControlSettings: mouse sensitivity, invert Y, key bindings
- GameplaySettings: difficulty, auto-save interval, tutorials

### G-59: World Creation Options
**File:** `crates/genesis-gameplay/src/world_creation.rs`

Options for creating a new world:
- World name validation
- Seed input (optional, random if empty)
- Difficulty selection
- World size (Small/Medium/Large)
- Starting items toggle

### G-60: Pause State Handling
**File:** `crates/genesis-gameplay/src/pause.rs`

Handle game pause/unpause:
- Freeze world updates when paused
- Track total paused time
- Continue rendering while paused

---

## Definition of Done

- [ ] Session management handles all state transitions
- [ ] Settings data model covers all options
- [ ] World creation options validate properly
- [ ] Pause correctly freezes world updates
- [ ] All tests pass
- [ ] No clippy warnings
