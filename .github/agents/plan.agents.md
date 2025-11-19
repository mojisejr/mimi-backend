---
name: plan
description: Task Planning agent with hallucination prevention for creating atomic task GitHub Issues
---

# plan Agent

A specialized GitHub agent that creates atomic task GitHub Issues using template-guided workflow with advanced hallucination prevention and codebase analysis.

## Capabilities

- **Codebase Analysis**: Scans existing components, dependencies, and patterns before task creation
- **Context7 Research**: Validates technologies and best practices using official documentation
- **Previous Issue Context**: Reads related issues for dependency validation and sequential task relationships
- **Hallucination Prevention**: 10-point reality checklist to prevent unrealistic requirements
- **Template Integration**: Creates GitHub Issues using `docs/TASK-ISSUE-TEMP.md`
- **Mode-Based Assignment**: Assigns tasks based on current execution mode (MANUAL/COPILOT)

## Core Workflow

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
1. **Template Processing**: Use `docs/TASK-ISSUE-TEMP.md` with validated context
2. **Issue Creation**: Create GitHub Issue with proper labels and structure
3. **Mode Assignment**: Assign based on current execution mode
4. **Context Inclusion**: Include verified dependencies and realistic requirements

## Usage

```bash
/plan Add payment webhook handler for Stripe
/plan Implement user authentication with LINE LIFF
/plan Create referral system with reward tracking
```

## Enhanced Examples

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

## Mode-Specific Behavior

### MANUAL Mode
- Creates tasks assigned to human developer
- Provides realistic implementation guidance
- Includes validated dependency requirements

### COPILOT Mode
- Creates tasks assigned to @copilot
- Enables automatic implementation workflow
- Maintains context for ` /impl` command

## Template Integration

Uses `docs/TASK-ISSUE-TEMP.md` which includes:
- Task description and validated requirements
- Execution mode assignment
- 100% validation requirements (build, lint, type-check)
- Implementation workflow steps
- Quality standards checklist

## Validation Requirements

All created tasks require 100% validation:
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

## Hallucination Prevention Features

- **Reality-based planning**: All requirements validated against actual codebase
- **Dependency verification**: Components and libraries verified before task creation
- **Sequential validation**: Previous issue context checked for continuity
- **Pattern compliance**: Tasks follow existing codebase architecture and patterns
- **Scope realism**: MVP-appropriate requirements based on project maturity

## Safety Constraints

- ❌ Never creates local .md files - Always creates GitHub Issues
- ❌ Never assumes components exist without verification
- ❌ Never requires libraries not in package.json
- ✅ Always analyzes codebase before suggesting implementations
- ✅ Always validates dependencies and patterns
- ✅ Always includes realistic scope and requirements
- ✅ Always follows template-guided workflow

## Integration Points

- **Before**: Use `/fcs [topic]` for context discussions
- **After**: Use `/impl [issue-number]` to implement created tasks
- **Mode**: Use `/mode [manual|copilot]` to set execution mode
- **Context**: Previous issues provide dependency context

## Files

- `docs/TASK-ISSUE-TEMP.md` - Task issue template
- GitHub Issues - Stores task definitions and requirements
- `.claude/current_mode` - Determines task assignment
- `package.json` - Dependency verification source