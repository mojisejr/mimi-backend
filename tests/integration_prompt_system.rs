//! End-to-End Integration Tests for Encoded Prompt System
//!
//! This test suite validates the complete functionality of the encrypted prompt system,
//! ensuring all three agents work seamlessly together with real Gemini API calls,
//! proper JSON responses, and authentic "แม่หมอมี่" persona throughout the pipeline.

use std::sync::Arc;
use tokio::time::Duration;
use mimivibe_backend::{
    agents::{
        question_filter::QuestionFilter,
        question_analyzer::QuestionAnalyzer,
        reading_agent::ReadingAgent,
    },
    models::reading_agent::CardInfo,
};

/// Test suite for prompt system integration testing
pub struct PromptSystemTestSuite {
    pub question_filter: Arc<QuestionFilter>,
    pub question_analyzer: Arc<QuestionAnalyzer>,
    pub reading_agent: Arc<ReadingAgent>,
}

impl PromptSystemTestSuite {
    /// Create a new test suite instance with all agents initialized
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize all agents
        let question_filter = Arc::new(QuestionFilter::new().await?);
        let question_analyzer = Arc::new(QuestionAnalyzer::new().await?);
        let reading_agent = Arc::new(ReadingAgent::new().await?);

        Ok(Self {
            question_filter,
            question_analyzer,
            reading_agent,
        })
    }
}

/// Test scenario data structure for comprehensive testing
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: String,
    pub question: String,
    pub expected_mood: Option<String>,
    pub expected_topic: Option<String>,
    pub card_count: usize,
    pub thai_question: bool,
    pub should_pass_filter: bool,
}

/// Get comprehensive test scenarios covering various use cases
pub fn get_test_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            name: "Thai Love Question".to_string(),
            question: "จะมีคนมาสาธิตฉันใช่ไหมคะ".to_string(),
            expected_mood: Some("กังวล".to_string()),
            expected_topic: Some("ความรักและความสัมพันธ์".to_string()),
            card_count: 3,
            thai_question: true,
            should_pass_filter: true,
        },
        TestScenario {
            name: "English Career Question".to_string(),
            question: "Should I change my job?".to_string(),
            expected_mood: Some("สับสน".to_string()),
            expected_topic: Some("การงานและอาชีพ".to_string()),
            card_count: 5,
            thai_question: false,
            should_pass_filter: true,
        },
        TestScenario {
            name: "Thai Finance Question".to_string(),
            question: "ควรจะลงทุนอะไรดีครับ".to_string(),
            expected_mood: Some("สงสัย".to_string()),
            expected_topic: Some("การเงินและการลงทุน".to_string()),
            card_count: 3,
            thai_question: true,
            should_pass_filter: true,
        },
        TestScenario {
            name: "Thai Health Question".to_string(),
            question: "สุขภาพของฉันจะดีขึ้นไหม".to_string(),
            expected_mood: Some("กังวล".to_string()),
            expected_topic: Some("สุขภาพและความเป็นอยู่".to_string()),
            card_count: 5,
            thai_question: true,
            should_pass_filter: true,
        },
        TestScenario {
            name: "English General Question".to_string(),
            question: "What does my future look like?".to_string(),
            expected_mood: Some("อยากรู้".to_string()),
            expected_topic: Some("ชีวิตและอนาคต".to_string()),
            card_count: 3,
            thai_question: false,
            should_pass_filter: true,
        },
    ]
}

/// Helper function to create test cards for reading agent
pub fn create_test_cards(count: usize) -> Vec<CardInfo> {
    let mut cards = Vec::new();

    // Use predefined cards for consistent testing
    let test_cards = vec![
        CardInfo::new(0, "The Fool", "ไพ่คนโง่เง่า", 0)
            .with_short_meaning("จุดเริ่มต้นใหม่".to_string()),
        CardInfo::new(1, "The Lovers", "ไพ่คู่รัก", 1)
            .with_short_meaning("ความรักและการตัดสินใจ".to_string()),
        CardInfo::new(2, "The Star", "ไพ่ดาว", 2)
            .with_short_meaning("ความหวังและแรงบันดาลใจ".to_string()),
        CardInfo::new(3, "The Sun", "ไพ่ดวงอาทิตย์", 3)
            .with_short_meaning("ความสำเร็จและความสุข".to_string()),
        CardInfo::new(4, "The Moon", "ไพ่ดวงจันทร์", 4)
            .with_short_meaning("ความสับสนและความไม่แน่นอน".to_string()),
    ];

    for i in 0..count.min(test_cards.len()) {
        cards.push(test_cards[i].clone());
    }

    cards
}

/// Helper function to check if text contains Thai characters
pub fn contains_thai_text(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        (0x0E00..=0x0E7F).contains(&code) // Thai Unicode range
    })
}

/// Helper function to validate Thai text quality
pub fn validate_thai_text_quality(text: &str) -> bool {
    // Check if text contains Thai characters
    if !contains_thai_text(text) {
        return false;
    }

    // Additional quality checks can be added here
    // For now, just ensure it has Thai characters and reasonable length
    text.trim().len() >= 3
}

// ============================================================================
// COMPREHENSIVE INTEGRATION TESTS (RED PHASE - These should FAIL initially)
// ============================================================================

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_complete_tarot_pipeline() {
    let suite = PromptSystemTestSuite::new().await.unwrap();
    let scenario = &get_test_scenarios()[0];

    // Step 1: Question Filter
    let filter_result = suite.question_filter.validate_question(&scenario.question).await;
    assert!(filter_result.is_ok(), "Question should pass filtering: {:?}", filter_result.err());

    let filter_response = filter_result.unwrap();
    assert!(filter_response.is_valid, "Question should be valid");

    // Step 2: Question Analyzer
    let analysis_result = suite.question_analyzer.analyze_question(&scenario.question).await;
    assert!(analysis_result.is_ok(), "Question should be analyzed successfully: {:?}", analysis_result.err());

    let analysis = analysis_result.unwrap();

    // Validate analysis structure
    assert!(!analysis.mood.is_empty(), "Mood should not be empty");
    assert!(!analysis.topic.is_empty(), "Topic should not be empty");
    assert!(!analysis.period.is_empty(), "Period should not be empty");

    // Validate expected values if provided
    if let Some(expected_mood) = &scenario.expected_mood {
        assert_eq!(analysis.mood, *expected_mood, "Mood should match expected value");
    }

    if let Some(expected_topic) = &scenario.expected_topic {
        assert_eq!(analysis.topic, *expected_topic, "Topic should match expected value");
    }

    // Step 3: Reading Agent
    let cards = create_test_cards(scenario.card_count);
    let reading_result = suite.reading_agent.generate_reading(
        &scenario.question,
        &analysis.mood,
        &analysis.topic,
        &analysis.period,
        &cards
    ).await;
    assert!(reading_result.is_ok(), "Reading should be generated successfully: {:?}", reading_result.err());

    let reading = reading_result.unwrap();

    // Validate reading structure
    assert!(!reading.header.is_empty(), "Header should not be empty");
    assert!(!reading.reading.is_empty(), "Main reading should not be empty");
    assert_eq!(reading.cards_reading.len(), scenario.card_count, "Number of card readings should match card count");
    assert!(!reading.suggestions.is_empty(), "Suggestions should not be empty");
    assert!(!reading.r#final.is_empty(), "Final message should not be empty");
    assert!(!reading.end.is_empty(), "End message should not be empty");

    // Validate Thai language consistency for Thai questions
    if scenario.thai_question {
        assert!(validate_thai_text_quality(&reading.header), "Header should be in Thai");
        assert!(validate_thai_text_quality(&reading.reading), "Main reading should be in Thai");
        assert!(reading.suggestions.iter().all(|s| validate_thai_text_quality(s)), "All suggestions should be in Thai");
        assert!(validate_thai_text_quality(&reading.r#final.iter().next().unwrap()), "Final message should be in Thai");
        assert!(validate_thai_text_quality(&reading.end), "End message should be in Thai");
    }
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_question_filter_to_analyzer_flow() {
    let suite = PromptSystemTestSuite::new().await.unwrap();

    let question = "ควรจะลงทุนอะไรดีครับ";

    // Filter question
    let filter_response = suite.question_filter.validate_question(question).await.unwrap();
    assert!(filter_response.is_valid, "Question should pass filter");

    // Analyze question
    let analysis = suite.question_analyzer.analyze_question(question).await.unwrap();

    // Validate data consistency
    assert!(!analysis.mood.is_empty(), "Analysis should have mood");
    assert!(!analysis.topic.is_empty(), "Analysis should have topic");
    assert!(!analysis.period.is_empty(), "Analysis should have period");

    // Validate Thai language consistency
    assert!(contains_thai_text(&analysis.mood), "Mood should be in Thai");
    assert!(contains_thai_text(&analysis.topic), "Topic should be in Thai");
    assert!(contains_thai_text(&analysis.period), "Period should be in Thai");
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_analyzer_to_reading_agent_flow() {
    let suite = PromptSystemTestSuite::new().await.unwrap();

    let question = "จะมีคนมาสาธิตฉันใช่ไหมคะ";

    // Analyze question first
    let analysis = suite.question_analyzer.analyze_question(question).await.unwrap();

    // Generate reading with analysis context
    let cards = create_test_cards(3);
    let reading = suite.reading_agent.generate_reading(
        question,
        &analysis.mood,
        &analysis.topic,
        &analysis.period,
        &cards
    ).await.unwrap();

    // Validate data consistency
    assert!(reading.header.contains(&analysis.topic) || reading.reading.contains(&analysis.topic),
           "Reading should reference analysis topic");
    assert!(reading.reading.contains(question) || reading.header.contains(question),
           "Reading should reference original question");

    // Validate structured format
    assert!(!reading.header.is_empty());
    assert!(!reading.reading.is_empty());
    assert_eq!(reading.cards_reading.len(), 3);
    assert!(!reading.suggestions.is_empty());
    assert!(!reading.r#final.is_empty());
    assert!(!reading.end.is_empty());
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_performance_benchmarks() {
    let suite = PromptSystemTestSuite::new().await.unwrap();
    let scenario = &get_test_scenarios()[0];

    // Benchmark individual agent performance
    let filter_start = std::time::Instant::now();
    let filter_result = suite.question_filter.validate_question(&scenario.question).await;
    let filter_duration = filter_start.elapsed();

    assert!(filter_result.is_ok(), "Filter should succeed");
    assert!(filter_duration < Duration::from_millis(3000),
           "Filter should complete within 3 seconds, took: {:?}", filter_duration);

    let analysis_start = std::time::Instant::now();
    let analysis_result = suite.question_analyzer.analyze_question(&scenario.question).await;
    let analysis_duration = analysis_start.elapsed();

    assert!(analysis_result.is_ok(), "Analysis should succeed");
    assert!(analysis_duration < Duration::from_millis(3000),
           "Analysis should complete within 3 seconds, took: {:?}", analysis_duration);

    let analysis = analysis_result.unwrap();
    let cards = create_test_cards(3);

    let reading_start = std::time::Instant::now();
    let reading_result = suite.reading_agent.generate_reading(
        &scenario.question,
        &analysis.mood,
        &analysis.topic,
        &analysis.period,
        &cards
    ).await;
    let reading_duration = reading_start.elapsed();

    assert!(reading_result.is_ok(), "Reading should succeed");
    assert!(reading_duration < Duration::from_millis(5000),
           "Reading should complete within 5 seconds, took: {:?}", reading_duration);

    // Complete pipeline should complete within 10 seconds
    let total_duration = filter_duration + analysis_duration + reading_duration;
    assert!(total_duration < Duration::from_millis(10000),
           "Complete pipeline should complete within 10 seconds, took: {:?}", total_duration);
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_thai_language_consistency() {
    let suite = PromptSystemTestSuite::new().await.unwrap();

    let thai_questions = vec![
        "จะมีคนมาสาธิตฉันใช่ไหมคะ",
        "การงานของฉันจะดีขึ้นไหม",
        "ควรจะซื้อบ้านที่ไหนดีครับ",
    ];

    for question in thai_questions {
        // Analyze Thai question
        let analysis = suite.question_analyzer.analyze_question(question).await.unwrap();

        // All analysis results should be in Thai
        assert!(contains_thai_text(&analysis.mood), "Mood should be in Thai for: {}", question);
        assert!(contains_thai_text(&analysis.topic), "Topic should be in Thai for: {}", question);
        assert!(contains_thai_text(&analysis.period), "Period should be in Thai for: {}", question);

        // Generate reading for Thai question
        let cards = create_test_cards(3);
        let reading = suite.reading_agent.generate_reading(
            question,
            &analysis.mood,
            &analysis.topic,
            &analysis.period,
            &cards
        ).await.unwrap();

        // All reading content should be in Thai
        assert!(validate_thai_text_quality(&reading.header), "Header should be in Thai for: {}", question);
        assert!(validate_thai_text_quality(&reading.reading), "Main reading should be in Thai for: {}", question);
        assert!(reading.suggestions.iter().all(|s| validate_thai_text_quality(s)),
               "All suggestions should be in Thai for: {}", question);
        assert!(validate_thai_text_quality(&reading.r#final.iter().next().unwrap()), "Final message should be in Thai for: {}", question);
        assert!(validate_thai_text_quality(&reading.end), "End message should be in Thai for: {}", question);
    }
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_json_response_validation() {
    let suite = PromptSystemTestSuite::new().await.unwrap();
    let question = "ควรจะลงทุนอะไรดีครับ";

    // Test QuestionFilter response
    let filter_response = suite.question_filter.validate_question(question).await.unwrap();
    assert!(filter_response.is_valid || !filter_response.reason.is_empty(),
           "Filter response should be valid or have reason");

    // Test QuestionAnalyzer response
    let analysis = suite.question_analyzer.analyze_question(question).await.unwrap();
    assert!(!analysis.mood.is_empty(), "Mood should not be empty");
    assert!(!analysis.topic.is_empty(), "Topic should not be empty");
    assert!(!analysis.period.is_empty(), "Period should not be empty");

    // Test ReadingAgent response
    let cards = create_test_cards(3);
    let reading = suite.reading_agent.generate_reading(
        question,
        &analysis.mood,
        &analysis.topic,
        &analysis.period,
        &cards
    ).await.unwrap();

    // Validate complete response structure
    assert!(!reading.header.is_empty(), "Header should not be empty");
    assert_eq!(reading.cards_reading.len(), 3, "Should have 3 card readings");
    assert!(!reading.reading.is_empty(), "Main reading should not be empty");
    assert!(!reading.suggestions.is_empty(), "Suggestions should not be empty");
    assert!(!reading.r#final.is_empty(), "Final message should not be empty");
    assert!(!reading.end.is_empty(), "End message should not be empty");

    // Validate each card reading has proper structure
    for card_reading in &reading.cards_reading {
        assert!(!card_reading.name.is_empty(), "Card name should not be empty");
        assert!(!card_reading.display_name.is_empty(), "Display name should not be empty");
        assert!(!card_reading.short_meaning.is_empty(), "Short meaning should not be empty");
    }
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_environment_variable_loading() {
    // This test ensures all prompts load correctly from environment variables
    let suite = PromptSystemTestSuite::new().await.unwrap();

    // If agents were created successfully, environment variables are loaded
    // This test validates the initialization process
    let question = "ทดสอบระบบ";

    // Test that all agents can be initialized and used
    let _filter_result = suite.question_filter.validate_question(question).await;
    let _analysis_result = suite.question_analyzer.analyze_question(question).await;

    // If we get here without panics, environment variables are loaded correctly
    assert!(true, "Environment variables loaded successfully");
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_error_handling_pipeline() {
    let suite = PromptSystemTestSuite::new().await.unwrap();

    // Test with empty question
    let empty_result = suite.question_filter.validate_question("").await;
    assert!(empty_result.is_err(), "Empty question should return error");

    // Test with very long question
    let long_question = "ค".repeat(1000);
    let long_result = suite.question_filter.validate_question(&long_question).await;
    assert!(long_result.is_err(), "Very long question should return error");

    // Test analyzer with empty question
    let empty_analysis = suite.question_analyzer.analyze_question("").await;
    assert!(empty_analysis.is_err(), "Empty question analysis should return error");

    // Test reading agent with no cards
    let no_cards_result = suite.reading_agent.generate_reading(
        "คำถาม",
        "อารมณ์",
        "หัวข้อ",
        "ช่วงเวลา",
        &[]
    ).await;
    assert!(no_cards_result.is_err(), "Reading with no cards should return error");
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_concurrent_agent_processing() {
    use futures::future::join_all;

    let suite = Arc::new(PromptSystemTestSuite::new().await.unwrap());
    let scenarios = get_test_scenarios();

    // Process multiple questions concurrently
    let mut handles = Vec::new();

    for scenario in scenarios.iter().take(5) {
        let suite_clone = Arc::clone(&suite);
        let scenario_clone = scenario.clone();

        let handle = tokio::spawn(async move {
            // Filter question
            let filter_result = suite_clone.question_filter.validate_question(&scenario_clone.question).await;

            if filter_result.is_ok() {
                // Analyze question
                let analysis_result = suite_clone.question_analyzer.analyze_question(&scenario_clone.question).await;

                if analysis_result.is_ok() {
                    let analysis = analysis_result.unwrap();
                    let cards = create_test_cards(scenario_clone.card_count);

                    // Generate reading
                    let reading_result = suite_clone.reading_agent.generate_reading(
                        &scenario_clone.question,
                        &analysis.mood,
                        &analysis.topic,
                        &analysis.period,
                        &cards
                    ).await;

                    return reading_result.is_ok();
                }
            }

            false
        });

        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    let results: Vec<bool> = join_all(handles).await
        .into_iter()
        .map(|result| result.unwrap_or(false))
        .collect();

    // At least 80% of concurrent operations should succeed
    let success_rate = results.iter().filter(|&&success| success).count() as f64 / results.len() as f64;
    assert!(success_rate >= 0.8, "Success rate should be at least 80%, was: {:.2}", success_rate * 100.0);
}

#[tokio::test]
#[ignore] // This test should fail initially (Red phase)
async fn test_real_gemini_api_end_to_end() {
    let suite = PromptSystemTestSuite::new().await.unwrap();

    // Use real Thai question for end-to-end testing
    let question = "ชีวิตของฉันจะดีขึ้นไหมในปีหน้า";

    // Complete pipeline with real API calls
    let filter_result = suite.question_filter.validate_question(question).await;
    assert!(filter_result.is_ok(), "Real API: Question filter should work");

    let analysis_result = suite.question_analyzer.analyze_question(question).await;
    assert!(analysis_result.is_ok(), "Real API: Question analyzer should work");

    let analysis = analysis_result.unwrap();
    let cards = create_test_cards(5); // Use 5 cards for comprehensive test

    let reading_result = suite.reading_agent.generate_reading(
        question,
        &analysis.mood,
        &analysis.topic,
        &analysis.period,
        &cards
    ).await;
    assert!(reading_result.is_ok(), "Real API: Reading agent should work");

    let reading = reading_result.unwrap();

    // Validate complete structured response from real API
    assert!(!reading.header.is_empty(), "Real API: Header should not be empty");
    assert!(!reading.reading.is_empty(), "Real API: Main reading should not be empty");
    assert_eq!(reading.cards_reading.len(), 5, "Real API: Should have 5 card readings");
    assert!(!reading.suggestions.is_empty(), "Real API: Suggestions should not be empty");
    assert!(!reading.r#final.is_empty(), "Real API: Final message should not be empty");
    assert!(!reading.end.is_empty(), "Real API: End message should not be empty");

    // Validate Thai language from real API
    assert!(validate_thai_text_quality(&reading.header), "Real API: Header should be in Thai");
    assert!(validate_thai_text_quality(&reading.reading), "Real API: Main reading should be in Thai");
}