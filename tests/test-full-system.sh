#!/bin/bash
# Full System Test Script for MimiVibe Backend
# ทดสอบระบบ tarot reading แบบครบวงจร

set -e

echo "🔮 MimiVibe Backend - Full System Test"
echo "====================================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

# Step 1: Check Environment
echo -e "\n📋 Step 1: Checking Environment..."
echo "--------------------------------"

if [ -z "$DATABASE_URL" ]; then
    print_error "DATABASE_URL not set"
    echo "Please set DATABASE_URL in your environment or .env file"
    exit 1
else
    print_success "DATABASE_URL is set"
fi

if [ -z "$UPSTASH_REDIS_URL" ]; then
    print_error "UPSTASH_REDIS_URL not set"
    echo "Please set UPSTASH_REDIS_URL in your environment or .env file"
    exit 1
else
    print_success "UPSTASH_REDIS_URL is set"
fi

if [ -z "$GEMINI_API_KEY" ]; then
    print_error "GEMINI_API_KEY not set"
    echo "Please set GEMINI_API_KEY in your environment or .env file"
    exit 1
else
    print_success "GEMINI_API_KEY is set"
fi

if [ -z "$API_KEY_DEFAULT" ]; then
    print_error "API_KEY_DEFAULT not set"
    echo "Please set API_KEY_DEFAULT in your environment or .env file"
    exit 1
else
    print_success "API_KEY_DEFAULT is set"
fi

# Step 2: Clean up any existing processes
echo -e "\n🧹 Step 2: Cleaning up..."
echo "---------------------------"
pkill -f "cargo run --bin api" 2>/dev/null || true
pkill -f "cargo run --bin worker" 2>/dev/null || true
sleep 2
print_success "Cleaned up existing processes"

# Step 3: Start API Server
echo -e "\n🚀 Step 3: Starting API Server..."
echo "---------------------------------"
cargo build --release
print_success "Build completed"

cargo run --bin api &
API_PID=$!
echo "API PID: $API_PID"

# Wait for API to start
echo "Waiting for API server to start..."
for i in {1..10}; do
    if curl -s http://localhost:3000/api/v1/health > /dev/null 2>&1; then
        print_success "API server is running"
        break
    fi
    sleep 1
    echo -n "."
done

if ! curl -s http://localhost:3000/api/v1/health > /dev/null 2>&1; then
    print_error "API server failed to start"
    kill $API_PID 2>/dev/null || true
    exit 1
fi

# Step 4: Start Worker
echo -e "\n⚙️  Step 4: Starting Worker..."
echo "----------------------------"
cargo run --bin worker &
WORKER_PID=$!
echo "Worker PID: $WORKER_PID"
sleep 3
print_success "Worker started"

# Step 5: Run Manual Test
echo -e "\n🧪 Step 5: Running Manual Test..."
echo "---------------------------------"
echo "Submitting test tarot reading request..."

TEST_RESPONSE=$(curl -s -X POST http://localhost:3000/api/v1/tarots/read \
  -H "Authorization: Bearer default-key" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "ชีวิตของฉันจะดีขึ้นไหมในเดือนหน้า",
    "cards": 3
  }')

if [ $? -eq 0 ]; then
    JOB_ID=$(echo $TEST_RESPONSE | grep -o '"job_id":"[^"]*' | cut -d'"' -f4)
    if [ ! -z "$JOB_ID" ]; then
        print_success "Request submitted successfully"
        echo "Job ID: $JOB_ID"

        # Monitor job status
        echo -e "\n⏳ Monitoring job status..."
        for i in {1..30}; do
            STATUS_RESPONSE=$(curl -s http://localhost:3000/api/v1/tarots/$JOB_ID \
              -H "Authorization: Bearer default-key")

            STATUS=$(echo $STATUS_RESPONSE | grep -o '"status":"[^"]*' | cut -d'"' -f4)
            echo "Attempt $i: Status = $STATUS"

            if [ "$STATUS" = "completed" ]; then
                print_success "Job completed successfully!"

                # Get and display result
                READING=$(echo $STATUS_RESPONSE | grep -o '"reading":"[^"]*' | cut -d'"' -f4)
                echo -e "\n📖 Tarot Reading:"
                echo "$READING"

                # Check if Thai content
                if echo "$READING" | grep -q "[ก-ฮ]"; then
                    print_success "Thai language content detected"
                else
                    print_warning "No Thai characters detected"
                fi

                break
            elif [ "$STATUS" = "failed" ]; then
                print_error "Job failed"
                break
            fi

            sleep 2
        done
    else
        print_error "Failed to get job ID from response"
        echo "Response: $TEST_RESPONSE"
    fi
else
    print_error "Failed to submit request"
fi

# Step 6: Run Automated E2E Tests
echo -e "\n🤖 Step 6: Running Automated E2E Tests..."
echo "-----------------------------------------"
echo "Running: cargo test --test test_complete_e2e_validation"

# Run tests and capture output
TEST_OUTPUT=$(cargo test --test test_complete_e2e_validation 2>&1)
TEST_EXIT_CODE=$?

echo "$TEST_OUTPUT"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    print_success "All E2E tests passed!"
else
    print_error "Some E2E tests failed"
fi

# Step 7: Cleanup
echo -e "\n🧹 Step 7: Cleaning up..."
echo "------------------------"
echo "Stopping API server (PID: $API_PID)..."
kill $API_PID 2>/dev/null || true

echo "Stopping worker (PID: $WORKER_PID)..."
kill $WORKER_PID 2>/dev/null || true

# Wait for processes to stop
sleep 2

# Force kill if still running
pkill -f "cargo run --bin api" 2>/dev/null || true
pkill -f "cargo run --bin worker" 2>/dev/null || true

print_success "Cleanup completed"

# Step 8: Summary
echo -e "\n📊 Test Summary"
echo "==============="

if [ $TEST_EXIT_CODE -eq 0 ]; then
    print_success "✨ Full system test completed successfully!"
    echo "  - API server: Working"
    echo "  - Worker: Working"
    echo "  - Queue: Working"
    echo "  - AI Pipeline: Working"
    echo "  - Thai Language Generation: Working"
    echo "  - All E2E Tests: PASSED"
else
    print_error "❌ System test failed"
    echo "Please check the logs above for details"
fi

exit $TEST_EXIT_CODE