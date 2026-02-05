#!/usr/bin/env npx ts-node
/**
 * Grid Pattern Detection using edge detection
 * Analyzes images for rectangular/grid patterns vs organic shapes
 */

import * as fs from 'fs';
import sharp from 'sharp';

async function detectGridPatterns(imagePath: string) {
  console.log(`\n🔍 Analyzing grid patterns in: ${imagePath}\n`);

  const image = sharp(imagePath);
  const metadata = await image.metadata();
  const { width, height } = metadata;

  console.log(`📐 Image size: ${width}x${height}`);

  // Convert to grayscale and get raw pixels
  const grayBuffer = await image
    .grayscale()
    .raw()
    .toBuffer();

  // Sobel edge detection (horizontal and vertical)
  let horizontalEdges = 0;
  let verticalEdges = 0;
  let diagonalEdges = 0;
  let totalEdges = 0;

  const threshold = 20; // Edge detection threshold

  for (let y = 1; y < height! - 1; y++) {
    for (let x = 1; x < width! - 1; x++) {
      const idx = y * width! + x;
      const center = grayBuffer[idx];

      // Get neighbors
      const left = grayBuffer[idx - 1];
      const right = grayBuffer[idx + 1];
      const top = grayBuffer[idx - width!];
      const bottom = grayBuffer[idx + width!];
      const topLeft = grayBuffer[idx - width! - 1];
      const topRight = grayBuffer[idx - width! + 1];
      const bottomLeft = grayBuffer[idx + width! - 1];
      const bottomRight = grayBuffer[idx + width! + 1];

      // Horizontal edge (difference left-right)
      const hDiff = Math.abs(left - right);
      if (hDiff > threshold) {
        horizontalEdges++;
        totalEdges++;
      }

      // Vertical edge (difference top-bottom)
      const vDiff = Math.abs(top - bottom);
      if (vDiff > threshold) {
        verticalEdges++;
        totalEdges++;
      }

      // Diagonal edges
      const d1 = Math.abs(topLeft - bottomRight);
      const d2 = Math.abs(topRight - bottomLeft);
      if (d1 > threshold || d2 > threshold) {
        diagonalEdges++;
      }
    }
  }

  const totalPixels = width! * height!;
  const hPercent = (horizontalEdges / totalPixels * 100).toFixed(2);
  const vPercent = (verticalEdges / totalPixels * 100).toFixed(2);
  const dPercent = (diagonalEdges / totalPixels * 100).toFixed(2);

  console.log(`\n━━━━ Edge Analysis ━━━━`);
  console.log(`Horizontal edges: ${horizontalEdges} (${hPercent}%)`);
  console.log(`Vertical edges:   ${verticalEdges} (${vPercent}%)`);
  console.log(`Diagonal edges:   ${diagonalEdges} (${dPercent}%)`);

  // Grid detection heuristic:
  // High H+V with low diagonal = grid pattern
  // Balanced H+V+D = organic pattern
  const hvTotal = horizontalEdges + verticalEdges;
  const hvdRatio = diagonalEdges > 0 ? hvTotal / diagonalEdges : 999;

  console.log(`\n━━━━ Grid Detection ━━━━`);
  console.log(`H+V to Diagonal ratio: ${hvdRatio.toFixed(2)}`);

  if (hvdRatio > 3) {
    console.log(`⚠️  HIGH GRID LIKELIHOOD - strong horizontal/vertical bias`);
  } else if (hvdRatio > 2) {
    console.log(`⚡ MODERATE GRID - some rectangular tendency`);
  } else {
    console.log(`✅ ORGANIC - balanced edge distribution`);
  }

  // Additional: Check for regular spacing in edges
  // Sample horizontal lines for periodicity
  console.log(`\n━━━━ Periodicity Check ━━━━`);

  const sampleRow = Math.floor(height! / 2);
  let edgePositions: number[] = [];

  for (let x = 1; x < width! - 1; x++) {
    const idx = sampleRow * width! + x;
    const left = grayBuffer[idx - 1];
    const right = grayBuffer[idx + 1];
    if (Math.abs(left - right) > threshold) {
      edgePositions.push(x);
    }
  }

  // Check spacing between edges
  if (edgePositions.length > 10) {
    const spacings: number[] = [];
    for (let i = 1; i < edgePositions.length; i++) {
      spacings.push(edgePositions[i] - edgePositions[i-1]);
    }

    // Find most common spacing
    const spacingCounts: Record<number, number> = {};
    spacings.forEach(s => {
      const bucket = Math.round(s / 4) * 4; // Group by 4-pixel buckets
      spacingCounts[bucket] = (spacingCounts[bucket] || 0) + 1;
    });

    const sortedSpacings = Object.entries(spacingCounts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5);

    console.log(`Edge spacings (middle row sample):`);
    sortedSpacings.forEach(([spacing, count]) => {
      console.log(`  ${spacing}px: ${count} occurrences`);
    });

    // If one spacing dominates, it's likely a grid
    const topSpacingPercent = sortedSpacings[0] ? sortedSpacings[0][1] / spacings.length * 100 : 0;
    if (topSpacingPercent > 30) {
      console.log(`⚠️  REGULAR SPACING DETECTED at ~${sortedSpacings[0][0]}px`);
    } else {
      console.log(`✅ IRREGULAR SPACING - organic pattern`);
    }
  }
}

// Run on provided image
const imagePath = process.argv[2];
if (!imagePath) {
  console.log('Usage: npx ts-node detect-grid.ts <image_path>');
  process.exit(1);
}

detectGridPatterns(imagePath).catch(console.error);
