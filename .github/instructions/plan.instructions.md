---
applyTo: '**'
---

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

## Implementation

When creating a task issue:

### Phase 1: Hallucination Prevention Analysis
1. **Codebase Analysis**:
   - Scan existing components and patterns
   - Check `package.json` for installed dependencies
   - Verify available technologies and tools
   - Review current architecture and file structure

2. **Context7 Research**:
   - Document chosen technologies with official docs
   - Verify best practices and implementation patterns
   - Validate proposed solutions against documentation

3. **Previous Issue Context Check**:
   - Read all related issues for dependency context
   - Verify sequential task relationships
   - Check that referenced components actually exist/will exist
   - Validate implementation order and prerequisites

4. **Hallucination Prevention Checklist**:
   - ✅ Codebase components analyzed?
   - ✅ Dependencies verified in package.json?
   - ✅ Previous issue context checked?
   - ✅ Technology stack validated?
   - ✅ Implementation patterns reviewed?
   - ✅ File structure existence confirmed?
   - ✅ Sequential dependencies verified?
   - ✅ Context7 documentation consulted?
   - ✅ Assumptions vs reality checked?
   - ✅ MVP-appropriate scope confirmed?

### Phase 2: Task Creation
5. **Check Dependencies**:
   - Validate GitHub CLI (`gh`) availability
   - Verify `docs/TASK-ISSUE-TEMP.md` template exists
   - Get current execution mode from `/mode`

6. **Create Task Issue**:
   - Title: `[TASK] {task description}`
   - Labels: `task`, `{mode}-assignment` (manual/copilot)
   - Body: Use `docs/TASK-ISSUE-TEMP.md` template
   - Replace placeholders: `{{TASK_DESCRIPTION}}`, `{{EXECUTION_MODE}}`, `{{DATE}}`, `{{ASSIGNEE}}`
   - **Enhanced**: Include validated context and verified dependencies

7. **Mode-Based Assignment**:
   - **MANUAL**: Tasks assigned to human developer
   - **COPILOT**: Tasks assigned to @copilot

8. **Display Results**:
   - Show issue URL and number
   - Provide mode-specific next steps
   - List implementation requirements
   - **Enhanced**: Show validation context and verified dependencies

## Template Integration

Uses `docs/TASK-ISSUE-TEMP.md` template which includes:
- Task description and requirements
- Execution mode assignment
- 100% validation requirements (build, lint, type-check)
- Implementation workflow steps
- Quality standards checklist

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
- **Build validation**: `npm run build` (Next.js project)
- **Lint validation**: `npm run lint`
- **Format validation**: Auto-formatting (prettier)
- **Type check validation**: `npx tsc --noEmit`
- **Test validation**: `npm run test` (if available)

### Enhanced Validation Context
- **Dependencies verified**: Based on actual `package.json` analysis
- **Components confirmed**: Referenced components exist in codebase
- **Patterns validated**: Follow established codebase patterns
- **Scope realistic**: MVP-appropriate implementation requirements

## Workflow Integration

1. **Context Phase**: Use `/fcs [topic]` to create context discussion
2. **Planning Phase**: Use `/plan [task]` when context is ready
3. **Implementation Phase**: Use `/impl [issue-number]` to execute
4. **Review Phase**: Use `/pr [feedback]` to create pull request

## Files

- `docs/TASK-ISSUE-TEMP.md` - Task issue template
- GitHub Issues - Stores task definitions and requirements
- `.claude/current_mode` - Determines task assignment

## Notes

- Always creates GitHub Issues (NEVER local .md files)
- Tasks are atomic and focused on specific implementation
- Current mode affects task assignment and implementation workflow
- Ensure context is ready before creating tasks

### Hallucination Prevention Features
- **Reality-based planning**: All requirements validated against actual codebase
- **Dependency verification**: Components and libraries verified before task creation
- **Sequential validation**: Previous issue context checked for continuity
- **Pattern compliance**: Tasks follow existing codebase architecture and patterns
- **Scope realism**: MVP-appropriate requirements based on project maturity