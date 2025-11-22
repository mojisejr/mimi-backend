# /aud

Audit - วิเคราะห์ codebase ตอบคำถาม และเสนอแนวทางแก้ไข

## Usage

```
/aud [question]             # วิเคราะห์ codebase แล้วตอบคำถาม
/aud [context] "[question]" # วิเคราะห์พร้อมบริบท
```

## Examples

```bash
/aud "ทำไม queue ถึง fail?"
/aud "หา bottleneck" "ที่ไหนใน API ที่เด็ง"
/aud "แก้ไข error handling" "ในส่วนไหนของ code"
```

## Output Format

### 🔍 Codebase Analysis Summary
- Files analyzed
- Scope of investigation
- Key components found
- Relationships and dependencies

### 📍 Issues Found
For each issue:
- **Issue #**: Clear title
  - **Location**: ไฟล์ไหน บรรทัดไหน
  - **Current State**: สถานะปัจจุบัน
  - **Root Cause**: สาเหตุที่แท้จริง
  - **Why It Happens**: เหตุผลว่าทำไม
  - **Impact**: ส่งผลกระทบอะไร

### 💡 Fix Recommendations
For each fix:
- **Approach**: แนวทางการแก้
- **How to Fix**: ขั้นตอนการแก้ไข
- **Code Pattern**: code pattern ที่ควรใช้
- **Why This Fix**: เหตุผลของแนวทางนี้
- **Validation Steps**: วิธีตรวจสอบว่าแก้ได้

### 📊 Implementation Plan
- Step-by-step guide
- Files to modify
- Code changes needed
- Dependencies to consider
- Breaking changes (if any)

### ✅ Testing Strategy
- Unit tests needed
- Integration tests needed
- Manual verification steps
- Edge cases to cover
- Performance implications

### 🎯 Expected Outcome
- What will be improved
- Metrics to measure success
- Before/after comparison
- How to verify the fix works

### ⚠️ Risks & Considerations
- Potential side effects
- Dependencies to verify
- Backward compatibility
- Performance impact
- Security implications

## Supported Analysis Types

- 🐛 **Bug Analysis**: หาสาเหตุ error
- 📈 **Performance**: หา bottleneck
- 🔒 **Security**: หาช่องโหว่
- ♻️ **Refactoring**: suggest ปรับปรุง
- 🏗️ **Architecture**: วิเคราะห์ structure
- 🔗 **Integration**: ตรวจสอบ dependencies
- 📚 **Code Quality**: วิเคราะห์คุณภาพ

## Integration

- **Standalone**: ใช้ได้ทั้งอย่างเดียว
- **Before Planning**: `/pck` หลังจาก `/aud`
- **Before Implementation**: `/impl` หลังจาก `/aud`
- **Knowledge Capture**: ผล `/aud` สามารถส่วน `/kupdate` ได้

## Notes

- ✅ ไม่ update local files เลย - วิเคราะห์และตอบเท่านั้น
- ✅ ตรวจสอบ codebase ปัจจุบัน - real-time analysis
- ✅ ทำตามแบบ response.instructions.md - ตรงประเด็น ไม่ out of scope
- ✅ ใช้ได้กับทุก parts ของ codebase
- ✅ ผลวิเคราะห์สามารถเป็นฐานสำหรับ `/plan` ได้

---

**Last Updated**: November 22, 2025
