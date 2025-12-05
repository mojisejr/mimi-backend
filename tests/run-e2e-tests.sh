#!/bin/bash
# Run E2E Tests Only (Assuming API and Worker are already running)

echo "🧪 Running E2E Tests Only"
echo "========================"

# Check if API is running
if ! curl -s http://localhost:3000/api/v1/health > /dev/null 2>&1; then
    echo "❌ API server is not running on port 3000"
    echo "Please start it with: cargo run --bin api"
    exit 1
fi

echo "✅ API server is running"

# Run the tests
echo "🚀 Running all E2E tests..."
echo "============================"

# Run each test with clear output
tests=(
    "test_complete_tarot_reading_flow"
    "test_five_card_reading_flow"
    "test_multiple_job_processing"
    "test_error_handling_flow"
    "test_processing_time"
)

PASSED=0
FAILED=0

for test in "${tests[@]}"; do
    echo -e "\n📝 Running: $test"
    echo "----------------------------------------"

    if cargo test --test test_complete_e2e_validation $test; then
        echo -e "✅ $test PASSED\n"
        ((PASSED++))
    else
        echo -e "❌ $test FAILED\n"
        ((FAILED++))
    fi
done

# Summary
echo "============================"
echo "📊 Test Summary:"
echo "============================"
echo "✅ Passed: $PASSED"
echo "❌ Failed: $FAILED"
echo "📊 Total: $((PASSED + FAILED))"

if [ $FAILED -eq 0 ]; then
    echo -e "\n🎉 All tests passed! System is working correctly!"
    exit 0
else
    echo -e "\n⚠️  Some tests failed. Please check the logs above."
    exit 1
fi