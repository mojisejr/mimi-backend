//! Question Filter Agent
//!
//! Validates and filters user questions before processing through the tarot pipeline.
//! Uses Gemini API to check question appropriateness and validity.
//! Part of the LangGraph-style agent workflow.

use crate::utils::gemini::{GeminiClient, GeminiError};
use thiserror::Error;

/// Error types for question filtering
#[derive(Debug, Error)]
pub enum QuestionFilterError {
    #[error("Question is empty or too short")]
    EmptyQuestion,
    #[error("Question is too short: {length} characters (minimum: {min_length})")]
    TooShort { length: usize, min_length: usize },
    #[error("Question is too long: {length} characters (maximum: {max_length})")]
    TooLong { length: usize, max_length: usize },
    #[error("Question contains inappropriate content")]
    InappropriateContent,
    #[error("Gemini API error: {0}")]
    ApiError(#[from] GeminiError),
    #[error("Question validation failed: {reason}")]
    ValidationFailed { reason: String },
}

/// Question Filter Agent
#[derive(Debug, Clone)]
pub struct QuestionFilter {
    client: GeminiClient,
    min_length: usize,
    max_length: usize,
}

impl QuestionFilter {
    /// Create a new QuestionFilter instance
    pub fn new() -> Result<Self, QuestionFilterError> {
        Ok(Self {
            client: GeminiClient::new()?,
            min_length: 5,
            max_length: 500,
        })
    }

    /// Create a QuestionFilter with custom length limits
    pub fn with_limits(min_length: usize, max_length: usize) -> Result<Self, QuestionFilterError> {
        if min_length >= max_length {
            return Err(QuestionFilterError::ValidationFailed {
                reason: "Minimum length cannot be greater than or equal to maximum length"
                    .to_string(),
            });
        }

        Ok(Self {
            client: GeminiClient::new()?,
            min_length,
            max_length,
        })
    }

    /// Validate a user question
    pub async fn validate_question(&self, question: &str) -> Result<(), QuestionFilterError> {
        // Basic length validation first
        self.validate_length(question)?;

        // Use Gemini API for content validation
        self.validate_content_with_ai(question).await?;

        Ok(())
    }

    /// Check if a question is valid (returns boolean)
    pub async fn is_valid_question(&self, question: &str) -> bool {
        self.validate_question(question).await.is_ok()
    }

    /// Basic length validation
    fn validate_length(&self, question: &str) -> Result<(), QuestionFilterError> {
        let trimmed = question.trim();

        if trimmed.is_empty() {
            return Err(QuestionFilterError::EmptyQuestion);
        }

        if trimmed.len() < self.min_length {
            return Err(QuestionFilterError::TooShort {
                length: trimmed.len(),
                min_length: self.min_length,
            });
        }

        if trimmed.len() > self.max_length {
            return Err(QuestionFilterError::TooLong {
                length: trimmed.len(),
                max_length: self.max_length,
            });
        }

        Ok(())
    }

    /// AI-based content validation using Gemini
    async fn validate_content_with_ai(&self, question: &str) -> Result<(), QuestionFilterError> {
        let prompt = format!(
            r#"Please analyze this question for appropriateness in a tarot reading context:

Question: "{}"

Evaluate the question based on these criteria:
1. Is it a genuine question about life, future, relationships, career, or personal guidance?
2. Does it avoid harmful, illegal, or dangerous content?
3. Is it respectful and appropriate for a spiritual/divinatory context?
4. Is it written in a reasonable manner (not gibberish, spam, or offensive)?

Respond with only "APPROVED" if the question is appropriate, or provide a brief reason if it should be rejected."#,
            question.trim()
        );

        let response = self.client.generate_text(&prompt).await?;

        let trimmed_response = response.trim().to_uppercase();

        if trimmed_response == "APPROVED" {
            Ok(())
        } else {
            Err(QuestionFilterError::InappropriateContent)
        }
    }

    /// Filter and normalize a question
    pub async fn filter_question(&self, question: &str) -> Result<String, QuestionFilterError> {
        self.validate_question(question).await?;

        // Return cleaned/normalized version
        Ok(question.trim().to_string())
    }

    /// Check if question is in Thai language (basic check)
    pub fn is_thai_question(&self, question: &str) -> bool {
        question.chars().any(|c| {
            let code_point = c as u32;
            // Basic Thai Unicode range check
            (0x0E00..=0x0E7F).contains(&code_point)
        })
    }

    /// Get filter configuration
    pub fn config(&self) -> QuestionFilterConfig {
        QuestionFilterConfig {
            min_length: self.min_length,
            max_length: self.max_length,
        }
    }
}

impl Default for QuestionFilter {
    fn default() -> Self {
        Self::new().expect("GEMINI_API_KEY must be set")
    }
}

/// Configuration for QuestionFilter
#[derive(Debug, Clone)]
pub struct QuestionFilterConfig {
    pub min_length: usize,
    pub max_length: usize,
}

/// Legacy function for backward compatibility
pub async fn filter_question(question: &str) -> Result<bool, String> {
    let filter = QuestionFilter::new().map_err(|e| e.to_string())?;

    Ok(filter.is_valid_question(question).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_validation() {
        let filter = QuestionFilter::with_limits(5, 100).unwrap();

        // Test empty question
        let result = filter.validate_length("");
        assert!(matches!(result, Err(QuestionFilterError::EmptyQuestion)));

        // Test too short
        let result = filter.validate_length("abc");
        assert!(matches!(result, Err(QuestionFilterError::TooShort { .. })));

        // Test valid length
        let result = filter.validate_length("Hello world");
        assert!(result.is_ok());
    }

    #[test]
    fn test_thai_question_detection() {
        let filter = QuestionFilter::new().unwrap();

        // Test Thai question
        let thai_question = "ควรจะลงทุนอะไรดีครับ";
        assert!(filter.is_thai_question(thai_question));

        // Test English question
        let english_question = "What should I invest in?";
        assert!(!filter.is_thai_question(english_question));

        // Test mixed language
        let mixed_question = "สวัสดีครับ Hello";
        assert!(filter.is_thai_question(mixed_question));
    }

    #[test]
    fn test_filter_config() {
        let filter = QuestionFilter::with_limits(10, 200).unwrap();
        let config = filter.config();
        assert_eq!(config.min_length, 10);
        assert_eq!(config.max_length, 200);
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_ai_content_validation() {
        let filter = QuestionFilter::new().unwrap();

        // Test valid question
        let valid_question = "ควรจะลงทุนอะไรดีครับ";
        let result = filter.is_valid_question(valid_question).await;
        assert!(result);

        // Test invalid question (this might vary based on Gemini's assessment)
        let invalid_question = "ฆ่าคน";
        let result = filter.is_valid_question(invalid_question).await;
        assert!(!result);
    }

    #[test]
    fn test_invalid_limits() {
        let result = QuestionFilter::with_limits(10, 5);
        assert!(result.is_err());
    }
}
