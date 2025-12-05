//! Tests for worker queue connection
//!
//! Test-First Development: Write failing tests before implementation
//! These tests verify that the worker can connect to the TarotQueue

mod setup; // AUTO-LOAD .env via tests/setup.rs

use std::process::Command;

/// Test that TarotQueue can be created from environment
///
/// This test verifies that TarotQueue::from_env() works correctly
/// with the available environment variables
#[tokio::test]
async fn test_queue_connection_creation() {
    setup::setup();

    // Test queue creation from environment
    let result = mimivibe_backend::queue::TarotQueue::from_env().await;

    // Should succeed with proper environment setup
    assert!(
        result.is_ok(),
        "Failed to create TarotQueue: {:?}",
        result.err()
    );

    let queue = result.unwrap();

    // Verify queue has database connectivity by checking pending count
    let pending_result = queue.get_pending_count().await;
    assert!(
        pending_result.is_ok(),
        "Failed to get pending count: {:?}",
        pending_result.err()
    );

    let pending = pending_result.unwrap();
    // Should be able to retrieve queue statistics
    assert!(pending.total >= 0, "Pending count should be non-negative");
}

/// Test that worker starts with queue connection
///
/// This test verifies that the worker binary can start successfully
/// when TarotQueue connection is established
#[test]
fn test_worker_with_queue_connection() {
    setup::setup();

    // Test that worker can start with queue connection
    let result = Command::new("cargo")
        .args(&["run", "--bin", "worker"])
        .env("RUST_LOG", "info")
        .output()
        .expect("Failed to run worker binary");

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    // Worker should execute successfully (may terminate after demo)
    assert!(
        result.status.success() || result.status.code() == Some(124), // 124 = timeout
        "Worker should execute successfully or by timeout. Status: {:?}\nstdout:\n{}\nstderr:\n{}",
        result.status,
        stdout,
        stderr
    );

    // Should NOT contain queue connection errors
    assert!(
        !stderr.contains("Failed to create queue")
            && !stderr.contains("Queue connection failed")
            && !stderr.contains("Cannot start a runtime")
            && !stderr.contains("panicked"),
        "Should not contain queue or runtime errors:\n{}",
        stderr
    );

    // Should show queue connection success message
    assert!(
        stdout.contains("Connected to tarot job queue")
            || stdout.contains("Database connection successful")
            || stdout.contains("Using in-memory queue"),
        "Should show queue connection success:\n{}",
        stdout
    );
}

/// Test that queue can access database through worker setup
///
/// This test verifies that the queue can properly access the database
/// using the same setup as the worker
#[tokio::test]
async fn test_queue_database_access() {
    setup::setup();

    // Setup database connection like worker does
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for testing");

    let db_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database for testing");

    // Verify database is accessible
    let result = sqlx::query("SELECT 1 as test").fetch_one(&db_pool).await;

    assert!(
        result.is_ok(),
        "Database should be accessible: {:?}",
        result.err()
    );

    // Test queue creation with explicit database pool
    let queue = mimivibe_backend::queue::TarotQueue::from_env()
        .await
        .expect("Failed to create TarotQueue with explicit database");

    // Verify queue can perform database operations
    let pending_count = queue
        .get_pending_count()
        .await
        .expect("Failed to get pending count from queue");

    // Should return valid statistics
    assert!(
        pending_count.total >= 0,
        "Queue should return valid pending count"
    );
    assert!(
        pending_count.queued >= 0,
        "Queue should return valid queued count"
    );
    assert!(
        pending_count.processing >= 0,
        "Queue should return valid processing count"
    );
}

/// Test worker termination with queue connection
///
/// Verifies that worker can terminate gracefully when queue connection is established
#[test]
fn test_worker_terminates_gracefully_with_queue() {
    setup::setup();

    // Test that worker can start and complete its demo successfully with queue connection
    let result = Command::new("cargo")
        .args(&["run", "--bin", "worker"])
        .env("RUST_LOG", "info")
        .output()
        .expect("Failed to run worker binary");

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    // Worker should execute successfully
    assert!(
        result.status.success(),
        "Worker should execute successfully with queue connection. Status: {:?}\nstdout:\n{}\nstderr:\n{}",
        result.status,
        stdout,
        stderr
    );

    // Should show queue connection was established
    assert!(
        stdout.contains("Connected to tarot job queue"),
        "Should show successful queue connection:\n{}",
        stdout
    );

    // Should complete gracefully
    assert!(
        stdout.contains("Worker demonstration completed"),
        "Should show worker completed gracefully:\n{}",
        stdout
    );
}

/// Test that queue connection doesn't interfere with existing worker functionality
///
/// This test ensures that adding queue connection doesn't break
/// the existing AI pipeline functionality
#[test]
fn test_queue_preserves_worker_functionality() {
    setup::setup();

    // Test that worker can still perform AI operations with queue
    let result = Command::new("cargo")
        .args(&["run", "--bin", "worker"])
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to run worker binary");

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    // Should not contain AI pipeline errors
    assert!(
        !stderr.contains("AI Pipeline error")
            && !stderr.contains("Gemini API error")
            && !stderr.contains("panicked"),
        "Should not contain AI pipeline errors:\n{}",
        stderr
    );

    // Should show both queue and AI pipeline initialization
    let has_queue = stdout.contains("Connected to tarot job queue")
        || stdout.contains("Database connection successful")
        || stdout.contains("Using in-memory queue");

    let has_ai = stdout.contains("AI pipeline functionality")
        || stdout.contains("Successfully processed example job")
        || stdout.contains("Creating TarotWorker");

    assert!(
        has_queue && has_ai,
        "Should show both queue and AI pipeline functionality:\nQueue: {}\nAI: {}\nFull stdout:\n{}",
        has_queue,
        has_ai,
        stdout
    );
}
