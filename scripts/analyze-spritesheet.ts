#!/usr/bin/env npx ts-node

/**
 * Analyze Sprite Sheet CLI - Extract sprite bounding boxes using AI vision
 *
 * This script analyzes sprite sheets to identify individual frames and animations.
 * It handles large sprite sheets by splitting them into quadrants when needed.
 *
 * Usage:
 *   npx ts-node scripts/analyze-spritesheet.ts -i <image-path> [options]
 *
 * Examples:
 *   npx ts-node scripts/analyze-spritesheet.ts -i assets/player.png
 *   npx ts-node scripts/analyze-spritesheet.ts -i assets/player.png --split
 *   npx ts-node scripts/analyze-spritesheet.ts -i assets/player.png -o output.json
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
  accountId: '230639770018',
  defaultModel: 'us.amazon.nova-pro-v1:0',
  maxRetries: 10,
  maxTokens: 8192,
  temperature: 0.1,
  // Max image size before splitting (in pixels)
  maxImageSize: 1500,
  // Minimum transparent gap to use for splitting
  minTransparentGap: 8
};

// ============================================================================
// Types
// ============================================================================

interface BoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
  label?: string;
  confidence?: number;
}

interface SpriteAnimation {
  name: string;
  direction?: 'down' | 'left' | 'right' | 'up';
  frames: BoundingBox[];
  row?: number;
  frameCount?: number;
}

interface SpriteSheetAnalysis {
  imageWidth: number;
  imageHeight: number;
  frameWidth?: number;
  frameHeight?: number;
  gridCols?: number;
  gridRows?: number;
  animations: SpriteAnimation[];
  rawBoundingBoxes: BoundingBox[];
  splitInfo?: {
    wasplit: boolean;
    quadrants: Array<{ x: number; y: number; width: number; height: number }>;
  };
}

interface LLMMessage {
  role: 'user' | 'assistant' | 'system';
  content: Array<{
    text?: string;
    image?: {
      format: 'jpeg' | 'png' | 'gif' | 'webp';
      source: { bytes: string };
    };
  }>;
}

interface AnalyzeOptions {
  image: string;
  output?: string;
  split: boolean;
  model: string;
  verbose: boolean;
}

// ============================================================================
// Bounding Box Prompt Template
// ============================================================================

const SPRITE_ANALYSIS_PROMPT = `You are analyzing a sprite sheet image for a 2D game character.

Your task is to identify ALL sprite frames and their bounding boxes in this image.

## Required Output Format (JSON)

You MUST respond with valid JSON in exactly this format:

\`\`\`json
{
  "frameWidth": <estimated uniform frame width in pixels>,
  "frameHeight": <estimated uniform frame height in pixels>,
  "gridCols": <number of columns if grid-based>,
  "gridRows": <number of rows if grid-based>,
  "animations": [
    {
      "name": "<animation name, e.g. 'idle', 'walk'>",
      "direction": "<'down'|'left'|'right'|'up' if directional>",
      "row": <0-indexed row number>,
      "frameCount": <number of frames in this animation>,
      "frames": [
        { "x": <x>, "y": <y>, "width": <w>, "height": <h>, "label": "<frame label>" }
      ]
    }
  ],
  "rawBoundingBoxes": [
    { "x": <x>, "y": <y>, "width": <w>, "height": <h>, "label": "<description>" }
  ]
}
\`\`\`

## Analysis Guidelines

1. **Grid Detection**: Most sprite sheets use uniform grids. Measure the apparent cell size.
2. **Animation Rows**: Character sheets often have rows for different directions:
   - Row 0: Facing down (toward viewer)
   - Row 1: Facing left
   - Row 2: Facing right
   - Row 3: Facing up (away from viewer)
   - Rows 4-7: Same directions but walking animation
3. **Frame Counting**: Count visible sprite frames in each row.
4. **Bounding Boxes**: Provide pixel coordinates (x, y from top-left).
5. **Be Precise**: Use exact pixel measurements where possible.

## Image Offset Context

This image may be a quadrant of a larger sprite sheet.
- offsetX: {OFFSET_X}
- offsetY: {OFFSET_Y}

Add these offsets to all x/y coordinates in your response to get absolute positions.

Analyze the image and respond with JSON only, no other text.`;

const SPLIT_DETECTION_PROMPT = `Analyze this image to find optimal horizontal and vertical split lines.

Look for transparent or empty regions that span the full width or height of the image.

Respond with JSON in this exact format:

\`\`\`json
{
  "horizontalSplits": [<y-coordinate of horizontal transparent gap midpoints>],
  "verticalSplits": [<x-coordinate of vertical transparent gap midpoints>],
  "hasClearGrid": <true if image has obvious grid structure>,
  "suggestedGridSize": { "width": <cell width>, "height": <cell height> }
}
\`\`\`

Only list split points where there's at least 4 pixels of transparency.
If no good split points exist, return empty arrays.
Respond with JSON only.`;

// ============================================================================
// Bedrock Service
// ============================================================================

class BedrockService {
  private client: BedrockRuntimeClient;
  private region: string;

  constructor() {
    this.region = CONFIG.region;
    this.client = new BedrockRuntimeClient({
      region: this.region,
      credentials: fromNodeProviderChain({ profile: CONFIG.awsProfile })
    });
  }

  async analyzeImage(
    imageBuffer: Buffer,
    format: 'png' | 'jpeg',
    prompt: string,
    modelId?: string
  ): Promise<string> {
    const selectedModel = modelId || CONFIG.defaultModel;
    const isAnthropicModel = /anthropic/i.test(selectedModel);

    const imageContent = isAnthropicModel
      ? {
          type: 'input_image',
          source: {
            type: 'base64',
            media_type: format === 'png' ? 'image/png' : 'image/jpeg',
            data: imageBuffer.toString('base64')
          }
        }
      : {
          image: {
            format,
            source: {
              bytes: imageBuffer.toString('base64')
            }
          }
        };

    const textContent = isAnthropicModel
      ? { type: 'text', text: prompt }
      : { text: prompt };

    const messages = [
      {
        role: 'user',
        content: [imageContent, textContent]
      }
    ];

    let requestBody: any;

    if (isAnthropicModel) {
      requestBody = {
        anthropic_version: 'bedrock-2023-05-31',
        max_tokens: CONFIG.maxTokens,
        temperature: CONFIG.temperature,
        messages
      };
    } else {
      requestBody = {
        messages,
        inferenceConfig: {
          maxTokens: CONFIG.maxTokens,
          temperature: CONFIG.temperature
        }
      };
    }

    const command = new InvokeModelCommand({
      contentType: 'application/json',
      body: JSON.stringify(requestBody),
      modelId: selectedModel
    });

    let response;
    for (let attempt = 1; attempt <= CONFIG.maxRetries; attempt++) {
      try {
        response = await this.client.send(command);
        break;
      } catch (err: any) {
        if (attempt < CONFIG.maxRetries && this.isRetryable(err)) {
          const delay = Math.min(1000 * Math.pow(2, attempt - 1), 30000);
          console.warn(`⏳ Retry ${attempt}/${CONFIG.maxRetries} in ${delay}ms...`);
          await this.sleep(delay);
          continue;
        }
        throw err;
      }
    }

    if (!response) {
      throw new Error('No response from Bedrock');
    }

    const result = JSON.parse(new TextDecoder().decode(response.body));
    return this.extractText(result);
  }

  private extractText(result: any): string {
    // Nova format
    if (result?.output?.message?.content) {
      const textBlock = result.output.message.content.find((b: any) => b?.text);
      if (textBlock?.text) return textBlock.text;
    }
    // Anthropic format
    if (Array.isArray(result?.content)) {
      const texts = result.content.filter((b: any) => b?.type === 'text').map((b: any) => b.text);
      if (texts.length > 0) return texts.join('\n');
    }
    throw new Error('Could not extract text from response');
  }

  private isRetryable(err: any): boolean {
    const statusCode = err?.$metadata?.httpStatusCode;
    if (statusCode === 429 || statusCode >= 500) return true;
    if (/throttl|rate/i.test(err?.message || '')) return true;
    return false;
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// ============================================================================
// Image Processing
// ============================================================================

interface ImageQuadrant {
  buffer: Buffer;
  x: number;
  y: number;
  width: number;
  height: number;
}

async function getImageDimensions(imagePath: string): Promise<{ width: number; height: number }> {
  const metadata = await sharp(imagePath).metadata();
  return { width: metadata.width || 0, height: metadata.height || 0 };
}

async function findTransparentSplitLines(
  imagePath: string
): Promise<{ horizontal: number[]; vertical: number[] }> {
  const image = sharp(imagePath);
  const { width, height, channels } = await image.metadata();

  if (!width || !height || channels !== 4) {
    return { horizontal: [], vertical: [] };
  }

  // Get raw pixel data
  const { data } = await image.raw().toBuffer({ resolveWithObject: true });

  const horizontal: number[] = [];
  const vertical: number[] = [];

  // Find horizontal transparent lines (scan rows)
  let inGap = false;
  let gapStart = 0;

  for (let y = 0; y < height; y++) {
    let isTransparentRow = true;
    for (let x = 0; x < width && isTransparentRow; x++) {
      const alpha = data[(y * width + x) * 4 + 3];
      if (alpha > 10) isTransparentRow = false;
    }

    if (isTransparentRow) {
      if (!inGap) {
        gapStart = y;
        inGap = true;
      }
    } else if (inGap) {
      const gapSize = y - gapStart;
      if (gapSize >= CONFIG.minTransparentGap) {
        horizontal.push(Math.floor(gapStart + gapSize / 2));
      }
      inGap = false;
    }
  }

  // Find vertical transparent lines (scan columns)
  inGap = false;
  gapStart = 0;

  for (let x = 0; x < width; x++) {
    let isTransparentCol = true;
    for (let y = 0; y < height && isTransparentCol; y++) {
      const alpha = data[(y * width + x) * 4 + 3];
      if (alpha > 10) isTransparentCol = false;
    }

    if (isTransparentCol) {
      if (!inGap) {
        gapStart = x;
        inGap = true;
      }
    } else if (inGap) {
      const gapSize = x - gapStart;
      if (gapSize >= CONFIG.minTransparentGap) {
        vertical.push(Math.floor(gapStart + gapSize / 2));
      }
      inGap = false;
    }
  }

  return { horizontal, vertical };
}

async function splitImageIntoQuadrants(imagePath: string): Promise<ImageQuadrant[]> {
  const { width, height } = await getImageDimensions(imagePath);

  // Try to find natural split points at transparent edges
  const { horizontal, vertical } = await findTransparentSplitLines(imagePath);

  // Pick the best split points (closest to center)
  const midX = width / 2;
  const midY = height / 2;

  const splitX = vertical.length > 0
    ? vertical.reduce((best, x) => Math.abs(x - midX) < Math.abs(best - midX) ? x : best, vertical[0])
    : Math.floor(midX);

  const splitY = horizontal.length > 0
    ? horizontal.reduce((best, y) => Math.abs(y - midY) < Math.abs(best - midY) ? y : best, horizontal[0])
    : Math.floor(midY);

  console.log(`📐 Splitting at x=${splitX}, y=${splitY} (image: ${width}x${height})`);

  const quadrants: Array<{ x: number; y: number; w: number; h: number }> = [
    { x: 0, y: 0, w: splitX, h: splitY }, // Top-left
    { x: splitX, y: 0, w: width - splitX, h: splitY }, // Top-right
    { x: 0, y: splitY, w: splitX, h: height - splitY }, // Bottom-left
    { x: splitX, y: splitY, w: width - splitX, h: height - splitY } // Bottom-right
  ];

  const results: ImageQuadrant[] = [];

  for (const q of quadrants) {
    if (q.w > 0 && q.h > 0) {
      const buffer = await sharp(imagePath)
        .extract({ left: q.x, top: q.y, width: q.w, height: q.h })
        .png()
        .toBuffer();

      results.push({
        buffer,
        x: q.x,
        y: q.y,
        width: q.w,
        height: q.h
      });
    }
  }

  return results;
}

// ============================================================================
// Analysis Functions
// ============================================================================

function parseJsonFromResponse(text: string): any {
  // Try to extract JSON from markdown code blocks
  const jsonMatch = text.match(/```(?:json)?\s*([\s\S]*?)```/);
  if (jsonMatch) {
    return JSON.parse(jsonMatch[1].trim());
  }

  // Try direct parse
  try {
    return JSON.parse(text.trim());
  } catch {
    // Try to find JSON object
    const objectMatch = text.match(/\{[\s\S]*\}/);
    if (objectMatch) {
      return JSON.parse(objectMatch[0]);
    }
    throw new Error('Could not parse JSON from response');
  }
}

async function analyzeQuadrant(
  service: BedrockService,
  quadrant: ImageQuadrant,
  modelId: string,
  verbose: boolean
): Promise<Partial<SpriteSheetAnalysis>> {
  const prompt = SPRITE_ANALYSIS_PROMPT
    .replace('{OFFSET_X}', quadrant.x.toString())
    .replace('{OFFSET_Y}', quadrant.y.toString());

  if (verbose) {
    console.log(`  📍 Analyzing quadrant at (${quadrant.x}, ${quadrant.y}) size ${quadrant.width}x${quadrant.height}`);
  }

  const response = await service.analyzeImage(quadrant.buffer, 'png', prompt, modelId);

  if (verbose) {
    console.log(`  📝 Raw response preview: ${response.substring(0, 200)}...`);
  }

  const parsed = parseJsonFromResponse(response);

  // Apply offsets to all bounding boxes
  if (parsed.animations) {
    for (const anim of parsed.animations) {
      if (anim.frames) {
        for (const frame of anim.frames) {
          frame.x = (frame.x || 0) + quadrant.x;
          frame.y = (frame.y || 0) + quadrant.y;
        }
      }
    }
  }

  if (parsed.rawBoundingBoxes) {
    for (const box of parsed.rawBoundingBoxes) {
      box.x = (box.x || 0) + quadrant.x;
      box.y = (box.y || 0) + quadrant.y;
    }
  }

  return parsed;
}

async function analyzeSpriteSheet(
  imagePath: string,
  options: AnalyzeOptions
): Promise<SpriteSheetAnalysis> {
  const service = new BedrockService();
  const { width, height } = await getImageDimensions(imagePath);

  console.log(`\n🖼️  Image: ${path.basename(imagePath)} (${width}x${height})`);

  const needsSplit = options.split || width > CONFIG.maxImageSize || height > CONFIG.maxImageSize;

  if (needsSplit) {
    console.log(`📐 Splitting large image into quadrants...`);
    const quadrants = await splitImageIntoQuadrants(imagePath);

    const allAnimations: SpriteAnimation[] = [];
    const allBoxes: BoundingBox[] = [];
    let frameWidth: number | undefined;
    let frameHeight: number | undefined;
    let gridCols: number | undefined;
    let gridRows: number | undefined;

    for (let i = 0; i < quadrants.length; i++) {
      console.log(`\n🔍 Analyzing quadrant ${i + 1}/${quadrants.length}...`);
      const result = await analyzeQuadrant(service, quadrants[i], options.model, options.verbose);

      if (result.animations) allAnimations.push(...result.animations);
      if (result.rawBoundingBoxes) allBoxes.push(...result.rawBoundingBoxes);
      if (result.frameWidth && !frameWidth) frameWidth = result.frameWidth;
      if (result.frameHeight && !frameHeight) frameHeight = result.frameHeight;
      if (result.gridCols && !gridCols) gridCols = result.gridCols;
      if (result.gridRows && !gridRows) gridRows = result.gridRows;
    }

    return {
      imageWidth: width,
      imageHeight: height,
      frameWidth,
      frameHeight,
      gridCols,
      gridRows,
      animations: allAnimations,
      rawBoundingBoxes: allBoxes,
      splitInfo: {
        wasplit: true,
        quadrants: quadrants.map(q => ({ x: q.x, y: q.y, width: q.width, height: q.height }))
      }
    };
  } else {
    console.log(`\n🔍 Analyzing full image...`);
    const imageBuffer = readFileSync(imagePath);
    const prompt = SPRITE_ANALYSIS_PROMPT
      .replace('{OFFSET_X}', '0')
      .replace('{OFFSET_Y}', '0');

    const response = await service.analyzeImage(imageBuffer, 'png', prompt, options.model);

    if (options.verbose) {
      console.log(`📝 Raw response:\n${response}`);
    }

    const parsed = parseJsonFromResponse(response);

    return {
      imageWidth: width,
      imageHeight: height,
      frameWidth: parsed.frameWidth,
      frameHeight: parsed.frameHeight,
      gridCols: parsed.gridCols,
      gridRows: parsed.gridRows,
      animations: parsed.animations || [],
      rawBoundingBoxes: parsed.rawBoundingBoxes || [],
      splitInfo: { wasplit: false, quadrants: [] }
    };
  }
}

// ============================================================================
// CLI
// ============================================================================

function parseArgs(): AnalyzeOptions {
  const args = process.argv.slice(2);
  const options: AnalyzeOptions = {
    image: '',
    split: false,
    model: CONFIG.defaultModel,
    verbose: false
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    const next = args[i + 1];

    switch (arg) {
      case '-i':
      case '--image':
        options.image = next || '';
        i++;
        break;
      case '-o':
      case '--output':
        options.output = next || '';
        i++;
        break;
      case '--split':
        options.split = true;
        break;
      case '-m':
      case '--model':
        options.model = next || CONFIG.defaultModel;
        i++;
        break;
      case '-v':
      case '--verbose':
        options.verbose = true;
        break;
      case '-h':
      case '--help':
        console.log(`
Sprite Sheet Analyzer - Extract bounding boxes using AI vision

Usage: npx ts-node scripts/analyze-spritesheet.ts [options]

Options:
  -i, --image <path>   Path to sprite sheet image (required)
  -o, --output <path>  Output JSON file path (optional)
  --split              Force splitting image into quadrants
  -m, --model <id>     Model identifier (default: ${CONFIG.defaultModel})
  -v, --verbose        Show detailed output
  -h, --help           Show this help message

Examples:
  npx ts-node scripts/analyze-spritesheet.ts -i player.png
  npx ts-node scripts/analyze-spritesheet.ts -i player.png --split -o frames.json
  npx ts-node scripts/analyze-spritesheet.ts -i player.png -m us.anthropic.claude-sonnet-4-5-20250929-v1:0

Output JSON Schema:
{
  "imageWidth": number,
  "imageHeight": number,
  "frameWidth": number,       // Uniform frame width (if detected)
  "frameHeight": number,      // Uniform frame height (if detected)
  "gridCols": number,         // Grid columns (if grid-based)
  "gridRows": number,         // Grid rows (if grid-based)
  "animations": [
    {
      "name": string,         // e.g. "idle", "walk"
      "direction": string,    // "down" | "left" | "right" | "up"
      "row": number,          // 0-indexed row
      "frameCount": number,
      "frames": [
        { "x": number, "y": number, "width": number, "height": number, "label": string }
      ]
    }
  ],
  "rawBoundingBoxes": [       // All detected sprite regions
    { "x": number, "y": number, "width": number, "height": number, "label": string }
  ]
}
`);
        process.exit(0);
    }
  }

  if (!options.image) {
    console.error('Error: Image path is required. Use -i or --image');
    process.exit(1);
  }

  return options;
}

async function main(): Promise<void> {
  const options = parseArgs();
  const resolvedPath = path.resolve(options.image);

  if (!existsSync(resolvedPath)) {
    console.error(`❌ Image not found: ${resolvedPath}`);
    process.exit(1);
  }

  try {
    const result = await analyzeSpriteSheet(resolvedPath, options);

    console.log('\n' + '═'.repeat(80));
    console.log('📋 Analysis Result:');
    console.log('═'.repeat(80));

    console.log(`\n📐 Dimensions: ${result.imageWidth}x${result.imageHeight}`);
    if (result.frameWidth && result.frameHeight) {
      console.log(`🔲 Frame Size: ${result.frameWidth}x${result.frameHeight}`);
    }
    if (result.gridCols && result.gridRows) {
      console.log(`📊 Grid: ${result.gridCols} cols x ${result.gridRows} rows`);
    }

    console.log(`\n🎬 Animations: ${result.animations.length}`);
    for (const anim of result.animations) {
      const dir = anim.direction ? ` (${anim.direction})` : '';
      console.log(`  • ${anim.name}${dir}: ${anim.frameCount || anim.frames?.length || 0} frames`);
      if (anim.row !== undefined) {
        console.log(`    Row: ${anim.row}`);
      }
    }

    console.log(`\n📦 Bounding Boxes: ${result.rawBoundingBoxes.length}`);
    for (const box of result.rawBoundingBoxes.slice(0, 10)) {
      console.log(`  • [${box.x}, ${box.y}] ${box.width}x${box.height} - ${box.label || 'unlabeled'}`);
    }
    if (result.rawBoundingBoxes.length > 10) {
      console.log(`  ... and ${result.rawBoundingBoxes.length - 10} more`);
    }

    if (options.output) {
      const outputPath = path.resolve(options.output);
      mkdirSync(path.dirname(outputPath), { recursive: true });
      writeFileSync(outputPath, JSON.stringify(result, null, 2));
      console.log(`\n💾 Saved to: ${outputPath}`);
    } else {
      console.log('\n📄 Full JSON:');
      console.log(JSON.stringify(result, null, 2));
    }

  } catch (error) {
    console.error('\n❌ Analysis failed:');
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error('\n❌ Unexpected error:');
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
