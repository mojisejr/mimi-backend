//! Tarot Reading Agent with "แม่หมอมีมี่" Persona
//!
//! Generates authentic Thai tarot readings using Google Gemini API and encoded prompts.
//! Features structured JSON responses with cultural context and card information.

use crate::config::env::EnvironmentConfig;
use crate::models::reading_agent::*;
use crate::utils::gemini::GeminiClient;
use crate::utils::prompt_manager::PromptManager;
use std::sync::Arc;

/// Reading Agent with "แม่หมอมีมี่" persona
#[derive(Debug, Clone)]
pub struct ReadingAgent {
    client: GeminiClient,
    prompt_manager: Arc<PromptManager>,
}

impl ReadingAgent {
    /// Create a new ReadingAgent instance with PromptManager integration
    pub async fn new() -> Result<Self, ReadingAgentError> {
        let config = EnvironmentConfig::from_env()
            .map_err(|e| ReadingAgentError::ConfigError(e.to_string()))?;

        let prompt_manager = PromptManager::new(config);

        Ok(Self {
            client: GeminiClient::new()?,
            prompt_manager: Arc::new(prompt_manager),
        })
    }

    /// Generate complete tarot reading with new structured response
    pub async fn generate_reading(
        &self,
        question: &str,
        mood: &str,
        topic: &str,
        period: &str,
        cards: &[CardInfo],
    ) -> Result<ReadingAgentResponse, ReadingAgentError> {
        // Validate inputs
        if question.trim().is_empty() {
            return Err(ReadingAgentError::EmptyQuestion);
        }

        if cards.is_empty() {
            return Err(ReadingAgentError::NoCardsProvided);
        }

        if cards.len() > 10 {
            return Err(ReadingAgentError::TooManyCards);
        }

        // Check for duplicate cards
        let unique_ids: std::collections::HashSet<_> = cards.iter().map(|c| c.id).collect();
        if unique_ids.len() != cards.len() {
            return Err(ReadingAgentError::DuplicateCards);
        }

        self.generate_tarot_reading(question, mood, topic, period, cards)
            .await
    }

    /// Build reading context from parameters
    fn build_reading_context(
        &self,
        question: &str,
        mood: &str,
        topic: &str,
        period: &str,
        cards: &[CardInfo],
    ) -> TarotReadingContext {
        let reading_type = if cards.len() <= 3 {
            ReadingType::ThreeCard
        } else {
            ReadingType::FiveCard
        };

        TarotReadingContext {
            question: question.to_string(),
            mood: mood.to_string(),
            topic: topic.to_string(),
            period: period.to_string(),
            cards: cards.to_vec(),
            reading_type,
        }
    }

    /// Format cards for template substitution
    fn format_cards_for_template(&self, cards: &[CardInfo]) -> String {
        cards
            .iter()
            .map(|card| {
                format!(
                    "ไพ่ที่ {}: {} ({}): {}",
                    card.position + 1,
                    card.display_name,
                    card.name,
                    card.short_meaning.as_deref().unwrap_or("ความหมายของไพ่ใบนี้")
                )
            })
            .collect::<Vec<_>>()
            .join("\\n")
    }

    /// Render reading prompt with context
    async fn render_reading_prompt(
        &self,
        context: &TarotReadingContext,
    ) -> Result<String, ReadingAgentError> {
        let mut render_context =
            crate::models::prompt::PromptRenderContext::new(context.question.clone())
                .with_mood(context.mood.clone())
                .with_topic(context.topic.clone())
                .with_period(context.period.clone());

        // Format cards for template
        let cards_formatted = self.format_cards_for_template(&context.cards);
        render_context.cards = Some(vec![cards_formatted]);

        // Load and render Thai prompt template
        let template = self.prompt_manager.load_prompt("reading_agent")?;
        let formatted_prompt = self
            .prompt_manager
            .render_template(&template, &render_context)?;

        Ok(formatted_prompt)
    }

    /// Generate tarot reading with new structured response
    async fn generate_tarot_reading(
        &self,
        question: &str,
        mood: &str,
        topic: &str,
        period: &str,
        cards: &[CardInfo],
    ) -> Result<ReadingAgentResponse, ReadingAgentError> {
        // Build reading context
        let context = self.build_reading_context(question, mood, topic, period, cards);

        // Render prompt with all context
        let formatted_prompt = self.render_reading_prompt(&context).await?;

        // Call Gemini API with new prompt
        let response = self.client.generate_text(&formatted_prompt).await?;

        // Parse complex JSON response
        let mut reading_response: ReadingAgentResponse = serde_json::from_str(&response)
            .map_err(|e| ReadingAgentError::JsonParseError(e.to_string()))?;

        // Validate response structure
        self.validate_reading_response(&reading_response)?;

        // Enhance response with card information
        self.enhance_response_with_card_info(&mut reading_response, cards)?;

        Ok(reading_response)
    }

    /// Validate reading response structure
    fn validate_reading_response(
        &self,
        response: &ReadingAgentResponse,
    ) -> Result<(), ReadingAgentError> {
        // Validate required fields
        if response.header.is_empty() {
            return Err(ReadingAgentError::InvalidResponse {
                field: "header".to_string(),
                reason: "Header cannot be empty".to_string(),
            });
        }

        if response.reading.is_empty() {
            return Err(ReadingAgentError::InvalidResponse {
                field: "reading".to_string(),
                reason: "Reading content cannot be empty".to_string(),
            });
        }

        // Validate cards_reading
        if response.cards_reading.is_empty() {
            return Err(ReadingAgentError::InvalidResponse {
                field: "cards_reading".to_string(),
                reason: "Cards reading cannot be empty".to_string(),
            });
        }

        // Validate suggestions
        if response.suggestions.len() > 5 {
            return Err(ReadingAgentError::InvalidResponse {
                field: "suggestions".to_string(),
                reason: "Too many suggestions (max 5)".to_string(),
            });
        }

        Ok(())
    }

    /// Enhance response with correct card information
    fn enhance_response_with_card_info(
        &self,
        response: &mut ReadingAgentResponse,
        cards: &[CardInfo],
    ) -> Result<(), ReadingAgentError> {
        // Ensure cards_reading matches input cards
        if response.cards_reading.len() != cards.len() {
            // Override with correct card information
            response.cards_reading = cards
                .iter()
                .enumerate()
                .map(|(index, card)| {
                    CardReading {
                        id: card.id,
                        name: card.name.clone(),
                        display_name: card.display_name.clone(),
                        position: card.position + 1, // 1-based in response
                        short_meaning: response
                            .cards_reading
                            .get(index)
                            .map(|cr| cr.short_meaning.clone())
                            .unwrap_or_else(|| "ความหมายของไพ่ใบนี้".to_string()),
                    }
                })
                .collect();
        }

        Ok(())
    }

    /// Quick reading with default context
    pub async fn quick_reading(
        &self,
        question: &str,
        cards: &[CardInfo],
    ) -> Result<ReadingAgentResponse, ReadingAgentError> {
        let mood = "อยากรู้"; // Default mood
        let topic = "การตัดสินใจ"; // Default topic
        let period = "ไม่ระบุ"; // Default period

        self.generate_reading(question, mood, topic, period, cards)
            .await
    }

    /// Generate reading summary for logging/metrics
    pub async fn get_reading_summary(&self, response: &ReadingAgentResponse) -> ReadingSummary {
        ReadingSummary {
            card_count: response.cards_reading.len(),
            reading_length: response.reading.len(),
            suggestions_count: response.suggestions.len(),
            has_thai_content: self.contains_thai_text(&response.reading),
            response_format_valid: true,
        }
    }

    /// Check if text contains Thai characters
    fn contains_thai_text(&self, text: &str) -> bool {
        text.chars().any(|c| {
            let code_point = c as u32;
            (0x0E00..=0x0E7F).contains(&code_point)
        })
    }
}

// Legacy Default implementation - note this is synchronous but new() is async
impl Default for ReadingAgent {
    fn default() -> Self {
        // For legacy compatibility, we'll create a minimal instance
        // In practice, users should call `new()` async method
        let config = EnvironmentConfig::from_env().expect("Environment config must be available");
        let prompt_manager = PromptManager::new(config);

        Self {
            client: GeminiClient::new().expect("GEMINI_API_KEY must be set"),
            prompt_manager: Arc::new(prompt_manager),
        }
    }
}

/// Legacy function for backward compatibility (simplified version)
pub async fn generate_reading(question: &str, cards: Vec<String>) -> Result<String, String> {
    // Convert legacy card format to new CardInfo format
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

    let agent = ReadingAgent::new().await.map_err(|e| e.to_string())?;

    let response = agent
        .quick_reading(question, &card_infos)
        .await
        .map_err(|e| e.to_string())?;

    // Return just the reading text for backward compatibility
    Ok(response.reading)
}

// ========== RED PHASE: Comprehensive Failing Tests ==========
// These tests are written BEFORE implementation to ensure TDD workflow

#[cfg(test)]
mod red_phase_tests {
    use super::*;
    use crate::config::env::EnvironmentConfig;
    use crate::models::reading_agent::*;
    use crate::utils::prompt_manager::PromptManager;

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_reading_agent_thai_prompt_loading() {
        // Test that encoded Thai prompt loads and decodes correctly
        let config = EnvironmentConfig::from_env().expect("Environment config should load");
        let prompt_manager = PromptManager::new(config);

        let decoded_prompt = prompt_manager
            .load_prompt("reading_agent")
            .expect("Reading agent prompt should load and decode");

        // Should contain Thai text from "แม่หมอมีมี่" persona
        assert!(
            decoded_prompt.contains("มีมี่")
                || decoded_prompt.contains("แม่หมอ")
                || decoded_prompt.contains("ทาโรต์")
        );
        assert!(!decoded_prompt.is_empty());
        assert!(decoded_prompt.len() > 100); // Should be substantial prompt
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_json_response_parsing_success() {
        // Test that valid JSON response parses to ReadingAgentResponse
        let valid_json = r#"
        {
            "header": "🔮 ทาโรต์ 3 ใบ: ดูดวงชะตาประจำวันนี้",
            "cards_reading": [
                {
                    "id": 0,
                    "name": "The Fool",
                    "display_name": "ไพ่ผู้โง่เง่า",
                    "position": 1,
                    "short_meaning": "จุดเริ่มต้นใหม่ การผจญภัย"
                }
            ],
            "reading": "มีมี่เห็นว่าช่วงนี้เป็นช่วงเวลาแห่งการเริ่มต้นใหม่...",
            "suggestions": ["ลองทำสิ่งใหม่ๆ", "เปิดใจรับโอกาส"],
            "final": ["โชคดีกับการเดินทางใหม่ของคุณ"],
            "end": "ขอบคุณที่ไว้ใจให้มีมี่ช่วยดูดวงนะคะ ❤️"
        }
        "#;

        let response: ReadingAgentResponse =
            serde_json::from_str(valid_json).expect("Valid JSON should parse successfully");

        assert_eq!(response.header, "🔮 ทาโรต์ 3 ใบ: ดูดวงชะตาประจำวันนี้");
        assert_eq!(response.cards_reading.len(), 1);
        assert_eq!(response.cards_reading[0].name, "The Fool");
        assert_eq!(response.cards_reading[0].display_name, "ไพ่ผู้โง่เง่า");
        assert!(response.reading.contains("มีมี่"));
        assert_eq!(response.suggestions.len(), 2);
        assert_eq!(response.r#final.len(), 1);
        assert!(response.end.contains("มีมี่"));
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_cards_reading_structure_validation() {
        // Test that cards_reading array has required fields and valid structure
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![
            CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0),
            CardInfo::new(7, "The Chariot", "ไพ่รถม้าศึก", 1),
            CardInfo::new(21, "The World", "ไพ่โลก", 2),
        ];

        let response = agent
            .generate_reading(
                "ควรจะเริ่มต้นธุรกิจใหม่หรือไม่",
                "ตื่นเต้น",
                "ธุรกิจ",
                "ไตรมาสหน้า",
                &cards,
            )
            .await
            .expect("Reading should generate successfully");

        // Validate cards_reading structure
        assert!(
            !response.cards_reading.is_empty(),
            "Cards reading should not be empty"
        );
        assert_eq!(
            response.cards_reading.len(),
            cards.len(),
            "Should match input cards count"
        );

        for (index, card_reading) in response.cards_reading.iter().enumerate() {
            let original_card = &cards[index];

            assert_eq!(card_reading.id, original_card.id);
            assert_eq!(card_reading.name, original_card.name);
            assert_eq!(card_reading.display_name, original_card.display_name);
            assert_eq!(card_reading.position, original_card.position + 1); // 1-based in response
            assert!(
                !card_reading.short_meaning.is_empty(),
                "Short meaning should not be empty"
            );
        }
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_placeholder_substitution_all_variables() {
        // Test that {question}, {mood}, {topic}, {period}, {cards} placeholders work
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![
            CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0),
            CardInfo::new(1, "The Magician", "ไพ่นากมนตร์", 1),
            CardInfo::new(7, "The Chariot", "ไพ่รถม้าศึก", 2),
        ];

        let response = agent
            .generate_reading("ควรจะเปลี่ยนงานหรือไม่", "กังวล", "การงาน", "เดือนหน้า", &cards)
            .await
            .expect("Reading should generate successfully");

        // The response should contain context from all placeholders
        let response_text = format!("{} {}", response.reading, response.header);

        // Should contain references to the question context
        assert!(
            response_text.contains("งาน")
                || response_text.contains("เปลี่ยน")
                || response_text.contains("กังวล")
        );

        // Should contain card information
        assert!(response
            .cards_reading
            .iter()
            .any(|cr| cr.display_name.contains("ไพ่")));

        // All cards should be present in the response
        assert_eq!(response.cards_reading.len(), 3);
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_thai_persona_response_validation() {
        // Test that response is in Thai with "แม่หมอมีมี่" tone
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![CardInfo::new(0, "The Sun", "ไพ่ดวงอาทิตย์", 0)];

        let response = agent
            .generate_reading("ชีวิตจะดีขึ้นหรือไม่", "หวังดี", "ชีวิต", "ปีนี้", &cards)
            .await
            .expect("Reading should generate successfully");

        // Validate Thai persona characteristics
        let full_text = format!(
            "{} {} {} {}",
            response.header,
            response.reading,
            response.suggestions.join(" "),
            response.end
        );

        // Should contain Thai characters
        assert!(
            full_text.chars().any(|c| {
                let code_point = c as u32;
                (0x0E00..=0x0E7F).contains(&code_point)
            }),
            "Response should contain Thai characters"
        );

        // Should have "แม่หมอมีมี่" persona elements
        assert!(
            full_text.contains("มีมี่")
                || full_text.contains("แม่หมอ")
                || full_text.contains("คะ")
                || full_text.contains("ครับ")
                || full_text.contains("❤️")
                || full_text.contains("🔮")
        );

        // Should have appropriate Thai tone and cultural context
        assert!(!response.reading.is_empty());
        assert!(!response.end.is_empty());
        assert!(response.suggestions.len() > 0);
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_response_field_validation() {
        // Test that all required fields are present and valid
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![CardInfo::new(10, "Justice", "ไพ่ยุติธรรม", 0)];

        let response = agent
            .generate_reading(
                "จะได้รับความยุติธรรมหรือไม่",
                "วิตกกังวล",
                "ความยุติธรรม",
                "เดือนนี้",
                &cards,
            )
            .await
            .expect("Reading should generate successfully");

        // Validate all required fields
        assert!(!response.header.is_empty(), "Header should not be empty");
        assert!(!response.reading.is_empty(), "Reading should not be empty");
        assert!(!response.end.is_empty(), "End message should not be empty");

        // Validate array fields
        assert!(
            !response.cards_reading.is_empty(),
            "Cards reading should not be empty"
        );
        assert!(
            !response.suggestions.is_empty(),
            "Suggestions should not be empty"
        );
        assert!(
            !response.r#final.is_empty(),
            "Final thoughts should not be empty"
        );

        // Validate constraints
        assert!(
            response.suggestions.len() <= 5,
            "Should have max 5 suggestions"
        );
        assert!(
            response.r#final.len() <= 3,
            "Should have max 3 final thoughts"
        );

        // Validate card reading structure
        for card_reading in &response.cards_reading {
            assert!(
                !card_reading.name.is_empty(),
                "Card name should not be empty"
            );
            assert!(
                !card_reading.display_name.is_empty(),
                "Card display name should not be empty"
            );
            assert!(card_reading.position > 0, "Card position should be 1-based");
            assert!(
                !card_reading.short_meaning.is_empty(),
                "Short meaning should not be empty"
            );
        }
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_card_information_parsing() {
        // Test that card data is correctly extracted and formatted
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![
            CardInfo::new(5, "The Hierophant", "ไพ่นักบวช", 0)
                .with_short_meaning("ความเชื่อ ศาสนา ขนบธรรมเนียม".to_string()),
            CardInfo::new(14, "Temperance", "ไพ่ความอดทน", 1)
                .with_short_meaning("การประนีประนอม ความสมดุล".to_string()),
        ];

        let response = agent
            .generate_reading("ควรเชื่อฟังใครดี", "สับสน", "การตัดสินใจ", "ตอนนี้", &cards)
            .await
            .expect("Reading should generate successfully");

        // Validate card information preservation
        assert_eq!(response.cards_reading.len(), cards.len());

        for (index, card_reading) in response.cards_reading.iter().enumerate() {
            let original_card = &cards[index];

            assert_eq!(card_reading.id, original_card.id);
            assert_eq!(card_reading.name, original_card.name);
            assert_eq!(card_reading.display_name, original_card.display_name);
            assert_eq!(card_reading.position, original_card.position + 1);

            // Short meaning should be either from original or generated
            assert!(!card_reading.short_meaning.is_empty());
            if let Some(ref original_meaning) = original_card.short_meaning {
                // If original had meaning, it should be preserved or enhanced
                assert!(card_reading.short_meaning.len() >= original_meaning.len());
            }
        }
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_suggestions_array_validation() {
        // Test that suggestions are practical and Thai
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![CardInfo::new(11, "Strength", "ไพ่ความแข็งแกร่ง", 0)];

        let response = agent
            .generate_reading("มีกำลังพอที่จะผ่านช่วงเวลานี้ไหม", "อ่อนแอ", "พลังใจ", "ช่วงนี้", &cards)
            .await
            .expect("Reading should generate successfully");

        // Validate suggestions
        assert!(!response.suggestions.is_empty(), "Should have suggestions");
        assert!(
            response.suggestions.len() <= 5,
            "Should have max 5 suggestions"
        );

        for suggestion in &response.suggestions {
            // Should be practical advice
            assert!(!suggestion.is_empty(), "Suggestion should not be empty");
            assert!(suggestion.len() > 10, "Suggestion should be substantial");

            // Should be in Thai
            assert!(
                suggestion.chars().any(|c| {
                    let code_point = c as u32;
                    (0x0E00..=0x0E7F).contains(&code_point)
                }),
                "Suggestion should contain Thai characters"
            );

            // Should be actionable advice
            assert!(
                suggestion.contains("การ")
                    || suggestion.contains("ควร")
                    || suggestion.contains("อย่า")
                    || suggestion.contains("ควรจะ")
                    || suggestion.contains("ลอง")
                    || suggestion.contains("พยายาม"),
                "Suggestion should be actionable advice"
            );
        }
    }
}

// Integration Tests (will fail until implementation is complete)

#[cfg(test)]
mod integration_red_phase_tests {
    use super::*;
    use crate::models::reading_agent::*;

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_end_to_end_tarot_reading() {
        // Complete flow from question+cards to structured reading
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![
            CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0),
            CardInfo::new(1, "The Magician", "ไพ่นากมนตร์", 1),
            CardInfo::new(21, "The World", "ไพ่โลก", 2),
        ];

        let response = agent
            .generate_reading(
                "จะประสบความสำเร็จในอาชีพใหม่หรือไม่",
                "ตื่นเต้นและกังวล",
                "อาชีพ",
                "ปีนี้",
                &cards,
            )
            .await
            .expect("End-to-end reading should succeed");

        // Validate complete response structure
        assert!(!response.header.is_empty());
        assert_eq!(response.cards_reading.len(), 3);
        assert!(!response.reading.is_empty());
        assert!(!response.suggestions.is_empty());
        assert!(!response.r#final.is_empty());
        assert!(!response.end.is_empty());

        // Validate "แม่หมอมีมี่" persona consistency
        let full_response = format!(
            "{} {} {} {}",
            response.header,
            response.reading,
            response.suggestions.join(" "),
            response.end
        );

        assert!(
            full_response.contains("มีมี่")
                || full_response.contains("แม่หมอ")
                || full_response.contains("คะ")
                || full_response.contains("🔮")
                || full_response.contains("❤️")
        );
    }

    #[tokio::test]
    #[ignore] // Requires real API key - will fail until implementation is complete
    async fn test_real_gemini_api_integration_reading() {
        // Test with actual Gemini API call with new prompt format
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");
        let cards = vec![
            CardInfo::new(6, "The Lovers", "ไพ่คู่รัก", 0),
            CardInfo::new(2, "The High Priestess", "ไพ่นักบวชหญิง", 1),
        ];

        let response = agent
            .generate_reading("จะพบรักแท้ได้หรือไม่", "หวังดี", "ความรัก", "เร็วๆ นี้", &cards)
            .await
            .expect("Real API integration should work");

        // Should get realistic response from actual API
        assert!(response.reading.len() > 100); // Should be substantial reading
        assert!(response.cards_reading.len() == 2);

        // Should contain authentic Thai tarot reading content
        assert!(response.reading.chars().any(|c| {
            let code_point = c as u32;
            (0x0E00..=0x0E7F).contains(&code_point)
        }));
    }

    #[tokio::test]
    #[ignore] // Will fail until implementation is complete
    async fn test_multi_card_reading_scenario() {
        // Test both 3-card and 5-card reading scenarios
        let agent = ReadingAgent::new()
            .await
            .expect("ReadingAgent should create successfully");

        // Test 3-card reading
        let three_cards = vec![
            CardInfo::new(3, "The Empress", "ไพ่จักรพรรดินี", 0),
            CardInfo::new(4, "The Emperor", "ไพ่จักรพรรดิ", 1),
            CardInfo::new(5, "The Hierophant", "ไพ่นักบวช", 2),
        ];

        let three_card_response = agent
            .generate_reading(
                "ชีวิตครอบครัวจะเป็นอย่างไร",
                "ผูกพัน",
                "ครอบครัว",
                "ปีนี้",
                &three_cards,
            )
            .await
            .expect("3-card reading should succeed");

        assert_eq!(three_card_response.cards_reading.len(), 3);

        // Test 5-card reading
        let five_cards = vec![
            CardInfo::new(8, "Strength", "ไพ่ความแข็งแกร่ง", 0),
            CardInfo::new(9, "The Hermit", "ไพ่นักบวชสืบสวน", 1),
            CardInfo::new(10, "Wheel of Fortune", "ไพ่วงล้อแห่งโชคชะตา", 2),
            CardInfo::new(12, "The Hanged Man", "ไพ่คนแขวนคอ", 3),
            CardInfo::new(13, "Death", "ไพ่ยมนาบาต", 4),
        ];

        let five_card_response = agent
            .generate_reading(
                "เส้นทางชีวิตในอนาคต",
                "สงสัย",
                "ชะตากรรม",
                "5 ปีข้างหน้า",
                &five_cards,
            )
            .await
            .expect("5-card reading should succeed");

        assert_eq!(five_card_response.cards_reading.len(), 5);

        // Both should have valid "แม่หมอมีมี่" persona
        for response in [three_card_response, five_card_response] {
            assert!(!response.reading.is_empty());
            assert!(!response.suggestions.is_empty());
            assert!(response.cards_reading.len() > 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_agent_response_creation() {
        let _question = "ควรจะลงทุนอะไรดีครับ";

        // Test that we can create the new ReadingAgentResponse structure (without API calls)
        let response = ReadingAgentResponse {
            header: "ทาโรต์ 3 ใบ: การลงทุน".to_string(),
            cards_reading: vec![
                CardReading {
                    id: 0,
                    name: "The Fool".to_string(),
                    display_name: "ไพ่ผู้โง่เง่า".to_string(),
                    position: 1,
                    short_meaning: "จุดเริ่มต้นใหม่".to_string(),
                },
                CardReading {
                    id: 1,
                    name: "The Magician".to_string(),
                    display_name: "ไพ่นากมนตร์".to_string(),
                    position: 2,
                    short_meaning: "พลังแห่งการสร้างสรรค์".to_string(),
                },
            ],
            reading: "มีมี่เห็นว่า...".to_string(),
            suggestions: vec!["ลงทุนอย่างรอบคอบ".to_string()],
            r#final: vec!["โชคดีค่ะ".to_string()],
            end: "ขอบคุณที่ไว้ใจมีมี่นะคะ ❤️".to_string(),
        };

        assert_eq!(response.header, "ทาโรต์ 3 ใบ: การลงทุน");
        assert_eq!(response.cards_reading.len(), 2);
        assert_eq!(response.cards_reading[0].name, "The Fool");
        assert_eq!(response.cards_reading[0].display_name, "ไพ่ผู้โง่เง่า");
        assert_eq!(response.cards_reading[0].position, 1);
        assert_eq!(response.suggestions.len(), 1);
        assert_eq!(response.r#final.len(), 1);
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_generate_reading_new() {
        let agent = ReadingAgent::new().await.unwrap();

        let question = "ควรจะลงทุนอะไรดีครับ";
        let cards = vec![
            CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0),
            CardInfo::new(1, "The Magician", "ไพ่นากมนตร์", 1),
            CardInfo::new(17, "The Star", "ไพ่ดวงดาว", 2),
        ];

        let result = agent
            .generate_reading(question, "หวังดี", "การเงิน", "เดือนหน้า", &cards)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();

        assert!(!response.reading.is_empty());
        assert_eq!(response.cards_reading.len(), cards.len());
        assert!(!response.suggestions.is_empty());
        assert!(!response.end.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires API key to run
    async fn test_invalid_inputs_new() {
        let agent = ReadingAgent::new().await.unwrap();

        // Test empty question
        let result = agent
            .generate_reading(
                "",
                "หวังดี",
                "ทดสอบ",
                "เดือนหน้า",
                &[CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0)],
            )
            .await;
        assert!(result.is_err());

        // Test empty cards
        let result = agent
            .generate_reading("test", "หวังดี", "ทดสอบ", "เดือนหน้า", &[])
            .await;
        assert!(result.is_err());

        // Test duplicate cards
        let duplicate_cards = vec![
            CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 0),
            CardInfo::new(0, "The Fool", "ไพ่ผู้โง่เง่า", 1),
        ];
        let result = agent
            .generate_reading("test", "หวังดี", "ทดสอบ", "เดือนหน้า", &duplicate_cards)
            .await;
        assert!(result.is_err());
    }
}
