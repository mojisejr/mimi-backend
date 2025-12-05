//! Simplified Job Types for Queue Integration
//!
//! Minimal job types needed for queue integration without complex database dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

/// Simplified job status enum
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

/// Simplified job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Schema version for compatibility
    pub schema_version: String,
    /// Prompt version used for processing
    pub prompt_version: Option<String>,
    /// Deduplication key for idempotent operations
    pub dedupe_key: Option<String>,
}

/// Simplified job retry information
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

/// Simplified job error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobError {
    /// Last error message
    pub last_error: Option<String>,
    /// When the error occurred
    pub last_error_at: Option<DateTime<Utc>>,
}

/// Simplified reading job model
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
