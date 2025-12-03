//! Tarot Reading Agent
//!
//! Generates tarot readings using Google Gemini API.
//! Produces detailed interpretations of cards based on user questions and analysis.
//! Provides comprehensive tarot readings in Thai language.

use crate::utils::gemini::{GeminiClient, GeminiError};
use serde_json::Value;
use thiserror::Error;

/// Error types for tarot reading generation
#[derive(Debug, Error)]
pub enum ReadingAgentError {
    #[error("Gemini API error: {0}")]
    ApiError(#[from] GeminiError),
    #[error("Invalid cards provided: {reason}")]
    InvalidCards { reason: String },
    #[error("Question analysis is required for reading")]
    MissingAnalysis,
    #[error("Reading generation failed: {reason}")]
    GenerationFailed { reason: String },
}

/// Tarot Reading structure
#[derive(Debug, Clone)]
pub struct TarotReading {
    /// Original question
    pub question: String,
    /// Cards drawn for the reading
    pub cards: Vec<String>,
    /// Question analysis context
    pub question_analysis: Value,
    /// Complete reading interpretation
    pub interpretation: String,
    /// Individual card interpretations
    pub card_interpretations: Vec<CardInterpretation>,
    /// Overall guidance and advice
    pub guidance: String,
    /// Timestamp of the reading
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Individual card interpretation
#[derive(Debug, Clone)]
pub struct CardInterpretation {
    /// Card name
    pub card: String,
    /// Position in the spread (past, present, future, etc.)
    pub position: String,
    /// Card interpretation in context
    pub interpretation: String,
    /// Keywords associated with this card in reading
    pub keywords: Vec<String>,
}

/// Reading Agent
#[derive(Debug, Clone)]
pub struct ReadingAgent {
    client: GeminiClient,
}

impl ReadingAgent {
    /// Create a new ReadingAgent instance
    pub fn new() -> Result<Self, ReadingAgentError> {
        Ok(Self {
            client: GeminiClient::new()?,
        })
    }

    /// Generate a complete tarot reading
    pub async fn generate_reading(
        &self,
        question: &str,
        cards: &[String],
        question_analysis: &Value,
    ) -> Result<TarotReading, ReadingAgentError> {
        // Validate inputs
        self.validate_inputs(question, cards, question_analysis)?;

        // Generate complete interpretation
        let interpretation = self
            .generate_interpretation(question, cards, question_analysis)
            .await?;

        // Generate individual card interpretations
        let card_interpretations = self
            .generate_card_interpretations(cards, question_analysis)
            .await?;

        // Generate overall guidance
        let guidance = self
            .generate_guidance(question, &card_interpretations, question_analysis)
            .await?;

        Ok(TarotReading {
            question: question.to_string(),
            cards: cards.to_vec(),
            question_analysis: question_analysis.clone(),
            interpretation,
            card_interpretations,
            guidance,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Validate inputs for reading generation
    fn validate_inputs(
        &self,
        question: &str,
        cards: &[String],
        question_analysis: &Value,
    ) -> Result<(), ReadingAgentError> {
        if question.trim().is_empty() {
            return Err(ReadingAgentError::InvalidCards {
                reason: "Question cannot be empty".to_string(),
            });
        }

        if cards.is_empty() {
            return Err(ReadingAgentError::InvalidCards {
                reason: "At least one card is required".to_string(),
            });
        }

        if cards.len() > 10 {
            return Err(ReadingAgentError::InvalidCards {
                reason: "Too many cards for a reading (max 10)".to_string(),
            });
        }

        // Check for duplicate cards
        let unique_cards: std::collections::HashSet<_> = cards.iter().collect();
        if unique_cards.len() != cards.len() {
            return Err(ReadingAgentError::InvalidCards {
                reason: "Duplicate cards in reading".to_string(),
            });
        }

        if question_analysis.is_null() {
            return Err(ReadingAgentError::MissingAnalysis);
        }

        Ok(())
    }

    /// Generate complete interpretation
    async fn generate_interpretation(
        &self,
        question: &str,
        cards: &[String],
        question_analysis: &Value,
    ) -> Result<String, ReadingAgentError> {
        let category = question_analysis
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("general");
        let intent = question_analysis
            .get("intent")
            .and_then(|v| v.as_str())
            .unwrap_or("guidance");
        let emotion = question_analysis
            .get("emotion")
            .and_then(|v| v.as_str())
            .unwrap_or("neutral");

        let prompt = format!(
            r#"คุณเป็นหมอดูดวงไพ่ทาโรต์มืออาชีพที่มีประสบการณ์มากมาย โปรดทำนายไพ่ทาโรต์สำหรับคำถามนี้:

คำถาม: "{}"
หมวดหมู่: {}
เจตนาของผู้ถาม: {}
อารมณ์ของผู้ถาม: {}
ไพ่ที่ได้: {}

กรุณาให้คำทำนายโดยมีโครงสร้างดังนี้:
1. บทนำ - สรุปคำถามและสถานการณ์โดยรวม
2. การตีความไพ่แต่ละใบ - อธิบายความหมายของไพ่แต่ละใบในบริบทของคำถาม
3. การเชื่อมโยง - เชื่อมโยงความหมายของไพ่ทั้งหมดเข้าด้วยกัน
4. คำแนะนำ - ให้คำแนะนำและแนวทางที่เป็นประโยชน์

โปรดตอบเป็นภาษาไทยที่เข้าใจง่าย ใช้ภาษาที่สุภาพและเป็นกันเอง และให้คำทำนายที่มีความหมายลึกซึ้งและเป็นประโยชน์จริง"#,
            question,
            category,
            intent,
            emotion,
            cards.join(", ")
        );

        self.client
            .generate_text(&prompt)
            .await
            .map_err(ReadingAgentError::ApiError)
    }

    /// Generate individual card interpretations
    async fn generate_card_interpretations(
        &self,
        cards: &[String],
        question_analysis: &Value,
    ) -> Result<Vec<CardInterpretation>, ReadingAgentError> {
        let mut interpretations = Vec::new();

        for (index, card) in cards.iter().enumerate() {
            let position = self.get_card_position(index, cards.len());
            let interpretation = self
                .interpret_single_card(card, &position, question_analysis)
                .await?;

            interpretations.push(CardInterpretation {
                card: card.clone(),
                position,
                interpretation,
                keywords: self.extract_card_keywords(card).await?,
            });
        }

        Ok(interpretations)
    }

    /// Get position description for card in spread
    fn get_card_position(&self, index: usize, total_cards: usize) -> String {
        match (total_cards, index) {
            (3, 0) => "อดีต (Past)".to_string(),
            (3, 1) => "ปัจจุบัน (Present)".to_string(),
            (3, 2) => "อนาคต (Future)".to_string(),
            (5, 0) => "สถานการณ์ปัจจุบัน (Current Situation)".to_string(),
            (5, 1) => "ความท้าทาย (Challenge)".to_string(),
            (5, 2) => "ปัจจัยภายนอก (External Factors)".to_string(),
            (5, 3) => "คำแนะนำ (Advice)".to_string(),
            (5, 4) => "ผลลัพธ์ที่คาดหวัง (Potential Outcome)".to_string(),
            _ => format!("ไพ่ที่ {} (Card {})", index + 1, index + 1),
        }
    }

    /// Interpret a single card
    async fn interpret_single_card(
        &self,
        card: &str,
        position: &str,
        question_analysis: &Value,
    ) -> Result<String, ReadingAgentError> {
        let category = question_analysis
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let prompt = format!(
            r#"โปรดตีความไพ่ทาโรต์ "{}" ในตำแหน่ง "{}"
สำหรับคำถามในหมวดหมู่ "{}"

กรุณาให้ความหมายโดยเฉพาะเจาะจงสำหรับตำแหน่งนี้ และเชื่อมโยงกับประเด็นของคำถาม

ตอบเป็นภาษาไทยที่กระชับและเข้าใจง่าย"#,
            card, position, category
        );

        self.client
            .generate_text(&prompt)
            .await
            .map_err(ReadingAgentError::ApiError)
    }

    /// Extract keywords for a card
    async fn extract_card_keywords(&self, card: &str) -> Result<Vec<String>, ReadingAgentError> {
        let prompt = format!(
            r#"กรุณาให้คำสำคัญ (keywords) 5-10 คำสำหรับไพ่ทาโรต์ "{}"

ตอบเป็นคำๆ คั่นด้วยเครื่องหมายจุลภาค เช่น: คำสำคัญ1,คำสำคัญ2,คำสำคัญ3

ตอบเป็นภาษาไทยเท่านั้น"#,
            card
        );

        let response = self
            .client
            .generate_text(&prompt)
            .await
            .map_err(ReadingAgentError::ApiError)?;

        Ok(response
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Generate overall guidance
    async fn generate_guidance(
        &self,
        question: &str,
        card_interpretations: &[CardInterpretation],
        _question_analysis: &Value,
    ) -> Result<String, ReadingAgentError> {
        let card_summaries: Vec<String> = card_interpretations
            .iter()
            .map(|ci| format!("{}: {}", ci.card, ci.interpretation))
            .collect();

        let prompt = format!(
            r#"จากการทำนายไพ่ทาโรต์สำหรับคำถาม "{}"

สรุปการตีความไพ่:
{}

โปรดให้คำแนะนำโดยรวมและแนวทางที่เป็นประโยชน์สำหรับผู้ถาม
โดยคำนึงถึง:
- สิ่งที่ควรทำในปัจจุบัน
- สิ่งที่ควรหลีกเลี่ยง
- โอกาสและความเป็นไปได้
- คำแนะนำเชิงปฏิบัติ

ตอบเป็นภาษาไทยที่ให้กำลังใจและสร้างสรรค์"#,
            question,
            card_summaries.join("\n")
        );

        self.client
            .generate_text(&prompt)
            .await
            .map_err(ReadingAgentError::ApiError)
    }

    /// Generate quick reading (simplified version)
    pub async fn generate_quick_reading(
        &self,
        question: &str,
        cards: &[String],
    ) -> Result<String, ReadingAgentError> {
        let prompt = format!(
            r#"ทำนายไพ่ทาโรต์เร็วๆ สำหรับคำถาม: "{}"

ไพ่ที่ได้: {}

กรุณาให้คำทำนายโดยย่อๆ ประมาณ 2-3 ย่อหน้า โดยมีทั้งความหมายของไพ่และคำแนะนำ

ตอบเป็นภาษาไทยที่เข้าใจง่าย"#,
            question,
            cards.join(", ")
        );

        self.client
            .generate_text(&prompt)
            .await
            .map_err(ReadingAgentError::ApiError)
    }
}

impl Default for ReadingAgent {
    fn default() -> Self {
        Self::new().expect("GEMINI_API_KEY must be set")
    }
}

/// Legacy function for backward compatibility
pub async fn generate_reading(question: &str, cards: Vec<String>) -> Result<String, String> {
    let agent = ReadingAgent::new().map_err(|e| e.to_string())?;

    // Create a basic analysis for backward compatibility
    let basic_analysis = serde_json::json!({
        "category": "general",
        "intent": "guidance",
        "emotion": "neutral"
    });

    let reading = agent
        .generate_reading(question, &cards, &basic_analysis)
        .await
        .map_err(|e| e.to_string())?;

    Ok(reading.interpretation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_position() {
        let agent = ReadingAgent::new().unwrap();

        // Test 3-card spread positions
        assert_eq!(agent.get_card_position(0, 3), "อดีต (Past)");
        assert_eq!(agent.get_card_position(1, 3), "ปัจจุบัน (Present)");
        assert_eq!(agent.get_card_position(2, 3), "อนาคต (Future)");

        // Test 5-card spread positions
        assert_eq!(
            agent.get_card_position(0, 5),
            "สถานการณ์ปัจจุบัน (Current Situation)"
        );
        assert_eq!(
            agent.get_card_position(4, 5),
            "ผลลัพธ์ที่คาดหวัง (Potential Outcome)"
        );
    }

    #[test]
    fn test_tarot_reading_creation() {
        let question = "ควรจะลงทุนอะไรดีครับ";
        let cards = vec!["The Fool".to_string(), "The Magician".to_string()];
        let analysis = serde_json::json!({
            "category": "finance",
            "intent": "advice",
            "emotion": "hopeful"
        });

        // Test that we can create the structure (without API calls)
        let reading = TarotReading {
            question: question.to_string(),
            cards: cards.clone(),
            question_analysis: analysis.clone(),
            interpretation: "Test interpretation".to_string(),
            card_interpretations: vec![],
            guidance: "Test guidance".to_string(),
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(reading.question, question);
        assert_eq!(reading.cards, cards);
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_generate_reading() {
        let agent = ReadingAgent::new().unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let cards = vec![
            "The Fool".to_string(),
            "The Magician".to_string(),
            "The Star".to_string(),
        ];
        let analysis = serde_json::json!({
            "category": "finance",
            "intent": "advice",
            "emotion": "hopeful"
        });

        let result = agent.generate_reading(question, &cards, &analysis).await;

        assert!(result.is_ok());
        let reading = result.unwrap();

        assert!(!reading.interpretation.is_empty());
        assert_eq!(reading.cards.len(), cards.len());
        assert_eq!(reading.card_interpretations.len(), cards.len());
        assert!(!reading.guidance.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_invalid_inputs() {
        let agent = ReadingAgent::new().unwrap();
        let analysis = serde_json::json!({"category": "test"});

        // Test empty question
        let result = agent
            .generate_reading("", &["The Fool".to_string()], &analysis)
            .await;
        assert!(result.is_err());

        // Test empty cards
        let result = agent.generate_reading("test", &[], &analysis).await;
        assert!(result.is_err());

        // Test duplicate cards
        let duplicate_cards = vec!["The Fool".to_string(), "The Fool".to_string()];
        let result = agent
            .generate_reading("test", &duplicate_cards, &analysis)
            .await;
        assert!(result.is_err());
    }
}
