# Tools Agent — Current Prompt

> Updated: 2026-02-03 — Iteration 2 by Orchestrator

## Status

✅ **Iteration 1 Complete!** T-1 through T-7 merged to main.

Branch synced with main. Ready for Iteration 2.

## Next Priority Tasks

| ID | Task | Priority |
|----|------|----------|
| T-8 | Integration test harness | P0 |
| T-9 | Automated screenshot tests | P1 |
| T-10 | Memory profiler integration | P1 |
| T-11 | Hot reload support | P2 |

### T-8: Integration Test Harness

Create a harness for end-to-end testing:
- Headless mode (no window, software renderer)
- Simulate N frames with scripted inputs
- Assert on world state after simulation
- Compare against golden files

```rust
pub struct TestHarness {
    world: World,
    kernel: Kernel,
    gameplay: Gameplay,
}

impl TestHarness {
    pub fn new_headless() -> Self;
    pub fn load_scenario(&mut self, path: &Path);
    pub fn simulate(&mut self, frames: u32, inputs: &[Input]);
    pub fn assert_cell(&self, pos: (u32, u32), expected: Cell);
    pub fn assert_entity_exists(&self, id: EntityId);
    pub fn snapshot(&self) -> WorldSnapshot;
}
```

### T-9: Automated Screenshot Tests

Visual regression testing:
- Render frame to image buffer
- Compare against golden screenshot
- Report pixel differences
- Store golden images in `tests/golden/`

```rust
pub fn screenshot_test(name: &str, harness: &TestHarness) -> TestResult {
    let actual = harness.render_to_image();
    let golden = load_golden(name)?;
    compare_images(&actual, &golden, threshold: 0.01)
}
```

### T-10: Memory Profiler Integration

Track memory usage:
- Allocator wrapper that counts allocations
- Per-system memory tracking (kernel, gameplay, world)
- Memory usage in perf HUD
- Detect memory leaks in tests

### T-11: Hot Reload Support (stretch)

Reload assets without restart:
- Watch material definitions
- Watch shader files
- Reload on file change
- Useful for rapid iteration

## Rules

1. Work ONLY in `crates/genesis-tools`
2. Run validation after EVERY change
3. Commit only when validation passes

## Validation Command

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```
