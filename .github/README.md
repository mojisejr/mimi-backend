# 🔒 GitHub Repository Protection

This repository enforces **strict staging-only workflow** with comprehensive branch protection rules.

## 🚨 CRITICAL RULES

### ❌ FORBIDDEN ACTIONS
- **NEVER** create PRs to `main` branch
- **NEVER** push directly to `main` branch
- **NEVER** bypass staging review process
- **NEVER** target any branch other than `staging`

### ✅ REQUIRED WORKFLOW
1. **Feature Development**: Work on `feature/task-{number}-{description}` branches
2. **PR Creation**: Create PR to `staging` branch ONLY
3. **Code Review**: Team review and approval on staging
4. **Testing**: Comprehensive testing on staging
5. **Merge**: Merge to staging after approval
6. **Production**: Separate process from staging to production

## 🛡️ Protection Mechanisms

### GitHub Workflows
- **main-branch-protection.yml**: Blocks all PRs targeting main
- **pr-validation.yml**: Enforces staging-only PR policy
- **branch-protection.yml**: Prevents direct main branch pushes

### Automated Enforcement
- **PR Target Validation**: Automatically blocks PRs to main
- **Branch Naming**: Enforces `feature/task-{number}-{description}` pattern
- **Direct Push Blocking**: Prevents direct pushes to main branch
- **Security Auditing**: Logs all access attempts

### Error Messages
When violations occur, you'll see:
```
🚨 CRITICAL ERROR: PRs CANNOT target main branch!
✅ REQUIRED: PR must target 'staging' branch
❌ FORBIDDEN: Targeting 'main' branch is not allowed
🔒 SAFETY: Main branch protection is enforced
```

## 🔄 Valid Workflow

```bash
# ✅ CORRECT: Create feature branch
git checkout -b feature/task-123-api-implementation

# ✅ CORRECT: Push and create PR to staging
git push origin feature/task-123-api-implementation
# Create PR: feature/task-123-api-implementation → staging

# ✅ CORRECT: PR targets staging branch
# GitHub CI validates and approves staging-only workflow
```

## ❌ Invalid Workflows

```bash
# ❌ WRONG: PR to main branch (BLOCKED)
gh pr create --base main  # ❌ BLOCKED BY WORKFLOWS

# ❌ WRONG: Direct push to main (BLOCKED)
git push origin main  # ❌ BLOCKED BY WORKFLOWS

# ❌ WRONG: Invalid branch naming
git checkout -b feature/api-implementation  # ❌ MISSING TASK NUMBER
```

## 📋 Branch Naming Convention

### Required Format
```
feature/task-{issue-number}-{description}
```

### Examples
- ✅ `feature/task-123-user-authentication`
- ✅ `feature/task-456-payment-webhook`
- ✅ `feature/task-789-database-migration`
- ❌ `feature/api-endpoints` (missing task number)
- ❌ `hotfix/bug-fixes` (wrong pattern)

## 🔍 Workflow Validation

### Automated Checks
- **Target Branch**: Must be `staging`
- **Source Branch**: Must follow naming convention
- **Issue Reference**: Branch should reference a task issue
- **PR Title**: Should include issue number

### Manual Review Process
1. **Code Review**: Team review on staging PR
2. **Testing**: Comprehensive testing on staging branch
3. **Approval**: Team approval required
4. **Merge**: Merge to staging after approval

## 🚀 Emergency Procedures

### If Main Branch Access Needed
1. **Contact Repository Owner**: For emergency access
2. **Security Review**: Required for any main branch changes
3. **Documentation**: Document reason for exception
4. **Audit Log**: Access is logged and reviewed

### For Production Deployment
- Separate deployment process from staging to production
- Manual approval required for production changes
- Rollback procedures in place

## 📞 Support

### For Workflow Issues
- **Repository Admins**: Contact for branch protection questions
- **GitHub Actions**: Review workflow logs for errors
- **Team Communication**: Use project channels for coordination

### Training Resources
- Review this README before creating PRs
- Check workflow logs for validation failures
- Follow project guidelines strictly

## 🔗 Related Files

- **`.claude/commands/pr.md`**: CLI command with staging-only enforcement
- **`.github/workflows/`**: Automated protection workflows
- **Project Guidelines**: Additional workflow documentation

---

**🔒 REMEMBER**: Staging-only workflow is enforced for repository safety and code quality.