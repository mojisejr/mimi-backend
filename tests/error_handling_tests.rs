//! Test-First Development: Error Handling & User-Friendly Response Mapping Tests
//!
//! RED phase: These tests will FAIL initially until error handling module is implemented
//!
//! Tests cover:
//! - Error code to user message mapping
//! - Backend log context generation
//! - Error classification and severity levels
//! - Integration with queue and worker systems
//! - Display and Error trait implementations

use mimivibe_backend::error::{
    classify_error, create_error_response, ErrorContext, ErrorExt, ErrorSeverity, QueueError,
    WorkerError,
};
use std::error::Error;
use std::time::Duration;

// Test for error code to user message mapping
#[test]
fn test_queue_error_mapping() {
    let queue_error = QueueError::ConnectionFailed("Redis connection timeout".to_string());

    // Test error code generation
    let error_code = queue_error.error_code();
    assert_eq!(error_code, "QUEUE_CONNECTION_FAILED");

    // Test user-friendly message
    let user_message = queue_error.user_message();
    assert!(user_message.contains("Service temporarily unavailable"));
    assert!(!user_message.contains("Redis"));

    // Test backend log context
    let log_context = queue_error.log_context();
    assert!(log_context.contains("Redis connection timeout"));
    assert!(log_context.contains("error"));
}

#[test]
fn test_worker_error_mapping() {
    let worker_error = WorkerError::JobProcessingFailed {
        job_id: "job-123".to_string(),
        attempts: 3,
        reason: "Gemini API timeout".to_string(),
    };

    // Test error code generation
    let error_code = worker_error.error_code();
    assert_eq!(error_code, "WORKER_JOB_PROCESSING_FAILED");

    // Test user-friendly message
    let user_message = worker_error.user_message();
    assert!(user_message.contains("processing"));
    assert!(!user_message.contains("Gemini"));
    assert!(!user_message.contains("job-123"));

    // Test backend log context
    let log_context = worker_error.log_context();
    assert!(log_context.contains("job-123"));
    assert!(log_context.contains("Gemini API timeout"));
    assert!(log_context.contains("attempts=3"));
}

#[test]
fn test_error_severity_classification() {
    // Test timeout error severity
    let timeout_error = WorkerError::JobTimeout {
        job_id: "job-456".to_string(),
        timeout: Duration::from_secs(30),
    };
    assert_eq!(timeout_error.severity(), ErrorSeverity::Warning);

    // Test network error severity
    let network_error = QueueError::NetworkError("Connection refused".to_string());
    assert_eq!(network_error.severity(), ErrorSeverity::Error);

    // Test validation error severity
    let validation_error = QueueError::InvalidPayload("Invalid card count".to_string());
    assert_eq!(validation_error.severity(), ErrorSeverity::Warning);
}

#[test]
fn test_error_display_and_debug() {
    let error = QueueError::QueueFull("Maximum capacity reached".to_string());

    // Test Display trait
    let display_str = format!("{}", error);
    assert!(display_str.contains("QUEUE_QUEUE_FULL"));
    assert!(display_str.contains("high demand"));

    // Test Debug trait
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("QueueFull"));
    assert!(debug_str.contains("Maximum capacity reached"));
}

#[test]
fn test_error_response_creation() {
    let queue_error = QueueError::DequeueFailed("Queue locked".to_string());
    let response = create_error_response(&queue_error);

    assert_eq!(response.error_code, "QUEUE_DEQUEUE_FAILED");
    assert!(response.user_message.contains("Please try again"));
    assert_eq!(response.severity, "warning");
    assert!(!response.timestamp.is_empty());
}

#[test]
fn test_error_classification_function() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Config file not found");
    let classified = classify_error(&io_error);

    // Should be classified as some kind of system error
    matches!(classified, QueueError::InternalError(_));
}

#[test]
fn test_error_chaining() {
    let _root_cause =
        std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Network unreachable");
    let worker_error = WorkerError::JobProcessingFailed {
        job_id: "job-789".to_string(),
        attempts: 1,
        reason: "API call failed".to_string(),
    };

    // Test error source chaining
    let error_source = worker_error.source();
    assert!(error_source.is_none()); // WorkerError is the root error in this case

    // Test that the error can be converted to Box<dyn Error>
    let boxed_error: Box<dyn Error> = Box::new(worker_error);
    assert!(boxed_error
        .to_string()
        .contains("WORKER_JOB_PROCESSING_FAILED"));
}

// Integration tests for queue and worker error scenarios
#[tokio::test]
async fn test_queue_integration_error_handling() {
    // This will test integration with actual queue implementations
    // For now, test the error handling in isolation

    let enqueue_error = QueueError::EnqueueFailed {
        payload_id: "payload-123".to_string(),
        reason: "Queue size limit exceeded".to_string(),
    };

    let response = create_error_response(&enqueue_error);
    assert!(response.user_message.contains("Please try again later"));
    assert_eq!(response.error_code, "QUEUE_ENQUEUE_FAILED");
}

#[tokio::test]
async fn test_worker_retry_error_handling() {
    // Test error scenarios that would trigger retries
    let retry_error = WorkerError::RetryableError {
        job_id: "job-retry-123".to_string(),
        attempts: 2,
        next_retry_in: Duration::from_secs(5),
        reason: "Temporary API rate limit".to_string(),
    };

    assert_eq!(retry_error.severity(), ErrorSeverity::Warning);
    assert!(retry_error.log_context().contains("next_retry_in=5s"));

    let response = create_error_response(&retry_error);
    assert!(response.user_message.contains("processed"));
}

#[test]
fn test_all_error_variants_coverage() {
    // Ensure all error variants are covered and work correctly

    // Queue errors
    let queue_errors = vec![
        QueueError::ConnectionFailed("DB connection lost".to_string()),
        QueueError::NetworkError("DNS resolution failed".to_string()),
        QueueError::TimeoutError("Operation timed out".to_string()),
        QueueError::EnqueueFailed {
            payload_id: "test".to_string(),
            reason: "Full".to_string(),
        },
        QueueError::DequeueFailed("Empty queue".to_string()),
        QueueError::AckFailed {
            job_id: "job-1".to_string(),
            reason: "Job not found".to_string(),
        },
        QueueError::NackFailed {
            job_id: "job-2".to_string(),
            reason: "Invalid state".to_string(),
        },
        QueueError::QueueFull("Capacity exceeded".to_string()),
        QueueError::InvalidPayload("Bad format".to_string()),
        QueueError::InternalError("Panic occurred".to_string()),
    ];

    for error in queue_errors {
        // Test that all error variants implement required functionality
        let _code = error.error_code();
        let _user_msg = error.user_message();
        let _log_ctx = error.log_context();
        let _severity = error.severity();
        let _display = format!("{}", error);
        let _response = create_error_response(&error);
    }

    // Worker errors
    let worker_errors = vec![
        WorkerError::JobProcessingFailed {
            job_id: "job-1".to_string(),
            attempts: 1,
            reason: "Failed".to_string(),
        },
        WorkerError::JobTimeout {
            job_id: "job-2".to_string(),
            timeout: Duration::from_secs(30),
        },
        WorkerError::RetryableError {
            job_id: "job-3".to_string(),
            attempts: 2,
            next_retry_in: Duration::from_secs(10),
            reason: "Temp fail".to_string(),
        },
        WorkerError::MaxRetriesExceeded {
            job_id: "job-4".to_string(),
            total_attempts: 5,
        },
        WorkerError::InvalidJobData {
            job_id: "job-5".to_string(),
            validation_errors: vec!["Bad field".to_string()],
        },
        WorkerError::InternalError("Worker panic".to_string()),
    ];

    for error in worker_errors {
        // Test that all error variants implement required functionality
        let _code = error.error_code();
        let _user_msg = error.user_message();
        let _log_ctx = error.log_context();
        let _severity = error.severity();
        let _display = format!("{}", error);
        let _response = create_error_response(&error);
    }
}

#[test]
fn test_error_context_struct() {
    let context = ErrorContext {
        error_code: "TEST_ERROR".to_string(),
        severity: ErrorSeverity::Error,
        timestamp: chrono::Utc::now(),
        job_id: Some("job-123".to_string()),
        user_id: None,
        trace_id: Some("trace-456".to_string()),
        metadata: std::collections::HashMap::from([
            ("attempts".to_string(), "3".to_string()),
            ("component".to_string(), "queue".to_string()),
        ]),
    };

    // Test that ErrorContext can be serialized/deserialized
    let json = serde_json::to_string(&context).unwrap();
    let deserialized: ErrorContext = serde_json::from_str(&json).unwrap();

    assert_eq!(context.error_code, deserialized.error_code);
    assert_eq!(context.severity, deserialized.severity);
    assert_eq!(context.job_id, deserialized.job_id);
}

#[test]
fn test_log_context_formatting() {
    let error = QueueError::EnqueueFailed {
        payload_id: "payload-789".to_string(),
        reason: "Queue at maximum capacity".to_string(),
    };

    let log_context = error.log_context();

    // Verify log context contains expected fields
    assert!(log_context.contains("error_code=QUEUE_ENQUEUE_FAILED"));
    assert!(log_context.contains("severity=warning")); // severity is explicitly formatted
    assert!(log_context.contains("payload_id=\"payload-789\""));
    assert!(log_context.contains("reason=\"Queue at maximum capacity\""));
    assert!(log_context.contains("timestamp="));

    // Verify it's properly formatted for logging
    assert!(log_context.contains("]"));
}
