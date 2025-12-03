//! MimiVibe Backend - Worker Process
//!
//! Background worker for processing tarot readings asynchronously.
//! Handles long-running operations and queue processing with AI pipeline integration.

use mimivibe_backend::worker::TarotWorker;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting MimiVibe Worker process...");

    // Load environment variables
    dotenvy::dotenv().ok();

    let worker_id =
        env::var("WORKER_ID").unwrap_or_else(|_| format!("worker-{}", uuid::Uuid::new_v4()));

    // Create worker instance
    info!("Creating TarotWorker with ID: {}", worker_id);
    let mut worker = TarotWorker::new(worker_id.clone())?;

    // Display worker configuration
    info!("Worker configuration:");
    info!("  - Worker ID: {}", worker_id);
    info!("  - AI Model: gemini-pro");
    info!("  - Poll interval: 5 seconds");

    // Demonstrate worker functionality
    info!("🚀 Demonstrating AI pipeline functionality...");

    // Example job processing
    let question = "ควรจะลงทุนอะไรดีครับ";
    let card_count = 3;

    match worker.process_reading_job(question, card_count).await {
        Ok(result) => {
            info!("✅ Successfully processed example job");
            info!("  Question: {}", result["question"]);
            info!("  Cards: {}", result["cards"]);
            info!(
                "  Processing time: {}ms",
                result["metadata"]["processing_time_ms"]
            );
        }
        Err(e) => {
            info!("⚠️  Example job processing failed: {}", e);
        }
    }

    info!("Worker demonstration completed");
    Ok(())
}
