//! Database Tests - Reading Jobs Schema (Simple)
//!
//! Test-Driven Development for reading_jobs database schema and operations.
//! These tests are written BEFORE implementation (Red Phase).

use mimivibe_backend::models::{CreateJobInput, JobStatus, ReadingJob};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Test database migration creates jobs table successfully
#[tokio::test]
async fn test_migration_creates_jobs_table() {
    let pool = get_test_pool().await;

    // Query the jobs table - should fail initially if table doesn't exist
    let result = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'jobs'"
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(count) => {
            // Table exists - count should be 1
            assert_eq!(
                count.unwrap_or(0),
                1,
                "Jobs table should exist after migration"
            );
        }
        Err(_) => {
            // Table doesn't exist - test will fail (expected in Red Phase)
            panic!("Jobs table should be created by migration");
        }
    }
}

/// Test job status enum works correctly
#[tokio::test]
async fn test_job_status_enum() {
    // Test status enum conversion
    assert_eq!(JobStatus::Queued.to_string(), "queued");
    assert_eq!(JobStatus::Processing.to_string(), "processing");
    assert_eq!(JobStatus::Succeeded.to_string(), "succeeded");
    assert_eq!(JobStatus::Failed.to_string(), "failed");
    assert_eq!(JobStatus::Dlq.to_string(), "dlq");
}

/// Test CRUD operations for jobs
#[tokio::test]
async fn test_job_crud_operations() {
    let pool = get_test_pool().await;

    // Create a job
    let payload = json!({
        "question": "What is my future?",
        "user_id": Uuid::new_v4()
    });

    let create_input = CreateJobInput {
        payload: payload.clone(),
        dedupe_key: None,
        max_attempts: Some(5),
    };

    // Insert
    let job_id = ReadingJob::create(&pool, create_input).await;

    match job_id {
        Ok(id) => {
            assert!(!id.to_string().is_empty(), "Job ID should be generated");

            // Read
            let job = ReadingJob::find_by_id(&pool, id).await;
            assert!(job.is_ok(), "Job retrieval should succeed");

            let job = job.unwrap().expect("Job should exist");
            assert_eq!(job.id, id);
            assert_eq!(job.job_type, "tarot_reading");
            assert_eq!(job.status, JobStatus::Queued);
            assert_eq!(job.attempts, 0);
            assert_eq!(job.max_attempts, 5);

            // Update status
            let update_result = ReadingJob::update_status(
                &pool,
                id,
                JobStatus::Processing,
                Some("worker-1".to_string()),
            )
            .await;
            assert!(update_result.is_ok(), "Job status update should succeed");

            // Verify update
            let updated_job = ReadingJob::find_by_id(&pool, id).await.unwrap().unwrap();
            assert_eq!(updated_job.status, JobStatus::Processing);
            assert_eq!(updated_job.worker_id, Some("worker-1".to_string()));

            // Update result (success)
            let result_payload = json!({
                "reading": "Your future looks bright",
                "cards": ["magician", "star", "world"]
            });

            let result_update = ReadingJob::update_result(&pool, id, result_payload).await;
            assert!(result_update.is_ok(), "Job result update should succeed");

            // Verify completion
            let completed_job = ReadingJob::find_by_id(&pool, id).await.unwrap().unwrap();
            assert_eq!(completed_job.status, JobStatus::Succeeded);
            assert!(completed_job.result.is_some());
            assert!(completed_job.completed_at.is_some());
            assert!(completed_job.is_finished());
        }
        Err(e) => {
            // Database operations fail if schema doesn't exist (expected in Red Phase)
            panic!("Job CRUD operations should work after migration: {}", e);
        }
    }
}

/// Test job status queries
#[tokio::test]
async fn test_job_status_queries() {
    let pool = get_test_pool().await;

    // Create a queued job
    let payload = json!({"test": "data"});
    let create_input = CreateJobInput {
        payload: payload.clone(),
        dedupe_key: None,
        max_attempts: Some(3),
    };

    match ReadingJob::create(&pool, create_input).await {
        Ok(job_id) => {
            // Query by status
            let queued_jobs = ReadingJob::find_by_status(&pool, JobStatus::Queued, 10).await;
            assert!(queued_jobs.is_ok(), "Status query should succeed");

            let jobs = queued_jobs.unwrap();
            assert!(!jobs.is_empty(), "Should find at least one queued job");

            // Find our job in the results
            let found_job = jobs.iter().find(|j| j.id == job_id);
            assert!(
                found_job.is_some(),
                "Created job should be found in status query"
            );
            assert_eq!(found_job.unwrap().status, JobStatus::Queued);
        }
        Err(_) => {
            // Expected to fail if schema doesn't exist (Red Phase)
            panic!("Job status queries should work after migration");
        }
    }
}

/// Test job retry logic
#[tokio::test]
async fn test_job_retry_logic() {
    let pool = get_test_pool().await;

    let payload = json!({"test": "retry"});
    let create_input = CreateJobInput {
        payload,
        dedupe_key: None,
        max_attempts: Some(3),
    };

    match ReadingJob::create(&pool, create_input).await {
        Ok(job_id) => {
            let job = ReadingJob::find_by_id(&pool, job_id)
                .await
                .unwrap()
                .unwrap();

            // Initially should be able to retry (even though it hasn't failed yet)
            assert!(!job.can_retry(), "Fresh job should not need retry");

            // Test finished state
            assert!(!job.is_finished(), "Fresh job should not be finished");
        }
        Err(_) => {
            // Expected to fail if schema doesn't exist (Red Phase)
            panic!("Job retry logic test should work after migration");
        }
    }
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
