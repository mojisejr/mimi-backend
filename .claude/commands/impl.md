# impl

Implementation Workflow - Execute GitHub issue implementation based on current mode.

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

3. **Mode-Specific Execution**:

   **MANUAL Mode**:
   - Agent (Claude) implements directly using code editing tools
   - Execute all implementation steps automatically
   - Handle all validation requirements
   - Create commit with proper format
   - Push branch to remote (NO PR creation)

   **COPILOT Mode**:
   - GitHub Copilot handles implementation automatically
   - Execute all validation steps
   - Create commit with proper format
   - Push branch to remote (NO PR creation)

4. **Validation Requirements** (100% required):
   ```bash
   npm run build                  # Build validation
   npm run lint                   # Lint validation
   npm run type-check             # Type check validation
   npm test                       # Test validation (if applicable)
   ```

5. **Commit Format**:
   ```bash
   git commit -m "feat: [feature description]

   - Address #[issue-number]: [task title]
   - Build validation: 100% PASS (npm run build)
   - Lint validation: 100% PASS (npm run lint)
   - Type validation: 100% PASS (npm run type-check)

   🤖 Generated with Claude Code
   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

## Mode-Specific Behavior

### MANUAL Mode

**Agent (Claude) Execution**:
- Read and analyze task requirements from GitHub issue
- Implement code changes directly using editing tools (Read/Edit/Write)
- Run all validation steps automatically (build, lint, type-check)
- Create commit with proper format and push to feature branch
- **NO PR creation** - ends with branch push

### COPILOT Mode

**GitHub Copilot Execution**:
- Trigger GitHub Copilot to handle implementation workflow
- Monitor all validation steps completion
- Ensure proper commit formatting and branch push
- **NO PR creation** - ends with branch push

## Error Handling

- **Issue not found**: Clear error with issue number
- **Invalid environment**: Git status and directory checks
- **Validation failures**: Stop workflow and report errors
- **Mode-specific**: Provide appropriate guidance per mode

## Integration

- **Before**: Use `/plan [task]` to create task issues
- **After**: Use `/pr [feedback]` to create pull request
- **Mode**: Use `/mode [manual|copilot]` to set execution mode
- **Context**: Use `/fcs [topic]` for context discussions

## Files

- Feature branches: `feature/task-{issue}-{description}`
- `.claude/current_mode` - Determines execution behavior
- GitHub Issues - Task definitions and requirements

## Notes

- **CRITICAL**: Always works from staging branch as base - NEVER from main
- **HARD ENFORCED**: Command will fail if trying to run from main branch
- Feature branch naming is strictly enforced
- 100% validation is mandatory before commits
- Mode affects who performs implementation steps
- Never merge PRs yourself - wait for team approval