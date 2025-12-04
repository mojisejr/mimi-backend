//! Prompt Management
//!
//! Loads and manages system prompts for different agent stages using base64 encoded
//! templates stored in environment variables with dynamic template rendering.

use crate::config::env::EnvironmentConfig;
use crate::models::prompt::PromptRenderContext;
use base64::{engine::general_purpose, Engine as _};
use thiserror::Error;

/// Error types for prompt management
#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Missing prompt environment variable: {variable}")]
    MissingEnvironment { variable: String },

    #[error("Failed to decode base64 prompt: {variable}")]
    Base64Decode {
        variable: String,
        #[source]
        source: base64::DecodeError,
    },

    #[error("Prompt template error: {message}")]
    TemplateError { message: String },

    #[error("Invalid template placeholder: {placeholder}")]
    InvalidPlaceholder { placeholder: String },

    #[error("Empty prompt template for agent: {agent_name}")]
    EmptyTemplate { agent_name: String },
}

/// Prompt Manager for loading and rendering encoded prompt templates
#[derive(Debug, Clone)]
pub struct PromptManager {
    config: EnvironmentConfig,
}

impl PromptManager {
    /// Create a new PromptManager instance with the given configuration
    pub fn new(config: EnvironmentConfig) -> Self {
        Self { config }
    }

    /// Load and decode a prompt for the specified agent
    ///
    /// # Arguments
    ///
    /// * `agent_name` - Name of the agent ("question_filter", "question_analyzer", "reading_agent")
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Decoded prompt template
    /// * `Err(PromptError)` - Error loading or decoding the prompt
    pub fn load_prompt(&self, agent_name: &str) -> Result<String, PromptError> {
        let (encoded_prompt, env_var_name) = match agent_name {
            "question_filter" => (
                &self.config.question_filter_prompt,
                "QUESTION_FILTER_PROMPT",
            ),
            "question_analyzer" => (
                &self.config.question_analyzer_prompt,
                "QUESTION_ANALYZER_PROMPT",
            ),
            "reading_agent" => (&self.config.reading_agent_prompt, "READING_AGENT_PROMPT"),
            _ => {
                return Err(PromptError::TemplateError {
                    message: format!("Unknown agent: {}", agent_name),
                })
            }
        };

        if encoded_prompt.is_empty() {
            return Err(PromptError::MissingEnvironment {
                variable: env_var_name.to_string(),
            });
        }

        self.decode_base64(encoded_prompt).map_err(|e| match e {
            PromptError::Base64Decode { source, .. } => PromptError::Base64Decode {
                variable: env_var_name.to_string(),
                source,
            },
            _ => e,
        })
    }

    /// Render a template with the given context
    ///
    /// # Arguments
    ///
    /// * `template` - The prompt template with placeholders
    /// * `context` - Context containing values for placeholder substitution
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Rendered prompt with all placeholders substituted
    /// * `Err(PromptError)` - Error during template rendering
    pub fn render_template(
        &self,
        template: &str,
        context: &PromptRenderContext,
    ) -> Result<String, PromptError> {
        if template.is_empty() {
            return Err(PromptError::TemplateError {
                message: "Template cannot be empty".to_string(),
            });
        }

        let placeholder_map = context.as_placeholder_map();
        let mut result = template.to_string();

        // Replace all placeholders with their values
        for (placeholder, value) in placeholder_map {
            let placeholder_pattern = format!("{{{}}}", placeholder);
            result = result.replace(&placeholder_pattern, &value);
        }

        // Use simple validation for remaining placeholders
        self.validate_no_invalid_placeholders(&result)?;

        Ok(result)
    }

    /// Decode a base64 encoded string
    ///
    /// # Arguments
    ///
    /// * `encoded` - Base64 encoded string
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Decoded string
    /// * `Err(PromptError)` - Base64 decoding error
    fn decode_base64(&self, encoded: &str) -> Result<String, PromptError> {
        general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| PromptError::Base64Decode {
                variable: "decode".to_string(),
                source: e,
            })
            .and_then(|bytes| {
                String::from_utf8(bytes).map_err(|e| PromptError::TemplateError {
                    message: format!("Invalid UTF-8 in decoded prompt: {}", e),
                })
            })
    }

    /// Get the version of a prompt for migration management
    ///
    /// # Arguments
    ///
    /// * `agent_name` - Name of the agent
    ///
    /// # Returns
    ///
    /// * `String` - Version string (e.g., "v1")
    pub fn get_prompt_version(&self, agent_name: &str) -> String {
        match agent_name {
            "question_filter" => self.config.question_filter_version.clone(),
            "question_analyzer" => self.config.question_analyzer_version.clone(),
            "reading_agent" => self.config.reading_agent_version.clone(),
            _ => "unknown".to_string(),
        }
    }

    /// Validate that no invalid placeholders remain in the template (simple approach)
    ///
    /// # Arguments
    ///
    /// * `rendered` - The rendered template after placeholder substitution
    ///
    /// # Returns
    ///
    /// * `Ok(())` - No invalid placeholders found
    /// * `Err(PromptError)` - Invalid placeholder found
    fn validate_no_invalid_placeholders(&self, rendered: &str) -> Result<(), PromptError> {
        let valid_placeholders = ["question", "mood", "topic", "period", "cards"];

        let mut chars = rendered.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Find the closing brace
                let mut placeholder = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume the closing brace
                        break;
                    }
                    placeholder.push(chars.next().unwrap());
                }

                if !placeholder.is_empty() && !valid_placeholders.contains(&placeholder.as_str()) {
                    return Err(PromptError::InvalidPlaceholder {
                        placeholder: placeholder.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::env::{Environment, EnvironmentConfig, QueuePoolConfig};

    fn create_test_config() -> EnvironmentConfig {
        EnvironmentConfig {
            environment: Environment::Development,
            pool: QueuePoolConfig::development(),
            redis_url: Some("redis://localhost:6379".to_string()),
            upstash_url: Some("https://test-upstash.com".to_string()),
            upstash_token: Some("test-token".to_string()),
            stream_key: "test:stream".to_string(),
            consumer_group: "test-consumers".to_string(),
            question_filter_prompt: "SGVsbG8gV29ybGQ=".to_string(), // "Hello World"
            question_analyzer_prompt: "VGVzdCBwcm9tcHQ=".to_string(), // "Test prompt"
            reading_agent_prompt: "UmVhZGluZyBhZ2VudCBwcm9tcHQ=".to_string(), // "Reading agent prompt"
            question_filter_version: "v1".to_string(),
            question_analyzer_version: "v1".to_string(),
            reading_agent_version: "v1".to_string(),
        }
    }

    #[test]
    fn test_decode_base64_success() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let result = prompt_manager.decode_base64("SGVsbG8gV29ybGQ=");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");
    }

    #[test]
    fn test_decode_base64_invalid() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let result = prompt_manager.decode_base64("Invalid@Base64!!!");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PromptError::Base64Decode { .. }
        ));
    }

    #[test]
    fn test_load_prompt_question_filter() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let result = prompt_manager.load_prompt("question_filter");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");
    }

    #[test]
    fn test_load_prompt_invalid_agent() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let result = prompt_manager.load_prompt("invalid_agent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PromptError::TemplateError { .. }
        ));
    }

    #[test]
    fn test_load_prompt_missing_env_var() {
        let mut config = create_test_config();
        config.question_filter_prompt = String::new();
        let prompt_manager = PromptManager::new(config);

        let result = prompt_manager.load_prompt("question_filter");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PromptError::MissingEnvironment { .. }
        ));
    }

    #[test]
    fn test_render_template_single_placeholder() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "Hello {question}!";
        let context = PromptRenderContext::new("world".to_string());

        let result = prompt_manager.render_template(template, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello world!");
    }

    #[test]
    fn test_render_template_multiple_placeholders() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "Question: {question}, Mood: {mood}, Topic: {topic}";
        let context = PromptRenderContext::new("test".to_string())
            .with_mood("happy".to_string())
            .with_topic("life".to_string());

        let result = prompt_manager.render_template(template, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Question: test, Mood: happy, Topic: life");
    }

    #[test]
    fn test_render_template_invalid_placeholder() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "Hello {invalid_placeholder}!";
        let context = PromptRenderContext::new("world".to_string());

        let result = prompt_manager.render_template(template, &context);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PromptError::InvalidPlaceholder { .. }
        ));
    }

    #[test]
    fn test_render_template_empty_template() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        let template = "";
        let context = PromptRenderContext::new("test".to_string());

        let result = prompt_manager.render_template(template, &context);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PromptError::TemplateError { .. }
        ));
    }

    #[test]
    fn test_get_prompt_version() {
        let config = create_test_config();
        let prompt_manager = PromptManager::new(config);

        assert_eq!(prompt_manager.get_prompt_version("question_filter"), "v1");
        assert_eq!(prompt_manager.get_prompt_version("question_analyzer"), "v1");
        assert_eq!(prompt_manager.get_prompt_version("reading_agent"), "v1");
        assert_eq!(prompt_manager.get_prompt_version("unknown"), "unknown");
    }
}
