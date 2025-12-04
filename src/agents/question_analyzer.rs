//! Question Analyzer Agent
//!
//! Analyzes user questions to extract key information and context for tarot readings.
//! Uses Gemini API to provide structured analysis including categories, intent, and keywords.
//! Part of the LangGraph-style agent workflow.

use crate::config::env::EnvironmentConfig;
use crate::models::question_analyzer::QuestionAnalyzerResponse;
use crate::models::prompt::PromptRenderContext;
use crate::utils::gemini::{GeminiClient, GeminiError};
use crate::utils::prompt_manager::{PromptManager, PromptError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// Error types for question analysis
#[derive(Debug, Error)]
pub enum QuestionAnalyzerError {
    #[error("Gemini API error: {0}")]
    ApiError(#[from] GeminiError),
    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Failed to load or render prompt template: {0}")]
    TemplateError(#[from] PromptError),
    #[error("Response validation failed: {message}")]
    ValidationError { message: String },
    #[error("Question analysis failed: {reason}")]
    AnalysisFailed { reason: String },
    #[error("Empty question provided")]
    EmptyQuestion,
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Structured question analysis result (Legacy - kept for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnalysis {
    /// Primary category of the question
    pub category: String,
    /// Secondary category if applicable
    pub secondary_category: Option<String>,
    /// User's intent (advice-seeking, prediction, guidance, etc.)
    pub intent: String,
    /// Key themes and topics identified
    pub themes: Vec<String>,
    /// Important keywords extracted
    pub keywords: Vec<String>,
    /// Emotional tone or sentiment
    pub emotion: String,
    /// Time context (past, present, future)
    pub time_context: String,
    /// Specific entities or people mentioned
    pub entities: Vec<String>,
    /// Additional context insights
    pub context: String,
    /// Confidence score of the analysis
    pub confidence: f32,
}

/// Analysis summary for logging/metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Emotional mood from analysis
    pub mood: String,
    /// Main topic from analysis
    pub topic: String,
    /// Time period from analysis
    pub period: String,
    /// Question length for metrics
    pub question_length: usize,
    /// Whether question contains Thai characters
    pub has_thai_characters: bool,
}

/// Question Analyzer Agent
#[derive(Debug, Clone)]
pub struct QuestionAnalyzer {
    client: GeminiClient,
    prompt_manager: Arc<PromptManager>,
}

impl QuestionAnalyzer {
    /// Create a new QuestionAnalyzer instance
    pub async fn new() -> Result<Self, QuestionAnalyzerError> {
        let config = EnvironmentConfig::from_env()
            .map_err(|e| QuestionAnalyzerError::ConfigError(e.to_string()))?;

        let prompt_manager = PromptManager::new(config);

        Ok(Self {
            client: GeminiClient::new()?,
            prompt_manager: Arc::new(prompt_manager),
        })
    }

    /// Analyze a user question with new structured response format
    pub async fn analyze_question(
        &self,
        question: &str,
    ) -> Result<QuestionAnalyzerResponse, QuestionAnalyzerError> {
        // Validate input
        if question.trim().is_empty() {
            return Err(QuestionAnalyzerError::EmptyQuestion);
        }

        // Use Gemini API with Thai prompt for analysis
        self.analyze_question_content(question).await
    }

    /// Build context for template rendering
    async fn build_analysis_context(&self, question: &str) -> PromptRenderContext {
        PromptRenderContext::new(question.trim().to_string())
    }

    /// Analyze question content using Thai prompts and structured responses
    async fn analyze_question_content(&self, question: &str) -> Result<QuestionAnalyzerResponse, QuestionAnalyzerError> {
        // Build context for template rendering
        let context = self.build_analysis_context(question).await;

        // Load and render Thai prompt template
        let template = self.prompt_manager.load_prompt("question_analyzer")
            .map_err(QuestionAnalyzerError::TemplateError)?;
        let formatted_prompt = self.prompt_manager.render_template(&template, &context)
            .map_err(QuestionAnalyzerError::TemplateError)?;

        // Call Gemini API with new prompt
        let response = self.client.generate_text(&formatted_prompt).await?;

        // Parse and validate JSON response
        let analysis_response: QuestionAnalyzerResponse = serde_json::from_str(&response)
            .map_err(QuestionAnalyzerError::JsonParseError)?;

        // Validate response against allowed values
        analysis_response.validate()
            .map_err(|e| QuestionAnalyzerError::ValidationError { message: e.to_string() })?;

        Ok(analysis_response)
    }

    /// Quick analysis returning only mood (for filtering)
    pub async fn get_question_mood(&self, question: &str) -> Result<String, QuestionAnalyzerError> {
        let response = self.analyze_question(question).await?;
        Ok(response.mood)
    }

  /// Get analysis summary for logging/metrics
    pub async fn get_analysis_summary(&self, question: &str) -> Result<AnalysisSummary, QuestionAnalyzerError> {
        let response = self.analyze_question(question).await?;
        Ok(AnalysisSummary {
            mood: response.mood,
            topic: response.topic,
            period: response.period,
            question_length: question.len(),
            has_thai_characters: self.contains_thai_text(question),
        })
    }

    /// Check if text contains Thai characters
    fn contains_thai_text(&self, text: &str) -> bool {
        text.chars().any(|c| {
            let code_point = c as u32;
            (0x0E00..=0x0E7F).contains(&code_point)
        })
    }

    /// Get fallback analysis when API fails
    #[allow(dead_code)] // Reserved for future use in error handling
    async fn get_fallback_analysis(&self, question: &str) -> QuestionAnalyzerResponse {
        // Provide fallback analysis if API fails
        let mood = if self.contains_thai_text(question) {
            "อยากรู้" // Default for Thai questions
        } else {
            "curious" // Default for English questions - but this shouldn't happen in Thai system
        }.to_string();

        QuestionAnalyzerResponse {
            mood,
            topic: "การตัดสินใจ".to_string(),
            period: "ไม่ระบุ".to_string(),
        }
    }

    /// Get basic categorization (lightweight version) - Legacy compatibility
    pub async fn categorize_question(
        &self,
        question: &str,
    ) -> Result<String, QuestionAnalyzerError> {
        let response = self.analyze_question(question).await?;
        // Map Thai topics to English categories for backward compatibility
        match response.topic.as_str() {
            "ความรักและความสัมพันธ์" => Ok("love".to_string()),
            "การงานและอาชีพ" => Ok("career".to_string()),
            "การเงิน" => Ok("finance".to_string()),
            "สุขภาพ (ภาพรวม)" => Ok("health".to_string()),
            "ครอบครัว" => Ok("family".to_string()),
            "การพัฒนาตนเอง" => Ok("spirituality".to_string()),
            _ => Ok("general".to_string()),
        }
    }

    /// Extract keywords from question - Legacy compatibility (returns empty for now)
    pub async fn extract_keywords(
        &self,
        _question: &str,
    ) -> Result<Vec<String>, QuestionAnalyzerError> {
        // Legacy method - new system doesn't extract keywords
        // Return empty vector for backward compatibility
        Ok(vec![])
    }

    /// Detect emotional tone - Legacy compatibility
    pub async fn detect_emotion(&self, question: &str) -> Result<String, QuestionAnalyzerError> {
        let response = self.analyze_question(question).await?;
        // Return Thai mood directly
        Ok(response.mood)
    }

    /// Legacy analyze_question for backward compatibility
    pub async fn analyze_question_legacy(
        &self,
        question: &str,
    ) -> Result<QuestionAnalysis, QuestionAnalyzerError> {
        let response = self.analyze_question(question).await?;

        // Convert new response to legacy format
        Ok(QuestionAnalysis {
            category: self.categorize_question(question).await?,
            secondary_category: None,
            intent: "advice".to_string(), // Default intent
            themes: vec![response.topic.clone()],
            keywords: vec![],
            emotion: response.mood.clone(),
            time_context: response.period.clone(),
            entities: vec![],
            context: format!("คำถามเกี่ยวกับ {}", response.topic),
            confidence: 0.8, // Default confidence
        })
    }
}

impl Default for QuestionAnalyzer {
    fn default() -> Self {
        // Use tokio::task::block_in_place for async in Default
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Self::new().await.expect("GEMINI_API_KEY and environment must be set")
            })
        })
    }
}

/// Legacy function for backward compatibility
pub async fn analyze_question(question: &str) -> Result<String, String> {
    let analyzer = QuestionAnalyzer::new().await.map_err(|e| e.to_string())?;

    let analysis = analyzer
        .analyze_question_legacy(question)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&analysis).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_analyze_question_new_format() {
        let analyzer = QuestionAnalyzer::new().await.unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let result = analyzer.analyze_question(question).await;

        assert!(result.is_ok());
        let analysis = result.unwrap();

        assert!(!analysis.mood.is_empty());
        assert!(!analysis.topic.is_empty());
        assert!(!analysis.period.is_empty());
        assert!(analysis.contains_thai_chars());
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_categorize_question() {
        let analyzer = QuestionAnalyzer::new().await.unwrap();

        let love_question = "ควรจะแต่งงานกับคนนี้ไหม";
        let category_result = analyzer.categorize_question(love_question).await;

        assert!(category_result.is_ok());
        let category = category_result.unwrap();
        assert!(category.contains("love"));
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_extract_keywords() {
        let analyzer = QuestionAnalyzer::new().await.unwrap();

        let question = "ฉันควรเปลี่ยนงานไหม เพราะตอนนี้ทำงานเหนื่อยมาก";
        let keywords = analyzer.extract_keywords(question).await;

        assert!(keywords.is_ok());
        let keyword_list = keywords.unwrap();
        // Legacy method returns empty vector now
        assert_eq!(keyword_list.len(), 0);
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_get_question_mood() {
        let analyzer = QuestionAnalyzer::new().await.unwrap();

        let question = "ฉันกังวลเกี่ยวกับอนาคต";
        let mood_result = analyzer.get_question_mood(question).await;

        assert!(mood_result.is_ok());
        let mood = mood_result.unwrap();
        assert!(!mood.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_get_analysis_summary() {
        let analyzer = QuestionAnalyzer::new().await.unwrap();

        let question = "คำถามภาษาไทยเกี่ยวกับการงาน";
        let summary_result = analyzer.get_analysis_summary(question).await;

        assert!(summary_result.is_ok());
        let summary = summary_result.unwrap();
        assert_eq!(summary.question_length, question.len());
        assert!(summary.has_thai_characters);
        assert!(!summary.mood.is_empty());
        assert!(!summary.topic.is_empty());
        assert!(!summary.period.is_empty());
    }

    #[test]
    fn test_contains_thai_text() {
        // Test the contains_thai_text helper function directly
        fn contains_thai_text(text: &str) -> bool {
            text.chars().any(|c| {
                let code_point = c as u32;
                (0x0E00..=0x0E7F).contains(&code_point)
            })
        }

        assert!(contains_thai_text("สวัสดีครับ"));
        assert!(contains_thai_text("Hello สวัสดี"));
        assert!(!contains_thai_text("Hello World"));
        assert!(!contains_thai_text(""));
    }

    #[test]
    fn test_question_analysis_serialization() {
        let analysis = QuestionAnalysis {
            category: "career".to_string(),
            secondary_category: Some("finance".to_string()),
            intent: "advice".to_string(),
            themes: vec![
                "career_change".to_string(),
                "financial_stability".to_string(),
            ],
            keywords: vec!["เปลี่ยนงาน".to_string(), "เหนื่อย".to_string()],
            emotion: "tired".to_string(),
            time_context: "present".to_string(),
            entities: vec![],
            context: "Considering career change due to burnout".to_string(),
            confidence: 0.85,
        };

        // Test serialization
        let json_str = serde_json::to_string(&analysis).unwrap();
        let deserialized: QuestionAnalysis = serde_json::from_str(&json_str).unwrap();

        assert_eq!(analysis.category, deserialized.category);
        assert_eq!(analysis.intent, deserialized.intent);
        assert_eq!(analysis.keywords, deserialized.keywords);
        assert!((analysis.confidence - deserialized.confidence).abs() < 0.001);
    }

    #[tokio::test]
    #[ignore] // Requires environment variables to run
    async fn test_fallback_analysis() {
        let analyzer = QuestionAnalyzer::new().await.unwrap();

        let thai_question = "ฉันควรทำอะไร";
        let fallback = analyzer.get_fallback_analysis(thai_question).await;

        assert_eq!(fallback.mood, "อยากรู้");
        assert_eq!(fallback.topic, "การตัดสินใจ");
        assert_eq!(fallback.period, "ไม่ระบุ");
    }

    #[test]
    fn test_fallback_analysis_response() {
        // Test the fallback response structure directly without creating analyzer
        let thai_question = "ฉันควรทำอะไร";

        // Simulate fallback response logic
        let contains_thai = thai_question.chars().any(|c| {
            let code_point = c as u32;
            (0x0E00..=0x0E7F).contains(&code_point)
        });

        let mood = if contains_thai {
            "อยากรู้"
        } else {
            "curious"
        }.to_string();

        let response = QuestionAnalyzerResponse::new(
            mood,
            "การตัดสินใจ".to_string(),
            "ไม่ระบุ".to_string(),
        );

        assert_eq!(response.mood, "อยากรู้");
        assert_eq!(response.topic, "การตัดสินใจ");
        assert_eq!(response.period, "ไม่ระบุ");
    }
}
