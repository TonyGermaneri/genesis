# Tools Agent — Current Prompt

> Updated: 2026-02-03 15:08 by Orchestrator

## Status

✅ **Excellent work!** You completed T-1 through T-4:
- T-1: Replay recording ✅
- T-2: Replay playback ✅
- T-3: Determinism verification ✅
- T-4: Chunk viewer (egui) ✅

**Note:** You have an uncommitted `Cargo.lock` change. Commit it with your next change or discard if unneeded.

## Next Priority Tasks

| ID | Task | Priority |
|----|------|----------|
| T-5 | Cell inspector probe | P1 |
| T-6 | Performance HUD | P2 |
| T-7 | Event log viewer | P2 |

### T-5: Cell Inspector Probe

Create an interactive cell inspector:
- Click on any cell to select it
- Display all cell properties (material, state, temperature, etc.)
- Show material properties from LUT
- Show neighboring cell info
- Real-time update as simulation runs

```rust
pub struct CellInspector {
    selected_pos: Option<(u32, u32)>,
}

impl CellInspector {
    pub fn select(&mut self, x: u32, y: u32);
    pub fn render_ui(&self, ui: &mut egui::Ui, chunk: &Chunk);
}
```

### T-6: Performance HUD

Create an in-game performance overlay:
- FPS counter (current, avg, min, max)
- Frame time graph (last 100 frames)
- GPU dispatch time
- Memory usage (chunks loaded, entities active)
- Toggle with F3 or similar

### T-7: Event Log Viewer

Create a scrollable event log:
- Display kernel events (cell changes, physics events)
- Display gameplay events (entity spawns, item pickups)
- Filter by event type
- Search functionality
- Timestamp display

## Rules

1. Work ONLY in `crates/genesis-tools`
2. Run validation after EVERY change
3. Commit only when validation passes

## Validation Command

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```
