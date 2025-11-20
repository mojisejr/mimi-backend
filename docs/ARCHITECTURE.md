# Architecture Documentation
## MimiVibe Backend - Tarot Reading System

### Table of Contents
1. [System Overview](#system-overview)
2. [High-Level Architecture](#high-level-architecture)
3. [Technology Stack](#technology-stack)
4. [Component Architecture](#component-architecture)
5. [Monorepo Structure](#monorepo-structure)
6. [Data Flow Architecture](#data-flow-architecture)
7. [Queue & Worker Architecture](#queue--worker-architecture)
8. [Security Architecture](#security-architecture)
9. [Database Architecture](#database-architecture)
10. [Deployment Architecture](#deployment-architecture)
11. [Scalability Design](#scalability-design)
12. [Integration Points](#integration-points)

---

## System Overview

### Mission
Build a scalable, AI-powered tarot reading backend system using Google Gemini LLM with a sophisticated agent pipeline orchestrated in Rust, supporting multiple authentication providers and modern payment integration.

### Core Characteristics
- **Asynchronous Processing**: Heavy LLM operations offloaded to background workers
- **Multi-Tenant Ready**: Support multiple authentication providers (Clerk, Auth0, Firebase)
- **Gamified Economy**: Star (hard currency) and Coin (soft currency) system
- **Payment Processing**: Stripe integration for global card processing, PromptPay phase 2
- **Agent-Based**: LangChain Rust for sophisticated AI pipeline orchestration
- **Type-Safe**: Rust's type system ensures reliability and performance

### Key Design Principles
1. **Separation of Concerns**: API gateway and background workers in single monorepo
2. **Asynchronous Excellence**: Queue-based job processing for LLM operations
3. **Type Safety**: Compile-time guarantees via Rust
4. **Observability First**: Structured logging and Sentry integration
5. **Security by Default**: Multi-provider auth, encrypted prompts, secure payment flow

---

## High-Level Architecture

### System Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Client Layer                                   │
│                      (Frontend - Web/Mobile)                             │
└──────────────────────────────┬──────────────────────────────────────────┘
                               │
                    (HTTPS / JWT Token)
                               │
        ┌──────────────────────┴──────────────────────┐
        │                                             │
┌───────▼─────────────────────────────────┐   ┌──────▼──────────────────┐
│          API Gateway (Axum)              │   │   Webhook Handler       │
│  ┌─────────────────────────────────────┐ │   │ ┌────────────────────┐  │
│  │ REST Endpoints                      │ │   │ │ Stripe Webhooks    │  │
│  │ • /api/v1/tarot/read               │ │   │ │ • Payment success  │  │
│  │ • /api/v1/user/wallet              │ │   │ │ • Payment failed   │  │
│  │ • /api/v1/purchase/stars           │ │   │ └────────────────────┘  │
│  │ • /api/v1/user/invite              │ │   │                         │
│  │ • /api/v1/payment/*                │ │   └─────────────────────────┘
│  │                                     │ │
│  ├─────────────────────────────────────┤ │
│  │ Authentication Layer (Multi-provider)
│  │ • Clerk / Auth0 / Firebase JWT      │ │
│  │ • API Key validation (Phase 1)      │ │
│  │ • Rate limiting middleware          │ │
│  ├─────────────────────────────────────┤ │
│  │ Middleware Stack (Tower)            │ │
│  │ • CORS (Frontend domain)            │ │
│  │ • Compression & Tracing             │ │
│  │ • Request validation                │ │
│  └─────────────────────────────────────┘ │
└──────────────────────┬────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
┌───────▼────────────────┐   ┌────────▼──────────────┐
│ PostgreSQL (Supabase)  │   │  Upstash Redis Queue  │
│                        │   │ (Redis Streams)       │
│ ┌────────────────────┐ │   │ ┌──────────────────┐  │
│ │ Users              │ │   │ │ Stream: jobs     │  │
│ │ Payments           │ │   │ │ Consumer groups  │  │
│ │ Wallet Transactions│ │   │ │ Delayed retries  │  │
│ │ Tarot Readings     │ │   │ │ DLQ              │  │
│ │ User Invites       │ │   │ └──────────────────┘  │
│ └────────────────────┘ │   │                       │
└────────────────────────┘   └───────┬────────────────┘
                                     │
                    ┌────────────────┴────────────────┐
                    │                                 │
                 ┌──▼──────────────────────┐   ┌──────▼─────────────┐
                 │ Background Worker        │   │  Scheduler         │
                 │ (Rust Binary)            │   │  (Delayed retries) │
                 │ ┌──────────────────────┐ │   │ ┌─────────────────┐│
                 │ │ XREADGROUP from queue│ │   │ │ ZSET: jobs:delay││
                 │ │ Process tarot reading│ │   │ │ Re-enqueue ready││
                 │ │ Call Gemini LLM      │ │   │ │ jobs             ││
                 │ │ Save results to DB   │ │   │ │ Exponential      ││
                 │ │ XACK on completion   │ │   │ │ backoff logic    ││
                 │ │ Handle DLQ for fails │ │   │ └─────────────────┘│
                 │ └──────────────────────┘ │   │                     │
                 │                          │   │                     │
                 │ Concurrency: 5-10 workers│   │                     │
                 │ Timeout: 30s per reading │   │                     │
                 └──────────────────────────┘   └─────────────────────┘
                            │
        ┌───────────────────┴───────────────────┐
        │                                       │
    ┌───▼────────────────┐         ┌────────────▼────┐
    │ Google Gemini API  │         │ Stripe API      │
    │ ┌───────────────┐  │         │ ┌────────────┐  │
    │ │ Agent 1       │  │         │ │ Payment    │  │
    │ │ Agent 2       │  │         │ │ Processing │  │
    │ │ Agent 3       │  │         │ └────────────┘  │
    │ │ Card selector │  │         │                 │
    │ └───────────────┘  │         │                 │
    └────────────────────┘         └─────────────────┘

Observable via:
    - Sentry (error tracking, performance)
    - Structured logs (stdout/stderr)
    - /health endpoint (readiness/liveness)
```

---

## Technology Stack

### Backend Core
```
Language:           Rust (Edition 2021)
Async Runtime:      Tokio
Web Framework:      Axum (with Tower middleware)
Agent Orchestration: LangChain Rust
HTTP Client:        Reqwest
Serialization:      Serde + serde_json
Configuration:      dotenv
```

### External Services
```
Database:           PostgreSQL 15+ (Supabase)
ORM/Query Builder:  SQLx (type-safe)
Queue System:       Upstash Redis Streams
Payment Gateway:    Stripe API
LLM Provider:       Google Gemini API
Authentication:     Multi-provider (Clerk, Auth0, Firebase)
Error Tracking:     Sentry
```

### Infrastructure
```
Containerization:   Docker / Dockerfile
Orchestration:      Render (Web Service + Background Service)
Storage:            Supabase Storage (future)
CDN:                Cloudflare (future)
Secrets Management: Render environment variables / Vault
```

### Development Tools
```
Package Manager:    Cargo
Testing:            cargo test (built-in)
Linting:            cargo clippy
Code Formatting:    cargo fmt
Migration Tool:     SQLx migrate
Git:                GitHub (monorepo)
CI/CD:              GitHub Actions
```

---

## Component Architecture

### 1. API Gateway (Axum-based HTTP Server)

**Responsibilities:**
- Accept HTTP requests from frontend
- Validate authentication via multiple providers
- Route requests to appropriate handlers
- Rate limiting and request validation
- Return structured JSON responses

**Key Components:**

#### Router Definition
```rust
// Main router setup
pub fn create_app(state: AppState) -> Router {
    Router::new()
        // Public endpoints
        .route("/health", get(health_check))
        
        // Protected endpoints (require auth)
        .route("/api/v1/tarot/read", post(read_tarot))
        .route("/api/v1/user/wallet/:user_id", get(get_user_wallet))
        .route("/api/v1/user/invite", post(create_invite))
        .route("/api/v1/exchange-coins", post(exchange_coins_for_stars))
        
        // Payment endpoints
        .route("/api/v1/payment/create-intent", post(create_payment_intent))
        .route("/api/v1/payment/confirm", post(confirm_payment))
        .route("/api/v1/payment/history", get(get_payment_history))
        .route("/api/v1/payment/packages", get(list_payment_packages))
        
        // Webhook endpoints (no auth required)
        .route("/webhooks/stripe", post(stripe_webhook))
        
        // Middleware
        .layer(
            CorsLayer::permissive()
                .allow_origin("https://mimivibe.com".parse()?)
        )
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        
        .with_state(state)
}
```

#### Authentication Middleware
```rust
// Multi-provider JWT validation
pub async fn auth_middleware(
    State(config): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract JWT from Authorization header
    let token = extract_token(&req)?;
    
    // Decode JWT (supports Clerk, Auth0, Firebase)
    let claims = validate_jwt(token, &config)?;
    
    // Add claims to request extensions
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}
```

#### Rate Limiting Middleware
```rust
// Per-user rate limiting (100 requests/minute)
pub async fn rate_limit_middleware(
    claims: UserClaims,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user_id = claims.sub.clone();
    
    // Check against rate limit quota
    if !rate_limiter.allow_request(&user_id) {
        return Err(AppError::RateLimitExceeded);
    }
    
    Ok(next.run(req).await)
}
```

---

### 2. Agent Pipeline Engine (LangChain Rust)

**Responsibilities:**
- Orchestrate AI agents in sequence
- Manage state between agents
- Call Google Gemini LLM
- Generate random card selection
- Format final response

**Architecture:**

#### Sequential Chain: Agent Pipeline
```
Input Question
    ↓
Agent 1: Question Filter (LLMChain)
    └─→ Validate appropriateness (Boolean)
    ↓
Agent 2: Question Analyzer (LLMChain)
    └─→ Extract category, emotion, timeframe, themes
    ↓
Random Card Selection (Tool)
    └─→ Choose 3 or 5 cards (50/50 random)
    ↓
Agent 3: Reading Agent (LLMChain)
    └─→ Generate detailed tarot reading
    ↓
Response Formatter
    └─→ Structure JSON response
```

**Implementation Pattern:**

```rust
// Agent pipeline structure
pub struct TarotPipeline {
    filter_chain: LLMChain<GeminiLLM>,
    analyzer_chain: LLMChain<GeminiLLM>,
    reader_chain: LLMChain<GeminiLLM>,
    card_selector: CardSelectionTool,
}

impl TarotPipeline {
    pub async fn process_reading(
        &self,
        question: &str,
    ) -> Result<TarotResponse, PipelineError> {
        // Step 1: Filter question appropriateness
        let filter_result = self.filter_chain.invoke(question).await?;
        
        if !filter_result.is_allowed {
            return Err(PipelineError::QuestionNotAppropriate(
                filter_result.reason
            ));
        }
        
        // Step 2: Analyze question context
        let analysis = self.analyzer_chain
            .invoke_with_context(&question)
            .await?;
        
        // Step 3: Select random card count (3 or 5)
        let card_count = self.card_selector.random_count();
        
        // Step 4: Generate reading with cards
        let reading = self.reader_chain
            .invoke_with_cards(&analysis, card_count)
            .await?;
        
        // Step 5: Format response
        Ok(format_response(question, &analysis, reading))
    }
}
```

---

### 3. Job Queue System (Upstash Redis Streams)

**Responsibilities:**
- Queue tarot reading jobs asynchronously
- Manage job lifecycle (pending → processing → completed/failed)
- Handle retries with exponential backoff
- Track dead-letter queue (DLQ) for failed jobs
- Ensure visibility and monitoring

**Job Flow:**

```
1. API receives request
   ├─ Validate input & auth
   ├─ Create dedupe_key for idempotency
   ├─ Check for existing job in DB
   ├─ Create job record in DB (status='queued')
   └─ XADD job to Redis Streams (jobs:stream)
   
2. Worker processes job
   ├─ XREADGROUP from jobs:stream
   ├─ Update job status='processing'
   ├─ Execute agent pipeline
   ├─ Update job status='succeeded'
   ├─ Save results to tarot_readings table
   ├─ XACK the stream entry
   └─ Notify user (push notification/email)
   
3. On failure
   ├─ Log error details
   ├─ Update job_attempts table
   ├─ If attempts < max_attempts:
   │  └─ ZADD to jobs:delayed (with backoff delay)
   └─ Else (max retries exceeded):
      ├─ Update job status='dlq'
      ├─ XADD to jobs:dlq stream
      └─ Trigger alert (Sentry)
      
4. Scheduler processes delayed retries
   ├─ ZRANGEBYSCORE jobs:delayed -inf now
   ├─ For ready jobs: XADD back to jobs:stream
   ├─ Increment attempts counter
   └─ Apply exponential backoff
```

**Database Schema for Job Management:**

```sql
-- Jobs table (job lifecycle tracking)
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type TEXT NOT NULL DEFAULT 'tarot_reading',
    schema_version TEXT NOT NULL DEFAULT '1',
    prompt_version TEXT,
    dedupe_key TEXT UNIQUE,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued',
        -- queued, processing, succeeded, failed, dlq
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 5,
    visibility_timeout_secs INT DEFAULT 60,
    worker_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_error TEXT
);

-- Job attempts history
CREATE TABLE job_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    attempt_number INT NOT NULL,
    worker_id TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    success BOOLEAN,
    error TEXT,
    processing_time_ms INT
);
```

**Retry Strategy:**

```rust
// Exponential backoff with jitter
pub fn calculate_backoff_delay(attempt: u32) -> Duration {
    const BASE_DELAY_SECS: u64 = 2;
    const MAX_DELAY_SECS: u64 = 60;
    
    let exponential_delay = BASE_DELAY_SECS * 2_u64.pow(attempt - 1);
    let capped_delay = exponential_delay.min(MAX_DELAY_SECS);
    
    // Full jitter: random between 0 and capped_delay
    let jitter = rand::random::<u64>() % capped_delay;
    Duration::from_secs(jitter)
}

// Example delays:
// Attempt 1: 0-2 seconds
// Attempt 2: 0-4 seconds
// Attempt 3: 0-8 seconds
// Attempt 4: 0-16 seconds
// Attempt 5: 0-32 seconds (max retries)
// Beyond 5: marked as DLQ
```

---

### 4. Background Worker (Rust Binary)

**Responsibilities:**
- Read jobs from Redis Streams
- Execute agent pipeline
- Persist results to database
- Handle retries and failures
- Monitor job health

**Worker Binary Structure:**

```rust
// src/bin/worker.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging & tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize connections
    let db_pool = create_db_pool(&config.database_url).await?;
    let redis_client = create_redis_client(&config.upstash_url)?;
    
    // Create state
    let app_state = AppState {
        db_pool: db_pool.clone(),
        redis_client: redis_client.clone(),
        gemini_client: create_gemini_client(&config.gemini_api_key)?,
        tarot_pipeline: TarotPipeline::new()?,
    };
    
    // Create consumer group (one-time setup)
    setup_consumer_group(&redis_client).await?;
    
    // Start worker loop
    start_worker(app_state).await?;
    
    // Also run scheduler for delayed retries
    spawn_scheduler_task(&redis_client, &db_pool);
    
    Ok(())
}

// Main worker loop
async fn start_worker(state: AppState) -> Result<(), WorkerError> {
    loop {
        // Read next job from queue
        let entries = state.redis_client
            .xreadgroup(
                "jobs:group",
                "worker-1",
                &[("jobs:stream", ">")],
                None,  // No count limit
                Some(5000),  // 5 second timeout
            )
            .await?;
        
        // Process each job
        for (stream_key, entries) in entries {
            for (entry_id, fields) in entries {
                match process_job(&state, &fields).await {
                    Ok(_) => {
                        // Acknowledge on success
                        state.redis_client
                            .xack(&stream_key, "jobs:group", &[entry_id])
                            .await?;
                    }
                    Err(e) => {
                        // Handle failure (retry or DLQ)
                        handle_job_failure(&state, &fields, e).await?;
                    }
                }
            }
        }
    }
}

// Process individual job
async fn process_job(
    state: &AppState,
    fields: &HashMap<String, String>,
) -> Result<(), WorkerError> {
    let job_id: Uuid = fields.get("job_id")
        .ok_or(WorkerError::MissingJobId)?
        .parse()?;
    
    let payload: serde_json::Value = serde_json::from_str(
        fields.get("payload").ok_or(WorkerError::MissingPayload)?
    )?;
    
    // Update job status to processing
    update_job_status(&state.db_pool, job_id, "processing").await?;
    
    let start_time = Instant::now();
    
    // Execute tarot pipeline
    let result = state.tarot_pipeline
        .process_reading(&payload["question"].as_str().unwrap())
        .await;
    
    let processing_time_ms = start_time.elapsed().as_millis() as i32;
    
    // Save result
    match result {
        Ok(tarot_response) => {
            // Save to tarot_readings table
            save_tarot_reading(&state.db_pool, job_id, tarot_response).await?;
            
            // Update job status to succeeded
            update_job_status(&state.db_pool, job_id, "succeeded").await?;
            
            // Record attempt
            record_job_attempt(
                &state.db_pool,
                job_id,
                true,
                None,
                processing_time_ms,
            ).await?;
            
            tracing::info!(
                job_id = %job_id,
                processing_time_ms = processing_time_ms,
                "Job processed successfully"
            );
        }
        Err(e) => {
            // Record failed attempt
            record_job_attempt(
                &state.db_pool,
                job_id,
                false,
                Some(e.to_string()),
                processing_time_ms,
            ).await?;
            
            // Return error for retry logic
            return Err(WorkerError::ProcessingFailed(e.to_string()));
        }
    }
    
    Ok(())
}
```

---

### 5. Prompt Management System

**Responsibilities:**
- Secure storage of proprietary prompts
- Easy runtime updates without redeployment
- Version tracking for prompt iterations
- Support for multiple agents

**Implementation:**

```rust
// Prompt manager for secure handling
pub struct PromptManager {
    pub question_filter: String,
    pub question_analysis: String,
    pub reading_agent: String,
}

impl PromptManager {
    // Load from encrypted environment variables
    pub fn from_env() -> Result<Self, PromptError> {
        Ok(Self {
            question_filter: decode_base64(
                &env::var("PROMPT_QUESTION_FILTER")?
            )?,
            question_analysis: decode_base64(
                &env::var("PROMPT_QUESTION_ANALYSIS")?
            )?,
            reading_agent: decode_base64(
                &env::var("PROMPT_READING_AGENT")?
            )?,
        })
    }
    
    // Fetch from remote secure service (future)
    pub async fn from_remote_service() -> Result<Self, PromptError> {
        let client = reqwest::Client::new();
        let response = client
            .get(&env::var("PROMPT_SERVICE_URL")?)
            .header("Authorization", format!("Bearer {}",
                env::var("PROMPT_API_KEY")?))
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

// Usage in agents
pub async fn create_question_filter_chain(
    prompts: &PromptManager,
    gemini_llm: &GeminiLLM,
) -> Result<LLMChain, Error> {
    let prompt_template = PromptTemplate::from_template(
        &prompts.question_filter
    )?;
    
    let chain = LLMChain::new(gemini_llm, prompt_template);
    Ok(chain)
}
```

---

### 6. Payment Processing (Stripe Integration)

**Responsibilities:**
- Create payment intents
- Process payments securely
- Handle webhooks
- Update user wallet
- Track payment history

**Flow:**

```
Frontend
  ├─ Display payment packages
  ├─ User selects package
  ├─ Stripe Elements collects card info
  └─ POST /api/v1/payment/create-intent
     
     ↓
     
Backend API
  ├─ Validate user & package
  ├─ Create Stripe PaymentIntent
  ├─ Return client_secret to frontend
  └─ Store pending payment in DB
     
     ↓
     
Frontend
  ├─ Confirm payment with Stripe Elements
  ├─ Send confirmation_intent_id to backend
  └─ POST /api/v1/payment/confirm
     
     ↓
     
Backend API
  ├─ Verify payment intent with Stripe
  ├─ If succeeded:
  │  ├─ Add stars to user wallet
  │  ├─ Record wallet transaction
  │  ├─ Update payment status='succeeded'
  │  └─ Return success response
  └─ If failed:
     ├─ Update payment status='failed'
     └─ Return error response
     
     ↓
     
Stripe Webhook
  ├─ Send payment.intent.succeeded
  ├─ Backend verifies signature
  ├─ Double-check database state
  └─ Log for reconciliation
```

**Stripe Integration Code:**

```rust
pub async fn create_payment_intent(
    State(state): State<AppState>,
    claims: UserClaims,
    Json(request): Json<CreatePaymentIntentRequest>,
) -> Result<Json<PaymentIntentResponse>, AppError> {
    // Find package
    let package = STAR_PACKAGES
        .iter()
        .find(|p| p.package_type == request.package)
        .ok_or(AppError::InvalidPackage)?;
    
    // Create Stripe intent
    let params = CreatePaymentIntent {
        amount: Some(package.price),
        currency: Some(Currency::Thb),
        ..Default::default()
    };
    
    let intent = PaymentIntent::create(
        &state.stripe_client,
        params
    ).await?;
    
    // Save to database (status='pending')
    sqlx::query!(
        r#"
        INSERT INTO payments (
            id, user_id, stripe_payment_intent_id,
            amount, currency, status, package_type,
            stars_purchased, stars_bonus
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        Uuid::new_v4(),
        Uuid::parse(&claims.sub)?,
        intent.id.to_string(),
        package.price,
        "THB",
        "pending",
        format!("{:?}", package.package_type),
        package.stars,
        package.bonus_stars,
    )
    .execute(&state.db_pool)
    .await?;
    
    Ok(Json(PaymentIntentResponse {
        client_secret: intent.client_secret.unwrap(),
        payment_intent_id: intent.id.to_string(),
        amount: package.price,
        currency: "thb".to_string(),
    }))
}

pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    // Verify webhook signature
    let signature = headers.get("stripe-signature")
        .ok_or(AppError::MissingStripeSignature)?
        .to_str()?;
    
    let event = Webhook::construct_event(
        &body,
        signature,
        &state.stripe_webhook_secret,
    )?;
    
    // Handle payment success
    if event.type_ == EventType::PaymentIntentSucceeded {
        if let Some(intent) = event.data.object.as_payment_intent() {
            handle_payment_success(&state, intent).await?;
        }
    }
    
    Ok(StatusCode::OK)
}

async fn handle_payment_success(
    state: &AppState,
    intent: &PaymentIntent,
) -> Result<(), AppError> {
    // Find payment in database
    let payment = sqlx::query_scalar!(
        "SELECT user_id FROM payments WHERE stripe_payment_intent_id = $1",
        intent.id.to_string()
    )
    .fetch_one(&state.db_pool)
    .await?;
    
    let user_id: Uuid = Uuid::parse(&payment)?;
    
    // Get package for stars amount
    let package = STAR_PACKAGES.iter()
        .find(|p| p.price == intent.amount.unwrap_or(0))
        .ok_or(AppError::InvalidPackage)?;
    
    // Add stars to wallet
    let total_stars = package.stars + package.bonus_stars;
    sqlx::query!(
        r#"
        UPDATE users
        SET star_balance = star_balance + $1
        WHERE id = $2
        "#,
        total_stars as i64,
        user_id
    )
    .execute(&state.db_pool)
    .await?;
    
    // Update payment status
    sqlx::query!(
        "UPDATE payments SET status = 'succeeded' WHERE stripe_payment_intent_id = $1",
        intent.id.to_string()
    )
    .execute(&state.db_pool)
    .await?;
    
    Ok(())
}
```

---

## Monorepo Structure

### Directory Layout

```
mimi-r-backend/
├── src/
│   ├── lib.rs                          # Shared library code
│   ├── bin/
│   │   ├── api.rs                      # HTTP API server binary
│   │   └── worker.rs                   # Background worker binary
│   ├── agents/                         # Agent pipeline implementations
│   │   ├── mod.rs
│   │   ├── question_filter.rs
│   │   ├── question_analyzer.rs
│   │   └── reading_agent.rs
│   ├── api/                            # API handlers
│   │   ├── mod.rs
│   │   ├── tarot.rs                    # Tarot reading endpoints
│   │   ├── payment.rs                  # Payment endpoints
│   │   ├── user.rs                     # User endpoints
│   │   └── health.rs                   # Health check
│   ├── middleware/                     # Tower middleware
│   │   ├── mod.rs
│   │   ├── auth.rs                     # Multi-provider JWT auth
│   │   ├── rate_limit.rs               # Rate limiting
│   │   └── error_handler.rs            # Error handling
│   ├── models/                         # Data structures
│   │   ├── mod.rs
│   │   ├── requests.rs                 # Request models
│   │   ├── responses.rs                # Response models
│   │   ├── database.rs                 # Database models (SQLx)
│   │   └── tarot.rs                    # Tarot-specific models
│   ├── services/                       # Business logic
│   │   ├── mod.rs
│   │   ├── tarot_service.rs            # Tarot reading logic
│   │   ├── payment_service.rs          # Payment processing
│   │   ├── user_service.rs             # User management
│   │   └── invite_service.rs           # Invitation system
│   ├── queue/                          # Queue & job management
│   │   ├── mod.rs
│   │   ├── producer.rs                 # Job enqueue logic
│   │   ├── consumer.rs                 # Job processing
│   │   ├── scheduler.rs                # Retry scheduler
│   │   └── types.rs                    # Job types
│   ├── utils/                          # Utilities
│   │   ├── mod.rs
│   │   ├── prompt_manager.rs           # Secure prompt handling
│   │   ├── gemini_client.rs            # Gemini API wrapper
│   │   ├── config.rs                   # Configuration management
│   │   ├── errors.rs                   # Error types
│   │   └── validators.rs               # Input validation
│   ├── db/                             # Database utilities
│   │   ├── mod.rs
│   │   └── migrations.rs               # Database migrations
│   └── main.rs                         # (not used, only bin)
│
├── migrations/                         # SQLx migrations
│   ├── 20250101000001_initial_schema.sql
│   ├── 20250101000002_payment_schema.sql
│   └── ...
│
├── prompts/                            # Prompt templates
│   ├── QUESTION-FILTER.md
│   ├── QUESTION-ANALYSIS.md
│   └── READING-AGENT.md
│
├── docs/                               # Documentation
│   ├── PRD.md                          # Product Requirements
│   ├── ARCHITECTURE.md                 # This file
│   ├── DATABASE.md                     # Database schema details
│   ├── QUEUES.md                       # Queue system details
│   ├── API.md                          # API documentation
│   ├── DEPLOYMENT.md                   # Deployment guide
│   └── SECURITY.md                     # Security guidelines
│
├── tests/                              # Integration tests
│   ├── tarot_pipeline.rs
│   ├── payment_flow.rs
│   └── queue_operations.rs
│
├── Cargo.toml                          # Workspace manifest
├── Cargo.lock                          # Lock file
├── Dockerfile                          # Multi-stage build
├── docker-compose.yml                  # Local development
├── .env.example                        # Environment template
├── .env.local                          # (not committed, local only)
├── .github/
│   └── workflows/
│       ├── ci.yml                      # Build & test
│       └── deploy.yml                  # Deploy to Render
│
├── .gitignore
├── README.md
└── CLAUDE.md / AGENTS.md               # Project guidelines
```

---

## Data Flow Architecture

### Request Flow Diagram

```
1. HTTP Request (Frontend → API)
   ├─ Headers: Authorization: Bearer {jwt_token}
   ├─ Method: POST
   ├─ Path: /api/v1/tarot/read
   └─ Body: { question: "..." }

2. API Receives Request (Axum)
   ├─ Extract JWT token
   ├─ Validate JWT signature (multi-provider)
   ├─ Verify user exists in DB
   ├─ Check user has ≥ 1 star
   ├─ Validate question (length 8-180 chars)
   └─ Check rate limit (100/minute)

3. Enqueue Job (Producer)
   ├─ Generate dedupe_key (HMAC(user_id + question))
   ├─ Check if job exists (idempotency)
   ├─ Create job record in DB (status='queued')
   ├─ XADD to Redis Streams (jobs:stream)
   └─ Return job_id + trace_id to frontend
   
   Response: { success: true, job_id: "uuid", status: "queued" }

4. Worker Processes Job (Background)
   ├─ XREADGROUP from jobs:stream
   ├─ Update job status='processing'
   ├─ Execute agent pipeline:
   │  ├─ Agent 1: Question Filter (LLM)
   │  ├─ Agent 2: Question Analyzer (LLM)
   │  ├─ Random Card Selection (Tool)
   │  └─ Agent 3: Reading Agent (LLM)
   ├─ Save results to tarot_readings table
   ├─ Deduct 1 star from user wallet
   ├─ Record wallet transaction
   ├─ Update job status='succeeded'
   └─ XACK stream entry

5. Frontend Polls Status (Optional)
   ├─ GET /api/v1/reading/{job_id}
   └─ Response includes: status, results (if completed)
```

---

## Queue & Worker Architecture

### Redis Streams Implementation

**Stream Entries Structure:**

```
Stream: jobs:stream
Entry Format: {
    job_id: UUID,
    type: "tarot_reading",
    schema_version: "1",
    prompt_version: "v2025-11-20-a",
    dedupe_key: "user:123:abc123" (optional),
    payload: "{ user_id, question, ... }",
    created_at: "2025-11-20T08:00:00Z",
    attempts: 0,
    max_attempts: 5,
    visibility_timeout_secs: 60,
    trace_id: "req-abc-123",
}
```

**Consumer Group Setup:**

```sql
# One-time initialization
XGROUP CREATE jobs:stream jobs:group $ MKSTREAM

# Consumer (worker-1, worker-2, etc.)
XREADGROUP GROUP jobs:group worker-1 COUNT 1 BLOCK 5000 STREAMS jobs:stream >
```

**Retry Strategy Implementation:**

```
On Job Failure:
  1. Record attempt details in job_attempts table
  2. Increment attempts counter
  3. Calculate backoff delay: base * 2^(attempts-1), capped at 60s
  4. Add full jitter: random(0, delay)
  5. ZADD to jobs:delayed with score = now + delay
  6. XACK original entry
  
Scheduler Loop (every 5 seconds):
  1. ZRANGEBYSCORE jobs:delayed -inf now
  2. For each ready job:
     a. XADD back to jobs:stream
     b. Update job_attempts table
     c. ZREM from jobs:delayed
  
On Max Retries Exceeded (attempts >= 5):
  1. Update job status='dlq'
  2. XADD to jobs:dlq stream
  3. Send alert to Sentry
  4. Keep for manual review/debugging
```

**Consumer Group Monitoring:**

```
Health Checks:
- XINFO STREAM jobs:stream    # See stream length
- XINFO GROUPS jobs:stream    # Consumer group status
- XPENDING jobs:stream jobs:group  # Pending messages
- XCLAIM for visibility lease reclaim (on worker death)
```

---

## Security Architecture

### Multi-Provider Authentication

**Supported Providers:**
- Clerk (clerk.dev) - JWT via JWKS
- Auth0 (auth0.com) - JWT via JWKS
- Firebase - JWT via public key
- Custom (future) - Custom OAuth flow

**JWT Validation Flow:**

```
1. Extract Authorization header
2. Parse JWT header to detect issuer
3. Fetch public key for issuer (cached JWKS)
4. Verify JWT signature with RS256 algorithm
5. Check exp, aud, iss claims
6. Sync external user to internal database
7. Return user context for request
```

### API Key Management (Phase 1)

```
Environment Variables:
  TTRT_DEV_API_KEY=ttrt_dev_xxxxx
  TTRT_STAGING_API_KEY=ttrt_staging_xxxxx
  TTRT_PROD_API_KEY=ttrt_prod_xxxxx
  
Usage:
  curl -H "Authorization: Bearer ttrt_prod_xxxxx" \
       https://api.mimivibe.com/api/v1/tarot/read
  
Validation:
  1. Extract token from header
  2. Simple string comparison with env var
  3. Apply rate limiting (100 req/min per key)
  4. No database lookup required
```

### Prompt Security

```
Storage:
  - Prompts stored as base64 in environment variables
  - Never exposed in source code or logs
  - Encrypted in Render environment dashboard
  
Loading:
  - Load at startup: PromptManager::from_env()
  - Future: Remote prompt service with API key auth
  
Access Control:
  - Only API/Worker binaries can access
  - Separate keys per environment
  - All prompt changes logged via audit trail
```

### Data Privacy

```
PII Handling:
  - Questions considered sensitive data
  - Encrypted in database at rest (future)
  - Limited access via RLS policies (future)
  
Payment Data:
  - Never store credit card details
  - Use Stripe Elements (tokenization)
  - PCI DSS compliance via Stripe
  
User Isolation:
  - Row-level security in database
  - Users can only access their own data
  - Admin-only analytics queries
```

---

## Database Architecture

### Entity Relationships

```
users (1) ──────────── (∞) wallet_transactions
   ↑                           ↓
   │ invited_by                └─ related_payment_id
   │                                  ↑
   ├─ (1) ─────────────────────────────┤
   │                                    │
   │ ┌──────────────────────────────────┘
   ├──── (∞) payments
   │ (1)        ↓
   ├──────────── (∞) tarot_readings
   │ (1)
   └──────────── (∞) user_invites
                      ↓
                    users (invitee_id)
```

### Key Tables

**users** - Multi-provider user accounts
```sql
id, external_id, external_provider, email, name, picture_url,
star_balance, coin_balance, invite_code, invited_by, total_invites,
tier, created_at, last_login_at, updated_at
```

**payments** - Stripe payment records
```sql
id, user_id, stripe_payment_intent_id, amount, currency, status,
package_type, stars_purchased, stars_bonus, created_at, updated_at
```

**wallet_transactions** - All user currency transactions
```sql
id, user_id, transaction_type, amount, currency, balance_after,
metadata, related_payment_id, created_at
```

**tarot_readings** - Completed readings history
```sql
id, user_id, question, question_length, card_count, cards, reading,
question_analysis, processing_time_ms, created_at
```

**jobs** - Job queue status tracking
```sql
id, type, schema_version, prompt_version, dedupe_key, payload,
status, attempts, max_attempts, worker_id, created_at, updated_at, last_error
```

**job_attempts** - Retry history and metrics
```sql
id, job_id, attempt_number, worker_id, started_at, finished_at,
success, error, processing_time_ms
```

---

## Deployment Architecture

### Render Setup

**Two Services from Single Repository:**

1. **mimivibe-api** (Web Service)
   ```
   Build Command: cargo build --release --bin api
   Start Command: ./target/release/api
   Environment: API environment variables
   Region: Singapore (ap-southeast-1)
   Instance Type: Starter ($7/month)
   ```

2. **mimivibe-worker** (Background Service)
   ```
   Build Command: cargo build --release --bin worker
   Start Command: ./target/release/worker
   Environment: Worker environment variables
   Region: Singapore (ap-southeast-1)
   Instance Type: Starter ($7/month)
   Scaling: 2-3 concurrent workers (future)
   ```

**Environment Variables:**

```
[API Service]
DATABASE_URL=postgresql://...
UPSTASH_REDIS_URL=https://...
STRIPE_API_KEY=sk_test_...
GEMINI_API_KEY=...
CLERK_ISSUER=https://...
AUTH0_ISSUER=https://...
FIREBASE_PROJECT_ID=...
SENTRY_DSN=...
ENVIRONMENT=production

[Worker Service]
DATABASE_URL=postgresql://... (same)
UPSTASH_REDIS_URL=https://... (same)
GEMINI_API_KEY=... (same)
SENTRY_DSN=... (same)
ENVIRONMENT=production
WORKER_CONCURRENCY=5
MAX_PROCESSING_TIME_MS=30000
```

**CI/CD Pipeline (GitHub Actions):**

```yaml
# Build and test on every push
- cargo build --release --bins
- cargo clippy -- -D warnings
- cargo fmt -- --check
- cargo test

# Deploy to Render on merge to main
- Build both binaries
- Render auto-deploys via webhook
- Health checks pass before marking as ready
```

---

## Scalability Design

### Horizontal Scaling

**API Tier (Stateless):**
```
LoadBalancer (Render managed)
    ↓
┌───────────┬───────────┬───────────┐
│ API-1     │ API-2     │ API-3     │
│ Instance  │ Instance  │ Instance  │
└───────────┴───────────┴───────────┘
    ↓
Database Connection Pool (shared)
Redis Connection (shared)
```

**Worker Tier (Distributed):**
```
┌──────────────┬──────────────┬──────────────┐
│ Worker-1     │ Worker-2     │ Worker-3     │
│ Concurrency=5│ Concurrency=5│ Concurrency=5│
└──────────────┴──────────────┴──────────────┘
         ↓          ↓          ↓
    Redis Streams (shared job queue)
         ↓
   PostgreSQL (shared DB)
```

**Database Optimization:**
```
- Connection pooling (SQLx)
- Appropriate indexes on frequently queried columns
- Partitioning for tarot_readings (by created_at)
- Read replicas for analytics queries
- Caching layer (future)
```

**Queue Optimization:**
```
- Redis Streams for in-flight jobs
- Consumer groups for load distribution
- Visibility lease (60s timeout)
- Auto-reclaim via XCLAIM
- Dead-letter queue for visibility into failures
```

---

## Integration Points

### External Services

**Google Gemini API**
```
Endpoint: https://generativelanguage.googleapis.com/v1beta/...
Method: Streaming or non-streaming text generation
Timeout: 30 seconds per call
Retry: 3x with exponential backoff
Cost: ~$0.50-1.00 per 1M input tokens
```

**Stripe API**
```
Endpoint: https://api.stripe.com/v1/...
Webhooks: https://api.mimivibe.com/webhooks/stripe
Signature Verification: HMAC-SHA256
Idempotency Keys: Prevent duplicate charges
```

**Upstash Redis**
```
Endpoint: https://[endpoint]-[project].upstash.io
Protocol: Redis protocol over TLS
Features: Streams, Sorted Sets, ZADD/ZRANGE
Retention: 7 days minimum
```

**PostgreSQL (Supabase)**
```
Connection: SSL/TLS enforced
Pool Size: 20 connections (API) + 10 (Worker)
Backup: Daily automatic backups
Replication: Single region (expandable)
```

**Sentry (Error Tracking)**
```
DSN: https://[key]@sentry.io/[project]
Breadcrumbs: Request/response logging
Performance: Transaction tracing (2%)
Alerts: Spike detection for errors
```

---

## Summary

This architecture provides:

✅ **Scalability**: Separate API and worker tiers with horizontal scaling
✅ **Reliability**: Queue-based job processing with retry logic and DLQ
✅ **Security**: Multi-provider auth, encrypted prompts, PCI compliance
✅ **Maintainability**: Monorepo structure, clean separation of concerns
✅ **Observability**: Structured logging, Sentry integration, health checks
✅ **Performance**: Async/await throughout, connection pooling, efficient queuing
✅ **Flexibility**: Easy to swap components (LLM provider, payment gateway, auth provider)

---

**Document Version**: 1.0  
**Last Updated**: November 20, 2025  
**Architecture Pattern**: Event-Driven with Agent Orchestration  
**Target Deployment**: Render (Web Service + Background Service)
