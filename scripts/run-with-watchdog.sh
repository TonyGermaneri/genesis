#!/bin/bash
# Watchdog wrapper for running Genesis with timeout protection
# Kills the process if it doesn't exit within the specified timeout

TIMEOUT_SECONDS=${1:-60}  # Default 60 seconds
shift  # Remove timeout from args

GAME_CMD="$@"

if [ -z "$GAME_CMD" ]; then
    echo "Usage: $0 [timeout_seconds] <command>"
    echo "Example: $0 30 ./target/release/genesis --macro 'newgame; wait 2000; screenshot /tmp/test.png; quit'"
    exit 1
fi

echo "🐕 Watchdog: Starting with ${TIMEOUT_SECONDS}s timeout"
echo "🎮 Command: $GAME_CMD"

# Start the game in background
eval "$GAME_CMD" &
GAME_PID=$!

echo "🎮 Game PID: $GAME_PID"

# Monitor the process
START_TIME=$(date +%s)
while kill -0 $GAME_PID 2>/dev/null; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))

    if [ $ELAPSED -ge $TIMEOUT_SECONDS ]; then
        echo ""
        echo "⏰ Watchdog: Timeout reached (${ELAPSED}s >= ${TIMEOUT_SECONDS}s)"
        echo "🔪 Killing process $GAME_PID..."
        kill -9 $GAME_PID 2>/dev/null
        wait $GAME_PID 2>/dev/null
        echo "💀 Process killed"
        exit 124  # Same exit code as GNU timeout
    fi

    # Show progress every 5 seconds
    if [ $((ELAPSED % 5)) -eq 0 ] && [ $ELAPSED -gt 0 ]; then
        printf "\r🐕 Watchdog: %ds / %ds elapsed..." $ELAPSED $TIMEOUT_SECONDS
    fi

    sleep 0.5
done

wait $GAME_PID
EXIT_CODE=$?
CURRENT_TIME=$(date +%s)
ELAPSED=$((CURRENT_TIME - START_TIME))

echo ""
echo "✅ Game exited normally after ${ELAPSED}s (exit code: $EXIT_CODE)"
exit $EXIT_CODE
