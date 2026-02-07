# Genesis Engine - AI Development Instructions

## Preferred Development Method: E2E Macros & Screenshot Feedback Loop

This project uses **automated E2E testing with visual AI feedback** as the primary development and validation method. When making changes to rendering, world generation, biomes, or any visual feature:

### 1. Use Automation Macros
The game supports automation macros via CLI arguments:

```bash
# Run a macro from JSON file
./target/release/genesis --macro-file macros/<name>.json

# Run inline commands
./target/release/genesis --macro "newgame; wait 2000; screenshot test.png"

# Auto-start game (skip main menu)
./target/release/genesis --auto-start
```

### 2. Macro JSON Format

**IMPORTANT**: Macro files use a specific JSON structure. Each action must have a `type` field:

```json
{
  "name": "example_macro",
  "description": "Description of what the macro does",
  "actions": [
    {
      "type": "start_new_game"
    },
    {
      "type": "wait",
      "duration_ms": 2000
    },
    {
      "type": "set_zoom",
      "zoom": 2.0
    },
    {
      "type": "screenshot",
      "filename": "output.png",
      "prompt": "AI analysis prompt for the screenshot"
    },
    {
      "type": "move",
      "dx": 1.0,
      "dy": 0.0,
      "duration_ms": 3000
    },
    {
      "type": "log",
      "message": "Log message here"
    },
    {
      "type": "quit"
    }
  ]
}
```

### 3. Available Macro Action Types

| Type | Parameters | Description |
|------|------------|-------------|
| `start_new_game` | - | Start a new game |
| `wait` | `duration_ms` | Pause for milliseconds |
| `move` | `dx`, `dy`, `duration_ms` | Move in direction for duration |
| `set_position` | `x`, `y` | Teleport player to position |
| `set_zoom` | `zoom` | Set camera zoom level |
| `screenshot` | `filename`, `prompt` (optional) | Capture screenshot |
| `pause` | - | Open pause menu |
| `resume` | - | Resume game |
| `open_world_tools` | - | Open world tools panel |
| `select_tab` | `tab_name` | Select World Tools tab |
| `click_button` | `label` | Click a button by label |
| `set_seed` | `seed` | Set world generation seed |
| `regenerate_world` | - | Regenerate world terrain |
| `log` | `message` | Log a message |
| `quit` | - | Exit the game |

### 4. Screenshot Analysis
Use the AI image analysis script to evaluate visual output:

```bash
cd scripts
npx ts-node analyze-image.ts -i ../screenshots/test.png -p "Describe the terrain biomes"
```

### 5. Sprite Sheet Analysis (Bounding Boxes)
Use the sprite sheet analyzer to extract frame bounding boxes from character/animation sprite sheets:

```bash
cd scripts
npx ts-node analyze-spritesheet.ts -i ../assets/player.png
npx ts-node analyze-spritesheet.ts -i ../assets/player.png --split -o frames.json
```

**Features:**
- **Always returns structured JSON** with bounding boxes
- **Automatic splitting** for large images (>1500px) at transparent edges
- **Grid detection** for uniform sprite sheets
- **Animation detection** by row/direction

**Output JSON Schema:**
```json
{
  "imageWidth": 2781,
  "imageHeight": 1968,
  "frameWidth": 48,
  "frameHeight": 48,
  "gridCols": 58,
  "gridRows": 41,
  "animations": [
    {
      "name": "idle",
      "direction": "down",
      "row": 0,
      "frameCount": 4,
      "frames": [
        { "x": 0, "y": 0, "width": 48, "height": 48, "label": "idle_down_0" }
      ]
    }
  ],
  "rawBoundingBoxes": [
    { "x": 0, "y": 0, "width": 48, "height": 48, "label": "sprite frame" }
  ]
}
```

### 6. Full E2E Test Flow
Run automated tests with AI analysis:

```bash
npx ts-node scripts/run-e2e-test.ts --macro biome_exploration
```

### 7. Example Macro Files
- `macros/biome_exploration.json` - Explores world and captures screenshots
- `macros/seed_comparison.json` - Compares terrain across different seeds

## Key Files

| Path | Purpose |
|------|---------|
| `crates/genesis-engine/src/automation.rs` | Automation system implementation |
| `crates/genesis-kernel/src/player_sprite.rs` | Player sprite rendering system |
| `scripts/analyze-image.ts` | AI screenshot analysis (AWS Bedrock) |
| `scripts/analyze-spritesheet-cv.ts` | CV + LLM sprite sheet analyzer |
| `scripts/run-e2e-test.ts` | E2E test runner with AI feedback |
| `macros/*.json` | Macro definition files |
| `screenshots/` | Captured screenshots output |
| `assets/sprites/player/` | Player character sprite sheets |

## Development Workflow

1. **Make code changes** to rendering, biomes, world gen, etc.
2. **Create or update a macro** to exercise the feature
3. **Run the macro** to capture screenshots
4. **Analyze screenshots** with AI to evaluate results
5. **Iterate** based on AI feedback

## World Tools (In-Game)

Press `ESC` → World Tools to access:
- **Biomes Tab**: Configure biome parameters
- **Noise Tab**: Adjust terrain noise layers
- **Weather Tab**: Weather system settings
- **World Gen Tab**: Seed, chunk size, cave/ore generation

Changes can be applied via `Regenerate World` button or automation.

## Project Structure

- **Rust** game engine in `crates/genesis-engine/`
- **TypeScript** scripts in `scripts/`
- **JSON** macros in `macros/`
- Uses **wgpu** for rendering, **egui** for UI
- AWS Bedrock (Nova Pro) for AI image analysis
