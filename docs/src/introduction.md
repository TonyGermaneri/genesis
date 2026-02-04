# Project Genesis

**Project Genesis** is a systems-rich simulation game featuring GPU-accelerated cellular automata, complex entity interactions, and a deep modding system.

## What is Genesis?

Genesis is a unique simulation experience that combines:

- **GPU Compute Kernel**: Massively parallel cell simulation running on the GPU
- **Rich World System**: Procedurally generated terrain with streaming chunks
- **Deep Gameplay**: Crafting, economy, factions, and survival mechanics
- **Powerful Modding**: Full mod support with Lua scripting

## Quick Links

- [Quick Start Guide](getting-started/quick-start.md) - Get up and running in 5 minutes
- [Architecture Overview](architecture/overview.md) - Understand how Genesis works
- [Modding Guide](modding/introduction.md) - Create your own content
- [API Reference](api/index.md) - Dive into the code

## Features

### 🖥️ GPU-Accelerated Simulation

Genesis uses compute shaders to simulate millions of cells in parallel. Water flows, fire spreads, and materials interact—all at 60+ FPS.

### 🌍 Procedural World

Infinite procedurally generated terrain with multiple biomes, cave systems, and dynamic weather.

### ⚒️ Deep Crafting System

Hundreds of recipes with multi-stage crafting, tool requirements, and quality modifiers.

### 🏪 Living Economy

NPC traders, supply and demand, and player-driven markets.

### 🤝 Faction System

Multiple factions with unique cultures, reputations, and questlines.

### 🔧 Extensible Modding

Full mod support with:
- Custom items, recipes, and entities
- Lua scripting for behaviors
- Asset replacement and additions
- Compatibility with other mods

## System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| OS | Windows 10 / macOS 12 / Linux (glibc 2.31+) | Latest |
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 16 GB |
| GPU | Vulkan 1.2 / Metal / DX12 | Dedicated GPU with 4GB+ VRAM |
| Storage | 2 GB | 5 GB (for mods) |

## License

Project Genesis is dual-licensed under MIT and Apache 2.0. See LICENSE files for details.

## Contributing

We welcome contributions! See the [Contributing Guide](development/contributing.md) for details.
