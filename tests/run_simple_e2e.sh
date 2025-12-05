#!/bin/bash

# Simple E2E Test Runner for Tarot Reading Flow
# This runs a single comprehensive test

echo "🔮 Running Complete Tarot Reading Flow Test"
echo "============================================"

# Load environment
if [ -f .env.local ]; then
    export $(cat .env.local | grep -v '^#' | xargs)
fi

echo "📝 Starting test..."

# Run the test with cargo
cargo test --test e2e_complete_tarot_flow complete_tarot_reading_flow_demo -- --nocapture

echo
echo "✅ Test completed!"