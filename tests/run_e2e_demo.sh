#!/bin/bash

# E2E Tarot Reading Flow Demo Runner
# This script runs the complete end-to-end test and displays results

set -e

echo "🔮 MimiVibe Backend - Complete Tarot Reading Flow Demo"
echo "======================================================"
echo

# Check if environment is set up
if [ ! -f .env.local ]; then
    echo "⚠️  .env.local not found. Creating from template..."
    cp .env.example .env.local
    echo "📝 Please edit .env.local with your API keys and run again."
    exit 1
fi

# Check database connection
echo "🔍 Checking database connection..."
if ! cargo run --bin db-check 2>/dev/null; then
    echo "⚠️  Database check failed. Please ensure PostgreSQL is running."
    exit 1
fi

# Check Redis connection (optional for demo)
echo "🔍 Checking Redis connection..."
if redis-cli ping >/dev/null 2>&1; then
    echo "✅ Redis is connected"
else
    echo "⚠️  Redis not connected - using InMemoryQueue for demo"
fi

echo
echo "🚀 Starting End-to-End Tests..."
echo "==============================="

# Run the complete flow test
echo
echo "Test 1: Complete Tarot Reading Flow"
echo "-----------------------------------"
cargo test --test e2e_complete_tarot_flow complete_tarot_reading_flow_demo -- --nocapture

echo
echo "Test 2: Detailed Agent Pipeline Analysis"
echo "----------------------------------------"
cargo test --test agent_pipeline_detailed detailed_agent_pipeline_analysis -- --nocapture

echo
echo "Test 3: Database State Transitions"
echo "---------------------------------"
cargo test --test agent_pipeline_detailed test_database_job_state_transitions -- --nocapture

echo
echo "✅ All tests completed!"
echo
echo "📊 Test Summary:"
echo "==============="
echo "✅ API receives question and creates job"
echo "✅ Queue system processes job"
echo "✅ Worker executes all 3 agents"
echo "✅ Question Filter validates input"
echo "✅ Question Analyzer extracts context"
echo "✅ Reading Agent generates Thai tarot reading"
echo "✅ Database stores complete result"
echo "✅ Status updates correctly (queued → processing → completed)"
echo "✅ API returns final reading"
echo
echo "🎉 The complete tarot reading system is working correctly!"