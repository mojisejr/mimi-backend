//! AI Pipeline Service
//!
//! Orchestrates the complete AI processing pipeline for tarot readings.
//! Coordinates question filtering, analysis, card selection, and reading generation.
//! Provides a unified interface for the entire AI workflow.

use crate::{
    agents::{
        question_analyzer::QuestionAnalyzer, question_filter::QuestionFilter,
        reading_agent::ReadingAgent,
    },
    models::reading_agent::CardInfo,
    services::CardRandomizer,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Error types for AI pipeline processing
#[derive(Debug, Error)]
pub enum AIPipelineError {
    #[error("Question validation failed: {0}")]
    QuestionValidationFailed(String),
    #[error("Question analysis failed: {0}")]
    QuestionAnalysisFailed(String),
    #[error("Card selection failed: {0}")]
    CardSelectionFailed(String),
    #[error("Reading generation failed: {0}")]
    ReadingGenerationFailed(String),
    #[error("Invalid card count: {count}. Must be 3 or 5")]
    InvalidCardCount { count: u32 },
    #[error("Pipeline configuration error: {reason}")]
    ConfigurationError { reason: String },
}

/// AI Pipeline processing result
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Original question
    pub question: String,
    /// Cards selected for the reading
    pub cards: Vec<String>,
    /// Structured question analysis
    pub question_analysis: serde_json::Value,
    /// Complete reading interpretation
    pub reading: String,
    /// Processing metadata
    pub metadata: PipelineMetadata,
}

/// Pipeline processing metadata
#[derive(Debug, Clone)]
pub struct PipelineMetadata {
    /// Processing duration in milliseconds
    pub processing_time_ms: u64,
    /// Number of cards selected
    pub card_count: u32,
    /// Question category
    pub category: String,
    /// Question intent
    pub intent: String,
    /// AI model used
    pub model: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// AI Pipeline Service
#[derive(Debug, Clone)]
pub struct AIPipelineService {
    question_filter: QuestionFilter,
    question_analyzer: QuestionAnalyzer,
    reading_agent: ReadingAgent,
    card_randomizer: CardRandomizer,
}

impl AIPipelineService {
    /// Create a new AI Pipeline Service
    pub async fn new() -> Result<Self, AIPipelineError> {
        Ok(Self {
            question_filter: QuestionFilter::new().await.map_err(|e| {
                AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create QuestionFilter: {}", e),
                }
            })?,
            question_analyzer: QuestionAnalyzer::new().await.map_err(|e| {
                AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create QuestionAnalyzer: {}", e),
                }
            })?,
            reading_agent: ReadingAgent::new().await.map_err(|e| {
                AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create ReadingAgent: {}", e),
                }
            })?,
            card_randomizer: CardRandomizer::new(),
        })
    }

    /// Create a new AI Pipeline Service with prompt cache (preferred for database prompts)
    pub async fn with_prompt_cache(
        prompt_cache: Arc<HashMap<String, String>>,
    ) -> Result<Self, AIPipelineError> {
        Ok(Self {
            question_filter: QuestionFilter::with_prompt_cache(prompt_cache.clone())
                .await
                .map_err(|e| AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create QuestionFilter: {}", e),
                })?,
            question_analyzer: QuestionAnalyzer::with_prompt_cache(prompt_cache.clone())
                .await
                .map_err(|e| AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create QuestionAnalyzer: {}", e),
                })?,
            reading_agent: ReadingAgent::with_prompt_cache(prompt_cache)
                .await
                .map_err(|e| AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create ReadingAgent: {}", e),
                })?,
            card_randomizer: CardRandomizer::new(),
        })
    }

    /// Create a new AI Pipeline Service with fallback (database cache + environment fallback)
    pub async fn with_fallback(
        prompt_cache: Arc<HashMap<String, String>>,
    ) -> Result<Self, AIPipelineError> {
        Ok(Self {
            question_filter: QuestionFilter::with_fallback(prompt_cache.clone())
                .await
                .map_err(|e| AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create QuestionFilter: {}", e),
                })?,
            question_analyzer: QuestionAnalyzer::with_fallback(prompt_cache.clone())
                .await
                .map_err(|e| AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create QuestionAnalyzer: {}", e),
                })?,
            reading_agent: ReadingAgent::with_fallback(prompt_cache)
                .await
                .map_err(|e| AIPipelineError::ConfigurationError {
                    reason: format!("Failed to create ReadingAgent: {}", e),
                })?,
            card_randomizer: CardRandomizer::new(),
        })
    }

    /// Process a complete tarot reading through the AI pipeline
    pub async fn process_tarot_reading(
        &mut self,
        question: &str,
        card_count: u32,
    ) -> Result<PipelineResult, AIPipelineError> {
        let start_time = std::time::Instant::now();

        // Validate card count
        if card_count != 3 && card_count != 5 {
            return Err(AIPipelineError::InvalidCardCount { count: card_count });
        }

        // Step 1: Filter and validate the question
        let validated_question = self
            .question_filter
            .filter_question(question)
            .await
            .map_err(|e| AIPipelineError::QuestionValidationFailed(e.to_string()))?;

        // Step 2: Analyze the question
        let _question_analysis = self
            .question_analyzer
            .analyze_question(&validated_question)
            .await
            .map_err(|e| AIPipelineError::QuestionAnalysisFailed(e.to_string()))?;

        // Convert new QuestionAnalyzerResponse to legacy format for compatibility
        let legacy_analysis = self
            .question_analyzer
            .analyze_question_legacy(&validated_question)
            .await
            .map_err(|e| AIPipelineError::QuestionAnalysisFailed(e.to_string()))?;

        let category = legacy_analysis.category.clone();
        let intent = legacy_analysis.intent.clone();

        // Step 3: Select cards
        let cards = self
            .card_randomizer
            .pick_cards(card_count)
            .await
            .map_err(|e| AIPipelineError::CardSelectionFailed(e.to_string()))?;

        // Step 4: Generate reading - convert to new CardInfo format
        let card_infos: Vec<CardInfo> = cards
            .iter()
            .enumerate()
            .map(|(index, card_name)| {
                CardInfo::new(
                    index as u32,
                    card_name,
                    &format!("ไพ่{}", card_name),
                    index as u32,
                )
            })
            .collect();

        let mood = &legacy_analysis.emotion;
        let topic = &legacy_analysis.category;
        let period = "ไม่ระบุ"; // Default period

        let reading = self
            .reading_agent
            .generate_reading(&validated_question, mood, topic, period, &card_infos)
            .await
            .map_err(|e| AIPipelineError::ReadingGenerationFailed(e.to_string()))?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(PipelineResult {
            question: validated_question,
            cards,
            question_analysis: json!(legacy_analysis),
            reading: reading.reading,
            metadata: PipelineMetadata {
                processing_time_ms: processing_time,
                card_count,
                category,
                intent,
                model: "gemini-pro".to_string(),
                timestamp: chrono::Utc::now(),
            },
        })
    }

    /// Process tarot reading with existing cards (re-reading)
    pub async fn process_tarot_rereading(
        &self,
        question: &str,
        cards: &[String],
    ) -> Result<PipelineResult, AIPipelineError> {
        let start_time = std::time::Instant::now();

        // Validate question
        let validated_question = self
            .question_filter
            .filter_question(question)
            .await
            .map_err(|e| AIPipelineError::QuestionValidationFailed(e.to_string()))?;

        // Analyze the question
        let _question_analysis = self
            .question_analyzer
            .analyze_question(&validated_question)
            .await
            .map_err(|e| AIPipelineError::QuestionAnalysisFailed(e.to_string()))?;

        // Convert to legacy format for compatibility
        let legacy_analysis = self
            .question_analyzer
            .analyze_question_legacy(&validated_question)
            .await
            .map_err(|e| AIPipelineError::QuestionAnalysisFailed(e.to_string()))?;

        // Generate reading with provided cards - convert to new CardInfo format
        let card_infos: Vec<CardInfo> = cards
            .iter()
            .enumerate()
            .map(|(index, card_name)| {
                CardInfo::new(
                    index as u32,
                    card_name,
                    &format!("ไพ่{}", card_name),
                    index as u32,
                )
            })
            .collect();

        let mood = &legacy_analysis.emotion;
        let topic = &legacy_analysis.category;
        let period = "ไม่ระบุ"; // Default period

        let reading = self
            .reading_agent
            .generate_reading(&validated_question, mood, topic, period, &card_infos)
            .await
            .map_err(|e| AIPipelineError::ReadingGenerationFailed(e.to_string()))?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(PipelineResult {
            question: validated_question,
            cards: cards.to_vec(),
            question_analysis: json!(legacy_analysis),
            reading: reading.reading,
            metadata: PipelineMetadata {
                processing_time_ms: processing_time,
                card_count: cards.len() as u32,
                category: legacy_analysis.category,
                intent: legacy_analysis.intent,
                model: "gemini-pro".to_string(),
                timestamp: chrono::Utc::now(),
            },
        })
    }

    /// Validate a question only (no reading generation)
    pub async fn validate_question(&self, question: &str) -> Result<(), AIPipelineError> {
        let validation_result = self
            .question_filter
            .validate_question(question)
            .await
            .map_err(|e| AIPipelineError::QuestionValidationFailed(e.to_string()))?;

        if validation_result.is_valid {
            Ok(())
        } else {
            Err(AIPipelineError::QuestionValidationFailed(
                validation_result.reason,
            ))
        }
    }

    /// Analyze a question only (no reading generation)
    pub async fn analyze_question_only(
        &self,
        question: &str,
    ) -> Result<serde_json::Value, AIPipelineError> {
        let validated_question = self
            .question_filter
            .filter_question(question)
            .await
            .map_err(|e| AIPipelineError::QuestionValidationFailed(e.to_string()))?;

        let analysis = self
            .question_analyzer
            .analyze_question(&validated_question)
            .await
            .map_err(|e| AIPipelineError::QuestionAnalysisFailed(e.to_string()))?;

        Ok(json!(analysis))
    }

    /// Generate reading with pre-selected cards
    pub async fn generate_reading_only(
        &self,
        question: &str,
        cards: &[String],
        question_analysis: Option<&serde_json::Value>,
    ) -> Result<String, AIPipelineError> {
        // Use provided analysis or generate new one
        let _analysis = match question_analysis {
            Some(analysis) => analysis.clone(),
            None => {
                let validated_question = self
                    .question_filter
                    .filter_question(question)
                    .await
                    .map_err(|e| AIPipelineError::QuestionValidationFailed(e.to_string()))?;

                let new_analysis = self
                    .question_analyzer
                    .analyze_question(&validated_question)
                    .await
                    .map_err(|e| AIPipelineError::QuestionAnalysisFailed(e.to_string()))?;

                json!(new_analysis)
            }
        };

        let validated_question = self
            .question_filter
            .filter_question(question)
            .await
            .map_err(|e| AIPipelineError::QuestionValidationFailed(e.to_string()))?;

        // Convert to new CardInfo format
        let card_infos: Vec<CardInfo> = cards
            .iter()
            .enumerate()
            .map(|(index, card_name)| {
                CardInfo::new(
                    index as u32,
                    card_name,
                    &format!("ไพ่{}", card_name),
                    index as u32,
                )
            })
            .collect();

        // Extract mood, topic from analysis or use defaults
        let mood = "อยากรู้"; // Default mood
        let topic = "การตัดสินใจ"; // Default topic
        let period = "ไม่ระบุ"; // Default period

        let reading = self
            .reading_agent
            .generate_reading(&validated_question, mood, topic, period, &card_infos)
            .await
            .map_err(|e| AIPipelineError::ReadingGenerationFailed(e.to_string()))?;

        Ok(reading.reading)
    }

    /// Get pipeline statistics
    pub fn get_pipeline_info(&self) -> PipelineInfo {
        PipelineInfo {
            model: "gemini-pro".to_string(),
            supported_card_counts: vec![3, 5],
            features: vec![
                "question_validation".to_string(),
                "question_analysis".to_string(),
                "card_selection".to_string(),
                "reading_generation".to_string(),
                "rereading_support".to_string(),
            ],
        }
    }
}

/// Pipeline information
#[derive(Debug, Clone)]
pub struct PipelineInfo {
    pub model: String,
    pub supported_card_counts: Vec<u32>,
    pub features: Vec<String>,
}

impl Default for AIPipelineService {
    fn default() -> Self {
        // Use block_on for synchronous creation in Default
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();
        rt.block_on(Self::new())
            .expect("Failed to create AIPipelineService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_complete_pipeline() {
        let mut pipeline = AIPipelineService::new().await.unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let card_count = 3;

        let result = pipeline.process_tarot_reading(question, card_count).await;

        assert!(result.is_ok());
        let pipeline_result = result.unwrap();

        assert_eq!(pipeline_result.question, question);
        assert_eq!(pipeline_result.cards.len(), card_count as usize);
        assert!(!pipeline_result.reading.is_empty());
        assert!(pipeline_result.metadata.processing_time_ms > 0);
        assert_eq!(pipeline_result.metadata.card_count, card_count);
        assert!(!pipeline_result.metadata.category.is_empty());
        assert!(!pipeline_result.metadata.intent.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_card_count() {
        let mut pipeline = AIPipelineService::new().await.unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let invalid_card_count = 4; // Must be 3 or 5

        let result = pipeline
            .process_tarot_reading(question, invalid_card_count)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AIPipelineError::InvalidCardCount { count } => assert_eq!(count, 4),
            _ => panic!("Expected InvalidCardCount error"),
        }
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_validate_question() {
        let pipeline = AIPipelineService::new().await.unwrap();

        let valid_question = "ควรจะลงทุนอะไรดีครับ";
        let result = pipeline.validate_question(valid_question).await;
        assert!(result.is_ok());

        let invalid_question = "";
        let result = pipeline.validate_question(invalid_question).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_analyze_question_only() {
        let pipeline = AIPipelineService::new().await.unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let result = pipeline.analyze_question_only(question).await;

        assert!(result.is_ok());
        let analysis = result.unwrap();

        assert!(analysis.get("category").is_some());
        assert!(analysis.get("intent").is_some());
        assert!(analysis.get("keywords").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_rereading() {
        let pipeline = AIPipelineService::new().await.unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let cards = vec![
            "The Fool".to_string(),
            "The Magician".to_string(),
            "The Star".to_string(),
        ];

        let result = pipeline.process_tarot_rereading(question, &cards).await;

        assert!(result.is_ok());
        let pipeline_result = result.unwrap();

        assert_eq!(pipeline_result.question, question);
        assert_eq!(pipeline_result.cards, cards);
        assert!(!pipeline_result.reading.is_empty());
    }

    #[test]
    fn test_pipeline_info() {
        let pipeline = AIPipelineService::default();
        let info = pipeline.get_pipeline_info();

        assert_eq!(info.model, "gemini-pro");
        assert!(info.supported_card_counts.contains(&3));
        assert!(info.supported_card_counts.contains(&5));
        assert!(!info.features.is_empty());
        assert!(info.features.contains(&"question_validation".to_string()));
    }
}
