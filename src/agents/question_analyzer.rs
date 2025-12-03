//! Question Analyzer Agent
//!
//! Analyzes user questions to extract key information and context for tarot readings.
//! Uses Gemini API to provide structured analysis including categories, intent, and keywords.
//! Part of the LangGraph-style agent workflow.

use crate::utils::gemini::{GeminiClient, GeminiError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for question analysis
#[derive(Debug, Error)]
pub enum QuestionAnalyzerError {
    #[error("Gemini API error: {0}")]
    ApiError(#[from] GeminiError),
    #[error("Failed to parse analysis response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Question analysis failed: {reason}")]
    AnalysisFailed { reason: String },
}

/// Structured question analysis result
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

/// Question Analyzer Agent
#[derive(Debug, Clone)]
pub struct QuestionAnalyzer {
    client: GeminiClient,
}

impl QuestionAnalyzer {
    /// Create a new QuestionAnalyzer instance
    pub fn new() -> Result<Self, QuestionAnalyzerError> {
        Ok(Self {
            client: GeminiClient::new()?,
        })
    }

    /// Analyze a user question and return structured information
    pub async fn analyze_question(
        &self,
        question: &str,
    ) -> Result<QuestionAnalysis, QuestionAnalyzerError> {
        let prompt = self.build_analysis_prompt(question);

        let response = self.client.generate_text(&prompt).await?;

        // Try to parse as JSON first
        match serde_json::from_str::<QuestionAnalysis>(&response) {
            Ok(analysis) => Ok(analysis),
            Err(_) => {
                // If JSON parsing fails, try to extract structured info manually
                self.fallback_analysis(question, &response).await
            }
        }
    }

    /// Build the analysis prompt for Gemini
    fn build_analysis_prompt(&self, question: &str) -> String {
        format!(
            r#"Analyze this tarot reading question and provide structured insights:

Question: "{}"

Please analyze the question and respond with a JSON object containing:
- category: Primary category (love, career, finance, health, spirituality, family, etc.)
- secondary_category: Secondary category if applicable
- intent: User's primary intent (advice, prediction, guidance, understanding, decision-making)
- themes: Array of key themes or topics
- keywords: Array of important keywords
- emotion: Emotional tone (concerned, hopeful, confused, determined, etc.)
- time_context: Time focus (past, present, future, unspecified)
- entities: Array of specific people, places, or things mentioned
- context: Brief contextual description of what the user is seeking
- confidence: Your confidence in this analysis (0.0-1.0)

Respond only with valid JSON, no additional text."#,
            question.trim()
        )
    }

    /// Fallback analysis when JSON parsing fails
    async fn fallback_analysis(
        &self,
        question: &str,
        response: &str,
    ) -> Result<QuestionAnalysis, QuestionAnalyzerError> {
        // Try a simpler approach with explicit JSON request
        let simple_prompt = format!(
            r#"Extract structured information from this question: "{}"

Respond with this exact JSON format:
{{
  "category": "category_name",
  "intent": "user_intent",
  "themes": ["theme1", "theme2"],
  "keywords": ["keyword1", "keyword2"],
  "emotion": "emotion_tone",
  "time_context": "time_focus",
  "entities": ["entity1", "entity2"],
  "context": "brief_context",
  "confidence": 0.8
}}

Only valid JSON, no other text."#,
            question.trim()
        );

        let fallback_response = self.client.generate_text(&simple_prompt).await?;

        serde_json::from_str::<QuestionAnalysis>(&fallback_response).map_err(|e| {
            QuestionAnalyzerError::AnalysisFailed {
                reason: format!(
                    "Failed to parse AI response: {}. Original response: {}",
                    e, response
                ),
            }
        })
    }

    /// Get basic categorization (lightweight version)
    pub async fn categorize_question(
        &self,
        question: &str,
    ) -> Result<String, QuestionAnalyzerError> {
        let prompt = format!(
            r#"Categorize this tarot question in one word (love, career, finance, health, spirituality, family, general):

Question: "{}"

Respond with only the category word:"#,
            question.trim()
        );

        let response = self.client.generate_text(&prompt).await?;
        Ok(response.trim().to_lowercase())
    }

    /// Extract keywords from question
    pub async fn extract_keywords(
        &self,
        question: &str,
    ) -> Result<Vec<String>, QuestionAnalyzerError> {
        let analysis = self.analyze_question(question).await?;
        Ok(analysis.keywords)
    }

    /// Detect emotional tone
    pub async fn detect_emotion(&self, question: &str) -> Result<String, QuestionAnalyzerError> {
        let analysis = self.analyze_question(question).await?;
        Ok(analysis.emotion)
    }
}

impl Default for QuestionAnalyzer {
    fn default() -> Self {
        Self::new().expect("GEMINI_API_KEY must be set")
    }
}

/// Legacy function for backward compatibility
pub async fn analyze_question(question: &str) -> Result<String, String> {
    let analyzer = QuestionAnalyzer::new().map_err(|e| e.to_string())?;

    let analysis = analyzer
        .analyze_question(question)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&analysis).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_analyze_question() {
        let analyzer = QuestionAnalyzer::new().unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let result = analyzer.analyze_question(question).await;

        assert!(result.is_ok());
        let analysis = result.unwrap();

        assert!(!analysis.category.is_empty());
        assert!(!analysis.intent.is_empty());
        assert!(!analysis.keywords.is_empty());
        assert!(analysis.confidence >= 0.0 && analysis.confidence <= 1.0);
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_categorize_question() {
        let analyzer = QuestionAnalyzer::new().unwrap();

        let love_question = "ควรจะแต่งงานกับคนนี้ไหม";
        let category_result = analyzer.categorize_question(love_question).await;

        assert!(category_result.is_ok());
        let category = category_result.unwrap();
        assert!(category.contains("love") || category.contains("relationship"));
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_extract_keywords() {
        let analyzer = QuestionAnalyzer::new().unwrap();

        let question = "ฉันควรเปลี่ยนงานไหม เพราะตอนนี้ทำงานเหนื่อยมาก";
        let keywords = analyzer.extract_keywords(question).await;

        assert!(keywords.is_ok());
        let keyword_list = keywords.unwrap();
        assert!(!keyword_list.is_empty());
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
}
