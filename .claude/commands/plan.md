# plan

Task Planning - Create atomic task GitHub Issues for implementation workflow.

## Usage

```
/plan [task description]
```

## Examples

```bash
/plan Add payment webhook handler for Stripe
/plan Implement user authentication with LINE LIFF
/plan Create referral system with reward tracking
/plan Fix database migration performance issue
```

### Enhanced Examples (with Hallucination Prevention)

Before (hallucination risk):
```bash
/plan Implement comprehensive error handling system
```

After (reality-based):
```bash
# Plan analyzes codebase first:
# - No testing framework exists → Setup Jest infrastructure
# - Only Card/Button components exist → Error handling using existing patterns
# - Current error handling: Basic try-catch → Enhance with Card-based displays
/plan Add error handling using existing Card components and setup basic Jest testing
```

## Implementation

When creating a task issue:

### Phase 1: Actual Hallucination Prevention Analysis (IMPLEMENTED)
1. **Codebase Analysis** (ACTUALLY EXECUTED):
   - Load `.claude/utils/codebase-analyzer.js`
   - Execute `analyzeDependencies()` to check `Cargo.toml`
   - Run `analyzeComponents()` to scan existing UI components
   - Execute `generateCodebaseSummary()` for current state
   - Store results for validation step

2. **Feature Validation** (ACTUALLY EXECUTED):
   - Run `validateProposedFeature(taskDescription)`
   - Identify missing dependencies with installation commands
   - Check if existing components can support the requirement
   - Generate realistic alternatives for missing pieces

3. **Context7 Research**:
   - Document chosen technologies with official docs
   - Verify best practices and implementation patterns
   - Validate proposed solutions against documentation

4. **Previous Issue Context Check**:
   - Read all related issues for dependency context
   - Verify sequential task relationships
   - Check that referenced components actually exist/will exist
   - Validate implementation order and prerequisites

5. **Automated Hallucination Prevention Checklist** (ACTUALLY VERIFIED):
   - ✅ Codebase components analyzed? (`CodebaseAnalyzer.analyzeComponents()`)
   - ✅ Dependencies verified in `Cargo.toml`? (`CodebaseAnalyzer.analyzeDependencies()`)
   - ✅ Previous issue context checked? (`gh issue list` + analysis)
   - ✅ Technology stack validated? (Codebase summary)
   - ✅ Implementation patterns reviewed? (Pattern detection)
   - ✅ File structure existence confirmed? (Directory scanning)
   - ✅ Sequential dependencies verified? (Issue relationship check)
   - ✅ Context7 documentation consulted? (External docs lookup)
   - ✅ Assumptions vs reality checked? (`validateProposedFeature()`)
   - ✅ MVP-appropriate scope confirmed? (Reality-based scoping)

### Phase 2: Reality-Based Task Creation
6. **Check Dependencies**:
   - Validate GitHub CLI (`gh`) availability
   - Verify `docs/TASK-ISSUE-TEMP.md` template exists
   - Get current execution mode from `/mode`

7. **Generate Reality-Checked Task Content**:
   - Use CodebaseAnalyzer results to create accurate requirements
   - Include installation commands for missing dependencies
   - Specify exact file paths that actually exist
   - Reference components that are confirmed available
   - Provide fallback alternatives for missing pieces
   - Generate implementation approach based on existing patterns

8. **Create Task Issue**:
   - Title: `[TASK] {task description}`
   - Labels: `task`, `{mode}-assignment` (manual/copilot)
   - Body: Use `docs/TASK-ISSUE-TEMP.md` template
   - Replace placeholders: `{{TASK_DESCRIPTION}}`, `{{EXECUTION_MODE}}`, `{{DATE}}`, `{{ASSIGNEE}}`
   - **Enhanced**: Include "Reality Check" section with actual analysis:
     ```markdown
     ## Reality Check
     **Dependencies Verified:**
     - ✅ Available: next, react, typescript
   - ❌ Missing: zod, react-hook-form (Install: use `cargo add` for Rust crates or add appropriate crates)

     **Components Confirmed:**
     - ✅ Available: Button, Card, Input, Alert
     - ❌ Missing: Form components (Use existing Input + validation)

     **Implementation Path:**
     - Use existing API patterns from /api/farm
     - Follow Card-based layout patterns from /components
     - Implement manual validation initially, upgrade to zod when installed
     ```

9. **Mode-Based Assignment**:
   - **MANUAL**: Tasks assigned to human developer
   - **COPILOT**: Tasks assigned to @copilot

10. **Display Results**:
    - Show issue URL and number
    - Provide mode-specific next steps
    - List implementation requirements
    - **Enhanced**: Show validation context and verified dependencies
    - Display reality check summary with missing pieces clearly identified

## Template Integration

Uses `docs/TASK-ISSUE-TEMP.md` template which includes:
- Task description and requirements
- Execution mode assignment
- 100% validation requirements (build, clippy, fmt)
- Implementation workflow steps
- Quality standards checklist

## 🧪 TEST-FIRST REQUIREMENTS (MANDATORY)

All task planning must include explicit test-first requirements:

### Test Specification Template
```markdown
### 🧪 TEST-FIRST REQUIREMENTS (MANDATORY)
**Tests to write BEFORE code implementation:**
- [ ] Unit test: [test name] - [what should pass]
- [ ] Integration test: [test name] - [API/service behavior]
- [ ] Edge case test: [test name] - [boundary condition]

**Test Acceptance Criteria:**
- Tests must fail initially (Red phase)
- Tests document expected behavior
- All tests pass after implementation (Green phase)
- Code is refactored while tests remain passing (Refactor phase)
```

### Examples with Test-First Planning

**Before (No Test-First):**
```bash
/plan Add authentication module
```

**After (Test-First Integrated):**
```bash
/plan Add authentication module with unit tests for auth validation and integration tests for API endpoints
```

**Full Example Task Description:**
```markdown
### 🎯 SINGLE OBJECTIVE
- Implement question filter agent that validates tarot questions before processing

### 🧪 TEST-FIRST REQUIREMENTS (MANDATORY)
Tests to write BEFORE code implementation:
- [ ] Unit test: `test_empty_question_rejected` - empty questions should return FilterError::EmptyQuestion
- [ ] Unit test: `test_valid_question_accepted` - valid questions should pass validation
- [ ] Unit test: `test_whitespace_trimming` - questions with only whitespace should be rejected
- [ ] Integration test: `test_api_endpoint_validation` - POST /api/tarot validates question before processing
- [ ] Edge case test: `test_special_characters_thai` - Thai characters should be accepted

**Test Acceptance Criteria:**
- [ ] All tests fail initially (Red phase - before implementation)
- [ ] Tests pass after implementation (Green phase)
- [ ] Code is refactored for quality while tests remain passing (Refactor phase)
```

## Mode-Specific Next Steps

### MANUAL Mode
- Human developer will implement the task
- Use `/impl [issue-number]` when ready to implement
- Follow implementation workflow with 100% validation
- Create PR with `/pr [feedback]` after implementation

### COPILOT Mode
- Use `/impl [issue-number]` to trigger automatic implementation
- Copilot handles complete implementation workflow
- Includes PR creation via `/pr` after implementation

## Implementation Requirements

All tasks require 100% validation:
- **Build validation**: `cargo build --release`
- **Lint validation**: `cargo clippy -- -D warnings`
- **Format validation**: `cargo fmt -- --check`
- **Type check validation**: `cargo check`
- **Test validation**: `cargo test` (if available)

### Test-First Requirements
- All tasks MUST specify which tests need to be written first
- Test case specification is part of task description
- Tests must be written BEFORE code implementation (Red phase)
- Test coverage must be comprehensive for new/modified code

### Enhanced Validation Context
   - **Dependencies verified**: Based on actual `Cargo.toml` analysis
- **Components confirmed**: Referenced components exist in codebase
- **Patterns validated**: Follow established codebase patterns
- **Scope realistic**: MVP-appropriate implementation requirements
- **Test-First**: Tests written before code implementation

## Workflow Integration

1. **Context Phase**: Use `/fcs [topic]` to create context discussion
2. **Planning Phase**: Use `/plan [task]` when context is ready
3. **Implementation Phase**: Use `/impl [issue-number]` to execute
4. **Review Phase**: Use `/pr [feedback]` to create pull request

## Files

- `docs/TASK-ISSUE-TEMP.md` - Task issue template
- GitHub Issues - Stores task definitions and requirements
- `.claude/current_mode` - Determines task assignment
- `.claude/utils/codebase-analyzer.js` - Reality analysis utilities

## Hallucination Prevention Implementation

### Actually Executed Analysis:
The enhanced `/plan` command now performs real-time codebase analysis:

```javascript
// Example actual execution flow:
const analyzer = new CodebaseAnalyzer();
const summary = analyzer.generateCodebaseSummary();
const validation = analyzer.validateProposedFeature(taskDescription);

// Results used in task creation:
if (validation.missingRequirements.length > 0) {
  taskContent += "\n## Installation Requirements\n";
  validation.recommendations.forEach(rec => {
    taskContent += `- ${rec}\n`;
  });
}
```

### Reality Check Examples:

**Before (Hallucinated):**
```markdown
## Task Requirements
- Implement using zod validation schema
- Use react-hook-form for form management
- Add toast notifications for feedback
```

**After (Reality-Based):**
```markdown
## Reality Check
**Dependencies Verified:**
- ✅ Available: next, react, typescript, prisma
- ❌ Missing: zod, react-hook-form (Install: npm install zod react-hook-form)
- ❌ Missing: toast system (Use existing Alert components or install react-hot-toast)

**Components Confirmed:**
- ✅ Available: Button, Card, Input, Select, Alert
- ✅ Available: Form patterns from existing components
- ✅ Available: API patterns from /api/farm

**Implementation Path:**
1. Install missing dependencies first
2. Create manual validation as fallback
3. Use existing Card/Button components for UI
4. Follow established API route patterns
```

## Notes

- Always creates GitHub Issues (NEVER local .md files)
- Tasks are atomic and focused on specific implementation
- Current mode affects task assignment and implementation workflow
- Ensure context is ready before creating tasks
- **NEW**: All task requirements validated against actual codebase reality
- **TEST-FIRST MANDATORY**: All tasks must include explicit test-first requirements
- Tests must be written BEFORE code implementation
- Test coverage is mandatory, not optional

### Implemented Hallucination Prevention Features:
- ✅ **Real codebase analysis**: Uses CodebaseAnalyzer for actual dependency and component scanning
- ✅ **Automated validation**: validateProposedFeature() checks task feasibility
- ✅ **Installation guidance**: Provides exact crate-add guidance (e.g., `cargo add`) or equivalent for missing dependencies
- ✅ **Fallback alternatives**: Suggests workarounds for missing components
- ✅ **Reality-based requirements**: Task content based on actual project state
- ✅ **Pattern compliance**: Tasks follow existing codebase architecture
- ✅ **Sequential validation**: Dependencies between tasks verified
- ✅ **Scope realism**: MVP-appropriate requirements based on current capabilities