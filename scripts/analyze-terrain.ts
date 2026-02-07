/**
 * Terrain Material Analyzer
 *
 * Analyzes PNG screenshots to identify terrain materials based on the
 * biome-aware shader colors defined in render.rs MULTI_CHUNK_RENDER_SHADER.
 *
 * This accounts for the actual rendered colors which vary by biome.
 */

import * as fs from 'fs';
import * as path from 'path';
import { createCanvas, loadImage } from 'canvas';

// ============================================================================
// Biome-Specific Material Colors (from MULTI_CHUNK_RENDER_SHADER)
// ============================================================================

interface ColorDef {
  name: string;
  rgb: [number, number, number];
  tolerance: number;
}

// Convert shader vec3 to RGB bytes
const shaderToRgb = (r: number, g: number, b: number): [number, number, number] => [
  Math.round(r * 255),
  Math.round(g * 255),
  Math.round(b * 255),
];

// All biome colors defined in shader PLUS observed rendered colors
// The shader applies noise, lighting, and blending which shifts colors
const BIOME_COLORS: ColorDef[] = [
  // Observed rendered grass colors (includes lighting/noise)
  { name: 'RenderedGrass1', rgb: [112, 128, 80], tolerance: 30 },  // #708050
  { name: 'RenderedGrass2', rgb: [96, 112, 64], tolerance: 30 },   // #607040
  { name: 'RenderedGrass3', rgb: [115, 125, 84], tolerance: 25 },  // #737D54
  { name: 'RenderedGrass4', rgb: [96, 113, 68], tolerance: 25 },   // #607144

  // Observed stone/gray colors
  { name: 'RenderedStone1', rgb: [112, 112, 96], tolerance: 30 },  // #707060
  { name: 'RenderedStone2', rgb: [128, 128, 112], tolerance: 30 }, // #808070
  { name: 'RenderedStone3', rgb: [112, 112, 112], tolerance: 30 }, // #707070
  { name: 'RenderedStone4', rgb: [144, 144, 144], tolerance: 25 }, // #909090
  { name: 'RenderedStone5', rgb: [118, 110, 99], tolerance: 30 },  // #766E63

  // Observed sky/water colors
  { name: 'RenderedSky', rgb: [192, 224, 240], tolerance: 35 },    // #C0E0F0
  { name: 'RenderedWater', rgb: [188, 218, 243], tolerance: 35 },  // #BCDAF3

  // Forest biome (base shader colors)
  { name: 'ForestGrass', rgb: shaderToRgb(0.290, 0.486, 0.137), tolerance: 50 },
  { name: 'ForestDirt', rgb: shaderToRgb(0.545, 0.412, 0.078), tolerance: 45 },
  { name: 'ForestStone', rgb: shaderToRgb(0.400, 0.420, 0.380), tolerance: 40 },

  // Plains biome
  { name: 'PlainsGrass', rgb: shaderToRgb(0.486, 0.702, 0.259), tolerance: 50 },
  { name: 'PlainsDirt', rgb: shaderToRgb(0.627, 0.502, 0.376), tolerance: 40 },

  // Mountain biome
  { name: 'MountainStone', rgb: shaderToRgb(0.478, 0.478, 0.478), tolerance: 45 },
  { name: 'MountainSnow', rgb: shaderToRgb(0.910, 0.910, 0.910), tolerance: 30 },
  { name: 'DarkRock', rgb: shaderToRgb(0.380, 0.350, 0.320), tolerance: 40 },

  // Desert biome
  { name: 'DesertSand', rgb: shaderToRgb(0.761, 0.651, 0.333), tolerance: 40 },
  { name: 'DesertSandstone', rgb: shaderToRgb(0.722, 0.584, 0.431), tolerance: 35 },

  // Ocean biome
  { name: 'OceanWater', rgb: shaderToRgb(0.227, 0.486, 0.647), tolerance: 45 },
  { name: 'OceanDeep', rgb: shaderToRgb(0.118, 0.302, 0.420), tolerance: 40 },
  { name: 'BeachSand', rgb: shaderToRgb(0.761, 0.706, 0.549), tolerance: 35 },

  // Cave biome
  { name: 'CaveStone', rgb: shaderToRgb(0.350, 0.340, 0.350), tolerance: 40 },
  { name: 'CaveDirt', rgb: shaderToRgb(0.400, 0.320, 0.250), tolerance: 40 },

  // Forest lake water
  { name: 'ForestWater', rgb: shaderToRgb(0.200, 0.450, 0.500), tolerance: 45 },
  { name: 'ForestWaterDeep', rgb: shaderToRgb(0.100, 0.280, 0.350), tolerance: 40 },

  // Night sky
  { name: 'NightSky', rgb: shaderToRgb(0.05, 0.08, 0.2), tolerance: 35 },
];

// Material categories for analysis
const MATERIAL_CATEGORIES: Record<string, string[]> = {
  'Vegetation': ['RenderedGrass1', 'RenderedGrass2', 'RenderedGrass3', 'RenderedGrass4',
                 'ForestGrass', 'PlainsGrass'],
  'Soil': ['ForestDirt', 'PlainsDirt', 'CaveDirt'],
  'Stone': ['RenderedStone1', 'RenderedStone2', 'RenderedStone3', 'RenderedStone4', 'RenderedStone5',
            'ForestStone', 'MountainStone', 'CaveStone', 'DarkRock'],
  'Sand': ['DesertSand', 'DesertSandstone', 'BeachSand'],
  'Water': ['RenderedWater', 'OceanWater', 'OceanDeep', 'ForestWater', 'ForestWaterDeep'],
  'Snow': ['MountainSnow'],
  'Sky': ['RenderedSky', 'NightSky'],
};

// ============================================================================
// Color Matching
// ============================================================================

function colorDistance(r1: number, g1: number, b1: number, r2: number, g2: number, b2: number): number {
  // Weighted Euclidean distance (perceptual weighting)
  const rmean = (r1 + r2) / 2;
  const dr = r1 - r2;
  const dg = g1 - g2;
  const db = b1 - b2;
  return Math.sqrt(
    (2 + rmean / 256) * dr * dr +
    4 * dg * dg +
    (2 + (255 - rmean) / 256) * db * db
  );
}

function matchBiomeColor(r: number, g: number, b: number): ColorDef | null {
  let bestMatch: ColorDef | null = null;
  let bestDistance = Infinity;

  for (const color of BIOME_COLORS) {
    const dist = colorDistance(r, g, b, color.rgb[0], color.rgb[1], color.rgb[2]);
    if (dist < color.tolerance && dist < bestDistance) {
      bestDistance = dist;
      bestMatch = color;
    }
  }

  return bestMatch;
}

function getCategoryForColor(colorName: string): string {
  for (const [category, colors] of Object.entries(MATERIAL_CATEGORIES)) {
    if (colors.includes(colorName)) {
      return category;
    }
  }
  return 'Unknown';
}

// ============================================================================
// Analysis
// ============================================================================

interface MaterialAnalysis {
  filepath: string;
  width: number;
  height: number;
  totalPixels: number;
  biomeColors: Map<string, number>;
  categories: Map<string, number>;
  unmatchedPixels: number;
  timestamp?: string;
}

async function analyzeScreenshot(filepath: string): Promise<MaterialAnalysis> {
  const img = await loadImage(filepath);
  const canvas = createCanvas(img.width, img.height);
  const ctx = canvas.getContext('2d');
  ctx.drawImage(img, 0, 0);

  const imageData = ctx.getImageData(0, 0, img.width, img.height);
  const pixels = imageData.data;

  const biomeColors = new Map<string, number>();
  const categories = new Map<string, number>();
  let unmatchedPixels = 0;

  for (let i = 0; i < pixels.length; i += 4) {
    const r = pixels[i];
    const g = pixels[i + 1];
    const b = pixels[i + 2];

    const match = matchBiomeColor(r, g, b);
    if (match) {
      biomeColors.set(match.name, (biomeColors.get(match.name) || 0) + 1);
      const category = getCategoryForColor(match.name);
      categories.set(category, (categories.get(category) || 0) + 1);
    } else {
      unmatchedPixels++;
    }
  }

  // Extract timestamp from filename if present (e.g., "sim_test_t10.png" -> "t10")
  const timestampMatch = path.basename(filepath).match(/_t(\d+)\./);
  const timestamp = timestampMatch ? `t${timestampMatch[1]}` : undefined;

  return {
    filepath,
    width: img.width,
    height: img.height,
    totalPixels: img.width * img.height,
    biomeColors,
    categories,
    unmatchedPixels,
    timestamp,
  };
}

// ============================================================================
// Comparison
// ============================================================================

interface ComparisonResult {
  before: MaterialAnalysis;
  after: MaterialAnalysis;
  categoryChanges: Map<string, { before: number; after: number; delta: number }>;
  significantChanges: string[];
}

function compareAnalyses(before: MaterialAnalysis, after: MaterialAnalysis): ComparisonResult {
  const categoryChanges = new Map<string, { before: number; after: number; delta: number }>();
  const significantChanges: string[] = [];

  // Get all categories
  const allCategories = new Set([...before.categories.keys(), ...after.categories.keys()]);

  for (const category of allCategories) {
    const beforePct = ((before.categories.get(category) || 0) / before.totalPixels) * 100;
    const afterPct = ((after.categories.get(category) || 0) / after.totalPixels) * 100;
    const delta = afterPct - beforePct;

    categoryChanges.set(category, {
      before: beforePct,
      after: afterPct,
      delta,
    });

    // Flag significant changes (> 2% change)
    if (Math.abs(delta) > 2) {
      const direction = delta > 0 ? 'increased' : 'decreased';
      significantChanges.push(`${category} ${direction} by ${Math.abs(delta).toFixed(1)}%`);
    }
  }

  return { before, after, categoryChanges, significantChanges };
}

// ============================================================================
// Output
// ============================================================================

function printAnalysis(analysis: MaterialAnalysis): void {
  const title = analysis.timestamp
    ? `Analysis @ ${analysis.timestamp}`
    : `Analysis: ${path.basename(analysis.filepath)}`;

  console.log(`\n📊 ${title}`);
  console.log('─'.repeat(60));
  console.log(`Size: ${analysis.width}x${analysis.height} (${analysis.totalPixels.toLocaleString()} pixels)`);
  console.log('');

  // Category breakdown
  console.log('Material Categories:');
  const sortedCategories = Array.from(analysis.categories.entries())
    .sort((a, b) => b[1] - a[1]);

  for (const [category, count] of sortedCategories) {
    const pct = (count / analysis.totalPixels * 100).toFixed(1);
    const bar = '█'.repeat(Math.round(parseFloat(pct) / 2));
    console.log(`  ${category.padEnd(12)} | ${pct.padStart(5)}% | ${bar}`);
  }

  const unmatchedPct = (analysis.unmatchedPixels / analysis.totalPixels * 100).toFixed(1);
  console.log(`  ${'(unmatched)'.padEnd(12)} | ${unmatchedPct.padStart(5)}%`);
}

function printComparison(comparison: ComparisonResult): void {
  console.log('\n📈 Changes Over Time');
  console.log('═'.repeat(60));
  console.log(`  ${comparison.before.timestamp || 'Before'} → ${comparison.after.timestamp || 'After'}`);
  console.log('');

  console.log('Category Changes:');
  for (const [category, change] of comparison.categoryChanges.entries()) {
    const arrow = change.delta > 0 ? '↑' : change.delta < 0 ? '↓' : '→';
    const deltaStr = change.delta > 0 ? `+${change.delta.toFixed(1)}` : change.delta.toFixed(1);
    console.log(`  ${category.padEnd(12)} | ${change.before.toFixed(1)}% ${arrow} ${change.after.toFixed(1)}% (${deltaStr}%)`);
  }

  if (comparison.significantChanges.length > 0) {
    console.log('\n🔔 Significant Changes:');
    for (const change of comparison.significantChanges) {
      console.log(`  • ${change}`);
    }
  } else {
    console.log('\n  No significant category changes detected.');
  }
}

// ============================================================================
// Main
// ============================================================================

async function main(): Promise<void> {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log('Usage:');
    console.log('  npx ts-node analyze-terrain.ts <screenshot.png>');
    console.log('  npx ts-node analyze-terrain.ts <before.png> <after.png>');
    console.log('  npx ts-node analyze-terrain.ts <prefix>*  (analyze series)');
    process.exit(1);
  }

  // Check if comparing multiple files
  if (args.length === 1) {
    // Single file or wildcard
    const filepath = args[0];

    if (filepath.includes('*')) {
      // Wildcard - find matching files
      const dir = path.dirname(filepath);
      const pattern = path.basename(filepath).replace('*', '');
      const files = fs.readdirSync(dir)
        .filter(f => f.includes(pattern) && f.endsWith('.png'))
        .map(f => path.join(dir, f))
        .sort();

      if (files.length === 0) {
        console.log('No files found matching pattern');
        process.exit(1);
      }

      console.log(`\n🔬 Terrain Analysis Series (${files.length} screenshots)`);
      console.log('═'.repeat(60));

      const analyses: MaterialAnalysis[] = [];
      for (const file of files) {
        const analysis = await analyzeScreenshot(file);
        analyses.push(analysis);
        printAnalysis(analysis);
      }

      // Compare first to last
      if (analyses.length >= 2) {
        const comparison = compareAnalyses(analyses[0], analyses[analyses.length - 1]);
        printComparison(comparison);
      }
    } else {
      // Single file
      const analysis = await analyzeScreenshot(filepath);
      printAnalysis(analysis);
    }
  } else {
    // Two files - compare
    const before = await analyzeScreenshot(args[0]);
    const after = await analyzeScreenshot(args[1]);

    printAnalysis(before);
    printAnalysis(after);

    const comparison = compareAnalyses(before, after);
    printComparison(comparison);
  }
}

main().catch(console.error);
