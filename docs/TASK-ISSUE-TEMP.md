# Atomic Task Issue Template
**Template for creating explicit atomic tasks for implementation**

---

## 🚨 CRITICAL: ATOMIC TASK PRINCIPLES

### ✅ ATOMIC TASK CHARACTERISTICS
- **Single Responsibility**: One clear, deliverable outcome
- **Independent Execution**: No dependency chains or multiplexing
- **Complete Isolation**: Each task creates its own feature branch
- **100% Self-Contained**: All requirements and specifications included
- **100% Self-Contained**: All requirements and specifications included
- **Build Validation**: Must pass `cargo build --release` (or `cargo build`) with 100% success
- **Zero Linter Errors**: Must pass `cargo clippy -- -D warnings` with zero warnings/errors

### 🚫 FORBIDDEN PATTERNS
- ❌ **Chain Dependencies**: Task B depending on Task A completion
- ❌ **Multiplexing**: Multiple features in single task
- ❌ **Phase Sequencing**: "This depends on Phase X completion"
- ❌ **Incremental Builds**: Build failures allowed as intermediate steps
- ❌ **Shared Branches**: Multiple tasks on same feature branch

---

## 📋 Atomic Task Template Structure

### Title Format
```
[TASK-XXX-X] [Atomic]: [Single Deliverable Description]
```

### Body Template
```markdown
## [TASK-XXX-X] Atomic: [Single Deliverable]

### 🎯 SINGLE OBJECTIVE (MANDATORY)
**Complete this one specific deliverable end-to-end:**
- [EXACT single outcome this task must achieve]
- No additional features or modifications allowed

### 🧪 TEST-FIRST REQUIREMENTS (MANDATORY)
**Tests to write BEFORE code implementation:**
- [ ] Unit test: [test name] - [what should pass]
- [ ] Integration test: [test name] - [API/service behavior]
- [ ] Edge case test: [test name] - [boundary condition]

**Test Acceptance Criteria:**
- [ ] Tests must fail initially (Red phase - before implementation)
- [ ] Tests document expected behavior
- [ ] All tests pass after implementation (Green phase)
- [ ] Code is refactored while tests remain passing (Refactor phase)

### 📦 DELIVERABLE (MANDATORY)
**This task creates ONE complete deliverable:**
- **File(s) Created**: [exact file paths that will be created]
- **Functionality Added**: [single specific functionality]
- **Integration Points**: [where this connects to existing code]

### 🏗️ TECHNICAL REQUIREMENTS
- **Framework**: [FRAMEWORK] (project-specific)
- **Language**: [LANGUAGE] (e.g., TypeScript, Rust, Python)
- **Database**: [DATABASE] (e.g., PostgreSQL, MySQL)
- **UI Library**: [UI_LIBRARY] (optional)
- **Authentication**: [AUTH_METHOD] (e.g., OAuth, JWT)
- **Testing**: [TEST_TOOLING] (e.g., Jest, pytest)

### 📁 FILES TO CREATE (EXACT LIST) (Rust project examples)
```
src/[exact-path]/mod.rs
src/[exact-path]/handler.rs
tests/[exact-path]_tests.rs
migrations/[timestamp]_create_table.sql
```

### 🔁 FILES TO MODIFY (EXACT LIST)
```
src/[exact-path]/[existing-file].tsx    # exact file(s) that must be edited (replace mock)
src/[exact-path]/[existing-file].test.tsx # tests to update
```

### 🔧 EXACT IMPLEMENTATION REQUIREMENTS
- **No modifications to existing files** (unless specified)
  - If the task is intended to replace an existing mock or placeholder, list the exact files under "FILES TO MODIFY (EXACT LIST)" above. The implementer MUST modify those files rather than creating new pages/routes unless a new file is explicitly listed under "FILES TO CREATE (EXACT LIST)".
- **Complete type definitions/interfaces** for all data structures (per project language)
- **Comprehensive error handling** with user-friendly messages
- **Loading states** for all async operations (if applicable)
- **Accessibility compliance** where relevant
- **Mobile-first responsive design** when applicable

### 💾 DATABASE OPERATIONS (IF APPLICABLE)
```sql
-- EXACT SQL to execute (if any)
CREATE TABLE IF NOT EXISTS [table_name] (
  -- exact column definitions
);

-- EXACT RLS policies (if any)
ALTER POLICY ...;
```

### 🔧 BACKEND VALIDATION REQUIREMENTS (MANDATORY for all backend tasks)
**Environment Testing (MANDATORY):**
- [ ] All required environment variables are set and accessible
- [ ] External services connectivity verified with `/test-env` command
- [ ] Database connection to real Supabase PostgreSQL works
- [ ] Upstash Redis operations work correctly with real credentials
- [ ] Gemini API calls succeed with real API key

**Manual API Testing (MANDATORY):**
- [ ] New API endpoints respond correctly via curl/httpie
- [ ] Error handling works with real external services
- [ ] Performance meets requirements (target < 200ms response time)
- [ ] Queue operations complete successfully
- [ ] Database operations persist correctly in production-like environment

**Integration Points Validation:**
- [ ] API → Queue → Worker → Database end-to-end flow works
- [ ] External service error handling functions gracefully
- [ ] Authentication/authorization works with real API keys
- [ ] Background worker processes handle real queue jobs
- [ ] Retry logic and error recovery mechanisms work correctly

**Service Integration Tests:**
- [ ] Upstash Redis Streams operations test
- [ ] PostgreSQL transaction and query tests
- [ ] Gemini API integration and rate limiting tests
- [ ] Background worker concurrency and timeout tests

### 🧪 BACKEND TESTING REQUIREMENTS (Rust project)

**Unit Tests:** Core business logic and internal functions
- [ ] All core functions and utilities using `cargo test`
- [ ] Agent pipeline logic tests (Question Filter, Analyzer, Reader)
- [ ] Queue management logic tests
- [ ] Data transformation and validation tests
- [ ] Error handling and edge case tests

**Integration Tests:** API endpoints and external service interactions
- [ ] API endpoint tests under `tests/` (Axum handlers)
- [ ] Database operation tests with real PostgreSQL
- [ ] Queue operation tests with real Upstash Redis
- [ ] External service tests (Gemini API integration)
- [ ] End-to-end workflow tests (API → Queue → Worker → DB)

**Service Integration Tests:** Real external service connectivity
- [ ] Upstash Redis Streams operations test
- [ ] PostgreSQL transaction and query tests
- [ ] Gemini API integration and rate limiting tests
- [ ] Background worker process tests
- [ ] Configuration loading and environment validation tests

**Database and Schema Tests:**
- [ ] SQLx migrations run successfully in CI (`sqlx migrate` or `cargo sqlx prepare`)
- [ ] Database schema validation tests
- [ ] Data consistency and constraint tests
- [ ] Migration rollback tests (if applicable)

**Performance and Load Tests:**
- [ ] API response time tests (< 200ms target)
- [ ] Concurrent request handling tests
- [ ] Queue processing throughput tests
- [ ] Database query optimization tests

**Code Quality:**
- [ ] `cargo fmt -- --check` passes (code formatted)
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo check` passes (type safety)
- [ ] Code coverage meets minimum threshold

### ✅ ACCEPTANCE CRITERIA (100% MANDATORY) — Rust Backend

**Build and Code Quality:**
- [ ] `cargo build --release` passes with zero errors
- [ ] `cargo clippy -- -D warnings` passes with zero warnings/errors
- [ ] `cargo fmt -- --check` passes (code formatted)
- [ ] `cargo check` passes (type checks)
- [ ] Code follows project patterns and style guidelines

**Testing Requirements:**
- [ ] `cargo test` passes with zero failures
- [ ] Test-first implemented (tests written before code implementation)
- [ ] Test coverage complete for all new code paths
- [ ] Red-Green-Refactor cycle followed (Red → Green → Refactor)
- [ ] Integration tests with real external services pass

**Backend Validation (MANDATORY):**
- [ ] `/test-env` command passes all connectivity tests
- [ ] Environment variables validated and accessible
- [ ] New API endpoints work with real services (test via curl)
- [ ] Queue operations work with real Upstash Redis
- [ ] Database operations persist correctly
- [ ] External service integration (Gemini API) works
- [ ] Performance meets requirements (< 200ms API response)

**Operational Requirements:**
- [ ] Single deliverable works end-to-end with real services
- [ ] No unintended side effects on production environment
- [ ] Error handling works gracefully with external service failures
- [ ] Background worker processes jobs correctly
- [ ] Retry logic and error recovery mechanisms functional

**Documentation Requirements:**
- [ ] API endpoints documented (request/response formats)
- [ ] Environment variables usage documented
- [ ] Error scenarios and handling documented
- [ ] Integration points with external services documented

### 🔄 BACKEND WORKFLOW VALIDATION
**Pre-Implementation Requirements:**
- [ ] Run `/test-env` to verify all external services are accessible
- [ ] Check environment variables are properly configured
- [ ] Verify that current branch is clean (`git status`)
- [ ] Confirm that no conflicting changes are in progress

**Post-Implementation Validation:**
- [ ] Build validation: `cargo build --release` (100% success)
- [ ] Lint validation: `cargo clippy -- -D warnings` (zero warnings)
- [ ] Format validation: `cargo fmt -- --check` (consistent formatting)
- [ ] Type validation: `cargo check` (type safety)
- [ ] Test validation: `cargo test` (zero failures)

**Environment-Specific Testing:**
- [ ] API server starts and responds to health checks
- [ ] Background worker processes queue jobs correctly
- [ ] Database operations work with real Supabase
- [ ] Queue operations work with real Upstash Redis
- [ ] External service integrations work with real APIs

### 🔄 GIT WORKFLOW (MANDATORY)
- **Branch Name**: `feature/task-[XXX]-[X]-[description]`
- **Source Branch**: MUST branch from latest `[source branch]` (e.g., `staging` or `main`, per project)
- **No Merge Conflicts**: Branch must be clean and mergeable
- **Commit Format**:
  ```
  feat: [single deliverable]

  - Address TASK-XXX-X: [task title]
  - Complete atomic implementation
  - Test-first implemented (tests written before code)
  - Red-Green-Refactor cycle followed (Red → Green → Refactor)
  - Build validation: 100% PASS (cargo build --release)
  - Lint validation: 100% PASS (cargo clippy -- -D warnings)
  - Format validation: 100% PASS (cargo fmt -- --check)
  - Backend validation: Environment & services verified
  - API endpoint: Manual testing passed with real services
  - Tests: 100% PASS (0 failures)

  🤖 Generated with [Claude Code](https://claude.com/claude-code)
  Co-Authored-By: Claude <noreply@anthropic.com>
  ```

### 🚨 VALIDATION CHECKLIST (MANDATORY BEFORE COMMIT) — Rust
- [ ] `cargo build --release` → Must succeed
- [ ] `cargo clippy -- -D warnings` → No warnings/errors
- [ ] `cargo test` → All tests passed
- [ ] `cargo fmt -- --check` → No formatting diffs
- [ ] `cargo check` → Type checks pass

### 📖 IMPLEMENTATION INSTRUCTIONS
**Follow exactly these steps:**

1. **Create Feature Branch** (MANDATORY FIRST STEP):
  ```bash
  git checkout [source branch]
  git pull origin [source branch]
  git checkout -b feature/task-[XXX]-[X]-[description]
  ```

2. **Step 0: Write Tests First (Red Phase)** ⚠️ MANDATORY:
  - Write comprehensive unit tests for the new functionality
  - Write integration tests for API endpoints or service integrations
  - Tests should fail initially (Red phase - no implementation yet)
  - Tests document the expected behavior before code exists
  - Run: `cargo test` → Tests FAIL (expected at this stage)

3. **Implementation**: Build the single deliverable exactly as specified
  - If replacing a mock: open the file(s) listed in **FILES TO MODIFY** and replace the mock implementation there. Do not scaffold a new page or route. If the implementer finds that a new file is truly required, include an explanation in the PR and get an approver to confirm.
  - Follow Red-Green-Refactor cycle: Write minimal code to make tests pass (Green phase), then refactor for quality (Refactor phase)

4. **Validation** (MANDATORY BEFORE COMMIT):
  ```bash
  # Build
  cargo build --release

  # Lint (treat warnings as errors)
  cargo clippy -- -D warnings

  # Tests
  cargo test

  # Formatting check
  cargo fmt -- --check

  # Type check
  cargo check
  ```

5. **Commit Changes**:
   ```bash
   git add .
   git commit -m "feat: [single deliverable]..."
   ```

6. **Push Branch**:
   ```bash
   git push -u origin feature/task-[XXX]-[X]-[description]
   ```

### 🔗 RELATED CONTEXT (NO DEPENDENCIES)
- **Context Issue**: #[ISSUE-XXX] (for reference only)
- **No Task Dependencies**: This task must be executable independently
- **Reference Materials**: [relevant documentation links]

**Assign to**: [manual assignment as needed]
**Labels**: [no labels - clean issue creation]
```

---

## 🤖 MODE-BASED WORKFLOW SYSTEM

### Current Mode Tracking
**Global execution mode is stored in project settings:**
- **Default Mode**: MANUAL (human implementation)
- **Mode Switching**: Use `=mode` commands

### Mode Commands
```bash
=mode manual     # Switch to MANUAL mode (default)
=mode copilot     # Switch to COPILOT mode
=mode status      # Show current execution mode
```

### Task Creation by Mode
```bash
# Manual Mode (default)
=plan > [task description]                    # Creates task for human execution
=impl > [task-number]                        # Human implements task

# Copilot Mode
=mode copilot                                 # Switch to copilot mode
=plan > [task description]                    # Creates task for copilot execution
=impl > [task-number]                        # Copilot implements task
```

### Mode-Specific Behavior
**MANUAL Mode:**
- Task issues assigned to human executor
- =impl triggers manual implementation workflow
- Human performs all validation and commits

**COPILOT Mode:**
- Task issues assigned to @copilot
- =impl triggers copilot implementation workflow
- Copilot performs all validation and creates PRs

### Mode Persistence
- Mode setting persists throughout session
- Mode is stored in project configuration
- Can be changed anytime without affecting existing tasks

---

## 🚀 Atomic Task Examples

### Example 1: Database Table Creation (Rust)
```markdown
## [TASK-009-1] Atomic: Create Members Database Table

### 🎯 SINGLE OBJECTIVE (MANDATORY)
**Complete this one specific deliverable end-to-end:**
- Create `members` table in Supabase with exact schema and RLS policies

### 📦 DELIVERABLE (MANDATORY)
**This task creates ONE complete deliverable:**
- **File(s) Created**: `lib/supabase/schema/members.sql`
- **Functionality Added**: Database table for LINE OA member registration
- **Integration Points**: Connects to existing Supabase client configuration

### 🤖 COPILOT IMPLEMENTATION INSTRUCTIONS
**GitHub Copilot must:**
1. Create feature branch from staging: `feature/task-009-1-members-database-table`
2. Create SQL migration file under `migrations/` with exact table structure
3. Implement Row Level Security policies in SQL where applicable
4. Create Rust data models (structs/enums) and SQLx types for the new table
5. Validate with cargo commands: `cargo build --release`, `cargo clippy -- -D warnings`, `cargo test` (100% PASS)
6. Create pull request with proper documentation and migration notes

### 🏗️ TECHNICAL REQUIREMENTS
- **Framework**: [FRAMEWORK] (project-specific)
- **Language**: [LANGUAGE] (e.g., TypeScript, Rust, Python)
- **Database**: [DATABASE] (e.g., PostgreSQL, MySQL)
- **UI Library**: [UI_LIBRARY] (optional)
- **Authentication**: [AUTH_METHOD] (e.g., OAuth, JWT)
- **Testing**: [TEST_TOOLING] (e.g., Jest, pytest)

### 📁 FILES TO CREATE (EXACT LIST)
```
lib/[db]/schema/members.sql
lib/[db]/types/members.[ext]
```

### 💾 DATABASE OPERATIONS (MANDATORY)
```sql
-- File: lib/supabase/schema/members.sql
CREATE TABLE IF NOT EXISTS members (
  id SERIAL PRIMARY KEY,
  line_user_id VARCHAR(255) UNIQUE NOT NULL,
  display_name VARCHAR(255) NOT NULL,
  registration_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  contact_info TEXT,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Enable Row Level Security
ALTER TABLE members ENABLE ROW LEVEL SECURITY;

-- RLS Policy: Users can only see their own data
CREATE POLICY "Users can view own profile"
  ON members
  FOR SELECT
  USING (line_user_id = auth.jwt() ->> 'line_user_id');

-- RLS Policy: Users can only insert their own data
CREATE POLICY "Users can insert own profile"
  ON members
  FOR INSERT
  WITH CHECK (line_user_id = auth.jwt() ->> 'line_user_id');

-- RLS Policy: Users can only update their own data
CREATE POLICY "Users can update own profile"
  ON members
  FOR UPDATE
  USING (line_user_id = auth.jwt() ->> 'line_user_id');
```

### 🔧 EXACT IMPLEMENTATION REQUIREMENTS
- **No modifications to existing files** (unless specified in FILES TO MODIFY)
- **Complete Rust types** (structs/enums) for the Member model and SQLx query types
- **Comprehensive error handling** in database operations
- **Row Level Security** properly configured in SQL migrations
- **Timestamps** for created_at and updated_at

### ✅ ACCEPTANCE CRITERIA (100% MANDATORY) — Rust
- [ ] `cargo build --release` passes with **ZERO** errors or warnings
- [ ] `cargo clippy -- -D warnings` passes with **ZERO** warnings
- [ ] `cargo fmt -- --check` passes (formatting)
- [ ] All tests pass (`cargo test`) with **ZERO** failures
- [ ] SQL migration executes successfully (locally or in CI preview)
- [ ] Rust types compile without errors
- [ ] No unintended side effects

### 🔗 RELATED CONTEXT (NO DEPENDENCIES)
- **Context Issue**: #[ISSUE-008] (for reference only)
- **No Task Dependencies**: This task must be executable independently

**Assign to**: [manual assignment as needed]
**Labels**: [no labels - clean issue creation]
```

### Example 2: Axum Handler Creation (Manual Mode)
```markdown
## [TASK-009-2] Atomic: Create Tarot Reading API Handler

### 🎯 SINGLE OBJECTIVE (MANDATORY)
**Complete this one specific deliverable end-to-end:**
- Implement an Axum POST handler at `/api/v1/tarot/read` that accepts a JSON `TarotRequest` and returns `TarotResponse`.

### 📦 DELIVERABLE (MANDATORY)
**This task creates ONE complete deliverable:**
- **File(s) Created**: `src/handlers/tarot.rs`, `tests/tarot_integration_tests.rs`
- **Functionality Added**: Axum handler, request validation, invocation of existing pipeline function, and proper error mapping
- **Integration Points**: Connects to `TarotReadingPipeline::process_reading` and `AppState` for DB and LLM clients

### 👨‍💻 MANUAL IMPLEMENTATION INSTRUCTIONS
**Human developer must:**
1. Create feature branch from staging: `feature/task-009-2-tarot-handler`
2. Implement Axum handler with Serde request/response models
3. Wire handler into router in `main.rs` or router module (update only if specified in FILES TO MODIFY)
4. Add integration tests under `tests/` using `axum::Server::bind` test harness or `tower::Service` test utilities
5. Validate: `cargo build --release`, `cargo clippy -- -D warnings`, `cargo test` (100% PASS)
6. Commit with proper message format and create pull request

### 🏗️ TECHNICAL REQUIREMENTS
- **Framework**: Axum
- **Language**: Rust (edition 2021/2024)
- **Testing**: `cargo test` + integration tests in `tests/`

### 📁 FILES TO CREATE (EXACT LIST)
```
src/handlers/tarot.rs
tests/tarot_integration_tests.rs
```

### 🔧 EXACT IMPLEMENTATION REQUIREMENTS
- **No modifications to existing files** unless the task explicitly lists `FILES TO MODIFY`
- **Complete Rust types** for request/response (Serde) and mapping to internal models
- **Comprehensive error handling** with appropriate HTTP status codes
- **Add tests** to cover success and failure cases
- **Documentation**: brief README entry or comment indicating handler responsibilities

### ✅ ACCEPTANCE CRITERIA (100% MANDATORY)
- [ ] `cargo build --release` passes with **ZERO** errors or warnings
- [ ] `cargo clippy -- -D warnings` passes with **ZERO** warnings
- [ ] `cargo test` passes with **ZERO** failures (unit + integration)
- [ ] Handler correctly invokes pipeline and returns expected JSON shape
- [ ] No unintended side effects

### 🔗 RELATED CONTEXT (NO DEPENDENCIES)
- **Context Issue**: #[ISSUE-008] (for reference only)
- **No Task Dependencies**: This task must be executable independently

**Assign to**: [manual assignment as needed]
**Labels**: [no labels - clean issue creation]
```

---

## 🎯 MODE-SPECIFIC COMMANDS

### Manual Mode Commands
```bash
=plan > [task description]              # Create atomic task for manual implementation
=impl > [task-number]                  # Implement task manually
=mode status                           # Show current mode (should show MANUAL)
```

### Copilot Mode Commands
```bash
=mode copilot                          # Switch to COPILOT mode
=plan > [task description]              # Create atomic task for copilot implementation
=impl > [task-number]                  # Trigger copilot implementation
=mode status                           # Show current mode (should show COPILOT)
```

### Mode Switching Behavior
```bash
# Start in MANUAL mode (default)
=plan > create UI component               # Task assigned to human

# Switch to COPILOT mode
=mode copilot

# Now tasks will be assigned to copilot
=plan > create database table             # Task assigned to @copilot

# Switch back to MANUAL mode
=mode manual

# Tasks will be assigned to human again
=plan > implement auth logic              # Task assigned to human
```

---

## 📝 Atomic Task Creation Guidelines

### 1. Single Deliverable Focus
- **One Thing Only**: Each task must create exactly one deliverable
- **Complete End-to-End**: Deliverable must be fully functional when task completes
- **No Partial Work**: Don't split logical units into multiple tasks

### 2. Complete Independence
- **No Dependencies**: Task must not require completion of other tasks
- **Self-Contained**: All requirements included in single issue
- **Standalone Execution**: Copilot can start and complete without context from other tasks

### 3. Zero-Tolerance Quality Standards
- **Build 100% Success**: `npm run build` must pass with zero errors/warnings
- **Lint 100% Success**: `npm run lint` must pass with zero violations
- **Tests 100% Success**: All tests must pass with zero failures
- **Type Safety**: TypeScript compilation must pass completely

### 4. Exact Specifications
- **Precise File Paths**: Specify exact file locations and names
- **Detailed Requirements**: Include all technical specifications
- **Complete Examples**: Provide code examples and patterns to follow

### 5. Branch Isolation
- **One Branch Per Task**: Each atomic task gets its own feature branch
- **Clean Merges**: Branches must be mergeable without conflicts
- **Independent**: Branch does not depend on other feature branches

### 🚨 VALIDATION CHECKLIST BEFORE CREATING ATOMIC TASKS
- [ ] **Single Deliverable**: Task creates exactly one thing
- [ ] **No Dependencies**: Task can be executed independently
- [ ] **Complete Requirements**: All specifications included
- [ ] **Testable**: Success criteria are measurable
- [ ] **Build Ready**: Requirements ensure 100% build success
- [ ] **Lint Ready**: Requirements ensure zero lint violations

---

## ⚡ GitHub CLI Commands for Atomic Tasks

```bash
# Create atomic task issue
gh issue create \
  --title "[TASK-XXX-X] Atomic: [Single Deliverable]" \
  --body "$(cat docs/ISSUE-TEMP.md | sed -n '/## \[TASK-XXX-X\] Atomic:/,/```/p')" \
  
# Monitor atomic task progress
gh issue list --state open

# Review atomic task PR
gh pr list
```

---

## 🎯 Best Practices for Atomic Task Management

### Before Creating Atomic Tasks
1. **Break Down Features**: Split large features into smallest atomic units
2. **Verify Independence**: Ensure no task depends on another task
3. **Specify Exact Requirements**: Include all technical details
4. **Define Success Criteria**: Make acceptance criteria measurable

### During Task Creation
1. **Use Template**: Always use the atomic task template
2. **Be Specific**: Provide exact file paths and implementation details
3. **Include Examples**: Show expected code patterns and structures
4. **Test Requirements**: Specify all testing requirements

### After Task Assignment
1. **Monitor Progress**: Check build and lint results continuously
2. **Review Code**: Thoroughly review all pull requests
3. **Validate Independence**: Ensure task truly executed independently
4. **Track Metrics**: Monitor success rates and quality indicators

### Common Anti-Patterns to Avoid
- ❌ **Multiple Deliverables**: Creating more than one thing in a task
- ❌ **Task Dependencies**: Task B requiring Task A completion
- ❌ **Incremental Builds**: Allowing build failures as intermediate steps
- ❌ **Shared Branches**: Multiple tasks on same feature branch
- ❌ **Partial Requirements**: Incomplete specifications leading to confusion

---

## 📊 Atomic Task Performance Metrics

### Key Metrics to Track
- **Task Independence Rate**: Percentage of tasks that execute without dependencies
- **First-Time Success Rate**: Percentage of tasks that pass all validations on first attempt
- **Build Success Rate**: Percentage of atomic tasks with 100% build success
- **Lint Success Rate**: Percentage of atomic tasks with zero lint violations
- **Test Coverage**: Average test coverage for atomic task deliverables

### Quality Indicators
- **Zero Tolerance**: Any build failure, lint violation, or test failure is unacceptable
- **Complete Isolation**: Tasks must be truly independent
- **Specification Clarity**: Clear, detailed specifications reduce rework
- **Consistent Patterns**: Following established patterns improves success rates

---

*Last Updated: Use `date +"%Y-%m-%d %H:%M:%S"` to get current timestamp*
*Repository: [REPOSITORY_NAME]*