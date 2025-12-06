//! MimiVibe Backend - Worker Process
//!
//! Background worker for processing tarot readings asynchronously.
//! Handles long-running operations and queue processing with AI pipeline integration.
//!
//! Features:
//! - Prompt caching from database on startup
//! - Thread-safe Arc<HashMap> for prompt storage
//! - AI pipeline integration for tarot readings

use mimivibe_backend::models::job_types::JobStatus;
use mimivibe_backend::queue::TarotQueue;
use mimivibe_backend::repository::PromptRepository;
use mimivibe_backend::worker::TarotWorker;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create and run Tokio runtime manually
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())?;
    Ok(())
}

/// Main async function containing the worker logic
///
/// This function contains all the async operations that were previously
/// in the main function when using #[tokio::main]
async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
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
    let prompt_cache = load_prompts_cache(&db_pool).await;

    // Connect to TarotQueue
    info!("🔄 Connecting to tarot job queue...");
    let queue = TarotQueue::from_env().await?;
    info!("✅ Connected to tarot job queue");

    let worker_id =
        env::var("WORKER_ID").unwrap_or_else(|_| format!("worker-{}", uuid::Uuid::new_v4()));

    // Create worker instance with prompt cache (using database prompts)
    info!("Creating TarotWorker with ID: {} (using database prompts)", worker_id);
    let mut worker = TarotWorker::with_fallback(worker_id.clone(), prompt_cache).await?;

    // Display worker configuration
    info!("Worker configuration:");
    info!("  - Worker ID: {}", worker_id);
    info!("  - AI Model: gemini-pro");
    info!("  - Poll interval: 5 seconds");

    // Start continuous polling loop
    info!("🚀 Starting worker polling loop...");
    info!("⏰ Polling interval: 5 seconds");

    loop {
        match queue.poll_next_job().await {
            Ok(Some(job)) => {
                info!(
                    "📋 Found job: {} (question: \"{}\", cards: {})",
                    job.id, job.question, job.cards
                );

                // Note: Job is already marked as processing in poll_next_job()

                // Use the question and cards directly from the Job struct
                let question = job.question.as_str();
                let card_count = job.cards as u32;

                // Process through 3-agent pipeline
                match worker.process_reading_job(question, card_count).await {
                    Ok(result) => {
                        if let Err(e) = queue
                            .update_job_status(job.id, JobStatus::Succeeded, Some(result))
                            .await
                        {
                            info!("⚠️  Failed to update completed job status: {}", e);
                        } else {
                            info!("✅ Job {} completed", job.id);
                        }
                    }
                    Err(e) => {
                        if let Err(update_err) = queue
                            .update_job_status(job.id, JobStatus::Failed, None)
                            .await
                        {
                            info!("⚠️  Failed to update failed job status: {}", update_err);
                        } else {
                            info!("❌ Job {} failed: {}", job.id, e);
                        }
                    }
                }
            }
            Ok(None) => {
                info!("💤 No jobs available, waiting...");
            }
            Err(e) => {
                info!("⚠️  Error polling queue: {}", e);
            }
        }

        info!("⏳ Sleeping for 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
