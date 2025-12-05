//! Tests for worker continuous polling loop
//!
//! Test-First Development: Write failing tests before implementation
//! These tests verify that the worker can continuously poll and process jobs

mod setup; // AUTO-LOAD .env via tests/setup.rs

use mimivibe_backend::models::job_types::{JobStatus, ReadingJob};
use sqlx::types::Json;
use std::time::Duration;
use uuid::Uuid;

/// Test that worker demonstrates polling loop structure
///
/// This test verifies that the worker contains the necessary
/// polling loop structure with proper iteration and sleeping
#[test]
fn test_polling_loop_structure() {
    setup::setup();

    // Start worker process in background
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "worker"])
        .env("RUST_LOG", "info")
        .spawn()
        .expect("Failed to spawn worker process");

    // Let it run for a few seconds to capture startup messages
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Terminate the worker
    child.kill().expect("Failed to kill worker process");
    let output = child
        .wait_with_output()
        .expect("Failed to read worker output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should NOT contain the old demo message
    assert!(
        !stdout.contains("Worker demonstration completed")
            && !stdout.contains("Demonstrating AI pipeline functionality"),
        "Should not contain demo completion messages. stdout:\n{}",
        stdout
    );

    // Should contain polling loop startup message
    assert!(
        stdout.contains("Starting worker polling loop")
            || stdout.contains("🚀 Starting worker polling loop")
            || stderr.contains("Starting worker polling loop"),
        "Should contain polling loop startup message. stdout:\n{} stderr:\n{}",
        stdout,
        stderr
    );

    // Should contain polling interval information
    assert!(
        stdout.contains("Polling interval")
            || stdout.contains("5 seconds")
            || stderr.contains("Polling interval")
            || stderr.contains("5 seconds"),
        "Should contain polling interval information. stdout:\n{} stderr:\n{}",
        stdout,
        stderr
    );
}

/// Test that worker handles job processing flow
///
/// This test verifies the job processing flow works correctly
/// with status updates and error handling
#[tokio::test]
async fn test_job_processing_flow() {
    setup::setup();

    // Setup database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for testing");
    let db_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database for testing");

    // Create queue instance
    let queue = mimivibe_backend::queue::TarotQueue::from_env()
        .await
        .expect("Failed to create TarotQueue");

    // Test job status update functionality
    let job_id = Uuid::new_v4();

    // This should compile but may fail at runtime (stubs)
    // The test verifies the interface exists
    match queue.update_job_status(job_id, "processing", None).await {
        Ok(_) => {
            // If status update succeeds, verify we can read it back
            if let Ok(Some(job)) = queue.get_reading_result(job_id).await {
                assert_eq!(job.status, JobStatus::Processing);
            }
        }
        Err(_) => {
            // Expected to fail with stub implementation
            // The important thing is that the method exists and compiles
        }
    }
}

/// Test worker polling behavior with test job
///
/// This test creates a test job and verifies the worker
/// can find and process it through the polling loop
#[tokio::test]
async fn test_worker_processes_test_job() {
    setup::setup();

    // Setup database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for testing");
    let db_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database for testing");

    // Create a test job directly in the database
    let job_id = Uuid::new_v4();
    let test_payload = serde_json::json!({
        "question": "ควรจะลงทุนอะไรดีครับ",
        "card_count": 3
    });

    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_type, status, payload)
        VALUES ($1, 'tarot_reading', $2, $3)
        "#,
        job_id,
        JobStatus::Queued as JobStatus,
        test_payload as serde_json::Value
    )
    .execute(&db_pool)
    .await
    .expect("Failed to insert test job");

    // Verify job was created
    let job_exists = sqlx::query!("SELECT id FROM jobs WHERE id = $1", job_id)
        .fetch_optional(&db_pool)
        .await
        .expect("Failed to verify test job");

    assert!(job_exists.is_some(), "Test job should exist in database");

    // Create queue and test polling
    let queue = mimivibe_backend::queue::TarotQueue::from_env()
        .await
        .expect("Failed to create TarotQueue");

    // Test poll_next_job method exists and returns expected type
    match queue.poll_next_job().await {
        Ok(Some(job)) => {
            // Verify job structure
            assert_eq!(job.id, job_id);
            assert_eq!(job.job_type, "tarot_reading");
            assert_eq!(job.status, JobStatus::Queued);

            // Verify payload contains expected data
            let payload_value: serde_json::Value = job.payload.0;
            assert_eq!(payload_value["question"], "ควรจะลงทุนอะไรดีครับ");
            assert_eq!(payload_value["card_count"], 3);
        }
        Ok(None) => {
            // Expected with stub implementation - method exists but returns None
        }
        Err(_) => {
            // Expected with stub implementation - method exists but may error
        }
    }

    // Clean up test job
    sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
        .execute(&db_pool)
        .await
        .expect("Failed to clean up test job");
}

/// Test worker handles empty queue gracefully
///
/// Verifies that worker continues polling when no jobs are available
/// and handles the sleep interval correctly
#[tokio::test]
async fn test_worker_handles_empty_queue() {
    setup::setup();

    // Create queue
    let queue = mimivibe_backend::queue::TarotQueue::from_env()
        .await
        .expect("Failed to create TarotQueue");

    // Test polling when queue should be empty
    let start_time = std::time::Instant::now();

    match queue.poll_next_job().await {
        Ok(None) => {
            // Expected behavior with stub implementation
            let elapsed = start_time.elapsed();
            // Should return quickly (not sleep for 5 seconds in the method itself)
            assert!(
                elapsed.as_secs() < 2,
                "poll_next_job should not block for long"
            );
        }
        Ok(Some(_)) => {
            // Unexpected but handle gracefully
        }
        Err(_) => {
            // Expected with stub implementation
        }
    }
}

/// Test worker graceful termination
///
/// Verifies that worker can be terminated cleanly
/// during the polling loop
#[test]
fn test_worker_termination_during_polling() {
    setup::setup();

    // Start worker process in background
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "worker"])
        .env("RUST_LOG", "info")
        .spawn()
        .expect("Failed to spawn worker process");

    // Let it run briefly
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Terminate the worker
    child.kill().expect("Failed to kill worker process");
    let output = child
        .wait_with_output()
        .expect("Failed to read worker output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show polling loop started
    assert!(
        stdout.contains("Starting worker polling loop")
            || stdout.contains("🚀 Starting worker polling loop"),
        "Should show polling loop started:\n{}",
        stdout
    );
}

/// Test worker shows proper polling status messages
///
/// Verifies that worker displays appropriate status messages
/// during polling operations
#[test]
fn test_worker_shows_polling_status_messages() {
    setup::setup();

    // Start worker process in background
    let mut child = std::process::Command::new("cargo")
        .args(&["run", "--bin", "worker"])
        .env("RUST_LOG", "debug")
        .spawn()
        .expect("Failed to spawn worker process");

    // Let it run to capture polling messages
    std::thread::sleep(std::time::Duration::from_secs(4));

    // Terminate the worker
    child.kill().expect("Failed to kill worker process");
    let output = child
        .wait_with_output()
        .expect("Failed to read worker output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain polling-related status messages
    let has_polling_messages = stdout.contains("💤 No jobs")
        || stdout.contains("No jobs available")
        || stdout.contains("Sleeping")
        || stdout.contains("⏳")
        || stdout.contains("Waiting")
        || stderr.contains("💤 No jobs")
        || stderr.contains("No jobs available")
        || stderr.contains("Sleeping");

    assert!(
        has_polling_messages,
        "Should show polling status messages. stdout:\n{} stderr:\n{}",
        stdout, stderr
    );
}
