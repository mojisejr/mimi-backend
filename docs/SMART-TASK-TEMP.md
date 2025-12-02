# Smart Task Template - Compact Version
**Compressed template for rapid task generation while maintaining quality**

---

## 📋 [TASK-XXX] [Single Deliverable Description]

### 🎯 Objective
**Complete this one specific deliverable:**
- [Clear outcome this task must achieve]

### 🧪 Test Requirements (TDD - MANDATORY)
**Tests to write BEFORE code:**
- [ ] Unit test: [specific test] - [expected behavior]
- [ ] Integration test: [api/service test] - [what to verify]
- [ ] Error case test: [edge case] - [boundary condition]

**TDD Process:**
- **Red**: Write failing tests → `cargo test` → tests FAIL ✗
- **Green**: Implement minimal code → `cargo test` → tests PASS ✓
- **Refactor**: Improve quality → `cargo test` → tests still PASS ✓

### 📦 Implementation Scope
**Files to create/modify:**
- [ ] `src/...` - [specific file purpose]
- [ ] `tests/...` - [test file purpose]

**Functionality to implement:**
- [Feature 1] - [specific implementation detail]
- [Feature 2] - [specific implementation detail]

### 🔍 Dependencies & Environment
**Required dependencies:**
- [ ] Check: `Cargo.toml` for existing crates
- [ ] Add: [missing crate] with `cargo add ...`

**Environment variables needed:**
- [ ] ENV_VAR_NAME - [purpose]

**External services:**
- [ ] Service name - [integration point]
- [ ] Run `/test-env [service]` to verify

### ✅ Validation Requirements (100% mandatory)
```bash
# Build validation
cargo build --release          # ✅ 100% success, zero errors

# Lint validation
cargo clippy -- -D warnings    # ✅ Zero warnings, zero errors

# Format validation
cargo fmt -- --check           # ✅ Consistent formatting

# Test validation
cargo test                     # ✅ All tests pass

# Backend validation (if applicable)
cargo run --bin api &          # ✅ Server starts successfully
curl -f http://localhost:3000/health  # ✅ API responds
```

### 🚨 Acceptance Criteria
**Task is COMPLETE when:**
- [ ] All tests pass (including new ones written first)
- [ ] Code builds successfully with zero errors
- [ ] Zero clippy warnings
- [ ] Code formatting consistent
- [ ] Manual API testing successful (if applicable)
- [ ] External services integration works (if applicable)
- [ ] Feature works end-to-end

### 📝 Notes
- **Single Responsibility**: This task delivers ONE complete feature
- **Independent**: No dependencies on other incomplete tasks
- **Test-First**: Tests MUST be written before implementation code
- **Quality Gates**: All validation steps must pass 100%
- **Documentation**: Code is self-documenting with clear naming

---

## 🎯 Task Generation Examples

### Simple Backend Task:
```markdown
## 📋 [TASK-001] Add health check endpoint

### 🎯 Objective
Add GET /api/health endpoint that returns service status

### 🧪 Test Requirements
- [ ] Unit test: health endpoint returns 200
- [ ] Integration test: endpoint includes database status
- [ ] Error case test: database connection failure handling

### 📦 Implementation Scope
- [ ] `src/handlers/health.rs` - health check logic
- [ ] `tests/health_tests.rs` - integration tests

### ✅ Validation Requirements
Standard validation + API endpoint testing
```

### Complex Feature Task:
```markdown
## 📋 [TASK-002] Implement user authentication with JWT

### 🎯 Objective
Complete user authentication flow with JWT tokens

### 🧪 Test Requirements
- [ ] Unit test: JWT token generation/validation
- [ ] Integration test: login/logout flow
- [ ] Security test: invalid token rejection
- [ ] Performance test: auth endpoint < 200ms

### 📦 Implementation Scope
- [ ] `src/auth/mod.rs` - auth module
- [ ] `src/auth/jwt.rs` - JWT logic
- [ ] `src/handlers/auth.rs` - auth endpoints
- [ ] Modify `src/main.rs` - add auth middleware
- [ ] `tests/auth_integration.rs` - comprehensive tests

### 🔍 Dependencies
- [ ] Add: `jsonwebtoken` crate
- [ ] Add: `bcrypt` crate for passwords
- [ ] ENV: JWT_SECRET_KEY
- [ ] ENV: DATABASE_URL (users table)

### ✅ Validation Requirements
Standard validation + security testing + performance testing
```

---

**Template Size: ~100 lines vs 500+ lines (80% reduction)**
**Quality: Maintained with all critical requirements included**