# Iteration 11: Kernel Agent - Crafting Infrastructure

## Objective
Implement low-level crafting grid computation, item stack management, and workbench spatial detection.

## Tasks

### 1. Crafting Grid Compute (crafting_grid.rs)
- CraftingGrid: 3x3 or configurable grid of ItemSlot
- RecipePattern: GPU-friendly pattern representation
- Pattern matching via compute shader or CPU fallback
- Shapeless vs shaped recipe distinction

### 2. Item Stack Management (item_stack.rs)
- ItemStack: item_id, count, metadata, durability
- Stack combining with max stack size limits
- Stack splitting (shift-click, drag splitting)
- Serialization for save/load

### 3. Workbench Zones (workbench.rs)
- WorkbenchType enum (Basic, Forge, Anvil, Alchemy)
- Interaction radius detection
- Station capabilities (recipe categories)

### 4. Crafting Animation Data (crafting_anim.rs)
- CraftingProgress: 0.0 to 1.0 completion
- ParticleEmitter positions for effects
- Sound trigger points

### 5. Update lib.rs
Export: crafting_grid, item_stack, workbench, crafting_anim
