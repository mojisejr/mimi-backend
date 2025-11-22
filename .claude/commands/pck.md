# /pck

Plan Check - วิเคราะห์ task issue และแสดงแผนการ implement พร้อมขั้นตอน workflow

## Usage

```
/pck [issue-number]        # วิเคราะห์ task issue และแสดงแผนการ implement
/pck [issue-number] [msg]  # วิเคราะห์พร้อมบริบทเพิ่มเติม
```

## Examples

```bash
/pck 123                           # วิเคราะห์ task #123 แสดงแผน
/pck 456 "ใช้ Rust async pattern" # วิเคราะห์พร้อม context เพิ่มเติม
```

## Output Format

### 📋 Task Summary
- Issue number, title, description
- Current status and requirements

### 🔍 Codebase Analysis
- Related files and modules
- Current architecture
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

### ✅ Validation Checklist
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

- **Use Before**: `/impl [issue-number]` - ให้ agent ตรวจสอบแผนก่อน implement
- **Replaces**: Manual planning process
- **Codebase Context**: Real-time analysis ของ codebase ปัจจุบัน

## Notes

- ✅ ไม่ update local files เลย - วิเคราะห์และตอบเท่านั้น
- ✅ ตรวจสอบ task issue ก่อน - ถ้า issue ไม่มี จะให้ error
- ✅ ทำตามแบบ response.instructions.md - ตรงประเด็น ไม่ out of scope
- ✅ ใช้ได้กับทุก task types

---

**Last Updated**: November 22, 2025
