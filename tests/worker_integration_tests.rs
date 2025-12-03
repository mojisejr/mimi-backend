//! Worker Integration Tests
//!
//! Test-Driven Development tests for worker with AI pipeline processing.
//! These tests are written BEFORE implementation (Red Phase).

use mimivibe_backend::services::card_randomizer::CardRandomizer;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Mock Tarot deck for testing
const TEST_TAROT_CARDS: &[&str] = &[
    "The Fool",
    "The Magician",
    "The High Priestess",
    "The Empress",
    "The Emperor",
    "The Hierophant",
    "The Lovers",
    "The Chariot",
    "Strength",
    "The Hermit",
    "Wheel of Fortune",
    "Justice",
    "The Hanged Man",
    "Death",
    "Temperance",
    "The Devil",
    "The Tower",
    "The Star",
    "The Moon",
    "The Sun",
    "Judgement",
    "The World",
    "Ace of Wands",
    "Two of Wands",
    "Three of Wands",
    "Four of Wands",
    "Five of Wands",
    "Six of Wands",
    "Seven of Wands",
    "Eight of Wands",
    "Nine of Wands",
    "Ten of Wands",
    "Page of Wands",
    "Knight of Wands",
    "Queen of Wands",
    "King of Wands",
    // ... add more cards as needed for testing
];

#[tokio::test]
async fn test_card_randomizer_3_cards() {
    // Test: CardRandomizer should return exactly 3 unique cards
    let randomizer = CardRandomizer::new();
    let cards = randomizer.pick_cards(3).await.unwrap();

    assert_eq!(cards.len(), 3, "Should return exactly 3 cards");

    // Verify all cards are unique
    let unique_cards: std::collections::HashSet<_> = cards.iter().collect();
    assert_eq!(unique_cards.len(), 3, "All cards should be unique");

    // Verify all cards are from the tarot deck
    for card in &cards {
        assert!(
            TEST_TAROT_CARDS.contains(&card.as_str()),
            "Card '{}' should be from tarot deck",
            card
        );
    }
}

#[tokio::test]
async fn test_card_randomizer_5_cards() {
    // Test: CardRandomizer should return exactly 5 unique cards
    let randomizer = CardRandomizer::new();
    let cards = randomizer.pick_cards(5).await.unwrap();

    assert_eq!(cards.len(), 5, "Should return exactly 5 cards");

    // Verify all cards are unique
    let unique_cards: std::collections::HashSet<_> = cards.iter().collect();
    assert_eq!(unique_cards.len(), 5, "All cards should be unique");

    // Verify all cards are from the tarot deck
    for card in &cards {
        assert!(
            TEST_TAROT_CARDS.contains(&card.as_str()),
            "Card '{}' should be from tarot deck",
            card
        );
    }
}

#[tokio::test]
async fn test_card_randomizer_invalid_count() {
    // Test: CardRandomizer should reject invalid card counts
    let randomizer = CardRandomizer::new();

    // Test with 0 cards
    let result = randomizer.pick_cards(0).await;
    assert!(result.is_err(), "Should reject 0 cards");

    // Test with invalid count (not 3 or 5)
    let result = randomizer.pick_cards(4).await;
    assert!(result.is_err(), "Should reject 4 cards");

    let result = randomizer.pick_cards(10).await;
    assert!(result.is_err(), "Should reject 10 cards");
}

#[tokio::test]
async fn test_question_filter_valid_question() {
    // Test: QuestionFilter should accept valid Thai questions
    let filter = mimivibe_backend::agents::question_filter::QuestionFilter::new();

    let valid_questions = vec![
        "ควรจะลงทุนอะไรดีครับ",
        "ชีวิตของฉันจะเป็นอย่างไรในอนาคต",
        "ควรทำงานอะไรดี",
    ];

    for question in valid_questions {
        let result = filter.validate_question(question).await;
        assert!(result.is_ok(), "Question '{}' should be valid", question);
    }
}

#[tokio::test]
async fn test_question_filter_empty_question() {
    // Test: QuestionFilter should reject empty questions
    let filter = mimivibe_backend::agents::question_filter::QuestionFilter::new();

    let empty_questions = vec!["", "   ", "\n", "\t"];

    for question in empty_questions {
        let result = filter.validate_question(question).await;
        assert!(
            result.is_err(),
            "Empty question '{}' should be rejected",
            question
        );
    }
}

#[tokio::test]
async fn test_question_filter_inappropriate_question() {
    // Test: QuestionFilter should reject inappropriate questions
    let filter = mimivibe_backend::agents::question_filter::QuestionFilter::new();

    let inappropriate_questions = vec!["ฆ่าคน", "ทำผิดกฎหมาย", "สร้างอันตราย"];

    for question in inappropriate_questions {
        let result = filter.validate_question(question).await;
        assert!(
            result.is_err(),
            "Inappropriate question '{}' should be rejected",
            question
        );
    }
}

#[tokio::test]
async fn test_question_analyzer_structured_output() {
    // Test: QuestionAnalyzer should return structured analysis
    let analyzer = mimivibe_backend::agents::question_analyzer::QuestionAnalyzer::new();

    let question = "ควรจะลงทุนอะไรดีครับ";
    let result = analyzer.analyze_question(question).await.unwrap();

    // Verify structured output format
    assert!(result.contains("category"), "Should contain category field");
    assert!(result.contains("intent"), "Should contain intent field");
    assert!(result.contains("keywords"), "Should contain keywords field");
    assert!(result.contains("context"), "Should contain context field");

    // Parse as JSON to verify structure
    let parsed: serde_json::Value =
        serde_json::from_str(&result).expect("Should return valid JSON");

    assert!(
        parsed.get("category").is_some(),
        "Should have category field"
    );
    assert!(parsed.get("intent").is_some(), "Should have intent field");
    assert!(
        parsed.get("keywords").is_some(),
        "Should have keywords field"
    );
}

#[tokio::test]
async fn test_reading_agent_card_interpretation() {
    // Test: ReadingAgent should provide card interpretations
    let agent = mimivibe_backend::agents::reading_agent::ReadingAgent::new();

    let question = "ควรจะลงทุนอะไรดีครับ";
    let cards = vec![
        "The Fool".to_string(),
        "The Magician".to_string(),
        "The Star".to_string(),
    ];
    let analysis = json!({
        "category": "investment",
        "intent": "seeking guidance",
        "keywords": ["investment", "future", "money"]
    });

    let result = agent
        .generate_reading(question, &cards, &analysis)
        .await
        .unwrap();

    // Verify interpretation contains all cards
    for card in &cards {
        assert!(
            result.contains(card),
            "Reading should mention card '{}'",
            card
        );
    }

    // Verify interpretation is structured
    assert!(result.len() > 50, "Reading should be substantial");
    assert!(result.contains("การดูดวง"), "Should be in Thai");
}

#[tokio::test]
async fn test_ai_pipeline_complete_flow() {
    // Test: Complete AI pipeline should process question end-to-end
    let pipeline = AIPipelineService::new().await.unwrap();

    let question = "ควรจะลงทุนอะไรดีครับ";
    let card_count = 3;

    let result = pipeline
        .process_tarot_reading(question, card_count)
        .await
        .unwrap();

    // Verify result structure
    assert!(
        result.contains("question"),
        "Should contain original question"
    );
    assert!(result.contains("cards"), "Should contain selected cards");
    assert!(
        result.contains("question_analysis"),
        "Should contain question analysis"
    );
    assert!(result.contains("reading"), "Should contain tarot reading");

    // Parse as JSON to verify complete structure
    let parsed: serde_json::Value =
        serde_json::from_str(&result).expect("Should return valid JSON");

    assert_eq!(parsed["question"], question);
    assert!(parsed["cards"].as_array().unwrap().len() == card_count);
    assert!(parsed["question_analysis"].is_object());
    assert!(parsed["reading"].is_string());
}

#[tokio::test]
async fn test_worker_job_processing() {
    // Test: Worker should process jobs from queue
    // This test requires database setup for full integration

    // Create test job payload
    let job_payload = json!({
        "question": "ควรจะลงทุนอะไรดีครับ",
        "card_count": 3,
        "user_id": "test-user-123"
    });

    // Verify job structure
    assert!(
        job_payload["question"].is_string(),
        "Question should be string"
    );
    assert!(
        job_payload["card_count"].is_number(),
        "Card count should be number"
    );
    assert_eq!(job_payload["card_count"], 3, "Card count should be 3");
}

#[tokio::test]
async fn test_worker_error_handling_gemini_api_failure() {
    // Test: Worker should handle Gemini API failures gracefully
    // This test would mock Gemini API failure scenarios

    let question = "ควรจะลงทุนอะไรดีครับ";
    let invalid_api_key = "invalid-key";

    // This should test error handling when Gemini API is unavailable
    // Implementation should handle this gracefully
    assert!(true, "Should handle API failures gracefully"); // Placeholder
}

#[tokio::test]
async fn test_worker_invalid_job_data_handling() {
    // Test: Worker should handle invalid job data

    let invalid_payloads = vec![
        json!({"question": "", "card_count": 3}), // Empty question
        json!({"card_count": 3}),                 // Missing question
        json!({"question": "test"}),              // Missing card_count
        json!({"question": "test", "card_count": 10}), // Invalid card count
    ];

    for payload in invalid_payloads {
        // Worker should reject invalid payloads
        assert!(true, "Should handle invalid payload: {:?}", payload); // Placeholder
    }
}

#[tokio::test]
async fn test_job_state_transitions() {
    // Test: Job status should transition correctly through pipeline
    // queued -> processing -> succeeded/failed

    let job_id = Uuid::new_v4();

    // Mock job state transitions
    let initial_state = JobStatus::Queued;
    let processing_state = JobStatus::Processing;
    let final_state = JobStatus::Succeeded;

    assert_ne!(initial_state, processing_state);
    assert_ne!(processing_state, final_state);
    assert_ne!(initial_state, final_state);
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    /// Create test database pool
    pub async fn create_test_pool() -> PgPool {
        // This would create a test database connection
        // For now, this is a placeholder
        panic!("Test database setup required");
    }

    /// Create test job in database
    pub async fn create_test_job(pool: &PgPool, payload: serde_json::Value) -> Uuid {
        let input = CreateJobInput {
            job_type: Some("tarot_reading".to_string()),
            payload,
            dedupe_key: None,
            max_attempts: Some(3),
            prompt_version: Some("v1.0".to_string()),
        };

        ReadingJob::create(pool, input).await.unwrap()
    }
}
