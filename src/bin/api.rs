//! MimiVibe Backend - Axum API Server
//!
//! REST API server for tarot reading requests and responses.
//! Built with Axum web framework.
//!
//! Features:
//! - Prompt caching from database on startup
//! - Thread-safe Arc<HashMap> for prompt storage
//! - Redis rate limiting and caching
//! - TarotQueue with database persistence

use axum::{
    http::{HeaderValue, Method},
    middleware,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use mimivibe_backend::{
    api::tarot::{get_reading, health_check, request_reading, ApiState},
    config::env::EnvironmentConfig,
    middleware::rate_limiter::{rate_limit_middleware, RateLimiterConfig, RateLimiterState},
    queue::tarot_queue::TarotQueue,
    repository::PromptRepository,
};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Type alias for thread-safe prompt cache
pub type PromptCache = Arc<HashMap<String, String>>;

/// Initialize prompt cache from database
///
/// Loads all active prompts from the `prompts` table and stores them in
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
/// # Panics
///
/// This function will log a warning and return an empty cache if:
/// - Database connection fails
/// - No active prompts are found
async fn initialize_prompt_cache(pool: &sqlx::PgPool) -> PromptCache {
    println!("🔄 Initializing prompt cache from database...");

    match PromptRepository::load_all_active_prompts(pool).await {
        Ok(prompts) => {
            let prompt_count = prompts.len();
            let cache: HashMap<String, String> = prompts
                .into_iter()
                .map(|p| {
                    println!(
                        "   📝 Loaded prompt: {} (v{}, {} chars)",
                        p.agent_name,
                        p.version,
                        p.prompt_content.len()
                    );
                    (p.agent_name, p.prompt_content)
                })
                .collect();

            println!("✅ Prompt cache initialized with {} entries", prompt_count);

            // Verify expected prompts are loaded
            let expected = ["question_filter", "question_analyzer", "reading_agent"];
            for agent in expected {
                if !cache.contains_key(agent) {
                    println!("⚠️  Warning: Missing expected prompt for agent: {}", agent);
                }
            }

            Arc::new(cache)
        }
        Err(e) => {
            println!("⚠️  Failed to load prompts from database: {}", e);
            println!("⚠️  Continuing with empty prompt cache (will use .env fallback)");
            Arc::new(HashMap::new())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting MimiVibe API server...");

    // Load configuration
    let config = EnvironmentConfig::from_env()?;
    println!("✅ Environment configuration loaded");

    // Initialize Redis client for rate limiting and caching
    println!("🔄 Initializing Redis client...");
    let redis_client = match if config.has_upstash() {
        // Use local Redis for rate limiting when Upstash is configured for queue
        println!("Using local Redis for rate limiting (Upstash configured for queue)");
        redis::Client::open(config.redis_url()?)
    } else {
        redis::Client::open(config.redis_url()?)
    } {
        Ok(client) => {
            println!("✅ Redis client initialized for rate limiting and caching");
            client
        }
        Err(e) => {
            println!("❌ Redis client initialization failed: {}", e);
            // Continue without rate limiting in development
            println!("⚠️ Continuing without Redis rate limiting (development mode)");
            redis::Client::open("redis://127.0.0.1:6379")?
        }
    };

    // Initialize database connection pool for prompt caching
    println!("🔄 Initializing database connection pool...");
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable not set")?;
    let db_pool = sqlx::PgPool::connect(&database_url).await?;
    println!("✅ Database connection pool initialized");

    // Initialize prompt cache from database
    let prompt_cache = initialize_prompt_cache(&db_pool).await;

    // Initialize TarotQueue (which handles both Redis queue and database)
    println!("🔄 Initializing TarotQueue system...");
    let tarot_queue = match TarotQueue::from_env().await {
        Ok(queue) => {
            println!("✅ TarotQueue system initialized (Redis + Database integration)");
            Arc::new(queue)
        }
        Err(e) => {
            println!("❌ TarotQueue initialization failed: {}", e);
            return Err(e);
        }
    };

    // Create API state
    let api_state = ApiState {
        redis_client: redis_client.clone(),
        tarot_queue,
        prompt_cache,
    };

    // Create rate limiter state
    let rate_limiter_state = RateLimiterState::new(
        redis_client,
        RateLimiterConfig {
            key_prefix: "tarot_api".to_string(),
            window_seconds: 600, // 10 minutes
            max_requests: 1,     // 1 request per 10 minutes
        },
    );

    // Build our application with a route
    let app = Router::new()
        // API routes
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/tarots/read", post(request_reading))
        .route("/api/v1/tarots/:job_id", get(get_reading))
        // Add rate limiting middleware to tarot routes
        .layer(middleware::from_fn_with_state(
            rate_limiter_state.clone(),
            rate_limit_middleware,
        ))
        // Add CORS middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ])
                .allow_credentials(false),
        )
        // Add request ID middleware for tracing
        .layer(middleware::from_fn(request_id_middleware))
        // Provide shared state
        .with_state(api_state);

    // Get server port from environment or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let bind_addr = format!("0.0.0.0:{}", port);

    println!("🌐 MimiVibe API server starting on {}", bind_addr);
    println!("📋 Available endpoints:");
    println!("   GET  /api/v1/health - Health check");
    println!("   POST /api/v1/tarots/read - Request tarot reading");
    println!("   GET  /api/v1/tarots/:job_id - Get reading result");
    println!("🔒 Rate limiting: 1 request per 10 minutes per user/IP");
    println!("🌐 CORS: Enabled for all origins (development mode)");

    // Start the server
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Middleware to add unique request ID for tracing
async fn request_id_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let request_id = uuid::Uuid::new_v4().to_string();

    // Add request ID to response headers
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());

    response
}
