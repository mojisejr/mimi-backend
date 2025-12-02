//! Tarot Reading API Request/Response Models
//!
//! Type-safe models for tarot reading API endpoints with validation.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Error types for tarot request validation
#[derive(Debug, Error)]
pub enum TarotRequestError {
    #[error("Question must be between 5 and 100 characters (got {length})")]
    InvalidQuestionLength { length: usize },

    #[error("Question cannot be empty")]
    EmptyQuestion,

    #[error("Question contains only whitespace")]
    WhitespaceOnlyQuestion,

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid request format: {message}")]
    InvalidFormat { message: String },
}

/// Tarot reading request from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarotRequest {
    /// The user's question for tarot reading
    /// Must be between 5 and 100 characters
    pub question: String,

    /// Optional user identifier for rate limiting
    /// If not provided, will use IP address
    pub user_id: Option<String>,
}

impl TarotRequest {
    /// Validate the tarot request according to business rules
    pub fn validate(&self) -> Result<(), TarotRequestError> {
        // Check if question is empty
        if self.question.is_empty() {
            return Err(TarotRequestError::EmptyQuestion);
        }

        // Check if question contains only whitespace
        if self.question.trim().is_empty() {
            return Err(TarotRequestError::WhitespaceOnlyQuestion);
        }

        let trimmed_question = self.question.trim();

        // Check length constraints (5-100 characters)
        let length = trimmed_question.len();
        if !(5..=100).contains(&length) {
            return Err(TarotRequestError::InvalidQuestionLength { length });
        }

        Ok(())
    }

    /// Get the trimmed question
    pub fn get_trimmed_question(&self) -> String {
        self.question.trim().to_string()
    }

  
    /// Get user identifier for rate limiting
    pub fn get_rate_limit_key(&self, fallback_ip: Option<&str>) -> String {
        if let Some(user_id) = &self.user_id {
            format!("rate_limit:user:{}", user_id)
        } else if let Some(ip) = fallback_ip {
            format!("rate_limit:ip:{}", ip)
        } else {
            // Fallback to a default key if no identifier available
            "rate_limit:anonymous".to_string()
        }
    }
}

/// Tarot reading response to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarotResponse {
    /// Unique identifier for the job
    pub job_id: Uuid,

    /// Current status of the job
    pub status: String,

    /// Human-readable message
    pub message: String,

    /// Timestamp when the job was created
    pub created_at: String,

    /// Estimated processing time (optional)
    pub estimated_wait_seconds: Option<u32>,
}

impl TarotResponse {
    /// Create a successful response for a queued job
    pub fn queued(job_id: Uuid) -> Self {
        Self {
            job_id,
            status: "queued".to_string(),
            message: "Tarot reading request submitted successfully. Your job is now in the queue.".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            estimated_wait_seconds: Some(60), // 1 minute estimate
        }
    }

    /// Create an error response
    pub fn error(message: String) -> Self {
        Self {
            job_id: Uuid::new_v4(), // Generate dummy ID for consistency
            status: "error".to_string(),
            message,
            created_at: chrono::Utc::now().to_rfc3339(),
            estimated_wait_seconds: None,
        }
    }
}

/// API error response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,

    /// Error code for programmatic handling
    pub code: Option<String>,

    /// Timestamp of the error
    pub timestamp: String,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: String) -> Self {
        Self {
            error,
            code: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create an error response with code
    pub fn with_code(error: String, code: String) -> Self {
        Self {
            error,
            code: Some(code),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Service name
    pub service: String,

    /// Current timestamp
    pub timestamp: String,

    /// Version
    pub version: String,
}

impl HealthResponse {
    /// Create a healthy response
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            service: "mimivibe-backend-api".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: "0.1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_question_min_length() {
        let request = TarotRequest {
            question: "ควรทำ".to_string(), // Exactly 5 characters
            user_id: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_valid_question_max_length() {
        let request = TarotRequest {
            question: "a".repeat(100), // Exactly 100 ASCII characters
            user_id: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_question_too_short() {
        let request = TarotRequest {
            question: "abcd".to_string(), // 4 ASCII characters
            user_id: None,
        };

        assert!(request.validate().is_err());

        let error = request.validate().unwrap_err();
        match error {
            TarotRequestError::InvalidQuestionLength { length } => {
                assert_eq!(length, 4);
            }
            _ => panic!("Expected InvalidQuestionLength error"),
        }
    }

    #[test]
    fn test_invalid_question_too_long() {
        let request = TarotRequest {
            question: "a".repeat(101), // 101 ASCII characters
            user_id: None,
        };

        assert!(request.validate().is_err());

        let error = request.validate().unwrap_err();
        match error {
            TarotRequestError::InvalidQuestionLength { length } => {
                assert_eq!(length, 101);
            }
            _ => panic!("Expected InvalidQuestionLength error"),
        }
    }

    #[test]
    fn test_empty_question() {
        let request = TarotRequest {
            question: "".to_string(),
            user_id: None,
        };

        assert!(request.validate().is_err());

        let error = request.validate().unwrap_err();
        assert!(matches!(error, TarotRequestError::EmptyQuestion));
    }

    #[test]
    fn test_whitespace_only_question() {
        let request = TarotRequest {
            question: "   ".to_string(),
            user_id: None,
        };

        assert!(request.validate().is_err());

        let error = request.validate().unwrap_err();
        assert!(matches!(error, TarotRequestError::WhitespaceOnlyQuestion));
    }

    #[test]
    fn test_question_with_whitespace() {
        let request = TarotRequest {
            question: "  ควรจะทำอะไรดีครับ  ".to_string(),
            user_id: None,
        };

        assert!(request.validate().is_ok());
        assert_eq!(request.get_trimmed_question(), "ควรจะทำอะไรดีครับ");
    }

    
    #[test]
    fn test_get_rate_limit_key_with_user_id() {
        let request = TarotRequest {
            question: "ควรจะทำอะไรดีครับ".to_string(),
            user_id: Some("user123".to_string()),
        };

        assert_eq!(request.get_rate_limit_key(None), "rate_limit:user:user123");
    }

    #[test]
    fn test_get_rate_limit_key_with_ip() {
        let request = TarotRequest {
            question: "ควรจะทำอะไรดีครับ".to_string(),
            user_id: None,
        };

        assert_eq!(request.get_rate_limit_key(Some("192.168.1.1")), "rate_limit:ip:192.168.1.1");
    }

    #[test]
    fn test_get_rate_limit_key_anonymous() {
        let request = TarotRequest {
            question: "ควรจะทำอะไรดีครับ".to_string(),
            user_id: None,
        };

        assert_eq!(request.get_rate_limit_key(None), "rate_limit:anonymous");
    }

    #[test]
    fn test_tarot_response_creation() {
        let job_id = Uuid::new_v4();
        let response = TarotResponse::queued(job_id);

        assert_eq!(response.job_id, job_id);
        assert_eq!(response.status, "queued");
        assert!(!response.message.is_empty());
        assert_eq!(response.estimated_wait_seconds, Some(60));
    }

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse::new("Test error".to_string());

        assert_eq!(response.error, "Test error");
        assert!(response.code.is_none());
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_error_response_with_code() {
        let response = ErrorResponse::with_code("Test error".to_string(), "TEST_ERROR".to_string());

        assert_eq!(response.error, "Test error");
        assert_eq!(response.code, Some("TEST_ERROR".to_string()));
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_health_response_creation() {
        let response = HealthResponse::healthy();

        assert_eq!(response.status, "healthy");
        assert_eq!(response.service, "mimivibe-backend-api");
        assert_eq!(response.version, "0.1.0");
        assert!(!response.timestamp.is_empty());
    }
}