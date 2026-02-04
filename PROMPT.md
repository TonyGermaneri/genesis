# Iteration 11: Infra Agent - Crafting Integration

## Objective
Load recipes from assets, wire crafting to game systems, persist learned recipes.

## Tasks

### 1. Recipe Asset Loading (recipe_loader.rs)
- Load recipes from assets/recipes/*.toml
- Validate recipe data on load
- Hot-reload recipes in debug mode
- Recipe registry with fast lookup

### 2. Crafting Event Integration (crafting_events.rs)
- Wire CraftItem event to inventory system
- Trigger crafting sounds on start/complete
- Update player stats on craft
- Achievement/quest triggers

### 3. Crafting Persistence (crafting_save.rs)
- Save learned recipes to player save
- Persist workbench contents on exit
- Load crafting state on game load
- Migration for recipe format changes

### 4. Crafting Profiling (crafting_profile.rs)
- Measure recipe search performance
- Track crafting frequency stats
- Memory usage for recipe database

### 5. Update Engine Integration
- Add crafting to game loop
- Wire workbench interaction to input
- Connect UI to crafting system
