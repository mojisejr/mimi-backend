//! In-memory queue implementation for testing
//!
//! This module provides an in-memory queue implementation that can be used
//! for unit tests and integration tests without requiring external dependencies
//! like Redis or Upstash.
//!
//! # Features
//!
//! - Thread-safe concurrent access using Arc/Mutex
//! - FIFO job ordering
//! - Job acknowledgement and negative acknowledgement
//! - No external dependencies (only stdlib + tokio)
//!
//! # Thread Safety
//!
//! The InMemoryQueue uses Arc<Mutex<>> internally to ensure thread-safe
//! concurrent access from multiple workers/consumers.

use crate::queue::{JobPayload, Queue, QueuedJob};
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};

/// Internal state for the in-memory queue
///
/// This struct holds the actual queue data and is protected by a Mutex
/// for thread-safe concurrent access.
#[derive(Default)]
struct QueueState {
    /// Pending jobs waiting to be processed (FIFO queue)
    pending: VecDeque<JobPayload>,

    /// Jobs currently being processed by consumers
    /// Maps job_id -> (consumer_id, job_data)
    processing: HashMap<String, (String, QueuedJob)>,

    /// Track job attempts for retry logic
    /// Maps job_id -> attempt_count
    attempts: HashMap<String, u32>,
}

/// In-memory queue implementation
///
/// Provides a thread-safe, in-memory job queue suitable for testing.
/// Uses VecDeque for FIFO ordering and HashMap for tracking processing jobs.
///
/// # Example
///
/// ```ignore
/// use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;
/// use mimivibe_backend::queue::Queue;
///
/// let queue = InMemoryQueue::new();
/// let job_id = queue.enqueue(payload).await?;
/// let job = queue.dequeue("worker-1").await?;
/// queue.ack(&job_id, "worker-1").await?;
/// ```
pub struct InMemoryQueue {
    /// Shared state protected by Mutex for thread safety
    state: Arc<Mutex<QueueState>>,
}

impl InMemoryQueue {
    /// Create a new in-memory queue
    ///
    /// # Returns
    ///
    /// A new InMemoryQueue instance with empty queues
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(QueueState::default())),
        }
    }
}

impl Default for InMemoryQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Queue for InMemoryQueue {
    /// Enqueue a job to the in-memory queue
    ///
    /// Adds the job to the pending queue in FIFO order.
    /// Thread-safe for concurrent access.
    async fn enqueue(&self, payload: JobPayload) -> Result<String, Box<dyn Error>> {
        let job_id = payload.job_id.clone();

        let mut state = self.state.lock().map_err(|e| -> Box<dyn Error> {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire lock: {}",
                e
            )))
        })?;

        // Add to pending queue
        state.pending.push_back(payload);

        // Initialize attempt counter
        state.attempts.entry(job_id.clone()).or_insert(0);

        Ok(job_id)
    }

    /// Dequeue a job from the in-memory queue
    ///
    /// Retrieves the next job from the pending queue and marks it as
    /// being processed by the specified consumer.
    async fn dequeue(&self, consumer_id: &str) -> Result<Option<QueuedJob>, Box<dyn Error>> {
        let mut state = self.state.lock().map_err(|e| -> Box<dyn Error> {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire lock: {}",
                e
            )))
        })?;

        // Get next job from pending queue
        if let Some(payload) = state.pending.pop_front() {
            let job_id = payload.job_id.clone();

            // Increment attempt counter
            let attempts = state
                .attempts
                .entry(job_id.clone())
                .and_modify(|a| *a += 1)
                .or_insert(1);

            // Create QueuedJob
            let queued_job = QueuedJob {
                job_id: job_id.clone(),
                payload,
                attempts: *attempts,
                claimed_at: chrono::Utc::now(),
            };

            // Move to processing map
            state.processing.insert(
                job_id.clone(),
                (consumer_id.to_string(), queued_job.clone()),
            );

            Ok(Some(queued_job))
        } else {
            // No jobs available
            Ok(None)
        }
    }

    /// Acknowledge successful job completion
    ///
    /// Removes the job from the processing map, marking it as successfully completed.
    async fn ack(&self, job_id: &str, consumer_id: &str) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().map_err(|e| -> Box<dyn Error> {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire lock: {}",
                e
            )))
        })?;

        // Remove from processing map
        if let Some((processing_consumer, _)) = state.processing.get(job_id) {
            // Verify the consumer ID matches (optional security check)
            if processing_consumer == consumer_id {
                state.processing.remove(job_id);
                state.attempts.remove(job_id);
            } else {
                // Different consumer trying to ACK - this is suspicious but we'll allow it
                // In production, you might want to return an error here
                state.processing.remove(job_id);
                state.attempts.remove(job_id);
            }
        }
        // If job not found in processing, it's a duplicate ACK - just ignore it

        Ok(())
    }

    /// Negative acknowledgement - job failed
    ///
    /// Requeues the job for retry by moving it back to the pending queue.
    async fn nack(
        &self,
        job_id: &str,
        consumer_id: &str,
        reason: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().map_err(|e| -> Box<dyn Error> {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire lock: {}",
                e
            )))
        })?;

        // Find job in processing map
        if let Some((processing_consumer, queued_job)) = state.processing.remove(job_id) {
            // Verify consumer ID (optional)
            if processing_consumer != consumer_id {
                // Log mismatch but proceed anyway
                eprintln!(
                    "NACK consumer mismatch: expected {}, got {}",
                    processing_consumer, consumer_id
                );
            }

            // Log the reason if provided
            if let Some(ref r) = reason {
                eprintln!("Job {} NACK'd by {}: {}", job_id, consumer_id, r);
            }

            // Requeue the job by adding it back to pending
            // Put it at the front for immediate retry (could also go to back)
            state.pending.push_front(queued_job.payload);
        }

        Ok(())
    }

    /// Get the current queue length
    ///
    /// Returns the number of jobs in the pending queue (not including processing jobs).
    async fn get_queue_length(&self) -> Result<usize, Box<dyn Error>> {
        let state = self.state.lock().map_err(|e| -> Box<dyn Error> {
            Box::new(std::io::Error::other(format!(
                "Failed to acquire lock: {}",
                e
            )))
        })?;

        Ok(state.pending.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

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
            metadata: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn test_new_queue_is_empty() {
        let queue = InMemoryQueue::new();
        let length = queue.get_queue_length().await.unwrap();
        assert_eq!(length, 0);
    }

    #[tokio::test]
    async fn test_enqueue_increases_length() {
        let queue = InMemoryQueue::new();
        let payload = create_test_payload("test");

        queue.enqueue(payload).await.unwrap();

        let length = queue.get_queue_length().await.unwrap();
        assert_eq!(length, 1);
    }

    #[tokio::test]
    async fn test_dequeue_decreases_length() {
        let queue = InMemoryQueue::new();
        let payload = create_test_payload("test");

        queue.enqueue(payload).await.unwrap();
        queue.dequeue("worker-1").await.unwrap();

        let length = queue.get_queue_length().await.unwrap();
        assert_eq!(length, 0);
    }

    #[tokio::test]
    async fn test_dequeue_empty_returns_none() {
        let queue = InMemoryQueue::new();
        let result = queue.dequeue("worker-1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_ack_removes_job() {
        let queue = InMemoryQueue::new();
        let payload = create_test_payload("test");
        let job_id = payload.job_id.clone();

        queue.enqueue(payload).await.unwrap();
        let _job = queue.dequeue("worker-1").await.unwrap().unwrap();
        queue.ack(&job_id, "worker-1").await.unwrap();

        // Job should be removed
        let length = queue.get_queue_length().await.unwrap();
        assert_eq!(length, 0);
    }

    #[tokio::test]
    async fn test_nack_requeues_job() {
        let queue = InMemoryQueue::new();
        let payload = create_test_payload("test");
        let job_id = payload.job_id.clone();

        queue.enqueue(payload).await.unwrap();
        let _job = queue.dequeue("worker-1").await.unwrap().unwrap();
        queue
            .nack(&job_id, "worker-1", Some("test".to_string()))
            .await
            .unwrap();

        // Job should be back in queue
        let length = queue.get_queue_length().await.unwrap();
        assert_eq!(length, 1);
    }
}
