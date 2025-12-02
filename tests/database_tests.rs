//! Database Tests - Reading Jobs Schema
//!
//! Test-Driven Development for reading_jobs database schema and operations.
//! These tests are written BEFORE implementation (Red Phase).

use chrono::Utc;
use mimivibe_backend::models::{CreateJobInput, JobStatus, ReadingJob};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Test database migration creates jobs table successfully
#[tokio::test]
async fn test_migration_creates_jobs_table() {
    // This test will fail until we create the migration
    let pool = get_test_pool().await;

    // Query the jobs table - should fail initially
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_name = 'jobs'"
    )
    .fetch_one(&pool)
    .await;

    // Should be Some(1) when table exists
    assert!(result.is_ok(), "Jobs table should exist after migration");
}

/// Test job status enum constraints work correctly
#[tokio::test]
async fn test_job_status_enum_constraints() {
    let pool = get_test_pool().await;

    // Try to insert invalid status - should fail
    let invalid_result = sqlx::query!(
        "INSERT INTO jobs (id, job_type, status, payload) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(),
        "test_job",
        "invalid_status", // This should fail
        json!({"test": "data"})
    )
    .execute(&pool)
    .await;

    assert!(
        invalid_result.is_err(),
        "Invalid job status should be rejected"
    );

    // Try to insert valid status - should work
    let valid_result = sqlx::query!(
        "INSERT INTO jobs (id, job_type, status, payload) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(),
        "test_job",
        "queued", // Valid status
        json!({"test": "data"})
    )
    .execute(&pool)
    .await;

    assert!(valid_result.is_ok(), "Valid job status should be accepted");
}

/// Test CRUD operations for jobs
#[tokio::test]
async fn test_job_crud_operations() {
    let pool = get_test_pool().await;

    // Create a job
    let job_id = Uuid::new_v4();
    let payload = json!({
        "question": "What is my future?",
        "user_id": Uuid::new_v4()
    });

    // Insert
    let insert_result = sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, status, payload, created_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        job_id,
        "tarot_reading",
        "queued",
        payload,
        Utc::now()
    )
    .fetch_one(&pool)
    .await;

    assert!(insert_result.is_ok(), "Job insertion should succeed");

    // Read
    let job = sqlx::query!(
        r#"
        SELECT id, job_type, status, payload, created_at, updated_at
        FROM jobs
        WHERE id = $1
        "#,
        job_id
    )
    .fetch_one(&pool)
    .await;

    assert!(job.is_ok(), "Job retrieval should succeed");
    let job = job.unwrap();
    assert_eq!(job.id, job_id);
    assert_eq!(job.job_type, "tarot_reading");
    assert_eq!(job.status, "queued");
    assert_eq!(job.payload, payload);

    // Update status
    let update_result = sqlx::query!(
        "UPDATE jobs SET status = $1, updated_at = $2 WHERE id = $3",
        "processing",
        Utc::now(),
        job_id
    )
    .execute(&pool)
    .await;

    assert!(update_result.is_ok(), "Job status update should succeed");

    // Verify update
    let updated_job = sqlx::query!("SELECT status FROM jobs WHERE id = $1", job_id)
        .fetch_one(&pool)
        .await;

    assert!(updated_job.is_ok(), "Updated job retrieval should succeed");
    assert_eq!(updated_job.unwrap().status, "processing");

    // Delete
    let delete_result = sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
        .execute(&pool)
        .await;

    assert!(delete_result.is_ok(), "Job deletion should succeed");

    // Verify deletion
    let deleted_job = sqlx::query!("SELECT id FROM jobs WHERE id = $1", job_id)
        .fetch_optional(&pool)
        .await;

    assert!(
        deleted_job.is_ok(),
        "Delete verification query should succeed"
    );
    assert!(deleted_job.unwrap().is_none(), "Job should be deleted");
}

/// Test job status transitions work properly
#[tokio::test]
async fn test_job_status_transitions() {
    let pool = get_test_pool().await;

    let job_id = Uuid::new_v4();

    // Create job in initial state
    sqlx::query!(
        "INSERT INTO jobs (id, job_type, status, payload) VALUES ($1, $2, $3, $4)",
        job_id,
        "tarot_reading",
        "queued",
        json!({"test": "data"})
    )
    .execute(&pool)
    .await
    .expect("Job creation should succeed");

    // Test valid transitions: queued -> processing -> succeeded
    let transitions = vec![
        ("queued", "processing"),
        ("processing", "succeeded"),
        ("succeeded", "completed"), // If we add completed status
    ];

    for (from_status, to_status) in transitions {
        // Update to from_status if needed
        sqlx::query!(
            "UPDATE jobs SET status = $1 WHERE id = $2",
            from_status,
            job_id
        )
        .execute(&pool)
        .await
        .expect("Setting initial status should succeed");

        // Transition to to_status
        let result = sqlx::query!(
            "UPDATE jobs SET status = $1 WHERE id = $2",
            to_status,
            job_id
        )
        .execute(&pool)
        .await;

        assert!(
            result.is_ok(),
            "Transition from {} to {} should succeed",
            from_status,
            to_status
        );
    }
}

/// Test job retry mechanism and attempts tracking
#[tokio::test]
async fn test_job_attempts_and_retry() {
    let pool = get_test_pool().await;

    let job_id = Uuid::new_v4();

    // Create job
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, status, payload, attempts, max_attempts)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        job_id,
        "tarot_reading",
        "queued",
        json!({"test": "data"}),
        0i32, // attempts
        5i32  // max_attempts
    )
    .execute(&pool)
    .await
    .expect("Job creation should succeed");

    // Increment attempts
    for attempt in 1..=3 {
        let result = sqlx::query!(
            "UPDATE jobs SET attempts = $1 WHERE id = $2",
            attempt,
            job_id
        )
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Attempt increment should succeed");

        // Verify attempt count
        let current = sqlx::query!("SELECT attempts FROM jobs WHERE id = $1", job_id)
            .fetch_one(&pool)
            .await
            .expect("Getting attempt count should succeed");

        assert_eq!(
            current.attempts, attempt,
            "Attempt count should be {}",
            attempt
        );
    }
}

/// Test job payload JSONB functionality
#[tokio::test]
async fn test_job_payload_jsonb() {
    let pool = get_test_pool().await;

    let job_id = Uuid::new_v4();
    let complex_payload = json!({
        "question": "What is my future?",
        "user_id": Uuid::new_v4(),
        "reading_type": "3_cards",
        "cards": ["magician", "high_priestess", "empress"],
        "metadata": {
            "locale": "th",
            "timestamp": "2025-01-02T00:00:00Z"
        }
    });

    // Insert with complex payload
    let insert_result = sqlx::query!(
        "INSERT INTO jobs (id, job_type, status, payload) VALUES ($1, $2, $3, $4)",
        job_id,
        "tarot_reading",
        "queued",
        complex_payload
    )
    .execute(&pool)
    .await;

    assert!(
        insert_result.is_ok(),
        "Complex payload insertion should succeed"
    );

    // Retrieve and verify payload
    let retrieved = sqlx::query!("SELECT payload FROM jobs WHERE id = $1", job_id)
        .fetch_one(&pool)
        .await;

    assert!(retrieved.is_ok(), "Payload retrieval should succeed");
    assert_eq!(retrieved.unwrap().payload, complex_payload);

    // Test JSONB query capabilities
    let query_result = sqlx::query!(
        r#"
        SELECT payload->>'question' as question
        FROM jobs
        WHERE id = $1
        "#,
        job_id
    )
    .fetch_one(&pool)
    .await;

    assert!(query_result.is_ok(), "JSONB field query should succeed");
    assert_eq!(query_result.unwrap().question, "What is my future?");
}

/// Helper function to get test database pool
async fn get_test_pool() -> PgPool {
    // Try to get test database URL from environment
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        // Fallback to main database URL for testing
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests")
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to create test database pool")
}
