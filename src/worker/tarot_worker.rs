//! Tarot Worker Implementation
//!
//! Background worker for processing tarot reading jobs from the queue.
//! Integrates with AI pipeline to generate readings and manages job lifecycle.

use crate::{services::AIPipelineService, utils::gemini::GeminiError};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::info;

/// Error types for worker operations
#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("AI Pipeline error: {0}")]
    PipelineError(String),
    #[error("Gemini API error: {0}")]
    GeminiApiError(#[from] GeminiError),
    #[error("Invalid job data: {reason}")]
    InvalidJobData { reason: String },
}

/// Tarot Worker for processing tarot reading jobs
pub struct TarotWorker {
    worker_id: String,
    ai_pipeline: AIPipelineService,
    #[allow(dead_code)]
    poll_interval: Duration,
}

impl TarotWorker {
    /// Create a new TarotWorker instance (synchronous version)
    pub fn new(worker_id: String) -> Result<Self, WorkerError> {
        Ok(Self {
            worker_id,
            ai_pipeline: AIPipelineService::default(),
            poll_interval: Duration::from_secs(5),
        })
    }

    /// Create a new TarotWorker instance (async version to avoid runtime conflicts)
    pub async fn new_async(worker_id: String) -> Result<Self, WorkerError> {
        let ai_pipeline = AIPipelineService::new()
            .await
            .map_err(|e| WorkerError::PipelineError(e.to_string()))?;

        Ok(Self {
            worker_id,
            ai_pipeline,
            poll_interval: Duration::from_secs(5),
        })
    }

    /// Create a new TarotWorker instance with prompt cache (preferred for database prompts)
    pub async fn with_prompt_cache(
        worker_id: String,
        prompt_cache: Arc<HashMap<String, String>>,
    ) -> Result<Self, WorkerError> {
        let ai_pipeline = AIPipelineService::with_prompt_cache(prompt_cache)
            .await
            .map_err(|e| WorkerError::PipelineError(e.to_string()))?;

        Ok(Self {
            worker_id,
            ai_pipeline,
            poll_interval: Duration::from_secs(5),
        })
    }

    /// Create a new TarotWorker instance with fallback (database cache + environment fallback)
    pub async fn with_fallback(
        worker_id: String,
        prompt_cache: Arc<HashMap<String, String>>,
    ) -> Result<Self, WorkerError> {
        let ai_pipeline = AIPipelineService::with_fallback(prompt_cache)
            .await
            .map_err(|e| WorkerError::PipelineError(e.to_string()))?;

        Ok(Self {
            worker_id,
            ai_pipeline,
            poll_interval: Duration::from_secs(5),
        })
    }

    /// Process a tarot reading job through AI pipeline
    pub async fn process_reading_job(
        &mut self,
        question: &str,
        card_count: u32,
    ) -> Result<Value, WorkerError> {
        info!(
            "Processing reading job: {} with {} cards",
            question, card_count
        );

        // Validate card count
        if card_count != 3 && card_count != 5 {
            return Err(WorkerError::InvalidJobData {
                reason: format!("Invalid card count: {}. Must be 3 or 5", card_count),
            });
        }

        // Process through AI pipeline
        let pipeline_result = self
            .ai_pipeline
            .process_tarot_reading(question, card_count)
            .await
            .map_err(|e| WorkerError::PipelineError(e.to_string()))?;

        // Create result payload
        let result_payload = serde_json::json!({
            "question": pipeline_result.question,
            "cards": pipeline_result.cards,
            "question_analysis": pipeline_result.question_analysis,
            "reading": pipeline_result.reading,
            "metadata": {
                "processing_time_ms": pipeline_result.metadata.processing_time_ms,
                "card_count": pipeline_result.metadata.card_count,
                "category": pipeline_result.metadata.category,
                "intent": pipeline_result.metadata.intent,
                "model": pipeline_result.metadata.model,
                "worker_id": self.worker_id,
                "processed_at": pipeline_result.metadata.timestamp
            }
        });

        info!("Successfully processed reading job for: {}", question);
        Ok(result_payload)
    }
}

/// Worker statistics
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub worker_id: String,
    pub jobs_processed: u64,
    pub poll_interval_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        let worker = TarotWorker::new("test-worker".to_string());
        assert!(worker.is_ok());
    }

    #[test]
    fn test_worker_stats() {
        let stats = WorkerStats {
            worker_id: "test-worker".to_string(),
            jobs_processed: 10,
            poll_interval_ms: 5000,
        };

        assert_eq!(stats.worker_id, "test-worker");
        assert_eq!(stats.jobs_processed, 10);
        assert_eq!(stats.poll_interval_ms, 5000);
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_process_reading_job() {
        let mut worker = TarotWorker::new("test-worker".to_string()).unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let card_count = 3;

        let result = worker.process_reading_job(question, card_count).await;

        assert!(result.is_ok());
        let result_payload = result.unwrap();

        assert_eq!(result_payload["question"], question);
        assert_eq!(result_payload["metadata"]["card_count"], card_count);
        assert_eq!(result_payload["metadata"]["worker_id"], "test-worker");
    }

    #[tokio::test]
    async fn test_invalid_card_count() {
        let mut worker = TarotWorker::new("test-worker".to_string()).unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let invalid_card_count = 4; // Must be 3 or 5

        let result = worker
            .process_reading_job(question, invalid_card_count)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WorkerError::InvalidJobData { reason } => {
                assert!(reason.contains("Invalid card count"));
            }
            _ => panic!("Expected InvalidJobData error"),
        }
    }
}
