#!/usr/bin/env npx ts-node
/**
 * Analyze colors in an image using actual pixel analysis (CV).
 * Extracts dominant colors with exact hex codes and percentages.
 */

import * as fs from 'fs';
import * as path from 'path';
import { createCanvas, loadImage } from 'canvas';

interface ColorInfo {
  hex: string;
  rgb: [number, number, number];
  count: number;
  percentage: number;
}

async function analyzeColors(imagePath: string): Promise<void> {
  console.log(`\n🔍 Analyzing colors in: ${imagePath}\n`);

  // Load image
  const img = await loadImage(imagePath);
  const canvas = createCanvas(img.width, img.height);
  const ctx = canvas.getContext('2d');
  ctx.drawImage(img, 0, 0);

  const imageData = ctx.getImageData(0, 0, img.width, img.height);
  const pixels = imageData.data;
  const totalPixels = img.width * img.height;

  console.log(`📐 Image size: ${img.width}x${img.height} (${totalPixels.toLocaleString()} pixels)\n`);

  // Count colors (quantize to reduce noise - round to nearest 8)
  const colorCounts = new Map<string, number>();
  const exactColorCounts = new Map<string, number>();

  for (let i = 0; i < pixels.length; i += 4) {
    const r = pixels[i];
    const g = pixels[i + 1];
    const b = pixels[i + 2];
    const a = pixels[i + 3];

    // Skip fully transparent pixels
    if (a < 128) continue;

    // Exact color
    const exactHex = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`.toUpperCase();
    exactColorCounts.set(exactHex, (exactColorCounts.get(exactHex) || 0) + 1);

    // Quantized color (round to nearest 16 for grouping similar colors)
    const qr = Math.round(r / 16) * 16;
    const qg = Math.round(g / 16) * 16;
    const qb = Math.round(b / 16) * 16;
    const quantizedHex = `#${Math.min(qr, 255).toString(16).padStart(2, '0')}${Math.min(qg, 255).toString(16).padStart(2, '0')}${Math.min(qb, 255).toString(16).padStart(2, '0')}`.toUpperCase();
    colorCounts.set(quantizedHex, (colorCounts.get(quantizedHex) || 0) + 1);
  }

  // Sort by count
  const sortedColors = Array.from(colorCounts.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 20);

  const sortedExactColors = Array.from(exactColorCounts.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 30);

  // Print results
  console.log('━'.repeat(60));
  console.log('📊 TOP 20 QUANTIZED COLORS (grouped similar shades):');
  console.log('━'.repeat(60));
  console.log('Hex Code    | Count      | Percentage | Visual');
  console.log('-'.repeat(60));

  for (const [hex, count] of sortedColors) {
    const percentage = (count / totalPixels * 100).toFixed(2);
    const bar = '█'.repeat(Math.min(20, Math.round(count / totalPixels * 100)));
    console.log(`${hex}   | ${count.toString().padStart(10)} | ${percentage.padStart(6)}%    | ${bar}`);
  }

  console.log('\n' + '━'.repeat(60));
  console.log('📊 TOP 30 EXACT COLORS:');
  console.log('━'.repeat(60));
  console.log('Hex Code    | Count      | Percentage');
  console.log('-'.repeat(60));

  for (const [hex, count] of sortedExactColors) {
    const percentage = (count / totalPixels * 100).toFixed(3);
    console.log(`${hex}   | ${count.toString().padStart(10)} | ${percentage.padStart(7)}%`);
  }

  // Analyze specific debug colors from our atlas
  console.log('\n' + '━'.repeat(60));
  console.log('🎨 DEBUG ATLAS COLOR CHECK:');
  console.log('━'.repeat(60));
  
  const debugColors: Record<string, string> = {
    '#00C800': 'Row 0 - Bright Green (GrassLight)',
    '#8B5A2B': 'Row 1 - Brown (Dirt)',
    '#FFDC64': 'Row 2 - Yellow (Sand)',
    '#0064C8': 'Row 3 - Blue (Water)',
    '#323232': 'Row 4 - Dark Gray (Empty)',
    '#1A2633': 'Background clear color (r:0.1 g:0.15 b:0.2)',
    '#000000': 'Pure Black (transparent/missing)',
    '#FF00FF': 'Magenta (atlas missing)',
  };

  for (const [hex, desc] of Object.entries(debugColors)) {
    // Check for exact and nearby matches
    let found = 0;
    const targetR = parseInt(hex.slice(1, 3), 16);
    const targetG = parseInt(hex.slice(3, 5), 16);
    const targetB = parseInt(hex.slice(5, 7), 16);

    for (const [colorHex, count] of exactColorCounts.entries()) {
      const r = parseInt(colorHex.slice(1, 3), 16);
      const g = parseInt(colorHex.slice(3, 5), 16);
      const b = parseInt(colorHex.slice(5, 7), 16);
      
      // Check if within tolerance of 32
      if (Math.abs(r - targetR) <= 32 && Math.abs(g - targetG) <= 32 && Math.abs(b - targetB) <= 32) {
        found += count;
      }
    }

    const percentage = (found / totalPixels * 100).toFixed(2);
    const status = found > 0 ? '✓' : '✗';
    console.log(`${status} ${hex} ${desc}: ${percentage}% (${found.toLocaleString()} pixels)`);
  }

  // Summary
  console.log('\n' + '━'.repeat(60));
  console.log('📋 SUMMARY:');
  console.log('━'.repeat(60));
  console.log(`Total unique colors (exact): ${exactColorCounts.size}`);
  console.log(`Total unique colors (quantized): ${colorCounts.size}`);
}

// Main
const imagePath = process.argv[2];
if (!imagePath) {
  console.error('Usage: npx ts-node analyze-colors.ts <image-path>');
  process.exit(1);
}

analyzeColors(imagePath).catch(console.error);
