# Introduction to Modding

Genesis features a powerful modding system that lets you add new content, modify game behavior, and share your creations with others.

## What Can You Mod?

- **Items**: New tools, weapons, materials, consumables
- **Recipes**: Crafting recipes with custom ingredients
- **Entities**: NPCs, creatures, vehicles
- **Buildings**: Structures players can construct
- **Biomes**: New terrain types and generation rules
- **Scripts**: Custom game logic with Lua

## Mod Package Format

Mods are distributed as `.genesismod` files (ZIP archives):

```
my-mod.genesismod
├── mod.ron          # Manifest (required)
├── textures/        # Images
├── sounds/          # Audio
├── recipes/         # Crafting recipes
├── items/           # Item definitions
└── scripts/         # Lua scripts
```

See [Package Format](package-format.md) for the full specification.

## Quick Example

### 1. Create Mod Directory

```bash
mkdir my-first-mod
cd my-first-mod
```

### 2. Create Manifest

```ron
// mod.ron
Mod(
    id: "my-first-mod",
    name: "My First Mod",
    version: "1.0.0",
    genesis_version: "0.1.0",
    author: "Your Name",
    description: "A simple example mod",
)
```

### 3. Add Content

```ron
// items/super_pickaxe.ron
Item(
    id: "my-first-mod:super_pickaxe",
    name: "Super Pickaxe",
    description: "Mines 2x faster!",
    stack_size: 1,
    category: "tool",
    properties: {
        "mining_speed": 2.0,
        "durability": 500,
    },
)
```

### 4. Package and Install

```bash
# Create package
zip -r my-first-mod.genesismod *

# Install (copy to mods folder)
cp my-first-mod.genesismod ~/.genesis/mods/
```

### 5. Test

Launch Genesis—your mod loads automatically!

## Learning Path

1. [Package Format](package-format.md) - Understand mod structure
2. [Creating Your First Mod](first-mod.md) - Step-by-step tutorial
3. [Assets and Resources](assets.md) - Adding textures and sounds
4. [Data Files](data-files.md) - Items, recipes, entities in RON
5. [Lua Scripting](scripting.md) - Custom game logic

## Mod Load Order

Mods load in this order:
1. Core game content (priority: -1000)
2. Dependencies (resolved automatically)
3. Your mods (by priority, then alphabetically)

Higher priority mods can override lower priority content.

## Compatibility

- Use namespaced IDs: `my-mod:item_name`
- Declare dependencies explicitly
- Test with popular mods before release
- Use semantic versioning

## Getting Help

- [Genesis Discord](https://discord.gg/genesis) - #modding channel
- [Mod Repository](https://mods.genesis-game.dev) - Browse and share mods
- [API Reference](../api/index.md) - Technical documentation

## Tools

- **genesis-tools pack-mod** - Package your mod
- **genesis-tools validate-mod** - Check for errors
- **genesis-tools test-mod** - Run mod in sandbox

Ready to start? Head to [Creating Your First Mod](first-mod.md)!
