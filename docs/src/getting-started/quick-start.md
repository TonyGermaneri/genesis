# Quick Start

Get Genesis running in under 5 minutes!

## Prerequisites

- Rust 1.75 or later
- A GPU with Vulkan 1.2, Metal, or DirectX 12 support

## Option 1: Using Nix (Recommended)

If you have Nix installed with flakes enabled:

```bash
# Clone the repository
git clone https://github.com/tonygermaneri/genesis.git
cd genesis

# Enter development shell
nix develop

# Run the game
just run
```

## Option 2: Using Cargo

```bash
# Clone the repository
git clone https://github.com/tonygermaneri/genesis.git
cd genesis

# Build and run
cargo run --release --package genesis-engine
```

## Option 3: Download Release

1. Go to [Releases](https://github.com/tonygermaneri/genesis/releases)
2. Download the archive for your platform
3. Extract and run the `genesis` binary

## First Launch

On first launch, Genesis will:

1. Create a configuration directory
2. Generate an initial world
3. Display the main menu

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look around |
| E | Interact |
| I | Inventory |
| C | Crafting menu |
| Esc | Pause menu |

## Next Steps

- [Installation Guide](installation.md) - Detailed setup instructions
- [First Steps](first-steps.md) - Learn the basics
- [Modding Introduction](../modding/introduction.md) - Add custom content
