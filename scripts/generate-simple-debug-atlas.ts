#!/usr/bin/env npx ts-node
/**
 * Generate a simple debug autotile atlas with exact known colors.
 *
 * Each terrain row (192px tall) will be a single solid color:
 * - Row 0: Pure Red (#FF0000)
 * - Row 1: Pure Green (#00FF00)
 * - Row 2: Pure Blue (#0000FF)
 * - Row 3: Pure Yellow (#FFFF00)
 * - Row 4: Pure Cyan (#00FFFF)
 * - Row 5: Pure Magenta (#FF00FF)
 * - Rows 6-25: Grayscale gradient
 *
 * This makes it trivial to identify which terrain row is being sampled.
 */

import { createCanvas } from 'canvas';
import * as fs from 'fs';
import * as path from 'path';

const TILE_SIZE = 48;
const TILES_PER_STRIP_ROW = 12;
const ROWS_PER_TERRAIN = 4;
const TERRAIN_COUNT = 26;

const ATLAS_WIDTH = TILES_PER_STRIP_ROW * TILE_SIZE; // 576
const ATLAS_HEIGHT = TERRAIN_COUNT * ROWS_PER_TERRAIN * TILE_SIZE; // 4992

// Simple solid colors for each terrain row - easy to identify
const TERRAIN_COLORS: [number, number, number][] = [
  [255, 0, 0],     // 0: Pure Red
  [0, 255, 0],     // 1: Pure Green
  [0, 0, 255],     // 2: Pure Blue
  [255, 255, 0],   // 3: Pure Yellow
  [0, 255, 255],   // 4: Pure Cyan
  [255, 0, 255],   // 5: Pure Magenta
  [255, 128, 0],   // 6: Orange
  [128, 0, 255],   // 7: Purple
  [0, 128, 0],     // 8: Dark Green
  [128, 128, 0],   // 9: Olive
  [0, 128, 128],   // 10: Teal
  [128, 0, 128],   // 11: Dark Magenta
  [192, 192, 192], // 12: Silver
  [128, 128, 128], // 13: Gray
  [64, 64, 64],    // 14: Dark Gray
  [255, 192, 203], // 15: Pink
  [165, 42, 42],   // 16: Brown
  [255, 165, 0],   // 17: Bright Orange
  [0, 0, 128],     // 18: Navy
  [0, 128, 255],   // 19: Sky Blue
  [255, 255, 128], // 20: Light Yellow
  [128, 255, 128], // 21: Light Green
  [128, 128, 255], // 22: Light Blue
  [255, 128, 128], // 23: Light Red
  [64, 224, 208],  // 24: Turquoise
  [255, 215, 0],   // 25: Gold
];

function generateSimpleDebugAtlas(outputPath: string): void {
  console.log(`Generating SIMPLE debug atlas: ${ATLAS_WIDTH}x${ATLAS_HEIGHT}`);
  console.log(`Each terrain row (${ROWS_PER_TERRAIN * TILE_SIZE}px tall) = solid color`);

  const canvas = createCanvas(ATLAS_WIDTH, ATLAS_HEIGHT);
  const ctx = canvas.getContext('2d');

  // Fill each terrain strip with its solid color
  for (let terrain = 0; terrain < TERRAIN_COUNT; terrain++) {
    const [r, g, b] = TERRAIN_COLORS[terrain];
    const terrainBaseY = terrain * ROWS_PER_TERRAIN * TILE_SIZE;
    const terrainHeight = ROWS_PER_TERRAIN * TILE_SIZE; // 192 pixels

    // Fill entire terrain strip with solid color
    ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
    ctx.fillRect(0, terrainBaseY, ATLAS_WIDTH, terrainHeight);

    // Add a small label in top-left corner
    ctx.fillStyle = r + g + b > 384 ? 'black' : 'white';
    ctx.font = 'bold 16px monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'top';
    ctx.fillText(`T${terrain}`, 4, terrainBaseY + 4);
  }

  // Save the image
  const buffer = canvas.toBuffer('image/png');
  fs.writeFileSync(outputPath, buffer);
  console.log(`\n✅ Simple debug atlas saved to: ${outputPath}`);

  // Print color legend
  console.log('\n📊 TERRAIN → COLOR MAPPING:');
  console.log('=' .repeat(60));
  console.log('Row | Y Range     | Hex Code | RGB');
  console.log('-'.repeat(60));
  for (let i = 0; i < TERRAIN_COUNT; i++) {
    const [r, g, b] = TERRAIN_COLORS[i];
    const yStart = i * ROWS_PER_TERRAIN * TILE_SIZE;
    const yEnd = yStart + ROWS_PER_TERRAIN * TILE_SIZE - 1;
    const hex = `#${r.toString(16).padStart(2, '0').toUpperCase()}${g.toString(16).padStart(2, '0').toUpperCase()}${b.toString(16).padStart(2, '0').toUpperCase()}`;
    console.log(`${i.toString().padStart(2)}  | ${yStart.toString().padStart(4)}-${yEnd.toString().padStart(4)} | ${hex}  | (${r.toString().padStart(3)}, ${g.toString().padStart(3)}, ${b.toString().padStart(3)})`);
  }

  console.log('\n📋 BIOME → TERRAIN ROW MAPPING (from shader):');
  console.log('=' .repeat(60));
  console.log('Biome 0 (Forest)   → Row 0 → #FF0000 (Red)');
  console.log('Biome 1 (Desert)   → Row 2 → #0000FF (Blue)');
  console.log('Biome 2 (Cave)     → Row 1 → #00FF00 (Green)');
  console.log('Biome 3 (Ocean)    → Row 3 → #FFFF00 (Yellow)');
  console.log('Biome 4 (Plains)   → Row 0 → #FF0000 (Red)');
  console.log('Biome 5 (Mountain) → Row 1 → #00FF00 (Green)');
  console.log('Biome 6 (Swamp)    → Row 0 → #FF0000 (Red)');
  console.log('Biome 7 (River)    → Row 3 → #FFFF00 (Yellow)');
  console.log('Biome 8 (Farm)     → Row 0 → #FF0000 (Red)');
}

// Main
const outputPath = process.argv[2] || path.join(__dirname, '..', 'assets', 'debug_simple_atlas.png');
generateSimpleDebugAtlas(outputPath);
