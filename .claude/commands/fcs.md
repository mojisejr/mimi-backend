# fcs

Context Management - Create and manage context discussions for iterative development planning.

## Usage

```
/fcs [topic-name]    # Create new context issue
/fcs list           # Show all active context issues
```

## Examples

```bash
/fcs payment-system              # Create context for payment system discussion
/fcs user-authentication         # Create context for auth flow discussion
/fcs list                        # Show all active context issues
```

## Implementation

When creating a new context issue:

1. **Validate GitHub CLI**: Ensure `gh` command is available
2. **Check template**: Verify `docs/ISSUE-TEMP.md` exists
3. **Reality-Grounded Context Analysis**:
   - Run codebase analysis using `.claude/utils/codebase-analyzer.js`
   - Scan `package.json` for available dependencies
   - Analyze existing components and patterns
   - Validate proposed topic against current capabilities
   - Generate "Current State vs Proposed" analysis
4. **Create GitHub Issue**:
   - Title: `[CONTEXT] {topic-name}`
   - Labels: `context`
   - Body: Use `docs/ISSUE-TEMP.md` template
   - Replace placeholders: `{{TOPIC}}`, `{{DATE}}`, `{{MODE}}`
   - **Enhanced**: Include "Codebase Reality Check" section with actual analysis
5. **Track context**: Add to `.claude/active_contexts` file
6. **Display results**: Show issue URL and next steps

When listing active contexts:

1. **Read tracker**: Parse `.claude/active_contexts` file
2. **Display list**: Show issue numbers and topics
3. **Provide guidance**: Suggest next actions

## Template Integration

Uses `docs/ISSUE-TEMP.md` template which contains:
- DISCUSSION LOG section for iterative updates
- ACCUMULATED CONTEXT section for key decisions
- PLANNING READINESS CHECKLIST for validation

## Files

- `docs/ISSUE-TEMP.md` - Context issue template
- `.claude/active_contexts` - Tracks active context issues
- `.claude/utils/codebase-analyzer.js` - Reality analysis utilities
- GitHub Issues - Stores context discussions

## Codebase Reality Analysis

The enhanced `/fcs` command now performs actual codebase analysis:

### Reality Check Process:
1. **Dependency Scan**: Checks `package.json` for installed packages
2. **Component Analysis**: Scans `src/components` for existing UI components
3. **Pattern Detection**: Identifies existing patterns (API routes, auth, forms)
4. **Capability Validation**: Validates if proposed topic is realistic
5. **Gap Analysis**: Identifies missing requirements and provides installation steps

### Example Reality Check Output:
```
## Codebase Reality Check

**Current State:**
- Framework: Next.js 14 App Router
- Database: PostgreSQL via Supabase + Prisma ORM
- Available: Basic UI components (Button, Card, Input)
- Missing: Form libraries, toast notifications, testing framework

**Topic "Payment System" Analysis:**
- ✅ Realistic: Can implement using existing patterns
- ❌ Missing: Payment processing library (stripe)
- 💡 Recommendation: Install stripe npm package first
- 🏗️ Implementation: Use existing Card/Dialog components
```

### Hallucination Prevention:
- ✅ All technical assumptions validated against actual code
- ✅ Component existence verified before referencing
- ✅ Dependencies checked before proposing features
- ✅ Realistic recommendations based on current capabilities

## Integration

This command integrates with:
- `/plan` - Context should reach `[Ready for Planning]` status before planning
- `/mode` - Current mode included in context creation
- Workflow system - Context is Phase 1 of development workflow

## Context Status Flow

1. **Created** - Initial context issue created
2. **Discussion** - Iterative updates via `/fcs [topic]`
3. **Ready for Planning** - Context ready for task creation
4. **Implementation Ready** - Context ready for implementation

## Notes

- Always creates GitHub Issues (NEVER local .md files)
- Context issues are living documents for discussion
- Use existing context issue when adding to topics
- Context must be ready before creating tasks with `/plan`