//! Job Attempts Repository Implementation
//!
//! Repository pattern implementation for job attempt tracking operations.
//! Provides methods for attempt lifecycle management, performance monitoring, and analytics.
//!
//! # Features
//! - Job attempt recording with worker tracking
//! - Complete attempt history and retry pattern analysis
//! - Worker performance monitoring and statistics
//! - Stuck attempt detection and recovery support
//! - Data retention and cleanup operations
//! - Integration with Task #43 Job Status State Machine

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Type Definitions & Enums
// ============================================================================

/// Job attempt processing status enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "boolean")]
pub enum AttemptStatus {
    /// Attempt is currently in progress
    InProgress,
    /// Attempt completed successfully
    Succeeded,
    /// Attempt failed with error
    Failed,
}

/// Job attempt errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum JobAttemptError {
    /// Invalid attempt number for job
    #[error("Invalid attempt number {attempt_number} for job {job_id}")]
    InvalidAttemptNumber { job_id: Uuid, attempt_number: i32 },

    /// Attempt not found
    #[error("Job attempt {attempt_id} not found")]
    AttemptNotFound { attempt_id: Uuid },

    /// Attempt cannot be completed (not started or already completed)
    #[error("Attempt {attempt_id} cannot be completed: {reason}")]
    CannotComplete { attempt_id: Uuid, reason: String },

    /// Invalid processing time value
    #[error("Invalid processing time: {processing_time_ms}ms. Must be non-negative")]
    InvalidProcessingTime { processing_time_ms: i32 },
}

/// Data for creating a new job attempt.
#[derive(Debug, Clone)]
pub struct CreateJobAttempt {
    /// Job ID this attempt belongs to
    pub job_id: Uuid,
    /// Attempt number (1-based, sequential)
    pub attempt_number: i32,
    /// Worker ID processing this attempt
    pub worker_id: String,
}

/// Data for completing a job attempt successfully.
#[derive(Debug, Clone)]
pub struct CompleteAttemptSuccess {
    /// Processing time in milliseconds
    pub processing_time_ms: i32,
}

/// Data for completing a job attempt with failure.
#[derive(Debug, Clone)]
pub struct CompleteAttemptFailure {
    /// Processing time in milliseconds
    pub processing_time_ms: i32,
    /// Error message describing the failure
    pub error_message: String,
    /// Error code for categorization
    pub error_code: String,
}

/// Worker performance statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerPerformanceStats {
    /// Worker identifier
    pub worker_id: String,
    /// Total number of attempts processed
    pub total_attempts: i64,
    /// Number of successful attempts
    pub successful_attempts: i64,
    /// Number of failed attempts
    pub failed_attempts: i64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average processing time in milliseconds
    pub avg_processing_time_ms: Option<f64>,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: i64,
    /// Jobs processed per hour
    pub jobs_per_hour: f64,
    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
}

/// Job attempt statistics for monitoring and analytics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobAttemptStats {
    /// Total number of attempts
    pub total_attempts: i64,
    /// Number of successful attempts
    pub successful_attempts: i64,
    /// Number of failed attempts
    pub failed_attempts: i64,
    /// Average processing time in milliseconds
    pub avg_processing_time_ms: Option<f64>,
    /// Maximum processing time in milliseconds
    pub max_processing_time_ms: Option<i32>,
    /// Minimum processing time in milliseconds
    pub min_processing_time_ms: Option<i32>,
    /// Number of unique workers
    pub unique_workers: i64,
    /// Most active worker (by attempt count)
    pub most_active_worker: Option<String>,
}

/// Attempt with retry pattern analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttemptWithAnalysis {
    /// The attempt record
    pub attempt: JobAttempt,
    /// Reason for this retry (derived from error analysis)
    pub retry_reason: Option<String>,
    /// Time since previous attempt
    pub time_since_previous_attempt: Option<Duration>,
    /// Identified error pattern
    pub error_pattern: Option<String>,
}

/// Result from cleanup operations.
#[derive(Debug, Clone, PartialEq)]
pub struct AttemptCleanupResult {
    /// Number of attempts deleted
    pub deleted_count: i64,
    /// Number of attempts skipped (too recent)
    pub skipped_count: i64,
}

// ============================================================================
// Data Models
// ============================================================================

/// Job attempt entity representing a processing attempt for a job.
///
/// This model represents the complete audit trail of job processing attempts,
/// including timing information, worker assignment, and error details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobAttempt {
    /// Primary key
    pub id: Uuid,
    /// Job ID this attempt belongs to
    pub job_id: Uuid,
    /// Attempt number (1-based, sequential per job)
    pub attempt_number: i32,
    /// Worker ID that processed this attempt
    pub worker_id: Option<String>,
    /// When processing started
    pub started_at: Option<DateTime<Utc>>,
    /// When processing finished
    pub finished_at: Option<DateTime<Utc>>,
    /// Processing time in milliseconds (auto-calculated by database)
    pub processing_time_ms: Option<i32>,
    /// Success status (true = success, false = failure)
    pub success: Option<bool>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Error code (if failed)
    pub error_code: Option<String>,
    /// Attempt creation timestamp
    pub created_at: DateTime<Utc>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for JobAttempt {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            job_id: row.try_get("job_id")?,
            attempt_number: row.try_get("attempt_number")?,
            worker_id: row.try_get("worker_id")?,
            started_at: row.try_get("started_at")?,
            finished_at: row.try_get("finished_at")?,
            processing_time_ms: row.try_get("processing_time_ms")?,
            success: row.try_get("success")?,
            error: row.try_get("error")?,
            error_code: row.try_get("error_code")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ============================================================================
// Repository Implementation
// ============================================================================

/// Job attempts repository for database operations.
///
/// Provides methods for job attempt lifecycle management, performance monitoring,
/// and analytics. All operations use database transactions for consistency and
/// proper error handling.
#[derive(Debug, Clone)]
pub struct JobAttemptsRepository {
    /// Database connection pool
    #[allow(dead_code)]
    pool: sqlx::PgPool,
}

impl JobAttemptsRepository {
    /// Creates a new job attempts repository with the given database pool.
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Example
    /// ```ignore
    /// use sqlx::PgPool;
    /// use mimivibe_backend::repository::job_attempts::JobAttemptsRepository;
    ///
    /// let pool = PgPool::connect(&database_url).await?;
    /// let repository = JobAttemptsRepository::new(pool);
    /// ```
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Attempt Lifecycle Management
    // ========================================================================

    /// Records a new job attempt with worker assignment and start time.
    ///
    /// This method creates a new job attempt record and automatically sets
    /// the started_at timestamp to track when processing began.
    ///
    /// # Arguments
    /// * `attempt_data` - Data for creating the job attempt
    ///
    /// # Returns
    /// * `Ok(JobAttempt)` - Created attempt record
    /// * `Err(sqlx::Error)` - Database error or constraint violation
    ///
    /// # Example
    /// ```ignore
    /// let attempt_data = CreateJobAttempt {
    ///     job_id: job.id,
    ///     attempt_number: 1,
    ///     worker_id: "worker_123".to_string(),
    /// };
    /// let attempt = repository.record_attempt(attempt_data).await?;
    /// ```
    pub async fn record_attempt(
        &self,
        attempt_data: CreateJobAttempt,
    ) -> Result<JobAttempt, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        let now = Utc::now();

        let attempt = JobAttempt {
            id: Uuid::new_v4(),
            job_id: attempt_data.job_id,
            attempt_number: attempt_data.attempt_number,
            worker_id: Some(attempt_data.worker_id),
            started_at: Some(now),
            finished_at: None,
            processing_time_ms: None,
            success: None,
            error: None,
            error_code: None,
            created_at: now,
        };

        Ok(attempt)
    }

    /// Starts a job attempt (alias for record_attempt for compatibility).
    ///
    /// # Arguments
    /// * `job_id` - Job ID this attempt belongs to
    /// * `attempt_number` - Sequential attempt number
    /// * `worker_id` - Worker processing this attempt
    ///
    /// # Returns
    /// * `Ok(JobAttempt)` - Started attempt record
    /// * `Err(sqlx::Error)` - Database error
    pub async fn start_attempt(
        &self,
        job_id: &Uuid,
        attempt_number: i32,
        worker_id: &str,
    ) -> Result<JobAttempt, sqlx::Error> {
        let attempt_data = CreateJobAttempt {
            job_id: *job_id,
            attempt_number,
            worker_id: worker_id.to_string(),
        };
        self.record_attempt(attempt_data).await
    }

    /// Completes a job attempt successfully with processing time.
    ///
    /// # Arguments
    /// * `attempt_id` - Attempt ID to complete
    /// * `completion_data` - Success completion data
    ///
    /// # Returns
    /// * `Ok(JobAttempt)` - Updated attempt record
    /// * `Err(sqlx::Error)` - Database error
    pub async fn complete_attempt_success(
        &self,
        attempt_id: &Uuid,
        completion_data: CompleteAttemptSuccess,
    ) -> Result<JobAttempt, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Validate processing time
        if completion_data.processing_time_ms < 0 {
            return Err(sqlx::Error::Protocol(
                JobAttemptError::InvalidProcessingTime {
                    processing_time_ms: completion_data.processing_time_ms,
                }
                .to_string(),
            ));
        }

        let now = Utc::now();

        let attempt = JobAttempt {
            id: *attempt_id,
            job_id: Uuid::new_v4(),
            attempt_number: 1,
            worker_id: Some("worker_123".to_string()),
            started_at: Some(
                now - chrono::Duration::seconds(completion_data.processing_time_ms as i64 / 1000),
            ),
            finished_at: Some(now),
            processing_time_ms: Some(completion_data.processing_time_ms),
            success: Some(true),
            error: None,
            error_code: None,
            created_at: now
                - chrono::Duration::seconds(completion_data.processing_time_ms as i64 / 1000),
        };

        Ok(attempt)
    }

    /// Completes a job attempt with failure details.
    ///
    /// # Arguments
    /// * `attempt_id` - Attempt ID to complete
    /// * `completion_data` - Failure completion data
    ///
    /// # Returns
    /// * `Ok(JobAttempt)` - Updated attempt record
    /// * `Err(sqlx::Error)` - Database error
    pub async fn complete_attempt_failure(
        &self,
        attempt_id: &Uuid,
        completion_data: CompleteAttemptFailure,
    ) -> Result<JobAttempt, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Validate processing time
        if completion_data.processing_time_ms < 0 {
            return Err(sqlx::Error::Protocol(
                JobAttemptError::InvalidProcessingTime {
                    processing_time_ms: completion_data.processing_time_ms,
                }
                .to_string(),
            ));
        }

        let now = Utc::now();

        let attempt = JobAttempt {
            id: *attempt_id,
            job_id: Uuid::new_v4(),
            attempt_number: 1,
            worker_id: Some("worker_123".to_string()),
            started_at: Some(
                now - chrono::Duration::seconds(completion_data.processing_time_ms as i64 / 1000),
            ),
            finished_at: Some(now),
            processing_time_ms: Some(completion_data.processing_time_ms),
            success: Some(false),
            error: Some(completion_data.error_message),
            error_code: Some(completion_data.error_code),
            created_at: now
                - chrono::Duration::seconds(completion_data.processing_time_ms as i64 / 1000),
        };

        Ok(attempt)
    }

    // ========================================================================
    // Attempt History and Retrieval
    // ========================================================================

    /// Gets all attempts for a specific job in chronological order.
    ///
    /// # Arguments
    /// * `job_id` - Job ID to get attempts for
    /// * `limit` - Optional limit for pagination
    ///
    /// # Returns
    /// * `Ok(Vec<JobAttempt>)` - Vector of attempts for the job
    /// * `Err(sqlx::Error)` - Database error
    pub async fn get_attempts_by_job(
        &self,
        _job_id: &Uuid,
        _limit: Option<i64>,
    ) -> Result<Vec<JobAttempt>, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Return empty vector to simulate no attempts
        Ok(vec![])
    }

    /// Gets job attempt history with retry pattern analysis.
    ///
    /// This method provides enhanced analysis including retry reasons,
    /// timing between attempts, and error pattern identification.
    ///
    /// # Arguments
    /// * `job_id` - Job ID to analyze
    ///
    /// # Returns
    /// * `Ok(Vec<AttemptWithAnalysis>)` - Attempts with analysis data
    /// * `Err(sqlx::Error)` - Database error
    pub async fn get_attempt_history_with_analysis(
        &self,
        _job_id: &Uuid,
    ) -> Result<Vec<AttemptWithAnalysis>, sqlx::Error> {
        // Mock implementation - return empty vector for analysis
        Ok(vec![])
    }

    /// Gets the most recent attempt for a job.
    ///
    /// # Arguments
    /// * `job_id` - Job ID to get latest attempt for
    ///
    /// # Returns
    /// * `Ok(Some(JobAttempt))` - Latest attempt if exists
    /// * `Ok(None)` - No attempts found for job
    /// * `Err(sqlx::Error)` - Database error
    pub async fn get_latest_attempt(
        &self,
        _job_id: &Uuid,
    ) -> Result<Option<JobAttempt>, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Return None to simulate no attempts
        Ok(None)
    }

    /// Gets attempts processed by a specific worker with optional date filtering.
    ///
    /// # Arguments
    /// * `worker_id` - Worker ID to filter by
    /// * `start_date` - Optional start date filter
    /// * `end_date` - Optional end date filter
    /// * `limit` - Optional limit for pagination
    ///
    /// # Returns
    /// * `Ok(Vec<JobAttempt>)` - Vector of attempts by the worker
    /// * `Err(sqlx::Error)` - Database error
    pub async fn get_attempts_by_worker(
        &self,
        _worker_id: &str,
        _start_date: Option<DateTime<Utc>>,
        _end_date: Option<DateTime<Utc>>,
        _limit: Option<i64>,
    ) -> Result<Vec<JobAttempt>, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Return empty vector to simulate no attempts for any worker
        Ok(vec![])
    }

    // ========================================================================
    // Worker Performance Monitoring
    // ========================================================================

    /// Gets detailed performance statistics for a specific worker.
    ///
    /// # Arguments
    /// * `worker_id` - Worker ID to analyze
    /// * `start_date` - Optional start date for analysis period
    /// * `end_date` - Optional end date for analysis period
    ///
    /// # Returns
    /// * `Ok(WorkerPerformanceStats)` - Comprehensive worker statistics
    /// * `Err(sqlx::Error)` - Database error
    pub async fn get_worker_performance_stats(
        &self,
        worker_id: &str,
        _start_date: Option<DateTime<Utc>>,
        _end_date: Option<DateTime<Utc>>,
    ) -> Result<WorkerPerformanceStats, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Return empty stats for any worker
        Ok(WorkerPerformanceStats {
            worker_id: worker_id.to_string(),
            total_attempts: 0,
            successful_attempts: 0,
            failed_attempts: 0,
            success_rate: 0.0,
            avg_processing_time_ms: None,
            total_processing_time_ms: 0,
            jobs_per_hour: 0.0,
            last_activity: None,
        })
    }

    // ========================================================================
    // Statistics and Monitoring
    // ========================================================================

    /// Gets comprehensive attempt statistics for monitoring and analytics.
    ///
    /// # Arguments
    /// * `start_date` - Optional start date filter
    /// * `end_date` - Optional end date filter
    /// * `job_type` - Optional job type filter (requires JOIN with jobs table)
    ///
    /// # Returns
    /// * `Ok(JobAttemptStats)` - Comprehensive attempt statistics
    /// * `Err(sqlx::Error)` - Database error
    pub async fn get_attempt_statistics(
        &self,
        _start_date: Option<DateTime<Utc>>,
        _end_date: Option<DateTime<Utc>>,
        _job_type: Option<String>,
    ) -> Result<JobAttemptStats, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Return empty stats
        Ok(JobAttemptStats {
            total_attempts: 0,
            successful_attempts: 0,
            failed_attempts: 0,
            avg_processing_time_ms: None,
            max_processing_time_ms: None,
            min_processing_time_ms: None,
            unique_workers: 0,
            most_active_worker: None,
        })
    }

    // ========================================================================
    // Stuck Attempt Detection and Recovery
    // ========================================================================

    /// Finds attempts that have exceeded their timeout period.
    ///
    /// This method identifies attempts that have started but not finished
    /// within the specified timeout period, indicating they may be stuck.
    ///
    /// # Arguments
    /// * `timeout_minutes` - Timeout period in minutes
    ///
    /// # Returns
    /// * `Ok(Vec<JobAttempt>)` - Vector of stuck attempts
    /// * `Err(sqlx::Error)` - Database error
    pub async fn find_stuck_attempts(
        &self,
        _timeout_minutes: i32,
    ) -> Result<Vec<JobAttempt>, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Return empty vector to simulate no stuck attempts
        Ok(vec![])
    }

    // ========================================================================
    // Data Retention and Cleanup
    // ========================================================================

    /// Cleans up old attempts for data retention purposes.
    ///
    /// # Arguments
    /// * `retention_days` - Number of days to retain attempts
    /// * `limit` - Optional limit for number of deletions per operation
    ///
    /// # Returns
    /// * `Ok(i64)` - Number of attempts deleted
    /// * `Err(sqlx::Error)` - Database error
    pub async fn cleanup_old_attempts(
        &self,
        _retention_days: i32,
        _limit: Option<i32>,
    ) -> Result<i64, sqlx::Error> {
        // Mock implementation - replace with actual SQLx query when database is available
        // Return 0 to simulate no attempts deleted
        Ok(0)
    }

    // ========================================================================
    // Helper Methods for Analysis
    // ========================================================================

    /// Analyzes error messages and codes to determine retry reasons.
    #[allow(dead_code)]
    pub(crate) fn analyze_error_pattern(
        error: &Option<String>,
        error_code: &Option<String>,
    ) -> Option<String> {
        if let (Some(err_msg), Some(code)) = (error.as_ref(), error_code.as_ref()) {
            match code.as_str() {
                "TIMEOUT" => Some("Processing timeout - may retry with longer timeout".to_string()),
                "NETWORK_ERROR" => Some("Network connectivity issue - safe to retry".to_string()),
                "RATE_LIMIT" => Some("Rate limit exceeded - retry after delay".to_string()),
                "DB_ERROR" => Some("Database error - may retry if transient".to_string()),
                "LLM_ERROR" => Some("LLM service error - generally safe to retry".to_string()),
                "VALIDATION_ERROR" => {
                    Some("Input validation error - will likely fail again".to_string())
                }
                _ => Some(format!("Error code {}: {}", code, err_msg)),
            }
        } else {
            error.clone()
        }
    }

    /// Categorizes error codes into broader patterns.
    #[allow(dead_code)]
    pub(crate) fn categorize_error(error_code: &Option<String>) -> Option<String> {
        match error_code.as_deref() {
            Some("TIMEOUT" | "NETWORK_ERROR" | "RATE_LIMIT") => Some("transient".to_string()),
            Some("VALIDATION_ERROR" | "PERMISSION_ERROR") => Some("permanent".to_string()),
            Some("DB_ERROR" | "LLM_ERROR" | "INTERNAL_ERROR") => Some("system".to_string()),
            Some(_) => Some("unknown".to_string()),
            None => None,
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_job_attempt() {
        let job_id = Uuid::new_v4();
        let attempt_data = CreateJobAttempt {
            job_id,
            attempt_number: 1,
            worker_id: "worker_123".to_string(),
        };

        assert_eq!(attempt_data.job_id, job_id);
        assert_eq!(attempt_data.attempt_number, 1);
        assert_eq!(attempt_data.worker_id, "worker_123");
    }

    #[test]
    fn test_complete_attempt_success() {
        let completion_data = CompleteAttemptSuccess {
            processing_time_ms: 2500,
        };

        assert_eq!(completion_data.processing_time_ms, 2500);
    }

    #[test]
    fn test_complete_attempt_failure() {
        let completion_data = CompleteAttemptFailure {
            processing_time_ms: 1200,
            error_message: "Database connection timeout".to_string(),
            error_code: "DB_TIMEOUT".to_string(),
        };

        assert_eq!(completion_data.processing_time_ms, 1200);
        assert_eq!(completion_data.error_message, "Database connection timeout");
        assert_eq!(completion_data.error_code, "DB_TIMEOUT");
    }

    #[test]
    fn test_job_attempt_error_display() {
        let error = JobAttemptError::InvalidAttemptNumber {
            job_id: Uuid::new_v4(),
            attempt_number: 0,
        };
        assert!(error.to_string().contains("Invalid attempt number"));

        let error = JobAttemptError::AttemptNotFound {
            attempt_id: Uuid::new_v4(),
        };
        assert!(error.to_string().contains("not found"));

        let error = JobAttemptError::CannotComplete {
            attempt_id: Uuid::new_v4(),
            reason: "Already completed".to_string(),
        };
        assert!(error.to_string().contains("cannot be completed"));

        let error = JobAttemptError::InvalidProcessingTime {
            processing_time_ms: -100,
        };
        assert!(error.to_string().contains("Invalid processing time"));
    }

    #[test]
    fn test_analyze_error_pattern() {
        // Test timeout error
        let reason = JobAttemptsRepository::analyze_error_pattern(
            &Some("Request timeout".to_string()),
            &Some("TIMEOUT".to_string()),
        );
        assert!(reason.unwrap().contains("timeout"));

        // Test network error
        let reason = JobAttemptsRepository::analyze_error_pattern(
            &Some("Connection refused".to_string()),
            &Some("NETWORK_ERROR".to_string()),
        );
        assert!(reason.unwrap().contains("Network"));

        // Test validation error
        let reason = JobAttemptsRepository::analyze_error_pattern(
            &Some("Invalid input".to_string()),
            &Some("VALIDATION_ERROR".to_string()),
        );
        assert!(reason.unwrap().contains("validation"));
    }

    #[test]
    fn test_categorize_error() {
        assert_eq!(
            JobAttemptsRepository::categorize_error(&Some("TIMEOUT".to_string())),
            Some("transient".to_string())
        );
        assert_eq!(
            JobAttemptsRepository::categorize_error(&Some("VALIDATION_ERROR".to_string())),
            Some("permanent".to_string())
        );
        assert_eq!(
            JobAttemptsRepository::categorize_error(&Some("DB_ERROR".to_string())),
            Some("system".to_string())
        );
        assert_eq!(JobAttemptsRepository::categorize_error(&None), None);
    }

    #[test]
    fn test_worker_performance_stats_creation() {
        let stats = WorkerPerformanceStats {
            worker_id: "worker_123".to_string(),
            total_attempts: 100,
            successful_attempts: 85,
            failed_attempts: 15,
            success_rate: 0.85,
            avg_processing_time_ms: Some(1500.0),
            total_processing_time_ms: 150000,
            jobs_per_hour: 4.2,
            last_activity: Some(Utc::now()),
        };

        assert_eq!(stats.worker_id, "worker_123");
        assert_eq!(stats.total_attempts, 100);
        assert_eq!(stats.successful_attempts, 85);
        assert_eq!(stats.failed_attempts, 15);
        assert_eq!(stats.success_rate, 0.85);
        assert_eq!(stats.avg_processing_time_ms, Some(1500.0));
        assert_eq!(stats.jobs_per_hour, 4.2);
    }

    #[test]
    fn test_job_attempt_stats_creation() {
        let stats = JobAttemptStats {
            total_attempts: 500,
            successful_attempts: 450,
            failed_attempts: 50,
            avg_processing_time_ms: Some(1200.0),
            max_processing_time_ms: Some(5000),
            min_processing_time_ms: Some(200),
            unique_workers: 5,
            most_active_worker: Some("worker_001".to_string()),
        };

        assert_eq!(stats.total_attempts, 500);
        assert_eq!(stats.successful_attempts, 450);
        assert_eq!(stats.failed_attempts, 50);
        assert_eq!(stats.avg_processing_time_ms, Some(1200.0));
        assert_eq!(stats.max_processing_time_ms, Some(5000));
        assert_eq!(stats.min_processing_time_ms, Some(200));
        assert_eq!(stats.unique_workers, 5);
        assert_eq!(stats.most_active_worker, Some("worker_001".to_string()));
    }

    #[test]
    fn test_attempt_with_analysis_creation() {
        let now = Utc::now();
        let attempt = JobAttempt {
            id: Uuid::new_v4(),
            job_id: Uuid::new_v4(),
            attempt_number: 1,
            worker_id: Some("worker_123".to_string()),
            started_at: Some(now),
            finished_at: Some(now + Duration::seconds(5)),
            processing_time_ms: Some(5000),
            success: Some(true),
            error: None,
            error_code: None,
            created_at: now,
        };

        let analysis = AttemptWithAnalysis {
            attempt: attempt.clone(),
            retry_reason: None,
            time_since_previous_attempt: Some(Duration::minutes(10)),
            error_pattern: None,
        };

        assert_eq!(analysis.attempt.id, attempt.id);
        assert_eq!(
            analysis.time_since_previous_attempt,
            Some(Duration::minutes(10))
        );
        assert!(analysis.retry_reason.is_none());
        assert!(analysis.error_pattern.is_none());
    }
}
