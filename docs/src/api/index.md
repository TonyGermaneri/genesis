# API Reference

This section provides links to the auto-generated Rust documentation (rustdoc) for each crate.

## Viewing Documentation

### Online
Visit the hosted documentation at: **[docs.rs/genesis](https://docs.rs/genesis)** (when published)

### Local
Generate documentation locally:

```bash
# Generate docs for all crates
cargo doc --workspace --no-deps

# Generate and open in browser
cargo doc --workspace --no-deps --open
```

Documentation will be at `target/doc/genesis_engine/index.html`.

## Crate Overview

| Crate | Description | Docs |
|-------|-------------|------|
| [genesis-common](genesis-common.md) | Shared types, IDs, coordinates | [📖](../../../target/doc/genesis_common/index.html) |
| [genesis-kernel](genesis-kernel.md) | GPU compute, cell simulation | [📖](../../../target/doc/genesis_kernel/index.html) |
| [genesis-world](genesis-world.md) | Chunks, terrain generation | [📖](../../../target/doc/genesis_world/index.html) |
| [genesis-gameplay](genesis-gameplay.md) | Entities, inventory, crafting | [📖](../../../target/doc/genesis_gameplay/index.html) |
| [genesis-tools](genesis-tools.md) | Replay, inspector, profiling | [📖](../../../target/doc/genesis_tools/index.html) |
| genesis-engine | Main application | [📖](../../../target/doc/genesis/index.html) |

## Key Types

### IDs and Coordinates

```rust
// genesis-common
pub struct EntityId(u64);
pub struct ChunkCoord { x: i32, y: i32 }
pub struct CellCoord { x: i32, y: i32 }
```

### Cell System

```rust
// genesis-kernel
pub struct Cell {
    pub material: u8,
    pub state: u8,
    pub data: u16,
}
```

### Entity System

```rust
// genesis-gameplay
pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub position: Vec2,
}
```

### Chunk System

```rust
// genesis-world
pub struct Chunk {
    pub coord: ChunkCoord,
    pub cells: [[Cell; CHUNK_SIZE]; CHUNK_SIZE],
}
```

## Feature Flags

Some crates have optional features:

### genesis-tools
- `replay` - Enable replay recording/playback (default)
- `inspector` - Enable cell inspector (default)
- `perf` - Enable performance HUD (default)

### genesis-engine
- `dev-tools` - Include development tools (default in debug)

## Examples

See the `examples/` directory for code examples:

```bash
# Run a specific example
cargo run --example basic_world
```
