//! Question Analyzer Response Models
//!
//! Defines response structures for the QuestionAnalyzer agent with JSON serialization
//! and validation for structured responses from the AI analysis system with Thai
//! cultural context as per the "แม่หมอมีมี่" persona requirements.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Response from QuestionAnalyzer agent after AI analysis
///
/// Contains the structured analysis result with mood, topic, and period
/// in Thai language as per the "แม่หมอมีมี่" persona requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnalyzerResponse {
    /// Emotional mood of the question (Thai)
    pub mood: String,
    /// Main topic or category of the question (Thai)
    pub topic: String,
    /// Time period relevant to the question (Thai)
    pub period: String,
}

/// Allowed mood values in Thai
pub const ALLOWED_MOODS: &[&str] = &[
    "มีความหวัง",
    "กังวล",
    "อยากรู้",
    "มุ่งมั่น",
    "สับสน",
    "ตื่นเต้น",
    "ผิดหวัง",
    "สงสัย",
    "ต้องการกำลังใจ",
];

/// Allowed topic values in Thai
pub const ALLOWED_TOPICS: &[&str] = &[
    "ความรักและความสัมพันธ์",
    "การงานและอาชีพ",
    "การเงิน",
    "สุขภาพ (ภาพรวม)",
    "ครอบครัว",
    "การพัฒนาตนเอง",
    "เส้นทางชีวิต",
    "การตัดสินใจ",
];

/// Allowed period values in Thai
pub const ALLOWED_PERIODS: &[&str] = &[
    "ปัจจุบัน",
    "อนาคตอันใกล้ (1-3 เดือน)",
    "อนาคต (3-6 เดือน)",
    "อนาคต (6-12 เดือน)",
    "ไม่ระบุ",
];

/// Validation error for question analyzer responses
#[derive(Debug, Error)]
pub enum AnalysisValidationError {
    #[error("Invalid mood: {mood}. Allowed: {allowed_values}")]
    InvalidMood { mood: String, allowed_values: String },

    #[error("Invalid topic: {topic}. Allowed: {allowed_values}")]
    InvalidTopic { topic: String, allowed_values: String },

    #[error("Invalid period: {period}. Allowed: {allowed_values}")]
    InvalidPeriod { period: String, allowed_values: String },
}

impl QuestionAnalyzerResponse {
    /// Create a new question analyzer response
    ///
    /// # Arguments
    ///
    /// * `mood` - Emotional mood in Thai
    /// * `topic` - Main topic in Thai
    /// * `period` - Time period in Thai
    ///
    /// # Returns
    ///
    /// A new QuestionAnalyzerResponse instance
    pub fn new(mood: String, topic: String, period: String) -> Self {
        Self {
            mood,
            topic,
            period,
        }
    }

    /// Validate the response format and content against allowed Thai values
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Response is valid
    /// * `Err(AnalysisValidationError)` - Reason why response is invalid
    pub fn validate(&self) -> Result<(), AnalysisValidationError> {
        // Validate mood
        if !ALLOWED_MOODS.contains(&self.mood.as_str()) {
            return Err(AnalysisValidationError::InvalidMood {
                mood: self.mood.clone(),
                allowed_values: ALLOWED_MOODS.join(", "),
            });
        }

        // Validate topic
        if !ALLOWED_TOPICS.contains(&self.topic.as_str()) {
            return Err(AnalysisValidationError::InvalidTopic {
                topic: self.topic.clone(),
                allowed_values: ALLOWED_TOPICS.join(", "),
            });
        }

        // Validate period
        if !ALLOWED_PERIODS.contains(&self.period.as_str()) {
            return Err(AnalysisValidationError::InvalidPeriod {
                period: self.period.clone(),
                allowed_values: ALLOWED_PERIODS.join(", "),
            });
        }

        Ok(())
    }

    /// Check if the response contains Thai characters
    ///
    /// This is a basic validation to ensure Thai persona is maintained
    ///
    /// # Returns
    ///
    /// true if Thai characters are found, false otherwise
    pub fn contains_thai_chars(&self) -> bool {
        let combined_text = format!("{} {} {}", self.mood, self.topic, self.period);
        combined_text.chars().any(|c| {
            let code_point = c as u32;
            // Basic Thai Unicode range check
            (0x0E00..=0x0E7F).contains(&code_point)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_analyzer_response_creation() {
        let response = QuestionAnalyzerResponse::new(
            "มีความหวัง".to_string(),
            "ความรักและความสัมพันธ์".to_string(),
            "ปัจจุบัน".to_string(),
        );

        assert_eq!(response.mood, "มีความหวัง");
        assert_eq!(response.topic, "ความรักและความสัมพันธ์");
        assert_eq!(response.period, "ปัจจุบัน");
    }

    #[test]
    fn test_valid_response_validation() {
        let response = QuestionAnalyzerResponse::new(
            "กังวล".to_string(),
            "การงานและอาชีพ".to_string(),
            "อนาคตอันใกล้ (1-3 เดือน)".to_string(),
        );

        let result = response.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_mood_validation() {
        let response = QuestionAnalyzerResponse::new(
            "invalid_mood".to_string(),
            "การงานและอาชีพ".to_string(),
            "ปัจจุบัน".to_string(),
        );

        let result = response.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, AnalysisValidationError::InvalidMood { .. }));
    }

    #[test]
    fn test_invalid_topic_validation() {
        let response = QuestionAnalyzerResponse::new(
            "มีความหวัง".to_string(),
            "invalid_topic".to_string(),
            "ปัจจุบัน".to_string(),
        );

        let result = response.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, AnalysisValidationError::InvalidTopic { .. }));
    }

    #[test]
    fn test_invalid_period_validation() {
        let response = QuestionAnalyzerResponse::new(
            "มีความหวัง".to_string(),
            "การงานและอาชีพ".to_string(),
            "invalid_period".to_string(),
        );

        let result = response.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, AnalysisValidationError::InvalidPeriod { .. }));
    }

    #[test]
    fn test_contains_thai_chars_detection() {
        let thai_response = QuestionAnalyzerResponse::new(
            "มีความหวัง".to_string(),
            "ความรัก".to_string(),
            "ปัจจุบัน".to_string(),
        );
        assert!(thai_response.contains_thai_chars());

        let english_response = QuestionAnalyzerResponse::new(
            "hopeful".to_string(),
            "love".to_string(),
            "present".to_string(),
        );
        assert!(!english_response.contains_thai_chars());
    }

    #[test]
    fn test_json_serialization_deserialization() {
        let original = QuestionAnalyzerResponse::new(
            "อยากรู้".to_string(),
            "การเงิน".to_string(),
            "อนาคต (3-6 เดือน)".to_string(),
        );

        // Serialize to JSON
        let json = serde_json::to_string(&original).expect("Failed to serialize");

        // Deserialize from JSON
        let deserialized: QuestionAnalyzerResponse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(original.mood, deserialized.mood);
        assert_eq!(original.topic, deserialized.topic);
        assert_eq!(original.period, deserialized.period);
    }

    // Test-first requirements from GitHub issue #68

    #[test]
    fn test_question_analyzer_thai_prompt_loading() {
        // Test that Thai prompt loading logic works with mock data
        use crate::config::env::{Environment, EnvironmentConfig, QueuePoolConfig};
        use crate::utils::prompt_manager::PromptManager;

        // Create test config with encoded Thai prompt
        let thai_prompt = "วิเคราะห์คำถามนี้และตอบเป็นภาษาไทยในรูปแบบ JSON";
        use base64::Engine as _;
        let encoded_prompt = base64::engine::general_purpose::STANDARD.encode(thai_prompt);

        let config = EnvironmentConfig {
            environment: Environment::Development,
            pool: QueuePoolConfig::development(),
            redis_url: Some("redis://localhost:6379".to_string()),
            upstash_url: Some("https://test-upstash.com".to_string()),
            upstash_token: Some("test-token".to_string()),
            stream_key: "test:stream".to_string(),
            consumer_group: "test-consumers".to_string(),
            question_filter_prompt: String::new(),
            question_analyzer_prompt: encoded_prompt,
            reading_agent_prompt: String::new(),
            question_filter_version: "v1".to_string(),
            question_analyzer_version: "v1".to_string(),
            reading_agent_version: "v1".to_string(),
        };

        let prompt_manager = PromptManager::new(config);
        let result = prompt_manager.load_prompt("question_analyzer");

        assert!(result.is_ok());
        let decoded_prompt = result.unwrap();
        assert_eq!(decoded_prompt, thai_prompt);
    }

    #[test]
    fn test_json_response_parsing_success() {
        // This test will initially fail - Red Phase
        // Tests that valid JSON response parses to QuestionAnalyzerResponse
        let json_response = r#"
        {
            "mood": "มีความหวัง",
            "topic": "ความรักและความสัมพันธ์",
            "period": "ปัจจุบัน"
        }
        "#;

        let result: Result<QuestionAnalyzerResponse, _> = serde_json::from_str(json_response);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.mood, "มีความหวัง");
        assert_eq!(response.topic, "ความรักและความสัมพันธ์");
        assert_eq!(response.period, "ปัจจุบัน");
    }

    #[test]
    fn test_json_response_invalid_mood_format() {
        // This test will initially fail - Red Phase
        // Tests that invalid mood value returns ValidationError
        let json_response = r#"
        {
            "mood": "invalid_mood",
            "topic": "ความรักและความสัมพันธ์",
            "period": "ปัจจุบัน"
        }
        "#;

        let result: Result<QuestionAnalyzerResponse, _> = serde_json::from_str(json_response);
        assert!(result.is_ok());

        let response = result.unwrap();
        let validation_result = response.validate();
        assert!(validation_result.is_err());
        assert!(matches!(validation_result.unwrap_err(), AnalysisValidationError::InvalidMood { .. }));
    }

    #[test]
    fn test_placeholder_substitution_all_variables() {
        // Test that all placeholders are replaced correctly in prompt templates
        use crate::config::env::{Environment, EnvironmentConfig, QueuePoolConfig};
        use crate::utils::prompt_manager::PromptManager;
        use crate::models::PromptRenderContext;

        // Create test config with template containing placeholders
        let template_with_placeholders = "วิเคราะห์คำถาม: {question} ด้วยอารมณ์: {mood} ในหัวข้อ: {topic} ช่วงเวลา: {period}";
        use base64::Engine as _;
        let encoded_template = base64::engine::general_purpose::STANDARD.encode(template_with_placeholders);

        let config = EnvironmentConfig {
            environment: Environment::Development,
            pool: QueuePoolConfig::development(),
            redis_url: Some("redis://localhost:6379".to_string()),
            upstash_url: Some("https://test-upstash.com".to_string()),
            upstash_token: Some("test-token".to_string()),
            stream_key: "test:stream".to_string(),
            consumer_group: "test-consumers".to_string(),
            question_filter_prompt: String::new(),
            question_analyzer_prompt: encoded_template,
            reading_agent_prompt: String::new(),
            question_filter_version: "v1".to_string(),
            question_analyzer_version: "v1".to_string(),
            reading_agent_version: "v1".to_string(),
        };

        let prompt_manager = PromptManager::new(config);
        let template = prompt_manager.load_prompt("question_analyzer").unwrap();

        // Create context with all placeholders
        let context = PromptRenderContext::new("ฉันควรเปลี่ยนงานไหม".to_string())
            .with_mood("กังวล".to_string())
            .with_topic("การงานและอาชีพ".to_string())
            .with_period("อนาคตอันใกล้ (1-3 เดือน)".to_string());

        let rendered = prompt_manager.render_template(&template, &context).unwrap();

        assert!(rendered.contains("ฉันควรเปลี่ยนงานไหม"));
        assert!(rendered.contains("กังวล"));
        assert!(rendered.contains("การงานและอาชีพ"));
        assert!(rendered.contains("อนาคตอันใกล้ (1-3 เดือน)"));
        assert!(!rendered.contains("{question}"));
        assert!(!rendered.contains("{mood}"));
        assert!(!rendered.contains("{topic}"));
        assert!(!rendered.contains("{period}"));
    }

    #[test]
    fn test_analyze_question_response_structure() {
        // Test that response has all required fields
        let response = QuestionAnalyzerResponse::new(
            "มีความหวัง".to_string(),
            "ความรักและความสัมพันธ์".to_string(),
            "ปัจจุบัน".to_string(),
        );

        // Test all required fields are present and not empty
        assert!(!response.mood.is_empty());
        assert!(!response.topic.is_empty());
        assert!(!response.period.is_empty());

        // Test validation passes for valid values
        assert!(response.validate().is_ok());

        // Test Thai character detection works
        assert!(response.contains_thai_chars());

        // Test JSON serialization/deserialization preserves structure
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: QuestionAnalyzerResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.mood, deserialized.mood);
        assert_eq!(response.topic, deserialized.topic);
        assert_eq!(response.period, deserialized.period);
    }

    #[test]
    fn test_mood_validation_from_predefined_list() {
        // Test all valid moods
        for valid_mood in ALLOWED_MOODS {
            let response = QuestionAnalyzerResponse::new(
                valid_mood.to_string(),
                "ความรักและความสัมพันธ์".to_string(),
                "ปัจจุบัน".to_string(),
            );
            assert!(response.validate().is_ok(), "Valid mood '{}' should pass", valid_mood);
        }
    }

    #[test]
    fn test_topic_validation_from_predefined_list() {
        // Test all valid topics
        for valid_topic in ALLOWED_TOPICS {
            let response = QuestionAnalyzerResponse::new(
                "มีความหวัง".to_string(),
                valid_topic.to_string(),
                "ปัจจุบัน".to_string(),
            );
            assert!(response.validate().is_ok(), "Valid topic '{}' should pass", valid_topic);
        }
    }

    #[test]
    fn test_period_validation_from_predefined_list() {
        // Test all valid periods
        for valid_period in ALLOWED_PERIODS {
            let response = QuestionAnalyzerResponse::new(
                "มีความหวัง".to_string(),
                "ความรักและความสัมพันธ์".to_string(),
                valid_period.to_string(),
            );
            assert!(response.validate().is_ok(), "Valid period '{}' should pass", valid_period);
        }
    }

    #[test]
    fn test_end_to_end_question_analysis() {
        // Test complete flow from question to analysis result using mock data
        use crate::config::env::{Environment, EnvironmentConfig, QueuePoolConfig};
        use crate::utils::prompt_manager::PromptManager;
        use crate::models::PromptRenderContext;

        // Mock the complete flow:
        // 1. Load encoded prompt template
        // 2. Render with question context
        // 3. Parse structured response (simulated)

        let mock_prompt = "วิเคราะห์คำถาม: {question} และตอบในรูปแบบ JSON";
        use base64::Engine as _;
        let encoded_prompt = base64::engine::general_purpose::STANDARD.encode(mock_prompt);

        let config = EnvironmentConfig {
            environment: Environment::Development,
            pool: QueuePoolConfig::development(),
            redis_url: Some("redis://localhost:6379".to_string()),
            upstash_url: Some("https://test-upstash.com".to_string()),
            upstash_token: Some("test-token".to_string()),
            stream_key: "test:stream".to_string(),
            consumer_group: "test-consumers".to_string(),
            question_filter_prompt: String::new(),
            question_analyzer_prompt: encoded_prompt,
            reading_agent_prompt: String::new(),
            question_filter_version: "v1".to_string(),
            question_analyzer_version: "v1".to_string(),
            reading_agent_version: "v1".to_string(),
        };

        // Step 1: Load prompt template
        let prompt_manager = PromptManager::new(config);
        let template = prompt_manager.load_prompt("question_analyzer").unwrap();
        assert!(!template.is_empty());

        // Step 2: Build context and render template
        let context = PromptRenderContext::new("ฉันจะประสบความสำเร็จในการงานไหม".to_string());
        let rendered_prompt = prompt_manager.render_template(&template, &context).unwrap();
        assert!(rendered_prompt.contains("ฉันจะประสบความสำเร็จในการงานไหม"));

        // Step 3: Simulate AI response parsing (in real scenario, this comes from Gemini)
        let mock_ai_response = r#"
        {
            "mood": "มีความหวัง",
            "topic": "การงานและอาชีพ",
            "period": "อนาคต (3-6 เดือน)"
        }
        "#;

        let parsed_response: QuestionAnalyzerResponse = serde_json::from_str(mock_ai_response).unwrap();

        // Step 4: Validate the complete flow result
        assert!(parsed_response.validate().is_ok());
        assert_eq!(parsed_response.mood, "มีความหวัง");
        assert_eq!(parsed_response.topic, "การงานและอาชีพ");
        assert_eq!(parsed_response.period, "อนาคต (3-6 เดือน)");
        assert!(parsed_response.contains_thai_chars());
    }

    #[test]
    #[ignore] // Requires real environment variables to run
    fn test_real_gemini_api_integration_analyzer() {
        // Test actual Gemini API call with new prompt format
        // This test requires GEMINI_API_KEY environment variable
        // Run with: GEMINI_API_KEY=your_key cargo test test_real_gemini_api_integration_analyzer -- --ignored

        use std::env;

        // Check if required environment variables are set
        if env::var("GEMINI_API_KEY").is_err() {
            println!("Skipping test_real_gemini_api_integration_analyzer: GEMINI_API_KEY not set");
            return;
        }

        // This test would require real QuestionAnalyzer with real API calls
        // For now, just test that environment validation logic works
        let api_key = env::var("GEMINI_API_KEY").unwrap();
        assert!(!api_key.is_empty());

        // TODO: When running with real environment, test actual QuestionAnalyzer.analyze_question()
        // This would require:
        // 1. Real environment configuration
        // 2. Actual Gemini API calls
        // 3. Real Thai prompt template from environment variables

        println!("Environment validation passed - ready for real API integration testing");
    }
}