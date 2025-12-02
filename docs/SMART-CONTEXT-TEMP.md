# Smart Context Template - Compact Version
**Compressed template for rapid context creation while maintaining comprehensive planning**

---

## 🎯 [CONTEXT-XXX] [Topic Name]

### Context Objective
**Goal:** [Primary objective of this context discussion]
**Value:** [Why this is important for the project]

### Current Status
**Phase:** `[Planning/Ready for Planning/Implementation Ready]`
**Last Updated:** [Date]

### Technical Analysis

#### 🔍 System Components
**Current State:**
- **Framework:** [Current framework and version]
- **Database:** [Database system and configuration]
- **External Services:** [Connected external services]
- **Available:** [Existing components and patterns]
- **Missing:** [Required components not yet available]

#### 🏗️ Architecture Impact
**System Components Affected:**
- [Component 1] - [impact description]
- [Component 2] - [impact description]

**API Endpoints Needed:**
- [ ] `METHOD /path` - [purpose]
- [ ] `METHOD /path` - [purpose]

**External Integrations:**
- [ ] [Service Name] - [integration point]
- [ ] [Service Name] - [integration point]

#### 🔧 Environment Requirements
**Critical Environment Variables:**
- [ ] VAR_NAME - [purpose and validation]
- [ ] VAR_NAME - [purpose and validation]

**Service Connectivity:**
- [ ] `/test-env redis` - Redis connection required
- [ ] `/test-env database` - Database connection required
- [ ] `/test-env gemini` - Gemini API access required

### Implementation Strategy

#### 🎯 Feature Breakdown
**High-Level Features:**
1. [Feature 1] - [description and acceptance criteria]
2. [Feature 2] - [description and acceptance criteria]
3. [Feature 3] - [description and acceptance criteria]

#### 📋 Atomic Tasks (Pre-generated)
Based on analysis, this will require approximately [X] atomic tasks:

**Backend Tasks:**
- [TASK-XXX] [Specific atomic task]
- [TASK-XXX] [Specific atomic task]
- [TASK-XXX] [Specific atomic task]

**Frontend Tasks (if applicable):**
- [TASK-XXX] [Specific atomic task]
- [TASK-XXX] [Specific atomic task]

#### 🧪 Testing Strategy
**Test Coverage Areas:**
- **Unit Tests:** [specific areas to cover]
- **Integration Tests:** [API/service integrations]
- **End-to-End Tests:** [complete user flows]
- **Performance Tests:** [critical paths to validate]

#### ⚠️ Risk Assessment
**High-Risk Areas:**
- [Risk 1] - [mitigation strategy]
- [Risk 2] - [mitigation strategy]

**Dependencies:**
- [External Dependency 1] - [fallback plan]
- [Internal Dependency 2] - [resolution needed]

### Planning Readiness

#### ✅ Requirements Checklist
- [ ] Functional requirements clearly defined
- [ ] Technical approach validated
- [ ] Dependencies identified and available
- [ ] Scope boundaries defined
- [ ] Risk mitigation planned
- [ ] Test strategy comprehensive

#### 🚀 Ready for Planning When:
- All checklist items above are marked ✅
- External services connectivity verified
- Stakeholder approval obtained
- Resource allocation confirmed

### Session Notes

**Discussion Points:**
- [Key decision 1] - [rationale]
- [Key decision 2] - [rationale]

**Open Questions:**
- [Question 1] - [what needs clarification]
- [Question 2] - [what needs validation]

**Next Steps:**
- [ ] Address open questions
- [ ] Complete remaining checklist items
- [ ] Use `/plan [task]` when ready for implementation

---

## 🎯 Context Generation Examples

### Simple API Feature:
```markdown
## 🎯 [CONTEXT-001] User authentication system

### Context Objective
**Goal:** Implement secure user authentication with JWT tokens
**Value:** Enable protected routes and user-specific features

### Technical Analysis

#### 🔍 System Components
**Current State:**
- **Framework:** Rust + Axum v0.7
- **Database:** PostgreSQL via Supabase
- **Available:** Basic API structure, validation patterns
- **Missing:** JWT library, password hashing

#### 🔧 Environment Requirements
**Critical Environment Variables:**
- [ ] JWT_SECRET_KEY - 256-bit secret for token signing
- [ ] DATABASE_URL - Users table access

### Implementation Strategy

#### 📋 Atomic Tasks (3 estimated)
- [TASK-001] Create users table and migration
- [TASK-002] Implement JWT token service
- [TASK-003] Create auth endpoints (login/register)

### Planning Readiness
**Ready for Planning When:**
- JWT secret key generated
- User table schema designed
```

### Complex System Feature:
```markdown
## 🎯 [CONTEXT-002] Payment processing system

### Context Objective
**Goal:** Complete payment processing with Stripe integration
**Value:** Enable paid features and subscription management

### Technical Analysis

#### 🔍 System Components
**Current State:**
- **Framework:** Rust + Axum
- **External Services:** Stripe API integration needed
- **Available:** Database schema patterns, API error handling
- **Missing:** Stripe SDK, webhook handling

#### ⚠️ Risk Assessment
**High-Risk Areas:**
- Stripe API rate limits - implement retry logic
- Payment security - PCI compliance considerations
- Webhook reliability - implement idempotency

### Implementation Strategy

#### 📋 Atomic Tasks (5 estimated)
- [TASK-001] Set up Stripe SDK and configuration
- [TASK-002] Create payment intents endpoint
- [TASK-003] Implement webhook handler
- [TASK-004] Add payment status tracking
- [TASK-005] Create payment management UI

### Planning Readiness
**Ready for Planning When:**
- Stripe API keys obtained
- Payment flow requirements finalized
- Webhook security strategy defined
```

---

**Template Size: ~150 lines vs 200+ lines (25% reduction)**
**All critical planning elements preserved**
**Auto-generation ready for task breakdown**