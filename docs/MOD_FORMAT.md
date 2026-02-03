# Mod Package Format Specification

> Version: 1.0.0  
> Status: Draft

## Overview

Genesis supports modular content through `.genesismod` packages. This document defines the mod package format, loading behavior, and best practices.

## Package Structure

A `.genesismod` file is a ZIP archive with the following structure:

```
my-mod.genesismod (ZIP)
├── mod.ron              # Required: Mod manifest
├── README.md            # Optional: Mod documentation
├── LICENSE              # Optional: License file
├── textures/            # Optional: Image assets
│   ├── items/
│   └── entities/
├── sounds/              # Optional: Audio assets
├── models/              # Optional: 3D models
├── recipes/             # Optional: Crafting recipes (RON)
│   └── *.ron
├── items/               # Optional: Item definitions (RON)
│   └── *.ron
├── entities/            # Optional: Entity definitions (RON)
│   └── *.ron
├── buildings/           # Optional: Building definitions (RON)
│   └── *.ron
└── scripts/             # Optional: Lua scripts
    └── *.lua
```

## Manifest Format (mod.ron)

See `spec/schemas/mod_manifest.ron` for the complete schema.

### Minimal Example

```ron
Mod(
    id: "my-mod",
    name: "My Mod",
    version: "1.0.0",
    genesis_version: "0.1.0",
)
```

### Full Example

```ron
Mod(
    id: "expanded-crafting",
    name: "Expanded Crafting",
    version: "2.1.0",
    genesis_version: "0.1.0",
    author: "ModAuthor",
    description: "Adds 50+ new crafting recipes",
    homepage: "https://example.com/expanded-crafting",
    license: "MIT",
    dependencies: [
        ("base-materials", ">=1.0.0"),
    ],
    conflicts: ["simple-crafting"],
    priority: 10,
    assets: ["textures/"],
    data: ["recipes/", "items/"],
    scripts: ["scripts/init.lua"],
    tags: ["crafting", "content"],
)
```

## Mod Loading Order

1. **Discovery**: Scan `mods/` directory for `.genesismod` files
2. **Validation**: Parse and validate each `mod.ron` manifest
3. **Dependency Resolution**: Build dependency graph, detect conflicts
4. **Sort by Priority**: Lower priority loads first (default: 0)
5. **Load Assets**: Load textures, sounds, models
6. **Load Data**: Parse and merge RON data files
7. **Execute Scripts**: Run Lua initialization scripts

### Priority Rules

- Mods with same priority load in alphabetical order by ID
- Higher priority mods can override lower priority content
- Core game content has priority -1000 (loads first)

## Asset Loading

### Textures

- Supported formats: PNG, JPEG, WebP
- Path: `textures/<category>/<name>.<ext>`
- Access in game: `mod:my-mod/textures/items/sword.png`

### Sounds

- Supported formats: OGG, WAV, MP3
- Path: `sounds/<category>/<name>.<ext>`

### Models

- Supported formats: GLTF, GLB
- Path: `models/<name>.gltf`

## Data Files

All data files use RON (Rusty Object Notation) format.

### Recipes

```ron
// recipes/iron_sword.ron
Recipe(
    id: "my-mod:iron_sword",
    name: "Iron Sword",
    category: "weapons",
    ingredients: [
        ("base:iron_ingot", 3),
        ("base:wood_plank", 1),
    ],
    output: ("my-mod:iron_sword", 1),
    crafting_time: 5.0,
    station: "forge",
)
```

### Items

```ron
// items/iron_sword.ron
Item(
    id: "my-mod:iron_sword",
    name: "Iron Sword",
    description: "A sturdy iron blade",
    stack_size: 1,
    weight: 2.5,
    category: "weapon",
    texture: "textures/items/iron_sword.png",
    properties: {
        "damage": 15,
        "durability": 100,
    },
)
```

## Scripting (Lua)

Mods can include Lua scripts for custom behavior:

```lua
-- scripts/init.lua
genesis.log("Expanded Crafting mod loaded!")

genesis.on_event("player_craft", function(event)
    if event.recipe:starts_with("my-mod:") then
        genesis.log("Player crafted: " .. event.recipe)
    end
end)
```

### Available APIs

- `genesis.log(message)` - Log to console
- `genesis.on_event(name, callback)` - Subscribe to events
- `genesis.spawn_entity(type, x, y)` - Spawn entity
- `genesis.get_player()` - Get current player entity

## Conflict Resolution

When multiple mods define the same ID:

1. Higher priority mod wins
2. If same priority, alphabetically later mod wins
3. Conflicts can be declared explicitly in manifest

## Packaging Mods

```bash
# Create mod package
cd my-mod/
zip -r ../my-mod.genesismod *

# Or use the genesis-tools CLI (future)
genesis-tools pack-mod my-mod/
```

## Validation

Validate a mod package:

```bash
# Future CLI tool
genesis-tools validate-mod my-mod.genesismod
```

## Best Practices

1. **Use namespaced IDs**: Prefix all IDs with your mod ID (`my-mod:item_name`)
2. **Declare dependencies**: List all mods yours depends on
3. **Set appropriate priority**: Use 0 for content, higher for overrides
4. **Include documentation**: Add README.md explaining your mod
5. **Test compatibility**: Test with common mods before release
6. **Semantic versioning**: Follow semver for version numbers
