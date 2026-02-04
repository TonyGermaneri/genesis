# Project Genesis

> GPU-accelerated action RPG with pixel-cell simulation

[![Build Status](https://github.com/tonygermaneri/genesis/workflows/CI/badge.svg)](https://github.com/tonygermaneri/genesis/actions)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 🎮 Overview

Project Genesis is a GPU-accelerated action RPG featuring real-time pixel-cell simulation. Every pixel in the world is a simulated cell with physical properties, enabling emergent gameplay through a sophisticated compute shader pipeline.

### Key Features

- **🖥️ GPU Compute Pixel-Cell Simulation** — Every pixel is a simulated cell with properties like temperature, moisture, and material type
- **🌍 Infinite Procedural World** — Chunked, streamed, persistent world with multiple biomes
- **⚔️ Combat System** — Melee and ranged combat with damage calculations, equipment, and abilities
- **🔧 Crafting System** — Recipe-based crafting at workbenches with material requirements
- **👥 NPC System** — AI-driven NPCs with behaviors, dialogue, and spawning
- **🎵 Spatial Audio** — 3D audio with ambient sounds, music, and sound effects
- **💾 Save/Load System** — Full game state persistence with auto-save and save management
- **🎨 Modding Support** — Extensible architecture with TOML-based asset definitions
- **⚙️ Full Options Menu** — Graphics, audio, controls, and gameplay settings

### Statistics

- **~156,000 lines** of Rust code
- **187 source files** across 6 crates
- **14 development iterations** over 4 parallel agent branches
- **59 completed task groups** (K-1 to K-59, G-1 to G-60, T-1 to T-59, I-1 to I-56)

---

## 🚀 Quick Start

### Prerequisites

- **Rust 1.75+** — Install via [rustup](https://rustup.rs/)
- **GPU with Vulkan/Metal/DX12 support** — Required for wgpu compute shaders
- **[just](https://github.com/casey/just)** command runner (recommended)

### Build & Run

```bash
# Clone the repository
git clone https://github.com/tonygermaneri/genesis
cd genesis

# Build (debug)
cargo build

# Build (release - recommended for playing)
cargo build --release

# Run the game
./target/release/genesis

# Or use just
just run
```

### Using Nix (Recommended)

```bash
# Enter development shell with all dependencies
nix develop

# Build and run
just run
```

---

## 🎮 Controls

### Movement
| Key | Action |
|-----|--------|
| W/↑ | Move up |
| S/↓ | Move down |
| A/← | Move left |
| D/→ | Move right |
| Shift | Sprint |

### Combat
| Key | Action |
|-----|--------|
| Left Mouse | Primary attack |
| Right Mouse | Secondary attack / Block |
| 1-9 | Select hotbar slot |

### UI
| Key | Action |
|-----|--------|
| ESC | Pause menu |
| I | Inventory |
| C | Crafting |
| M | Map |
| E | Interact |
| F3 | Debug overlay |

### Camera
| Key | Action |
|-----|--------|
| Mouse Wheel | Zoom in/out |
| Middle Mouse | Pan camera |

---

## 📁 Project Structure

```
genesis/
├── crates/                     # Rust workspace crates
│   ├── genesis-engine/         # Main game binary & integration
│   │   └── src/
│   │       ├── app.rs          # Main application loop
│   │       ├── menu_state.rs   # Game state machine
│   │       ├── save_manager.rs # Save/load system
│   │       ├── input_rebind.rs # Input configuration
│   │       └── ...
│   ├── genesis-kernel/         # GPU compute & rendering
│   │   └── src/
│   │       ├── chunk.rs        # Chunk management
│   │       ├── pipeline.rs     # Compute pipelines
│   │       ├── audio_backend.rs# Audio processing
│   │       └── ...
│   ├── genesis-gameplay/       # Game logic & systems
│   │   └── src/
│   │       ├── player.rs       # Player entity
│   │       ├── combat.rs       # Combat mechanics
│   │       ├── crafting.rs     # Crafting system
│   │       ├── npc.rs          # NPC behaviors
│   │       └── ...
│   ├── genesis-tools/          # UI & development tools
│   │   └── src/
│   │       ├── ui/             # egui-based UI components
│   │       │   ├── main_menu.rs
│   │       │   ├── inventory.rs
│   │       │   ├── crafting_grid.rs
│   │       │   └── ...
│   │       └── ...
│   ├── genesis-world/          # World generation
│   └── genesis-common/         # Shared types
├── assets/                     # Game assets (moddable)
│   ├── recipes/                # Crafting recipes
│   ├── weapons/                # Weapon definitions
│   ├── sounds/                 # Audio files
│   └── locales/                # Translations
├── spec/                       # Contracts & schemas
│   ├── CONTRACTS.md            # Inter-system interfaces
│   └── schemas/                # JSON schemas
├── docs/                       # Documentation
│   ├── ARCHITECTURE.md         # System design
│   ├── MOD_FORMAT.md           # Modding guide
│   └── WORKTREE_SETUP.md       # Development setup
└── scripts/                    # Build & utility scripts
```

---

## ⚙️ Configuration

### Game Configuration

The game stores configuration at:
- **macOS**: `~/Library/Application Support/genesis/`
- **Linux**: `~/.config/genesis/`
- **Windows**: `%APPDATA%\genesis\`

#### genesis.toml (Main Config)

```toml
[window]
width = 1280
height = 720
vsync = true
fullscreen = false

[render]
chunk_render_distance = 4
shadow_quality = "high"
particle_density = "medium"

[audio]
master_volume = 1.0
music_volume = 0.8
sfx_volume = 1.0
ambient_volume = 0.7
```

#### settings.toml (User Settings)

```toml
[graphics]
resolution = [1920, 1080]
fullscreen = false
vsync = true
render_distance = 12

[audio]
master_volume = 1.0
music_volume = 0.8
sfx_volume = 1.0
mute_when_unfocused = true

[controls]
mouse_sensitivity = 1.0
invert_y = false

[controls.bindings]
move_forward = "W"
move_back = "S"
move_left = "A"
move_right = "D"
jump = "Space"
interact = "E"
inventory = "I"
pause = "Escape"

[gameplay]
difficulty = "Normal"
auto_save_interval = 5
show_tutorials = true
camera_shake = true
```

---

## 💾 Save System

### Save File Locations

- **macOS**: `~/Library/Application Support/genesis/saves/`
- **Linux**: `~/.local/share/genesis/saves/`
- **Windows**: `%APPDATA%\genesis\saves\`

### Save File Structure

Each save slot contains:

```
saves/
└── <slot_name>/
    ├── world.dat          # World state (compressed)
    ├── player.dat         # Player data
    ├── npcs.dat           # NPC states
    ├── progress.dat       # Progress tracking
    ├── thumbnail.png      # Save preview image
    └── metadata.json      # Save metadata
```

### Auto-Save

- Configurable interval (default: 5 minutes)
- Saves to a rotating auto-save slot
- Can be disabled in options

---

## 🎨 Modding Guide

### Asset Types

Genesis supports modding through TOML-based asset files:

#### Recipes (`assets/recipes/`)

```toml
# basic.toml - Crafting recipes

[[recipes]]
id = "wooden_pickaxe"
name = "Wooden Pickaxe"
category = "tools"
workbench = "basic"
craft_time = 2.0

[recipes.ingredients]
wood = 3
stone = 2

[recipes.result]
item = "wooden_pickaxe"
count = 1
```

#### Weapons (`assets/weapons/`)

```toml
# melee.toml - Melee weapon definitions

[[weapons]]
id = "iron_sword"
name = "Iron Sword"
weapon_type = "melee"
damage = 15
attack_speed = 1.2
range = 1.5
durability = 100

[weapons.modifiers]
critical_chance = 0.1
knockback = 0.5
```

#### Sound Configuration (`assets/sounds/`)

```
sounds/
├── music/
│   ├── exploration_plains.ogg
│   └── combat_intense.ogg
├── sfx/
│   ├── footstep_grass.ogg
│   ├── sword_swing.ogg
│   └── pickup_item.ogg
└── ambient/
    ├── forest_day.ogg
    └── rain_light.ogg
```

#### Localization (`assets/locales/`)

```toml
# en.toml - English translations

[menu]
new_game = "New Game"
continue = "Continue"
options = "Options"
exit = "Exit"

[inventory]
title = "Inventory"
weight = "Weight: {current}/{max}"
```

### Adding Mods

1. Create a `mods/` folder in the game directory
2. Add your mod with the structure:
   ```
   mods/
   └── my_mod/
       ├── mod.toml        # Mod metadata
       ├── recipes/        # Additional recipes
       ├── weapons/        # Additional weapons
       └── sounds/         # Additional sounds
   ```

3. Create `mod.toml`:
   ```toml
   [mod]
   id = "my_mod"
   name = "My Awesome Mod"
   version = "1.0.0"
   author = "Your Name"
   description = "Adds cool stuff"

   [dependencies]
   genesis = ">=0.1.0"
   ```

---

## 🔧 Building from Source

### Debug Build

```bash
cargo build
./target/debug/genesis
```

### Release Build (Optimized)

```bash
cargo build --release
./target/release/genesis
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p genesis-gameplay

# With output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Full validation (format + clippy + tests)
just validate
```

### Documentation

```bash
# Generate docs
cargo doc --workspace --no-deps --open
```

---

## 🏗️ Architecture

### Crate Responsibilities

| Crate | Purpose |
|-------|---------|
| **genesis-engine** | Main binary, app lifecycle, system integration |
| **genesis-kernel** | GPU compute shaders, chunk rendering, audio backend |
| **genesis-gameplay** | Player, NPCs, combat, crafting, game rules |
| **genesis-tools** | egui UI, debug panels, development tools |
| **genesis-world** | World generation, biomes, terrain |
| **genesis-common** | Shared types, utilities |

### Data Flow

```
Input → Engine → Gameplay Update → Kernel Compute → Render
                     ↓
              Save/Load System
                     ↓
                Tools UI
```

### Multi-Agent Development

The project was developed using 4 parallel agent branches:

1. **kernel-agent** — GPU compute, rendering, audio
2. **gameplay-agent** — Game logic, entities, systems
3. **tools-agent** — UI components, debug panels
4. **infra-agent** — Integration, build system, I/O

See [WORKTREE_SETUP.md](docs/WORKTREE_SETUP.md) for details.

---

## � Coordinate System & UI Integration

### High-DPI / Retina Display Support

The game uses three coordinate systems that must be kept in sync for UI interaction to work correctly:

1. **Physical Pixels** — Actual screen pixels (what wgpu renders to)
2. **Logical Pixels** — Window coordinates (used by winit for events)
3. **Egui Points** — UI coordinates (used by egui for hit testing)

On high-DPI displays (like macOS Retina), the scale factor relates these:
- `physical_pixels = logical_pixels × scale_factor`
- `egui_points = physical_pixels ÷ pixels_per_point`

### Critical Implementation Details

For mouse clicks to be detected correctly by egui UI elements:

1. **On Window Creation**: Set egui's `pixels_per_point` to match the window's scale factor
   ```rust
   renderer.set_scale_factor(window.scale_factor() as f32);
   ```

2. **On Window Resize**: Update egui's scale factor along with surface configuration
   ```rust
   renderer.resize(new_size);
   renderer.set_scale_factor(window.scale_factor() as f32);
   ```

3. **On Scale Factor Change**: Handle `WindowEvent::ScaleFactorChanged` explicitly
   ```rust
   WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
       renderer.set_scale_factor(scale_factor as f32);
   }
   ```

4. **During Rendering**: Use consistent scale factor in `ScreenDescriptor`
   ```rust
   let screen_descriptor = egui_wgpu::ScreenDescriptor {
       size_in_pixels: [self.size.width, self.size.height],
       pixels_per_point: window.scale_factor() as f32,
   };
   ```

### Why This Matters

Without proper scale factor synchronization:
- Mouse hover detection works (uses relative coordinates)
- Mouse click detection fails (egui's hit test uses absolute coordinates)
- UI appears correct but is not interactive
- Issue manifests differently at different window sizes

This is implemented in:
- `crates/genesis-engine/src/app.rs` — Window event handling
- `crates/genesis-engine/src/renderer.rs` — `set_scale_factor()` method
- `crates/genesis-tools/src/egui_integration.rs` — `set_pixels_per_point()` method

---

## �🐛 Troubleshooting

### Common Issues

#### "Failed to create GPU adapter"
- Ensure you have a GPU with Vulkan/Metal/DX12 support
- Update your graphics drivers
- Try setting `WGPU_BACKEND=vulkan` (or `metal` on macOS)

#### "Audio file not found" warnings
- Audio files are optional; the game runs without them
- Add `.ogg` files to `assets/sounds/` to enable audio

#### Game runs slowly
- Try release build: `cargo build --release`
- Reduce render distance in options
- Lower particle density

#### Save files not found
- Check the save directory for your platform (see Save System section)
- Ensure the game has write permissions

---

## 📊 Development Progress

### Completed Iterations

| Iteration | Focus | Tasks |
|-----------|-------|-------|
| 1-3 | Bootstrap & Cell Simulation | K-1 to K-27 |
| 4 | Multi-chunk Streaming | K-28 to K-31 |
| 5 | Biome System | K-32 to K-35, G-29 to G-36 |
| 6 | NPC System | K-36 to K-39, G-37 to G-40 |
| 7 | Audio System | K-40 to K-43, G-41 to G-44 |
| 8-9 | UI & Tools | T-1 to T-47 |
| 10 | Sound Integration | Audio events & ambient |
| 11 | Crafting System | K-44 to K-47, G-45 to G-48 |
| 12 | Combat System | K-48 to K-51, G-49 to G-52 |
| 13 | Save/Load System | K-52 to K-55, G-53 to G-56 |
| 14 | Main Menu & Options | K-56 to K-59, G-57 to G-60 |

---

## 📝 License

This project is dual-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

You may choose either license.

---

## 🙏 Acknowledgments

Built with:
- [wgpu](https://wgpu.rs/) — GPU compute & rendering
- [winit](https://github.com/rust-windowing/winit) — Window management
- [egui](https://github.com/emilk/egui) — Immediate mode GUI
- [rodio](https://github.com/RustAudio/rodio) — Audio playback
- [serde](https://serde.rs/) — Serialization
- [toml](https://github.com/toml-rs/toml) — Configuration format

---

## 📫 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run `just validate` before committing
4. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
