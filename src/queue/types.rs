//! Type definitions for queue jobs
//!
//! Defines the data structures used for job payloads, metadata, and job types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job payload structure
///
/// Contains all the information needed to process a tarot reading job.
/// This struct matches the PRD schema and is serialized/deserialized
/// when jobs are enqueued and dequeued.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPayload {
    /// Unique identifier for this job
    pub job_id: String,

    /// User ID requesting the tarot reading
    pub user_id: Uuid,

    /// The question to be answered by the tarot reading
    pub question: String,

    /// Number of cards to draw (3 or 5)
    pub card_count: u32,

    /// Schema version for compatibility tracking
    pub schema_version: String,

    /// Prompt version identifier (e.g., "v2025-11-20-a")
    pub prompt_version: String,

    /// Optional deduplication key to prevent duplicate jobs
    pub dedupe_key: Option<String>,

    /// Optional trace ID for distributed tracing
    pub trace_id: Option<String>,

    /// Timestamp when the job was created
    pub created_at: DateTime<Utc>,

    /// Additional metadata as flexible JSON
    pub metadata: serde_json::Value,
}

/// Job metadata structure
///
/// Contains additional contextual information about the job.
/// This is typically stored in the `metadata` field of JobPayload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Locale/language code (e.g., "th" for Thai)
    pub locale: String,

    /// Source of the request ("mobile", "web", etc.)
    pub source: String,
}

/// Job type enumeration
///
/// Defines different types of jobs that can be processed.
/// This allows the queue system to handle multiple job types
/// in the future.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobType {
    /// Tarot reading job
    TarotReading,

    /// Notification job
    Notification,

    /// Maintenance/housekeeping job
    Maintenance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_payload_creation() {
        let payload = JobPayload {
            job_id: "test-123".to_string(),
            user_id: Uuid::new_v4(),
            question: "Test question".to_string(),
            card_count: 3,
            schema_version: "1".to_string(),
            prompt_version: "v2025-11-20-a".to_string(),
            dedupe_key: None,
            trace_id: None,
            created_at: Utc::now(),
            metadata: serde_json::json!({}),
        };

        assert_eq!(payload.job_id, "test-123");
        assert_eq!(payload.card_count, 3);
    }

    #[test]
    fn test_job_metadata_serialization() {
        let metadata = JobMetadata {
            locale: "th".to_string(),
            source: "mobile".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("th"));
        assert!(json.contains("mobile"));
    }

    #[test]
    fn test_job_type_variants() {
        let types = vec![
            JobType::TarotReading,
            JobType::Notification,
            JobType::Maintenance,
        ];

        for job_type in types {
            let json = serde_json::to_string(&job_type).unwrap();
            let deserialized: JobType = serde_json::from_str(&json).unwrap();
            assert_eq!(job_type, deserialized);
        }
    }
}
