//! Reading Job Models - Simple Version
//!
//! Simplified database models for tarot reading job management.
//! Uses runtime SQL queries to avoid compile-time database dependency.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, PgPool, Row};
use uuid::Uuid;

/// Job status enum for tracking job lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is queued and waiting for processing
    Queued,
    /// Job is currently being processed by a worker
    Processing,
    /// Job completed successfully
    Succeeded,
    /// Job failed and may be retried
    Failed,
    /// Job moved to Dead Letter Queue (permanent failure)
    Dlq,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::Processing => write!(f, "processing"),
            JobStatus::Succeeded => write!(f, "succeeded"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Dlq => write!(f, "dlq"),
        }
    }
}

/// Error type for job status operations
#[derive(Debug, thiserror::Error)]
pub enum JobStatusError {
    #[error("Invalid job status: {0}")]
    InvalidStatus(String),
}

/// Main job model representing a tarot reading job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingJob {
    /// Unique job identifier
    pub id: Uuid,
    /// Type of job (typically "tarot_reading")
    pub job_type: String,
    /// Current job status
    pub status: JobStatus,
    /// Job input data
    pub payload: Json<serde_json::Value>,
    /// Job result (when completed)
    pub result: Option<Json<serde_json::Value>>,
    /// Worker ID processing this job
    pub worker_id: Option<String>,
    /// Number of attempts made
    pub attempts: i32,
    /// Maximum allowed attempts
    pub max_attempts: i32,
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Input for creating a new job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobInput {
    /// Job input data
    pub payload: serde_json::Value,
    /// Optional deduplication key
    pub dedupe_key: Option<String>,
    /// Maximum allowed attempts (defaults to 5)
    pub max_attempts: Option<i32>,
}

impl ReadingJob {
    /// Create a new reading job
    pub async fn create(pool: &PgPool, input: CreateJobInput) -> Result<Uuid, sqlx::Error> {
        let job_id = Uuid::new_v4();
        let max_attempts = input.max_attempts.unwrap_or(5);

        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO jobs (
                id, job_type, payload, dedupe_key, max_attempts
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(job_id)
        .bind("tarot_reading")
        .bind(Json(input.payload))
        .bind(input.dedupe_key)
        .bind(max_attempts)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    /// Find a job by ID
    pub async fn find_by_id(
        pool: &PgPool,
        job_id: Uuid,
    ) -> Result<Option<ReadingJob>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id, job_type, status, payload, result, worker_id,
                attempts, max_attempts, created_at, updated_at,
                started_at, completed_at
            FROM jobs
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "queued" => JobStatus::Queued,
                    "processing" => JobStatus::Processing,
                    "succeeded" => JobStatus::Succeeded,
                    "failed" => JobStatus::Failed,
                    "dlq" => JobStatus::Dlq,
                    _ => JobStatus::Queued, // Default fallback
                };

                Ok(Some(ReadingJob {
                    id: row.get("id"),
                    job_type: row.get("job_type"),
                    status,
                    payload: row.get("payload"),
                    result: row.get("result"),
                    worker_id: row.get("worker_id"),
                    attempts: row.get("attempts"),
                    max_attempts: row.get("max_attempts"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    started_at: row.get("started_at"),
                    completed_at: row.get("completed_at"),
                }))
            }
            None => Ok(None),
        }
    }

    /// Get jobs by status (for queue processing)
    pub async fn find_by_status(
        pool: &PgPool,
        status: JobStatus,
        limit: i64,
    ) -> Result<Vec<ReadingJob>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, job_type, status, payload, result, worker_id,
                attempts, max_attempts, created_at, updated_at,
                started_at, completed_at
            FROM jobs
            WHERE status = $1
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(status.to_string())
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut jobs = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "queued" => JobStatus::Queued,
                "processing" => JobStatus::Processing,
                "succeeded" => JobStatus::Succeeded,
                "failed" => JobStatus::Failed,
                "dlq" => JobStatus::Dlq,
                _ => JobStatus::Queued,
            };

            jobs.push(ReadingJob {
                id: row.get("id"),
                job_type: row.get("job_type"),
                status,
                payload: row.get("payload"),
                result: row.get("result"),
                worker_id: row.get("worker_id"),
                attempts: row.get("attempts"),
                max_attempts: row.get("max_attempts"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
            });
        }

        Ok(jobs)
    }

    /// Update job status
    pub async fn update_status(
        pool: &PgPool,
        job_id: Uuid,
        status: JobStatus,
        worker_id: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE jobs SET
                status = $1,
                worker_id = $2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            "#,
        )
        .bind(status.to_string())
        .bind(worker_id)
        .bind(job_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update job result when completed successfully
    pub async fn update_result(
        pool: &PgPool,
        job_id: Uuid,
        result: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE jobs SET
                result = $1,
                status = 'succeeded',
                completed_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(Json(result))
        .bind(job_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.status == JobStatus::Failed && self.attempts < self.max_attempts
    }

    /// Check if job is finished (no further processing possible)
    pub fn is_finished(&self) -> bool {
        matches!(self.status, JobStatus::Succeeded | JobStatus::Dlq)
    }
}
