//! JobStatus Enum Compatibility Tests
//!
//! Test suite for verifying JobStatus enum works correctly with database operations.
//! This ensures that worker can update job status in the database without type errors.

mod setup; // AUTO-LOAD .env via tests/setup.rs

use mimivibe_backend::models::job_types::JobStatus;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Test that JobStatus enum works with job_status database type
#[tokio::test]
async fn test_job_status_enum_compatibility() {
    setup::setup();

    // Get database connection from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Test that JobStatus enum works with job_status type
    let result = sqlx::query_scalar!("SELECT $1::job_status", JobStatus::Queued as JobStatus)
        .fetch_one(&pool)
        .await;

    assert!(
        result.is_ok(),
        "JobStatus should map to job_status enum type"
    );
}

/// Test worker status update using JobStatus enum values
#[tokio::test]
async fn test_worker_status_update_with_enum() {
    setup::setup();

    // Get database connection from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create a test job
    let job_id = Uuid::new_v4();

    // Insert a test job with queued status
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status)
        VALUES ($1, 'test_tarot_reading', $2, 'queued')
        "#,
        job_id,
        serde_json::json!({"question": "test"})
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test job");

    // Test updating job status using JobStatus enum
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING status as "status: JobStatus"
        "#,
        JobStatus::Processing as JobStatus,
        job_id
    )
    .fetch_one(&pool)
    .await;

    assert!(
        result.is_ok(),
        "Should be able to update job status using JobStatus enum"
    );

    // Verify the status was updated correctly
    let updated_status = result.unwrap().status;
    assert_eq!(updated_status, JobStatus::Processing);

    // Clean up
    sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test job");
}

/// Test complete job flow with status transitions
#[tokio::test]
async fn test_complete_job_flow_with_status_updates() {
    setup::setup();

    // Get database connection from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let job_id = Uuid::new_v4();

    // Step 1: Create job with Queued status
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, payload, status)
        VALUES ($1, 'test_tarot_reading', $2, $3)
        "#,
        job_id,
        serde_json::json!({"question": "test complete flow"}),
        JobStatus::Queued as JobStatus
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test job");

    // Step 2: Transition to Processing
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = $1, started_at = NOW(), updated_at = NOW()
        WHERE id = $2
        "#,
        JobStatus::Processing as JobStatus,
        job_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update to processing");

    // Step 3: Complete with Success
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = $1, result = $2, completed_at = NOW(), updated_at = NOW()
        WHERE id = $3
        "#,
        JobStatus::Succeeded as JobStatus,
        serde_json::json!({"reading": "test complete"}),
        job_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update to succeeded");

    // Verify final state
    let final_state = sqlx::query!(
        "SELECT status as \"status: JobStatus\", result FROM jobs WHERE id = $1",
        job_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch final state");

    assert_eq!(final_state.status, JobStatus::Succeeded);
    assert!(final_state.result.is_some());

    // Clean up
    sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up test job");
}

/// Test that invalid status mapping is rejected
#[tokio::test]
async fn test_invalid_status_mapping_rejected() {
    setup::setup();

    // Get database connection from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Test that trying to use invalid string with job_status enum fails
    let result = sqlx::query_scalar!("SELECT $1::job_status", "invalid_status" as &str)
        .fetch_one(&pool)
        .await;

    assert!(
        result.is_err(),
        "Invalid status string should be rejected by job_status enum"
    );
}

/// Test TarotQueue update_job_status method with enum
#[tokio::test]
async fn test_tarot_queue_status_update_with_enum() {
    setup::setup();

    // This test verifies that TarotQueue.update_job_status()
    // can accept JobStatus enum instead of &str

    let queue = mimivibe_backend::queue::TarotQueue::from_env()
        .await
        .expect("Failed to create TarotQueue");

    let job_id = Uuid::new_v4();

    // Create a test job first
    let submission_result = queue
        .submit_reading_request("test question for queue status update", None, Some(3))
        .await
        .expect("Failed to submit reading request");

    // Verify initial status
    assert_eq!(submission_result.status.to_string(), "queued");

    // This should work now that update_job_status accepts JobStatus enum
    let update_result = queue
        .update_job_status(
            submission_result.job_id,
            JobStatus::Processing, // Using enum instead of string
            None,
        )
        .await;

    assert!(
        update_result.is_ok(),
        "Status update with JobStatus enum should succeed"
    );
}

/// Test JobStatus serialization/deserialization
#[test]
fn test_job_status_serialization_consistency() {
    setup::setup();

    // Test that JobStatus enum serializes/deserializes correctly
    let statuses = vec![
        JobStatus::Succeeded,
        JobStatus::Failed,
        JobStatus::Processing,
        JobStatus::Queued,
        JobStatus::Dlq,
    ];

    for status in statuses {
        let json_str = serde_json::to_string(&status).expect("JobStatus should serialize");

        let deserialized: JobStatus =
            serde_json::from_str(&json_str).expect("JobStatus should deserialize");

        assert_eq!(
            status, deserialized,
            "JobStatus should serialize and deserialize correctly"
        );
    }
}

/// Test that JobStatus display implementation works
#[test]
fn test_job_status_display_format() {
    setup::setup();

    let test_cases = vec![
        (JobStatus::Queued, "queued"),
        (JobStatus::Processing, "processing"),
        (JobStatus::Succeeded, "succeeded"),
        (JobStatus::Failed, "failed"),
        (JobStatus::Dlq, "dlq"),
    ];

    for (status, expected_string) in test_cases {
        assert_eq!(
            status.to_string(),
            expected_string,
            "JobStatus::{} should display as '{}'",
            status,
            expected_string
        );
    }
}
