#!/usr/bin/env npx ts-node
/**
 * Generate a reference autotile atlas that visually shows the 47-tile blob pattern.
 *
 * Each tile shows its neighbor configuration:
 * - Filled areas indicate where same-terrain neighbors exist
 * - This helps verify the engine is selecting correct tiles
 *
 * The 47-tile blob format uses an 8-bit bitmask:
 *   NW(1)  N(2)  NE(4)
 *    W(8)   *   E(16)
 *   SW(32) S(64) SE(128)
 *
 * Corner bits only count if both adjacent cardinals are present.
 */

import { createCanvas, CanvasRenderingContext2D } from 'canvas';
import * as fs from 'fs';
import * as path from 'path';

const TILE_SIZE = 48;
const TILES_PER_STRIP_ROW = 12;
const ROWS_PER_TERRAIN = 4;
const TERRAIN_COUNT = 26;
const TILES_PER_TERRAIN = TILES_PER_STRIP_ROW * ROWS_PER_TERRAIN; // 48

const ATLAS_WIDTH = TILES_PER_STRIP_ROW * TILE_SIZE; // 576
const ATLAS_HEIGHT = TERRAIN_COUNT * ROWS_PER_TERRAIN * TILE_SIZE; // 4992

// Neighbor mask bits
const NW = 1;
const N = 2;
const NE = 4;
const W = 8;
const E = 16;
const SW = 32;
const S = 64;
const SE = 128;

// The 47 unique effective masks in standard order
// These map tile index 0-46 to their bitmask
const TILE_TO_MASK: number[] = [
  0,   // 0: Isolated (no neighbors)
  2,   // 1: N only
  8,   // 2: W only
  10,  // 3: N+W
  11,  // 4: N+W+NW
  16,  // 5: E only
  18,  // 6: N+E
  22,  // 7: N+E+NE
  24,  // 8: W+E
  26,  // 9: N+W+E
  27,  // 10: N+W+E+NW
  30,  // 11: N+E+W+NE
  31,  // 12: N+E+W+NW+NE
  64,  // 13: S only
  66,  // 14: N+S
  72,  // 15: W+S
  74,  // 16: N+W+S
  75,  // 17: N+W+S+NW
  80,  // 18: E+S
  82,  // 19: N+E+S
  86,  // 20: N+E+S+NE
  88,  // 21: W+E+S
  90,  // 22: N+W+E+S
  91,  // 23: N+W+E+S+NW
  94,  // 24: N+W+E+S+NE
  95,  // 25: N+W+E+S+NW+NE
  104, // 26: W+S+SW
  106, // 27: N+W+S+SW
  107, // 28: N+W+S+NW+SW
  120, // 29: W+E+S+SW
  122, // 30: N+W+E+S+SW
  123, // 31: N+W+E+S+NW+SW
  126, // 32: N+W+E+S+NE+SW
  127, // 33: N+W+E+S+NW+NE+SW
  208, // 34: E+S+SE
  210, // 35: N+E+S+SE
  214, // 36: N+E+S+NE+SE
  216, // 37: W+E+S+SE
  218, // 38: N+W+E+S+SE
  219, // 39: N+W+E+S+NW+SE
  222, // 40: N+W+E+S+NE+SE
  223, // 41: N+W+E+S+NW+NE+SE
  248, // 42: W+E+S+SW+SE
  250, // 43: N+W+E+S+SW+SE
  251, // 44: N+W+E+S+NW+SW+SE
  254, // 45: N+W+E+S+NE+SW+SE
  255, // 46: All neighbors (full fill)
];

// Terrain colors - distinct hues for each terrain type
const TERRAIN_COLORS: [number, number, number][] = [
  [34, 139, 34],    // 0: Forest Green (GrassLight)
  [50, 205, 50],    // 1: Lime Green (GrassMedium)
  [0, 100, 0],      // 2: Dark Green (GrassDark)
  [85, 107, 47],    // 3: Olive (GrassForest)
  [60, 179, 113],   // 4: Medium Sea Green (GrassWater1)
  [46, 139, 87],    // 5: Sea Green (GrassWater2)
  [32, 178, 170],   // 6: Light Sea Green (GrassWater3)
  [0, 139, 139],    // 7: Dark Cyan (GrassWater4)
  [144, 238, 144],  // 8: Light Green (GrassFenced)
  [0, 0, 139],      // 9: Dark Blue (DeepWater)
  [139, 69, 19],    // 10: Saddle Brown (Fence1)
  [160, 82, 45],    // 11: Sienna (Fence2)
  [205, 133, 63],   // 12: Peru (Fence3)
  [128, 128, 128],  // 13: Gray (Mound1)
  [169, 169, 169],  // 14: Dark Gray (Mound2)
  [112, 128, 144],  // 15: Slate Gray (Wall1)
  [119, 136, 153],  // 16: Light Slate Gray (Wall2)
  [176, 196, 222],  // 17: Light Steel Blue (Wall3)
  [210, 180, 140],  // 18: Tan (Dirt)
  [124, 252, 0],    // 19: Lawn Green (GrassProps)
  [139, 90, 43],    // 20: Brown (FenceProps)
  [105, 105, 105],  // 21: Dim Gray (WallProps)
  [70, 130, 180],   // 22: Steel Blue (WaterProps)
  [222, 184, 135],  // 23: Burlywood (DirtProps)
  [255, 99, 71],    // 24: Tomato (Other1)
  [255, 165, 0],    // 25: Orange (Other2)
];

/**
 * Draw a single autotile based on its bitmask.
 * The tile is divided into a 3x3 grid showing which neighbors exist.
 */
function drawAutotileBitmask(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  mask: number,
  baseColor: [number, number, number],
  tileIndex: number,
  showNumbers: boolean = true
): void {
  const [r, g, b] = baseColor;
  const cellSize = TILE_SIZE / 3;

  // Background (no neighbor = darker/empty look)
  ctx.fillStyle = `rgb(${Math.floor(r * 0.3)}, ${Math.floor(g * 0.3)}, ${Math.floor(b * 0.3)})`;
  ctx.fillRect(x, y, TILE_SIZE, TILE_SIZE);

  // Fill center always (this is the cell itself)
  ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
  ctx.fillRect(x + cellSize, y + cellSize, cellSize, cellSize);

  // Check each neighbor and fill if present
  // North
  if (mask & N) {
    ctx.fillRect(x + cellSize, y, cellSize, cellSize);
  }
  // South
  if (mask & S) {
    ctx.fillRect(x + cellSize, y + 2 * cellSize, cellSize, cellSize);
  }
  // West
  if (mask & W) {
    ctx.fillRect(x, y + cellSize, cellSize, cellSize);
  }
  // East
  if (mask & E) {
    ctx.fillRect(x + 2 * cellSize, y + cellSize, cellSize, cellSize);
  }
  // NW (only if N and W are present)
  if ((mask & NW) && (mask & N) && (mask & W)) {
    ctx.fillRect(x, y, cellSize, cellSize);
  }
  // NE (only if N and E are present)
  if ((mask & NE) && (mask & N) && (mask & E)) {
    ctx.fillRect(x + 2 * cellSize, y, cellSize, cellSize);
  }
  // SW (only if S and W are present)
  if ((mask & SW) && (mask & S) && (mask & W)) {
    ctx.fillRect(x, y + 2 * cellSize, cellSize, cellSize);
  }
  // SE (only if S and E are present)
  if ((mask & SE) && (mask & S) && (mask & E)) {
    ctx.fillRect(x + 2 * cellSize, y + 2 * cellSize, cellSize, cellSize);
  }

  // Draw tile index in center (only if showNumbers is true)
  if (showNumbers) {
    // Draw grid lines to show the 3x3 structure (debug only)
    ctx.strokeStyle = 'rgba(0, 0, 0, 0.3)';
    ctx.lineWidth = 1;
    for (let i = 0; i <= 3; i++) {
      // Vertical lines
      ctx.beginPath();
      ctx.moveTo(x + i * cellSize, y);
      ctx.lineTo(x + i * cellSize, y + TILE_SIZE);
      ctx.stroke();
      // Horizontal lines
      ctx.beginPath();
      ctx.moveTo(x, y + i * cellSize);
      ctx.lineTo(x + TILE_SIZE, y + i * cellSize);
      ctx.stroke();
    }

    ctx.fillStyle = 'white';
    ctx.font = 'bold 12px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.strokeStyle = 'black';
    ctx.lineWidth = 2;
    const label = tileIndex.toString();
    ctx.strokeText(label, x + TILE_SIZE / 2, y + TILE_SIZE / 2);
    ctx.fillText(label, x + TILE_SIZE / 2, y + TILE_SIZE / 2);

    // Draw tile border (only when showing numbers for debug purposes)
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
    ctx.lineWidth = 2;
    ctx.strokeRect(x + 1, y + 1, TILE_SIZE - 2, TILE_SIZE - 2);
  }
}

function generateReferenceAtlas(outputPath: string, showNumbers: boolean = true): void {
  const mode = showNumbers ? 'with labels' : 'clean (no labels)';
  console.log(`🎨 Generating Reference Autotile Atlas (${mode})`);
  console.log(`   Size: ${ATLAS_WIDTH}x${ATLAS_HEIGHT} pixels`);
  console.log(`   Terrain types: ${TERRAIN_COUNT}`);
  console.log(`   Tiles per terrain: ${TILES_PER_TERRAIN}`);
  console.log('');

  const canvas = createCanvas(ATLAS_WIDTH, ATLAS_HEIGHT);
  const ctx = canvas.getContext('2d');

  // Fill background with magenta (error indicator)
  ctx.fillStyle = '#FF00FF';
  ctx.fillRect(0, 0, ATLAS_WIDTH, ATLAS_HEIGHT);

  // Draw each terrain type
  for (let terrain = 0; terrain < TERRAIN_COUNT; terrain++) {
    const baseColor = TERRAIN_COLORS[terrain];
    const terrainBaseY = terrain * ROWS_PER_TERRAIN * TILE_SIZE;

    // Draw each tile in this terrain strip (48 tiles: 12 cols x 4 rows)
    for (let tileIndex = 0; tileIndex < TILES_PER_TERRAIN; tileIndex++) {
      const stripCol = tileIndex % TILES_PER_STRIP_ROW; // 0-11
      const stripRow = Math.floor(tileIndex / TILES_PER_STRIP_ROW); // 0-3

      const tileX = stripCol * TILE_SIZE;
      const tileY = terrainBaseY + stripRow * TILE_SIZE;

      // Get the bitmask for this tile index (0-46), or use 255 for tile 47
      const mask = tileIndex < TILE_TO_MASK.length ? TILE_TO_MASK[tileIndex] : 255;

      drawAutotileBitmask(ctx, tileX, tileY, mask, baseColor, tileIndex, showNumbers);
    }

    // Draw terrain label on first tile (only if showing numbers)
    if (showNumbers) {
      const firstTileX = 0;
      const firstTileY = terrainBaseY;
      ctx.fillStyle = 'yellow';
      ctx.font = 'bold 8px monospace';
      ctx.textAlign = 'left';
      ctx.textBaseline = 'bottom';
      ctx.strokeStyle = 'black';
      ctx.lineWidth = 2;
      const terrainLabel = `T${terrain}`;
      ctx.strokeText(terrainLabel, firstTileX + 2, firstTileY + TILE_SIZE - 2);
      ctx.fillText(terrainLabel, firstTileX + 2, firstTileY + TILE_SIZE - 2);
    }
  }

  // Save the image
  const buffer = canvas.toBuffer('image/png');
  fs.writeFileSync(outputPath, buffer);
  console.log(`✅ Reference atlas saved to: ${outputPath}`);
  console.log('');
  console.log('📖 How to interpret the tiles:');
  console.log('   - Each tile shows a 3x3 grid');
  console.log('   - Center cell = the terrain cell itself (always filled)');
  console.log('   - Filled cells = neighbors of same terrain type');
  console.log('   - Dark cells = no neighbor (edge/corner)');
  console.log('');
  console.log('🔍 Test this atlas in the engine:');
  console.log('   - If edge tiles appear in the middle of a biome = bug in neighbor detection');
  console.log('   - If center tiles appear at edges = bug in neighbor detection');
  console.log('   - Tile 46 (all filled) should appear in solid biome interiors');
  console.log('   - Tile 0 (only center) should only appear for isolated single cells');
}

// Main
const args = process.argv.slice(2);
const cleanMode = args.includes('--clean');
const outputArg = args.find(a => !a.startsWith('--'));
const outputPath = outputArg || path.join(__dirname, '..', 'assets', cleanMode ? 'reference_autotile_atlas_clean.png' : 'reference_autotile_atlas.png');
generateReferenceAtlas(outputPath, !cleanMode);
