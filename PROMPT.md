# Iteration 13: Tools Agent - Save/Load UI

## Objective
Create save/load menu, slot previews, auto-save indicator, and save management.

## Tasks

### 1. Save/Load Menu UI (ui/save_menu.rs)
- Main menu save/load buttons
- Save slot grid (5-10 slots)
- New game button
- Continue last save button

### 2. Save Slot Previews (ui/save_preview.rs)
- Screenshot thumbnail per slot
- Player name and level
- Playtime and last played date
- World name and seed

### 3. Auto-save Indicator (ui/autosave_indicator.rs)
- Spinning icon during save
- Configurable position (corner)
- Fade in/out animation
- Error indicator on fail

### 4. Save Management UI (ui/save_management.rs)
- Delete save confirmation dialog
- Copy save to new slot
- Export save to file
- Import save from file

### 5. Update ui/mod.rs
Export: save_menu, save_preview, autosave_indicator, save_management
