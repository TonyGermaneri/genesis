# Iteration 11: Tools Agent - Crafting UI

## Objective
Create drag-drop crafting interface, recipe browser, and workbench panels.

## Tasks

### 1. Crafting Grid UI (ui/crafting_grid.rs)
- 3x3 drag-drop grid for ingredients
- Output slot with result preview
- Craft button with cooldown visual
- Clear grid button

### 2. Recipe Book UI (ui/recipe_book.rs)
- Categorized recipe browser
- Search/filter by name or ingredient
- Show required materials
- Highlight craftable vs locked recipes

### 3. Crafting Preview (ui/crafting_preview.rs)
- Show output item on valid recipe match
- Display item stats/description
- Ingredient availability indicators
- Crafting time estimate

### 4. Workbench UI (ui/workbench_ui.rs)
- Station-specific crafting panels
- Forge UI with fuel slot
- Alchemy UI with flask slots
- Progress bars for timed crafting

### 5. Update ui/mod.rs
Export: crafting_grid, recipe_book, crafting_preview, workbench_ui
