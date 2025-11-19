# Product Requirements Document (PRD)
## MimiVibe Backend - Tarot Reading System

### Project Overview
สร้างระบบ backend สำหรับดูดวงไพ่ทาโร่ โดยใช้ Google Gemini LLM และ implement workflow engine แบบ LangGraph ในภาษา Rust สำหรับ MimiVibe application

---

## System Architecture

### High-Level Workflow
```
User (Frontend) → API Gateway → Authentication → Agent Pipeline → Response
                                        ↓
                                Question Filter → Question Analyzer →
                                Random Card Count → Reading Agent → JSON Response
```

### Agent Pipeline Flow
1. **Authentication Layer** - Simple API key validation (environment-based)
2. **Input Validation** - Question length constraints (8-180 characters)
3. **Agent 1: Question Filter** - Validate question appropriateness
4. **Agent 2: Question Analyzer** - Analyze question context and emotions
5. **Random Card Count** - Generate 3 or 5 cards count
6. **Agent 3: Reading Agent** - Generate tarot reading prediction
7. **Response Formatting** - Return structured JSON response

---

## Functional Requirements

### 1. Authentication & Authorization
- **Authentication Method**: Simple API Key validation (Phase 1 - Internal Use Only)
- **Access Control**: ผู้ใช้ที่มี API key ถูกต้องเท่านั้น
- **Token Validation**: String comparison with environment-based keys

#### Phase 1 API Key Strategy (Internal Applications Only)

**Environment-Based Keys:**
```bash
# .env.local (not committed to repository)
TTRT_DEV_API_KEY=ttrt_dev_a1b2c3d4e5f6
TTRT_STAGING_API_KEY=ttrt_staging_b7c8d9e0f1g2
TTRT_PROD_API_KEY=ttrt_prod_h3i4j5k6l7m8
```

**Key Selection Logic:**
- Development Environment: `TTRT_DEV_API_KEY`
- Staging Environment: `TTRT_STAGING_API_KEY`
- Production Environment: `TTRT_PROD_API_KEY`
- Default (fallback): Development key

**Implementation Approach:**
- Static API keys stored in environment variables
- Simple string comparison validation
- No database required for Phase 1
- Basic rate limiting (100 requests/minute per key)
- HTTPS enforcement in production

**Security Considerations:**
- Keys stored in `.env.local` (not committed to git)
- Different keys per environment
- Long random strings (32+ characters)
- Simple, fast, and secure for internal use

**Future Expansion (Phase 2):**
- Multi-tier API key system
- Third-party developer access
- Expiration dates and renewal
- Developer portal for key generation

#### Frontend Integration
```javascript
// Environment-specific API key configuration
const API_KEYS = {
  development: process.env.TTRT_DEV_API_KEY,
  staging: process.env.TTRT_STAGING_API_KEY,
  production: process.env.TTRT_PROD_API_KEY
};

const API_KEY = API_KEYS[process.env.NODE_ENV] || API_KEYS.development;
```

### 2. Input Requirements
- **Question Format**: String
- **Minimum Length**: 8 ตัวอักษร
- **Maximum Length**: 180 ตัวอักษร
- **Validation**: ปฏิเสธคำถามที่ไม่ตรงตามข้อกำหนด

### 3. Agent Pipeline Specifications

#### Agent 1: Question Filter
- **Purpose**: วิเคราะห์ว่าคำถามเหมาะสมกับการดูดวงหรือไม่
- **Prompt Reference**: `/prompts/QUESTION-FILTER.md`
- **Input**: User's original question
- **Output**: Boolean (allow/deny) + reasoning
- **Error Handling**: ปฏิเสธคำถามที่ไม่เหมาะสมพร้อมเหตุผล

#### Agent 2: Question Analyzer
- **Purpose**: วิเคราะห์ข้อมูลเชิงลึกจากคำถาม
- **Prompt Reference**: `/prompts/QUESTION-ANALYSIS.md`
- **Input**: Filtered question
- **Output**:
  - Question category (เช่น ความรัก, การงาน, สุขภาพ)
  - Emotional context (อารมณ์ของผู้ถาม)
  - Time frame (ช่วงเวลาที่ต้องการคำตอบ)
  - Key themes/concerns

#### Agent 3: Reading Agent
- **Purpose**: ทำนายดวงจากไพ่ทาโร่ (แม่หมอ)
- **Prompt Reference**: `/prompts/READING-AGENT.md`
- **Input**:
  - Analysis from Question Analyzer
  - Card count (3 or 5 cards - randomly generated)
- **Output**: Structured JSON containing:
  - Selected tarot cards
  - Card interpretations
  - Overall prediction
  - Advice/guidance

### 4. Random Card Generation
- **Card Count Options**: 3 หรือ 5 ใบเท่านั้น
- **Generation Method**: สุ่มแบบ 50/50
- **Card Selection**: Random from traditional 78-card tarot deck
- **Integration**: Pass card count to Reading Agent

### 5. Response Format
```json
{
  "success": true,
  "data": {
    "original_question": "string",
    "question_analysis": {
      "category": "string",
      "emotion": "string",
      "timeframe": "string",
      "themes": ["string"]
    },
    "tarot_reading": {
      "header": "คำทักทายและทวนคำถามพร้อมเปิดประเด็นชวนคิด (1 ประโยคในภาษาเดียวกันกับคำถาม)",
      "cards_reading": [
        {
          "id": 1,
          "name": "The Sun",
          "displayName": "ไพ่พระอาทิตย์",
          "imageUrl": "the_sun.png",
          "position": 1,
          "shortMeaning": "ความสำเร็จ ความสุข แสงสว่าง",
          "keywords": "ความสำเร็จ, ความสุข, แสงสว่าง, โอกาสดี"
        }
      ],
      "reading": "คำทำนายหลักจากไพ่ทั้งหมดในภาพรวม **ที่ตอบคำถามหลักอย่างชัดเจนและฟันธง (ใช่/ไม่ใช่/มีโอกาสสูง/มีโอกาสน้อย/ไม่แน่นอน) ในประโยคแรกสุด โดยห้ามใช้คำว่า 'อาจจะ' หรือ 'เป็นไปได้ว่า' หากไพ่บ่งชี้แนวโน้มชัดเจน** หากไพ่บ่งชี้ก้ำกึ่ง ให้ระบุแนวโน้มความน่าจะเป็น โดยเชื่อมโยงกับสถานการณ์และให้มุมมองลึกซึ้ง (1 ย่อหน้าในภาษาเดียวกันกับคำถาม อาจมีคำถามชวนคิดแทรก)",
      "suggestions": [
        "คำแนะนำเชิงปฏิบัติที่ชัดเจน ทันสมัย และเป็นกันเอง เน้นสิ่งที่ควรทำ (2-3 ข้อในภาษาเดียวกันกับคำถาม)",
        "ข้อแนะนำ 2",
        "ข้อแนะนำ 3"
      ],
      "next_questions": [
        "คำถามแนะนำที่สามารถถามต่อได้ 3 คำถาม (ในภาษาเดียวกันกับคำถาม)",
        "คำถาม 2",
        "คำถาม 3"
      ],
      "final": [
        "สรุปผลลัพธ์ของคำถามหลักในรูปแบบ **กระชับ ชัดเจน ตรงประเด็น ภายใน 1 ประโยคเท่านั้น** เช่น 'คุณมีโอกาสได้ตามที่คาดหวังไว้ค่ะ', 'แนวโน้มยังไม่ชัดเจนนักในช่วงนี้นะคะ' หรือ 'ดูแล้วคุณอาจต้องใช้เวลาอีกหน่อยค่ะ'",
        "หลีกเลี่ยงการขยายความหรือให้คำแนะนำซ้ำซ้อน เพราะคำทำนายหลักได้อธิบายไว้แล้ว — จุดประสงค์ของส่วนนี้คือการ **ย้ำคำตอบแบบฟันธงอีกครั้ง** เพื่อให้ลูกดวงเข้าใจอย่างชัดเจนและมั่นใจค่ะ"
      ],
      "end": "ข้อความปิดท้ายอย่างอบอุ่นและเป็นกันเอง (1 ประโยคในภาษาเดียวกับคำถาม)",
      "notice": "ข้อความเตือนให้ใช้วิจารณญาณในการรับคำทำนาย (1 ประโยคในภาษาเดียวกับคำถาม)"
    }
  }
}
```

### Response Format Details
- **header**: คำทักทายและทวนคำถามพร้อมเปิดประเด็นชวนคิด (1 ประโยค)
- **cards_reading**: Array ของไพ่ที่ถูกหยิบ (3 หรือ 5 ใบ)
  - **id**: รหัสไพ่ (number)
  - **name**: ชื่อไพ่ภาษาอังกฤษ (The Sun, The Moon, etc.)
  - **displayName**: ชื่อไพ่ภาษาไทย (ไพ่พระอาทิตย์, ไพ่พระจันทร์)
  - **imageUrl**: ชื่อไฟล์ภาพไพ่ (the_sun.png)
  - **position**: ตำแหน่งของไพ่ (1, 2, 3, 4, 5)
  - **shortMeaning**: ความหมายสั้นๆ ของไพ่
  - **keywords**: คำสำคัญของไพ่
- **reading**: คำทำนายหลักที่ตอบคำถามอย่างชัดเจนและฟันธง
- **suggestions**: คำแนะนำเชิงปฏิบัติ 2-3 ข้อ
- **next_questions**: คำถามแนะนำที่สามารถถามต่อได้ 3 คำถาม
- **final**: สรุปผลลัพธ์แบบกระชับชัดเจน 1 ประโยค
- **end**: ข้อความปิดท้ายอบอุ่น
- **notice**: ข้อความเตือนให้ใช้วิจารณญาณ

---

## Technical Requirements

### Technology Stack
- **Language**: Rust
- **Web Framework**: **Axum** (Selected for JavaScript developer familiarity and excellent performance)
- **Async Runtime**: Tokio
- **HTTP Middleware**: Tower & Tower-HTTP
- **LLM Integration**: Google Gemini API
- **Agent Orchestration**: **LangChain Rust** (LangGraph-style workflow engine for agent pipeline)
- **Authentication**: **Multi-Provider JWT System** (Clerk/Auth0/Firebase support), Environment-based API Keys (Phase 1)
- **Database**: **Supabase PostgreSQL** with **SQLx** for type-safe database operations
- **HTTP Client**: Reqwest (for Gemini API & Stripe integration)
- **Serialization**: Serde (JSON request/response handling)
- **Configuration**: Environment variables
- **Payment Gateway**: Stripe (Card processing, PromptPay in Phase 2)

### Key Components

#### 1. API Gateway (Axum-based)
- **RESTful API endpoints**: Router-based route definitions
- **Middleware Stack**: Tower layers for cross-cutting concerns
  - Tower-HTTP CORS layer (frontend integration)
  - Tower-HTTP Compression layer (response optimization)
  - Tower-HTTP Trace layer (request logging)
  - Custom API key validation middleware
  - Rate limiting middleware (100 req/min per API key)
- **Request Validation**: Axum extractors with Serde validation
- **Error Handling**: Centralized error response formatting

#### 2. Agent Engine (LangChain Rust-based)
- **Framework**: LangChain Rust (provides LangGraph-style orchestration)
- **Sequential Chain**: SequentialChain for Agent 1 → Agent 2 → Agent 3 workflow
- **State Management**: LLMChain with message history and context passing
- **Agent System**: AgentExecutor with tool-based agent architecture
- **Prompt Templates**: System prompts with templating for each agent
- **Error Handling**: Chain-level error handling with fallback strategies

**LangChain Rust Implementation**:
```rust
// Sequential Agent Pipeline
use langchain_rust::chains::sequential::SequentialChain;
use langchain_rust::chains::llm::LLMChain;
use langchain_rust::agents::AgentExecutor;

// Agent Pipeline: Question Filter → Question Analyzer → Reading Agent
let tarot_pipeline = SequentialChain::new(vec![
    question_filter_chain,    // Agent 1: Validate question appropriateness
    question_analyzer_chain,  // Agent 2: Analyze context and emotions
    reading_agent_chain       // Agent 3: Generate tarot reading with cards
]);

// Agent with Tools for random card selection
let agent = Agent::create_from_llm_and_tools(gemini_llm, vec![
    random_card_selector_tool,  // 3 or 5 cards random selection
    gemini_api_tool            // Google Gemini LLM integration
]).await.unwrap();
```

#### 3. LLM Integration (LangChain Rust + Gemini API)
- **Framework**: LangChain Rust LLM abstraction layer
- **Gemini Integration**: Custom Gemini LLM implementation for LangChain Rust
- **HTTP Client**: LangChain Rust built-in HTTP client with Gemini API integration
- **Request Builder**: Structured prompt formatting through LangChain templates
- **Response Parser**: LangChain Rust response parsing and JSON validation
- **Error Handling**: Chain-level error handling with automatic retry logic
- **Streaming Support**: LangChain Rust async response streaming for long operations

**Custom Gemini LLM Implementation**:
```rust
use langchain_rust::language_models::llm::LLM;

pub struct GeminiLLM {
    client: gemini_api::Client,
    model: String,
}

impl LLM for GeminiLLM {
    async fn invoke(&self, prompt: &str) -> Result<String, LLMError> {
        // LangChain Rust handles the HTTP request/response cycle
        // Custom implementation for Gemini API integration
    }
}
```

#### 4. Data Layer (SQLx + Supabase)
- **Database**: Supabase PostgreSQL with SQLx for type-safe operations
- **Connection Pooling**: SQLx connection pool (20 max, 5 min connections)
- **User Management**: Users, wallet balances, authentication data
- **Transaction Records**: Payment history, star/coin transactions
- **Tarot Data**: Reading history, card selections, agent pipeline results
- **Analytics**: Payment analytics, user engagement metrics
- **Migrations**: SQLx built-in migration system
- **Query Optimization**: Compile-time query validation with SQLx

### Performance Requirements
- **Response Time**: < 5 seconds per reading (including LLM processing)
- **API Response Time**: < 200ms for non-LLM endpoints (wallet, auth, static data)
- **Concurrent Users**: Support 100+ simultaneous requests via Tokio async runtime
- **Availability**: 99.9% uptime with graceful degradation
- **Memory Usage**: < 512MB per instance with efficient async handling
- **API Rate Limit**: 100 requests/minute per API key (configurable)
- **LLM Timeout**: 30 seconds maximum for Gemini API calls
- **Database Performance**: < 50ms query response time (when implemented)

---

## LangChain Rust Integration Benefits

### Why LangChain Rust for MimiVibe Backend

**1. LangGraph-style Agent Pipeline**
- **Sequential Chains**: Perfect match for Agent 1 → Agent 2 → Agent 3 workflow
- **State Management**: Built-in message history and context passing between agents
- **Error Handling**: Chain-level error propagation with fallback strategies
- **Familiar Pattern**: Similar concepts to LangGraph for developers with Python experience

**2. Gemini API Integration**
```rust
// Custom LLM implementation for Google Gemini
impl LLM for GeminiLLM {
    async fn invoke(&self, prompt: &str) -> Result<String, LLMError> {
        // Direct integration with Google Gemini API
        // Automatic retry logic and error handling
    }
}
```

**3. Agent Architecture Benefits**
- **Tool System**: Random card selection as a tool within the agent framework
- **Prompt Templates**: System prompts from `/prompts/` directory easily integrated
- **Modular Design**: Each agent (filter, analyzer, reader) as separate chain
- **Scalability**: Easy to add new agents or modify existing workflow

**4. Production-Ready Features**
- **Async Support**: Full tokio async/await support
- **Type Safety**: Rust's type system with LangChain's abstractions
- **Performance**: Efficient memory usage and fast execution
- **Reliability**: Built-in retry mechanisms and error recovery

**Implementation Example**:
```rust
// Complete tarot reading pipeline
pub struct TarotReadingPipeline {
    filter_chain: LLMChain<GeminiLLM>,      // Agent 1: Question Filter
    analyzer_chain: LLMChain<GeminiLLM>,    // Agent 2: Question Analyzer
    reader_chain: LLMChain<GeminiLLM>,      // Agent 3: Reading Agent
    card_selector: RandomCardTool,         // Random 3/5 card selection
}

impl TarotReadingPipeline {
    pub async fn process_reading(&self, question: String) -> Result<TarotResponse, Error> {
        // Sequential execution with state passing
        let filtered = self.filter_chain.invoke(&question).await?;
        let analyzed = self.analyzer_chain.invoke_with_context(&filtered).await?;
        let card_count = self.card_selector.random_count();
        let reading = self.reader_chain.invoke_with_cards(analyzed, card_count).await?;

        Ok(reading)
    }
}
```

---

## Prompt Security & IP Protection

### Protecting Proprietary Prompt Templates

**Security Requirement**: All agent prompts contain proprietary know-how and must be protected from unauthorized access or copying while maintaining easy update capability.

#### **Recommended Approach: Environment-based Encryption**

**1. Encrypted Environment Variables**
```bash
# .env.local (never committed to repository)
PROMPT_QUESTION_FILTER=$(echo "ทักทาย..." | base64)
PROMPT_QUESTION_ANALYSIS=$(echo "วิเคราะห์..." | base64)
PROMPT_READING_AGENT=$(echo "ทำนาย..." | base64)

# Optional: API-based prompt management
PROMPT_SERVICE_URL=https://api.mimivibe.com/prompts
PROMPT_API_KEY=secure_prompt_api_key
```

**2. Prompt Manager Implementation**
```rust
// src/utils/prompt_manager.rs
use std::env;
use base64;

pub struct PromptManager {
    pub question_filter: String,
    pub question_analysis: String,
    pub reading_agent: String,
}

impl PromptManager {
    pub fn from_env() -> Result<Self, PromptError> {
        Ok(Self {
            question_filter: decode_base64(&env::var("PROMPT_QUESTION_FILTER")?)?,
            question_analysis: decode_base64(&env::var("PROMPT_QUESTION_ANALYSIS")?)?,
            reading_agent: decode_base64(&env::var("PROMPT_READING_AGENT")?)?,
        })
    }

    pub async fn from_remote_service() -> Result<Self, PromptError> {
        // Fetch from secure prompt management service
        let client = reqwest::Client::new();
        let response = client
            .get(&env::var("PROMPT_SERVICE_URL")?)
            .header("Authorization", format!("Bearer {}", env::var("PROMPT_API_KEY")?))
            .send()
            .await?;

        let prompts: PromptConfig = response.json().await?;
        Ok(Self {
            question_filter: prompts.question_filter,
            question_analysis: prompts.question_analysis,
            reading_agent: prompts.reading_agent,
        })
    }
}

fn decode_base64(encoded: &str) -> Result<String, PromptError> {
    let decoded = base64::decode(encoded)?;
    Ok(String::from_utf8(decoded)?)
}
```

**3. Agent Integration with Secure Prompts**
```rust
// src/agents/question_filter.rs
use crate::utils::prompt_manager::PromptManager;

pub struct QuestionFilter {
    prompt: String,
}

impl QuestionFilter {
    pub fn new() -> Result<Self, PromptError> {
        let prompt_manager = PromptManager::from_env()?;
        Ok(Self {
            prompt: prompt_manager.question_filter,
        })
    }

    pub async fn from_remote() -> Result<Self, PromptError> {
        let prompt_manager = PromptManager::from_remote_service().await?;
        Ok(Self {
            prompt: prompt_manager.question_filter,
        })
    }

    pub async fn validate_question(&self, question: &str) -> Result<bool, PromptError> {
        let full_prompt = format!("{}\n\nQuestion: {}", self.prompt, question);
        // Process with Gemini LLM
        let response = self.gemini_llm.invoke(&full_prompt).await?;

        // Parse response for allow/deny decision
        self.parse_filter_response(&response)
    }
}
```

#### **Deployment Strategy**

**Development Environment:**
```bash
# .env.local (development)
PROMPT_QUESTION_FILTER=SGVsbG8gSSBhbSBhIHRhcm90IHJlYWRlci4uLg==
PROMPT_QUESTION_ANALYSIS=QW5hbHl6ZSB0aGUgcXVlc3Rpb24gY29udGV4dC4uLg==
PROMPT_READING_AGENT=R2VuZXJhdGUgdGFyb3QgcmVhZGluZyB3aXRoIGNhcmRzLi4uC```

**Production Environment:**
```bash
# Environment variables or secret management
# AWS Secrets Manager, Google Secret Manager, or HashiCorp Vault
aws secretsmanager get-secret-value --secret-id mimivibe/prompts
```

#### **Update Workflow**

**1. Local Testing:**
```bash
# Update prompt in .env.local
PROMPT_QUESTION_FILTER=$(echo "New improved prompt..." | base64)

# Test locally
cargo test
```

**2. Production Update:**
```bash
# Update via secure admin dashboard or API
curl -X POST https://api.mimivibe.com/admin/prompts \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"agent": "question_filter", "prompt": "New improved prompt..."}'
```

#### **Security Benefits**

**✅ IP Protection**: Prompts never exposed in source code repository
**✅ Easy Updates**: Change prompts without code deployment
**✅ Version Control**: Track prompt changes through audit logs
**✅ Access Control**: Only authorized personnel can update prompts
**✅ Environment Isolation**: Different prompts for dev/staging/prod
**✅ Backup & Recovery**: Encrypted backups of prompt configurations

#### **Dependencies for Prompt Security**

```toml
[dependencies]
# Prompt encryption/decryption
base64 = "0.21"
aes-gcm = "0.10"  # For AES encryption (optional)

# Remote prompt management
reqwest = { version = "0.11", features = ["json"] }

# Secure configuration
dotenv = "0.15"
```

#### **Template Structure**

**`.env.template` (committed to repository):**
```bash
# Template for prompt configuration
# Do not commit actual prompt values - use encrypted values in production

PROMPT_QUESTION_FILTER=BASE64_ENCODED_QUESTION_FILTER_PROMPT
PROMPT_QUESTION_ANALYSIS=BASE64_ENCODED_QUESTION_ANALYSIS_PROMPT
PROMPT_READING_AGENT=BASE64_ENCODED_READING_AGENT_PROMPT

# Optional: Remote prompt service configuration
PROMPT_SERVICE_URL=https://your-prompt-service.com/api/prompts
PROMPT_API_KEY=your_secure_prompt_api_key
```

---

## Error Handling

### Axum Error Handling Architecture

#### Error Types & HTTP Status Mapping

**1. Authentication Errors (401/403)**
- Invalid API key → `401 Unauthorized`
- Expired/Revoked API key → `403 Forbidden`
- Missing API key → `401 Unauthorized`
- Rate limit exceeded → `429 Too Many Requests`

**2. Validation Errors (400)**
- Question too short (< 8 chars) → `400 Bad Request`
- Question too long (> 180 chars) → `400 Bad Request`
- Invalid JSON format → `400 Bad Request`
- Missing required fields → `400 Bad Request`

**3. Agent Pipeline Errors (422/502)**
- Question filter rejection → `422 Unprocessable Entity`
- LLM API failures → `502 Bad Gateway`
- LLM timeout errors → `504 Gateway Timeout`
- Invalid LLM responses → `502 Bad Gateway`
- Agent execution failures → `500 Internal Server Error`

**4. Payment Errors (400/402/502)**
- Invalid payment amount → `400 Bad Request`
- Stripe API failures → `502 Bad Gateway`
- Card declined → `402 Payment Required`
- Insufficient stars → `400 Bad Request`

**5. System Errors (500)**
- Database connection issues → `500 Internal Server Error`
- Configuration errors → `500 Internal Server Error`
- Memory/CPU overload → `503 Service Unavailable`
- Internal server errors → `500 Internal Server Error`

#### Axum Error Response Format

```rust
// Unified error response structure
#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
    details: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
    request_id: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_detail) = match self {
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message: msg,
                    details: None,
                    timestamp: chrono::Utc::now(),
                    request_id: None,
                }
            ),
            // ... other error variants
        };

        let body = Json(ErrorResponse {
            success: false,
            error: error_detail,
        });

        (status, body).into_response()
    }
}
```

**Example API Error Responses:**

```json
// Validation Error (400)
{
  "success": false,
  "error": {
    "code": "QUESTION_TOO_SHORT",
    "message": "Question must be at least 8 characters long",
    "details": "Received 5 characters, minimum required is 8",
    "timestamp": "2025-11-19T10:30:00Z",
    "request_id": "req_123456789"
  }
}

// Payment Error (402)
{
  "success": false,
  "error": {
    "code": "PAYMENT_DECLINED",
    "message": "Payment method was declined",
    "details": "Card issuer declined the transaction",
    "timestamp": "2025-11-19T10:35:00Z",
    "request_id": "req_123456790"
  }
}

// Rate Limit Error (429)
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "API rate limit exceeded",
    "details": "Maximum 100 requests per minute allowed for this API key",
    "timestamp": "2025-11-19T10:40:00Z",
    "request_id": "req_123456791"
  }
}

---

## Security Considerations

### API Security
- JWT token validation
- Rate limiting per user
- Input sanitization
- SQL injection prevention
- Request size limits

### Data Privacy
- Question logging consent
- Personal data minimization
- Secure storage practices
- GDPR compliance considerations

---

## Future Enhancements

### Phase 2 Features
- User accounts and profiles
- Reading history storage
- Card spread options (Celtic Cross, etc.)
- Multi-language support
- Premium tier features

### Phase 3 Features
- Real-time notifications
- Social sharing capabilities
- Advanced analytics
- Mobile app integration
- Voice input support

---

## Payment & Gamification System

### Currency System Design

#### 1. Twin-Token Economy
**Star (ดาว)** - Hard Currency
- **Purpose**: ใช้ซื้อคำถามทาโร่ (1 Star = 1 คำถาม)
- **Acquisition**: ซื้อด้วยเงินจริง (in-app purchase)
- **Characteristics**:
  - มีมูลค่าคงที่
  - สามารถซื้อขาดได้เท่านั้น
  - ไม่สามารถหาได้จากการเล่นเกม

**Coin (เหรียญ)** - Soft Currency
- **Purpose**: สะสมเพื่อแลกเป็น Star
- **Acquisition**: ได้จากการชวนเพื่อน, โบนัสพิเศษ
- **Characteristics**:
  - สามารถหาได้จากกิจกรรมในแอป
  - มีอัตราการแลกเปลี่ยนที่ปรับได้
  - ใช้เพื่อลดความลังเลในการเริ่มใช้งาน

#### 2. Invitation Reward System

**Basic Reward Mechanics**
- **Reward**: สุ่มได้ Coin ไม่เกิน 50% ของค่าใช้จ่ายในการแลก 1 Star
- **Example**: ถ้า 1 Star = 10 Coin → การ invite จะได้ 3-5 Coin (สุ่ม)
- **Purpose**: สร้างความตื่นเต้น ลดความน่าเบื่อ รักษาคุณค่าของ Star

**Advanced Reward Tiers**
```
Bronze Tier (1-5 invites):
- 3-5 Coin per invite
- Basic reward rate

Silver Tier (6-15 invites):
- 4-6 Coin per invite
- +20% bonus reward

Gold Tier (16+ invites):
- 5-8 Coin per invite
- +50% bonus reward
```

#### 3. Dynamic Exchange System

**Exchange Rate Configuration**
```typescript
const exchangeConfig = {
  coinToStar: 10,           // 10 Coin = 1 Star (default)
  randomRewardRange: {
    minPercentage: 30,      // ได้ขั้นต่ำ 30% ของค่าแลกเปลี่ยน
    maxPercentage: 50       // ได้สูงสุด 50% ของค่าแลกเปลี่ยน
  },
  // สามารถปรับค่าเหล่านี้ได้ตลอดเวลา
};
```

**Real-time Rate Adjustment**
- Admin สามารถปรับอัตราส่วน Coin:Star ได้
- Rate limiting ป้องกันการปรับที่รวดเร็วเกินไป
- Historical tracking ของการเปลี่ยนแปลง

#### 4. Gamification Features

**Variable Reward System**
- Random coin rewards เพื่อสร้างความตื่นเต้น
- "Mystery Bonus" พิเศษในวันหยุดหรือ event พิเศษ
- Lucky draw สำหรับผู้ใช้ที่ active ประจำ

**Progress Tracking**
- Invite progress bar (10/50 to next bonus)
- Achievement badges (First Invite, Social Butterfly)
- Leaderboards (Weekly Top Inviter)

**Time-Limited Events**
- 2x Coin Weekends
- Bonus Coin for first 100 invites per day
- Special events ตามฤดูกาล (Chinese New Year, Valentine)

#### 5. Economic Model Benefits

**Growth Loop Strategy**
```
User has question → Needs Star → Invites friends → Gets Coins →
Exchange for Stars → Ask question → Satisfied → Invites more friends
```

**Revenue Protection**
- Coin rewards ไม่เกิน 50% ของมูลค่า Star
- Maintain perceived value of Stars
- Encourage first purchase (accelerate first reading)

**Viral Coefficient Target**
- Target k-factor ≥ 1.2 (ผู้ใช้ 1 คน invite 1.2 คนโดยเฉลี่ย)
- Natural fit กับ tarot/content about relationships

#### 6. Technical Implementation

**Backend Requirements**
- User wallet system (Star + Coin balance)
- Transaction history tracking
- Invite link generation & tracking
- Anti-fraud detection system
- Real-time exchange rate configuration
- Stripe payment gateway integration

**API Endpoints**
```
POST /api/v1/user/invite
POST /api/v1/user/exchange-coins
GET  /api/v1/user/wallet
POST /api/v1/purchase/stars
GET  /api/v1/admin/exchange-config (admin only)
PUT  /api/v1/admin/exchange-config (admin only)

# Payment Gateway (Stripe Integration)
POST /api/v1/payment/create-intent     # Create Stripe payment intent
POST /api/v1/payment/confirm          # Confirm payment client-side
POST /api/v1/payment/webhook          # Stripe webhook handler
GET  /api/v1/payment/history         # User payment history
GET  /api/v1/payment/packages        # Available packages
```

**Rust Data Models (Serde-based)**
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub star_balance: i64,
    pub coin_balance: i64,
    pub invite_code: String,
    pub invited_by: Option<Uuid>,
    pub total_invites: i32,
    pub tier: UserTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub currency: Currency,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Purchase,
    InviteReward,
    Exchange,
    Reading,
    Refund,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Currency {
    Star,
    Coin,
}

// Axum Request/Response Models
#[derive(Debug, Deserialize)]
pub struct TarotRequest {
    pub question: String,
}

#[derive(Debug, Serialize)]
pub struct TarotResponse {
    pub success: bool,
    pub data: TarotReadingData,
}

#[derive(Debug, Serialize)]
pub struct TarotReadingData {
    pub original_question: String,
    pub question_analysis: QuestionAnalysis,
    pub tarot_reading: TarotReading,
}
```

#### 7. Fraud Prevention

**Invite Validation**
- IP-based rate limiting (max 10 invites/day/IP)
- Device fingerprinting ป้องกัน fake accounts
- Behavioral analysis (active usage requirement)
- Manual review for suspicious patterns

**Economic Safeguards**
- Maximum daily coin earnings cap
- Cooling period ระหว่างการแลกเปลี่ยน
- Transaction monitoring สำหรับ unusual patterns
- Reserve banking สำหรับ coin redemptions

#### 8. Success Metrics

**Engagement Metrics**
- Daily active users (DAU)
- Average session duration
- Invite conversion rate
- Coin-to-Star exchange rate

**Economic Metrics**
- Average revenue per user (ARPU)
- Customer acquisition cost (CAC)
- Lifetime value (LTV)
- Viral coefficient (k-factor)

**Retention Metrics**
- 7-day retention rate
- 30-day retention rate
- Churn rate analysis
- Re-purchase frequency

---

## Payment Gateway Integration (Stripe)

### Payment Strategy Overview

**Phase 1: Debit/Credit Cards (Immediate Implementation)**
- Target: Quick revenue generation (2-3 weeks implementation)
- Coverage: Global customers, Thailand cards supported
- Technology: Stripe Elements + Payment Intents

**Phase 2: PromptPay Integration (Future Expansion)**
- Target: Local market optimization (3-6 months timeline)
- Requirement: Business registration in Thailand
- Prerequisite: 1,000+ active users, >50,000 THB monthly revenue

### Stripe Implementation Architecture

#### 1. Payment Flow System
```
Frontend (Stripe Elements) → Payment Intent Creation →
Backend (Verification) → Stripe Processing →
Webhook Confirmation → Database Update → User Notification
```

#### 2. Star Package Pricing
**Package Design Philosophy:**
- Low barrier entry for first-time users
- Volume discounts for regular users
- Psychological pricing for conversion optimization

**Available Packages (Rust Enums):**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarPackage {
    pub package_type: PackageType,
    pub stars: u32,
    pub price: i64,      // Price in satang (100 satang = 1 THB)
    pub description: String,
    pub bonus_stars: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageType {
    Starter,
    Basic,
    Premium,
}

pub const STAR_PACKAGES: &[StarPackage] = &[
    StarPackage {
        package_type: PackageType::Starter,
        stars: 5,
        price: 5000,     // 50.00 THB in satang
        description: "เริ่มต้น 5 คำถาม ทดลองใช้งาน".to_string(),
        bonus_stars: 0,
    },
    StarPackage {
        package_type: PackageType::Basic,
        stars: 15,
        price: 13000,    // 130.00 THB in satang
        description: "คุ้มค่า 15 คำถาม ประหยัด 20%".to_string(),
        bonus_stars: 2,  // Free bonus stars
    },
    StarPackage {
        package_type: PackageType::Premium,
        stars: 35,
        price: 28000,    // 280.00 THB in satang
        description: "มหาศาล 35 คำถาม ประหยัด 30%".to_string(),
        bonus_stars: 5,  // Free bonus stars
    },
];
```

#### 3. Technical Requirements

**Backend Dependencies (Rust - Cargo.toml):**
```toml
[package]
name = "mimivibe-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web Framework & Async Runtime
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "compression"] }

# Serialization & JSON
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP Client (for Gemini API & Stripe)
reqwest = { version = "0.11", features = ["json"] }

# UUID Generation
uuid = { version = "1.0", features = ["v4", "serde"] }

# Time & Date Handling
chrono = { version = "0.4", features = ["serde"] }

# Environment Variables
dotenv = "0.15"

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# LLM Orchestration (LangChain Rust)
langchain-rust = "0.1.0"

# Prompt Security & Encryption
base64 = "0.21"
aes-gcm = "0.10"  # For AES encryption (optional for higher security)

# Payment Gateway (Stripe)
stripe = "0.23.0"

# Async Utilities (for retry logic)
tokio-retry = "0.3"

# Configuration
config = "0.13"

# Tracing & Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Rate Limiting
governor = "0.5"

# Hash Maps (for rate limiting & caching)
dashmap = "5.4"

# JWT Authentication (Multi-Provider Support)
jsonwebtoken = "9.2"
reqwest = { version = "0.11", features = ["json"] } # Already listed above

# Database (SQLx)
sqlx = { version = "0.7", features = [
    "postgres",
    "runtime-tokio-rustls",
    "uuid",
    "chrono",
    "json",
    "migrate"
] }
```

**Supabase Database Schema (SQLx-ready):**
```sql
-- Users table (Enhanced for Multi-Provider Auth)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255) NOT NULL,      -- External auth provider ID
    external_provider VARCHAR(50) NOT NULL, -- "clerk", "auth0", "firebase", "custom"
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    picture_url TEXT,

    -- MimiVibe specific fields
    star_balance BIGINT DEFAULT 0,
    coin_balance BIGINT DEFAULT 0,
    invite_code VARCHAR(10) UNIQUE NOT NULL,
    invited_by UUID REFERENCES users(id),
    total_invites INTEGER DEFAULT 0,
    tier VARCHAR(20) DEFAULT 'bronze' CHECK (tier IN ('bronze', 'silver', 'gold')),

    -- Authentication & activity tracking
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Constraints
    UNIQUE(external_id, external_provider) -- One external ID per provider
);

-- Payment transactions
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_intent_id VARCHAR(255) UNIQUE,
    stripe_charge_id VARCHAR(255),
    amount BIGINT NOT NULL,              -- in satang (smallest unit)
    currency VARCHAR(3) DEFAULT 'THB',
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'succeeded', 'failed', 'refunded')),
    package_type VARCHAR(20) NOT NULL CHECK (package_type IN ('starter', 'basic', 'premium')),
    stars_purchased INTEGER NOT NULL,
    stars_bonus INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User payment methods (for saved cards)
CREATE TABLE user_payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_method_id VARCHAR(255) UNIQUE,
    type VARCHAR(20) NOT NULL CHECK (type IN ('card')),
    brand VARCHAR(20),                   -- visa, mastercard, etc.
    last4 VARCHAR(4),                    -- Last 4 digits
    exp_month INTEGER,
    exp_year INTEGER,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Star and Coin transactions (for wallet history)
CREATE TABLE wallet_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    transaction_type VARCHAR(20) NOT NULL CHECK (transaction_type IN ('purchase', 'invite_reward', 'exchange', 'reading', 'refund')),
    amount INTEGER NOT NULL,
    currency VARCHAR(10) NOT NULL CHECK (currency IN ('STAR', 'COIN')),
    balance_after BIGINT NOT NULL,       -- Wallet balance after this transaction
    metadata JSONB,                      -- Additional transaction data
    related_payment_id UUID REFERENCES payments(id), -- Link to payment if applicable
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Tarot readings history
CREATE TABLE tarot_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    question_length INTEGER NOT NULL,
    card_count INTEGER NOT NULL CHECK (card_count IN (3, 5)),
    cards JSONB NOT NULL,                -- Array of selected cards with positions
    reading JSONB NOT NULL,              -- Complete AI-generated reading
    question_analysis JSONB,             -- Agent 2 analysis results
    processing_time_ms INTEGER,          -- How long the reading took
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User invitation tracking
CREATE TABLE user_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inviter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invitee_id UUID REFERENCES users(id) ON DELETE SET NULL,
    invite_code VARCHAR(10) NOT NULL REFERENCES users(invite_code),
    coin_reward INTEGER,                 -- Coins awarded to inviter
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'accepted', 'expired')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    accepted_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT (NOW() + INTERVAL '30 days')
);

-- Indexes for performance optimization
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_invite_code ON users(invite_code);
CREATE INDEX idx_users_tier ON users(tier);
CREATE INDEX idx_users_external_id_provider ON users(external_id, external_provider);
CREATE INDEX idx_users_last_login ON users(last_login_at);
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_created_at ON payments(created_at);
CREATE INDEX idx_wallet_transactions_user_id ON wallet_transactions(user_id);
CREATE INDEX idx_wallet_transactions_created_at ON wallet_transactions(created_at);
CREATE INDEX idx_tarot_readings_user_id ON tarot_readings(user_id);
CREATE INDEX idx_tarot_readings_created_at ON tarot_readings(created_at);
CREATE INDEX idx_user_invites_inviter_id ON user_invites(inviter_id);
CREATE INDEX idx_user_invites_invite_code ON user_invites(invite_code);

-- Row Level Security (RLS) for multi-tenant security
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;
ALTER TABLE wallet_transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE tarot_readings ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_payment_methods ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_invites ENABLE ROW LEVEL SECURITY;
```

#### 4. Payment Processing Flow

**Step 1: Create Payment Intent (Axum Handler)**
```rust
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use stripe::{Client, PaymentIntent};

#[derive(Debug, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub package: PackageType,
    pub payment_method_id: Option<String>, // For saved cards
}

#[derive(Debug, Serialize)]
pub struct PaymentIntentResponse {
    pub client_secret: String,
    pub payment_intent_id: String,
    pub amount: i64,    // in satang
    pub currency: String,
}

pub async fn create_payment_intent(
    State(stripe_client): State<Client>,
    Json(request): Json<CreatePaymentIntentRequest>,
) -> Result<Json<PaymentIntentResponse>, AppError> {
    // Find package configuration
    let package = STAR_PACKAGES
        .iter()
        .find(|p| p.package_type == request.package)
        .ok_or(AppError::InvalidPackage)?;

    // Create Stripe PaymentIntent
    let mut payment_intent_params = PaymentIntent::create_params();
    payment_intent_params.amount = Some(package.price);
    payment_intent_params.currency = Some(stripe::Currency::THB);
    payment_intent_params.payment_method = request.payment_method_id
        .map(|id| id.parse().unwrap())
        .map(Some);
    payment_intent_params.confirm = Some(false);
    payment_intent_params.setup_future_usage = Some(stripe::SetupFutureUsage::OnSession);

    let payment_intent = PaymentIntent::create(&stripe_client, payment_intent_params)
        .await
        .map_err(|e| AppError::StripeError(e.to_string()))?;

    Ok(Json(PaymentIntentResponse {
        client_secret: payment_intent.client_secret.unwrap_or_default(),
        payment_intent_id: payment_intent.id.to_string(),
        amount: package.price,
        currency: "thb".to_string(),
    }))
}
```

**Step 2: Webhook Processing (Axum Handler)**
```rust
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use stripe::{Webhook, Event};

pub async fn stripe_webhook(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
) -> Result<(), AppError> {
    // Get webhook signature from headers
    let stripe_signature = headers
        .get("stripe-signature")
        .ok_or(AppError::MissingStripeSignature)?
        .to_str()
        .map_err(|_| AppError::InvalidStripeSignature)?;

    // Read request body
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|_| AppError::InvalidRequestBody)?;

    // Verify webhook signature
    let webhook_secret = &app_state.stripe_webhook_secret;
    let event = Webhook::construct_event(&body_bytes, stripe_signature, webhook_secret)
        .map_err(|_| AppError::InvalidStripeSignature)?;

    // Process the event
    match event.type_ {
        stripe::EventType::PaymentIntentSucceeded => {
            handle_payment_success(&app_state, event).await?;
        }
        stripe::EventType::PaymentIntentPaymentFailed => {
            handle_payment_failure(&app_state, event).await?;
        }
        // Handle other event types as needed
        _ => {}
    }

    Ok(())
}

async fn handle_payment_success(
    app_state: &AppState,
    event: Event,
) -> Result<(), AppError> {
    let payment_intent = event.data.object.as_payment_intent()
        .ok_or(AppError::InvalidEventData)?;

    let user_id = extract_user_id_from_metadata(&payment_intent.metadata)?;
    let package_type = extract_package_type_from_metadata(&payment_intent.metadata)?;

    // Find package configuration
    let package = STAR_PACKAGES
        .iter()
        .find(|p| p.package_type == package_type)
        .ok_or(AppError::InvalidPackage)?;

    // Update payment record in database
    update_payment_status(&app_state.db, &payment_intent.id.to_string(), "succeeded").await?;

    // Add stars to user wallet (stars + bonus)
    let total_stars = package.stars + package.bonus_stars;
    add_stars_to_user_wallet(&app_state.db, user_id, total_stars as i64).await?;

    // Log transaction for analytics
    log_revenue_event(&app_state.db, user_id, package.price, total_stars).await?;

    tracing::info!(
        "Payment succeeded: user_id={}, payment_intent_id={}, stars_added={}",
        user_id,
        payment_intent.id,
        total_stars
    );

    Ok(())
}
```

#### 5. Security & Compliance

**PCI DSS Compliance:**
- Never store credit card details on our servers
- Use Stripe Elements for card entry (tokenization)
- All sensitive data handled by Stripe infrastructure
- HTTPS enforcement for all payment endpoints

**Webhook Security:**
```rust
// Secure webhook signature verification in production
pub fn verify_webhook_signature(
    payload: &[u8],
    signature: &str,
    webhook_secret: &str,
) -> Result<Event, stripe::WebhookError> {
    Webhook::construct_event(payload, signature, webhook_secret)
}

// Axum middleware for webhook security
pub async fn webhook_auth_middleware(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    request: Request,
) -> Result<Request, AppError> {
    // Verify Stripe signature before processing
    let signature = headers
        .get("stripe-signature")
        .ok_or(AppError::MissingStripeSignature)?
        .to_str()
        .map_err(|_| AppError::InvalidStripeSignature)?;

    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|_| AppError::InvalidRequestBody)?;

    verify_webhook_signature(&body_bytes, signature, &app_state.stripe_webhook_secret)
        .map_err(|_| AppError::InvalidStripeSignature)?;

    // Reconstruct request with body for next handler
    let request = Request::builder()
        .method("POST")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body_bytes))
        .map_err(|_| AppError::InvalidRequestBody)?;

    Ok(request)
}
```

#### 6. Multi-Provider Authentication System

**Two-Layer Authentication Architecture:**

**Layer 1: External Auth Verification (Provider Agnostic)**
```rust
use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::{Deserialize, Serialize};
use axum::{extract::Request, middleware::Next, response::Response};

// Generic JWT claims structure
#[derive(Debug, Deserialize, Serialize)]
pub struct UserClaims {
    pub sub: String,        // User ID from auth provider
    pub email: String,      // User email
    pub name: Option<String>, // User name
    pub picture: Option<String>, // Profile picture
    pub iss: String,        // Issuer (clerk.dev, auth0.com, etc.)
    pub aud: String,        // Audience (your app)
    pub exp: usize,         // Expiration
    pub iat: usize,         // Issued at
}

// Multi-Provider Configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub clerk: ClerkConfig,
    pub auth0: Auth0Config,
    pub firebase: FirebaseConfig,
    pub custom: CustomAuthConfig,
}

#[derive(Debug, Clone)]
pub struct ClerkConfig {
    pub jwks_url: String,    // Clerk JWKS endpoint
    pub issuer: String,      // https://clerk.your-domain.com
    pub audience: String,    // Your app's audience
}

// Application state with auth configuration
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub stripe_client: stripe::Client,
    pub auth_config: AuthConfig,
    pub jwt_public_keys: JwtPublicKeys,
}

// Universal JWT middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .ok_or(AppError::MissingAuthToken)?
        .to_str()
        .map_err(|_| AppError::InvalidAuthToken)?;

    // Extract JWT token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::InvalidAuthToken)?;

    // Decode JWT (supports multiple providers)
    let claims = decode_jwt(token, &state.jwt_public_keys, &state.auth_config)?;

    // Add user context to request
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// Support multiple JWT issuers
fn decode_jwt(
    token: &str,
    public_keys: &JwtPublicKeys,
    auth_config: &AuthConfig,
) -> Result<UserClaims, AppError> {
    // Decode without signature verification first to get issuer
    let header = jsonwebtoken::decode_header(token)
        .map_err(|_| AppError::InvalidAuthToken)?;

    let token_data = decode::<UserClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_keys.get_key_for_token(token)?),
        &Validation::new(jsonwebtoken::Algorithm::RS256),
    )
    .map_err(|_| AppError::InvalidAuthToken)?;

    // Validate issuer against our supported providers
    validate_issuer(&token_data.claims.iss, auth_config)?;

    Ok(token_data.claims)
}

// Helper to detect provider from JWT issuer
fn detect_provider(issuer: &str) -> Result<String, AppError> {
    match issuer {
        iss if iss.contains("clerk.") => Ok("clerk".to_string()),
        iss if iss.contains("auth0.com") => Ok("auth0".to_string()),
        iss if iss.contains("firebase") => Ok("firebase".to_string()),
        iss if iss.contains("your-domain.com") => Ok("custom".to_string()),
        _ => Err(AppError::UnsupportedAuthProvider),
    }
}
```

**Layer 2: Internal User Management**
```rust
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,                    // Our internal UUID
    pub external_id: String,         // External auth provider user ID
    pub external_provider: String,   // "clerk", "auth0", "firebase", "custom"
    pub email: String,
    pub name: Option<String>,
    pub picture_url: Option<String>,

    // MimiVibe specific data
    pub star_balance: i64,
    pub coin_balance: i64,
    pub invite_code: String,
    pub tier: UserTier,
    pub total_invites: i32,
    pub created_at: DateTime<Utc>,
    pub last_login_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User synchronization logic
pub async fn get_or_create_user(
    State(state): State<AppState>,
    claims: UserClaims,
) -> Result<User, AppError> {
    let provider = detect_provider(&claims.iss)?;
    let external_id = claims.sub.clone();

    // Try to find existing user
    if let Some(user) = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
        WHERE external_id = $1 AND external_provider = $2
        "#,
        external_id,
        provider
    )
    .fetch_optional(&state.db_pool)
    .await?
    {
        // Update last login and return user
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                last_login_at = NOW(),
                email = COALESCE($3, email),
                name = COALESCE($4, name),
                picture_url = COALESCE($5, picture_url),
                updated_at = NOW()
            WHERE id = $6
            RETURNING *
            "#,
            claims.email,
            claims.name,
            claims.picture,
            user.id
        )
        .fetch_one(&state.db_pool)
        .await?;

        Ok(updated_user)
    } else {
        // Create new user
        let new_user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                id, external_id, external_provider, email, name, picture_url,
                star_balance, coin_balance, invite_code, tier,
                total_invites, created_at, last_login_at, updated_at
            ) VALUES (
                gen_random_uuid(), $1, $2, $3, $4, $5,
                0, 0, $6, 'bronze', 0, NOW(), NOW(), NOW()
            )
            RETURNING *
            "#,
            external_id,
            provider,
            claims.email,
            claims.name,
            claims.picture,
            generate_invite_code()
        )
        .fetch_one(&state.db_pool)
        .await?;

        // Send welcome bonus (e.g., 5 free stars)
        add_welcome_bonus(&state.db_pool, new_user.id).await?;

        Ok(new_user)
    }
}

// Protected route handler example
pub async fn protected_tarot_reading(
    State(state): State<AppState>,
    claims: UserClaims, // From auth middleware
    Json(request): Json<TarotRequest>,
) -> Result<Json<TarotResponse>, AppError> {
    // Get or create internal user
    let user = get_or_create_user(State(state.clone()), claims).await?;

    // Check user has enough stars for reading
    if user.star_balance < 1 {
        return Err(AppError::InsufficientStars);
    }

    // Process tarot reading
    let reading = process_tarot_reading(State(state), user.id, request).await?;

    // Deduct star and create transaction
    deduct_star_for_reading(State(state), user.id).await?;

    Ok(Json(reading))
}
```

**Frontend Integration Examples:**

**Clerk Integration:**
```typescript
import { useAuth } from '@clerk/nextjs';

export default function TarotReading() {
  const { getToken } = useAuth();

  const handleTarotReading = async (question: string) => {
    const token = await getToken(); // Get Clerk JWT

    const response = await fetch('/api/v1/tarot/read', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ question }),
    });

    const result = await response.json();
    return result;
  };
}
```

**Auth0 Integration:**
```typescript
import { useAuth0 } from '@auth0/auth0-react';

export default function TarotReading() {
  const { getAccessTokenSilently } = useAuth0();

  const handleTarotReading = async (question: string) => {
    const token = await getAccessTokenSilently(); // Get Auth0 JWT

    const response = await fetch('/api/v1/tarot/read', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ question }),
    });

    return await response.json();
  };
}
```

**Environment Configuration (.env):**
```bash
# Clerk Authentication
CLERK_ISSUER=https://your-domain.clerk.accounts.dev
CLERK_AUDIENCE=your-app-id
CLERK_JWKS_URL=https://your-domain.clerk.accounts.dev/v1/jwks

# Auth0 Authentication
AUTH0_ISSUER=https://your-domain.auth0.com
AUTH0_AUDIENCE=your-api-identifier
AUTH0_JWKS_URL=https://your-domain.auth0.com/.well-known/jwks.json

# Firebase Authentication
FIREBASE_PROJECT_ID=your-project-id

# Database Configuration
DATABASE_URL=postgresql://postgres:[password]@db.[project-ref].supabase.co:5432/postgres
```

#### 7. SQLx Database Integration

**Database Connection Setup:**
```rust
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)           // Max concurrent connections
        .min_connections(5)            // Min connections to keep alive
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(database_url)
        .await
}

// Application state with database pool
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub stripe_client: stripe::Client,
}

// Initialize in main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let db_pool = create_db_pool(&database_url).await?;

    // Run SQLx migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    let app_state = AppState {
        db_pool,
        stripe_client: create_stripe_client()?,
    };

    let app = Router::new()
        .route("/api/v1/tarot/read", post(read_tarot))
        .route("/api/v1/user/wallet/:user_id", get(get_user_wallet))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // ... rest of Axum setup
}
```

**SQLx Data Models:**
```rust
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub star_balance: i64,
    pub coin_balance: i64,
    pub invite_code: String,
    pub invited_by: Option<Uuid>,
    pub total_invites: i32,
    pub tier: UserTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_payment_intent_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub package_type: String,
    pub stars_purchased: i32,
    pub stars_bonus: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: i32,
    pub currency: Currency,
    pub balance_after: i64,
    pub metadata: serde_json::Value,
    pub related_payment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct TarotReading {
    pub id: Uuid,
    pub user_id: Uuid,
    pub question: String,
    pub question_length: i32,
    pub card_count: i32,
    pub cards: serde_json::Value,
    pub reading: serde_json::Value,
    pub question_analysis: Option<serde_json::Value>,
    pub processing_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}
```

**SQLx Query Examples:**
```rust
// Get user wallet with type safety
pub async fn get_user_wallet(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserWallet>, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id, email, star_balance, coin_balance,
            invite_code, invited_by, total_invites, tier,
            created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::UserNotFound)?;

    Ok(Json(UserWallet {
        user_id: user.id,
        star_balance: user.star_balance,
        coin_balance: user.coin_balance,
        tier: user.tier,
    }))
}

// Add stars to user wallet with transaction record
pub async fn add_stars_to_wallet(
    State(state): State<AppState>,
    user_id: Uuid,
    stars_to_add: i64,
    transaction_type: TransactionType,
    related_payment_id: Option<Uuid>,
) -> Result<(), AppError> {
    // Use SQL transaction for atomicity
    let mut tx = state.db_pool.begin().await?;

    // Update user balance
    let update_result = sqlx::query!(
        r#"
        UPDATE users
        SET
            star_balance = star_balance + $1,
            updated_at = NOW()
        WHERE id = $2
        RETURNING star_balance
        "#,
        stars_to_add,
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // Create transaction record
    sqlx::query_as!(
        WalletTransaction,
        r#"
        INSERT INTO wallet_transactions (
            id, user_id, transaction_type, amount,
            currency, balance_after, metadata, related_payment_id
        ) VALUES (
            gen_random_uuid(), $1, $2, $3, 'STAR', $4, NULL, $5
        )
        RETURNING *
        "#,
        user_id,
        transaction_type as TransactionType,
        stars_to_add as i32,
        update_result.star_balance,
        related_payment_id
    )
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok(())
}

// Get user transaction history with pagination
pub async fn get_user_transactions(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<TransactionQueryParams>,
) -> Result<Json<Vec<WalletTransaction>>, AppError> {
    let transactions = sqlx::query_as!(
        WalletTransaction,
        r#"
        SELECT
            id, user_id, transaction_type, amount,
            currency, balance_after, metadata, related_payment_id,
            created_at
        FROM wallet_transactions
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        params.limit as i64,
        params.offset as i64
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(Json(transactions))
}

// Create tarot reading record
pub async fn save_tarot_reading(
    State(state): State<AppState>,
    user_id: Uuid,
    question: String,
    cards: serde_json::Value,
    reading: serde_json::Value,
    processing_time_ms: i32,
) -> Result<TarotReading, AppError> {
    let saved_reading = sqlx::query_as!(
        TarotReading,
        r#"
        INSERT INTO tarot_readings (
            user_id, question, question_length, card_count,
            cards, reading, processing_time_ms
        ) VALUES (
            $1, $2, $3, 4, $4, $5, $6
        )
        RETURNING *
        "#,
        user_id,
        question,
        question.len() as i32,
        (json!(cards).as_array().unwrap_or(&vec![]).len() as i32),
        cards,
        reading,
        processing_time_ms
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(saved_reading)
}
```

**SQLx Migration Example:**
```sql
-- migrations/001_create_users.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    star_balance BIGINT DEFAULT 0,
    coin_balance BIGINT DEFAULT 0,
    invite_code VARCHAR(10) UNIQUE NOT NULL,
    invited_by UUID REFERENCES users(id),
    total_invites INTEGER DEFAULT 0,
    tier VARCHAR(20) DEFAULT 'bronze' CHECK (tier IN ('bronze', 'silver', 'gold')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_invite_code ON users(invite_code);
```

**Environment Configuration (.env):**
```bash
# Database Configuration
DATABASE_URL=postgresql://postgres:[password]@db.[project-ref].supabase.co:5432/postgres

# SQLx Configuration (offline mode for compile-time checking)
SQLX_OFFLINE=true

# Supabase Configuration
SUPABASE_URL=https://[project-ref].supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
```

#### 7. Frontend Integration

**Stripe Elements Setup:**
```typescript
import { loadStripe } from '@stripe/stripe-js';
import { CardElement, useStripe, useElements } from '@stripe/react-stripe-js';

const PaymentForm = ({ selectedPackage }) => {
  const stripe = useStripe();
  const elements = useElements();

  const handleSubmit = async (event) => {
    event.preventDefault();

    const { error, paymentMethod } = await stripe.createPaymentMethod({
      type: 'card',
      card: elements.getElement(CardElement),
      billing_details: {
        name: user.name,
        email: user.email,
      },
    });

    if (error) {
      setError(error.message);
      return;
    }

    // Confirm payment with backend
    const result = await confirmPayment(paymentMethod.id, selectedPackage);

    if (result.success) {
      // Show success animation and update UI
      showSuccessScreen();
      updateUserBalance();
    }
  };
};
```

#### 7. Error Handling & User Experience

**Common Payment Errors:**
- **Card Declined**: Clear message + retry options
- **Insufficient Funds**: Suggest different package
- **Processing Error**: Auto-retry + manual option
- **Network Issues**: Save progress + retry later

**User Experience Flow:**
1. **Package Selection** → Visual package comparison
2. **Payment Entry** → Stripe Elements (mobile optimized)
3. **Processing** → Loading animation with progress
4. **Success** → Confetti + stars added animation
5. **Confirmation** → Email + in-app notification

#### 8. Testing & Deployment Strategy

**Testing Environment:**
- Stripe Test Mode (test keys)
- Test card numbers for different scenarios
- Webhook testing with Stripe CLI
- End-to-end integration testing

**Production Deployment:**
- Environment-specific configuration
- Gradual rollout (10% → 50% → 100%)
- Real-time monitoring & alerting
- Automatic rollback on high error rates

#### 9. Revenue & Analytics

**Key Performance Metrics:**
```rust
struct PaymentAnalytics {
    total_revenue: f64,                    // Daily/weekly revenue
    conversion_rate: f64,                  // Payment success rate
    average_transaction_value: f64,        // ARPPU (Average Revenue Per Paying User)
    package_popularity: HashMap<String, i32>, // Which packages sell best
    payment_method_distribution: HashMap<String, f64>,
    chargeback_rate: f64,                  // Dispute rate
    refund_rate: f64,                      // Refund percentage
}
```

**Revenue Projections (Phase 1):**
- **Target ARPU**: 150-300 THB/month
- **Conversion Rate**: 15-25% from free to paid
- **Customer LTV**: 500-1,000 THB over 6 months
- **Break-even Point**: 200 paying users

#### 10. Implementation Timeline

**Week 1-2: Foundation**
- Stripe account setup & API keys
- Database schema implementation
- Basic payment intent creation
- Test environment configuration

**Week 3-4: Core Integration**
- Payment processing pipeline
- Webhook event handling
- Star crediting system
- Error handling & validation

**Week 5-6: Frontend & UX**
- Stripe Elements integration
- Payment UI/UX optimization
- Mobile responsive design
- Success/error states

**Week 7-8: Testing & Launch**
- End-to-end testing suite
- Security audit & penetration testing
- Production deployment
- Monitoring & alerting setup

---

## Success Metrics

### Technical Metrics
- API response time < 5 seconds
- 99.9% system uptime
- Zero security breaches
- 99% request success rate

### Business Metrics
- User adoption rate
- Reading accuracy satisfaction
- Daily active users
- API usage growth

---

## Dependencies & References

### Prompt Templates
- `/prompts/QUESTION-FILTER.md` - Agent 1 system prompt
- `/prompts/QUESTION-ANALYSIS.md` - Agent 2 system prompt
- `/prompts/READING-AGENT.md` - Agent 3 system prompt

### External APIs
- Google Gemini API - LLM processing
- Authentication provider (TBD)

---

**Version**: 1.0
**Last Updated**: 2025-11-19
**Author**: Team Discussion