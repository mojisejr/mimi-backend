//! Tarot-Specific Queue Implementation
//!
//! High-level queue operations specific to tarot reading workflow.
//! Integrates Redis queue with database persistence through JobRepository.

use crate::{queue::Queue, repository::JobRepository};
use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

/// Tarot Queue - Tarot-specific queue operations
///
/// Provides high-level operations specifically designed for tarot reading workflows.
/// Integrates queue operations with database persistence through JobRepository.
#[derive(Clone)]
pub struct TarotQueue {
    /// Job repository for database operations
    repository: JobRepository,
}

impl TarotQueue {
    /// Create new TarotQueue instance
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `queue` - Queue implementation (Redis, Upstash, InMemory)
    pub fn new(pool: PgPool, queue: Arc<dyn Queue + Send + Sync>) -> Self {
        let repository = JobRepository::new(pool, queue);
        Self { repository }
    }

    /// Submit tarot reading request
    ///
    /// High-level method for submitting tarot reading requests.
    /// Handles validation, deduplication, and job creation.
    ///
    /// # Arguments
    ///
    /// * `question` - The tarot reading question
    /// * `user_id` - Optional user identifier
    /// * `card_count` - Number of cards to draw (3 or 5, defaults to 3)
    ///
    /// # Returns
    ///
    /// * `Ok(ReadingSubmissionResult)` - Job tracking information
    /// * `Err(Box<dyn Error>)` - Error if submission failed
    pub async fn submit_reading_request(
        &self,
        question: &str,
        user_id: Option<&str>,
        card_count: Option<u32>,
    ) -> Result<ReadingSubmissionResult, Box<dyn Error>> {
        let job_id = self
            .repository
            .submit_tarot_reading(question, user_id, card_count)
            .await?;

        Ok(ReadingSubmissionResult {
            job_id,
            status: crate::models::JobStatusType::Queued,
            message: "Tarot reading request submitted successfully".to_string(),
            estimated_wait_seconds: 60, // TODO: Calculate based on queue depth
            submitted_at: chrono::Utc::now(),
        })
    }

    /// Get reading result
    ///
    /// Retrieve the result of a tarot reading job.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier
    ///
    /// # Returns
    ///
    /// * `Ok(Option<ReadingResult>)` - Reading result if available
    /// * `Err(Box<dyn Error>)` - Error if lookup failed
    pub async fn get_reading_result(
        &self,
        job_id: Uuid,
    ) -> Result<Option<ReadingResult>, Box<dyn Error>> {
        let job = self.repository.get_job(job_id).await?;

        match job {
            Some(job_record) => {
                let result = ReadingResult {
                    job_id: job_record.id,
                    status: job_record.status,
                    question: job_record
                        .payload
                        .get("question")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown question")
                        .to_string(),
                    result: job_record.result.map(|r| r.0),
                    created_at: job_record.created_at,
                    started_at: job_record.started_at,
                    completed_at: job_record.completed_at,
                    error_message: job_record.error_info.last_error,
                };
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Complete reading job
    ///
    /// Mark a reading job as completed with the reading result.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier
    /// * `reading_result` - The tarot reading result
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(Box<dyn Error>)` - Error if update failed
    pub async fn complete_reading(
        &self,
        job_id: Uuid,
        reading_result: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        self.repository.complete_job(job_id, reading_result).await
    }

    /// Mark reading job as failed
    ///
    /// Mark a reading job as failed with optional error message.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier
    /// * `worker_id` - Optional worker identifier
    /// * `error_message` - Optional error description
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(Box<dyn Error>)` - Error if update failed
    pub async fn fail_reading(
        &self,
        job_id: Uuid,
        worker_id: Option<String>,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        self.repository
            .fail_job(job_id, worker_id, error_message)
            .await
    }

    /// Get pending readings count
    ///
    /// Returns the number of pending tarot reading requests.
    ///
    /// # Returns
    ///
    /// * `Ok(PendingCount)` - Pending reading statistics
    /// * `Err(Box<dyn Error>)` - Error if lookup failed
    pub async fn get_pending_count(&self) -> Result<PendingCount, Box<dyn Error>> {
        let stats = self.repository.get_queue_stats().await?;

        Ok(PendingCount {
            queued: stats.queued_in_db,
            processing: stats.processing_in_db,
            total: stats.total_pending,
        })
    }

    /// Create TarotQueue from environment variables
    ///
    /// Configures the queue using environment variables:
    /// - `DATABASE_URL`: PostgreSQL connection string
    /// - Queue configuration based on available services
    ///
    /// # Returns
    ///
    /// * `Ok(TarotQueue)` - Configured TarotQueue
    /// * `Err(Box<dyn Error>)` - Configuration error
    pub async fn from_env() -> Result<Self, Box<dyn Error>> {
        use crate::queue::inmemory_queue::InMemoryQueue;

        // Setup database connection
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable not set")?;
        println!(
            "Connecting to database: {}...",
            &database_url[..database_url.find('@').unwrap_or(database_url.len())]
        );
        let pool = sqlx::PgPool::connect(&database_url).await?;
        println!("✅ Database connection successful");

        // Try to setup queue based on available configuration
        let queue: Arc<dyn Queue + Send + Sync> =
            // Use in-memory queue for development (temporary fix)
            {
                println!("Using in-memory queue (development mode)");
                Arc::new(InMemoryQueue::new())
            };

        Ok(Self::new(pool, queue))
    }
}

/// Result of submitting a tarot reading request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadingSubmissionResult {
    /// Job identifier for tracking
    pub job_id: Uuid,
    /// Current job status
    pub status: crate::models::JobStatusType,
    /// Human-readable message
    pub message: String,
    /// Estimated wait time in seconds
    pub estimated_wait_seconds: u32,
    /// When the request was submitted
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

/// Result of a completed tarot reading
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadingResult {
    /// Job identifier
    pub job_id: Uuid,
    /// Current job status
    pub status: crate::models::JobStatusType,
    /// Original question
    pub question: String,
    /// Reading result if completed
    pub result: Option<serde_json::Value>,
    /// When the job was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When processing started
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When processing completed
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Count of pending readings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingCount {
    /// Number of queued jobs
    pub queued: usize,
    /// Number of jobs currently processing
    pub processing: usize,
    /// Total pending jobs
    pub total: usize,
}

#[async_trait]
pub trait TarotQueueTrait: Send + Sync {
    async fn submit_reading_request(
        &self,
        question: &str,
        user_id: Option<&str>,
        card_count: Option<u32>,
    ) -> Result<ReadingSubmissionResult, Box<dyn Error>>;

    async fn get_reading_result(
        &self,
        job_id: Uuid,
    ) -> Result<Option<ReadingResult>, Box<dyn Error>>;

    async fn complete_reading(
        &self,
        job_id: Uuid,
        reading_result: serde_json::Value,
    ) -> Result<(), Box<dyn Error>>;

    async fn fail_reading(
        &self,
        job_id: Uuid,
        worker_id: Option<String>,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl TarotQueueTrait for TarotQueue {
    async fn submit_reading_request(
        &self,
        question: &str,
        user_id: Option<&str>,
        card_count: Option<u32>,
    ) -> Result<ReadingSubmissionResult, Box<dyn Error>> {
        self.submit_reading_request(question, user_id, card_count)
            .await
    }

    async fn get_reading_result(
        &self,
        job_id: Uuid,
    ) -> Result<Option<ReadingResult>, Box<dyn Error>> {
        self.get_reading_result(job_id).await
    }

    async fn complete_reading(
        &self,
        job_id: Uuid,
        reading_result: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        self.complete_reading(job_id, reading_result).await
    }

    async fn fail_reading(
        &self,
        job_id: Uuid,
        worker_id: Option<String>,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        self.fail_reading(job_id, worker_id, error_message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_tarot_queue_creation() {
        // This test will fail until we have proper database setup
        // For now, we'll structure it to expect the functionality

        // let pool = setup_test_db().await;
        // let queue = Arc::new(InMemoryQueue::new());
        // let tarot_queue = TarotQueue::new(pool, queue);

        // let result = tarot_queue.submit_reading_request(
        //     "ควรจะลงทุนอะไรดีครับ",
        //     Some("test-user"),
        //     Some(3)
        // ).await;

        // assert!(result.is_ok());
        // let submission = result.unwrap();
        // assert_ne!(submission.job_id, Uuid::nil());
        // assert_eq!(submission.status, JobStatus::Queued);

        // For now, we'll just test that we can create the struct
        // This should work without database
        panic!("Database setup required for full testing");
    }

    #[test]
    fn test_reading_submission_result_serialization() {
        let result = ReadingSubmissionResult {
            job_id: Uuid::new_v4(),
            status: crate::models::JobStatusType::Queued,
            message: "Test submission".to_string(),
            estimated_wait_seconds: 60,
            submitted_at: chrono::Utc::now(),
        };

        // Test serialization
        let json_str = serde_json::to_string(&result).expect("Should serialize");
        let deserialized: ReadingSubmissionResult =
            serde_json::from_str(&json_str).expect("Should deserialize");

        assert_eq!(result.job_id, deserialized.job_id);
        assert_eq!(result.message, deserialized.message);
        assert_eq!(
            result.estimated_wait_seconds,
            deserialized.estimated_wait_seconds
        );
    }

    #[test]
    fn test_reading_result_serialization() {
        let result = ReadingResult {
            job_id: Uuid::new_v4(),
            status: crate::models::JobStatusType::Succeeded,
            question: "ควรจะลงทุนอะไรดีครับ".to_string(),
            result: Some(json!({"cards": ["The Fool", "The Magician"]})),
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            completed_at: Some(chrono::Utc::now()),
            error_message: None,
        };

        // Test serialization
        let json_str = serde_json::to_string(&result).expect("Should serialize");
        let deserialized: ReadingResult =
            serde_json::from_str(&json_str).expect("Should deserialize");

        assert_eq!(result.job_id, deserialized.job_id);
        assert_eq!(result.question, deserialized.question);
        assert_eq!(result.status, deserialized.status);
        assert!(deserialized.result.is_some());
    }
}
