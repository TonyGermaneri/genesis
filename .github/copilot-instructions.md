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

### 2. Available Macro Actions
| Action | Syntax | Description |
|--------|--------|-------------|
| `wait` | `wait <ms>` | Pause for milliseconds |
| `move` | `move <dx> <dy> <ms>` | Move in direction for duration |
| `setpos` | `setpos <x> <y>` | Teleport player to position |
| `zoom` | `zoom <level>` | Set camera zoom level |
| `screenshot` | `screenshot [filename]` | Capture screenshot |
| `newgame` | `newgame` | Start new game |
| `pause` | `pause` | Open pause menu |
| `resume` | `resume` | Resume game |
| `worldtools` | `worldtools` | Open world tools panel |
| `seed` | `seed <value>` | Set world generation seed |
| `regen` | `regen` | Regenerate world terrain |
| `log` | `log <message>` | Log a message |
| `quit` | `quit` | Exit the game |

### 3. Screenshot Analysis
Use the AI image analysis script to evaluate visual output:

```bash
cd scripts
npx ts-node analyze-image.ts -i ../screenshots/test.png -p "Describe the terrain biomes"
```

### 4. Sprite Sheet Analysis (Bounding Boxes)
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

### 5. Full E2E Test Flow
Run automated tests with AI analysis:

```bash
npx ts-node scripts/run-e2e-test.ts --macro biome_exploration
```

### 6. Example Macro Files
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
