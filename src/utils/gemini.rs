//! Google Gemini API Client
//!
//! Provides interface for communicating with Google Gemini LLM API.
//! Supports text generation with proper error handling and structured output.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

/// Errors for Gemini API operations
#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("API key not found in environment variables")]
    MissingApiKey,
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("Failed to parse API response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("No response content from API")]
    EmptyResponse,
    #[error("API response indicates blocked content: {reason}")]
    BlockedContent { reason: String },
}

/// Gemini API request structure
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: Option<GenerationConfig>,
    safety_settings: Option<Vec<SafetySetting>>,
}

/// Content structure for Gemini API
#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
    role: Option<String>,
}

/// Part structure for Gemini API content
#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

/// Generation configuration for Gemini API
#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    top_k: i32,
    top_p: f32,
    max_output_tokens: i32,
}

/// Safety setting for Gemini API
#[derive(Debug, Serialize)]
struct SafetySetting {
    category: String,
    threshold: String,
}

/// Gemini API response structure
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    prompt_feedback: Option<PromptFeedback>,
}

/// Candidate response from Gemini API
#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
    finish_reason: Option<String>,
    #[allow(dead_code)]
    index: i32,
    #[allow(dead_code)]
    safety_ratings: Vec<SafetyRating>,
}

/// Safety rating from Gemini API
#[derive(Debug, Deserialize)]
struct SafetyRating {
    #[allow(dead_code)]
    category: String,
    #[allow(dead_code)]
    blocked: bool,
}

/// Feedback about the prompt from Gemini API
#[derive(Debug, Deserialize)]
struct PromptFeedback {
    block_reason: Option<String>,
    #[allow(dead_code)]
    safety_ratings: Vec<SafetyRating>,
}

/// Gemini API client
#[derive(Debug, Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new() -> Result<Self, GeminiError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| GeminiError::MissingApiKey)?;

        Ok(Self {
            client: Client::new(),
            api_key,
            model: "gemini-pro".to_string(), // Default model
        })
    }

    /// Create a new Gemini client with specific model
    pub fn with_model(model: &str) -> Result<Self, GeminiError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| GeminiError::MissingApiKey)?;

        Ok(Self {
            client: Client::new(),
            api_key,
            model: model.to_string(),
        })
    }

    /// Generate text using Gemini API
    pub async fn generate_text(&self, prompt: &str) -> Result<String, GeminiError> {
        self.generate_text_with_config(prompt, None, None).await
    }

    /// Generate text with custom configuration
    pub async fn generate_text_with_config(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<i32>,
    ) -> Result<String, GeminiError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        let generation_config = Some(GenerationConfig {
            temperature: temperature.unwrap_or(0.7),
            top_k: 40,
            top_p: 0.95,
            max_output_tokens: max_tokens.unwrap_or(1024),
        });

        let safety_settings = Some(vec![
            SafetySetting {
                category: "HARM_CATEGORY_HARASSMENT".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
        ]);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
                role: None,
            }],
            generation_config,
            safety_settings,
        };

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GeminiError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let response_body: GeminiResponse = response.json().await?;

        // Check for blocked content
        if let Some(feedback) = response_body.prompt_feedback {
            if let Some(block_reason) = feedback.block_reason {
                return Err(GeminiError::BlockedContent {
                    reason: block_reason,
                });
            }
        }

        // Get the first candidate's content
        let candidate = response_body
            .candidates
            .into_iter()
            .next()
            .ok_or(GeminiError::EmptyResponse)?;

        // Check finish reason
        if let Some(reason) = candidate.finish_reason {
            if reason == "SAFETY" {
                return Err(GeminiError::BlockedContent {
                    reason: "Content blocked by safety filters".to_string(),
                });
            }
        }

        let text = candidate
            .content
            .parts
            .into_iter()
            .next()
            .ok_or(GeminiError::EmptyResponse)?
            .text;

        if text.trim().is_empty() {
            return Err(GeminiError::EmptyResponse);
        }

        Ok(text)
    }

    /// Generate JSON structured output
    pub async fn generate_json(&self, prompt: &str) -> Result<serde_json::Value, GeminiError> {
        let json_prompt = format!(
            "{}\n\nPlease respond with valid JSON only, without any additional text.",
            prompt
        );

        let response = self.generate_text(&json_prompt).await?;

        // Try to parse as JSON
        let json_value: serde_json::Value = serde_json::from_str(&response)?;
        Ok(json_value)
    }

    /// Get the model name being used
    pub fn model(&self) -> &str {
        &self.model
    }
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self::new().expect("GEMINI_API_KEY must be set in environment variables")
    }
}

/// Legacy function for backward compatibility
pub async fn call_gemini(prompt: &str) -> Result<String, String> {
    let client = GeminiClient::new().map_err(|e| e.to_string())?;

    client
        .generate_text(prompt)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_client_missing_api_key() {
        // Temporarily clear the API key for testing
        let original_key = env::var("GEMINI_API_KEY").ok();
        env::remove_var("GEMINI_API_KEY");

        let result = GeminiClient::new();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GeminiError::MissingApiKey));

        // Restore original key if it existed
        if let Some(key) = original_key {
            env::set_var("GEMINI_API_KEY", key);
        }
    }

    #[test]
    fn test_gemini_client_with_model() {
        // Temporarily set API key for testing
        let original_key = env::var("GEMINI_API_KEY").ok();
        env::set_var("GEMINI_API_KEY", "test-key");

        let client = GeminiClient::with_model("gemini-pro-vision").unwrap();
        assert_eq!(client.model(), "gemini-pro-vision");

        // Restore original key if it existed
        if let Some(key) = original_key {
            env::set_var("GEMINI_API_KEY", key);
        } else {
            env::remove_var("GEMINI_API_KEY");
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default to avoid actual API calls during testing
    async fn test_generate_text_real_api() {
        // This test requires a real API key to run
        let client = GeminiClient::new().unwrap();

        let prompt = "What is 2 + 2?";
        let result = client.generate_text(prompt).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("4") || response.contains("four"));
    }
}
