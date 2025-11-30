# [CONTEXT] Tarot Reading API & Agent Pipeline (Sync MVP)

## 🎯 CONTEXT OBJECTIVE
- สร้าง API สำหรับอ่านไพ่ทาโรต์แบบ sync (user ส่งคำถาม → agent pipeline → คืนผลลัพธ์ทันที)
- ไม่ต้องมี JWT/auth, ไม่ต้อง async/queue
- เตรียมโครงสร้างสำหรับต่อยอดเป็น async/queue/worker ใน phase ถัดไป

## 📝 DISCUSSION LOG
- 2025-11-30 15:00: สร้าง context สำหรับ vertical slice "อ่านไพ่ทาโรต์"
- Codebase snapshot: มีแค่ handler function ใน API, agent pipeline พร้อม, ยังไม่มี routing/logic/response
- Next: ออกแบบ routing, handler, integration agent pipeline, response format

## 📈 ACCUMULATED CONTEXT
- API ยังไม่มี routing/logic สำหรับอ่านไพ่
- agent pipeline (reading_agent.rs) พร้อมสำหรับเรียกใช้งาน
- ยังไม่มี model/response format ตาม spec
- ยังไม่มี test สำหรับ API นี้

## 🧪 TEST-FIRST REQUIREMENTS (MANDATORY)
- [ ] Unit test: ส่ง question แล้วได้ response ที่มีผลลัพธ์ (sync)
- [ ] Integration test: POST /api/tarot/read แล้วได้ผลลัพธ์ถูกต้อง
- [ ] Edge case test: ส่ง question ว่าง/สั้น/ยาวเกินไป ต้อง reject
- [ ] Agent test: reading_agent.generate_reading() คืนผลลัพธ์ที่มี card และข้อความ

## 📋 PLANNING READINESS CHECKLIST
- [x] Objective ชัดเจน (อ่านไพ่ sync, ไม่ JWT)
- [x] Codebase snapshot ล่าสุด
- [x] Test-first requirements ครบ
- [x] Milestone: API routing, handler, agent integration, response format, test
- [x] Security: ไม่มี auth, input validation ขั้นต่ำ
- [x] Output: คืนผลลัพธ์ทันที, response format ตาม spec

---

**สรุป context ล่าสุด:**  
จะเริ่ม vertical slice "อ่านไพ่ sync" ก่อน  
เตรียม test-first, routing, handler, agent integration  
ยังไม่ต้องทำ async/queue/JWT
