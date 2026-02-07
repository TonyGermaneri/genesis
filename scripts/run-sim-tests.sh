#!/bin/bash
# Run terrain simulation tests with pure colors mode
# Usage: ./run-sim-tests.sh [test_name]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
GENESIS="$ROOT_DIR/target/release/genesis"
MACROS_DIR="$ROOT_DIR/macros"
SCREENSHOTS_DIR="$ROOT_DIR/screenshots"

# Build release if needed
if [ ! -f "$GENESIS" ]; then
    echo "🔨 Building genesis (release)..."
    cd "$ROOT_DIR"
    cargo build --release
fi

# Create screenshots directory
mkdir -p "$SCREENSHOTS_DIR"

# Function to run a single test
run_test() {
    local test_name="$1"
    local macro_file="$MACROS_DIR/${test_name}.json"

    if [ ! -f "$macro_file" ]; then
        echo "❌ Macro file not found: $macro_file"
        return 1
    fi

    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "🧪 Running test: $test_name"
    echo "═══════════════════════════════════════════════════════════════════"

    # Run with pure colors mode
    "$GENESIS" --pure-colors --macro-file "$macro_file"

    echo "✅ Test completed: $test_name"
}

# Function to analyze test screenshots
analyze_test() {
    local test_prefix="$1"

    echo ""
    echo "📊 Analyzing screenshots for: $test_prefix"
    echo "───────────────────────────────────────────────────────────────────"

    for screenshot in "$SCREENSHOTS_DIR/${test_prefix}"*.png; do
        if [ -f "$screenshot" ]; then
            echo ""
            echo "📸 $(basename "$screenshot"):"
            cd "$SCRIPT_DIR"
            npx ts-node analyze-colors.ts "$screenshot" 2>/dev/null | head -40
        fi
    done
}

# Function to compare screenshot hashes
compare_screenshots() {
    local pattern="$1"

    echo ""
    echo "🔍 Comparing screenshot hashes for: $pattern"
    echo "───────────────────────────────────────────────────────────────────"

    local prev_hash=""
    local prev_file=""
    local changed=0
    local total=0

    # Use explicit glob pattern with full path
    for screenshot in "$SCREENSHOTS_DIR"/${pattern}*.png; do
        if [ -f "$screenshot" ]; then
            total=$((total + 1))
            current_hash=$(shasum -a 256 "$screenshot" | cut -d' ' -f1)
            current_file=$(basename "$screenshot")

            if [ -n "$prev_hash" ]; then
                if [ "$current_hash" != "$prev_hash" ]; then
                    echo "  ✅ $prev_file → $current_file: CHANGED"
                    changed=$((changed + 1))
                else
                    echo "  ⚠️  $prev_file → $current_file: IDENTICAL"
                fi
            else
                echo "  📸 $current_file: (baseline)"
            fi

            prev_hash="$current_hash"
            prev_file="$current_file"
        fi
    done

    echo ""
    if [ $total -eq 0 ]; then
        echo "  ⚠️  No screenshots found matching pattern: ${pattern}*"
    elif [ $changed -gt 0 ]; then
        echo "  Result: $changed changes detected across $total screenshots"
        echo "  ✅ SIMULATION IS ACTIVE"
    else
        echo "  Result: No changes detected across $total screenshots"
        echo "  ❌ SIMULATION MAY NOT BE RUNNING"
    fi
}

# Main
echo "🌍 Terrain Simulation Test Suite"
echo "═══════════════════════════════════════════════════════════════════"

if [ -n "$1" ]; then
    # Run specific test
    run_test "$1"

    # Derive the screenshot prefix from the macro file
    # The macros use different naming conventions, so check the JSON
    MACRO_FILE="$MACROS_DIR/${1}.json"
    if [ -f "$MACRO_FILE" ]; then
        # Extract first screenshot filename to derive prefix
        FIRST_SCREENSHOT=$(grep -o '"filename": "[^"]*"' "$MACRO_FILE" | head -1 | sed 's/"filename": "//;s/"//')
        if [ -n "$FIRST_SCREENSHOT" ]; then
            # Get prefix before _t (e.g., sim_test from sim_test_t0.png)
            PREFIX=$(echo "$FIRST_SCREENSHOT" | sed 's/_t[0-9]*\.png$//')
            echo ""
            analyze_test "$PREFIX"
            compare_screenshots "$PREFIX"
        fi
    fi
else
    # Run all tests
    echo "Available tests:"
    echo "  - test_simulation_active"
    echo "  - test_erosion"
    echo "  - test_water_cycle"
    echo "  - test_thermal_erosion"
    echo "  - test_vegetation"
    echo ""
    echo "Usage: $0 <test_name>"
    echo "Example: $0 test_simulation_active"
fi
