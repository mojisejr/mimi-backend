//! Integration tests for InMemoryQueue implementation
//!
//! Tests the InMemoryQueue backend implementation for unit/integration testing.
//! These tests verify:
//! - Thread-safe concurrent operations (enqueue, dequeue, ack)
//! - Multiple worker scenarios
//! - Edge cases (duplicate ack, worker crashes, concurrent access)
//!
//! Test-First Development: These tests are written BEFORE implementation
//! RED phase: Tests will fail initially until InMemoryQueue is implemented

use chrono::Utc;
use mimivibe_backend::queue::{JobPayload, Queue};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

// Helper function to create test job payload
fn create_test_payload(question: &str) -> JobPayload {
    JobPayload {
        job_id: Uuid::new_v4().to_string(),
        user_id: Uuid::new_v4(),
        question: question.to_string(),
        card_count: 3,
        schema_version: "1".to_string(),
        prompt_version: "v2025-11-20-a".to_string(),
        dedupe_key: None,
        trace_id: None,
        created_at: Utc::now(),
        metadata: json!({
            "locale": "th",
            "source": "test"
        }),
    }
}

#[tokio::test]
async fn test_inmemory_queue_basic_enqueue_dequeue() {
    // Test basic enqueue and dequeue operations
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = InMemoryQueue::new();
    let payload = create_test_payload("ความรักของฉันจะเป็นอย่างไร");
    let job_id = payload.job_id.clone();

    // Enqueue job
    let enqueue_result = queue.enqueue(payload).await;
    assert!(enqueue_result.is_ok(), "Should enqueue successfully");
    assert_eq!(enqueue_result.unwrap(), job_id);

    // Check queue length
    let length = queue.get_queue_length().await.unwrap();
    assert_eq!(length, 1, "Queue should have 1 job");

    // Dequeue job
    let dequeue_result = queue.dequeue("worker-1").await;
    assert!(dequeue_result.is_ok(), "Should dequeue successfully");

    let queued_job = dequeue_result.unwrap();
    assert!(queued_job.is_some(), "Should return a job");

    let job = queued_job.unwrap();
    assert_eq!(job.job_id, job_id);
    assert_eq!(job.payload.question, "ความรักของฉันจะเป็นอย่างไร");

    // Queue should be empty after dequeue (job is now processing)
    let empty_result = queue.dequeue("worker-2").await;
    assert!(empty_result.is_ok());
    assert!(empty_result.unwrap().is_none(), "Queue should be empty");
}

#[tokio::test]
async fn test_inmemory_queue_ack_removes_job() {
    // Test that ACK properly removes job from processing
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = InMemoryQueue::new();
    let payload = create_test_payload("การงานของฉันจะดีขึ้นไหม");
    let _job_id = payload.job_id.clone();

    // Enqueue and dequeue
    queue.enqueue(payload).await.unwrap();
    let job = queue.dequeue("worker-1").await.unwrap().unwrap();

    // Acknowledge job
    let ack_result = queue.ack(&job.job_id, "worker-1").await;
    assert!(ack_result.is_ok(), "Should ACK successfully");

    // Queue should remain empty
    let length = queue.get_queue_length().await.unwrap();
    assert_eq!(length, 0, "Queue should be empty after ACK");
}

#[tokio::test]
async fn test_inmemory_queue_concurrent_enqueue() {
    // Test concurrent enqueue operations from multiple tasks
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = Arc::new(InMemoryQueue::new());
    let mut handles = vec![];

    // Spawn 10 tasks, each enqueueing 10 jobs
    for i in 0..10 {
        let queue_clone = Arc::clone(&queue);
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let payload = create_test_payload(&format!("Question {} from task {}", j, i));
                queue_clone.enqueue(payload).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Check that all 100 jobs are in the queue
    let length = queue.get_queue_length().await.unwrap();
    assert_eq!(length, 100, "Should have 100 jobs in queue");
}

#[tokio::test]
async fn test_inmemory_queue_concurrent_dequeue() {
    // Test concurrent dequeue operations from multiple workers
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = Arc::new(InMemoryQueue::new());

    // Enqueue 50 jobs
    for i in 0..50 {
        let payload = create_test_payload(&format!("Job {}", i));
        queue.enqueue(payload).await.unwrap();
    }

    let mut handles = vec![];
    let processed_jobs = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Spawn 5 workers, each trying to dequeue jobs
    for worker_id in 0..5 {
        let queue_clone = Arc::clone(&queue);
        let processed_clone = Arc::clone(&processed_jobs);

        let handle = tokio::spawn(async move {
            let consumer_id = format!("worker-{}", worker_id);
            loop {
                // Dequeue and immediately handle result to avoid Send issues
                let job_option = match queue_clone.dequeue(&consumer_id).await {
                    Ok(opt) => opt,
                    Err(_) => break,
                };

                match job_option {
                    Some(job) => {
                        let job_id = job.job_id.clone();

                        // Simulate processing
                        sleep(Duration::from_millis(10)).await;

                        // Acknowledge job
                        let _ = queue_clone.ack(&job_id, &consumer_id).await;

                        // Track processed job
                        let mut processed = processed_clone.lock().await;
                        processed.push(job_id);
                    }
                    None => break,
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all workers to finish
    for handle in handles {
        handle.await.unwrap();
    }

    // Check that all jobs were processed
    let processed = processed_jobs.lock().await;
    assert_eq!(processed.len(), 50, "All 50 jobs should be processed");

    // Queue should be empty
    let length = queue.get_queue_length().await.unwrap();
    assert_eq!(length, 0, "Queue should be empty after processing");
}

#[tokio::test]
async fn test_inmemory_queue_duplicate_ack() {
    // Edge case: Test duplicate ACK handling
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = InMemoryQueue::new();
    let payload = create_test_payload("Test duplicate ack");

    queue.enqueue(payload).await.unwrap();
    let job = queue.dequeue("worker-1").await.unwrap().unwrap();

    // First ACK should succeed
    let ack1 = queue.ack(&job.job_id, "worker-1").await;
    assert!(ack1.is_ok(), "First ACK should succeed");

    // Second ACK should handle gracefully (no panic)
    let ack2 = queue.ack(&job.job_id, "worker-1").await;
    assert!(ack2.is_ok(), "Duplicate ACK should not panic");
}

#[tokio::test]
async fn test_inmemory_queue_nack_requeues_job() {
    // Test that NACK requeues job for retry
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = InMemoryQueue::new();
    let payload = create_test_payload("Test NACK behavior");
    let _job_id = payload.job_id.clone();

    queue.enqueue(payload).await.unwrap();
    let job = queue.dequeue("worker-1").await.unwrap().unwrap();

    // NACK the job
    let nack_result = queue
        .nack(&job.job_id, "worker-1", Some("Worker crashed".to_string()))
        .await;
    assert!(nack_result.is_ok(), "NACK should succeed");

    // Job should be back in queue or available for dequeue
    let length = queue.get_queue_length().await.unwrap();
    assert!(length >= 1, "Job should be available after NACK");
}

#[tokio::test]
async fn test_inmemory_queue_worker_crash_simulation() {
    // Edge case: Simulate worker crash without ACK
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = Arc::new(InMemoryQueue::new());

    // Enqueue job
    let payload = create_test_payload("Test worker crash");
    let _job_id = payload.job_id.clone();
    queue.enqueue(payload).await.unwrap();

    // Worker 1 dequeues but crashes (no ACK/NACK)
    {
        let job = queue.dequeue("worker-1").await.unwrap().unwrap();
        assert!(!job.job_id.is_empty());
        // Worker crashes here (no ACK)
    }

    // Job should eventually be available for another worker
    // This tests timeout/requeue logic if implemented
    let length = queue.get_queue_length().await.unwrap();
    // Job is in processing state (not in pending queue)
    // This is expected behavior - the job is "lost" until timeout/requeue is implemented
    assert_eq!(
        length, 0,
        "Queue should be empty (job is in processing state)"
    );
}

#[tokio::test]
async fn test_inmemory_queue_multiple_consumers_same_job() {
    // Test that multiple consumers cannot get the same job
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = Arc::new(InMemoryQueue::new());

    // Enqueue single job
    let payload = create_test_payload("Single job test");
    queue.enqueue(payload).await.unwrap();

    let queue1 = Arc::clone(&queue);
    let queue2 = Arc::clone(&queue);

    // Two workers try to dequeue simultaneously
    let (result1, result2) = tokio::join!(
        async move { queue1.dequeue("worker-1").await },
        async move { queue2.dequeue("worker-2").await }
    );

    // Only one should get the job
    let got_job_1 = result1.unwrap().is_some();
    let got_job_2 = result2.unwrap().is_some();

    assert!(
        (got_job_1 && !got_job_2) || (!got_job_1 && got_job_2),
        "Only one worker should get the job"
    );
}

#[tokio::test]
async fn test_inmemory_queue_fifo_order() {
    // Test FIFO (First In First Out) ordering
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = InMemoryQueue::new();
    let mut job_ids = vec![];

    // Enqueue jobs in order
    for i in 0..5 {
        let payload = create_test_payload(&format!("Job {}", i));
        let job_id = payload.job_id.clone();
        job_ids.push(job_id);
        queue.enqueue(payload).await.unwrap();
    }

    // Dequeue and verify order
    for expected_id in job_ids {
        let job = queue.dequeue("worker-1").await.unwrap().unwrap();
        assert_eq!(
            job.job_id, expected_id,
            "Jobs should be dequeued in FIFO order"
        );
        queue.ack(&job.job_id, "worker-1").await.unwrap();
    }
}

#[tokio::test]
async fn test_inmemory_queue_empty_dequeue() {
    // Test dequeue from empty queue
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = InMemoryQueue::new();

    // Dequeue from empty queue
    let result = queue.dequeue("worker-1").await;
    assert!(result.is_ok(), "Dequeue from empty queue should not error");
    assert!(
        result.unwrap().is_none(),
        "Should return None for empty queue"
    );
}

#[tokio::test]
async fn test_inmemory_queue_length_tracking() {
    // Test accurate queue length tracking
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    let queue = InMemoryQueue::new();

    // Initial length should be 0
    assert_eq!(queue.get_queue_length().await.unwrap(), 0);

    // Enqueue 3 jobs
    for i in 0..3 {
        queue
            .enqueue(create_test_payload(&format!("Job {}", i)))
            .await
            .unwrap();
    }
    assert_eq!(queue.get_queue_length().await.unwrap(), 3);

    // Dequeue 1 job
    let job = queue.dequeue("worker-1").await.unwrap().unwrap();
    assert_eq!(queue.get_queue_length().await.unwrap(), 2);

    // ACK the job
    queue.ack(&job.job_id, "worker-1").await.unwrap();
    assert_eq!(queue.get_queue_length().await.unwrap(), 2);

    // Dequeue remaining
    queue.dequeue("worker-1").await.unwrap();
    queue.dequeue("worker-1").await.unwrap();
    assert_eq!(queue.get_queue_length().await.unwrap(), 0);
}
