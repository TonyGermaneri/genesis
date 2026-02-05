#!/usr/bin/env npx ts-node
/**
 * Automated E2E Testing Script for Genesis
 * 
 * This script:
 * 1. Runs the game with an automation macro
 * 2. Waits for screenshots to be captured
 * 3. Analyzes each screenshot with AI
 * 4. Generates a report
 * 
 * Usage:
 *   npx ts-node scripts/run-e2e-test.ts --macro biome_exploration
 *   npx ts-node scripts/run-e2e-test.ts --macro-file macros/seed_comparison.json
 */

import { spawn, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

// Configuration
const SCREENSHOT_DIR = path.join(__dirname, '..', 'screenshots');
const MACROS_DIR = path.join(__dirname, '..', 'macros');
const ANALYZE_SCRIPT = path.join(__dirname, 'analyze-image.ts');

interface TestResult {
  screenshot: string;
  prompt: string;
  analysis: string;
  timestamp: Date;
}

interface MacroAction {
  type: string;
  filename?: string;
  prompt?: string;
  [key: string]: unknown;
}

interface MacroDefinition {
  name: string;
  description?: string;
  actions: MacroAction[];
}

async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function waitForFile(filePath: string, timeoutMs: number = 30000): Promise<boolean> {
  const startTime = Date.now();
  while (Date.now() - startTime < timeoutMs) {
    if (fs.existsSync(filePath)) {
      // Wait a bit more for file to be fully written
      await sleep(500);
      return true;
    }
    await sleep(100);
  }
  return false;
}

async function analyzeImage(imagePath: string, prompt: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const proc = spawn('npx', ['ts-node', ANALYZE_SCRIPT, '-i', imagePath, '-p', prompt], {
      cwd: path.dirname(ANALYZE_SCRIPT),
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    proc.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    proc.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    proc.on('close', (code) => {
      if (code === 0) {
        // Parse the analysis from stdout
        const lines = stdout.split('\n');
        const analysisStart = lines.findIndex(l => l.includes('Analysis:'));
        if (analysisStart >= 0) {
          resolve(lines.slice(analysisStart + 1).join('\n').trim());
        } else {
          resolve(stdout.trim());
        }
      } else {
        reject(new Error(`Analysis failed: ${stderr}`));
      }
    });
  });
}

function loadMacro(macroPath: string): MacroDefinition {
  const content = fs.readFileSync(macroPath, 'utf-8');
  return JSON.parse(content);
}

function extractScreenshotInfo(macro: MacroDefinition): Array<{ filename: string; prompt: string }> {
  const screenshots: Array<{ filename: string; prompt: string }> = [];
  
  for (const action of macro.actions) {
    if (action.type === 'screenshot' && action.filename) {
      screenshots.push({
        filename: action.filename,
        prompt: action.prompt || 'Describe what you see in this game screenshot.',
      });
    }
  }
  
  return screenshots;
}

async function runGame(macroFile: string): Promise<ChildProcess> {
  const gameDir = path.join(__dirname, '..');
  
  console.log(`Starting game with macro: ${macroFile}`);
  
  const proc = spawn('cargo', ['run', '--release', '--', '--macro-file', macroFile], {
    cwd: gameDir,
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env, RUST_LOG: 'genesis=info' },
  });

  proc.stdout?.on('data', (data) => {
    const lines = data.toString().split('\n');
    for (const line of lines) {
      if (line.includes('[AUTOMATION]') || line.includes('Screenshot')) {
        console.log(`  ${line.trim()}`);
      }
    }
  });

  proc.stderr?.on('data', (data) => {
    const lines = data.toString().split('\n');
    for (const line of lines) {
      if (line.includes('[AUTOMATION]') || line.includes('Screenshot')) {
        console.log(`  ${line.trim()}`);
      }
    }
  });

  return proc;
}

function generateReport(results: TestResult[], macroName: string): string {
  let report = `# E2E Test Report: ${macroName}\n\n`;
  report += `Generated: ${new Date().toISOString()}\n\n`;
  report += `## Summary\n\n`;
  report += `- Total screenshots analyzed: ${results.length}\n\n`;
  report += `## Results\n\n`;

  for (const result of results) {
    report += `### ${result.screenshot}\n\n`;
    report += `**Prompt:** ${result.prompt}\n\n`;
    report += `**Analysis:**\n\n${result.analysis}\n\n`;
    report += `---\n\n`;
  }

  return report;
}

async function main() {
  const args = process.argv.slice(2);
  let macroFile: string | undefined;
  let macroName: string | undefined;

  // Parse arguments
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--macro' || args[i] === '-m') {
      macroName = args[i + 1];
      macroFile = path.join(MACROS_DIR, `${macroName}.json`);
      i++;
    } else if (args[i] === '--macro-file' || args[i] === '-f') {
      macroFile = args[i + 1];
      i++;
    }
  }

  if (!macroFile) {
    console.log('Usage: npx ts-node run-e2e-test.ts --macro <name> | --macro-file <path>');
    console.log('');
    console.log('Available macros:');
    if (fs.existsSync(MACROS_DIR)) {
      const files = fs.readdirSync(MACROS_DIR).filter(f => f.endsWith('.json'));
      for (const file of files) {
        const macro = loadMacro(path.join(MACROS_DIR, file));
        console.log(`  ${macro.name}: ${macro.description || 'No description'}`);
      }
    }
    process.exit(1);
  }

  if (!fs.existsSync(macroFile)) {
    console.error(`Macro file not found: ${macroFile}`);
    process.exit(1);
  }

  // Load macro and extract expected screenshots
  const macro = loadMacro(macroFile);
  const expectedScreenshots = extractScreenshotInfo(macro);

  console.log(`\nMacro: ${macro.name}`);
  console.log(`Description: ${macro.description || 'None'}`);
  console.log(`Expected screenshots: ${expectedScreenshots.length}\n`);

  // Clear old screenshots
  console.log('Clearing old screenshots...');
  if (fs.existsSync(SCREENSHOT_DIR)) {
    for (const file of fs.readdirSync(SCREENSHOT_DIR)) {
      const filePath = path.join(SCREENSHOT_DIR, file);
      if (expectedScreenshots.some(s => s.filename === file)) {
        fs.unlinkSync(filePath);
      }
    }
  }

  // Run the game
  console.log('\nStarting game...');
  const gameProc = await runGame(macroFile);

  // Wait for all screenshots
  console.log('\nWaiting for screenshots...');
  const results: TestResult[] = [];

  for (const expected of expectedScreenshots) {
    const screenshotPath = path.join(SCREENSHOT_DIR, expected.filename);
    console.log(`  Waiting for: ${expected.filename}`);
    
    const found = await waitForFile(screenshotPath, 60000);
    if (found) {
      console.log(`  ✓ Found: ${expected.filename}`);
      
      // Analyze the screenshot
      console.log(`    Analyzing...`);
      try {
        const analysis = await analyzeImage(screenshotPath, expected.prompt);
        results.push({
          screenshot: expected.filename,
          prompt: expected.prompt,
          analysis,
          timestamp: new Date(),
        });
        console.log(`    ✓ Analysis complete`);
      } catch (error) {
        console.log(`    ✗ Analysis failed: ${error}`);
        results.push({
          screenshot: expected.filename,
          prompt: expected.prompt,
          analysis: `Analysis failed: ${error}`,
          timestamp: new Date(),
        });
      }
    } else {
      console.log(`  ✗ Timeout waiting for: ${expected.filename}`);
    }
  }

  // Wait a bit then kill the game
  console.log('\nClosing game...');
  await sleep(2000);
  gameProc.kill('SIGTERM');

  // Generate report
  console.log('\nGenerating report...');
  const report = generateReport(results, macro.name);
  
  const reportPath = path.join(__dirname, '..', 'screenshots', `report_${macro.name}_${Date.now()}.md`);
  fs.writeFileSync(reportPath, report);
  console.log(`Report saved: ${reportPath}`);

  // Print summary
  console.log('\n=== Test Complete ===');
  console.log(`Screenshots captured: ${results.length}/${expectedScreenshots.length}`);
  
  if (results.length > 0) {
    console.log('\n=== Analysis Summary ===\n');
    for (const result of results) {
      console.log(`[${result.screenshot}]`);
      console.log(result.analysis.substring(0, 200) + (result.analysis.length > 200 ? '...' : ''));
      console.log('');
    }
  }
}

main().catch(console.error);
