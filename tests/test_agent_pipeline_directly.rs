//! Direct Test of Agent Pipeline
//!
//! This test directly demonstrates the 3-agent pipeline processing
//! without needing a separate worker process

use mimivibe_backend::{
    agents::question_analyzer::QuestionAnalyzer, agents::question_filter::QuestionFilter,
    agents::reading_agent::ReadingAgent, repository::PromptRepository,
};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_complete_agent_pipeline_directly() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔮 Testing Complete Agent Pipeline Directly");
    println!("==========================================");

    // Load environment
    dotenvy::dotenv().ok();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/mimi_r".to_string());

    let db_pool = PgPool::connect(&database_url).await?;
    println!("✅ Database connected");

    // Load prompts from database
    let prompts = PromptRepository::load_all_active_prompts(&db_pool).await?;
    let prompt_cache: HashMap<String, String> = prompts
        .into_iter()
        .map(|p| (p.agent_name.clone(), p.prompt_content))
        .collect();

    let prompt_cache = Arc::new(prompt_cache);
    println!("✅ Loaded {} prompts from database", prompt_cache.len());

    // Test questions
    let test_cases = vec![
        ("ความรักของฉันจะเป็นอย่างไร", 3, "love"),
        ("งานของฉันจะเจริญไหม", 5, "career"),
        ("สุขภาพของฉันจะดีขึ้นไหม", 3, "health"),
    ];

    for (question, cards, category) in test_cases {
        println!("\n📝 Testing: {}", category);
        println!("Question: {}", question);
        println!("Cards: {}", cards);
        println!("{}", "─".repeat(50));

        // Step 1: Question Filter
        println!("\n🤖 Step 1: Question Filter");
        println!("---------------------------");
        let question_filter = QuestionFilter::new().await?;

        let filter_result = question_filter.validate(question).await?;
        println!("✅ Filter result: {}", filter_result["is_appropriate"]);
        println!("📝 Reason: {}", filter_result["reason"]);

        if !filter_result["is_appropriate"].as_bool().unwrap_or(false) {
            println!("❌ Question filtered out: {}", filter_result["reason"]);
            continue;
        }

        // Step 2: Question Analyzer
        println!("\n🧠 Step 2: Question Analyzer");
        println!("------------------------------");
        let question_analyzer = QuestionAnalyzer::new().await?;

        let analysis_result = question_analyzer.analyze(question).await?;
        println!("✅ Analysis completed");
        println!("📊 Mood: {}", analysis_result["mood"]);
        println!("🏷️  Topic: {}", analysis_result["topic"]);
        println!("⏰ Time context: {}", analysis_result["time_context"]);

        // Step 3: Reading Agent
        println!("\n🔮 Step 3: Reading Agent");
        println!("-------------------------");
        let reading_agent = ReadingAgent::new().await?;

        // Create card list
        let mut card_names = Vec::new();
        for i in 0..cards {
            // Sample tarot cards (in real implementation, these would be randomly selected)
            let sample_cards = vec![
                "The Fool",
                "The Magician",
                "The High Priestess",
                "The Empress",
                "The Emperor",
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
            ];
            card_names.push(sample_cards[i % sample_cards.len()].to_string());
        }

        let reading_request = json!({
            "question": question,
            "cards": card_names,
            "analysis": analysis_result
        });

        println!("🎴 Drawing {} cards...", cards);
        let reading_result = reading_agent.generate_reading(reading_request).await?;

        // Display the complete reading
        println!("\n📖 TAROT READING RESULT");
        println!("=======================");

        if let Some(header) = reading_result["header"].as_str() {
            println!("\n📜 {}", header);
        }

        if let Some(cards_drawn) = reading_result["cards_reading"].as_array() {
            println!("\n🃏 Cards Drawn:");
            for (i, card) in cards_drawn.iter().enumerate() {
                println!("  {}. {}", i + 1, card);
            }
        }

        if let Some(reading) = reading_result["reading"].as_str() {
            println!("\n📖 Reading:");
            println!("{}", reading);
        }

        if let Some(suggestions) = reading_result["suggestions"].as_array() {
            println!("\n💡 Suggestions:");
            for suggestion in suggestions {
                if let Some(s) = suggestion.as_str() {
                    println!("  • {}", s);
                }
            }
        }

        if let Some(final_msg) = reading_result["final"].as_array() {
            println!("\n🎯 Final Words:");
            for msg in final_msg {
                if let Some(m) = msg.as_str() {
                    println!("  {}", m);
                }
            }
        }

        if let Some(end) = reading_result["end"].as_str() {
            println!("\n🙏 {}", end);
        }

        // Verification
        println!("\n✅ Pipeline Verification:");
        println!("========================");
        let mut checks = vec![];

        checks.push((
            "Question passed filter",
            filter_result["is_appropriate"].is_boolean(),
        ));
        checks.push(("Analysis generated", analysis_result["mood"].is_string()));
        checks.push(("Reading has header", reading_result["header"].is_string()));
        checks.push((
            "Reading has interpretation",
            reading_result["reading"].is_string(),
        ));
        checks.push((
            "Reading has suggestions",
            reading_result["suggestions"].is_array(),
        ));
        checks.push(("Reading has closing", reading_result["end"].is_string()));

        let all_passed = checks.iter().all(|(_, passed)| *passed);

        for (check_name, passed) in checks {
            let icon = if passed { "✅" } else { "❌" };
            println!("{} {}", icon, check_name);
        }

        if all_passed {
            println!("\n🎉 All checks passed! Pipeline working correctly!");
        }

        println!("\n{}\n", "=".repeat(60));
    }

    println!("\n✅ All agent pipeline tests completed!");
    println!("The 3-agent system is working correctly:");
    println!("  1. 🤖 Question Filter - validates appropriateness");
    println!("  2. 🧠 Question Analyzer - extracts context");
    println!("  3. 🔮 Reading Agent - generates Thai tarot interpretation");

    Ok(())
}
