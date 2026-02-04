# Iteration 11: Gameplay Agent - Crafting Logic

## Objective
Implement recipe definitions, crafting validation, workbench types, and progression.

## Tasks

### 1. Recipe Data (recipes.rs)
- Recipe struct: id, ingredients, pattern, output, station_required
- RecipeCategory enum (Tools, Weapons, Armor, Potions, Building)
- Shaped vs shapeless recipes
- Recipe tags for filtering

### 2. Crafting Logic (crafting.rs)
- validate_recipe: check if grid matches any recipe
- craft_item: consume ingredients, produce output
- get_craftable_recipes: list recipes player can make
- Handle partial ingredient matching

### 3. Workbench Types (workbench_types.rs)
- WorkbenchDefinition: type, recipes_unlocked, tier
- Basic crafting (no station)
- Forge: metal items, smelting
- Anvil: weapon/armor upgrades
- Alchemy table: potions

### 4. Crafting Progression (crafting_progression.rs)
- LearnedRecipes tracking per player
- Recipe discovery via experimentation
- Skill-gated recipes
- Recipe unlocks from NPCs/quests

### 5. Update lib.rs
Export: recipes, crafting, workbench_types, crafting_progression
