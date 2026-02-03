# Project Genesis

> GPU-accelerated action RPG with pixel-cell simulation

## Overview

Project Genesis is an action RPG featuring:
- **GPU compute pixel-cell simulation** — every pixel is a simulated cell
- **Dual gameplay modes** — top-down overworld (Jackal) + platform interiors (River City Ransom)
- **Infinite procedural world** — chunked, streamed, persistent
- **Hardcore RPG systems** — crafting, economy, factions, needs
- **Modular vehicles** — enter/exit at will, sci-fi modular design
- **Built-in modding** — extensible architecture

## Quick Start

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- GPU with Vulkan/Metal/DX12 support
- [just](https://github.com/casey/just) command runner (recommended)

### Build & Run

```bash
# Clone the repository
git clone https://github.com/tonygermaneri/genesis
cd genesis

# Build
just build

# Run tests
just test

# Run the engine
just run
```

### Using Nix (Recommended)

```bash
# Enter development shell
nix develop

# All dependencies are automatically available
just run
```

## Project Structure

```
genesis/
├── crates/
│   ├── genesis-engine    # Main binary
│   ├── genesis-kernel    # GPU compute
│   ├── genesis-gameplay  # RPG systems
│   ├── genesis-world     # World management
│   ├── genesis-common    # Shared types
│   └── genesis-tools     # Dev tools
├── spec/                 # Contracts & schemas
├── docs/                 # Documentation
└── tools/                # Build utilities
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design and data flow
- [Contracts](spec/CONTRACTS.md) — Inter-system interfaces
- [Task Board](TASKS.md) — Current development status
- [Worktree Setup](docs/WORKTREE_SETUP.md) — Multi-agent workflow

## Development

### Validation Loop

All code must pass before commit:

```bash
just validate
```

This runs:
1. `cargo fmt --check` — formatting
2. `cargo clippy -- -D warnings` — linting
3. `cargo test --workspace` — tests

### Multi-Agent Development

This project uses git worktrees for parallel development:

```bash
# Create agent worktrees
git worktree add ../genesis-kernel kernel-agent
git worktree add ../genesis-gameplay gameplay-agent

# Open in separate VS Code windows
code ../genesis-kernel
code ../genesis-gameplay
```

See [WORKTREE_SETUP.md](docs/WORKTREE_SETUP.md) for details.

## License

MIT OR Apache-2.0 (at your option)
