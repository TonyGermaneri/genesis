#!/usr/bin/env npx ts-node
/**
 * Check for chunk boundary artifacts at 256-pixel intervals
 */

import sharp from 'sharp';

async function checkChunkBoundaries(imagePath: string) {
  console.log(`\n🔍 Checking chunk boundaries in: ${imagePath}\n`);

  const image = sharp(imagePath);
  const meta = await image.metadata();
  const gray = await image.grayscale().raw().toBuffer();

  const width = meta.width!;
  const height = meta.height!;

  // Check for vertical lines at chunk boundaries (every 256 pixels)
  const chunkSize = 256;
  let boundaryEdges = 0;
  let normalEdges = 0;

  for (let y = 100; y < height - 100; y++) {
    for (let x = 1; x < width - 1; x++) {
      const idx = y * width + x;
      const diff = Math.abs(gray[idx-1] - gray[idx+1]);
      if (diff > 15) {
        if (x % chunkSize < 3 || x % chunkSize > chunkSize - 3) {
          boundaryEdges++;
        } else {
          normalEdges++;
        }
      }
    }
  }

  console.log('Edges at chunk boundaries (±3px of 256n):', boundaryEdges);
  console.log('Edges elsewhere:', normalEdges);
  console.log('Boundary edge ratio:', (boundaryEdges / normalEdges * 100).toFixed(2) + '%');
  console.log(boundaryEdges / normalEdges > 0.05 ? '⚠️ CHUNK BOUNDARY ARTIFACTS DETECTED' : '✅ No chunk seams');
}

const imagePath = process.argv[2];
if (!imagePath) {
  console.log('Usage: npx ts-node check-chunks.ts <image_path>');
  process.exit(1);
}

checkChunkBoundaries(imagePath).catch(console.error);
