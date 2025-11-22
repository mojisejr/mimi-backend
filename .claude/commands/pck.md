# /pck

Plan Check - วิเคราะห์ GitHub task issue และแสดงแผนการ implement พร้อมขั้นตอน workflow

## Usage

```
/pck [issue-number]        # อ่าน GitHub issue #[number] และวิเคราะห์ task
/pck [issue-number] [msg]  # อ่าน GitHub issue พร้อมบริบทเพิ่มเติม
```

## Examples

```bash
/pck 123                           # อ่าน GitHub issue #123 และแสดงแผน
/pck 456 "ใช้ Rust async pattern" # อ่าน GitHub issue #456 พร้อม context เพิ่มเติม
```

## GitHub Integration 🔗

### Data Source
- **Repository**: mojisejr/mimi-backend
- **Source**: GitHub Issues (ONLY) - ไม่ใช่ local files
- **Access Method**: 
  - Primary: GitHub MCP tools (`mcp_github_github_issue_read`)
  - Fallback: `gh` command line tool (`gh issue view [number]`)

### Implementation Flow

1. **Read GitHub Issue**:
   ```bash
   # Using gh command
   gh issue view [issue-number] --json title,body,labels,state
   
   # Using GitHub MCP
   mcp_github_github_issue_read(method: "get", owner: "mojisejr", repo: "mimi-backend", issue_number: [number])
   ```

2. **Verify Issue Type**:
   - Must have `task` label
   - Must NOT be `context` or other types
   - Status must be `OPEN`

3. **Extract Data**:
   - Title: Task description
   - Body: Full requirements and specifications
   - Labels: Task metadata (manual/copilot mode)
   - Links: Related issues and PRs

## Output Format

### 📋 GitHub Issue Summary
- Issue #[number] - [Status: OPEN/CLOSED]
- Title: [Task description]
- Labels: task, [mode-assignment]
- Description: Full task requirements
- Related Issues/PRs: Links if available

### 🔍 Codebase Analysis
- Related files and modules (based on task description)
- Current architecture relevant to this task
- Dependencies and constraints
- Potential impact areas

### 📍 Implementation Steps
For each step:
- **Step N: [Title]**
  - **What**: อธิบายว่าจะทำอะไร
  - **How**: วิธีการทำและคำสั่ง/code pattern
  - **Why**: เหตุผลที่ต้องทำแบบนี้
  - **Outcome**: ผลลัพธ์ที่คาดหวัง
  - **Validate**: วิธีตรวจสอบ/ทดสอบ

### ✅ Validation Checklist (from GitHub Issue)
- Build validation (cargo build --release)
- Lint validation (cargo clippy -- -D warnings)
- Format check (cargo fmt -- --check)
- Type check (cargo check)
- Test execution
- Integration verification

### 🎯 Expected Result
- What will be delivered
- How to test/verify
- Success criteria

### ⚠️ Risks & Considerations
- Potential blockers
- Dependencies to verify
- Performance implications
- Security considerations

### 📊 Effort Estimate
- Time complexity
- Code changes scope
- Testing requirements

## Integration

- **Source**: GitHub Issues only - read from repository
- **Use Before**: `/impl [issue-number]` - ให้ agent ตรวจสอบแผนก่อน implement
- **Replaces**: Manual GitHub issue reading process
- **Codebase Context**: Real-time analysis ของ codebase ปัจจุบัน
- **Access**: GitHub MCP or `gh` command

## Implementation

### ⚠️ MANDATORY WORKFLOW (ต้องทำตามลำดับเสมอ - ห้ามข้าม)

#### Step 1️⃣: Read GitHub Issue (REQUIRED)
```bash
# Using gh command
gh issue view [issue-number] --json title,body,labels,state

# Using GitHub MCP
mcp_github_github_issue_read(method: "get", owner: "mojisejr", repo: "mimi-backend", issue_number: [number])
```
**ต้องหา:**
- ✅ Issue title and description
- ✅ Task requirements and acceptance criteria
- ✅ Labels (must have `task`)
- ✅ Issue status (must be OPEN)
- ❌ **STOP HERE ถ้าไม่ใช่ task issue** - ให้ error

#### Step 2️⃣: Analyze Codebase (REQUIRED - ห้ามข้าม!)
**หลังจากอ่าน GitHub issue สำเร็จแล้ว ต้องทำการวิเคราะห์ codebase:**

1. **Scan Related Files**:
   ```bash
   # Search for related code patterns
   grep -r "[search-term-from-issue]" src/
   find src/ -name "*.rs" -type f | grep -E "[pattern]"
   ```

2. **Understand Current Architecture**:
   - Read relevant source files mentioned in issue
   - Check existing implementations
   - Verify dependencies in Cargo.toml
   - Understand existing patterns and conventions

3. **Identify Impact Areas**:
   - Files that will be modified
   - Functions/modules affected
   - Dependencies needed
   - Tests that might be impacted
   - Performance considerations

4. **Extract Implementation Constraints**:
   - Technology stack being used
   - Existing patterns to follow
   - Limitations from architecture
   - Performance requirements from issue

**ต้องเจอ:**
- ✅ Related source files (specific paths)
- ✅ Current implementation patterns
- ✅ Dependencies and constraints
- ✅ Files that need modification
- ✅ Existing code to reference

#### Step 3️⃣: Answer with Complete Analysis
**เท่านั้นถึงจะตอบได้:**
- GitHub issue summary (จากขั้นตอน 1)
- Codebase analysis (จากขั้นตอน 2)
- Implementation steps ที่ชัดเจน
- Validation checklist
- Expected results

### ⛔ ห้ามกระทำการเหล่านี้:
- ❌ ตอบโดยไม่อ่าน GitHub issue
- ❌ ข้ามการอ่าน GitHub issue และไปตอบตรงๆ
- ❌ ให้ implementation steps โดยไม่วิเคราะห์ codebase
- ❌ ยกตัวอย่าง code ที่ไม่ใช่จากการวิเคราะห์ codebase ปัจจุบัน
- ❌ สรุปผล โดยไม่มี codebase context

### Prerequisites Check
1. **GitHub CLI**: Verify `gh` command is available (fallback)
2. **Repository Access**: Can read GitHub issues from mojisejr/mimi-backend
3. **Issue Validation**: Verify issue exists and is a task type

### Error Handling
- Issue not found: Clear error with available issues list
- Not a task issue: Warning with correction
- Access denied: Helpful error message
- GitHub MCP unavailable: Use `gh` command as fallback
- Codebase analysis failed: Explain what couldn't be found and why

## Notes

- ✅ **ข้อมูลมาจาก GitHub Issues เท่านั้น** - ไม่ใช้ local files
- ✅ **วิเคราะห์โดยอ่าน issue ตรงจาก GitHub** - ใช้ GitHub MCP หรือ `gh` command
- ✅ **ไม่ update GitHub issue** - เพียงแค่อ่านและวิเคราะห์เท่านั้น
- ✅ **ตรวจสอบ issue type ก่อน** - ต้องเป็น task label
- ✅ **ทำตามแบบ response.instructions.md** - ตรงประเด็น ไม่ out of scope
- ✅ **ใช้ได้กับทุก task types** - อ่านจาก GitHub issue เสมอ

---

**Last Updated**: November 22, 2025
**GitHub Integration**: Full GitHub MCP + `gh` command support
