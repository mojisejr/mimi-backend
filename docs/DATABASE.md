# Database Documentation
## MimiVibe Backend - Database Schema & Design

### Table of Contents
1. [Database Overview](#database-overview)
2. [Technology Stack](#technology-stack)
3. [Schema Design Principles](#schema-design-principles)
4. [Data Models](#data-models)
5. [Table Definitions](#table-definitions)
6. [Indexes & Performance](#indexes--performance)
7. [Row Level Security (RLS)](#row-level-security-rls)
8. [Migration Strategy](#migration-strategy)
9. [Data Access Patterns](#data-access-patterns)
10. [Backup & Recovery](#backup--recovery)

---

## Database Overview

MimiVibe Backend uses **Supabase PostgreSQL** for reliable, scalable data storage with built-in Row Level Security (RLS) for multi-tenant isolation. The database is designed to support:

- **Multi-provider authentication** (Clerk, Auth0, Firebase, Custom)
- **Payment & wallet system** (Stars, Coins, transactions)
- **Tarot reading history** (questions, responses, analytics)
- **Job queue management** (Redis Streams metadata + PostgreSQL state)
- **Invitation rewards system** (viral growth tracking)

### Key Features
- ✅ JSONB support for flexible schema (LLM responses, metadata)
- ✅ Full-text search for reading history
- ✅ Row Level Security for data isolation
- ✅ Transactional integrity for wallet operations
- ✅ Comprehensive audit trail for payments & transactions
- ✅ ACID compliance for financial operations

---

## Technology Stack

### Database Engine
- **Primary**: Supabase PostgreSQL 15+
- **Connection Pool**: SQLx with connection pooling (20 max, 5 min)
- **ORM**: SQLx (type-safe, compile-time SQL validation)
- **Migrations**: SQLx built-in migration system
- **Job Queue**: Upstash Redis Streams (with PostgreSQL metadata)

### Database Tools
- **IDE**: pgAdmin or DBeaver
- **Monitoring**: Supabase dashboard + custom Prometheus metrics
- **Backup**: Supabase automated daily backups
- **Query Optimization**: EXPLAIN ANALYZE, Index monitoring

---

## Schema Design Principles

### 1. Multi-Tenancy via Auth Provider
Each user is identified by:
- `external_id` - User ID from auth provider (Clerk, Auth0, Firebase)
- `external_provider` - Provider name ("clerk", "auth0", "firebase", "custom")
- `id` - Internal UUID (primary key)

This allows:
- Multiple providers in single database
- Easy provider migration/switching
- External auth provider independence

### 2. Financial Transaction Integrity
**ACID Requirements:**
- All wallet transactions must be atomic
- Prevent double-spending with transactions
- Maintain referential integrity
- Audit trail for compliance

**Pattern:**
```sql
BEGIN TRANSACTION;
-- Check balance
-- Deduct stars
-- Create transaction record
-- Update reading count
COMMIT;
```

### 3. Flexible Metadata Storage
Use JSONB for:
- LLM responses and readings
- Payment provider metadata
- Agent pipeline analysis
- User preferences and settings

Benefits:
- Schema evolution without migrations
- Complex nested structures
- Full-text search capability
- Efficient JSON queries

### 4. Event-Driven Architecture
All significant operations create records:
- Wallet transactions
- Tarot readings
- Payment events
- Job processing attempts

Enables:
- Complete audit trail
- Analytics & reporting
- Debugging & troubleshooting
- User activity tracking

---

## Data Models

### Core Entity Relationships

```
users (1) ──┬──→ (many) wallet_transactions
            ├──→ (many) tarot_readings
            ├──→ (many) payments
            ├──→ (many) user_payment_methods
            ├──→ (many) user_invites (as inviter)
            └──→ (many) user_invites (as invitee)

payments (1) ──→ (many) wallet_transactions
            └──→ (many) job_attempts

tarot_readings (1) ──→ (1) user
                   └──→ (many) job_attempts

jobs (1) ──→ (many) job_attempts
         └──→ (many) wallet_transactions
```

### Type Definitions

#### UserTier Enum
```sql
CREATE TYPE user_tier AS ENUM (
    'bronze',   -- 1-5 invites (base rewards)
    'silver',   -- 6-15 invites (+20% bonus)
    'gold'      -- 16+ invites (+50% bonus)
);
```

#### TransactionType Enum
```sql
CREATE TYPE transaction_type AS ENUM (
    'purchase',         -- Buying stars
    'invite_reward',    -- Coins from invitations
    'exchange',         -- Converting coins to stars
    'reading',          -- Spending stars on reading
    'refund'            -- Payment refund/reversal
);
```

#### Currency Enum
```sql
CREATE TYPE currency AS ENUM (
    'STAR',     -- Hard currency (paid)
    'COIN'      -- Soft currency (earned)
);
```

#### JobStatus Enum
```sql
CREATE TYPE job_status AS ENUM (
    'queued',       -- Waiting in queue
    'processing',   -- Being processed
    'succeeded',    -- Completed successfully
    'failed',       -- Failed (retry pending)
    'dlq'           -- Dead Letter Queue (permanent failure)
);
```

#### PaymentStatus Enum
```sql
CREATE TYPE payment_status AS ENUM (
    'pending',      -- Awaiting payment
    'succeeded',    -- Payment completed
    'failed',       -- Payment declined
    'refunded'      -- Payment reversed
);
```

---

## Table Definitions

### 1. Users Table

**Purpose**: Store user accounts from multiple auth providers

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Authentication (Multi-Provider)
    external_id VARCHAR(255) NOT NULL,        -- Auth provider user ID
    external_provider VARCHAR(50) NOT NULL,   -- "clerk", "auth0", "firebase", "custom"
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    picture_url TEXT,

    -- MimiVibe Wallet System
    star_balance BIGINT NOT NULL DEFAULT 0,   -- Hard currency (stars)
    coin_balance BIGINT NOT NULL DEFAULT 0,   -- Soft currency (coins)

    -- Invitation & Tier System
    invite_code VARCHAR(10) UNIQUE NOT NULL GENERATED ALWAYS AS (
        substring(md5(id::text || NOW()::text), 1, 10)
    ) STORED,
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    total_invites INTEGER NOT NULL DEFAULT 0,
    tier user_tier NOT NULL DEFAULT 'bronze',

    -- Tracking & Audit
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    UNIQUE(external_id, external_provider),  -- One auth account per provider
    CHECK (star_balance >= 0),
    CHECK (coin_balance >= 0),
    CHECK (total_invites >= 0)
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_external_id_provider ON users(external_id, external_provider);
CREATE INDEX idx_users_invite_code ON users(invite_code);
CREATE INDEX idx_users_tier ON users(tier);
CREATE INDEX idx_users_created_at ON users(created_at DESC);
CREATE INDEX idx_users_last_login ON users(last_login_at DESC);

-- Enable Row Level Security
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Multi-provider support (external_id + external_provider composite unique)
- Invite code auto-generated from UUID + timestamp
- Tier system for rewards (bronze/silver/gold)
- Balanced wallet tracking (stars + coins)

---

### 2. Payments Table

**Purpose**: Track all payment transactions for audit & compliance

```sql
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Stripe Integration
    stripe_payment_intent_id VARCHAR(255) UNIQUE,  -- Stripe reference (nullable for pending)
    stripe_charge_id VARCHAR(255) UNIQUE,         -- Stripe charge ID
    stripe_metadata JSONB,                        -- Full Stripe response metadata

    -- Payment Details
    amount BIGINT NOT NULL,                       -- Amount in satang (smallest unit)
    currency VARCHAR(3) NOT NULL DEFAULT 'THB',
    status payment_status NOT NULL DEFAULT 'pending',
    package_type VARCHAR(20) NOT NULL,            -- "starter", "basic", "premium"

    -- Stars Allocation
    stars_purchased INTEGER NOT NULL CHECK (stars_purchased > 0),
    stars_bonus INTEGER NOT NULL DEFAULT 0,       -- Free bonus stars
    stars_total INTEGER GENERATED ALWAYS AS (
        stars_purchased + stars_bonus
    ) STORED,

    -- Error Tracking
    error_code VARCHAR(50),                       -- Stripe error code (if failed)
    error_message TEXT,                           -- Stripe error message (if failed)

    -- Tracking & Audit
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CHECK (amount > 0),
    CHECK (stars_total > 0)
);

-- Indexes
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_stripe_intent ON payments(stripe_payment_intent_id) WHERE stripe_payment_intent_id IS NOT NULL;
CREATE INDEX idx_payments_created_at ON payments(created_at DESC);
CREATE INDEX idx_payments_user_created ON payments(user_id, created_at DESC);

-- Enable RLS
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Stripe integration with full metadata storage
- Package type tracking for analytics
- Bonus stars separated from purchased
- Error logging for failed payments
- Composite indexes for common queries

---

### 3. Wallet Transactions Table

**Purpose**: Complete audit trail of all wallet operations

```sql
CREATE TABLE wallet_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Transaction Details
    transaction_type transaction_type NOT NULL,
    amount INTEGER NOT NULL,                      -- Amount in stars or coins
    currency currency NOT NULL,                   -- STAR or COIN
    balance_after BIGINT NOT NULL,               -- Balance after transaction

    -- Metadata & References
    metadata JSONB,                              -- Context-specific data
    related_payment_id UUID REFERENCES payments(id) ON DELETE SET NULL,
    related_reading_id UUID REFERENCES tarot_readings(id) ON DELETE SET NULL,
    related_invite_id UUID REFERENCES user_invites(id) ON DELETE SET NULL,

    -- Audit Trail
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CHECK (amount >= 0),
    CHECK (balance_after >= 0)
);

-- Indexes (critical for wallet queries)
CREATE INDEX idx_wallet_transactions_user_id ON wallet_transactions(user_id);
CREATE INDEX idx_wallet_transactions_type ON wallet_transactions(transaction_type);
CREATE INDEX idx_wallet_transactions_currency ON wallet_transactions(currency);
CREATE INDEX idx_wallet_transactions_user_created ON wallet_transactions(user_id, created_at DESC);
CREATE INDEX idx_wallet_transactions_created_at ON wallet_transactions(created_at DESC);

-- Enable RLS
ALTER TABLE wallet_transactions ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Atomic recording of all balance changes
- Cross-references to related operations
- JSONB metadata for flexible context
- Time-ordered queries for history
- Immutable records (no updates after creation)

---

### 4. Tarot Readings Table

**Purpose**: Store tarot reading history & LLM responses

```sql
CREATE TABLE tarot_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Question & Input
    question TEXT NOT NULL,
    question_length INTEGER NOT NULL,
    question_locale VARCHAR(10) DEFAULT 'th',    -- "th", "en", etc.

    -- Card Selection
    card_count INTEGER NOT NULL CHECK (card_count IN (3, 5)),
    cards JSONB NOT NULL,                        -- Array of selected cards

    -- LLM Processing
    question_analysis JSONB,                     -- Agent 2 output
    tarot_reading JSONB NOT NULL,                -- Agent 3 complete output
    processing_time_ms INTEGER,                  -- LLM processing time

    -- Job Tracking
    related_job_id UUID,                         -- Link to job queue (Upstash)
    job_attempts INTEGER DEFAULT 0,

    -- Metadata
    metadata JSONB,                              -- Additional context
    search_tsvector tsvector,                    -- For full-text search

    -- Audit & Tracking
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CHECK (processing_time_ms IS NULL OR processing_time_ms > 0)
);

-- Indexes
CREATE INDEX idx_tarot_readings_user_id ON tarot_readings(user_id);
CREATE INDEX idx_tarot_readings_created_at ON tarot_readings(created_at DESC);
CREATE INDEX idx_tarot_readings_user_created ON tarot_readings(user_id, created_at DESC);
CREATE INDEX idx_tarot_readings_locale ON tarot_readings(question_locale);

-- Full-text search index
CREATE INDEX idx_tarot_readings_fts ON tarot_readings USING gin(
    to_tsvector('thai', question)
);

-- Enable RLS
ALTER TABLE tarot_readings ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Stores complete LLM responses as JSONB
- Full-text search support for question analysis
- Processing time tracking for optimization
- Job queue reference for async processing
- Flexible metadata for future enhancements

---

### 5. Jobs Table (Queue Management)

**Purpose**: Track async tarot processing jobs

```sql
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(50) NOT NULL DEFAULT 'tarot_reading',

    -- Job Metadata
    schema_version VARCHAR(20) NOT NULL DEFAULT '1',
    prompt_version VARCHAR(50),
    dedupe_key VARCHAR(255) UNIQUE,              -- For idempotency

    -- Job State
    status job_status NOT NULL DEFAULT 'queued',
    payload JSONB NOT NULL,                      -- Job input data
    result JSONB,                                -- Job result (when succeeded)

    -- Processing Tracking
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    visibility_timeout_secs INTEGER DEFAULT 60,
    worker_id VARCHAR(255),

    -- Error Tracking
    last_error TEXT,
    last_error_at TIMESTAMP WITH TIME ZONE,

    -- Retry Scheduling
    next_retry_at TIMESTAMP WITH TIME ZONE,

    -- Audit & Timing
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,

    CHECK (attempts >= 0),
    CHECK (attempts <= max_attempts),
    CHECK (max_attempts > 0),
    CHECK (visibility_timeout_secs > 0)
);

-- Indexes (critical for job queue)
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_created_at ON jobs(created_at DESC);
CREATE INDEX idx_jobs_dedupe_key ON jobs(dedupe_key);
CREATE INDEX idx_jobs_worker_id ON jobs(worker_id);
CREATE INDEX idx_jobs_status_created ON jobs(status, created_at DESC);
CREATE INDEX idx_jobs_retry_at ON jobs(next_retry_at) WHERE status = 'failed';

-- Enable RLS
ALTER TABLE jobs ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Upstash Redis Streams metadata in PostgreSQL
- Idempotency via dedupe_key
- Retry scheduling with exponential backoff
- Worker tracking for load balancing
- Version control for schema/prompts

---

### 6. Job Attempts Table

**Purpose**: Complete audit trail of job processing

```sql
CREATE TABLE job_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,

    -- Attempt Details
    attempt_number INTEGER NOT NULL,
    worker_id VARCHAR(255),

    -- Processing Time
    started_at TIMESTAMP WITH TIME ZONE,
    finished_at TIMESTAMP WITH TIME ZONE,
    processing_time_ms INTEGER GENERATED ALWAYS AS (
        EXTRACT(EPOCH FROM (finished_at - started_at)) * 1000
    ) STORED,

    -- Outcome
    success BOOLEAN,
    error TEXT,
    error_code VARCHAR(50),

    -- Audit
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CHECK (processing_time_ms IS NULL OR processing_time_ms >= 0)
);

-- Indexes
CREATE INDEX idx_job_attempts_job_id ON job_attempts(job_id);
CREATE INDEX idx_job_attempts_worker_id ON job_attempts(worker_id);
CREATE INDEX idx_job_attempts_success ON job_attempts(success);
CREATE INDEX idx_job_attempts_created_at ON job_attempts(created_at DESC);

-- Enable RLS
ALTER TABLE job_attempts ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Complete job processing history
- Auto-calculated processing time
- Error tracking for debugging
- Worker performance metrics
- Immutable audit trail

---

### 7. User Payment Methods Table

**Purpose**: Store saved payment methods (cards)

```sql
CREATE TABLE user_payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Stripe Integration
    stripe_payment_method_id VARCHAR(255) UNIQUE,
    stripe_customer_id VARCHAR(255),

    -- Card Details (Tokenized)
    type VARCHAR(20) NOT NULL DEFAULT 'card',
    brand VARCHAR(20),                          -- "visa", "mastercard", etc.
    last4 VARCHAR(4),                          -- Last 4 digits (safe to display)
    exp_month INTEGER,
    exp_year INTEGER,

    -- Settings
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CHECK (exp_month >= 1 AND exp_month <= 12),
    CHECK (exp_year > 2025)
);

-- Indexes
CREATE INDEX idx_user_payment_methods_user_id ON user_payment_methods(user_id);
CREATE INDEX idx_user_payment_methods_is_default ON user_payment_methods(user_id, is_default);
CREATE INDEX idx_user_payment_methods_stripe_id ON user_payment_methods(stripe_payment_method_id);

-- Only one default payment method per user
CREATE UNIQUE INDEX idx_user_payment_methods_default ON user_payment_methods(user_id)
    WHERE is_default = true;

-- Enable RLS
ALTER TABLE user_payment_methods ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Stripe tokenized storage (no card data on our servers)
- Multiple payment methods per user
- Default method selection
- PCI DSS compliant
- Activation/deactivation without deletion

---

### 8. User Invites Table

**Purpose**: Track invitation rewards & viral growth

```sql
CREATE TABLE user_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Participants
    inviter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invitee_id UUID REFERENCES users(id) ON DELETE SET NULL,
    invite_code VARCHAR(10) NOT NULL REFERENCES users(invite_code),

    -- Reward
    coin_reward INTEGER,                       -- Coins awarded to inviter
    is_reward_claimed BOOLEAN NOT NULL DEFAULT false,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- "pending", "accepted", "expired"

    -- Metadata
    metadata JSONB,                            -- Campaign tracking, source, etc.

    -- Audit & Timing
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    accepted_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT (NOW() + INTERVAL '30 days'),

    CHECK (coin_reward IS NULL OR coin_reward > 0)
);

-- Indexes
CREATE INDEX idx_user_invites_inviter_id ON user_invites(inviter_id);
CREATE INDEX idx_user_invites_invitee_id ON user_invites(invitee_id) WHERE invitee_id IS NOT NULL;
CREATE INDEX idx_user_invites_status ON user_invites(status);
CREATE INDEX idx_user_invites_invite_code ON user_invites(invite_code);
CREATE INDEX idx_user_invites_created_at ON user_invites(created_at DESC);
CREATE INDEX idx_user_invites_expires_at ON user_invites(expires_at) WHERE status = 'pending';

-- Enable RLS
ALTER TABLE user_invites ENABLE ROW LEVEL SECURITY;
```

**Key Points:**
- Tracks invitation lifecycle (pending → accepted → expired)
- Reward management (claimed vs unclaimed)
- Campaign tracking via metadata
- Automatic expiration (30 days)
- Viral coefficient analytics

---

## Indexes & Performance

### Critical Indexes (Must Have)

```sql
-- User queries
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_external_id_provider ON users(external_id, external_provider);

-- Wallet queries (high volume)
CREATE INDEX idx_wallet_transactions_user_id ON wallet_transactions(user_id);
CREATE INDEX idx_wallet_transactions_user_created ON wallet_transactions(user_id, created_at DESC);

-- Job queue queries
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_status_created ON jobs(status, created_at DESC);

-- Payment tracking
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);

-- Reading history
CREATE INDEX idx_tarot_readings_user_id ON tarot_readings(user_id);
CREATE INDEX idx_tarot_readings_user_created ON tarot_readings(user_id, created_at DESC);
```

### Query Optimization Tips

**1. Pagination Pattern:**
```sql
-- Get user's last 20 readings
SELECT * FROM tarot_readings
WHERE user_id = $1
ORDER BY created_at DESC
LIMIT 20
OFFSET 0;

-- Better: use keyset pagination
SELECT * FROM tarot_readings
WHERE user_id = $1 AND created_at < $2
ORDER BY created_at DESC
LIMIT 20;
```

**2. Wallet Balance Efficiency:**
```sql
-- Use cached balance in users table
SELECT star_balance, coin_balance FROM users WHERE id = $1;

-- Instead of calculating from transactions
SELECT
    COALESCE(SUM(CASE WHEN currency = 'STAR' THEN amount ELSE 0 END), 0) as stars,
    COALESCE(SUM(CASE WHEN currency = 'COIN' THEN amount ELSE 0 END), 0) as coins
FROM wallet_transactions
WHERE user_id = $1;
```

**3. Job Queue Queries:**
```sql
-- Get pending jobs (use index)
SELECT * FROM jobs
WHERE status = 'queued'
ORDER BY created_at ASC
LIMIT 100;

-- Get retry candidates (use partial index)
SELECT * FROM jobs
WHERE status = 'failed' AND next_retry_at <= NOW()
ORDER BY next_retry_at ASC
LIMIT 50;
```

### Index Monitoring

```sql
-- Check missing indexes
SELECT * FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Identify slow queries
SELECT query, calls, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Monitor index size
SELECT schemaname, tablename, indexname, pg_size_pretty(pg_relation_size(indexrelid))
FROM pg_indexes i
JOIN pg_stat_user_indexes s ON i.indexname = s.indexrelname
ORDER BY pg_relation_size(indexrelid) DESC;
```

---

## Row Level Security (RLS)

### RLS Policies (Multi-Tenant Isolation)

**Users Table - Self Access Only**
```sql
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

CREATE POLICY "users_view_own_profile" ON users
    FOR SELECT
    USING (auth.uid()::text = id::text);

CREATE POLICY "users_update_own_profile" ON users
    FOR UPDATE
    USING (auth.uid()::text = id::text)
    WITH CHECK (auth.uid()::text = id::text);

-- Prevent self-deletion
CREATE POLICY "users_prevent_delete" ON users
    FOR DELETE
    USING (false);
```

**Wallet Transactions - User's Own Transactions Only**
```sql
ALTER TABLE wallet_transactions ENABLE ROW LEVEL SECURITY;

CREATE POLICY "wallet_transactions_view_own" ON wallet_transactions
    FOR SELECT
    USING (user_id = auth.uid());

-- Prevent direct inserts (use function instead)
CREATE POLICY "wallet_transactions_prevent_insert" ON wallet_transactions
    FOR INSERT
    WITH CHECK (false);

CREATE POLICY "wallet_transactions_prevent_delete" ON wallet_transactions
    FOR DELETE
    USING (false);
```

**Tarot Readings - User's Own Readings Only**
```sql
ALTER TABLE tarot_readings ENABLE ROW LEVEL SECURITY;

CREATE POLICY "tarot_readings_view_own" ON tarot_readings
    FOR SELECT
    USING (user_id = auth.uid());

CREATE POLICY "tarot_readings_insert_own" ON tarot_readings
    FOR INSERT
    WITH CHECK (user_id = auth.uid());

CREATE POLICY "tarot_readings_prevent_delete" ON tarot_readings
    FOR DELETE
    USING (false);
```

**Payments - User's Own Payments Only**
```sql
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;

CREATE POLICY "payments_view_own" ON payments
    FOR SELECT
    USING (user_id = auth.uid());

CREATE POLICY "payments_prevent_insert" ON payments
    FOR INSERT
    WITH CHECK (false);

CREATE POLICY "payments_prevent_update" ON payments
    FOR UPDATE
    USING (false);
```

### RLS Helper Functions

```sql
-- Check if user is admin
CREATE OR REPLACE FUNCTION is_admin(user_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS(
        SELECT 1 FROM users
        WHERE id = user_id AND external_provider = 'admin'
    );
END;
$$ LANGUAGE plpgsql;

-- Check if user can access reading
CREATE OR REPLACE FUNCTION user_can_access_reading(reading_id UUID, user_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS(
        SELECT 1 FROM tarot_readings
        WHERE id = reading_id AND user_id = user_id
    );
END;
$$ LANGUAGE plpgsql;
```

---

## Migration Strategy

### Using SQLx Migrations

**Create Migrations:**
```bash
# Create initial schema
sqlx migrate add initial_schema

# Create payment system
sqlx migrate add add_payments_and_wallet

# Create job queue
sqlx migrate add add_job_queue
```

**Migration File Structure:**
```
migrations/
├── 001_initial_schema.sql
├── 002_add_payments_and_wallet.sql
├── 003_add_job_queue.sql
└── 004_add_rls_policies.sql
```

**Run Migrations:**
```bash
# Run all pending migrations
sqlx migrate run

# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert
```

### Example Migration File (001_initial_schema.sql)

```sql
-- Create enums
CREATE TYPE user_tier AS ENUM ('bronze', 'silver', 'gold');
CREATE TYPE transaction_type AS ENUM ('purchase', 'invite_reward', 'exchange', 'reading', 'refund');
CREATE TYPE currency AS ENUM ('STAR', 'COIN');
CREATE TYPE job_status AS ENUM ('queued', 'processing', 'succeeded', 'failed', 'dlq');
CREATE TYPE payment_status AS ENUM ('pending', 'succeeded', 'failed', 'refunded');

-- Create tables
CREATE TABLE users (
    -- ... (full definition from above)
);

-- ... (rest of tables)

-- Create indexes
-- ... (all indexes)

-- Enable RLS
-- ... (all RLS policies)
```

---

## Data Access Patterns

### Using SQLx in Rust

**Get User Profile:**
```rust
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT
            id, external_id, external_provider, email, name, picture_url,
            star_balance, coin_balance, invite_code, invited_by,
            total_invites, tier, created_at, last_login_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_user_by_external_id(
    pool: &PgPool,
    external_id: &str,
    provider: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT *
        FROM users
        WHERE external_id = $1 AND external_provider = $2
        "#,
        external_id,
        provider
    )
    .fetch_one(pool)
    .await
}
```

**Get Wallet Balance:**
```rust
pub async fn get_user_wallet(pool: &PgPool, user_id: Uuid) -> Result<Wallet, sqlx::Error> {
    sqlx::query_as!(
        Wallet,
        r#"
        SELECT star_balance, coin_balance
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
}
```

**Create Tarot Reading:**
```rust
use serde_json::json;

pub async fn create_tarot_reading(
    pool: &PgPool,
    user_id: Uuid,
    question: String,
    cards: Vec<Card>,
    reading: Reading,
    analysis: Option<QuestionAnalysis>,
) -> Result<UUID, sqlx::Error> {
    let cards_json = serde_json::to_value(&cards)?;
    let reading_json = serde_json::to_value(&reading)?;
    let analysis_json = analysis.map(|a| serde_json::to_value(&a)).transpose()?;

    sqlx::query_scalar!(
        r#"
        INSERT INTO tarot_readings (
            user_id, question, question_length, card_count,
            cards, tarot_reading, question_analysis
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        user_id,
        &question,
        question.len() as i32,
        cards.len() as i32,
        cards_json,
        reading_json,
        analysis_json
    )
    .fetch_one(pool)
    .await
}
```

**Record Wallet Transaction:**
```rust
pub async fn record_wallet_transaction(
    pool: &PgPool,
    user_id: Uuid,
    transaction_type: TransactionType,
    amount: i32,
    currency: Currency,
    related_payment_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    // Get current balance
    let current_balance = if currency == Currency::STAR {
        sqlx::query_scalar!("SELECT star_balance FROM users WHERE id = $1", user_id)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar!("SELECT coin_balance FROM users WHERE id = $1", user_id)
            .fetch_one(pool)
            .await?
    };

    let new_balance = current_balance + amount as i64;

    // Insert transaction record
    sqlx::query!(
        r#"
        INSERT INTO wallet_transactions (
            user_id, transaction_type, amount, currency, balance_after, related_payment_id
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        transaction_type as TransactionType,
        amount,
        currency as Currency,
        new_balance,
        related_payment_id
    )
    .execute(pool)
    .await?;

    // Update user balance
    if currency == Currency::STAR {
        sqlx::query!(
            "UPDATE users SET star_balance = $1, updated_at = NOW() WHERE id = $2",
            new_balance,
            user_id
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            "UPDATE users SET coin_balance = $1, updated_at = NOW() WHERE id = $2",
            new_balance,
            user_id
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
```

### Transaction Safety

```rust
use sqlx::Transaction;

pub async fn purchase_stars(
    pool: &PgPool,
    user_id: Uuid,
    payment_id: Uuid,
    stars_amount: i32,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Verify payment succeeded
    let payment = sqlx::query_as!(
        Payment,
        "SELECT * FROM payments WHERE id = $1",
        payment_id
    )
    .fetch_one(&mut *tx)
    .await?;

    if payment.status != "succeeded" {
        return Err(sqlx::Error::RowNotFound);
    }

    // 2. Add stars to user
    sqlx::query!(
        "UPDATE users SET star_balance = star_balance + $1, updated_at = NOW() WHERE id = $2",
        stars_amount as i64,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Record transaction
    sqlx::query!(
        r#"
        INSERT INTO wallet_transactions (
            user_id, transaction_type, amount, currency, balance_after, related_payment_id
        )
        SELECT $1, 'purchase'::transaction_type, $2, 'STAR'::currency, star_balance, $3
        FROM users WHERE id = $1
        "#,
        user_id,
        stars_amount,
        payment_id
    )
    .execute(&mut *tx)
    .await?;

    // 4. Commit all changes
    tx.commit().await?;

    Ok(())
}
```

---

## Backup & Recovery

### Supabase Automated Backups

Supabase provides:
- ✅ Daily automated backups (retained for 7 days)
- ✅ Point-in-time recovery (PITR) capability
- ✅ Cross-region backup redundancy
- ✅ One-click backup restoration

**Enable in Supabase Dashboard:**
1. Go to Project Settings → Backups
2. Enable "Backups" toggle
3. Set retention policy (default: 7 days)

### Manual Backup

```bash
# Using pg_dump via Supabase connection
pg_dump \
  -h [project-ref].supabase.co \
  -U postgres \
  -d postgres \
  > backup-$(date +%Y%m%d).sql

# Or using pg_basebackup for streaming backup
pg_basebackup \
  -h [project-ref].supabase.co \
  -U postgres \
  -D ./backup \
  -Ft \
  -z \
  -P
```

### Restore Process

```bash
# Restore from SQL dump
psql -h [new-host] -U postgres < backup.sql

# Verify restore
psql -h [new-host] -U postgres -d postgres \
  -c "SELECT count(*) as users FROM users;"
```

### Backup Verification

```sql
-- Check backup status in Supabase
SELECT
    backup_id,
    created_at,
    status,
    size_bytes
FROM pgsql_backups
ORDER BY created_at DESC
LIMIT 5;
```

---

## Summary

This database schema provides:

✅ **Multi-tenancy**: Support for multiple auth providers  
✅ **Financial integrity**: ACID transactions for wallet operations  
✅ **Audit trail**: Complete history of all operations  
✅ **Performance**: Optimized indexes for common queries  
✅ **Security**: Row Level Security for data isolation  
✅ **Scalability**: JSONB flexibility for evolving data  
✅ **Reliability**: Built-in backup & recovery  

For implementation, follow the migration strategy and use SQLx for type-safe database operations in Rust.

---

**Document Version**: 1.0  
**Last Updated**: November 20, 2025  
**Status**: Ready for Implementation  
