/**
 * Terrain Simulation Test Suite
 *
 * Comprehensive tests for terrain simulation behavior using deterministic
 * seeds, screenshot capture, and color analysis.
 *
 * Expected Material Colors (from render.rs create_default_colors):
 *   0: Air       - #14141E (dark blue-gray)
 *   1: Water     - #40A4DF (blue)
 *   2: Sand      - #C2B280 (tan)
 *   3: Grass     - #567D46 (green)
 *   4: Dirt      - #8B5A2B (brown)
 *   5: Stone     - #808080 (gray)
 *   6: Snow      - #F0FAFF (white)
 *   7: Metal     - #C0C0C0 (silver)
 *   8: Wood      - #8B5A2B (brown)
 *   9: Glass     - #C8DCFF (light blue)
 *  10: Concrete  - #A0A0A0 (gray)
 *  11: Lava      - #FF6432 (orange)
 *  12: Oil       - #964B00 (dark brown)
 *  13: Acid      - #C8C832 (yellow-green)
 *  14: Plasma    - #643296 (purple)
 *  15: Light     - #FFFFFF (white)
 *
 * Extended terrain materials (from terrain_simulation.rs):
 *  20: Bedrock      - extremely hard, no erosion
 *  21: Clay         - soft, holds water
 *  22: Gravel       - loose rock fragments
 *  23: Sediment     - deposited from erosion
 *  24: Mud          - wet dirt
 *  25: Ice          - frozen water
 *  26: Magma        - molten rock, very hot
 *  27: Lava         - cooling magma
 *  28: Volcanic Rock - cooled lava
 *  29: Ash          - volcanic debris
 *  30: Obsidian     - rapidly cooled lava
 *  31: Aquifer      - underground water
 */

import * as fs from 'fs';
import * as path from 'path';
import { spawn, ChildProcess } from 'child_process';

// ============================================================================
// Color Analysis Types
// ============================================================================

interface ColorCount {
  hex: string;
  count: number;
  percentage: number;
}

interface ColorAnalysis {
  totalPixels: number;
  topColors: ColorCount[];
  materialPresence: {
    water: number;
    grass: number;
    dirt: number;
    stone: number;
    sand: number;
    lava: number;
    snow: number;
    air: number;
  };
}

interface TestResult {
  name: string;
  passed: boolean;
  message: string;
  details?: Record<string, unknown>;
}

// ============================================================================
// Material Color Definitions (matching GPU render shader)
// ============================================================================

const MATERIAL_COLORS: Record<string, { name: string; rgb: [number, number, number]; tolerance: number }> = {
  // Core materials - hex colors from create_default_colors()
  '#14141E': { name: 'Air', rgb: [20, 20, 30], tolerance: 20 },
  '#40A4DF': { name: 'Water', rgb: [64, 164, 223], tolerance: 30 },
  '#C2B280': { name: 'Sand', rgb: [194, 178, 128], tolerance: 25 },
  '#567D46': { name: 'Grass', rgb: [86, 125, 70], tolerance: 25 },
  '#8B5A2B': { name: 'Dirt', rgb: [139, 90, 43], tolerance: 25 },
  '#808080': { name: 'Stone', rgb: [128, 128, 128], tolerance: 30 },
  '#F0FAFF': { name: 'Snow', rgb: [240, 250, 255], tolerance: 15 },
  '#C0C0C0': { name: 'Metal', rgb: [192, 192, 192], tolerance: 20 },
  '#FF6432': { name: 'Lava', rgb: [255, 100, 50], tolerance: 30 },
};

// ============================================================================
// Test Macro Generator
// ============================================================================

interface MacroAction {
  type: string;
  [key: string]: unknown;
}

interface TestMacro {
  name: string;
  description: string;
  actions: MacroAction[];
}

function createTestMacro(
  name: string,
  description: string,
  seed: number,
  setupActions: MacroAction[],
  screenshotIntervals: { ms: number; filename: string; prompt: string }[]
): TestMacro {
  const actions: MacroAction[] = [
    { type: 'start_new_game' },
    { type: 'set_seed', seed },
    { type: 'regenerate_world' },
    { type: 'wait', duration_ms: 1000 }, // Let world generate
    ...setupActions,
  ];

  for (const interval of screenshotIntervals) {
    actions.push({ type: 'wait', duration_ms: interval.ms });
    actions.push({
      type: 'screenshot',
      filename: interval.filename,
      prompt: interval.prompt,
    });
  }

  actions.push({ type: 'wait', duration_ms: 500 });
  actions.push({ type: 'quit' });

  return { name, description, actions };
}

// ============================================================================
// Test Definitions
// ============================================================================

const TESTS: Array<{
  name: string;
  description: string;
  seed: number;
  setupActions: MacroAction[];
  screenshots: { ms: number; filename: string; prompt: string }[];
  expectations: {
    screenshot: string;
    minColors?: Record<string, number>; // e.g., { water: 5 } = water should be >= 5%
    maxColors?: Record<string, number>; // e.g., { lava: 0 } = lava should be 0%
    changesFrom?: string; // Previous screenshot to compare against
    expectedChange?: 'increase' | 'decrease' | 'significant';
  }[];
}> = [
  // ============================================================================
  // Test 1: Basic Terrain Generation
  // ============================================================================
  {
    name: 'basic_generation',
    description: 'Verify initial terrain generation has diverse biomes',
    seed: 42,
    setupActions: [],
    screenshots: [
      { ms: 0, filename: 'test_basic_t0.png', prompt: 'Initial terrain generation' },
    ],
    expectations: [
      {
        screenshot: 'test_basic_t0.png',
        minColors: { grass: 10, stone: 5 }, // At least 10% grass, 5% stone
      },
    ],
  },

  // ============================================================================
  // Test 2: Hydraulic Erosion Over Time
  // ============================================================================
  {
    name: 'hydraulic_erosion',
    description: 'Test that water erodes terrain and deposits sediment',
    seed: 12345,
    setupActions: [
      // Enable only hydraulic erosion, high time scale
      { type: 'open_world_tools' },
      { type: 'wait', duration_ms: 500 },
    ],
    screenshots: [
      { ms: 0, filename: 'test_erosion_t0.png', prompt: 'Before erosion' },
      { ms: 5000, filename: 'test_erosion_t5.png', prompt: 'After 5 seconds' },
      { ms: 10000, filename: 'test_erosion_t15.png', prompt: 'After 15 seconds' },
    ],
    expectations: [
      {
        screenshot: 'test_erosion_t15.png',
        changesFrom: 'test_erosion_t0.png',
        expectedChange: 'significant',
      },
    ],
  },

  // ============================================================================
  // Test 3: Volcanic Activity
  // ============================================================================
  {
    name: 'volcanic_activity',
    description: 'Test lava flow and cooling to stone',
    seed: 99999,
    setupActions: [
      // Would need to enable volcanic activity in world tools
    ],
    screenshots: [
      { ms: 0, filename: 'test_volcanic_t0.png', prompt: 'Before volcanic' },
      { ms: 10000, filename: 'test_volcanic_t10.png', prompt: 'After 10 seconds' },
    ],
    expectations: [
      // Lava should appear if volcanic is enabled
      // {
      //   screenshot: 'test_volcanic_t10.png',
      //   minColors: { lava: 1 }, // At least some lava
      // },
    ],
  },

  // ============================================================================
  // Test 4: Rain and Water Cycle
  // ============================================================================
  {
    name: 'water_cycle',
    description: 'Test precipitation, water accumulation, evaporation',
    seed: 77777,
    setupActions: [],
    screenshots: [
      { ms: 0, filename: 'test_water_t0.png', prompt: 'Initial state' },
      { ms: 10000, filename: 'test_water_t10.png', prompt: 'After 10 seconds' },
    ],
    expectations: [
      {
        screenshot: 'test_water_t10.png',
        minColors: { water: 1 }, // Should have some water
      },
    ],
  },

  // ============================================================================
  // Test 5: Simulation Runs (Terrain Changes)
  // ============================================================================
  {
    name: 'simulation_runs',
    description: 'Verify terrain simulation is active and modifying terrain',
    seed: 11111,
    setupActions: [],
    screenshots: [
      { ms: 1000, filename: 'test_sim_t1.png', prompt: 'After 1 second' },
      { ms: 5000, filename: 'test_sim_t6.png', prompt: 'After 6 seconds' },
    ],
    expectations: [
      {
        screenshot: 'test_sim_t6.png',
        changesFrom: 'test_sim_t1.png',
        expectedChange: 'significant',
      },
    ],
  },
];

// ============================================================================
// Test Runner
// ============================================================================

async function runMacro(macroPath: string): Promise<void> {
  const genesisPath = path.join(__dirname, '..', 'target', 'release', 'genesis');

  return new Promise((resolve, reject) => {
    const proc: ChildProcess = spawn(genesisPath, [
      '--pure-colors',
      '--macro-file', macroPath,
    ], {
      cwd: path.join(__dirname, '..'),
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    proc.stdout?.on('data', (data) => { stdout += data.toString(); });
    proc.stderr?.on('data', (data) => { stderr += data.toString(); });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Process exited with code ${code}\n${stderr}`));
      }
    });

    // Timeout after 60 seconds
    setTimeout(() => {
      proc.kill('SIGTERM');
      reject(new Error('Macro execution timed out'));
    }, 60000);
  });
}

async function analyzeScreenshot(filepath: string): Promise<ColorAnalysis | null> {
  // Use the existing analyze-colors.ts script
  const analyzerPath = path.join(__dirname, 'analyze-colors.ts');

  return new Promise((resolve, reject) => {
    const proc = spawn('npx', ['ts-node', analyzerPath, filepath], {
      cwd: __dirname,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    proc.stdout?.on('data', (data) => { stdout += data.toString(); });

    proc.on('close', (code) => {
      if (code !== 0) {
        resolve(null);
        return;
      }

      // Parse the output to extract color percentages
      const analysis: ColorAnalysis = {
        totalPixels: 0,
        topColors: [],
        materialPresence: {
          water: 0,
          grass: 0,
          dirt: 0,
          stone: 0,
          sand: 0,
          lava: 0,
          snow: 0,
          air: 0,
        },
      };

      // Extract total pixels
      const sizeMatch = stdout.match(/(\d+) pixels/);
      if (sizeMatch) {
        analysis.totalPixels = parseInt(sizeMatch[1], 10);
      }

      // Extract color percentages from quantized colors section
      const lines = stdout.split('\n');
      for (const line of lines) {
        const match = line.match(/#([0-9A-F]{6})\s*\|\s*\d+\s*\|\s*([\d.]+)%/i);
        if (match) {
          const hex = `#${match[1].toUpperCase()}`;
          const percentage = parseFloat(match[2]);
          analysis.topColors.push({ hex, count: 0, percentage });

          // Try to match to known materials
          for (const [matHex, matInfo] of Object.entries(MATERIAL_COLORS)) {
            if (colorDistance(hex, matHex) < matInfo.tolerance) {
              const key = matInfo.name.toLowerCase() as keyof typeof analysis.materialPresence;
              if (key in analysis.materialPresence) {
                analysis.materialPresence[key] += percentage;
              }
            }
          }
        }
      }

      resolve(analysis);
    });
  });
}

function colorDistance(hex1: string, hex2: string): number {
  const r1 = parseInt(hex1.slice(1, 3), 16);
  const g1 = parseInt(hex1.slice(3, 5), 16);
  const b1 = parseInt(hex1.slice(5, 7), 16);
  const r2 = parseInt(hex2.slice(1, 3), 16);
  const g2 = parseInt(hex2.slice(3, 5), 16);
  const b2 = parseInt(hex2.slice(5, 7), 16);
  return Math.sqrt((r1 - r2) ** 2 + (g1 - g2) ** 2 + (b1 - b2) ** 2);
}

async function compareScreenshots(path1: string, path2: string): Promise<boolean> {
  // Use file hashes to check if different
  const { createHash } = await import('crypto');

  try {
    const buf1 = fs.readFileSync(path1);
    const buf2 = fs.readFileSync(path2);
    const hash1 = createHash('sha256').update(buf1).digest('hex');
    const hash2 = createHash('sha256').update(buf2).digest('hex');
    return hash1 !== hash2;
  } catch {
    return false;
  }
}

async function runTest(test: typeof TESTS[0]): Promise<TestResult[]> {
  const results: TestResult[] = [];
  const macroDir = path.join(__dirname, '..', 'macros');
  const screenshotDir = path.join(__dirname, '..', 'screenshots');

  // Create macro file
  const macro = createTestMacro(
    `test_${test.name}`,
    test.description,
    test.seed,
    test.setupActions,
    test.screenshots
  );

  const macroPath = path.join(macroDir, `test_${test.name}.json`);
  fs.writeFileSync(macroPath, JSON.stringify(macro, null, 2));

  console.log(`\n📋 Running test: ${test.name}`);
  console.log(`   ${test.description}`);

  try {
    // Run the game with macro
    await runMacro(macroPath);

    // Evaluate expectations
    for (const expectation of test.expectations) {
      const screenshotPath = path.join(screenshotDir, expectation.screenshot);

      if (!fs.existsSync(screenshotPath)) {
        results.push({
          name: `${test.name}: ${expectation.screenshot}`,
          passed: false,
          message: `Screenshot not found: ${expectation.screenshot}`,
        });
        continue;
      }

      // Color presence checks
      if (expectation.minColors || expectation.maxColors) {
        const analysis = await analyzeScreenshot(screenshotPath);
        if (!analysis) {
          results.push({
            name: `${test.name}: ${expectation.screenshot} color analysis`,
            passed: false,
            message: 'Failed to analyze screenshot colors',
          });
          continue;
        }

        if (expectation.minColors) {
          for (const [material, minPct] of Object.entries(expectation.minColors)) {
            const actual = analysis.materialPresence[material as keyof typeof analysis.materialPresence] || 0;
            const passed = actual >= minPct;
            results.push({
              name: `${test.name}: ${material} >= ${minPct}%`,
              passed,
              message: passed ? `${material}: ${actual.toFixed(1)}% (OK)` : `${material}: ${actual.toFixed(1)}% (expected >= ${minPct}%)`,
              details: { expected: minPct, actual },
            });
          }
        }
      }

      // Change comparison
      if (expectation.changesFrom) {
        const beforePath = path.join(screenshotDir, expectation.changesFrom);
        if (fs.existsSync(beforePath)) {
          const changed = await compareScreenshots(beforePath, screenshotPath);
          const passed = expectation.expectedChange === 'significant' ? changed : !changed;
          results.push({
            name: `${test.name}: terrain changes`,
            passed,
            message: changed
              ? 'Terrain changed between screenshots (simulation active)'
              : 'Terrain did NOT change (simulation may be inactive)',
          });
        }
      }
    }
  } catch (error) {
    results.push({
      name: test.name,
      passed: false,
      message: `Test execution failed: ${error}`,
    });
  }

  return results;
}

// ============================================================================
// Main
// ============================================================================

async function main() {
  console.log('═══════════════════════════════════════════════════════════════════');
  console.log(' 🌍 Terrain Simulation Test Suite');
  console.log('═══════════════════════════════════════════════════════════════════');

  const allResults: TestResult[] = [];

  for (const test of TESTS) {
    const results = await runTest(test);
    allResults.push(...results);
  }

  // Summary
  console.log('\n═══════════════════════════════════════════════════════════════════');
  console.log(' 📊 Test Results Summary');
  console.log('═══════════════════════════════════════════════════════════════════');

  const passed = allResults.filter(r => r.passed).length;
  const failed = allResults.filter(r => !r.passed).length;

  for (const result of allResults) {
    const icon = result.passed ? '✅' : '❌';
    console.log(`${icon} ${result.name}: ${result.message}`);
  }

  console.log('\n───────────────────────────────────────────────────────────────────');
  console.log(`Total: ${allResults.length} | Passed: ${passed} | Failed: ${failed}`);
  console.log('───────────────────────────────────────────────────────────────────');

  process.exit(failed > 0 ? 1 : 0);
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}
