# Infra Agent — Iteration 14: Main Menu & Options

## Branch: `infra-agent`

You are implementing the menu state machine, settings persistence, input rebinding, and graceful exit handling.

---

## Your Tasks

| ID | Task | Priority | Description |
|----|------|----------|-------------|
| I-53 | Menu state machine | P0 | MainMenu -> Playing -> Paused transitions |
| I-54 | Settings persistence | P0 | Save/load settings.toml |
| I-55 | Input rebinding system | P0 | Configurable key bindings |
| I-56 | Graceful exit handling | P1 | Save on exit, cleanup resources |

---

## Detailed Requirements

### I-53: Menu State Machine
**File:** `crates/genesis-engine/src/menu_state.rs`

Manage all game state transitions:
- States: Initializing, MainMenu, NewGameWizard, Loading, Playing, Paused, Options, Exiting
- Valid transitions between states
- Query methods: is_playing(), is_paused(), should_update_world()

### I-54: Settings Persistence
**File:** `crates/genesis-engine/src/settings_persistence.rs`

Load and save settings to TOML:
- Settings path: ~/.config/genesis/settings.toml
- Load on startup, save on change
- Reset to defaults option

### I-55: Input Rebinding System
**File:** `crates/genesis-engine/src/input_rebind.rs`

Allow users to remap controls:
- GameAction enum for all actions
- KeyBinding with primary/secondary keys
- Listen for key press during rebind
- Conflict detection
- Reset to defaults

### I-56: Graceful Exit Handling
**File:** `crates/genesis-engine/src/exit_handler.rs`

Handle game exit properly:
- Exit confirmation if unsaved changes
- Cleanup tasks: save game, save settings, stop audio
- Window close event handling

---

## Definition of Done

- [ ] State machine handles all valid transitions
- [ ] Settings save/load to settings.toml
- [ ] Input rebinding works with conflict detection
- [ ] Exit saves game and cleans up
- [ ] All tests pass
- [ ] No clippy warnings
