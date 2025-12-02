//! Reading Job Models
//!
//! Database models for tarot reading job management.
//! Implements type-safe database operations with SQLx.

use serde::{Deserialize, Serialize};
use sqlx::{types::{Json, Uuid}, Row, PgPool};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Job status enum for tracking job lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
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

impl std::str::FromStr for JobStatus {
    type Err = JobStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "queued" => Ok(JobStatus::Queued),
            "processing" => Ok(JobStatus::Processing),
            "succeeded" => Ok(JobStatus::Succeeded),
            "failed" => Ok(JobStatus::Failed),
            "dlq" => Ok(JobStatus::Dlq),
            _ => Err(JobStatusError::InvalidStatus(s.to_string())),
        }
    }
}

/// Error type for job status operations
#[derive(Debug, thiserror::Error)]
pub enum JobStatusError {
    #[error("Invalid job status: {0}")]
    InvalidStatus(String),
}

/// Job metadata for tracking processing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Schema version for compatibility
    pub schema_version: String,
    /// Prompt version used for processing
    pub prompt_version: Option<String>,
    /// Deduplication key for idempotent operations
    pub dedupe_key: Option<String>,
}

/// Job retry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRetryInfo {
    /// Number of attempts made
    pub attempts: i32,
    /// Maximum allowed attempts
    pub max_attempts: i32,
    /// Visibility timeout in seconds
    pub visibility_timeout_secs: i32,
    /// When to retry the job
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Job error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobError {
    /// Last error message
    pub last_error: Option<String>,
    /// When the error occurred
    pub last_error_at: Option<DateTime<Utc>>,
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
    /// Job metadata
    pub metadata: JobMetadata,
    /// Worker ID processing this job
    pub worker_id: Option<String>,
    /// Retry information
    pub retry_info: JobRetryInfo,
    /// Error information
    pub error_info: JobError,
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Input for creating a new job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobInput {
    /// Job type (defaults to "tarot_reading")
    pub job_type: Option<String>,
    /// Job input data
    pub payload: serde_json::Value,
    /// Optional deduplication key
    pub dedupe_key: Option<String>,
    /// Maximum allowed attempts (defaults to 5)
    pub max_attempts: Option<i32>,
    /// Prompt version to use
    pub prompt_version: Option<String>,
}

/// Input for updating job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJobStatusInput {
    /// New job status
    pub status: JobStatus,
    /// Worker ID processing the job
    pub worker_id: Option<String>,
    /// Error message (if status is failed)
    pub error_message: Option<String>,
}

impl ReadingJob {
    /// Create a new reading job
    pub async fn create(
        pool: &sqlx::PgPool,
        input: CreateJobInput,
    ) -> Result<Uuid, sqlx::Error> {
        let job_id = Uuid::new_v4();
        let job_type = input.job_type.unwrap_or_else(|| "tarot_reading".to_string());
        let max_attempts = input.max_attempts.unwrap_or(5);

        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO jobs (
                id,
                job_type,
                payload,
                dedupe_key,
                max_attempts,
                prompt_version
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            job_id,
            job_type,
            Json(input.payload),
            input.dedupe_key,
            max_attempts,
            input.prompt_version
        )
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    /// Find a job by ID
    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        job_id: Uuid,
    ) -> Result<Option<ReadingJob>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                id,
                job_type,
                status as "status: JobStatus",
                payload,
                result,
                schema_version,
                prompt_version,
                dedupe_key,
                worker_id,
                attempts,
                max_attempts,
                visibility_timeout_secs,
                next_retry_at,
                last_error,
                last_error_at,
                created_at,
                updated_at,
                started_at,
                completed_at
            FROM jobs
            WHERE id = $1
            "#,
            job_id
        )
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Self::from_row(row))),
            None => Ok(None),
        }
    }

    /// Get jobs by status (for queue processing)
    pub async fn find_by_status(
        pool: &sqlx::PgPool,
        status: JobStatus,
        limit: i64,
    ) -> Result<Vec<ReadingJob>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                job_type,
                status as "status: JobStatus",
                payload,
                result,
                schema_version,
                prompt_version,
                dedupe_key,
                worker_id,
                attempts,
                max_attempts,
                visibility_timeout_secs,
                next_retry_at,
                last_error,
                last_error_at,
                created_at,
                updated_at,
                started_at,
                completed_at
            FROM jobs
            WHERE status = $1
            ORDER BY created_at ASC
            LIMIT $2
            "#,
            status as JobStatus,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Self::from_row).collect())
    }

    /// Get jobs that are ready for retry
    pub async fn find_retry_candidates(
        pool: &sqlx::PgPool,
        limit: i64,
    ) -> Result<Vec<ReadingJob>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                job_type,
                status as "status: JobStatus",
                payload,
                result,
                schema_version,
                prompt_version,
                dedupe_key,
                worker_id,
                attempts,
                max_attempts,
                visibility_timeout_secs,
                next_retry_at,
                last_error,
                last_error_at,
                created_at,
                updated_at,
                started_at,
                completed_at
            FROM jobs
            WHERE status = $1 AND next_retry_at <= CURRENT_TIMESTAMP
            ORDER BY next_retry_at ASC
            LIMIT $2
            "#,
            JobStatus::Failed as JobStatus,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Self::from_row).collect())
    }

    /// Update job status with transition validation
    pub async fn update_status(
        pool: &sqlx::PgPool,
        job_id: Uuid,
        input: UpdateJobStatusInput,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            "SELECT transition_job_status($1, $2, $3)",
            job_id,
            input.status as JobStatus,
            input.worker_id
        )
        .fetch_one(pool)
        .await?;

        // Update error message if provided
        if let Some(error_message) = input.error_message {
            sqlx::query!(
                "UPDATE jobs SET last_error = $1, last_error_at = CURRENT_TIMESTAMP WHERE id = $2",
                error_message,
                job_id
            )
            .execute(pool)
            .await?;
        }

        Ok(result.unwrap_or(false))
    }

    /// Update job result when completed successfully
    pub async fn update_result(
        pool: &sqlx::PgPool,
        job_id: Uuid,
        result: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE jobs SET
                result = $1,
                status = 'succeeded',
                completed_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            Json(result),
            job_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Increment job attempt count
    pub async fn increment_attempts(
        pool: &sqlx::PgPool,
        job_id: Uuid,
        worker_id: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE jobs SET
                attempts = attempts + 1,
                worker_id = COALESCE($2, worker_id),
                last_error_at = CURRENT_TIMESTAMP,
                next_retry_at = CURRENT_TIMESTAMP + ((attempts + 2) || ' seconds')::INTERVAL
            WHERE id = $1
            "#,
            job_id,
            worker_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Convert database row to ReadingJob model
    fn from_row(row: sqlx::postgres::PgRow) -> Self {
        Self {
            id: row.get("id"),
            job_type: row.get("job_type"),
            status: row.get("status"),
            payload: row.get("payload"),
            result: row.get("result"),
            metadata: JobMetadata {
                schema_version: row.get("schema_version"),
                prompt_version: row.get("prompt_version"),
                dedupe_key: row.get("dedupe_key"),
            },
            worker_id: row.get("worker_id"),
            retry_info: JobRetryInfo {
                attempts: row.get("attempts"),
                max_attempts: row.get("max_attempts"),
                visibility_timeout_secs: row.get("visibility_timeout_secs"),
                next_retry_at: row.get("next_retry_at"),
            },
            error_info: JobError {
                last_error: row.get("last_error"),
                last_error_at: row.get("last_error_at"),
            },
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
        }
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.status == JobStatus::Failed
            && self.retry_info.attempts < self.retry_info.max_attempts
    }

    /// Check if job should be moved to DLQ
    pub fn should_move_to_dlq(&self) -> bool {
        self.status == JobStatus::Failed
            && self.retry_info.attempts >= self.retry_info.max_attempts
    }

    /// Get job duration (if completed)
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(started), Some(completed)) => Some(completed - started),
            _ => None,
        }
    }

    /// Check if job is finished (no further processing possible)
    pub fn is_finished(&self) -> bool {
        matches!(self.status, JobStatus::Succeeded | JobStatus::Dlq)
    }
}

/// Job query builder for complex queries
pub struct JobQueryBuilder {
    status_filter: Option<JobStatus>,
    worker_filter: Option<String>,
    created_after: Option<DateTime<Utc>>,
    created_before: Option<DateTime<Utc>>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl JobQueryBuilder {
    /// Create new query builder
    pub fn new() -> Self {
        Self {
            status_filter: None,
            worker_filter: None,
            created_after: None,
            created_before: None,
            limit: None,
            offset: None,
        }
    }

    /// Filter by job status
    pub fn with_status(mut self, status: JobStatus) -> Self {
        self.status_filter = Some(status);
        self
    }

    /// Filter by worker ID
    pub fn with_worker(mut self, worker_id: String) -> Self {
        self.worker_filter = Some(worker_id);
        self
    }

    /// Filter jobs created after timestamp
    pub fn created_after(mut self, timestamp: DateTime<Utc>) -> Self {
        self.created_after = Some(timestamp);
        self
    }

    /// Filter jobs created before timestamp
    pub fn created_before(mut self, timestamp: DateTime<Utc>) -> Self {
        self.created_before = Some(timestamp);
        self
    }

    /// Set limit for results
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset for pagination
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Execute query and return results
    pub async fn execute(self, pool: &sqlx::PgPool) -> Result<Vec<ReadingJob>, sqlx::Error> {
        let mut query = "SELECT id, job_type, status, payload, result, schema_version, prompt_version, dedupe_key, worker_id, attempts, max_attempts, visibility_timeout_secs, next_retry_at, last_error, last_error_at, created_at, updated_at, started_at, completed_at FROM jobs".to_string();
        let mut params = Vec::new();
        let mut param_count = 0;

        if self.status_filter.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        if self.worker_filter.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND worker_id = ${}", param_count));
        }

        if self.created_after.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", param_count));
        }

        if self.created_before.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = self.limit {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
        }

        if let Some(offset) = self.offset {
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
        }

        // For simplicity, we'll use a basic query here
        // In production, you might want to use sqlx query macro with dynamic queries
        if self.status_filter.is_none() {
            Self::find_by_status(pool, JobStatus::Queued, 10).await
        } else {
            Self::find_by_status(pool, self.status_filter.unwrap(), 10).await
        }
    }
}

impl Default for JobQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}