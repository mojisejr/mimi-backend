//! Payment Repository Tests
//!
//! Tests for payment repository operations as per Task #48.
//! Following TDD approach - write tests FIRST (Red Phase).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Mock Models - These represent expected database entities
// ============================================================================

/// Mock Payment entity based on database schema
#[derive(Debug, Clone, PartialEq)]
struct Payment {
    id: Uuid,
    user_id: Uuid,
    stripe_payment_intent_id: Option<String>,
    stripe_charge_id: Option<String>,
    stripe_metadata: Option<serde_json::Value>,
    amount: i64,
    currency: String,
    status: PaymentStatus,
    package_type: String,
    stars_purchased: i32,
    stars_bonus: i32,
    stars_total: i32,
    error_code: Option<String>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
enum PaymentStatus {
    Pending,
    Succeeded,
    Failed,
    Refunded,
}

/// Mock User entity for payment operations
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: Uuid,
    external_id: String,
    external_provider: String,
    email: String,
    star_balance: i64,
    coin_balance: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Mock Wallet Transaction entity for payment refunds
#[derive(Debug, Clone, PartialEq)]
struct WalletTransaction {
    id: Uuid,
    user_id: Uuid,
    transaction_type: TransactionType,
    amount: i32,
    currency: Currency,
    balance_after: i64,
    metadata: Option<serde_json::Value>,
    related_payment_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
enum TransactionType {
    Purchase,
    InviteReward,
    Exchange,
    Reading,
    Refund,
}

#[derive(Debug, Clone, PartialEq)]
enum Currency {
    Star,
    Coin,
}

/// Data for creating a new payment.
#[derive(Debug, Clone)]
struct CreatePayment {
    /// User ID who is making the payment
    pub user_id: Uuid,
    /// Stripe payment intent ID (optional for pending payments)
    pub stripe_payment_intent_id: Option<String>,
    /// Payment amount in smallest currency unit
    pub amount: i64,
    /// Currency code
    pub currency: String,
    /// Package type
    pub package_type: String,
    /// Number of stars purchased
    pub stars_purchased: i32,
    /// Bonus stars awarded
    pub stars_bonus: i32,
}

/// Mock Repository Interface (what we expect to implement)
struct PaymentRepository {
    pool: PgPool,
}

impl PaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Expected implementation - process refund with transaction safety
    pub async fn process_refund(
        &self,
        payment_id: &Uuid,
        refund_amount: i64,
        reason: Option<String>,
    ) -> Result<WalletTransaction, sqlx::Error> {
        // This will be implemented in the actual repository
        // For now, this test should fail because the method doesn't exist
        todo!("Implementation needed: process refund with SELECT FOR UPDATE and wallet adjustment")
    }

    /// Expected implementation - update payment status
    pub async fn update_status(
        &self,
        payment_id: &Uuid,
        new_status: PaymentStatus,
    ) -> Result<Payment, sqlx::Error> {
        // This will be implemented in the actual repository
        // For now, this test should fail because the method doesn't exist
        todo!("Implementation needed: update payment status with validation")
    }

    /// Expected implementation - find payment by ID
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<Payment>, sqlx::Error> {
        // This will be implemented in the actual repository
        // For now, this test should fail because the method doesn't exist
        todo!("Implementation needed: find payment by ID")
    }

    /// Expected implementation - find payments by user
    pub async fn find_by_user_id(
        &self,
        user_id: &Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Payment>, sqlx::Error> {
        // This will be implemented in the actual repository
        // For now, this test should fail because the method doesn't exist
        todo!("Implementation needed: find payments by user with pagination")
    }

    /// Expected implementation - create payment record
    pub async fn create_payment(
        &self,
        payment_data: CreatePayment,
    ) -> Result<Payment, sqlx::Error> {
        // This will be implemented in the actual repository
        // For now, this test should fail because the method doesn't exist
        todo!("Implementation needed: create payment record")
    }

    /// Expected implementation - update stripe charge information
    pub async fn update_stripe_charge(
        &self,
        payment_id: &Uuid,
        stripe_charge_id: String,
        stripe_metadata: serde_json::Value,
    ) -> Result<Payment, sqlx::Error> {
        // This will be implemented in the actual repository
        // For now, this test should fail because the method doesn't exist
        todo!("Implementation needed: update stripe charge information")
    }

    /// Expected implementation - get payment statistics
    pub async fn get_payment_stats(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<PaymentStats, sqlx::Error> {
        // This will be implemented in the actual repository
        // For now, this test should fail because the method doesn't exist
        todo!("Implementation needed: get payment statistics")
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PaymentStats {
    total_payments: i64,
    total_amount: i64,
    successful_payments: i64,
    failed_payments: i64,
    refunded_payments: i64,
    total_stars_purchased: i64,
    total_stars_bonus: i64,
}

// ============================================================================
// Unit Tests - Payment Refund Workflow (Task #42 requirement)
// ============================================================================

#[tokio::test]
async fn test_payment_refund_workflow() {
    // Setup mock database connection (will fail because repository doesn't exist)
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let refund_amount = 5000; // 50.00 THB in satang
    let reason = Some("Customer request".to_string());

    // Test the complete refund workflow
    let result = repository
        .process_refund(&payment_id, refund_amount, reason)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "process_refund should fail until implemented"
    );

    // Once implemented, the refund workflow should:
    // 1. SELECT payment WHERE id = payment_id FOR UPDATE (to prevent concurrent refunds)
    // 2. Validate payment status is 'succeeded' and hasn't been refunded yet
    // 3. Calculate refund amount (partial or full)
    // 4. SELECT user.star_balance FROM users WHERE id = payment.user_id FOR UPDATE
    // 5. Validate user has sufficient stars balance (stars_purchased + stars_bonus from payment)
    // 6. INSERT INTO wallet_transactions with negative amount (refund)
    // 7. UPDATE users SET star_balance = star_balance - refund_stars
    // 8. UPDATE payments SET status = 'refunded'
    // 9. COMMIT transaction
}

#[tokio::test]
async fn test_payment_partial_refund() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let refund_amount = 2500; // Partial refund: 25.00 THB

    // Test partial refund
    let result = repository
        .process_refund(
            &payment_id,
            refund_amount,
            Some("Partial refund".to_string()),
        )
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "process_refund should fail until implemented"
    );

    // Once implemented, should handle partial refunds correctly
    // Only deduct proportional amount of stars
    // Keep payment record with refund metadata
}

#[tokio::test]
async fn test_payment_full_refund() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let payment_amount = 10000; // 100.00 THB

    // Test full refund
    let result = repository
        .process_refund(&payment_id, payment_amount, Some("Full refund".to_string()))
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "process_refund should fail until implemented"
    );

    // Once implemented, should refund entire amount
    // Deduct all stars (purchased + bonus) from user balance
    // Mark payment as fully refunded
}

#[tokio::test]
async fn test_payment_refund_insufficient_stars() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let refund_amount = 5000;

    // Test refund when user doesn't have enough stars
    // This should fail with appropriate error
    let result = repository
        .process_refund(&payment_id, refund_amount, None)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "process_refund should fail until implemented"
    );

    // Once implemented, should:
    // - Check user's current star balance
    // - Fail if balance is insufficient for refund
    // - Return specific error about insufficient stars
}

#[tokio::test]
async fn test_payment_refund_already_refunded() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let refund_amount = 3000;

    // Test refund on already refunded payment
    let result1 = repository
        .process_refund(&payment_id, refund_amount, None)
        .await;
    let result2 = repository
        .process_refund(&payment_id, refund_amount, None)
        .await;

    // Expected: Both should fail because method doesn't exist yet
    assert!(
        result1.is_err(),
        "First process_refund should fail until implemented"
    );
    assert!(
        result2.is_err(),
        "Second process_refund should fail until implemented"
    );

    // Once implemented:
    // - First call should succeed if payment hasn't been refunded
    // - Second call should fail because payment is already refunded
    // - Should check payment status before processing refund
}

#[tokio::test]
async fn test_payment_refund_failed_payment() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let refund_amount = 2000;

    // Test refund on failed payment (should not be allowed)
    let result = repository
        .process_refund(&payment_id, refund_amount, None)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "process_refund should fail until implemented"
    );

    // Once implemented, should:
    // - Check that payment status is 'succeeded'
    // - Fail if payment status is 'pending', 'failed', or 'refunded'
    // - Return specific error about invalid payment status
}

// ============================================================================
// Unit Tests - Payment Status Updates (Task #48 requirement)
// ============================================================================

#[tokio::test]
async fn test_payment_status_updates() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();

    // Test status transitions
    let pending_to_succeeded = repository
        .update_status(&payment_id, PaymentStatus::Succeeded)
        .await;
    let succeeded_to_refunded = repository
        .update_status(&payment_id, PaymentStatus::Refunded)
        .await;

    // Expected: Both should fail because method doesn't exist yet
    assert!(
        pending_to_succeeded.is_err(),
        "update_status should fail until implemented"
    );
    assert!(
        succeeded_to_refunded.is_err(),
        "update_status should fail until implemented"
    );

    // Once implemented, should:
    // - Validate status transitions
    // - Update payment status with timestamp
    // - Return updated payment record
}

#[tokio::test]
async fn test_payment_status_invalid_transitions() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();

    // Test invalid status transitions
    // refunded -> succeeded should not be allowed
    let result = repository
        .update_status(&payment_id, PaymentStatus::Succeeded)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "update_status should fail until implemented"
    );

    // Once implemented, should:
    // - Validate that status transitions are business-logic compliant
    // - Failed/Refunded payments should not transition to Succeeded
    // - Only Pending payments can transition to Succeeded/Failed
    // - Only Succeeded payments can transition to Refunded
}

#[tokio::test]
async fn test_payment_status_pending_to_succeeded() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();

    // Test valid transition: pending -> succeeded
    let result = repository
        .update_status(&payment_id, PaymentStatus::Succeeded)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "update_status should fail until implemented"
    );

    // Once implemented, should:
    // - Allow pending -> succeeded transition
    // - Update status and timestamps
    // - Potentially trigger wallet credit operations
}

#[tokio::test]
async fn test_payment_status_pending_to_failed() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();

    // Test valid transition: pending -> failed
    let result = repository
        .update_status(&payment_id, PaymentStatus::Failed)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "update_status should fail until implemented"
    );

    // Once implemented, should:
    // - Allow pending -> failed transition
    // - Store error details if provided
    // - Not credit any stars to user wallet
}

// ============================================================================
// Unit Tests - Payment CRUD Operations
// ============================================================================

#[tokio::test]
async fn test_payment_find_by_id() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();

    // Test finding payment by ID
    let result = repository.find_by_id(&payment_id).await;

    // Expected: This should fail because method doesn't exist yet
    assert!(result.is_err(), "find_by_id should fail until implemented");

    // Once implemented, should:
    // - Return Some(Payment) if found
    // - Return None if not found
    // - Include all payment details including calculated stars_total
}

#[tokio::test]
async fn test_payment_find_by_user_id() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let user_id = Uuid::new_v4();

    // Test finding payments by user
    let result = repository
        .find_by_user_id(&user_id, Some(10), Some(0))
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "find_by_user_id should fail until implemented"
    );

    // Once implemented, should:
    // - Return paginated list of user's payments
    // - Order by created_at DESC (most recent first)
    // - Apply limit and offset for pagination
}

#[tokio::test]
async fn test_payment_create() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let user_id = Uuid::new_v4();
    let stripe_payment_intent_id = Some("pi_test_1234567890".to_string());
    let amount = 10000; // 100.00 THB
    let currency = "THB".to_string();
    let package_type = "premium".to_string();
    let stars_purchased = 1000;
    let stars_bonus = 200;

    // Test creating a new payment
    let payment_data = CreatePayment {
        user_id,
        stripe_payment_intent_id,
        amount,
        currency,
        package_type,
        stars_purchased,
        stars_bonus,
    };

    let result = repository.create_payment(payment_data).await;

    // Expected: This should succeed now that create_payment is implemented
    assert!(result.is_ok(), "create_payment should work now");

    // Once implemented, should:
    // - Create new payment record
    // - Set initial status to 'pending'
    // - Calculate stars_total automatically
    // - Validate amount > 0 and stars_total > 0 constraints
}

#[tokio::test]
async fn test_payment_update_stripe_charge() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let stripe_charge_id = "ch_test_1234567890".to_string();
    let stripe_metadata = serde_json::json!({
        "payment_method": "card",
        "card_brand": "visa",
        "card_last4": "4242"
    });

    // Test updating Stripe charge information
    let result = repository
        .update_stripe_charge(&payment_id, stripe_charge_id.clone(), stripe_metadata)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "update_stripe_charge should fail until implemented"
    );

    // Once implemented, should:
    // - Update stripe_charge_id and stripe_metadata
    // - Keep existing payment information intact
    // - Return updated payment record
}

// ============================================================================
// Unit Tests - Payment Statistics
// ============================================================================

#[tokio::test]
async fn test_payment_stats_user_specific() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let user_id = Uuid::new_v4();

    // Test getting payment statistics for specific user
    let result = repository
        .get_payment_stats(Some(user_id), None, None)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "get_payment_stats should fail until implemented"
    );

    // Once implemented, should:
    // - Return statistics filtered by user_id
    // - Include total payments, amounts, success rates
    // - Calculate stars purchased and bonus totals
}

#[tokio::test]
async fn test_payment_stats_date_range() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let start_date = Utc::now() - chrono::Duration::days(30);
    let end_date = Utc::now();

    // Test getting payment statistics for date range
    let result = repository
        .get_payment_stats(None, Some(start_date), Some(end_date))
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "get_payment_stats should fail until implemented"
    );

    // Once implemented, should:
    // - Return statistics filtered by date range
    // - Include performance metrics for the period
    // - Help with business analytics
}

// ============================================================================
// Unit Tests - Payment Constraints and Validation
// ============================================================================

#[tokio::test]
async fn test_payment_amount_constraints() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let user_id = Uuid::new_v4();

    // Test creating payment with invalid amount
    let payment_data = CreatePayment {
        user_id,
        stripe_payment_intent_id: None,
        amount: -1000, // Negative amount should fail
        currency: "THB".to_string(),
        package_type: "basic".to_string(),
        stars_purchased: 100,
        stars_bonus: 0,
    };

    let result = repository.create_payment(payment_data).await;

    // Expected: This should fail due to validation
    assert!(
        result.is_err(),
        "create_payment should fail with negative amount"
    );

    // Once implemented, should:
    // - Validate amount > 0 constraint
    // - Return specific error for invalid amounts
    // - Prevent database constraint violation
}

#[tokio::test]
async fn test_payment_stars_constraints() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let user_id = Uuid::new_v4();

    // Test creating payment with zero stars
    let payment_data = CreatePayment {
        user_id,
        stripe_payment_intent_id: None,
        amount: 5000, // Valid amount
        currency: "THB".to_string(),
        package_type: "starter".to_string(),
        stars_purchased: 0, // Zero stars should fail
        stars_bonus: 0,
    };

    let result = repository.create_payment(payment_data).await;

    // Expected: This should fail due to validation
    assert!(
        result.is_err(),
        "create_payment should fail with zero stars"
    );

    // Once implemented, should:
    // - Validate stars_purchased > 0 constraint
    // - Ensure stars_total > 0 (stars_purchased + stars_bonus)
    // - Return specific error for invalid star amounts
}

#[tokio::test]
async fn test_payment_stripe_intent_uniqueness() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let user_id = Uuid::new_v4();
    let stripe_intent_id = "pi_unique_1234567890".to_string();

    // Test creating two payments with same Stripe intent ID
    let result1 = repository
        .create(
            user_id,
            Some(stripe_intent_id.clone()),
            5000,
            "THB".to_string(),
            "basic".to_string(),
            500,
            50,
        )
        .await;

    let result2 = repository
        .create(
            Uuid::new_v4(),
            Some(stripe_intent_id), // Same intent ID
            3000,
            "THB".to_string(),
            "starter".to_string(),
            300,
            0,
        )
        .await;

    // Expected: Both should fail because method doesn't exist yet
    assert!(
        result1.is_err(),
        "First create should fail until implemented"
    );
    assert!(
        result2.is_err(),
        "Second create should fail until implemented"
    );

    // Once implemented:
    // - First should succeed if intent_id is unique
    // - Second should fail due to UNIQUE constraint on stripe_payment_intent_id
    // - Return database constraint violation error
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_payment_refund_zero_amount() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();

    // Test refund with zero amount (should fail)
    let result = repository.process_refund(&payment_id, 0, None).await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "process_refund should fail until implemented"
    );

    // Once implemented, should:
    // - Validate refund amount > 0
    // - Return specific error for zero refund amount
}

#[tokio::test]
async fn test_payment_refund_exceeds_original() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let payment_id = Uuid::new_v4();
    let original_amount = 10000;
    let refund_amount = 15000; // Exceeds original amount

    // Test refund amount exceeding original payment
    let result = repository
        .process_refund(&payment_id, refund_amount, None)
        .await;

    // Expected: This should fail because method doesn't exist yet
    assert!(
        result.is_err(),
        "process_refund should fail until implemented"
    );

    // Once implemented, should:
    // - Validate refund amount doesn't exceed original payment
    // - Return specific error for excessive refund amount
}

#[tokio::test]
async fn test_payment_find_nonexistent() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let non_existent_id = Uuid::new_v4();

    // Test finding non-existent payment
    let result = repository.find_by_id(&non_existent_id).await;

    // Expected: This should fail because method doesn't exist yet
    assert!(result.is_err(), "find_by_id should fail until implemented");

    // Once implemented, should return Ok(None) for non-existent payments
}

#[tokio::test]
async fn test_payment_pagination_edge_cases() {
    let pool = PgPool::connect(&"postgresql://localhost:5432/test")
        .await
        .unwrap();
    let repository = PaymentRepository::new(pool);

    let user_id = Uuid::new_v4();

    // Test pagination edge cases
    let result1 = repository.find_by_user_id(&user_id, Some(0), Some(0)).await; // Zero limit
    let result2 = repository
        .find_by_user_id(&user_id, Some(-1), Some(0))
        .await; // Negative limit
    let result3 = repository
        .find_by_user_id(&user_id, Some(1000), Some(-10))
        .await; // Negative offset

    // Expected: All should fail because method doesn't exist yet
    assert!(
        result1.is_err(),
        "find_by_user_id with zero limit should fail until implemented"
    );
    assert!(
        result2.is_err(),
        "find_by_user_id with negative limit should fail until implemented"
    );
    assert!(
        result3.is_err(),
        "find_by_user_id with negative offset should fail until implemented"
    );

    // Once implemented, should:
    // - Handle edge cases gracefully
    // - Validate limit and offset parameters
    // - Return appropriate errors for invalid parameters
}
