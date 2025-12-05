//! MimiVibe Backend - Worker Process
//!
//! Background worker for processing tarot readings asynchronously.
//! Handles long-running operations and queue processing with AI pipeline integration.
//!
//! Features:
//! - Prompt caching from database on startup
//! - Thread-safe Arc<HashMap> for prompt storage
//! - AI pipeline integration for tarot readings

use mimivibe_backend::repository::PromptRepository;
use mimivibe_backend::worker::TarotWorker;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tracing::info;

/// Type alias for thread-safe prompt cache
pub type PromptCache = Arc<HashMap<String, String>>;

/// Load prompts cache from database
///
/// Queries the database for all active prompts and stores them in
/// an Arc<HashMap> for thread-safe concurrent access.
///
/// # Arguments
///
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
///
/// * `PromptCache` - Thread-safe prompt cache containing all active prompts
///
/// # Error Handling
///
/// Logs warning and returns empty cache on database failure
async fn load_prompts_cache(pool: &sqlx::PgPool) -> PromptCache {
    info!("🔄 Loading prompts cache from database...");

    match PromptRepository::load_all_active_prompts(pool).await {
        Ok(prompts) => {
            let prompt_count = prompts.len();
            let cache: HashMap<String, String> = prompts
                .into_iter()
                .map(|p| {
                    info!(
                        "   📝 Loaded prompt: {} (v{}, {} chars)",
                        p.agent_name,
                        p.version,
                        p.prompt_content.len()
                    );
                    (p.agent_name, p.prompt_content)
                })
                .collect();

            info!("✅ Prompts cache loaded with {} entries", prompt_count);

            // Verify expected prompts are loaded
            let expected = ["question_filter", "question_analyzer", "reading_agent"];
            for agent in expected {
                if !cache.contains_key(agent) {
                    info!("⚠️  Warning: Missing expected prompt for agent: {}", agent);
                }
            }

            Arc::new(cache)
        }
        Err(e) => {
            info!("⚠️  Failed to load prompts from database: {}", e);
            info!("⚠️  Continuing with empty prompt cache (will use .env fallback)");
            Arc::new(HashMap::new())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting MimiVibe Worker process...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database connection pool for prompt caching
    info!("🔄 Initializing database connection pool...");
    let database_url =
        env::var("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable not set")?;
    let db_pool = sqlx::PgPool::connect(&database_url).await?;
    info!("✅ Database connection pool initialized");

    // Load prompts cache from database
    let _prompt_cache = load_prompts_cache(&db_pool).await;

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
