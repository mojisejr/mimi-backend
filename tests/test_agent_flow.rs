#!/usr/bin/env rust-script
//! Test script to demonstrate agent flow output

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    println!("🔮 Testing MimiVibe Agent Flow");
    println!("================================");

    // Test question
    let question = "ควรจะลงทุนอะไรดีครับ";
    let card_count = 3;

    println!("\n📝 Input Question: {}", question);
    println!("🃏 Card Count: {}", card_count);

    // Create AIPipelineService
    println!("\n🔄 Initializing AIPipelineService...");
    let mut pipeline = mimivibe_backend::services::ai_pipeline::AIPipelineService::new().await?;
    println!("✅ AIPipelineService created successfully");

    // Process tarot reading
    println!("\n🚀 Starting agent flow processing...");
    println!("----------------------------------------");

    let result = pipeline.process_tarot_reading(question, card_count).await?;

    println!("----------------------------------------");
    println!("✅ Agent flow completed successfully!");

    // Display results
    println!("\n📊 Results Summary:");
    println!("==================");
    println!("🎯 Question: {}", result.question);
    println!("🃏 Cards: {:?}", result.cards);
    println!("📂 Category: {}", result.metadata.category);
    println!("💭 Intent: {}", result.metadata.intent);
    println!(
        "⏱️  Processing Time: {}ms",
        result.metadata.processing_time_ms
    );
    println!("🤖 AI Model: {}", result.metadata.model);

    println!("\n📖 Reading Preview (first 200 chars):");
    println!("{}", &result.reading[..result.reading.len().min(200)]);
    if result.reading.len() > 200 {
        println!("...");
    }

    println!("\n🎉 Agent flow test completed successfully!");

    Ok(())
}
