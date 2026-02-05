#!/usr/bin/env npx ts-node

/**
 * Training Data Asset Indexer
 *
 * Parses the OpenGameArt training data JSONL files and creates a searchable
 * index of assets by type (tileset, character, terrain, etc.)
 *
 * Usage:
 *   npx ts-node scripts/index-training-data.ts [options]
 */

import { readFileSync, writeFileSync, existsSync, readdirSync } from 'fs';
import path from 'path';

// ============================================================================
// Types
// ============================================================================

interface AssetEntry {
  url: string;
  title: string;
  author: string;
  author_url: string;
  post_date: string;
  art_type: string;
  tags: string[];
  licenses: string[];
  collections: string[];
  preview_images: string[];
  description: string;
  files: Array<{
    url: string;
    name: string;
    size: string | null;
  }>;
}

interface AssetIndex {
  totalAssets: number;
  byCategory: Record<string, AssetEntry[]>;
  byTag: Record<string, AssetEntry[]>;
  terrainAssets: AssetEntry[];
  characterAssets: AssetEntry[];
  tilesetAssets: AssetEntry[];
  animationAssets: AssetEntry[];
}

// ============================================================================
// Category Detection
// ============================================================================

const TERRAIN_KEYWORDS = [
  'terrain', 'ground', 'grass', 'dirt', 'sand', 'stone', 'water', 'snow',
  'forest', 'desert', 'mountain', 'swamp', 'biome', 'nature', 'landscape',
  'outdoor', 'environment'
];

const CHARACTER_KEYWORDS = [
  'character', 'player', 'hero', 'sprite', 'human', 'npc', 'enemy', 'monster',
  'creature', 'person', 'figure', 'avatar', 'unit', 'walk', 'run', 'idle'
];

const TILESET_KEYWORDS = [
  'tileset', 'tiles', 'tile', 'tilemap', 'map', 'rpg maker', 'lpc',
  'top-down', 'topdown', 'orthogonal', 'isometric'
];

const ANIMATION_KEYWORDS = [
  'animation', 'animated', 'spritesheet', 'sprite sheet', 'frames',
  'walk cycle', 'run cycle', 'attack', 'idle'
];

function matchesKeywords(entry: AssetEntry, keywords: string[]): boolean {
  const searchText = [
    entry.title,
    entry.description,
    ...entry.tags,
    ...entry.collections
  ].join(' ').toLowerCase();

  return keywords.some(kw => searchText.includes(kw.toLowerCase()));
}

// ============================================================================
// JSONL Parsing
// ============================================================================

function parseJsonl(filePath: string): AssetEntry[] {
  const content = readFileSync(filePath, 'utf-8');
  const entries: AssetEntry[] = [];

  // JSONL has one JSON object per line, but they might not have newlines
  // Try to split by }{ pattern
  const jsonObjects = content.split(/\}\s*\{/).map((chunk, i, arr) => {
    if (i === 0) return chunk + '}';
    if (i === arr.length - 1) return '{' + chunk;
    return '{' + chunk + '}';
  });

  for (const jsonStr of jsonObjects) {
    try {
      const entry = JSON.parse(jsonStr);
      entries.push(entry);
    } catch (e) {
      // Skip malformed entries
    }
  }

  return entries;
}

// ============================================================================
// Indexing
// ============================================================================

function buildIndex(entries: AssetEntry[]): AssetIndex {
  const index: AssetIndex = {
    totalAssets: entries.length,
    byCategory: {},
    byTag: {},
    terrainAssets: [],
    characterAssets: [],
    tilesetAssets: [],
    animationAssets: []
  };

  for (const entry of entries) {
    // Index by art type
    const artType = entry.art_type || 'Unknown';
    if (!index.byCategory[artType]) {
      index.byCategory[artType] = [];
    }
    index.byCategory[artType].push(entry);

    // Index by tags
    for (const tag of entry.tags || []) {
      const normalizedTag = tag.toLowerCase();
      if (!index.byTag[normalizedTag]) {
        index.byTag[normalizedTag] = [];
      }
      index.byTag[normalizedTag].push(entry);
    }

    // Categorize by content type
    if (matchesKeywords(entry, TERRAIN_KEYWORDS)) {
      index.terrainAssets.push(entry);
    }
    if (matchesKeywords(entry, CHARACTER_KEYWORDS)) {
      index.characterAssets.push(entry);
    }
    if (matchesKeywords(entry, TILESET_KEYWORDS)) {
      index.tilesetAssets.push(entry);
    }
    if (matchesKeywords(entry, ANIMATION_KEYWORDS)) {
      index.animationAssets.push(entry);
    }
  }

  return index;
}

// ============================================================================
// Local File Matching
// ============================================================================

function findLocalFiles(trainingDir: string): Map<string, string[]> {
  const fileMap = new Map<string, string[]>();

  const artDirs = readdirSync(trainingDir).filter(d =>
    d.startsWith('2D_Art_') || d.startsWith('Texture_')
  );

  for (const dir of artDirs) {
    const fullPath = path.join(trainingDir, dir);
    try {
      const files = readdirSync(fullPath);
      for (const file of files) {
        const baseName = path.parse(file).name.toLowerCase();
        if (!fileMap.has(baseName)) {
          fileMap.set(baseName, []);
        }
        fileMap.get(baseName)!.push(path.join(fullPath, file));
      }
    } catch (e) {
      // Skip inaccessible directories
    }
  }

  return fileMap;
}

// ============================================================================
// Report Generation
// ============================================================================

function generateReport(index: AssetIndex): string {
  const lines: string[] = [
    '# Training Data Asset Index',
    '',
    `Total Assets: ${index.totalAssets}`,
    '',
    '## By Category',
    ''
  ];

  for (const [category, assets] of Object.entries(index.byCategory)) {
    lines.push(`- **${category}**: ${assets.length} assets`);
  }

  lines.push('', '## Content Types', '');
  lines.push(`- **Terrain/Environment**: ${index.terrainAssets.length} assets`);
  lines.push(`- **Characters/Sprites**: ${index.characterAssets.length} assets`);
  lines.push(`- **Tilesets**: ${index.tilesetAssets.length} assets`);
  lines.push(`- **Animations**: ${index.animationAssets.length} assets`);

  lines.push('', '## Top Tags', '');
  const sortedTags = Object.entries(index.byTag)
    .sort((a, b) => b[1].length - a[1].length)
    .slice(0, 30);

  for (const [tag, assets] of sortedTags) {
    lines.push(`- ${tag}: ${assets.length}`);
  }

  lines.push('', '## Sample Terrain Assets', '');
  for (const asset of index.terrainAssets.slice(0, 10)) {
    lines.push(`### ${asset.title}`);
    lines.push(`- Author: ${asset.author}`);
    lines.push(`- Tags: ${asset.tags.join(', ')}`);
    lines.push(`- License: ${asset.licenses.join(', ')}`);
    lines.push('');
  }

  return lines.join('\n');
}

// ============================================================================
// CLI
// ============================================================================

async function main(): Promise<void> {
  const trainingDir = '/Users/tonygermaneri/gh/game_assets/training';

  console.log('📚 Indexing training data...\n');

  // Parse JSONL files
  const jsonlFiles = ['2D_Art.jsonl', 'Texture.jsonl'];
  let allEntries: AssetEntry[] = [];

  for (const file of jsonlFiles) {
    const filePath = path.join(trainingDir, file);
    if (existsSync(filePath)) {
      console.log(`  Parsing ${file}...`);
      const entries = parseJsonl(filePath);
      console.log(`    Found ${entries.length} entries`);
      allEntries = allEntries.concat(entries);
    }
  }

  console.log(`\n📊 Total entries: ${allEntries.length}`);

  // Build index
  const index = buildIndex(allEntries);

  // Find local files
  console.log('\n📁 Scanning local files...');
  const localFiles = findLocalFiles(trainingDir);
  console.log(`  Found ${localFiles.size} unique file names`);

  // Print summary
  console.log('\n' + '='.repeat(60));
  console.log('📋 INDEX SUMMARY');
  console.log('='.repeat(60));

  console.log(`\n📦 By Art Type:`);
  for (const [type, assets] of Object.entries(index.byCategory)) {
    console.log(`  ${type}: ${assets.length}`);
  }

  console.log(`\n🏷️ Content Categories:`);
  console.log(`  Terrain/Environment: ${index.terrainAssets.length}`);
  console.log(`  Characters/Sprites: ${index.characterAssets.length}`);
  console.log(`  Tilesets: ${index.tilesetAssets.length}`);
  console.log(`  Animations: ${index.animationAssets.length}`);

  console.log(`\n🔝 Top 20 Tags:`);
  const topTags = Object.entries(index.byTag)
    .sort((a, b) => b[1].length - a[1].length)
    .slice(0, 20);
  for (const [tag, assets] of topTags) {
    console.log(`  ${tag}: ${assets.length}`);
  }

  // Generate and save report
  const report = generateReport(index);
  const reportPath = path.join(trainingDir, 'asset_index.md');
  writeFileSync(reportPath, report);
  console.log(`\n💾 Report saved: ${reportPath}`);

  // Save JSON index
  const jsonIndex = {
    totalAssets: index.totalAssets,
    categoryCounts: Object.fromEntries(
      Object.entries(index.byCategory).map(([k, v]) => [k, v.length])
    ),
    tagCounts: Object.fromEntries(
      Object.entries(index.byTag).map(([k, v]) => [k, v.length])
    ),
    terrainCount: index.terrainAssets.length,
    characterCount: index.characterAssets.length,
    tilesetCount: index.tilesetAssets.length,
    animationCount: index.animationAssets.length,
    // Include first few of each for quick reference
    sampleTerrain: index.terrainAssets.slice(0, 5).map(e => ({
      title: e.title,
      tags: e.tags,
      preview: e.preview_images[0]
    })),
    sampleCharacters: index.characterAssets.slice(0, 5).map(e => ({
      title: e.title,
      tags: e.tags,
      preview: e.preview_images[0]
    }))
  };

  const jsonPath = path.join(trainingDir, 'asset_index.json');
  writeFileSync(jsonPath, JSON.stringify(jsonIndex, null, 2));
  console.log(`💾 JSON index saved: ${jsonPath}`);
}

main().catch(console.error);
