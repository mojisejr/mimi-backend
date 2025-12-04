//! Reading Agent Models
//!
//! Comprehensive data structures for the ReadingAgent with "แม่หมอมีมี่" persona.
//! Supports complex JSON responses with Thai cultural context and structured tarot readings.

use serde::{Deserialize, Serialize};

/// Complete ReadingAgent response with Thai cultural context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingAgentResponse {
    /// Header for the reading (e.g., "🔮 ทาโรต์ 3 ใบ: ดูดวงชะตาประจำวันนี้")
    pub header: String,
    /// Detailed information about each card in the reading
    pub cards_reading: Vec<CardReading>,
    /// Full tarot reading interpretation in "แม่หมอมีมี่" style
    pub reading: String,
    /// Practical suggestions and advice (1-5 items)
    pub suggestions: Vec<String>,
    /// Final thoughts and conclusions
    pub r#final: Vec<String>,
    /// Closing message from "แม่หมอมีมี่"
    pub end: String,
}

/// Individual card reading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardReading {
    /// Unique card identifier
    pub id: u32,
    /// Card name in English
    pub name: String,
    /// Card name in Thai
    pub display_name: String,
    /// Position in the reading spread (1-based)
    pub position: u32,
    /// Short meaning in the context of this reading
    pub short_meaning: String,
}

/// Context for tarot reading generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarotReadingContext {
    /// User's question
    pub question: String,
    /// Emotional tone/mood of the question
    pub mood: String,
    /// Main topic or category
    pub topic: String,
    /// Time period relevant to the question
    pub period: String,
    /// Cards drawn for the reading
    pub cards: Vec<CardInfo>,
    /// Type of reading (3-card or 5-card)
    pub reading_type: ReadingType,
}

/// Reading type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadingType {
    /// 3-card reading (Past, Present, Future)
    ThreeCard,
    /// 5-card reading (Situation, Challenge, External Factors, Advice, Outcome)
    FiveCard,
}

/// Card information for input to reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInfo {
    /// Unique card identifier
    pub id: u32,
    /// Card name in English
    pub name: String,
    /// Card name in Thai
    pub display_name: String,
    /// Position in the spread (0-based)
    pub position: u32,
    /// Optional image URL
    pub image_url: Option<String>,
    /// Optional short meaning
    pub short_meaning: Option<String>,
}

impl CardInfo {
    /// Create a new CardInfo instance
    pub fn new(id: u32, name: &str, display_name: &str, position: u32) -> Self {
        Self {
            id,
            name: name.to_string(),
            display_name: display_name.to_string(),
            position,
            image_url: None,
            short_meaning: None,
        }
    }

    /// Add image URL to the card
    pub fn with_image_url(mut self, image_url: String) -> Self {
        self.image_url = Some(image_url);
        self
    }

    /// Add short meaning to the card
    pub fn with_short_meaning(mut self, short_meaning: String) -> Self {
        self.short_meaning = Some(short_meaning);
        self
    }
}

/// Reading summary for metrics and logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingSummary {
    /// Number of cards in the reading
    pub card_count: usize,
    /// Length of the reading text
    pub reading_length: usize,
    /// Number of suggestions provided
    pub suggestions_count: usize,
    /// Whether the reading contains Thai text
    pub has_thai_content: bool,
    /// Whether the response format is valid
    pub response_format_valid: bool,
}

/// ReadingAgent error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum ReadingAgentError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Gemini API error: {0}")]
    ApiError(String),

    #[error("JSON parsing error: {0}")]
    JsonParseError(String),

    #[error("Invalid response in field '{field}': {reason}")]
    InvalidResponse { field: String, reason: String },

    #[error("Empty question provided")]
    EmptyQuestion,

    #[error("No cards provided for reading")]
    NoCardsProvided,

    #[error("Too many cards provided (max 10)")]
    TooManyCards,

    #[error("Duplicate cards in reading")]
    DuplicateCards,

    #[error("Invalid card data: {0}")]
    InvalidCardData(String),
}

impl From<crate::utils::gemini::GeminiError> for ReadingAgentError {
    fn from(error: crate::utils::gemini::GeminiError) -> Self {
        ReadingAgentError::ApiError(error.to_string())
    }
}

impl From<crate::utils::prompt_manager::PromptError> for ReadingAgentError {
    fn from(error: crate::utils::prompt_manager::PromptError) -> Self {
        ReadingAgentError::TemplateError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_info_creation() {
        let card = CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0)
            .with_image_url("https://example.com/fool.jpg".to_string())
            .with_short_meaning("จุดเริ่มต้นใหม่".to_string());

        assert_eq!(card.id, 0);
        assert_eq!(card.name, "The Fool");
        assert_eq!(card.display_name, "ไพ่ผู้โง่เง่า");
        assert_eq!(card.position, 0);
        assert_eq!(
            card.image_url,
            Some("https://example.com/fool.jpg".to_string())
        );
        assert_eq!(card.short_meaning, Some("จุดเริ่มต้นใหม่".to_string()));
    }

    #[test]
    fn test_tarot_reading_context_creation() {
        let cards = vec![
            CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0),
            CardInfo::new(1, "The Magician", "ไพ่นากมนตร์", 1),
            CardInfo::new(7, "The Chariot", "ไพ่รถม้าศึก", 2),
        ];

        let context = TarotReadingContext {
            question: "ควรจะลงทุนอะไรดีครับ".to_string(),
            mood: "หวังดี".to_string(),
            topic: "การเงิน".to_string(),
            period: "เดือนหน้า".to_string(),
            cards: cards.clone(),
            reading_type: ReadingType::ThreeCard,
        };

        assert_eq!(context.question, "ควรจะลงทุนอะไรดีครับ");
        assert_eq!(context.mood, "หวังดี");
        assert_eq!(context.topic, "การเงิน");
        assert_eq!(context.period, "เดือนหน้า");
        assert_eq!(context.cards.len(), 3);
        assert!(matches!(context.reading_type, ReadingType::ThreeCard));
    }

    #[test]
    fn test_reading_agent_response_structure() {
        let response = ReadingAgentResponse {
            header: "🔮 ทาโรต์ 3 ใบ: ดูดวงเรื่องการเงิน".to_string(),
            cards_reading: vec![CardReading {
                id: 0,
                name: "The Fool".to_string(),
                display_name: "ไพ่ผู้โง่เง่า".to_string(),
                position: 1,
                short_meaning: "จุดเริ่มต้นใหม่".to_string(),
            }],
            reading: "มีมี่เห็นว่า...".to_string(),
            suggestions: vec!["ลงทุนอย่างรอบคอบ".to_string()],
            r#final: vec!["โชคดีค่ะ".to_string()],
            end: "ขอบคุณที่ไว้ใจมีมี่นะคะ ❤️".to_string(),
        };

        assert!(!response.header.is_empty());
        assert_eq!(response.cards_reading.len(), 1);
        assert!(!response.reading.is_empty());
        assert_eq!(response.suggestions.len(), 1);
        assert_eq!(response.r#final.len(), 1);
        assert!(!response.end.is_empty());
    }

    #[test]
    fn test_reading_summary_creation() {
        let summary = ReadingSummary {
            card_count: 3,
            reading_length: 500,
            suggestions_count: 4,
            has_thai_content: true,
            response_format_valid: true,
        };

        assert_eq!(summary.card_count, 3);
        assert_eq!(summary.reading_length, 500);
        assert_eq!(summary.suggestions_count, 4);
        assert!(summary.has_thai_content);
        assert!(summary.response_format_valid);
    }

    #[test]
    fn test_reading_type_enum() {
        let three_card = ReadingType::ThreeCard;
        let five_card = ReadingType::FiveCard;

        assert!(matches!(three_card, ReadingType::ThreeCard));
        assert!(matches!(five_card, ReadingType::FiveCard));
    }

    #[test]
    fn test_reading_agent_error_creation() {
        let config_error = ReadingAgentError::ConfigError("Test error".to_string());
        assert!(matches!(config_error, ReadingAgentError::ConfigError(_)));

        let invalid_response = ReadingAgentError::InvalidResponse {
            field: "header".to_string(),
            reason: "Cannot be empty".to_string(),
        };
        assert!(matches!(
            invalid_response,
            ReadingAgentError::InvalidResponse { .. }
        ));
    }
}
