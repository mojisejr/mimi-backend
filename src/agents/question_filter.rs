//! Question Filter Agent
//!
//! Validates and filters user questions before processing through the tarot pipeline.
//! Uses Gemini API to check question appropriateness and validity.
//! Part of the LangGraph-style agent workflow.

use crate::utils::gemini::{GeminiClient, GeminiError};
use crate::utils::prompt_manager::PromptManager;
use std::collections::HashMap;
use std::sync::Arc;
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
    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Invalid response format: {message}")]
    InvalidResponse { message: String },
    #[error("Template error: {0}")]
    TemplateError(#[from] crate::utils::prompt_manager::PromptError),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Question Filter Agent
#[derive(Debug, Clone)]
pub struct QuestionFilter {
    client: GeminiClient,
    prompt_manager: Arc<PromptManager>,
    min_length: usize,
    max_length: usize,
}

impl QuestionFilter {
    /// Create a new QuestionFilter instance
    pub async fn new() -> Result<Self, QuestionFilterError> {
        let config = crate::config::env::EnvironmentConfig::from_env()
            .map_err(|e| QuestionFilterError::ConfigError(e.to_string()))?;

        let prompt_manager = PromptManager::new(config);

        Ok(Self {
            client: GeminiClient::new()?,
            prompt_manager: Arc::new(prompt_manager),
            min_length: 5,
            max_length: 500,
        })
    }

    /// Create a QuestionFilter with custom length limits
    pub async fn with_limits(
        min_length: usize,
        max_length: usize,
    ) -> Result<Self, QuestionFilterError> {
        if min_length >= max_length {
            return Err(QuestionFilterError::ValidationFailed {
                reason: "Minimum length cannot be greater than or equal to maximum length"
                    .to_string(),
            });
        }

        let config = crate::config::env::EnvironmentConfig::from_env()
            .map_err(|e| QuestionFilterError::ConfigError(e.to_string()))?;

        let prompt_manager = PromptManager::new(config);

        Ok(Self {
            client: GeminiClient::new()?,
            prompt_manager: Arc::new(prompt_manager),
            min_length,
            max_length,
        })
    }

    /// Create a QuestionFilter with prompt cache (preferred for database prompts)
    pub async fn with_prompt_cache(
        prompt_cache: Arc<HashMap<String, String>>,
    ) -> Result<Self, QuestionFilterError> {
        let prompt_manager = PromptManager::with_cache(prompt_cache);

        Ok(Self {
            client: GeminiClient::new()?,
            prompt_manager: Arc::new(prompt_manager),
            min_length: 5,
            max_length: 500,
        })
    }

    /// Create a QuestionFilter with prompt cache and custom limits
    pub async fn with_prompt_cache_and_limits(
        prompt_cache: Arc<HashMap<String, String>>,
        min_length: usize,
        max_length: usize,
    ) -> Result<Self, QuestionFilterError> {
        if min_length >= max_length {
            return Err(QuestionFilterError::ValidationFailed {
                reason: "Minimum length cannot be greater than or equal to maximum length"
                    .to_string(),
            });
        }

        let prompt_manager = PromptManager::with_cache(prompt_cache);

        Ok(Self {
            client: GeminiClient::new()?,
            prompt_manager: Arc::new(prompt_manager),
            min_length,
            max_length,
        })
    }

    /// Create a QuestionFilter with fallback (database cache + environment fallback)
    pub async fn with_fallback(
        prompt_cache: Arc<HashMap<String, String>>,
    ) -> Result<Self, QuestionFilterError> {
        let config = crate::config::env::EnvironmentConfig::from_env()
            .map_err(|e| QuestionFilterError::ConfigError(e.to_string()))?;

        let prompt_manager = PromptManager::with_fallback(config, prompt_cache);

        Ok(Self {
            client: GeminiClient::new()?,
            prompt_manager: Arc::new(prompt_manager),
            min_length: 5,
            max_length: 500,
        })
    }

    /// Validate a user question with new JSON response format
    pub async fn validate_question(
        &self,
        question: &str,
    ) -> Result<crate::models::question_filter::QuestionFilterResponse, QuestionFilterError> {
        // Basic length validation first
        self.validate_length(question)?;

        // Use Gemini API with Thai prompt for content validation
        self.validate_content_with_ai(question).await
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

    /// Render the filter prompt with the given question
    async fn render_filter_prompt(&self, question: &str) -> Result<String, QuestionFilterError> {
        let context = crate::models::prompt::PromptRenderContext::new(question.to_string());
        let template = self.prompt_manager.load_prompt("question_filter")?;
        self.prompt_manager
            .render_template(&template, &context)
            .map_err(QuestionFilterError::TemplateError)
    }

    /// AI-based content validation using Gemini with structured JSON response
    async fn validate_content_with_ai(
        &self,
        question: &str,
    ) -> Result<crate::models::question_filter::QuestionFilterResponse, QuestionFilterError> {
        // Load and render Thai prompt template
        let formatted_prompt = self.render_filter_prompt(question).await?;

        // Call Gemini API with new prompt
        let response = self.client.generate_text(&formatted_prompt).await?;

        // Parse JSON response
        let filter_response: crate::models::question_filter::QuestionFilterResponse =
            serde_json::from_str(&response).map_err(QuestionFilterError::JsonParseError)?;

        // Validate response format
        if filter_response.reason.is_empty() {
            return Err(QuestionFilterError::InvalidResponse {
                message: "Response reason cannot be empty".to_string(),
            });
        }

        Ok(filter_response)
    }

    /// Filter and normalize a question
    pub async fn filter_question(&self, question: &str) -> Result<String, QuestionFilterError> {
        let validation_result = self.validate_question(question).await?;

        if validation_result.is_valid {
            Ok(question.trim().to_string())
        } else {
            Err(QuestionFilterError::ValidationFailed {
                reason: validation_result.reason,
            })
        }
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

// Default implementation removed since constructor is now async

/// Configuration for QuestionFilter
#[derive(Debug, Clone)]
pub struct QuestionFilterConfig {
    pub min_length: usize,
    pub max_length: usize,
}

/// Legacy function for backward compatibility
pub async fn filter_question(question: &str) -> Result<bool, String> {
    let filter = QuestionFilter::new().await.map_err(|e| e.to_string())?;

    Ok(filter.is_valid_question(question).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::env::{Environment, EnvironmentConfig, QueuePoolConfig};
    use crate::models::prompt::PromptRenderContext;
    use crate::models::question_filter::QuestionFilterResponse;
    use crate::utils::prompt_manager::PromptManager;
    use serde_json;

    // Test fixture for creating test config with encoded prompt
    fn create_test_config() -> EnvironmentConfig {
        // Thai prompt: "คุณเป็นแม่หมอมีมี่ กรุณาตรวจสอบคำถาม: {question} และตอบเป็นภาษาไทยเท่านั้น"
        let thai_prompt_base64 = "4Lir4Lil4LiZ4Lix4Liq4LiB4Lil4LiV4Liy4LiB4LmA4LiZ4Lix4Lii4LmA4LiZ4LiI4Lil4LiB4Liy4Lil4Li04LiZ4Lix4LiB4Lia4Lii4LiB4LiE4Li44LiB4Liy4Li04LiU6IHtxdWVzdGlvbn0g4LmA4LiV4Liq4Li14Li34LmA4LmA4LiZ4Liq4Li14Li34LmB4LmM4LiZ4LmE4Lix4Li04LiU=";

        EnvironmentConfig {
            environment: Environment::Development,
            pool: QueuePoolConfig::development(),
            redis_url: Some("redis://localhost:6379".to_string()),
            upstash_url: Some("https://test-upstash.com".to_string()),
            upstash_token: Some("test-token".to_string()),
            stream_key: "test:stream".to_string(),
            consumer_group: "test-consumers".to_string(),
            question_filter_prompt: thai_prompt_base64.to_string(),
            question_analyzer_prompt: "VGVzdCBwcm9tcHQ=".to_string(),
            reading_agent_prompt: "UmVhZGluZyBhZ2VudCBwcm9tcHQ=".to_string(),
            question_filter_version: "v1".to_string(),
            question_analyzer_version: "v1".to_string(),
            reading_agent_version: "v1".to_string(),
        }
    }

    #[test]
    fn test_question_filter_thai_prompt_loading() {
        // Test that Thai prompt loads and decodes correctly
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let result = prompt_manager.load_prompt("question_filter");

        // This should pass - prompt should load and decode correctly
        assert!(result.is_ok(), "Thai prompt should load successfully");

        let decoded_prompt = result.unwrap();
        assert!(
            !decoded_prompt.is_empty(),
            "Decoded prompt should not be empty"
        );
        assert!(
            decoded_prompt.contains("แม่หมอมีมี่"),
            "Decoded prompt should contain Thai persona name"
        );
        assert!(
            decoded_prompt.contains("{question}"),
            "Decoded prompt should contain placeholder"
        );
    }

    #[test]
    fn test_json_response_parsing_success() {
        // Test valid JSON response parsing to QuestionFilterResponse
        let json_response = r#"{
            "is_valid": true,
            "reason": "คำถามนี้เหมาะสมสำหรับการทำนายดวงชะตาค่ะ"
        }"#;

        let result: Result<QuestionFilterResponse, serde_json::Error> =
            serde_json::from_str(json_response);

        assert!(result.is_ok(), "Valid JSON should parse successfully");

        let response = result.unwrap();
        assert!(response.is_valid, "Response should be marked as valid");
        assert_eq!(response.reason, "คำถามนี้เหมาะสมสำหรับการทำนายดวงชะตาค่ะ");
    }

    #[test]
    fn test_json_response_invalid_format() {
        // Test malformed JSON returns ParsingError
        let invalid_json = r#"{
            "is_valid": true,
            "reason": "คำถามนี้เหมาะสม"
            // Missing closing brace and comma
        "#;

        let result: Result<QuestionFilterResponse, serde_json::Error> =
            serde_json::from_str(invalid_json);

        assert!(result.is_err(), "Invalid JSON should fail to parse");

        let error = result.unwrap_err();
        assert!(
            format!("{}", error).contains("expected"),
            "Error should indicate parsing failure: {}",
            error
        );
    }

    #[test]
    fn test_placeholder_question_substitution() {
        // Test that {question} placeholder is replaced correctly
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "คุณเป็นแม่หมอมีมี่ กรุณาตรวจสอบคำถาม: {question} และตอบเป็นภาษาไทยเท่านั้น";
        let context = PromptRenderContext::new("ควรจะลงทุนอะไรดีครับ".to_string());

        let result = prompt_manager.render_template(template, &context);

        assert!(result.is_ok(), "Template rendering should succeed");

        let rendered = result.unwrap();
        assert!(
            rendered.contains("ควรจะลงทุนอะไรดีครับ"),
            "Rendered template should contain the question"
        );
        assert!(
            !rendered.contains("{question}"),
            "Rendered template should not contain placeholder"
        );
    }

    #[test]
    fn test_validate_question_approved_response() {
        // Test isValid:true response should pass validation
        let response =
            QuestionFilterResponse::valid("คำถามของคุณเหมาะสมสำหรับการทำนายดวงชะตาค่ะ".to_string());

        let validation_result = response.validate();

        assert!(
            validation_result.is_ok(),
            "Valid response should pass validation"
        );
        assert!(
            response.contains_thai_chars(),
            "Valid response should contain Thai characters"
        );
    }

    #[test]
    fn test_validate_question_rejected_response() {
        // Test isValid:false response should fail with reason
        let response =
            QuestionFilterResponse::invalid("คำถามนี้ไม่เหมาะสมเนื่องจากมีเนื้อหาที่เป็นอันตราย".to_string());

        let validation_result = response.validate();

        // Invalid responses should pass validation (they're valid responses)
        assert!(
            validation_result.is_ok(),
            "Invalid response should pass validation"
        );
        assert!(!response.is_valid, "Response should be marked as invalid");
        assert!(!response.reason.is_empty(), "Reason should not be empty");
    }

    #[test]
    fn test_thai_persona_response_validation() {
        // Test response should contain Thai text only
        let valid_thai_response = QuestionFilterResponse::valid(
            "คำถามนี้เหมาะสมสำหรับการทำนายดวงชะตา แม่หมอมีมี่ยินดีให้คำแนะนำค่ะ".to_string(),
        );

        let validation_result = valid_thai_response.validate();

        assert!(
            validation_result.is_ok(),
            "Thai persona response should pass validation"
        );
        assert!(
            valid_thai_response.contains_thai_chars(),
            "Thai response should contain Thai characters"
        );

        // Test non-Thai response fails validation
        let non_thai_response = QuestionFilterResponse::valid(
            "This question is appropriate for tarot reading".to_string(),
        );

        let validation_result = non_thai_response.validate();
        assert!(
            validation_result.is_err(),
            "Non-Thai response should fail validation"
        );
    }

    #[tokio::test]
    async fn test_length_validation() {
        let filter = QuestionFilter::with_limits(5, 100).await.unwrap();

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

    #[tokio::test]
    async fn test_thai_question_detection() {
        let filter = QuestionFilter::with_limits(5, 500).await.unwrap();

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

    #[tokio::test]
    async fn test_filter_config() {
        let filter = QuestionFilter::with_limits(10, 200).await.unwrap();
        let config = filter.config();
        assert_eq!(config.min_length, 10);
        assert_eq!(config.max_length, 200);
    }

    #[tokio::test]
    async fn test_invalid_limits() {
        let result = QuestionFilter::with_limits(10, 5).await;
        assert!(result.is_err());
    }
}
