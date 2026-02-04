# Tools Agent — Iteration 14: Main Menu & Options

## Branch: `tools-agent`

You are implementing the UI for main menu, pause menu, options menu, and new game wizard using egui.

---

## Your Tasks

| ID | Task | Priority | Description |
|----|------|----------|-------------|
| T-56 | Main menu UI | P0 | New Game, Continue, Load, Options, Exit |
| T-57 | Pause/ESC menu UI | P0 | Resume, Save, Load, Options, Quit to Menu |
| T-58 | Options menu UI | P0 | Graphics, Audio, Controls, Gameplay tabs |
| T-59 | New game wizard UI | P1 | World name, seed, difficulty selection |

---

## Detailed Requirements

### T-56: Main Menu UI
**File:** `crates/genesis-tools/src/ui/main_menu.rs`

Create the main menu interface:
- New Game button
- Continue button (if save exists)
- Load Game button
- Options button
- Exit button
- Keyboard navigation (up/down, enter)
- Version display

### T-57: Pause/ESC Menu UI
**File:** `crates/genesis-tools/src/ui/pause_menu.rs`

In-game pause menu:
- Resume button
- Save Game button
- Load Game button
- Options button
- Quit to Menu button
- Quit to Desktop button
- ESC key toggles
- Semi-transparent overlay

### T-58: Options Menu UI
**File:** `crates/genesis-tools/src/ui/options_menu.rs`

Tabbed options interface:
- Graphics tab: resolution, fullscreen, vsync, render distance, shadows
- Audio tab: master, music, sfx, ambient volumes
- Controls tab: mouse sensitivity, invert Y, key rebinding
- Gameplay tab: difficulty, auto-save, tutorials
- Apply/Cancel/Reset buttons

### T-59: New Game Wizard UI
**File:** `crates/genesis-tools/src/ui/new_game_wizard.rs`

Multi-step new game creation:
- Step 1: World name input
- Step 2: Seed, world size, difficulty
- Step 3: Confirmation summary
- Back/Next/Cancel navigation

---

## Definition of Done

- [ ] Main menu displays with all buttons
- [ ] Pause menu toggles with ESC
- [ ] Options menu has all tabs functional
- [ ] New game wizard validates input
- [ ] Keyboard navigation works
- [ ] All tests pass
- [ ] No clippy warnings
