#!/bin/bash
# Build and compress game assets
#
# Usage: ./scripts/build_assets.sh [ASSET_DIR] [OUTPUT_DIR]
#
# This script processes assets from the source directory and outputs
# them to the target directory with compression and manifest generation.

set -e

ASSET_DIR="${1:-assets}"
OUTPUT_DIR="${2:-target/assets}"
MANIFEST_FILE="$OUTPUT_DIR/manifest.json"

echo "╔══════════════════════════════════════════╗"
echo "║        Genesis Asset Builder             ║"
echo "╠══════════════════════════════════════════╣"
echo "║ Source: $ASSET_DIR"
echo "║ Output: $OUTPUT_DIR"
echo "╚══════════════════════════════════════════╝"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Initialize manifest
cat > "$MANIFEST_FILE" << 'EOF'
{
  "version": 1,
  "assets": {
EOF

FIRST=true

# Function to get asset type from extension
get_asset_type() {
    local ext="${1##*.}"
    case "$ext" in
        png|jpg|jpeg|webp|bmp|tga)
            echo "Texture"
            ;;
        wav|ogg|mp3|flac)
            echo "Sound"
            ;;
        ttf|otf|woff|woff2)
            echo "Font"
            ;;
        wgsl|glsl|spv|hlsl)
            echo "Shader"
            ;;
        json)
            # Check if it's a locale file
            if [[ "$1" == *"/locales/"* ]]; then
                echo "Localization"
            else
                echo "Data"
            fi
            ;;
        ron|toml|yaml|yml|xml)
            echo "Data"
            ;;
        *)
            echo "Data"
            ;;
    esac
}

# Function to calculate file hash
get_hash() {
    if command -v sha256sum &> /dev/null; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        echo "no_hash"
    fi
}

# Process assets
if [ -d "$ASSET_DIR" ]; then
    find "$ASSET_DIR" -type f | while read -r file; do
        # Get relative path
        rel_path="${file#$ASSET_DIR/}"

        # Get asset ID (path without extension)
        asset_id="${rel_path%.*}"

        # Get asset info
        asset_type=$(get_asset_type "$file")
        file_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo 0)
        file_hash=$(get_hash "$file")

        # Determine if we should compress (files > 10KB)
        compressed="false"
        output_file="$OUTPUT_DIR/$rel_path"

        mkdir -p "$(dirname "$output_file")"

        if [ "$file_size" -gt 10240 ] && command -v lz4 &> /dev/null; then
            # Compress with LZ4
            lz4 -9 -f "$file" "$output_file.lz4" 2>/dev/null && {
                compressed="true"
                output_file="$output_file.lz4"
            } || {
                # Fallback to copy if compression fails
                cp "$file" "$output_file"
            }
        else
            # Copy without compression
            cp "$file" "$output_file"
        fi

        # Add to manifest
        if [ "$FIRST" = true ]; then
            FIRST=false
        else
            echo "," >> "$MANIFEST_FILE"
        fi

        cat >> "$MANIFEST_FILE" << EOF
    "$asset_id": {
      "path": "$rel_path",
      "asset_type": "$asset_type",
      "hash": "$file_hash",
      "size": $file_size,
      "compressed": $compressed
    }
EOF

        echo "  ✓ $asset_id ($asset_type, ${file_size} bytes)"
    done
fi

# Close manifest JSON
cat >> "$MANIFEST_FILE" << 'EOF'

  }
}
EOF

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║            Build Complete                ║"
echo "╚══════════════════════════════════════════╝"
echo ""
echo "Manifest written to: $MANIFEST_FILE"

# Count assets
if command -v jq &> /dev/null; then
    ASSET_COUNT=$(jq '.assets | length' "$MANIFEST_FILE")
    echo "Total assets: $ASSET_COUNT"
fi
