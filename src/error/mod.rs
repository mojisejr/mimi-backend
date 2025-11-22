//! Error Handling & User-Friendly Response Mapping
//!
//! Provides comprehensive error handling for queue and worker systems with:
//! - Error code to user message mapping
//! - Backend log context generation
//! - Error classification and severity levels
//! - User-friendly responses without exposing internal details

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::time::Duration;

/// Error severity levels for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational message, not an error
    Info,
    /// Warning that doesn't stop operation
    Warning,
    /// Error that requires attention
    Error,
    /// Critical error that needs immediate attention
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "info"),
            ErrorSeverity::Warning => write!(f, "warning"),
            ErrorSeverity::Error => write!(f, "error"),
            ErrorSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Error context for logging and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error code identifier
    pub error_code: String,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Optional job identifier
    pub job_id: Option<String>,
    /// Optional user identifier
    pub user_id: Option<String>,
    /// Optional trace identifier for distributed tracing
    pub trace_id: Option<String>,
    /// Additional metadata for debugging
    pub metadata: HashMap<String, String>,
}

/// Queue-specific errors
#[derive(Debug, Clone)]
pub enum QueueError {
    /// Connection to queue backend failed
    ConnectionFailed(String),
    /// Network-related error
    NetworkError(String),
    /// Operation timed out
    TimeoutError(String),
    /// Failed to enqueue a job
    EnqueueFailed { payload_id: String, reason: String },
    /// Failed to dequeue a job
    DequeueFailed(String),
    /// Failed to acknowledge a job
    AckFailed { job_id: String, reason: String },
    /// Failed to negative acknowledge a job
    NackFailed { job_id: String, reason: String },
    /// Queue is at maximum capacity
    QueueFull(String),
    /// Invalid job payload
    InvalidPayload(String),
    /// Internal queue system error
    InternalError(String),
}

/// Worker-specific errors
#[derive(Debug, Clone)]
pub enum WorkerError {
    /// Job processing failed
    JobProcessingFailed {
        job_id: String,
        attempts: u32,
        reason: String,
    },
    /// Job processing timed out
    JobTimeout { job_id: String, timeout: Duration },
    /// Error that is retryable
    RetryableError {
        job_id: String,
        attempts: u32,
        next_retry_in: Duration,
        reason: String,
    },
    /// Maximum retry attempts exceeded
    MaxRetriesExceeded { job_id: String, total_attempts: u32 },
    /// Invalid job data
    InvalidJobData {
        job_id: String,
        validation_errors: Vec<String>,
    },
    /// Internal worker error
    InternalError(String),
}

/// User-friendly error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Machine-readable error code
    pub error_code: String,
    /// User-friendly error message
    pub user_message: String,
    /// Error severity level
    pub severity: String,
    /// When the error occurred
    pub timestamp: String,
    /// Optional request ID for tracking
    pub request_id: Option<String>,
}

// Trait implementations for error codes, user messages, and log context
pub trait ErrorExt {
    /// Get machine-readable error code
    fn error_code(&self) -> String;

    /// Get user-friendly error message (hides internal details)
    fn user_message(&self) -> String;

    /// Get backend log context with full details
    fn log_context(&self) -> String;

    /// Get error severity level
    fn severity(&self) -> ErrorSeverity;
}

impl ErrorExt for QueueError {
    fn error_code(&self) -> String {
        match self {
            QueueError::ConnectionFailed(_) => "QUEUE_CONNECTION_FAILED".to_string(),
            QueueError::NetworkError(_) => "QUEUE_NETWORK_ERROR".to_string(),
            QueueError::TimeoutError(_) => "QUEUE_TIMEOUT_ERROR".to_string(),
            QueueError::EnqueueFailed { .. } => "QUEUE_ENQUEUE_FAILED".to_string(),
            QueueError::DequeueFailed(_) => "QUEUE_DEQUEUE_FAILED".to_string(),
            QueueError::AckFailed { .. } => "QUEUE_ACK_FAILED".to_string(),
            QueueError::NackFailed { .. } => "QUEUE_NACK_FAILED".to_string(),
            QueueError::QueueFull(_) => "QUEUE_QUEUE_FULL".to_string(),
            QueueError::InvalidPayload(_) => "QUEUE_INVALID_PAYLOAD".to_string(),
            QueueError::InternalError(_) => "QUEUE_INTERNAL_ERROR".to_string(),
        }
    }

    fn user_message(&self) -> String {
        match self {
            QueueError::ConnectionFailed(_) | QueueError::NetworkError(_) => {
                "Service temporarily unavailable. Please try again in a few moments.".to_string()
            }
            QueueError::TimeoutError(_) => "Request timed out. Please try again.".to_string(),
            QueueError::EnqueueFailed { .. } | QueueError::QueueFull(_) => {
                "Service is experiencing high demand. Please try again later.".to_string()
            }
            QueueError::DequeueFailed(_) => {
                "Unable to process request at this time. Please try again.".to_string()
            }
            QueueError::AckFailed { .. } | QueueError::NackFailed { .. } => {
                "Job processing encountered an issue. Please contact support if this persists."
                    .to_string()
            }
            QueueError::InvalidPayload(_) => {
                "Invalid request format. Please check your input and try again.".to_string()
            }
            QueueError::InternalError(_) => {
                "An unexpected error occurred. Please try again or contact support.".to_string()
            }
        }
    }

    fn log_context(&self) -> String {
        let timestamp = Utc::now().to_rfc3339();
        let error_code = self.error_code();
        let severity = self.severity();

        let mut context = format!(
            "[{}] error_code={} severity={} timestamp={}",
            error_code, error_code, severity, timestamp
        );

        // Add specific context based on error type
        match self {
            QueueError::ConnectionFailed(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
            QueueError::NetworkError(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
            QueueError::TimeoutError(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
            QueueError::EnqueueFailed { payload_id, reason } => {
                context.push_str(&format!(
                    " payload_id=\"{}\" reason=\"{}\"",
                    payload_id, reason
                ));
            }
            QueueError::DequeueFailed(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
            QueueError::AckFailed { job_id, reason } => {
                context.push_str(&format!(" job_id=\"{}\" reason=\"{}\"", job_id, reason));
            }
            QueueError::NackFailed { job_id, reason } => {
                context.push_str(&format!(" job_id=\"{}\" reason=\"{}\"", job_id, reason));
            }
            QueueError::QueueFull(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
            QueueError::InvalidPayload(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
            QueueError::InternalError(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
        }

        context
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            QueueError::ConnectionFailed(_)
            | QueueError::NetworkError(_)
            | QueueError::InternalError(_) => ErrorSeverity::Error,
            QueueError::TimeoutError(_)
            | QueueError::EnqueueFailed { .. }
            | QueueError::DequeueFailed(_)
            | QueueError::AckFailed { .. }
            | QueueError::NackFailed { .. }
            | QueueError::QueueFull(_) => ErrorSeverity::Warning,
            QueueError::InvalidPayload(_) => ErrorSeverity::Warning,
        }
    }
}

impl ErrorExt for WorkerError {
    fn error_code(&self) -> String {
        match self {
            WorkerError::JobProcessingFailed { .. } => "WORKER_JOB_PROCESSING_FAILED".to_string(),
            WorkerError::JobTimeout { .. } => "WORKER_JOB_TIMEOUT".to_string(),
            WorkerError::RetryableError { .. } => "WORKER_RETRYABLE_ERROR".to_string(),
            WorkerError::MaxRetriesExceeded { .. } => "WORKER_MAX_RETRIES_EXCEEDED".to_string(),
            WorkerError::InvalidJobData { .. } => "WORKER_INVALID_JOB_DATA".to_string(),
            WorkerError::InternalError(_) => "WORKER_INTERNAL_ERROR".to_string(),
        }
    }

    fn user_message(&self) -> String {
        match self {
            WorkerError::JobProcessingFailed { attempts, .. } => {
                if *attempts > 1 {
                    format!("Job processing is taking longer than expected. Attempt {} of {}. Please be patient.", attempts, 3)
                } else {
                    "Your request is being processed. This may take a few moments.".to_string()
                }
            }
            WorkerError::JobTimeout { .. } => {
                "Request processing timed out. Please try again with a simpler query.".to_string()
            }
            WorkerError::RetryableError { .. } => {
                "Your request is still being processed. Please check back in a few moments."
                    .to_string()
            }
            WorkerError::MaxRetriesExceeded { .. } => {
                "Request processing failed after multiple attempts. Please try again later."
                    .to_string()
            }
            WorkerError::InvalidJobData { .. } => {
                "Invalid request format. Please check your input and try again.".to_string()
            }
            WorkerError::InternalError(_) => {
                "An unexpected error occurred during processing. Please try again.".to_string()
            }
        }
    }

    fn log_context(&self) -> String {
        let timestamp = Utc::now().to_rfc3339();
        let error_code = self.error_code();
        let severity = self.severity();

        let mut context = format!(
            "[{}] error_code={} severity={} timestamp={}",
            error_code, error_code, severity, timestamp
        );

        // Add specific context based on error type
        match self {
            WorkerError::JobProcessingFailed {
                job_id,
                attempts,
                reason,
            } => {
                context.push_str(&format!(
                    " job_id=\"{}\" attempts={} reason=\"{}\"",
                    job_id, attempts, reason
                ));
            }
            WorkerError::JobTimeout { job_id, timeout } => {
                context.push_str(&format!(
                    " job_id=\"{}\" timeout_seconds={}",
                    job_id,
                    timeout.as_secs()
                ));
            }
            WorkerError::RetryableError {
                job_id,
                attempts,
                next_retry_in,
                reason,
            } => {
                context.push_str(&format!(
                    " job_id=\"{}\" attempts={} next_retry_in={}s reason=\"{}\"",
                    job_id,
                    attempts,
                    next_retry_in.as_secs(),
                    reason
                ));
            }
            WorkerError::MaxRetriesExceeded {
                job_id,
                total_attempts,
            } => {
                context.push_str(&format!(
                    " job_id=\"{}\" total_attempts={}",
                    job_id, total_attempts
                ));
            }
            WorkerError::InvalidJobData {
                job_id,
                validation_errors,
            } => {
                context.push_str(&format!(
                    " job_id=\"{}\" validation_errors=[{}]",
                    job_id,
                    validation_errors.join(", ")
                ));
            }
            WorkerError::InternalError(reason) => {
                context.push_str(&format!(" reason=\"{}\"", reason));
            }
        }

        context
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            WorkerError::JobProcessingFailed { .. } | WorkerError::RetryableError { .. } => {
                ErrorSeverity::Warning
            }
            WorkerError::JobTimeout { .. } | WorkerError::InvalidJobData { .. } => {
                ErrorSeverity::Warning
            }
            WorkerError::MaxRetriesExceeded { .. } | WorkerError::InternalError(_) => {
                ErrorSeverity::Error
            }
        }
    }
}

// Display trait implementations
impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.user_message())
    }
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.user_message())
    }
}

// Error trait implementations
impl StdError for QueueError {}

impl StdError for WorkerError {}

/// Classify a generic error into our error types
pub fn classify_error(error: &(dyn StdError + Send + Sync)) -> QueueError {
    let error_msg = error.to_string();

    // Simple classification based on error message content
    if error_msg.contains("connection") || error_msg.contains("connect") {
        QueueError::ConnectionFailed(error_msg)
    } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
        QueueError::TimeoutError(error_msg)
    } else if error_msg.contains("network") || error_msg.contains("dns") {
        QueueError::NetworkError(error_msg)
    } else if error_msg.contains("full") || error_msg.contains("capacity") {
        QueueError::QueueFull(error_msg)
    } else {
        QueueError::InternalError(error_msg)
    }
}

/// Create a user-friendly error response from any error that implements ErrorExt
pub fn create_error_response<E: ErrorExt>(error: &E) -> ErrorResponse {
    ErrorResponse {
        error_code: error.error_code(),
        user_message: error.user_message(),
        severity: error.severity().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        request_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_display() {
        assert_eq!(ErrorSeverity::Info.to_string(), "info");
        assert_eq!(ErrorSeverity::Warning.to_string(), "warning");
        assert_eq!(ErrorSeverity::Error.to_string(), "error");
        assert_eq!(ErrorSeverity::Critical.to_string(), "critical");
    }

    #[test]
    fn test_queue_error_basic() {
        let error = QueueError::ConnectionFailed("Test connection failed".to_string());
        assert_eq!(error.error_code(), "QUEUE_CONNECTION_FAILED");
        assert!(error.user_message().contains("temporarily unavailable"));
        assert_eq!(error.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_worker_error_basic() {
        let error = WorkerError::JobProcessingFailed {
            job_id: "job-123".to_string(),
            attempts: 2,
            reason: "Test failure".to_string(),
        };
        assert_eq!(error.error_code(), "WORKER_JOB_PROCESSING_FAILED");
        assert!(error.user_message().contains("taking longer"));
        assert_eq!(error.severity(), ErrorSeverity::Warning);
    }
}
