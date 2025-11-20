# API Documentation
## MimiVibe Backend - Complete API Reference

### Table of Contents
1. [Overview](#overview)
2. [Base URL](#base-url)
3. [Authentication](#authentication)
4. [Response Format](#response-format)
5. [Error Handling](#error-handling)
6. [API Endpoints](#api-endpoints)
   - [Tarot Reading Endpoints](#tarot-reading-endpoints)
   - [User & Wallet Endpoints](#user--wallet-endpoints)
   - [Payment Endpoints](#payment-endpoints)
   - [Invitation System Endpoints](#invitation-system-endpoints)
   - [Admin Endpoints](#admin-endpoints)
7. [Data Models](#data-models)
8. [Rate Limiting](#rate-limiting)
9. [Examples](#examples)

---

## Overview

MimiVibe Backend provides a comprehensive API for tarot reading services with integrated payment processing, gamification through a coin/star system, and user wallet management.

**Key Features:**
- Multi-agent LLM pipeline for intelligent tarot readings
- Flexible authentication supporting Clerk, Auth0, Firebase, and custom providers
- Stripe payment integration for star purchases
- Invite reward system with dynamic coin rewards
- Complete user wallet and transaction history tracking

**Technology Stack:**
- Language: Rust
- Framework: Axum (async, tower middleware)
- LLM: Google Gemini API with LangChain Rust
- Database: PostgreSQL (Supabase) with SQLx
- Payment: Stripe (Cards, PromptPay in Phase 2)
- Queue System: Upstash Redis Streams (MVP with background workers)

---

## Base URL

```
Development:    http://localhost:3000
Staging:        https://staging.api.mimivibe.com
Production:     https://api.mimivibe.com
```

---

## Authentication

### Multi-Provider JWT Authentication

MimiVibe supports multiple authentication providers:
- **Clerk** (https://clerk.dev)
- **Auth0** (https://auth0.com)
- **Firebase** (https://firebase.google.com)
- **Custom** (Internal authentication system)

### Authorization Header

```http
Authorization: Bearer <jwt_token>
```

The JWT token is obtained from your authentication provider and must be included in all protected endpoint requests.

### Environment-Based Authentication

**Phase 1 (Development) - Simple API Key:**

For internal testing and development, a simple API key approach is available:

```bash
# .env.local configuration
TTRT_DEV_API_KEY=ttrt_dev_a1b2c3d4e5f6
TTRT_STAGING_API_KEY=ttrt_staging_b7c8d9e0f1g2
TTRT_PROD_API_KEY=ttrt_prod_h3i4j5k6l7m8

# Request header
X-API-Key: ttrt_dev_a1b2c3d4e5f6
```

### Token Lifecycle

- **Expiration**: Depends on provider (typically 24-48 hours)
- **Refresh**: Use provider's refresh token mechanism
- **Revocation**: Handled by auth provider

---

## Response Format

### Standard Success Response

```json
{
  "success": true,
  "data": {
    // Response payload based on endpoint
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

### Standard Error Response

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": "Additional context (optional)"
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

### HTTP Status Codes

| Code | Meaning | Common Cause |
|------|---------|--------------|
| 200 | OK | Successful request |
| 201 | Created | Resource created successfully |
| 202 | Accepted | Request accepted for processing (async) |
| 400 | Bad Request | Invalid input validation |
| 401 | Unauthorized | Missing/invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource not found |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |
| 502 | Bad Gateway | LLM API failure |
| 503 | Service Unavailable | Maintenance or overload |
| 504 | Gateway Timeout | LLM API timeout |

---

## Error Handling

### Error Response Examples

#### Validation Error (400)
```json
{
  "success": false,
  "error": {
    "code": "QUESTION_TOO_SHORT",
    "message": "Question must be at least 8 characters long",
    "details": "Received 5 characters, minimum required is 8"
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

#### Authentication Error (401)
```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Missing or invalid authentication token",
    "details": "Ensure 'Authorization: Bearer <token>' is provided"
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

#### Rate Limit Error (429)
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "API rate limit exceeded",
    "details": "Maximum 100 requests per minute allowed"
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

#### LLM Processing Error (502)
```json
{
  "success": false,
  "error": {
    "code": "LLM_API_ERROR",
    "message": "Failed to process reading with Gemini API",
    "details": "The AI service encountered an error. Please try again."
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

#### Payment Error (402)
```json
{
  "success": false,
  "error": {
    "code": "PAYMENT_DECLINED",
    "message": "Payment method was declined",
    "details": "Your card issuer declined the transaction. Please try another payment method."
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

### Common Error Codes

| Code | HTTP | Description |
|------|------|-------------|
| UNAUTHORIZED | 401 | Missing or invalid JWT token |
| FORBIDDEN | 403 | User lacks required permissions |
| NOT_FOUND | 404 | Resource does not exist |
| VALIDATION_ERROR | 400 | Input validation failed |
| QUESTION_TOO_SHORT | 400 | Question < 8 characters |
| QUESTION_TOO_LONG | 400 | Question > 180 characters |
| QUESTION_FILTER_REJECTED | 422 | Question inappropriate for tarot |
| INSUFFICIENT_STARS | 400 | User doesn't have enough stars |
| PAYMENT_DECLINED | 402 | Card declined or payment failed |
| PAYMENT_PROCESSING_ERROR | 502 | Stripe API error |
| LLM_API_ERROR | 502 | Gemini API failure |
| LLM_TIMEOUT | 504 | LLM request exceeded 30 seconds |
| RATE_LIMIT_EXCEEDED | 429 | Too many requests |
| INTERNAL_ERROR | 500 | Server error |

---

## API Endpoints

### Tarot Reading Endpoints

#### POST /api/v1/tarot/read
**Read a Tarot Card**

Processes a question through the complete agent pipeline and returns a tarot reading.

**Authentication**: Required (JWT Bearer Token)

**Request Headers**:
```http
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request Body**:
```json
{
  "question": "string (8-180 characters)",
  "trace_id": "string (optional - for request tracing)"
}
```

**Parameters**:
- `question` (required, string): The user's question for tarot reading
  - Minimum 8 characters
  - Maximum 180 characters
  - Must not be empty after trimming whitespace
- `trace_id` (optional, string): Request correlation ID for logging

**Processing Pipeline**:
1. **Input Validation** - Validates question length and format
2. **Authentication** - Verifies user JWT token
3. **Star Deduction** - Confirms user has ≥1 star (deducted upon successful reading)
4. **Agent Pipeline** (Enqueued to Upstash Redis):
   - **Agent 1: Question Filter** - Validates question appropriateness
   - **Agent 2: Question Analyzer** - Extracts context and emotions
   - **Agent 3: Reading Agent** - Generates tarot interpretation
5. **Response Formatting** - Structures JSON response
6. **Database Logging** - Stores reading history

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "job_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "status": "processing",
    "original_question": "อยากรู้เรื่องการงานในปีหน้า",
    "question_analysis": {
      "category": "การงาน",
      "emotion": "ระดับสนใจ",
      "timeframe": "ปีหน้า",
      "themes": ["อาชีพ", "โอกาส", "ความสำเร็จ"]
    },
    "tarot_reading": {
      "header": "สวัสดีค่ะ มาดูดวงเรื่องการงานของคุณในปีหน้านะคะ",
      "cards_reading": [
        {
          "id": 19,
          "name": "The Sun",
          "displayName": "ไพ่พระอาทิตย์",
          "imageUrl": "the_sun.png",
          "position": 1,
          "shortMeaning": "ความสำเร็จ ความสุข แสงสว่าง",
          "keywords": "ความสำเร็จ, ความสุข, แสงสว่าง, โอกาสดี"
        },
        {
          "id": 1,
          "name": "The Magician",
          "displayName": "ไพ่นักมายากล",
          "imageUrl": "the_magician.png",
          "position": 2,
          "shortMeaning": "ความสามารถ ความชำนาญ การสร้างสรรค์",
          "keywords": "ความสามารถ, ทักษะ, การกระทำ, ความชำนาญ"
        }
      ],
      "reading": "ดูแล้วเห็นว่าปีหน้านี้คุณมีโอกาสดีมากในด้านการงานค่ะ ไพ่พระอาทิตย์บ่งชี้ว่าคุณจะต้องพบกับแสงสว่างและความสำเร็จในด้านอาชีพของคุณ ไพ่นักมายากลแสดงว่าคุณมีทักษะและความสามารถที่พอเพียงในการทำให้เป้าหมายของคุณเป็นจริง",
      "suggestions": [
        "ให้ความสำคัญกับการพัฒนาทักษะของคุณเพิ่มเติม เพราะสิ่งนี้จะช่วยให้คุณประสบความสำเร็จ",
        "อย่ากลัวที่จะเปลี่ยนแปลงและลองสิ่งใหม่ๆ โอกาสจะมาจากสิ่งที่คุณไม่ได้คาดหวัง",
        "รักษาสมดุลระหว่างการทำงานและการพักผ่อน นี่จะช่วยให้คุณอยู่ในสภาวะที่ดีที่สุด"
      ],
      "next_questions": [
        "ความท้าทายหลักของฉันในการงานคืออะไร",
        "ฉันควรตัดสินใจอะไรเกี่ยวกับการเปลี่ยนงาน",
        "โอกาสการรึ่นสูงสำหรับฉันคือเมื่อไร"
      ],
      "final": [
        "สรุปว่าปีหน้า คุณมีโอกาสได้ดีมากในด้านการงานค่ะ และความสำเร็จนั้นอยู่ในมือของคุณ"
      ],
      "end": "ขอให้โชคดีกับการงานของคุณในปีหน้าค่ะ และหวังว่าคำทำนายนี้จะให้แรงใจและความมั่นใจแก่คุณ",
      "notice": "โปรดใช้วิจารณญาณในการรับคำทำนาย คำทำนายเหล่านี้เป็นเพียงการให้คำแนะนำและมุมมองเท่านั้น ไม่ได้มีจุดประสงค์เพื่อให้คำปรึกษาด้านการเงิน การกฎหมาย หรือการแพทย์"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

**Response** (202 Accepted - Async Processing):
```json
{
  "success": true,
  "data": {
    "job_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "status": "queued",
    "message": "Your tarot reading is being processed",
    "estimated_wait_seconds": 8,
    "polling_url": "/api/v1/tarot/reading/3fa85f64-5717-4562-b3fc-2c963f66afa6"
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

**Error Responses**:
- `400 Bad Request` - Question validation failed
- `401 Unauthorized` - Invalid/missing JWT token
- `402 Payment Required` - Insufficient stars
- `422 Unprocessable Entity` - Question rejected by filter agent
- `429 Too Many Requests` - Rate limit exceeded
- `502 Bad Gateway` - LLM API failure
- `504 Gateway Timeout` - LLM request timeout
- `500 Internal Server Error` - Server error

**Rate Limit**: 100 requests/minute per user

**Example cURL**:
```bash
curl -X POST http://localhost:3000/api/v1/tarot/read \
  -H "Authorization: Bearer eyJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "question": "อยากรู้เรื่องความรักในปีหน้า"
  }'
```

---

#### GET /api/v1/tarot/reading/:job_id
**Get Tarot Reading Status**

Polls the status of a tarot reading job (for async processing).

**Authentication**: Required (JWT Bearer Token)

**Path Parameters**:
- `job_id` (required, UUID): Job ID returned from POST /api/v1/tarot/read

**Response** (200 OK - Completed):
```json
{
  "success": true,
  "data": {
    "job_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "status": "completed",
    "original_question": "อยากรู้เรื่องการงานในปีหน้า",
    "question_analysis": { /* ... */ },
    "tarot_reading": { /* ... */ }
  },
  "timestamp": "2025-11-20T10:35:00Z",
  "request_id": "req_123456790"
}
```

**Response** (202 Accepted - Still Processing):
```json
{
  "success": true,
  "data": {
    "job_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "status": "processing",
    "message": "Your tarot reading is still being processed",
    "estimated_wait_seconds": 3
  },
  "timestamp": "2025-11-20T10:33:00Z",
  "request_id": "req_123456791"
}
```

**Error Responses**:
- `404 Not Found` - Job ID not found
- `401 Unauthorized` - Invalid/missing JWT token
- `500 Internal Server Error` - Server error

---

#### GET /api/v1/tarot/history
**Get Reading History**

Retrieves user's tarot reading history with pagination.

**Authentication**: Required (JWT Bearer Token)

**Query Parameters**:
- `page` (optional, integer): Page number (default: 1)
- `limit` (optional, integer): Items per page (default: 20, max: 100)
- `sort` (optional, string): Sort order - `newest` (default) or `oldest`

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "readings": [
      {
        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "question": "อยากรู้เรื่องความรักในปีหน้า",
        "card_count": 3,
        "cards_summary": [
          { "name": "The Lovers", "displayName": "ไพ่คู่รัก" },
          { "name": "The Sun", "displayName": "ไพ่พระอาทิตย์" },
          { "name": "The Star", "displayName": "ไพ่ดาว" }
        ],
        "created_at": "2025-11-19T10:30:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 45,
      "total_pages": 3
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

### User & Wallet Endpoints

#### GET /api/v1/user/profile
**Get User Profile**

Retrieves current user's profile information.

**Authentication**: Required (JWT Bearer Token)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      "email": "user@example.com",
      "name": "John Doe",
      "picture_url": "https://avatar.example.com/john.jpg",
      "tier": "silver",
      "total_invites": 8,
      "created_at": "2025-11-01T00:00:00Z",
      "last_login_at": "2025-11-20T10:30:00Z"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### GET /api/v1/user/wallet
**Get User Wallet**

Retrieves user's star and coin balances.

**Authentication**: Required (JWT Bearer Token)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "wallet": {
      "star_balance": 45,
      "coin_balance": 250,
      "total_spent": 10500,
      "total_earned_from_invites": 145,
      "currency": "THB"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### GET /api/v1/user/wallet/transactions
**Get Wallet Transaction History**

Retrieves user's transaction history with pagination.

**Authentication**: Required (JWT Bearer Token)

**Query Parameters**:
- `page` (optional, integer): Page number (default: 1)
- `limit` (optional, integer): Items per page (default: 20, max: 100)
- `type` (optional, string): Filter by transaction type
  - `purchase` - Star purchase
  - `invite_reward` - Invite reward
  - `exchange` - Coin-to-star exchange
  - `reading` - Star deduction for reading
  - `refund` - Refund

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "type": "purchase",
        "amount": 15,
        "currency": "STAR",
        "balance_after": 45,
        "description": "Purchased 15 stars + 2 bonus stars",
        "created_at": "2025-11-19T15:30:00Z"
      },
      {
        "id": "4gb96g75-6828-5673-c4gd-3d074g77bga7",
        "type": "invite_reward",
        "amount": 5,
        "currency": "COIN",
        "balance_after": 250,
        "description": "Invite reward from referral",
        "created_at": "2025-11-18T12:00:00Z"
      },
      {
        "id": "5hc07h86-7939-6784-d5he-4e185h88chi8",
        "type": "reading",
        "amount": -1,
        "currency": "STAR",
        "balance_after": 44,
        "description": "Star deducted for tarot reading",
        "created_at": "2025-11-17T10:30:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 120,
      "total_pages": 6
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### POST /api/v1/user/exchange-coins
**Exchange Coins for Stars**

Convert accumulated coins into stars.

**Authentication**: Required (JWT Bearer Token)

**Request Body**:
```json
{
  "coins_amount": 10
}
```

**Parameters**:
- `coins_amount` (required, integer): Number of coins to exchange
  - Minimum 10 coins
  - Maximum available coin balance
  - Must be divisible by current exchange rate

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "exchange": {
      "coins_spent": 10,
      "stars_gained": 1,
      "transaction_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      "new_coin_balance": 240,
      "new_star_balance": 46,
      "exchange_rate": "10 coins = 1 star"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

**Error Responses**:
- `400 Bad Request` - Invalid amount or insufficient coins
- `422 Unprocessable Entity` - Amount not divisible by exchange rate

---

### Payment Endpoints

#### GET /api/v1/payment/packages
**Get Available Star Packages**

Retrieves all available star purchase packages.

**Authentication**: Optional

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "packages": [
      {
        "package_type": "starter",
        "stars": 5,
        "price": 5000,
        "price_display": "50.00 THB",
        "bonus_stars": 0,
        "description": "เริ่มต้น 5 คำถาม ทดลองใช้งาน"
      },
      {
        "package_type": "basic",
        "stars": 15,
        "price": 13000,
        "price_display": "130.00 THB",
        "bonus_stars": 2,
        "description": "คุ้มค่า 15 คำถาม ประหยัด 20%"
      },
      {
        "package_type": "premium",
        "stars": 35,
        "price": 28000,
        "price_display": "280.00 THB",
        "bonus_stars": 5,
        "description": "มหาศาล 35 คำถาม ประหยัด 30%"
      }
    ],
    "exchange_config": {
      "coin_to_star_rate": 10,
      "description": "10 coins = 1 star"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### POST /api/v1/payment/create-intent
**Create Stripe Payment Intent**

Creates a Stripe PaymentIntent for star purchase.

**Authentication**: Required (JWT Bearer Token)

**Request Body**:
```json
{
  "package": "basic",
  "payment_method_id": "pm_1234567890abcdef"
}
```

**Parameters**:
- `package` (required, string): Package type - `starter`, `basic`, or `premium`
- `payment_method_id` (optional, string): Stripe payment method ID for saved card

**Response** (201 Created):
```json
{
  "success": true,
  "data": {
    "payment_intent": {
      "client_secret": "pi_1234567890_secret_abcdef",
      "payment_intent_id": "pi_1234567890",
      "amount": 13000,
      "amount_display": "130.00 THB",
      "currency": "thb",
      "status": "requires_payment_method"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### POST /api/v1/payment/confirm
**Confirm Stripe Payment**

Confirms a payment intent (called after Stripe Elements processes payment on frontend).

**Authentication**: Required (JWT Bearer Token)

**Request Body**:
```json
{
  "payment_intent_id": "pi_1234567890"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "payment": {
      "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      "status": "succeeded",
      "amount": 13000,
      "amount_display": "130.00 THB",
      "stars_purchased": 15,
      "bonus_stars": 2,
      "total_stars": 17,
      "transaction_id": "txn_1234567890",
      "receipt_url": "https://receipts.stripe.com/..."
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### POST /api/v1/payment/webhook
**Stripe Webhook Handler**

Handles Stripe webhook events for payment confirmation.

**Authentication**: Stripe Signature Verification

**Webhook Events Handled**:
- `payment_intent.succeeded` - Payment completed successfully
- `payment_intent.payment_failed` - Payment failed
- `payment_intent.canceled` - Payment canceled

**Response** (204 No Content):
Successful webhook processing returns empty response.

---

#### GET /api/v1/payment/history
**Get Payment History**

Retrieves user's payment transaction history.

**Authentication**: Required (JWT Bearer Token)

**Query Parameters**:
- `page` (optional, integer): Page number (default: 1)
- `limit` (optional, integer): Items per page (default: 20, max: 100)
- `status` (optional, string): Filter by status - `pending`, `succeeded`, `failed`, `refunded`

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "payments": [
      {
        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "status": "succeeded",
        "amount": 13000,
        "amount_display": "130.00 THB",
        "package": "basic",
        "stars_purchased": 15,
        "bonus_stars": 2,
        "stripe_charge_id": "ch_1234567890",
        "created_at": "2025-11-19T15:30:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 5,
      "total_pages": 1
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

### Invitation System Endpoints

#### GET /api/v1/user/invite-code
**Get User's Invite Code**

Retrieves user's unique invitation code and referral link.

**Authentication**: Required (JWT Bearer Token)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "invite": {
      "code": "A1B2C3D4",
      "referral_link": "https://mimivibe.com/join?code=A1B2C3D4",
      "total_invites": 8,
      "tier": "silver",
      "coins_earned": 145
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### POST /api/v1/user/invite-join
**Join via Invite Code**

Accept an invitation and receive reward.

**Authentication**: Required (JWT Bearer Token)

**Request Body**:
```json
{
  "invite_code": "A1B2C3D4"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "join": {
      "inviter_name": "John Doe",
      "coins_earned": 5,
      "new_coin_balance": 15,
      "message": "ยินดีต้อนรับสู่ MimiVibe! คุณได้รับเหรียญ 5 เหรียญจากการใช้รหัส A1B2C3D4"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

**Error Responses**:
- `400 Bad Request` - Invalid invite code
- `404 Not Found` - Invite code not found
- `422 Unprocessable Entity` - Already redeemed or expired

---

#### GET /api/v1/user/invites
**Get Invite Statistics**

Retrieves detailed invite statistics and referral information.

**Authentication**: Required (JWT Bearer Token)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "invites": {
      "total_invites": 8,
      "accepted_invites": 7,
      "pending_invites": 1,
      "tier": "silver",
      "tier_progress": {
        "current": "silver",
        "next": "gold",
        "invites_for_next": 8,
        "invites_needed": 1
      },
      "earnings": {
        "total_coins_from_invites": 145,
        "average_per_invite": 20.71,
        "recent_rewards": [
          {
            "invitee_name": "Jane Smith",
            "coins_earned": 5,
            "date": "2025-11-18T14:00:00Z"
          }
        ]
      },
      "referral_link": "https://mimivibe.com/join?code=A1B2C3D4"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

### Admin Endpoints

#### GET /api/v1/admin/exchange-config
**Get Current Exchange Configuration**

Retrieves the current coin-to-star exchange rate.

**Authentication**: Required (Admin JWT)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "exchange_config": {
      "coin_to_star": 10,
      "reward_range": {
        "min_percentage": 30,
        "max_percentage": 50
      },
      "last_updated": "2025-11-20T08:00:00Z",
      "updated_by": "admin@mimivibe.com"
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### PUT /api/v1/admin/exchange-config
**Update Exchange Configuration**

Updates the coin-to-star exchange rate (admin only).

**Authentication**: Required (Admin JWT)

**Request Body**:
```json
{
  "coin_to_star": 12,
  "reward_range": {
    "min_percentage": 30,
    "max_percentage": 50
  }
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "exchange_config": {
      "coin_to_star": 12,
      "reward_range": {
        "min_percentage": 30,
        "max_percentage": 50
      },
      "effective_immediately": true
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### GET /api/v1/admin/analytics/overview
**Get Analytics Overview**

Retrieves high-level analytics and metrics.

**Authentication**: Required (Admin JWT)

**Query Parameters**:
- `period` (optional, string): Time period - `today`, `week`, `month` (default: `month`)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "analytics": {
      "users": {
        "total": 5432,
        "active_today": 1200,
        "new_today": 45,
        "new_this_period": 520
      },
      "readings": {
        "total": 25680,
        "today": 1850,
        "this_period": 15420,
        "average_per_user": 4.73
      },
      "revenue": {
        "total_usd": 12450.75,
        "total_thb": 435000,
        "today_thb": 25000,
        "this_period_thb": 180000,
        "average_per_transaction": 450
      },
      "invites": {
        "total": 8900,
        "successful": 7050,
        "success_rate": 79.2,
        "viral_coefficient": 1.45
      }
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### GET /api/v1/admin/jobs
**Get Job Queue Status**

Retrieves status of jobs in Upstash Redis queue.

**Authentication**: Required (Admin JWT)

**Query Parameters**:
- `status` (optional, string): Filter by status - `queued`, `processing`, `completed`, `failed`
- `limit` (optional, integer): Number of jobs to return (default: 20)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "queue": {
      "total_jobs": 150,
      "queued": 45,
      "processing": 8,
      "completed_today": 892,
      "failed_today": 12,
      "dlq_count": 3,
      "average_processing_time_ms": 4250,
      "p95_processing_time_ms": 6800,
      "jobs": [
        {
          "job_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
          "status": "processing",
          "attempts": 1,
          "started_at": "2025-11-20T10:28:00Z",
          "estimated_completion": "2025-11-20T10:33:00Z"
        }
      ]
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### GET /api/v1/admin/dlq
**Get Dead-Letter Queue**

Retrieves jobs that have failed permanently.

**Authentication**: Required (Admin JWT)

**Query Parameters**:
- `page` (optional, integer): Page number (default: 1)
- `limit` (optional, integer): Items per page (default: 20)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "dlq_jobs": [
      {
        "job_id": "5hc07h86-7939-6784-d5he-4e185h88chi8",
        "type": "tarot_reading",
        "user_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "attempts": 5,
        "max_attempts": 5,
        "last_error": "Gemini API rate limit exceeded",
        "created_at": "2025-11-19T14:30:00Z",
        "failed_at": "2025-11-20T08:15:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 3,
      "total_pages": 1
    }
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

#### GET /api/v1/health
**Health Check**

Simple health check endpoint for monitoring.

**Authentication**: Optional

**Response** (200 OK):
```json
{
  "status": "healthy",
  "timestamp": "2025-11-20T10:30:00Z",
  "services": {
    "database": "connected",
    "stripe": "connected",
    "gemini": "connected",
    "redis": "connected"
  }
}
```

---

## Data Models

### User
```json
{
  "id": "UUID",
  "external_id": "string (from auth provider)",
  "external_provider": "string (clerk|auth0|firebase|custom)",
  "email": "string",
  "name": "string (optional)",
  "picture_url": "string (optional)",
  "star_balance": "integer (≥0)",
  "coin_balance": "integer (≥0)",
  "invite_code": "string (8 chars)",
  "invited_by": "UUID (optional)",
  "total_invites": "integer",
  "tier": "enum (bronze|silver|gold)",
  "created_at": "ISO8601 datetime",
  "last_login_at": "ISO8601 datetime",
  "updated_at": "ISO8601 datetime"
}
```

### StarPackage
```json
{
  "package_type": "enum (starter|basic|premium)",
  "stars": "integer (5|15|35)",
  "price": "integer (satang - 100 = 1 THB)",
  "description": "string (Thai)",
  "bonus_stars": "integer (0|2|5)"
}
```

### TarotReading
```json
{
  "id": "UUID",
  "user_id": "UUID",
  "question": "string",
  "card_count": "integer (3|5)",
  "cards": [
    {
      "id": "integer (0-77)",
      "name": "string (English)",
      "displayName": "string (Thai)",
      "imageUrl": "string",
      "position": "integer (1-5)",
      "shortMeaning": "string (Thai)",
      "keywords": "string (comma-separated Thai)"
    }
  ],
  "reading": {
    "header": "string (Thai)",
    "cards_reading": "array (see above)",
    "reading": "string (Thai paragraph)",
    "suggestions": "array[string] (Thai)",
    "next_questions": "array[string] (Thai)",
    "final": "array[string] (Thai)",
    "end": "string (Thai)",
    "notice": "string (Thai)"
  },
  "processing_time_ms": "integer",
  "created_at": "ISO8601 datetime"
}
```

### Payment
```json
{
  "id": "UUID",
  "user_id": "UUID",
  "stripe_payment_intent_id": "string",
  "stripe_charge_id": "string (optional)",
  "amount": "integer (satang)",
  "currency": "string (THB)",
  "status": "enum (pending|succeeded|failed|refunded)",
  "package_type": "enum (starter|basic|premium)",
  "stars_purchased": "integer",
  "stars_bonus": "integer",
  "created_at": "ISO8601 datetime",
  "updated_at": "ISO8601 datetime"
}
```

### WalletTransaction
```json
{
  "id": "UUID",
  "user_id": "UUID",
  "transaction_type": "enum (purchase|invite_reward|exchange|reading|refund)",
  "amount": "integer",
  "currency": "enum (STAR|COIN)",
  "balance_after": "integer",
  "metadata": "object (optional)",
  "related_payment_id": "UUID (optional)",
  "created_at": "ISO8601 datetime"
}
```

---

## Rate Limiting

### Rate Limit Rules

| Endpoint Group | Limit | Window |
|---|---|---|
| Tarot Reading | 100 | per minute per user |
| Payment | 10 | per minute per user |
| General API | 300 | per minute per user |
| Admin | 50 | per minute per admin |

### Rate Limit Headers

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1700472600
```

### Rate Limit Exceeded Response

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "API rate limit exceeded",
    "details": "Maximum 100 requests per minute allowed",
    "retry_after": 35
  },
  "timestamp": "2025-11-20T10:30:00Z",
  "request_id": "req_123456789"
}
```

---

## Examples

### JavaScript/TypeScript Example

```typescript
import axios from 'axios';

class MimiVibeAPI {
  private baseURL = 'https://api.mimivibe.com';
  private token: string;

  constructor(token: string) {
    this.token = token;
  }

  private async request(method: string, path: string, data?: any) {
    try {
      const response = await axios({
        method,
        url: `${this.baseURL}${path}`,
        data,
        headers: {
          'Authorization': `Bearer ${this.token}`,
          'Content-Type': 'application/json',
        },
      });

      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        throw error.response?.data || error.message;
      }
      throw error;
    }
  }

  // Get tarot reading
  async getTarotReading(question: string) {
    return this.request('POST', '/api/v1/tarot/read', { question });
  }

  // Poll reading status
  async getTarotReadingStatus(jobId: string) {
    return this.request('GET', `/api/v1/tarot/reading/${jobId}`);
  }

  // Get user profile
  async getProfile() {
    return this.request('GET', '/api/v1/user/profile');
  }

  // Get wallet balance
  async getWallet() {
    return this.request('GET', '/api/v1/user/wallet');
  }

  // Exchange coins
  async exchangeCoins(coinsAmount: number) {
    return this.request('POST', '/api/v1/user/exchange-coins', {
      coins_amount: coinsAmount,
    });
  }

  // Create payment intent
  async createPaymentIntent(packageType: string) {
    return this.request('POST', '/api/v1/payment/create-intent', {
      package: packageType,
    });
  }

  // Get invite code
  async getInviteCode() {
    return this.request('GET', '/api/v1/user/invite-code');
  }

  // Join via invite
  async joinViaInvite(inviteCode: string) {
    return this.request('POST', '/api/v1/user/invite-join', {
      invite_code: inviteCode,
    });
  }
}

// Usage example
const api = new MimiVibeAPI('eyJhbGc...');

// Get tarot reading
const response = await api.getTarotReading('อยากรู้เรื่องความรักในปีหน้า');
console.log(response.data.tarot_reading);

// Exchange coins
const exchangeResult = await api.exchangeCoins(10);
console.log(`Got ${exchangeResult.data.exchange.stars_gained} stars`);

// Get invite code for sharing
const inviteData = await api.getInviteCode();
console.log(`Share this: ${inviteData.data.invite.referral_link}`);
```

### cURL Examples

#### Get Tarot Reading
```bash
curl -X POST https://api.mimivibe.com/api/v1/tarot/read \
  -H "Authorization: Bearer eyJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "question": "อยากรู้เรื่องการงานในปีหน้า"
  }'
```

#### Poll Reading Status
```bash
curl -X GET https://api.mimivibe.com/api/v1/tarot/reading/3fa85f64-5717-4562-b3fc-2c963f66afa6 \
  -H "Authorization: Bearer eyJhbGc..."
```

#### Get User Wallet
```bash
curl -X GET https://api.mimivibe.com/api/v1/user/wallet \
  -H "Authorization: Bearer eyJhbGc..."
```

#### Create Payment Intent
```bash
curl -X POST https://api.mimivibe.com/api/v1/payment/create-intent \
  -H "Authorization: Bearer eyJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "package": "basic"
  }'
```

#### Exchange Coins
```bash
curl -X POST https://api.mimivibe.com/api/v1/user/exchange-coins \
  -H "Authorization: Bearer eyJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "coins_amount": 10
  }'
```

#### Get Invite Code
```bash
curl -X GET https://api.mimivibe.com/api/v1/user/invite-code \
  -H "Authorization: Bearer eyJhbGc..."
```

---

## Appendix: Implementation Checklist

### Phase 1 Implementation
- [ ] Axum web server with basic routing
- [ ] Multi-provider JWT authentication (Clerk, Auth0, Firebase)
- [ ] LangChain Rust integration with Gemini API
- [ ] Agent pipeline (Question Filter, Analyzer, Reader)
- [ ] Database schema with SQLx migrations
- [ ] User profile endpoints
- [ ] Tarot reading endpoint (sync)
- [ ] Basic error handling and logging

### Phase 2 Implementation
- [ ] Upstash Redis Streams queue
- [ ] Background worker for async processing
- [ ] Reading history persistence
- [ ] Stripe payment integration
- [ ] Wallet system (stars and coins)
- [ ] Transaction history tracking
- [ ] Admin analytics endpoints
- [ ] Sentry error tracking and monitoring

### Phase 3 Implementation
- [ ] Invite system with referrals
- [ ] Dynamic exchange rate configuration
- [ ] Advanced analytics and reporting
- [ ] PromptPay integration
- [ ] Mobile app endpoints
- [ ] WebSocket support for real-time updates
- [ ] Advanced fraud detection

---

**Document Version**: 1.0
**Last Updated**: November 20, 2025
**API Version**: v1
