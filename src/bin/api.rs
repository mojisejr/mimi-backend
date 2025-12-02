//! MimiVibe Backend - Axum API Server
//!
//! REST API server for tarot reading requests and responses.
//! Built with Axum web framework.

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
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

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
    let redis_client = if config.has_upstash() {
        redis::Client::open(format!(
            "redis://:{}@{}",
            config.upstash_token()?,
            config.upstash_url()?
        ))?
    } else {
        redis::Client::open(config.redis_url()?)?
    };
    println!("✅ Redis client initialized for rate limiting and caching");

    // Initialize TarotQueue (which handles both Redis queue and database)
    let tarot_queue = Arc::new(TarotQueue::from_env().await?);
    println!("✅ TarotQueue system initialized (Redis + Database integration)");

    // Create API state
    let api_state = ApiState {
        redis_client: redis_client.clone(),
        tarot_queue,
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
