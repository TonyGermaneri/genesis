# Kernel Agent — Iteration 14: Main Menu & Options

## Branch: `kernel-agent`

You are implementing GPU-side rendering support for the main menu and options system.

---

## Your Tasks

| ID | Task | Priority | Description |
|----|------|----------|-------------|
| K-56 | Menu background rendering | P0 | Animated/static menu backdrop |
| K-57 | Transition effects | P1 | Fade in/out between screens |
| K-58 | Screenshot capture | P1 | For save slot thumbnails |
| K-59 | Resolution switching | P1 | Apply resolution changes live |

---

## Detailed Requirements

### K-56: Menu Background Rendering
**File:** `crates/genesis-kernel/src/menu_backdrop.rs`

Create an animated background for the main menu:
- Slowly moving clouds/particles
- Subtle parallax effect
- Low-GPU-cost shader effect
- Support for static fallback image
- Day/night cycle ambient effect

### K-57: Transition Effects
**File:** `crates/genesis-kernel/src/transitions.rs`

Screen transitions for smooth UX:
- Fade to black / fade from black
- Crossfade between scenes
- Configurable duration (default 0.3s)
- GPU-based alpha blending

### K-58: Screenshot Capture
**File:** `crates/genesis-kernel/src/screenshot.rs`

Capture framebuffer for save thumbnails:
- Capture current frame to CPU buffer
- Downsample to thumbnail size (256x144)
- Return as byte array (PNG-ready)
- Async to avoid frame stalls

### K-59: Resolution Switching
**File:** `crates/genesis-kernel/src/resolution.rs`

Handle runtime resolution changes:
- Resize all render targets
- Maintain aspect ratio
- Update projection matrices
- Support fullscreen toggle

---

## Definition of Done

- [ ] Menu backdrop renders with animation
- [ ] Transitions work for fade in/out
- [ ] Screenshots capture to thumbnail size
- [ ] Resolution can be changed at runtime
- [ ] All tests pass
- [ ] No clippy warnings
