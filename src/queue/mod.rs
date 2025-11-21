//! Queue abstraction layer
//!
//! Provides a standardized interface for job queues that can be backed by
//! different implementations (Redis, Upstash, In-Memory).

pub mod redis_dedupe;
pub mod types;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub use types::{JobMetadata, JobPayload, JobType};

/// Job status enumeration
///
/// Represents the lifecycle states of a job in the queue system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    /// Job has been queued and is waiting to be processed
    Queued,
    /// Job is currently being processed by a worker
    Processing,
    /// Job completed successfully
    Succeeded,
    /// Job failed during processing
    Failed,
    /// Job has been moved to the Dead Letter Queue
    #[serde(rename = "DLQ")]
    DLQ,
}

/// Represents a job that has been dequeued and is ready for processing
///
/// Contains the job payload along with metadata about processing attempts
/// and when the job was claimed by a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedJob {
    /// Unique identifier for this job
    pub job_id: String,
    /// The actual job data to be processed
    pub payload: JobPayload,
    /// Number of times this job has been attempted
    pub attempts: u32,
    /// Timestamp when this job was claimed by the current consumer
    pub claimed_at: chrono::DateTime<chrono::Utc>,
}

/// Queue trait defining the standard interface for all queue implementations
///
/// This trait provides the core operations needed for a job queue system:
/// - Enqueueing new jobs
/// - Dequeueing jobs for processing
/// - Acknowledging successful completion
/// - Negative acknowledgement for failures
/// - Monitoring queue depth
///
/// All queue implementations (RedisQueue, UpstashQueue, InMemoryQueue) must
/// implement this trait to ensure consistent behavior across backends.
#[async_trait]
pub trait Queue: Send + Sync {
    /// Enqueue a job to the queue
    ///
    /// Adds a new job to the queue for processing. Returns a unique job ID
    /// that can be used to track the job's progress.
    ///
    /// # Arguments
    ///
    /// * `payload` - The job data to be queued
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The unique job ID assigned to this job
    /// * `Err(Box<dyn Error>)` - If the enqueue operation failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// let job_id = queue.enqueue(job_payload).await?;
    /// println!("Enqueued job: {}", job_id);
    /// ```
    async fn enqueue(&self, payload: JobPayload) -> Result<String, Box<dyn Error>>;

    /// Dequeue a job from the queue
    ///
    /// Retrieves the next available job from the queue. This operation may
    /// block or timeout depending on the implementation. Returns `None` if
    /// no jobs are available.
    ///
    /// # Arguments
    ///
    /// * `consumer_id` - Unique identifier for the consumer/worker
    ///
    /// # Returns
    ///
    /// * `Ok(Some(QueuedJob))` - A job was successfully dequeued
    /// * `Ok(None)` - No jobs available in the queue
    /// * `Err(Box<dyn Error>)` - If the dequeue operation failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(job) = queue.dequeue("worker-1").await? {
    ///     // Process the job
    ///     process_job(job).await?;
    /// }
    /// ```
    async fn dequeue(&self, consumer_id: &str) -> Result<Option<QueuedJob>, Box<dyn Error>>;

    /// Acknowledge successful job completion
    ///
    /// Marks a job as successfully completed and removes it from the queue.
    /// This should be called after a job has been processed successfully.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The unique identifier of the job to acknowledge
    /// * `consumer_id` - The consumer that processed the job
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Job was successfully acknowledged
    /// * `Err(Box<dyn Error>)` - If the acknowledgement failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// queue.ack(&job.job_id, "worker-1").await?;
    /// ```
    async fn ack(&self, job_id: &str, consumer_id: &str) -> Result<(), Box<dyn Error>>;

    /// Negative acknowledgement (job failed)
    ///
    /// Indicates that a job could not be processed successfully. Depending on
    /// the implementation, this may requeue the job for retry or move it to
    /// a dead letter queue.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The unique identifier of the failed job
    /// * `consumer_id` - The consumer that attempted to process the job
    /// * `reason` - Optional description of why the job failed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Job failure was recorded successfully
    /// * `Err(Box<dyn Error>)` - If the nack operation failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// queue.nack(&job.job_id, "worker-1", Some("Invalid data".to_string())).await?;
    /// ```
    async fn nack(
        &self,
        job_id: &str,
        consumer_id: &str,
        reason: Option<String>,
    ) -> Result<(), Box<dyn Error>>;

    /// Get the current queue length
    ///
    /// Returns the number of jobs currently in the queue waiting to be
    /// processed. Useful for monitoring and metrics.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The number of jobs in the queue
    /// * `Err(Box<dyn Error>)` - If the operation failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// let backlog = queue.get_queue_length().await?;
    /// println!("Queue has {} pending jobs", backlog);
    /// ```
    async fn get_queue_length(&self) -> Result<usize, Box<dyn Error>>;
}
