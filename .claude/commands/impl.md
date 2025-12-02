# impl

Implementation Workflow - Execute GitHub issue implementation.

## Usage

```
/impl [issue-number] [optional message]
```

## Examples

```bash
/impl 123                    # Implement issue #123
/impl 123 with extra context # Implement with additional context
/impl 456                    # Implement issue #456
```

## Implementation

### Pre-Implementation Validation

1. **Check Dependencies**:
   - Validate GitHub CLI (`gh`) availability
   - Verify Git tools are available

2. **Validate Issue**:
   - Check issue exists and is open
   - Verify issue has `task` label
   - Extract issue title and metadata

3. **Validate Environment**:
   - Ensure clean git working directory
   - Verify we're in a git repository
   - **CRITICAL**: Verify we are NOT on main branch - abort if on main

### Implementation Steps

1. **FORCE Staging Branch Checkout**:
   ```bash
   # HARD ENFORCE: staging ONLY - never checkout main
   git fetch origin staging --update-head-ok
   git checkout -B staging origin/staging --force
   git pull origin staging

   # VERIFY we are on staging branch
   if [ "$(git branch --show-current)" != "staging" ]; then
     echo "ERROR: Failed to switch to staging branch. Current branch: $(git branch --show-current)"
     exit 1
   fi
   echo "✅ CONFIRMED: Now on staging branch"
   ```

2. **Create Feature Branch**:
   ```bash
   git checkout -b feature/task-[issue-number]-[description]
   ```
   - Extract description from issue title
   - Use naming convention: `feature/task-{issue}-{description}`

3. **Step 0: Write Tests First (Red Phase)** ⚠️ MANDATORY:
   ```bash
   # Create test files BEFORE implementing code
   # Tests should fail initially (Red phase)
   cargo test
   # Expected: Tests fail (no implementation yet)
   ```
   - Write comprehensive unit tests for the new functionality
   - Write integration tests for API endpoints or service integrations
   - Tests document the expected behavior before code exists
   - This ensures Test-Driven Development (TDD) workflow

4. **Implementation Execution**:
   - Agent (Claude) implements directly using code editing tools
   - Execute all implementation steps automatically
   - Handle all validation requirements
   - Create commit with proper format
   - Push branch to remote (NO PR creation)

## Validation Requirements (100% required):
   ```bash
   ✅ Test must be written BEFORE code implementation (Red Phase)
   ✅ Test coverage must be comprehensive for new/modified code
   ✅ Tests must PASS (Green Phase complete)
   cargo build --release          # Build validation
   cargo clippy -- -D warnings    # Lint validation
   cargo check                    # Type check validation
   cargo test                     # Test validation (MANDATORY)
   ```

## 🔴🟢🔵 Red-Green-Refactor Cycle (TDD)

The Red-Green-Refactor cycle is the core of Test-Driven Development:

### 🔴 Red Phase (Tests First)
- **Write failing tests** for the functionality you want to implement
- Tests document the expected behavior
- Run tests: `cargo test` → tests FAIL (because code doesn't exist yet)
- Example:
  ```rust
  // tests/question_filter_tests.rs
  #[test]
  fn test_empty_question_rejected() {
    let result = filter_question("");
    assert!(result.is_err());
  }
  
  #[test]
  fn test_valid_question_accepted() {
    let result = filter_question("What is my future?");
    assert!(result.is_ok());
  }
  ```

### 🟢 Green Phase (Minimal Implementation)
- **Write minimal code** to make the failing tests pass
- Don't implement extra features yet
- Focus only on passing the tests you wrote
- Run tests: `cargo test` → tests PASS
- Example:
  ```rust
  // src/agents/question_filter.rs
  pub fn filter_question(q: &str) -> Result<String, FilterError> {
    if q.is_empty() {
      return Err(FilterError::EmptyQuestion);
    }
    Ok(q.to_string())
  }
  ```

### 🔵 Refactor Phase (Improve Code)
- **Refactor the code** for clarity, performance, and maintainability
- Keep tests passing while improving code quality
- Run tests: `cargo test` → tests still PASS
- Run linter: `cargo clippy -- -D warnings` → zero warnings
- Run formatter: `cargo fmt` → consistent style
- Example improvements:
  ```rust
  pub fn filter_question(q: &str) -> Result<String, FilterError> {
    q.trim()
      .is_empty()
      .then(|| Err(FilterError::EmptyQuestion))
      .unwrap_or_else(|| Ok(q.trim().to_string()))
  }
  ```

### Complete TDD Workflow Example
```bash
# Step 1: RED - Create failing tests
# Write test file: tests/question_filter_tests.rs
cargo test                                  # → FAILS (no implementation)

# Step 2: GREEN - Implement minimal code
# Write code: src/agents/question_filter.rs
cargo test                                  # → PASSES
cargo build --release                       # → Success

# Step 3: REFACTOR - Improve code quality
# Improve implementation while keeping tests passing
cargo clippy -- -D warnings                 # → Zero warnings
cargo fmt                                   # → Formatted
cargo test                                  # → Still PASSES

# Final validation
cargo build --release                       # ✅ 100% SUCCESS
cargo clippy -- -D warnings                 # ✅ 100% SUCCESS
cargo test                                  # ✅ 100% SUCCESS
```

## Validation Requirements (100% required):

6. **Commit Format**:
   ```bash
   git commit -m "feat: [feature description]

   - Address #[issue-number]: [task title]
   - Test-first implemented: Tests written before code implementation
   - Red-Green-Refactor cycle followed (Red → Green → Refactor)
   - Build validation: 100% PASS (cargo build --release)
   - Lint validation: 100% PASS (cargo clippy -- -D warnings)
   - Type validation: 100% PASS (cargo check)

   🤖 Generated with Claude Code
   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

## Implementation Behavior

**Agent (Claude) Execution**:
- Read and analyze task requirements from GitHub issue
- Implement code changes directly using editing tools (Read/Edit/Write)
- Run all validation steps automatically (build, lint, type-check)
- Run **Backend-specific validation** (see below)
- Create commit with proper format and push to feature branch
- **NO PR creation** - ends with branch push

## Backend API Validation (MANDATORY after code implementation)

### Phase 1: Environment Verification
6. **Environment Variables Check**:
   ```bash
   # ตรวจสอบ environment variables ที่จำเป็นสำหรับ backend
   echo "DATABASE_URL: ${DATABASE_URL:0:20}..."
   echo "UPSTASH_REDIS_URL: ${UPSTASH_REDIS_URL:0:20}..."
   echo "UPSTASH_REDIS_TOKEN: ${UPSTASH_REDIS_TOKEN:0:15}..."
   echo "GEMINI_API_KEY: ${GEMINI_API_KEY:0:10}..."
   echo "API_KEY_DEFAULT: ${API_KEY_DEFAULT:0:15}..."
   echo "REDIS_STREAM_NAME: ${REDIS_STREAM_NAME:-not set}"
   echo "REDIS_CONSUMER_GROUP: ${REDIS_CONSUMER_GROUP:-not set}"
   ```

### Phase 2: Service Integration Test
7. **External Services Connectivity**:
   ```bash
   # Test Upstash Redis connection
   curl -s "${UPSTASH_REDIS_URL}/ping" \
        -H "Authorization: Bearer ${UPSTASH_REDIS_TOKEN}" > /dev/null 2>&1
   if [ $? -eq 0 ]; then
     echo "✅ Upstash Redis connected"
   else
     echo "❌ Upstash Redis connection failed"
     exit 1
   fi

   # Test Gemini API (simple request)
   curl -s -H "Content-Type: application/json" \
        -H "x-goog-api-key: ${GEMINI_API_KEY}" \
        -d '{"contents":[{"parts":[{"text":"test"}]}]}' \
        https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent \
        > /dev/null 2>&1
   if [ $? -eq 0 ]; then
     echo "✅ Gemini API connected"
   else
     echo "❌ Gemini API connection failed"
     exit 1
   fi
   ```

### Phase 3: API Service Testing
8. **API Server Validation**:
   ```bash
   # Start API server
   cargo run --bin api &
   API_PID=$!
   sleep 8

   # Test health endpoint
   curl -f http://localhost:3000/api/v1/health || exit 1
   echo "✅ API server responding"

   # Test new endpoints if any (example)
   # curl -X POST http://localhost:3000/api/v1/tarot/read \
   #   -H "Content-Type: application/json" \
   #   -H "Authorization: Bearer ${API_KEY_DEFAULT}" \
   #   -d '{"question":"ทดสอบระบบ"}' || exit 1

   kill $API_PID
   ```

### Phase 4: Worker Service Testing (if applicable)
9. **Background Worker Validation**:
   ```bash
   # Start worker if task includes queue processing
   cargo run --bin worker &
   WORKER_PID=$!
   sleep 5

   # Verify worker is running (basic check)
   kill -0 $WORKER_PID 2>/dev/null
   if [ $? -eq 0 ]; then
     echo "✅ Worker process running"
   else
     echo "❌ Worker process failed to start"
     exit 1
   fi

   kill $WORKER_PID
   ```

### Enhanced Backend Commit Format
```bash
git commit -m "feat: [feature description]

- Address #[issue-number]: [task title]
- Test-first implemented: Tests written before code implementation
- Red-Green-Refactor cycle followed (Red → Green → Refactor)
- Build validation: 100% PASS (cargo build --release)
- Lint validation: 100% PASS (cargo clippy -- -D warnings)
- Type validation: 100% PASS (cargo check)
- Backend validation: Environment & services verified
- API endpoint: Manual testing passed

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

## Error Handling

- **Issue not found**: Clear error with issue number
- **Invalid environment**: Git status and directory checks
- **Validation failures**: Stop workflow and report errors

## Integration

- **Before**: Use `/plan [task]` to create task issues
- **After**: Use `/pr [feedback]` to create pull request
- **Context**: Use `/fcs [topic]` for context discussions

## Files

- Feature branches: `feature/task-{issue}-{description}`
- GitHub Issues - Task definitions and requirements

## Notes

- **CRITICAL**: Always works from staging branch as base - NEVER from main
- **HARD ENFORCED**: Command will fail if trying to run from main branch
- Feature branch naming is strictly enforced
- 100% validation is mandatory before commits
- Never merge PRs yourself - wait for team approval