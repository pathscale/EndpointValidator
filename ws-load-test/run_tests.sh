#!/bin/bash
set -e

# ─────────────────────────────────────────────────────────────────────────────
# Load Testing Script
#
# This script runs load tests on a specified provider and machine type.
# Configure the variables below to customize test scenarios.
# ─────────────────────────────────────────────────────────────────────────────

# Configuration: Choose provider and machine type
PROVIDER="gcr"           # Options: fly, bunny
MACHINE_TYPE="default-8x"  # Fly options: shared-1x, shared-8x, perf-1x, perf-16x

# Test scenarios: pairs of (num-parallel, num-requests)
# Format: "num_parallel:num_requests"
SCENARIOS=(
    "10:25"
    "10:25"
    "20:100"
    "20:100"
    "50:100"
    "50:100"
    "100:100"
    "200:100"
)

# ─────────────────────────────────────────────────────────────────────────────
# Provider-specific configuration
# ─────────────────────────────────────────────────────────────────────────────

case "$PROVIDER" in
    fly)
        PROTOCOL_HEADER_FILE="protocol_header_fly.txt"
        SERVER_URL="wss://nofilterio-be.fly.dev"
        OUTPUT_BASE_DIR="./results/${PROVIDER}-${MACHINE_TYPE}"
        ;;
    bunny)
        PROTOCOL_HEADER_FILE="protocol_header_bunny.txt"
        SERVER_URL="wss://api.nofilter.io"
        OUTPUT_BASE_DIR="./results/${PROVIDER}"
        ;;
    gcr)
        PROTOCOL_HEADER_FILE="protocol_header_gcr.txt"
        SERVER_URL="wss://nofilter-be-image-974817704642.northamerica-northeast1.run.app"
        OUTPUT_BASE_DIR="./results/${PROVIDER}-${MACHINE_TYPE}"
        ;;
    *)
        echo "Error: Unknown provider '$PROVIDER'. Options: fly, bunny"
        exit 1
        ;;
esac

# ─────────────────────────────────────────────────────────────────────────────
# Validation
# ─────────────────────────────────────────────────────────────────────────────

if [[ ! -f "$PROTOCOL_HEADER_FILE" ]]; then
    echo "Error: Protocol header file not found: $PROTOCOL_HEADER_FILE"
    exit 1
fi

mkdir -p "$OUTPUT_BASE_DIR"

# ─────────────────────────────────────────────────────────────────────────────
# Run tests
# ─────────────────────────────────────────────────────────────────────────────

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Load Testing: $PROVIDER - $MACHINE_TYPE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Protocol Header: $PROTOCOL_HEADER_FILE"
echo "Server URL:      $SERVER_URL"
echo "Output Dir:      $OUTPUT_BASE_DIR"
echo "Scenarios:       ${#SCENARIOS[@]}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

total_scenarios=${#SCENARIOS[@]}
current_scenario=0

for scenario in "${SCENARIOS[@]}"; do
    current_scenario=$((current_scenario + 1))
    IFS=':' read -r num_parallel num_requests <<< "$scenario"

    echo "[${current_scenario}/${total_scenarios}] Running: --num-parallel $num_parallel --num-requests $num_requests"

    ws-simple -- \
        --protocol-header-file "$PROTOCOL_HEADER_FILE" \
        --num-parallel "$num_parallel" \
        --num-requests "$num_requests" \
        --server-url "$SERVER_URL" \
        --type connection \
        --store-raw-responses \
        --timeout-ms 3000 \
        -o "$OUTPUT_BASE_DIR"

    echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✓ All tests complete. Results written to: $OUTPUT_BASE_DIR"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
