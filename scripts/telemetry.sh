#!/bin/bash
# scripts/telemetry.sh
# Collect and report build metrics for CI telemetry
#
# Usage: ./scripts/telemetry.sh [--json]
#
# Output: Build metrics including time, binary size, and dependencies

set -e

OUTPUT_FORMAT="${1:-text}"
BINARY_NAME="genesis"

# Start timing
BUILD_START=$(date +%s)

# Build release
echo "Building release binary..." >&2
cargo build --workspace --release 2>&1 | tee build.log >&2

# End timing
BUILD_END=$(date +%s)
BUILD_TIME=$((BUILD_END - BUILD_START))

# Get binary size
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    BINARY_SIZE=$(stat -f%z "target/release/${BINARY_NAME}" 2>/dev/null || echo 0)
else
    # Linux
    BINARY_SIZE=$(stat -c%s "target/release/${BINARY_NAME}" 2>/dev/null || echo 0)
fi

# Human-readable size
if [ "$BINARY_SIZE" -gt 0 ]; then
    if [ "$BINARY_SIZE" -gt 1048576 ]; then
        BINARY_SIZE_HUMAN="$((BINARY_SIZE / 1048576)) MB"
    elif [ "$BINARY_SIZE" -gt 1024 ]; then
        BINARY_SIZE_HUMAN="$((BINARY_SIZE / 1024)) KB"
    else
        BINARY_SIZE_HUMAN="${BINARY_SIZE} B"
    fi
else
    BINARY_SIZE_HUMAN="N/A"
fi

# Count dependencies
DEP_COUNT=$(cargo tree --workspace --prefix none 2>/dev/null | sort -u | wc -l | tr -d ' ')

# Count lines of code (if tokei is available)
if command -v tokei &> /dev/null; then
    LOC=$(tokei --output json 2>/dev/null | jq '.Rust.code // 0' 2>/dev/null || echo 0)
else
    LOC=$(find crates -name '*.rs' -exec cat {} + 2>/dev/null | wc -l | tr -d ' ')
fi

# Get commit info
COMMIT_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
COMMIT_SHORT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

# Get timestamp
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Output based on format
if [[ "$OUTPUT_FORMAT" == "--json" ]]; then
    cat <<EOF
{
  "build_time_seconds": ${BUILD_TIME},
  "binary_size_bytes": ${BINARY_SIZE},
  "dependency_count": ${DEP_COUNT},
  "lines_of_code": ${LOC},
  "timestamp": "${TIMESTAMP}",
  "commit": "${COMMIT_SHA}",
  "commit_short": "${COMMIT_SHORT}",
  "branch": "${BRANCH}"
}
EOF
else
    echo ""
    echo "╔══════════════════════════════════════════╗"
    echo "║         Build Telemetry Report           ║"
    echo "╠══════════════════════════════════════════╣"
    printf "║ %-18s │ %18s ║\n" "Build Time" "${BUILD_TIME}s"
    printf "║ %-18s │ %18s ║\n" "Binary Size" "${BINARY_SIZE_HUMAN}"
    printf "║ %-18s │ %18s ║\n" "Dependencies" "${DEP_COUNT}"
    printf "║ %-18s │ %18s ║\n" "Lines of Code" "${LOC}"
    printf "║ %-18s │ %18s ║\n" "Commit" "${COMMIT_SHORT}"
    printf "║ %-18s │ %18s ║\n" "Branch" "${BRANCH}"
    echo "╚══════════════════════════════════════════╝"
fi
