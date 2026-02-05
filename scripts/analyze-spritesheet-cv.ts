#!/usr/bin/env npx ts-node

/**
 * Sprite Sheet Analyzer with Computer Vision + LLM
 *
 * Uses traditional CV methods (row detection via transparency) combined with
 * LLM vision for content identification.
 *
 * Key assumptions for character sheets:
 * - Frames are exactly 48px wide (no gaps between frames)
 * - Rows have transparent gaps between them
 * - Character sprites are 48x73 pixels
 *
 * Usage:
 *   npx ts-node analyze-spritesheet-cv.ts -i <image> [options]
 */

import { readFileSync, existsSync, writeFileSync, mkdirSync } from 'fs';
import path from 'path';
import { BedrockRuntimeClient, InvokeModelCommand } from '@aws-sdk/client-bedrock-runtime';
import { fromNodeProviderChain } from '@aws-sdk/credential-providers';
import sharp from 'sharp';

// ============================================================================
// Configuration
// ============================================================================

const CONFIG = {
  awsProfile: '230639770018_cr-AdminAccess',
  region: 'us-east-1',
  defaultModel: 'us.amazon.nova-pro-v1:0',
  maxRetries: 10,
  maxTokens: 8192,
  temperature: 0.1,
  // Frame dimensions
  frameWidth: 48,
  characterHeight: 73,
  // Minimum transparent row to consider a gap
  minTransparentGap: 4,
  // Transparency threshold (0-255)
  alphaThreshold: 10
};

// ============================================================================
// Types
// ============================================================================

interface RowBounds {
  index: number;
  startY: number;
  endY: number;
  height: number;
  frameCount: number;
  contentWidth: number;
}

interface RowAnalysis {
  row: RowBounds;
  contentType: string;
  animationType?: string;
  direction?: string;
  frameDetails?: string[];
}

interface SpriteSheetAnalysis {
  imagePath: string;
  imageWidth: number;
  imageHeight: number;
  frameWidth: number;
  totalRows: number;
  rows: RowAnalysis[];
}

// ============================================================================
// Bedrock Service
// ============================================================================

class BedrockService {
  private client: BedrockRuntimeClient;

  constructor() {
    this.client = new BedrockRuntimeClient({
      region: CONFIG.region,
      credentials: fromNodeProviderChain({ profile: CONFIG.awsProfile })
    });
  }

  async analyzeImage(imageBuffer: Buffer, prompt: string): Promise<string> {
    const requestBody = {
      messages: [
        {
          role: 'user',
          content: [
            {
              image: {
                format: 'png',
                source: { bytes: imageBuffer.toString('base64') }
              }
            },
            { text: prompt }
          ]
        }
      ],
      inferenceConfig: {
        maxTokens: CONFIG.maxTokens,
        temperature: CONFIG.temperature
      }
    };

    const command = new InvokeModelCommand({
      contentType: 'application/json',
      body: JSON.stringify(requestBody),
      modelId: CONFIG.defaultModel
    });

    for (let attempt = 1; attempt <= CONFIG.maxRetries; attempt++) {
      try {
        const response = await this.client.send(command);
        const result = JSON.parse(new TextDecoder().decode(response.body));
        const textBlock = result?.output?.message?.content?.find((b: any) => b?.text);
        return textBlock?.text || '';
      } catch (err: any) {
        if (attempt < CONFIG.maxRetries && (err?.$metadata?.httpStatusCode === 429 || err?.$metadata?.httpStatusCode >= 500)) {
          const delay = Math.min(1000 * Math.pow(2, attempt - 1), 30000);
          console.warn(`⏳ Retry ${attempt}/${CONFIG.maxRetries}...`);
          await new Promise(r => setTimeout(r, delay));
          continue;
        }
        throw err;
      }
    }
    throw new Error('Max retries exceeded');
  }
}

// ============================================================================
// Computer Vision: Row Detection
// ============================================================================

async function detectRowBoundaries(imagePath: string): Promise<RowBounds[]> {
  const image = sharp(imagePath);
  const metadata = await image.metadata();
  const width = metadata.width || 0;
  const height = metadata.height || 0;

  if (!width || !height) {
    throw new Error('Could not read image dimensions');
  }

  console.log(`📐 Image: ${width}x${height}`);

  // Get raw RGBA pixel data
  const { data } = await image.raw().ensureAlpha().toBuffer({ resolveWithObject: true });

  // Scan for transparent rows (march from top to bottom)
  const rowAlpha: number[] = [];

  for (let y = 0; y < height; y++) {
    let rowMaxAlpha = 0;
    for (let x = 0; x < width; x++) {
      const alpha = data[(y * width + x) * 4 + 3];
      rowMaxAlpha = Math.max(rowMaxAlpha, alpha);
    }
    rowAlpha.push(rowMaxAlpha);
  }

  // Find row boundaries by detecting transitions between transparent and opaque
  const rows: RowBounds[] = [];
  let inContent = false;
  let contentStart = 0;
  let rowIndex = 0;

  for (let y = 0; y < height; y++) {
    const isTransparent = rowAlpha[y] < CONFIG.alphaThreshold;

    if (!inContent && !isTransparent) {
      // Start of new content row
      inContent = true;
      contentStart = y;
    } else if (inContent && isTransparent) {
      // End of content row - check if gap is significant
      let gapSize = 0;
      for (let gy = y; gy < height && rowAlpha[gy] < CONFIG.alphaThreshold; gy++) {
        gapSize++;
      }

      if (gapSize >= CONFIG.minTransparentGap) {
        // Calculate content width by scanning for rightmost non-transparent pixel
        let contentWidth = 0;
        for (let cy = contentStart; cy < y; cy++) {
          for (let x = width - 1; x >= 0; x--) {
            const alpha = data[(cy * width + x) * 4 + 3];
            if (alpha >= CONFIG.alphaThreshold) {
              contentWidth = Math.max(contentWidth, x + 1);
              break;
            }
          }
        }

        const rowHeight = y - contentStart;
        const frameCount = Math.ceil(contentWidth / CONFIG.frameWidth);

        rows.push({
          index: rowIndex++,
          startY: contentStart,
          endY: y,
          height: rowHeight,
          contentWidth,
          frameCount
        });

        inContent = false;
      }
    }
  }

  // Handle case where content extends to bottom of image
  if (inContent) {
    let contentWidth = 0;
    for (let cy = contentStart; cy < height; cy++) {
      for (let x = width - 1; x >= 0; x--) {
        const alpha = data[(cy * width + x) * 4 + 3];
        if (alpha >= CONFIG.alphaThreshold) {
          contentWidth = Math.max(contentWidth, x + 1);
          break;
        }
      }
    }

    rows.push({
      index: rowIndex,
      startY: contentStart,
      endY: height,
      height: height - contentStart,
      contentWidth,
      frameCount: Math.ceil(contentWidth / CONFIG.frameWidth)
    });
  }

  return rows;
}

// ============================================================================
// Extract Row Image
// ============================================================================

async function extractRowImage(imagePath: string, row: RowBounds): Promise<Buffer> {
  return sharp(imagePath)
    .extract({
      left: 0,
      top: row.startY,
      width: row.contentWidth,
      height: row.height
    })
    .png()
    .toBuffer();
}

// ============================================================================
// LLM Row Content Analysis
// ============================================================================

const ROW_ANALYSIS_PROMPT = `Analyze this sprite sheet row. This is a single row extracted from a character sprite sheet.

Each frame is exactly 48 pixels wide. Characters are 48x73 pixels (48 wide, 73 tall).

Describe in JSON format:
\`\`\`json
{
  "contentType": "<'character_animation'|'heads'|'sitting'|'ui_elements'|'props'|'beds'|'other'>",
  "animationType": "<'idle'|'walk'|'run'|'attack'|'throw'|'use'|'sit'|'sleep'|'die'|null>",
  "direction": "<'down'|'left'|'right'|'up'|'multi'|null>",
  "frameCount": <number of distinct frames>,
  "description": "<brief description of content>"
}
\`\`\`

Respond with JSON only.`;

async function analyzeRowContent(
  service: BedrockService,
  rowImage: Buffer,
  row: RowBounds,
  verbose: boolean
): Promise<RowAnalysis> {
  if (verbose) {
    console.log(`  🔍 Analyzing row ${row.index}: y=${row.startY}-${row.endY}, height=${row.height}, frames≈${row.frameCount}`);
  }

  try {
    const response = await service.analyzeImage(rowImage, ROW_ANALYSIS_PROMPT);

    // Parse JSON from response
    const jsonMatch = response.match(/```(?:json)?\s*([\s\S]*?)```/) || response.match(/\{[\s\S]*\}/);
    if (jsonMatch) {
      const parsed = JSON.parse(jsonMatch[1] || jsonMatch[0]);
      return {
        row,
        contentType: parsed.contentType || 'unknown',
        animationType: parsed.animationType,
        direction: parsed.direction,
        frameDetails: parsed.description ? [parsed.description] : undefined
      };
    }
  } catch (err) {
    if (verbose) {
      console.warn(`  ⚠️ Failed to analyze row ${row.index}: ${err}`);
    }
  }

  return {
    row,
    contentType: 'unknown'
  };
}

// ============================================================================
// Main Analysis
// ============================================================================

async function analyzeSpriteSheet(imagePath: string, verbose: boolean): Promise<SpriteSheetAnalysis> {
  const resolvedPath = path.resolve(imagePath);
  const metadata = await sharp(resolvedPath).metadata();

  console.log(`\n🖼️  Analyzing: ${path.basename(imagePath)}`);
  console.log(`📐 Dimensions: ${metadata.width}x${metadata.height}`);

  // Step 1: Detect row boundaries using CV
  console.log(`\n📊 Step 1: Detecting row boundaries (CV)...`);
  const rows = await detectRowBoundaries(resolvedPath);
  console.log(`   Found ${rows.length} rows`);

  for (const row of rows) {
    console.log(`   Row ${row.index}: y=${row.startY}-${row.endY}, h=${row.height}, ~${row.frameCount} frames`);
  }

  // Step 2: Analyze each row with LLM
  console.log(`\n🧠 Step 2: Analyzing row content (LLM)...`);
  const service = new BedrockService();
  const rowAnalyses: RowAnalysis[] = [];

  for (const row of rows) {
    const rowImage = await extractRowImage(resolvedPath, row);
    const analysis = await analyzeRowContent(service, rowImage, row, verbose);
    rowAnalyses.push(analysis);

    const dir = analysis.direction ? ` [${analysis.direction}]` : '';
    const anim = analysis.animationType ? ` - ${analysis.animationType}` : '';
    console.log(`   Row ${row.index}: ${analysis.contentType}${anim}${dir}`);
  }

  return {
    imagePath: resolvedPath,
    imageWidth: metadata.width || 0,
    imageHeight: metadata.height || 0,
    frameWidth: CONFIG.frameWidth,
    totalRows: rows.length,
    rows: rowAnalyses
  };
}

// ============================================================================
// CLI
// ============================================================================

interface Options {
  image: string;
  output?: string;
  verbose: boolean;
  rowsOnly: boolean;
}

function parseArgs(): Options {
  const args = process.argv.slice(2);
  const options: Options = {
    image: '',
    verbose: false,
    rowsOnly: false
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '-i':
      case '--image':
        options.image = args[++i] || '';
        break;
      case '-o':
      case '--output':
        options.output = args[++i] || '';
        break;
      case '-v':
      case '--verbose':
        options.verbose = true;
        break;
      case '--rows-only':
        options.rowsOnly = true;
        break;
      case '-h':
      case '--help':
        console.log(`
Sprite Sheet CV Analyzer - Row detection + LLM content analysis

Usage: npx ts-node analyze-spritesheet-cv.ts [options]

Options:
  -i, --image <path>   Sprite sheet image (required)
  -o, --output <path>  Output JSON file
  -v, --verbose        Detailed output
  --rows-only          Only detect rows (no LLM analysis)
  -h, --help           Show help

This tool:
1. Uses pixel analysis to detect rows (transparent gaps)
2. Uses LLM vision to identify content type per row
3. Outputs structured JSON with row bounds and content types
`);
        process.exit(0);
    }
  }

  if (!options.image) {
    console.error('Error: -i/--image is required');
    process.exit(1);
  }

  return options;
}

async function main(): Promise<void> {
  const options = parseArgs();
  const imagePath = path.resolve(options.image);

  if (!existsSync(imagePath)) {
    console.error(`❌ Image not found: ${imagePath}`);
    process.exit(1);
  }

  try {
    let result: any;

    if (options.rowsOnly) {
      console.log(`\n🖼️  Detecting rows in: ${path.basename(imagePath)}`);
      const rows = await detectRowBoundaries(imagePath);
      result = {
        imagePath,
        totalRows: rows.length,
        rows
      };

      console.log(`\n📊 Found ${rows.length} rows:`);
      for (const row of rows) {
        console.log(`  Row ${row.index}: y=${row.startY}-${row.endY}, height=${row.height}, ~${row.frameCount} frames`);
      }
    } else {
      result = await analyzeSpriteSheet(imagePath, options.verbose);
    }

    if (options.output) {
      const outputPath = path.resolve(options.output);
      mkdirSync(path.dirname(outputPath), { recursive: true });
      writeFileSync(outputPath, JSON.stringify(result, null, 2));
      console.log(`\n💾 Saved: ${outputPath}`);
    } else {
      console.log('\n📄 Result JSON:');
      console.log(JSON.stringify(result, null, 2));
    }

  } catch (error) {
    console.error('\n❌ Failed:', error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

main();
