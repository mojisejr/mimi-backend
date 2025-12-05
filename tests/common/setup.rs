//! Test setup utilities for end-to-end tests

use std::sync::Arc;
use reqwest::Client;
use sqlx::PgPool;
use redis::Client as RedisClient;
use mimivibe_backend::create_app;
use mimivibe_backend::queue::Queue;
use mimivibe_backend::queue::tarot_queue::TarotQueue;

pub struct TestApp {
    pub client: Client,
    pub db_pool: PgPool,
    pub redis_client: RedisClient,
    pub queue: Arc<dyn Queue<serde_json::Value>>,
}

pub async fn create_test_app() -> Result<TestApp, Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Create database pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/mimi_r".to_string());

    let db_pool = PgPool::connect(&database_url).await?;

    // Create Redis client (fallback to dummy if not available)
    let redis_url = std::env::var("REDIS_URL").ok();
    let redis_client = if let Some(url) = redis_url {
        RedisClient::open(url)?
    } else {
        // Create a dummy Redis client for testing
        RedisClient::open("redis://localhost:6379")?
    };

    // Create queue (uses InMemoryQueue if Redis not available)
    let queue = Arc::new(TarotQueue::from_env(db_pool.clone()).await?);

    // Create app instance
    let app = create_app(db_pool.clone(), redis_client.clone(), queue.clone()).await;

    // Get the server port
    let addr = format!("http://localhost:8080");

    // Create HTTP client
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    Ok(TestApp {
        client,
        db_pool,
        redis_client,
        queue,
    })
}