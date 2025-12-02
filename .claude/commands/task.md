# task

Smart Task Management - Unified command for context creation, planning, and workflow suggestions with auto-complexity detection.

## Usage

```
/task [feature description]              # Auto-detect complexity and suggest workflow
/task [feature description] --quick     # Force quick implementation
/task [feature description] --deep      # Force comprehensive planning
/task [feature description] --milestone # Force milestone breakdown
/task --analyze [description]            # Analyze complexity only
```

## Examples

```bash
# Auto-detection (recommended)
/task "Add user authentication system"
/task "Fix typo in API response"
/task "Implement payment processing with Stripe"
/task "Create health check endpoint"

# Force specific workflow
/task "Build complete dashboard" --deep      # Comprehensive planning
/task "Add email notification" --quick       # Quick implementation
/task "Migrate database schema" --milestone  # Milestone-based

# Analysis only
/task --analyze "Complex user workflow system"
```

## Implementation

### Phase 1: Complexity Analysis Engine

When `/task [description]` is executed:

1. **Keyword Analysis**:
   ```bash
   # High Complexity Indicators:
   - "system", "workflow", "architecture", "integration"
   - "multiple", "complex", "complete", "full"
   - External service names: "stripe", "paypal", "aws", "redis"
   - Multiple component keywords: "api + database + worker"

   # Medium Complexity Indicators:
   - "endpoint", "handler", "service", "component"
   - Single external service integration
   - Database operations

   # Low Complexity Indicators:
   - "fix", "update", "add", "small change"
   - Single file modifications
   - Configuration changes
   ```

2. **Scope Detection**:
   ```bash
   # Analyze for multiple components:
   components = extract_components(description)
   if components.length >= 3: HIGH_COMPLEXITY
   elif components.length >= 2: MEDIUM_COMPLEXITY
   else: LOW_COMPLEXITY
   ```

3. **Existing Codebase Impact**:
   ```bash
   # Check against current codebase:
   files_to_modify = analyze_impact(description)
   if files_to_modify.length >= 5: HIGH_COMPLEXITY
   elif files_to_modify.length >= 3: MEDIUM_COMPLEXITY
   else: LOW_COMPLEXITY
   ```

### Phase 2: Workflow Suggestion

Based on complexity analysis:

**LOW_COMPLEXITY (Score 1-3):**
```
🚀 Quick Implementation Detected
Estimated time: 5-15 minutes
Suggested workflow: /task [desc] --quick

✅ Features:
- Skip context creation
- Auto-generate single task
- Direct implementation
- Essential testing only

Proceed with quick implementation? (y/n)
```

**MEDIUM_COMPLEXITY (Score 4-7):**
```
🔧 Standard Implementation Detected
Estimated time: 20-45 minutes
Suggested workflow: /task [desc] (guided)

✅ Features:
- Lightweight context
- 2-3 atomic tasks
- Full testing
- Environment validation

Proceed with standard implementation? (y/n)
```

**HIGH_COMPLEXITY (Score 8-10):**
```
🏗️ Complex System Detected
Estimated time: 1-3 hours
Suggested workflow: /task [desc] --deep

✅ Features:
- Comprehensive context with milestones
- 4+ atomic tasks
- Full architecture planning
- Progressive implementation
- Complete test coverage

Proceed with comprehensive implementation? (y/n)
```

### Phase 3: Auto-Generation

**For LOW_COMPLEXITY (--quick):**
```bash
# Skip context creation, generate single task directly
/auto-generate-task [description]
```

**For MEDIUM_COMPLEXITY:**
```bash
# Create lightweight context + 2-3 tasks
/fcs [description] --lightweight
/auto-generate-tasks from context --count 2-3
```

**For HIGH_COMPLEXITY (--deep):**
```bash
# Full context with milestone breakdown
/fcs [description] --deep --milestone
/auto-generate-comprehensive-plan from context
```

### Phase 4: Smart Template Generation

**Auto-fill based on analysis:**
```markdown
# Example: "Add user authentication with JWT"

## Auto-Generated Task Requirements:
### 🧪 Auto-Suggested Tests:
- [ ] Unit test: JWT token generation/validation
- [ ] Integration test: Login/logout flow
- [ ] Security test: Invalid token rejection
- [ ] Performance test: Auth endpoint < 200ms

### 📦 Auto-Detected Dependencies:
- [ ] Required: `jsonwebtoken` crate
- [ ] Required: `bcrypt` for password hashing
- [ ] Suggested: `chrono` for token expiry

### 🔧 Auto-Generated Files:
- [ ] Create: `src/auth/mod.rs`
- [ ] Create: `src/auth/jwt.rs`
- [ ] Modify: `src/main.rs` (add auth middleware)
- [ ] Create: `tests/auth_integration.rs`

### ✅ Auto-Validation Requirements:
- [ ] Environment: JWT_SECRET variable
- [ ] Database: Users table exists
- [ ] API: Auth endpoints accessible
```

## Progressive Planning Features

### Milestone Auto-Breakdown

```bash
/task "Complete e-commerce system" --deep
```

**Auto-Generates:**
```markdown
## 📋 Auto-Detected Milestones

### Milestone 1: Core Product Management (Week 1)
- [TASK-001] Create products database schema
- [TASK-002] Implement product CRUD API
- [TASK-003] Add product search functionality

### Milestone 2: Shopping Cart System (Week 2)
- [TASK-004] Create cart management endpoints
- [TASK-005] Implement cart session handling
- [TASK-006] Add cart persistence

### Milestone 3: Payment Processing (Week 3)
- [TASK-007] Integrate Stripe payment API
- [TASK-008] Create payment workflow
- [TASK-009] Add payment status tracking
```

### Risk Assessment

```markdown
## ⚠️ Auto-Detected Risks

### High Risk Areas:
- **External API Dependency**: Stripe integration → Plan B: Manual payment?
- **Performance**: Database queries under load → Suggest: Add caching layer
- **Security**: JWT token management → Suggest: Implement refresh token flow

### Mitigation Strategies:
- [ ] Create payment provider abstraction layer
- [ ] Add database query optimization
- [ ] Implement comprehensive auth testing
```

## Integration with Existing Commands

### Command Mapping

```bash
# Old workflow → New unified command
/fcs topic → /task topic --context-only
/plan task → /task "implement task" --plan-only
/impl issue → /task --continue issue
```

### Backward Compatibility

```bash
# All existing commands still work:
/fcs [topic]           # Unchanged
/plan [task]           # Unchanged
/impl [issue]          # Unchanged

# But /task provides smart shortcuts:
/task [complex feature]     # Replaces fcs + plan + multiple impl
/task [simple feature]      # Reforms direct impl with auto-validation
/task --analyze [desc]      # New complexity analysis tool
```

## Error Handling

### Ambiguous Descriptions

```bash
/task "Fix stuff"
❌ Error: Description too vague
💡 Suggestions:
  - Be specific: "Fix user login timeout error"
  - Include context: "Fix API response format for mobile clients"
  - Specify scope: "Fix database connection pool leak"
```

### Resource Conflicts

```bash
# If similar context/task exists:
/task "User authentication system"
ℹ️  Similar issues found:
   - #123 [CONTEXT] User auth system design
   - #124 [TASK] JWT token implementation
   - #125 [TASK] Login endpoint

Options:
  [1] Continue with new implementation
  [2] Update existing context #123
  [3] Link to existing tasks
```

## Files

- `.claude/commands/task.md` - This command file
- `.claude/utils/complexity-analyzer.js` - Complexity detection engine
- `.claude/utils/task-generator.js` - Auto-generation utilities
- GitHub Issues - Context and task storage
- Templates - Enhanced for auto-generation

## Notes

- **Always validates against actual codebase** - no hallucination
- **Maintains TDD compliance** - test-first for all implementations
- **Preserves quality gates** - 100% validation requirements
- **Provides escape hatches** --quick, --deep, --milestone flags
- **Learns from patterns** - gets smarter with each implementation
- **Environment aware** - checks external services automatically

The `/task` command provides intelligent workflow guidance while maintaining the quality and rigor of the original system.