//! Job Repository - Bridge between Queue and Database
//!
//! Provides dual persistence for jobs: queue for processing + database for state tracking.
//! Implements the repository pattern for job lifecycle management.

use crate::{
    models::{
        CreateJobInputType as CreateJobInput, JobError, JobMetadata, JobRetryInfo,
        JobStatusType as JobStatus, ReadingJobType as ReadingJob,
    },
    queue::types::JobPayload,
    queue::Queue,
};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

/// Job Repository - Manages job lifecycle with dual persistence
///
/// This repository provides the core integration between:
/// - Database: Persistent job storage with state tracking
/// - Queue: High-performance job processing
/// - API: High-level job management operations
#[derive(Clone)]
pub struct JobRepository {
    /// Database connection pool
    pool: PgPool,
    /// Queue implementation for job processing
    queue: Arc<dyn Queue + Send + Sync>,
}

impl JobRepository {
    /// Create new JobRepository instance
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `queue` - Queue implementation (Redis, Upstash, InMemory)
    pub fn new(pool: PgPool, queue: Arc<dyn Queue + Send + Sync>) -> Self {
        Self { pool, queue }
    }

    /// Create a new job with dual persistence
    ///
    /// This method implements the core workflow:
    /// 1. Create database record with status = Queued
    /// 2. Submit job to queue for processing
    /// 3. Return job ID for tracking
    ///
    /// # Arguments
    ///
    /// * `input` - Job creation parameters
    ///
    /// # Returns
    ///
    /// * `Ok(Uuid)` - Job ID if successfully created
    /// * `Err(Box<dyn Error>)` - Error if creation failed
    pub async fn create_job(&self, input: CreateJobInput) -> Result<Uuid, Box<dyn Error>> {
        // Generate job ID first
        let job_id = Uuid::new_v4();

        // Step 1: Convert to queue payload first (to avoid move issues)
        let payload_json = input.payload.clone();
        let payload = JobPayload {
            job_id: job_id.to_string(),
            user_id: payload_json
                .get("user_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            question: payload_json
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("Default question")
                .to_string(),
            card_count: payload_json
                .get("card_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as u32,
            schema_version: "1".to_string(),
            prompt_version: input
                .prompt_version
                .unwrap_or_else(|| "v2025-11-20-a".to_string()),
            dedupe_key: input.dedupe_key,
            trace_id: Some(job_id.to_string()),
            created_at: chrono::Utc::now(),
            metadata: payload_json
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        };

        // Step 2: Create database record (simplified for now - will be enhanced)
        // TODO: Replace with actual database implementation once schema is ready
        let _query_result = sqlx::query!(
            r#"
            INSERT INTO jobs (
                id,
                job_type,
                payload,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
            job_id,
            input
                .job_type
                .unwrap_or_else(|| "tarot_reading".to_string()),
            serde_json::Value::from(input.payload),
            chrono::Utc::now(),
            chrono::Utc::now()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create job in database: {}", e))?;

        // Step 3: Submit to queue
        let queue_job_id = self.queue.enqueue(payload).await?;

        // Step 4: Verify queue job ID matches database job ID
        if queue_job_id != job_id.to_string() {
            // Log warning but don't fail - this is just for debugging
            eprintln!(
                "Warning: Queue job ID ({}) doesn't match database job ID ({})",
                queue_job_id, job_id
            );
        }

        Ok(job_id)
    }

    /// Submit tarot reading request (high-level API)
    ///
    /// Convenience method that handles the complete tarot reading submission workflow.
    ///
    /// # Arguments
    ///
    /// * `question` - The tarot reading question
    /// * `user_id` - Optional user identifier
    /// * `card_count` - Number of cards to draw (3 or 5)
    ///
    /// # Returns
    ///
    /// * `Ok(Uuid)` - Job ID for tracking
    /// * `Err(Box<dyn Error>)` - Error if submission failed
    pub async fn submit_tarot_reading(
        &self,
        question: &str,
        user_id: Option<&str>,
        card_count: Option<u32>,
    ) -> Result<Uuid, Box<dyn Error>> {
        // Validate inputs
        let trimmed_question = question.trim();
        if trimmed_question.is_empty() {
            return Err("Question cannot be empty".into());
        }

        let actual_card_count = card_count.unwrap_or(3);
        if !(1..=7).contains(&actual_card_count) {
            return Err("Card count must be between 1 and 7".into());
        }

        // Create job input
        let job_input = CreateJobInput {
            job_type: Some("tarot_reading".to_string()),
            payload: json!({
                "question": trimmed_question,
                "user_id": user_id.unwrap_or("anonymous"),
                "card_count": actual_card_count,
                "locale": "th",
                "source": "api"
            }),
            dedupe_key: Some(format!(
                "tarot:{}:{}",
                user_id.unwrap_or("anonymous"),
                trimmed_question.chars().take(50).collect::<String>()
            )),
            max_attempts: Some(5),
            prompt_version: Some("v2025-11-20-a".to_string()),
        };

        self.create_job(job_input).await
    }

    /// Get job by ID with full information
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job UUID
    ///
    /// # Returns
    ///
    /// * `Ok(Option<ReadingJob>)` - Job if found, None if not found
    /// * `Err(Box<dyn Error>)` - Database error
    pub async fn get_job(&self, job_id: Uuid) -> Result<Option<ReadingJob>, Box<dyn Error>> {
        // Simplified implementation for now - TODO: Use full ReadingJob implementation
        let row = sqlx::query!(
            r#"
            SELECT
                id,
                job_type,
                payload,
                result,
                created_at,
                updated_at,
                started_at,
                completed_at
            FROM jobs
            WHERE id = $1
            "#,
            job_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch job: {}", e))?;

        match row {
            Some(row) => {
                let job = ReadingJob {
                    id: row.id,
                    job_type: row.job_type,
                    status: JobStatus::Queued, // Default status for now
                    payload: sqlx::types::Json(row.payload),
                    result: row.result.map(sqlx::types::Json),
                    metadata: JobMetadata {
                        schema_version: "1".to_string(),
                        prompt_version: Some("v2025-11-20-a".to_string()),
                        dedupe_key: None,
                    },
                    worker_id: None,
                    retry_info: JobRetryInfo {
                        attempts: 0,
                        max_attempts: 5,
                        visibility_timeout_secs: 300,
                        next_retry_at: None,
                    },
                    error_info: JobError {
                        last_error: None,
                        last_error_at: None,
                    },
                    created_at: row.created_at.unwrap_or_else(Utc::now),
                    updated_at: row.updated_at.unwrap_or_else(Utc::now),
                    started_at: row.started_at,
                    completed_at: row.completed_at,
                };
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Get jobs by status
    ///
    /// # Arguments
    ///
    /// * `status` - Job status filter
    /// * `limit` - Maximum number of jobs to return
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ReadingJob>)` - List of jobs
    /// * `Err(Box<dyn Error>)` - Database error
    pub async fn get_jobs_by_status(
        &self,
        _status: JobStatus,
        limit: i64,
    ) -> Result<Vec<ReadingJob>, Box<dyn Error>> {
        // Simplified implementation - return empty list for now
        // TODO: Implement proper status filtering once database schema is complete
        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                job_type,
                payload,
                result,
                created_at,
                updated_at
            FROM jobs
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch jobs by status: {}", e))?;

        let jobs = rows
            .into_iter()
            .map(|row| {
                ReadingJob {
                    id: row.id,
                    job_type: row.job_type,
                    status: JobStatus::Queued, // Default for now
                    payload: sqlx::types::Json(row.payload),
                    result: row.result.map(sqlx::types::Json),
                    metadata: JobMetadata {
                        schema_version: "1".to_string(),
                        prompt_version: Some("v2025-11-20-a".to_string()),
                        dedupe_key: None,
                    },
                    worker_id: None,
                    retry_info: JobRetryInfo {
                        attempts: 0,
                        max_attempts: 5,
                        visibility_timeout_secs: 300,
                        next_retry_at: None,
                    },
                    error_info: JobError {
                        last_error: None,
                        last_error_at: None,
                    },
                    created_at: row.created_at.unwrap_or_else(Utc::now),
                    updated_at: row.updated_at.unwrap_or_else(Utc::now),
                    started_at: None,
                    completed_at: None,
                }
            })
            .collect();

        Ok(jobs)
    }

    /// Update job result when processing completes
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job UUID
    /// * `result` - Processing result as JSON value
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(Box<dyn Error>)` - Database error
    pub async fn complete_job(
        &self,
        job_id: Uuid,
        result: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        let _query_result = sqlx::query!(
            r#"
            UPDATE jobs SET
                result = $1,
                status = 'succeeded',
                completed_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            serde_json::Value::from(result),
            job_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to complete job: {}", e))?;

        Ok(())
    }

    /// Mark job as failed with optional error message
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job UUID
    /// * `worker_id` - Optional worker identifier
    /// * `error_message` - Optional error description
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(Box<dyn Error>)` - Database error
    pub async fn fail_job(
        &self,
        job_id: Uuid,
        _worker_id: Option<String>,
        _error_message: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        // Simplified implementation - just update status
        let _query_result = sqlx::query!(
            r#"
            UPDATE jobs SET
                status = 'failed',
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
            job_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to mark job as failed: {}", e))?;

        Ok(())
    }

    /// Get queue statistics
    ///
    /// Returns current queue depth and database statistics.
    ///
    /// # Returns
    ///
    /// * `Ok(QueueStats)` - Queue and database statistics
    /// * `Err(Box<dyn Error>)` - Error if statistics unavailable
    pub async fn get_queue_stats(&self) -> Result<QueueStats, Box<dyn Error>> {
        let queue_length = self.queue.get_queue_length().await?;
        let queued_jobs = self.get_jobs_by_status(JobStatus::Queued, 1000).await?;
        let processing_jobs = self.get_jobs_by_status(JobStatus::Processing, 1000).await?;

        Ok(QueueStats {
            queue_depth: queue_length,
            queued_in_db: queued_jobs.len(),
            processing_in_db: processing_jobs.len(),
            total_pending: queued_jobs.len() + processing_jobs.len(),
        })
    }
}

/// Queue statistics structure
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Number of jobs in the queue
    pub queue_depth: usize,
    /// Number of queued jobs in database
    pub queued_in_db: usize,
    /// Number of processing jobs in database
    pub processing_in_db: usize,
    /// Total pending jobs
    pub total_pending: usize,
}

#[async_trait]
pub trait JobRepositoryTrait: Send + Sync {
    async fn create_job(&self, input: CreateJobInput) -> Result<Uuid, Box<dyn Error>>;
    async fn submit_tarot_reading(
        &self,
        question: &str,
        user_id: Option<&str>,
        card_count: Option<u32>,
    ) -> Result<Uuid, Box<dyn Error>>;
    async fn get_job(&self, job_id: Uuid) -> Result<Option<ReadingJob>, Box<dyn Error>>;
    async fn complete_job(
        &self,
        job_id: Uuid,
        result: serde_json::Value,
    ) -> Result<(), Box<dyn Error>>;
    async fn fail_job(
        &self,
        job_id: Uuid,
        worker_id: Option<String>,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl JobRepositoryTrait for JobRepository {
    async fn create_job(&self, input: CreateJobInput) -> Result<Uuid, Box<dyn Error>> {
        self.create_job(input).await
    }

    async fn submit_tarot_reading(
        &self,
        question: &str,
        user_id: Option<&str>,
        card_count: Option<u32>,
    ) -> Result<Uuid, Box<dyn Error>> {
        self.submit_tarot_reading(question, user_id, card_count)
            .await
    }

    async fn get_job(&self, job_id: Uuid) -> Result<Option<ReadingJob>, Box<dyn Error>> {
        self.get_job(job_id).await
    }

    async fn complete_job(
        &self,
        job_id: Uuid,
        result: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        self.complete_job(job_id, result).await
    }

    async fn fail_job(
        &self,
        job_id: Uuid,
        worker_id: Option<String>,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        self.fail_job(job_id, worker_id, error_message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::inmemory_queue::InMemoryQueue;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_job_repository_creation() {
        // This test will fail until we have proper database setup
        // For now, we'll structure it to expect the functionality

        // let pool = setup_test_db().await;
        // let queue = Arc::new(InMemoryQueue::new());
        // let repository = JobRepository::new(pool, queue);

        // let input = CreateJobInput {
        //     job_type: Some("tarot_reading".to_string()),
        //     payload: json!({"question": "test"}),
        //     dedupe_key: None,
        //     max_attempts: Some(5),
        //     prompt_version: Some("v1".to_string()),
        // };

        // let result = repository.create_job(input).await;
        // assert!(result.is_ok());

        // For now, we'll just test that we can create the struct
        // This should work without database
        panic!("Database setup required for full testing");
    }

    // Tests temporarily disabled due to import conflicts
    // Will be re-enabled when full schema is implemented
    /*
    #[test]
    fn test_convert_db_job_to_queue_payload() {
        // This test would need the repository instance
        // For now, we'll just validate the structure
        panic!("Repository instance needed for conversion testing");
    }
    */
}
