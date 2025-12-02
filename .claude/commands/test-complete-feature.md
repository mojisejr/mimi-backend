# test-complete-feature

Complete Feature Testing - Comprehensive end-to-end testing for newly implemented features.

## Usage

```
/test-complete-feature [feature-name]        # Test all aspects of implemented feature
/test-complete-feature [feature-name] --deep  # Include performance and security testing
/test-complete-feature --recent              # Test most recently completed feature
```

## Examples

```bash
# Test specific feature
/test-complete-feature user-authentication
/test-complete-feature payment-processing --deep

# Test recent work
/test-complete-feature --recent

# Test complex feature
/test-complete-feature e-commerce-system --deep
```

## Implementation

### Phase 1: Feature Discovery

When `/test-complete-feature [feature-name]` is executed:

1. **Feature Detection**:
   ```bash
   # Search for related files and code
   - Scan src/ for feature-related files
   - Find tests/ directories
   - Identify API endpoints
   - Locate database migrations
   - Check external service integrations
   ```

2. **Task Analysis**:
   ```bash
   # Check GitHub Issues for completed tasks
   gh issue list --label "task" --state closed --search "[feature-name]"

   # Identify related task numbers
   # Parse commit history for feature implementation
   git log --grep="[feature-name]" --oneline
   ```

### Phase 2: Test Plan Generation

**Auto-generates comprehensive test plan:**

```markdown
## 🧪 Test Plan: [feature-name]

### 🔍 Components Identified
- **API Endpoints**: [endpoints found]
- **Database Changes**: [tables/migrations found]
- **External Services**: [services integrated]
- **Background Jobs**: [workers implemented]

### 📋 Test Categories

#### 1. Unit Tests (Already written)
- [ ] Verify all unit tests pass
- [ ] Check test coverage percentage
- [ ] Validate edge case handling

#### 2. Integration Tests
- [ ] API endpoint functionality
- [ ] Database operations
- [ ] External service connectivity
- [ ] Queue system operations

#### 3. End-to-End Tests
- [ ] Complete user workflow
- [ ] Error handling scenarios
- [ ] Performance validation

#### 4. Environment Tests (--deep flag)
- [ ] Load testing
- [ ] Security vulnerability scan
- [ ] Memory usage analysis
```

### Phase 3: Automated Test Execution

**Execute tests in logical order:**

```bash
# 1. Build and Unit Test Validation
echo "🏗️ Building and running unit tests..."
cargo build --release
if [ $? -ne 0 ]; then
  echo "❌ Build failed - stopping tests"
  exit 1
fi

cargo test --lib
echo "✅ Unit tests passed"

# 2. Environment Validation
echo "🔍 Testing environment connectivity..."
/test-env
if [ $? -ne 0 ]; then
  echo "❌ Environment validation failed"
  exit 1
fi

# 3. API Testing
echo "🌐 Testing API endpoints..."
# Start API server
cargo run --bin api &
API_PID=$!
sleep 5

# Test each discovered endpoint
curl -f http://localhost:3000/api/v1/health
# Add more endpoint tests...

kill $API_PID
echo "✅ API tests passed"

# 4. Worker Testing (if applicable)
if [ -f "src/workers/" ]; then
  echo "⚙️ Testing background workers..."
  cargo run --bin worker &
  WORKER_PID=$!
  sleep 3

  # Test worker functionality
  # Add queue job test...

  kill $WORKER_PID
  echo "✅ Worker tests passed"
fi
```

### Phase 4: Performance Testing (--deep flag)

```bash
# 5. Performance Benchmarks
echo "⚡ Running performance tests..."

# API response time testing
for endpoint in "${endpoints[@]}"; do
  echo "Testing $endpoint..."
  start_time=$(date +%s%N)
  curl -s "http://localhost:3000$endpoint" > /dev/null
  end_time=$(date +%s%N)

  response_time=$(( (end_time - start_time) / 1000000 ))
  if [ $response_time -gt 200 ]; then
    echo "⚠️ Slow response: ${response_time}ms for $endpoint"
  else
    echo "✅ Fast response: ${response_time}ms for $endpoint"
  fi
done

# Database query performance
echo "🗄️ Testing database performance..."
# Run query performance tests...

# Memory usage testing
echo "💾 Testing memory usage..."
# Monitor memory during operations...
```

### Phase 5: Security Testing (--deep flag)

```bash
# 6. Security Validation
echo "🔒 Running security tests..."

# Input validation testing
curl -X POST http://localhost:3000/api/v1/test \
  -H "Content-Type: application/json" \
  -d '{"malicious": "<script>alert(1)</script>"}'

# Authentication testing
curl -H "Authorization: Bearer invalid-token" \
  http://localhost:3000/api/v1/protected

# Rate limiting testing
for i in {1..100}; do
  curl -s http://localhost:3000/api/v1/endpoint > /dev/null
done

echo "✅ Security tests completed"
```

### Phase 6: Generate Test Report

**Create comprehensive test report:**

```markdown
# 🧪 Test Report: [feature-name]

## 📊 Test Summary
- **Total Tests**: [count]
- **Passed**: [count] ✓
- **Failed**: [count] ❌
- **Coverage**: [percentage]%
- **Duration**: [time]

## 🔍 Test Results

### Unit Tests
- ✅ All [count] unit tests passed
- 📊 Coverage: [percentage]%

### Integration Tests
- ✅ API endpoints: [count]/[count] passed
- ✅ Database operations: All tests passed
- ✅ External services: [service] connected successfully

### Performance Tests (if --deep)
- ⚡ Average response time: [avg]ms
- 🎯 95th percentile: [p95]ms
- 💾 Peak memory usage: [MB]MB

### Security Tests (if --deep)
- 🔒 Input validation: Passed
- 🔒 Authentication: Passed
- 🔒 Rate limiting: Passed

## 🐛 Issues Found
[if any issues were discovered]

## 🎯 Recommendations
[performance optimization suggestions]
[security improvements]
[additional test coverage areas]

## ✅ Readiness Assessment
**Feature is READY for production**: [Yes/No]
**Confidence Level**: [High/Medium/Low]
```

### Phase 7: Test Result Storage

```bash
# Save test results
mkdir -p .claude/test-results
report_file=".claude/test-results/feature-$(date +%Y%m%d-%H%M%S).md"

# Create GitHub Issue with test results (if failures found)
if [ $failed_tests -gt 0 ]; then
  gh issue create \
    --title "Test Failures: [feature-name]" \
    --label "test-failure" \
    --body "$(cat $report_file)"
fi

# Update task issues with test status
for task in $related_tasks; do
  gh issue comment $task \
    --body "✅ Feature testing completed successfully. See test report for details."
done
```

## Test Categories

### Standard Testing
- **Build Validation**: Code compiles successfully
- **Unit Tests**: All unit tests pass
- **Environment Tests**: External services accessible
- **API Tests**: Endpoints respond correctly
- **Integration Tests**: Components work together

### Deep Testing (--deep flag)
- **Performance Tests**: Response times and load handling
- **Security Tests**: Input validation and authentication
- **Memory Tests**: Memory usage and leaks
- **Load Tests**: Concurrent request handling

## Integration

This command integrates with:
- `/task` - Tests feature created with smart task system
- `/test-env` - Environment validation before testing
- GitHub Issues - Reports test results and failures
- Cargo - Rust testing framework

## Files

- `.claude/test-results/` - Test reports and results
- GitHub Issues - Test failure reports and status updates
- Feature source files - Discovered and tested automatically
- Test files - Located and executed automatically

## Notes

- **Auto-discovers** feature components and generates appropriate tests
- **Environment aware** - validates external services automatically
- **Progressive testing** - standard vs deep testing based on needs
- **Result tracking** - stores results and updates related issues
- **Security focused** - includes security validation for critical features
- **Performance monitoring** - measures and reports performance metrics

The test-complete-feature command provides comprehensive validation that your feature works correctly, performs well, and is secure before deployment.