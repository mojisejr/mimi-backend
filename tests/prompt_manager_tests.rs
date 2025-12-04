//! Prompt Manager Tests
//!
//! Test-first implementation for Base64 Encoded Prompt Templates Infrastructure
//! These tests will initially fail (Red phase) and then pass after implementation

use mimivibe_backend::config::env::EnvironmentConfig;
use mimivibe_backend::models::prompt::PromptRenderContext;
use mimivibe_backend::utils::prompt_manager::{PromptError, PromptManager};
use std::env;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that valid base64 strings decode correctly
    #[test]
    fn test_base64_decode_success() {
        // This should pass after implementation
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        // Test with known valid base64 string
        let test_encoded = "SGVsbG8gV29ybGQ="; // "Hello World"
        let result = prompt_manager.decode_base64(test_encoded);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");
    }

    /// Test that invalid base64 strings return proper error
    #[test]
    fn test_base64_decode_invalid() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        // Test with invalid base64 string
        let invalid_encoded = "Invalid@Base64!!!";
        let result = prompt_manager.decode_base64(invalid_encoded);

        assert!(result.is_err());
        match result.unwrap_err() {
            PromptError::Base64Decode { variable, .. } => {
                assert_eq!(variable, "test");
            }
            _ => panic!("Expected Base64Decode error"),
        }
    }

    /// Test template rendering with single placeholder
    #[test]
    fn test_template_rendering_single_placeholder() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "Hello {question}, how are you?";
        let context = PromptRenderContext::new("What is my future?".to_string());

        let result = prompt_manager.render_template(template, &context);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello What is my future?, how are you?");
    }

    /// Test template rendering with multiple placeholders
    #[test]
    fn test_template_rendering_multiple_placeholders() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "Question: {question}, Mood: {mood}, Topic: {topic}, Period: {period}";
        let context = PromptRenderContext::new("Should I invest?".to_string())
            .with_mood("anxious".to_string())
            .with_topic("finance".to_string())
            .with_period("next_month".to_string());

        let result = prompt_manager.render_template(template, &context);

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(
            rendered,
            "Question: Should I invest?, Mood: anxious, Topic: finance, Period: next_month"
        );
    }

    /// Test missing environment variable handling
    #[test]
    fn test_missing_environment_variable() {
        let mut config = create_test_config();
        config.question_filter_prompt = String::new(); // Simulate missing env var

        let prompt_manager = PromptManager::new(config);
        let result = prompt_manager.load_prompt("question_filter");

        assert!(result.is_err());
        match result.unwrap_err() {
            PromptError::MissingEnvironment { variable } => {
                assert_eq!(variable, "QUESTION_FILTER_PROMPT");
            }
            _ => panic!("Expected MissingEnvironment error"),
        }
    }

    /// Test prompt manager loading question filter prompt
    #[test]
    fn test_prompt_manager_load_question_filter() {
        let config = create_test_config_with_encoded_prompts();
        let prompt_manager = PromptManager::new(config);

        let result = prompt_manager.load_prompt("question_filter");

        assert!(result.is_ok());
        let decoded_prompt = result.unwrap();
        assert!(decoded_prompt.contains("question")); // Should contain Thai prompt content
    }

    /// Test end-to-end template rendering with real encoded prompts
    #[test]
    fn test_end_to_end_template_rendering() {
        let config = create_test_config_with_encoded_prompts();
        let prompt_manager = PromptManager::new(config);

        // Load encoded prompt
        let template_result = prompt_manager.load_prompt("question_filter");
        assert!(template_result.is_ok());
        let template = template_result.unwrap();

        // Render with context
        let context = PromptRenderContext::new("ควรจะลงทุนอะไรดีครับ".to_string());
        let rendered_result = prompt_manager.render_template(&template, &context);

        assert!(rendered_result.is_ok());
        let rendered = rendered_result.unwrap();
        assert!(rendered.contains("ควรจะลงทุนอะไรดีครับ"));
    }

    /// Test performance requirements (<5ms for decode + render)
    #[test]
    fn test_performance_requirements() {
        let config = create_test_config_with_encoded_prompts();
        let prompt_manager = PromptManager::new(config);

        let start = std::time::Instant::now();

        // Load and render prompt
        let template_result = prompt_manager.load_prompt("question_filter");
        assert!(template_result.is_ok());

        let context = PromptRenderContext::new("test question".to_string());
        let rendered_result = prompt_manager.render_template(&template_result.unwrap(), &context);
        assert!(rendered_result.is_ok());

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 5,
            "Performance requirement: <5ms, took {}ms",
            duration.as_millis()
        );
    }

    /// Test template rendering with cards placeholder
    #[test]
    fn test_template_rendering_with_cards() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "Cards drawn: {cards}";
        let context = PromptRenderContext::new("test".to_string()).with_cards(vec![
            "The Sun".to_string(),
            "The Moon".to_string(),
            "The Star".to_string(),
        ]);

        let result = prompt_manager.render_template(template, &context);

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("The Sun"));
        assert!(rendered.contains("The Moon"));
        assert!(rendered.contains("The Star"));
    }

    /// Test error handling for invalid template placeholders
    #[test]
    fn test_invalid_template_placeholder() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "Hello {invalid_placeholder}";
        let context = PromptRenderContext::new("test".to_string());

        let result = prompt_manager.render_template(template, &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            PromptError::InvalidPlaceholder { placeholder } => {
                assert_eq!(placeholder, "invalid_placeholder");
            }
            _ => panic!("Expected InvalidPlaceholder error"),
        }
    }

    // Helper functions for test setup

    fn create_test_config() -> EnvironmentConfig {
        EnvironmentConfig {
            environment: mimivibe_backend::config::env::Environment::Development,
            pool: mimivibe_backend::config::env::QueuePoolConfig::development(),
            redis_url: Some("redis://localhost:6379".to_string()),
            upstash_url: Some("https://test-upstash.com".to_string()),
            upstash_token: Some("test-token".to_string()),
            stream_key: "test:stream".to_string(),
            consumer_group: "test-consumers".to_string(),
            question_filter_prompt: "test-prompt".to_string(),
            question_analyzer_prompt: "test-prompt".to_string(),
            reading_agent_prompt: "test-prompt".to_string(),
            question_filter_version: "v1".to_string(),
            question_analyzer_version: "v1".to_string(),
            reading_agent_version: "v1".to_string(),
        }
    }

    fn create_test_config_with_encoded_prompts() -> EnvironmentConfig {
        EnvironmentConfig {
            environment: mimivibe_backend::config::env::Environment::Development,
            pool: mimivibe_backend::config::env::QueuePoolConfig::development(),
            redis_url: Some("redis://localhost:6379".to_string()),
            upstash_url: Some("https://test-upstash.com".to_string()),
            upstash_token: Some("test-token".to_string()),
            stream_key: "test:stream".to_string(),
            consumer_group: "test-consumers".to_string(),
            question_filter_prompt: "4LiE4Li44LiT4LiE4Li34Lit4Lic4Li54LmJ4LiK4Lil4Li44LmM4LiX4Lii4Liy4LiB4Liy4Li14LiB4Liw4LiB4Lij4LiW4LiH4Liy4Lih4Li34LmM4LiH4Lii4Li04LiE4Liy4LmE4Li44LiK4Liy4LiB4Lix4Lij4LiU4LiE4Liy4Li44LiK4Liw4LiH4Liy4Lih4Li04LmI4Li34Lij4LiU4LiE4Liy4Li44LiK4LiE4Liy4Li04LiE4Liy4LmE4Li44LiK4LiB4Liy4Lih4Li04LmI4Li34Lij4LiU".to_string(), // Sample encoded prompt
            question_analyzer_prompt: "4LiE4Li44LiT4LiE4Li34Lit4Lic4Li54LmJ4LiK".to_string(), // Shorter encoded for testing
            reading_agent_prompt: "IyBTaW1wbGlmaWVkIFJlYWRpbmcgQWdlbnQgUHJvbXB0IFRlbXBsYXRl".to_string(), // Sample encoded prompt
            question_filter_version: "v1".to_string(),
            question_analyzer_version: "v1".to_string(),
            reading_agent_version: "v1".to_string(),
        }
    }
}
