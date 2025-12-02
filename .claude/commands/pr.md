# pr

Pull Request Creation - Create Pull Request from feature branch to **STAGING ONLY**.

🚨 **CRITICAL SAFETY**: This command **NEVER** creates PRs to main branch. Main branch merging is **FORBIDDEN** and **BLOCKED** by system validation.

## Usage

```
/pr [optional feedback]
```

## Examples

```bash
/pr                           # Create PR without additional feedback
/pr Ready for review          # Create PR with feedback message
/pr Implements user auth flow # Create PR with description
```

## Implementation

### Pre-PR Validation

1. **Check Dependencies**:
   - Validate GitHub CLI (`gh`) availability
   - Verify Git tools are available

2. **Validate Environment**:
   - Ensure clean git working directory
   - Verify we're on a feature branch
   - Check branch follows naming: `feature/task-{issue}-{description}`
   - Confirm branch is pushed to remote
   - Verify staging branch exists

3. **🚨 CRITICAL: Staging Branch Enforcement**:
   - Confirm main branch protection is active
   - Verify staging branch exists and is valid target
   - Block any attempt to target main branch
   - Validate PR will NOT merge to main under any circumstances

4. **Extract Issue Information**:
   - Parse issue number from branch name
   - Validate issue exists and is a task
   - Get issue title and description

### Pre-PR Validations (100% Required)

```bash
cargo build --release           # Build validation
cargo clippy --all-targets --all-features  # Lint validation
cargo fmt -- --check           # Format validation
cargo check                    # Type check validation
cargo test                     # Test validation (if applicable)
```

### PR Creation

1. **Generate PR Title**:
   ```
   feat: {clean task title} (resolve #{issue-number})
   ```

2. **Create PR Body** with sections:
   - **Summary**: Task description and resolution
   - **Changes**: Implementation checklist
   - **Validation**: All validation results
   - **Test Plan**: Testing checklist
   - **Additional Notes**: User feedback (if provided)

3. **Create Pull Request**:
   ```bash
   # 🚨 CRITICAL: ALWAYS target staging branch - NEVER main
   gh pr create \
     --title "{title}" \
     --base staging \
     --head "{feature-branch}" \
     --body "{body}" \
     --label "auto-pr"
   ```

4. **🚨 STAGING BRANCH ENFORCEMENT**:
   ```bash
   # Verify PR targets staging branch (NEVER main)
   gh pr view --json baseRefName | jq -r '.baseRefName' | grep -q "^staging$" || {
     echo "🚨 ERROR: PR must target staging branch ONLY!"
     echo "❌ Found target: $(gh pr view --json baseRefName | jq -r '.baseRefName')"
     echo "✅ Required target: staging"
     exit 1
   }
   ```

## PR Body Template

```markdown
## Summary

This PR implements: **{task description}**

- Resolves #{issue-number}: {task title}
- Created from feature branch: `{branch-name}`

## Changes

- [ ] Implementation completed according to task requirements
- [ ] Code follows project standards and conventions
- [ ] Tests added where applicable
- [ ] Documentation updated if needed

## Validation

- ✅ Build validation: 100% PASS (`cargo build --release`)
- ✅ Clippy validation: 100% PASS (`cargo clippy`)
- ✅ Format validation: 100% PASS (`cargo fmt`)
- ✅ Type check validation: 100% PASS (`cargo check`)

## Test Plan

- [ ] Manual testing completed
- [ ] Automated tests pass
- [ ] Integration with existing systems verified
- [ ] Performance impact assessed (if applicable)

## Additional Notes

{user feedback}

---

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>
```

## Error Handling

- **Not on feature branch**: Clear error with current branch
- **Branch not pushed**: Instructions to push branch first
- **Validation failures**: Stop and report specific failures
- **Issue not found**: Validate issue exists before PR creation
- **Staging branch missing**: Error with available branches
- **🚨 CRITICAL**: Attempt to merge to main branch - BLOCKED with error message
- **🚨 SECURITY**: Any main branch reference - IMMEDIATE REJECTION with warning

## Integration

- **Before**: Use `/impl [issue-number]` to complete implementation
- **After**: Wait for team review and approval
- **🚨 TARGET**: STRICTLY `staging` branch ONLY - **NEVER** `main`
- **🚨 BLOCKED**: Any attempt to target main branch is REJECTED
- **Context**: PR resolves specific GitHub issue
- **Safety**: System enforces staging-only workflow with validation checks

## Branch Naming Requirements

Feature branches must follow pattern:
```
feature/task-{issue-number}-{description}
```

Examples:
- `feature/task-123-user-authentication`
- `feature/task-456-payment-webhook`
- `feature/task-789-database-migration`

## Important Notes

- **🚨 CRITICAL**: ALWAYS creates PR to staging branch - NEVER to main
- **🚨 FORBIDDEN**: Main branch merging is BLOCKED by system validation
- **🚨 SECURITY**: Any attempt to target main branch will be REJECTED
- **NEVER** merge PRs yourself - wait for team approval
- **100% validation** required before PR creation
- Feature branch must be pushed to remote
- Working directory must be clean
- PR must resolve a specific task issue
- **STAGING ONLY**: PRs are exclusively for staging branch review and testing

## Files

- Feature branches following naming convention
- GitHub Pull Requests - Code review and discussion
- GitHub Issues - Task definitions and requirements