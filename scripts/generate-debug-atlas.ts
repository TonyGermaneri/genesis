#!/usr/bin/env npx ts-node
/**
 * Generate a debug autotile atlas with distinct colors for each tile position.
 *
 * Layout matches the Modern Exteriors autotile atlas:
 * - 576x4992 pixels total
 * - 48x48 pixel tiles
 * - 12 tiles per row (strip)
 * - 4 rows per terrain type (48 tiles per terrain)
 * - 26 terrain types stacked vertically
 *
 * Each tile will have:
 * - A distinct base color based on terrain type (row)
 * - A visible tile index number
 * - Border to show tile boundaries
 */

import { createCanvas } from 'canvas';
import * as fs from 'fs';
import * as path from 'path';

const TILE_SIZE = 48;
const TILES_PER_STRIP_ROW = 12;
const ROWS_PER_TERRAIN = 4;
const TERRAIN_COUNT = 26;
const TILES_PER_TERRAIN = TILES_PER_STRIP_ROW * ROWS_PER_TERRAIN; // 48

const ATLAS_WIDTH = TILES_PER_STRIP_ROW * TILE_SIZE; // 576
const ATLAS_HEIGHT = TERRAIN_COUNT * ROWS_PER_TERRAIN * TILE_SIZE; // 4992

// Distinct colors for each terrain type (26 colors)
const TERRAIN_COLORS: [number, number, number][] = [
  [0, 200, 0],      // 0: Bright Green (Grass)
  [139, 90, 43],    // 1: Brown (Dirt)
  [255, 220, 100],  // 2: Yellow (Sand)
  [0, 100, 200],    // 3: Blue (Water)
  [50, 50, 50],     // 4: Dark Gray (Empty/Reserved)
  [100, 255, 100],  // 5: Light Green
  [180, 120, 60],   // 6: Light Brown
  [255, 255, 150],  // 7: Light Yellow
  [50, 150, 255],   // 8: Light Blue
  [80, 80, 80],     // 9: Gray
  [0, 150, 50],     // 10: Forest Green
  [200, 150, 100],  // 11: Tan
  [255, 200, 50],   // 12: Gold
  [0, 80, 150],     // 13: Dark Blue
  [100, 100, 100],  // 14: Medium Gray
  [150, 255, 150],  // 15: Pale Green
  [220, 180, 140],  // 16: Beige
  [255, 240, 200],  // 17: Cream
  [100, 180, 255],  // 18: Sky Blue
  [120, 120, 120],  // 19: Silver
  [50, 100, 50],    // 20: Dark Green
  [160, 100, 60],   // 21: Sienna
  [200, 180, 100],  // 22: Khaki
  [0, 60, 120],     // 23: Navy
  [140, 140, 140],  // 24: Light Gray
  [200, 200, 200],  // 25: Very Light Gray
];

// Names for terrain types (for labeling)
const TERRAIN_NAMES: string[] = [
  'GrassLt',   // 0
  'GrassMd',   // 1
  'GrassDk',   // 2
  'GrassFor',  // 3
  'GrassW1',   // 4
  'GrassW2',   // 5
  'GrassW3',   // 6
  'GrassW4',   // 7
  'GrassFnc',  // 8
  'DeepWtr',   // 9
  'Fence1',    // 10
  'Fence2',    // 11
  'Fence3',    // 12
  'Mound1',    // 13
  'Mound2',    // 14
  'Wall1',     // 15
  'Wall2',     // 16
  'Wall3',     // 17
  'Dirt',      // 18
  'GrassPr',   // 19
  'FencePr',   // 20
  'WallPr',    // 21
  'WaterPr',   // 22
  'DirtPr',    // 23
  'Other1',    // 24
  'Other2',    // 25
];

function generateDebugAtlas(outputPath: string): void {
  console.log(`Generating debug atlas: ${ATLAS_WIDTH}x${ATLAS_HEIGHT}`);
  console.log(`  Tiles: ${TILES_PER_STRIP_ROW}x${ROWS_PER_TERRAIN} per terrain = ${TILES_PER_TERRAIN} tiles`);
  console.log(`  Terrain types: ${TERRAIN_COUNT}`);
  console.log(`  Total tiles: ${TERRAIN_COUNT * TILES_PER_TERRAIN}`);

  const canvas = createCanvas(ATLAS_WIDTH, ATLAS_HEIGHT);
  const ctx = canvas.getContext('2d');

  // Fill background with magenta (easy to spot if shown)
  ctx.fillStyle = '#FF00FF';
  ctx.fillRect(0, 0, ATLAS_WIDTH, ATLAS_HEIGHT);

  // Draw each terrain type
  for (let terrain = 0; terrain < TERRAIN_COUNT; terrain++) {
    const [r, g, b] = TERRAIN_COLORS[terrain];
    const terrainBaseY = terrain * ROWS_PER_TERRAIN * TILE_SIZE;

    // Draw each tile in this terrain strip (48 tiles: 12 cols x 4 rows)
    for (let tileIndex = 0; tileIndex < TILES_PER_TERRAIN; tileIndex++) {
      const stripCol = tileIndex % TILES_PER_STRIP_ROW; // 0-11
      const stripRow = Math.floor(tileIndex / TILES_PER_STRIP_ROW); // 0-3

      const tileX = stripCol * TILE_SIZE;
      const tileY = terrainBaseY + stripRow * TILE_SIZE;

      // Vary brightness based on tile index for visibility
      const brightness = 0.6 + (tileIndex / TILES_PER_TERRAIN) * 0.4;
      const tileR = Math.floor(r * brightness);
      const tileG = Math.floor(g * brightness);
      const tileB = Math.floor(b * brightness);

      // Fill tile with color
      ctx.fillStyle = `rgb(${tileR}, ${tileG}, ${tileB})`;
      ctx.fillRect(tileX, tileY, TILE_SIZE, TILE_SIZE);

      // Draw border
      ctx.strokeStyle = 'rgba(0, 0, 0, 0.5)';
      ctx.lineWidth = 1;
      ctx.strokeRect(tileX + 0.5, tileY + 0.5, TILE_SIZE - 1, TILE_SIZE - 1);

      // Draw tile index in center
      ctx.fillStyle = 'white';
      ctx.font = 'bold 10px monospace';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';

      // Draw text with black outline for readability
      const label = `${tileIndex}`;
      ctx.strokeStyle = 'black';
      ctx.lineWidth = 2;
      ctx.strokeText(label, tileX + TILE_SIZE / 2, tileY + TILE_SIZE / 2);
      ctx.fillText(label, tileX + TILE_SIZE / 2, tileY + TILE_SIZE / 2);

      // Draw terrain type label in top-left of first tile
      if (tileIndex === 0) {
        ctx.font = 'bold 8px monospace';
        ctx.textAlign = 'left';
        ctx.textBaseline = 'top';
        ctx.strokeStyle = 'black';
        ctx.lineWidth = 2;
        const terrainLabel = `T${terrain}`;
        ctx.strokeText(terrainLabel, tileX + 2, tileY + 2);
        ctx.fillStyle = 'yellow';
        ctx.fillText(terrainLabel, tileX + 2, tileY + 2);
      }
    }
  }

  // Save the image
  const buffer = canvas.toBuffer('image/png');
  fs.writeFileSync(outputPath, buffer);
  console.log(`\n✅ Debug atlas saved to: ${outputPath}`);

  // Print color legend
  console.log('\n📊 Terrain Color Legend:');
  console.log('=' .repeat(50));
  for (let i = 0; i < TERRAIN_COUNT; i++) {
    const [r, g, b] = TERRAIN_COLORS[i];
    console.log(`  Row ${i.toString().padStart(2)}: ${TERRAIN_NAMES[i].padEnd(10)} RGB(${r.toString().padStart(3)}, ${g.toString().padStart(3)}, ${b.toString().padStart(3)})`);
  }
}

// Main
const outputPath = process.argv[2] || path.join(__dirname, '..', 'assets', 'debug_autotile_atlas.png');
generateDebugAtlas(outputPath);
